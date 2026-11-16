//! PLAN-968 (M2-A): the task actor & event scheduler — `task`/`on`
//! declarations, `Task.spawn`, handle `send`/`ask`/`stop`, `reply`,
//! `await`, `go`, `sleep`, bounded mailboxes (backpressure), virtual
//! timers, orderly stop-hook drain and named rejections. Independent
//! oracles: exact stdout bytes + exit values (the canonical
//! `message_handler`/`spawn_with_state` obligations and the plan's
//! V-DETAIL-01..03, V-08 observables).

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::ExecOutcome;
use ace_av_core::verify_mir;
use ace_core::source::compile_scheduler_source;

/// Full producer→consumer chain: scheduler source → compile → lower →
/// verify → ABC admit → the deterministic cooperative scheduler.
fn run_sched(src: &str) -> Result<(ExecOutcome, String), String> {
    let compiled = compile_scheduler_source(src, "probe.at", &[]).map_err(|ds| {
        ds.iter()
            .map(|d| d.render("probe.at", src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let abc = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    let flat = abc.to_flat();
    let loaded =
        ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner).map_err(|e| e.to_string())?;
    let mut host = ConsoleCapture::default();
    let (out, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
    let stdout = String::from_utf8_lossy(&host.stdout).to_string();
    Ok((out, stdout))
}

fn ok(src: &str, want_stdout: &str, want_exit: Option<i32>) {
    match run_sched(src) {
        Ok((ExecOutcome::Success { value, .. }, _)) => {
            let got = match value {
                ace_av_core::contract::Value::I32(v) => v,
                other => panic!("non-i32 exit {other:?}"),
            };
            if let Some(w) = want_exit {
                assert_eq!(got, w, "exit mismatch for source:\n{src}");
            }
        }
        Ok((other, stdout)) => {
            panic!("expected success, got {other:?} (stdout {stdout:?}) for source:\n{src}")
        }
        Err(e) => panic!("compile failed: {e} for source:\n{src}"),
    }
    let (_, stdout) = run_sched(src).expect("second run deterministic");
    assert_eq!(stdout, want_stdout, "stdout mismatch for source:\n{src}");
}

fn rejected(src: &str, want_code: &str) {
    match run_sched(src) {
        Err(e) => {
            assert!(
                e.contains(want_code),
                "expected diagnostic {want_code}, got: {e} for source:\n{src}"
            );
        }
        Ok((out, stdout)) => panic!(
            "expected rejection {want_code}, got {out:?} (stdout {stdout:?}) for source:\n{src}"
        ),
    }
}

fn outcome_is(src: &str, msg_part: &str) {
    match run_sched(src) {
        Ok((out, _)) => {
            let text = format!("{out:?}");
            assert!(
                text.contains(msg_part),
                "expected outcome containing {msg_part:?}, got {text} for source:\n{src}"
            );
        }
        Err(e) => panic!("compile failed: {e} for source:\n{src}"),
    }
}

/// The canonical `message_handler.at` obligation (V-DETAIL-01): send(1)
/// prints exactly "got one"; unit-main observes through stdout only.
#[test]
fn sched_968_message_handler_canonical() {
    ok(
        "task Echo {\n    fn start() ! {\n    }\n    on {\n        1 -> {\n            print(\"got one\")\n        }\n        2 -> {\n            print(\"got two\")\n        }\n    }\n}\n\nfn main() {\n    let h = Task.spawn(\"Echo\", 16)\n    h.send(1)\n}\n",
        "got one\n",
        Some(0),
    );
}

/// The canonical `spawn_with_state.at` obligation (V-DETAIL-01): state
/// fields with defaults, positional spawn initializers and the legacy
/// bare-brace f-string interpolation print "mycounter: 15".
#[test]
fn sched_968_spawn_with_state_canonical() {
    ok(
        "task Counter {\n    count = 0\n    label = \"default\"\n    on { n int -> {\n        count = count + n\n        print(f\"{label}: {count}\")\n    } }\n}\n\nfn main() {\n    let h = Task.spawn(\"Counter\", 16, 10, \"mycounter\")\n    h.send(5)\n}\n",
        "mycounter: 15\n",
        Some(0),
    );
}

/// V-DETAIL-01: FIFO mailbox ordering — two sends dispatch in order.
#[test]
fn sched_968_mailbox_fifo_order() {
    ok(
        "task E {\n    on {\n        1 -> {\n            print(\"got one\")\n        }\n        2 -> {\n            print(\"got two\")\n        }\n    }\n}\nfn main() {\n    let h = Task.spawn(\"E\", 16)\n    h.send(2)\n    h.send(1)\n}\n",
        "got two\ngot one\n",
        Some(0),
    );
}

/// V-02/V-08: ask/await/reply correlation — the reply carries 41+1.
#[test]
fn sched_968_ask_await_reply() {
    ok(
        "task Q {\n    on {\n        n int -> {\n            reply(n + 1)\n        }\n    }\n}\nfn main() {\n    let h = Task.spawn(\"Q\", 16)\n    let v = await h.ask(41)\n    print(\"reply:\")\n    print(v)\n}\n",
        "reply:\n42\n",
        Some(0),
    );
}

/// V-08: fire-and-forget `go` runs the call concurrently; the program
/// joins it before exiting.
#[test]
fn sched_968_go_fire_and_forget() {
    ok(
        "fn worker(n int) {\n    print(\"go:\")\n    print(n)\n}\nfn main() {\n    go worker(7)\n}\n",
        "go:\n7\n",
        Some(0),
    );
}

/// V-08/T-08: bounded mailbox backpressure — capacity 1, the sender
/// parks until the handler drains; all three messages still deliver.
#[test]
fn sched_968_backpressure_bounded_mailbox() {
    ok(
        "task B {\n    on {\n        n int -> {\n            sleep(50)\n            print(n)\n        }\n    }\n}\nfn main() {\n    let h = Task.spawn(\"B\", 1)\n    h.send(1)\n    h.send(2)\n    h.send(3)\n}\n",
        "1\n2\n3\n",
        Some(0),
    );
}

/// T-08/V-DETAIL-03: the stop hook runs at the orderly drain.
#[test]
fn sched_968_stop_hook_drain() {
    ok(
        "task S {\n    fn stop() ! {\n        print(\"stopped\")\n    }\n    on {\n        1 -> {\n            print(\"one\")\n        }\n    }\n}\nfn main() {\n    let h = Task.spawn(\"S\", 16)\n    h.send(1)\n}\n",
        "one\nstopped\n",
        Some(0),
    );
}

/// V-01: a virtual-timer sleep orders events across chains without wall
/// time (the sleeper prints after main).
#[test]
fn sched_968_virtual_timer_order() {
    ok(
        "fn main() {\n    go sleeper()\n    print(\"main\")\n}\nfn sleeper() {\n    sleep(100)\n    print(\"later\")\n}\n",
        "main\nlater\n",
        Some(0),
    );
}

/// V-DETAIL-02: unknown task name — named bind rejection.
#[test]
fn sched_968_neg_unknown_task() {
    rejected(
        "fn main() {\n    let h = Task.spawn(\"Nope\", 16)\n}\n",
        "source.sched-task",
    );
}

/// V-DETAIL-02: a malformed task declaration — named rejection, never a
/// panic (the recovery surface maps to the profile rejection).
#[test]
fn sched_968_neg_malformed_task() {
    rejected(
        "task T {\n    on {\n        1 ->\n    }\n}\nfn main() {\n}\n",
        "source.",
    );
}

/// V-DETAIL-01/02: no late commits — a send after stop is a named
/// terminal rejection.
#[test]
fn sched_968_neg_send_on_stopped_task() {
    outcome_is(
        "task C {\n    fn stop() ! {\n    }\n    on {\n        1 -> {\n        }\n    }\n}\nfn main() {\n    let h = Task.spawn(\"C\", 16)\n    h.stop()\n    h.send(1)\n}\n",
        "send on a closed task",
    );
}

/// V-DETAIL-02: `reply` outside an ask context — named terminal
/// rejection (the send-mode arm).
#[test]
fn sched_968_neg_reply_in_send_mode() {
    outcome_is(
        "task R {\n    on {\n        1 -> {\n            reply(9)\n        }\n    }\n}\nfn main() {\n    let h = Task.spawn(\"R\", 16)\n    h.send(1)\n}\n",
        "reply is only legal",
    );
}
