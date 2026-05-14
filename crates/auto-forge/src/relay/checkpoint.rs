//! Checkpoint & Durability
//!
//! Enables long-running flows to survive crashes and resume without context loss.
//! Every handoff produces a checkpoint containing: git state, file manifest,
//! handoff document, and pipeline state. On resume, the next agent starts fresh
//! from the Handoff Document — token cost is identical to continuous execution.

use crate::forge::SpecsDocument;
use crate::relay::budget::BudgetTracker;
use crate::relay::flow::FlowSpec;
use crate::relay::handoff::{HandoffDocument, TokenUsage};
use crate::relay::pipeline::{PipelineEngine, PipelineStatus, StepRecord};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A snapshot of pipeline state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: u64,
    pub run_id: String,
    pub timestamp: u64,
    /// Git commit SHA if the working tree was clean.
    pub git_commit: Option<String>,
    /// Git diff patch if the working tree was dirty.
    pub git_diff: Option<String>,
    /// Deep copy of the ledger (specs) at this point.
    pub ledger_state: Option<SpecsDocument>,
    /// The handoff that triggered this checkpoint.
    pub handoff: HandoffDocument,
    /// Snapshot of every touched file at this point.
    pub file_manifest: Vec<FileState>,
    /// Token usage up to this checkpoint.
    pub token_usage: TokenUsage,
    /// Which step index to resume at (next step after the completed one).
    pub current_step: usize,
    /// History of completed steps up to this point.
    pub step_history: Vec<StepRecord>,
    /// Loop iteration counters.
    pub loop_counters: HashMap<String, u32>,
    /// Cumulative tokens spent.
    pub cumulative_tokens: u64,
    /// Gate feedback accumulated.
    pub gate_feedback: HashMap<String, Vec<String>>,
    /// Budget tracker state for cost analytics persistence.
    pub budget_tracker: BudgetTracker,
    /// Execution mode (GSD or Check) for gate behavior on resume.
    pub mode: crate::relay::pipeline::RelayMode,
}

/// State of a single file at checkpoint time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub path: String,
    /// SHA-256 hex digest of file content.
    pub hash: String,
    pub size: u64,
    /// Inline file content for restore (simplifies self-contained checkpoints).
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum CheckpointError {
    Io(String),
    Serialize(String),
    Git(String),
    Restore(String),
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointError::Io(s) => write!(f, "IO error: {}", s),
            CheckpointError::Serialize(s) => write!(f, "Serialize error: {}", s),
            CheckpointError::Git(s) => write!(f, "Git error: {}", s),
            CheckpointError::Restore(s) => write!(f, "Restore error: {}", s),
        }
    }
}

impl std::error::Error for CheckpointError {}

impl Checkpoint {
    /// Create a checkpoint from the current pipeline state.
    pub fn create(
        engine: &PipelineEngine,
        project_dir: &Path,
        ledger_state: Option<SpecsDocument>,
    ) -> Result<Self, CheckpointError> {
        let now = now_secs();
        let checkpoint_id = engine.step_history.len() as u64;

        // Git snapshot
        let (git_commit, git_diff) = git_snapshot(project_dir);

        // File manifest from files touched in the most recent step
        let file_manifest = build_file_manifest(project_dir, engine);

        // Handoff from the last completed step, or empty if none
        let handoff = engine
            .step_history
            .last()
            .and_then(|r| r.handoff.clone())
            .unwrap_or_else(|| HandoffDocument::new("", "", &engine.run_id, checkpoint_id));

        let token_usage = engine
            .step_history
            .last()
            .and_then(|r| r.handoff.as_ref())
            .map(|h| h.token_usage.clone())
            .unwrap_or_default();

        Ok(Self {
            id: checkpoint_id,
            run_id: engine.run_id.clone(),
            timestamp: now,
            git_commit,
            git_diff,
            ledger_state,
            handoff,
            file_manifest,
            token_usage,
            current_step: engine.current_step,
            step_history: engine.step_history.clone(),
            loop_counters: engine.loop_counters.clone(),
            cumulative_tokens: engine.cumulative_tokens,
            gate_feedback: engine.gate_feedback.clone(),
            budget_tracker: engine.budget_tracker.clone(),
            mode: engine.mode,
        })
    }

    /// Serialize checkpoint to disk as JSON.
    pub fn save(&self, dir: &Path) -> Result<PathBuf, CheckpointError> {
        std::fs::create_dir_all(dir)
            .map_err(|e| CheckpointError::Io(format!("create_dir_all: {}", e)))?;

        let path = dir.join(format!("checkpoint-{}.json", self.id));
        let json =
            serde_json::to_string_pretty(self).map_err(|e| CheckpointError::Serialize(e.to_string()))?;
        std::fs::write(&path, json).map_err(|e| CheckpointError::Io(e.to_string()))?;

        Ok(path)
    }

