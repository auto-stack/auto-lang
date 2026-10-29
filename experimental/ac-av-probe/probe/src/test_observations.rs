//! Feature-only observations at real C producer calls; no outcome decisions.
use std::{cell::RefCell, collections::BTreeMap, time::Instant};

#[derive(Default, serde::Serialize)]
pub struct Metrics {
    pub calls: BTreeMap<String, u64>,
    pub spans_ms: Vec<(String, f64)>,
}

thread_local! { static ACTIVE: RefCell<Option<Metrics>> = const { RefCell::new(None) }; }

pub fn capture<T>(f: impl FnOnce() -> T) -> Result<(T, Metrics), String> {
    if ACTIVE.with(|slot| slot.borrow().is_some()) {
        return Err("nested test observation capture".into());
    }
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ACTIVE.with(|slot| {
                slot.borrow_mut().take();
            });
        }
    }
    ACTIVE.with(|slot| *slot.borrow_mut() = Some(Metrics::default()));
    let _reset = Reset;
    let result = f();
    let metrics = ACTIVE
        .with(|slot| slot.borrow_mut().take())
        .ok_or("observation capture lost")?;
    Ok((result, metrics))
}

pub fn observed<T>(stage: &str, f: impl FnOnce() -> T) -> T {
    let enabled = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(metrics) = slot.as_mut() {
            *metrics.calls.entry(stage.into()).or_default() += 1;
            true
        } else {
            false
        }
    });
    let started = Instant::now();
    let result = f();
    if enabled {
        ACTIVE.with(|slot| {
            if let Some(metrics) = slot.borrow_mut().as_mut() {
                metrics
                    .spans_ms
                    .push((stage.into(), started.elapsed().as_secs_f64() * 1000.0));
            }
        });
    }
    result
}
