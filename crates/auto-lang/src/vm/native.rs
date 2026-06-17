use crate::vm::collections::{SpecializedHashMap, SpecializedHashSet};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::task::AutoTask;
use auto_val::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::RwLock;

use crate::{for_each_native, gen_native_constants};

/// Decode a tagged string index from a NanoValue popped from the stack.
#[inline]
fn decode_str_idx_nv(nv: auto_val::NanoValue) -> usize {
    auto_val::decode_string(nv) as usize
}

/// Encode as a NanoValue string tag.
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
            let mut natives = BIGVM_NATIVES.lock().unwrap();
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
        // Plan 249 Phase 2: All shim bindings generated from unified catalog.
        // We define a local macro to consume for_each_native! because #[macro_export]
        // macros cannot use `self` (hygiene restriction). Local macros have no such issue.
        macro_rules! __register_shims {
            (($id:expr, $name:ident, $fn:ident, $canonical:expr) $(, $rest:tt)*) => {
                self.register($name, $fn);
                __register_shims!($($rest),*);
            };
            () => {};
        }
        for_each_native!(__register_shims);

        // Plan 249 Phase 5: Canonical name → ID mappings for CALL_SPEC fallback.
        // Generated from the same catalog as shim bindings (4th field = canonical name).
        {
            macro_rules! __register_names {
                (($id:expr, $name:ident, $fn:ident, $canonical:expr) $(, $rest:tt)*) => {
                    self.register_name($canonical, $name);
                    __register_names!($($rest),*);
                };
                () => {};
            }
            for_each_native!(__register_names);
        }

        // --- Extra aliases (1-to-N mappings, not in catalog) ---
        self.register_name("Result.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("Result.Ok.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("Result.Err.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("auto.hashmap.insert", NATIVE_HASHMAP_INSERT_STR);
        self.register_name("auto.hashmap.set", NATIVE_HASHMAP_INSERT_STR);
        self.register_name("auto.hashmap.get", NATIVE_HASHMAP_GET_STR);
        self.register_name("auto.hashmap.contains_key", NATIVE_HASHMAP_CONTAINS);
        self.register_name("auto.hashmap.len", NATIVE_HASHMAP_SIZE);
    }
}


// Plan 249: Native constants generated from unified catalog
for_each_native!(gen_native_constants);


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
// (needed because #[rust_fn] uses VMConvertible which handles f64 encoding)
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

// === Manual constants (registered via register_shim_by_name, not in catalog) ===

// String methods (canonical IDs: auto.str.contains=1504, auto.str.starts_with=1505, auto.str.ends_with=1506)
pub const NATIVE_STR_CONTAINS: u16 = 1504;
pub const NATIVE_STR_STARTS_WITH: u16 = 1505;
pub const NATIVE_STR_ENDS_WITH: u16 = 1506;
pub const NATIVE_STR_TO_INT: u16 = 1516;

// Math functions registered in stdlib.rs via register_shim_by_name
pub const NATIVE_MATH_ABS: u16 = 1700;
pub const NATIVE_MATH_MIN: u16 = 1701;
pub const NATIVE_MATH_MAX: u16 = 1702;
pub const NATIVE_MATH_SQRT: u16 = 1750;  // Changed from 1703 to avoid conflict
pub const NATIVE_MATH_MIN_F: u16 = 1714;
pub const NATIVE_MATH_MAX_F: u16 = 1715;
pub const NATIVE_MATH_CLAMP: u16 = 1725;

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
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else if auto_val::is_null(nv) {
            // None value (null)
            vm_print(vm, "None");
        } else if auto_val::is_object(nv) {
            let handle = auto_val::decode_object(nv) as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_print(vm, &format!("<obj:{}>", handle));
                }
            } else {
                vm_print(vm, &format!("<invalid object: {}>", handle));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            // Boolean sentinel values: i32::MIN = true, i32::MIN+1 = false
            if val == -2147483648 {
                vm_print(vm, "1");
            } else if val == -2147483647 {
                vm_print(vm, "0");
            } else if val > 0 {
                // Check if positive value is a heap object handle (RustStdlibObject etc.)
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
            } else {
                vm_print(vm, &val.to_string());
            }
        }
        Ok(())
    }
}

pub fn shim_print_f32(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        // f64 occupies 2 slots (value at sp-2, marker at sp-1).
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
pub fn format_rust_stdlib_obj(obj: &RustStdlibObject) -> String {
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
        "semver::Version" => {
            if let Some(mutex) = obj.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                format!("{}", mutex.lock().unwrap())
            } else {
                "<semver::Version>".to_string()
            }
        }
        "semver::VersionReq" => {
            if let Some(mutex) = obj.downcast_ref::<std::sync::Mutex<semver::VersionReq>>() {
                format!("{}", mutex.lock().unwrap())
            } else {
                "<semver::VersionReq>".to_string()
            }
        }
        other => format!("<{}>", other),
    }
}

