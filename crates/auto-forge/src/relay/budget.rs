//! Token Budgeting
//!
//! Tracks and enforces per-step and per-run token spend.
//! Makes cost a first-class constraint, not a surprise bill.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token budget with limit, warning threshold, and enforcement strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub limit: u64,
    pub warning_at: u64,
    pub strategy: BudgetStrategy,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl TokenBudget {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            warning_at: (limit as f64 * 0.7) as u64,
            strategy: BudgetStrategy::HardStop,
        }
    }

    pub fn with_strategy(limit: u64, strategy: BudgetStrategy) -> Self {
        Self {
            limit,
            warning_at: (limit as f64 * 0.7) as u64,
            strategy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetStrategy {
    /// Halt step and request human decision.
    HardStop,
    /// Switch to cheaper model for remainder.
    EscalateModel,
    /// Aggressively compress context.
    SummarizeContext,
    /// Skip non-critical work.
    SkipOptional,
}

/// Tracks token usage across a run, per step and cumulatively.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BudgetTracker {
    pub run_budget: TokenBudget,
    pub step_budgets: HashMap<String, TokenBudget>,
    pub cumulative: u64,
    pub per_step: HashMap<String, u64>,
}

impl BudgetTracker {
    pub fn new(run_budget: TokenBudget) -> Self {
        Self {
            run_budget,
            step_budgets: HashMap::new(),
            cumulative: 0,
            per_step: HashMap::new(),
        }
    }

    /// Record token usage from an API response.
    pub fn record(&mut self, step: &str, input: u64, output: u64) {
        let total = input + output;
        self.cumulative += total;
        *self.per_step.entry(step.to_string()).or_insert(0) += total;
    }

    /// Check if current spend triggers any budget action.
    pub fn check(&self, step: &str) -> BudgetAction {
        let step_used = self.per_step.get(step).copied().unwrap_or(0);

        // Check step budget
        if let Some(step_budget) = self.step_budgets.get(step) {
            if step_used >= step_budget.limit {
                return BudgetAction::HardStop;
            }
            if step_used >= step_budget.warning_at {
                return BudgetAction::Warning {
                    remaining: step_budget.limit - step_used,
                };
            }
        }

        // Check run budget
        if self.cumulative >= self.run_budget.limit {
            return BudgetAction::HardStop;
        }
        if self.cumulative >= self.run_budget.warning_at {
            return BudgetAction::Warning {
                remaining: self.run_budget.limit - self.cumulative,
            };
        }

        BudgetAction::None
    }

    /// Set a per-step budget.
    pub fn set_step_budget(&mut self, step: &str, budget: TokenBudget) {
        self.step_budgets.insert(step.to_string(), budget);
    }

    /// Estimate what parallel multi-agent execution would have cost.
    /// Heuristic: N agents × avg_context_tokens × coordination_rounds.
    pub fn estimate_parallel_cost(&self, num_agents: u32, avg_context: u64, rounds: u32) -> u64 {
        (num_agents as u64) * avg_context * (rounds as u64) * 2 // 2× for input+output
    }

    /// Calculate savings vs parallel estimate.
    pub fn savings_vs_parallel(&self, num_agents: u32, avg_context: u64, rounds: u32) -> (u64, f64) {
        let parallel = self.estimate_parallel_cost(num_agents, avg_context, rounds);
        let saved = parallel.saturating_sub(self.cumulative);
        let ratio = if parallel > 0 {
            (saved as f64) / (parallel as f64)
        } else {
            0.0
        };
        (saved, ratio)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetAction {
    None,
    Warning { remaining: u64 },
    Compress,
    HardStop,
}

/// Cost analytics report for a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReport {
    pub run_id: String,
    pub total_tokens: u64,
    pub total_input: u64,
    pub total_output: u64,
    pub per_profession: HashMap<String, u64>,
    pub per_step: HashMap<String, u64>,
    pub budget_limit: u64,
    pub budget_remaining: u64,
    pub parallel_estimate: u64,
    pub savings: u64,
    pub savings_ratio: f64,
    pub checkpoints: u32,
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_warning_and_hardstop() {
        let budget = TokenBudget::new(1000);
        let mut tracker = BudgetTracker::new(budget);

        // 600 tokens — below warning
        tracker.record("planner", 400, 200);
        assert_eq!(tracker.check("planner"), BudgetAction::None);

        // 800 tokens — above 70% warning
        tracker.record("planner", 200, 0);
        assert_eq!(
            tracker.check("planner"),
            BudgetAction::Warning { remaining: 200 }
        );

        // 1200 tokens — hard stop
        tracker.record("planner", 400, 0);
        assert_eq!(tracker.check("planner"), BudgetAction::HardStop);
    }

    #[test]
    fn test_per_step_budget() {
        let run_budget = TokenBudget::new(10000);
        let mut tracker = BudgetTracker::new(run_budget);
        tracker.set_step_budget("planner", TokenBudget::new(1000));

        // 500 tokens — below 700 warning threshold
        tracker.record("planner", 400, 100);
        assert_eq!(tracker.check("planner"), BudgetAction::None);

        // 800 tokens — above 700 warning threshold
        tracker.record("planner", 300, 0);
        assert_eq!(
            tracker.check("planner"),
            BudgetAction::Warning { remaining: 200 }
        );

        // 1200 tokens — above 1000 limit
        tracker.record("planner", 400, 0);
        assert_eq!(tracker.check("planner"), BudgetAction::HardStop);
    }

    #[test]
    fn test_parallel_cost_estimate() {
        let budget = TokenBudget::new(100000);
        let tracker = BudgetTracker::new(budget);
        let cost = tracker.estimate_parallel_cost(5, 8000, 3);
        assert_eq!(cost, 240000); // 5 × 8000 × 3 × 2
    }

    #[test]
    fn test_savings_vs_parallel() {
        let run_budget = TokenBudget::new(100000);
        let mut tracker = BudgetTracker::new(run_budget);
        tracker.record("planner", 5000, 3000);
        tracker.record("architect", 8000, 6000);
        tracker.record("coder", 15000, 12000);

        let (saved, ratio) = tracker.savings_vs_parallel(5, 8000, 3);
        let parallel = 240000;
        assert_eq!(saved, parallel - 49000);
        assert!((ratio - 0.79).abs() < 0.01);
    }
}
