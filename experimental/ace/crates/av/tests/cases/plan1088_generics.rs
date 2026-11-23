use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{
    execute_observed, execute_sched, execute_with_arena, CancelFlag, ExecLimits, ExecTrace,
};
use ace_av_core::contract::{
    ExecOutcome, ResourceKind, Value, AV_DYNAMIC_PROFILE, MIR_DYNAMIC_SCHEMA,
    SHARED_COLLECTION_ARGS_CAPABILITY, TEXT_CASE_CAPABILITY,
};
use ace_av_core::host::{ConsoleHost, HostError, Stream};
use ace_av_core::mir::{MirInstr, MirModule};
use ace_av_core::owned_text::StrArena;
use ace_core::source::adapter::compile_language_source_with;

fn lower(source: &str) -> MirModule {
    let caps = ["ac.source.language.av/v1".into(), "console.stdout".into()];
    let path = "generics.at";
    let checked = compile_language_source_with(source, path, &caps, |program| {
        ace_source_query::instances::instantiate_auto_program(program, path, &caps)
    })
    .expect("compile ordinary source");
    ace_lowering::lower_checked_auto_language(&checked.checked, "d_main", 0).expect("lower to MIR")
}

#[derive(Default)]
struct GenericOwnerConsole {
    cancel_on_write: bool,
    writes: usize,
}

impl ConsoleHost for GenericOwnerConsole {
    fn retained_bytes(&self) -> usize {
        0
    }

    fn write(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        assert_eq!(stream, Stream::Stdout);
        assert_eq!(bytes, b"live\n");
        self.writes += 1;
        if self.cancel_on_write {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            return Err(HostError::Cancelled);
        }
        Ok(())
    }
}

#[test]
fn cancelled_callback_releases_the_generic_owner_and_replays() {
    let module = load(&lower(
        r#"
fn keep<T>(move value T) T { return value.move }
fn main() int {
    let held = keep("live".clone())
    print(held)
    return 38 + held.len()
}
"#,
    ));
    let mut host = GenericOwnerConsole {
        cancel_on_write: true,
        ..Default::default()
    };
    let limits = ExecLimits::default();
    let (outcome, trace) = execute_sched(&module, &limits, &CancelFlag::new(false), &mut host);
    assert!(matches!(outcome, ExecOutcome::Cancelled));
    assert_eq!(host.writes, 1);
    let cancelled_allocations = assert_released(trace);
    assert!(
        cancelled_allocations >= 1,
        "the provider saw a live generic owner"
    );

    let mut host = GenericOwnerConsole::default();
    let (outcome, trace) = execute_sched(&module, &limits, &CancelFlag::new(false), &mut host);
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(42),
            ..
        }
    ));
    assert_eq!(host.writes, 1);
    assert_eq!(assert_released(trace), cancelled_allocations);
}

fn load(mir: &MirModule) -> LoadedModule {
    ace_av_core::verify_mir(mir).expect("verify MIR");
    let abc = encode_abc(mir).expect("encode ABC");
    verify_abc_bytes(&abc.to_flat(), &mir.owner).expect("admit ABC")
}

fn assert_released(trace: ExecTrace) -> usize {
    assert_eq!(trace.frames_remaining, 0);
    let arena = trace.arena.expect("text and list arena");
    let mut allocations: Vec<_> = arena
        .alloc_events()
        .into_iter()
        .map(|event| event.id)
        .collect();
    let mut releases: Vec<_> = arena
        .free_events()
        .into_iter()
        .map(|event| event.id)
        .collect();
    allocations.sort_unstable();
    releases.sort_unstable();
    assert_eq!(allocations, releases);
    assert_eq!(arena.live(), 0);
    allocations.len()
}

