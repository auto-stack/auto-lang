//! Plan 127: Scheduler Message Dispatch Loop
//!
//! Implements Actor-style message passing between tasks.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     TaskSystem.start() [Daemon Loop]                │
//! │                                                                     │
//! │   sys_rx.recv().await ──┬──► Spawn ──► spawn_dynamic_task()        │
//! │                         │                                           │
//! │                         └──► Stop ──► break (shutdown)              │
//! │                                                                     │
//! │  ┌──────────────────────────────────────────────────────────────┐   │
//! │  │                    tokio::spawn × N                           │   │
//! │  │                                                               │   │
//! │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │   │
//! │  │  │ TaskCtx 1   │  │ TaskCtx 2   │  │ TaskCtx N   │            │   │
//! │  │  │ (Logger)    │  │ (Monitor)   │  │ (Worker)    │            │   │
//! │  │  │             │  │             │  │             │            │   │
//! │  │  │ mailbox: rx │  │ mailbox: rx │  │ mailbox: rx │            │   │
//! │  │  │ task.ram    │  │ task.ram    │  │ task.ram    │            │   │
//! │  │  │ sys_tx      │  │ sys_tx      │  │ sys_tx      │            │   │
//! │  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │   │
//! │  │         │                │                │                   │   │
//! │  │         ▼                ▼                ▼                   │   │
//! │  │  ┌────────────────────────────────────────────────────────┐   │   │
//! │  │  │              GlobalMeta (Arc<GlobalMeta>)              │   │   │
//! │  │  │              (zero locks, zero copies, zero contention)│   │   │
//! │  │  │                                                        │   │   │
//! │  │  │  bytecode: VirtualFlash                                │   │   │
//! │  │  │  string_pool: Vec<Vec<u8>>                             │   │   │
//! │  │  │  native_interface: NativeInterface                     │   │   │
//! │  │  │  handler_tables: HashMap<String, TaskHandlerTable>     │   │   │
//! │  │  └────────────────────────────────────────────────────────┘   │   │
//! │  │                                                               │   │
//! │  └──────────────────────────────────────────────────────────────┘   │
//! │                                                                     │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use crate::vm::native::NativeInterface;
use crate::vm::task::AutoTask;
use crate::vm::task_handler::TaskHandlerTable;
use crate::vm::virt_memory::VirtualFlash;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Global read-only metadata - wrapped in `Arc<GlobalMeta>`
///
/// No inner Arcs needed since outer Arc provides protection.
/// This struct is intentionally not Clone - it should be shared via Arc.
pub struct GlobalMeta {
    /// Bytecode (Flash) - read-only
    pub bytecode: VirtualFlash,
    /// String pool (read-only)
    pub string_pool: Vec<Vec<u8>>,
    /// Native interface (read-only)
    pub native_interface: NativeInterface,
    /// Handler tables per task type
    pub handler_tables: HashMap<String, TaskHandlerTable>,
}

impl GlobalMeta {
    /// Create a new empty GlobalMeta
    pub fn new() -> Self {
        Self {
            bytecode: VirtualFlash::new(0),
            string_pool: Vec::new(),
            native_interface: NativeInterface::new(),
            handler_tables: HashMap::new(),
        }
    }

    /// Create GlobalMeta from existing VM components
    pub fn from_components(
        bytecode: VirtualFlash,
        string_pool: Vec<Vec<u8>>,
        native_interface: NativeInterface,
        handler_tables: HashMap<String, TaskHandlerTable>,
    ) -> Self {
        Self {
            bytecode,
            string_pool,
            native_interface,
            handler_tables,
        }
    }

    /// Get handler table for a task type
    pub fn get_handler_table(&self, task_type: &str) -> Option<&TaskHandlerTable> {
        self.handler_tables.get(task_type)
    }
}

impl Default for GlobalMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// System command for privileged operations
#[derive(Debug, Clone)]
pub enum SystemCommand {
    /// Shutdown the system
    Stop,

    /// Dynamically spawn a child task
    Spawn {
        task_type: String,
        capacity: usize,
        parent_id: Option<u64>,
    },
}

/// Per-task execution context - completely isolated, no shared mutable state
pub struct TaskContext {
    // ========== Shared Read-Only Metadata ==========
    pub meta: Arc<GlobalMeta>,

    // ========== Task-Isolated State ==========
    pub task_type: String,
    pub instance_id: u64,
    pub mailbox: mpsc::Receiver<auto_val::Value>,
    pub sys_tx: mpsc::Sender<SystemCommand>,
    pub task: AutoTask,

    // ========== Handler Table Reference ==========
    pub handlers: TaskHandlerTable,
}

impl TaskContext {
    /// Create a new task context
    pub fn new(
        meta: Arc<GlobalMeta>,
        task_type: String,
        instance_id: u64,
        mailbox: mpsc::Receiver<auto_val::Value>,
        sys_tx: mpsc::Sender<SystemCommand>,
        task: AutoTask,
    ) -> Self {
        // Clone the handler table for this task type (or empty if not found)
        let handlers = meta
            .get_handler_table(&task_type)
            .cloned()
            .unwrap_or_else(|| TaskHandlerTable::new(task_type.clone()));

        Self {
            meta,
            task_type,
            instance_id,
            mailbox,
            sys_tx,
            task,
            handlers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_global_meta_new() {
        let meta = GlobalMeta::new();
        assert!(meta.bytecode.memory.is_empty());
    }

    #[test]
    fn test_global_meta_default() {
        let meta = GlobalMeta::default();
        assert!(meta.string_pool.is_empty());
    }

    #[test]
    fn test_task_context_new() {
        let meta = Arc::new(GlobalMeta::new());
        let (_tx, rx) = tokio::sync::mpsc::channel::<auto_val::Value>(16);
        let (sys_tx, _sys_rx) = tokio::sync::mpsc::channel::<SystemCommand>(16);

        let ctx = TaskContext::new(
            Arc::clone(&meta),
            "TestTask".to_string(),
            1,
            rx,
            sys_tx,
            crate::vm::task::AutoTask::new(1, 1024, 0),
        );

        assert_eq!(ctx.task_type, "TestTask");
        assert_eq!(ctx.instance_id, 1);
    }
}
