use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{execute_observed, execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{ExecOutcome, LanguageErrorKind, ResourceKind, Value};
use ace_av_core::host::{ConsoleHost, HostError, Stream, STDOUT};
use ace_av_core::mir::{MirInstr, MirModule, Terminator};

const STACK: &str = r#"
fn leaf(n int) int {
    let text = "leaf".clone()
    if n == 0 { fail(text) }
    return n + text.len()
}
fn middle(n int) int {
    let text = "mid".clone()
    return leaf(n) + text.len()
}
fn outer(n int) int {
    let text = "outer".clone()
    return middle(n) + text.len()
}
fn main() int {
    try { return outer(0) }
    catch e {
        if e == "leaf" { return outer(30) }
        return 0
    }
}
"#;

fn lower(source: &str) -> MirModule {
    let checked = ace_core::source::compile_language_source(source, "errors.at", &[STDOUT.into()])
        .expect("check ordinary Auto source");
    ace_lowering::lower_checked_auto_language(&checked.checked, "d_main", 0)
        .expect("lower checked source")
}

fn load(mir: &MirModule) -> LoadedModule {
    ace_av_core::verify_mir(mir).expect("verify MIR");
    let abc = encode_abc(mir).expect("encode ABC");
    verify_abc_bytes(&abc.to_flat(), &mir.owner).expect("admit ABC")
}

fn assert_released(trace: &ExecTrace, minimum_allocations: usize) {
    assert_eq!(trace.frames_remaining, 0);
    let arena = trace.arena.as_ref().expect("owned arena");
    let mut allocations: Vec<_> = arena.alloc_events().iter().map(|event| event.id).collect();
    let mut releases: Vec<_> = arena.free_events().iter().map(|event| event.id).collect();
    assert!(allocations.len() >= minimum_allocations);
    allocations.sort_unstable();
    releases.sort_unstable();
    assert_eq!(allocations, releases);
    assert_eq!(arena.live(), 0);
}

#[test]
fn unwind_releases_three_activations_and_allows_a_second_call() {
    let module = load(&lower(STACK));
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
    assert_eq!(trace.handled_language_errors, 1);
    assert_eq!(trace.call_frame_commits, 6);
    let error = trace
        .last_language_error
        .expect("handled error observation");
    assert_eq!(error.kind, LanguageErrorKind::Explicit);
    assert!(error.span.is_some());
    assert_released(&trace, 6);
}

#[test]
fn frame_budget_exit_does_not_enter_language_handler() {
    let module = load(&lower(STACK));
    let limits = ExecLimits {
        max_frames: 3,
        ..Default::default()
    };
    let (outcome, trace) = execute_observed(&module, &limits, &CancelFlag::new(false));
    assert!(
        matches!(
            outcome,
            ExecOutcome::Resource {
                kind: ResourceKind::Frames,
                ..
            }
        ),
        "{outcome:?}"
    );
    assert_eq!(trace.handled_language_errors, 0);
    assert_eq!(trace.call_frame_commits, 2);
    assert_released(&trace, 2);
}

#[test]
fn rethrow_keeps_original_index_error_span() {
    let source = r#"
fn leaf() int { let xs = [1]; return xs[9] }
fn main() int { try { return leaf() } catch { rethrow() } }
"#;
    let module = load(&lower(source));
    let (outcome, trace) =
        execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Error {
        message,
        span: Some(span),
    } = outcome
    else {
        panic!("expected rethrow with a source span, got {outcome:?}");
    };
    assert_eq!(message, "IndexError: list index out of range");
    assert_eq!(&source[span.start as usize..span.end as usize], "xs[9]");
    let origin = trace.last_language_error.expect("origin preserved");
    assert_eq!(origin.kind, LanguageErrorKind::ListIndex);
    assert_eq!(origin.span, Some(span));
    assert_eq!(trace.handled_language_errors, 1);
    assert_released(&trace, 1);
}

#[test]
fn verifier_rejects_reading_the_failed_call_result() {
    let mut mir = lower(STACK);
    let entry = &mut mir.funcs[mir.entry as usize];
    let (unwind, destination) = entry
        .blocks
        .iter()
        .find_map(|block| {
            if let (Terminator::Invoke { unwind, .. }, Some(MirInstr::Call { dst, .. })) =
                (&block.term, block.instrs.last())
            {
                Some((*unwind, *dst))
            } else {
                None
            }
        })
        .expect("shared lowering emitted a checked call and exceptional edge");
    entry.blocks[unwind as usize].term = Terminator::Return { value: destination };
    assert_eq!(
        ace_av_core::verify_mir(&mir).unwrap_err().code(),
        "mir.uninitialized-read",
    );
}

#[test]
fn console_authority_cancellation_and_io_failure_stay_outside_catch() {
    struct RejectingConsole {
        cancel: bool,
        writes: Vec<Vec<u8>>,
    }

    impl ConsoleHost for RejectingConsole {
        fn retained_bytes(&self) -> usize {
            0
        }

        fn write(
            &mut self,
            stream: Stream,
            bytes: &[u8],
            flag: &CancelFlag,
        ) -> Result<(), HostError> {
            assert_eq!(stream, Stream::Stdout);
            self.writes.push(bytes.to_vec());
            if self.cancel {
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                Err(HostError::Cancelled)
            } else {
                Err(HostError::Io("deliberate failure".into()))
            }
        }
    }

    let source = r#"
fn leaf() int {
    let text = "entered".clone()
    print(text)
    fail("language error")
}
fn main() int {
    try { return leaf() }
    catch { print("caught"); return 42 }
}
"#;
    let mir = lower(source);
    let module = load(&mir);
    let limits = ExecLimits::default();
    let mut host = RejectingConsole {
        cancel: false,
        writes: Vec::new(),
    };
    let (absent, trace) = execute_observed(&module, &limits, &CancelFlag::new(false));
    assert!(
        matches!(absent, ExecOutcome::Infra(message) if message.starts_with("host.unauthorized:"))
    );
    assert_eq!(trace.handled_language_errors, 0);
    assert!(host.writes.is_empty());

    let flag = CancelFlag::new(false);
    let (outcome, trace) = execute_sched(&module, &limits, &flag, &mut host);
    assert!(
        matches!(outcome, ExecOutcome::Infra(ref message) if message == "host.io: deliberate failure")
    );
    assert_eq!(trace.handled_language_errors, 0);
    assert_eq!(trace.frames_remaining, 0);
    assert_eq!(host.writes, [b"entered\n".to_vec()]);

    host.writes.clear();
    host.cancel = true;
    let (outcome, trace) = execute_sched(&module, &limits, &flag, &mut host);
    assert!(matches!(outcome, ExecOutcome::Cancelled));
    assert_eq!(trace.handled_language_errors, 0);
    assert_eq!(trace.frames_remaining, 0);
    assert_eq!(host.writes, [b"entered\n".to_vec()]);
}
