//! Plan 317 Phase 1: Task/Msg actor handler execution — regression tests.
//!
//! Before Phase 1, `task` definitions' `fn start()!{}` hooks and `on { }`
//! message handlers never executed: AutoVM.task_handler_registry was empty,
//! there was no message queue, and run_task_loop couldn't wake message-loop
//! tasks. These tests verify the fix (path B: VM-internal scheduling).

use crate::run_with_capture;

/// `fn start()!{}` hook executes when Task.spawn creates the actor.
#[test]
fn actor_start_hook_runs() {
    let code = r#"
task Greeter {
    fn start() ! {
        print("Greeter started")
    }
    on {
        1 -> {
            print("got msg")
        }
    }
}

fn main() {
    let h = Task.spawn("Greeter", 16)
    h.send(1)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    assert!(stdout.contains("Greeter started"), "start hook: stdout={:?} result={:?}", stdout, result);
}

/// `on { Pat -> {} }` message handler executes for a matching message.
#[test]
fn actor_message_handler_runs() {
    let code = r#"
task Echo {
    fn start() ! {
    }
    on {
        1 -> {
            print("got one")
        }
        2 -> {
            print("got two")
        }
    }
}

fn main() {
    let h = Task.spawn("Echo", 16)
    h.send(1)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    assert!(stdout.contains("got one"), "handler match: stdout={:?} result={:?}", stdout, result);
}

/// Multiple messages each trigger their handler, in send order.
#[test]
fn actor_multiple_messages_dispatched() {
    let code = r#"
task Echo {
    fn start() ! {
    }
    on {
        1 -> {
            print("got one")
        }
        2 -> {
            print("got two")
        }
    }
}

fn main() {
    let h = Task.spawn("Echo", 16)
    h.send(1)
    h.send(2)
    h.send(1)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // All three messages dispatched.
    assert_eq!(stdout.trim(), "got one\ngot two\ngot one", "multi-msg: stdout={:?} result={:?}", stdout, result);
}

/// `else -> {}` handler runs when no pattern matches.
#[test]
fn actor_else_handler_runs() {
    let code = r#"
task Router {
    fn start() ! {
    }
    on {
        1 -> {
            print("matched one")
        }
        else -> {
            print("fell through")
        }
    }
}

fn main() {
    let h = Task.spawn("Router", 16)
    h.send(99)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    assert!(stdout.contains("fell through"), "else handler: stdout={:?} result={:?}", stdout, result);
    assert!(!stdout.contains("matched one"), "else handler should not match: stdout={:?}", stdout);
}

/// Actor does not hang the VM after messages are consumed: main returns and
/// the idle actor (empty mailbox) lets run_task_loop exit cleanly.
#[test]
fn actor_vm_exits_after_messages() {
    let code = r#"
task Echo {
    fn start() ! {
    }
    on {
        1 -> {
            print("ping")
        }
    }
}

fn main() {
    let h = Task.spawn("Echo", 16)
    h.send(1)
    print("main done")
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // Both the handler output and main's "done" must appear, and the VM must
    // have returned (result is the last expression repr, not a timeout).
    assert!(stdout.contains("ping"), "handler ran: stdout={:?}", stdout);
    assert!(stdout.contains("main done"), "main completed: stdout={:?} result={:?}", stdout, result);
}

/// Plan 317 §11 Phase 8 (P4'): a task with `on` handlers but NO `fn start()` and
/// NO state fields must still bind the message payload correctly.
///
/// Before the fix, such a task never got a `#start` export (the #start emit
/// block was gated on `start_hook.is_some() || has_state`), so
/// `shim_task_spawn_vm` fell back to start_offset=0, the spawned task ran from
/// the wrong ip, and the `on { n int -> }` binding read 0 instead of the sent
/// value (`h.send(42)` → handler printed "got 0"). The fix also emits `#start`
/// when `has_handlers`, so every message-receiving actor has a proper entry
/// that parks it in the message loop.
#[test]
fn actor_no_start_no_state_binds_payload() {
    let code = r#"
task Solo {
    on { n int -> {
        print("got " + n.to(str))
    } }
}
fn main() {
    let h = Task.spawn("Solo", 0)
    h.send(42)
    h.send(99)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    assert!(stdout.contains("got 42"), "payload msg1 (42): stdout={:?} result={:?}", stdout, result);
    assert!(stdout.contains("got 99"), "payload msg2 (99): stdout={:?} result={:?}", stdout, result);
}

/// Plan 317 §11 Phase 8 (P4'): cross-actor coexistence + independent payload
/// binding. Two actors of different types each receive their own messages; the
/// payload bindings must not bleed across actors and each `n` must equal the
/// sent value.
#[test]
fn actor_two_types_independent_payload() {
    let code = r#"
task Adder1 {
    total = 0
    on { n int -> {
        total = total + n
        print(total)
    } }
}
task Adder2 {
    total = 0
    on { n int -> {
        total = total + n
        print(total)
    } }
}
fn main() {
    let h1 = Task.spawn("Adder1", 16)
    let h2 = Task.spawn("Adder2", 16)
    h1.send(5)
    h2.send(7)
    h1.send(3)
    h2.send(9)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // Adder1: 0+5=5, 5+3=8; Adder2: 0+7=7, 7+9=16. Independent totals, correct
    // payloads (no bleed across actors).
    assert!(stdout.contains("5"), "Adder1 msg1 (total=5): stdout={:?} result={:?}", stdout, result);
    assert!(stdout.contains("8"), "Adder1 msg2 (total=8): stdout={:?} result={:?}", stdout, result);
    assert!(stdout.contains("7"), "Adder2 msg1 (total=7): stdout={:?} result={:?}", stdout, result);
    assert!(stdout.contains("16"), "Adder2 msg2 (total=16): stdout={:?} result={:?}", stdout, result);
}
