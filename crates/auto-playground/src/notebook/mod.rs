use auto_lang::autovm_persistent::AutovmReplSession;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};

/// A notebook cell's execution output
#[derive(Debug, Clone, serde::Serialize)]
pub struct CellOutput {
    pub stdout: String,
    pub stderr: String,
    pub result: String,
    pub time_ms: u64,
}

/// A single notebook cell
#[derive(Debug, Clone, serde::Serialize)]
pub struct Cell {
    pub cell_id: String,
    pub source: String,
    pub output: Option<CellOutput>,
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
    created_at: Instant,
    last_active: Instant,
}

impl NotebookSession {
    fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            vm: AutovmReplSession::new(),
            cells: Vec::new(),
            created_at: Instant::now(),
            last_active: Instant::now(),
        }
    }

    fn execute(&mut self, cell_id: &str, source: &str) -> CellOutput {
        let start = Instant::now();
        self.last_active = start;

        // Set up stdout capture buffer
        let capture = Arc::new(std::sync::RwLock::new(String::new()));
        self.vm.vm.output_buffer = Some(capture.clone());

        let result = self.vm.run(source);
        let time_ms = start.elapsed().as_millis() as u64;

        // Collect captured stdout
        let stdout = capture.read().unwrap().clone();

        // Clear buffer for next execution
        self.vm.vm.output_buffer = None;

        let output = match result {
            Ok(_) => CellOutput {
                stdout,
                stderr: String::new(),
                result: self.vm.format_last_result().unwrap_or_default(),
                time_ms,
            },
            Err(e) => CellOutput {
                stdout,
                stderr: e.to_string(),
                result: String::new(),
                time_ms,
            },
        };

        // Update or append cell
        if let Some(cell) = self.cells.iter_mut().find(|c| c.cell_id == cell_id) {
            cell.source = source.to_string();
            cell.output = Some(output.clone());
        } else {
            self.cells.push(Cell {
                cell_id: cell_id.to_string(),
                source: source.to_string(),
                output: Some(output.clone()),
            });
        }

        output
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
                        respond,
                    } => {
                        let output = sessions
                            .get_mut(&sid)
                            .map(|s| s.execute(&cell_id, &source))
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

    pub async fn execute(&self, sid: String, cell_id: String, source: String) -> CellOutput {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(NotebookCommand::Execute {
            sid,
            cell_id,
            source,
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
