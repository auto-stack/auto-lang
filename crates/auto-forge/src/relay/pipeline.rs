//! Pipeline Engine
//!
//! The deterministic state machine that executes Flow specs.
//! Pure Rust code — zero LLM tokens spent on orchestration.

use crate::relay::budget::{BudgetTracker, TokenBudget};
use crate::relay::flow::{ExitRouting, FlowSpec, GateType};
use crate::relay::handoff::HandoffDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of advancing the pipeline — tells the caller what to do next.
#[derive(Debug, Clone, PartialEq)]
pub enum AdvanceResult {
    /// Execute the given step by running its agent.
    ExecuteStep {
        step_id: String,
        profession_id: String,
    },
    /// Pause for human approval at a gate.
    WaitForHuman {
        gate: GateType,
        step_id: String,
    },
    /// Flow completed successfully.
    Completed,
    /// Flow failed with an error.
    Failed {
        error: String,
    },
}

/// Decision from a human at a gate.
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    /// Approve and continue.
    Approve,
    /// Reject and redraft — routes back to the same step.
    Reject {
        feedback: String,
    },
    /// Approve with edits — continues but includes edit notes in context.
    Edit {
        changes: String,
    },
}

/// Record of a completed step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step_id: String,
    pub profession_id: String,
    pub handoff: Option<HandoffDocument>,
    pub started_at: u64,
    pub completed_at: u64,
    pub iteration: u32,
}

/// The pipeline engine state machine.
#[derive(Debug, Clone)]
pub struct PipelineEngine {
    pub flow: FlowSpec,
    /// Index into flow.steps of the current (or next) step.
    pub current_step: usize,
    pub status: PipelineStatus,
    pub run_id: String,
    /// History of completed steps.
    pub step_history: Vec<StepRecord>,
    /// Loop iteration counters per step_id.
    pub loop_counters: HashMap<String, u32>,
    /// Pending human gate (if status is WaitingForHuman).
    pub pending_gate: Option<PendingGate>,
    /// Feedback from rejected gates, keyed by step_id.
    pub gate_feedback: HashMap<String, Vec<String>>,
    /// Tracks which step had its gate resolved for the current attempt.
    pub gate_resolved_for_step: Option<String>,
    /// Accumulated token usage across all steps.
    pub cumulative_tokens: u64,
    /// Budget tracker for runaway cost prevention and analytics.
    pub budget_tracker: BudgetTracker,
}

/// Current state of the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStatus {
    /// Flow loaded, ready to start.
    Idle,
    /// A step is currently executing.
    Running {
        step_id: String,
        profession_id: String,
        started_at: u64,
    },
    /// Paused waiting for human approval.
    WaitingForHuman {
        gate: GateType,
        step_id: String,
        since: u64,
    },
    /// All steps completed.
    Completed,
    /// Unrecoverable failure.
    Failed {
        error: String,
    },
    /// Explicitly paused (not via gate).
    Paused {
        at_step: usize,
    },
}

/// Information about a gate that is awaiting human resolution.
#[derive(Debug, Clone)]
pub struct PendingGate {
    pub step_id: String,
    pub gate: GateType,
    pub since: u64,
}

impl PipelineEngine {
    /// Create a new pipeline from a flow spec.
    pub fn new(flow: FlowSpec, run_id: impl Into<String>) -> Self {
        Self::with_budget(flow, run_id, TokenBudget::new(100_000))
    }

    /// Create a new pipeline with a custom run budget.
    pub fn with_budget(flow: FlowSpec, run_id: impl Into<String>, run_budget: TokenBudget) -> Self {
        Self {
            flow,
            current_step: 0,
            status: PipelineStatus::Idle,
            run_id: run_id.into(),
            step_history: Vec::new(),
            loop_counters: HashMap::new(),
            pending_gate: None,
            gate_feedback: HashMap::new(),
            gate_resolved_for_step: None,
            cumulative_tokens: 0,
            budget_tracker: BudgetTracker::new(run_budget),
        }
    }

