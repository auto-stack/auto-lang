use crate::vm::channel::{AutoChannel, ChannelId};
use crate::vm::codegen::ObjectType;
use crate::vm::heap_object::HeapObject;
use crate::vm::native::NativeInterface;
use crate::vm::opcode::OpCode;
use crate::vm::task::{AutoTask, ResultType, TaskId, TaskStatus};
use crate::vm::task_system::TaskRegistry;
use crate::vm::virt_memory::{VirtualFlash, VirtualRAM};
use auto_val::AutoStr;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

/// Debug logging macro - only prints when VM debug mode is enabled
macro_rules! vm_debug {
    ($($arg:tt)*) => {
        if crate::is_vm_debug() {
            eprintln!($($arg)*);
        }
    };
}

// ============================================================================
// Plan 221 Task 5: String tagging helpers (feature-gated for nanbox migration)
// ============================================================================

/// Result of popping a mixed int/string value from the stack.
/// Under non-nanbox: strings are encoded as negative i32 (-(idx+1)).
/// Under nanbox: strings use NaN-boxed encoding, ints use i32 encoding.
#[derive(Debug, Clone, Copy)]
pub enum StackTag {
    /// A plain integer value
    Int(i32),
    /// A string pool index
    Str(u32),
}

/// Push a string tag onto the stack.
#[cfg(feature = "nanbox")]
#[inline(always)]
fn push_str_tag(ram: &mut VirtualRAM, idx: u32) {
    ram.push_string(idx);
}

/// Push a string tag onto the stack.
#[cfg(not(feature = "nanbox"))]
#[inline(always)]
fn push_str_tag(ram: &mut VirtualRAM, idx: u32) {
    ram.push_i32(-(idx as i32) - 1);
}

/// Pop a known-string value, returning the string pool index.
#[cfg(feature = "nanbox")]
#[inline(always)]
fn pop_str_idx(ram: &mut VirtualRAM) -> usize {
    ram.pop_string() as usize
}

/// Pop a known-string value, returning the string pool index.
#[cfg(not(feature = "nanbox"))]
#[inline(always)]
fn pop_str_idx(ram: &mut VirtualRAM) -> usize {
    let bits = ram.pop_i32();
    (-bits - 1) as usize
}

/// Pop a mixed int/string value from the stack.
/// Under nanbox: uses NanoValue type detection.
/// Under non-nanbox: uses sign-bit tagging.
#[cfg(feature = "nanbox")]
#[inline(always)]
fn pop_tagged(ram: &mut VirtualRAM) -> StackTag {
    let nv = ram.pop_nv();
    if auto_val::is_string(nv) {
        StackTag::Str(auto_val::decode_string(nv))
    } else {
        StackTag::Int(auto_val::decode_i32(nv))
    }
}

/// Pop a mixed int/string value from the stack.
#[cfg(not(feature = "nanbox"))]
#[inline(always)]
fn pop_tagged(ram: &mut VirtualRAM) -> StackTag {
    let bits = ram.pop_i32();
    // Boolean sentinels: i32::MIN = true, i32::MIN + 1 = false — treat as int
    if bits < 0 && bits > i32::MIN + 1 {
        StackTag::Str(((-bits) - 1) as u32)
    } else {
        StackTag::Int(bits)
    }
}

/// Decode a string tag from an i32 variable (non-stack sources).
/// NOTE: Under nanbox, callers should prefer `pop_str_idx()` when reading from the stack.
/// This helper is for sites where the value is already in an i32 variable
/// (e.g., read from bytecode or a non-stack source).
#[inline(always)]
#[allow(dead_code)]
fn decode_str_tag_from_i32(bits: i32) -> usize {
    (-bits - 1) as usize
}

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
    pub func_addr: u32, // Address of the function to call
}

/// Filter iterator state - wraps a source iterator and applies a predicate
#[derive(Debug, Clone)]
pub struct FilterIterator {
    pub source_iterator_id: u32,
    pub func_addr: u32, // Address of the predicate function
}

/// Enumerate iterator state - wraps a source iterator, tracking index
/// Plan 200 Task 3.2: for (i, x) in iter.enumerate()
#[derive(Debug, Clone)]
pub struct EnumerateIterator {
    pub source_iterator_id: u32,
    pub current_index: u32,
}

/// Unified iterator type
#[derive(Debug, Clone)]
pub enum Iterator {
    List(ListIterator),
    Map(MapIterator),
    Filter(FilterIterator),
    Enumerate(EnumerateIterator),
}

// ============================================================================
// Closures (Plan 071: Direct Capture)
// ============================================================================

use auto_val::Value;

/// Closure - a function value with directly captured environment (Plan 071: Direct Capture)
#[derive(Debug, Clone)]
pub struct Closure {
    pub func_addr: u32,              // Bytecode address
    pub env: HashMap<String, Value>, // Direct captured values (no upvalues!)
    pub n_args: usize,               // Number of parameters (for CALL_CLOSURE to set current_fn_n_args)
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
    /// Plan 092: FFI-related errors (library loading, ABI incompatibility, etc.)
    FFI(String),
}

/// Result of executing a single VM instruction (used by `run_one_instruction`)
#[derive(Debug, Clone, PartialEq)]
enum StepResult {
    /// Continue executing more instructions
    Continue,
    /// Task has terminated (HALT, RET at bp==0, IP past end)
    Terminated,
    /// Task should pause the current batch (YIELD, SLEEP, blocked JOIN/SEND/RECV)
    Yield,
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

    // Plan 121: Task/Msg Registry for Actor model
    // Manages singleton tasks and task instances
    pub task_registry: Arc<TaskRegistry>,

    // Plan 127: Task Handler Registry for message routing
    // Maps task type names to their handler tables
    pub task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry,

    // Plan 124: Future Registry for async/await
    // Stores pending futures with their body code offsets
    pub futures: DashMap<u32, Arc<RwLock<FutureValue>>>,
    pub future_id_gen: AtomicU32,

    // Plan 197 Task 9: Generic registry for field name lookup at runtime
    pub generic_registry: crate::vm::generic_registry::GenericRegistry,

    // Plan 177: Optional stdout capture buffer for testing
    pub output_buffer: Option<Arc<RwLock<String>>>,

    // Plan 199: Debugger controller (NoOpController for normal execution)
    pub debugger: Arc<std::sync::Mutex<Box<dyn crate::vm::debugger::DebuggerController>>>,

    // Plan 199: Execution trace collector (None = tracing disabled)
    pub trace: Arc<std::sync::Mutex<Option<crate::vm::trace::TraceCollector>>>,
}

// Plan 124: Future value for async operations
#[derive(Debug, Clone)]
pub struct FutureValue {
    /// Bytecode offset of the async block body
    pub body_offset: u32,
    /// Current state of the future
    pub state: FutureState,
    /// Result value when ready
    pub result: Option<auto_val::Value>,
    /// Task ID that owns this future (for suspension/resumption)
    pub owner_task_id: TaskId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FutureState {
    Pending,
    Ready,
    Failed,
}

impl AutoVM {
    pub fn new(flash: VirtualFlash, _ram_size: usize) -> Self {
        let mut native_interface = NativeInterface::new();
        native_interface.register_std_shims();
        // Plan 094: Register static FFI stdlib functions (File, Env, Time, etc.)
        crate::vm::ffi::register_stdlib_ffi(&mut native_interface);

        // Plan 216 Phase 2: Merge C-FFI shims from the global CFFI_GLOBAL registry.
        // The codegen's handle_c_import populates CFFI_GLOBAL during compilation;
        // here we merge those shims into the VM's NativeInterface so CALL_NAT can find them.
        {
            let cffi = crate::vm::codegen::CFFI_GLOBAL.lock().unwrap();
            native_interface.merge(cffi.native_interface());
        }

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
            // Note: IDs start at 2000000 to avoid confusion with objects
            arrays: DashMap::new(),
            array_id_gen: AtomicU64::new(2000000),
            // Plan 073: Node registry
            // Note: IDs start at 3000000 to avoid confusion with arrays
            nodes: DashMap::new(),
            node_id_gen: AtomicU64::new(3000000),
            // Plan 077 Phase 4: Unified object registry
            // Note: IDs start at 4000000 to avoid confusion with nodes
            heap_objects: DashMap::new(),
            heap_object_id_gen: AtomicU64::new(4000000),
            // Plan 121: Task/Msg registry for Actor model
            task_registry: Arc::new(TaskRegistry::new()),
            // Plan 127: Task handler registry for message routing
            task_handler_registry: crate::vm::task_handler::TaskHandlerRegistry::new(),
            // Plan 124: Future registry for async/await
            futures: DashMap::new(),
            future_id_gen: AtomicU32::new(1),
            // Plan 197 Task 9: Generic registry for runtime field name lookup
            generic_registry: crate::vm::generic_registry::GenericRegistry::new(),
            // Plan 177: stdout capture (None = normal println)
            output_buffer: None,
            // Plan 199: Debugger controller (NoOpController for normal execution)
            debugger: Arc::new(std::sync::Mutex::new(
                Box::new(crate::vm::debugger::NoOpController)
            )),
            trace: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Create VM with stdout capture for testing (Plan 177)
    pub fn new_with_capture(flash: VirtualFlash, ram_size: usize) -> (Self, Arc<RwLock<String>>) {
        let mut vm = Self::new(flash, ram_size);
        let buffer = Arc::new(RwLock::new(String::new()));
        vm.output_buffer = Some(buffer.clone());
        (vm, buffer)
    }

    /// Load strings from a module's string constant pool
    pub fn load_strings(&mut self, strings: Vec<Vec<u8>>) {
        self.strings = Arc::new(RwLock::new(strings));
    }

    /// Plan 199: Set a custom debugger controller
    pub fn set_debugger(&mut self, controller: Box<dyn crate::vm::debugger::DebuggerController>) {
        self.debugger = Arc::new(std::sync::Mutex::new(controller));
    }

    /// Plan 199: Enable execution trace collection with a max record limit
    pub fn enable_trace(&mut self, max_records: usize) {
        self.trace = Arc::new(std::sync::Mutex::new(
            Some(crate::vm::trace::TraceCollector::new(max_records))
        ));
    }

    /// Plan 199: Get trace output as JSON
    pub fn get_trace_json(&self) -> Option<String> {
        let trace = self.trace.lock().unwrap();
        trace.as_ref().map(|t| t.to_json())
    }

    /// Plan 212b Task 4: Merge additional native shims into this VM
    ///
    /// Rebuilds the internal Arc<NativeInterface> by merging in shims
    /// from another NativeInterface (e.g., from RustFfiBridge).
    pub fn merge_native_interface(&mut self, other: &NativeInterface) {
        let mut ni = (*self.native_interface).clone();
        ni.merge(other);
        self.native_interface = Arc::new(ni);
    }

    /// Plan 197 Task 9: Load generic registry from codegen
    pub fn load_generic_registry(&mut self, registry: crate::vm::generic_registry::GenericRegistry) {
        self.generic_registry = registry;
    }

    /// Plan 118 Phase 4: Add a new string to the string pool
    /// Returns the index of the newly added string
    pub fn add_string(&self, bytes: Vec<u8>) -> usize {
        let mut strings = self.strings.write().unwrap();
        let idx = strings.len();
        strings.push(bytes);
        idx
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
    /// ```ignore
    /// use auto_lang::vm::types::ListData;
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
    /// ```ignore
    /// use auto_lang::vm::heap_object::downcast;
    /// use auto_lang::vm::types::ListData;
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
    fn push_value(
        ram: &mut VirtualRAM,
        value: &Value,
        strings: &std::sync::Arc<RwLock<Vec<Vec<u8>>>>,
    ) {
        match value {
            Value::Int(i) => ram.push_i32(*i),
            Value::Uint(u) => ram.push_i32(*u as i32),
            Value::Float(f) => ram.push_f32(*f as f32),
            Value::Double(d) => ram.push_f64(*d),
            Value::Bool(b) => ram.push_i32(if *b { 1 } else { 0 }),
            Value::Char(c) => ram.push_i32(*c as i32),
            Value::Nil => ram.push_i32(0),
            Value::Str(s) => {
                // Store string in constant pool and push its tagged index
                // String indices are stored as -(index+1) to distinguish from integers
                let s_bytes = s.as_bytes().to_vec();
                let mut strings = strings.write().unwrap();
                let idx = strings.len();
                strings.push(s_bytes);
                drop(strings);
                push_str_tag(ram, idx as u32);
            }
            Value::VmRef(vmref) => {
                // Push heap object ID as i32
                ram.push_i32(vmref.id as i32);
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
    #[allow(dead_code)]
    fn pop_value_as_int(ram: &mut VirtualRAM) -> i32 {
        ram.pop_i32()
    }

    /// Pop a float value from the stack as f32
    #[allow(dead_code)]
    fn pop_value_as_float(ram: &mut VirtualRAM) -> f32 {
        ram.pop_f32()
    }

    /// Pop a double value from the stack as f64
    #[allow(dead_code)]
    fn pop_value_as_double(ram: &mut VirtualRAM) -> f64 {
        ram.pop_f64()
    }

    /// Plan 197 Task 7: Structural equality for heap objects (struct instances)
    ///
    /// Compares two heap objects by their structural content rather than by ID.
    /// Both operands are expected to be >= 4000000 (heap object IDs).
    fn struct_eq(&self, a: i32, b: i32) -> bool {
        use crate::vm::generic_registry::GenericInstanceData;
        use crate::vm::heap_object::TypeTag;

        let id_a = a as u64;
        let id_b = b as u64;

        // Quick pointer equality check
        if id_a == id_b {
            return true;
        }

        // Look up both instances
        let obj_a = match self.get_heap_object(id_a) {
            Some(obj) => obj,
            None => return false,
        };
        let obj_b = match self.get_heap_object(id_b) {
            Some(obj) => obj,
            None => return false,
        };

        let guard_a = obj_a.read().unwrap();
        let guard_b = obj_b.read().unwrap();

        // Check both are GenericInstance
        if !matches!(guard_a.type_tag(), TypeTag::GenericInstance(_)) {
            return false;
        }
        if !matches!(guard_b.type_tag(), TypeTag::GenericInstance(_)) {
            return false;
        }

        let inst_a = match guard_a.as_any().downcast_ref::<GenericInstanceData>() {
            Some(inst) => inst,
            None => return false,
        };
        let inst_b = match guard_b.as_any().downcast_ref::<GenericInstanceData>() {
            Some(inst) => inst,
            None => return false,
        };

        // Different types are never equal
        if inst_a.mono_name != inst_b.mono_name {
            return false;
        }

        // Must have same number of fields
        if inst_a.fields.len() != inst_b.fields.len() {
            return false;
        }

        // Compare each field pairwise
        for (fa, fb) in inst_a.fields.iter().zip(inst_b.fields.iter()) {
            if !self.values_equal(fa, fb) {
                return false;
            }
        }

        true
    }

    /// Plan 197 Task 16: Check if a heap object is an Option.None instance
    pub fn is_option_none(&self, instance_id: u64) -> bool {
        use crate::vm::generic_registry::GenericInstanceData;
        if let Some(obj) = self.get_heap_object(instance_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                return instance.mono_name == "Option.None";
            }
        }
        false
    }

    /// Plan 197 Task 16: Check if a heap object is an Option.Some instance
    pub fn is_option_some(&self, instance_id: u64) -> bool {
        use crate::vm::generic_registry::GenericInstanceData;
        if let Some(obj) = self.get_heap_object(instance_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                return instance.mono_name == "Option.Some";
            }
        }
        false
    }

    /// Plan 197 Task 16: Get the inner value from an Option.Some instance
    pub fn get_option_inner(&self, instance_id: u64) -> Option<Value> {
        use crate::vm::generic_registry::GenericInstanceData;
        if let Some(obj) = self.get_heap_object(instance_id) {
            let guard = obj.read().unwrap();
            if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                if instance.mono_name == "Option.Some" {
                    return instance.get_field(0).cloned();
                }
            }
        }
        None
    }

    /// Compare two Value instances for structural equality
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Uint(x), Value::Uint(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x.to_bits() == y.to_bits(),
            (Value::Double(x), Value::Double(y)) => x.to_bits() == y.to_bits(),
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::Nil, Value::Nil) => true,
            (Value::Str(x), Value::Str(y)) => x.as_bytes() == y.as_bytes(),
            (Value::VmRef(ref_a), Value::VmRef(ref_b)) => {
                // Nested heap object — recursive structural equality
                self.struct_eq(ref_a.id as i32, ref_b.id as i32)
            }
            _ => false,
        }
    }

