pub mod ai;
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

/// A notebook cell's execution output
#[derive(Debug, Clone, serde::Serialize)]
pub struct CellOutput {
    pub stdout: String,
    pub stderr: String,
    pub result: String,
    pub time_ms: u64,
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
            },
            Err(e) => CellOutput {
                stdout,
                stderr: e.to_string(),
                result: String::new(),
                time_ms: 0,
            },
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
                            });
                        let _ = respond.send(output);
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
        })
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
