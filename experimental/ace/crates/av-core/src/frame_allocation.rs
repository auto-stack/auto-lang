//! Scoped fault injection at the executor's real frame-slot reserve boundary.
//! Counts frame-slot buffers, not arbitrary process/diagnostic allocations.
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AllocationStats {
    pub attempts: u64,
    pub successful: u64,
    pub released: u64,
    pub live_bytes: usize,
    pub peak_bytes: usize,
}

#[derive(Default)]
struct State {
    fail_at: Option<u64>,
    stats: AllocationStats,
}

thread_local! {
    static ACTIVE: RefCell<Option<Arc<Mutex<State>>>> = const { RefCell::new(None) };
}

/// An opt-in, per-thread scope. No environment variables or global allocator.
#[derive(Clone)]
pub struct FrameAllocationProbe(Arc<Mutex<State>>);

impl FrameAllocationProbe {
    pub fn new(fail_at: Option<u64>) -> Self {
        assert!(fail_at != Some(0), "allocation indices start at one");
        Self(Arc::new(Mutex::new(State {
            fail_at,
            ..State::default()
        })))
    }

    pub fn snapshot(&self) -> AllocationStats {
        self.0.lock().unwrap().stats
    }

    pub fn run<R>(&self, f: impl FnOnce() -> R) -> R {
        struct Restore(Option<Arc<Mutex<State>>>);
        impl Drop for Restore {
            fn drop(&mut self) {
                ACTIVE.with(|a| *a.borrow_mut() = self.0.take());
            }
        }
        let _restore = Restore(ACTIVE.with(|a| a.replace(Some(self.0.clone()))));
        f()
    }
}

pub(crate) struct AllocationLease {
    state: Arc<Mutex<State>>,
    bytes: usize,
}

impl Drop for AllocationLease {
    fn drop(&mut self) {
        let mut s = self.state.lock().unwrap();
        s.stats.released += 1;
        s.stats.live_bytes -= self.bytes;
    }
}

pub(crate) fn reserve_slots<T>(len: usize) -> Result<(Vec<T>, Option<AllocationLease>), ()> {
    if len == 0 {
        return Ok((Vec::new(), None));
    }
    let active = ACTIVE.with(|a| a.borrow().clone());
    if let Some(s) = &active {
        let mut s = s.lock().unwrap();
        s.stats.attempts += 1;
        if s.fail_at == Some(s.stats.attempts) {
            return Err(());
        }
    }
    let mut values = Vec::new();
    values.try_reserve_exact(len).map_err(|_| ())?;
    let lease = active.map(|state| {
        let bytes = values.capacity() * std::mem::size_of::<T>();
        {
            let mut s = state.lock().unwrap();
            s.stats.successful += 1;
            s.stats.live_bytes += bytes;
            s.stats.peak_bytes = s.stats.peak_bytes.max(s.stats.live_bytes);
        }
        AllocationLease { state, bytes }
    });
    Ok((values, lease))
}