    /// Load a checkpoint from disk.
    pub fn load(path: &Path) -> Result<Self, CheckpointError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CheckpointError::Io(format!("read {}: {}", path.display(), e)))?;
        serde_json::from_str(&content)
            .map_err(|e| CheckpointError::Serialize(format!("parse {}: {}", path.display(), e)))
    }

    /// Restore file system state from this checkpoint.
    pub fn restore_files(&self, project_dir: &Path) -> Result<(), CheckpointError> {
        for file in &self.file_manifest {
            let target = project_dir.join(&file.path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CheckpointError::Restore(format!("mkdir {}: {}", parent.display(), e)))?;
            }
            std::fs::write(&target, &file.content)
                .map_err(|e| CheckpointError::Restore(format!("write {}: {}", target.display(), e)))?;
        }
        Ok(())
    }

    /// Compute a hash of the checkpoint for integrity verification.
    pub fn integrity_hash(&self) -> String {
        let json = serde_json::to_vec(self).unwrap_or_default();
        hex_hash(&json)
    }
}

// ─── PipelineEngine integration ──────────────────────────────────────────────

impl PipelineEngine {
    /// Save a checkpoint of the current pipeline state.
    pub fn save_checkpoint(
        &self,
        checkpoints_dir: &Path,
        project_dir: &Path,
        ledger_state: Option<SpecsDocument>,
    ) -> Result<Checkpoint, CheckpointError> {
        let checkpoint = Checkpoint::create(self, project_dir, ledger_state)?;
        let run_dir = checkpoints_dir.join(&self.run_id);
        checkpoint.save(&run_dir)?;
        Ok(checkpoint)
    }

    /// Resume a pipeline from a checkpoint.
    ///
    /// The flow spec must match the one used when the checkpoint was created.
    /// The resumed engine starts fresh — no chat history is restored.
    pub fn from_checkpoint(checkpoint: Checkpoint, flow: FlowSpec) -> Result<Self, CheckpointError> {
        Ok(Self {
            flow,
            current_step: checkpoint.current_step,
            status: PipelineStatus::Idle,
            run_id: checkpoint.run_id,
            step_history: checkpoint.step_history,
            loop_counters: checkpoint.loop_counters,
            pending_gate: None,
            gate_feedback: checkpoint.gate_feedback,
            cumulative_tokens: checkpoint.cumulative_tokens,
            gate_resolved_for_step: None,
            budget_tracker: checkpoint.budget_tracker,
            mode: checkpoint.mode,
        })
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hex_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Build a file manifest from files touched in the pipeline history.
fn build_file_manifest(
    project_dir: &Path,
    engine: &PipelineEngine,
) -> Vec<FileState> {
    let mut manifest = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for record in &engine.step_history {
        if let Some(ref handoff) = record.handoff {
            for wp in &handoff.work_product {
                let path = &wp.path;
                if !seen.insert(path.clone()) {
                    continue;
                }
                let full_path = project_dir.join(path);
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    let size = content.len() as u64;
                    let hash = hex_hash(content.as_bytes());
                    manifest.push(FileState {
                        path: path.clone(),
                        hash,
                        size,
                        content,
                    });
                }
            }
        }
    }

    manifest
}

/// Capture git state: (commit_sha, diff_patch).
fn git_snapshot(project_dir: &Path) -> (Option<String>, Option<String>) {
    let commit = run_git(project_dir, &["rev-parse", "HEAD"]);
    let diff = run_git(project_dir, &["diff", "HEAD"]);

    let commit_opt = commit.filter(|s| !s.is_empty());
    let diff_opt = diff.filter(|s| !s.is_empty());

    (commit_opt, diff_opt)
}

fn run_git(project_dir: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .current_dir(project_dir)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::flow::{FlowSpec, FlowStep, GateType};
    use crate::relay::handoff::{HandoffDocument, WorkProduct};
    use crate::relay::pipeline::{AdvanceResult, GateDecision};

    fn temp_project() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        (tmp, project)
    }

    fn write_file(project: &Path, rel: &str, content: &str) {
        let path = project.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }

    // ── S4.1: Checkpoint restores after simulated crash ──────────────────────

