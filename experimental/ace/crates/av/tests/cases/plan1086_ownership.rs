use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{execute_observed, execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{ExecOutcome, ResourceKind, Value};
use ace_av_core::host::{ConsoleHost, HostError, Stream};
use ace_core::source::compile_language_source;
use ace_lowering::lower_checked_auto_language;
use std::sync::atomic::Ordering;

fn load(source: &str) -> LoadedModule {
    load_with_caps(source, &[])
}

fn load_with_caps(source: &str, caps: &[String]) -> LoadedModule {
    let checked = compile_language_source(source, "ownership.at", caps).expect("check source");
    let mir = lower_checked_auto_language(&checked.checked, "d_main", 0).expect("lower source");
    ace_av_core::verify_mir(&mir).expect("verify MIR");
    let abc = encode_abc(&mir).expect("encode ABC");
    verify_abc_bytes(&abc.to_flat(), &mir.owner).expect("admit ABC")
}

fn assert_released(trace: ExecTrace, minimum_allocations: usize) {
    assert_eq!(trace.frames_remaining, 0);
    let arena = trace.arena.expect("owned arena");
    let mut allocs: Vec<_> = arena.alloc_events().iter().map(|event| event.id).collect();
    let mut frees: Vec<_> = arena.free_events().iter().map(|event| event.id).collect();
    assert!(
        allocs.len() >= minimum_allocations,
        "expected actual owned buffers"
    );
    allocs.sort_unstable();
    frees.sort_unstable();
    assert_eq!(allocs, frees);
    assert_eq!(arena.live(), 0);
}

#[test]
fn owned_temporaries_release_on_continue_break_and_return() {
    let module = load(
        r#"
fn run() int {
    var n = 0
    loop {
        let text = "x".clone() + "y"
        n = n + text.len()
        if n == 2 { continue }
        if n == 4 { break }
    }
    return 38 + n
}
fn main() int { return run() }
"#,
    );
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(42),
            ..
        }
    ));
    assert_released(trace, 4);
}

#[test]
fn fuel_refusal_releases_live_owner() {
    let module = load(
        r#"
fn main() int {
    let text = "live bytes".clone()
    var n = 0
    loop { n = n + 1; if n == 4000 { break } }
    return text.len()
}
"#,
    );
    let limit = ExecLimits {
        max_steps: 128,
        ..Default::default()
    };
    let (outcome, trace) = execute_observed(&module, &limit, &CancelFlag::new(false));
    assert!(matches!(
        outcome,
        ExecOutcome::Resource {
            kind: ResourceKind::InstrSteps,
            ..
        }
    ));
    assert_released(trace, 1);

    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(10),
            ..
        }
    ));
    assert_released(trace, 1);
}

#[test]
fn av_text_scan_admission_and_byte_fuel() {
    use ace_av_core::{contract::TEXT_SCAN_CAPABILITY, mir::MirInstr};

    // Static input exercises scan work without charging for an unrelated
    // owned literal allocation first.
    let source = format!(
        "fn main() int {{ return __ace_text_find(\"{}\", \"z\", 0) }}",
        "a".repeat(4_096)
    );
    let compiled = compile_language_source(&source, "scan.at", &[]).expect("compile text scan");
    let mut mir = lower_checked_auto_language(&compiled.checked, "d_main", 0).expect("lower scan");
    assert!(mir
        .funcs
        .iter()
        .flat_map(|f| &f.blocks)
        .flat_map(|b| &b.instrs)
        .any(|instr| matches!(instr, MirInstr::StrFind { .. })));
    mir.capabilities.retain(|cap| cap != TEXT_SCAN_CAPABILITY);
    let result = ace_av_core::verify_mir(&mir);
    assert!(matches!(result, Err(error) if error.code() == "mir.profile-instr"));
    mir.capabilities.push(TEXT_SCAN_CAPABILITY.into());
    let abc = encode_abc(&mir).expect("encode checked scan");
    let module = verify_abc_bytes(&abc.to_flat(), &mir.owner).expect("admit broadcast capability");
    let cancel = CancelFlag::new(false);
    let limited = ExecLimits {
        max_steps: 128,
        ..Default::default()
    };
    let (outcome, _) = execute_observed(&module, &limited, &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Resource {
            kind: ResourceKind::InstrSteps,
            ..
        }
    ));
    let (outcome, _) = execute_observed(&module, &ExecLimits::default(), &cancel);
    assert!(
        matches!(outcome, ExecOutcome::Success { value: Value::I32(-1), steps } if steps > 4_096)
    );
}

