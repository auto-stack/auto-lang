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
        self.register(NATIVE_PRINT_I32, shim_print_i32);
        self.register(NATIVE_PRINT_F32, shim_print_f32);
        self.register(NATIVE_PRINT_STR, shim_print_str);
    }
}

pub const NATIVE_PRINT_I32: u16 = 1;
pub const NATIVE_PRINT_F32: u16 = 2;
pub const NATIVE_PRINT_STR: u16 = 3;

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
