use auto_lang::autovm_persistent::AutovmReplSession;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};

/// Metadata for a cell sent from the frontend during execution
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct NotebookCellMeta {
    pub cell_id: String,
    pub source: String,
    pub depends_on: Vec<String>,
}

/// A structured diagnostic message
#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub severity: String, // "error" | "warning"
    pub message: String,
    pub line: Option<usize>,
}

/// A notebook cell's execution output
#[derive(Debug, Clone, serde::Serialize)]
pub struct CellOutput {
    pub stdout: String,
    pub stderr: String,
    pub result: String,
    pub time_ms: u64,
    pub diagnostics: Vec<Diagnostic>,
}

/// A single notebook cell (internal state)
#[derive(Debug, Clone, serde::Serialize)]
pub struct Cell {
    pub cell_id: String,
    pub source: String,
    pub output: Option<CellOutput>,
    pub depends_on: Vec<String>,
}

/// Variable info for the inspector
#[derive(Debug, Clone, serde::Serialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Closed,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VariableInfo {
    pub name: String,
    pub kind: String,
}

/// Internal notebook session (lives in the actor thread)
struct NotebookSession {
    id: String,
    vm: AutovmReplSession,
    cells: Vec<Cell>,
    cell_snapshots: HashMap<String, String>,
    #[allow(dead_code)] // session metadata, reserved for diagnostics/reporting
    created_at: Instant,
    last_active: Instant,
}