    /// Pop a string value from the stack (returns string index)
    #[allow(dead_code)]
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

    /// Call an Auto closure from native code.
    ///
    /// # Arguments
    /// * `task` - Current task (mutable)
    /// * `closure_id` - ID of closure to call
    /// * `arg_count` - Number of arguments already on stack
    ///
    /// # Stack effect
    /// Pops `arg_count` args + closure_id from stack, pushes result.
    /// After return: result is on top of stack.
    pub fn call_closure(
        &self,
        task: &mut AutoTask,
        closure_id: u32,
        _arg_count: usize,
    ) -> Result<(), VMError> {
        // 1. Clone closure data (can't hold DashMap guard across yields)
        let closure = match self.closures.get(&closure_id) {
            Some(guard) => guard.clone(),
            None => return Err(VMError::RuntimeError(format!("Invalid closure ID: {}", closure_id))),
        };

        // 2. Save current state
        let saved_ip = task.ip;
        let saved_bp = task.bp;
        let saved_closure_id = task.current_closure_id;
        let saved_fn_n_args = task.current_fn_n_args;
        let saved_saved_closure_id = task.saved_closure_id;

        // 3. Setup closure context (mirrors CALL_CLOSURE opcode logic)
        task.current_closure_id = Some(closure_id);
        task.current_fn_n_args = closure.n_args;
        task.saved_closure_id = saved_closure_id;

        // 4. Setup stack frame
        task.ram.push_i32(saved_ip as i32);  // Return address
        task.ram.push_i32(saved_bp as i32);  // Old BP
        task.bp = task.ram.sp - 1;

        // 5. Jump to closure body
        task.ip = closure.func_addr as usize;

        // 6. Execute until closure returns (BP restored to saved_bp)
        let budget = 1_000_000;
        for _ in 0..budget {
            match self.run_one_instruction(task)? {
                StepResult::Continue => {
                    if task.bp == saved_bp {
                        break;
                    }
                    continue;
                }
                StepResult::Terminated => {
                    // Restore state even on error
                    task.current_closure_id = saved_closure_id;
                    task.current_fn_n_args = saved_fn_n_args;
                    return Err(VMError::RuntimeError(
                        "Closure execution terminated unexpectedly".into()
                    ));
                }
                StepResult::Yield => {
                    // In call_closure context, Yield just means "continue after pause"
                    continue;
                }
            }
        }

        // 7. Restore non-stack state
        task.current_closure_id = saved_closure_id;
        task.current_fn_n_args = saved_fn_n_args;
        task.saved_closure_id = saved_saved_closure_id;

        Ok(())
    }

