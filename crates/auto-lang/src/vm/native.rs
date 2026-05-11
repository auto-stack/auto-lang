use crate::vm::collections::{SpecializedHashMap, SpecializedHashSet};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::task::AutoTask;
use auto_val::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

/// Decode a tagged string index from stack value.
/// LOAD_STR pushes string indices as negative tagged values: -(str_idx as i32) - 1
/// This function decodes the tag to get the actual string pool index.
#[cfg(not(feature = "nanbox"))]
#[inline]
fn decode_str_idx(bits: i32) -> usize {
    if bits < 0 {
        (-bits - 1) as usize
    } else {
        bits as usize
    }
}

/// Under nanbox, decode from a NanoValue popped from the stack.
#[cfg(feature = "nanbox")]
#[inline]
fn decode_str_idx_nv(nv: auto_val::NanoValue) -> usize {
    auto_val::decode_string(nv) as usize
}

/// Encode a string pool index as a tagged value (negative).
#[cfg(not(feature = "nanbox"))]
#[inline]
#[allow(dead_code)]
fn encode_str_idx(idx: i32) -> i32 {
    -(idx + 1)
}

/// Under nanbox, encode as a NanoValue string tag.
#[cfg(feature = "nanbox")]
#[inline]
fn encode_str_idx_nv(idx: u32) -> auto_val::NanoValue {
    auto_val::encode_string(idx)
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
#[derive(Clone)]
pub struct NativeInterface {
    /// Static shims: direct array lookup for maximum performance
    static_shims: Vec<Option<ShimFunc>>,
    /// Dynamic shims: HashMap for flexibility
    dynamic_shims: HashMap<u16, ShimFunc>,
    /// Next available dynamic ID
    next_dynamic_id: u16,
    /// Plan 200 Task 3.3: name -> ID mapping for CALL_SPEC fallback
    name_to_id: HashMap<String, u16>,
}

impl NativeInterface {
    pub fn new() -> Self {
        Self {
            static_shims: vec![None; STATIC_ID_MAX as usize],
            dynamic_shims: HashMap::new(),
            next_dynamic_id: DYNAMIC_ID_START,
            name_to_id: HashMap::new(),
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

    /// Register a name -> ID mapping for CALL_SPEC fallback (Plan 200 Task 3.3)
    pub fn register_name(&mut self, name: &str, id: u16) {
        self.name_to_id.insert(name.to_string(), id);
    }

    /// Look up a native ID by qualified name (e.g., "Result.Ok.map_err")
    ///
    /// Supports canonical normalization: "List.push" → checks "auto.list.push" etc.
    pub fn resolve(&self, name: &str) -> Option<u16> {
        // Direct lookup first
        if let Some(id) = self.name_to_id.get(name).copied() {
            return Some(id);
        }
        // Canonical normalization: "List.push" → "auto.list.push"
        if !name.starts_with("auto.") && !name.starts_with("rust.") && !name.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(name) {
                if let Some(id) = self.name_to_id.get(&canonical).copied() {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Convert a short native name to its canonical "auto.X.Y" form.
    fn to_canonical(name: &str) -> Option<String> {
        let (prefix, rest) = name.split_once('.')?;
        use crate::vm::native_registry::TYPE_CANONICAL_MAP;
        for &(short, canonical_prefix) in TYPE_CANONICAL_MAP {
            if prefix == short {
                return Some(format!("{}.{}", canonical_prefix, rest));
            }
        }
        let lower = prefix.to_lowercase();
        Some(format!("auto.{}.{}", lower, rest))
    }

    /// Get the next available dynamic ID
    pub fn next_dynamic_id(&self) -> u16 {
        self.next_dynamic_id
    }

    /// Plan 212b Task 4: Merge shims from another NativeInterface into this one
    ///
    /// Used to merge Rust FFI bridge native shims into the main VM's
    /// NativeInterface after the bridge has loaded and registered functions.
    pub fn merge(&mut self, other: &NativeInterface) {
        // Merge static shims
        for (id, shim) in other.static_shims.iter().enumerate() {
            if let Some(shim) = shim {
                if id < self.static_shims.len() {
                    self.static_shims[id] = Some(shim.clone());
                }
            }
        }
        // Merge dynamic shims
        for (id, shim) in &other.dynamic_shims {
            self.dynamic_shims.insert(*id, shim.clone());
        }
        // Advance next_dynamic_id if needed
        if other.next_dynamic_id > self.next_dynamic_id {
            self.next_dynamic_id = other.next_dynamic_id;
        }
    }

    /// Register a shim by name, looking up the ID from BIGVM_NATIVES.
    ///
    /// Used by inventory-based auto-registration (Plan 198).
    /// Returns the resolved ID.
    pub fn register_shim_by_name<F>(&mut self, name: &str, func: F) -> u16
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        use crate::vm::native_registry::BIGVM_NATIVES;
        let id = BIGVM_NATIVES
            .lock()
            .unwrap()
            .resolve_qualified(name)
            .unwrap_or_else(|| panic!("register_shim_by_name: '{}' not found in BIGVM_NATIVES", name));
        self.register_static(id, func);
        self.register_name(name, id);
        id
    }

    /// Collect all inventory-submitted FFI registrations and register them.
    ///
    /// Called during VM init after BIGVM_NATIVES is populated.
    pub fn build_from_inventory(&mut self) {
        use crate::vm::ffi::StaticFFIRegistration;
        for entry in inventory::iter::<StaticFFIRegistration> {
            use crate::vm::native_registry::BIGVM_NATIVES;
            let natives = BIGVM_NATIVES.lock().unwrap();
            let id = natives
                .resolve_qualified(entry.name)
                .unwrap_or_else(|| panic!("build_from_inventory: '{}' not found in BIGVM_NATIVES", entry.name));
            assert!(id < STATIC_ID_MAX, "Inventory shim '{}' resolved to dynamic ID {}", entry.name, id);
            self.static_shims[id as usize] = Some(Arc::new(entry.shim));
            // Register names for CALL_SPEC resolve
            // Register the inventory name itself (e.g., "Str.char_at")
            self.name_to_id.entry(entry.name.to_string()).or_insert(id);
            // Register the canonical form (e.g., "auto.str.char_at")
            if let Some(canonical) = natives.resolve_qualified_to_canonical(entry.name) {
                self.name_to_id.entry(canonical).or_insert(id);
            }
            // Also register lowercase variant for CALL_SPEC dispatch
            if let Some(lower) = Self::to_canonical(entry.name) {
                self.name_to_id.entry(lower).or_insert(id);
            }
        }
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
        self.register(NATIVE_PRINT_F64, shim_print_f64);
        self.register(NATIVE_PRINT_STR, shim_print_str);
        self.register(NATIVE_WRITE_STR, shim_write_str);

        // Assert functions
        self.register(NATIVE_ASSERT, shim_assert);
        self.register(NATIVE_ASSERT_EQ, shim_assert_eq);
        self.register(NATIVE_ASSERT_NE, shim_assert_ne);

        // Runtime panic (for #[vm] stubs when native not found)
        self.register(NATIVE_RUNTIME_PANIC, shim_runtime_panic);

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
        self.register(NATIVE_LIST_RESERVE, shim_list_reserve);

        // List higher-order functions (Plan 206)
        self.register(NATIVE_LIST_MAP, shim_list_map);
        self.register(NATIVE_LIST_FILTER, shim_list_filter);
        self.register(NATIVE_LIST_FOREACH, shim_list_for_each);
        self.register(NATIVE_LIST_FIND, shim_list_find);
        self.register(NATIVE_LIST_ANY, shim_list_any);
        self.register(NATIVE_LIST_ALL, shim_list_all);
        self.register(NATIVE_LIST_REDUCE, shim_list_reduce);
        self.register(NATIVE_LIST_SORT, shim_list_sort);
        self.register(NATIVE_LIST_SORT_BY, shim_list_sort_by);
        self.register(NATIVE_LIST_JOIN, shim_list_join);
        self.register(NATIVE_LIST_CONTAINS, shim_list_contains);

        // Plan 200 Task 3.3: Result.map_err(closure)
        self.register(NATIVE_RESULT_MAP_ERR, shim_result_map_err);
        self.register_name("Result.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("Result.Ok.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("Result.Err.map_err", NATIVE_RESULT_MAP_ERR);

        // Iterator functions
        self.register(NATIVE_LIST_ITER, shim_list_iter);
        self.register(NATIVE_ITERATOR_NEXT, shim_iterator_next);
        self.register(NATIVE_ITERATOR_MAP, shim_iterator_map);
        self.register(NATIVE_ITERATOR_FILTER, shim_iterator_filter);
        self.register(NATIVE_ITERATOR_COLLECT, shim_iterator_collect);
        self.register(NATIVE_ITERATOR_REDUCE, shim_iterator_reduce);
        self.register(NATIVE_ITERATOR_FIND, shim_iterator_find);
        self.register(NATIVE_ITERATOR_ENUMERATE, shim_iterator_enumerate);

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
        self.register(NATIVE_HASHMAP_IS_EMPTY, shim_hashmap_is_empty);
        self.register(NATIVE_HASHMAP_GET_OR, shim_hashmap_get_or);
        self.register(NATIVE_HASHMAP_KEYS, shim_hashmap_keys);

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
        self.register(NATIVE_STRING_FROM, shim_string_from);

        // String/Uint extension functions (235-236)
        self.register(NATIVE_STR_BYTES, shim_str_bytes);
        self.register(NATIVE_UINT_TO_HEX, shim_uint_to_hex);

        // Mutable String functions (177-186)
        self.register(NATIVE_STRING_NEW, shim_string_new);
        self.register(NATIVE_STRING_PUSH, shim_string_push);
        self.register(NATIVE_STRING_POP, shim_string_pop);
        self.register(NATIVE_STRING_GET, shim_string_get);
        self.register(NATIVE_STRING_SET, shim_string_set);
        self.register(NATIVE_STRING_INSERT, shim_string_insert);
        self.register(NATIVE_STRING_REMOVE, shim_string_remove);
        self.register(NATIVE_STRING_CLEAR, shim_string_clear);
        self.register(NATIVE_STRING_IS_EMPTY, shim_string_is_empty);
        self.register(NATIVE_STRING_RESERVE, shim_string_reserve);

        // Memory allocation functions
        self.register(NATIVE_ALLOC_ARRAY, shim_alloc_array);
        self.register(NATIVE_REALLOC_ARRAY, shim_realloc_array);
        self.register(NATIVE_FREE_ARRAY, shim_free_array);

        // Storage functions
        self.register(NATIVE_HEAP_NEW, shim_heap_new);
        self.register(NATIVE_HEAP_CAPACITY, shim_heap_capacity);
        self.register(NATIVE_HEAP_TRY_GROW, shim_heap_try_grow);
        self.register(NATIVE_HEAP_DROP, shim_heap_drop);
        self.register(NATIVE_INLINE_INT64_NEW, shim_inline_int64_new);
        self.register(NATIVE_INLINE_INT64_CAPACITY, shim_inline_int64_capacity);
        self.register(NATIVE_INLINE_INT64_TRY_GROW, shim_inline_int64_try_grow);
        self.register(NATIVE_INLINE_INT64_DROP, shim_inline_int64_drop);

        // List extra functions
        self.register(NATIVE_LIST_CAPACITY, shim_list_capacity);

        // Plan 178: Bit operation shims
        self.register(NATIVE_INT_AND, shim_int_and);
        self.register(NATIVE_INT_OR, shim_int_or);
        self.register(NATIVE_INT_XOR, shim_int_xor);
        self.register(NATIVE_INT_NOT, shim_int_not);
        self.register(NATIVE_INT_SHL, shim_int_shl);
        self.register(NATIVE_INT_SHR, shim_int_shr);
        self.register(NATIVE_INT_SAR, shim_int_sar);
        self.register(NATIVE_INT_ROL, shim_int_rol);
        self.register(NATIVE_INT_ROR, shim_int_ror);
        self.register(NATIVE_INT_COUNT_ONES, shim_int_count_ones);
        self.register(NATIVE_INT_LEADING_ZEROS, shim_int_leading_zeros);
        self.register(NATIVE_INT_TRAILING_ZEROS, shim_int_trailing_zeros);
        self.register(NATIVE_INT_BITREV, shim_int_bitrev);
        self.register(NATIVE_INT_BIT_READ, shim_int_bit_read);
        self.register(NATIVE_INT_BIT_TEST, shim_int_bit_test);
        self.register(NATIVE_INT_BIT_ON, shim_int_bit_on);
        self.register(NATIVE_INT_BIT_OFF, shim_int_bit_off);
        self.register(NATIVE_INT_BIT_FLIP, shim_int_bit_flip);

        // CALL_SPEC fallback: register canonical names for type.method dispatch
        // These allow CALL_SPEC to resolve "List.push" → canonical "auto.list.push" → ID
        self.register_name("auto.list.new", NATIVE_LIST_NEW);
        self.register_name("auto.list.push", NATIVE_LIST_PUSH);
        self.register_name("auto.list.pop", NATIVE_LIST_POP);
        self.register_name("auto.list.len", NATIVE_LIST_LEN);
        self.register_name("auto.list.is_empty", NATIVE_LIST_IS_EMPTY);
        self.register_name("auto.list.clear", NATIVE_LIST_CLEAR);
        self.register_name("auto.list.get", NATIVE_LIST_GET);
        self.register_name("auto.list.set", NATIVE_LIST_SET);
        self.register_name("auto.list.insert", NATIVE_LIST_INSERT);
        self.register_name("auto.list.remove", NATIVE_LIST_REMOVE);
        self.register_name("auto.list.drop", NATIVE_LIST_DROP);
        self.register_name("auto.list.reserve", NATIVE_LIST_RESERVE);
        self.register_name("auto.list.capacity", NATIVE_LIST_CAPACITY);
        self.register_name("auto.list.map", NATIVE_LIST_MAP);
        self.register_name("auto.list.filter", NATIVE_LIST_FILTER);
        self.register_name("auto.list.for_each", NATIVE_LIST_FOREACH);
        self.register_name("auto.list.find", NATIVE_LIST_FIND);
        self.register_name("auto.list.any", NATIVE_LIST_ANY);
        self.register_name("auto.list.all", NATIVE_LIST_ALL);
        self.register_name("auto.list.reduce", NATIVE_LIST_REDUCE);
        self.register_name("auto.list.sort", NATIVE_LIST_SORT);
        self.register_name("auto.list.sort_by", NATIVE_LIST_SORT_BY);
        self.register_name("auto.list.iter", NATIVE_LIST_ITER);
        self.register_name("auto.list.join", NATIVE_LIST_JOIN);
        self.register_name("auto.list.contains", NATIVE_LIST_CONTAINS);

        self.register_name("auto.hashmap.new", NATIVE_HASHMAP_NEW);
        self.register_name("auto.hashmap.insert", NATIVE_HASHMAP_INSERT_STR);
        self.register_name("auto.hashmap.set", NATIVE_HASHMAP_INSERT_STR); // Auto syntax: map.set()
        self.register_name("auto.hashmap.insert_str", NATIVE_HASHMAP_INSERT_STR);
        self.register_name("auto.hashmap.insert_int", NATIVE_HASHMAP_INSERT_INT);
        self.register_name("auto.hashmap.get", NATIVE_HASHMAP_GET_STR);
        self.register_name("auto.hashmap.get_str", NATIVE_HASHMAP_GET_STR);
        self.register_name("auto.hashmap.get_int", NATIVE_HASHMAP_GET_INT);
        self.register_name("auto.hashmap.contains", NATIVE_HASHMAP_CONTAINS);
        self.register_name("auto.hashmap.contains_key", NATIVE_HASHMAP_CONTAINS);
        self.register_name("auto.hashmap.remove", NATIVE_HASHMAP_REMOVE);
        self.register_name("auto.hashmap.size", NATIVE_HASHMAP_SIZE);
        self.register_name("auto.hashmap.len", NATIVE_HASHMAP_SIZE);
        self.register_name("auto.hashmap.clear", NATIVE_HASHMAP_CLEAR);
        self.register_name("auto.hashmap.drop", NATIVE_HASHMAP_DROP);
        self.register_name("auto.hashmap.is_empty", NATIVE_HASHMAP_IS_EMPTY);
        self.register_name("auto.hashmap.get_or", NATIVE_HASHMAP_GET_OR);
        self.register_name("auto.hashmap.keys", NATIVE_HASHMAP_KEYS);

        // String methods for CALL_SPEC dispatch
        self.register_name("auto.str.len", NATIVE_STR_LEN);
        self.register_name("auto.str.contains", NATIVE_STR_CONTAINS);
        self.register_name("auto.str.starts_with", NATIVE_STR_STARTS_WITH);
        self.register_name("auto.str.ends_with", NATIVE_STR_ENDS_WITH);

        // Plan 212 Phase 2: Rand native shims (built-in, no external crate needed)
        self.register(NATIVE_RAND_THREAD_RNG, shim_rand_thread_rng);
        self.register(NATIVE_RNG_GEN_RANGE, shim_rng_gen_range);
        self.register(NATIVE_RNG_GEN, shim_rng_gen);
        self.register(NATIVE_RNG_DROP, shim_rng_drop);
        self.register(NATIVE_RAND_RANDOM, shim_rand_random);
        self.register_name("auto.rand.thread_rng", NATIVE_RAND_THREAD_RNG);
        self.register_name("auto.rand.random", NATIVE_RAND_RANDOM);
        self.register_name("auto.rng.gen_range", NATIVE_RNG_GEN_RANGE);
        self.register_name("auto.rng.gen", NATIVE_RNG_GEN);
        self.register_name("auto.rng.drop", NATIVE_RNG_DROP);

        // Plan 212 Phase 2: Log no-op shim (env_logger.init(), log.set_max_level(), etc.)
        self.register(NATIVE_LOG_NOOP, shim_log_noop);
        self.register_name("auto.log.noop", NATIVE_LOG_NOOP);

        // Plan 212 Phase 2.2: Regex opaque struct shims
        self.register(NATIVE_RE_OPAQUE_NEW, shim_re_opaque_new);
        self.register(NATIVE_RE_OPAQUE_IS_MATCH, shim_re_opaque_is_match);
        self.register(NATIVE_RE_OPAQUE_FIND, shim_re_opaque_find);
        self.register(NATIVE_RE_OPAQUE_FIND_ALL, shim_re_opaque_find_all);
        self.register(NATIVE_RE_OPAQUE_REPLACE_ALL, shim_re_opaque_replace_all);
        self.register(NATIVE_RE_OPAQUE_CAPTURES, shim_re_opaque_captures);
        self.register(NATIVE_RE_OPAQUE_DROP, shim_re_opaque_drop);
        self.register_name("auto.re_opaque.new", NATIVE_RE_OPAQUE_NEW);
        self.register_name("auto.re_opaque.is_match", NATIVE_RE_OPAQUE_IS_MATCH);
        self.register_name("auto.re_opaque.find", NATIVE_RE_OPAQUE_FIND);
        self.register_name("auto.re_opaque.find_all", NATIVE_RE_OPAQUE_FIND_ALL);
        self.register_name("auto.re_opaque.replace_all", NATIVE_RE_OPAQUE_REPLACE_ALL);
        self.register_name("auto.re_opaque.captures", NATIVE_RE_OPAQUE_CAPTURES);
        self.register_name("auto.re_opaque.drop", NATIVE_RE_OPAQUE_DROP);

        // Plan 212 Phase 2.2: Url opaque struct shims
        self.register(NATIVE_URL_OPAQUE_PARSE, shim_url_opaque_parse);
        self.register(NATIVE_URL_OPAQUE_SCHEME, shim_url_opaque_scheme);
        self.register(NATIVE_URL_OPAQUE_HOST_STR, shim_url_opaque_host_str);
        self.register(NATIVE_URL_OPAQUE_PATH, shim_url_opaque_path);
        self.register(NATIVE_URL_OPAQUE_FRAGMENT, shim_url_opaque_fragment);
        self.register(NATIVE_URL_OPAQUE_PORT, shim_url_opaque_port);
        self.register(NATIVE_URL_OPAQUE_QUERY_PAIRS, shim_url_opaque_query_pairs);
        self.register(NATIVE_URL_OPAQUE_JOIN, shim_url_opaque_join);
        self.register(NATIVE_URL_OPAQUE_ORIGIN, shim_url_opaque_origin);
        self.register(NATIVE_URL_OPAQUE_DROP, shim_url_opaque_drop);
        self.register_name("auto.url_opaque.parse", NATIVE_URL_OPAQUE_PARSE);
        self.register_name("auto.url_opaque.scheme", NATIVE_URL_OPAQUE_SCHEME);
        self.register_name("auto.url_opaque.host_str", NATIVE_URL_OPAQUE_HOST_STR);
        self.register_name("auto.url_opaque.path", NATIVE_URL_OPAQUE_PATH);
        self.register_name("auto.url_opaque.fragment", NATIVE_URL_OPAQUE_FRAGMENT);
        self.register_name("auto.url_opaque.port", NATIVE_URL_OPAQUE_PORT);
        self.register_name("auto.url_opaque.query_pairs", NATIVE_URL_OPAQUE_QUERY_PAIRS);
        self.register_name("auto.url_opaque.join", NATIVE_URL_OPAQUE_JOIN);
        self.register_name("auto.url_opaque.origin", NATIVE_URL_OPAQUE_ORIGIN);
        self.register_name("auto.url_opaque.drop", NATIVE_URL_OPAQUE_DROP);

        // Plan 212 Phase 2.2: Semver opaque struct shims
        self.register(NATIVE_SEMVER_OPAQUE_PARSE, shim_semver_opaque_parse);
        self.register(NATIVE_SEMVER_OPAQUE_MAJOR, shim_semver_opaque_major);
        self.register(NATIVE_SEMVER_OPAQUE_MINOR, shim_semver_opaque_minor);
        self.register(NATIVE_SEMVER_OPAQUE_PATCH, shim_semver_opaque_patch);
        self.register(NATIVE_SEMVER_OPAQUE_PRE, shim_semver_opaque_pre);
        self.register(NATIVE_SEMVER_OPAQUE_TO_STRING, shim_semver_opaque_to_string);
        self.register(NATIVE_SEMVER_OPAQUE_CMP_GT, shim_semver_opaque_cmp_gt);
        self.register(NATIVE_SEMVER_OPAQUE_DROP, shim_semver_opaque_drop);
        self.register_name("auto.semver_opaque.parse", NATIVE_SEMVER_OPAQUE_PARSE);
        self.register_name("auto.semver_opaque.major", NATIVE_SEMVER_OPAQUE_MAJOR);
        self.register_name("auto.semver_opaque.minor", NATIVE_SEMVER_OPAQUE_MINOR);
        self.register_name("auto.semver_opaque.patch", NATIVE_SEMVER_OPAQUE_PATCH);
        self.register_name("auto.semver_opaque.pre", NATIVE_SEMVER_OPAQUE_PRE);
        self.register_name("auto.semver_opaque.to_string", NATIVE_SEMVER_OPAQUE_TO_STRING);
        self.register_name("auto.semver_opaque.cmp_gt", NATIVE_SEMVER_OPAQUE_CMP_GT);
        self.register_name("auto.semver_opaque.drop", NATIVE_SEMVER_OPAQUE_DROP);

        // Plan 212 Phase 2.3: chrono opaque struct shims
        self.register(NATIVE_CHRONO_LOCAL_NOW, shim_chrono_local_now);
        self.register(NATIVE_CHRONO_YEAR, shim_chrono_year);
        self.register(NATIVE_CHRONO_MONTH, shim_chrono_month);
        self.register(NATIVE_CHRONO_DAY, shim_chrono_day);
        self.register(NATIVE_CHRONO_HOUR, shim_chrono_hour);
        self.register(NATIVE_CHRONO_MINUTE, shim_chrono_minute);
        self.register(NATIVE_CHRONO_SECOND, shim_chrono_second);
        self.register(NATIVE_CHRONO_TIMESTAMP, shim_chrono_timestamp);
        self.register(NATIVE_CHRONO_FORMAT, shim_chrono_format);
        self.register(NATIVE_CHRONO_DROP, shim_chrono_drop);
        self.register_name("auto.chrono_opaque.local_now", NATIVE_CHRONO_LOCAL_NOW);
        self.register_name("auto.chrono_opaque.year", NATIVE_CHRONO_YEAR);
        self.register_name("auto.chrono_opaque.month", NATIVE_CHRONO_MONTH);
        self.register_name("auto.chrono_opaque.day", NATIVE_CHRONO_DAY);
        self.register_name("auto.chrono_opaque.hour", NATIVE_CHRONO_HOUR);
        self.register_name("auto.chrono_opaque.minute", NATIVE_CHRONO_MINUTE);
        self.register_name("auto.chrono_opaque.second", NATIVE_CHRONO_SECOND);
        self.register_name("auto.chrono_opaque.timestamp", NATIVE_CHRONO_TIMESTAMP);
        self.register_name("auto.chrono_opaque.format", NATIVE_CHRONO_FORMAT);
        self.register_name("auto.chrono_opaque.drop", NATIVE_CHRONO_DROP);

        // Plan 212 Phase 2.3: base64 pure function shims
        self.register(NATIVE_BASE64_ENCODE, shim_base64_encode);
        self.register(NATIVE_BASE64_DECODE, shim_base64_decode);
        self.register_name("auto.base64.encode", NATIVE_BASE64_ENCODE);
        self.register_name("auto.base64.decode", NATIVE_BASE64_DECODE);

        // Plan 212 Phase 2.3: hex pure function shims
        self.register(NATIVE_HEX_ENCODE, shim_hex_encode);
        self.register(NATIVE_HEX_DECODE, shim_hex_decode);
        self.register_name("auto.hex.encode", NATIVE_HEX_ENCODE);
        self.register_name("auto.hex.decode", NATIVE_HEX_DECODE);

        // Plan 212 Phase 2.3: sha2 opaque struct shims
        self.register(NATIVE_SHA2_SHA256_NEW, shim_sha2_sha256_new);
        self.register(NATIVE_SHA2_UPDATE, shim_sha2_update);
        self.register(NATIVE_SHA2_FINALIZE, shim_sha2_finalize);
        self.register(NATIVE_SHA2_DROP, shim_sha2_drop);
        self.register_name("auto.sha2_opaque.sha256_new", NATIVE_SHA2_SHA256_NEW);
        self.register_name("auto.sha2_opaque.update", NATIVE_SHA2_UPDATE);
        self.register_name("auto.sha2_opaque.finalize", NATIVE_SHA2_FINALIZE);
        self.register_name("auto.sha2_opaque.drop", NATIVE_SHA2_DROP);

        // Plan 212 Phase 2.3: mime_guess pure function shim
        self.register(NATIVE_MIME_FROM_PATH, shim_mime_from_path);
        self.register_name("auto.mime.from_path", NATIVE_MIME_FROM_PATH);

        // Plan 240 VM-1: Math shims for f64 methods
        self.register(NATIVE_MATH_SIN, shim_math_sin);
        self.register(NATIVE_MATH_COS, shim_math_cos);
        self.register(NATIVE_MATH_TAN, shim_math_tan);
        // sqrt registered via #[rust_fn("Math.sqrt")] in ffi/stdlib.rs
        self.register(NATIVE_MATH_ABS_F, shim_math_abs_f);
        self.register(NATIVE_MATH_FLOOR, shim_math_floor);
        self.register(NATIVE_MATH_CEIL, shim_math_ceil);
        self.register(NATIVE_MATH_ROUND, shim_math_round);
        self.register(NATIVE_MATH_POW, shim_math_pow);
        self.register(NATIVE_MATH_POWF, shim_math_powf);
        self.register(NATIVE_MATH_POWI, shim_math_powi);
        self.register(NATIVE_MATH_EXP, shim_math_exp);
        self.register(NATIVE_MATH_LN, shim_math_ln);
        self.register(NATIVE_MATH_LOG2, shim_math_log2);
        self.register(NATIVE_MATH_LOG10, shim_math_log10);
        self.register(NATIVE_MATH_SIGNUM, shim_math_signum);
        self.register(NATIVE_MATH_ASIN, shim_math_asin);
        self.register(NATIVE_MATH_ACOS, shim_math_acos);
        self.register(NATIVE_MATH_ATAN, shim_math_atan);
        self.register(NATIVE_MATH_ATAN2, shim_math_atan2);
        self.register(NATIVE_MATH_TO_RADIANS, shim_math_to_radians);
        self.register(NATIVE_MATH_TO_DEGREES, shim_math_to_degrees);
        self.register_name("auto.math.sin", NATIVE_MATH_SIN);
        self.register_name("auto.math.cos", NATIVE_MATH_COS);
        self.register_name("auto.math.tan", NATIVE_MATH_TAN);
        self.register_name("auto.math.sqrt", NATIVE_MATH_SQRT);
        self.register_name("auto.math.abs_f", NATIVE_MATH_ABS_F);
        self.register_name("auto.math.floor", NATIVE_MATH_FLOOR);
        self.register_name("auto.math.ceil", NATIVE_MATH_CEIL);
        self.register_name("auto.math.round", NATIVE_MATH_ROUND);
        self.register_name("auto.math.pow", NATIVE_MATH_POW);
        self.register_name("auto.math.powf", NATIVE_MATH_POWF);
        self.register_name("auto.math.powi", NATIVE_MATH_POWI);
        self.register_name("auto.math.exp", NATIVE_MATH_EXP);
        self.register_name("auto.math.ln", NATIVE_MATH_LN);
        self.register_name("auto.math.log2", NATIVE_MATH_LOG2);
        self.register_name("auto.math.log10", NATIVE_MATH_LOG10);
        self.register_name("auto.math.signum", NATIVE_MATH_SIGNUM);
        self.register_name("auto.math.asin", NATIVE_MATH_ASIN);
        self.register_name("auto.math.acos", NATIVE_MATH_ACOS);
        self.register_name("auto.math.atan", NATIVE_MATH_ATAN);
        self.register_name("auto.math.atan2", NATIVE_MATH_ATAN2);
        self.register_name("auto.math.to_radians", NATIVE_MATH_TO_RADIANS);
        self.register_name("auto.math.to_degrees", NATIVE_MATH_TO_DEGREES);

        // Plan 240: Instant opaque shims
        self.register(NATIVE_INSTANT_NOW, shim_instant_now);
        self.register(NATIVE_INSTANT_ELAPSED, shim_instant_elapsed);
        self.register_name("auto.time.instant_now", NATIVE_INSTANT_NOW);
        self.register_name("auto.time.instant_elapsed", NATIVE_INSTANT_ELAPSED);

        // Plan 240: OnceCell opaque shims
        self.register(NATIVE_ONCE_NEW, shim_once_new);
        self.register(NATIVE_ONCE_SET, shim_once_set);
        self.register(NATIVE_ONCE_GET, shim_once_get);
        self.register_name("auto.cell.once_new", NATIVE_ONCE_NEW);
        self.register_name("auto.cell.once_set", NATIVE_ONCE_SET);
        self.register_name("auto.cell.once_get", NATIVE_ONCE_GET);

        // Plan 240: File I/O opaque shims
        self.register(NATIVE_FILE_CREATE_HANDLE, shim_file_create_handle);
        self.register(NATIVE_FILE_OPEN_HANDLE, shim_file_open_handle);
        self.register(NATIVE_FILE_WRITE_HANDLE, shim_file_write_handle);
        self.register(NATIVE_FILE_TRY_CLONE, shim_file_try_clone);
        self.register_name("auto.file.create_handle", NATIVE_FILE_CREATE_HANDLE);
        self.register_name("auto.file.open_handle", NATIVE_FILE_OPEN_HANDLE);
        self.register_name("auto.file.write_handle", NATIVE_FILE_WRITE_HANDLE);
        self.register_name("auto.file.try_clone", NATIVE_FILE_TRY_CLONE);
    }
}

pub const NATIVE_PRINT_I32: u16 = 1;
pub const NATIVE_PRINT_F32: u16 = 2;
pub const NATIVE_PRINT_F64: u16 = 4;
pub const NATIVE_PRINT_STR: u16 = 3;
pub const NATIVE_WRITE_STR: u16 = 2900;
pub const NATIVE_ASSERT: u16 = 4;
pub const NATIVE_ASSERT_EQ: u16 = 5;
pub const NATIVE_ASSERT_NE: u16 = 6;
pub const NATIVE_RUNTIME_PANIC: u16 = 7;

// ============================================================================
// Plan 178: Bit Operation Shims
// ============================================================================

/// int.and(mask) — Bitwise AND: val & mask
/// Stack: [self, mask] -> result
fn shim_int_and(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let mask = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val & mask);
    Ok(())
}

/// int.or(mask) — Bitwise OR: val | mask
fn shim_int_or(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let mask = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val | mask);
    Ok(())
}

/// int.xor(mask) — Bitwise XOR: val ^ mask
fn shim_int_xor(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let mask = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val ^ mask);
    Ok(())
}

/// int.not() — Bitwise NOT: ~val
/// Stack: [self] -> result
fn shim_int_not(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(!val);
    Ok(())
}

/// int.shl(n) — Logical left shift: val << n
fn shim_int_shl(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.wrapping_shl(n as u32));
    Ok(())
}

/// int.shr(n) — Logical right shift: val >> n (unsigned)
fn shim_int_shr(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32((val as u32).wrapping_shr(n as u32) as i32);
    Ok(())
}

/// int.sar(n) — Arithmetic right shift: val >> n (preserves sign)
fn shim_int_sar(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.wrapping_shr(n as u32));
    Ok(())
}

/// int.rol(n) — Rotate left
fn shim_int_rol(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.rotate_left(n as u32));
    Ok(())
}