/// Print a string from the string constant pool, or an integer if not a string.
/// Expects tagged string index on TOS (LOAD_STR pushes -(idx+1)).
/// If the value is non-negative and not a valid string index, prints as integer.
pub fn shim_print_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_print(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_print(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else if auto_val::is_null(nv) {
            vm_print(vm, "None");
        } else if auto_val::is_object(nv) {
            let handle = auto_val::decode_object(nv) as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_print(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_print(vm, &format!("<obj:{}>", handle));
                }
            } else {
                vm_print(vm, &format!("<invalid object: {}>", handle));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            // Boolean sentinel values
            if val == -2147483648 {
                vm_print(vm, "1");
            } else if val == -2147483647 {
                vm_print(vm, "0");
            } else if val > 0 {
                // Check if positive value is a heap object handle
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
    {
        let nv = task.ram.pop_nv();
        if auto_val::is_string(nv) {
            let str_index = auto_val::decode_string(nv) as u16;
            if let Some(bytes) = vm.get_string(str_index) {
                vm_write(vm, &String::from_utf8_lossy(&bytes));
            } else {
                vm_write(vm, &format!("<invalid string index: {}>", str_index));
            }
        } else if auto_val::is_object(nv) {
            let handle = auto_val::decode_object(nv) as u64;
            if let Some(obj) = vm.get_heap_object(handle) {
                let guard = obj.read().unwrap();
                if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
                    vm_write(vm, &format_rust_stdlib_obj(rust_obj));
                } else {
                    vm_write(vm, &format!("<obj:{}>", handle));
                }
            } else {
                vm_write(vm, &format!("<invalid object: {}>", handle));
            }
        } else {
            let val = auto_val::decode_i32(nv);
            vm_write(vm, &val.to_string());
        }
    }
    Ok(())
}

pub fn shim_assert(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
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
    Ok(())
}

/// Runtime panic: pops a string (tagged index) from stack and returns it as an error.
/// Used by #[vm] function stubs when no matching native implementation is found.
pub fn shim_runtime_panic(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
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
    {
        let right_nv = task.ram.pop_nv();
        let left_nv = task.ram.pop_nv();

        let left_is_str = auto_val::is_string(left_nv);
        let right_is_str = auto_val::is_string(right_nv);
        let equal = if left_is_str && right_is_str {
            let lidx = auto_val::decode_string(left_nv) as u16;
            let ridx = auto_val::decode_string(right_nv) as u16;
            let left_str = vm.get_string(lidx)
                .map(|b| String::from_utf8_lossy(&b).to_string());
            let right_str = vm.get_string(ridx)
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
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_push(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let elem = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.push(elem);
            task.ram.push_i32(0);
            return Ok(());
        }
    }

    // Fallback: arrays DashMap (Vec<Value> from [...] literals)
    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let mut arr = arr_ref.write().unwrap();
        arr.push(Value::Int(elem));
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Pop an element from the end of the list.
/// Stack: list_id -> elem
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_pop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = task.ram.pop_i32() as u64;

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let elem = list.pop().unwrap_or(0);
            task.ram.push_i32(elem);
            return Ok(());
        }
    }

    // Fallback: arrays DashMap (Vec<Value> from [...] literals)
    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let mut arr = arr_ref.write().unwrap();
        if let Some(val) = arr.pop() {
            task.ram.push_i32(val.as_int());
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
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = task.ram.pop_i32() as u64;

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.len() as i32);
            return Ok(());
        }
    }

    // Fallback: arrays DashMap (Vec<Value> from [...] literals)
    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let arr = arr_ref.read().unwrap();
        task.ram.push_i32(arr.len() as i32);
        return Ok(());
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
/// Negative values are string tags that must use push_str_idx
/// to preserve the TAG_STRING type tag in the NanoValue encoding.
fn push_tagged_value(ram: &mut crate::vm::virt_memory::VirtualRAM, val: i32) {
    if val < 0 {
        let str_idx = (-(val) - 1) as u32;
        ram.push_nv(auto_val::encode_string(str_idx));
    } else {
        ram.push_i32(val);
    }
}

pub fn shim_list_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

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
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_insert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let elem = task.ram.pop_i32();
    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            list.insert(index, elem);
            task.ram.push_i32(0);
            return Ok(());
        }
    }

    // Fallback: arrays DashMap (Vec<Value> from [...] literals)
    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let mut arr = arr_ref.write().unwrap();
        let pos = index.min(arr.len());
        arr.insert(pos, Value::Int(elem));
    }

    // Return success (0)
    task.ram.push_i32(0);
    Ok(())
}

