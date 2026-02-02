use crate::vm::engine::{BigVM, VMError};
use crate::vm::task::AutoTask;
use std::collections::HashMap;
use std::sync::Arc;

pub type ShimFunc = Arc<dyn Fn(&mut AutoTask, &BigVM) -> Result<(), VMError> + Send + Sync>;

pub struct NativeInterface {
    registry: HashMap<u16, ShimFunc>,
}

impl NativeInterface {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &BigVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        self.registry.insert(id, Arc::new(func));
    }

    pub fn get(&self, id: u16) -> Option<&ShimFunc> {
        self.registry.get(&id)
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
        self.register(NATIVE_LIST_DROP, shim_list_drop);

        // Iterator functions
        self.register(NATIVE_LIST_ITER, shim_list_iter);
        self.register(NATIVE_ITERATOR_NEXT, shim_iterator_next);
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
pub const NATIVE_LIST_DROP: u16 = 108;

// === Iterator Native Functions (109+) ===
pub const NATIVE_LIST_ITER: u16 = 109;
pub const NATIVE_ITERATOR_NEXT: u16 = 110;

// === Standard Shims ===

pub fn shim_print_i32(task: &mut AutoTask, _vm: &BigVM) -> Result<(), VMError> {
    // Expect arg on TOS.
    // Callee cleanup: logic assumes we pop the arg.
    let val = task.ram.pop_i32();
    println!("{}", val);
    // Push Unit (0) as return value
    task.ram.push_i32(0);
    Ok(())
}

pub fn shim_print_f32(task: &mut AutoTask, _vm: &BigVM) -> Result<(), VMError> {
    // Not implemented in RAM yet, treating as i32 for now or implementing primitive float read
    // For MVP Phase 1/4 compatibility, assuming i32-as-bits if needed, or simple placeholder
    // But let's assume raw bits.
    let val_bits = task.ram.pop_i32() as u32;
    let val = f32::from_bits(val_bits);
    println!("{}", val);
    // Push Unit (0) as return value
    task.ram.push_i32(0);
    Ok(())
}

/// Print a string from the string constant pool.
/// Expects string index (u16) on TOS as i32.
pub fn shim_print_str(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let str_index = task.ram.pop_i32() as u16;
    if let Some(bytes) = vm.get_string(str_index) {
        let s = String::from_utf8_lossy(bytes);
        println!("{}", s);
    } else {
        println!("<invalid string index: {}>", str_index);
    }
    // Push Unit (0) as return value
    task.ram.push_i32(0);
    Ok(())
}

// ============================================================================
// List Native Shims
// ============================================================================

/// Create a new empty list.
/// Stack: -> list_id
/// Returns: list_id (u64 as i32)
pub fn shim_list_new(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;

    let list_id = vm.list_id_gen.fetch_add(1, Ordering::Relaxed);
    let list = Vec::new();
    vm.lists.insert(list_id, Arc::new(std::sync::RwLock::new(list)));

    // Return list_id
    task.ram.push_i32(list_id as i32);
    Ok(())
}

/// Push an element to the end of the list.
/// Stack: list_id, elem -> result (0)
pub fn shim_list_push(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let elem = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let mut list = list.write().unwrap();
        list.push(elem);
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Pop an element from the end of the list.
/// Stack: list_id -> elem
pub fn shim_list_pop(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let mut list = list.write().unwrap();
        let elem = list.pop().unwrap_or(0);
        task.ram.push_i32(elem);
    } else {
        task.ram.push_i32(0); // Invalid list_id
    }

    Ok(())
}

/// Get the length of the list.
/// Stack: list_id -> len
pub fn shim_list_len(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let list = list.read().unwrap();
        task.ram.push_i32(list.len() as i32);
    } else {
        task.ram.push_i32(0); // Invalid list_id
    }

    Ok(())
}

/// Check if the list is empty.
/// Stack: list_id -> is_empty (1 if empty, 0 otherwise)
pub fn shim_list_is_empty(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let list = list.read().unwrap();
        task.ram.push_i32(if list.is_empty() { 1 } else { 0 });
    } else {
        task.ram.push_i32(1); // Invalid list_id treated as empty
    }

    Ok(())
}

/// Clear all elements from the list.
/// Stack: list_id -> result (0)
pub fn shim_list_clear(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let mut list = list.write().unwrap();
        list.clear();
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Get element at index.
/// Stack: list_id, index -> elem
pub fn shim_list_get(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let list = list.read().unwrap();
        let elem = list.get(index).copied().unwrap_or(0);
        task.ram.push_i32(elem);
    } else {
        task.ram.push_i32(0); // Invalid list_id
    }

    Ok(())
}

/// Set element at index.
/// Stack: list_id, index, elem -> result (0)
pub fn shim_list_set(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let elem = task.ram.pop_i32();
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(list) = vm.lists.get(&list_id) {
        let mut list = list.write().unwrap();
        if index < list.len() {
            list[index] = elem;
        }
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Drop/free the list.
/// Stack: list_id -> result (0)
pub fn shim_list_drop(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let list_id = task.ram.pop_i32() as u64;
    vm.lists.remove(&list_id);

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
pub fn shim_list_iter(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;

    let list_id = task.ram.pop_i32() as u64;

    // Allocate new iterator ID
    let iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);

    // Create iterator state
    let iterator = crate::vm::engine::ListIterator {
        list_id,
        current_index: 0,
    };

    // Store iterator
    vm.iterators.insert(iterator_id, iterator);

    // Return iterator_id
    task.ram.push_i32(iterator_id as i32);
    Ok(())
}

/// Get next element from iterator.
/// Stack: iterator_id -> element (or -1 for nil)
/// Returns: element value, or -1 if exhausted
pub fn shim_iterator_next(task: &mut AutoTask, vm: &BigVM) -> Result<(), VMError> {
    let iterator_id = task.ram.pop_i32() as u32;

    // Get the iterator (need to clone to update)
    let result = if let Some(mut iter_mut) = vm.iterators.get_mut(&iterator_id) {
        // Get the list
        if let Some(list) = vm.lists.get(&iter_mut.list_id) {
            let list = list.read().unwrap();

            // Check if exhausted
            if iter_mut.current_index >= list.len() as u32 {
                // Iterator exhausted - return -1 (nil)
                -1
            } else {
                // Get element at current_index
                let elem = list[iter_mut.current_index as usize];

                // Increment index for next call
                iter_mut.current_index += 1;

                elem
            }
        } else {
            // Invalid list - return -1 (nil)
            -1
        }
    } else {
        // Invalid iterator - return -1 (nil)
        -1
    };

    task.ram.push_i32(result);
    Ok(())
}
