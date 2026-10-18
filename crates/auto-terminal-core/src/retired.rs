//! Process-wide retired cursor/viewport cache (PLAN-808 R3-F02).
//!
//! A handle's frozen cursor/viewport values must remain readable after free
//! from **any** consumer (another VM, a direct `EngineService::free` caller),
//! not only from the VM that executed the shim free and not only if someone
//! read the values before the owner was pruned. The engine service records a
//! pure value copy here on every successful session retirement; the map
//! holds no service/reference and never touches a freed DLL pointer.
//!
//! Handles are process-globally unique, so entries cannot alias a new
//! session. The map is bounded (FIFO) — it is a compatibility projection of
//! the legacy post-free `CURSORS`/`VIEWPORTS` values, not durable history.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use crate::types::TerminalRetiredCache;

/// Bound for the process-wide retired-value projection.
const RETIRED_CACHE_CAP: usize = 4096;

struct Store {
    map: HashMap<i64, TerminalRetiredCache>,
    order: VecDeque<i64>,
}

impl Store {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn insert(&mut self, handle: i64, cache: TerminalRetiredCache) {
        if !self.map.contains_key(&handle) {
            while self.map.len() >= RETIRED_CACHE_CAP {
                let Some(oldest) = self.order.pop_front() else {
                    break;
                };
                self.map.remove(&oldest);
            }
            self.order.push_back(handle);
        }
        self.map.insert(handle, cache);
    }

    fn get(&self, handle: i64) -> Option<TerminalRetiredCache> {
        self.map.get(&handle).copied()
    }
}

static STORE: OnceLock<Mutex<Store>> = OnceLock::new();

fn store() -> &'static Mutex<Store> {
    STORE.get_or_init(|| Mutex::new(Store::new()))
}

/// Record the frozen cursor/viewport values of a retired handle. Called by
/// `EngineService::free` before the session table entry is removed (the
/// values are pure data; the service may be dropped afterwards).
pub(crate) fn remember(handle: i64, cache: TerminalRetiredCache) {
    store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(handle, cache);
}

/// Retired cursor/viewport values for `handle` (None = no retired record).
pub fn retired_cache(handle: i64) -> Option<TerminalRetiredCache> {
    store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(handle)
}

#[cfg(test)]
pub(crate) fn clear_for_test() {
    let mut store = store().lock().unwrap_or_else(|e| e.into_inner());
    store.map.clear();
    store.order.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_store_keeps_values_and_evicts_fifo() {
        clear_for_test();
        let cache = TerminalRetiredCache {
            cursor_row: 3,
            cursor_col: 7,
            viewport_cols: 90,
            viewport_rows: 30,
        };
        remember(41, cache);
        assert_eq!(retired_cache(41), Some(cache));
        assert_eq!(retired_cache(42), None);
        clear_for_test();
    }
}
