//! Message / output / queue budgets (PLAN-815 §5.3, T-01 §4).
//!
//! Values are frozen contract constants, shared by the tool host and every
//! client so neither side invents its own limit.

/// Frozen default budgets (units are bytes / messages as named).
pub const DEFAULT_BUDGETS: Budgets = Budgets {
    // Maximum size of a single framed message (payload bytes, excluding
    // the 4-byte length prefix), enforced in both directions.
    max_frame_bytes: 8 * 1024 * 1024,
    // Maximum cumulative progress-event payload bytes per operation.
    max_progress_bytes_per_op: 1024 * 1024,
    // Maximum count of not-yet-consumed notifications a client may queue
    // before applying backpressure. The A-stage host answers synchronously,
    // so this bound is structural there; executors (PLAN-816) enforce it.
    max_queue_depth: 128,
};

/// Named budget set with explicit units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budgets {
    /// Bytes; one framed message, both directions.
    pub max_frame_bytes: usize,
    /// Bytes; cumulative progress payload per operation.
    pub max_progress_bytes_per_op: usize,
    /// Messages; queued-but-unconsumed notifications per client.
    pub max_queue_depth: usize,
}

impl Budgets {
    /// The frozen contract limits.
    pub const fn contract() -> Self {
        DEFAULT_BUDGETS
    }

    /// Classify an observed frame length against the frame budget.
    pub fn check_frame(&self, observed: usize) -> Result<(), usize> {
        if observed <= self.max_frame_bytes {
            Ok(())
        } else {
            Err(observed)
        }
    }
}

impl Default for Budgets {
    fn default() -> Self {
        DEFAULT_BUDGETS
    }
}