/// Remove element at index and return it.
/// Stack: list_id, index -> elem
// Plan 077 Phase 5: Updated to use unified registry
// Plan 289: Added arrays DashMap fallback for Value::Array literals
pub fn shim_list_remove(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let index = task.ram.pop_i32() as usize;
    let list_id = task.ram.pop_i32() as u64;

    // First try heap_objects (ListData<i32> from List.new())
    if let Some(obj) = vm.get_heap_object(list_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            if let Some(elem) = list.remove(index) {
                task.ram.push_i32(elem);
                return Ok(());
            }
        }
    }

    // Fallback: arrays DashMap (Vec<Value> from [...] literals)
    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let mut arr = arr_ref.write().unwrap();
        if index < arr.len() {
            let val = arr.remove(index);
            task.ram.push_i32(val.as_int());
            return Ok(());
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

/// Sort a list in-place.
/// Stack: list_id -> void
pub fn shim_list_sort(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let list_id = task.ram.pop_i32() as u64;

    // Try arrays registry first (most common path for List created by CREATE_ARRAY)
    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let mut arr = arr_ref.write().unwrap();
        arr.sort_by(|a, b| {
            match (a, b) {
                (auto_val::Value::Int(x), auto_val::Value::Int(y)) => x.cmp(y),
                (auto_val::Value::Uint(x), auto_val::Value::Uint(y)) => x.cmp(y),
                (auto_val::Value::Float(x), auto_val::Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                (auto_val::Value::Double(x), auto_val::Value::Double(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                (auto_val::Value::Bool(x), auto_val::Value::Bool(y)) => x.cmp(y),
                (auto_val::Value::Str(x), auto_val::Value::Str(y)) => x.to_string().cmp(&y.to_string()),
                (auto_val::Value::String(x), auto_val::Value::String(y)) => x.as_str().cmp(y.as_str()),
                (auto_val::Value::VmRef(x), auto_val::Value::VmRef(y)) => x.id.cmp(&y.id),
                _ => std::cmp::Ordering::Equal,
            }
        });
    } else if let Some(obj) = vm.get_heap_object(list_id) {
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

    if let Some(arr_ref) = vm.arrays.get(&list_id) {
        let mut arr = arr_ref.write().unwrap();
        arr.sort_by(|a, b| {
            match (a, b) {
                (auto_val::Value::Int(x), auto_val::Value::Int(y)) => x.cmp(y),
                (auto_val::Value::Uint(x), auto_val::Value::Uint(y)) => x.cmp(y),
                (auto_val::Value::Float(x), auto_val::Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                (auto_val::Value::Double(x), auto_val::Value::Double(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                (auto_val::Value::VmRef(x), auto_val::Value::VmRef(y)) => x.id.cmp(&y.id),
                _ => std::cmp::Ordering::Equal,
            }
        });
    } else if let Some(obj) = vm.get_heap_object(list_id) {
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
                        Iterator::Generator(_) => {
                            // Generator as Map source not yet supported
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
            Iterator::Generator(gen_state) => {
                // Plan 321 Phase 2: Generator next() driver — MVP approach.
                // Execute generator body directly on the caller's task using
                // save/restore of ip+bp. This avoids the !Send/blocking_lock
                // issue of spawning a separate task.
                //
                // NOTE: This is a simplified MVP that works for simple linear
                // generators. It does NOT support nested calls or complex
                // control flow within generators. Full task-based isolation
                // will be added later.
                if gen_state.done {
                    return Ok(());
                }

                // For MVP: we can't properly suspend/resume on the same task
                // without corrupting the caller's stack. So we use a simpler
                // approach: execute the entire generator body at once (like
                // a regular function), collecting all yield values into a list,
                // and return them one at a time.
                //
                // This is a "eager" generator — it runs fully on first next(),
                // stores results, and returns them one by one. Not truly lazy,
                // but correct for MVP.

                if !gen_state.started {
                    // First call: run the entire generator and collect results.
                    gen_state.started = true;
                    // Run via call_fn_by_name on the current task
                    // (the generator function body runs, YIELD_VALs are treated
                    // as "push value to a collection" — but we can't do that
                    // in the current architecture without a separate task).
                    //
                    // MVP workaround: use the task-based approach but with
                    // std::thread::spawn + try_lock instead of blocking_lock.
                    // Actually, let's use try_lock which returns immediately
                    // if the lock is held (it shouldn't be, since we're the
                    // only ones accessing this generator task).

                    let tid = vm.spawn_task(gen_state.func_addr as usize, 8192);
                    gen_state.task_id = Some(tid);

                    // Set up frame
                    if let Some(gen_task_arc) = vm.tasks.get(&tid) {
                        if let Ok(mut gt) = gen_task_arc.try_lock() {
                            gt.current_fn_n_args = gen_state.n_args as usize;
                            gt.ram.push_i32(0); // return addr
                            gt.ram.push_i32(0); // old BP
                            gt.bp = gt.ram.sp - 1;
                        }
                    }

                    // Collect ALL yielded values eagerly
                    let mut collected: Vec<i32> = Vec::new();
                    if let Some(gen_task_arc) = vm.tasks.get(&tid) {
                        // Use try_lock to avoid blocking_lock panic
                        loop {
                            match gen_task_arc.try_lock() {
                                Ok(mut gt) => {
                                    let budget = 1_000_000;
                                    let mut got_yield = false;
                                    for _ in 0..budget {
                                        match vm.run_one_instruction(&mut gt) {
                                            Ok(crate::vm::engine::StepResult::Continue) => continue,
                                            Ok(crate::vm::engine::StepResult::GeneratorYield) => {
                                                collected.push(auto_val::decode_i32(gt.ram.pop_nv()));
                                                got_yield = true;
                                                break;
                                            }
                                            Ok(crate::vm::engine::StepResult::Terminated) => break,
                                            Ok(crate::vm::engine::StepResult::Yield) => continue,
                                            Ok(crate::vm::engine::StepResult::AwaitFuture { .. }) => continue,
                                            Err(e) => {
                                                eprintln!("[Generator] Error: {:?}", e);
                                                break;
                                            }
                                        }
                                    }
                                    if !got_yield {
                                        // Check if Terminated was the result
                                        break;
                                    }
                                    drop(gt);
                                }
                                Err(_) => {
                                    // Lock contention — shouldn't happen for fresh task
                                    break;
                                }
                            }
                        }
                    }

                    // Clean up generator task
                    vm.tasks.remove(&tid);
                    gen_state.task_id = None;

                    // Store collected values in gen_state for sequential return
                    gen_state.collected = collected;
                    gen_state.collected_idx = 0;
                }

                // Return next collected value or -1 (done)
                let result = if gen_state.collected_idx < gen_state.collected.len() {
                    let v = gen_state.collected[gen_state.collected_idx];
                    gen_state.collected_idx += 1;
                    v
                } else {
                    gen_state.done = true;
                    -1
                };

                task.ram.push_i32(result);
                return Ok(());
            }
        }
    } else {
        // Fallback: if iterator_id is actually a heap list object, auto-create iterator
        if let Some(obj) = vm.get_heap_object(iterator_id as u64) {
            let guard = obj.read().unwrap();
            if guard.type_tag() == crate::vm::heap_object::TypeTag::ListInt {
                if let Some(_list_data) = guard.as_any().downcast_ref::<ListData<i32>>() {
                    // Create an iterator for this list on the fly
                    let list_iter = crate::vm::engine::ListIterator {
                        list_id: iterator_id as u64,
                        current_index: 0,
                    };
                    // Use iterator_id (the list's heap ID) as key so subsequent
                    // iterator_next calls find it with the same ID the loop passes
                    vm.iterators.insert(iterator_id, Iterator::List(list_iter));
                    drop(guard);
                    if let Some(mut iter_mut) = vm.iterators.get_mut(&iterator_id) {
                        if let Iterator::List(ref mut li) = *iter_mut {
                            if let Some(obj2) = vm.get_heap_object(li.list_id) {
                                let list = obj2.read().unwrap();
                                if let Some(list_data) = list.as_any().downcast_ref::<ListData<i32>>() {
                                    if li.current_index < list_data.len() as u32 {
                                        let elem = list_data.get(li.current_index as usize).copied().unwrap_or(0);
                                        li.current_index += 1;
                                        task.ram.push_i32(elem);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
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
            Iterator::Map(_) | Iterator::Filter(_) | Iterator::Enumerate(_) | Iterator::Generator(_) => {
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
            Iterator::Map(_) | Iterator::Filter(_) | Iterator::Enumerate(_) | Iterator::Generator(_) => {
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
            Iterator::Map(_) | Iterator::Filter(_) | Iterator::Enumerate(_) | Iterator::Generator(_) => {
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
    {
        let value_nv = task.ram.pop_nv();
        let key_nv = task.ram.pop_nv();
        let map_nv = task.ram.pop_nv();
        // Key should be a string pool reference. The tag can be
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
                        {
                            task.ram.push_nv(auto_val::encode_object(vm_ref.id as u32));
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
    {
        task.ram.push_nv(auto_val::encode_null());
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
    {
        task.ram.push_nv(auto_val::encode_null());
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
    {
        task.ram.push_nv(auto_val::encode_null());
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
}

/// Helper: decode a NanoValue to a String (from string pool)
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
    {
        let sub_nv = task.ram.pop_nv();
        let str_nv = task.ram.pop_nv();
        let str_s = nv_to_string(str_nv, vm);
        let sub_s = nv_to_string(sub_nv, vm);
        task.ram.push_i32(if str_s.contains(sub_s.as_str()) { 1 } else { 0 });
    }
    Ok(())
}

/// str.starts_with(prefix) — check if string starts with prefix
pub fn shim_str_starts_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let prefix_nv = task.ram.pop_nv();
        let str_nv = task.ram.pop_nv();
        let str_s = nv_to_string(str_nv, vm);
        let prefix_s = nv_to_string(prefix_nv, vm);
        task.ram.push_i32(if str_s.starts_with(prefix_s.as_str()) { 1 } else { 0 });
    }
    Ok(())
}

/// str.ends_with(suffix) — check if string ends with suffix
pub fn shim_str_ends_with(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let suffix_nv = task.ram.pop_nv();
        let str_nv = task.ram.pop_nv();
        let str_s = nv_to_string(str_nv, vm);
        let suffix_s = nv_to_string(suffix_nv, vm);
        task.ram.push_i32(if str_s.ends_with(suffix_s.as_str()) { 1 } else { 0 });
    }
    Ok(())
}

/// str.to_int() / str.parse_int() — parse string to int, return Result<int>
/// Stack: str -> CREATE_OK(int) or CREATE_ERR(str)
/// Returns the value as CREATE_OK for proper .?() chaining
pub fn shim_str_to_int_nv(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
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
    Ok(())
}

/// Get the length of a string (String.len).
/// Supports both constant pool strings (tagged index) and heap-based mutable Strings (SpecializedStringBuilder).
/// Stack: str_idx_or_sb_id -> length (as i32, char count for heap, byte count for const pool)
pub fn shim_string_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
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

/// rng.gen_range(range) -> i32
/// Stack: [range_or_hi, lo_or_rng_id, maybe_rng_id] -> [result]
/// Supports both range expression (single marker) and explicit (hi, lo) args.
pub fn shim_rng_gen_range(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let top = task.ram.pop_i32();

    // Check if top is a range marker (-1000000 + idx)
    let (lo, hi, rng_id) = if top <= -1000000 && top > -2000000 {
        let range_idx = (top + 1000000) as usize;
        if let Some(&(start, end, _eq)) = task.ram.ranges.get(range_idx) {
            let rng_id = task.ram.pop_i32() as u64;
            (start, end, rng_id)
        } else {
            let rng_id = task.ram.pop_i32() as u64;
            (0, 1, rng_id)
        }
    } else {
        // Legacy: hi, lo, rng_id layout
        let hi = top;
        let lo = task.ram.pop_i32();
        let rng_id = task.ram.pop_i32() as u64;
        (lo, hi, rng_id)
    };

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

/// Helper: pop a string from the VM stack
fn pop_vm_string(task: &mut AutoTask, vm: &AutoVM) -> String {
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
    let id = vm.insert_heap_object(list);
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
    let id = vm.insert_heap_object(list);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// url.query() → raw query string (without ?)
/// Stack: [handle_i32] -> [query_str]
pub fn shim_url_opaque_query(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let s = url.lock().unwrap().query().unwrap_or("").to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// url.to_string() → full URL string
/// Stack: [handle_i32] -> [url_str]
pub fn shim_url_opaque_to_string(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let url_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(url_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(url) = rso.downcast_ref::<std::sync::Mutex<url::Url>>() {
                let s = url.lock().unwrap().to_string();
                push_vm_string(task, vm, &s);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
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
            task.ram.push_nv(auto_val::encode_object(id as u32));
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("Version::parse failed: {}", e)));
        }
    }
    Ok(())
}
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

/// VersionReq.parse(">=1.0") → opaque VersionReq handle
/// Stack: [spec_str] -> [handle_i32]
pub fn shim_semver_opaque_versionreq_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let spec_str = pop_vm_string(task, vm);
    match semver::VersionReq::parse(&spec_str) {
        Ok(req) => {
            let obj = RustStdlibObject::new("semver::VersionReq", std::sync::Mutex::new(req));
            let id = vm.insert_heap_object(obj);
            task.ram.push_nv(auto_val::encode_object(id as u32));
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!("VersionReq::parse failed: {}", e)));
        }
    }
    Ok(())
}

/// req.matches(version_handle) → 1 if true, 0 if false
/// Stack: [req_handle, version_handle] -> [result_i32]
pub fn shim_semver_opaque_versionreq_matches(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = task.ram.pop_i32() as u64;
    let req_id = task.ram.pop_i32() as u64;

    let version: Option<semver::Version> = (|| {
        let obj = vm.get_heap_object(ver_id)?;
        let guard = obj.read().ok()?;
        let rso = guard.as_any().downcast_ref::<RustStdlibObject>()?;
        let mutex = rso.downcast_ref::<std::sync::Mutex<semver::Version>>()?;
        let locked = mutex.lock().ok()?;
        Some(locked.clone())
    })();

    let req: Option<semver::VersionReq> = (|| {
        let obj = vm.get_heap_object(req_id)?;
        let guard = obj.read().ok()?;
        let rso = guard.as_any().downcast_ref::<RustStdlibObject>()?;
        let mutex = rso.downcast_ref::<std::sync::Mutex<semver::VersionReq>>()?;
        let locked = mutex.lock().ok()?;
        Some(locked.clone())
    })();

    match (version, req) {
        (Some(ver), Some(req)) => {
            task.ram.push_i32(if req.matches(&ver) { 1 } else { 0 });
        }
        _ => {
            task.ram.push_i32(0);
        }
    }
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
                // Push as i32 if fits, otherwise as f64 for numeric comparison
                if ts >= i32::MIN as i64 && ts <= i32::MAX as i64 {
                    task.ram.push_i32(ts as i32);
                } else {
                    task.ram.push_i64(ts);
                }
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
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
    {
        let nv = auto_val::encode_string(idx as u32);
        task.ram.push_nv(nv);
        task.ram.push_nv(auto_val::encode_null());
    }
    Ok(())
}

// Plan 240: File I/O opaque shims

/// Helper to pop a string from the stack
fn pop_string(task: &mut AutoTask, vm: &AutoVM) -> String {
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
pub fn shim_file_try_clone(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
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

// ============================================================================
// Plan 250: Stdlib Enhancement — New Native Shims
// ============================================================================

/// Bool to string.
/// Stack: bool_val (i32, 0/1) -> str_idx
pub fn shim_bool_to_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    let s = if val != 0 { "true" } else { "false" };
    let str_idx = vm.add_string(s.as_bytes().to_vec());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// f64 to string.
/// Stack: f64_val -> str_idx
pub fn shim_f64_to_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_f64();
    let s = format!("{}", val);
    let str_idx = vm.add_string(s.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

// --- Result shims (2760-2766) ---
// Result<T,E> is represented as a tagged i32:
//   Ok(v)  => v >= 0 (the value itself)
//   Err(e) => e < 0  (negated error code)

/// Check if a Result is Ok.
/// Stack: result_val (i32) -> bool (i32)
pub fn shim_result_is_ok(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(if val >= 0 { 1 } else { 0 });
    Ok(())
}

/// Check if a Result is Err.
/// Stack: result_val (i32) -> bool (i32)
pub fn shim_result_is_err(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(if val < 0 { 1 } else { 0 });
    Ok(())
}

/// Unwrap Result — returns value if Ok, panics if Err.
/// Stack: result_val (i32) -> value (i32)
pub fn shim_result_unwrap(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    if val < 0 {
        return Err(VMError::RuntimeError("unwrap on Err result".to_string()));
    }
    task.ram.push_i32(val);
    Ok(())
}

/// Unwrap Result with default.
/// Stack: default (i32), result_val (i32) -> value (i32)
pub fn shim_result_unwrap_or(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let default = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(if val >= 0 { val } else { default });
    Ok(())
}

/// Unwrap the Err value from a Result.
/// Stack: result_val (i32) -> err_value (i32)
pub fn shim_result_unwrap_err(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    if val >= 0 {
        return Err(VMError::RuntimeError("unwrap_err on Ok result".to_string()));
    }
    task.ram.push_i32(val);
    Ok(())
}

/// Create Ok result — passthrough.
/// Stack: value (i32) -> result (i32)
pub fn shim_result_ok(_task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    Ok(())
}

/// Create Err result — negate to mark as error.
/// Stack: value (i32) -> result (i32)
pub fn shim_result_err(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(if val >= 0 { -val - 1 } else { val });
    Ok(())
}

// --- List Reverse (2770) ---

/// Reverse a list in-place.
/// Stack: list_handle -> void
pub fn shim_list_reverse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let handle = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid list handle in reverse".to_string()))?;
    let mut guard = obj.write().unwrap();
    if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
        list.elems.reverse();
    }
    task.ram.push_i32(0);
    Ok(())
}

// --- Random convenience (2780-2783) ---

/// Thread-local random int in [0, max).
/// Stack: max (i32) -> i32
pub fn shim_rand_int(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let max = task.ram.pop_i32();
    let max = if max <= 0 { 1 } else { max as u32 };
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    let val = (rng.next() % max as u64) as i32;
    task.ram.push_i32(val);
    Ok(())
}

/// Thread-local random float in [0, 1).
/// Stack: -> f64
pub fn shim_rand_float(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    let val = (rng.next() as f64) / (u64::MAX as f64);
    task.ram.push_f64(val);
    Ok(())
}

/// Thread-local random bool.
/// Stack: -> bool (i32)
pub fn shim_rand_bool(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = Xorshift64::new(seed);
    let val = rng.next() % 2 == 0;
    task.ram.push_i32(if val { 1 } else { 0 });
    Ok(())
}

/// Shuffle a list in-place using Fisher-Yates.
/// Stack: list_handle -> void
pub fn shim_rand_shuffle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let handle = task.ram.pop_i32() as u64;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid list handle in shuffle".to_string()))?;
    let mut guard = obj.write().unwrap();
    if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
        let mut rng = Xorshift64::new(seed);
        let mut i = list.elems.len();
        while i > 1 {
            i -= 1;
            let j = (rng.next() % (i as u64 + 1)) as usize;
            list.elems.swap(i, j);
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

// --- DateTime extended (2790-2792) ---
// These use the existing chrono_opaque infrastructure from stdlib.rs

/// Create DateTime from Unix timestamp.
/// Stack: timestamp (i32) -> opaque_handle
pub fn shim_chrono_from_timestamp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let ts = task.ram.pop_i32() as i64;
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap());
    let obj = RustStdlibObject::new("DateTime", std::sync::Mutex::new(dt));
    let handle = vm.insert_heap_object(obj) as i32;
    task.ram.push_i32(handle);
    Ok(())
}

/// Create DateTime from year/month/day.
/// Stack: day, month, year -> opaque_handle
pub fn shim_chrono_from_ymd(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let day = task.ram.pop_i32();
    let month = task.ram.pop_i32();
    let year = task.ram.pop_i32();
    let dt = chrono::NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0)).unwrap());
    let obj = RustStdlibObject::new("DateTime", std::sync::Mutex::new(dt));
    let handle = vm.insert_heap_object(obj) as i32;
    task.ram.push_i32(handle);
    Ok(())
}

/// Get weekday (0=Monday..6=Sunday).
/// Stack: opaque_handle -> i32
pub fn shim_chrono_weekday(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use chrono::Datelike;
    let handle = task.ram.pop_i32() as u64;
    let obj = vm.get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid DateTime handle".to_string()))?;
    let guard = obj.read().unwrap();
    if let Some(rust_obj) = guard.as_any().downcast_ref::<RustStdlibObject>() {
        if let Some(dt) = rust_obj.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
            task.ram.push_i32(dt.lock().unwrap().weekday().num_days_from_monday() as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

// --- CSV (2800-2803) ---
// CSV functions return JSON strings, matching the pattern used by file.walk etc.

/// Parse CSV text into JSON array of arrays.
/// Stack: str_idx -> str_idx (JSON)
pub fn shim_csv_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u16;
    let text = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let rows: Vec<Vec<String>> = text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split(',').map(|f| f.trim().to_string()).collect()
        })
        .collect();
    let json = serde_json::to_string(&rows)
        .map_err(|e| VMError::RuntimeError(format!("csv_parse: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Parse CSV text with custom delimiter.
/// Stack: str_idx(delimiter), str_idx(text) -> str_idx (JSON)
pub fn shim_csv_parse_delim(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delim_idx = task.ram.pop_str_idx() as u16;
    let text_idx = task.ram.pop_str_idx() as u16;
    let delim = if let Some(bytes) = vm.get_string(delim_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        ",".to_string()
    };
    let text = if let Some(bytes) = vm.get_string(text_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let delim_char = delim.chars().next().unwrap_or(',');
    let rows: Vec<Vec<String>> = text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split(delim_char).map(|f| f.trim().to_string()).collect()
        })
        .collect();
    let json = serde_json::to_string(&rows)
        .map_err(|e| VMError::RuntimeError(format!("csv_parse_delim: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Encode list of list of strings (JSON) to CSV.
/// Stack: str_idx (JSON) -> str_idx (CSV)
pub fn shim_csv_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let json_idx = task.ram.pop_str_idx() as u16;
    let json = if let Some(bytes) = vm.get_string(json_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let rows: Vec<Vec<String>> = serde_json::from_str(&json)
        .map_err(|e| VMError::RuntimeError(format!("csv_encode: {}", e)))?;
    let mut result = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 { result.push('\n'); }
        result.push_str(&row.join(","));
    }
    let str_idx = vm.add_string(result.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Encode CSV with custom delimiter.
/// Stack: str_idx(delim), str_idx(JSON) -> str_idx (CSV)
pub fn shim_csv_encode_delim(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delim_idx = task.ram.pop_str_idx() as u16;
    let json_idx = task.ram.pop_str_idx() as u16;
    let delim = if let Some(bytes) = vm.get_string(delim_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        ",".to_string()
    };
    let json = if let Some(bytes) = vm.get_string(json_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let rows: Vec<Vec<String>> = serde_json::from_str(&json)
        .map_err(|e| VMError::RuntimeError(format!("csv_encode_delim: {}", e)))?;
    let mut result = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 { result.push('\n'); }
        result.push_str(&row.join(&delim));
    }
    let str_idx = vm.add_string(result.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

// --- Hashing (2810-2813) ---

/// MD5 hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_md5(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u16;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    // Simple MD5 implementation using digest
    let digest = md5_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// SHA1 hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_sha1(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u16;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    let digest = sha1_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// SHA256 one-shot hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_sha256(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u16;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    let digest = sha256_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// SHA512 hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_sha512(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u16;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    let digest = sha512_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

// Simple hash implementations using pure Rust
fn md5_hash(data: &[u8]) -> String {
    use md5::{Md5, Digest};
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

fn sha1_hash(data: &[u8]) -> String {
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

fn sha256_hash(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

fn sha512_hash(data: &[u8]) -> String {
    use sha2::{Sha512, Digest};
    let mut hasher = Sha512::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

// --- Test assertions (2820-2825) ---

/// Assert true.
/// Stack: message_str_idx, condition (i32) -> void
pub fn shim_test_assert_true(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u16;
    let condition = task.ram.pop_i32();
    if condition == 0 {
        let msg = if let Some(bytes) = vm.get_string(msg_idx) {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            "assertion failed".to_string()
        };
        return Err(VMError::RuntimeError(format!("assert_true failed: {}", msg)));
    }
    Ok(())
}

/// Assert false.
/// Stack: message_str_idx, condition (i32) -> void
pub fn shim_test_assert_false(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u16;
    let condition = task.ram.pop_i32();
    if condition != 0 {
        let msg = if let Some(bytes) = vm.get_string(msg_idx) {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            "assertion failed".to_string()
        };
        return Err(VMError::RuntimeError(format!("assert_false failed: {}", msg)));
    }
    Ok(())
}

/// Assert string contains.
/// Stack: message_str_idx, needle_str_idx, haystack_str_idx -> void
pub fn shim_test_assert_contains(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u16;
    let needle_idx = task.ram.pop_str_idx() as u16;
    let haystack_idx = task.ram.pop_str_idx() as u16;
    let haystack = vm.get_string(haystack_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let needle = vm.get_string(needle_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let msg = vm.get_string(msg_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    if !haystack.contains(&needle) {
        return Err(VMError::RuntimeError(format!("assert_contains failed: '{}' not found in '{}' — {}", needle, haystack, msg)));
    }
    Ok(())
}

/// Assert list length.
/// Stack: message_str_idx, expected_len (i32), list_handle -> void
pub fn shim_test_assert_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let msg_idx = task.ram.pop_str_idx() as u16;
    let expected = task.ram.pop_i32();
    let handle = task.ram.pop_i32() as u64;
    let msg = vm.get_string(msg_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let len = {
        let obj = vm.get_heap_object(handle)
            .ok_or_else(|| VMError::RuntimeError("Invalid list handle in assert_len".to_string()))?;
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            list.elems.len() as i32
        } else {
            -1
        }
    };
    if len != expected {
        return Err(VMError::RuntimeError(format!("assert_len failed: expected {} got {} — {}", expected, len, msg)));
    }
    Ok(())
}

/// Assert result is Ok.
/// Stack: message_str_idx, result_val -> void
pub fn shim_test_assert_ok(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u16;
    let val = task.ram.pop_i32();
    let msg = vm.get_string(msg_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    if val < 0 {
        return Err(VMError::RuntimeError(format!("assert_ok failed: result is Err — {}", msg)));
    }
    Ok(())
}

/// Assert result is Err.
/// Stack: message_str_idx, result_val -> void
pub fn shim_test_assert_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg_idx = task.ram.pop_str_idx() as u16;
    let val = task.ram.pop_i32();
    let msg = vm.get_string(msg_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    if val >= 0 {
        return Err(VMError::RuntimeError(format!("assert_err failed: result is Ok — {}", msg)));
    }
    Ok(())
}

// --- Format (2830-2832) ---

/// sprintf — format string with positional arguments.
/// Stack: ...args..., arg_count (i32), format_str_idx -> str_idx
pub fn shim_fmt_sprintf(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let arg_count = task.ram.pop_i32();
    let fmt_idx = task.ram.pop_str_idx() as u16;
    let fmt_str = vm.get_string(fmt_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();

    // Pop arguments in reverse order
    let mut args: Vec<String> = Vec::new();
    for _ in 0..arg_count {
        // Try to pop as string (simplified — real impl would need type info)
        let val = task.ram.pop_i32();
        args.push(val.to_string());
    }
    args.reverse();

    // Simple {} replacement
    let mut result = fmt_str;
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, &arg);
        }
    }

    let str_idx = vm.add_string(result.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// printf — format and print to stdout.
pub fn shim_fmt_printf(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let arg_count = task.ram.pop_i32();
    let fmt_idx = task.ram.pop_str_idx() as u16;
    let fmt_str = vm.get_string(fmt_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();

    let mut args: Vec<String> = Vec::new();
    for _ in 0..arg_count {
        let val = task.ram.pop_i32();
        args.push(val.to_string());
    }
    args.reverse();

    let mut result = fmt_str;
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, &arg);
        }
    }

    print!("{}", result);
    Ok(())
}

/// eprintf — format and print to stderr.
pub fn shim_fmt_eprintf(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let arg_count = task.ram.pop_i32();
    let fmt_idx = task.ram.pop_str_idx() as u16;
    let fmt_str = vm.get_string(fmt_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();

    let mut args: Vec<String> = Vec::new();
    for _ in 0..arg_count {
        let val = task.ram.pop_i32();
        args.push(val.to_string());
    }
    args.reverse();

    let mut result = fmt_str;
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, &arg);
        }
    }

    eprint!("{}", result);
    Ok(())
}

// --- FS extended (2840-2847) ---

/// Get system temp directory.
/// Stack: -> str_idx
pub fn shim_fs_temp_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let temp = std::env::temp_dir();
    let s = temp.to_string_lossy().to_string();
    let str_idx = vm.add_string(s.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Create a temporary file and return its path.
/// Stack: -> str_idx
pub fn shim_fs_temp_file(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let temp_dir = std::env::temp_dir();
    let id = std::sync::atomic::AtomicU64::new(0).fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = temp_dir.join(format!("auto_tmp_{}", id));
    // Create the file
    std::fs::File::create(&path).map_err(|e| VMError::RuntimeError(format!("temp_file failed: {}", e)))?;
    let s = path.to_string_lossy().to_string();
    let str_idx = vm.add_string(s.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Rename a file or directory.
/// Stack: str_idx(new), str_idx(old) -> void
pub fn shim_fs_rename(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let new_idx = task.ram.pop_str_idx() as u16;
    let old_idx = task.ram.pop_str_idx() as u16;
    let old_path = vm.get_string(old_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let new_path = vm.get_string(new_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    std::fs::rename(&old_path, &new_path)
        .map_err(|e| VMError::RuntimeError(format!("rename failed: {} -> {}: {}", old_path, new_path, e)))?;
    task.ram.push_i32(0);
    Ok(())
}

/// Read directory entries (non-recursive) — returns JSON array of filenames.
/// Stack: str_idx(path) -> str_idx (JSON)
pub fn shim_fs_read_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let entries: Vec<String> = std::fs::read_dir(&path)
        .map_err(|e| VMError::RuntimeError(format!("read_dir failed: {}: {}", path, e)))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let json = serde_json::to_string(&entries)
        .map_err(|e| VMError::RuntimeError(format!("read_dir json: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Canonicalize a path (absolute, resolved symlinks).
/// Stack: str_idx -> str_idx
pub fn shim_fs_canonical(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let canonical = std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(path);
    let str_idx = vm.add_string(canonical.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Get file extension.
/// Stack: str_idx -> str_idx
pub fn shim_fs_ext(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let ext = std::path::Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(ext.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Get filename stem (name without extension).
/// Stack: str_idx -> str_idx
pub fn shim_fs_stem(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let stem = std::path::Path::new(&path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(stem.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Walk directory recursively, returning only files — JSON array of paths.
/// Stack: str_idx(dir) -> str_idx (JSON)
pub fn shim_fs_walk_files(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dir_idx = task.ram.pop_str_idx() as u16;
    let dir = vm.get_string(dir_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let mut files: Vec<String> = Vec::new();
    if let Ok(entries) = walkdir_recursive(&dir) {
        files = entries;
    }
    let json = serde_json::to_string(&files)
        .map_err(|e| VMError::RuntimeError(format!("walk_files json: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Simple recursive directory walker.
fn walkdir_recursive(dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let sub = walkdir_recursive(&path.to_string_lossy())?;
            result.extend(sub);
        } else {
            result.push(path.to_string_lossy().to_string());
        }
    }
    Ok(result)
}

// --- FS more (2860-2865) ---

/// Walk directory recursively, return JSON array of ALL paths (files + dirs).
/// Stack: str_idx(dir) -> str_idx (JSON)
pub fn shim_fs_walk(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dir_idx = task.ram.pop_str_idx() as u16;
    let dir = vm.get_string(dir_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let mut paths: Vec<String> = Vec::new();
    if let Ok(entries) = walkdir_all(&dir) {
        paths = entries;
    }
    let json = serde_json::to_string(&paths)
        .map_err(|e| VMError::RuntimeError(format!("walk json: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Recursive directory walker returning ALL paths (files + dirs).
fn walkdir_all(dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        result.push(path_str.clone());
        if path.is_dir() {
            let sub = walkdir_all(&path.to_string_lossy())?;
            result.extend(sub);
        }
    }
    Ok(result)
}

/// Get file metadata as JSON string.
/// Stack: str_idx(path) -> str_idx (JSON with len, is_dir, is_file, readonly)
pub fn shim_fs_metadata(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let metadata = std::fs::metadata(&path)
        .map_err(|e| VMError::RuntimeError(format!("metadata failed: {}: {}", path, e)))?;
    let json = serde_json::json!({
        "len": metadata.len(),
        "is_dir": metadata.is_dir(),
        "is_file": metadata.is_file(),
        "readonly": metadata.permissions().readonly(),
    });
    let result = serde_json::to_string(&json)
        .map_err(|e| VMError::RuntimeError(format!("metadata json: {}", e)))?;
    let str_idx = vm.add_string(result.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Recursive copy of directory or file.
/// Stack: str_idx(dst), str_idx(src) -> void
pub fn shim_fs_copy_recursive(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dst_idx = task.ram.pop_str_idx() as u16;
    let src_idx = task.ram.pop_str_idx() as u16;
    let dst = vm.get_string(dst_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let src = vm.get_string(src_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    copy_recursive(&src, &dst)
        .map_err(|e| VMError::RuntimeError(format!("copy_recursive failed: {}", e)))?;
    Ok(())
}

fn copy_recursive(src: &str, dst: &str) -> Result<(), std::io::Error> {
    let src_path = std::path::Path::new(src);
    if src_path.is_dir() {
        let dst_path = std::path::Path::new(dst);
        std::fs::create_dir_all(dst_path)?;
        for entry in std::fs::read_dir(src_path)? {
            let entry = entry?;
            let src_child = entry.path();
            let dst_child = dst_path.join(entry.file_name());
            copy_recursive(&src_child.to_string_lossy(), &dst_child.to_string_lossy())?;
        }
    } else {
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

/// Extract filename from path.
/// Stack: str_idx(path) -> str_idx (filename)
pub fn shim_fs_filename(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let filename = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(filename.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Get parent directory of path.
/// Stack: str_idx(path) -> str_idx (parent)
pub fn shim_fs_parent(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let parent = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(parent.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// Join path components.
/// Stack: str_idx(b), str_idx(a) -> str_idx (joined path)
pub fn shim_fs_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let b_idx = task.ram.pop_str_idx() as u16;
    let a_idx = task.ram.pop_str_idx() as u16;
    let b = vm.get_string(b_idx).map(|bytes| String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
    let a = vm.get_string(a_idx).map(|bytes| String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
    let joined = std::path::Path::new(&a).join(&b).to_string_lossy().to_string();
    let str_idx = vm.add_string(joined.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

// --- Hash extended (2814-2816) ---

/// HMAC-SHA256.
/// Stack: str_idx(key), str_idx(data) -> str_idx (hex digest)
pub fn shim_hash_hmac_sha256(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_idx = task.ram.pop_str_idx() as u16;
    let data_idx = task.ram.pop_str_idx() as u16;
    let key = vm.get_string(key_idx).unwrap_or_default();
    let data = vm.get_string(data_idx).unwrap_or_default();
    let digest = hmac_sha256_hash(&key, &data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// MD5 hash of file contents.
/// Stack: str_idx(path) -> str_idx (hex digest)
pub fn shim_hash_file_md5(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let data = std::fs::read(&path)
        .map_err(|e| VMError::RuntimeError(format!("file_md5: {}: {}", path, e)))?;
    let digest = md5_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

/// SHA256 hash of file contents.
/// Stack: str_idx(path) -> str_idx (hex digest)
pub fn shim_hash_file_sha256(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u16;
    let path = vm.get_string(path_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let data = std::fs::read(&path)
        .map_err(|e| VMError::RuntimeError(format!("file_sha256: {}: {}", path, e)))?;
    let digest = sha256_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

fn hmac_sha256_hash(key: &[u8], data: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    let result = mac.finalize();
    format!("{:02x}", result.into_bytes())
}

// --- Random type (2870-2874) ---

/// Create RNG from system entropy.
/// Stack: [] -> rng_handle (i32)
pub fn shim_random_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let rng = std::sync::Mutex::new(Xorshift64::new(seed));
    let obj = RustStdlibObject::new("random::Rng", rng);
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// Create seeded RNG.
/// Stack: seed (i32) -> rng_handle (i32)
pub fn shim_random_seeded(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let seed = task.ram.pop_i32() as u64;
    let rng = std::sync::Mutex::new(Xorshift64::new(seed));
    let obj = RustStdlibObject::new("random::Rng", rng);
    let id = vm.insert_heap_object(obj);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// Instance method: rng.int(max) -> i32
/// Stack: max (i32), rng_handle (i32) -> i32
pub fn shim_random_instance_int(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let max = task.ram.pop_i32();
    let rng_id = task.ram.pop_i32() as u64;
    let max = if max <= 0 { 1 } else { max as u32 };
    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = rng.get_mut().unwrap().next();
                let result = (val % max as u64) as i32;
                task.ram.push_i32(result);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// Instance method: rng.float() -> f64
/// Stack: rng_handle (i32) -> f64
pub fn shim_random_instance_float(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let rng_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = (rng.get_mut().unwrap().next() as f64) / (u64::MAX as f64);
                task.ram.push_f64(val);
                return Ok(());
            }
        }
    }
    task.ram.push_f64(0.0);
    Ok(())
}

/// Instance method: rng.bool() -> bool (i32)
/// Stack: rng_handle (i32) -> i32 (0 or 1)
pub fn shim_random_instance_bool(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    let rng_id = task.ram.pop_i32() as u64;
    if let Some(obj) = vm.get_heap_object(rng_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(rng) = rso.downcast_mut::<std::sync::Mutex<Xorshift64>>() {
                let val = rng.get_mut().unwrap().next() % 2 == 0;
                task.ram.push_i32(if val { 1 } else { 0 });
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

// --- Fmt (2752) ---

/// Float debug string with full precision.
/// Stack: f64_val -> str_idx
pub fn shim_f64_debug(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_f64();
    let s = format!("{:?}", val);
    let str_idx = vm.add_string(s.into_bytes());
    task.ram.push_str_idx(str_idx as u32);
    Ok(())
}

// --- Cmp (2880) ---

/// String lexicographic comparison, returns Ordering (-1, 0, 1).
/// Stack: str_idx(b), str_idx(a) -> i32 (-1, 0, 1)
pub fn shim_str_cmp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let b_idx = task.ram.pop_str_idx() as u16;
    let a_idx = task.ram.pop_str_idx() as u16;
    let a = vm.get_string(a_idx).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let b = vm.get_string(b_idx).map(|bytes| String::from_utf8_lossy(&bytes).to_string()).unwrap_or_default();
    use std::cmp::Ordering;
    let result = match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    };
    task.ram.push_i32(result);
    Ok(())
}

// --- DateTime cmp (2794) ---

/// Compare two datetimes by timestamp.
/// Stack: handle_b (i32), handle_a (i32) -> i32 (-1, 0, 1)
pub fn shim_datetime_cmp(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let b_id = task.ram.pop_i32() as u64;
    let a_id = task.ram.pop_i32() as u64;

    let ts_a = get_datetime_timestamp(vm, a_id);
    let ts_b = get_datetime_timestamp(vm, b_id);

    use std::cmp::Ordering;
    let result = match ts_a.cmp(&ts_b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    };
    task.ram.push_i32(result);
    Ok(())
}

fn get_datetime_timestamp(vm: &AutoVM, handle: u64) -> i64 {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    if let Some(obj) = vm.get_heap_object(handle) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                return dt.lock().unwrap().and_utc().timestamp();
            }
        }
    }
    0
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