    /// Advance the pipeline by one logical action.
    ///
    /// Returns what the caller should do next:
    /// - `ExecuteStep` → run the agent for this step, then call `submit_handoff()`
    /// - `WaitForHuman` → pause and wait for `resolve_gate()`
    /// - `Completed` or `Failed` → terminal states
    pub fn advance(&mut self) -> AdvanceResult {
        match &self.status {
            PipelineStatus::Completed => return AdvanceResult::Completed,
            PipelineStatus::Failed { error } => {
                return AdvanceResult::Failed { error: error.clone() };
            }
            PipelineStatus::WaitingForHuman { .. } => {
                return AdvanceResult::Failed {
                    error: "Cannot advance while waiting for human gate. Call resolve_gate() first.".into(),
                };
            }
            _ => {}
        }

        // Check if we've exhausted all steps
        if self.current_step >= self.flow.steps.len() {
            self.status = PipelineStatus::Completed;
            return AdvanceResult::Completed;
        }

        let step = &self.flow.steps[self.current_step];
        let now = now_secs();

        // Check gate
        if step.gate == GateType::Human && self.gate_resolved_for_step.as_ref() != Some(&step.id) {
            self.status = PipelineStatus::WaitingForHuman {
                gate: GateType::Human,
                step_id: step.id.clone(),
                since: now,
            };
            self.pending_gate = Some(PendingGate {
                step_id: step.id.clone(),
                gate: GateType::Human,
                since: now,
            });
            return AdvanceResult::WaitForHuman {
                gate: GateType::Human,
                step_id: step.id.clone(),
            };
        }

        // Transition to Running
        self.status = PipelineStatus::Running {
            step_id: step.id.clone(),
            profession_id: step.profession_id.clone(),
            started_at: now,
        };

        AdvanceResult::ExecuteStep {
            step_id: step.id.clone(),
            profession_id: step.profession_id.clone(),
        }
    }

    /// Submit the result of an agent turn to continue the pipeline.
    ///
    /// The handoff's `to` field and `routing_key` determine next routing.
    pub fn submit_handoff(&mut self, handoff: HandoffDocument) -> AdvanceResult {
        let now = now_secs();

        // Record the completed step
        let step_id = match &self.status {
            PipelineStatus::Running { step_id, .. } => step_id.clone(),
            _ => {
                self.status = PipelineStatus::Failed {
                    error: "submit_handoff called but no step is running".into(),
                };
                return self.advance();
            }
        };

        // Consume the gate resolution — next attempt at this step needs re-approval
        self.gate_resolved_for_step = None;

        let profession_id = self.flow.steps[self.current_step].profession_id.clone();

        self.step_history.push(StepRecord {
            step_id: step_id.clone(),
            profession_id: profession_id.clone(),
            handoff: Some(handoff.clone()),
            started_at: 0, // Would be captured in a real impl
            completed_at: now,
            iteration: *self.loop_counters.get(&step_id).unwrap_or(&0),
        });

        // Update cumulative tokens
        let step_tokens = handoff.token_usage.step_input + handoff.token_usage.step_output;
        self.cumulative_tokens += step_tokens;

        // Track in budget tracker
        self.budget_tracker.record(&profession_id, handoff.token_usage.step_input, handoff.token_usage.step_output);

        // Check budget enforcement
        match self.budget_tracker.check(&profession_id) {
            crate::relay::budget::BudgetAction::HardStop => {
                self.status = PipelineStatus::Failed {
                    error: format!("Budget exceeded: {} tokens spent vs {} limit", self.budget_tracker.cumulative, self.budget_tracker.run_budget.limit),
                };
                return AdvanceResult::Failed {
                    error: match &self.status {
                        PipelineStatus::Failed { error } => error.clone(),
                        _ => unreachable!(),
                    },
                };
            }
            _ => {} // Warning and None are non-fatal at this point
        }

        // Determine next step based on exit routing
        let step_id = self.flow.steps[self.current_step].id.clone();
        let exit = self.flow.steps[self.current_step].exit.clone();
        let next_index = self.resolve_next_step(&step_id, &exit, &handoff);

        match next_index {
            NextStep::Index(idx) => {
                self.current_step = idx;
                self.advance()
            }
            NextStep::Complete => {
                self.current_step = self.flow.steps.len();
                self.status = PipelineStatus::Completed;
                AdvanceResult::Completed
            }
            NextStep::Error(msg) => {
                self.status = PipelineStatus::Failed { error: msg };
                AdvanceResult::Failed {
                    error: match &self.status {
                        PipelineStatus::Failed { error } => error.clone(),
                        _ => unreachable!(),
                    },
                }
            }
        }
    }

