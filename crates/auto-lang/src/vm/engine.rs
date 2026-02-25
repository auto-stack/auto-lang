use crate::vm::channel::{AutoChannel, ChannelId};
use crate::vm::codegen::ObjectType;
use crate::vm::heap_object::HeapObject;
use crate::vm::native::NativeInterface;
use crate::vm::opcode::OpCode;
use crate::vm::task::{AutoTask, TaskId, TaskStatus};
use crate::vm::virt_memory::{VirtualFlash, VirtualRAM};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

/// List iterator state
#[derive(Debug, Clone)]
pub struct ListIterator {
    pub list_id: u64,
    pub current_index: u32,
}

/// Map iterator state - wraps a source iterator and applies a function
#[derive(Debug, Clone)]
pub struct MapIterator {
    pub source_iterator_id: u32,
    pub func_addr: u32,  // Address of the function to call
}

/// Filter iterator state - wraps a source iterator and applies a predicate
#[derive(Debug, Clone)]
pub struct FilterIterator {
    pub source_iterator_id: u32,
    pub func_addr: u32,  // Address of the predicate function
}

/// Unified iterator type
#[derive(Debug, Clone)]
pub enum Iterator {
    List(ListIterator),
    Map(MapIterator),
    Filter(FilterIterator),
}

// ============================================================================
// Closures (Plan 071: Direct Capture)
// ============================================================================

use auto_val::Value;

/// Closure - a function value with directly captured environment (Plan 071: Direct Capture)
#[derive(Debug, Clone)]
pub struct Closure {
    pub func_addr: u32,                        // Bytecode address
    pub env: HashMap<String, Value>,           // Direct captured values (no upvalues!)
}


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

pub struct AutoVM {
    pub flash: Arc<VirtualFlash>,
    pub native_interface: Arc<NativeInterface>,
    /// String constant pool (Plan 073: Made mutable for runtime string field access)
    pub strings: Arc<RwLock<Vec<Vec<u8>>>>,

    pub tasks: DashMap<TaskId, Arc<Mutex<AutoTask>>>,
    pub id_gen: AtomicU64,

    // Channel Registry
    pub channels: DashMap<ChannelId, Arc<AutoChannel>>,
    pub channel_id_gen: AtomicU64,

    // Plan 077 Phase 6: Legacy list registry removed - all lists now use unified heap_objects registry

    // Iterator Registry
    pub iterators: DashMap<u32, Iterator>,
    pub iterator_id_gen: AtomicU32,

    // Closure Registry (Plan 071: Direct Capture, no upvalues)
    pub closures: DashMap<u32, Closure>,
    pub closure_id_gen: AtomicU32,

    // Object Registry (Plan 073: Object literals)
    pub objects: DashMap<u64, Arc<RwLock<crate::vm::types::ObjectData>>>,
    pub object_id_gen: AtomicU64,

    // Array Registry (Plan 073: Array literals)
    pub arrays: DashMap<u64, Arc<RwLock<Vec<auto_val::Value>>>>,
    pub array_id_gen: AtomicU64,

    // Node Registry (Plan 073: Node instances for type construction)
    pub nodes: DashMap<u64, Arc<RwLock<auto_val::Node>>>,
    pub node_id_gen: AtomicU64,

    // Plan 077 Phase 4: Unified Object Registry
    // Single registry for all heap-allocated objects (lists, maps, sets, etc.)
    pub heap_objects: DashMap<u64, Arc<RwLock<dyn HeapObject>>>,
    pub heap_object_id_gen: AtomicU64,
}

impl AutoVM {
    pub fn new(flash: VirtualFlash, _ram_size: usize) -> Self {
        let mut native_interface = NativeInterface::new();
        native_interface.register_std_shims();
        Self {
            flash: Arc::new(flash),
            native_interface: Arc::new(native_interface),
            strings: Arc::new(RwLock::new(Vec::new())),
            tasks: DashMap::new(),
            id_gen: AtomicU64::new(0),
            channels: DashMap::new(),
            channel_id_gen: AtomicU64::new(0),
            // Plan 077 Phase 6: Legacy list registry removed
            iterators: DashMap::new(),
            iterator_id_gen: AtomicU32::new(0),
            closures: DashMap::new(),
            closure_id_gen: AtomicU32::new(0),
            // Note: IDs start at 1000000 to avoid confusion with small integer values
            objects: DashMap::new(),
            object_id_gen: AtomicU64::new(1000000),
            // Plan 073: Array registry
            // Note: IDs start at 1000000 to avoid confusion with small integer values
            arrays: DashMap::new(),
            array_id_gen: AtomicU64::new(1000000),
            // Plan 073: Node registry
            // Note: IDs start at 1000000 to avoid confusion with small integer values
            nodes: DashMap::new(),
            node_id_gen: AtomicU64::new(1000000),
            // Plan 077 Phase 4: Unified object registry
            // Note: IDs start at 1000000 to avoid confusion with small integer values
            heap_objects: DashMap::new(),
            heap_object_id_gen: AtomicU64::new(1000000),
        }
    }

    /// Load strings from a module's string constant pool
    pub fn load_strings(&mut self, strings: Vec<Vec<u8>>) {
        self.strings = Arc::new(RwLock::new(strings));
    }

    // ============================================================================
    // Plan 077 Phase 4: Unified Object Registry Helper Methods
    // ============================================================================

