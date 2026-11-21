use ace_av::console_host::ConsoleCapture;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, ResourceKind, Value};
use ace_av_core::host::{ConsoleHost, HostError, Stream};
use ace_av_core::LoadedModule;
use std::sync::atomic::Ordering;

fn checked_case(name: &str) -> LoadedModule {
    let document = include_str!(
        "../../../../../../crates/auto-lang/test/cases/language/async/tasks-actors.case.ad"
    );
    let marker = format!("id: \"av.parity.1090.{name}\"");
    let section = document.split_once(&marker).expect("canonical case").1;
    let source = section
        .split_once("```auto\n")
        .expect("Auto source")
        .1
        .split_once("```")
        .expect("closed Auto fence")
        .0;
    let caps = [
        "ac.source.language.av/v1".into(),
        "console.stdout".into(),
        "console.stderr".into(),
    ];
    let source = ace_core::source::compile_language_source(source, "main.at", &caps)
        .expect("checked ordinary Auto");
    let mir = ace_lowering::lower_checked_auto_language(&source.checked, "d_main", 0)
        .expect("ordinary AV lowering");
    ace_av_core::verify_mir(&mir).expect("MIR verification");
    let bytes = ace_av_core::abc::encode_abc(&mir).expect("ABC encoding");
    ace_av_core::verify_abc_bytes(&bytes.to_flat(), &mir.owner).expect("ABC admission")
}

#[test]
fn full_mailbox_parks_and_wakes_the_sender_before_stop() {
    let module = checked_case("park-and-drain");
    let mut host = ConsoleCapture::default();
    let (outcome, trace) = execute_sched(
        &module,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
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
    assert_eq!(host.stdout, b"19\n23\nstopped\n");
    assert!(host.stderr.is_empty());
    let schedule = trace.scheduler.expect("scheduler observation");
    assert_eq!(schedule.parked_senders, 1);
    assert_eq!(schedule.woken_senders, 1);
}

#[test]
fn owned_capture_survives_wait_and_releases_after_delivery() {
    let module = checked_case("owned-capture");
    let mut host = ConsoleCapture::default();
    let (outcome, trace) = execute_sched(
        &module,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
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
    assert_eq!(host.stdout, b"stays owned\n");
    let schedule = trace.scheduler.expect("scheduler observation");
    assert_eq!(schedule.parked_awaits, 1);
    assert_eq!(schedule.woken_awaits, 1);
    assert!(schedule.text_allocations > 0);
    assert_eq!(schedule.live_text_after_cleanup, 0);
}

#[derive(Default)]
struct CancelAfterOutput(ConsoleCapture);

impl ConsoleHost for CancelAfterOutput {
    fn retained_bytes(&self) -> usize {
        self.0.retained_bytes()
    }

    fn write(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        self.0.write(stream, bytes, cancel)?;
        cancel.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn cancellation_while_awaiting_clears_the_owned_capture() {
    let module = checked_case("owned-capture");
    let mut host = CancelAfterOutput::default();
    let (outcome, trace) = execute_sched(
        &module,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
    assert!(matches!(outcome, ExecOutcome::Cancelled), "{outcome:?}");
    assert_eq!(host.0.stdout, b"stays owned\n");
    let schedule = trace.scheduler.expect("scheduler observation");
    assert_eq!(schedule.parked_awaits, 1);
    assert_eq!(schedule.woken_awaits, 0);
    assert!(schedule.text_allocations > 0);
    assert_eq!(schedule.live_text_after_cleanup, 0);
}

#[test]
fn budget_exhaustion_after_a_park_keeps_its_own_outcome() {
    let module = checked_case("park-and-drain");
    let mut host = ConsoleCapture::default();
    let limits = ExecLimits {
        max_steps: 20,
        ..ExecLimits::default()
    };
    let (outcome, trace) = execute_sched(&module, &limits, &CancelFlag::new(false), &mut host);
    assert!(
        matches!(
            outcome,
            ExecOutcome::Resource {
                kind: ResourceKind::InstrSteps,
                ..
            }
        ),
        "{outcome:?}"
    );
    let schedule = trace.scheduler.expect("scheduler observation");
    assert_eq!(schedule.parked_senders, 1);
    assert_eq!(schedule.live_text_after_cleanup, 0);
}

struct UnavailableConsole;

impl ConsoleHost for UnavailableConsole {
    fn retained_bytes(&self) -> usize {
        0
    }

    fn write(
        &mut self,
        _stream: Stream,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        Err(HostError::Io("unavailable test console".into()))
    }
}

#[test]
fn console_failure_while_awaiting_remains_a_host_failure() {
    let module = checked_case("owned-capture");
    let (outcome, trace) = execute_sched(
        &module,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut UnavailableConsole,
    );
    assert!(
        matches!(&outcome, ExecOutcome::Infra(message) if message == "host.io: unavailable test console"),
        "{outcome:?}"
    );
    assert_eq!(trace.handled_language_errors, 0);
    let schedule = trace.scheduler.expect("scheduler observation");
    assert_eq!(schedule.parked_awaits, 1);
    assert_eq!(schedule.woken_awaits, 0);
    assert_eq!(schedule.live_text_after_cleanup, 0);
}