    /// Resolve a human gate decision.
    pub fn resolve_gate(&mut self, decision: GateDecision) -> AdvanceResult {
        let pending = match self.pending_gate.take() {
            Some(g) => g,
            None => {
                return AdvanceResult::Failed {
                    error: "No pending gate to resolve".into(),
                };
            }
        };

        match decision {
            GateDecision::Approve | GateDecision::Edit { .. } => {
                // Mark gate as resolved for this step attempt
                self.gate_resolved_for_step = Some(pending.step_id.clone());
                self.status = PipelineStatus::Idle;
                self.advance()
            }
            GateDecision::Reject { feedback } => {
                // Store feedback and redraft: stay on same step
                self.gate_feedback
                    .entry(pending.step_id.clone())
                    .or_default()
                    .push(feedback);
                // Also mark resolved so we can re-enter the step
                self.gate_resolved_for_step = Some(pending.step_id.clone());
                self.status = PipelineStatus::Idle;
                self.advance()
            }
        }
    }

    /// Pause the pipeline at the current position.
    pub fn pause(&mut self) {
        if matches!(self.status, PipelineStatus::Running { .. }) {
            self.status = PipelineStatus::Paused {
                at_step: self.current_step,
            };
        }
    }

    /// Resume from a paused state.
    pub fn resume(&mut self) {
        if matches!(self.status, PipelineStatus::Paused { .. }) {
            self.status = PipelineStatus::Idle;
        }
    }

    /// Resolve the next step index from exit routing.
    fn resolve_next_step(&mut self, step_id: &str, exit: &ExitRouting, handoff: &HandoffDocument) -> NextStep {
        match exit {
            ExitRouting::Next => {
                let next = self.current_step + 1;
                if next >= self.flow.steps.len() {
                    NextStep::Complete
                } else {
                    NextStep::Index(next)
                }
            }
            ExitRouting::Branch { on, arms, default } => {
                let key = if on == "intent" {
                    handoff.to.clone()
                } else {
                    handoff.to.clone()
                };
                let target_id = arms.get(&key).unwrap_or(default);
                match self.flow.get_step_index(target_id) {
                    Some(idx) => NextStep::Index(idx),
                    None => NextStep::Error(format!("Branch target '{}' not found", target_id)),
                }
            }
            ExitRouting::Loop {
                target_step_id,
                max_iterations,
            } => {
                let count = self.loop_counters.entry(step_id.to_string()).or_insert(0);
                *count += 1;
                if *count >= *max_iterations {
                    // Break loop, go to next step
                    let next = self.current_step + 1;
                    if next >= self.flow.steps.len() {
                        NextStep::Complete
                    } else {
                        NextStep::Index(next)
                    }
                } else {
                    match self.flow.get_step_index(target_step_id) {
                        Some(idx) => NextStep::Index(idx),
                        None => NextStep::Error(format!(
                            "Loop target '{}' not found",
                            target_step_id
                        )),
                    }
                }
            }
        }
    }

    /// Convenience: which profession is currently/next expected.
    pub fn current_profession_id(&self) -> Option<&str> {
        self.flow.steps.get(self.current_step).map(|s| s.profession_id.as_str())
    }

    /// Convenience: current step ID.
    pub fn current_step_id(&self) -> Option<&str> {
        self.flow.steps.get(self.current_step).map(|s| s.id.as_str())
    }
}

/// Internal result of next-step resolution.
#[derive(Debug)]
enum NextStep {
    Index(usize),
    Complete,
    Error(String),
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::flow::FlowStep;
    use crate::relay::handoff::HandoffDocument;
    use std::collections::HashMap;

    fn make_handoff(from: &str, to: &str) -> HandoffDocument {
        HandoffDocument::new(from, to, "test-run", 1)
    }

    // ── S2.1: Sequential execution ───────────────────────────────────────────

    #[test]
    fn test_pipeline_executes_steps_in_order() {
        let mut flow = FlowSpec::new("test-seq");
        flow.add_step(FlowStep::new("s1", "planner"));
        flow.add_step(FlowStep::new("s2", "architect"));
        flow.add_step(FlowStep::new("s3", "coder"));

        let mut engine = PipelineEngine::new(flow, "run-1");

        // Step 1
        let r1 = engine.advance();
        assert_eq!(
            r1,
            AdvanceResult::ExecuteStep {
                step_id: "s1".into(),
                profession_id: "planner".into(),
            }
        );

        let h1 = make_handoff("planner", "architect");
        let r2 = engine.submit_handoff(h1);
        assert_eq!(
            r2,
            AdvanceResult::ExecuteStep {
                step_id: "s2".into(),
                profession_id: "architect".into(),
            }
        );

        // Step 2
        let h2 = make_handoff("architect", "coder");
        let r3 = engine.submit_handoff(h2);
        assert_eq!(
            r3,
            AdvanceResult::ExecuteStep {
                step_id: "s3".into(),
                profession_id: "coder".into(),
            }
        );

        // Step 3 → Complete
        let h3 = make_handoff("coder", "tester");
        let r4 = engine.submit_handoff(h3);
        assert_eq!(r4, AdvanceResult::Completed);
        assert_eq!(engine.status, PipelineStatus::Completed);
        assert_eq!(engine.step_history.len(), 3);
    }

