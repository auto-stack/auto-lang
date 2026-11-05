//! Scoped observations at the actual neutral interface producer ports.
//! Scopes compose and unwind safely; no compiler/store dependency lives here.
use std::{cell::RefCell, sync::Arc};

pub type Observer = Arc<dyn Fn(&'static str, bool) + Send + Sync>;
thread_local! { static OBSERVERS: RefCell<Vec<Observer>> = const { RefCell::new(Vec::new()) }; }

pub fn with_observer<T>(observer: Observer, f: impl FnOnce() -> T) -> T {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            OBSERVERS.with(|o| {
                o.borrow_mut().pop();
            });
        }
    }
    OBSERVERS.with(|o| o.borrow_mut().push(observer));
    let _guard = Guard;
    f()
}
pub(crate) fn observe(stage: &'static str, produced: bool) {
    let observers = OBSERVERS.with(|o| o.borrow().clone());
    for observer in observers {
        observer(stage, produced);
    }
}
pub(crate) fn produce<T, E>(stage: &'static str, f: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    observe(stage, false);
    let result = f();
    if result.is_ok() {
        observe(stage, true);
    }
    result
}