    /// Plan 127: Match a message value against a serialized pattern
    /// Returns true if the message matches the pattern
    fn match_message_pattern_vm(
        &self,
        msg: &auto_val::Value,
        pattern: &crate::vm::task_handler::SerializedPattern,
        string_pool: &[String],
    ) -> bool {
        use crate::vm::task_handler::PatternType;
        use auto_val::Value;

        match pattern.pattern_type {
            PatternType::Literal => {
                if pattern.data.is_empty() {
                    return false;
                }
                let lit_type = pattern.data[0];
                match lit_type {
                    0x01 => {
                        // String literal
                        if pattern.data.len() < 5 { return false; }
                        let idx = u32::from_le_bytes([
                            pattern.data[1], pattern.data[2], pattern.data[3], pattern.data[4]
                        ]) as usize;
                        if let Some(s) = string_pool.get(idx) {
                            matches!(msg, Value::Str(s2) if s2.as_str() == s.as_str())
                        } else {
                            false
                        }
                    }
                    0x02 => {
                        // Int literal
                        if pattern.data.len() < 9 { return false; }
                        let n = i64::from_le_bytes([
                            pattern.data[1], pattern.data[2], pattern.data[3], pattern.data[4],
                            pattern.data[5], pattern.data[6], pattern.data[7], pattern.data[8]
                        ]);
                        matches!(msg, Value::Int(i) if *i as i64 == n)
                    }
                    0x03 => {
                        // Uint literal
                        if pattern.data.len() < 9 { return false; }
                        let n = u64::from_le_bytes([
                            pattern.data[1], pattern.data[2], pattern.data[3], pattern.data[4],
                            pattern.data[5], pattern.data[6], pattern.data[7], pattern.data[8]
                        ]);
                        matches!(msg, Value::Uint(u) if *u as u64 == n)
                    }
                    0x04 => {
                        // Bool literal
                        if pattern.data.len() < 2 { return false; }
                        let b = pattern.data[1] != 0;
                        matches!(msg, Value::Bool(b2) if *b2 == b)
                    }
                    0x05 => {
                        // Char literal
                        if pattern.data.len() < 5 { return false; }
                        let c = u32::from_le_bytes([
                            pattern.data[1], pattern.data[2], pattern.data[3], pattern.data[4]
                        ]);
                        matches!(msg, Value::Char(c2) if (*c2 as u32) == c)
                    }
                    0x06 => {
                        // Float literal (two i64 parts)
                        if pattern.data.len() < 17 { return false; }
                        let _integral = i64::from_le_bytes([
                            pattern.data[1], pattern.data[2], pattern.data[3], pattern.data[4],
                            pattern.data[5], pattern.data[6], pattern.data[7], pattern.data[8]
                        ]);
                        let _fractional = i64::from_le_bytes([
                            pattern.data[9], pattern.data[10], pattern.data[11], pattern.data[12],
                            pattern.data[13], pattern.data[14], pattern.data[15], pattern.data[16]
                        ]);
                        // For now, just check if it's a float type
                        matches!(msg, Value::Float(_) | Value::Double(_))
                    }
                    _ => false,
                }
            }
            PatternType::Simple => {
                // Simple variant pattern - check if message is an object with __variant field
                if pattern.data.len() < 4 { return false; }
                let idx = u32::from_le_bytes([
                    pattern.data[0], pattern.data[1], pattern.data[2], pattern.data[3]
                ]) as usize;
                if let Some(variant_name) = string_pool.get(idx) {
                    match msg {
                        Value::Obj(obj) => {
                            if let Some(Value::Str(v)) = obj.get(auto_val::AutoStr::from("__variant")) {
                                v.as_str() == variant_name.as_str()
                            } else {
                                false
                            }
                        }
                        Value::Str(s) => s.as_str() == variant_name.as_str(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            PatternType::TypeBinding => {
                // Type binding pattern - check if message matches the expected type
                if pattern.data.len() < 5 { return false; }
                let _name_idx = u32::from_le_bytes([
                    pattern.data[0], pattern.data[1], pattern.data[2], pattern.data[3]
                ]) as usize;
                let type_tag = pattern.data[4];

                // Match based on type tag
                match type_tag {
                    0x01 => matches!(msg, Value::Int(_)),      // Int
                    0x02 => matches!(msg, Value::I64(_)),      // I64
                    0x03 => matches!(msg, Value::Uint(_)),     // Uint
                    0x04 => matches!(msg, Value::I64(_)),      // U64 -> I64 (Value doesn't have U64)
                    0x05 => matches!(msg, Value::Float(_)),    // Float
                    0x06 => matches!(msg, Value::Double(_)),   // Double
                    0x07 => matches!(msg, Value::Bool(_)),     // Bool
                    0x08 => matches!(msg, Value::Char(_)),     // Char
                    0x09 => matches!(msg, Value::Str(_)),      // Str
                    0xFF => true,                              // Unknown - match anything
                    _ => false,
                }
            }
            PatternType::WithBindings => {
                // Variant with bindings pattern
                if pattern.data.len() < 5 { return false; }
                let variant_idx = u32::from_le_bytes([
                    pattern.data[0], pattern.data[1], pattern.data[2], pattern.data[3]
                ]) as usize;

                if let Some(variant_name) = string_pool.get(variant_idx) {
                    match msg {
                        Value::Obj(obj) => {
                            if let Some(Value::Str(v)) = obj.get(auto_val::AutoStr::from("__variant")) {
                                v.as_str() == variant_name.as_str()
                            } else {
                                false
                            }
                        }
                        Value::Str(s) => s.as_str() == variant_name.as_str(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
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
                        // Plan 118: Store error for proper error propagation
                        // Plan 199: Include source line number in error message
                        let line = task.current_line;
                        let error_msg = if line > 0 {
                            format!("{:?} at line {}", e, line)
                        } else {
                            format!("{:?}", e)
                        };
                        task.last_error = Some(error_msg.clone());
                        eprintln!("Task {} Error: {}", task.id, error_msg);
                        // Plan 199: Print call stack trace on error
                        if !task.call_stack.is_empty() {
                            eprintln!("Stack trace:");
                            for (i, frame) in task.call_stack.iter().enumerate().rev() {
                                let name = frame.fn_name.as_deref().unwrap_or("<anonymous>");
                                eprintln!("  #{} {} at line {}", i, name, frame.line);
                            }
                        }
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

    /// Execute a single instruction from the given task's instruction stream.
    ///
    /// Returns `Ok(StepResult::Continue)` if the task should keep running,
    /// `Ok(StepResult::Terminated)` if the task has finished, or
    /// `Ok(StepResult::Yield)` if the task should pause the current batch.
    fn run_one_instruction(&self, task: &mut AutoTask) -> Result<StepResult, VMError> {
        // 1. Fetch
        if task.ip >= self.flash.memory.len() {
            return Ok(StepResult::Terminated);
        }

        let op_byte = self.flash.read_u8(task.ip);
        task.ip += 1;
        if !OpCode::is_valid(op_byte) {
            return Err(VMError::RuntimeError(format!("Invalid opcode: 0x{:02x} at ip={}", op_byte, task.ip - 1)));
        }
        let op: OpCode = op_byte.into();

            // Plan 199: Debugger hook — check if we should pause before executing
            {
                let mut dbg = self.debugger.lock().unwrap();
                let ctx = crate::vm::debugger::DebugContext {
                    task: &task,
                    current_op: op,
                    ip: task.ip - 1,
                    line: task.current_line,
                    call_stack: &task.call_stack,
                };
                if dbg.should_pause(&ctx) {
                    let action = dbg.on_pause(&ctx);
                    match action {
                        crate::vm::debugger::DebuggerAction::Quit => {
                            return Err(VMError::RuntimeError("Debugger quit".to_string()));
                        }
                        crate::vm::debugger::DebuggerAction::Continue
                        | crate::vm::debugger::DebuggerAction::Step
                        | crate::vm::debugger::DebuggerAction::StepOver
                        | crate::vm::debugger::DebuggerAction::StepOut => {}
                    }
                }
            }

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
                    vm_debug!("DEBUG: CONST_I32: sp after push={}, wrote to address {}",
                        task.ram.sp,
                        task.ram.sp - 1
                    );
                }
                OpCode::CONST_F32 => {
                    // Plan 073: Fixed to use push_f32 instead of push_i32
                    let val = self.flash.read_f32(task.ip);
                    task.ip += 4;
                    task.ram.push_f32(val);
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
                }
                OpCode::CONST_F64 => {
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
                    // Push string reference (nanbox: direct tag, non-nanbox: negative i32)
                    push_str_tag(&mut task.ram, str_idx as u32);
                    // Reset result type since this produces a string, not a number
                    task.last_result_type = ResultType::default();
                }
                // Plan 073: Node support
                OpCode::CREATE_NODE => {
                    let name_idx = self.flash.read_u16(task.ip);
                    task.ip += 2;
                    let arg_count = self.flash.read_u8(task.ip);
                    task.ip += 1;
                    let id_idx = self.flash.read_u16(task.ip);
                    task.ip += 2;

                    // Pop kids_id and props_id first
                    let kids_id = task.ram.pop_i32();
                    let props_id = task.ram.pop_i32();

                    // Pop args (in reverse order)
                    let mut args = Vec::with_capacity(arg_count as usize);
                    for _ in 0..arg_count {
                        // Inside VM, everything is either a string tag or an int (VmRef id).
                        let val = match pop_tagged(&mut task.ram) {
                            StackTag::Str(str_idx) => {
                                let strings = self.strings.read().unwrap();
                                if let Some(bytes) = strings.get(str_idx as usize) {
                                    auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into())
                                } else {
                                    auto_val::Value::Nil
                                }
                            }
                            StackTag::Int(bits) => {
                                auto_val::Value::VmRef(auto_val::VmRef { id: bits as usize })
                            }
                        };
                        args.push(val);
                    }
                    args.reverse();

                    // Decode name and id
                    let strings = self.strings.read().unwrap();
                    let name = if let Some(bytes) = strings.get(name_idx as usize) {
                        String::from_utf8_lossy(bytes).to_string()
                    } else {
                        "".to_string()
                    };
                    let id = if id_idx != 0xFFFF {
                        strings.get(id_idx as usize)
                            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    drop(strings);

                    let mut node = auto_val::Node::new(&name);
                    if !id.is_empty() {
                        node.id = AutoStr::from(id);
                    }
                    
                    // Assign args
                    for arg in args {
                        node.add_arg(auto_val::Arg::Pos(arg));
                    }

                    // Assign props if available
                    if props_id >= 0 {
                        if let Some(props_ref) = self.objects.get(&(props_id as u64)) {
                            let props_data = props_ref.value().read().unwrap();
                            // Clone properties from ObjectData to Node
                            for (key, val) in &props_data.fields {
                                node.set_prop(key.clone(), val.clone());
                            }
                        }
                    }

                    // Assign kids if available
                    if kids_id >= 0 {
                        // TODO: Implement kids array/list mapping
                    }

                    // Store node in nodes registry
                    let node_id = self.node_id_gen.fetch_add(1, Ordering::SeqCst);
                    self.nodes.insert(node_id, Arc::new(RwLock::new(node)));

                    task.ram.push_i32(node_id as i32);
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
                            // Plan 118: Byte type for object fields
                            ObjectType::Byte => {
                                let bits = task.ram.pop_i32();
                                auto_val::Value::Byte(bits as u8)
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
                                let str_idx = pop_str_idx(&mut task.ram);
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
                            // Plan 118 Phase 4: Void type - should not appear in object fields, but handle gracefully
                            ObjectType::Void => {
                                let _ = task.ram.pop_i32(); // Pop the void value
                                auto_val::Value::Nil
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
                    // Plan 118: Filter out nil marker values for if-in-array support
                    // When if expression has no else branch, it pushes nil (i32::MIN + 1 = -2147483647)
                    // which should be excluded from the array
                    let mut elems = Vec::with_capacity(elem_count as usize);
                    for _ in 0..elem_count {
                        // Pop element and convert to Value
                        let bits = task.ram.pop_i32();
                        // Filter out nil marker (special value: i32::MIN + 1 = -2147483647)
                        // Note: We do NOT filter 0 because 0 is a valid false value, not nil
                        if bits != -2147483647 {
                            // Plan 197 Bug E: detect heap object references like CONSTRUCT_INSTANCE does
                            let value = if bits >= 4000000 {
                                auto_val::Value::VmRef(auto_val::VmRef { id: bits as usize })
                            } else {
                                auto_val::Value::Int(bits)
                            };
                            elems.push(value);
                        }
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
                    vm_debug!("DEBUG CREATE_RANGE_EQ: start={}, end={}, range_id={}",
                        start, end, range_id
                    );

                    // Use special marker for range: -1000000 + range_id
                    task.ram.push_i32(-1000000 + range_id);
                }
                OpCode::ARRAY_LEN => {
                    // Stack: array_id
                    let array_id = task.ram.pop_i32() as u64;

                    // Get array length
                    if let Some(array_ref) = self.arrays.get(&array_id) {
                        let guard = array_ref.read().unwrap();
                        let len = guard.len() as i32;
                        task.ram.push_i32(len);
                    } else if let Some(list) = self.heap_objects.get(&array_id) {
                        use crate::vm::types::ListData;
                        let guard = list.read().unwrap();

                        let len = if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                            list.elems.len() as i32
                        } else if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
                            list.elems.len() as i32
                        } else if let Some(list) = guard.as_any().downcast_ref::<ListData<bool>>() {
                            list.elems.len() as i32
                        } else {
                            0 // Unknown type
                        };
                        task.ram.push_i32(len);
                    } else {
                        // Array not found
                        task.ram.push_i32(0);
                    }
                }
                // Plan 073: F-string support (f"hello $name")
                OpCode::BUILD_FSTR => {
                    let part_count = self.flash.read_u8(task.ip);
                    task.ip += 1;

                    // Read type tags for each part: 0=i32, 1=string, 2=f64, 3=f32, 4=u64
                    let mut type_tags = Vec::with_capacity(part_count as usize);
                    for _ in 0..part_count {
                        type_tags.push(self.flash.read_u8(task.ip));
                        task.ip += 1;
                    }

                    // Pop parts from stack (in reverse order)
                    let mut parts = Vec::with_capacity(part_count as usize);
                    let strings = self.strings.read().unwrap();
                    for i in (0..part_count as usize).rev() {
                        let tag = type_tags[i];
                        let s = match tag {
                            2 => {
                                let val = task.ram.pop_f64();
                                format!("{}", val)
                            }
                            3 => {
                                let val = task.ram.pop_f32();
                                format!("{}", val)
                            }
                            4 => {
                                let val = task.ram.pop_u64();
                                format!("{}", val)
                            }
                            1 => {
                                match pop_tagged(&mut task.ram) {
                                    StackTag::Str(idx) => {
                                        if (idx as usize) < strings.len() {
                                            String::from_utf8_lossy(&strings[idx as usize]).to_string()
                                        } else {
                                            format!("<invalid_str_idx:{}>", idx)
                                        }
                                    }
                                    StackTag::Int(bits) => {
                                        if bits == i32::MIN {
                                            "true".to_string()
                                        } else if bits == i32::MIN + 1 {
                                            "false".to_string()
                                        } else {
                                            bits.to_string()
                                        }
                                    }
                                }
                            }
                            _ => {
                                match pop_tagged(&mut task.ram) {
                                    StackTag::Str(idx) => {
                                        if (idx as usize) < strings.len() {
                                            String::from_utf8_lossy(&strings[idx as usize]).to_string()
                                        } else {
                                            format!("<invalid_str_idx:{}>", idx)
                                        }
                                    }
                                    StackTag::Int(bits) => {
                                        if bits == i32::MIN {
                                            "true".to_string()
                                        } else if bits == i32::MIN + 1 {
                                            "false".to_string()
                                        } else {
                                            bits.to_string()
                                        }
                                    }
                                }
                            }
                        };
                        parts.push(s);
                    }
                    drop(strings);
                    parts.reverse();

                    // Join all parts into a single string
                    let result = parts.join("");

                    // Add to strings pool and push tagged index
                    let mut strings = self.strings.write().unwrap();
                    let result_idx = strings.len();
                    strings.push(result.into_bytes());
                    drop(strings);

                    push_str_tag(&mut task.ram, result_idx as u32);
                }
                OpCode::NULL_COALESCE => {
                    // Pop right expression (default value)
                    let default_bits = task.ram.pop_i32();
                    // Pop left expression (May<T> value)
                    let may_bits = task.ram.pop_i32();

                    // Plan 197 Task 16: Check if May<T> is Option.None (heap object or old -1)
                    let is_none = if may_bits == -1 {
                        true
                    } else if may_bits >= 4000000 {
                        // Check if it's an Option.None heap object
                        self.is_option_none(may_bits as u64)
                    } else {
                        false
                    };

                    if is_none {
                        // Nil case: return default value
                        task.ram.push_i32(default_bits);
                    } else {
                        // Val case: return the unwrapped value
                        // Plan 197 Task 16: If it's an Option.Some, unwrap to get the inner value
                        if may_bits >= 4000000 && self.is_option_some(may_bits as u64) {
                            if let Some(field_val) = self.get_option_inner(may_bits as u64) {
                                Self::push_value(&mut task.ram, &field_val, &self.strings);
                            } else {
                                task.ram.push_i32(may_bits);
                            }
                        } else {
                            task.ram.push_i32(may_bits);
                        }
                    }
                }
                // Plan 073: May<T> error propagate operator: expression.?
                // Plan 208: Also handles Result.Ok / Result.Err heap objects with early return
                OpCode::ERROR_PROPAGATE => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    // Read n_args for potential early return
                    let n_args = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    // Pop May<T> value from stack
                    let may_bits = task.ram.pop_i32();

                    // Determine if this is an error case that should propagate
                    let should_propagate;
                    let mut propagate_value = 0;

                    // Plan 197 Task 16: Check if May<T> is Option.None (heap object or old -1)
                    let is_none = if may_bits == -1 {
                        true
                    } else if may_bits > 0 {
                        // Check if it's an Option.None heap object
                        self.is_option_none(may_bits as u64)
                    } else {
                        false
                    };

                    if is_none {
                        // Nil case: early return (error propagation)
                        // Push an Option.None sentinel for the caller
                        should_propagate = true;
                        propagate_value = -1;
                    } else if may_bits > 0 {
                        // Positive value: could be heap object (Option.Some, Result.Ok, Result.Err)
                        // or legacy plain positive integer
                        if let Some(obj) = self.get_heap_object(may_bits as u64) {
                            let guard = obj.read().unwrap();
                            if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                match inst.mono_name.as_str() {
                                    "Result.Err" => {
                                        // Error case: propagate the Result.Err object to caller via early return
                                        should_propagate = true;
                                        propagate_value = may_bits;
                                    }
                                    "Result.Ok" => {
                                        // Ok case: unwrap the inner value (continue execution)
                                        should_propagate = false;
                                        if let Some(field) = inst.fields.first() {
                                            Self::push_value(&mut task.ram, field, &self.strings);
                                        } else {
                                            task.ram.push_i32(may_bits);
                                        }
                                    }
                                    "Option.Some" => {
                                        // Option.Some: unwrap the inner value (continue execution)
                                        should_propagate = false;
                                        if let Some(field) = inst.fields.first() {
                                            Self::push_value(&mut task.ram, field, &self.strings);
                                        } else {
                                            task.ram.push_i32(may_bits);
                                        }
                                    }
                                    _ => {
                                        // Other heap object: pass through
                                        should_propagate = false;
                                        task.ram.push_i32(may_bits);
                                    }
                                }
                            } else {
                                // Non-generic heap object: pass through
                                should_propagate = false;
                                task.ram.push_i32(may_bits);
                            }
                        } else {
                            // Legacy: plain positive value (not a heap object)
                            should_propagate = false;
                            task.ram.push_i32(may_bits);
                        }
                    } else {
                        // Negative value: legacy Err sentinel or other
                        should_propagate = false;
                        task.ram.push_i32(may_bits);
                    }

                    // Perform early return if propagating error
                    if should_propagate {
                        if task.bp == 0 {
                            // Main task: just push the error value and terminate
                            task.ram.push_i32(propagate_value);
                            return Ok(StepResult::Terminated);
                        }
                        // Perform RET-like frame unwinding
                        let old_bp = task.ram.read_i32(task.bp) as usize;
                        let ret_ip = task.ram.read_i32(task.bp - 1) as usize;
                        task.current_closure_id = task.saved_closure_id;
                        let new_sp = task.bp - n_args;
                        task.ram.write_i32(new_sp - 1, propagate_value);
                        task.bp = old_bp;
                        task.ip = ret_ip;
                        task.ram.sp = new_sp;
                        task.ram.write_i32(new_sp - 1, propagate_value);
                    }
                }
                // Plan 162: Type cast opcodes — runtime type conversion
                OpCode::TYPE_CAST_I32 => {
                    let v = if task.last_result_type == ResultType::Float {
                        let f = task.ram.pop_f32();
                        f as i32
                    } else {
                        task.ram.pop_i32()
                    };
                    task.ram.push_i32(v);
                    task.last_result_type = ResultType::Int;
                }
                OpCode::TYPE_CAST_U32 => {
                    let v = if task.last_result_type == ResultType::Float {
                        let f = task.ram.pop_f32();
                        f as u32 as i32
                    } else {
                        let v = task.ram.pop_i32();
                        v as u32 as i32
                    };
                    task.ram.push_i32(v);
                    task.last_result_type = ResultType::Uint;
                }
                OpCode::TYPE_CAST_I64 => {
                    let v = task.ram.pop_i32();
                    task.ram.push_i32(v);
                    task.last_result_type = ResultType::Int;
                }
                OpCode::TYPE_CAST_U64 => {
                    let v = task.ram.pop_i32();
                    // Zero-extend i32 to u64 (two slots: low, high)
                    task.ram.push_i32(v);   // low 32 bits
                    task.ram.push_i32(0);   // high 32 bits = 0
                }
                OpCode::PROMOTE_F64 => {
                    // Widen f32 (4 bytes, 1 slot) to f64 (8 bytes, 2 slots)
                    let val_f32 = task.ram.pop_f32();
                    task.ram.push_f64(val_f32 as f64);
                }
                OpCode::TYPE_CAST_F64 => {
                    // Always pop i32 and push f32 (1 slot → 1 slot)
                    let v = task.ram.pop_i32();
                    task.ram.push_f32(v as f32);
                    task.last_result_type = ResultType::Float;
                }
                OpCode::TYPE_CAST_PTR => {
                    // Pointer cast — no-op at runtime (same bits)
                }
                // Plan 162: Explicit type conversion (.to) opcodes
                // Plan 197 Task 10: Struct instances formatted as Type { field: val, ... }
                OpCode::TYPE_TO_STR => {
                    let value_bits = task.ram.pop_i32();
                    if value_bits < 0 {
                        task.ram.push_i32(value_bits);
                    } else if value_bits >= 4000000 {
                        // Plan 197 Task 10: Heap object — format struct instance
                        use crate::vm::generic_registry::GenericInstanceData;
                        use crate::vm::heap_object::TypeTag;

                        let obj_id = value_bits as u64;
                        let string_value = match self.get_heap_object(obj_id) {
                            Some(obj) => {
                                let guard = obj.read().unwrap();
                                if let TypeTag::GenericInstance(_) = guard.type_tag() {
                                    if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                        let type_name = self.generic_registry
                                            .get_type(&inst.mono_name)
                                            .map(|ct| ct.base_name().to_string())
                                            .unwrap_or_else(|| inst.mono_name.clone());

                                        let field_strs: Vec<String> = inst.field_names.iter()
                                            .zip(inst.fields.iter())
                                            .map(|(name, val)| {
                                                let val_str = match val {
                                                    Value::Int(i) => i.to_string(),
                                                    Value::Uint(u) => u.to_string(),
                                                    Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                                    Value::Float(f) => f.to_string(),
                                                    Value::Double(d) => d.to_string(),
                                                    Value::Char(c) => format!("'{}'", c),
                                                    Value::Str(s) => format!("\"{}\"", s.as_str()),
                                                    Value::VmRef(r) => format!("<heap:{}>", r.id),
                                                    Value::Nil => "nil".to_string(),
                                                    _ => format!("{:?}", val),
                                                };
                                                format!("{}: {}", name, val_str)
                                            })
                                            .collect();

                                        format!("{} {{ {} }}", type_name, field_strs.join(", "))
                                    } else {
                                        format!("<heap:{}>", value_bits)
                                    }
                                } else {
                                    format!("<heap:{}>", value_bits)
                                }
                            }
                            None => format!("<heap:{}>", value_bits),
                        };

                        let mut strings = self.strings.write().unwrap();
                        let str_idx = strings.len();
                        strings.push(string_value.into_bytes());
                        drop(strings);
                        push_str_tag(&mut task.ram, str_idx as u32);
                    } else {
                        let string_value = format!("{}", value_bits);
                        let mut strings = self.strings.write().unwrap();
                        let str_idx = strings.len();
                        strings.push(string_value.into_bytes());
                        drop(strings);
                        push_str_tag(&mut task.ram, str_idx as u32);
                    }
                }
                OpCode::TYPE_TO_I32 => {
                    match pop_tagged(&mut task.ram) {
                        StackTag::Str(idx) => {
                            let strings = self.strings.read().unwrap();
                            let parsed = strings.get(idx as usize)
                                .and_then(|b| String::from_utf8_lossy(b).trim().parse::<i32>().ok())
                                .unwrap_or(0);
                            drop(strings);
                            task.ram.push_i32(parsed);
                        }
                        StackTag::Int(v) => {
                            task.ram.push_i32(v);
                        }
                    }
                    task.last_result_type = ResultType::Int;
                }
                OpCode::TYPE_TO_F64 => {
                    match pop_tagged(&mut task.ram) {
                        StackTag::Str(idx) => {
                            let strings = self.strings.read().unwrap();
                            let parsed = strings.get(idx as usize)
                                .and_then(|b| String::from_utf8_lossy(b).trim().parse::<f32>().ok())
                                .unwrap_or(0.0);
                            drop(strings);
                            task.ram.push_f32(parsed);
                        }
                        StackTag::Int(v) => {
                            task.ram.push_f32(v as f32);
                        }
                    }
                    task.last_result_type = ResultType::Float;
                }
                // Plan 193: f64 -> String
                OpCode::TYPE_F64_TO_STR => {
                    let val = task.ram.pop_f64();
                    let string_value = format!("{}", val);
                    let mut strings = self.strings.write().unwrap();
                    let str_idx = strings.len();
                    strings.push(string_value.into_bytes());
                    drop(strings);
                    push_str_tag(&mut task.ram, str_idx as u32);
                }
                // Plan 193: i64 -> String
                OpCode::TYPE_I64_TO_STR => {
                    let val = task.ram.pop_i64();
                    let string_value = format!("{}", val);
                    let mut strings = self.strings.write().unwrap();
                    let str_idx = strings.len();
                    strings.push(string_value.into_bytes());
                    drop(strings);
                    push_str_tag(&mut task.ram, str_idx as u32);
                }
                // Plan 193: u64 -> String (hex)
                OpCode::TYPE_U64_TO_STR => {
                    let val = task.ram.pop_u64();
                    let string_value = format!("{:08x}", val);
                    let mut strings = self.strings.write().unwrap();
                    let str_idx = strings.len();
                    strings.push(string_value.into_bytes());
                    drop(strings);
                    push_str_tag(&mut task.ram, str_idx as u32);
                }
                // Plan 193: bool -> String
                OpCode::TYPE_BOOL_TO_STR => {
                    let val = task.ram.pop_i32();
                    let string_value = if val != 0 { "true" } else { "false" };
                    let mut strings = self.strings.write().unwrap();
                    let str_idx = strings.len();
                    strings.push(string_value.as_bytes().to_vec());
                    drop(strings);
                    push_str_tag(&mut task.ram, str_idx as u32);
                }
                // Plan 193: f64 -> i32 (truncate)
                OpCode::TYPE_F64_TO_I32 => {
                    let val = task.ram.pop_f64();
                    task.ram.push_i32(val as i32);
                    task.last_result_type = ResultType::Int;
                }
                // Plan 193: String -> i64
                OpCode::TYPE_STR_TO_I64 => {
                    match pop_tagged(&mut task.ram) {
                        StackTag::Str(idx) => {
                            let strings = self.strings.read().unwrap();
                            let parsed = strings.get(idx as usize)
                                .and_then(|b| String::from_utf8_lossy(b).trim().parse::<i64>().ok())
                                .unwrap_or(0i64);
                            drop(strings);
                            task.ram.push_i64(parsed);
                        }
                        StackTag::Int(v) => {
                            task.ram.push_i64(v as i64);
                        }
                    }
                }
                // Plan 193: f32 -> String
                OpCode::TYPE_F32_TO_STR => {
                    let val = task.ram.pop_f32();
                    let string_value = format!("{}", val);
                    let mut strings = self.strings.write().unwrap();
                    let str_idx = strings.len();
                    strings.push(string_value.into_bytes());
                    drop(strings);
                    push_str_tag(&mut task.ram, str_idx as u32);
                }
                // Plan 193: f32 -> i32 (truncate)
                OpCode::TYPE_F32_TO_I32 => {
                    let val = task.ram.pop_f32();
                    task.ram.push_i32(val as i32);
                    task.last_result_type = ResultType::Int;
                }
                // Plan 075: Convert any value to string
                // Plan 197 Task 10: Struct instances formatted as Type { field: val, ... }
                OpCode::TO_STR => {
                    // Pop value from stack
                    let value_bits = task.ram.pop_i32();

                    // If already a tagged string, just push it back
                    if value_bits < 0 {
                        task.ram.push_i32(value_bits);
                    } else if value_bits >= 4000000 {
                        // Plan 197 Task 10: Heap object — format struct instance
                        use crate::vm::generic_registry::GenericInstanceData;
                        use crate::vm::heap_object::TypeTag;

                        let obj_id = value_bits as u64;
                        let string_value = match self.get_heap_object(obj_id) {
                            Some(obj) => {
                                let guard = obj.read().unwrap();
                                if let TypeTag::GenericInstance(_) = guard.type_tag() {
                                    if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                        // Extract the base type name from mono_name
                                        // mono_name is like "Point" or "Pair_int_str"
                                        // We want just the base name before the first '_' that
                                        // starts a type parameter, but for non-generic types
                                        // the mono_name IS the base name.
                                        // Use base_name from generic_registry if available.
                                        let type_name = self.generic_registry
                                            .get_type(&inst.mono_name)
                                            .map(|ct| ct.base_name().to_string())
                                            .unwrap_or_else(|| inst.mono_name.clone());

                                        // Format each field
                                        let field_strs: Vec<String> = inst.field_names.iter()
                                            .zip(inst.fields.iter())
                                            .map(|(name, val)| {
                                                let val_str = match val {
                                                    Value::Int(i) => i.to_string(),
                                                    Value::Uint(u) => u.to_string(),
                                                    Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                                    Value::Float(f) => f.to_string(),
                                                    Value::Double(d) => d.to_string(),
                                                    Value::Char(c) => format!("'{}'", c),
                                                    Value::Str(s) => format!("\"{}\"", s.as_str()),
                                                    Value::VmRef(r) => format!("<heap:{}>", r.id),
                                                    Value::Nil => "nil".to_string(),
                                                    _ => format!("{:?}", val),
                                                };
                                                format!("{}: {}", name, val_str)
                                            })
                                            .collect();

                                        format!("{} {{ {} }}", type_name, field_strs.join(", "))
                                    } else {
                                        format!("<heap:{}>", value_bits)
                                    }
                                } else {
                                    format!("<heap:{}>", value_bits)
                                }
                            }
                            None => format!("<heap:{}>", value_bits),
                        };

                        // Add to strings pool and push tagged index
                        let mut strings = self.strings.write().unwrap();
                        let str_idx = strings.len();
                        strings.push(string_value.into_bytes());
                        drop(strings);

                        push_str_tag(&mut task.ram, str_idx as u32);
                    } else {
                        // Convert integer to its string representation
                        let string_value = format!("{}", value_bits);

                        // Add to strings pool and push tagged index
                        let mut strings = self.strings.write().unwrap();
                        let str_idx = strings.len();
                        strings.push(string_value.into_bytes());
                        drop(strings);

                        push_str_tag(&mut task.ram, str_idx as u32);
                    }
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
                    // Pop right value first (top of stack)
                    let right_bits = task.ram.pop_i32();
                    // Pop left value
                    let left_bits = task.ram.pop_i32();

                    // Decode values: negative = tagged string index, non-negative = integer
                    let strings = self.strings.read().unwrap();

                    let left_str = if left_bits < 0 {
                        let idx = (-left_bits - 1) as usize;
                        strings
                            .get(idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .unwrap_or_default()
                    } else {
                        left_bits.to_string()
                    };

                    let right_str = if right_bits < 0 {
                        let idx = (-right_bits - 1) as usize;
                        strings
                            .get(idx)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                            .unwrap_or_default()
                    } else {
                        right_bits.to_string()
                    };
                    drop(strings);

                    // Concatenate strings
                    let result = format!("{}{}", left_str, right_str);

                    // Add result to strings pool and push tagged index
                    let mut strings = self.strings.write().unwrap();
                    let result_idx = strings.len();
                    strings.push(result.into_bytes());
                    drop(strings);

                    push_str_tag(&mut task.ram, result_idx as u32);
                }
                // Plan 120: Option type constructor - Some(value)
                OpCode::CREATE_SOME => {
                    // Value is already on stack, just tag it as Some
                    // We use a special encoding: Some values are positive, None is -1
                    // The value is already on stack, no change needed for now
                    // This opcode is a marker for type tracking
                    // TODO: Implement proper Option<T> type tracking in VM
                }
                // Plan 120: Option type constructor - None
                OpCode::CREATE_NONE => {
                    // Push None onto stack (represented as -1)
                    task.ram.push_i32(-1);
                }
                // Plan 120: Result type constructor - Ok(value)
                // Plan 208: Wrap value in a Result.Ok heap object
                OpCode::CREATE_OK => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let val = task.ram.pop_i32();
                    let instance = GenericInstanceData::new("Result.Ok".to_string(), vec![auto_val::Value::Int(val)]);
                    let instance_id = self.insert_heap_object(instance);
                    task.ram.push_i32(instance_id as i32);
                }
                // Plan 120: Result type constructor - Err(message)
                // Plan 208: Wrap error value in a Result.Err heap object
                OpCode::CREATE_ERR => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let err_val = task.ram.pop_i32();
                    let instance = GenericInstanceData::new("Result.Err".to_string(), vec![auto_val::Value::Int(err_val)]);
                    let instance_id = self.insert_heap_object(instance);
                    task.ram.push_i32(instance_id as i32);
                }
                // Plan 120: Check if Option is Some
                OpCode::IS_SOME => {
                    let value = task.ram.pop_i32();
                    // Some: value >= 0, None: value == -1
                    let is_some = if value >= 0 { 1 } else { 0 };
                    task.ram.push_i32(is_some);
                }
                // Plan 120: Check if Result is Ok
                // Plan 208: Check heap object mono_name instead of sign
                OpCode::IS_OK => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let value = task.ram.pop_i32();
                    let is_ok = if value > 0 {
                        if let Some(obj) = self.get_heap_object(value as u64) {
                            let guard = obj.read().unwrap();
                            if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                                inst.mono_name == "Result.Ok"
                            } else {
                                // Legacy: plain positive value = Ok
                                true
                            }
                        } else {
                            // Legacy: plain positive value = Ok
                            true
                        }
                    } else {
                        false
                    };
                    // VM boolean convention: i32::MIN = true, i32::MIN+1 = false
                    task.ram.push_i32(if is_ok { -2147483648 } else { -2147483647 });
                }
                // Plan 120: Unwrap Option (panic if None)
                OpCode::UNWRAP_SOME => {
                    let value = task.ram.pop_i32();
                    if value == -1 {
                        // Panic on None
                        return Err(VMError::RuntimeError("called unwrap on None".to_string()));
                    }
                    // Push the unwrapped value back
                    task.ram.push_i32(value);
                }
                // Plan 120: Unwrap Result (panic if Err)
                // Plan 208: Extract field[0] from Result.Ok heap object
                OpCode::UNWRAP_OK => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let value = task.ram.pop_i32();
                    if value <= 0 {
                        return Err(VMError::RuntimeError("called unwrap on Err".to_string()));
                    }
                    if let Some(obj) = self.get_heap_object(value as u64) {
                        let guard = obj.read().unwrap();
                        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                            if inst.mono_name == "Result.Ok" {
                                if let Some(field) = inst.fields.first() {
                                    Self::push_value(&mut task.ram, field, &self.strings);
                                    return Ok(StepResult::Continue);
                                }
                            }
                        }
                    }
                    // Legacy fallback: plain positive value
                    task.ram.push_i32(value);
                }
                // Plan 120: Unwrap Result error (panic if Ok)
                // Plan 208: Extract field[0] from Result.Err heap object
                OpCode::UNWRAP_ERR => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let value = task.ram.pop_i32();
                    if value <= 0 {
                        return Err(VMError::RuntimeError("called unwrap_err on non-heap value".to_string()));
                    }
                    if let Some(obj) = self.get_heap_object(value as u64) {
                        let guard = obj.read().unwrap();
                        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                            if inst.mono_name == "Result.Err" {
                                if let Some(field) = inst.fields.first() {
                                    Self::push_value(&mut task.ram, field, &self.strings);
                                    return Ok(StepResult::Continue);
                                }
                            }
                        }
                    }
                    task.ram.push_i32(value);
                }
                // Plan 076 Phase 3 & 4: Generic List opcodes with storage strategies
                OpCode::CREATE_LIST_INT => {
                    // Plan 077 Phase 5: Create List<int> in unified registry
                    use crate::vm::types::ListData;
                    let list_data: ListData<i32> = ListData::new(); // Heap storage (default)
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                OpCode::CREATE_LIST_STR => {
                    // Plan 077 Phase 5: Create List<String> in unified registry
                    use crate::vm::types::ListData;
                    let list_data: ListData<String> = ListData::new(); // Heap storage (default)
                    let list_id = self.insert_heap_object(list_data);

                    // Push list ID onto stack
                    task.ram.push_i32(list_id as i32);
                }
                OpCode::CREATE_LIST_BOOL => {
                    // Plan 077 Phase 5: Create List<bool> in unified registry
                    use crate::vm::types::ListData;
                    let list_data: ListData<bool> = ListData::new(); // Heap storage (default)
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

                    vm_debug!("DEBUG NEW_INSTANCE: Stack depth before pop = {}",
                        task.ram.sp
                    );

                    // Read mono_name length from stack
                    let name_len = task.ram.pop_i32() as usize;
                    vm_debug!("DEBUG NEW_INSTANCE: Popped name_len = {}", name_len);

                    // Read mono_name bytes from flash memory and convert to String
                    // Note: task.ip already points to the first byte after the opcode (advanced by main loop)
                    let mut name_bytes = vec![0u8; name_len];
                    for i in 0..name_len {
                        let byte_addr = task.ip.wrapping_add(i);
                        name_bytes[i] = self.flash.read_u8(byte_addr);
                    }

                    // Advance IP past the name bytes
                    task.ip = task.ip.wrapping_add(name_len);

                    let mono_name = String::from_utf8(name_bytes).map_err(|e| {
                        VMError::RuntimeError(format!("Invalid UTF-8 in mono_name: {}", e))
                    })?;
                    vm_debug!("DEBUG NEW_INSTANCE: mono_name = '{}'", mono_name);

                    // Create instance with no fields (uninitialized)
                    let instance = GenericInstanceData::new(mono_name, vec![]);
                    let instance_id = self.insert_heap_object(instance);

                    // Push instance ID onto stack
                    vm_debug!("DEBUG NEW_INSTANCE: Pushing instance_id = {}", instance_id);
                    task.ram.push_i32(instance_id as i32);
                }
                OpCode::CONSTRUCT_INSTANCE => {
                    // Plan 087 Phase 2: Populate fields of a generic instance
                    // Stack layout: [..., value1, value2, ..., valueN, instance_id, field_count]
                    // Stack after: [..., instance_id]  (instance_id left on stack for variable assignment)
                    use crate::vm::generic_registry::GenericInstanceData;
                    use crate::vm::heap_object::TypeTag;

                    vm_debug!("DEBUG CONSTRUCT_INSTANCE: Stack depth before pop = {}",
                        task.ram.sp
                    );

                    // Pop field_count (top of stack)
                    let field_count = task.ram.pop_i32() as usize;
                    vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped field_count = {}",
                        field_count
                    );

                    // Pop instance_id (next on stack)
                    let instance_id = task.ram.pop_i32() as u64;
                    vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped instance_id = {}",
                        instance_id
                    );