    // ── S2.2: Branching routes DIRECT intent ─────────────────────────────────

    #[test]
    fn test_pipeline_branch_direct_skips_planner() {
        let mut flow = FlowSpec::new("test-branch");

        let mut arms = HashMap::new();
        arms.insert("coder".to_string(), "coder-step".to_string());
        arms.insert("planner".to_string(), "planner-step".to_string());

        flow.add_step(
            FlowStep::new("intaker-step", "intaker").with_exit(ExitRouting::Branch {
                on: "intent".to_string(),
                arms,
                default: "planner-step".to_string(),
            }),
        );
        flow.add_step(FlowStep::new("planner-step", "planner"));
        flow.add_step(FlowStep::new("coder-step", "coder"));

        let mut engine = PipelineEngine::new(flow, "run-2");

        // Intaker runs
        let r1 = engine.advance();
        assert_eq!(r1, AdvanceResult::ExecuteStep { step_id: "intaker-step".into(), profession_id: "intaker".into() });

        // Intaker classifies as DIRECT → handoff to coder
        let mut h = make_handoff("intaker", "coder");
        h.to = "coder".to_string();
        let r2 = engine.submit_handoff(h);
        assert_eq!(
            r2,
            AdvanceResult::ExecuteStep {
                step_id: "coder-step".into(),
                profession_id: "coder".into(),
            }
        );

        // Verify planner was skipped
        let professions_run: Vec<&str> = engine
            .step_history
            .iter()
            .map(|r| r.profession_id.as_str())
            .collect();
        assert_eq!(professions_run, vec!["intaker"]);

        // Finish coder
        let h2 = make_handoff("coder", "tester");
        let r3 = engine.submit_handoff(h2);
        assert_eq!(r3, AdvanceResult::Completed);
    }

    // ── S6.1: Human gate pauses until approval ───────────────────────────────

    #[test]
    fn test_human_gate_pauses_and_approves() {
        let mut flow = FlowSpec::new("test-gate");
        flow.add_step(FlowStep::new("s1", "planner").with_gate(GateType::Human));
        flow.add_step(FlowStep::new("s2", "architect"));

        let mut engine = PipelineEngine::new(flow, "run-gate");

        // First advance hits the gate
        let r1 = engine.advance();
        assert_eq!(
            r1,
            AdvanceResult::WaitForHuman {
                gate: GateType::Human,
                step_id: "s1".into(),
            }
        );
        assert!(matches!(engine.status, PipelineStatus::WaitingForHuman { .. }));

        // Cannot advance while waiting
        let r_err = engine.advance();
        assert!(matches!(r_err, AdvanceResult::Failed { .. }));

        // Approve the gate
        let r2 = engine.resolve_gate(GateDecision::Approve);
        assert_eq!(
            r2,
            AdvanceResult::ExecuteStep {
                step_id: "s1".into(),
                profession_id: "planner".into(),
            }
        );

        // Submit handoff → architect
        let h = make_handoff("planner", "architect");
        let r3 = engine.submit_handoff(h);
        assert_eq!(
            r3,
            AdvanceResult::ExecuteStep {
                step_id: "s2".into(),
                profession_id: "architect".into(),
            }
        );
    }

    #[test]
    fn test_human_gate_reject_redrafts() {
        let mut flow = FlowSpec::new("test-reject");
        flow.add_step(FlowStep::new("s1", "planner").with_gate(GateType::Human));

        let mut engine = PipelineEngine::new(flow, "run-reject");

        // Hit gate
        let _ = engine.advance();
        assert!(matches!(engine.status, PipelineStatus::WaitingForHuman { .. }));

        // Reject with feedback
        let r = engine.resolve_gate(GateDecision::Reject {
            feedback: "Need more detail on error handling".into(),
        });

        // Should re-enter the same step
        assert_eq!(
            r,
            AdvanceResult::ExecuteStep {
                step_id: "s1".into(),
                profession_id: "planner".into(),
            }
        );

        // Feedback stored
        assert_eq!(engine.gate_feedback.get("s1").unwrap().len(), 1);
    }

    // ── Budget enforcement ───────────────────────────────────────────────────

