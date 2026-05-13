//! Integration test: End-to-end Agents Relay flow
//!
//! P6.1: Intake → Planner → Architect → Coder → Tester → Reviewer


use auto_forge::relay::handoff::{HandoffDocument, TokenUsage};
use auto_forge::relay::pipeline::{AdvanceResult, GateDecision, PipelineStatus};
use auto_forge::relay::store::{advance_run, get_run, new_run_store, resolve_gate, start_run, submit_handoff};
use auto_forge::relay::flows::standard_spec_flow;

#[test]
fn test_end_to_end_standard_flow_with_mock_handoffs() {
    let store = new_run_store();
    let flow = standard_spec_flow();

    // ── Start the run ───────────────────────────────────────────────────────
    let run_id = "e2e-standard-1";
    start_run(&store, flow, run_id).expect("start run");

    // Verify initial state
    let state = get_run(&store, run_id).unwrap();
    assert_eq!(state.total_steps, 6);
    assert_eq!(state.status, "Idle");

    // ── Step 1: Intaker ─────────────────────────────────────────────────────
    let r1 = advance_run(&store, run_id).unwrap();
    assert!(matches!(r1, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "intaker"));

    let mut h1 = HandoffDocument::new("intaker", "planner", run_id, 0);
    h1.summary = "Classified as COMPLEX feature request.".into();
    h1.token_usage = TokenUsage { step_input: 500, step_output: 300, cumulative: 800, budget_remaining: 99200 };
    let r1b = submit_handoff(&store, run_id, h1).unwrap();
    assert!(matches!(r1b, AdvanceResult::WaitForHuman { .. })); // planner has human gate

    let state1 = get_run(&store, run_id).unwrap();
    assert_eq!(state1.steps[0].status, "completed");
    assert_eq!(state1.steps[1].status, "waiting_gate");
    assert_eq!(state1.cumulative_tokens, 800);

    // ── Resolve gate: Approve planner ───────────────────────────────────────
    let r_gate = resolve_gate(&store, run_id, GateDecision::Approve).unwrap();
    assert!(matches!(r_gate, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "planner"));

    // ── Step 2: Planner ─────────────────────────────────────────────────────
    let mut h2 = HandoffDocument::new("planner", "architect", run_id, 1);
    h2.summary = "Decomposed into 3 sub-tasks.".into();
    h2.token_usage = TokenUsage { step_input: 1200, step_output: 800, cumulative: 2000, budget_remaining: 98000 };
    let r2 = submit_handoff(&store, run_id, h2).unwrap();
    assert!(matches!(r2, AdvanceResult::WaitForHuman { .. })); // architect has human gate

    let state2 = get_run(&store, run_id).unwrap();
    assert_eq!(state2.steps[1].status, "completed");
    assert_eq!(state2.steps[2].status, "waiting_gate");
    assert_eq!(state2.cumulative_tokens, 2800);

    // ── Resolve gate: Approve architect ─────────────────────────────────────
    resolve_gate(&store, run_id, GateDecision::Approve);

    // ── Step 3: Architect ───────────────────────────────────────────────────
    let mut h3 = HandoffDocument::new("architect", "coder", run_id, 2);
    h3.summary = "Designed auth module with 3 components.".into();
    h3.token_usage = TokenUsage { step_input: 1500, step_output: 1000, cumulative: 2500, budget_remaining: 97500 };
    let r3 = submit_handoff(&store, run_id, h3).unwrap();
    assert!(matches!(r3, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "coder"));

    // ── Step 4: Coder ───────────────────────────────────────────────────────
    let mut h4 = HandoffDocument::new("coder", "tester", run_id, 3);
    h4.summary = "Implemented auth module.".into();
    h4.token_usage = TokenUsage { step_input: 3000, step_output: 2500, cumulative: 5500, budget_remaining: 94500 };
    let r4 = submit_handoff(&store, run_id, h4).unwrap();
    assert!(matches!(r4, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "tester"));

    // ── Step 5: Tester ──────────────────────────────────────────────────────
    let mut h5 = HandoffDocument::new("tester", "reviewer", run_id, 4);
    h5.summary = "All tests pass.".into();
    h5.token_usage = TokenUsage { step_input: 2000, step_output: 1500, cumulative: 3500, budget_remaining: 96500 };
    let r5 = submit_handoff(&store, run_id, h5).unwrap();
    assert!(matches!(r5, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "reviewer"));

    // ── Step 6: Reviewer ────────────────────────────────────────────────────
    let mut h6 = HandoffDocument::new("reviewer", "done", run_id, 5);
    h6.summary = "Code reviewed and approved.".into();
    h6.token_usage = TokenUsage { step_input: 1500, step_output: 1000, cumulative: 2500, budget_remaining: 97500 };
    let r6 = submit_handoff(&store, run_id, h6).unwrap();
    assert_eq!(r6, AdvanceResult::Completed);

    // ── Final state ─────────────────────────────────────────────────────────
    let final_state = get_run(&store, run_id).unwrap();
    assert_eq!(final_state.status, "Completed");
    assert_eq!(final_state.current_step, 6);
    assert_eq!(final_state.step_history.len(), 6);

    // All steps completed
    for step in &final_state.steps {
        assert_eq!(step.status, "completed", "step {} should be completed", step.id);
    }

    // Budget tracked across all steps
    let expected_total = 800 + 2000 + 2500 + 5500 + 3500 + 2500;
    assert_eq!(final_state.cumulative_tokens, expected_total);

    // Savings vs parallel should be positive
    assert!(final_state.savings > 0);
    assert!(final_state.savings_ratio > 0.0);
}