impl NotebookSession {
    fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            vm: AutovmReplSession::new(),
            cells: Vec::new(),
            cell_snapshots: HashMap::new(),
            created_at: Instant::now(),
            last_active: Instant::now(),
        }
    }

    fn execute(
        &mut self,
        cell_id: &str,
        source: &str,
        notebook_cells: Option<Vec<NotebookCellMeta>>,
    ) -> CellOutput {
        let start = Instant::now();
        self.last_active = start;

        // Update notebook structure if provided
        if let Some(cells_meta) = notebook_cells {
            let mut new_cells: Vec<Cell> = cells_meta
                .into_iter()
                .map(|m| {
                    let existing_output = self
                        .cells
                        .iter()
                        .find(|c| c.cell_id == m.cell_id)
                        .and_then(|c| c.output.clone());
                    Cell {
                        cell_id: m.cell_id,
                        source: m.source,
                        output: existing_output,
                        depends_on: m.depends_on,
                    }
                })
                .collect();

            // Preserve any cells not in the new list (e.g., newly added target)
            for old in &self.cells {
                if !new_cells.iter().any(|c| c.cell_id == old.cell_id) {
                    new_cells.push(old.clone());
                }
            }
            self.cells = new_cells;
        }

        // Ensure target cell exists in the list
        if !self.cells.iter().any(|c| c.cell_id == cell_id) {
            self.cells.push(Cell {
                cell_id: cell_id.to_string(),
                source: source.to_string(),
                output: None,
                depends_on: Vec::new(),
            });
        }

        // Build execution queue: dirty upstream cells + target
        let queue = self.build_execution_queue(cell_id, source);

        let mut target_output = None;
        let mut total_time_ms = 0u64;

        for (exec_id, exec_source) in queue {
            let cell_start = Instant::now();
            let output = self.run_single_cell(&exec_id, &exec_source);
            let cell_time = cell_start.elapsed().as_millis() as u64;
            total_time_ms += cell_time;

            if exec_id == cell_id {
                target_output = Some(CellOutput {
                    stdout: output.stdout.clone(),
                    stderr: output.stderr.clone(),
                    result: output.result.clone(),
                    time_ms: total_time_ms,
                    diagnostics: output.diagnostics.clone(),
                });
            }

            // Update snapshot and cell output
            self.cell_snapshots.insert(exec_id.clone(), exec_source.clone());
            if let Some(cell) = self.cells.iter_mut().find(|c| c.cell_id == exec_id) {
                cell.output = Some(output);
                cell.source = exec_source;
            }
        }

        target_output.unwrap_or_else(|| CellOutput {
            stdout: String::new(),
            stderr: String::new(),
            result: String::new(),
            time_ms: total_time_ms,
            diagnostics: Vec::new(),
        })
    }

    fn run_single_cell(&mut self, _cell_id: &str, source: &str) -> CellOutput {
        let capture = Arc::new(std::sync::RwLock::new(String::new()));
        self.vm.vm.output_buffer = Some(capture.clone());

        let result = self.vm.run(source);

        let stdout = capture.read().unwrap().clone();
        self.vm.vm.output_buffer = None;

        match result {
            Ok(_) => CellOutput {
                stdout,
                stderr: String::new(),
                result: self.vm.format_last_result().unwrap_or_default(),
                time_ms: 0,
                diagnostics: Vec::new(),
            },
            Err(e) => {
                let err_str = e.to_string();
                let diagnostics = extract_diagnostics(&err_str);
                CellOutput {
                    stdout,
                    stderr: err_str,
                    result: String::new(),
                    time_ms: 0,
                    diagnostics,
                }
            }
        }
    }

    fn build_execution_queue(&self, target_id: &str, target_source: &str) -> Vec<(String, String)> {
        let mut dirty = HashSet::new();

        // Mark cells with changed source as dirty
        for cell in &self.cells {
            let snapshot = self.cell_snapshots.get(&cell.cell_id);
            if snapshot.map(|s| *s != cell.source).unwrap_or(true) {
                dirty.insert(cell.cell_id.clone());
            }
        }

        // Cascade: mark downstream cells as dirty
        let mut changed = true;
        while changed {
            changed = false;
            for cell in &self.cells {
                if dirty.contains(&cell.cell_id) {
                    continue;
                }
                // Determine dependencies
                let deps = if cell.depends_on.is_empty() {
                    // Default: depend on all preceding cells
                    self.cells
                        .iter()
                        .take_while(|c| c.cell_id != cell.cell_id)
                        .map(|c| c.cell_id.clone())
                        .collect::<Vec<_>>()
                } else {
                    cell.depends_on.clone()
                };
                if deps.iter().any(|d| dirty.contains(d)) {
                    dirty.insert(cell.cell_id.clone());
                    changed = true;
                }
            }
        }

        // Build queue: all dirty cells up to and including target, in notebook order
        let mut queue = Vec::new();
        let mut target_seen = false;

        for cell in &self.cells {
            if cell.cell_id == target_id {
                queue.push((cell.cell_id.clone(), target_source.to_string()));
                target_seen = true;
                break;
            }
            if dirty.contains(&cell.cell_id) {
                queue.push((cell.cell_id.clone(), cell.source.clone()));
            }
        }

        // If target wasn't in the ordered list, append it
        if !target_seen {
            queue.push((target_id.to_string(), target_source.to_string()));
        }

        queue
    }

    fn status(&self) -> SessionStatus {
        let idle_duration = self.last_active.elapsed();
        if idle_duration.as_secs() > 300 {
            SessionStatus::Idle
        } else {
            SessionStatus::Active
        }
    }

    fn variables(&self) -> Vec<VariableInfo> {
        let mut vars = Vec::new();
        for name in self.vm.locals() {
            vars.push(VariableInfo {
                name,
                kind: "local".to_string(),
            });
        }
        for name in self.vm.functions() {
            vars.push(VariableInfo {
                name,
                kind: "function".to_string(),
            });
        }
        vars
    }
}

// ============================================================================
// Actor pattern: all VM sessions live in a dedicated thread
// ============================================================================

enum NotebookCommand {
    CreateSession { respond: oneshot::Sender<String> },
    Execute {
        sid: String,
        cell_id: String,
        source: String,
        notebook_cells: Option<Vec<NotebookCellMeta>>,
        respond: oneshot::Sender<CellOutput>,
    },
    Status {
        sid: String,
        respond: oneshot::Sender<SessionStatus>,
    },
    Variables {
        sid: String,
        respond: oneshot::Sender<Vec<VariableInfo>>,
    },
    Destroy { sid: String },
}

/// Thread-safe handle to the notebook actor
#[derive(Clone)]
pub struct NotebookActor {
    tx: mpsc::UnboundedSender<NotebookCommand>,
}

impl NotebookActor {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<NotebookCommand>();