/// int.ror(n) — Rotate right
fn shim_int_ror(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.rotate_right(n as u32));
    Ok(())
}

/// int.count_ones() — Population count
fn shim_int_count_ones(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.count_ones() as i32);
    Ok(())
}

/// int.leading_zeros() — Count leading zeros
fn shim_int_leading_zeros(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.leading_zeros() as i32);
    Ok(())
}

/// int.trailing_zeros() — Count trailing zeros
fn shim_int_trailing_zeros(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.trailing_zeros() as i32);
    Ok(())
}

/// int.flip() — Bit-reverse (mirror all bits)
fn shim_int_bitrev(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.reverse_bits());
    Ok(())
}

// === Phase 4: Dynamic bitfield views ===

/// int.bits_read(start, len) — Read bitfield: (val >> start) & ((1 << len) - 1)
fn shim_int_bit_read(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let len = task.ram.pop_i32();
    let start = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    let mask = if len >= 32 { -1 } else { (1i32 << len) - 1 };
    task.ram.push_i32((val.wrapping_shr(start as u32)) & mask);
    Ok(())
}

/// int.bit_test(n) — Test bit n: (val >> n) & 1 → bool (1 or 0)
fn shim_int_bit_test(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    let result = (val.wrapping_shr(n as u32)) & 1;
    task.ram.push_i32(result);
    Ok(())
}

