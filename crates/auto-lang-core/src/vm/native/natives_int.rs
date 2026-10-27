//! PLAN-843 结构拆分：native int 域实现（自 native.rs 原样搬移）。
use super::*;

/// int.and(mask) — Bitwise AND: val & mask
/// Stack: [self, mask] -> result
pub(super) fn shim_int_and(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let mask = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val & mask);
    Ok(())
}

/// int.or(mask) — Bitwise OR: val | mask
pub(super) fn shim_int_or(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let mask = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val | mask);
    Ok(())
}

/// int.xor(mask) — Bitwise XOR: val ^ mask
pub(super) fn shim_int_xor(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let mask = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val ^ mask);
    Ok(())
}

/// int.not() — Bitwise NOT: ~val
/// Stack: [self] -> result
pub(super) fn shim_int_not(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(!val);
    Ok(())
}

/// int.shl(n) — Logical left shift: val << n
/// For n >= 32, the result is 0 (all bits shifted out).
pub(super) fn shim_int_shl(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    // wrapping_shl masks n to 5 bits, so n>=32 wraps to n & 31. Clamp to 0 instead.
    let result = if n >= 32 || n < 0 {
        0
    } else {
        val.wrapping_shl(n as u32)
    };
    task.ram.push_i32(result);
    Ok(())
}

/// int.shr(n) — Logical right shift: val >> n (unsigned)
/// For n >= 32, the result is 0 (all bits shifted out).
pub(super) fn shim_int_shr(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    // wrapping_shr masks n to 5 bits, so n>=32 wraps to n & 31. Clamp to 0 instead.
    let result = if n >= 32 || n < 0 {
        0
    } else {
        (val as u32 >> n as u32) as i32
    };
    task.ram.push_i32(result);
    Ok(())
}

/// int.sar(n) — Arithmetic right shift: val >> n (preserves sign)
/// For n >= 32, the result is sign-filled (0 if non-negative, -1 if negative).
pub(super) fn shim_int_sar(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    // wrapping_shr masks n to 5 bits, so n>=32 wraps to n & 31. Sign-fill instead.
    let result = if n >= 32 || n < 0 {
        if val < 0 {
            -1
        } else {
            0
        }
    } else {
        val >> n
    };
    task.ram.push_i32(result);
    Ok(())
}

/// int.rol(n) — Rotate left
pub(super) fn shim_int_rol(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.rotate_left(n as u32));
    Ok(())
}

/// int.ror(n) — Rotate right
pub(super) fn shim_int_ror(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.rotate_right(n as u32));
    Ok(())
}

/// int.count_ones() — Population count
pub(super) fn shim_int_count_ones(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.count_ones() as i32);
    Ok(())
}

/// int.leading_zeros() — Count leading zeros
pub(super) fn shim_int_leading_zeros(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.leading_zeros() as i32);
    Ok(())
}

/// int.trailing_zeros() — Count trailing zeros
pub(super) fn shim_int_trailing_zeros(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.trailing_zeros() as i32);
    Ok(())
}

/// int.flip() — Bit-reverse (mirror all bits)
pub(super) fn shim_int_bitrev(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_i32();
    task.ram.push_i32(val.reverse_bits());
    Ok(())
}

/// int.bits_read(start, len) — Read bitfield: (val >> start) & ((1 << len) - 1)
pub(super) fn shim_int_bit_read(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let len = task.ram.pop_i32();
    let start = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    let mask = if len >= 32 { -1 } else { (1i32 << len) - 1 };
    // wrapping_shr masks start to 5 bits, so start>=32 wraps to start & 31. Clamp to 0 instead.
    let shifted = if start >= 32 || start < 0 {
        0
    } else {
        (val as u32 >> start as u32) as i32
    };
    task.ram.push_i32(shifted & mask);
    Ok(())
}

/// int.bit_test(n) — Test bit n: (val >> n) & 1 → bool (1 or 0)
pub(super) fn shim_int_bit_test(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    // For n >= 32 or n < 0, the bit does not exist in an i32, so the result is false.
    let result = !(n >= 32 || n < 0) && ((val >> n) & 1) != 0;
    task.ram.push_nv(auto_val::encode_bool(result));
    Ok(())
}

/// int.bit_on(n) — Set bit n: val | (1 << n)
/// For n outside [0, 31], the bit does not exist in an i32, so val is returned unchanged.
pub(super) fn shim_int_bit_on(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    let result = if n >= 0 && n < 32 {
        val | (1 << n)
    } else {
        val
    };
    task.ram.push_i32(result);
    Ok(())
}

/// int.bit_off(n) — Clear bit n: val & !(1 << n)
/// For n outside [0, 31], the bit does not exist in an i32, so val is returned unchanged.
pub(super) fn shim_int_bit_off(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    let result = if n >= 0 && n < 32 {
        val & !(1 << n)
    } else {
        val
    };
    task.ram.push_i32(result);
    Ok(())
}