    #[test]
    fn test_checkpoint_save_and_load() {
        let (_tmp, project) = temp_project();
        let ckpt_dir = project.join(".autoforge").join("checkpoints");

        // Build a pipeline with some history
        let mut flow = FlowSpec::new("test-ckpt");
        flow.add_step(FlowStep::new("s1", "planner"));
        flow.add_step(FlowStep::new("s2", "architect"));

        let mut engine = PipelineEngine::new(flow.clone(), "run-ckpt-1");

        // Step 1
        let _ = engine.advance();
        let mut h1 = HandoffDocument::new("planner", "architect", "run-ckpt-1", 0);
        h1.work_product.push(WorkProduct {
            path: "src/main.rs".to_string(),
            description: "Added auth".to_string(),
            lines: Some(42),
        });
        let _ = engine.submit_handoff(h1);

        // Step 2
        let _ = engine.advance();
        let mut h2 = HandoffDocument::new("architect", "coder", "run-ckpt-1", 1);
        h2.work_product.push(WorkProduct {
            path: "src/lib.rs".to_string(),
            description: "Added utils".to_string(),
            lines: Some(20),
        });
        let _ = engine.submit_handoff(h2);

        // Create files in project dir for manifest capture
        write_file(&project, "src/main.rs", "fn main() {}");
        write_file(&project, "src/lib.rs", "pub mod utils;");

        // Save checkpoint
        let checkpoint = engine
            .save_checkpoint(&ckpt_dir, &project, None)
            .expect("save_checkpoint");

        assert_eq!(checkpoint.run_id, "run-ckpt-1");
        assert_eq!(checkpoint.current_step, 2); // after both steps
        assert_eq!(checkpoint.step_history.len(), 2);
        assert_eq!(checkpoint.file_manifest.len(), 2);

        // Load it back
        let ckpt_path = ckpt_dir.join("run-ckpt-1").join("checkpoint-2.json");
        let loaded = Checkpoint::load(&ckpt_path).expect("load checkpoint");
        assert_eq!(loaded.id, checkpoint.id);
        assert_eq!(loaded.run_id, checkpoint.run_id);
        assert_eq!(loaded.current_step, checkpoint.current_step);
        assert_eq!(loaded.file_manifest.len(), checkpoint.file_manifest.len());
    }

    #[test]
    fn test_checkpoint_restore_files() {
        let (_tmp, project) = temp_project();

        // Create original files
        write_file(&project, "src/main.rs", "fn main() { old }");
        write_file(&project, "src/lib.rs", "pub mod old;");

        // Build a checkpoint manifest manually
        let checkpoint = Checkpoint {
            id: 0,
            run_id: "run-1".into(),
            timestamp: 0,
            git_commit: None,
            git_diff: None,
            ledger_state: None,
            handoff: HandoffDocument::new("", "", "run-1", 0),
            file_manifest: vec![
                FileState {
                    path: "src/main.rs".into(),
                    hash: hex_hash(b"fn main() { restored }"),
                    size: 22,
                    content: "fn main() { restored }".into(),
                },
                FileState {
                    path: "src/lib.rs".into(),
                    hash: hex_hash(b"pub mod restored;"),
                    size: 18,
                    content: "pub mod restored;".into(),
                },
            ],
            token_usage: TokenUsage::default(),
            current_step: 0,
            step_history: Vec::new(),
            loop_counters: HashMap::new(),
            cumulative_tokens: 0,
            gate_feedback: HashMap::new(),
            budget_tracker: BudgetTracker::default(),
            mode: crate::relay::pipeline::RelayMode::GSD,
        };

        // Mutate files before restore
        write_file(&project, "src/main.rs", "fn main() { mutated }");

        // Restore
        checkpoint.restore_files(&project).expect("restore_files");

        // Verify restored state
        let main_content = std::fs::read_to_string(project.join("src/main.rs")).unwrap();
        let lib_content = std::fs::read_to_string(project.join("src/lib.rs")).unwrap();
        assert_eq!(main_content, "fn main() { restored }");
        assert_eq!(lib_content, "pub mod restored;");
    }

    #[test]
    fn test_checkpoint_integrity_hash() {
        let cp1 = Checkpoint {
            id: 0,
            run_id: "run-1".into(),
            timestamp: 0,
            git_commit: None,
            git_diff: None,
            ledger_state: None,
            handoff: HandoffDocument::new("a", "b", "run-1", 0),
            file_manifest: Vec::new(),
            token_usage: TokenUsage::default(),
            current_step: 0,
            step_history: Vec::new(),
            loop_counters: HashMap::new(),
            cumulative_tokens: 0,
            gate_feedback: HashMap::new(),
            budget_tracker: BudgetTracker::default(),
            mode: crate::relay::pipeline::RelayMode::GSD,
        };
        let cp2 = cp1.clone();
        assert_eq!(cp1.integrity_hash(), cp2.integrity_hash());

        let mut cp3 = cp1.clone();
        cp3.run_id = "run-2".into();
        assert_ne!(cp1.integrity_hash(), cp3.integrity_hash());
    }

