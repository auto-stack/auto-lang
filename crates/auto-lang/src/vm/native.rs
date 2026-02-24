use crate::vm::collections::AutoVMHashMap;
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;
use std::collections::HashMap;
use std::sync::Arc;

pub type ShimFunc = Arc<dyn Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync>;

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
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
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

        // String functions
        self.register(NATIVE_STR_LEN, shim_str_len);
        self.register(NATIVE_STRING_LEN, shim_string_len);
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

// === String Native Function IDs (132+) ===
pub const NATIVE_STR_LEN: u16 = 132;
pub const NATIVE_STRING_LEN: u16 = 133;

// === Standard Shims ===

pub fn shim_print_i32(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Expect arg on TOS.
    // Callee cleanup: logic assumes we pop the arg.
    let val = task.ram.pop_i32();
    println!("{}", val);
    // Push Unit (0) as return value
    task.ram.push_i32(0);
    Ok(())
}

pub fn shim_print_f32(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
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
pub fn shim_print_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_index = task.ram.pop_i32() as u16;
    if let Some(bytes) = vm.get_string(str_index) {
        let s = String::from_utf8_lossy(&bytes);
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

/// Create a new list with optional initial elements.
/// Stack: [elem1, elem2, ...] -> list_id
/// Returns: list_id (u64 as i32)
///
/// If elements are on the stack (above bp + 1 + num_locals), they are used to initialize the list.
/// Elements are in LIFO order (top of stack is last element).
pub fn shim_list_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Calculate the target stack pointer
    // For main() (bp=0): stack layout is [local0, local1, ..., localN-1, temps...]
    //                    where local0..N-1 are the reserved zeros, temps start after them
    // For regular functions: [ret_addr, old_bp, local0, local1, ..., localN, temps...]
    //                        bp points to old_bp, locals at bp+1..bp+N, temps start after
    let target_sp = if task.bp == 0 {
        // Main function: locals are at addresses 0..num_locals-1, temps start at num_locals
        task.num_locals
    } else {
        // Regular function: bp points to saved BP, locals at bp+1..bp+num_locals
        // Temps start at bp + 1 + num_locals
        task.bp + 1 + task.num_locals
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
    

    let map = AutoVMHashMap::new();
    let map_id = vm.insert_heap_object(map);

    task.ram.push_i32(map_id as i32);
    Ok(())
}

/// Insert a string key with i32 value
/// Stack: hashmap_id, key_str_id, value -> result (0)
pub fn shim_hashmap_insert_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let value = task.ram.pop_i32();
    let key_str_id = task.ram.pop_i32() as u64;
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(_map) = guard.as_any().downcast_ref::<AutoVMHashMap>() {
            // Get string from strings pool
            let key_bytes = vm.strings.read().unwrap().get(key_str_id as usize).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            // We need to drop the read guard before we can get a write guard
            drop(guard);

            // Get write guard and insert
            let mut guard = obj.write().unwrap();
            if let Some(map) = guard.as_any_mut().downcast_mut::<AutoVMHashMap>() {
                map.data.insert(key_str, value);
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Insert an integer key (as string) with i32 value
/// Stack: hashmap_id, key_int, value -> result (0)
pub fn shim_hashmap_insert_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let value = task.ram.pop_i32();
    let key_int = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<AutoVMHashMap>() {
            let key_str = key_int.to_string();
            map.data.insert(key_str, value);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Get value by string key
/// Stack: hashmap_id, key_str_id -> value (0 if not found)
pub fn shim_hashmap_get_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let key_str_id = task.ram.pop_i32() as u64;
    let map_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<AutoVMHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_id as usize).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
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

/// Get value by integer key (as string)
/// Stack: hashmap_id, key_int -> value (0 if not found)
pub fn shim_hashmap_get_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let key_int = task.ram.pop_i32();
    let map_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<AutoVMHashMap>() {
            let key_str = key_int.to_string();
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
/// Stack: hashmap_id, key_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashmap_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let key_str_id = task.ram.pop_i32() as u64;
    let map_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<AutoVMHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_id as usize).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
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
/// Stack: hashmap_id, key_str_id -> result (0)
pub fn shim_hashmap_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let key_str_id = task.ram.pop_i32() as u64;
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<AutoVMHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_id as usize).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            map.data.remove(&key_str);
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
        if let Some(map) = guard.as_any().downcast_ref::<AutoVMHashMap>() {
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

/// Clear all entries
/// Stack: hashmap_id -> result (0)
pub fn shim_hashmap_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<AutoVMHashMap>() {
            map.data.clear();
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

/// Get the length of a string from the constant pool.
/// Stack: str_idx -> length (as i32)
pub fn shim_str_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_i32() as u16;

    if let Some(bytes) = vm.get_string(str_idx) {
        task.ram.push_i32(bytes.len() as i32);
    } else {
        task.ram.push_i32(0);
    }
    Ok(())
}

/// Get the length of a string from the constant pool (String.len alias).
/// Stack: str_idx -> length (as i32)
pub fn shim_string_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_i32() as u16;

    if let Some(bytes) = vm.get_string(str_idx) {
        task.ram.push_i32(bytes.len() as i32);
    } else {
        task.ram.push_i32(0);
    }
    Ok(())
}