/// int.bit_on(n) — Set bit n: val | (1 << n)
fn shim_int_bit_on(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val | (1 << n));
    Ok(())
}

/// int.bit_off(n) — Clear bit n: val & !(1 << n)
fn shim_int_bit_off(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val & !(1 << n));
    Ok(())
}

/// int.bit_flip(n) — Toggle bit n: val ^ (1 << n)
fn shim_int_bit_flip(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val ^ (1 << n));
    Ok(())
}

// ============================================================================
// Plan 240 VM-1: Math shims for f64 methods (sin/cos/tan/sqrt/pow etc.)
// ============================================================================

macro_rules! math_unary_shim {
    ($name:ident, $method:ident) => {
        pub fn $name(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
            let n = task.ram.pop_f64();
            task.ram.push_f64(n.$method());
            Ok(())
        }
    };
}

macro_rules! math_binary_shim {
    ($name:ident, $method:ident) => {
        pub fn $name(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
            let exp = task.ram.pop_f64();
            let base = task.ram.pop_f64();
            task.ram.push_f64(base.$method(exp));
            Ok(())
        }
    };
}

math_unary_shim!(shim_math_sin, sin);
math_unary_shim!(shim_math_cos, cos);
math_unary_shim!(shim_math_tan, tan);
// sqrt — registered in native.rs but shim lives in ffi/stdlib.rs with #[rust_fn]
// (needed because #[rust_fn] uses VMConvertible which handles nanbox f64 encoding)
math_unary_shim!(shim_math_abs_f, abs);
math_unary_shim!(shim_math_floor, floor);
math_unary_shim!(shim_math_ceil, ceil);
math_unary_shim!(shim_math_round, round);
math_unary_shim!(shim_math_exp, exp);
math_unary_shim!(shim_math_ln, ln);
math_unary_shim!(shim_math_log2, log2);
math_unary_shim!(shim_math_log10, log10);
math_unary_shim!(shim_math_signum, signum);
math_unary_shim!(shim_math_asin, asin);
math_unary_shim!(shim_math_acos, acos);
math_unary_shim!(shim_math_atan, atan);
math_unary_shim!(shim_math_to_radians, to_radians);
math_unary_shim!(shim_math_to_degrees, to_degrees);
math_binary_shim!(shim_math_pow, powf);
math_binary_shim!(shim_math_powf, powf);

pub fn shim_math_powi(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let exp = task.ram.pop_i32();
    let base = task.ram.pop_f64();
    task.ram.push_f64(base.powi(exp));
    Ok(())
}

pub fn shim_math_atan2(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let x = task.ram.pop_f64();
    let y = task.ram.pop_f64();
    task.ram.push_f64(y.atan2(x));
    Ok(())
}

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
pub const NATIVE_LIST_RESERVE: u16 = 118;

// List higher-order functions (Plan 206)
pub const NATIVE_LIST_MAP: u16 = 2060;
pub const NATIVE_LIST_FILTER: u16 = 2061;
pub const NATIVE_LIST_FOREACH: u16 = 2062;
pub const NATIVE_LIST_FIND: u16 = 2063;
pub const NATIVE_LIST_ANY: u16 = 2064;
pub const NATIVE_LIST_ALL: u16 = 2065;
pub const NATIVE_LIST_REDUCE: u16 = 2066;
pub const NATIVE_LIST_SORT: u16 = 2067;
pub const NATIVE_LIST_SORT_BY: u16 = 2068;
pub const NATIVE_LIST_JOIN: u16 = 2080;
pub const NATIVE_LIST_CONTAINS: u16 = 2069;

// === Result HOF Native Functions ===
// Plan 200 Task 3.3: .map_err() closure callback
pub const NATIVE_RESULT_MAP_ERR: u16 = 2070;

// === Iterator Native Functions (111+) ===
pub const NATIVE_LIST_ITER: u16 = 111;
pub const NATIVE_ITERATOR_NEXT: u16 = 112;
pub const NATIVE_ITERATOR_MAP: u16 = 113;
pub const NATIVE_ITERATOR_FILTER: u16 = 114;
pub const NATIVE_ITERATOR_COLLECT: u16 = 115;
pub const NATIVE_ITERATOR_REDUCE: u16 = 116;
pub const NATIVE_ITERATOR_FIND: u16 = 117;
pub const NATIVE_ITERATOR_ENUMERATE: u16 = 118;

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
pub const NATIVE_HASHMAP_IS_EMPTY: u16 = 1290;
pub const NATIVE_HASHMAP_GET_OR: u16 = 1291;
pub const NATIVE_HASHMAP_KEYS: u16 = 1292;

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
pub const NATIVE_STRING_FROM: u16 = 176;  // Plan 155: String.from(str) -> String
pub const NATIVE_STRING_NEW: u16 = 177;       // String.new() -> sb_id
pub const NATIVE_STRING_PUSH: u16 = 178;      // s.push(char) -> 0
pub const NATIVE_STRING_POP: u16 = 179;       // s.pop() -> char_codepoint (0 if empty)
pub const NATIVE_STRING_GET: u16 = 180;       // s.get(index) -> char_codepoint
pub const NATIVE_STRING_SET: u16 = 181;       // s.set(index, char) -> 0
pub const NATIVE_STRING_INSERT: u16 = 182;    // s.insert(index, char) -> 0
pub const NATIVE_STRING_REMOVE: u16 = 183;    // s.remove(index) -> char_codepoint
pub const NATIVE_STRING_CLEAR: u16 = 184;     // s.clear() -> 0
pub const NATIVE_STRING_IS_EMPTY: u16 = 185;  // s.is_empty() -> bool (1/0)
pub const NATIVE_STRING_RESERVE: u16 = 186;   // s.reserve(n) -> 0

// === Memory Allocation Native IDs (190+) ===
pub const NATIVE_ALLOC_ARRAY: u16 = 190;
pub const NATIVE_REALLOC_ARRAY: u16 = 191;
pub const NATIVE_FREE_ARRAY: u16 = 192;

// === Storage Native IDs (195+) ===
pub const NATIVE_HEAP_NEW: u16 = 195;
pub const NATIVE_HEAP_CAPACITY: u16 = 196;
pub const NATIVE_HEAP_TRY_GROW: u16 = 197;
pub const NATIVE_HEAP_DROP: u16 = 198;
pub const NATIVE_INLINE_INT64_NEW: u16 = 199;
pub const NATIVE_INLINE_INT64_CAPACITY: u16 = 200;
pub const NATIVE_INLINE_INT64_TRY_GROW: u16 = 201;
pub const NATIVE_INLINE_INT64_DROP: u16 = 202;

// === List Extra Native IDs (205+) ===
pub const NATIVE_LIST_CAPACITY: u16 = 205;

// === Bit Operation Native IDs (210+) — Plan 178 ===
pub const NATIVE_INT_AND: u16 = 210;
pub const NATIVE_INT_OR: u16 = 211;
pub const NATIVE_INT_XOR: u16 = 212;
pub const NATIVE_INT_NOT: u16 = 213;
pub const NATIVE_INT_SHL: u16 = 214;
pub const NATIVE_INT_SHR: u16 = 215;
pub const NATIVE_INT_SAR: u16 = 216;
pub const NATIVE_INT_ROL: u16 = 217;
pub const NATIVE_INT_ROR: u16 = 218;

// === Bit Scan Native IDs (220+) — Plan 178 ===
pub const NATIVE_INT_COUNT_ONES: u16 = 220;
pub const NATIVE_INT_LEADING_ZEROS: u16 = 221;
pub const NATIVE_INT_TRAILING_ZEROS: u16 = 222;
pub const NATIVE_INT_BITREV: u16 = 223;

// Phase 4: Dynamic bitfield views
pub const NATIVE_INT_BIT_READ: u16 = 230;   // .bits(start, len).read()
pub const NATIVE_INT_BIT_TEST: u16 = 231;   // .bit(n).test() → bool
pub const NATIVE_INT_BIT_ON: u16 = 232;     // .bit(n).on() → val | (1 << n)
pub const NATIVE_INT_BIT_OFF: u16 = 233;    // .bit(n).off() → val & !(1 << n)
pub const NATIVE_INT_BIT_FLIP: u16 = 234;   // .bit(n).flip() → val ^ (1 << n)

// === String/Uint Extension Native IDs (235+) ===
pub const NATIVE_STR_BYTES: u16 = 235;    // str.bytes() → iterator of byte values
pub const NATIVE_UINT_TO_HEX: u16 = 236; // uint.to_hex(pad) → hex string

// === String Method Native IDs (1504+) ===
pub const NATIVE_STR_CONTAINS: u16 = 1504;
pub const NATIVE_STR_STARTS_WITH: u16 = 1505;
pub const NATIVE_STR_ENDS_WITH: u16 = 1506;
pub const NATIVE_STR_TO_INT: u16 = 1516;

// === Math Native IDs (1700+) — Plan 240 VM-1 ===
pub const NATIVE_MATH_ABS: u16 = 1700;
pub const NATIVE_MATH_MIN: u16 = 1701;
pub const NATIVE_MATH_MAX: u16 = 1702;
pub const NATIVE_MATH_SQRT: u16 = 1750;  // Changed from 1703 to avoid conflict
pub const NATIVE_MATH_FLOOR: u16 = 1710;
pub const NATIVE_MATH_CEIL: u16 = 1711;
pub const NATIVE_MATH_ROUND: u16 = 1712;
pub const NATIVE_MATH_POW: u16 = 1713;
pub const NATIVE_MATH_MIN_F: u16 = 1714;
pub const NATIVE_MATH_MAX_F: u16 = 1715;
pub const NATIVE_MATH_SIN: u16 = 1716;
pub const NATIVE_MATH_COS: u16 = 1717;
pub const NATIVE_MATH_TAN: u16 = 1718;
pub const NATIVE_MATH_EXP: u16 = 1719;
pub const NATIVE_MATH_LN: u16 = 1720;
pub const NATIVE_MATH_LOG2: u16 = 1721;
pub const NATIVE_MATH_LOG10: u16 = 1722;
pub const NATIVE_MATH_ABS_F: u16 = 1723;
pub const NATIVE_MATH_SIGNUM: u16 = 1724;
pub const NATIVE_MATH_CLAMP: u16 = 1725;
pub const NATIVE_MATH_ASIN: u16 = 1726;
pub const NATIVE_MATH_ACOS: u16 = 1727;
pub const NATIVE_MATH_ATAN: u16 = 1728;
pub const NATIVE_MATH_ATAN2: u16 = 1729;
pub const NATIVE_MATH_POWI: u16 = 1730;
pub const NATIVE_MATH_POWF: u16 = 1731;
pub const NATIVE_MATH_TO_RADIANS: u16 = 1732;
pub const NATIVE_MATH_TO_DEGREES: u16 = 1733;

// === Instant Native IDs (1203+) — Plan 240 ===
pub const NATIVE_INSTANT_NOW: u16 = 1203;
pub const NATIVE_INSTANT_ELAPSED: u16 = 1204;

// === OnceCell Native IDs (2850+) — Plan 240 ===
pub const NATIVE_ONCE_NEW: u16 = 2850;
pub const NATIVE_ONCE_SET: u16 = 2851;
pub const NATIVE_ONCE_GET: u16 = 2852;

// === File I/O Opaque Native IDs (1010+) — Plan 240 ===
pub const NATIVE_FILE_CREATE_HANDLE: u16 = 1010;
pub const NATIVE_FILE_OPEN_HANDLE: u16 = 1011;
pub const NATIVE_FILE_WRITE_HANDLE: u16 = 1012;
pub const NATIVE_FILE_TRY_CLONE: u16 = 1013;

// === Rand Native IDs (1850+) — Plan 212 Phase 2 ===
pub const NATIVE_RAND_THREAD_RNG: u16 = 1850; // thread_rng() → opaque Rng handle
pub const NATIVE_RNG_GEN_RANGE: u16 = 1851;   // rng.gen_range(lo, hi) → i32
pub const NATIVE_RNG_GEN: u16 = 1852;         // rng.gen() → i32
pub const NATIVE_RNG_DROP: u16 = 1853;        // drop Rng handle
pub const NATIVE_RAND_RANDOM: u16 = 1854;     // rand::random() → i32

pub const NATIVE_LOG_NOOP: u16 = 1804;        // no-op for env_logger.init(), etc.

// === Regex Opaque Shims (2450+) — Plan 212 Phase 2.2 ===
pub const NATIVE_RE_OPAQUE_NEW: u16 = 2450;
pub const NATIVE_RE_OPAQUE_IS_MATCH: u16 = 2451;
pub const NATIVE_RE_OPAQUE_FIND: u16 = 2452;
pub const NATIVE_RE_OPAQUE_FIND_ALL: u16 = 2453;
pub const NATIVE_RE_OPAQUE_REPLACE_ALL: u16 = 2454;
pub const NATIVE_RE_OPAQUE_CAPTURES: u16 = 2455;
pub const NATIVE_RE_OPAQUE_DROP: u16 = 2459;

// === Url Opaque Shims (2500+) — Plan 212 Phase 2.2 ===
pub const NATIVE_URL_OPAQUE_PARSE: u16 = 2500;
pub const NATIVE_URL_OPAQUE_SCHEME: u16 = 2501;
pub const NATIVE_URL_OPAQUE_HOST_STR: u16 = 2502;
pub const NATIVE_URL_OPAQUE_PATH: u16 = 2503;
pub const NATIVE_URL_OPAQUE_FRAGMENT: u16 = 2504;
pub const NATIVE_URL_OPAQUE_PORT: u16 = 2505;
pub const NATIVE_URL_OPAQUE_QUERY_PAIRS: u16 = 2506;
pub const NATIVE_URL_OPAQUE_JOIN: u16 = 2507;
pub const NATIVE_URL_OPAQUE_ORIGIN: u16 = 2508;
pub const NATIVE_URL_OPAQUE_DROP: u16 = 2509;

