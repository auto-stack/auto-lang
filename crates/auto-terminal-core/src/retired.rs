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
//! session. The map is deliberately **unbounded**: it is a compatibility
//! projection of the legacy post-free `CURSORS`/`VIEWPORTS` values, which
//! were never evicted, and a still-queryable frozen value may not be
//! dropped because unrelated sessions retired later (R4-F02) — any bound
//! would be an unauthorized semantic reduction. Growth is bounded by the
//! sessions actually retired in the process, and each entry is a small
//! pure value.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::types::TerminalRetiredCache;

struct Store {
    map: HashMap<i64, TerminalRetiredCache>,
}

impl Store {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, handle: i64, cache: TerminalRetiredCache) {
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
    store()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .map
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_store_keeps_all_values_without_eviction() {
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
        // Far beyond the retired R3-era FIFO bound (4096): the first value
        // must still be readable — frozen values are never evicted (R4-F02).
        let filler: Vec<(i64, TerminalRetiredCache)> = (0..5000i64)
            .map(|i| {
                (
                    1_000_000 + i,
                    TerminalRetiredCache {
                        cursor_row: i,
                        cursor_col: i,
                        viewport_cols: 2,
                        viewport_rows: 1,
                    },
                )
            })
            .collect();
        for (handle, value) in &filler {
            remember(*handle, *value);
        }
        assert_eq!(retired_cache(41), Some(cache));
        for (handle, value) in &filler {
            assert_eq!(retired_cache(*handle), Some(*value));
        }
        clear_for_test();
    }
}
