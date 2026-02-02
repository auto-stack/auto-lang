use crate::vm::channel::{AutoChannel, ChannelId};
use crate::vm::native::NativeInterface;
/// BigVM Execution Engine
/// The core loop that executes AutoByteCode (ABC).
use crate::vm::opcode::OpCode;
use crate::vm::task::{AutoTask, TaskId, TaskStatus};
use crate::vm::virt_memory::{VirtualFlash, VirtualRAM};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    StackUnderflow,
    InvalidOpCode(u8),
    DivisionByZero,
    Halt,
    MissingNative(u16),
    RuntimeError(String),
}

pub struct BigVM {
    pub flash: Arc<VirtualFlash>,
    pub native_interface: Arc<NativeInterface>,
    /// String constant pool
    pub strings: Arc<Vec<Vec<u8>>>,

    pub tasks: DashMap<TaskId, Arc<Mutex<AutoTask>>>,
    pub id_gen: AtomicU64,

    // Channel Registry
    pub channels: DashMap<ChannelId, Arc<AutoChannel>>,
    pub channel_id_gen: AtomicU64,
}

impl BigVM {
    pub fn new(flash: VirtualFlash, _ram_size: usize) -> Self {
        let mut native_interface = NativeInterface::new();
        native_interface.register_std_shims();
        Self {
            flash: Arc::new(flash),
            native_interface: Arc::new(native_interface),
            strings: Arc::new(Vec::new()),
            tasks: DashMap::new(),
            id_gen: AtomicU64::new(0),
            channels: DashMap::new(),
            channel_id_gen: AtomicU64::new(0),
        }
    }

    /// Load strings from a module's string constant pool
    pub fn load_strings(&mut self, strings: Vec<Vec<u8>>) {
        self.strings = Arc::new(strings);
    }

    /// Spawn a new task starting at the given instruction pointer
    /// Returns the TaskId
    pub fn spawn_task(&self, start_ip: usize, ram_size: usize) -> TaskId {
        let id = self.id_gen.fetch_add(1, Ordering::Relaxed);
        let task = AutoTask::new(id, ram_size, start_ip);
        self.tasks.insert(id, Arc::new(Mutex::new(task)));
        id
    }

    /// Get string by index from the constant pool
    pub fn get_string(&self, index: u16) -> Option<&[u8]> {
        self.strings.get(index as usize).map(|v| v.as_slice())
    }

    /// The main async loop that schedules and runs tasks.
    pub async fn run_task_loop(&self) {
        loop {
            let mut active_count = 0;
            let mut alive_count = 0;

            // Collect tasks to iterate
            // We use a Vec of Arcs to avoid holding the map lock during execution
            let tasks: Vec<(TaskId, Arc<Mutex<AutoTask>>)> = self
                .tasks
                .iter()
                .map(|r| (*r.key(), r.value().clone()))
                .collect();

            if tasks.is_empty() {
                break; // No tasks left, exit VM
            }

            for (_id, task_mutex) in tasks {
                let mut task = task_mutex.lock().await;

                if task.status == TaskStatus::Terminated {
                    continue;
                }

                // Check if sleeping task should wake up
                if let Some(wake_time) = task.wake_time {
                    if Instant::now() >= wake_time {
                        task.wake_time = None;
                        task.status = TaskStatus::Ready;
                    } else {
                        alive_count += 1;
                        continue; // Still sleeping
                    }
                }

                alive_count += 1;

                // Check if task is runnable
                if task.status != TaskStatus::Running && task.status != TaskStatus::Ready {
                    continue;
                }

                active_count += 1;
                task.status = TaskStatus::Running;

                // Run a chunk of instructions
                match self.execute_task(&mut task) {
                    Ok(new_status) => {
                        task.status = new_status;
                    }
                    Err(e) => {
                        println!("Task {} Error: {:?}", task.id, e);
                        task.status = TaskStatus::Terminated;
                    }
                }
            }

            // Cleanup terminated tasks
            // This is a simplified garbage collection for MVP
            /*
            self.tasks.retain(|_, v| {
                // We need to try_lock to avoid deadlocks or blocking?
                // Since we are single-threaded loop essentially here (sequential iteration),
                // blocking_lock or try_lock is fine if no one else holds it.
                // But wait, if we are in async context, blocking_lock is bad.
                // However, we cloned the Arcs above, so we don't hold the map lock.
                // Re-acquiring lock here is okay.
                if let Ok(task) = v.try_lock() {
                    task.status != TaskStatus::Terminated
                } else {
                    true // Keep it if locked (should be rare/impossible in this simple loop)
                }
            });
            */

            if alive_count == 0 {
                break;
            }

            if active_count == 0 {
                if self.tasks.is_empty() {
                    break;
                }
                // All tasks waiting/sleeping?
                sleep(Duration::from_millis(10)).await;
            }

            // Yield to tokio runtime to let other things happen
            tokio::task::yield_now().await;
        }
    }