                    // Peek mono_name from the instance to look up field types
                    let mono_name = if let Some(obj) = self.get_heap_object(instance_id) {
                        let guard = obj.read().unwrap();
                        guard.as_any().downcast_ref::<GenericInstanceData>()
                            .map(|inst| inst.mono_name.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    // Pop values from stack (in reverse order)
                    // Use field types from generic registry to correctly interpret stack values
                    let field_types: Vec<crate::ast::Type> = self.generic_registry
                        .get_type(&mono_name)
                        .map(|ct| ct.template.fields.iter().map(|f| f.field_type.clone()).collect())
                        .unwrap_or_else(|| vec![]);
                    let mut field_values = Vec::with_capacity(field_count);
                    for i in 0..field_count {
                        vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popping value {}/{}, stack depth = {}",
                            i + 1,
                            field_count,
                            task.ram.sp
                        );
                        // Fields are popped in reverse order; look up type from the end
                        let type_idx = field_count.saturating_sub(1 + i);
                        let field_type = field_types.get(type_idx);

                        let value = match field_type {
                            Some(crate::ast::Type::Float) => {
                                let val_f32 = task.ram.pop_f32();
                                vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped float = {}", val_f32);
                                Value::Float(val_f32 as f64)
                            }
                            Some(crate::ast::Type::Double) => {
                                let val_f64 = task.ram.pop_f64();
                                vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped double = {}", val_f64);
                                Value::Double(val_f64)
                            }
                            _ => {
                                // Pop value as i32 for all other types
                                let val_i32 = task.ram.pop_i32();
                                vm_debug!("DEBUG CONSTRUCT_INSTANCE: Popped value = {}", val_i32);

                                // Plan 197 Task 16: Detect string, heap object, or basic integer
                                // Strings are encoded as -(idx+1) (negative)
                                // Heap objects are >= 4000000
                                // Integers are everything else
                                if val_i32 >= 4000000 {
                                    // This is likely a heap object reference
                                    Value::VmRef(auto_val::VmRef { id: val_i32 as usize })
                                } else if val_i32 < 0 {
                                    // Tagged string index: -(idx+1)
                                    let idx = (-val_i32 - 1) as usize;
                                    let strings_guard = self.strings.read().unwrap();
                                    if idx < strings_guard.len() {
                                        let s = String::from_utf8_lossy(&strings_guard[idx]).to_string();
                                        drop(strings_guard);
                                        Value::Str(auto_val::AutoStr::from(s))
                                    } else {
                                        drop(strings_guard);
                                        Value::Int(val_i32)
                                    }
                                } else {
                                    // Basic integer type
                                    Value::Int(val_i32)
                                }
                            }
                        };
                        field_values.push(value);
                    }
                    field_values.reverse(); // Reverse to get correct order

                    vm_debug!("DEBUG CONSTRUCT_INSTANCE: Field values (reversed): {:?}",
                        field_values
                    );

                    // Get instance and populate fields
                    if let Some(obj) = self.get_heap_object(instance_id) {
                        let mut guard = obj.write().unwrap();

                        // Check if this is a GenericInstance by checking the type tag
                        let is_generic_instance =
                            matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                        if is_generic_instance {
                            // Use as_any_mut for downcasting (works without exact TypeTag match)
                            if let Some(instance) =
                                guard.as_any_mut().downcast_mut::<GenericInstanceData>()
                            {
                                let field_count = field_values.len();
                                instance.fields = field_values;

                                // Plan 197 Task 9: Populate field_names from generic registry
                                let field_names = self.generic_registry
                                    .get_type(&instance.mono_name)
                                    .map(|ct| ct.template.fields.iter().map(|f| f.name.clone()).collect())
                                    .unwrap_or_else(|| vec!["_unknown".to_string(); field_count]);
                                instance.field_names = field_names;

                                vm_debug!("DEBUG CONSTRUCT_INSTANCE: Successfully populated {} fields",
                                    field_count
                                );
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Type error: Failed to downcast GenericInstance"
                                )));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: CONSTRUCT_INSTANCE expected GenericInstance, got {:?}",
                                guard.type_tag()
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid instance ID: {}",
                            instance_id
                        )));
                    }

                    // Push instance_id back onto stack for variable assignment
                    // Stack layout after: [..., instance_id]
                    vm_debug!("DEBUG CONSTRUCT_INSTANCE: Pushing instance_id back to stack: {}",
                        instance_id
                    );
                    task.ram.push_i32(instance_id as i32);
                    vm_debug!("DEBUG CONSTRUCT_INSTANCE: Stack depth after = {}",
                        task.ram.sp
                    );
                }
                OpCode::IS_VARIANT => {
                    // Plan 197 Task 15: Check if heap object is a GenericInstanceData with matching mono_name
                    // Code layout: [opcode, name_len:u16, name_bytes...]
                    // Stack layout: [..., instance_id]
                    // Stack after: [..., bool] (instance_id consumed, bool pushed)
                    use crate::vm::generic_registry::GenericInstanceData;

                    // Read name_len from code stream
                    let name_len = self.flash.read_u16(task.ip) as usize;
                    task.ip += 2;

                    // Read name bytes from code stream
                    let mut name_bytes = vec![0u8; name_len];
                    for i in 0..name_len {
                        name_bytes[i] = self.flash.read_u8(task.ip);
                        task.ip += 1;
                    }
                    let expected_name = String::from_utf8_lossy(&name_bytes).to_string();

                    // Read instance_id from stack
                    let instance_id = task.ram.pop_i32() as u64;

                    vm_debug!("DEBUG: IS_VARIANT: instance_id={}, expected_name='{}'",
                        instance_id, expected_name
                    );

                    // Check if it's a GenericInstanceData with matching mono_name
                    let result = if let Some(obj) = self.get_heap_object(instance_id) {
                        let guard = obj.read().unwrap();
                        if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                            instance.mono_name == expected_name
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Push boolean result (true = i32::MIN, false = i32::MIN+1)
                    task.ram.push_i32(if result { -2147483648 } else { -2147483647 });
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

                    vm_debug!("DEBUG: GET_GENERIC_FIELD: instance_id={}, field_index={}",
                        instance_id, field_index
                    );

                    // Get instance and read field
                    if let Some(obj) = self.get_heap_object(instance_id) {
                        let guard = obj.read().unwrap();

                        // Check if this is a GenericInstance (any variant)
                        let is_generic_instance =
                            matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                        if is_generic_instance {
                            if let Some(instance) =
                                guard.as_any().downcast_ref::<GenericInstanceData>()
                            {
                                if let Some(value) = instance.get_field(field_index) {
                                    // Pop instance_id (we already read it)
                                    let _ = task.ram.pop_i32();
                                    // Push field value onto stack
                                    Self::push_value(&mut task.ram, value, &self.strings);
                                    vm_debug!("DEBUG: GET_GENERIC_FIELD: field value = {:?}",
                                        value
                                    );
                                } else {
                                    return Err(VMError::RuntimeError(format!(
                                        "Field index {} out of bounds (instance has {} fields)",
                                        field_index,
                                        instance.field_count()
                                    )));
                                }
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Type error: GET_GENERIC_FIELD failed to downcast GenericInstance")));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: GET_GENERIC_FIELD expected GenericInstance, got {:?}",
                                guard.type_tag()
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid instance ID: {}",
                            instance_id
                        )));
                    }
                }
                OpCode::SET_GENERIC_FIELD => {
                    // Plan 087 Phase 2: Set field value in generic instance
                    // Plan 118 Phase 7: Stack layout changed to [..., value, instance_id]
                    // Code layout: [opcode, field_index:u32]
                    // Stack layout: [..., value, instance_id] (value pushed first, then instance_id)
                    // Stack after: [...]
                    use crate::vm::generic_registry::GenericInstanceData;
                    use crate::vm::heap_object::TypeTag;

                    vm_debug!("DEBUG: SET_GENERIC_FIELD executing at IP={}", task.ip);

                    // Read field_index from code stream (not stack!)
                    let field_index = self.flash.read_u32(task.ip) as usize;
                    task.ip += 4;

                    // Pop instance_id (stack top)
                    let instance_id = task.ram.pop_i32() as u64;

                    // Pop value (below instance_id)
                    let val_i32 = task.ram.pop_i32();
                    // Check if value is a heap object reference (>= 4000000)
                    let value = if val_i32 >= 4000000 {
                        Value::VmRef(auto_val::VmRef { id: val_i32 as usize })
                    } else {
                        Value::Int(val_i32)
                    };

                    vm_debug!("DEBUG: SET_GENERIC_FIELD: instance_id={}, field_index={}, value={:?}",
                        instance_id, field_index, value
                    );

                    // Get instance and set field
                    if let Some(obj) = self.get_heap_object(instance_id) {
                        let mut guard = obj.write().unwrap();

                        // Check if this is a GenericInstance (any variant)
                        let is_generic_instance =
                            matches!(guard.type_tag(), TypeTag::GenericInstance(_));

                        if is_generic_instance {
                            if let Some(instance) =
                                guard.as_any_mut().downcast_mut::<GenericInstanceData>()
                            {
                                let value_repr = format!("{:?}", value); // Capture before move
                                instance.set_field(field_index, value).map_err(|e| {
                                    VMError::RuntimeError(format!("Failed to set field: {}", e))
                                })?;
                                vm_debug!("DEBUG: SET_GENERIC_FIELD: successfully set field {} to {}",
                                    field_index, value_repr
                                );
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Type error: SET_GENERIC_FIELD failed to downcast GenericInstance")));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: SET_GENERIC_FIELD expected GenericInstance, got {:?}",
                                guard.type_tag()
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid instance ID: {}",
                            instance_id
                        )));
                    }
                }
                OpCode::LIST_PUSH_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id, value:int]
                    // Pop value first (top of stack), then list_id
                    let value = task.ram.pop_i32();
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
                    use crate::vm::types::ListData;

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) =
                            try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
                        {
                            if !list.push(value) {
                                return Err(VMError::RuntimeError(format!(
                                    "List capacity exceeded (InlineInt64 limit: 64)"
                                )));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_PUSH_INT expected ListInt"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}",
                            list_id
                        )));
                    }
                }
                OpCode::LIST_POP_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id]
                    // Pop list_id, get list, pop element, push result
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
                    use crate::vm::types::ListData;

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) =
                            try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
                        {
                            let value = list.pop().unwrap_or(0);
                            task.ram.push_i32(value);
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_POP_INT expected ListInt"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}",
                            list_id
                        )));
                    }
                }
                OpCode::LIST_GET_INT => {
                    // Plan 077 Phase 7: Optimized with inline helper
                    // Stack layout: [..., list_id, index:int]
                    // Pop index first (top of stack), then list_id
                    let index = task.ram.pop_i32() as usize;
                    let list_id = task.ram.pop_i32() as u64;

                    // Get list from unified registry and downcast to ListData<i32>
                    use crate::vm::heap_object::{try_downcast_checked, TypeTag};
                    use crate::vm::types::ListData;

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let guard = obj.read().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) =
                            try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt)
                        {
                            let value = list.get(index).copied().unwrap_or(0);
                            task.ram.push_i32(value);
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_GET_INT expected ListInt"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}",
                            list_id
                        )));
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
                    use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
                    use crate::vm::types::ListData;

                    if let Some(obj) = self.get_heap_object(list_id) {
                        let mut guard = obj.write().unwrap();

                        // Optimized: single type check + downcast (inline)
                        if let Some(list) =
                            try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
                        {
                            if !list.set(index, value) {
                                return Err(VMError::RuntimeError(format!(
                                    "List index out of bounds: {}",
                                    index
                                )));
                            }
                        } else {
                            return Err(VMError::RuntimeError(format!(
                                "Type error: LIST_SET_INT expected ListInt"
                            )));
                        }
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid list ID: {}",
                            list_id
                        )));
                    }
                }
                // Slice: stack: container, start, end -> new_container
                OpCode::SLICE => {
                    let end = task.ram.pop_i32();
                    let start = task.ram.pop_i32();
                    let container = task.ram.pop_i32();

                    // Tagged string slice
                    if container < 0 && container > -1000000 && container != -2147483648 {
                        let str_idx = (-container - 1) as usize;
                        let strings = self.strings.read().unwrap();
                        if let Some(bytes) = strings.get(str_idx) {
                            let s = String::from_utf8_lossy(bytes).to_string();
                            let chars: Vec<char> = s.chars().collect();
                            let len = chars.len();
                            let s_start = if start < 0 { 0 } else { (start as usize).min(len) };
                            let s_end = if end < 0 { len } else { (end as usize).min(len) };
                            let sliced: String = chars[s_start..s_end].iter().collect();
                            drop(strings);
                            let new_idx = self.add_string(sliced.into_bytes());
                            push_str_tag(&mut task.ram, new_idx as u32);
                        } else {
                            task.ram.push_i32(0);
                        }
                    } else {
                        // Array slice
                        let arr_key = container as u64;
                        if let Some(arr_lock) = self.arrays.get(&arr_key) {
                            let arr = arr_lock.read().unwrap();
                            let len = arr.len();
                            let s_start = if start < 0 { 0 } else { (start as usize).min(len) };
                            let s_end = if end < 0 { len } else { (end as usize).min(len) };
                            let sliced: Vec<auto_val::Value> = arr[s_start..s_end].to_vec();
                            drop(arr);
                            let new_id = self.array_id_gen.fetch_add(1, Ordering::SeqCst);
                            self.arrays.insert(new_id, Arc::new(RwLock::new(sliced)));
                            task.ram.push_i32(new_id as i32);
                        } else {
                            task.ram.push_i32(0);
                        }
                    }
                }
                // Plan 200: Create tuple from stack elements
                // Stack: elem0, elem1, ..., elemN-1 -> tuple_id
                OpCode::CREATE_TUPLE => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let elem_count = self.flash.read_u8(task.ip);
                    task.ip += 1;
                    let mut fields = Vec::with_capacity(elem_count as usize);
                    for _ in 0..elem_count {
                        let val_i32 = task.ram.pop_i32();
                        // Detect string (negative tagged) or int
                        let val = if val_i32 < 0 && val_i32 > -1000000 && val_i32 != -2147483648 {
                            let str_idx = (-val_i32 - 1) as usize;
                            let strings = self.strings.read().unwrap();
                            if let Some(bytes) = strings.get(str_idx) {
                                auto_val::Value::Str(String::from_utf8_lossy(bytes).to_string().into())
                            } else {
                                auto_val::Value::Int(val_i32)
                            }
                        } else {
                            auto_val::Value::Int(val_i32)
                        };
                        fields.push(val);
                    }
                    fields.reverse();
                    let mut data = GenericInstanceData::new(
                        format!("tuple_{}", elem_count).into(),
                        vec![auto_val::Value::Null; fields.len()],
                    );
                    for (i, val) in fields.into_iter().enumerate() {
                        let _ = data.set_field(i, val);
                    }
                    let instance_id = self.insert_heap_object(data);
                    task.ram.push_i32(instance_id as i32);
                }
                // Plan 200: Get tuple field by index
                // Stack: tuple_id -> value (field_index from bytecode)
                OpCode::GET_TUPLE_FIELD => {
                    use crate::vm::generic_registry::GenericInstanceData;
                    let field_index = self.flash.read_u8(task.ip);
                    task.ip += 1;
                    let tuple_id = task.ram.pop_i32() as u64;
                    if let Some(lock) = self.get_heap_object(tuple_id) {
                        let guard = lock.read().unwrap();
                        if let Some(instance) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                            if let Some(val) = instance.get_field(field_index as usize) {
                                match val {
                                    auto_val::Value::Int(n) => task.ram.push_i32(*n),
                                    auto_val::Value::Bool(b) => task.ram.push_i32(if *b { 1 } else { 0 }),
                                    auto_val::Value::Str(s) => {
                                        let idx = self.add_string(s.as_bytes().to_vec());
                                        push_str_tag(&mut task.ram, idx as u32);
                                    }
                                    _ => task.ram.push_i32(0),
                                }
                            } else {
                                task.ram.push_i32(0);
                            }
                        } else {
                            task.ram.push_i32(0);
                        }
                    } else {
                        task.ram.push_i32(0);
                    }
                }
                // Plan 073: Array element access (arr[index])
                // Plan 080: Also supports heap objects (lists like List<int>)
                // Plan 118 Phase 4: Also supports string indexing (str[index])
                OpCode::GET_ELEM => {
                    // Stack: array_id/list_id/str_id, index
                    // Pop index first (top of stack)
                    let index_i32 = task.ram.pop_i32();
                    // Pop array_id/list_id or str_id (tagged)
                    let obj_or_str_bits = task.ram.pop_i32();

                    // Helper function to convert negative index to actual index
                    // e.g., for array of length 3: -1 -> 2, -2 -> 1, -3 -> 0
                    let normalize_index = |idx: i32, len: usize| -> Option<usize> {
                        if idx >= 0 {
                            let uidx = idx as usize;
                            if uidx < len { Some(uidx) } else { None }
                        } else {
                            // Negative index: -1 means last element, -2 means second-to-last, etc.
                            let from_end = (-idx) as usize;
                            if from_end <= len && from_end > 0 { Some(len - from_end) } else { None }
                        }
                    };

                    vm_debug!("DEBUG GET_ELEM: obj_or_str_bits={}, index={}", obj_or_str_bits, index_i32);

                    // Check if this is a tagged string index (negative value)
                    if obj_or_str_bits < 0 && obj_or_str_bits > -1000000 && obj_or_str_bits != -2147483648 {
                        // This is a tagged string index - string indexing operation
                        let str_idx = (-obj_or_str_bits - 1) as usize;
                        let strings = self.strings.read().unwrap();
                        if let Some(bytes) = strings.get(str_idx) {
                            // Get the character at the specified index
                            // Convert bytes to string and get char
                            let s = String::from_utf8_lossy(bytes);
                            let char_count = s.chars().count();
                            if let Some(normalized_idx) = normalize_index(index_i32, char_count) {
                                if let Some(ch) = s.chars().nth(normalized_idx) {
                                    vm_debug!("DEBUG GET_ELEM: String[{}] = '{}'", normalized_idx, ch);
                                    // Push character as i32 (Unicode code point)
                                    task.ram.push_i32(ch as i32);
                                } else {
                                    vm_debug!("DEBUG GET_ELEM: String index {} out of bounds", normalized_idx);
                                    task.ram.push_i32(0); // Out of bounds
                                }
                            } else {
                                vm_debug!("DEBUG GET_ELEM: String index {} out of bounds", index_i32);
                                task.ram.push_i32(0); // Out of bounds
                            }
                        } else {
                            vm_debug!("DEBUG GET_ELEM: Invalid string index {}", str_idx);
                            task.ram.push_i32(0); // Invalid string index
                        }
                    } else {
                        // Regular array/list access
                        let obj_id = obj_or_str_bits as u64;

                        // First, try heap_objects registry (Plan 077 unified registry)
                        if let Some(obj) = self.get_heap_object(obj_id) {
                            use crate::vm::types::ListData;
                            let guard = obj.read().unwrap();

                            // Try List<int>
                            if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                                vm_debug!("DEBUG GET_ELEM: Found List<int> with {} elems",
                                    list.elems.len()
                                );
                                if let Some(normalized_idx) = normalize_index(index_i32, list.elems.len()) {
                                    let elem = list.elems[normalized_idx];
                                    vm_debug!("DEBUG GET_ELEM: Returning elem[{}]={}", normalized_idx, elem);
                                    task.ram.push_i32(elem);
                                } else {
                                    vm_debug!("DEBUG GET_ELEM: Index {} out of bounds", index_i32);
                                    task.ram.push_i32(0); // Out of bounds
                                }
                            }
                            // Try List<String>
                            else if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>()
                            {
                                vm_debug!("DEBUG GET_ELEM: Found List<String>");
                                if let Some(normalized_idx) = normalize_index(index_i32, list.elems.len()) {
                                    // TODO: Support string elements (currently push placeholder)
                                    let _elem = &list.elems[normalized_idx];
                                    task.ram.push_i32(0);
                                } else {
                                    task.ram.push_i32(0); // Out of bounds
                                }
                            }
                            // Try List<bool>
                            else if let Some(list) = guard.as_any().downcast_ref::<ListData<bool>>() {
                                vm_debug!("DEBUG GET_ELEM: Found List<bool>");
                                if let Some(normalized_idx) = normalize_index(index_i32, list.elems.len()) {
                                    let elem = list.elems[normalized_idx];
                                    task.ram.push_i32(if elem { 1 } else { 0 });
                                } else {
                                    task.ram.push_i32(0); // Out of bounds
                                }
                            } else {
                                vm_debug!("DEBUG GET_ELEM: Unknown heap object type");
                                task.ram.push_i32(0); // Unknown heap object type
                            }
                        }
                        // Fallback to legacy arrays registry
                        else if let Some(array_ref) = self.arrays.get(&obj_id) {
                            let array = array_ref.read().unwrap();

                            // Use normalized index for negative index support
                            if let Some(normalized_idx) = normalize_index(index_i32, array.len()) {
                                // Get element value
                                let elem = &array[normalized_idx];

                                // Push element value onto stack based on type
                                match elem {
                                    auto_val::Value::Int(i) => task.ram.push_i32(*i),
                                    auto_val::Value::Uint(u) => task.ram.push_i32(*u as i32),
                                    auto_val::Value::Float(f) => task.ram.push_f32(*f as f32),
                                    auto_val::Value::Double(d) => task.ram.push_f64(*d),
                                    auto_val::Value::Bool(b) => {
                                        task.ram.push_i32(if *b { 1 } else { 0 })
                                    }
                                    auto_val::Value::Char(c) => task.ram.push_i32(*c as i32),
                                    auto_val::Value::Nil => task.ram.push_i32(0),
                                    // Plan 197 Bug E: heap object references stored in arrays
                                    auto_val::Value::VmRef(r) => task.ram.push_i32(r.id as i32),
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
                    } // end of else block for non-string case
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
                            // Plan 118: Return error for out-of-bounds assignment
                            return Err(VMError::RuntimeError(format!(
                                "Array index {} out of bounds (array length: {})",
                                index, array.len()
                            )));
                        }
                    } else {
                        // Plan 118: Return error for invalid array IDs
                        return Err(VMError::RuntimeError(format!(
                            "Invalid array ID: {}",
                            array_id
                        )));
                    }
                }
                // Plan 075: Object field assignment (obj.field = value)
                OpCode::SET_FIELD => {
                    // Stack: value, object_id, field_name_idx (compiled in this order by codegen)
                    // Pop field_name_idx first (top of stack)
                    let tagged = task.ram.pop_i32();
                    // Decode negative-tagged string index
                    let field_idx = if tagged < 0 {
                        (-tagged - 1) as usize
                    } else {
                        tagged as usize
                    };
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
                            "Invalid string index: {}",
                            field_idx
                        )));
                    };
                    drop(strings); // Release lock before writing

                    // Get object from registry
                    if let Some(obj_ref) = self.objects.get(&obj_id) {
                        let mut obj = obj_ref.write().unwrap();
                        // Try multiple key formats: string, integer, boolean (same as GET_FIELD)
                        let key = if obj.get(&auto_val::ValueKey::Str(field_name.clone().into())).is_some() {
                            auto_val::ValueKey::Str(field_name.into())
                        } else if let Ok(int_key) = field_name.parse::<i32>() {
                            if obj.get(&auto_val::ValueKey::Int(int_key)).is_some() {
                                auto_val::ValueKey::Int(int_key)
                            } else {
                                // Plan 118: Integer field not found - return error
                                return Err(VMError::RuntimeError(format!(
                                    "Field '{}' not found on object",
                                    field_name
                                )));
                            }
                        } else if field_name == "true" {
                            if obj.get(&auto_val::ValueKey::Bool(true)).is_some() {
                                auto_val::ValueKey::Bool(true)
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Field '{}' not found on object",
                                    field_name
                                )));
                            }
                        } else if field_name == "false" {
                            if obj.get(&auto_val::ValueKey::Bool(false)).is_some() {
                                auto_val::ValueKey::Bool(false)
                            } else {
                                return Err(VMError::RuntimeError(format!(
                                    "Field '{}' not found on object",
                                    field_name
                                )));
                            }
                        } else {
                            // Plan 118: Field not found - return error instead of creating new field
                            return Err(VMError::RuntimeError(format!(
                                "Field '{}' not found on object",
                                field_name
                            )));
                        };
                        obj.set(key, auto_val::Value::Int(value));
                    } else {
                        // Plan 118: Return error for invalid object IDs
                        return Err(VMError::RuntimeError(format!(
                            "Invalid object ID: {}",
                            obj_id
                        )));
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
                            "Invalid string index: {}",
                            field_idx
                        )));
                    };
                    drop(strings); // Release lock before potentially writing below

                    // Get object from registry
                    if let Some(obj_ref) = self.objects.get(&obj_id) {
                        let obj = obj_ref.read().unwrap();

                        // Try multiple key formats: string, integer, boolean
                        // This handles cases like { 1: 2, 3: 4 } accessed as a.3
                        let value = if let Some(v) = obj.get(&auto_val::ValueKey::Str(field_name.clone().into())) {
                            Some(v.clone())
                        } else if let Ok(int_key) = field_name.parse::<i32>() {
                            obj.get(&auto_val::ValueKey::Int(int_key)).cloned()
                        } else if field_name == "true" {
                            obj.get(&auto_val::ValueKey::Bool(true)).cloned()
                        } else if field_name == "false" {
                            obj.get(&auto_val::ValueKey::Bool(false)).cloned()
                        } else {
                            None
                        };

                        if let Some(value) = value {
                            // Push field value onto stack based on type
                            match value {
                                auto_val::Value::Int(i) => task.ram.push_i32(i),
                                auto_val::Value::Uint(u) => task.ram.push_i32(u as i32),
                                auto_val::Value::Float(f) => task.ram.push_f32(f as f32),
                                auto_val::Value::Double(d) => task.ram.push_f64(d),
                                auto_val::Value::Bool(b) => {
                                    task.ram.push_i32(if b { 1 } else { 0 })
                                }
                                auto_val::Value::Char(c) => task.ram.push_i32(c as i32),
                                auto_val::Value::Str(s) => {
                                    // Push tagged string index (nanbox: direct tag, non-nanbox: negative i32)
                                    let str_bytes = s.as_bytes().to_vec();
                                    let mut strings = self.strings.write().unwrap();
                                    let str_idx = strings.len();
                                    strings.push(str_bytes);
                                    drop(strings);
                                    push_str_tag(&mut task.ram, str_idx as u32);
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
                            // Plan 118: Field not found - return error
                            return Err(VMError::RuntimeError(format!(
                                "Field '{}' not found on object",
                                field_name
                            )));
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
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
                }
                OpCode::SUB_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(a - b);
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
                }
                OpCode::MUL_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(a * b);
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
                }
                OpCode::DIV_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    if b == 0.0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_f32(a / b);
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
                }
                OpCode::NEG_F => {
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(-a);
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
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
                OpCode::MOD => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_i32(a % b);
                }
                OpCode::MOD_F => {
                    let b = task.ram.pop_f32();
                    let a = task.ram.pop_f32();
                    task.ram.push_f32(a % b);
                }
                OpCode::MOD_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_f64(a % b);
                }
                OpCode::NEG_D => {
                    let a = task.ram.pop_f64();
                    task.ram.push_f64(-a);
                }

                // 64-bit integer arithmetic (u64 stored as two i32 slots)
                OpCode::ADD_U64 => {
                    let b = task.ram.pop_u64();
                    let a = task.ram.pop_u64();
                    task.ram.push_u64(a.wrapping_add(b));
                }
                OpCode::SUB_U64 => {
                    let b = task.ram.pop_u64();
                    let a = task.ram.pop_u64();
                    task.ram.push_u64(a.wrapping_sub(b));
                }
                OpCode::MUL_U64 => {
                    let b = task.ram.pop_u64();
                    let a = task.ram.pop_u64();
                    task.ram.push_u64(a.wrapping_mul(b));
                }
                OpCode::DIV_U64 => {
                    let b = task.ram.pop_u64();
                    let a = task.ram.pop_u64();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_u64(a / b);
                }
                OpCode::MOD_U64 => {
                    let b = task.ram.pop_u64();
                    let a = task.ram.pop_u64();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    task.ram.push_u64(a % b);
                }

                // Plan 117: Type coercion for mixed arithmetic
                OpCode::I32_TO_F32 => {
                    let val = task.ram.pop_i32();
                    task.ram.push_f32(val as f32);
                    task.last_result_type = ResultType::Float; // Plan 117/118: Mark result as float
                }
                OpCode::I64_TO_F64 => {
                    let val = task.ram.pop_i64();
                    task.ram.push_f64(val as f64);
                    task.last_result_type = ResultType::Float;
                }
                OpCode::U64_TO_F64 => {
                    let val = task.ram.pop_u64();
                    task.ram.push_f64(val as f64);
                    task.last_result_type = ResultType::Float;
                }

                OpCode::NOT => {
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(!a);
                }
                OpCode::CALL => {
                    vm_debug!("DEBUG CALL: Stack depth before = {}", task.ram.sp);
                    // Print stack before CALL
                    if task.ram.sp > 0 {
                        vm_debug!("DEBUG CALL: Stack[0] = {}", task.ram.read_i32(0));
                    }

                    let target = self.flash.read_u32(task.ip) as usize;
                    vm_debug!("DEBUG CALL: Calling function at address 0x{:04x}", target);
                    task.ip += 4;

                    // Push Return Address (IP)
                    task.ram.push_i32(task.ip as i32);
                    // Push Old Stack Frame (BP)
                    task.ram.push_i32(task.bp as i32);

                    // New BP points to the saved BP location (SP - 1)
                    task.bp = task.ram.sp - 1;

                    // Plan 199: Push structured call frame for debugging
                    task.call_stack.push(crate::vm::task::CallFrame {
                        return_ip: task.ip,
                        old_bp: task.bp,
                        fn_name: None,
                        line: task.current_line,
                    });

                    vm_debug!("DEBUG CALL: Stack depth after setup = {}, BP = {}",
                        task.ram.sp, task.bp
                    );
                    vm_debug!("DEBUG CALL: Stack[0] = {}, [1] = {}, [2] = {}",
                        task.ram.read_i32(0),
                        task.ram.read_i32(1),
                        task.ram.read_i32(2)
                    );

                    // Jump
                    task.ip = target;
                }
                OpCode::CALL_SPEC => {
                    // Dynamic dispatch via spec vtable
                    // Reads: method_name string index (u16), arg_count (u8)
                    // Stack: [..., receiver, arg0, arg1, ..., argN-1]
                    let method_name_idx = self.flash.read_u16(task.ip) as usize;
                    task.ip += 2;
                    let arg_count = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    // Get method name from string pool
                    let method_name = self.strings.read().unwrap()
                        .get(method_name_idx)
                        .map(|b| String::from_utf8_lossy(b).to_string())
                        .unwrap_or_default();

                    // The receiver is at stack position sp - arg_count - 1
                    // (args are on top, receiver is below them)
                    let receiver_pos = task.ram.sp - arg_count - 1;
                    let receiver = task.ram.read_i32(receiver_pos);

                    // Look up the object's mono_name from heap
                    let type_name = if receiver > 0 {
                        let obj_key = receiver as u64;
                        if let Some(obj_lock) = self.heap_objects.get(&obj_key) {
                            let guard = obj_lock.read().unwrap();
                            if let Some(inst) = guard.as_any().downcast_ref::<crate::vm::generic_registry::GenericInstanceData>() {
                                inst.mono_name.split('_').next()
                                    .unwrap_or(&inst.mono_name).to_string()
                            } else {
                                format!("<unknown:{}>", obj_key)
                            }
                        } else {
                            format!("<unknown:{}>", obj_key)
                        }
                    } else {
                        format!("<invalid:{}>", receiver)
                    };

                    // Construct function name: TypeName.method
                    let func_name = format!("{}.{}", type_name, method_name);

                    // Look up function address in exports first
                    if let Some(&addr) = self.flash.exports_by_name.get(&func_name) {
                        // Standard CALL sequence: push return address, old BP, set new BP, jump
                        task.ram.push_i32(task.ip as i32);
                        task.ram.push_i32(task.bp as i32);
                        task.bp = task.ram.sp - 1;
                        task.ip = addr as usize;
                    } else if let Some(native_id) = self.native_interface.resolve(&func_name) {
                        // Plan 200 Task 3.3: Fallback to native registry for type.method natives
                        // (e.g., Result.Ok.map_err -> shim_result_map_err)
                        if let Some(shim) = self.native_interface.get(native_id).cloned() {
                            shim(task, self)?;
                        } else {
                            return Err(VMError::MissingNative(native_id));
                        }
                    } else {
                        return Err(VMError::RuntimeError(
                            format!("CALL_SPEC: no function '{}' for type '{}'", func_name, type_name)
                        ));
                    }
                }
                OpCode::CALL_NAT => {
                    let native_id = self.flash.read_u16(task.ip);
                    task.ip += 2;

                    // Execute Native Shim
                    if let Some(shim) = self.native_interface.get(native_id).cloned() {
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
                        return Ok(StepResult::Terminated);
                    }

                    // Expect Result on Top of Stack
                    let result = task.ram.pop_i32();

                    let old_bp = task.ram.read_i32(task.bp) as usize;
                    let ret_ip = task.ram.read_i32(task.bp - 1) as usize;

                    // Plan 071 Phase 5: Restore previous closure from saved_closure_id
                    task.current_closure_id = task.saved_closure_id;

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

                    // Plan 199: Pop structured call frame
                    task.call_stack.pop();
                }
                // RET_D: Return with 2-slot value (f64, u64, i64)
                OpCode::RET_D => {
                    let n_args = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    if task.bp == 0 {
                        return Ok(StepResult::Terminated);
                    }

                    // Pop 2 slots: high (top) then low
                    let result_high = task.ram.pop_i32();
                    let result_low = task.ram.pop_i32();

                    let old_bp = task.ram.read_i32(task.bp) as usize;
                    let ret_ip = task.ram.read_i32(task.bp - 1) as usize;

                    task.current_closure_id = task.saved_closure_id;

                    let new_sp = task.bp - n_args;

                    // Restore frame, then write 2-slot result.
                    // Must write at new_sp-1/new_sp (replacing ret_addr/old_bp slots)
                    // rather than pushing after new_sp, to avoid leaving ret_addr on stack.
                    // This mirrors how 1-slot RET writes at new_sp-1.
                    task.ram.write_i32(new_sp - 1, result_low);
                    task.ram.write_i32(new_sp, result_high);

                    task.bp = old_bp;
                    task.ip = ret_ip;
                    task.ram.sp = new_sp + 1;
                }

                // === Closures (Plan 071: Direct Capture) ===
                OpCode::CLOSURE => {
                    // Stack: capture_count × value -> closure_id
                    // Immediate: func_addr (u32), capture_count (u8), n_args (u8)
                    let func_addr = self.flash.read_u32(task.ip);
                    task.ip += 4;
                    let capture_count = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;
                    let n_args = self.flash.read_u8(task.ip) as usize;
                    task.ip += 1;

                    vm_debug!("DEBUG CLOSURE: func_addr={}, capture_count={}, n_args={}, ip after header={}, sp before={}", func_addr, capture_count, n_args, task.ip, task.ram.sp);

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
                    let closure = Closure { func_addr, env, n_args };

                    vm_debug!("DEBUG CLOSURE: created closure_id={}, ip after names={}, sp after={}", closure_id, task.ip, task.ram.sp);

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
                            "Invalid string index: {}",
                            var_name_idx
                        )));
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
                        VMError::RuntimeError(
                            "LOAD_CAPTURED called outside of closure context".to_string(),
                        )
                    })?;

                    // Plan 073: Now uses RwLock for strings access
                    let strings = self.strings.read().unwrap();
                    let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                        String::from_utf8_lossy(var_name_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}",
                            var_name_idx
                        )));
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
                        VMError::RuntimeError(
                            "STORE_CAPTURED called outside of closure context".to_string(),
                        )
                    })?;

                    // Plan 073: Now uses RwLock for strings access
                    let strings = self.strings.read().unwrap();
                    let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
                        String::from_utf8_lossy(var_name_bytes).to_string()
                    } else {
                        return Err(VMError::RuntimeError(format!(
                            "Invalid string index: {}",
                            var_name_idx
                        )));
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

                        // Set current_fn_n_args for LOAD_LOCAL parameter access
                        task.current_fn_n_args = _closure.n_args;

                        // Store old closure ID in task (not on stack) to avoid breaking parameter layout
                        // The RET opcode will restore it from task.saved_closure_id
                        task.saved_closure_id = old_closure_id;

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
                    return Ok(StepResult::Yield);
                }
                OpCode::SLEEP => {
                    let ms = self.flash.read_u32(task.ip) as u64;
                    task.ip += 4;

                    // Set wake time
                    task.wake_time = Some(Instant::now() + std::time::Duration::from_millis(ms));
                    task.status = TaskStatus::Waiting(format!("sleep for {}ms", ms));
                    return Ok(StepResult::Yield);
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
                            return Ok(StepResult::Yield);
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
                        return Ok(StepResult::Yield);
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
                        return Ok(StepResult::Yield);
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

                // Plan 126: SPAWN_GO - fire-and-forget spawn
                // Pop function address and arg_count from stack, spawn in background
                // Returns void (no value pushed to stack)
                // Stack layout: [..., func_addr:i32, arg_count:i32] (high to low)
                OpCode::SPAWN_GO => {
                    // Pop the function address (or closure reference)
                    let target = task.ram.pop_i32() as usize;
                    // Pop arg count
                    let arg_count = task.ram.pop_i32() as usize;

                    // Collect args from stack
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(task.ram.pop_i32());
                    }

                    // Spawn a new task for this function
                    let new_task_id = self.spawn_task(target, 1024);

                    // Initialize the new task's stack with args
                    if let Some(new_task_arc) = self.tasks.get(&new_task_id) {
                        if let Ok(mut new_task) = new_task_arc.try_lock() {
                            // Push args in reverse order (A, B, C)
                            for arg in args.into_iter().rev() {
                                new_task.ram.push_i32(arg);
                            }
                        }
                        // If we can't lock, the task will just sit idle
                        // This is fire-and-forget, so we don't propagate errors
                    }

                    // Fire-and-forget: no value pushed to stack (returns void)
                    // Unlike SPAWN, we don't push task_id back
                }

                // Plan 127: TASK_LOOP - enter task message processing loop
                // This opcode marks the start of a task's message handling loop.
                // The task will wait for messages and dispatch them to handlers.
                // Stack layout: [task_type_str_idx:i32]
                OpCode::TASK_LOOP => {
                    // Get task type string index
                    let task_type_idx = task.ram.pop_i32() as u16;
                    let strings = self.strings.read().unwrap();
                    let task_type = strings.get(task_type_idx as usize)
                        .map(|b| String::from_utf8_lossy(b).to_string())
                        .unwrap_or_default();
                    drop(strings);

                    // Store that this task is now in message loop mode
                    task.in_message_loop = true;
                    task.task_type_name = Some(task_type.clone());

                    // Set task to waiting state (will be woken when messages arrive)
                    task.status = TaskStatus::Waiting("message_loop".to_string());

                    if crate::is_vm_debug() {
                        eprintln!("[TASK_LOOP] Task {} entering message loop for type {}",
                            task.id, task_type);
                    }
                }

                // Plan 127: HANDLE_MSG - dispatch message to matched handler
                // The message value is on the stack, handlers are looked up from metadata.
                // Stack layout: [..., msg_value:i32]
                // Pushes: handler_found:bool, handler_offset:i32 (if found)
                OpCode::HANDLE_MSG => {
                    // Get message value from stack
                    let msg_value = task.ram.pop_i32();

                    // Get task type for handler lookup
                    let task_type = task.task_type_name.clone().unwrap_or_default();

                    // Try to find a matching handler using PatternMatcher
                    if let Some(table) = self.task_handler_registry.get_table(&task_type) {
                        // Convert message i32 to Value for pattern matching
                        let msg = auto_val::Value::Int(msg_value);

                        // Try each pattern in order
                        let mut found = false;
                        for handler in table.get_handlers() {
                            if let Some(pattern) = table.get_pattern(handler.pattern_idx) {
                                // Use PatternMatcher for matching
                                if self.match_message_pattern_vm(&msg, pattern, &table.string_pool) {
                                    // Found matching handler
                                    task.ram.push_i32(1); // true - handler found
                                    task.ram.push_i32(handler.body_offset as i32);

                                    // Store if handler has context
                                    task.current_handler_has_context = handler.has_context;

                                    found = true;
                                    break;
                                }
                            }
                        }

                        if !found {
                            // No matching handler
                            task.ram.push_i32(0); // false - no handler
                            if crate::is_vm_debug() {
                                eprintln!("[HANDLE_MSG] No handler found for message {} in task {}", msg_value, task_type);
                            }
                        }
                    } else {
                        // No handlers registered for this task type
                        task.ram.push_i32(0); // false
                        if crate::is_vm_debug() {
                            eprintln!("[HANDLE_MSG] No handler table for task type {}", task_type);
                        }
                    }
                }

                // Plan 127: REPLY - send reply via current MessageContext
                // Stack layout: [..., reply_value:i32]
                // Pops the reply value and sends it through the reply channel.
                OpCode::REPLY => {
                    // Get reply value from stack
                    let reply_value = task.ram.pop_i32();

                    // Check if we have a current message context with reply capability
                    if let Some(ref ctx) = task.current_msg_context {
                        // Convert i32 to Value for reply
                        let value = auto_val::Value::Int(reply_value);
                        match ctx.reply(value) {
                            Ok(()) => {
                                if crate::is_vm_debug() {
                                    eprintln!("[REPLY] Sent reply value {}", reply_value);
                                }
                            }
                            Err(_e) => {
                                if crate::is_vm_debug() {
                                    eprintln!("[REPLY] Failed to send reply: {}", _e);
                                }
                            }
                        }
                    } else {
                        if crate::is_vm_debug() {
                            eprintln!("[REPLY] No message context available for reply");
                        }
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
                        let param_idx = idx - 0x80; // 0x80 -> param 0, 0x81 -> param 1, etc.
                                                    // Stack layout: [..., args(0), args(1), ..., return_addr, old_bp, locals...]
                                                    //                        ^- BP-n_args           ^- BP-1    ^- BP

                        // Plan 088 Phase 4: Read n_args from function metadata (set by FN_PROLOG)
                        let n_args = task.current_fn_n_args;
                        let offset = n_args - param_idx; // For n_args=1, param 0: offset=1

                        // Stack layout for n_args=1: [arg0, ret_addr, old_bp, ...]
                        //                                    ^-BP-2 ^-BP-1  ^-BP
                        // For n_args=2:             [arg0, arg1, ret_addr, old_bp, ...]
                        //                                    ^-BP-3 ^-BP-2 ^-BP-1  ^-BP
                        let actual_offset = offset + 1; // +1 for return_addr
                        let val = task.ram.read_i32(task.bp - actual_offset);
                        vm_debug!("DEBUG: LOAD_LOCAL param {}: BP-{} (n_args={}, offset={}) = {}",
                            param_idx, actual_offset, n_args, offset, val
                        );
                        task.ram.push_i32(val);
                    } else {
                        // Local variable: load from bp+1+idx (bp+1 is first local variable)
                        let val = task.ram.read_i32(task.bp + 1 + idx);
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
                        let param_idx = idx - 0x80; // 0x80 -> param 0, 0x81 -> param 1, etc.
                        let n_args = task.current_fn_n_args;
                        let offset = n_args - param_idx;
                        let actual_offset = offset + 1; // +1 for return_addr

                        // Store to parameter location
                        task.ram.write_i32(task.bp - actual_offset, val);
                        vm_debug!("DEBUG: STORE_LOCAL param {}: BP-{} = {}",
                            param_idx, actual_offset, val
                        );
                    } else {
                        // Local variable: store to bp+1+idx (bp+1 is first local variable)
                        task.ram.write_i32(task.bp + 1 + idx, val);
                    }
                }
                OpCode::LOAD_LOC_0 => {
                    let addr = task.bp + 1;
                    let val = task.ram.read_i32(addr);
                    task.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_1 => {
                    let val = task.ram.read_i32(task.bp + 2);
                    task.ram.push_i32(val);
                }
                OpCode::LOAD_LOC_2 => {
                    let val = task.ram.read_i32(task.bp + 3);
                    task.ram.push_i32(val);
                }
                OpCode::STORE_LOC_0 => {
                    let val = task.ram.pop_i32();
                    task.ram.write_i32(task.bp + 1, val);
                }
                OpCode::STORE_LOC_1 => {
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

                    vm_debug!("DEBUG FN_PROLOG: n_args={}, n_locals={}", n_args, n_locals);

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

                    vm_debug!("DEBUG RESERVE_STACK: n_locals={}, sp before={}", n_locals, task.ram.sp);

                    // Push n_locals+1 zeros to reserve space for local variables + 1 extra slot
                    // The extra slot ensures SP starts beyond all local variable addresses
                    for _ in 0..n_locals + 1 {
                        task.ram.push_i32(0);
                    }

                    vm_debug!("DEBUG RESERVE_STACK: sp after={}", task.ram.sp);

                    // Track num_locals for native shims
                    task.num_locals = n_locals;
                }

                // === Comparison ===
                OpCode::EQ => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    // Plan 091: Use special values for boolean results
                    // i32::MIN = true, i32::MIN+1 = false
                    // Plan 197 Task 2: Content-aware string comparison
                    // Plan 197 Task 7: Structural equality for heap objects
                    let result = if a == b {
                        true
                    } else if a >= 4000000 && b >= 4000000 {
                        // Heap objects — structural equality
                        self.struct_eq(a, b)
                    } else if a < 0 && b < 0 && a > i32::MIN && b > i32::MIN {
                        // Both are tagged string indices — compare actual string contents
                        let a_idx = ((-a) - 1) as u16;
                        let b_idx = ((-b) - 1) as u16;
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => sa == sb,
                            _ => false,
                        }
                    } else {
                        false
                    };
                    task.ram.push_i32(if result { -2147483648 } else { -2147483647 });
                }
                OpCode::NE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    // Plan 197 Task 2: Content-aware string comparison
                    // Plan 197 Task 7: Structural equality for heap objects
                    let result = if a == b {
                        false
                    } else if a >= 4000000 && b >= 4000000 {
                        // Heap objects — structural inequality
                        !self.struct_eq(a, b)
                    } else if a < 0 && b < 0 && a > i32::MIN && b > i32::MIN {
                        // Both are tagged string indices — compare actual string contents
                        let a_idx = ((-a) - 1) as u16;
                        let b_idx = ((-b) - 1) as u16;
                        let a_str = self.get_string(a_idx);
                        let b_str = self.get_string(b_idx);
                        match (a_str, b_str) {
                            (Some(sa), Some(sb)) => sa != sb,
                            _ => true,
                        }
                    } else {
                        true
                    };
                    task.ram.push_i32(if result { -2147483648 } else { -2147483647 });
                }
                OpCode::LT => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram
                        .push_i32(if a < b { -2147483648 } else { -2147483647 });
                }
                OpCode::GT => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram
                        .push_i32(if a > b { -2147483648 } else { -2147483647 });
                }
                OpCode::LE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram
                        .push_i32(if a <= b { -2147483648 } else { -2147483647 });
                }
                OpCode::GE => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram
                        .push_i32(if a >= b { -2147483648 } else { -2147483647 });
                }

                // f64 comparison opcodes (each pops 2+2 slots, pushes 1 bool)
                OpCode::EQ_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_i32(if a == b { -2147483648 } else { -2147483647 });
                }
                OpCode::NE_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_i32(if a != b { -2147483648 } else { -2147483647 });
                }
                OpCode::LT_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_i32(if a < b { -2147483648 } else { -2147483647 });
                }
                OpCode::GT_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_i32(if a > b { -2147483648 } else { -2147483647 });
                }
                OpCode::LE_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_i32(if a <= b { -2147483648 } else { -2147483647 });
                }
                OpCode::GE_D => {
                    let b = task.ram.pop_f64();
                    let a = task.ram.pop_f64();
                    task.ram.push_i32(if a >= b { -2147483648 } else { -2147483647 });
                }

                // === Logical ===
                OpCode::AND => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a & b);
                }
                OpCode::OR => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a | b);
                }
                OpCode::XOR => {
                    let b = task.ram.pop_i32();
                    let a = task.ram.pop_i32();
                    task.ram.push_i32(a ^ b);
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
                OpCode::SOURCE_LINE => {
                    // Plan 199: Record current source line for debugging
                    let line = self.flash.read_u16(task.ip);
                    task.ip += 2;
                    task.current_line = line as u32;
                }
                OpCode::HALT => {
                    return Ok(StepResult::Terminated);
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

                    vm_debug!("DEBUG: LOAD_REF: var_index={}, bp={}", var_index, task.bp);
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

                    vm_debug!("DEBUG: STORE_REF: var_index={}, val={}, bp={}",
                        var_index, val, task.bp
                    );
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

                    vm_debug!("DEBUG: STORE_MUT_REF: var_index={}, val={}, bp={}",
                        var_index, val, task.bp
                    );
                }

                // === Plan 124: Async/Future/Await Instructions ===
                OpCode::CREATE_FUTURE => {
                    // Create a Future value from async block body
                    // Format: body_code_offset: u32
                    let body_offset = self.flash.read_u32(task.ip);
                    task.ip += 4;

                    // Allocate a new future ID from VM's registry
                    let future_id = self.future_id_gen.fetch_add(1, Ordering::SeqCst);

                    // Create the future value with pending state
                    let future = FutureValue {
                        body_offset,
                        state: FutureState::Pending,
                        result: None,
                        owner_task_id: task.id,
                    };

                    // Store in VM's future registry
                    self.futures.insert(future_id, Arc::new(RwLock::new(future)));

                    // For Phase 2.1, we encode Future on stack as: (future_id << 8) | 0xF0
                    // The 0xF0 marker distinguishes futures from other values
                    let future_bits = ((future_id as i32) << 8) | 0xF0;
                    task.ram.push_i32(future_bits);

                    vm_debug!("DEBUG: CREATE_FUTURE: id={}, body_offset={}", future_id, body_offset);
                }
                OpCode::AWAIT_FUTURE => {
                    // Wait for future completion (blocking)
                    // Stack: [..., future_bits]
                    // Returns: value when ready
                    let future_bits = task.ram.pop_i32();

                    // Check if this is a valid future encoding
                    if (future_bits & 0xFF) == 0xF0 {
                        let future_id = (future_bits >> 8) as u32;

                        // Look up the future in the registry
                        if let Some(future_arc) = self.futures.get(&future_id) {
                            let mut future = future_arc.write().unwrap();

                            match future.state {
                                FutureState::Ready => {
                                    // Future is ready - return the result
                                    vm_debug!("DEBUG: AWAIT_FUTURE: id={} is ready", future_id);
                                    if let Some(ref result) = future.result {
                                        // Push the result value
                                        // For Phase 2.1, we only support i32 results
                                        match result {
                                            auto_val::Value::Int(n) => task.ram.push_i32(*n as i32),
                                            auto_val::Value::Nil => task.ram.push_i32(0),
                                            _ => task.ram.push_i32(0), // Default to nil for unsupported types
                                        }
                                    } else {
                                        task.ram.push_i32(0); // No result = nil
                                    }
                                }
                                FutureState::Failed => {
                                    // Future failed - return nil
                                    vm_debug!("DEBUG: AWAIT_FUTURE: id={} failed", future_id);
                                    task.ram.push_i32(0);
                                }
                                FutureState::Pending => {
                                    // Phase 2.1: Execute the async body synchronously
                                    // In full implementation, this would suspend the task
                                    // and schedule execution on a worker thread
                                    vm_debug!("DEBUG: AWAIT_FUTURE: id={} is pending, executing synchronously", future_id);

                                    // Save current IP
                                    let saved_ip = task.ip;
                                    let body_offset = future.body_offset as usize;

                                    // Jump to async body and execute
                                    task.ip = body_offset;

                                    // Execute until we hit a marker or run out of instructions
                                    // For Phase 2.1, the async body will execute and leave a result on stack
                                    // We'll execute a limited number of instructions

                                    // Execute the body (simplified - just set result as complete)
                                    // In real implementation, we'd run the bytecode interpreter here

                                    // Restore IP
                                    task.ip = saved_ip;

                                    // Mark future as ready with a placeholder result
                                    future.state = FutureState::Ready;
                                    future.result = Some(auto_val::Value::Int(0));

                                    // Push the result
                                    task.ram.push_i32(0);
                                }
                            }
                        } else {
                            // Future not found - return nil
                            vm_debug!("DEBUG: AWAIT_FUTURE: id={} not found in registry", future_id);
                            task.ram.push_i32(0);
                        }
                    } else {
                        // Not a future - push back as-is (identity)
                        task.ram.push_i32(future_bits);
                    }
                }
                OpCode::POLL_FUTURE => {
                    // Non-blocking poll for future state
                    // Stack: [..., future_bits]
                    // Returns: (is_ready: bool, value_or_nil)
                    let future_bits = task.ram.pop_i32();

                    if (future_bits & 0xFF) == 0xF0 {
                        let future_id = (future_bits >> 8) as u32;

                        // Look up the future in the registry
                        if let Some(future_arc) = self.futures.get(&future_id) {
                            let future = future_arc.read().unwrap();

                            match future.state {
                                FutureState::Ready => {
                                    // Push is_ready = 1
                                    task.ram.push_i32(1);

                                    // Push the result value
                                    if let Some(ref result) = future.result {
                                        match result {
                                            auto_val::Value::Int(n) => task.ram.push_i32(*n as i32),
                                            auto_val::Value::Nil => task.ram.push_i32(0),
                                            _ => task.ram.push_i32(0),
                                        }
                                    } else {
                                        task.ram.push_i32(0);
                                    }
                                }
                                FutureState::Failed => {
                                    // Push is_ready = 1 (failed is also "complete")
                                    task.ram.push_i32(1);
                                    // Push nil for failed
                                    task.ram.push_i32(0);
                                }
                                FutureState::Pending => {
                                    // Push is_ready = 0
                                    task.ram.push_i32(0);
                                    // Push nil (no value yet)
                                    task.ram.push_i32(0);
                                }
                            }
                        } else {
                            // Future not found - return not ready
                            task.ram.push_i32(0);
                            task.ram.push_i32(0);
                        }
                    } else {
                        // Not a future - return not ready
                        task.ram.push_i32(0);
                        task.ram.push_i32(0);
                    }
                }

                _ => {
                    // Unimplemented opcodes for Phase 1
                    return Err(VMError::InvalidOpCode(op_byte));
                }
            }

            Ok(StepResult::Continue)
        }

    /// Execute a chunk of opcodes for a specific task
    fn execute_task(&self, task: &mut AutoTask) -> Result<TaskStatus, VMError> {
        let budget = 100; // OpCode Budget
        for _ in 0..budget {
            let ip_before = task.ip;
            let line_before = task.current_line;
            match self.run_one_instruction(task)? {
                StepResult::Continue => {
                    // Plan 199: Record trace if enabled
                    self.record_trace(ip_before, line_before, task);
                    continue;
                }
                StepResult::Terminated => return Ok(TaskStatus::Terminated),
                StepResult::Yield => {
                    // SLEEP sets task.status to Waiting; YIELD/JOIN/SEND/RECV leave it Ready
                    if matches!(task.status, TaskStatus::Waiting(_)) {
                        return Ok(task.status.clone());
                    }
                    return Ok(TaskStatus::Ready);
                }
            }
        }
        Ok(TaskStatus::Ready)
    }

    /// Plan 199: Record a trace entry if tracing is enabled
    fn record_trace(&self, ip: usize, line: u32, task: &AutoTask) {
        let mut trace = self.trace.lock().unwrap();
        if let Some(ref mut collector) = *trace {
            let op_name = if ip < self.flash.memory.len() {
                let byte = self.flash.read_u8(ip);
                if OpCode::is_valid(byte) {
                    OpCode::from(byte).to_mnemonic().to_string()
                } else {
                    format!("0x{:02x}", byte)
                }
            } else {
                "EOF".to_string()
            };
            collector.record(ip, &op_name, line, task.ram.sp, task.call_stack.len());
        }
    }
}
