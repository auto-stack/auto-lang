//! Query-level resource budget (PLAN-796 §5.4, contract-decision D9).
//!
//! One monotonic step counter + one wall-clock deadline shared by a
//! single semantic task. Step definition (frozen here, D9): one step =
//! one worklist constraint solved, OR one AST node visited, OR one
//! scope/candidate lookup. Crossing a limit is an INFRASTRUCTURE state
//! (`BudgetError`), never a type error, and never cached as a program
//! verdict.

use std::time::{Duration, Instant};

/// Default per-query limits (probe run policy, not a language limit).
pub const DEFAULT_MAX_STEPS: u64 = 1_000_000;
pub const DEFAULT_WALL: Duration = Duration::from_secs(5);

#[derive(Clone, Debug)]
pub struct Budget {
    pub max_steps: u64,
    pub wall: Duration,
    steps: u64,
    start: Instant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BudgetError {
    StepsExhausted { steps: u64, max: u64 },
    DeadlineExceeded { elapsed_ms: u128, max_ms: u128 },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::StepsExhausted { steps, max } => {
                write!(f, "resource: {steps} steps exceeded the {max}-step budget")
            }
            BudgetError::DeadlineExceeded { elapsed_ms, max_ms } => {
                write!(f, "resource: {elapsed_ms}ms exceeded the {max_ms}ms wall budget")
            }
        }
    }
}

impl std::error::Error for BudgetError {}

impl Default for Budget {
    fn default() -> Self {
        Budget::new(DEFAULT_MAX_STEPS, DEFAULT_WALL)
    }
}

impl Budget {
    pub fn new(max_steps: u64, wall: Duration) -> Self {
        Budget { max_steps, wall, steps: 0, start: Instant::now() }
    }

    /// Charge `n` steps and check both limits. Called from solver loops
    /// at bounded granularity (never per byte).
    pub fn charge(&mut self, n: u64) -> Result<(), BudgetError> {
        self.steps = self.steps.saturating_add(n);
        if self.steps > self.max_steps {
            return Err(BudgetError::StepsExhausted { steps: self.steps, max: self.max_steps });
        }
        self.check_deadline()
    }

    pub fn check_deadline(&self) -> Result<(), BudgetError> {
        let elapsed = self.start.elapsed();
        if elapsed > self.wall {
            return Err(BudgetError::DeadlineExceeded {
                elapsed_ms: elapsed.as_millis(),
                max_ms: self.wall.as_millis(),
            });
        }
        Ok(())
    }

    pub fn steps_used(&self) -> u64 {
        self.steps
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}