// === Semver Opaque Shims (2600+) — Plan 212 Phase 2.2 ===
pub const NATIVE_SEMVER_OPAQUE_PARSE: u16 = 2600;
pub const NATIVE_SEMVER_OPAQUE_MAJOR: u16 = 2601;
pub const NATIVE_SEMVER_OPAQUE_MINOR: u16 = 2602;
pub const NATIVE_SEMVER_OPAQUE_PATCH: u16 = 2603;
pub const NATIVE_SEMVER_OPAQUE_PRE: u16 = 2604;
pub const NATIVE_SEMVER_OPAQUE_TO_STRING: u16 = 2605;
pub const NATIVE_SEMVER_OPAQUE_CMP_GT: u16 = 2606;
pub const NATIVE_SEMVER_OPAQUE_DROP: u16 = 2609;

// Plan 212 Phase 2.3: chrono opaque struct shims (2700-2709)
pub const NATIVE_CHRONO_LOCAL_NOW: u16 = 2700;
pub const NATIVE_CHRONO_YEAR: u16 = 2701;
pub const NATIVE_CHRONO_MONTH: u16 = 2702;
pub const NATIVE_CHRONO_DAY: u16 = 2703;
pub const NATIVE_CHRONO_HOUR: u16 = 2704;
pub const NATIVE_CHRONO_MINUTE: u16 = 2705;
pub const NATIVE_CHRONO_SECOND: u16 = 2706;
pub const NATIVE_CHRONO_TIMESTAMP: u16 = 2707;
pub const NATIVE_CHRONO_FORMAT: u16 = 2708;
pub const NATIVE_CHRONO_DROP: u16 = 2709;

// Plan 212 Phase 2.3: base64 pure function shims (2710-2719)
pub const NATIVE_BASE64_ENCODE: u16 = 2710;
pub const NATIVE_BASE64_DECODE: u16 = 2711;

// Plan 212 Phase 2.3: hex pure function shims (2720-2729)
pub const NATIVE_HEX_ENCODE: u16 = 2720;
pub const NATIVE_HEX_DECODE: u16 = 2721;

// Plan 212 Phase 2.3: sha2 opaque struct shims (2730-2739)
pub const NATIVE_SHA2_SHA256_NEW: u16 = 2730;
pub const NATIVE_SHA2_UPDATE: u16 = 2731;
pub const NATIVE_SHA2_FINALIZE: u16 = 2732;
pub const NATIVE_SHA2_DROP: u16 = 2739;

// Plan 212 Phase 2.3: mime_guess pure function shim (2740-2749)
pub const NATIVE_MIME_FROM_PATH: u16 = 2740;

// === Standard Shims ===

/// Plan 177: Helper to print output, captures to buffer if present
fn vm_print(vm: &AutoVM, s: &str) {
    if let Some(ref buf) = vm.output_buffer {
        let mut guard = buf.write().unwrap();
        guard.push_str(s);
        guard.push('\n');
    } else {
        println!("{}", s);
    }
}

/// Helper for write() — same as vm_print but without trailing newline
fn vm_write(vm: &AutoVM, s: &str) {
    if let Some(ref buf) = vm.output_buffer {
        let mut guard = buf.write().unwrap();
        guard.push_str(s);
    } else {
        print!("{}", s);
    }
}

/// Generic print that handles any value type.
/// If the value is a tagged string index (negative), prints the string.
/// Otherwise prints as an integer.
pub fn shim_print(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    {
        let val = task.ram.pop_i32();
        if val < 0 {
            let str_index = ((-val) - 1) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else {
            vm_print(vm, &val.to_string());
        }
    }
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else {
            vm_print(vm, &auto_val::decode_i32(nv).to_string());
        }
    }
    Ok(())
}

pub fn shim_print_i32(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    // Boolean sentinel values: i32::MIN = true, i32::MIN+1 = false
    if val == -2147483648 {
        vm_print(vm, "1");
        return Ok(());
    } else if val == -2147483647 {
        vm_print(vm, "0");
        return Ok(());
    }
    // Check if it's a Rust stdlib heap handle
    let handle = val as u64;
    if let Some(obj) = vm.get_heap_object(handle) {
        let guard = obj.read().unwrap();
        if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            vm_print(vm, &format_rust_stdlib_obj(rust_obj));
            return Ok(());
        }
    }
    vm_print(vm, &val.to_string());
    Ok(())
}

pub fn shim_print_f32(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        // In nanbox mode, f64 occupies 2 slots (value at sp-2, marker at sp-1).
        // Check sp-2 for f64 bits, then sp-1 for f32.
        if task.ram.sp >= 2 {
            let nv = task.ram.raw_nv[task.ram.sp - 2];
            if auto_val::is_f64(nv) {
                task.ram.sp -= 2; // consume both slots
                let val = f64::from_bits(nv);
                vm_print(vm, &val.to_string());
                return Ok(());
            }
        }
        if task.ram.sp >= 1 {
            let nv = task.ram.raw_nv[task.ram.sp - 1];
            if auto_val::is_f32(nv) {
                task.ram.sp -= 1;
                let val = auto_val::decode_f32(nv);
                vm_print(vm, &val.to_string());
                return Ok(());
            }
        }
    }
    let val_bits = task.ram.pop_i32() as u32;
    let val = f32::from_bits(val_bits);
    vm_print(vm, &val.to_string());
    Ok(())
}

/// Print an f64 value (2 slots on stack).
pub fn shim_print_f64(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_f64();
    vm_print(vm, &val.to_string());
    Ok(())
}

/// Format a RustStdlibObject for display.
fn format_rust_stdlib_obj(obj: &RustStdlibObject) -> String {
    match obj.type_name.as_str() {
        "Instant" => "<Instant>".to_string(),
        "Duration" => {
            if let Some(dur) = obj.downcast_ref::<std::time::Duration>() {
                format!("{}ms", dur.as_millis())
            } else {
                "<Duration>".to_string()
            }
        }
        "PathBuf" => {
            if let Some(p) = obj.downcast_ref::<std::path::PathBuf>() {
                format!("{}", p.display())
            } else {
                "<PathBuf>".to_string()
            }
        }
        "Arc" => "<Arc>".to_string(),
        "Mutex" => "<Mutex>".to_string(),
        "Box" => "<Box>".to_string(),
        "RefCell" => "<RefCell>".to_string(),
        "std::cell::OnceCell::Value" => {
            if let Some(s) = obj.downcast_ref::<String>() {
                s.clone()
            } else {
                "<OnceCell::Value>".to_string()
            }
        }
        other => format!("<{}>", other),
    }
}

/// Print a string from the string constant pool, or an integer if not a string.
/// Expects tagged string index on TOS (LOAD_STR pushes -(idx+1)).
/// If the value is non-negative and not a valid string index, prints as integer.
pub fn shim_print_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    {
        let tagged = task.ram.pop_i32();
        let str_index = if tagged < 0 {
            ((-tagged) - 1) as u16
        } else {
            tagged as u16
        };
        if tagged < 0 {
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else {
            let handle = tagged as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_print(vm, &tagged.to_string());
                }
            } else {
                vm_print(vm, &tagged.to_string());
            }
        }
    }
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            let handle = val as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_print(vm, &val.to_string());
                }
            } else {
                vm_print(vm, &val.to_string());
            }
        }
    }
    Ok(())
}

/// Write a string without trailing newline (write() in Auto).
/// Same logic as shim_print_str but uses vm_write instead of vm_print.
pub fn shim_write_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    {
        let tagged = task.ram.pop_i32();
        let str_index = if tagged < 0 {
            ((-tagged) - 1) as u16
        } else {
            tagged as u16
        };
        if tagged < 0 {
            if let Some(bytes) = vm.get_string(str_index) {
                vm_write(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_write(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else {
            let handle = tagged as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_write(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_write(vm, &tagged.to_string());
                }
            } else {
                vm_write(vm, &tagged.to_string());
            }
        }
    }
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_write(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_write(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            let handle = val as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_write(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_write(vm, &val.to_string());
                }
            } else {
                vm_write(vm, &val.to_string());
            }
        }
    }
    Ok(())
}

pub fn shim_assert(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        let is_true = if auto_val::is_bool(nv) {
            auto_val::decode_bool(nv)
        } else if auto_val::is_i32(nv) {
            let v = auto_val::decode_i32(nv);
            v != 0 && v != (-2147483647i32)
        } else {
            true
        };
        if !is_true {
            return Err(VMError::RuntimeError("Assertion failed".to_string()));
        }
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let cond = task.ram.pop_i32();
        if cond == 0 || cond == -2147483647 {
            return Err(VMError::RuntimeError("Assertion failed".to_string()));
        }
    }
    Ok(())
}

/// Runtime panic: pops a string (tagged index) from stack and returns it as an error.
/// Used by #[vm] function stubs when no matching native implementation is found.
pub fn shim_runtime_panic(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    let msg = {
        let tagged = task.ram.pop_i32();
        if tagged < 0 {
            let idx = ((-tagged) - 1) as u16;
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_else(|| format!("Runtime panic (invalid string index {})", idx))
        } else {
            format!("Runtime panic (unexpected stack value: {})", tagged)
        }
    };
    #[cfg(feature = "nanbox")]
    let msg = {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as u16;
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_else(|| format!("Runtime panic (invalid string index {})", idx))
        } else {
            format!("Runtime panic (unexpected stack value: {:?})", nv)
        }
    };
    Err(VMError::RuntimeError(msg))
}

pub fn shim_assert_eq(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    {
        let right = task.ram.pop_i32();
        let left = task.ram.pop_i32();

        let equal = if left < 0 && right < 0 {
            // Both are tagged string indices — compare actual string contents
            let left_str = vm.get_string(decode_str_idx(left) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm.get_string(decode_str_idx(right) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            left_str.as_deref() == right_str.as_deref()
        } else {
            left == right
        };

        if !equal {
            return Err(VMError::RuntimeError(
                format!("Assertion failed: {} != {}", left, right)
            ));
        }
        Ok(())
    }

    #[cfg(feature = "nanbox")]
    {
        let right_nv = task.ram.pop_nv();
        let left_nv = task.ram.pop_nv();

        let left_is_str = auto_val::is_string(left_nv);
        let right_is_str = auto_val::is_string(right_nv);

        let equal = if left_is_str && right_is_str {
            let left_str = vm.get_string(auto_val::decode_string(left_nv) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm.get_string(auto_val::decode_string(right_nv) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            left_str.as_deref() == right_str.as_deref()
        } else if left_nv == right_nv {
            true
        } else if auto_val::is_i32(left_nv) && auto_val::is_i32(right_nv) {
            auto_val::decode_i32(left_nv) == auto_val::decode_i32(right_nv)
        } else if auto_val::is_object(left_nv) && auto_val::is_object(right_nv) {
            vm.struct_eq(auto_val::decode_object(left_nv) as i32, auto_val::decode_object(right_nv) as i32)
        } else {
            false
        };

        if !equal {
            return Err(VMError::RuntimeError(
                format!("Assertion failed: {:?} != {:?}", left_nv, right_nv)
            ));
        }
        Ok(())
    }
}

pub fn shim_assert_ne(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    {
        let right = task.ram.pop_i32();
        let left = task.ram.pop_i32();

        let equal = if left < 0 && right < 0 {
            let left_str = vm.get_string(decode_str_idx(left) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm.get_string(decode_str_idx(right) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            left_str.as_deref() == right_str.as_deref()
        } else {
            left == right
        };

        if equal {
            return Err(VMError::RuntimeError(
                format!("Assertion failed: {} == {}", left, right)
            ));
        }
        Ok(())
    }

    #[cfg(feature = "nanbox")]
    {
        let right_nv = task.ram.pop_nv();
        let left_nv = task.ram.pop_nv();

        let left_is_str = auto_val::is_string(left_nv);
        let right_is_str = auto_val::is_string(right_nv);

        let equal = if left_is_str && right_is_str {
            let left_str = vm.get_string(auto_val::decode_string(left_nv) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm.get_string(auto_val::decode_string(right_nv) as u16)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            left_str.as_deref() == right_str.as_deref()
        } else if left_nv == right_nv {
            true
        } else if auto_val::is_i32(left_nv) && auto_val::is_i32(right_nv) {
            auto_val::decode_i32(left_nv) == auto_val::decode_i32(right_nv)
        } else if auto_val::is_object(left_nv) && auto_val::is_object(right_nv) {
            vm.struct_eq(auto_val::decode_object(left_nv) as i32, auto_val::decode_object(right_nv) as i32)
        } else {
            false
        };

        if equal {
            return Err(VMError::RuntimeError(
                format!("Assertion failed: {:?} == {:?}", left_nv, right_nv)
            ));
        }
        Ok(())
    }
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
/// Push a tagged value from ListData<i32> back onto the stack.
/// In non-nanbox mode, all values are plain i32 so push_i32 is fine.
/// In nanbox mode, negative values are string tags that must use push_str_idx
/// to preserve the TAG_STRING type tag in the NanoValue encoding.
#[cfg(feature = "nanbox")]
fn push_tagged_value(ram: &mut crate::vm::virt_memory::VirtualRAM, val: i32) {
    if val < 0 {
        let str_idx = (-(val) - 1) as u32;
        ram.push_nv(auto_val::encode_string(str_idx));
    } else {
        ram.push_i32(val);
    }
}

#[cfg(not(feature = "nanbox"))]
fn push_tagged_value(ram: &mut crate::vm::virt_memory::VirtualRAM, val: i32) {
    ram.push_i32(val);
}

pub fn shim_list_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    #[cfg(feature = "nanbox")]
    {
        let index_nv = task.ram.pop_nv();
        let list_nv = task.ram.pop_nv();
        let index = if auto_val::is_i32(index_nv) { auto_val::decode_i32(index_nv) as usize } else { 0usize };
        let list_id = if auto_val::is_i32(list_nv) { auto_val::decode_i32(list_nv) as u64 } else { 0u64 };

        if let Some(obj) = vm.get_heap_object(list_id) {
            let guard = obj.read().unwrap();
            if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                if let Some(&val) = list.get(index) {
                    push_tagged_value(&mut task.ram, val);
                } else {
                    task.ram.push_i32(0);
                }
                return Ok(());
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
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

/// List.reserve(list_id, additional) -> 0
/// Pre-allocate capacity for additional elements
pub fn shim_list_reserve(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let additional = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.reserve(additional);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

// ============================================================
// List Higher-Order Functions (Plan 206: Closure HOF)
// ============================================================

/// Helper: get list elements as Vec<i32> from a list heap object
fn get_list_i32_elements(vm: &AutoVM, list_id: u64) -> Result<Vec<i32>, VMError> {
    use crate::vm::types::ListData;

    // First check heap objects (List created via List.new or previous HOF)
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            return Ok(list.elems.clone());
        }
    }

    // Fallback: check vm.arrays (array literals created via CREATE_ARRAY)
    if let Some(array_ref) = vm.arrays.get(&list_id) {
        let guard = array_ref.read().unwrap();
        let elems: Vec<i32> = guard.iter().map(|v| {
            match v {
                auto_val::Value::Int(n) => *n,
                _ => 0,
            }
        }).collect();
        return Ok(elems);
    }

    Err(VMError::RuntimeError(format!("Invalid list ID: {}", list_id)))
}

/// Helper: create a new array from Vec<i32> elements, return array ID
/// Stores in vm.arrays (same as CREATE_ARRAY) so results work with ARRAY_LEN etc.
fn create_list_from_i32(vm: &AutoVM, elems: Vec<i32>) -> u64 {
    use std::sync::atomic::Ordering;

    let values: Vec<auto_val::Value> = elems.into_iter()
        .map(|e| auto_val::Value::Int(e))
        .collect();
    let new_id = vm.array_id_gen.fetch_add(1, Ordering::SeqCst);
    vm.arrays.insert(new_id, Arc::new(RwLock::new(values)));
    new_id
}

/// Helper: check if a VM value is truthy (handles both conventions)
/// True values: 1, i32::MIN (-2147483648), or any non-zero/non-false value
/// False values: 0, i32::MIN+1 (-2147483647)
#[inline]
fn vm_is_truthy(val: i32) -> bool {
    val != 0 && val != i32::MIN + 1
}

/// Helper: convert a VM value to a printable boolean (1 or 0)
#[inline]
#[allow(dead_code)]
fn vm_to_printable_bool(val: i32) -> i32 {
    if vm_is_truthy(val) { 1 } else { 0 }
}

/// List.map(closure) -> new List
/// Stack: closure_id, list_id -> result_list_id
pub fn shim_list_map(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    let mut results = Vec::with_capacity(elements.len());

    for elem in elements {
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 1)?;
        results.push(task.ram.pop_i32());
    }

    let new_id = create_list_from_i32(vm, results);
    task.ram.push_i32(new_id as i32);
    Ok(())
}

/// List.filter(closure) -> new List
/// Stack: closure_id, list_id -> result_list_id
pub fn shim_list_filter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    let mut results = Vec::new();

    for elem in elements {
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 1)?;
        let predicate = task.ram.pop_i32();
        if vm_is_truthy(predicate) {
            results.push(elem);
        }
    }

    let new_id = create_list_from_i32(vm, results);
    task.ram.push_i32(new_id as i32);
    Ok(())
}

/// List.for_each(closure) -> void
/// Stack: closure_id, list_id -> 0
pub fn shim_list_for_each(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    for elem in elements {
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 1)?;
        task.ram.pop_i32(); // Discard result
    }
    task.ram.push_i32(0);
    Ok(())
}

/// List.find(closure) -> ?T (found value or -1 for None)
/// Stack: closure_id, list_id -> result
pub fn shim_list_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    for elem in elements {
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 1)?;
        let found = task.ram.pop_i32();
        if vm_is_truthy(found) {
            task.ram.push_i32(elem);
            return Ok(());
        }
    }
    task.ram.push_i32(-1); // None
    Ok(())
}

/// List.any(closure) -> bool
/// Stack: closure_id, list_id -> bool (1/0)
pub fn shim_list_any(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    for elem in elements {
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 1)?;
        let result = task.ram.pop_i32();
        if vm_is_truthy(result) {
            task.ram.push_i32(1);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// List.all(closure) -> bool
/// Stack: closure_id, list_id -> bool (1/0)
pub fn shim_list_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    for elem in elements {
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 1)?;
        let result = task.ram.pop_i32();
        if !vm_is_truthy(result) {
            task.ram.push_i32(0);
            return Ok(());
        }
    }
    task.ram.push_i32(1);
    Ok(())
}

/// List.reduce(init, closure) -> accumulated value
/// Stack: closure_id, init_val, list_id -> result
pub fn shim_list_reduce(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let init_val = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_i32_elements(vm, list_id)?;
    let mut acc = init_val;

    for elem in elements {
        task.ram.push_i32(acc);
        task.ram.push_i32(elem);
        vm.call_closure(task, closure_id, 2)?;
        acc = task.ram.pop_i32();
    }

    task.ram.push_i32(acc);
    Ok(())
}

/// List.join(separator) -> str
/// Stack: separator (str tag), list_id -> joined_str (str tag)
/// Joins string list elements with the given separator.
pub fn shim_list_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Pop separator (string pool tag)
    #[cfg(not(feature = "nanbox"))]
    let sep_idx = decode_str_idx(task.ram.pop_i32());
    #[cfg(feature = "nanbox")]
    let sep_idx = decode_str_idx_nv(task.ram.pop_nv());
    let separator = vm.strings.read().unwrap()
        .get(sep_idx)
        .map(|b| String::from_utf8_lossy(b).to_string())
        .unwrap_or_default();

    // Pop list_id
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        // Try ListData<Value> (FFI bridge uses this for Vec<String>)
        if let Some(list) = guard.as_any().downcast_ref::<ListData<auto_val::Value>>() {
            let parts: Vec<String> = list.elems.iter().filter_map(|v| {
                if let auto_val::Value::Str(s) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            }).collect();
            let joined = parts.join(&separator);
            let str_idx = vm.add_string(joined.into_bytes());
            task.ram.push_str_idx(str_idx as u32);
            return Ok(());
        }
        // Try ListData<String>
        if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
            let joined = list.elems.join(&separator);
            let str_idx = vm.add_string(joined.into_bytes());
            task.ram.push_str_idx(str_idx as u32);
            return Ok(());
        }
        // Try ListData<i32> as fallback
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            let parts: Vec<String> = list.elems.iter().map(|e| e.to_string()).collect();
            let joined = parts.join(&separator);
            let str_idx = vm.add_string(joined.into_bytes());
            task.ram.push_str_idx(str_idx as u32);
            return Ok(());
        }
    }

    // Fallback: return empty string
    let str_idx = vm.add_string(Vec::new());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// List.contains(value) -> bool
