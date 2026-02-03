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
    pub status: TaskStatus,
    pub wake_time: Option<Instant>, // For SLEEP opcode
    pub current_closure_id: Option<u32>, // Plan 071: Current closure being executed
}

impl AutoTask {
    pub fn new(id: TaskId, ram_size: usize, start_ip: usize) -> Self {
        Self {
            id,
            ram: VirtualRAM::new(ram_size),
            ip: start_ip,
            bp: 0,
            status: TaskStatus::Ready,
            wake_time: None,
            current_closure_id: None,
        }
    }
}
