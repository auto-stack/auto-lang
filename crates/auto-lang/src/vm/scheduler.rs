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

use crate::vm::engine::VMError;
use crate::vm::native::NativeInterface;
use crate::vm::opcode::OpCode;
use crate::vm::task::AutoTask;
use crate::vm::task::TaskStatus;
use crate::vm::task_handler::TaskHandlerTable;
use crate::vm::task_system::TaskHandle;
use crate::vm::virt_memory::VirtualFlash;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
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

/// Execute handler fully - async to support await, yields instead of breaking
///
/// This function runs a handler to completion (RET or HALT), with:
/// - Cooperative yielding via `tokio::task::yield_now()` to prevent CPU starvation
/// - Async FFI support via AWAIT_EXT opcode (future)
/// - Budget defense that preserves task.ip state
pub async fn execute_handler_fully(
    meta: &GlobalMeta,
    task: &mut AutoTask,
) -> Result<TaskStatus, VMError> {
    const BUDGET: u32 = 10_000;
    let mut ops_executed = 0;

    loop {
        // Bounds check
        if task.ip >= meta.bytecode.memory.len() {
            return Ok(TaskStatus::Terminated);
        }

        // Fetch opcode
        let op_byte = meta.bytecode.read_u8(task.ip);
        let opcode = OpCode::from(op_byte);
        task.ip += 1;

        match opcode {
            OpCode::RET | OpCode::HALT => {
                // Normal completion
                return Ok(TaskStatus::Terminated);
            }

            OpCode::NOP => {
                // Do nothing
            }

            // Note: AWAIT_EXT would be handled here for async FFI
            // OpCode::AWAIT_EXT => {
            //     handle_async_ffi(meta, task).await;
            // }

            _ => {
                // For now, skip unknown opcodes
                // In full implementation, this would call execute_single_op()
            }
        }

        // Budget defense: yield CPU, but preserve task.ip state
        ops_executed += 1;
        if ops_executed >= BUDGET {
            tokio::task::yield_now().await;
            ops_executed = 0;
        }
    }
}

/// Plan 224: VM-aware handler execution using execute_single_frame.
/// This runs a handler to completion with full opcode support via the VM engine.
pub async fn execute_handler_with_vm(
    vm: &crate::vm::engine::AutoVM,
    task: &mut AutoTask,
) -> Result<TaskStatus, VMError> {
    use crate::vm::engine::FrameResult;

    loop {
        match vm.execute_single_frame(task, 10_000) {
            FrameResult::Return => return Ok(TaskStatus::Terminated),
            FrameResult::Yielded => {
                if matches!(task.status, TaskStatus::Waiting(_)) {
                    return Ok(task.status.clone());
                }
                tokio::task::yield_now().await;
            }
            FrameResult::AwaitFuture { future_id, body_offset } => {
                vm.handle_await_future(task, future_id, body_offset)?;
            }
            FrameResult::BudgetExhausted => {
                tokio::task::yield_now().await;
            }
            FrameResult::Error(e) => return Err(e),
            FrameResult::Continue => unreachable!(),
        }
    }
}

/// Task message loop - runs until mailbox closes or HALT
///
/// Lifecycle:
/// 1. Execute start hook (if present)
/// 2. Loop: receive message -> match pattern -> execute handler
/// 3. Execute stop hook (if present, on mailbox close)
pub async fn task_loop(ctx: &mut TaskContext) {
    // 1. Start Hook
    if let Some(start_offset) = ctx.handlers.start_hook_offset {
        ctx.task.ip = start_offset as usize;
        let _ = execute_handler_fully(&ctx.meta, &mut ctx.task).await;
    }

    // 2. Main Message Loop
    loop {
        // Wait for message
        let msg = match ctx.mailbox.recv().await {
            Some(m) => m,
            None => break, // Mailbox closed
        };

        // Try to match a handler
        let mut matched = false;
        for handler in ctx.handlers.get_handlers() {
            if let Some(pattern) = ctx.handlers.get_pattern(handler.pattern_idx) {
                if let Some(_bindings) = try_match_pattern(&ctx.handlers, pattern, &msg) {
                    // Execute handler
                    ctx.task.ip = handler.body_offset as usize;
                    let _ = execute_handler_fully(&ctx.meta, &mut ctx.task).await;
                    matched = true;
                    break;
                }
            }
        }

        // 3. Else Handler (if no match)
        if !matched {
            if let Some(else_offset) = ctx.handlers.else_handler_offset {
                ctx.task.ip = else_offset as usize;
                let _ = execute_handler_fully(&ctx.meta, &mut ctx.task).await;
            }
        }
    }

    // 4. Stop Hook
    if let Some(stop_offset) = ctx.handlers.stop_hook_offset {
        ctx.task.ip = stop_offset as usize;
        let _ = execute_handler_fully(&ctx.meta, &mut ctx.task).await;
    }

    ctx.task.status = TaskStatus::Terminated;
}

