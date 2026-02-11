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

/// Represents a single concurrent task in the AutoVM
/// Holds its own stack, instruction pointer, and execution state.
pub struct AutoTask {
    pub id: TaskId,
    pub ram: VirtualRAM,
    pub ip: usize,
    pub bp: usize, // Base Pointer
    pub num_locals: usize, // Number of local variables in current stack frame
    pub status: TaskStatus,
    pub wake_time: Option<Instant>, // For SLEEP opcode
    pub current_closure_id: Option<u32>, // Plan 071: Current closure being executed
    // Plan 088 Phase 4: Function metadata from FN_PROLOG instruction
    pub current_fn_n_args: usize, // Number of arguments in current function
    pub current_fn_n_locals: usize, // Number of local variables in current function (from prologue)
}

impl AutoTask {
    pub fn new(id: TaskId, ram_size: usize, start_ip: usize) -> Self {
        Self {
            id,
            ram: VirtualRAM::new(ram_size),
            ip: start_ip,
            bp: 0,
            num_locals: 0,
            status: TaskStatus::Ready,
            wake_time: None,
            current_closure_id: None,
            current_fn_n_args: 0, // Plan 088 Phase 4: Initialize to 0
            current_fn_n_locals: 0, // Plan 088 Phase 4: Initialize to 0
        }
    }
}