/// int.bit_flip(n) — Toggle bit n: val ^ (1 << n)
/// For n outside [0, 31], the bit does not exist in an i32, so val is returned unchanged.
pub(super) fn shim_int_bit_flip(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let n = task.ram.pop_i32();
    let val = task.ram.pop_i32();
    let result = if n >= 0 && n < 32 {
        val ^ (1 << n)
    } else {
        val
    };
    task.ram.push_i32(result);
    Ok(())
}

pub fn shim_math_powi(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let exp = crate::vm::native::pop_arg_i32(task);

    let _stake_exp = crate::vm::native::StakeGuard::new(vm, exp as i64 as u64);
    let base = task.ram.pop_f64();
    task.ram.push_f64(base.powi(exp));
    Ok(())
}

pub fn shim_math_atan2(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let x = task.ram.pop_f64();
    let y = task.ram.pop_f64();
    task.ram.push_f64(y.atan2(x));
    Ok(())
}

/// Plan 118 Phase 4: Convert integer to string.
/// Stack: int_val (i32) -> str_idx (tagged)
pub fn shim_int_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop integer value
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);

    // Convert to string
    let str_val = val.to_string();
    let bytes = str_val.into_bytes();

    // Add to string pool
    let str_idx = vm.add_string(bytes);

    // Return tagged string index
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// uint.to_hex(pad) -> hex string
/// Formats a u64 value as a zero-padded lowercase hex string.
/// Stack: pad_width (i32), val_lo (i32), val_hi (i32) -> str_idx (tagged)
pub fn shim_uint_to_hex(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // u64 is stored as two i32 slots: low pushed first, high pushed second (on top)
    let pad_width = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_pad_width = crate::vm::native::StakeGuard::new(vm, pad_width as i64 as u64);
    let val = task.ram.pop_u64();

    let hex_str = format!("{:0width$x}", val, width = pad_width);
    let bytes = hex_str.into_bytes();
    let str_idx = vm.add_string(bytes);

    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Bool to string.
/// Stack: bool_val -> str_idx
pub fn shim_bool_to_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let nv = crate::vm::native::pop_arg_nv(task);

    let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
    // Prefer the TAG_BOOL encoding; fall back to legacy i32 truthiness
    // (0 / i32::MIN+1 = false, anything else = true) for values pushed
    // by older code paths that still use push_i32.
    let is_true = if auto_val::is_bool(nv) {
        auto_val::decode_bool(nv)
    } else {
        let v = auto_val::decode_i32(nv);
        v != 0 && v != -2147483647
    };
    let s = if is_true { "true" } else { "false" };
    let str_idx = vm.add_string(s.as_bytes().to_vec());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// f64 to string.
/// Stack: f64_val -> str_idx
pub fn shim_f64_to_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_f64();
    let s = format!("{}", val);
    let str_idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// sprintf — format string with positional arguments.
/// Stack: ...args..., arg_count (i32), format_str_idx -> str_idx
pub fn shim_fmt_sprintf(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let arg_count = crate::vm::native::pop_arg_i32(task);

    let _stake_arg_count = crate::vm::native::StakeGuard::new(vm, arg_count as i64 as u64);
    let fmt_idx = task.ram.pop_str_idx() as u32;
    let fmt_str = vm
        .get_string(fmt_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();

    // Pop arguments in reverse order
    let mut args: Vec<String> = Vec::new();
    for _ in 0..arg_count {
        // Try to pop as string (simplified — real impl would need type info)
        let val = crate::vm::native::pop_arg_i32(task);

        let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
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
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// printf — format and print to stdout.
pub fn shim_fmt_printf(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let arg_count = crate::vm::native::pop_arg_i32(task);

    let _stake_arg_count = crate::vm::native::StakeGuard::new(vm, arg_count as i64 as u64);
    let fmt_idx = task.ram.pop_str_idx() as u32;
    let fmt_str = vm
        .get_string(fmt_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();

    let mut args: Vec<String> = Vec::new();
    for _ in 0..arg_count {
        let val = crate::vm::native::pop_arg_i32(task);

        let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
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
    let arg_count = crate::vm::native::pop_arg_i32(task);

    let _stake_arg_count = crate::vm::native::StakeGuard::new(vm, arg_count as i64 as u64);
    let fmt_idx = task.ram.pop_str_idx() as u32;
    let fmt_str = vm
        .get_string(fmt_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();

    let mut args: Vec<String> = Vec::new();
    for _ in 0..arg_count {
        let val = crate::vm::native::pop_arg_i32(task);

        let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
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

/// Float debug string with full precision.
/// Stack: f64_val -> str_idx
pub fn shim_f64_debug(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = task.ram.pop_f64();
    let s = format!("{:?}", val);
    let str_idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}
