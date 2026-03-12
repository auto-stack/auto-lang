use crate::vm::collections::{AutoVMHashMap, SpecializedHashMap, SpecializedHashSet, SpecializedStringBuilder, SpecializedVecDeque, SpecializedBTreeMap};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;
use auto_val::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

/// Decode a tagged string index from stack value.
/// LOAD_STR pushes string indices as negative tagged values: -(str_idx as i32) - 1
/// This function decodes the tag to get the actual string pool index.
#[inline]
fn decode_str_idx(bits: i32) -> usize {
    if bits < 0 {
        (-bits - 1) as usize
    } else {
        bits as usize
    }
}

/// Encode a string pool index as a tagged value (negative).
/// This function encodes the index so it can be identified as a string reference.
#[inline]
fn encode_str_idx(idx: i32) -> i32 {
    -(idx + 1)
}

// Plan 094: ID ranges for hybrid FFI
/// Maximum ID for static FFI bindings
pub const STATIC_ID_MAX: u16 = 10000;
/// Starting ID for dynamic FFI bindings
pub const DYNAMIC_ID_START: u16 = 10000;

pub type ShimFunc = Arc<dyn Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync>;

/// Plan 094: NativeInterface with hybrid lookup support
///
/// Supports two types of native functions:
/// - **Static** (IDs 0-9999): Built into VM, registered at compile time
/// - **Dynamic** (IDs 10000+): Loaded via `use.rust`, registered at runtime
pub struct NativeInterface {
    /// Static shims: direct array lookup for maximum performance
    static_shims: Vec<Option<ShimFunc>>,
    /// Dynamic shims: HashMap for flexibility
    dynamic_shims: HashMap<u16, ShimFunc>,
    /// Next available dynamic ID
    next_dynamic_id: u16,
}

impl NativeInterface {
    pub fn new() -> Self {
        Self {
            static_shims: vec![None; STATIC_ID_MAX as usize],
            dynamic_shims: HashMap::new(),
            next_dynamic_id: DYNAMIC_ID_START,
        }
    }

    /// Register a static shim (IDs 0-9999)
    ///
    /// Used by VM intrinsics and built-in stdlib functions.
    /// Panics if ID is out of range.
    pub fn register_static<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        assert!(id < STATIC_ID_MAX, "Static ID must be < {}", STATIC_ID_MAX);
        self.static_shims[id as usize] = Some(Arc::new(func));
    }

    /// Register a dynamic shim (IDs 10000+)
    ///
    /// Used by `use.rust` for runtime-loaded crates.
    /// Returns the assigned ID.
    pub fn register_dynamic<F>(&mut self, func: F) -> u16
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        let id = self.next_dynamic_id;
        self.next_dynamic_id += 1;
        self.dynamic_shims.insert(id, Arc::new(func));
        id
    }

    /// Register a dynamic shim with a specific ID
    ///
    /// Used when the caller wants to control the ID assignment.
    pub fn register_dynamic_with_id<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        assert!(id >= DYNAMIC_ID_START, "Dynamic ID must be >= {}", DYNAMIC_ID_START);
        self.dynamic_shims.insert(id, Arc::new(func));
        if id >= self.next_dynamic_id {
            self.next_dynamic_id = id + 1;
        }
    }

    /// Unified lookup - used by CALL_NAT opcode
    ///
    /// Checks static array first (fast path), then dynamic HashMap.
    pub fn get(&self, id: u16) -> Option<&ShimFunc> {
        if id < STATIC_ID_MAX {
            self.static_shims.get(id as usize)?.as_ref()
        } else {
            self.dynamic_shims.get(&id)
        }
    }

    /// Check if an ID is static or dynamic
    pub fn is_static(&self, id: u16) -> bool {
        id < STATIC_ID_MAX
    }

    /// Get the next available dynamic ID
    pub fn next_dynamic_id(&self) -> u16 {
        self.next_dynamic_id
    }

    /// Legacy method for backwards compatibility
    /// Routes to register_static for IDs < 10000, register_dynamic otherwise
    pub fn register<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        if id < STATIC_ID_MAX {
            self.register_static(id, func);
        } else {
            self.register_dynamic_with_id(id, func);
        }
    }

    pub fn register_std_shims(&mut self) {
        // Print functions
        self.register(NATIVE_PRINT_I32, shim_print_i32);
        self.register(NATIVE_PRINT_F32, shim_print_f32);
        self.register(NATIVE_PRINT_STR, shim_print_str);

        // List functions
        self.register(NATIVE_LIST_NEW, shim_list_new);
        self.register(NATIVE_LIST_PUSH, shim_list_push);
        self.register(NATIVE_LIST_POP, shim_list_pop);
        self.register(NATIVE_LIST_LEN, shim_list_len);
        self.register(NATIVE_LIST_IS_EMPTY, shim_list_is_empty);
        self.register(NATIVE_LIST_CLEAR, shim_list_clear);
        self.register(NATIVE_LIST_GET, shim_list_get);
        self.register(NATIVE_LIST_SET, shim_list_set);
        self.register(NATIVE_LIST_INSERT, shim_list_insert);
        self.register(NATIVE_LIST_REMOVE, shim_list_remove);
        self.register(NATIVE_LIST_DROP, shim_list_drop);

        // Iterator functions
        self.register(NATIVE_LIST_ITER, shim_list_iter);
        self.register(NATIVE_ITERATOR_NEXT, shim_iterator_next);
        self.register(NATIVE_ITERATOR_MAP, shim_iterator_map);
        self.register(NATIVE_ITERATOR_FILTER, shim_iterator_filter);
        self.register(NATIVE_ITERATOR_COLLECT, shim_iterator_collect);
        self.register(NATIVE_ITERATOR_REDUCE, shim_iterator_reduce);
        self.register(NATIVE_ITERATOR_FIND, shim_iterator_find);

        // HashMap functions
        self.register(NATIVE_HASHMAP_NEW, shim_hashmap_new);
        self.register(NATIVE_HASHMAP_INSERT_STR, shim_hashmap_insert_str);
        self.register(NATIVE_HASHMAP_INSERT_INT, shim_hashmap_insert_int);
        self.register(NATIVE_HASHMAP_GET_STR, shim_hashmap_get_str);
        self.register(NATIVE_HASHMAP_GET_INT, shim_hashmap_get_int);
        self.register(NATIVE_HASHMAP_CONTAINS, shim_hashmap_contains);
        self.register(NATIVE_HASHMAP_REMOVE, shim_hashmap_remove);
        self.register(NATIVE_HASHMAP_SIZE, shim_hashmap_size);
        self.register(NATIVE_HASHMAP_CLEAR, shim_hashmap_clear);
        self.register(NATIVE_HASHMAP_DROP, shim_hashmap_drop);

        // HashSet functions
        self.register(NATIVE_HASHSET_NEW, shim_hashset_new);
        self.register(NATIVE_HASHSET_INSERT, shim_hashset_insert);
        self.register(NATIVE_HASHSET_CONTAINS, shim_hashset_contains);
        self.register(NATIVE_HASHSET_REMOVE, shim_hashset_remove);
        self.register(NATIVE_HASHSET_SIZE, shim_hashset_size);
        self.register(NATIVE_HASHSET_CLEAR, shim_hashset_clear);
        self.register(NATIVE_HASHSET_DROP, shim_hashset_drop);

        // StringBuilder functions
        self.register(NATIVE_STRINGBUILDER_NEW, shim_stringbuilder_new);
        self.register(NATIVE_STRINGBUILDER_APPEND, shim_stringbuilder_append);
        self.register(NATIVE_STRINGBUILDER_APPEND_INT, shim_stringbuilder_append_int);
        self.register(NATIVE_STRINGBUILDER_APPEND_CHAR, shim_stringbuilder_append_char);
        self.register(NATIVE_STRINGBUILDER_LEN, shim_stringbuilder_len);
        self.register(NATIVE_STRINGBUILDER_CLEAR, shim_stringbuilder_clear);
        self.register(NATIVE_STRINGBUILDER_DROP, shim_stringbuilder_drop);
        self.register(NATIVE_STRINGBUILDER_BUILD, shim_stringbuilder_build);

        // VecDeque functions
        self.register(NATIVE_VECDEQUE_NEW, shim_vecdeque_new);
        self.register(NATIVE_VECDEQUE_PUSH_BACK, shim_vecdeque_push_back);
        self.register(NATIVE_VECDEQUE_PUSH_FRONT, shim_vecdeque_push_front);
        self.register(NATIVE_VECDEQUE_POP_BACK, shim_vecdeque_pop_back);
        self.register(NATIVE_VECDEQUE_POP_FRONT, shim_vecdeque_pop_front);
        self.register(NATIVE_VECDEQUE_FRONT, shim_vecdeque_front);
        self.register(NATIVE_VECDEQUE_BACK, shim_vecdeque_back);
        self.register(NATIVE_VECDEQUE_SIZE, shim_vecdeque_size);
        self.register(NATIVE_VECDEQUE_IS_EMPTY, shim_vecdeque_is_empty);
        self.register(NATIVE_VECDEQUE_CLEAR, shim_vecdeque_clear);
        self.register(NATIVE_VECDEQUE_DROP, shim_vecdeque_drop);

        // BTreeMap functions
        self.register(NATIVE_BTREEMAP_NEW, shim_btreemap_new);
        self.register(NATIVE_BTREEMAP_INSERT, shim_btreemap_insert);
        self.register(NATIVE_BTREEMAP_GET, shim_btreemap_get);
        self.register(NATIVE_BTREEMAP_CONTAINS, shim_btreemap_contains);
        self.register(NATIVE_BTREEMAP_REMOVE, shim_btreemap_remove);
        self.register(NATIVE_BTREEMAP_SIZE, shim_btreemap_size);
        self.register(NATIVE_BTREEMAP_IS_EMPTY, shim_btreemap_is_empty);
        self.register(NATIVE_BTREEMAP_CLEAR, shim_btreemap_clear);
        self.register(NATIVE_BTREEMAP_FIRST_KEY, shim_btreemap_first_key);
        self.register(NATIVE_BTREEMAP_LAST_KEY, shim_btreemap_last_key);
        self.register(NATIVE_BTREEMAP_DROP, shim_btreemap_drop);

        // String functions
        self.register(NATIVE_STR_LEN, shim_str_len);
        self.register(NATIVE_STRING_LEN, shim_string_len);
        self.register(NATIVE_STR_NEW, shim_str_new);
        self.register(NATIVE_STR_APPEND, shim_str_append);
        self.register(NATIVE_INT_STR, shim_int_str);
        self.register(NATIVE_STR_UPPER, shim_str_upper);
    }
}