#[test]
fn caught_failure_releases_owned_locals_and_continues() {
    let module = load(
        r#"
fn main() int {
    var n = 0
    try {
        let text = "root".clone()
        n = text.len()
        fail("caught")
    } catch message {
        n = n + message.len()
    }
    return 32 + n
}
"#,
    );
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(42),
            ..
        }
    ));
    assert_released(trace, 2);
}

#[test]
fn uncaught_failure_releases_owned_locals() {
    let module = load(
        r#"
fn main() int {
    let text = "root".clone()
    if text.len() == 4 { fail("boom") }
    return 0
}
"#,
    );
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(outcome, ExecOutcome::Error { ref message, .. } if message == "boom"));
    assert_released(trace, 1);
}

struct CancelOnWrite {
    calls: u32,
}

impl ConsoleHost for CancelOnWrite {
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
        assert_eq!(bytes, b"root\n");
        self.calls += 1;
        cancel.store(true, Ordering::Relaxed);
        Err(HostError::Cancelled)
    }
}

#[test]
fn live_owner_released_on_provider_cancellation() {
    let module = load_with_caps(
        r#"
fn main() int {
    let text = "root".clone()
    print(text)
    return 38 + text.len()
}
"#,
        &["console.stdout".to_owned()],
    );
    let mut host = CancelOnWrite { calls: 0 };
    let (outcome, trace) = execute_sched(
        &module,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
    assert!(matches!(outcome, ExecOutcome::Cancelled));
    assert_eq!(host.calls, 1);
    assert_released(trace, 1);
}

#[test]
fn explicit_close_releases_suspended_owned_text() {
    let module = load(
        r#"
fn guarded() ~Iter<int> {
    try {
        let text = "root".clone()
        yield text.len()
        yield text.len() + 1
    } finally {
        fail("explicit close")
    }
}
fn main() int {
    var iterator = guarded()
    let first = iterator.next() ?? -1
    if first == 4 { iterator.close() }
    return first
}
"#,
    );
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(outcome, ExecOutcome::Error { ref message, .. } if message == "explicit close")
    );
    assert_released(trace, 2);
}

// PLAN-1089 uses the same authored AutoDown input for the AV check
// and the in-process lifecycle observer.
fn generator_source(case: &str) -> &'static str {
    let document = include_str!(
        "../../../../../../crates/auto-lang/test/cases/language/generators/yield-close.case.ad"
    );
    let heading = format!("\n## Case: {case}\n");
    let (_, section) = document.split_once(&heading).expect("generator case");
    let (_, source) = section.split_once("\n```auto\n").expect("Auto source");
    source.split_once("\n```").expect("source fence").0
}

#[test]
fn plan1089_releases_owned_variants_and_lists_after_two_items() {
    use ace_av_core::lifecycle::{Act, OwnVia, OwnerKind};

    let module = load(generator_source("payload-owner-early-stop"));
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            outcome,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "{outcome:?}"
    );
    let events = trace.lifecycle.events();
    for kind in [OwnerKind::Str, OwnerKind::List, OwnerKind::Variant] {
        let created = events
            .iter()
            .filter(|event| {
                event.act == Act::Own
                    && event.kind == Some(kind)
                    && event.via_owned == Some(OwnVia::Created)
            })
            .count();
        assert!(
            created >= 2,
            "two independently created {kind:?} owners: {created}"
        );
    }
    assert!(events.iter().any(|event| event.act == Act::End));
    assert_released(trace, 2);
}

#[test]
fn plan1089_provider_cancel_releases_a_suspended_iterator() {
    #[derive(Default)]
    struct CancelAfterSecondItem {
        writes: Vec<Vec<u8>>,
    }

    impl ConsoleHost for CancelAfterSecondItem {
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
            self.writes.push(bytes.to_vec());
            if self.writes.len() == 2 {
                cancel.store(true, Ordering::Relaxed);
                return Err(HostError::Cancelled);
            }
            Ok(())
        }
    }

    let module = load_with_caps(
        generator_source("cancel-during-suspension"),
        &["console.stdout".into()],
    );
    let mut host = CancelAfterSecondItem::default();
    let (outcome, trace) = execute_sched(
        &module,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
    assert!(matches!(outcome, ExecOutcome::Cancelled), "{outcome:?}");
    assert_eq!(
        host.writes,
        [b"twenty\n".to_vec(), b"twenty-two\n".to_vec()]
    );
    assert_eq!(trace.handled_language_errors, 0);
    assert_released(trace, 2);
}

#[test]
fn plan1089_caught_close_error_releases_the_displaced_iterator() {
    let module = load(generator_source("close-throws-then-terminal"));
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            outcome,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "{outcome:?}"
    );
    // The close driver receives the cleanup error, then the source catch
    // receives the rethrow. The second close does not reenter a handler.
    assert_eq!(trace.handled_language_errors, 2);
    assert_released(trace, 2);
}