    // ── S4.2: Resume rehydrates agent without chat history ───────────────────

    #[test]
    fn test_resume_from_checkpoint_rehydrates_clean() {
        let mut flow = FlowSpec::new("test-resume");
        flow.add_step(FlowStep::new("s1", "planner"));
        flow.add_step(FlowStep::new("s2", "architect"));
        flow.add_step(FlowStep::new("s3", "coder"));

        let mut engine = PipelineEngine::new(flow.clone(), "run-resume");

        // Run planner step
        let _ = engine.advance();
        let h1 = HandoffDocument::new("planner", "architect", "run-resume", 0);
        let _ = engine.submit_handoff(h1);

        // Create checkpoint
        let checkpoint = Checkpoint::create(&engine, Path::new("."), None).unwrap();
        assert_eq!(checkpoint.current_step, 1); // next is architect
        assert_eq!(checkpoint.step_history.len(), 1);

        // Simulate crash: drop engine
        drop(engine);

        // Resume from checkpoint
        let mut resumed = PipelineEngine::from_checkpoint(checkpoint, flow).unwrap();

        // Verify resumed state
        assert_eq!(resumed.current_step, 1);
        assert_eq!(resumed.status, PipelineStatus::Idle);
        assert_eq!(resumed.run_id, "run-resume");
        assert_eq!(resumed.step_history.len(), 1);

        // Continue: should execute architect
        let r = resumed.advance();
        assert_eq!(
            r,
            AdvanceResult::ExecuteStep {
                step_id: "s2".into(),
                profession_id: "architect".into(),
            }
        );

        // Submit handoff → coder
        let h2 = HandoffDocument::new("architect", "coder", "run-resume", 1);
        let mut engine2 = resumed;
        let r2 = engine2.submit_handoff(h2);
        assert_eq!(
            r2,
            AdvanceResult::ExecuteStep {
                step_id: "s3".into(),
                profession_id: "coder".into(),
            }
        );
    }

    #[test]
    fn test_resume_preserves_loop_counters_and_feedback() {
        let mut flow = FlowSpec::new("test-resume-state");
        flow.add_step(
            FlowStep::new("s1", "tester").with_exit(crate::relay::flow::ExitRouting::Loop {
                target_step_id: "s1".into(),
                max_iterations: 5,
            }),
        );

        let mut engine = PipelineEngine::new(flow.clone(), "run-state");

        // Run 2 iterations
        for _ in 0..2 {
            let _ = engine.advance();
            let h = HandoffDocument::new("tester", "tester", "run-state", 0);
            let _ = engine.submit_handoff(h);
        }

        assert_eq!(engine.loop_counters.get("s1"), Some(&2));

        // Save checkpoint
        let checkpoint = Checkpoint::create(&engine, Path::new("."), None).unwrap();

        // Resume
        let resumed = PipelineEngine::from_checkpoint(checkpoint, flow).unwrap();
        assert_eq!(resumed.loop_counters.get("s1"), Some(&2));
        assert_eq!(resumed.current_step, 0); // still on s1 because loop
    }

    #[test]
    fn test_checkpoint_with_human_gate_feedback() {
        let mut flow = FlowSpec::new("test-gate-ckpt");
        flow.add_step(FlowStep::new("s1", "advisor").with_gate(GateType::Human));
        flow.add_step(FlowStep::new("s2", "architect"));

        let mut engine = PipelineEngine::new(flow.clone(), "run-gate-ckpt");

        // Hit gate
        let _ = engine.advance();
        assert!(matches!(engine.status, PipelineStatus::WaitingForHuman { .. }));

        // Reject with feedback
        let _ = engine.resolve_gate(GateDecision::Reject {
            feedback: "Need more detail".into(),
        });

        // Run step with feedback
        let _ = engine.advance();
        let h = HandoffDocument::new("planner", "architect", "run-gate-ckpt", 0);
        let _ = engine.submit_handoff(h);

        // Checkpoint preserves feedback
        let checkpoint = Checkpoint::create(&engine, Path::new("."), None).unwrap();
        assert!(checkpoint.gate_feedback.contains_key("s1"));
        assert_eq!(checkpoint.gate_feedback.get("s1").unwrap().len(), 1);

        let resumed = PipelineEngine::from_checkpoint(checkpoint, flow).unwrap();
        assert!(resumed.gate_feedback.contains_key("s1"));
    }
}
