//! Plan 327 Phase 1: Task/Msg actor handler execution — regression tests.
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