/// Stack: value (i32), list_id -> bool (1/0)
pub fn shim_list_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;
    let elements = get_list_i32_elements(vm, list_id)?;
    let found = elements.iter().any(|&e| e == value);
    task.ram.push_i32(if found { 1 } else { 0 });
    Ok(())
}

/// Sort a list of i32 values in-place.
/// Stack: list_id -> void
pub fn shim_list_sort(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.elems.sort();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Sort a list with a comparator closure.
/// Stack: list_id, closure_id -> void
/// (Currently delegates to default sort — custom comparators not yet supported in VM)
pub fn shim_list_sort_by(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let _closure_id = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.elems.sort();
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

// ============================================================================
// Result HOF Native Shims
// ============================================================================

/// Result.map_err(closure) — if Err, call closure with error value; if Ok, pass through.
/// Stack: closure_id, result_instance_id -> new_result_instance_id
/// Plan 200 Task 3.3
pub fn shim_result_map_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    let closure_id = task.ram.pop_i32() as u32;
    let result_id = task.ram.pop_i32() as u64;

    // Look up the Result heap object
    let obj = vm.get_heap_object(result_id)
        .ok_or_else(|| VMError::RuntimeError(format!("map_err: invalid heap object {}", result_id)))?;
    let guard = obj.read().unwrap();
    let instance = guard.as_any().downcast_ref::<GenericInstanceData>()
        .ok_or_else(|| VMError::RuntimeError("map_err: not a Result heap object".into()))?;

    if instance.mono_name == "Result.Err" {
        let err_val = match instance.fields.first() {
            Some(auto_val::Value::Int(v)) => *v,
            _ => return Err(VMError::RuntimeError("map_err: Err field not an int".into())),
        };
        // Release the read lock before calling closure (which may access heap)
        drop(guard);

        // Call closure with the error value as argument
        task.ram.push_i32(err_val);
        vm.call_closure(task, closure_id, 1)?;
        let new_err_val = task.ram.pop_i32();

        // Wrap back in Result.Err heap object
        let new_instance = GenericInstanceData::new("Result.Err".to_string(), vec![auto_val::Value::Int(new_err_val)]);
        let new_id = vm.insert_heap_object(new_instance);
        task.ram.push_i32(new_id as i32);
    } else {
        // Ok — pass through unchanged
        drop(guard);
        task.ram.push_i32(result_id as i32);
    }
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
                        Iterator::Enumerate(enumerate_iter) => {
                            // Get next from source, then push index on top
                            if let Some(mut source_iter) = vm.iterators.get_mut(&enumerate_iter.source_iterator_id) {
                                match &mut *source_iter {
                                    Iterator::List(list_iter) => {
                                        if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                            let list = obj.read().unwrap();
                                            if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                                -1
                                            } else if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                                                if list_iter.current_index >= list_data.len() as u32 {
                                                    -1
                                                } else {
                                                    let elem = list_data.get(list_iter.current_index as usize).copied().unwrap_or(0);
                                                    list_iter.current_index += 1;
                                                    let idx = enumerate_iter.current_index as i32;
                                                    enumerate_iter.current_index += 1;
                                                    // Push value then index (caller pops index first)
                                                    task.ram.push_i32(elem);
                                                    task.ram.push_i32(idx);
                                                    return Ok(());
                                                }
                                            } else {
                                                -1
                                            }
                                        } else {
                                            -1
                                        }
                                    }
                                    _ => -1,
                                }
                            } else {
                                -1
                            }
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
            Iterator::Enumerate(enumerate_iter) => {
                // Plan 200 Task 3.2: Get next from source, push (index, value)
                if let Some(mut source_iter) = vm.iterators.get_mut(&enumerate_iter.source_iterator_id) {
                    match &mut *source_iter {
                        Iterator::List(list_iter) => {
                            if let Some(obj) = vm.get_heap_object(list_iter.list_id) {
                                let list = obj.read().unwrap();
                                if list.type_tag() != crate::vm::heap_object::TypeTag::ListInt {
                                    -1
                                } else if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                                    if list_iter.current_index >= list_data.len() as u32 {
                                        -1
                                    } else {
                                        let elem = list_data.get(list_iter.current_index as usize).copied().unwrap_or(0);
                                        list_iter.current_index += 1;
                                        let idx = enumerate_iter.current_index as i32;
                                        enumerate_iter.current_index += 1;
                                        // Push value then index (two values on stack)
                                        task.ram.push_i32(elem);
                                        task.ram.push_i32(idx);
                                        return Ok(());
                                    }
                                } else {
                                    -1
                                }
                            } else {
                                -1
                            }
                        }
                        _ => -1,
                    }
                } else {
                    -1
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

/// Create an enumerate adapter iterator.
/// Plan 200 Task 3.2: Wraps a source iterator, tracking index.
/// Stack: iterator_id -> new_iterator_id
pub fn shim_iterator_enumerate(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;
    use crate::vm::engine::{Iterator, EnumerateIterator};

    let source_iterator_id = task.ram.pop_i32() as u32;

    if !vm.iterators.contains_key(&source_iterator_id) {
        task.ram.push_i32(-1);
        return Ok(());
    }

    let new_iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);

    let iterator = Iterator::Enumerate(EnumerateIterator {
        source_iterator_id,
        current_index: 0,
    });

    vm.iterators.insert(new_iterator_id, iterator);
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
            Iterator::Map(_) | Iterator::Filter(_) | Iterator::Enumerate(_) => {
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
            Iterator::Map(_) | Iterator::Filter(_) | Iterator::Enumerate(_) => {
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
            Iterator::Map(_) | Iterator::Filter(_) | Iterator::Enumerate(_) => {
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
/// Insert a string key with any value type (str, int, bool, ref)
/// Stack: hashmap_id, key_str_id, value -> result (0)
///
/// The value is auto-detected: negative (but not bool sentinels) = string tag,
/// positive/zero/bool sentinels = int.
pub fn shim_hashmap_insert_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(not(feature = "nanbox"))]
    {
        let value_bits = task.ram.pop_i32();
        let key_bits = task.ram.pop_i32();
        let map_id = task.ram.pop_i32() as u64;
        if let Some(obj) = vm.get_heap_object(map_id) {
            let key_str_idx = ((-key_bits) - 1) as usize;
            let strings = vm.strings.read().unwrap();
            let key_bytes = strings.get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            let value = if value_bits < 0 && value_bits > i32::MIN + 1 {
                let str_idx = ((-value_bits) - 1) as usize;
                if let Some(bytes) = strings.get(str_idx) {
                    Value::Str(auto_val::AutoStr::from(String::from_utf8_lossy(bytes).as_ref()))
                } else { Value::Int(0) }
            } else if value_bits == i32::MIN { Value::Bool(true) }
            else if value_bits == i32::MIN + 1 { Value::Bool(false) }
            else { Value::Int(value_bits) };
            drop(strings);
            let mut guard = obj.write().unwrap();
            if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
                map.insert(key_str, value).map_err(|e| VMError::RuntimeError(e))?;
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
    #[cfg(feature = "nanbox")]
    {
        let value_nv = task.ram.pop_nv();
        let key_idx = task.ram.pop_str_idx();
        let map_id = task.ram.pop_i32() as u64;
        if let Some(obj) = vm.get_heap_object(map_id) {
            let strings = vm.strings.read().unwrap();
            let key_bytes = strings.get(key_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            let value = if auto_val::is_string(value_nv) {
                let str_idx = auto_val::decode_string(value_nv) as usize;
                if let Some(bytes) = strings.get(str_idx) {
                    Value::Str(auto_val::AutoStr::from(String::from_utf8_lossy(bytes).as_ref()))
                } else { Value::Int(0) }
            } else if auto_val::is_bool(value_nv) {
                Value::Bool(auto_val::decode_bool(value_nv))
            } else if auto_val::is_object(value_nv) {
                Value::VmRef(auto_val::VmRef { id: auto_val::decode_object(value_nv) as usize })
            } else if auto_val::is_list(value_nv) {
                Value::VmRef(auto_val::VmRef { id: auto_val::decode_list(value_nv) as usize })
            } else {
                Value::Int(auto_val::decode_i32(value_nv))
            };
            drop(strings);
            let mut guard = obj.write().unwrap();
            if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
                map.insert(key_str, value).map_err(|e| VMError::RuntimeError(e))?;
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
}

/// Insert a string key with i32 value
/// Stack: hashmap_id, key_str_id, value -> result (0)
pub fn shim_hashmap_insert_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        let value_nv = task.ram.pop_nv();
        let key_nv = task.ram.pop_nv();
        let map_nv = task.ram.pop_nv();
        // Key should be a string pool reference. In nanbox mode, the tag can be
        // TAG_STRING or (due to i32 round-trips in some code paths) TAG_I32.
        // Both use the same payload encoding (negative i32 = string pool index),
        // so decode_string works for both.
        if !auto_val::is_string(key_nv) && !auto_val::is_i32(key_nv) {
            return Err(VMError::RuntimeError(format!(
                "Invalid key for insert_int: key_nv={:#018x} tag={}", key_nv, auto_val::tag_of(key_nv)
            )));
        }
        let key_str_idx = auto_val::decode_string(key_nv) as usize;
        // Decode value: support i32, object ref, string ref, etc.
        let value = if auto_val::is_i32(value_nv) {
            Value::Int(auto_val::decode_i32(value_nv))
        } else if auto_val::is_object(value_nv) {
            // Store object reference as VmRef so get_str can restore the tag
            Value::VmRef(auto_val::VmRef { id: auto_val::decode_object(value_nv) as usize })
        } else if auto_val::is_string(value_nv) {
            // Store the string pool index as an int for later retrieval
            Value::Int(auto_val::decode_string(value_nv) as i32)
        } else if auto_val::is_list(value_nv) {
            Value::VmRef(auto_val::VmRef { id: auto_val::decode_list(value_nv) as usize })
        } else {
            Value::Int(0)
        };
        let map_id = if auto_val::is_i32(map_nv) { auto_val::decode_i32(map_nv) as u64 }
                     else if auto_val::is_object(map_nv) { auto_val::decode_object(map_nv) as u64 }
                     else { 0 };
        if let Some(obj) = vm.get_heap_object(map_id) {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            drop(key_bytes);
            let mut guard = obj.write().unwrap();
            if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
                map.insert(key_str, value)
                    .map_err(|e| VMError::RuntimeError(e))?;
            }
        }
        task.ram.push_i32(0);
        return Ok(());
    }
    #[allow(unreachable_code)]
    {
    let value = task.ram.pop_i32();
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

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
}

/// Get value by string key
/// Stack: hashmap_id, key_str_id -> value (0 if not found)
pub fn shim_hashmap_get_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let key_str_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            // Get the value from map
            if let Some(value) = map.get(&key_str) {
                match value {
                    auto_val::Value::Int(i) => {
                        task.ram.push_i32(i);
                        return Ok(());
                    }
                    auto_val::Value::Uint(u) => {
                        task.ram.push_i32(u as i32);
                        return Ok(());
                    }
                    auto_val::Value::Bool(b) => {
                        task.ram.push_i32(if b { 1 } else { 0 });
                        return Ok(());
                    }
                    auto_val::Value::Str(s) => {
                        let mut strings = vm.strings.write().unwrap();
                        let str_idx = strings.len() as u32;
                        strings.push(s.as_bytes().to_vec());
                        drop(strings);
                        task.ram.push_str_idx(str_idx);
                        return Ok(());
                    }
                    auto_val::Value::VmRef(vm_ref) => {
                        #[cfg(feature = "nanbox")]
                        {
                            task.ram.push_nv(auto_val::encode_object(vm_ref.id as u32));
                        }
                        #[cfg(not(feature = "nanbox"))]
                        {
                            task.ram.push_i32(vm_ref.id as i32);
                        }
                        return Ok(());
                    }
                    _ => {
                        // Unsupported value type — push nil
                    }
                }
            }
        }
    }

    // Not found — push nil marker
    #[cfg(feature = "nanbox")]
    {
        task.ram.push_nv(auto_val::encode_null());
    }
    #[cfg(not(feature = "nanbox"))]
    {
        task.ram.push_i32(i32::MIN + 1);
    }
    Ok(())
}

/// Get value by string key (returns i32)
/// Stack: hashmap_id, key_str_id -> value (or nil if not found)
pub fn shim_hashmap_get_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid key string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            drop(key_bytes);

            if let Some(value) = map.get(&key_str) {
                if let auto_val::Value::Int(i) = value {
                    task.ram.push_i32(i);
                    return Ok(());
                }
            }
        }
    }

    // Not found — push nil marker
    #[cfg(feature = "nanbox")]
    {
        task.ram.push_nv(auto_val::encode_null());
    }
    #[cfg(not(feature = "nanbox"))]
    {
        task.ram.push_i32(i32::MIN + 1);
    }
    Ok(())
}