    #[test]
    fn test_budget_hardstop_prevents_runaway() {
        use crate::relay::budget::TokenBudget;
        use crate::relay::handoff::TokenUsage;

        let mut flow = FlowSpec::new("test-budget");
        flow.add_step(FlowStep::new("s1", "planner"));
        flow.add_step(FlowStep::new("s2", "architect"));

        // Tight budget: 500 tokens
        let mut engine = PipelineEngine::with_budget(flow, "run-budget", TokenBudget::new(500));

        // Step 1: 300 tokens — under budget
        let _ = engine.advance();
        let mut h1 = make_handoff("planner", "architect");
        h1.token_usage = TokenUsage { step_input: 200, step_output: 100, cumulative: 300, budget_remaining: 200 };
        let r1 = engine.submit_handoff(h1);
        assert!(matches!(r1, AdvanceResult::ExecuteStep { .. }));
        assert_eq!(engine.budget_tracker.cumulative, 300);

        // Step 2: 300 tokens — cumulative 600 > 500 limit → HardStop
        let _ = engine.advance();
        let mut h2 = make_handoff("architect", "coder");
        h2.token_usage = TokenUsage { step_input: 200, step_output: 100, cumulative: 600, budget_remaining: 0 };
        let r2 = engine.submit_handoff(h2);
        assert!(matches!(r2, AdvanceResult::Failed { .. }), "Expected budget hard-stop");
        assert!(matches!(engine.status, PipelineStatus::Failed { .. }));
    }

    #[test]
    fn test_budget_warning_non_fatal() {
        use crate::relay::budget::TokenBudget;
        use crate::relay::handoff::TokenUsage;

        let mut flow = FlowSpec::new("test-budget-warn");
        flow.add_step(FlowStep::new("s1", "planner"));

        // Budget 1000, warning at 700
        let mut engine = PipelineEngine::with_budget(flow, "run-warn", TokenBudget::new(1000));

        let _ = engine.advance();
        let mut h = make_handoff("planner", "done");
        // 800 tokens — above warning (700) but below limit (1000)
        h.token_usage = TokenUsage { step_input: 500, step_output: 300, cumulative: 800, budget_remaining: 200 };
        let r = engine.submit_handoff(h);
        // Should complete normally; warning is advisory
        assert_eq!(r, AdvanceResult::Completed);
    }

    // ── Loop routing ─────────────────────────────────────────────────────────

    #[test]
    fn test_loop_routing_bounded() {
        let mut flow = FlowSpec::new("test-loop");
        flow.add_step(
            FlowStep::new("s1", "tester").with_exit(ExitRouting::Loop {
                target_step_id: "s1".to_string(),
                max_iterations: 3,
            }),
        );
        flow.add_step(FlowStep::new("s2", "reviewer"));

        let mut engine = PipelineEngine::new(flow, "run-loop");

        // Iteration 1
        let _ = engine.advance();
        let h = make_handoff("tester", "tester");
        let r = engine.submit_handoff(h);
        assert_eq!(
            r,
            AdvanceResult::ExecuteStep {
                step_id: "s1".into(),
                profession_id: "tester".into(),
            }
        );
        assert_eq!(engine.loop_counters.get("s1"), Some(&1));

        // Iteration 2
        let h = make_handoff("tester", "tester");
        let r = engine.submit_handoff(h);
        assert_eq!(r, AdvanceResult::ExecuteStep { step_id: "s1".into(), profession_id: "tester".into() });
        assert_eq!(engine.loop_counters.get("s1"), Some(&2));

        // Iteration 3 → break to reviewer
        let h = make_handoff("tester", "tester");
        let r = engine.submit_handoff(h);
        assert_eq!(
            r,
            AdvanceResult::ExecuteStep {
                step_id: "s2".into(),
                profession_id: "reviewer".into(),
            }
        );
        assert_eq!(engine.loop_counters.get("s1"), Some(&3));
    }

    // ── Edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_empty_flow_completes_immediately() {
        let flow = FlowSpec::new("empty");
        let mut engine = PipelineEngine::new(flow, "run-empty");
        let r = engine.advance();
        assert_eq!(r, AdvanceResult::Completed);
    }

    #[test]
    fn test_completed_engine_stays_completed() {
        let mut flow = FlowSpec::new("tiny");
        flow.add_step(FlowStep::new("s1", "intaker"));
        let mut engine = PipelineEngine::new(flow, "run");

        let _ = engine.advance();
        let _ = engine.submit_handoff(make_handoff("intaker", "done"));
        assert_eq!(engine.status, PipelineStatus::Completed);

        let r = engine.advance();
        assert_eq!(r, AdvanceResult::Completed);
    }
}