    /// Insert a heap-allocated object into the unified registry
    ///
    /// Returns the object ID assigned to this object.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use auto_lang::universe::ListData;
    /// use auto_lang::vm::engine::AutoVM;
    ///
    /// let mut list: ListData<i32> = ListData::new();
    /// list.push(1);
    /// list.push(2);
    ///
    /// let id = vm.insert_heap_object(list);
    /// ```
    pub fn insert_heap_object<T: HeapObject + Send + Sync + 'static>(&self, obj: T) -> u64 {
        let id = self.heap_object_id_gen.fetch_add(1, Ordering::Relaxed);
        self.heap_objects.insert(id, Arc::new(RwLock::new(obj)));
        id
    }

    /// Get a heap object by ID, returning a read guard
    ///
    /// Returns `None` if the object doesn't exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use auto_lang::vm::heap_object::downcast;
    /// use auto_lang::universe::ListData;
    ///
    /// if let Some(obj) = vm.get_heap_object(id) {
    ///     let guard = obj.read().unwrap();
    ///     if let Some(list) = downcast::<ListData<i32>>(&*guard) {
    ///         println!("Got list with {} elements", list.len());
    ///     }
    /// }
    /// ```
    pub fn get_heap_object(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        self.heap_objects.get(&id).map(|v| v.clone())
    }

    /// Get a heap object by ID with mutable access
    ///
    /// Returns `None` if the object doesn't exist.
    pub fn get_heap_object_mut(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        self.heap_objects.get(&id).map(|v| v.clone())
    }

    /// Remove a heap object from the registry
    ///
    /// Returns `None` if the object doesn't exist.
    pub fn remove_heap_object(&self, id: u64) -> Option<Arc<RwLock<dyn HeapObject>>> {
        self.heap_objects.remove(&id).map(|(_, v)| v)
    }

    /// Get the number of heap objects in the registry
    pub fn heap_object_count(&self) -> usize {
        self.heap_objects.len()
    }

    /// Check if a heap object exists in the registry
    pub fn contains_heap_object(&self, id: u64) -> bool {
        self.heap_objects.contains_key(&id)
    }

    /// Clear all heap objects from the registry
    pub fn clear_heap_objects(&self) {
        self.heap_objects.clear();
    }

    // ============================================================================
    // Plan 087 Phase 2: Generic Instance Value Helper Functions
    // ============================================================================

    /// Push a Value onto the stack based on its type
    ///
    /// For Phase 2, supports: Int, Uint, Float, Double, Bool, Char, Nil, Str
    fn push_value(ram: &mut VirtualRAM, value: &Value, strings: &std::sync::Arc<RwLock<Vec<Vec<u8>>>>) {
        match value {
            Value::Int(i) => ram.push_i32(*i),
            Value::Uint(u) => ram.push_i32(*u as i32),
            Value::Float(f) => ram.push_f32(*f as f32),
            Value::Double(d) => ram.push_f64(*d),
            Value::Bool(b) => ram.push_i32(if *b { 1 } else { 0 }),
            Value::Char(c) => ram.push_i32(*c as i32),
            Value::Nil => ram.push_i32(0),
            Value::Str(s) => {
                // Store string in constant pool and push its index
                let s_bytes = s.as_bytes().to_vec();
                let mut strings = strings.write().unwrap();
                let idx = strings.len();
                strings.push(s_bytes);
                drop(strings);
                ram.push_i32(idx as i32);
            }
            _ => {
                eprintln!("WARNING: push_value unsupported type: {:?}", value);
                ram.push_i32(0);
            }
        }
    }

    /// Pop a basic value from the stack as i32
    ///
    /// For Phase 2, assumes the value is a basic type (int, bool, char, nil)
    fn pop_value_as_int(ram: &mut VirtualRAM) -> i32 {
        ram.pop_i32()
    }

    /// Pop a float value from the stack as f32
    fn pop_value_as_float(ram: &mut VirtualRAM) -> f32 {
        ram.pop_f32()
    }

    /// Pop a double value from the stack as f64
    fn pop_value_as_double(ram: &mut VirtualRAM) -> f64 {
        ram.pop_f64()
    }

    /// Pop a string value from the stack (returns string index)
    fn pop_value_as_string_index(ram: &mut VirtualRAM) -> i32 {
        ram.pop_i32()
    }

    // ============================================================================
    // End Plan 087 Phase 2
    // ============================================================================

    /// Spawn a new task starting at the given instruction pointer
    /// Returns the TaskId
    pub fn spawn_task(&self, start_ip: usize, ram_size: usize) -> TaskId {
        let id = self.id_gen.fetch_add(1, Ordering::Relaxed);
        let task = AutoTask::new(id, ram_size, start_ip);
        self.tasks.insert(id, Arc::new(Mutex::new(task)));
        id
    }

    /// Get string by index from the constant pool
    pub fn get_string(&self, index: u16) -> Option<Vec<u8>> {
        let strings = self.strings.read().unwrap();
        strings.get(index as usize).cloned()
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
                    eprintln!("DEBUG: CONST_I32: val={}, sp before push={}", val, task.ram.sp);
                    task.ram.push_i32(val);
                    eprintln!("DEBUG: CONST_I32: sp after push={}, wrote to address {}", task.ram.sp, task.ram.sp - 1);
                }
                OpCode::CONST_F32 => {
                    // Plan 073: Fixed to use push_f32 instead of push_i32
                    let val = self.flash.read_f32(task.ip);
                    task.ip += 4;
                    task.ram.push_f32(val);
                }
                OpCode::CONST_F64 => {
                    // Plan 073: Double precision constant
                    let val = self.flash.read_f64(task.ip);
                    task.ip += 8;
                    task.ram.push_f64(val);
                }
                OpCode::CONST_I64 => {
                    // Plan 073: 64-bit integer constant
                    let val = self.flash.read_i64(task.ip);
                    task.ip += 8;
                    task.ram.push_i64(val);
                }
                OpCode::CONST_U64 => {
                    // Plan 073: 64-bit unsigned integer constant
                    let val = self.flash.read_u64(task.ip);
                    task.ip += 8;
                    task.ram.push_u64(val);
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
                // Plan 073: Object literal support
                OpCode::CREATE_OBJ => {
                    let key_index = self.flash.read_u16(task.ip);
                    task.ip += 2;
                    let field_count = self.flash.read_u8(task.ip);
                    task.ip += 1;

                    // Get keys from flash metadata
                    let keys = &self.flash.object_keys[key_index as usize];
                    // Get types from flash metadata
                    let types = &self.flash.object_types[key_index as usize];

                    // Pop values from stack (in reverse order since last value is on top)
                    // Convert each value based on its type
                    let mut values = Vec::with_capacity(field_count as usize);
                    for i in 0..field_count {
                        let type_idx = (field_count - 1 - i) as usize;
                        let obj_type = types.get(type_idx).copied().unwrap_or(ObjectType::Int);

                        let value = match obj_type {
                            ObjectType::Int => {
                                let bits = task.ram.pop_i32();
                                auto_val::Value::Int(bits)
                            }
                            ObjectType::Uint => {
                                let bits = task.ram.pop_i32();
                                auto_val::Value::Uint(bits as u32)
                            }
                            ObjectType::Float => {
                                let bits = task.ram.pop_f32();
                                auto_val::Value::Float(bits as f64)
                            }
                            ObjectType::Double => {
                                let bits = task.ram.pop_f64();
                                auto_val::Value::Double(bits)
                            }
                            ObjectType::String => {
                                let str_idx = task.ram.pop_i32() as usize;
                                let strings = self.strings.read().unwrap();
                                if let Some(str_bytes) = strings.get(str_idx) {
                                    let s = String::from_utf8_lossy(str_bytes).to_string();
                                    auto_val::Value::Str(s.into())
                                } else {
                                    auto_val::Value::Nil
                                }
                            }
                            ObjectType::Bool => {
                                let bits = task.ram.pop_i32();
                                auto_val::Value::Bool(bits != 0)
                            }
                            ObjectType::Char => {
                                let bits = task.ram.pop_i32();
                                if let Some(c) = char::from_u32(bits as u32) {
                                    auto_val::Value::Char(c)
                                } else {
                                    auto_val::Value::Nil
                                }
                            }
                            // Plan 073: Nested object field - store object ID as VmRef
                            ObjectType::NestedObject => {
                                let nested_id = task.ram.pop_i32() as usize;
                                auto_val::Value::VmRef(auto_val::VmRef { id: nested_id })
                            }
                            // Plan 073: Array field - store as VmRef (for when arrays are implemented)
                            ObjectType::Array => {
                                let array_id = task.ram.pop_i32() as usize;
                                auto_val::Value::VmRef(auto_val::VmRef { id: array_id })
                            }
                        };
                        values.push(value);
                    }

                    // Create object from key-value pairs
                    let mut obj = crate::vm::types::ObjectData::new();
                    for (i, key) in keys.iter().enumerate() {
                        // Values were popped in reverse order, so reverse them back
                        let val = &values[field_count as usize - 1 - i];
                        obj.set(key.clone(), val.clone());
                    }

                    // Store object in objects registry and get ID
                    let obj_id = self.object_id_gen.fetch_add(1, Ordering::SeqCst);
                    self.objects.insert(obj_id, Arc::new(RwLock::new(obj)));

                    // Push object ID onto stack
                    task.ram.push_i32(obj_id as i32);
                }
                // Plan 073: Array literal support
                OpCode::CREATE_ARRAY => {
                    let elem_count = self.flash.read_u8(task.ip);
                    task.ip += 1;

                    // Pop elements from stack (in reverse order since last element is on top)
                    let mut elems = Vec::with_capacity(elem_count as usize);
                    for _ in 0..elem_count {
                        // Pop element and convert to Value
                        let bits = task.ram.pop_i32();
                        elems.push(auto_val::Value::Int(bits));
                    }

                    // Reverse to get correct order (elements were popped LIFO)
                    elems.reverse();

                    // Store array in arrays registry and get ID
                    let array_id = self.array_id_gen.fetch_add(1, Ordering::SeqCst);
                    self.arrays.insert(array_id, Arc::new(RwLock::new(elems)));

                    // Push array ID onto stack
                    task.ram.push_i32(array_id as i32);
                }
                // Plan 073: Range expression support (0..10, 0..=10)
                OpCode::CREATE_RANGE => {
                    // Stack layout: [..., end, start]
                    // Pop end first (top of stack), then start
                    let end = task.ram.pop_i32();
                    let start = task.ram.pop_i32();

                    // Store range in ranges registry and push range_id
                    let range_id = task.ram.ranges.len() as i32;
                    task.ram.ranges.push((start, end, false)); // false = exclusive
                    
                    // Use special marker for range: -1000000 + range_id
                    task.ram.push_i32(-1000000 + range_id);
                }
                OpCode::CREATE_RANGE_EQ => {
                    // Stack layout: [..., end, start]
                    // Pop end first (top of stack), then start
                    let end = task.ram.pop_i32();
                    let start = task.ram.pop_i32();

                    // Create RangeEq value (inclusive)
                    let _range_value = auto_val::Value::RangeEq(start, end);

                    // Store range in ranges registry and push range_id
                    let range_id = task.ram.ranges.len() as i32;
                    task.ram.ranges.push((start, end, true)); // true = inclusive
                    eprintln!("DEBUG CREATE_RANGE_EQ: start={}, end={}, range_id={}", start, end, range_id);
                    
                    // Use special marker for range: -1000000 + range_id
                    task.ram.push_i32(-1000000 + range_id);
                }
                // Plan 073: F-string support (f"hello $name")
                OpCode::BUILD_FSTR => {
                    let part_count = self.flash.read_u8(task.ip);
                    task.ip += 1;

                    // Pop parts from stack (in reverse order)
                    let mut parts = Vec::with_capacity(part_count as usize);
                    for i in 0..part_count {
                        let idx = (part_count - 1 - i) as usize;
                        let bits = task.ram.pop_i32();

                        // Convert each part to Value and then to string
                        // For now, we treat all parts as integers
                        // TODO: Support proper Value types when stack supports them
                        let value = auto_val::Value::Int(bits);
                        parts.insert(idx, value.to_astr().to_string());
                    }

                    // Join all parts into a single string
                    let result = parts.join("");

                    // For now, we can't push a full string onto the stack
                    // Push the string length as a placeholder
                    // The f-string semantics are encoded in the bytecode itself
                    // TODO: Add proper string support when stack supports Value types
                    task.ram.push_i32(result.len() as i32);
                }
                // Plan 073: May<T> null coalesce operator: left ?? right
                OpCode::NULL_COALESCE => {
                    // Pop right expression (default value)
                    let default_bits = task.ram.pop_i32();
                    // Pop left expression (May<T> value)
                    let may_bits = task.ram.pop_i32();

                    // Check if May<T> is Nil (represented as -1)
                    // If May value is Nil (-1), push default value
                    // Otherwise, push the May value itself
                    if may_bits == -1 {
                        // Nil case: return default value
                        task.ram.push_i32(default_bits);
                    } else {
                        // Val case: return the unwrapped value
                        // TODO: When stack supports proper May<T> types, extract the actual value
                        task.ram.push_i32(may_bits);
                    }
                }
                // Plan 073: May<T> error propagate operator: expression.?
                OpCode::ERROR_PROPAGATE => {
                    // Pop May<T> value from stack
                    let may_bits = task.ram.pop_i32();

                    // Check if May<T> is Nil
                    if may_bits == -1 {
                        // Nil case: early return (error propagation)
                        // For now, we just return Nil as the function result
                        // TODO: Implement proper early return mechanism
                        task.ram.push_i32(-1);
                    } else {
                        // Val case: push the unwrapped value
                        // TODO: When stack supports proper May<T> types, extract the actual value
                        task.ram.push_i32(may_bits);
                    }
                }
                // Plan 075: Convert any value to string
                OpCode::TO_STR => {
                    // Pop value from stack
                    let value_bits = task.ram.pop_i32();

                    // Convert to string based on type
                    // For now, we'll treat all values as their string representation
                    // TODO: Proper type-based conversion
                    let string_value = format!("{}", value_bits);

                    // Add to strings pool and push index
                    let mut strings = self.strings.write().unwrap();
                    let str_idx = strings.len() as u16;
                    strings.push(string_value.into_bytes());
                    drop(strings);

                    task.ram.push_i32(str_idx as i32);
                }
                // Plan 075: Check if value is nil
                OpCode::IS_NIL => {
                    // Pop value from stack
                    let value_bits = task.ram.pop_i32();

                    // Check if nil (-1 represents nil in May<T> implementation)
                    let is_nil = if value_bits == -1 { 1 } else { 0 };

                    task.ram.push_i32(is_nil);
                }
                // Plan 075: Concatenate two strings
                OpCode::STR_CAT => {
                    // Pop right string index first (top of stack)
                    let right_idx = task.ram.pop_i32() as usize;
                    // Pop left string index
                    let left_idx = task.ram.pop_i32() as usize;

                    // Get strings from pool
                    let strings = self.strings.read().unwrap();

                    let left_str = if let Some(bytes) = strings.get(left_idx) {
                        String::from_utf8_lossy(bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", left_idx)));
                    };

                    let right_str = if let Some(bytes) = strings.get(right_idx) {
                        String::from_utf8_lossy(bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", right_idx)));
                    };
                    drop(strings);

                    // Concatenate strings
                    let result = format!("{}{}", left_str, right_str);

                    // Add result to strings pool and push index
                    let mut strings = self.strings.write().unwrap();
                    let result_idx = strings.len() as u16;
                    strings.push(result.into_bytes());
                    drop(strings);

                    task.ram.push_i32(result_idx as i32);
                }
                // Plan 076 Phase 3 & 4: Generic List opcodes with storage strategies
                OpCode::CREATE_LIST_INT => {
                    // Plan 077 Phase 5: Create List<int> in unified registry
                    use crate::vm::types::ListData;
                    let list_data: ListData<i32> = ListData::new();  // Heap storage (default)
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                OpCode::CREATE_LIST_STR => {
                    // Plan 077 Phase 5: Create List<String> in unified registry
                    use crate::vm::types::ListData;
                    let list_data: ListData<String> = ListData::new();  // Heap storage (default)
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                OpCode::CREATE_LIST_BOOL => {
                    // Plan 077 Phase 5: Create List<bool> in unified registry
                    use crate::vm::types::ListData;
                    let list_data: ListData<bool> = ListData::new();  // Heap storage (default)
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                // Plan 076 Phase 4: InlineInt64 storage variants
                OpCode::CREATE_LIST_INT_INLINE => {
                    // Plan 077 Phase 5: Create List<int> with InlineInt64 storage in unified registry
                    use crate::vm::types::{ListData, ListStorage};
                    let mut list_data: ListData<i32> = ListData::new();
                    list_data.storage = Some(ListStorage::InlineInt64);
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                OpCode::CREATE_LIST_STR_INLINE => {
                    // Plan 077 Phase 5: Create List<String> with InlineInt64 storage in unified registry
                    use crate::vm::types::{ListData, ListStorage};
                    let mut list_data: ListData<String> = ListData::new();
                    list_data.storage = Some(ListStorage::InlineInt64);
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                OpCode::CREATE_LIST_BOOL_INLINE => {
                    // Plan 077 Phase 5: Create List<bool> with InlineInt64 storage in unified registry
                    use crate::vm::types::{ListData, ListStorage};
                    let mut list_data: ListData<bool> = ListData::new();
                    list_data.storage = Some(ListStorage::InlineInt64);
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }

                // === Plan 087 Phase 2: Generic Instance Support ===
                OpCode::NEW_INSTANCE => {
                    // Plan 087 Phase 2: Create a new generic instance (type-erased storage)
                    // Stack layout: [..., mono_name_len]
                    // Code layout: [opcode, mono_name_bytes...]
                    // Stack after: [..., instance_id]
                    use crate::vm::generic_registry::GenericInstanceData;

                    eprintln!("DEBUG NEW_INSTANCE: Stack depth before pop = {}", task.ram.sp);

                    // Read mono_name length from stack
                    let name_len = task.ram.pop_i32() as usize;
                    eprintln!("DEBUG NEW_INSTANCE: Popped name_len = {}", name_len);

                    // Read mono_name bytes from flash memory and convert to String
                    // Note: task.ip already points to the first byte after the opcode (advanced by main loop)
                    let mut name_bytes = vec![0u8; name_len];
                    for i in 0..name_len {
                        let byte_addr = task.ip.wrapping_add(i);
                        name_bytes[i] = self.flash.read_u8(byte_addr);
                    }

                    // Advance IP past the name bytes
                    task.ip = task.ip.wrapping_add(name_len);

                    let mono_name = String::from_utf8(name_bytes)
                        .map_err(|e| VMError::RuntimeError(format!("Invalid UTF-8 in mono_name: {}", e)))?;
                    eprintln!("DEBUG NEW_INSTANCE: mono_name = '{}'", mono_name);

                    // Create instance with no fields (uninitialized)
                    let instance = GenericInstanceData::new(mono_name, vec![]);
                    let instance_id = self.insert_heap_object(instance);

                    // Push instance ID onto stack
                    eprintln!("DEBUG NEW_INSTANCE: Pushing instance_id = {}", instance_id);
                    task.ram.push_i32(instance_id as i32);
                    eprintln!("DEBUG NEW_INSTANCE: Stack depth after push = {}, top value = {}",
                        task.ram.sp,
                        if task.ram.sp > 0 { task.ram.raw[(task.ram.sp - 1) as usize] } else { 0 }
                    );
                }
                OpCode::CONSTRUCT_INSTANCE => {
                    // Plan 087 Phase 2: Populate fields of a generic instance
                    // Stack layout: [..., value1, value2, ..., valueN, instance_id, field_count]
                    // Stack after: [..., instance_id]  (instance_id left on stack for variable assignment)
                    use crate::vm::generic_registry::GenericInstanceData;
                    use crate::vm::heap_object::TypeTag;

                    eprintln!("DEBUG CONSTRUCT_INSTANCE: Stack depth before pop = {}", task.ram.sp);

                    // Pop field_count (top of stack)
                    let field_count = task.ram.pop_i32() as usize;
                    eprintln!("DEBUG CONSTRUCT_INSTANCE: Popped field_count = {}", field_count);

                    // Pop instance_id (next on stack)
                    let instance_id = task.ram.pop_i32() as u64;
                    eprintln!("DEBUG CONSTRUCT_INSTANCE: Popped instance_id = {}", instance_id);

                    // Pop values from stack (in reverse order)
                    // For Phase 2, we assume all values are basic types (int, float, bool, etc.)
                    let mut field_values = Vec::with_capacity(field_count);
                    for i in 0..field_count {
                        eprintln!("DEBUG CONSTRUCT_INSTANCE: Popping value {}/{}, stack depth = {}", i+1, field_count, task.ram.sp);
                        // Pop value as i32 (basic type)
                        let val_i32 = task.ram.pop_i32();
                        eprintln!("DEBUG CONSTRUCT_INSTANCE: Popped value = {}", val_i32);
                        // For Phase 2, convert to Int (simplified)
                        field_values.push(Value::Int(val_i32));
                    }
                    field_values.reverse(); // Reverse to get correct order

                    eprintln!("DEBUG CONSTRUCT_INSTANCE: Field values (reversed): {:?}", field_values);

                    // Get instance and populate fields
                    if let Some(obj) = self.get_heap_object(instance_id) {
                        let mut guard = obj.write().unwrap();

                        // Check if this is a GenericInstance by checking the type tag
                        let is_generic_instance = matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                        if is_generic_instance {
                            // Use as_any_mut for downcasting (works without exact TypeTag match)
                            if let Some(instance) = guard.as_any_mut().downcast_mut::<GenericInstanceData>() {
                                let field_count = field_values.len();
                                instance.fields = field_values;
                                eprintln!("DEBUG CONSTRUCT_INSTANCE: Successfully populated {} fields", field_count);
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Type error: Failed to downcast GenericInstance")));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: CONSTRUCT_INSTANCE expected GenericInstance, got {:?}", guard.type_tag())));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid instance ID: {}", instance_id)));
                    }

                    // Push instance_id back onto stack for variable assignment
                    // Stack layout after: [..., instance_id]
                    eprintln!("DEBUG CONSTRUCT_INSTANCE: Pushing instance_id back to stack: {}", instance_id);
                    task.ram.push_i32(instance_id as i32);
                    eprintln!("DEBUG CONSTRUCT_INSTANCE: Stack depth after = {}", task.ram.sp);
                }
                OpCode::GET_GENERIC_FIELD => {
                    // Plan 087 Phase 2: Get field value from generic instance
                    // Code layout: [opcode, field_index:u32]
                    // Stack layout: [..., instance_id]
                    // Stack after: [..., value, instance_id]  (instance_id restored to top)
                    use crate::vm::generic_registry::GenericInstanceData;
                    use crate::vm::heap_object::TypeTag;

                    // Read field_index from code stream (not stack!)
                    let field_index = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;

                    // Read instance_id from stack WITHOUT popping it
                    // Stack: [..., instance_id, ...]
                    let instance_id = task.ram.read_i32(task.ram.sp - 1) as u64;

                    eprintln!("DEBUG: GET_GENERIC_FIELD: instance_id={}, field_index={}",
                        instance_id, field_index);

                    // Get instance and read field
                    if let Some(obj) = self.get_heap_object(instance_id) {
                        let guard = obj.read().unwrap();

                        // Check if this is a GenericInstance (any variant)
                        let is_generic_instance = matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                        if is_generic_instance {
                            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                if let Some(value) = instance.get_field(field_index) {
                                    // Pop instance_id (we already read it)
                                    let _ = task.ram.pop_i32();
                                    // Push field value onto stack
                                    Self::push_value(&mut task.ram, value, &self.strings);
                                    eprintln!("DEBUG: GET_GENERIC_FIELD: field value = {:?}", value);
                                } else {
                                    return Err(VMError::RuntimeError(format!(
                                        "Field index {} out of bounds (instance has {} fields)",
                                        field_index, instance.field_count())));
                                }
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Type error: GET_GENERIC_FIELD failed to downcast GenericInstance")));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: GET_GENERIC_FIELD expected GenericInstance, got {:?}",
                                guard.type_tag())));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid instance ID: {}", instance_id)));
                    }
                }
                OpCode::SET_GENERIC_FIELD => {
                    // Plan 087 Phase 2: Set field value in generic instance
                    // Code layout: [opcode, field_index:u32]
                    // Stack layout: [..., instance_id, value]
                    // Stack after: [...]
                    use crate::vm::generic_registry::GenericInstanceData;
                    use crate::vm::heap_object::TypeTag;

                    // Read field_index from code stream (not stack!)
                    let field_index = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;

                    // Pop value (for Phase 2, assume it's a basic int)
                    let val_i32 = task.ram.pop_i32();
                    let value = Value::Int(val_i32);

                    // Pop instance_id
                    let instance_id = task.ram.pop_i32() as u64;

                    eprintln!("DEBUG: SET_GENERIC_FIELD: instance_id={}, field_index={}, value={}",
                        instance_id, field_index, val_i32);

                    // Get instance and set field
                    if let Some(obj) = self.get_heap_object(instance_id) {
                        let mut guard = obj.write().unwrap();

                        // Check if this is a GenericInstance (any variant)
                        let is_generic_instance = matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                        if is_generic_instance {
                            if let Some(instance) = guard.as_any_mut().downcast_mut::<GenericInstanceData>() {
                                let value_repr = format!("{:?}", value);  // Capture before move
                                instance.set_field(field_index, value)
                                    .map_err(|e| VMError::RuntimeError(format!("Failed to set field: {}", e)))?;
                                eprintln!("DEBUG: SET_GENERIC_FIELD: successfully set field {} to {}",
                                    field_index, value_repr);
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Type error: SET_GENERIC_FIELD failed to downcast GenericInstance")));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: SET_GENERIC_FIELD expected GenericInstance, got {:?}",
                                guard.type_tag())));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid instance ID: {}", instance_id)));
                    }
                }
                OpCode::LIST_PUSH_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id, value:int]
                    // Pop value first (top of stack), then list_id
                    let value = task.ram.pop_i32();
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::types::ListData;
                    use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                            if !list.push(value) {
                                return Err(VMError::RuntimeError(format!(
                                    "List capacity exceeded (InlineInt64 limit: 64)")));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_PUSH_INT expected ListInt")));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}", list_id)));
                    }
                }
                OpCode::LIST_POP_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id]
                    // Pop list_id, get list, pop element, push result
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::types::ListData;
                    use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                            let value = list.pop().unwrap_or(0);
                            task.ram.push_i32(value);
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_POP_INT expected ListInt")));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}", list_id)));
                    }
                }
                OpCode::LIST_GET_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id, index:int]
                    // Pop index first (top of stack), then list_id
                    let index = task.ram.pop_i32() as usize;
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::types::ListData;
                    use crate::vm::heap_object::{try_downcast_checked, TypeTag};

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let guard = obj.read().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
                            let value = list.get(index).copied().unwrap_or(0);
                            task.ram.push_i32(value);
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_GET_INT expected ListInt")));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}", list_id)));
                    }
                }
                OpCode::LIST_SET_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id, index:int, value:int]
                    // Pop value first, then index, then list_id
                    let value = task.ram.pop_i32();
                    let index = task.ram.pop_i32() as usize;
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::types::ListData;
                    use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) = try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt) {
                            if !list.set(index, value) {
                                return Err(VMError::RuntimeError(format!(
                                    "List index out of bounds: {}", index)));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_SET_INT expected ListInt")));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}", list_id)));
                    }
                }
                // Plan 073: Node creation (for type instances and tree structures)
                OpCode::CREATE_NODE => {
                    // Format: CREATE_NODE <name_str_idx:u16> <arg_count:u8>
                    let name_str_idx = self.flash.read_u16(task.ip);
                    task.ip += 2;
                    let arg_count = self.flash.read_u8(task.ip);
                    task.ip += 1;

                    // Get node name from string pool
                    let name = if let Ok(strings) = self.strings.read() {
                        if let Some(bytes) = strings.get(name_str_idx as usize) {
                            String::from_utf8(bytes.clone()).unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    // Pop arguments from stack (in reverse order)
                    let mut args = Vec::with_capacity(arg_count as usize);
                    for i in 0..arg_count {
                        let idx = (arg_count - 1 - i) as usize;
                        let bits = task.ram.pop_i32();
                        args.insert(idx, auto_val::Value::Int(bits));
                    }

                    // Create node
                    let mut node = auto_val::Node::new(&name);

                    // Add arguments as properties
                    for (i, arg) in args.iter().enumerate() {
                        // Use positional keys: 0, 1, 2, ...
                        let key = auto_val::ValueKey::Int(i as i32);
                        node.set_prop(key, arg.clone());
                    }

                    // Store node in nodes registry and get ID
                    let node_id = self.node_id_gen.fetch_add(1, Ordering::SeqCst);
                    self.nodes.insert(node_id, Arc::new(RwLock::new(node)));

                    // Push node ID onto stack
                    task.ram.push_i32(node_id as i32);
                }
                // Plan 073: Array element access (arr[index])
                // Plan 080: Also supports heap objects (lists like List<int>)
                OpCode::GET_ELEM => {
                    // Stack: array_id/list_id, index
                    // Pop index first (top of stack)
                    let index = task.ram.pop_i32() as usize;
                    // Pop array_id/list_id
                    let obj_id = task.ram.pop_i32() as u64;

                    eprintln!("DEBUG GET_ELEM: obj_id={}, index={}", obj_id, index);

                    // First, try heap_objects registry (Plan 077 unified registry)
                    if let Some(obj) = self.get_heap_object(obj_id) {
                        use crate::vm::types::ListData;
                        let guard = obj.read().unwrap();

                        // Try List<int>
                        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                            eprintln!("DEBUG GET_ELEM: Found List<int> with {} elems", list.elems.len());
                            if let Some(&elem) = list.elems.get(index) {
                                eprintln!("DEBUG GET_ELEM: Returning elem[{}]={}", index, elem);
                                task.ram.push_i32(elem);
                            } else {
                                eprintln!("DEBUG GET_ELEM: Index {} out of bounds", index);
                                task.ram.push_i32(0); // Out of bounds
                            }
                        }
                        // Try List<String>
                        else if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
                            eprintln!("DEBUG GET_ELEM: Found List<String>");
                            if let Some(_elem) = list.elems.get(index) {
                                // TODO: Support string elements (currently push placeholder)
                                task.ram.push_i32(0);
                            } else {
                                task.ram.push_i32(0); // Out of bounds
                            }
                        }
                        // Try List<bool>
                        else if let Some(list) = guard.as_any().downcast_ref::<ListData<bool>>() {
                            eprintln!("DEBUG GET_ELEM: Found List<bool>");
                            if let Some(&elem) = list.elems.get(index) {
                                task.ram.push_i32(if elem { 1 } else { 0 });
                            } else {
                                task.ram.push_i32(0); // Out of bounds
                            }
                        }
                        else {
                            eprintln!("DEBUG GET_ELEM: Unknown heap object type");
                            task.ram.push_i32(0); // Unknown heap object type
                        }
                    }
                    // Fallback to legacy arrays registry
                    else if let Some(array_ref) = self.arrays.get(&obj_id) {
                        let array = array_ref.read().unwrap();

                        // Check bounds
                        if index < array.len() {
                            // Get element value
                            let elem = &array[index];

                            // Push element value onto stack based on type
                            match elem {
                                auto_val::Value::Int(i) => task.ram.push_i32(*i),
                                auto_val::Value::Uint(u) => task.ram.push_i32(*u as i32),
                                auto_val::Value::Float(f) => task.ram.push_f32(*f as f32),
                                auto_val::Value::Double(d) => task.ram.push_f64(*d),
                                auto_val::Value::Bool(b) => task.ram.push_i32(if *b { 1 } else { 0 }),
                                auto_val::Value::Char(c) => task.ram.push_i32(*c as i32),
                                auto_val::Value::Nil => task.ram.push_i32(0),
                                _ => {
                                    // Unsupported type - push 0 as placeholder
                                    task.ram.push_i32(0);
                                }
                            }
                        } else {
                            // Index out of bounds - push 0 as error sentinel
                            // TODO: Proper error handling for out-of-bounds access
                            task.ram.push_i32(0);
                        }
                    } else {
                        // Object not found - push 0 as error sentinel
                        // TODO: Proper error handling for invalid object IDs
                        task.ram.push_i32(0);
                    }
                }
                // Plan 073: Array element assignment (arr[index] = value)
                OpCode::SET_ELEM => {
                    // Stack: value, array_id, index (compiled in this order by codegen)
                    // Pop index first (top of stack)
                    let index = task.ram.pop_i32() as usize;
                    // Pop array_id
                    let array_id = task.ram.pop_i32() as u64;
                    // Pop value (bottom of stack)
                    let value = task.ram.pop_i32();

                    // Get array from registry
                    if let Some(array_ref) = self.arrays.get(&array_id) {
                        let mut array = array_ref.write().unwrap();

                        // Check bounds
                        if index < array.len() {
                            // Update element value
                            // Convert i32 value to appropriate Value type
                            // For now, store as Int (we can enhance this later with type tracking)
                            array[index] = auto_val::Value::Int(value);
                        } else {
                            // Index out of bounds - silent fail for now
                            // TODO: Proper error handling for out-of-bounds assignment
                        }
                    } else {
                        // Array not found - silent fail for now
                        // TODO: Proper error handling for invalid array IDs
                    }
                }
                // Plan 075: Object field assignment (obj.field = value)
                OpCode::SET_FIELD => {
                    // Stack: value, object_id, field_name_idx (compiled in this order by codegen)
                    // Pop field_name_idx first (top of stack)
                    let field_idx = task.ram.pop_i32() as usize;
                    // Pop object_id
                    let obj_id = task.ram.pop_i32() as u64;
                    // Pop value (bottom of stack)
                    let value = task.ram.pop_i32();

                    // Get field name from strings pool
                    let strings = self.strings.read().unwrap();
                    let field_name = if let Some(field_bytes) = strings.get(field_idx) {
                        String::from_utf8_lossy(field_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", field_idx)));
                    };
                    drop(strings); // Release lock before writing

                    // Get object from registry
                    if let Some(obj_ref) = self.objects.get(&obj_id) {
                        let mut obj = obj_ref.write().unwrap();
                        // Set field value (convert i32 to Value)
                        let key = auto_val::ValueKey::Str(field_name.into());
                        obj.set(key, auto_val::Value::Int(value));
                    } else {
                        // Object not found - silent fail for now
                        // TODO: Proper error handling for invalid object IDs
                    }
                }
                // Plan 073: Object field access (obj.field)
                OpCode::GET_FIELD => {
                    let field_idx = self.flash.read_u16(task.ip);
                    task.ip += 2;

                    // Pop object ID from stack
                    let obj_id = task.ram.pop_i32() as u64;

                    // Get field name from strings pool (Plan 073: Now uses RwLock)
                    let strings = self.strings.read().unwrap();
                    let field_name = if let Some(field_bytes) = strings.get(field_idx as usize) {
                        String::from_utf8_lossy(field_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", field_idx)));
                    };
                    drop(strings); // Release lock before potentially writing below

                    // Get object from registry
                    if let Some(obj_ref) = self.objects.get(&obj_id) {
                        let obj = obj_ref.read().unwrap();
                        // Look up field by name (convert to ValueKey)
                        let key = auto_val::ValueKey::Str(field_name.into());

                        if let Some(value) = obj.get(&key) {
                            // Push field value onto stack based on type
                            match value {
                                auto_val::Value::Int(i) => task.ram.push_i32(*i),
                                auto_val::Value::Uint(u) => task.ram.push_i32(*u as i32),
                                auto_val::Value::Float(f) => task.ram.push_f32(*f as f32),
                                auto_val::Value::Double(d) => task.ram.push_f64(*d),
                                auto_val::Value::Bool(b) => task.ram.push_i32(if *b { 1 } else { 0 }),
                                auto_val::Value::Char(c) => task.ram.push_i32(*c as i32),
                                auto_val::Value::Str(s) => {
                                    // Plan 073: Add string to mutable pool and push index
                                    let str_bytes = s.as_bytes().to_vec();
                                    let mut strings = self.strings.write().unwrap();
                                    let str_idx = strings.len() as u16;
                                    strings.push(str_bytes);
                                    drop(strings);
                                    task.ram.push_i32(str_idx as i32);
                                }
                                auto_val::Value::Nil => task.ram.push_i32(0),
                                // Plan 073: Nested objects/arrays - push their ID
                                auto_val::Value::VmRef(vm_ref) => {
                                    task.ram.push_i32(vm_ref.id as i32);
                                }
                                _ => {
                                    // Unsupported type - push 0 as placeholder
                                    task.ram.push_i32(0);
                                }
                            }
                        } else {
                            // Field not found - push 0 as error sentinel
                            // TODO: Proper error handling for missing fields
                            task.ram.push_i32(0);
                        }
                    } else {
                        // Object not found - push 0 as error sentinel
                        // TODO: Proper error handling for invalid object IDs
                        task.ram.push_i32(0);
                    }
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

                // Plan 073 Stage A: Floating-point arithmetic (f32)
                OpCode::ADD_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(a + b);
                }
                OpCode::SUB_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(a - b);
                }
                OpCode::MUL_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(a * b);
                }
                OpCode::DIV_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    if b == 0.0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_f32(a / b);
                }
                OpCode::NEG_F => {
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(-a);
                }

                // Plan 073 Stage A: Double precision arithmetic (f64)
                OpCode::ADD_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_f64(a + b);
                }
                OpCode::SUB_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_f64(a - b);
                }
                OpCode::MUL_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_f64(a * b);
                }
                OpCode::DIV_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    if b == 0.0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_f64(a / b);
                }
                OpCode::NEG_D => {
                    let a = task.ram.pop_f64();
                    task.ram.push_f64(-a);
                }

                OpCode::NOT => {
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(!a);
                }
                OpCode::CALL => {
                    eprintln!("DEBUG CALL: Stack depth before = {}", task.ram.sp);
                    // Print stack before CALL
                    if task.ram.sp > 0 {
                        eprintln!("DEBUG CALL: Stack[0] = {}", task.ram.read_i32(0));
                    }

                    let target = self.flash.read_u32(task.ip) as usize;
                    eprintln!("DEBUG CALL: Calling function at address 0x{:04x}", target);
                    task.ip += 4;

                    // Push Return Address (IP)
                    task.ram.push_i32(task.ip as i32);
                    // Push Old Stack Frame (BP)
                    task.ram.push_i32(task.bp as i32);

                    // New BP points to the saved BP location (SP - 1)
                    task.bp = task.ram.sp - 1;

                    eprintln!("DEBUG CALL: Stack depth after setup = {}, BP = {}", task.ram.sp, task.bp);
                    eprintln!("DEBUG CALL: Stack[0] = {}, [1] = {}, [2] = {}",
                        task.ram.read_i32(0), task.ram.read_i32(1), task.ram.read_i32(2));

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

                    // Plan 071 Phase 5: Restore previous closure (if any)
                    // Stack layout: [..., old_closure_id, ret_ip, old_bp, args...]
                    //               bp-2          bp-1    bp
                    // Only restore if bp - 2 is valid (not in main task)
                    let old_closure_id_val = if task.bp >= 2 {
                        task.ram.read_i32(task.bp - 2)
                    } else {
                        0
                    };
                    task.current_closure_id = if old_closure_id_val == 0 {
                        None
                    } else {
                        Some(old_closure_id_val as u32)
                    };

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

                // === Closures (Plan 071: Direct Capture) ===
                OpCode::CLOSURE => {
                    // Stack: capture_count × value -> closure_id
                    // Immediate: func_addr (u32), capture_count (u8)
                    let func_addr = self.flash.read_u32(task.ip);
                    task.ip += 4;
                    let capture_count = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    // Pop captured values from stack and build environment
                    let mut env = HashMap::new();
                    for _i in 0..capture_count {
                        // Read variable name from string table (stored in reverse order)
                        let var_name_idx = self.flash.read_u16(task.ip) as usize;
                        task.ip += 2;

                        // Pop value from stack (values pushed in order, popped in reverse)
                        let value = task.ram.pop_i32();

                        // Look up variable name from string table (Plan 073: Now uses RwLock)
                        let strings = self.strings.read().unwrap();
                        let var_name_str = if let Some(var_name) = strings.get(var_name_idx) {
                            String::from_utf8_lossy(var_name).to_string()
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Invalid string index for captured variable: {}",
                                var_name_idx
                            )));
                        };
                        drop(strings);
                        env.insert(var_name_str, Value::Int(value));
                    }

                    // Create closure
                    let closure_id = self.closure_id_gen.fetch_add(1, Ordering::Relaxed);
                    let closure = Closure { func_addr, env };

                    self.closures.insert(closure_id, closure);
                    task.ram.push_i32(closure_id as i32);
                }
                OpCode::CAPTURE_VAR => {
                    // Stack: -> value
                    // Immediate: var_name_idx (u16)
                    // Load variable by name from current scope and push value
                    let var_name_idx = self.flash.read_u16(task.ip) as usize;
                    task.ip += 2;

                    // Plan 073: Now uses RwLock for strings access
                    let strings = self.strings.read().unwrap();
                    let _var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                        String::from_utf8_lossy(var_name_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", var_name_idx)));
                    };
                    drop(strings);

                    // Look up variable in local scope (from stack frame)
                    // TODO: For MVP, we'll need to implement proper scope lookup
                    // For now, push placeholder value
                    task.ram.push_i32(0);
                }
                OpCode::LOAD_CAPTURED => {
                    // Plan 071 Phase 5: Load captured variable from current closure
                    // Stack: -> value (no longer pops closure_id)
                    // Immediate: var_name_idx (u16)
                    let var_name_idx = self.flash.read_u16(task.ip) as usize;
                    task.ip += 2;

                    // Use current_closure_id instead of popping from stack
                    let closure_id = task.current_closure_id.ok_or_else(|| {
                        VMError::RuntimeError("LOAD_CAPTURED called outside of closure context".to_string())
                    })?;

                    // Plan 073: Now uses RwLock for strings access
                    let strings = self.strings.read().unwrap();
                    let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                        String::from_utf8_lossy(var_name_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", var_name_idx)));
                    };
                    drop(strings);

                    if let Some(closure) = self.closures.get(&closure_id) {
                        if let Some(value) = closure.env.get(var_name.as_str()) {
                            // Push value to stack
                            match value {
                                Value::Int(i) => task.ram.push_i32(*i),
                                // TODO: Handle other value types
                                _ => task.ram.push_i32(0),
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Captured variable '{}' not found in closure {}",
                                var_name, closure_id
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid closure ID: {}",
                            closure_id
                        )));
                    }
                }
                OpCode::STORE_CAPTURED => {
                    // Plan 071 Phase 5: Store to captured variable in current closure
                    // Stack: value -> (no longer pops closure_id)
                    // Immediate: var_name_idx (u16)
                    let var_name_idx = self.flash.read_u16(task.ip) as usize;
                    task.ip += 2;

                    let value = task.ram.pop_i32();

                    // Use current_closure_id instead of popping from stack
                    let closure_id = task.current_closure_id.ok_or_else(|| {
                        VMError::RuntimeError("STORE_CAPTURED called outside of closure context".to_string())
                    })?;

                    // Plan 073: Now uses RwLock for strings access
                    let strings = self.strings.read().unwrap();
                    let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                        String::from_utf8_lossy(var_name_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}", var_name_idx)));
                    };
                    drop(strings);

                    if let Some(mut closure) = self.closures.get_mut(&closure_id) {
                        closure.env.insert(var_name, Value::Int(value));
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid closure ID: {}",
                            closure_id
                        )));
                    }
                }
                OpCode::CALL_CLOSURE => {
                    // Stack: closure_id, [args...] -> result
                    // Immediate: arg_count (u8)
                    let _arg_count = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    let closure_id = task.ram.pop_i32() as u32;

                    if let Some(_closure) = self.closures.get(&closure_id) {
                        // Plan 071 Phase 5: Set current closure for LOAD_CAPTURED access
                        let old_closure_id = task.current_closure_id;
                        task.current_closure_id = Some(closure_id);

                        // Push old closure ID to stack (to restore on RET)
                        // We'll use a special marker value: -1 means "no closure"
                        let old_closure_val = old_closure_id.unwrap_or(0);
                        task.ram.push_i32(old_closure_val as i32);

                        // Push Return Address (IP)
                        task.ram.push_i32(task.ip as i32);
                        // Push Old Stack Frame (BP)
                        task.ram.push_i32(task.bp as i32);

                        // New BP points to the saved BP location (SP - 1)
                        task.bp = task.ram.sp - 1;

                        // Jump to closure function
                        task.ip = _closure.func_addr as usize;
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid closure ID: {}",
                            closure_id
                        )));
                    }
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
                //
                // Stack frame layout (Plan 080):
                //   [..., ret_ip, old_bp, local0, local1, ..., temps...]
                //                     bp
                //   Local variables are at bp+1, bp+2, ... (not bp+0!)
                //
                // For main task (bp=1):
                //   [unused, local0, local1, ..., temps...]
                //      bp-1     bp     bp+1
                //
                OpCode::LOAD_LOCAL => {
                    let idx = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    // Plan 087 Phase 3: Check if this is a parameter (idx >= 0x80)
                    if idx >= 0x80 {
                        // Parameter: decode parameter index
                        let param_idx = idx - 0x80;  // 0x80 -> param 0, 0x81 -> param 1, etc.
                        // Stack layout: [..., args(0), args(1), ..., return_addr, old_bp, locals...]
                        //                        ^- BP-n_args           ^- BP-1    ^- BP

                        // Plan 088 Phase 4: Read n_args from function metadata (set by FN_PROLOG)
                        let n_args = task.current_fn_n_args;
                        let offset = n_args - param_idx;  // For n_args=1, param 0: offset=1

                        // Stack layout for n_args=1: [arg0, ret_addr, old_bp, ...]
                        //                                    ^-BP-2 ^-BP-1  ^-BP
                        // For n_args=2:             [arg0, arg1, ret_addr, old_bp, ...]
                        //                                    ^-BP-3 ^-BP-2 ^-BP-1  ^-BP
                        let actual_offset = offset + 1;  // +1 for return_addr
                        let val = task.ram.read_i32(task.bp - actual_offset);
                        eprintln!("DEBUG: LOAD_LOCAL param {}: BP-{} (n_args={}, offset={}) = {}",
                            param_idx, actual_offset, n_args, offset, val);
                        task.ram.push_i32(val);
                    } else {
                        // Local variable: load from bp+1+idx (bp+1 is first local variable)
                        let val = task.ram.read_i32(task.bp + 1 + idx);
                        eprintln!("DEBUG: LOAD_LOCAL local {}: BP+1+{} = {}", idx, idx, val);
                        task.ram.push_i32(val);
                    }
                }
                OpCode::STORE_LOCAL => {
                    let idx = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;
                    let val = task.ram.pop_i32();

                    // Plan 088 Phase 4: Check if this is a parameter (idx >= 0x80)
                    if idx >= 0x80 {
                        // Parameter: decode parameter index
                        let param_idx = idx - 0x80;  // 0x80 -> param 0, 0x81 -> param 1, etc.
                        let n_args = task.current_fn_n_args;
                        let offset = n_args - param_idx;
                        let actual_offset = offset + 1;  // +1 for return_addr

                        // Store to parameter location
                        task.ram.write_i32(task.bp - actual_offset, val);
                        eprintln!("DEBUG: STORE_LOCAL param {}: BP-{} = {}",
                            param_idx, actual_offset, val);
                    } else {
                        // Local variable: store to bp+1+idx (bp+1 is first local variable)
                        task.ram.write_i32(task.bp + 1 + idx, val);
                        eprintln!("DEBUG: STORE_LOCAL local {}: BP+1+{} = {}",
                            idx, idx, val);
                    }
                }
                OpCode::LOAD_LOC_0 => {
                    // Load from bp+1 (first local variable)
                    let val = task.ram.read_i32(task.bp + 1);
                    task.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_1 => {
                    // Load from bp+2 (second local variable)
                    let val = task.ram.read_i32(task.bp + 2);
                    task.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_2 => {
                    // Load from bp+3 (third local variable)
                    let val = task.ram.read_i32(task.bp + 3);
                    task.ram.push_i32(val);
                }
                OpCode::STORE_LOC_0 => {
                    // Store to bp+1 (first local variable)
                    let val = task.ram.pop_i32();
                    task.ram.write_i32(task.bp + 1, val);
                }
                OpCode::STORE_LOC_1 => {
                    // Store to bp+2 (second local variable)
                    let val = task.ram.pop_i32();
                    task.ram.write_i32(task.bp + 2, val);
                }

                // === Stack ===
                OpCode::DROP => {
                    task.ram.pop_i32();
                }
                // Plan 088 Phase 4: Function Prologue
                OpCode::FN_PROLOG => {
                    // Read function metadata
                    let n_args = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;
                    let n_locals = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    eprintln!("DEBUG FN_PROLOG: n_args={}, n_locals={}", n_args, n_locals);

                    // Save function metadata in task for use by LOAD_LOCAL/STORE_LOCAL
                    task.current_fn_n_args = n_args;
                    task.current_fn_n_locals = n_locals;
                }
                OpCode::RESERVE_STACK => {
                    // Reserve stack space for n_locals to prevent stack from overwriting locals
                    // Layout: [local_0, local_1, ..., local_n-1, stack..., return_addr, old_bp, args...]
                    //           0          1          n_locals-1  n_locals         ...
                    // BP points to saved BP location (in normal function calls), or 0 in main task
                    // STORE_LOC_0 writes to BP+1, STORE_LOC_1 writes to BP+2, etc.
                    // Stack operations (push/pop) use SP which should be >= n_locals + 1 to avoid overlap
                    let n_locals = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    // Push n_locals+1 zeros to reserve space for local variables + 1 extra slot
                    // The extra slot ensures SP starts beyond all local variable addresses
                    for _ in 0..n_locals + 1 {
                        task.ram.push_i32(0);
                    }

                    // Track num_locals for native shims
                    task.num_locals = n_locals;
                }

                // === Comparison ===
                OpCode::EQ => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    // Plan 091: Use special values for boolean results
                    // i32::MIN = true, i32::MIN+1 = false
                    task.ram.push_i32(if a == b { -2147483648 } else { -2147483647 });
                }
                OpCode::NE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a != b { -2147483648 } else { -2147483647 });
                }
                OpCode::LT => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a < b { -2147483648 } else { -2147483647 });
                }
                OpCode::GT => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a > b { -2147483648 } else { -2147483647 });
                }
                OpCode::LE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a <= b { -2147483648 } else { -2147483647 });
                }
                OpCode::GE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(if a >= b { -2147483648 } else { -2147483647 });
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
                    // Plan 091: Handle boolean values
                    // false = -2147483647 (i32::MIN + 1)
                    // Also treat 0 as false for backward compatibility
                    if cond == 0 || cond == -2147483647 {
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
                    // Plan 091: Handle boolean values
                    // true = -2147483648 (i32::MIN)
                    // Jump if true (or any other non-zero, non-false value)
                    if cond != 0 && cond != -2147483647 {
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

                // === Plan 088 Phase 5: Reference Passing Instructions ===
                // Note: For Phase 5, references are implemented as var_index on the stack
                // LOAD_REF/LOAD_MUT_REF push the var_index, STORE_REF/STORE_MUT_REF use it
                OpCode::LOAD_REF => {
                    // Plan 088 Phase 5: Load immutable reference
                    // Format: var_index: u32
                    let var_index = self.flash.read_u32(task.ip);
                    task.ip += 4;

                    // Push var_index onto stack as the "reference"
                    // This will be used by subsequent STORE_REF or other operations
                    task.ram.push_i32(var_index as i32);

                    eprintln!("DEBUG: LOAD_REF: var_index={}, bp={}", var_index, task.bp);
                }
                OpCode::STORE_REF => {
                    // Plan 088 Phase 5: Store through immutable reference
                    // Format: var_index: u32
                    let var_index = self.flash.read_u32(task.ip);
                    task.ip += 4;

                    // Pop the value to store
                    let val = task.ram.pop_i32();

                    // Store to bp+1+var_index (same as LOAD_LOCAL logic)
                    task.ram.write_i32(task.bp + 1 + var_index as usize, val);

                    eprintln!("DEBUG: STORE_REF: var_index={}, val={}, bp={}", var_index, val, task.bp);
                }
                OpCode::LOAD_MUT_REF => {
                    // Plan 088 Phase 5: Load mutable reference
                    // Format: var_index: u32
                    let var_index = self.flash.read_u32(task.ip);
                    task.ip += 4;

                    // Push var_index onto stack as the "mutable reference"
                    task.ram.push_i32(var_index as i32);
                }
                OpCode::STORE_MUT_REF => {
                    // Plan 088 Phase 5: Store through mutable reference
                    // Format: var_index: u32
                    let var_index = self.flash.read_u32(task.ip);
                    task.ip += 4;

                    // Pop the value to store
                    let val = task.ram.pop_i32();

                    // Store to bp+1+var_index (same as STORE_LOCAL logic)
                    task.ram.write_i32(task.bp + 1 + var_index as usize, val);

                    eprintln!("DEBUG: STORE_MUT_REF: var_index={}, val={}, bp={}", var_index, val, task.bp);
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