#[test]
fn lowercase_roundtrips_the_wire_and_preserves_contextual_unicode() {
    let source = r#"
fn main() int {
    let title = "xİSTANBUL ΟΣ ΣΟ Жz".clone()
    let folded = title.slice(1, title.len() - 1).to_lower()
    let other = "ΟΣ".lower()
    if folded == "i̇stanbul ος σο ж" && other == "ος" && title == "xİSTANBUL ΟΣ ΣΟ Жz" {
        return 1
    }
    return 0
}
"#;
    let mut mir = lower(source);
    assert!(mir
        .funcs
        .iter()
        .flat_map(|function| &function.blocks)
        .flat_map(|block| &block.instrs)
        .any(|instr| matches!(instr, MirInstr::StrLower { .. })));
    for shared_widget_profile in [false, true] {
        if shared_widget_profile {
            mir.profile = AV_DYNAMIC_PROFILE.into();
            mir.owner.schema = MIR_DYNAMIC_SCHEMA.into();
            mir.capabilities.push(SHARED_COLLECTION_ARGS_CAPABILITY.into());
            for function in &mut mir.funcs {
                function.owner.schema.clone_from(&mir.owner.schema);
            }
        }
        mir.capabilities.retain(|cap| cap != TEXT_CASE_CAPABILITY);
        let error = ace_av_core::verify_mir(&mir).expect_err("lowercase requires explicit admission");
        assert_eq!(error.code(), "mir.profile-instr");
        mir.capabilities.push(TEXT_CASE_CAPABILITY.into());
        let module = load(&mir);
        let (outcome, trace) =
            execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
        assert!(
            matches!(outcome, ExecOutcome::Success { value: Value::I32(1), .. }),
            "{outcome:?}"
        );
        assert!(assert_released(trace) >= 3);
    }
}

#[test]
fn uppercase_checks_admission_and_fuel_with_owned_generic_live() {
    let mut mir = lower(&format!(
        r#"
fn keep<T>(move value T) T {{ return value.move }}
fn main() int {{
    let held = keep("fresh".clone())
    let converted = "{}".to_upper()
    return held.len() + converted.len()
}}
"#,
        "ß".repeat(1_024)
    ));
    assert!(mir
        .funcs
        .iter()
        .flat_map(|f| &f.blocks)
        .flat_map(|b| &b.instrs)
        .any(|instr| matches!(instr, MirInstr::StrUpper { .. })));
    mir.capabilities.retain(|cap| cap != TEXT_CASE_CAPABILITY);
    let error = ace_av_core::verify_mir(&mir).expect_err("uppercase requires explicit admission");
    assert_eq!(error.code(), "mir.profile-instr");
    mir.capabilities.push(TEXT_CASE_CAPABILITY.into());
    let module = load(&mir);
    let limited = ExecLimits {
        max_steps: 2_500,
        ..ExecLimits::default()
    };
    let cancel = CancelFlag::new(false);
    let (outcome, trace) = execute_observed(&module, &limited, &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Resource {
            kind: ResourceKind::InstrSteps,
            ..
        }
    ));
    assert!(assert_released(trace) >= 1, "a generic owner was live");
    let (outcome, trace) = execute_observed(&module, &ExecLimits::default(), &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(2053),
            ..
        }
    ));
    assert!(assert_released(trace) >= 2);
}

#[test]
fn typed_filter_allocation_fault_releases_owners_and_allows_replay() {
    let module = load(&lower(
        r#"
type Note { id int; title str }
fn main() int {
    let notes = List<Note>.new([Note(0,"x"), Note(19,"y"), Note(23,"z")])
    let selected = notes.filter(fn(n Note) bool { return n.id > 0 })
    return selected[0].id + selected[1].id
}
"#,
    ));
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    let (outcome, trace) = execute_observed(&module, &limits, &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(42),
            ..
        }
    ));
    let allocations = assert_released(trace);
    assert!(
        allocations >= 8,
        "source, snapshot, and result owners were allocated"
    );

    let arena = StrArena::new();
    arena.set_fault_at(allocations as u64);
    let (outcome, trace) = execute_with_arena(&module, &limits, &cancel, arena);
    assert!(matches!(outcome, ExecOutcome::Infra(ref reason) if reason.contains("fault-injected")));
    assert_eq!(assert_released(trace), allocations - 1);

    let (outcome, trace) = execute_observed(&module, &limits, &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(42),
            ..
        }
    ));
    assert_eq!(assert_released(trace), allocations);
}