/// Try to match a serialized pattern against a message
fn try_match_pattern(
    table: &TaskHandlerTable,
    pattern: &crate::vm::task_handler::SerializedPattern,
    msg: &auto_val::Value,
) -> Option<crate::vm::pattern_matcher::MatchResult> {
    use crate::vm::task_handler::PatternType;

    match pattern.pattern_type {
        PatternType::Literal => {
            if pattern.data.is_empty() {
                return None;
            }
            let lit_type = pattern.data[0];
            match lit_type {
                0x01 => {
                    // String literal
                    if pattern.data.len() < 5 {
                        return None;
                    }
                    let idx = u32::from_le_bytes([
                        pattern.data[1],
                        pattern.data[2],
                        pattern.data[3],
                        pattern.data[4],
                    ]) as usize;
                    if let Some(s) = table.get_string(idx as u32) {
                        if let auto_val::Value::Str(v) = msg {
                            if s == v.as_str() {
                                return Some(crate::vm::pattern_matcher::MatchResult::empty());
                            }
                        }
                    }
                    None
                }
                0x02 => {
                    // Int literal
                    if pattern.data.len() < 9 {
                        return None;
                    }
                    let n = i64::from_le_bytes([
                        pattern.data[1],
                        pattern.data[2],
                        pattern.data[3],
                        pattern.data[4],
                        pattern.data[5],
                        pattern.data[6],
                        pattern.data[7],
                        pattern.data[8],
                    ]);
                    if let auto_val::Value::Int(v) = msg {
                        if n == *v as i64 {
                            return Some(crate::vm::pattern_matcher::MatchResult::empty());
                        }
                    }
                    None
                }
                _ => None,
            }
        }
        PatternType::Simple => {
            // Simple variant matching
            if pattern.data.len() < 4 {
                return None;
            }
            let idx = u32::from_le_bytes([
                pattern.data[0],
                pattern.data[1],
                pattern.data[2],
                pattern.data[3],
            ]) as usize;
            if let Some(name) = table.get_string(idx as u32) {
                // Match against string value (for MVP)
                if let auto_val::Value::Str(v) = msg {
                    if name == v.as_str() {
                        return Some(crate::vm::pattern_matcher::MatchResult::empty());
                    }
                }
                // Match against object with __variant field
                if let auto_val::Value::Obj(obj) = msg {
                    if let Some(auto_val::Value::Str(v)) =
                        obj.get(auto_val::AutoStr::from("__variant"))
                    {
                        if name == v.as_str() {
                            return Some(crate::vm::pattern_matcher::MatchResult::empty());
                        }
                    }
                }
            }
            None
        }
        PatternType::TypeBinding => {
            // Type binding - matches any value of the type
            if pattern.data.len() >= 4 {
                let _name_idx = u32::from_le_bytes([
                    pattern.data[0],
                    pattern.data[1],
                    pattern.data[2],
                    pattern.data[3],
                ]);
                // For MVP, just accept the message
                return Some(crate::vm::pattern_matcher::MatchResult::new(vec![(
                    "msg".to_string(),
                    msg.clone(),
                )]));
            }
            None
        }
        PatternType::WithBindings => {
            // Variant with bindings - simplified for MVP
            if pattern.data.len() < 4 {
                return None;
            }
            let idx = u32::from_le_bytes([
                pattern.data[0],
                pattern.data[1],
                pattern.data[2],
                pattern.data[3],
            ]) as usize;
            if let Some(name) = table.get_string(idx as u32) {
                if let auto_val::Value::Obj(obj) = msg {
                    if let Some(auto_val::Value::Str(v)) =
                        obj.get(auto_val::AutoStr::from("__variant"))
                    {
                        if name == v.as_str() {
                            return Some(crate::vm::pattern_matcher::MatchResult::empty());
                        }
                    }
                }
            }
            None
        }
    }
}

// ============================================================================
// Task Spawning and Scheduler Daemon
// ============================================================================

/// Dynamic task ID counter (starts at 1000 to distinguish from static IDs)
static DYNAMIC_TASK_ID: AtomicU64 = AtomicU64::new(1000);

/// Spawn a task context from a handle
///
/// Creates a TaskContext ready to run in its own tokio::spawn
pub fn spawn_task(
    meta: Arc<GlobalMeta>,
    task_type: String,
    instance_id: u64,
    _handle: TaskHandle,
    sys_tx: mpsc::Sender<SystemCommand>,
) -> TaskContext {
    // For MVP, we create a dummy receiver that will be replaced
    let (_dummy_tx, rx) = mpsc::channel::<auto_val::Value>(16);

    let task = AutoTask::new(instance_id, 1024, 0);

    TaskContext::new(meta, task_type, instance_id, rx, sys_tx, task)
}