    /// Execute a chunk of opcodes for a specific task
    fn execute_task(&self, task: &mut AutoTask) -> Result<TaskStatus, VMError> {
        let budget = 100; // OpCode Budget
        let mut ops_executed = 0;

        while ops_executed < budget {
            // 1. Fetch
            if task.ip >= self.flash.memory.len() {
                return Ok(TaskStatus::Terminated);
            }

            let op_byte = self.flash.read_u8(task.ip);
            task.ip += 1;

            let op: OpCode = op_byte.into();

            // 2. Decode & Execute
            match op {
                OpCode::NOP => {
                    // Do nothing
                }
                OpCode::POP => {
                    task.ram.pop_i32();
                }
                OpCode::DUP => {
                    let val = task.ram.top().unwrap_or(0);
                    task.ram.push_i32(val);
                }

                // === Constants ===
                OpCode::CONST_I32 => {
                    let val = self.flash.read_i32(task.ip);
                    task.ip += 4;
                    task.ram.push_i32(val);
                }
                OpCode::CONST_F32 => {
                    let val = self.flash.read_i32(task.ip);
                    task.ip += 4;
                    task.ram.push_i32(val);
                }
                OpCode::CONST_0 => {
                    task.ram.push_i32(0);
                }
                OpCode::CONST_1 => {
                    task.ram.push_i32(1);
                }
                OpCode::LOAD_STR => {
                    let str_idx = self.flash.read_u16(task.ip);
                    task.ip += 2;
                    task.ram.push_i32(str_idx as i32);
                }
                // === Arithmetic ===
                OpCode::ADD => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a.wrapping_add(b));
                }
                OpCode::SUB => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a.wrapping_sub(b));
                }
                OpCode::MUL => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a.wrapping_mul(b));
                }
                OpCode::DIV => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_i32(a.wrapping_div(b));
                }

                // === Control Flow ===
                OpCode::NEG => {
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a.wrapping_neg());
                }
                OpCode::NOT => {
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(!a);
                }
                OpCode::CALL => {
                    let target = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;

                    // Push Return Address (IP)
                    task.ram.push_i32(task.ip as i32);
                    // Push Old Stack Frame (BP)
                    task.ram.push_i32(task.bp as i32);

                    // New BP points to the saved BP location (SP - 1)
                    task.bp = task.ram.sp - 1;

                    // Jump
                    task.ip = target;
                }
                OpCode::CALL_NAT => {
                    let native_id = self.flash.read_u16(task.ip);
                    task.ip += 2;

                    // Execute Native Shim
                    let shim = self.native_interface.get(native_id).cloned();

                    if let Some(shim) = shim {
                        // Pass task and vm
                        shim(task, self)?;
                    } else {
                        return Err(VMError::MissingNative(native_id));
                    }
                }
                OpCode::RET => {
                    // Spec: RET n_args
                    let n_args = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    // Check if we're in the main task (bp == 0 means no caller)
                    if task.bp == 0 {
                        // Main task returning - just terminate
                        return Ok(TaskStatus::Terminated);
                    }

                    // Expect Result on Top of Stack
                    let result = task.ram.pop_i32();

                    let old_bp = task.ram.read_i32(task.bp) as usize;
                    let ret_ip = task.ram.read_i32(task.bp - 1) as usize;

                    let new_sp = task.bp - n_args;

                    // Safety check for underflow
                    if task.bp < n_args {
                        // In valid stack frame logic, bp should be >= args_count if args were pushed before call.
                        // But actually logic depends on calling convention.
                        // Assuming simple verification for now.
                    }

                    task.ram.write_i32(new_sp - 1, result);

                    task.bp = old_bp;
                    task.ip = ret_ip;
                    task.ram.sp = new_sp;
                    task.ram.write_i32(new_sp - 1, result); // Write Result confirmed
                }

                // === Concurrency ===
                OpCode::SPAWN => {
                    let target = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;
                    let arg_count = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    let mut args = Vec::new();
                    for _ in 0..arg_count {
                        args.push(task.ram.pop_i32());
                    }

                    let new_task_id = self.spawn_task(target, 1024);

                    if let Some(new_task_arc) = self.tasks.get(&new_task_id) {
                        if let Ok(mut new_task) = new_task_arc.try_lock() {
                            // Push args in reverse order (A, B, C)
                            for arg in args.into_iter().rev() {
                                new_task.ram.push_i32(arg);
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Failed to lock spawned task {}",
                                new_task_id
                            )));
                        }
                    }
                    task.ram.push_i32(new_task_id as i32);
                }
                OpCode::TASK_ID => {
                    task.ram.push_i32(task.id as i32);
                }
                OpCode::YIELD => {
                    return Ok(TaskStatus::Ready);
                }
                OpCode::SLEEP => {
                    let ms = self.flash.read_u32(task.ip) as u64;
                    task.ip += 4;

                    // Set wake time
                    task.wake_time = Some(Instant::now() + std::time::Duration::from_millis(ms));
                    task.status = TaskStatus::Waiting(format!("sleep for {}ms", ms));
                    return Ok(task.status.clone());
                }
                OpCode::JOIN => {
                    let target_task_id = task.ram.pop_i32() as u64;

                    // Get Arc first (must outlive the try_lock call)
                    let target_task_opt: Option<Arc<Mutex<AutoTask>>> =
                        self.tasks.get(&target_task_id).map(|r| r.value().clone());

                    let join_result: Option<(bool, i32)> = match &target_task_opt {
                        Some(target_task) => {
                            match target_task.try_lock() {
                                Ok(target) => {
                                    if target.status == TaskStatus::Terminated {
                                        Some((true, target.ram.top().unwrap_or(0)))
                                    } else {
                                        Some((false, 0))
                                    }
                                }
                                Err(_) => None, // Couldn't lock
                            }
                        }
                        None => Some((true, 0)), // Task not found, return 0
                    };

                    match join_result {
                        Some((true, result)) => {
                            task.ram.push_i32(result);
                        }
                        Some((false, _)) | None => {
                            // Task still running or lock failed, yield and retry
                            task.ip -= 1;
                            task.ram.push_i32(target_task_id as i32);
                            return Ok(TaskStatus::Ready);
                        }
                    }
                }
                OpCode::CHAN_NEW => {
                    let id = self.channel_id_gen.fetch_add(1, Ordering::Relaxed) as u32;
                    let chan = Arc::new(AutoChannel::new(id, 16));
                    self.channels.insert(id, chan);
                    task.ram.push_i32(id as i32);
                }
                OpCode::SEND => {
                    let data = task.ram.pop_i32();
                    let chan_id = task.ram.pop_i32() as u32;
                    let mut success = false;
                    let mut closed = false;

                    if let Some(chan_ref) = self.channels.get(&chan_id) {
                        let chan = chan_ref.value().clone();
                        drop(chan_ref);
                        match chan.tx.try_send(data) {
                            Ok(_) => success = true,
                            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                                // Channel full
                            }
                            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                closed = true;
                            }
                        }
                    } else {
                        closed = true;
                    }

                    if !success && !closed {
                        // Retry later
                        task.ip -= 1;
                        task.ram.push_i32(chan_id as i32);
                        task.ram.push_i32(data);
                        return Ok(TaskStatus::Ready);
                    }
                }
                OpCode::RECV => {
                    let chan_id = task.ram.pop_i32() as u32;
                    let mut success = false;
                    let mut val = 0;
                    let mut closed = false;
                    match self.channels.get(&chan_id) {
                        Some(chan_ref) => {
                            let chan = chan_ref.value().clone();
                            drop(chan_ref);
                            // Lock rx
                            let mut rx = chan.rx.lock().unwrap();
                            match rx.try_recv() {
                                Ok(v) => {
                                    val = v;
                                    success = true;
                                }
                                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                                    // Empty
                                }
                                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                                    closed = true;
                                }
                            }
                        }
                        None => {
                            closed = true; // Invalid = closed
                            val = -1; // Error code?
                        }
                    }

                    if success {
                        task.ram.push_i32(val);
                    } else if closed {
                        task.ram.push_i32(0); // TODO: Null/None
                    } else {
                        // Empty, Retry
                        task.ip -= 1;
                        task.ram.push_i32(chan_id as i32);
                        return Ok(TaskStatus::Ready);
                    }
                }
                OpCode::TRY_RECV => {
                    let chan_id = task.ram.pop_i32() as u32;
                    let mut success = false;
                    let mut val = 0;
                    let mut closed = false;
                    match self.channels.get(&chan_id) {
                        Some(chan_ref) => {
                            let chan = chan_ref.value().clone();
                            drop(chan_ref);
                            // Lock rx
                            let mut rx = chan.rx.lock().unwrap();
                            match rx.try_recv() {
                                Ok(v) => {
                                    val = v;
                                    success = true;
                                }
                                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                                    // Empty - return 0 without blocking
                                }
                                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                                    closed = true;
                                }
                            }
                        }
                        None => {
                            closed = true; // Invalid = closed
                            val = -1; // Error code?
                        }
                    }

                    if success {
                        task.ram.push_i32(val);
                    } else if closed {
                        task.ram.push_i32(0); // TODO: Null/None
                    } else {
                        // Empty channel - return 0 immediately (non-blocking)
                        task.ram.push_i32(0);
                    }
                }

                // === Local Variables ===
                OpCode::LOAD_LOCAL => {
                    let idx = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;
                    let val = task.ram.read_i32(task.bp + idx);
                    task.ram.push_i32(val);
                }
                OpCode::STORE_LOCAL => {
                    let idx = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;
                    let val = task.ram.pop_i32();
                    task.ram.write_i32(task.bp + idx, val);
                }
                OpCode::LOAD_LOC_0 => {
                    let val = task.ram.read_i32(task.bp + 0);
                    task.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_1 => {
                    let val = task.ram.read_i32(task.bp + 1);
                    task.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_2 => {
                    let val = task.ram.read_i32(task.bp + 2);
                    task.ram.push_i32(val);
                }
                OpCode::STORE_LOC_0 => {
                    let val = task.ram.pop_i32();
                    task.ram.write_i32(task.bp + 0, val);
                }
                OpCode::STORE_LOC_1 => {
                    let val = task.ram.pop_i32();
                    task.ram.write_i32(task.bp + 1, val);
                }

                // === Stack ===
                OpCode::POP => {
                    task.ram.pop_i32();
                }
                OpCode::DROP => {
                    task.ram.pop_i32();
                }

                // === Comparison ===
                OpCode::EQ => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a == b { 1 } else { 0 });
                }
                OpCode::NE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a != b { 1 } else { 0 });
                }
                OpCode::LT => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a < b { 1 } else { 0 });
                }
                OpCode::GT => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a > b { 1 } else { 0 });
                }
                OpCode::LE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a <= b { 1 } else { 0 });
                }
                OpCode::GE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a >= b { 1 } else { 0 });
                }

                // === Control Flow ===
                OpCode::JMP => {
                    let offset = self.flash.read_i16(task.ip) as isize;
                    task.ip += 2;

                    let new_ip = (task.ip as isize) + offset;

                    if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                        return Err(VMError::InvalidOpCode(0xFF));
                    }

                    task.ip = new_ip as usize;
                }
                OpCode::JMP_IF_Z => {
                    let offset = self.flash.read_i16(task.ip) as isize;
                    task.ip += 2;

                    let cond = task.ram.pop_i32();
                    if cond == 0 {
                        let new_ip = (task.ip as isize) + offset;
                        if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                            return Err(VMError::InvalidOpCode(0xFF));
                        }
                        task.ip = new_ip as usize;
                    }
                }
                OpCode::JMP_IF_NZ => {
                    let offset = self.flash.read_i16(task.ip) as isize;
                    task.ip += 2;

                    let cond = task.ram.pop_i32();
                    if cond != 0 {
                        let new_ip = (task.ip as isize) + offset;
                        if new_ip < 0 || new_ip as usize >= self.flash.memory.len() {
                            return Err(VMError::InvalidOpCode(0xFF));
                        }
                        task.ip = new_ip as usize;
                    }
                }

                // === Debug ===
                OpCode::HALT => {
                    return Ok(TaskStatus::Terminated);
                }

                _ => {
                    // Unimplemented opcodes for Phase 1
                    return Err(VMError::InvalidOpCode(op_byte));
                }
            }

            ops_executed += 1;
        }

        Ok(TaskStatus::Ready)
    }
}
