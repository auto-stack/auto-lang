//! Relay Run Store
//!
//! In-memory store for active and completed pipeline runs.
//! Provides the bridge between the deterministic PipelineEngine and HTTP APIs.

use crate::relay::flow::FlowSpec;
use crate::relay::handoff::HandoffDocument;
use crate::relay::pipeline::{AdvanceResult, GateDecision, PipelineEngine, PipelineStatus, StepRecord};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Shared in-memory store for all relay runs.
pub type RunStore = Arc<Mutex<HashMap<String, RunEntry>>>;

/// An entry in the run store.
#[derive(Debug, Clone)]
pub struct RunEntry {
    pub run_id: String,
    pub engine: PipelineEngine,
    pub created_at: u64,
    pub updated_at: u64,
    /// Serialized events for SSE replay.
    pub events: Vec<RunEvent>,
}

/// A run event for SSE streaming and history.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEvent {
    StepStarted { step_id: String, profession_id: String },
    StepCompleted { step_id: String, handoff_summary: String },
    GateWaiting { step_id: String, gate: String },
    GateResolved { step_id: String, decision: String },
    RunCompleted,
    RunFailed { error: String },
    TokenSpend { cumulative: u64, step_tokens: u64 },
}

/// Summary of a run for listing.
#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub status: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub current_profession: Option<String>,
    pub cumulative_tokens: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Detailed run state for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct RunState {
    pub run_id: String,
    pub status: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub steps: Vec<StepState>,
    pub step_history: Vec<StepRecord>,
    pub cumulative_tokens: u64,
    pub budget_limit: u64,
    pub budget_remaining: u64,
    pub waiting_for_gate: Option<GateState>,
    pub parallel_estimate: u64,
    pub savings: u64,
    pub savings_ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepState {
    pub id: String,
    pub profession_id: String,
    pub status: String, // "pending", "running", "completed", "failed"
    pub gate: String,   // "auto", "human"
}

#[derive(Debug, Clone, Serialize)]
pub struct GateState {
    pub step_id: String,
    pub profession_id: String,
    pub since: u64,
}

/// Create a new shared run store.
pub fn new_run_store() -> RunStore {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Start a new run with the given flow spec.
pub fn start_run(store: &RunStore, flow: FlowSpec, run_id: impl Into<String>) -> Result<RunState, String> {
    let run_id = run_id.into();
    let mut map = store.lock().unwrap();
    if map.contains_key(&run_id) {
        return Err(format!("Run {} already exists", run_id));
    }

    let engine = PipelineEngine::new(flow.clone(), &run_id);
    let now = now_secs();
    let entry = RunEntry {
        run_id: run_id.clone(),
        engine,
        created_at: now,
        updated_at: now,
        events: Vec::new(),
    };

    let state = build_run_state(&entry);
    map.insert(run_id, entry);
    Ok(state)
}

/// Get the current state of a run.
pub fn get_run(store: &RunStore, run_id: &str) -> Option<RunState> {
    let map = store.lock().unwrap();
    map.get(run_id).map(build_run_state)
}

/// List all runs.
pub fn list_runs(store: &RunStore) -> Vec<RunSummary> {
    let map = store.lock().unwrap();
    map.values().map(|e| RunSummary {
        run_id: e.run_id.clone(),
        status: format!("{:?}", e.engine.status),
        current_step: e.engine.current_step,
        total_steps: e.engine.flow.steps.len(),
        current_profession: e.engine.current_profession_id().map(|s| s.to_string()),
        cumulative_tokens: e.engine.cumulative_tokens,
        created_at: e.created_at,
        updated_at: e.updated_at,
    }).collect()
}

/// Advance a run by one step.
pub fn advance_run(store: &RunStore, run_id: &str) -> Option<AdvanceResult> {
    let mut map = store.lock().unwrap();
    let entry = map.get_mut(run_id)?;
    let result = entry.engine.advance();
    entry.updated_at = now_secs();

    match &result {
        AdvanceResult::ExecuteStep { step_id, profession_id } => {
            entry.events.push(RunEvent::StepStarted {
                step_id: step_id.clone(),
                profession_id: profession_id.clone(),
            });
        }
        AdvanceResult::WaitForHuman { step_id, .. } => {
            entry.events.push(RunEvent::GateWaiting {
                step_id: step_id.clone(),
                gate: "human".into(),
            });
        }
        AdvanceResult::Completed => {
            entry.events.push(RunEvent::RunCompleted);
        }
        AdvanceResult::Failed { error } => {
            entry.events.push(RunEvent::RunFailed { error: error.clone() });
        }
    }

    Some(result.clone())
}

/// Submit a handoff for the current step.
pub fn submit_handoff(store: &RunStore, run_id: &str, handoff: HandoffDocument) -> Option<AdvanceResult> {
    let mut map = store.lock().unwrap();
    let entry = map.get_mut(run_id)?;
    let result = entry.engine.submit_handoff(handoff.clone());
    entry.updated_at = now_secs();

    let step_tokens = handoff.token_usage.step_input + handoff.token_usage.step_output;
    entry.events.push(RunEvent::TokenSpend {
        cumulative: entry.engine.cumulative_tokens,
        step_tokens,
    });

    match &result {
        AdvanceResult::ExecuteStep { step_id, .. } => {
            entry.events.push(RunEvent::StepCompleted {
                step_id: step_id.clone(),
                handoff_summary: handoff.summary.clone(),
            });
        }
        AdvanceResult::Completed => {
            entry.events.push(RunEvent::RunCompleted);
        }
        AdvanceResult::Failed { error } => {
            entry.events.push(RunEvent::RunFailed { error: error.clone() });
        }
        _ => {}
    }

    Some(result.clone())
}

/// Resolve a human gate for a run.
pub fn resolve_gate(store: &RunStore, run_id: &str, decision: GateDecision) -> Option<AdvanceResult> {
    let mut map = store.lock().unwrap();
    let entry = map.get_mut(run_id)?;
    let result = entry.engine.resolve_gate(decision.clone());
    entry.updated_at = now_secs();

    let decision_str = match decision {
        GateDecision::Approve => "approve",
        GateDecision::Reject { .. } => "reject",
        GateDecision::Edit { .. } => "edit",
    };

    if let Some(step_id) = entry.engine.current_step_id() {
        entry.events.push(RunEvent::GateResolved {
            step_id: step_id.to_string(),
            decision: decision_str.into(),
        });
    }

    Some(result.clone())
}

/// Build a RunState from a RunEntry.
fn build_run_state(entry: &RunEntry) -> RunState {
    let engine = &entry.engine;
    let steps: Vec<StepState> = engine.flow.steps.iter().enumerate().map(|(idx, step)| {
        let status = if idx < engine.current_step {
            "completed"
        } else if idx == engine.current_step && matches!(engine.status, PipelineStatus::Running { .. }) {
            "running"
        } else if idx == engine.current_step && matches!(engine.status, PipelineStatus::WaitingForHuman { .. }) {
            "waiting_gate"
        } else {
            "pending"
        };
        StepState {
            id: step.id.clone(),
            profession_id: step.profession_id.clone(),
            status: status.into(),
            gate: match step.gate {
                crate::relay::flow::GateType::Auto => "auto",
                crate::relay::flow::GateType::Human => "human",
            }.into(),
        }
    }).collect();

    let waiting_for_gate = if let PipelineStatus::WaitingForHuman { step_id, since, .. } = &engine.status {
        engine.flow.get_step(step_id).map(|step| GateState {
            step_id: step_id.clone(),
            profession_id: step.profession_id.clone(),
            since: *since,
        })
    } else {
        None
    };

    let (savings, savings_ratio) = engine.budget_tracker.savings_vs_parallel(
        engine.flow.steps.len() as u32,
        5000, // avg_context heuristic
        3,    // rounds heuristic
    );

    RunState {
        run_id: entry.run_id.clone(),
        status: format!("{:?}", engine.status),
        current_step: engine.current_step,
        total_steps: engine.flow.steps.len(),
        steps,
        step_history: engine.step_history.clone(),
        cumulative_tokens: engine.cumulative_tokens,
        budget_limit: engine.budget_tracker.run_budget.limit,
        budget_remaining: engine.budget_tracker.run_budget.limit.saturating_sub(engine.budget_tracker.cumulative),
        waiting_for_gate,
        parallel_estimate: engine.budget_tracker.estimate_parallel_cost(engine.flow.steps.len() as u32, 5000, 3),
        savings,
        savings_ratio,
    }
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
    use crate::relay::flow::{FlowSpec, FlowStep, GateType};
    use crate::relay::handoff::{HandoffDocument, TokenUsage};

    #[test]
    fn test_run_store_start_and_get() {
        let store = new_run_store();
        let mut flow = FlowSpec::new("test");
        flow.add_step(FlowStep::new("s1", "planner"));
        flow.add_step(FlowStep::new("s2", "coder"));

        let state = start_run(&store, flow, "run-1").unwrap();
        assert_eq!(state.run_id, "run-1");
        assert_eq!(state.total_steps, 2);
        assert_eq!(state.status, "Idle");

        let fetched = get_run(&store, "run-1").unwrap();
        assert_eq!(fetched.run_id, "run-1");
    }

    #[test]
    fn test_run_store_advance_and_handoff() {
        let store = new_run_store();
        let mut flow = FlowSpec::new("test");
        flow.add_step(FlowStep::new("s1", "planner"));

        start_run(&store, flow, "run-1").unwrap();

        let r = advance_run(&store, "run-1").unwrap();
        assert!(matches!(r, AdvanceResult::ExecuteStep { .. }));

        let h = HandoffDocument::new("planner", "done", "run-1", 0);
        let r2 = submit_handoff(&store, "run-1", h).unwrap();
        assert_eq!(r2, AdvanceResult::Completed);

        let state = get_run(&store, "run-1").unwrap();
        assert_eq!(state.status, "Completed");
        assert_eq!(state.current_step, 1);
    }

    #[test]
    fn test_run_store_gate_waiting() {
        let store = new_run_store();
        let mut flow = FlowSpec::new("test");
        flow.add_step(FlowStep::new("s1", "advisor").with_gate(GateType::Human));

        start_run(&store, flow, "run-gate").unwrap();
        let r = advance_run(&store, "run-gate").unwrap();
        assert!(matches!(r, AdvanceResult::WaitForHuman { .. }));

        let state = get_run(&store, "run-gate").unwrap();
        assert!(state.waiting_for_gate.is_some());
        assert_eq!(state.steps[0].status, "waiting_gate");

        // Resolve gate
        let r2 = resolve_gate(&store, "run-gate", GateDecision::Approve).unwrap();
        assert!(matches!(r2, AdvanceResult::ExecuteStep { .. }));

        let state2 = get_run(&store, "run-gate").unwrap();
        assert!(state2.waiting_for_gate.is_none());
    }

    #[test]
    fn test_run_store_budget_tracking() {
        let store = new_run_store();
        let mut flow = FlowSpec::new("test");
        flow.add_step(FlowStep::new("s1", "planner"));

        start_run(&store, flow, "run-budget").unwrap();
        advance_run(&store, "run-budget");

        let mut h = HandoffDocument::new("planner", "done", "run-budget", 0);
        h.token_usage = TokenUsage { step_input: 1000, step_output: 500, cumulative: 1500, budget_remaining: 98500 };
        submit_handoff(&store, "run-budget", h);

        let state = get_run(&store, "run-budget").unwrap();
        assert_eq!(state.cumulative_tokens, 1500);
        assert_eq!(state.budget_limit, 100_000);
        assert_eq!(state.budget_remaining, 100_000 - 1500);
    }
}