/// Check if key exists
/// Stack: hashmap_id, key_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashmap_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {

    let key_str_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

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

    let key_str_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let mut guard = obj.write().unwrap();
        if let Some(map) = guard.as_any_mut().downcast_mut::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if let Some(value) = map.remove(&key_str) {
                match value {
                    auto_val::Value::Int(i) => {
                        task.ram.push_i32(i);
                        return Ok(());
                    }
                    auto_val::Value::Str(s) => {
                        let mut strings = vm.strings.write().unwrap();
                        let str_idx = strings.len() as u32;
                        strings.push(s.as_bytes().to_vec());
                        drop(strings);
                        task.ram.push_str_idx(str_idx);
                        return Ok(());
                    }
                    _ => {
                        task.ram.push_i32(0);
                        return Ok(());
                    }
                }
            }
        }
    }

    // Not found — push nil marker
    #[cfg(feature = "nanbox")]
    {
        task.ram.push_nv(auto_val::encode_null());
    }
    #[cfg(not(feature = "nanbox"))]
    {
        task.ram.push_i32(i32::MIN + 1);
    }
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

/// Check if HashMap is empty
/// Stack: hashmap_id -> bool (1 if empty, 0 if not)
pub fn shim_hashmap_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    let is_empty = if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            map.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    task.ram.push_i32(if is_empty { 1 } else { 0 });
    Ok(())
}

/// Get value from HashMap with default fallback
/// Stack: hashmap_id, key_str_id, default_value -> value
pub fn shim_hashmap_get_or(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let default_val = task.ram.pop_i32();
    let key_str_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            let key_bytes = vm.strings.read().unwrap().get(key_str_idx).cloned()
                .ok_or(VMError::RuntimeError("Invalid string ID".into()))?;
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();

            if let Some(value) = map.get(&key_str) {
                match value {
                    auto_val::Value::Int(i) => { task.ram.push_i32(i); return Ok(()); }
                    auto_val::Value::Uint(u) => { task.ram.push_i32(u as i32); return Ok(()); }
                    auto_val::Value::Bool(b) => { task.ram.push_i32(if b { 1 } else { 0 }); return Ok(()); }
                    auto_val::Value::Str(s) => {
                        let mut strings = vm.strings.write().unwrap();
                        let str_idx = strings.len() as u32;
                        strings.push(s.as_bytes().to_vec());
                        drop(strings);
                        task.ram.push_str_idx(str_idx);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    task.ram.push_i32(default_val);
    Ok(())
}

/// Get all keys from a HashMap as a List of strings
/// Stack: hashmap_id -> list_id (List of string keys)
pub fn shim_hashmap_keys(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let map_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(map_id) {
        let guard = obj.read().unwrap();
        if let Some(map) = guard.as_any().downcast_ref::<SpecializedHashMap>() {
            // Collect keys from the specialized map
            let keys: Vec<String> = match map {
                SpecializedHashMap::StringInt(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringBool(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringString(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringDouble(m) => m.keys().cloned().collect(),
                SpecializedHashMap::StringValue(m) => m.keys().cloned().collect(),
            };
            drop(guard);

            // Create a List<Value> with string entries
            use crate::vm::types::ListData;
            let mut list: ListData<auto_val::Value> = ListData::new();
            for key in keys {
                list.push(auto_val::Value::Str(auto_val::AutoStr::from(key.as_str())));
            }
            let list_id = vm.insert_heap_object(list);
            task.ram.push_i32(list_id as i32);
            return Ok(());
        }
    }

    // Empty list if map not found
    use crate::vm::types::ListData;
    let list: ListData<auto_val::Value> = ListData::new();
    let list_id = vm.insert_heap_object(list);
    task.ram.push_i32(list_id as i32);
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
    #[cfg(feature = "nanbox")]
    let key = {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings.read().unwrap().get(idx).cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    };
    #[cfg(not(feature = "nanbox"))]
    let key = {
        let raw = task.ram.pop_i32();
        if raw < 0 {
            let idx = decode_str_idx(raw);
            vm.strings.read().unwrap().get(idx).cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            raw.to_string()
        }
    };

    let set_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(set_id) {
        let mut guard = obj.write().unwrap();
        if let Some(set) = guard.as_any_mut().downcast_mut::<SpecializedHashSet>() {
            set.data.insert(key, ());
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// Check if element exists in the set
/// Stack: hashset_id, elem_str_id -> result (1 if exists, 0 otherwise)
pub fn shim_hashset_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    let key = {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings.read().unwrap().get(idx).cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    };
    #[cfg(not(feature = "nanbox"))]
    let key = {
        let raw = task.ram.pop_i32();
        if raw < 0 {
            let idx = decode_str_idx(raw);
            vm.strings.read().unwrap().get(idx).cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_else(|| "INVALID".to_string())
        } else {
            raw.to_string()
        }
    };

    let set_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(set_id) {
        let guard = obj.read().unwrap();
        if let Some(set) = guard.as_any().downcast_ref::<SpecializedHashSet>() {
            if set.data.contains_key(&key) { 1 } else { 0 }
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
    let elem_str_idx = task.ram.pop_str_idx();
    let set_id = task.ram.pop_i32() as u64;

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
    let str_idx = task.ram.pop_str_idx();
    let sb_id = task.ram.pop_i32() as u64;

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

    // Return as tagged string index
    task.ram.push_str_idx(str_idx as u32);
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
    let key_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

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
    let key_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

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
    let key_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

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
    let key_idx = task.ram.pop_str_idx();
    let map_id = task.ram.pop_i32() as u64;

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
    // When compiler can't determine receiver type, it may call str.len on a List heap ID.
    // Detect this and dispatch to list.len instead.
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_i32(nv) {
            let i = auto_val::decode_i32(nv);
            if i >= 4000000 {
                // Looks like a heap object ID — try list.len
                use crate::vm::types::ListData;
                if let Some(obj) = vm.get_heap_object(i as u64) {
                    let guard = obj.read().unwrap();
                    if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                        task.ram.push_i32(list.len() as i32);
                        return Ok(());
                    }
                }
                task.ram.push_i32(0);
                return Ok(());
            }
        }
        // Normal string path
        let str_idx = auto_val::decode_string(nv) as u16;
        if let Some(bytes) = vm.get_string(str_idx) {
            task.ram.push_i32(bytes.len() as i32);
        } else {
            task.ram.push_i32(0);
        }
        return Ok(());
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let str_idx = task.ram.pop_str_idx() as u16;
        if let Some(bytes) = vm.get_string(str_idx) {
            task.ram.push_i32(bytes.len() as i32);
        } else {
            task.ram.push_i32(0);
        }
        Ok(())
    }
}

/// Helper: decode a NanoValue to a String (from string pool)
#[cfg(feature = "nanbox")]
fn nv_to_string(nv: auto_val::NanoValue, vm: &AutoVM) -> String {
    if auto_val::is_string(nv) {
        vm.get_string(auto_val::decode_string(nv) as u16)
            .map(|b| String::from_utf8_lossy(&b[..]).to_string())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

/// str.contains(substring) — check if string contains substring
/// Stack: substring, string -> bool
pub fn shim_str_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        let sub_nv = task.ram.pop_nv();
        let str_nv = task.ram.pop_nv();
        let str_s = nv_to_string(str_nv, vm);
        let sub_s = nv_to_string(sub_nv, vm);
        task.ram.push_i32(if str_s.contains(sub_s.as_str()) { -2147483648 } else { -2147483647 });
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let sub_idx = task.ram.pop_str_idx() as u16;
        let str_idx = task.ram.pop_str_idx() as u16;
        let str_s = vm.get_string(str_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        let sub_s = vm.get_string(sub_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        task.ram.push_i32(if str_s.contains(&sub_s) { -2147483648 } else { -2147483647 });
    }
    Ok(())
}

/// str.starts_with(prefix) — check if string starts with prefix
pub fn shim_str_starts_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        let prefix_nv = task.ram.pop_nv();
        let str_nv = task.ram.pop_nv();
        let str_s = nv_to_string(str_nv, vm);
        let prefix_s = nv_to_string(prefix_nv, vm);
        task.ram.push_i32(if str_s.starts_with(prefix_s.as_str()) { -2147483648 } else { -2147483647 });
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let prefix_idx = task.ram.pop_str_idx() as u16;
        let str_idx = task.ram.pop_str_idx() as u16;
        let str_s = vm.get_string(str_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        let prefix_s = vm.get_string(prefix_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        task.ram.push_i32(if str_s.starts_with(&prefix_s) { -2147483648 } else { -2147483647 });
    }
    Ok(())
}

/// str.ends_with(suffix) — check if string ends with suffix
pub fn shim_str_ends_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        let suffix_nv = task.ram.pop_nv();
        let str_nv = task.ram.pop_nv();
        let str_s = nv_to_string(str_nv, vm);
        let suffix_s = nv_to_string(suffix_nv, vm);
        task.ram.push_i32(if str_s.ends_with(suffix_s.as_str()) { -2147483648 } else { -2147483647 });
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let suffix_idx = task.ram.pop_str_idx() as u16;
        let str_idx = task.ram.pop_str_idx() as u16;
        let str_s = vm.get_string(str_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        let suffix_s = vm.get_string(suffix_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        task.ram.push_i32(if str_s.ends_with(&suffix_s) { -2147483648 } else { -2147483647 });
    }
    Ok(())
}

/// str.to_int() / str.parse_int() — parse string to int, return Result<int>
/// Stack: str -> CREATE_OK(int) or CREATE_ERR(str)
/// In nanbox mode, returns the value as CREATE_OK for proper .?() chaining
pub fn shim_str_to_int_nv(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        let s = if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv);
            vm.get_string(idx as u16)
                .map(|b| String::from_utf8_lossy(&b[..]).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let result = s.trim().parse::<i32>().unwrap_or(0);
        task.ram.push_nv(auto_val::encode_i32(result));
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let str_idx = task.ram.pop_str_idx() as u16;
        let s = vm.get_string(str_idx).map(|b| String::from_utf8_lossy(&b[..]).to_string()).unwrap_or_default();
        let result = s.trim().parse::<i32>().unwrap_or(0);
        task.ram.push_i32(result);
    }
    Ok(())
}

/// Get the length of a string (String.len).
/// Supports both constant pool strings (tagged index) and heap-based mutable Strings (SpecializedStringBuilder).
/// Stack: str_idx_or_sb_id -> length (as i32, char count for heap, byte count for const pool)
pub fn shim_string_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Under non-nanbox: raw i32 is either a heap object ID (positive) or tagged string index (negative).
    // Under nanbox: NanoValue is either an int (heap ID) or a string tag.
    #[cfg(not(feature = "nanbox"))]
    {
        let bits = task.ram.pop_i32();

        // First try as heap object (mutable String)
        let sb_id = bits as u64;
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let guard = obj.read().unwrap();
            if let Some(sb) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
                task.ram.push_i32(sb.buffer.chars().count() as i32);
                return Ok(());
            }
        }

        // Fall back to constant pool string (tagged index)
        let str_idx = decode_str_idx(bits) as u16;
        if let Some(bytes) = vm.get_string(str_idx) {
            task.ram.push_i32(bytes.len() as i32);
        } else {
            task.ram.push_i32(0);
        }
        Ok(())
    }

    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();

        // If it's a string, compute byte length directly
        if auto_val::is_string(nv) {
            let str_idx = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_idx) {
                task.ram.push_i32(bytes.len() as i32);
            } else {
                task.ram.push_i32(0);
            }
            return Ok(());
        }

        // Otherwise try as heap object ID
        let sb_id = auto_val::decode_i32(nv) as u64;
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let guard = obj.read().unwrap();
            if let Some(sb) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
                task.ram.push_i32(sb.buffer.chars().count() as i32);
                return Ok(());
            }
        }

        task.ram.push_i32(0);
        Ok(())
    }
}

/// Plan 118 Phase 4: Create a new mutable string with initial content and capacity.
/// Stack: capacity (i32), initial_str_idx (tagged) -> mut_str_id (i32)
/// The mutable string is stored in heap_objects as a SpecializedStringBuilder.
pub fn shim_str_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop capacity (not used in this simple implementation)
    let _capacity = task.ram.pop_i32();

    // Pop initial string index
    let str_idx = task.ram.pop_str_idx() as u16;

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
    let str_idx = task.ram.pop_str_idx() as u16;

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
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Plan 118 Phase 4: Convert string to uppercase.
/// Stack: str_idx (tagged) -> str_idx (tagged)
pub fn shim_str_upper(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string index
    let str_idx = task.ram.pop_str_idx() as u16;

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
    task.ram.push_str_idx(new_idx as u32);
    Ok(())
}

/// str.bytes() -> iterator of byte values
/// Creates a list of i32 byte values from a string and returns a ListIterator.
/// Stack: str_idx (tagged) -> iterator_id
pub fn shim_str_bytes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;
    use crate::vm::engine::{Iterator, ListIterator};
    use crate::vm::types::ListData;

    let str_idx = task.ram.pop_str_idx() as u16;

    // Get string bytes
    let bytes: Vec<i32> = if let Some(str_bytes) = vm.get_string(str_idx) {
        str_bytes.iter().map(|&b| b as i32).collect()
    } else {
        Vec::new()
    };

    // Create a ListData<i32> with byte values
    let mut list_data: ListData<i32> = ListData::new();
    for b in bytes {
        list_data.push(b);
    }
    let list_id = vm.insert_heap_object(list_data);

    // Create iterator
    let iterator_id = vm.iterator_id_gen.fetch_add(1, Ordering::Relaxed);
    let iterator = Iterator::List(ListIterator {
        list_id,
        current_index: 0,
    });
    vm.iterators.insert(iterator_id, iterator);

    task.ram.push_i32(iterator_id as i32);
    Ok(())
}

/// uint.to_hex(pad) -> hex string
/// Formats a u64 value as a zero-padded lowercase hex string.
/// Stack: pad_width (i32), val_lo (i32), val_hi (i32) -> str_idx (tagged)
pub fn shim_uint_to_hex(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // u64 is stored as two i32 slots: low pushed first, high pushed second (on top)
    let pad_width = task.ram.pop_i32() as usize;
    let val = task.ram.pop_u64();

    let hex_str = format!("{:0width$x}", val, width = pad_width);
    let bytes = hex_str.into_bytes();
    let str_idx = vm.add_string(bytes);

    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Plan 155: String.from(str) -> String
/// Creates an owned mutable String (heap object) from a string literal.
/// Stack: str_idx (tagged) -> sb_id (heap object ID)
pub fn shim_string_from(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u16;

    // Get string content from constant pool
    let content = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(bytes.as_slice()).to_string()
    } else {
        String::new()
    };

    // Create a SpecializedStringBuilder with the content
    let mut sb = crate::vm::collections::SpecializedStringBuilder::new();
    sb.buffer = content;

    // Register in heap
    let sb_id = vm.insert_heap_object(sb);
    task.ram.push_i32(sb_id as i32);
    Ok(())
}

// ============================================================================
// Mutable String Shims (177-186)
// ============================================================================

/// String.new() -> sb_id
/// Create a new mutable String (backed by SpecializedStringBuilder).
/// Stack: [] -> sb_id
pub fn shim_string_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb = crate::vm::collections::SpecializedStringBuilder::new();
    let sb_id = vm.insert_heap_object(sb);
    task.ram.push_i32(sb_id as i32);
    Ok(())
}