#[test]
fn test_end_to_end_reject_gate_routes_back() {
    let store = new_run_store();
    let flow = standard_spec_flow();

    let run_id = "e2e-reject-1";
    start_run(&store, flow, run_id).unwrap();

    // Intaker → Planner gate
    advance_run(&store, run_id);
    let h = HandoffDocument::new("intaker", "planner", run_id, 0);
    submit_handoff(&store, run_id, h);

    // Reject planner gate
    let r = resolve_gate(&store, run_id, GateDecision::Reject {
        feedback: "Need more detail on error handling".into(),
    }).unwrap();

    // Should re-enter planner step
    assert!(matches!(r, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "planner"));

    let state = get_run(&store, run_id).unwrap();
    assert_eq!(state.current_step, 1); // still on planner
    assert_eq!(state.steps[1].status, "running");
}

#[test]
fn test_checkpoint_during_flow() {
    use auto_forge::relay::checkpoint::Checkpoint;

    let store = new_run_store();
    let flow = standard_spec_flow();

    let run_id = "e2e-checkpoint-1";
    start_run(&store, flow.clone(), run_id).unwrap();

    // Run through 3 steps
    advance_run(&store, run_id);
    submit_handoff(&store, run_id, HandoffDocument::new("intaker", "planner", run_id, 0));
    resolve_gate(&store, run_id, GateDecision::Approve);

    advance_run(&store, run_id);
    submit_handoff(&store, run_id, HandoffDocument::new("planner", "architect", run_id, 1));
    resolve_gate(&store, run_id, GateDecision::Approve);

    advance_run(&store, run_id);
    submit_handoff(&store, run_id, HandoffDocument::new("architect", "coder", run_id, 2));

    // Create checkpoint
    let map = store.lock().unwrap();
    let entry = map.get(run_id).unwrap();
    let checkpoint = Checkpoint::create(&entry.engine, std::path::Path::new("."), None).unwrap();
    drop(map);

    assert_eq!(checkpoint.run_id, run_id);
    assert_eq!(checkpoint.current_step, 3); // next is coder
    assert_eq!(checkpoint.step_history.len(), 3);

    // Resume from checkpoint
    let mut resumed = auto_forge::relay::pipeline::PipelineEngine::from_checkpoint(checkpoint, flow).unwrap();
    assert_eq!(resumed.current_step, 3);
    assert_eq!(resumed.status, PipelineStatus::Idle);
    assert_eq!(resumed.step_history.len(), 3);

    // Can continue
    let r = resumed.advance();
    assert!(matches!(r, AdvanceResult::ExecuteStep { profession_id, .. } if profession_id == "coder"));
}
