use crate::vm::virt_memory::VirtualRAM;
use std::time::Instant;

pub type TaskId = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Waiting(String), // Reason for waiting
    Terminated,
}

/// Plan 118: Track result type for proper output formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResultType {
    #[default]
    Int,
    Float,
    Byte,
    Uint,
}

/// Plan 199: Structured call frame for stack trace debugging
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_ip: usize,
    pub old_bp: usize,
    pub fn_name: Option<String>,
    pub line: u32,
    pub old_fn_n_args: usize,
    pub old_fn_n_locals: usize,
}

/// Represents a single concurrent task in the AutoVM
/// Holds its own stack, instruction pointer, and execution state.
pub struct AutoTask {
    pub id: TaskId,
    pub ram: VirtualRAM,
    pub ip: usize,
    pub bp: usize, // Base Pointer
    pub prev_sp: usize, // Debug: previous sp for tracking drift
    pub num_locals: usize, // Number of local variables in current stack frame
    pub status: TaskStatus,
    pub wake_time: Option<Instant>, // For SLEEP opcode
    pub current_closure_id: Option<u32>, // Plan 071: Current closure being executed
    pub saved_closure_id: Option<u32>,   // Saved closure ID for restoration on RET
    // Plan 088 Phase 4: Function metadata from FN_PROLOG instruction
    pub current_fn_n_args: usize, // Number of arguments in current function
    pub current_fn_n_locals: usize, // Number of local variables in current function (from prologue)
    // Plan 117/118: Track result type for proper output formatting
    pub last_result_type: ResultType,
    // Plan 118: Store last error for proper error propagation
    pub last_error: Option<String>,
    // Plan 127: Task message loop support
    pub in_message_loop: bool, // Whether task is in message processing loop
    pub task_type_name: Option<String>, // Task type name for handler lookup
    pub current_handler_has_context: bool, // Whether current handler has ctx parameter
    pub current_msg_context: Option<crate::vm::message_context::MessageContext>, // Current reply context
    // Plan 327: Actor state fields (persist across handler invocations).
    // Indexed by field_idx (assigned by codegen per task type). Stored here
    // (not in ram) so they survive handler RET and are independent of bp.
    // Accessed via LOAD_STATE_FIELD / STORE_STATE_FIELD opcodes.
    pub state_vars: Vec<auto_val::NanoValue>,
    // Plan 199: Source line tracking for debugging
    pub current_line: u32,
    pub current_source: Option<String>,
    // Plan 199: Structured call stack for debugging
    pub call_stack: Vec<CallFrame>,
}

impl AutoTask {
    pub fn new(id: TaskId, ram_size: usize, start_ip: usize) -> Self {
        Self {
            id,
            ram: VirtualRAM::new(ram_size),
            ip: start_ip,
            bp: 0,
            prev_sp: 0,
            num_locals: 0,
            status: TaskStatus::Ready,
            wake_time: None,
            current_closure_id: None,
            saved_closure_id: None,
            current_fn_n_args: 0, // Plan 088 Phase 4: Initialize to 0
            current_fn_n_locals: 0, // Plan 088 Phase 4: Initialize to 0
            last_result_type: ResultType::default(), // Plan 118: Initialize to Int
            last_error: None, // Plan 118: Initialize to None
            in_message_loop: false,
            task_type_name: None,
            current_handler_has_context: false,
            current_msg_context: None,
            state_vars: Vec::new(), // Plan 327: actor state fields
            current_line: 0,
            current_source: None,
            call_stack: Vec::new(),
        }
    }
}