/// s.push(char) -> 0
/// Push a character (as codepoint) to the end of the mutable string.
/// Stack: [char_codepoint, sb_id] -> 0
pub fn shim_string_push(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_codepoint = task.ram.pop_i32();
    let sb_id = task.ram.pop_i32() as u64;

    if let Some(ch) = char::from_u32(char_codepoint as u32) {
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

/// s.pop() -> char_codepoint (0 if empty)
/// Pop the last character from the mutable string.
/// Stack: [sb_id] -> char_codepoint
pub fn shim_string_pop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
            match sb.buffer.pop() {
                Some(ch) => ch as i32,
                None => 0,
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

/// s.get(index) -> char_codepoint
/// Get the character at the given char index.
/// Stack: [index, sb_id] -> char_codepoint
pub fn shim_string_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index = task.ram.pop_i32() as usize;
    let sb_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.chars().nth(index).map(|ch| ch as i32).unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    task.ram.push_i32(result);
    Ok(())
}

/// s.set(index, char) -> 0
/// Replace the character at the given char index.
/// Stack: [char_codepoint, index, sb_id] -> 0
pub fn shim_string_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_codepoint = task.ram.pop_i32();
    let index = task.ram.pop_i32() as usize;
    let sb_id = task.ram.pop_i32() as u64;

    if let Some(new_ch) = char::from_u32(char_codepoint as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
                // Find byte offset and old char len at char position `index`
                if let Some((byte_offset, old_ch)) = sb.buffer.char_indices().nth(index) {
                    let old_len = old_ch.len_utf8();
                    sb.buffer.replace_range(byte_offset..byte_offset + old_len, &new_ch.to_string());
                }
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// s.insert(index, char) -> 0
/// Insert a character at the given char index.
/// Stack: [char_codepoint, index, sb_id] -> 0
pub fn shim_string_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let char_codepoint = task.ram.pop_i32();
    let index = task.ram.pop_i32() as usize;
    let sb_id = task.ram.pop_i32() as u64;

    if let Some(ch) = char::from_u32(char_codepoint as u32) {
        if let Some(obj) = vm.get_heap_object(sb_id) {
            let mut guard = obj.write().unwrap();
            if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
                // Find byte offset at char position `index`
                let byte_offset = sb.buffer.char_indices()
                    .nth(index)
                    .map(|(offset, _)| offset)
                    .unwrap_or(sb.buffer.len());
                sb.buffer.insert(byte_offset, ch);
            }
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// s.remove(index) -> char_codepoint
/// Remove the character at the given char index and return it.
/// Stack: [index, sb_id] -> char_codepoint
pub fn shim_string_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index = task.ram.pop_i32() as usize;
    let sb_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
            // Find byte offset and char len at char position `index`
            if let Some((byte_offset, old_ch)) = sb.buffer.char_indices().nth(index) {
                let old_len = old_ch.len_utf8();
                let removed_codepoint = old_ch as i32;
                sb.buffer.drain(byte_offset..byte_offset + old_len);
                removed_codepoint
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

/// s.clear() -> 0
/// Clear all characters from the mutable string.
/// Stack: [sb_id] -> 0
pub fn shim_string_clear(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
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

/// s.is_empty() -> bool (1/0)
/// Check if the mutable string is empty.
/// Stack: [sb_id] -> 1/0
pub fn shim_string_is_empty(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sb_id = task.ram.pop_i32() as u64;

    let result = if let Some(obj) = vm.get_heap_object(sb_id) {
        let guard = obj.read().unwrap();
        if let Some(sb) = guard.as_any().downcast_ref::<crate::vm::collections::SpecializedStringBuilder>() {
            if sb.buffer.is_empty() { 1 } else { 0 }
        } else {
            1
        }
    } else {
        1
    };

    task.ram.push_i32(result);
    Ok(())
}

/// s.reserve(n) -> 0
/// Reserve capacity for at least n additional bytes.
/// Stack: [n, sb_id] -> 0
pub fn shim_string_reserve(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32() as usize;
    let sb_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(sb_id) {
        let mut guard = obj.write().unwrap();
        if let Some(sb) = guard.as_any_mut().downcast_mut::<crate::vm::collections::SpecializedStringBuilder>() {
            sb.buffer.reserve(n);
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

// ============================================================================
// Memory Allocation Shims
// ============================================================================

/// alloc_array(size) -> list_id
/// Allocate a new array of the given size initialized to 0.
/// Stack: [size] -> list_id
/// Uses the same arrays registry as CREATE_ARRAY for compatibility with SET_ELEM/GET_ELEM.
pub fn shim_alloc_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use std::sync::atomic::Ordering;

    let size_raw = task.ram.pop_i32();
    if size_raw < 0 {
        return Err(VMError::RuntimeError(format!(
            "alloc_array: invalid size {} (must be >= 0)", size_raw
        )));
    }
    let size = size_raw as usize;
    let elems: Vec<auto_val::Value> = vec![auto_val::Value::Int(0); size];

    let array_id = vm.array_id_gen.fetch_add(1, Ordering::SeqCst);
    vm.arrays.insert(array_id, Arc::new(RwLock::new(elems)));
    task.ram.push_i32(array_id as i32);
    Ok(())
}

/// realloc_array([array_id, new_size]) -> array_id
/// Reallocate list to new size, preserving data.
/// Stack: [new_size, array_id] -> array_id
pub fn shim_realloc_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let new_size = task.ram.pop_i32() as usize;
    let arr_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(arr_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.elems.resize(new_size, 0);
            drop(guard);
            task.ram.push_i32(arr_id as i32);
            return Ok(());
        }
    }
    // Fallback: create new list
    let mut list: ListData<i32> = ListData::new();
    for _ in 0..new_size {
        list.push(0);
    }
    let id = vm.insert_heap_object(list);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// free_array(array_id) -> nil
/// Free an array (no-op in GC-managed VM).
/// Stack: [array_id] -> nil (-2147483647)
pub fn shim_free_array(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _arr_id = task.ram.pop_i32();
    task.ram.push_i32(-2147483647); // nil marker
    Ok(())
}

// ============================================================================
// Storage Shims (Heap, InlineInt64)
// ============================================================================

/// Heap.new() -> instance_id
/// Create a new Heap storage instance using a ListData<i32> internally.
pub fn shim_heap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Create an empty list to represent the Heap storage
    let list: ListData<i32> = ListData::new();
    let id = vm.insert_heap_object(list);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// heap.capacity() -> capacity
/// Get Heap storage capacity.
/// Stack: [instance_id] -> capacity
pub fn shim_heap_capacity(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let inst_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(inst_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.elems.capacity() as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// heap.try_grow(min_cap) -> bool
/// Try to grow Heap storage to at least min_cap elements.
/// Stack: [min_cap, instance_id] -> bool (1/0)
pub fn shim_heap_try_grow(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let min_cap = task.ram.pop_i32() as usize;
    let inst_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(inst_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let cap = list.elems.capacity();
            let new_cap = if cap == 0 {
                std::cmp::max(8, min_cap)
            } else {
                std::cmp::max(cap * 2, min_cap)
            };
            list.elems.resize(new_cap, 0);
            drop(guard);
            task.ram.push_i32(1); // success
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// heap.drop() -> nil
/// Free Heap storage.
/// Stack: [instance_id] -> nil
pub fn shim_heap_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let inst_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(inst_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.clear();
        }
    }
    task.ram.push_i32(-2147483647); // nil
    Ok(())
}

/// InlineInt64.new() -> instance_id
pub fn shim_inline_int64_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let mut list: ListData<i32> = ListData::new();
    for _ in 0..64 {
        list.push(0);
    }
    let id = vm.insert_heap_object(list);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// InlineInt64.capacity() -> 64
/// Stack: [instance_id] -> 64
pub fn shim_inline_int64_capacity(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _inst_id = task.ram.pop_i32();
    task.ram.push_i32(64);
    Ok(())
}

/// InlineInt64.try_grow(min_cap) -> bool
/// Stack: [min_cap, instance_id] -> bool
pub fn shim_inline_int64_try_grow(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let min_cap = task.ram.pop_i32() as u32;
    let _inst_id = task.ram.pop_i32();
    task.ram.push_i32(if min_cap <= 64 { 1 } else { 0 });
    Ok(())
}

/// InlineInt64.drop() -> nil
pub fn shim_inline_int64_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _inst_id = task.ram.pop_i32();
    task.ram.push_i32(-2147483647); // nil
    Ok(())
}

// ============================================================================
// List Extra Shims
// ============================================================================

/// list.capacity() -> capacity
/// Get list capacity.
/// Stack: [list_id] -> capacity
pub fn shim_list_capacity(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.elems.capacity() as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2: Rand Native Shims
// ============================================================================

/// Simple xorshift64 PRNG — no external rand crate needed
#[derive(Debug)]
struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0xDEAD_BEEF_CAFE_BABE } else { seed } }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

/// thread_rng() -> opaque Rng handle (stored as RustStdlibObject on heap)
/// Stack: [] -> [rng_id_i32]
pub fn shim_rand_thread_rng(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let rng = std::sync::Mutex::new(Xorshift64::new(seed));
    let obj = RustStdlibObject::new("rand::ThreadRng", rng);
    let rng_id = vm.insert_heap_object(obj);
    task.ram.push_i32(rng_id as i32);
    Ok(())
}

/// rng.gen_range(lo, hi) -> i32
/// Stack: [hi, lo, rng_id] -> [result]
pub fn shim_rng_gen_range(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let hi = task.ram.pop_i32();
    let lo = task.ram.pop_i32();
    let rng_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let range = (hi - lo).max(1);
                let val = rng.get_mut().unwrap().next();
                let result = lo + ((val % range as u64) as i32);
                task.ram.push_i32(result);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// rng.gen() -> i32
/// Stack: [rng_id] -> [result]
pub fn shim_rng_gen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let rng_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = rng.get_mut().unwrap().next() as i32;
                task.ram.push_i32(val);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// rng.drop() — no-op, GC will handle
pub fn shim_rng_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _rng_id = task.ram.pop_i32();
    Ok(())
}

/// rand::random() → random i32
/// Stack: [] -> [random_i32]
pub fn shim_rand_random(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    task.ram.push_i32(rng.next() as i32);
    Ok(())
}

/// log no-op — swallows all arguments, returns nothing.
/// Used for env_logger.init(), log::set_max_level(), tracing::init(), etc.
pub fn shim_log_noop(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // Arguments are already consumed by the caller's stack management.
    // Nothing to do — these functions are pure side-effects we ignore.
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.2: Regex Opaque Struct Shims
// ============================================================================

/// Helper: pop a string from the VM stack (handles both nanbox and non-nanbox)
fn pop_vm_string(task: &mut AutoTask, vm: &AutoVM) -> String {
    #[cfg(not(feature = "nanbox"))]
    {
        let tagged = task.ram.pop_i32();
        if tagged < 0 {
            let idx = ((-tagged) - 1) as u16;
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default()
        } else {
            format!("{}", tagged)
        }
    }
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as u16;
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default()
        } else {
            let val = auto_val::decode_i32(nv);
            format!("{}", val)
        }
    }
}

/// Helper: push a string onto the VM stack via string pool
fn push_vm_string(task: &mut AutoTask, vm: &AutoVM, s: &str) {
    let idx = vm.add_string(s.as_bytes().to_vec()) as u32;
    task.ram.push_str_idx(idx);
}

/// Regex.new(pattern) → opaque handle
/// Stack: [pattern_str] -> [handle_i32]
pub fn shim_re_opaque_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let pattern = pop_vm_string(task, vm);
    match regex::Regex::new(&pattern) {
        Ok(re) => {
            let obj = RustStdlibObject::new("regex::Regex", std::sync::Mutex::new(re));
            let id = vm.insert_heap_object(obj);
            task.ram.push_i32(id as i32);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("Regex::new failed: {}", e)));
        }
    }
    Ok(())
}

/// re.is_match(text) → bool
/// Stack: [text_str, handle_i32] -> [bool_i32]
pub fn shim_re_opaque_is_match(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let text = pop_vm_string(task, vm);
    let re_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().is_match(&text);
                task.ram.push_i32(if result { 1 } else { 0 });
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// re.find(text) → string (first match) or empty string if none
/// Stack: [text_str, handle_i32] -> [result_str]
pub fn shim_re_opaque_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let text = pop_vm_string(task, vm);
    let re_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().find(&text);
                let s = result.map(|m| m.as_str().to_string()).unwrap_or_default();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// re.find_all(text) → List of matched strings
/// Stack: [text_str, handle_i32] -> [list_id_i32]
pub fn shim_re_opaque_find_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use crate::vm::types::ListData;

    let text = pop_vm_string(task, vm);
    let re_id = task.ram.pop_i32() as u64;

    let mut matches: Vec<i32> = Vec::new();
    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                for m in re.lock().unwrap().find_iter(&text) {
                    let match_str = m.as_str();
                    let idx = vm.add_string(match_str.as_bytes().to_vec()) as u32;
                    matches.push(-(idx as i32) - 1);
                }
            }
        }
    }

    let mut list = ListData::<i32>::new();
    list.elems = matches;
    let obj = RustStdlibObject::new("List<i32>", std::sync::Mutex::new(list));
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// re.replace_all(text, replacement) → string
/// Stack: [replacement_str, text_str, handle_i32] -> [result_str]
pub fn shim_re_opaque_replace_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let replacement = pop_vm_string(task, vm);
    let text = pop_vm_string(task, vm);
    let re_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(re_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                let result = re.lock().unwrap().replace_all(&text, replacement.as_str());
                let result_str = result.to_string();
                push_vm_string(task, vm, &result_str);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, &text);
    Ok(())
}

/// re.captures(text) → opaque captures handle
/// Stores captures as Vec<String> (owned copy of each capture group)
/// Stack: [text_str, handle_i32] -> [captures_handle_i32]
pub fn shim_re_opaque_captures(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let text = pop_vm_string(task, vm);
    let re_id = task.ram.pop_i32() as u64;

    let captures: Option<Vec<String>> = {
        if let Some(obj) = vm.get_heap_object(re_id) {
            let guard = obj.read().unwrap();
            if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                if let Some(re) = rso.downcast_ref::<std::sync::Mutex<regex::Regex>>() {
                    re.lock().unwrap().captures(&text).map(|caps| {
                        caps.iter().map(|m| m.map(|s| s.as_str().to_string()).unwrap_or_default()).collect()
                    })
                } else { None }
            } else { None }
        } else { None }
    };

    if let Some(groups) = captures {
        let caps_obj = RustStdlibObject::new("regex::Captures", std::sync::Mutex::new(groups));
        let id = vm.insert_heap_object(caps_obj);
        task.ram.push_i32(id as i32);
    } else {
        task.ram.push_i32(0);
    }
    Ok(())
}

/// re.drop() — no-op, GC handles cleanup
pub fn shim_re_opaque_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _re_id = task.ram.pop_i32();
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.2: Url Opaque Struct Shims
// ============================================================================

/// Url.parse(url_str) → opaque handle
/// Stack: [url_str] -> [handle_i32]
pub fn shim_url_opaque_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_str = pop_vm_string(task, vm);
    match url::Url::parse(&url_str) {
        Ok(url) => {
            let obj = RustStdlibObject::new("url::Url", std::sync::Mutex::new(url));
            let id = vm.insert_heap_object(obj);
            task.ram.push_i32(id as i32);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("Url::parse failed: {}", e)));
        }
    }
    Ok(())
}

