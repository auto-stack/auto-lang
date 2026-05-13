//! Flow Specification
//!
//! Declarative flow definitions that the PipelineEngine executes.
//! Flows are deterministic — the orchestrator is pure code, not an LLM.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A flow is an ordered list of steps with routing logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSpec {
    pub id: String,
    pub steps: Vec<FlowStep>,
}

impl FlowSpec {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            steps: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: FlowStep) -> &mut Self {
        self.steps.push(step);
        self
    }

    pub fn get_step(&self, step_id: &str) -> Option<&FlowStep> {
        self.steps.iter().find(|s| s.id == step_id)
    }

    pub fn get_step_index(&self, step_id: &str) -> Option<usize> {
        self.steps.iter().position(|s| s.id == step_id)
    }

    /// Resolve a profession_id to the first step that uses it.
    pub fn step_for_profession(&self, profession_id: &str) -> Option<usize> {
        self.steps.iter().position(|s| s.profession_id == profession_id)
    }
}

/// A single step in a flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub id: String,
    pub profession_id: String,
    pub gate: GateType,
    /// Max LLM turns before forced handoff (overrides profession default).
    pub max_turns: Option<u32>,
    /// How to route after this step completes.
    pub exit: ExitRouting,
}

impl FlowStep {
    pub fn new(id: impl Into<String>, profession_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            profession_id: profession_id.into(),
            gate: GateType::Auto,
            max_turns: None,
            exit: ExitRouting::Next,
        }
    }

    pub fn with_gate(mut self, gate: GateType) -> Self {
        self.gate = gate;
        self
    }

    pub fn with_exit(mut self, exit: ExitRouting) -> Self {
        self.exit = exit;
        self
    }
}

/// Gate type controlling whether a step needs human approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    /// Proceed automatically.
    Auto,
    /// Pause for human approval before executing.
    Human,
}

/// Routing logic after a step completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitRouting {
    /// Go to the next step in sequence.
    Next,
    /// Branch based on a field in the handoff.
    Branch {
        /// Name of the handoff field to branch on (e.g., "intent").
        on: String,
        /// Map of field value → target step_id.
        arms: HashMap<String, String>,
        /// Fallback step_id if no arm matches.
        default: String,
    },
    /// Loop back to a target step.
    Loop {
        /// Step to return to.
        target_step_id: String,
        /// Max iterations before breaking to next.
        max_iterations: u32,
    },
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_spec_builder() {
        let mut flow = FlowSpec::new("test-flow");
        flow.add_step(FlowStep::new("step-1", "planner"));
        flow.add_step(FlowStep::new("step-2", "architect"));

        assert_eq!(flow.steps.len(), 2);
        assert_eq!(flow.get_step("step-1").unwrap().profession_id, "planner");
        assert_eq!(flow.get_step_index("step-2"), Some(1));
    }

    #[test]
    fn test_branch_routing() {
        let mut arms = HashMap::new();
        arms.insert("DIRECT".to_string(), "coder-step".to_string());
        arms.insert("COMPLEX".to_string(), "planner-step".to_string());

        let exit = ExitRouting::Branch {
            on: "intent".to_string(),
            arms,
            default: "planner-step".to_string(),
        };

        match exit {
            ExitRouting::Branch { on, arms, default } => {
                assert_eq!(on, "intent");
                assert_eq!(arms.get("DIRECT"), Some(&"coder-step".to_string()));
                assert_eq!(default, "planner-step");
            }
            _ => panic!("Expected Branch"),
        }
    }

    #[test]
    fn test_loop_routing() {
        let exit = ExitRouting::Loop {
            target_step_id: "step-1".to_string(),
            max_iterations: 3,
        };

        match exit {
            ExitRouting::Loop { target_step_id, max_iterations } => {
                assert_eq!(target_step_id, "step-1");
                assert_eq!(max_iterations, 3);
            }
            _ => panic!("Expected Loop"),
        }
    }
}