        std::thread::spawn(move || {
            let mut sessions: HashMap<String, NotebookSession> = HashMap::new();

            while let Some(cmd) = rx.blocking_recv() {
                match cmd {
                    NotebookCommand::CreateSession { respond } => {
                        let session = NotebookSession::new();
                        let id = session.id.clone();
                        sessions.insert(id.clone(), session);
                        let _ = respond.send(id);
                    }
                    NotebookCommand::Execute {
                        sid,
                        cell_id,
                        source,
                        notebook_cells,
                        respond,
                    } => {
                        let output = sessions
                            .get_mut(&sid)
                            .map(|s| s.execute(&cell_id, &source, notebook_cells))
                            .unwrap_or_else(|| CellOutput {
                                stdout: String::new(),
                                stderr: format!("Session '{}' not found", sid),
                                result: String::new(),
                                time_ms: 0,
                                diagnostics: Vec::new(),
                            });
                        let _ = respond.send(output);
                    }
                    NotebookCommand::Status { sid, respond } => {
                        let status = sessions
                            .get(&sid)
                            .map(|s| s.status())
                            .unwrap_or(SessionStatus::Closed);
                        let _ = respond.send(status);
                    }
                    NotebookCommand::Variables { sid, respond } => {
                        let vars = sessions
                            .get(&sid)
                            .map(|s| s.variables())
                            .unwrap_or_default();
                        let _ = respond.send(vars);
                    }
                    NotebookCommand::Destroy { sid } => {
                        sessions.remove(&sid);
                    }
                }
            }
        });

        Self { tx }
    }

    pub async fn create_session(&self) -> String {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(NotebookCommand::CreateSession { respond: tx });
        rx.await.unwrap_or_default()
    }

    pub async fn execute(
        &self,
        sid: String,
        cell_id: String,
        source: String,
        notebook_cells: Option<Vec<NotebookCellMeta>>,
    ) -> CellOutput {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(NotebookCommand::Execute {
            sid,
            cell_id,
            source,
            notebook_cells,
            respond: tx,
        });
        rx.await.unwrap_or_else(|_| CellOutput {
            stdout: String::new(),
            stderr: "Notebook actor closed".to_string(),
            result: String::new(),
            time_ms: 0,
            diagnostics: Vec::new(),
        })
    }

    pub async fn status(&self, sid: String) -> SessionStatus {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(NotebookCommand::Status { sid, respond: tx });
        rx.await.unwrap_or(SessionStatus::Closed)
    }

    pub async fn variables(&self, sid: String) -> Vec<VariableInfo> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(NotebookCommand::Variables { sid, respond: tx });
        rx.await.unwrap_or_default()
    }

    pub fn destroy(&self, sid: String) {
        let _ = self.tx.send(NotebookCommand::Destroy { sid });
    }
}