pub const NATIVE_PRINT_I32: u16 = 1;
pub const NATIVE_PRINT_F32: u16 = 2;
pub const NATIVE_PRINT_STR: u16 = 3;

// === List Native Function IDs (100+) ===

pub const NATIVE_LIST_NEW: u16 = 100;
pub const NATIVE_LIST_PUSH: u16 = 101;
pub const NATIVE_LIST_POP: u16 = 102;
pub const NATIVE_LIST_LEN: u16 = 103;
pub const NATIVE_LIST_IS_EMPTY: u16 = 104;
pub const NATIVE_LIST_CLEAR: u16 = 105;
pub const NATIVE_LIST_GET: u16 = 106;
pub const NATIVE_LIST_SET: u16 = 107;
pub const NATIVE_LIST_INSERT: u16 = 108;
pub const NATIVE_LIST_REMOVE: u16 = 109;
pub const NATIVE_LIST_DROP: u16 = 110;

// === Iterator Native Functions (111+) ===
pub const NATIVE_LIST_ITER: u16 = 111;
pub const NATIVE_ITERATOR_NEXT: u16 = 112;
pub const NATIVE_ITERATOR_MAP: u16 = 113;
pub const NATIVE_ITERATOR_FILTER: u16 = 114;
pub const NATIVE_ITERATOR_COLLECT: u16 = 115;
pub const NATIVE_ITERATOR_REDUCE: u16 = 116;
pub const NATIVE_ITERATOR_FIND: u16 = 117;

// === HashMap Native Functions (119+) ===
pub const NATIVE_HASHMAP_NEW: u16 = 119;
pub const NATIVE_HASHMAP_INSERT_STR: u16 = 120;
pub const NATIVE_HASHMAP_INSERT_INT: u16 = 121;
pub const NATIVE_HASHMAP_GET_STR: u16 = 122;
pub const NATIVE_HASHMAP_GET_INT: u16 = 123;
pub const NATIVE_HASHMAP_CONTAINS: u16 = 124;
pub const NATIVE_HASHMAP_REMOVE: u16 = 125;
pub const NATIVE_HASHMAP_SIZE: u16 = 126;
pub const NATIVE_HASHMAP_CLEAR: u16 = 127;
pub const NATIVE_HASHMAP_DROP: u16 = 128;

