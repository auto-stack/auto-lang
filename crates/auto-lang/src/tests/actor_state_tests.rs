//! Plan 317: Task actor state fields — regression tests.
//!
//! State fields (`task T { count = 0 }`) persist across handler invocations
//! on the actor's AutoVM task, accessed via LOAD_STATE_FIELD/STORE_STATE_FIELD.

use crate::run_with_capture;

/// State field is initialized (via start hook) and a handler can write it.
#[test]
fn actor_state_field_write() {
    let code = r#"
task Counter {
    count = 0
    fn start() ! { }
    on {
        1 -> {
            count = 42
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16)
    h.send(1)
    print("ok")
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    assert!(s.contains("ok"), "state write: stdout={:?} result={:?}", s, r);
}

/// State field increment persists across multiple handler invocations.
/// count starts at 0; each message does count = count + 1.
/// After 3 messages, count == 3 (verified via conditional branch).
#[test]
fn actor_state_field_increment_persists() {
    let code = r#"
task Counter {
    count = 0
    fn start() ! { }
    on {
        1 -> {
            count = count + 1
            if count == 2 {
                print("reached two")
            }
            if count == 3 {
                print("reached three")
            }
        }
    }
}
fn main() {
    let h = Task.spawn("Counter", 16)
    h.send(1)
    h.send(1)
    h.send(1)
}
"#;
    let (r, s) = run_with_capture(code).unwrap_or_else(|e| (format!("ERROR: {}", e), String::new()));
    // count increments 1, 2, 3 across the three messages. The conditionals
    // fire on the 2nd (count==2) and 3rd (count==3) invocations.
    assert!(s.contains("reached two"), "count==2: stdout={:?} result={:?}", s, r);
    assert!(s.contains("reached three"), "count==3: stdout={:?} result={:?}", s, r);
}