/// url.scheme() → string
/// Stack: [handle_i32] -> [scheme_str]
pub fn shim_url_opaque_scheme(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                push_vm_string(task, vm, url.lock().unwrap().scheme());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.host_str() → string or empty
/// Stack: [handle_i32] -> [host_str]
pub fn shim_url_opaque_host_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let locked = url.lock().unwrap();
                let s = locked.host_str().unwrap_or("").to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.path() → string
/// Stack: [handle_i32] -> [path_str]
pub fn shim_url_opaque_path(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                push_vm_string(task, vm, url.lock().unwrap().path());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.fragment() → string or empty
/// Stack: [handle_i32] -> [fragment_str]
pub fn shim_url_opaque_fragment(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let locked = url.lock().unwrap();
                let s = locked.fragment().unwrap_or("").to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.port() → int (0 if none)
/// Stack: [handle_i32] -> [port_i32]
pub fn shim_url_opaque_port(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let port = url.lock().unwrap().port().unwrap_or(0) as i32;
                task.ram.push_i32(port);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// url.query_pairs() → List of "key=value" strings
/// Stack: [handle_i32] -> [list_id_i32]
pub fn shim_url_opaque_query_pairs(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use crate::vm::types::ListData;

    let url_id = task.ram.pop_i32() as u64;
    let mut pairs: Vec<i32> = Vec::new();

    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                for (k, v) in url.lock().unwrap().query_pairs() {
                    let pair = format!("{}={}", k, v);
                    let idx = vm.add_string(pair.as_bytes().to_vec()) as u32;
                    pairs.push(-(idx as i32) - 1);
                }
            }
        }
    }

    let mut list = ListData::<i32>::new();
    list.elems = pairs;
    let obj = RustStdlibObject::new("List<i32>", std::sync::Mutex::new(list));
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// url.join(relative) → new opaque handle
/// Stack: [relative_str, handle_i32] -> [new_handle_i32]
pub fn shim_url_opaque_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let relative = pop_vm_string(task, vm);
    let url_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                match url.lock().unwrap().join(&relative) {
                    Ok(joined) => {
                        let new_obj = RustStdlibObject::new("url::Url", std::sync::Mutex::new(joined));
                        let id = vm.insert_heap_object(new_obj);
                        task.ram.push_i32(id as i32);
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(VMError::RuntimeError(format!("Url::join failed: {}", e)));
                    }
                }
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// url.origin() → string
/// Stack: [handle_i32] -> [origin_str]
pub fn shim_url_opaque_origin(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let origin = url.lock().unwrap().origin().ascii_serialization();
                push_vm_string(task, vm, &origin);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.drop() — no-op, GC handles cleanup
pub fn shim_url_opaque_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _url_id = task.ram.pop_i32();
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.2: Semver Opaque Struct Shims
// ============================================================================

/// Version.parse(ver_str) → opaque handle
/// Stack: [ver_str] -> [handle_i32]
pub fn shim_semver_opaque_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_str = pop_vm_string(task, vm);
    match semver::Version::parse(&ver_str) {
        Ok(ver) => {
            let obj = RustStdlibObject::new("semver::Version", std::sync::Mutex::new(ver));
            let id = vm.insert_heap_object(obj);
            task.ram.push_i32(id as i32);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("Version::parse failed: {}", e)));
        }
    }
    Ok(())
}

/// v.major → u64
/// Stack: [handle_i32] -> [major_i32]
pub fn shim_semver_opaque_major(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                task.ram.push_i32(ver.lock().unwrap().major as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// v.minor → u64
/// Stack: [handle_i32] -> [minor_i32]
pub fn shim_semver_opaque_minor(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                task.ram.push_i32(ver.lock().unwrap().minor as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// v.patch → u64
/// Stack: [handle_i32] -> [patch_i32]
pub fn shim_semver_opaque_patch(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                task.ram.push_i32(ver.lock().unwrap().patch as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// v.pre → string
/// Stack: [handle_i32] -> [pre_str]
pub fn shim_semver_opaque_pre(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                push_vm_string(task, vm, &ver.lock().unwrap().pre.to_string());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// v.to_string() → string
/// Stack: [handle_i32] -> [ver_str]
pub fn shim_semver_opaque_to_string(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                push_vm_string(task, vm, &ver.lock().unwrap().to_string());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// v1 > v2 → bool
/// Stack: [v2_handle, v1_handle] -> [bool_i32]
pub fn shim_semver_opaque_cmp_gt(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let v2_id = task.ram.pop_i32() as u64;
    let v1_id = task.ram.pop_i32() as u64;

    let v1 = if let Some(obj) = vm.get_heap_object(v1_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                Some(ver.lock().unwrap().clone())
            } else { None }
        } else { None }
    } else { None };

    let v2 = if let Some(obj) = vm.get_heap_object(v2_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                Some(ver.lock().unwrap().clone())
            } else { None }
        } else { None }
    } else { None };

    match (v1, v2) {
        (Some(a), Some(b)) => task.ram.push_i32(if a > b { 1 } else { 0 }),
        _ => task.ram.push_i32(0),
    }
    Ok(())
}

/// version.drop() — no-op, GC handles cleanup
pub fn shim_semver_opaque_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _ver_id = task.ram.pop_i32();
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.3: Chrono Opaque Struct Shims
// ============================================================================

/// Local.now() → opaque NaiveDateTime handle
/// Stack: [] -> [handle_i32]
pub fn shim_chrono_local_now(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let dt = chrono::Local::now().naive_local();
    let obj = RustStdlibObject::new("chrono::NaiveDateTime", std::sync::Mutex::new(dt));
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// dt.year() → i32
/// Stack: [handle_i32] -> [year_i32]
pub fn shim_chrono_year(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().year());
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.month() → i32
/// Stack: [handle_i32] -> [month_i32]
pub fn shim_chrono_month(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().month() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.day() → i32
/// Stack: [handle_i32] -> [day_i32]
pub fn shim_chrono_day(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().day() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.hour() → i32
/// Stack: [handle_i32] -> [hour_i32]
pub fn shim_chrono_hour(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Timelike;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().hour() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.minute() → i32
/// Stack: [handle_i32] -> [minute_i32]
pub fn shim_chrono_minute(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Timelike;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().minute() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.second() → i32
/// Stack: [handle_i32] -> [second_i32]
pub fn shim_chrono_second(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Timelike;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                task.ram.push_i32(dt.lock().unwrap().second() as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// dt.timestamp() → i64 (pushed as two i32)
/// Stack: [handle_i32] -> [timestamp_lo, timestamp_hi]
pub fn shim_chrono_timestamp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let dt_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                let ts = dt.lock().unwrap().and_utc().timestamp();
                task.ram.push_i64(ts);
                return Ok(());
            }
        }
    }
    task.ram.push_i64(0);
    Ok(())
}

/// dt.format(fmt) → string
/// Stack: [fmt_str, handle_i32] -> [result_str]
pub fn shim_chrono_format(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let fmt = pop_vm_string(task, vm);
    let dt_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(dt_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                let formatted = dt.lock().unwrap().format(&fmt).to_string();
                push_vm_string(task, vm, &formatted);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// dt.drop() — no-op, GC handles cleanup
pub fn shim_chrono_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _dt_id = task.ram.pop_i32();
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.3: Base64 Pure Function Shims
// ============================================================================

/// base64::encode(input) → string
/// Stack: [input_str] -> [encoded_str]
pub fn shim_base64_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &input);
    push_vm_string(task, vm, &encoded);
    Ok(())
}

/// base64::decode(input) → string (decoded bytes as UTF-8 string)
/// Stack: [input_str] -> [decoded_str]
pub fn shim_base64_decode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &input) {
        Ok(bytes) => {
            let decoded = String::from_utf8_lossy(&bytes).into_owned();
            push_vm_string(task, vm, &decoded);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("base64::decode failed: {}", e)));
        }
    }
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.3: Hex Pure Function Shims
// ============================================================================

/// hex::encode(input) → string
/// Stack: [input_str] -> [hex_str]
pub fn shim_hex_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    let encoded = hex::encode(input.as_bytes());
    push_vm_string(task, vm, &encoded);
    Ok(())
}

/// hex::decode(input) → string (decoded bytes as UTF-8 string)
/// Stack: [input_str] -> [decoded_str]
pub fn shim_hex_decode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let input = pop_vm_string(task, vm);
    match hex::decode(&input) {
        Ok(bytes) => {
            let decoded = String::from_utf8_lossy(&bytes).into_owned();
            push_vm_string(task, vm, &decoded);
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("hex::decode failed: {}", e)));
        }
    }
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.3: Sha2 Opaque Struct Shims
// ============================================================================

/// Sha256::new() → opaque handle
/// Stack: [] -> [handle_i32]
pub fn shim_sha2_sha256_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use sha2::Digest;

    let hasher = sha2::Sha256::new();
    let obj = RustStdlibObject::new("sha2::Sha256", std::sync::Mutex::new(hasher));
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// hasher.update(data) → void
/// Stack: [data_str, handle_i32] -> [0]
pub fn shim_sha2_update(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use sha2::Digest;

    let data = pop_vm_string(task, vm);
    let hasher_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(hasher_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(hasher) = rso.downcast_mut::<std::sync::Mutex<sha2::Sha256>>() {
                hasher.get_mut().unwrap().update(data.as_bytes());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// hasher.finalize() → hex string
/// Note: finalize() takes ownership of the hasher, so we clone the hasher state
/// to compute the hash while keeping the original available for further updates.
/// Stack: [handle_i32] -> [hex_str]
pub fn shim_sha2_finalize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use sha2::Digest;

    let hasher_id = task.ram.pop_i32() as u64;

    if let Some(obj) = vm.get_heap_object(hasher_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(hasher) = rso.downcast_ref::<std::sync::Mutex<sha2::Sha256>>() {
                // Clone the hasher to compute without consuming
                let cloned = hasher.lock().unwrap().clone();
                let result = cloned.finalize();
                let hex_str = hex::encode(result);
                drop(guard);
                push_vm_string(task, vm, &hex_str);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// hasher.drop() — no-op, GC handles cleanup
pub fn shim_sha2_drop(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let _hasher_id = task.ram.pop_i32();
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.3: Mime Guess Pure Function Shim
// ============================================================================

/// mime_guess::from_path(path) → string (MIME type or empty)
/// Stack: [path_str] -> [mime_str]
pub fn shim_mime_from_path(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_vm_string(task, vm);
    let mime = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();
    push_vm_string(task, vm, &mime);
    Ok(())
}

// Plan 240: Instant opaque shims for std::time::Instant

/// Create an Instant::now() opaque handle.
/// Stack: -> handle_id
pub fn shim_instant_now(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let instant = std::time::Instant::now();
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
        "std::time::Instant",
        instant,
    );
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// Get elapsed time from Instant handle as a formatted string.
/// Stack: handle_id -> string (e.g., "123ms")
pub fn shim_instant_elapsed(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid Instant handle".to_string()))?;
    let guard = obj.read().unwrap();
    let rust_obj = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let instant = rust_obj.downcast_ref::<std::time::Instant>()
        .ok_or_else(|| VMError::RuntimeError("Not an Instant object".to_string()))?;
    let elapsed = instant.elapsed();
    let millis = elapsed.as_millis();
    let nanos = elapsed.as_nanos();
    let result = if millis > 0 {
        format!("{}ms", millis)
    } else {
        format!("{}ns", nanos)
    };
    drop(guard);
    let bytes = result.into_bytes();
    let idx = {
        let mut strings = vm.strings.write().unwrap();
        strings.push(bytes);
        strings.len() - 1
    };
    #[cfg(feature = "nanbox")]
    {
        let nv = auto_val::encode_string(idx as u32);
        task.ram.push_nv(nv);
        task.ram.push_nv(auto_val::encode_null());
    }
    #[cfg(not(feature = "nanbox"))]
    {
        task.ram.push_i32(idx as i32);
    }
    Ok(())
}

// Plan 240: File I/O opaque shims

/// Helper to pop a string from the stack (nanbox-aware)
fn pop_string(task: &mut AutoTask, vm: &AutoVM) -> String {
    #[cfg(feature = "nanbox")]
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings.read().unwrap().get(idx).cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    }
    #[cfg(not(feature = "nanbox"))]
    {
        let idx = task.ram.pop_i32() as usize;
        vm.strings.read().unwrap().get(idx).cloned()
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .unwrap_or_default()
    }
}

/// File.create(path) → opaque handle with BufWriter<File>
/// Stack: path_string -> handle_id
pub fn shim_file_create_handle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_string(task, vm);
    let file = std::fs::File::create(&path)
        .map_err(|e| VMError::RuntimeError(format!("File.create failed: {}", e)))?;
    let writer: Box<dyn std::io::Write + Send + Sync> = Box::new(std::io::BufWriter::new(file));
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("std::fs::File", writer);
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// File.open(path) → opaque handle
/// Stack: path_string -> handle_id
pub fn shim_file_open_handle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_string(task, vm);
    let file = std::fs::File::open(&path)
        .map_err(|e| VMError::RuntimeError(format!("File.open failed: {}", e)))?;
    let content = {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::BufReader::new(file), &mut buf)
            .map_err(|e| VMError::RuntimeError(format!("File.open read failed: {}", e)))?;
        buf
    };
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("std::fs::FileContent", content);
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// handle.write(data) → write string to file handle
/// Stack: data_string, handle_id -> void
pub fn shim_file_write_handle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data = pop_string(task, vm);
    let handle = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid File handle".to_string()))?;
    let mut guard = obj.write().unwrap();
    let rust_obj = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let writer = rust_obj.downcast_mut::<Box<dyn std::io::Write + Send + Sync>>()
        .ok_or_else(|| VMError::RuntimeError("Not a File writer".to_string()))?;
    writer.write_all(data.as_bytes())
        .map_err(|e| VMError::RuntimeError(format!("write failed: {}", e)))?;
    drop(guard);
    task.ram.push_i32(0);
    Ok(())
}

/// handle.try_clone() → clone the handle (returns same content for read handles)
/// Stack: handle_id -> handle_id (clone)
pub fn shim_file_try_clone(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // For simplicity, just push a copy of the handle ID
    let handle = task.ram.pop_i32();
    task.ram.push_i32(handle);
    Ok(())
}

// Plan 240: OnceCell opaque shims using RustStdlibObject wrapping Option<String>

/// Create a new OnceCell (empty).
/// Stack: -> handle_id
pub fn shim_once_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cell: Option<String> = None;
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
        "std::cell::OnceCell",
        cell,
    );
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// Set a value in the OnceCell.
/// Stack: string_value, handle_id -> void (pushes 1 for success, 0 for already set)
pub fn shim_once_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop string value
    #[cfg(feature = "nanbox")]
    let value = {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings.read().unwrap().get(idx).cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    };
    #[cfg(not(feature = "nanbox"))]
    let value = {
        let idx = task.ram.pop_i32() as usize;
        vm.strings.read().unwrap().get(idx).cloned()
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .unwrap_or_default()
    };

    let handle = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid OnceCell handle".to_string()))?;
    let mut guard = obj.write().unwrap();
    let rust_obj = guard.as_any_mut().downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let cell = rust_obj.downcast_mut::<Option<String>>()
        .ok_or_else(|| VMError::RuntimeError("Not an Option<String>".to_string()))?;
    if cell.is_none() {
        *cell = Some(value);
        task.ram.push_i32(1); // success
    } else {
        task.ram.push_i32(0); // already set
    }
    Ok(())
}

/// Get the value from the OnceCell.
/// Returns opaque handle (>=0) if set, or -1 if None.
/// Stack: handle_id -> i32 (opaque handle or -1)
pub fn shim_once_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid OnceCell handle".to_string()))?;
    let guard = obj.read().unwrap();
    let rust_obj = guard.as_any().downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let cell = rust_obj.downcast_ref::<Option<String>>()
        .ok_or_else(|| VMError::RuntimeError("Not an Option<String>".to_string()))?;

    match cell {
        None => {
            task.ram.push_i32(-1);
        }
        Some(value) => {
            let string_obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
                "std::cell::OnceCell::Value",
                value.clone(),
            );
            let id = vm.insert_heap_object(string_obj);
            task.ram.push_i32(id as i32);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_direct_lookup() {
        let mut ni = NativeInterface::new();
        ni.register_name("Result.map_err", 2070);
        assert_eq!(ni.resolve("Result.map_err"), Some(2070));
        assert_eq!(ni.resolve("nonexistent.method"), None);
    }

    #[test]
    fn test_resolve_canonical_normalization() {
        let mut ni = NativeInterface::new();
        ni.register_name("auto.list.push", 101);
        ni.register_name("auto.hashmap.insert", 120);
        ni.register_name("auto.list.join", 2080);

        assert_eq!(ni.resolve("List.push"), Some(101));
        assert_eq!(ni.resolve("List.join"), Some(2080));
        assert_eq!(ni.resolve("HashMap.insert"), Some(120));
        assert_eq!(ni.resolve("Map.insert"), Some(120));
    }

    #[test]
    fn test_resolve_no_canonical_for_qualified() {
        let mut ni = NativeInterface::new();
        ni.register_name("auto.list.push", 101);

        assert_eq!(ni.resolve("auto.list.push"), Some(101));
        assert_eq!(ni.resolve("rust.something"), None);
    }

    #[test]
    fn test_to_canonical() {
        assert_eq!(NativeInterface::to_canonical("List.push"), Some("auto.list.push".to_string()));
        assert_eq!(NativeInterface::to_canonical("HashMap.get"), Some("auto.hashmap.get".to_string()));
        assert_eq!(NativeInterface::to_canonical("Map.new"), Some("auto.hashmap.new".to_string()));
        assert_eq!(NativeInterface::to_canonical("Array.len"), Some("auto.list.len".to_string()));
        assert_eq!(NativeInterface::to_canonical("nopart"), None);
    }
}