// === HashSet Native Function IDs (129+) ===
pub const NATIVE_HASHSET_NEW: u16 = 129;
pub const NATIVE_HASHSET_INSERT: u16 = 130;
pub const NATIVE_HASHSET_CONTAINS: u16 = 131;
pub const NATIVE_HASHSET_REMOVE: u16 = 132;
pub const NATIVE_HASHSET_SIZE: u16 = 133;
pub const NATIVE_HASHSET_CLEAR: u16 = 134;
pub const NATIVE_HASHSET_DROP: u16 = 135;

// === VecDeque Native Function IDs (136+) ===
pub const NATIVE_VECDEQUE_NEW: u16 = 136;
pub const NATIVE_VECDEQUE_PUSH_BACK: u16 = 137;
pub const NATIVE_VECDEQUE_PUSH_FRONT: u16 = 138;
pub const NATIVE_VECDEQUE_POP_BACK: u16 = 139;
pub const NATIVE_VECDEQUE_POP_FRONT: u16 = 140;
pub const NATIVE_VECDEQUE_FRONT: u16 = 141;
pub const NATIVE_VECDEQUE_BACK: u16 = 142;
pub const NATIVE_VECDEQUE_SIZE: u16 = 143;
pub const NATIVE_VECDEQUE_IS_EMPTY: u16 = 144;
pub const NATIVE_VECDEQUE_CLEAR: u16 = 145;
pub const NATIVE_VECDEQUE_DROP: u16 = 146;

// === BTreeMap Native Function IDs (147+) ===
pub const NATIVE_BTREEMAP_NEW: u16 = 147;
pub const NATIVE_BTREEMAP_INSERT: u16 = 148;
pub const NATIVE_BTREEMAP_GET: u16 = 149;
pub const NATIVE_BTREEMAP_CONTAINS: u16 = 150;
pub const NATIVE_BTREEMAP_REMOVE: u16 = 151;
pub const NATIVE_BTREEMAP_SIZE: u16 = 152;
pub const NATIVE_BTREEMAP_IS_EMPTY: u16 = 153;
pub const NATIVE_BTREEMAP_CLEAR: u16 = 154;
pub const NATIVE_BTREEMAP_FIRST_KEY: u16 = 155;
pub const NATIVE_BTREEMAP_LAST_KEY: u16 = 156;
pub const NATIVE_BTREEMAP_DROP: u16 = 157;

// === StringBuilder Native Function IDs (160+) ===
pub const NATIVE_STRINGBUILDER_NEW: u16 = 160;
pub const NATIVE_STRINGBUILDER_APPEND: u16 = 161;
pub const NATIVE_STRINGBUILDER_APPEND_INT: u16 = 162;
pub const NATIVE_STRINGBUILDER_APPEND_CHAR: u16 = 163;
pub const NATIVE_STRINGBUILDER_LEN: u16 = 164;
pub const NATIVE_STRINGBUILDER_CLEAR: u16 = 165;
pub const NATIVE_STRINGBUILDER_DROP: u16 = 166;
pub const NATIVE_STRINGBUILDER_BUILD: u16 = 167;

// === String Native Function IDs (170+) ===
pub const NATIVE_STR_LEN: u16 = 170;
pub const NATIVE_STRING_LEN: u16 = 171;
pub const NATIVE_STR_NEW: u16 = 172;      // Plan 118: String creation with capacity
pub const NATIVE_STR_APPEND: u16 = 173;   // Plan 118: String append
pub const NATIVE_INT_STR: u16 = 174;      // Plan 118 Phase 4: int to string
pub const NATIVE_STR_UPPER: u16 = 175;    // Plan 118 Phase 4: string to uppercase

// === Standard Shims ===

pub fn shim_print_i32(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Expect arg on TOS.
    // Callee cleanup: logic assumes we pop the arg.
    let val = task.ram.pop_i32();
    println!("{}", val);
    // Plan 118: print() is a void function, don't push return value
    Ok(())
}

pub fn shim_print_f32(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Not implemented in RAM yet, treating as i32 for now or implementing primitive float read
    // For MVP Phase 1/4 compatibility, assuming i32-as-bits if needed, or simple placeholder
    // But let's assume raw bits.
    let val_bits = task.ram.pop_i32() as u32;
    let val = f32::from_bits(val_bits);
    println!("{}", val);
    // Plan 118: print() is a void function, don't push return value
    Ok(())
}

/// Print a string from the string constant pool.
/// Expects string index (u16) on TOS as i32.
pub fn shim_print_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_index = task.ram.pop_i32() as u16;
    if let Some(bytes) = vm.get_string(str_index) {
        let s = String::from_utf8_lossy(&bytes);
        println!("{}", s);
    } else {
        println!("<invalid string index: {}>", str_index);
    }
    // Plan 118: print() is a void function, don't push return value
    Ok(())
}

// ============================================================================
// List Native Shims
// ============================================================================

/// Create a new list with optional initial elements.
/// Stack: [elem1, elem2, ...] -> list_id
/// Returns: list_id (u64 as i32)
///
/// If elements are on the stack (above bp + 1 + num_locals), they are used to initialize the list.
/// Elements are in LIFO order (top of stack is last element).
pub fn shim_list_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Calculate the target stack pointer (where temps start)
    // RESERVE_STACK pushes n_locals + 1 zeros, so temps start at:
    // - Main task (bp=0): position num_locals + 1
    // - Functions (bp!=0): position bp + 1 + num_locals + 1
    let target_sp = if task.bp == 0 {
        // Main function: padding + locals at 0..num_locals, temps start at num_locals + 1
        task.num_locals + 1
    } else {
        // Regular function: bp points to old_bp, ret_addr at bp-1, old_bp at bp
        // locals at bp+1..bp+num_locals, padding at bp+num_locals+1, temps at bp+num_locals+2
        task.bp + task.num_locals + 2
    };

    // Collect all argument values from the stack
    let mut elems = Vec::new();
    while task.ram.sp > target_sp {
        elems.push(task.ram.pop_i32());
    }

    // Reverse since stack is LIFO (last pushed = first element)
    elems.reverse();

    // Create list with initial elements
    let mut list: ListData<i32> = ListData::new();
    for elem in &elems {
        list.push(*elem);
    }

    // Register the list in the heap
    let list_id = vm.insert_heap_object(list);

    // Return list_id
    task.ram.push_i32(list_id as i32);
    Ok(())
}

/// Push an element to the end of the list.
/// Stack: list_id, elem -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_push(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let elem = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.push(elem);
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Pop an element from the end of the list.
/// Stack: list_id -> elem
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_pop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let elem = list.pop().unwrap_or(0);
            task.ram.push_i32(elem);
            return Ok(());
        }
    }

    // Invalid list_id
    task.ram.push_i32(0);
    Ok(())
}

