//! Feature-only observations of actual parent boundaries and successful spawns.
//! No worker protocol fields or expectation/comparison decisions are changed.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(serde::Serialize)]
pub struct Observations {
    pub worker_pids: Vec<u32>,
    pub phases_ms: BTreeMap<&'static str, f64>,
}

impl Default for Observations {
    fn default() -> Self {
        Self {
            worker_pids: Vec::new(),
            phases_ms: [
                "prepare_ms",
                "transport_ms",
                "admission_compare_receipt_ms",
                "cleanup_ms",
            ]
            .into_iter()
            .map(|key| (key, 0.0))
            .collect(),
        }
    }
}

thread_local! { static ACTIVE: RefCell<Option<Observations>> = const { RefCell::new(None) }; }

pub fn capture<R>(operation: impl FnOnce() -> R) -> (R, Observations) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ACTIVE.with(|state| {
                state.borrow_mut().take();
            });
        }
    }
    ACTIVE.with(|state| {
        assert!(
            state.borrow().is_none(),
            "parent observation capture cannot nest"
        );
        *state.borrow_mut() = Some(Observations::default());
    });
    let reset = Reset;
    let result = operation();
    let observations = ACTIVE.with(|state| state.borrow_mut().take().unwrap());
    drop(reset);
    (result, observations)
}

fn enabled() -> bool {
    ACTIVE.with(|state| state.borrow().is_some())
}
fn total(key: &'static str) -> f64 {
    ACTIVE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|s| s.phases_ms[key])
            .unwrap_or(0.0)
    })
}
fn add(key: &'static str, milliseconds: f64) {
    ACTIVE.with(|state| {
        if let Some(state) = state.borrow_mut().as_mut() {
            *state.phases_ms.get_mut(key).expect("declared parent phase") += milliseconds;
        }
    });
}

pub(super) fn spawned(pid: u32) {
    ACTIVE.with(|state| {
        if let Some(state) = state.borrow_mut().as_mut() {
            state.worker_pids.push(pid);
        }
    });
}

pub(super) struct Clock {
    last: Option<Instant>,
    cleanup: f64,
}
impl Clock {
    pub(super) fn new() -> Self {
        Self {
            last: enabled().then(Instant::now),
            cleanup: total("cleanup_ms"),
        }
    }
    pub(super) fn lap(&mut self, key: &'static str) {
        if let Some(start) = self.last {
            let now = Instant::now();
            let cleanup = total("cleanup_ms");
            let nested_cleanup = if key == "transport_ms" {
                cleanup - self.cleanup
            } else {
                0.0
            };
            add(
                key,
                ((now - start).as_secs_f64() * 1000.0 - nested_cleanup).max(0.0),
            );
            self.last = Some(now);
            self.cleanup = cleanup;
        }
    }
}

pub(super) struct Phase {
    key: &'static str,
    start: Option<Instant>,
}
impl Phase {
    pub(super) fn start(key: &'static str) -> Self {
        Self {
            key,
            start: enabled().then(Instant::now),
        }
    }
}
impl Drop for Phase {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            add(self.key, start.elapsed().as_secs_f64() * 1000.0);
        }
    }
}