/// Shared notebook state across requests
pub type NotebookState = NotebookActor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_diagnostics_with_line() {
        let err = "error at line 5: unexpected token\nline 10: another error";
        let diags = extract_diagnostics(err);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].line, Some(5));
        assert_eq!(diags[0].message, "error at line 5: unexpected token");
        assert_eq!(diags[1].line, Some(10));
    }

    #[test]
    fn test_extract_diagnostics_without_line() {
        let err = "Something went wrong\nUnknown symbol 'foo'";
        let diags = extract_diagnostics(err);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].line, None);
        assert_eq!(diags[0].message, "Something went wrong");
    }

    #[test]
    fn test_extract_diagnostics_empty() {
        let diags = extract_diagnostics("");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "");
    }

    #[tokio::test]
    async fn test_notebook_actor_create_session() {
        let actor = NotebookActor::new();
        let sid = actor.create_session().await;
        assert!(!sid.is_empty());
    }

    #[tokio::test]
    async fn test_notebook_actor_execute_and_result() {
        let actor = NotebookActor::new();
        let sid = actor.create_session().await;

        // Pure expression should return result via format_last_result
        let output = actor.execute(sid.clone(), "c1".to_string(), "42".to_string(), None).await;
        assert!(output.stderr.is_empty(), "stderr: {}", output.stderr);
        assert_eq!(output.result, "42", "expected result 42, got: {}", output.result);
    }

    #[tokio::test]
    async fn test_notebook_actor_functions() {
        let actor = NotebookActor::new();
        let sid = actor.create_session().await;

        // Define a function
        let out = actor.execute(sid.clone(), "c1".to_string(), "fn add(a int, b int) int { a + b }".to_string(), None).await;
        assert!(out.stderr.is_empty(), "stderr: {}", out.stderr);

        let vars = actor.variables(sid).await;
        assert!(vars.iter().any(|v| v.name == "add" && v.kind == "function"), "expected function 'add', got: {:?}", vars);
    }

    #[test]
    fn test_build_execution_queue_dirty_upstream() {
        let mut session = NotebookSession::new();
        session.cells = vec![
            Cell { cell_id: "c1".to_string(), source: "v1".to_string(), output: None, depends_on: vec![] },
            Cell { cell_id: "c2".to_string(), source: "v2".to_string(), output: None, depends_on: vec!["c1".to_string()] },
        ];
        session.cell_snapshots.insert("c1".to_string(), "v1".to_string());
        session.cell_snapshots.insert("c2".to_string(), "v2".to_string());

        // Modify c1 source
        session.cells[0].source = "v1_new".to_string();

        let queue = session.build_execution_queue("c2", "v2");
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].0, "c1");
        assert_eq!(queue[0].1, "v1_new");
        assert_eq!(queue[1].0, "c2");
    }

    #[test]
    fn test_build_execution_queue_no_dirty() {
        let mut session = NotebookSession::new();
        session.cells = vec![
            Cell { cell_id: "c1".to_string(), source: "v1".to_string(), output: None, depends_on: vec![] },
            Cell { cell_id: "c2".to_string(), source: "v2".to_string(), output: None, depends_on: vec![] },
        ];
        session.cell_snapshots.insert("c1".to_string(), "v1".to_string());
        session.cell_snapshots.insert("c2".to_string(), "v2".to_string());

        // Nothing modified — only target should be in queue
        let queue = session.build_execution_queue("c2", "v2");
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].0, "c2");
    }

    #[test]
    fn test_build_execution_queue_cascade_dirty() {
        let mut session = NotebookSession::new();
        session.cells = vec![
            Cell { cell_id: "c1".to_string(), source: "v1".to_string(), output: None, depends_on: vec![] },
            Cell { cell_id: "c2".to_string(), source: "v2".to_string(), output: None, depends_on: vec!["c1".to_string()] },
            Cell { cell_id: "c3".to_string(), source: "v3".to_string(), output: None, depends_on: vec!["c2".to_string()] },
        ];
        session.cell_snapshots.insert("c1".to_string(), "v1".to_string());
        session.cell_snapshots.insert("c2".to_string(), "v2".to_string());
        session.cell_snapshots.insert("c3".to_string(), "v3".to_string());

        // Modify c1 — should cascade dirty to c2 and c3
        session.cells[0].source = "v1_new".to_string();

        let queue = session.build_execution_queue("c3", "v3");
        assert_eq!(queue.len(), 3);
        assert_eq!(queue[0].0, "c1");
        assert_eq!(queue[1].0, "c2");
        assert_eq!(queue[2].0, "c3");
    }

    #[tokio::test]
    async fn test_notebook_actor_status() {
        let actor = NotebookActor::new();
        let sid = actor.create_session().await;
        let status = actor.status(sid).await;
        assert!(matches!(status, SessionStatus::Active));
    }

    #[tokio::test]
    async fn test_notebook_actor_destroy() {
        let actor = NotebookActor::new();
        let sid = actor.create_session().await;
        actor.destroy(sid.clone());
        // After destroy, variables should be empty
        let vars = actor.variables(sid).await;
        assert!(vars.is_empty());
    }
}

/// Try to extract structured diagnostics from a raw error string.
/// Falls back to a single diagnostic with the full message if no line info is found.
fn extract_diagnostics(err: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Look for patterns like "line 5:" or "at line 5" or "[line 5]"
    let line_re = regex::Regex::new(r"(?i)(?:line\s+(\d+)|:(\d+):|@(\d+))").ok();

    for line in err.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut diag_line: Option<usize> = None;
        if let Some(re) = &line_re {
            if let Some(caps) = re.captures(trimmed) {
                diag_line = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .or_else(|| caps.get(3))
                    .and_then(|m| m.as_str().parse().ok());
            }
        }
        diagnostics.push(Diagnostic {
            severity: "error".to_string(),
            message: trimmed.to_string(),
            line: diag_line,
        });
    }

    if diagnostics.is_empty() {
        diagnostics.push(Diagnostic {
            severity: "error".to_string(),
            message: err.to_string(),
            line: None,
        });
    }

    diagnostics
}
