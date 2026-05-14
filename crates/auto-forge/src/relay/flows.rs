//! Built-in Flow Specifications
//!
//! Pre-defined flow specs for common development workflows.

use crate::relay::flow::{FlowSpec, FlowStep, GateType};

/// The standard spec-driven development flow (v2).
///
/// Assistant → Advisor → Architect → Planner → Tester → Coder → Tester → Reviewer → Documenter
///
/// With human gate at Advisor→Architect boundary (GoalGate).
/// In GSD mode, only the Advisor gate pauses. In Check mode, all gates pause.
pub fn standard_spec_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("standard-spec-driven-development");
    flow.add_step(FlowStep::new("intake", "assistant"));
    flow.add_step(
        FlowStep::new("discover", "advisor")
            .with_gate(GateType::Human),
    );
    flow.add_step(FlowStep::new("design", "architect"));
    flow.add_step(FlowStep::new("plan", "planner"));
    flow.add_step(FlowStep::new("draft-tests", "tester"));
    flow.add_step(FlowStep::new("code", "coder"));
    flow.add_step(
        FlowStep::new("run-tests", "tester")
            .with_exit(crate::relay::flow::ExitRouting::Loop {
                target_step_id: "code".into(),
                max_iterations: 3,
            }),
    );
    flow.add_step(FlowStep::new("review", "reviewer"));
    flow.add_step(FlowStep::new("report", "documenter"));
    flow
}

/// A fast-track flow for small, well-understood tasks.
///
/// Assistant classifies as DIRECT → Coder only.
/// Falls back to full flow if classification is COMPLEX.
pub fn fast_track_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("fast-track");
    flow.add_step(
        FlowStep::new("intake", "assistant"),
    );
    flow.add_step(FlowStep::new("code", "coder"));
    flow
}

/// A bug-fix flow with tester-review loop.
///
/// Coder → Tester → Reviewer, with loop back to Coder if tests fail.
pub fn bug_fix_flow() -> FlowSpec {
    let mut flow = FlowSpec::new("bug-fix");
    flow.add_step(FlowStep::new("intake", "assistant"));
    flow.add_step(FlowStep::new("code", "coder"));
    flow.add_step(
        FlowStep::new("test", "tester")
            .with_exit(crate::relay::flow::ExitRouting::Loop {
                target_step_id: "code".into(),
                max_iterations: 3,
            }),
    );
    flow.add_step(FlowStep::new("review", "reviewer"));
    flow
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::flow::GateType;

    #[test]
    fn test_standard_flow_has_nine_steps() {
        let flow = standard_spec_flow();
        assert_eq!(flow.steps.len(), 9);
        assert_eq!(flow.steps[0].profession_id, "assistant");
        assert_eq!(flow.steps[1].profession_id, "advisor");
        assert_eq!(flow.steps[2].profession_id, "architect");
        assert_eq!(flow.steps[3].profession_id, "planner");
        assert_eq!(flow.steps[4].profession_id, "tester");
        assert_eq!(flow.steps[5].profession_id, "coder");
        assert_eq!(flow.steps[6].profession_id, "tester");
        assert_eq!(flow.steps[7].profession_id, "reviewer");
        assert_eq!(flow.steps[8].profession_id, "documenter");
    }

    #[test]
    fn test_standard_flow_has_human_gate_at_advisor() {
        let flow = standard_spec_flow();
        assert_eq!(flow.steps[1].gate, GateType::Human); // advisor → architect (GoalGate)
        assert_eq!(flow.steps[2].gate, GateType::Auto);  // architect → planner
        assert_eq!(flow.steps[3].gate, GateType::Auto);  // planner → tester
        assert_eq!(flow.steps[4].gate, GateType::Auto);  // tester → coder
    }

    #[test]
    fn test_fast_track_flow_has_two_steps() {
        let flow = fast_track_flow();
        assert_eq!(flow.steps.len(), 2);
        assert_eq!(flow.steps[0].profession_id, "assistant");
        assert_eq!(flow.steps[1].profession_id, "coder");
    }

    #[test]
    fn test_bug_fix_flow_has_loop() {
        let flow = bug_fix_flow();
        assert_eq!(flow.steps.len(), 4);
        match &flow.steps[2].exit {
            crate::relay::flow::ExitRouting::Loop { target_step_id, max_iterations } => {
                assert_eq!(target_step_id, "code");
                assert_eq!(*max_iterations, 3);
            }
            _ => panic!("Expected Loop exit on tester step"),
        }
    }
}