/// Get the length of the list.
/// Stack: list_id -> len
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.len() as i32);
            return Ok(());
        }
    }

    // Invalid list_id
    task.ram.push_i32(0);
    Ok(())
}

/// Check if the list is empty.
/// Stack: list_id -> is_empty (1 if empty, 0 otherwise)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(if list.is_empty() { 1 } else { 0 });
            return Ok(());
        }
    }

    // Invalid list_id treated as empty
    task.ram.push_i32(1);
    Ok(())
}

/// Clear all elements from the list.
/// Stack: list_id -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.clear();
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Get element at index.
/// Stack: list_id, index -> elem
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            let value = list.get(index).copied().unwrap_or(0);
            task.ram.push_i32(value);
            return Ok(());
        }
    }

    // Invalid list_id
    task.ram.push_i32(0);
    Ok(())
}

/// Set element at index.
/// Stack: list_id, index, elem -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let elem = task.ram.pop_i32();
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.set(index, elem);
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Insert element at index.
/// Stack: list_id, index, elem -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let elem = task.ram.pop_i32();
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.insert(index, elem);
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Remove element at index and return it.
/// Stack: list_id, index -> elem
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            if let Some(elem) = list.remove(index) {
                task.ram.push_i32(elem);
                return Ok(());
            }
        }
    }

    // Return default value if index out of bounds
    task.ram.push_i32(0);
    Ok(())
}

/// Drop/free the list.
/// Stack: list_id -> result (0)
// Plan 077 Phase 5: Updated to use unified registry
pub fn shim_list_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let list_id = task.ram.pop_i32() as u64;
    vm.remove_heap_object(list_id);

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

// ============================================================================
// Iterator Native Shims
// ============================================================================

/// Create an iterator for a list.
/// Stack: list_id -> iterator_id
/// Returns: iterator_id (u32 as i32)
pub fn shim_list_iter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;
    use crate::vm::engine::{Iterator, ListIterator};

    let list_id = task.ram.pop_i32() as u64;

    // Allocate new iterator ID
    let iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);

    // Create iterator state
    let iterator = Iterator::List(ListIterator {
        list_id,
        current_index: 0,
    });

    // Store iterator
    vm.iterators.insert(iterator_id, iterator);

    // Return iterator_id
    task.ram.push_i32(iterator_id as i32);
    Ok(())
}

/// Get next element from iterator.
/// Stack: iterator_id -> element (or -1 for nil)
/// Returns: element value, or -1 if exhausted
// Plan 077 Phase 6: Updated to use unified registry
pub fn shim_iterator_next(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    use crate::vm::engine::Iterator;
    
    let iterator_id = task.ram.pop_i32() as u32;

    // Plan 076 Phase 3: Always return i32 by extracting Int from Value
    // Get the iterator (need to clone to update)
    let result = if let Some(mut iter_mut) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter_mut {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list = obj.read().unwrap();

                    // Check type tag
                    if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                        -1 // Wrong type - return nil
                    } else if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                        // Check if exhausted
                        if list_iter.current_index >= list_data.len() as u32 {
                            // Iterator exhausted - return -1 (nil)
                            -1
                        } else {
                            // Get element at current_index
                            let elem = list_data.get(list_iter.current_index as usize).copied().unwrap_or(0);

                            // Increment index for next call
                            list_iter.current_index += 1;

                            elem
                        }
                    } else {
                        -1 // Downcast failed
                    }
                } else {
                    // Invalid list - return -1 (nil)
                    -1
                }
            }
            Iterator::Map(map_iter) => {
                // Recursively get next element from source iterator
                // For MVP, we don't actually call the function yet
                // We just return the source element as-is

                // Call next() on source iterator
                // We need to manually call the logic here since we can't recursively call shim_iterator_next
                if let Some(mut source_iter) = vm.iterators.get_mut(&map_iter.source_iterator_id) {
                    match &mut *source_iter {
                        Iterator::List(list_iter) => {
                            // Plan 077 Phase 6: Get list from unified registry
                            if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                let list = obj.read().unwrap();

                                if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                    -1 // Wrong type
                                } else if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                                    if list_iter.current_index >= list_data.len() as u32 {
                                        -1 // Source exhausted
                                    } else {
                                        let elem = list_data.get(list_iter.current_index as usize).copied().unwrap_or(0);
                                        list_iter.current_index += 1;

                                        // TODO: Call the function at map_iter.func_addr with elem
                                        // For MVP, just return the element without transformation
                                        elem
                                    }
                                } else {
                                    -1 // Downcast failed
                                }
                            } else {
                                -1 // Invalid list
                            }
                        }
                        Iterator::Map(_) => {
                            // Nested Map not supported yet
                            -1
                        }
                        Iterator::Filter(_) => {
                            // Filter source not supported yet
                            -1
                        }
                    }
                } else {
                    -1 // Source iterator not found
                }
            }
            Iterator::Filter(filter_iter) => {
                // Recursively get next element from source iterator
                // For MVP, we don't actually call the predicate yet
                // We just return the source element as-is (no filtering)

                // Call next() on source iterator
                if let Some(mut source_iter) = vm.iterators.get_mut(&filter_iter.source_iterator_id) {
                    match &mut *source_iter {
                        Iterator::List(list_iter) => {
                            // Plan 077 Phase 6: Get list from unified registry
                            if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                let list = obj.read().unwrap();

                                if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                    -1 // Wrong type
                                } else if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                                    if list_iter.current_index >= list_data.len() as u32 {
                                        -1 // Source exhausted
                                    } else {
                                        let elem = list_data.get(list_iter.current_index as usize).copied().unwrap_or(0);
                                        list_iter.current_index += 1;

                                        // TODO: Call the predicate at filter_iter.func_addr with elem
                                        // For MVP, just return the element without filtering
                                        elem
                                    }
                                } else {
                                    -1 // Downcast failed
                                }
                            } else {
                                -1 // Invalid list
                            }
                        }
                        _ => {
                            // Nested adapters not yet supported
                            return Err(VMError::RuntimeError("Nested adapters not yet implemented".to_string()));
                        }
                    }
                } else {
                    -1 // Invalid source iterator
                }
            }
        }
    } else {
        // Invalid iterator - return -1 (nil)
        -1
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Create a map adapter iterator.
/// Stack: func_addr, iterator_id -> new_iterator_id
/// Returns: new iterator_id (u32 as i32)
///
/// NOTE: For MVP, this creates the MapIterator but the actual function
/// calling during iteration is not yet implemented. The map iterator
/// will currently return an error when next() is called.
pub fn shim_iterator_map(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;
    use crate::vm::engine::{Iterator, MapIterator};

    // Stack: func_addr, iterator_id
    // Pop in reverse order (stack is LIFO)
    let func_addr = task.ram.pop_i32() as u32;
    let source_iterator_id = task.ram.pop_i32() as u32;

    // Verify source iterator exists
    if !vm.iterators.contains_key(&source_iterator_id) {
        task.ram.push_i32(-1); // Return -1 on error
        return Ok(());
    }

    // Allocate new iterator ID for the map adapter
    let new_iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);

    // Create map iterator
    let iterator = Iterator::Map(MapIterator {
        source_iterator_id,
        func_addr,
    });

    // Store iterator
    vm.iterators.insert(new_iterator_id, iterator);

    // Return new iterator_id
    task.ram.push_i32(new_iterator_id as i32);
    Ok(())
}

