//! Plan 442 A5 regression tests: one-shot scheduler primitives
//! (`sched.set_timeout` / `sched.clear_timeout`) on the VM render target.
//!
//! ## Design
//!
//! auto-down 008 Phase 3's render scheduler needs an injectable one-shot
//! timer (`setTimeout(fn, ms) -> handle` + `clearTimeout(handle)`, 16ms
//! cadence) — the VM render target had only blocking `time.sleep_ms` (which
//! stalls the engine thread and never worked inside handlers anyway). A5
//! adds:
//! - a per-VM timer registry (`AutoVM::timers`, `set_timer`/`clear_timer`/
//!   `due_timers`);
//! - `sched.set_timeout(callback, delay_ms)` accepting a closure value
//!   (Int closure id) or an event-name string, and `sched.clear_timeout`;
//! - dispatch on the render loop: the iced `__timer_tick` subscription
//!   (16ms, only while timers are pending) calls `DynamicComponent::
//!   poll_timers`, which dispatches event-form callbacks like a UI event
//!   and closure-form callbacks via `call_closure`.
//!
//! Corpus: `test/ui/plan442_sched/` — event form asserts observable state
//! change; closure form asserts the poll fires (count) without error.

#[cfg(test)]
mod plan442_sched_tests {
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan442_sched/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        crate::plan370_test_support::build_component_from_app(&locate_corpus()?)
    }

    fn state_str(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(auto_val::Value::Int(i)) => i.to_string(),
            Ok(auto_val::Value::Bool(b)) => b.to_string(),
            Ok(other) => format!("{:?}", other),
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    /// REGRESSION: the event-form timer fires after its deadline and updates
    /// state through the normal handler dispatch path.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn sched_event_timer_fires_via_poll() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // .Init scheduled "Fired" at +20ms. Before the deadline: nothing.
        assert!(dc.has_pending_timers(), "timer should be pending after Init");
        let early = dc.poll_timers();
        assert_eq!(early, 0, "timer must not fire before its deadline");
        assert_eq!(state_str(&dc, "ticks"), "0");
        // After the deadline: exactly one fire, state updated by the handler.
        std::thread::sleep(std::time::Duration::from_millis(40));
        let fired = dc.poll_timers();
        assert_eq!(fired, 1);
        assert_eq!(state_str(&dc, "ticks"), "1");
        assert!(!dc.has_pending_timers(), "one-shot timer must not repeat");
    }

    /// REGRESSION: the closure-form timer (SchedulerTimer parity) schedules
    /// and fires without error.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn sched_closure_timer_fires() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // Drain Init's event-form timer first so this test observes only the
        // closure timer.
        std::thread::sleep(std::time::Duration::from_millis(40));
        let _ = dc.poll_timers();
        assert!(!dc.has_pending_timers(), "precondition: no pending timers");
        dc.on_with_input("Schedule", None);
        assert!(dc.has_pending_timers());
        std::thread::sleep(std::time::Duration::from_millis(40));
        let fired = dc.poll_timers();
        assert_eq!(fired, 1, "closure-form timer should fire exactly once");
        assert!(!dc.has_pending_timers());
    }

    /// REGRESSION: clear_timeout cancels a pending timer — the SchedulerTimer
    /// contract's clearTimeout(handle).
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn sched_clear_timeout_cancels() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // .Init scheduled "Fired" at +20ms; cancel it before it fires.
        dc.on_with_input("Cancel", None);
        assert_eq!(state_str(&dc, "cancelled"), "true", "clear_timeout should report removal");
        assert!(!dc.has_pending_timers());
        std::thread::sleep(std::time::Duration::from_millis(40));
        assert_eq!(dc.poll_timers(), 0);
        assert_eq!(state_str(&dc, "ticks"), "0", "cancelled timer must never fire");
    }
}