/// Scheduler daemon loop - handles system commands
///
/// Runs until Stop command is received
pub async fn run_scheduler_daemon(
    meta: Arc<GlobalMeta>,
    mut sys_rx: mpsc::Receiver<SystemCommand>,
) {
    while let Some(cmd) = sys_rx.recv().await {
        match cmd {
            SystemCommand::Spawn {
                task_type,
                capacity,
                parent_id,
            } => {
                // Spawn a new task
                let _ = spawn_dynamic_task(meta.clone(), task_type, capacity, parent_id);
            }
            SystemCommand::Stop => {
                // Shutdown requested
                break;
            }
        }
    }
}

/// Spawn a dynamic task at runtime
fn spawn_dynamic_task(
    meta: Arc<GlobalMeta>,
    task_type: String,
    _capacity: usize,
    _parent_id: Option<u64>,
) -> Result<u64, String> {
    let instance_id = DYNAMIC_TASK_ID.fetch_add(1, Ordering::SeqCst);

    // Create task context (without actual mailbox for MVP)
    let (sys_tx, _sys_rx) = mpsc::channel::<SystemCommand>(16);
    let (_tx, rx) = mpsc::channel::<auto_val::Value>(16);
    let task = AutoTask::new(instance_id, 1024, 0);

    let mut ctx = TaskContext::new(meta, task_type.clone(), instance_id, rx, sys_tx, task);

    // Spawn the task loop
    tokio::spawn(async move {
        task_loop(&mut ctx).await;
    });

    Ok(instance_id)
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

    #[tokio::test]
    async fn test_execute_handler_fully_returns_on_ret() {
        let bytecode = VirtualFlash {
            memory: vec![0x00], // NOP opcode - handler should complete
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            exports_by_name: HashMap::new(),
        };

        let meta = Arc::new(GlobalMeta::from_components(
            bytecode,
            Vec::new(),
            NativeInterface::new(),
            HashMap::new(),
        ));

        let mut task = AutoTask::new(1, 1024, 0);
        task.ip = 0;

        // execute_handler_fully should complete without error
        let result = execute_handler_fully(&meta, &mut task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_loop_processes_messages() {
        let bytecode = VirtualFlash {
            memory: vec![0x00], // NOP
            symbol_map: HashMap::new(),
            object_keys: Vec::new(),
            object_types: Vec::new(),
            exports_by_name: HashMap::new(),
        };

        let meta = Arc::new(GlobalMeta::from_components(
            bytecode,
            Vec::new(),
            NativeInterface::new(),
            HashMap::new(),
        ));

        let (tx, rx) = mpsc::channel::<auto_val::Value>(16);
        let (sys_tx, _sys_rx) = mpsc::channel::<SystemCommand>(16);

        let task = AutoTask::new(1, 1024, 0);
        let mut ctx = TaskContext::new(
            meta,
            "TestTask".to_string(),
            1,
            rx,
            sys_tx,
            task,
        );

        // Send a message
        tx.send(auto_val::Value::Int(42)).await.unwrap();

        // Drop sender to close mailbox after this message
        drop(tx);

        // Run task loop - should process one message then exit
        task_loop(&mut ctx).await;

        // Task should have terminated (mailbox closed)
        assert_eq!(ctx.task.status, TaskStatus::Terminated);
    }

    #[tokio::test]
    async fn test_spawn_task_creates_context() {
        let meta = Arc::new(GlobalMeta::new());
        let (sys_tx, _sys_rx) = mpsc::channel::<SystemCommand>(16);

        let (tx, _rx) = mpsc::channel::<auto_val::Value>(16);
        let handle = TaskHandle::new("TestTask".to_string(), 1, tx);

        let ctx = spawn_task(
            Arc::clone(&meta),
            "TestTask".to_string(),
            1,
            handle,
            sys_tx,
        );

        assert_eq!(ctx.task_type, "TestTask");
        assert_eq!(ctx.instance_id, 1);
        // Note: handle is intentionally consumed by spawn_task
    }

    #[tokio::test]
    async fn test_scheduler_daemon_handles_stop() {
        let meta = Arc::new(GlobalMeta::new());
        let (sys_tx, sys_rx) = mpsc::channel::<SystemCommand>(16);

        // Send stop command immediately
        sys_tx.send(SystemCommand::Stop).await.unwrap();

        // Run daemon loop - should exit immediately
        run_scheduler_daemon(meta, sys_rx).await;
    }
}