/// Create a filter adapter iterator.
/// Stack: func_addr, iterator_id -> new_iterator_id
/// Returns: new iterator_id (u32 as i32)
///
/// NOTE: For MVP, this creates the FilterIterator but the actual predicate
/// calling during iteration is not yet implemented. The filter iterator
/// will currently return all elements without filtering.
pub fn shim_iterator_filter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;
    use crate::vm::engine::{Iterator, FilterIterator};

    // Stack: func_addr, iterator_id
    // Pop in reverse order (stack is LIFO)
    let func_addr = task.ram.pop_i32() as u32;
    let source_iterator_id = task.ram.pop_i32() as u32;

    // Verify source iterator exists
    if !vm.iterators.contains_key(&source_iterator_id) {
        task.ram.push_i32(-1); // Return -1 on error
        return Ok(());
    }

    // Allocate new iterator ID for the filter adapter
    let new_iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);

    // Create filter iterator
    let iterator = Iterator::Filter(FilterIterator {
        source_iterator_id,
        func_addr,
    });

    // Store iterator
    vm.iterators.insert(new_iterator_id, iterator);

    // Return new iterator_id
    task.ram.push_i32(new_iterator_id as i32);
    Ok(())
}

// ============================================================================
// Terminal Operations
// ============================================================================

/// Collect all elements from an iterator into a new list.
/// Stack: iterator_id -> list_id
/// Returns: new list_id (lower 32 bits of u64 as i32)
// Plan 077 Phase 6: Updated to use unified registry
pub fn shim_iterator_collect(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    use crate::vm::engine::Iterator;
        use crate::vm::types::ListData;

    let iterator_id = task.ram.pop_i32() as u32;

    // Collect all elements from the iterator
    let mut elements = Vec::new();

    // Get the iterator and consume all elements
    if let Some(mut iter) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list_ref = obj.read().unwrap();

                    if list_ref.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                        if let Some(list_data) = list_ref.as_any().downcast_ref::<ListData<i32>>() {
                            // Collect all remaining elements
                            while list_iter.current_index < list_data.len() as u32 {
                                if let Some(&elem) = list_data.get(list_iter.current_index as usize) {
                                    elements.push(elem);
                                }
                                list_iter.current_index += 1;
                            }
                        }
                    }
                }
            }
            Iterator::Map(_) | Iterator::Filter(_) => {
                // For adapters, we'd need to recursively call next()
                // For MVP, only support direct list iteration
                return Err(VMError::RuntimeError("Collect from adapters not yet implemented".to_string()));
            }
        }
    }

    // Plan 077 Phase 6: Create a new list in unified registry
    let mut list_data: ListData<i32> = ListData::new();
    for elem in elements {
        list_data.push(elem);
    }
    let list_id = vm.insert_heap_object(list_data);

    // Return list_id as i32 (lower 32 bits only for MVP)
    task.ram.push_i32(list_id as i32);

    Ok(())
}

/// Reduce all elements from an iterator using a function.
/// Stack: initial, func_addr, iterator_id -> result
/// Returns: final reduced value
///
/// NOTE: For MVP, this just sums all elements without calling the function.
pub fn shim_iterator_reduce(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::Iterator;
        use crate::vm::types::ListData;

    let iterator_id = task.ram.pop_i32() as u32;
    let _func_addr = task.ram.pop_i32() as u32; // Not used in MVP
    let initial = task.ram.pop_i32();

    let mut result = initial;

    // Plan 077 Phase 6: Reduce all elements from the iterator
    if let Some(mut iter) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list_ref = obj.read().unwrap();

                    if list_ref.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                        if let Some(list_data) = list_ref.as_any().downcast_ref::<ListData<i32>>() {
                            // Sum all remaining elements
                            while list_iter.current_index < list_data.len() as u32 {
                                if let Some(&elem) = list_data.get(list_iter.current_index as usize) {
                                    result += elem;
                                }
                                list_iter.current_index += 1;
                            }
                        }
                    }
                }
            }
            Iterator::Map(_) | Iterator::Filter(_) => {
                return Err(VMError::RuntimeError("Reduce from adapters not yet implemented".to_string()));
            }
        }
    }

    task.ram.push_i32(result);
    Ok(())
}

/// Find the first element from an iterator that matches a predicate.
/// Stack: func_addr, iterator_id -> element (or -1 if not found)
/// Returns: first matching element, or -1 if none match
///
/// NOTE: For MVP, this just returns the first element without calling the predicate.
// Plan 077 Phase 6: Updated to use unified registry
pub fn shim_iterator_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::engine::Iterator;
        use crate::vm::types::ListData;

    let iterator_id = task.ram.pop_i32() as u32;
    let _func_addr = task.ram.pop_i32() as u32; // Not used in MVP

    let mut result = -1; // Default: not found

    // Plan 077 Phase 6: Find first element from the iterator
    if let Some(mut iter) = vm.iterators.get_mut(&iterator_id) {
        match &mut *iter {
            Iterator::List(list_iter) => {
                // Plan 077 Phase 6: Get list from unified registry
                if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                    let list_ref = obj.read().unwrap();

                    if list_ref.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                        if let Some(list_data) = list_ref.as_any().downcast_ref::<ListData<i32>>() {
                            // Return first element
                            if list_iter.current_index < list_data.len() as u32 {
                                if let Some(&elem) = list_data.get(list_iter.current_index as usize) {
                                    result = elem;
                                }
                                list_iter.current_index += 1;
                            }
                        }
                    }
                }
            }
            Iterator::Map(_) | Iterator::Filter(_) => {
                return Err(VMError::RuntimeError("Find from adapters not yet implemented".to_string()));
            }
        }
    }

    task.ram.push_i32(result);
    Ok(())
}

// ============================================================================
// HashMap Shims (Plan 086)
// ============================================================================

/// Create a new HashMap
/// Stack: -> hashmap_id
pub fn shim_hashmap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {


    let map = SpecializedHashMap::new("value");  // Use StringValue variant for generic storage
    let map_id = vm.insert_heap_object(map);

    task.ram.push_i32(map_id as i32);
    Ok(())
}

/// Insert a string key with string value
/// Stack: hashmap_id, key_str_id, value_str_id -> result (0)
pub fn shim_hashmap_insert_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let value_str_bits = task.ram.pop_i32();
    let key_str_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string indices
    let key_str_idx = decode_str_idx(key_str_bits);
    let value_str_idx = decode_str_idx(value_str_bits);

    if let Some(obj) = vm.get_heap_object(map_id) {
        // Get strings from strings pool
        let strings = vm.strings.read().unwrap();
        let key_bytes = strings.get(key_str_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
        let value_bytes = strings.get(value_str_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid value string ID".into()))?;
        drop(strings);

        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        let value_str = auto_val::AutoStr::from(String::from_utf8_lossy(&value_bytes).as_ref());

        // Get write guard and insert
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            map.insert(key_str, Value::Str(value_str))
                .map_err(|e| VMError::RuntimeError(e))?;
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Insert a string key with i32 value
/// Stack: hashmap_id, key_str_id, value -> result (0)
pub fn shim_hashmap_insert_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let value = task.ram.pop_i32();
    let key_str_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index for key
    let key_str_idx = decode_str_idx(key_str_bits);

    if let Some(obj) = vm.get_heap_object(map_id) {
        // Get key string from pool
        let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        drop(key_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            map.insert(key_str, Value::Int(value))
                .map_err(|e| VMError::RuntimeError(e))?;
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get value by string key
/// Stack: hashmap_id, key_str_id -> value (0 if not found)
pub fn shim_hashmap_get_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let key_str_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_str_idx = decode_str_idx(key_str_bits);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            // Get the value from map
            if let Some(value) = map.get(&key_str) {
                // If it's a string, push as tagged string index
                if let auto_val::Value::Str(s) = value {
                    // Add string to strings pool and get index
                    let mut strings = vm.strings.write().unwrap();
                    let str_idx = strings.len() as u16;
                    strings.push(s.as_bytes().to_vec());
                    // Push as tagged string index: -(idx + 1)
                    task.ram.push_i32(-((str_idx as i32) + 1));
                    return Ok(());
                }
            }
        }
    }

    // Not found or not a string - push 0
    task.ram.push_i32(0);
    Ok(())
}

/// Get value by string key (returns i32)
/// Stack: hashmap_id, key_str_id -> value (0 if not found)
pub fn shim_hashmap_get_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let key_str_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_str_idx = decode_str_idx(key_str_bits);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            drop(key_bytes);

            if let Some(value) = map.get(&key_str) {
                if let auto_val::Value::Int(i) = value { i } else { 0 }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Check if key exists
/// Stack: hashmap_id, key_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashmap_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let key_str_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_str_idx = decode_str_idx(key_str_bits);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if map.contains_key(&key_str) { 1 } else { 0 }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Remove a key-value pair
/// Stack: hashmap_id, key_str_id -> result (0)
pub fn shim_hashmap_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let key_str_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_str_idx = decode_str_idx(key_str_bits);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            map.remove(&key_str);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the number of entries
/// Stack: hashmap_id -> size
pub fn shim_hashmap_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let map_id = task.ram.pop_i32() as u64;

    let size = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            map.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Clear all entries
/// Stack: hashmap_id -> result (0)
pub fn shim_hashmap_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            map.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the HashMap (no-op for now, heap objects are managed by Arc)
/// Stack: hashmap_id -> result (0)
pub fn shim_hashmap_drop(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // No-op: heap objects are managed by Arc<RwLock<>>
    // When the last reference is dropped, the object is automatically freed
    Ok(())
}

// ============================================================================
// HashSet Shims (Plan 118 Phase 3)
// ============================================================================

/// Create a new HashSet
/// Stack: -> hashset_id
pub fn shim_hashset_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let set = SpecializedHashSet::new();
    let set_id = vm.insert_heap_object(set);

    task.ram.push_i32(set_id as i32);
    Ok(())
}

/// Insert a string element into the set
/// Stack: hashset_id, elem_str_id -> result (0)
pub fn shim_hashset_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem_str_bits = task.ram.pop_i32();
    let set_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let elem_str_idx = decode_str_idx(elem_str_bits);

    if let Some(obj) = vm.get_heap_object(set_id) {
        // Get string from pool
        let elem_bytes = vm.strings.read().unwrap().get(elem_str_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid element string ID".into()))?;
        let elem_str = String::from_utf8_lossy(&elem_bytes).to_string();
        drop(elem_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.insert(elem_str, ());
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Check if element exists in the set
/// Stack: hashset_id, elem_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashset_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem_str_bits = task.ram.pop_i32();
    let set_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let elem_str_idx = decode_str_idx(elem_str_bits);

    let result = if let Some(obj) = vm.get_heap_object(set_id) {
        let guard = obj.read().unwrap();
        if let Some(set) = guard.as_any().downcast_ref::<SpecializedHashSet>() {
            let elem_bytes = vm.strings.read().unwrap().get(elem_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid element string ID".into()))?;
            let elem_str = String::from_utf8_lossy(&elem_bytes).to_string();

            if set.data.contains_key(&elem_str) { 1 } else { 0 }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Remove an element from the set
/// Stack: hashset_id, elem_str_id -> result (0)
pub fn shim_hashset_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem_str_bits = task.ram.pop_i32();
    let set_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let elem_str_idx = decode_str_idx(elem_str_bits);

    if let Some(obj) = vm.get_heap_object(set_id) {
        let elem_bytes = vm.strings.read().unwrap().get(elem_str_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid element string ID".into()))?;
        let elem_str = String::from_utf8_lossy(&elem_bytes).to_string();
        drop(elem_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.remove(&elem_str);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the number of elements
/// Stack: hashset_id -> size
pub fn shim_hashset_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let set_id = task.ram.pop_i32() as u64;

    let size = if let Some(obj) = vm.get_heap_object(set_id) {
        let guard = obj.read().unwrap();
        if let Some(set) = guard.as_any().downcast_ref::<SpecializedHashSet>() {
            set.data.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Clear all elements
/// Stack: hashset_id -> result (0)
pub fn shim_hashset_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let set_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(set_id) {
        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the HashSet (no-op for now, heap objects are managed by Arc)
/// Stack: hashset_id -> result (0)
pub fn shim_hashset_drop(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // No-op: heap objects are managed by Arc<RwLock<>>
    Ok(())
}

// ============================================================================
// StringBuilder Shims (Plan 118 Phase 3)
// ============================================================================

/// Create a new StringBuilder
/// Stack: capacity -> sb_id
pub fn shim_stringbuilder_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _capacity = task.ram.pop_i32() as usize;

    let sb = crate::vm::collections::SpecializedStringBuilder::with_capacity(_capacity.max(16));
    let sb_id = vm.insert_heap_object(sb);

    task.ram.push_i32(sb_id as i32);
    Ok(())
}

/// Append a string to the StringBuilder
/// Stack: sb_id, str_id -> result (0)
pub fn shim_stringbuilder_append(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_bits = task.ram.pop_i32();
    let sb_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let str_idx = decode_str_idx(str_bits);

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let bytes = vm.strings.read().unwrap().get(str_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
        let s = String::from_utf8_lossy(&bytes).to_string();
        drop(bytes);

        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.push_str(&s);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Append an integer to the StringBuilder
/// Stack: sb_id, int_val -> result (0)
pub fn shim_stringbuilder_append_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let int_val = task.ram.pop_i32();
    let sb_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.push_str(&int_val.to_string());
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Append a character to the StringBuilder
/// Stack: sb_id, char_val -> result (0)
pub fn shim_stringbuilder_append_char(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_bits = task.ram.pop_i32();
    let sb_id = task.ram.pop_i32() as u64;

    // Decode character (char is stored as i32 representing a Unicode code point)
    if let Some(ch) = char::from_u32(char_bits as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
                sb.buffer.push(ch);
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the length of the StringBuilder content
/// Stack: sb_id -> length
pub fn shim_stringbuilder_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = task.ram.pop_i32() as u64;

    let len = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(len);
    Ok(())
}

/// Clear the StringBuilder
/// Stack: sb_id -> result (0)
pub fn shim_stringbuilder_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the StringBuilder (no-op, managed by Arc)
/// Stack: sb_id -> result (0)
pub fn shim_stringbuilder_drop(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    Ok(())
}

/// Build the final string from StringBuilder
/// Stack: sb_id -> str_id (tagged string index)
pub fn shim_stringbuilder_build(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = task.ram.pop_i32() as u64;

    let result_str = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.clone()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Store the string in the string pool and return tagged index
    let str_idx = vm.strings.read().unwrap().len() as i32;
    vm.strings.write().unwrap().push(result_str.into_bytes());

    // Return as tagged string index (negative)
    let tagged_idx = encode_str_idx(str_idx);
    task.ram.push_i32(tagged_idx);
    Ok(())
}

// ============================================================================
// VecDeque Shims (Plan 118 Phase 3)
// ============================================================================

/// Create a new VecDeque
/// Stack: -> deque_id
pub fn shim_vecdeque_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque = crate::vm::collections::SpecializedVecDeque::new();
    let deque_id = vm.insert_heap_object(deque);

    task.ram.push_i32(deque_id as i32);
    Ok(())
}

/// Push an element to the back
/// Stack: deque_id, elem -> result (0)
pub fn shim_vecdeque_push_back(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem = task.ram.pop_i32();
    let deque_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedVecDeque>() {
            deque.data.push_back(elem);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Push an element to the front
/// Stack: deque_id, elem -> result (0)
pub fn shim_vecdeque_push_front(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let elem = task.ram.pop_i32();
    let deque_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedVecDeque>() {
            deque.data.push_front(elem);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Pop an element from the back
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_pop_back(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedVecDeque>() {
            deque.data.pop_back().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Pop an element from the front
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_pop_front(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedVecDeque>() {
            deque.data.pop_front().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the front element
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_front(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedVecDeque>() {
            *deque.data.front().unwrap_or(&0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the back element
/// Stack: deque_id -> elem (or 0 if empty)
pub fn shim_vecdeque_back(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedVecDeque>() {
            *deque.data.back().unwrap_or(&0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the size
/// Stack: deque_id -> size
pub fn shim_vecdeque_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    let size = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedVecDeque>() {
            deque.data.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Check if empty
/// Stack: deque_id -> is_empty (1 or 0)
pub fn shim_vecdeque_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    let is_empty = if let Some(obj) = vm.get_heap_object(deque_id) {
        let guard = obj.read().unwrap();
        if let Some(deque) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedVecDeque>() {
            if deque.data.is_empty() { 1 } else { 0 }
        } else {
            1
        }
    } else {
        1
    };

    task.ram.push_i32(is_empty);
    Ok(())
}

/// Clear the deque
/// Stack: deque_id -> result (0)
pub fn shim_vecdeque_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let deque_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(deque_id) {
        let mut guard = obj.write().unwrap();
        if let Some(deque) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedVecDeque>() {
            deque.data.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Drop the VecDeque (no-op, managed by Arc)
/// Stack: deque_id -> result (0)
pub fn shim_vecdeque_drop(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    Ok(())
}

// ============================================================================
// BTreeMap Shims (Plan 118 Phase 3)
// ============================================================================

/// Create a new BTreeMap
/// Stack: -> btreemap_id
pub fn shim_btreemap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map = crate::vm::collections::SpecializedBTreeMap::new();
    let map_id = vm.insert_heap_object(map);

    task.ram.push_i32(map_id as i32);
    Ok(())
}

/// Insert a key-value pair
/// Stack: btreemap_id, key_str_id, value -> result (0)
pub fn shim_btreemap_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value = task.ram.pop_i32();
    let key_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_idx = decode_str_idx(key_bits);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let key_bytes = vm.strings.read().unwrap().get(key_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        drop(key_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedBTreeMap>() {
            map.data.insert(key_str, value);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get a value by key
/// Stack: btreemap_id, key_str_id -> value (0 if not found)
pub fn shim_btreemap_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_idx = decode_str_idx(key_bits);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedBTreeMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            map.data.get(&key_str).copied().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Check if key exists
/// Stack: btreemap_id, key_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_btreemap_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_idx = decode_str_idx(key_bits);

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedBTreeMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if map.data.contains_key(&key_str) { 1 } else { 0 }
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Remove a key-value pair
/// Stack: btreemap_id, key_str_id -> result (0)
pub fn shim_btreemap_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_bits = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    // Decode tagged string index
    let key_idx = decode_str_idx(key_bits);

    if let Some(obj) = vm.get_heap_object(map_id) {
        let key_bytes = vm.strings.read().unwrap().get(key_idx).cloned()
            .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        drop(key_bytes);

        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedBTreeMap>() {
            map.data.remove(&key_str);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the size
/// Stack: btreemap_id -> size
pub fn shim_btreemap_size(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    let size = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedBTreeMap>() {
            map.data.len() as i32
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(size);
    Ok(())
}

/// Check if empty
/// Stack: btreemap_id -> is_empty (1 or 0)
pub fn shim_btreemap_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    let is_empty = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedBTreeMap>() {
            if map.data.is_empty() { 1 } else { 0 }
        } else {
            1
        }
    } else {
        1
    };

    task.ram.push_i32(is_empty);
    Ok(())
}

/// Clear the map
/// Stack: btreemap_id -> result (0)
pub fn shim_btreemap_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedBTreeMap>() {
            map.data.clear();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get the first (smallest) key
/// Stack: btreemap_id -> key_str_id (or -1 if empty)
pub fn shim_btreemap_first_key(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedBTreeMap>() {
            if let Some(first_key) = map.data.keys().next() {
                // Add string to pool and return tagged index
                let mut strings = vm.strings.write().unwrap();
                let str_idx = strings.len() as u16;
                strings.push(first_key.as_bytes().to_vec());
                drop(strings);
                // Return as tagged string index
                -((str_idx as i32) + 1)
            } else {
                -1  // Empty map
            }
        } else {
            -1
        }
    } else {
        -1
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Get the last (largest) key
/// Stack: btreemap_id -> key_str_id (or -1 if empty)
pub fn shim_btreemap_last_key(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedBTreeMap>() {
            if let Some(last_key) = map.data.keys().next_back() {
                // Add string to pool and return tagged index
                let mut strings = vm.strings.write().unwrap();
                let str_idx = strings.len() as u16;
                strings.push(last_key.as_bytes().to_vec());
                drop(strings);
                // Return as tagged string index
                -((str_idx as i32) + 1)
            } else {
                -1  // Empty map
            }
        } else {
            -1
        }
    } else {
        -1
    };

    task.ram.push_i32(result);
    Ok(())
}

/// Drop the BTreeMap (no-op, managed by Arc)
/// Stack: btreemap_id -> result (0)
pub fn shim_btreemap_drop(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    Ok(())
}

/// Get the length of a string from the constant pool.
/// Stack: str_idx (tagged) -> length (as i32)
pub fn shim_str_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_bits = task.ram.pop_i32();
    // Decode tagged string index
    let str_idx = decode_str_idx(str_bits) as u16;

    if let Some(bytes) = vm.get_string(str_idx) {
        task.ram.push_i32(bytes.len() as i32);
    } else {
        task.ram.push_i32(0);
    }
    Ok(())
}

/// Get the length of a string from the constant pool (String.len alias).
/// Stack: str_idx (tagged) -> length (as i32)
pub fn shim_string_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_bits = task.ram.pop_i32();
    // Decode tagged string index
    let str_idx = decode_str_idx(str_bits) as u16;

    if let Some(bytes) = vm.get_string(str_idx) {
        task.ram.push_i32(bytes.len() as i32);
    } else {
        task.ram.push_i32(0);
    }
    Ok(())
}

/// Plan 118 Phase 4: Create a new mutable string with initial content and capacity.
/// Stack: capacity (i32), initial_str_idx (tagged) -> mut_str_id (i32)
/// The mutable string is stored in heap_objects as a SpecializedStringBuilder.
pub fn shim_str_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop capacity (not used in this simple implementation)
    let _capacity = task.ram.pop_i32();

    // Pop initial string index
    let str_bits = task.ram.pop_i32();
    let str_idx = decode_str_idx(str_bits) as u16;

    // Get initial string content
    let initial_content = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_string()
    } else {
        String::new()
    };

    // Create a SpecializedStringBuilder with initial content
    let mut builder = crate::vm::collections::SpecializedStringBuilder::new();
    builder.buffer = initial_content;

    // Store in heap_objects
    let obj_id = vm.heap_object_id_gen.fetch_add(1, Ordering::SeqCst);
    let obj: Arc<RwLock<dyn crate::vm::heap_object::HeapObject>> = Arc::new(RwLock::new(builder));
    vm.heap_objects.insert(obj_id, obj);

    // Return object ID
    task.ram.push_i32(obj_id as i32);
    Ok(())
}

/// Plan 118 Phase 4: Append a string to a mutable string.
/// Stack: str_idx (tagged), mut_str_id (i32) -> mut_str_id (i32)
pub fn shim_str_append(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string to append
    let str_bits = task.ram.pop_i32();
    let str_idx = decode_str_idx(str_bits) as u16;

    // Pop mutable string ID
    let obj_id = task.ram.pop_i32() as u64;

    // Get string to append
    let to_append = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_string()
    } else {
        String::new()
    };

    // Get and modify the mutable string
    if let Some(obj_arc) = vm.heap_objects.get(&obj_id) {
        let mut obj = obj_arc.write().unwrap();
        if let Some(builder) = obj
            .as_any_mut()
            .downcast_mut::<crate::vm::collections::SpecializedStringBuilder>()
        {
            builder.buffer.push_str(&to_append);
        }
    }

    // Return the same mutable string ID
    task.ram.push_i32(obj_id as i32);
    Ok(())
}

/// Plan 118 Phase 4: Convert integer to string.
/// Stack: int_val (i32) -> str_idx (tagged)
pub fn shim_int_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop integer value
    let val = task.ram.pop_i32();

    // Convert to string
    let str_val = val.to_string();
    let bytes = str_val.into_bytes();

    // Add to string pool
    let str_idx = vm.add_string(bytes);

    // Return tagged string index
    let tagged = encode_str_idx(str_idx as i32);
    task.ram.push_i32(tagged);
    Ok(())
}

/// Plan 118 Phase 4: Convert string to uppercase.
/// Stack: str_idx (tagged) -> str_idx (tagged)
pub fn shim_str_upper(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string index
    let str_bits = task.ram.pop_i32();
    let str_idx = decode_str_idx(str_bits) as u16;

    // Get string content
    let upper_str = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_uppercase()
    } else {
        String::new()
    };

    // Add to string pool
    let bytes = upper_str.into_bytes();
    let new_idx = vm.add_string(bytes);

    // Return tagged string index
    let tagged = encode_str_idx(new_idx as i32);
    task.ram.push_i32(tagged);
    Ok(())
}
