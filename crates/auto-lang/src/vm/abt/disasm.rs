//! Plan 226: ABC → ABT disassembler
//!
//! Converts binary bytecode into a structured `AbtProgram` with labels and metadata.

use crate::vm::abt::{AbtInstruction, AbtOperand, AbtProgram};
use crate::vm::opcode::OpCode;
use crate::vm::virt_memory::VirtualFlash;
use std::collections::{HashMap, HashSet};

/// Disassemble a `VirtualFlash` into an `AbtProgram`.
///
/// `strings` is the optional string constant pool (from `AutoVM.strings` or `CompiledPackage.string_pool`).
/// If provided, the `.strings` section will be populated in the output.
pub fn disassemble_flash(flash: &VirtualFlash, strings: Option<&[Vec<u8>]>) -> AbtProgram {
    let end = flash.memory.len();

    // === Pass 1: Collect all jump/call targets ===
    let mut targets = HashSet::new();
    collect_targets(flash, 0, end, &mut targets);

    // Add export entry points as targets
    for (&addr, _) in &flash.addr_to_name {
        targets.insert(addr as usize);
    }
    for (addr, _) in flash.exports_by_name.values().map(|&a| (a, ())) {
        targets.insert(addr as usize);
    }

    // === Pass 2: Generate labels ===
    let mut labels = HashMap::new();
    // Export names get priority
    for (&addr, name) in &flash.addr_to_name {
        labels.insert(sanitize_label(name), addr as usize);
    }
    for (name, &addr) in &flash.exports_by_name {
        let label = sanitize_label(name);
        if !labels.values().any(|&v| v == addr as usize) {
            labels.insert(label, addr as usize);
        }
    }

    // Remaining targets get synthetic labels
    let mut sorted_targets: Vec<_> = targets.iter().copied().collect();
    sorted_targets.sort();
    for target in &sorted_targets {
        if !labels.values().any(|&v| v == *target) {
            labels.insert(format!("L_{:04x}", target), *target);
        }
    }

    // === Pass 3: Disassemble into AbtInstructions ===
    let mut code = Vec::new();
    let mut ip = 0;
    let mut current_line: Option<u32> = None;

    while ip < end {
        let offset = ip;
        let op_byte = flash.read_u8(ip);
        ip += 1;

        if !OpCode::is_valid(op_byte) {
            code.push(AbtInstruction {
                offset,
                opcode: OpCode::NOP, // placeholder
                operands: vec![AbtOperand::Bytes(vec![op_byte])],
                source_line: current_line,
            });
            continue;
        }

        let op: OpCode = op_byte.into();

        if op == OpCode::SOURCE_LINE {
            let line = flash.read_u16(ip);
            ip += 2;
            current_line = Some(line as u32);
            code.push(AbtInstruction {
                offset,
                opcode: OpCode::SOURCE_LINE,
                operands: vec![AbtOperand::ImmU16(line)],
                source_line: current_line,
            });
            continue;
        }

        let (operands, advance) = decode_operands(flash, op, ip, offset, &labels);
        ip += advance;

        code.push(AbtInstruction {
            offset,
            opcode: op,
            operands,
            source_line: current_line,
        });
    }

    // === Build metadata ===
    let strings: Vec<String> = strings
        .map(|pool| pool.iter().map(|b| String::from_utf8_lossy(b).to_string()).collect())
        .unwrap_or_default();
    let exports: Vec<(String, String)> = flash
        .exports_by_name
        .iter()
        .map(|(name, &addr)| {
            let target = labels.iter().find(|(_, &o)| o == addr as usize).map(|(l, _)| l.clone())
                .unwrap_or_else(|| format!("0x{:04x}", addr));
            (name.clone(), target)
        })
        .collect();
    let object_keys: Vec<Vec<String>> = flash
        .object_keys
        .iter()
        .map(|keys| keys.iter().map(|k| k.to_string()).collect())
        .collect();
    let object_types = flash.object_types.clone();

    AbtProgram {
        strings,
        exports,
        object_keys,
        object_types,
        code,
        labels,
    }
}

/// First pass: scan bytecode to find all jump/call targets.
fn collect_targets(flash: &VirtualFlash, start: usize, end: usize, targets: &mut HashSet<usize>) {
    let mut ip = start;
    while ip < end {
        let offset = ip;
        let op_byte = flash.read_u8(ip);
        ip += 1;

        if !OpCode::is_valid(op_byte) {
            continue;
        }

        let op: OpCode = op_byte.into();

        if op == OpCode::SOURCE_LINE {
            ip += 2;
            continue;
        }

        let advance = operand_size(flash, op, ip, offset);

        // Record jump/call targets
        match op {
            OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ => {
                let rel = i16::from_le_bytes([flash.read_u8(ip), flash.read_u8(ip + 1)]);
                targets.insert((ip + 2).wrapping_add(rel as usize));
            }
            OpCode::JMP_L | OpCode::JMP_FAR => {
                let rel = i32::from_le_bytes([
                    flash.read_u8(ip),
                    flash.read_u8(ip + 1),
                    flash.read_u8(ip + 2),
                    flash.read_u8(ip + 3),
                ]);
                targets.insert((ip + 4).wrapping_add(rel as usize));
            }
            OpCode::CALL | OpCode::CLOSURE => {
                let addr = flash.read_u32(ip);
                targets.insert(addr as usize);
            }
            // Plan 321: CREATE_GENERATOR also has a func_addr target
            OpCode::CREATE_GENERATOR => {
                let addr = flash.read_u32(ip);
                targets.insert(addr as usize);
            }
            OpCode::CREATE_FUTURE => {
                let addr = flash.read_u32(ip);
                targets.insert(addr as usize);
            }
            _ => {}
        }

        ip += advance;
    }
}

/// Compute operand size for an opcode (same logic as decode_operands but faster).
fn operand_size(flash: &VirtualFlash, op: OpCode, ip: usize, offset: usize) -> usize {
    match op {
        // No operands
        OpCode::NOP | OpCode::POP | OpCode::DUP | OpCode::SWAP | OpCode::DROP
        | OpCode::CONST_0 | OpCode::CONST_1 | OpCode::HALT | OpCode::PRINT
        | OpCode::RET_D | OpCode::YIELD_TASK | OpCode::YIELD_VAL | OpCode::CREATE_NONE
        | OpCode::IS_SOME | OpCode::IS_OK | OpCode::UNWRAP_SOME | OpCode::UNWRAP_OK
        | OpCode::UNWRAP_ERR | OpCode::IS_NIL | OpCode::NEG | OpCode::NEG_F
        | OpCode::NEG_D | OpCode::NOT | OpCode::TO_STR | OpCode::STR_CAT
        | OpCode::ADD | OpCode::SUB | OpCode::MUL | OpCode::DIV | OpCode::MOD
        | OpCode::ADD_F | OpCode::SUB_F | OpCode::MUL_F | OpCode::DIV_F
        | OpCode::ADD_D | OpCode::SUB_D | OpCode::MUL_D | OpCode::DIV_D
        | OpCode::ADD_U64 | OpCode::SUB_U64 | OpCode::MUL_U64 | OpCode::DIV_U64
        | OpCode::MOD_U64 | OpCode::AND | OpCode::OR | OpCode::XOR
        | OpCode::SHL | OpCode::SHR | OpCode::EQ | OpCode::NE | OpCode::LT
        | OpCode::GT | OpCode::LE | OpCode::GE | OpCode::EQ_D | OpCode::NE_D
        | OpCode::LT_D | OpCode::GT_D | OpCode::LE_D | OpCode::GE_D
        | OpCode::I32_TO_F32 | OpCode::I64_TO_F64 | OpCode::U64_TO_F64
        | OpCode::PROMOTE_F64 | OpCode::NULL_COALESCE
        | OpCode::TASK_ID | OpCode::SPAWN_GO | OpCode::REPLY | OpCode::HANDLE_MSG
        | OpCode::CALL_CLOSURE | OpCode::TYPE_F64_TO_I32 | OpCode::TYPE_STR_TO_I64
        | OpCode::TYPE_F32_TO_I32 | OpCode::TYPE_CAST_PTR | OpCode::ARRAY_LEN
        | OpCode::MOD_F | OpCode::MOD_D
        | OpCode::CREATE_SOME | OpCode::CREATE_ERR
        | OpCode::CREATE_RANGE | OpCode::CREATE_RANGE_EQ
        | OpCode::CHAN_NEW | OpCode::RECV | OpCode::TRY_RECV
        | OpCode::TASK_LOOP | OpCode::AWAIT_FUTURE | OpCode::POLL_FUTURE
        | OpCode::CONSTRUCT_INSTANCE
        | OpCode::CREATE_LIST_INT | OpCode::CREATE_LIST_STR | OpCode::CREATE_LIST_BOOL
        | OpCode::CREATE_LIST_INT_INLINE | OpCode::CREATE_LIST_STR_INLINE
        | OpCode::CREATE_LIST_BOOL_INLINE | OpCode::LIST_PUSH_INT
        | OpCode::LIST_POP_INT | OpCode::LIST_GET_INT | OpCode::LIST_SET_INT
        | OpCode::GET_ELEM | OpCode::SET_ELEM | OpCode::SET_FIELD | OpCode::SLICE
        | OpCode::PUSH_NIL
        | OpCode::LOAD_LOC_0 | OpCode::LOAD_LOC_1 | OpCode::LOAD_LOC_2
        | OpCode::STORE_LOC_0 | OpCode::STORE_LOC_1
        | OpCode::TYPE_CAST_I32 | OpCode::TYPE_CAST_U32 | OpCode::TYPE_CAST_I64
        | OpCode::TYPE_CAST_U64 | OpCode::TYPE_CAST_F64
        | OpCode::TYPE_TO_STR | OpCode::TYPE_TO_I32 | OpCode::TYPE_TO_F64
        | OpCode::TYPE_F64_TO_STR | OpCode::TYPE_I64_TO_STR | OpCode::TYPE_U64_TO_STR
        | OpCode::TYPE_BOOL_TO_STR | OpCode::TYPE_F32_TO_STR
        | OpCode::POP_HANDLER
            => 0,

        OpCode::CONST_U8 | OpCode::POP_N | OpCode::RESERVE_STACK | OpCode::RET
        | OpCode::ERROR_PROPAGATE | OpCode::CREATE_OK | OpCode::GET_GENERIC_FIELD
        | OpCode::SET_GENERIC_FIELD | OpCode::GET_TUPLE_FIELD
        | OpCode::CREATE_ARRAY | OpCode::CREATE_TUPLE
        | OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL
        | OpCode::LOAD_STATE_FIELD | OpCode::STORE_STATE_FIELD
        | OpCode::LOAD_GLOBAL | OpCode::STORE_GLOBAL
        | OpCode::PUSH_BOOL   // Plan 318: 1 byte operand (0|1)
            => 1,

        OpCode::FN_PROLOG => 2,

        OpCode::CONST_I32 | OpCode::CONST_F32 | OpCode::CALL | OpCode::CLOSURE
        | OpCode::SLEEP | OpCode::JOIN | OpCode::SEND | OpCode::CREATE_FUTURE
        | OpCode::LOAD_REF | OpCode::STORE_REF | OpCode::LOAD_MUT_REF | OpCode::STORE_MUT_REF
            => 4,

        // Plan 321: CREATE_GENERATOR has 5 operand bytes
        OpCode::CREATE_GENERATOR => 5,

        OpCode::CONST_I64 | OpCode::CONST_U64 | OpCode::CONST_F64 => 8,

        OpCode::LOAD_STR | OpCode::CALL_NAT | OpCode::CAPTURE_VAR | OpCode::LOAD_CAPTURED
        | OpCode::STORE_CAPTURED | OpCode::GET_FIELD | OpCode::JMP | OpCode::JMP_IF_Z
        | OpCode::JMP_IF_NZ | OpCode::PUSH_HANDLER | OpCode::IS_VARIANT
            => 2,

        OpCode::JMP_L | OpCode::JMP_FAR | OpCode::CALL_SPEC => 4,

        OpCode::SPAWN => 5,

        OpCode::CREATE_OBJ | OpCode::CALL_PY => 3,

        OpCode::CREATE_NODE => 5,

        OpCode::BUILD_FSTR => {
            let part_count = flash.read_u8(ip);
            1 + part_count as usize
        }

        OpCode::NEW_INSTANCE => {
            if offset >= 5 && flash.read_u8(offset - 5) == OpCode::CONST_I32 as u8 {
                flash.read_i32(offset - 4) as usize
            } else {
                0
            }
        }

        OpCode::SOURCE_LINE => 2,
    }
}

/// Decode operands into AbtOperands, using labels where appropriate.
fn decode_operands(
    flash: &VirtualFlash,
    op: OpCode,
    ip: usize,
    offset: usize,
    labels: &HashMap<String, usize>,
) -> (Vec<AbtOperand>, usize) {
    let label = |addr: usize| -> AbtOperand {
        if let Some((name, _)) = labels.iter().find(|(_, &o)| o == addr) {
            AbtOperand::Label(name.clone())
        } else {
            AbtOperand::ImmU32(addr as u32)
        }
    };

    match op {
        // No operands
        OpCode::NOP | OpCode::POP | OpCode::DUP | OpCode::SWAP | OpCode::DROP
        | OpCode::CONST_0 | OpCode::CONST_1 | OpCode::HALT | OpCode::PRINT
        | OpCode::RET_D | OpCode::YIELD_TASK | OpCode::YIELD_VAL | OpCode::CREATE_NONE
        | OpCode::IS_SOME | OpCode::IS_OK | OpCode::UNWRAP_SOME | OpCode::UNWRAP_OK
        | OpCode::UNWRAP_ERR | OpCode::IS_NIL | OpCode::NEG | OpCode::NEG_F
        | OpCode::NEG_D | OpCode::NOT | OpCode::TO_STR | OpCode::STR_CAT
        | OpCode::ADD | OpCode::SUB | OpCode::MUL | OpCode::DIV | OpCode::MOD
        | OpCode::ADD_F | OpCode::SUB_F | OpCode::MUL_F | OpCode::DIV_F
        | OpCode::ADD_D | OpCode::SUB_D | OpCode::MUL_D | OpCode::DIV_D
        | OpCode::ADD_U64 | OpCode::SUB_U64 | OpCode::MUL_U64 | OpCode::DIV_U64
        | OpCode::MOD_U64 | OpCode::AND | OpCode::OR | OpCode::XOR
        | OpCode::SHL | OpCode::SHR | OpCode::EQ | OpCode::NE | OpCode::LT
        | OpCode::GT | OpCode::LE | OpCode::GE | OpCode::EQ_D | OpCode::NE_D
        | OpCode::LT_D | OpCode::GT_D | OpCode::LE_D | OpCode::GE_D
        | OpCode::I32_TO_F32 | OpCode::I64_TO_F64 | OpCode::U64_TO_F64
        | OpCode::PROMOTE_F64 | OpCode::NULL_COALESCE
        | OpCode::TASK_ID | OpCode::SPAWN_GO | OpCode::REPLY | OpCode::HANDLE_MSG
        | OpCode::CALL_CLOSURE | OpCode::TYPE_F64_TO_I32 | OpCode::TYPE_STR_TO_I64
        | OpCode::TYPE_F32_TO_I32 | OpCode::TYPE_CAST_PTR | OpCode::ARRAY_LEN
        | OpCode::MOD_F | OpCode::MOD_D
        | OpCode::CREATE_SOME | OpCode::CREATE_ERR
        | OpCode::CREATE_RANGE | OpCode::CREATE_RANGE_EQ
        | OpCode::CHAN_NEW | OpCode::RECV | OpCode::TRY_RECV
        | OpCode::TASK_LOOP | OpCode::AWAIT_FUTURE | OpCode::POLL_FUTURE
        | OpCode::CONSTRUCT_INSTANCE
        | OpCode::CREATE_LIST_INT | OpCode::CREATE_LIST_STR | OpCode::CREATE_LIST_BOOL
        | OpCode::CREATE_LIST_INT_INLINE | OpCode::CREATE_LIST_STR_INLINE
        | OpCode::CREATE_LIST_BOOL_INLINE | OpCode::LIST_PUSH_INT
        | OpCode::LIST_POP_INT | OpCode::LIST_GET_INT | OpCode::LIST_SET_INT
        | OpCode::GET_ELEM | OpCode::SET_ELEM | OpCode::SET_FIELD | OpCode::SLICE
        | OpCode::PUSH_NIL
        | OpCode::POP_HANDLER
            => (vec![], 0),

        OpCode::CONST_U8 => {
            let v = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(v)], 1)
        }
        OpCode::POP_N => {
            let v = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(v)], 1)
        }
        OpCode::RESERVE_STACK => {
            let v = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(v)], 1)
        }
        OpCode::RET => {
            let v = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(v)], 1)
        }
        OpCode::ERROR_PROPAGATE => {
            let v = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(v)], 1)
        }
        OpCode::FN_PROLOG => {
            let n_args = flash.read_u8(ip);
            let n_locals = flash.read_u8(ip + 1);
            (vec![AbtOperand::ImmU8(n_args), AbtOperand::ImmU8(n_locals)], 2)
        }
        OpCode::SPAWN => {
            let func = flash.read_u32(ip) as usize;
            let argc = flash.read_u8(ip + 4);
            (vec![label(func), AbtOperand::ImmU8(argc)], 5)
        }
        OpCode::SLEEP => {
            let ms = flash.read_u32(ip);
            (vec![AbtOperand::ImmU32(ms)], 4)
        }
        OpCode::JOIN | OpCode::SEND => {
            let v = flash.read_u32(ip);
            (vec![AbtOperand::ImmU32(v)], 4)
        }

        OpCode::CONST_I32 => {
            let v = flash.read_i32(ip);
            (vec![AbtOperand::ImmI32(v)], 4)
        }
        OpCode::CONST_F32 => {
            let v = flash.read_f32(ip);
            (vec![AbtOperand::ImmF32(v)], 4)
        }
        OpCode::CONST_I64 => {
            let v = flash.read_i64(ip);
            (vec![AbtOperand::ImmI64(v)], 8)
        }
        OpCode::CONST_U64 => {
            let v = flash.read_u64(ip);
            (vec![AbtOperand::ImmU64(v)], 8)
        }
        OpCode::CONST_F64 => {
            let v = flash.read_f64(ip);
            (vec![AbtOperand::ImmF64(v)], 8)
        }

        OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL
        | OpCode::LOAD_STATE_FIELD | OpCode::STORE_STATE_FIELD
        | OpCode::LOAD_GLOBAL | OpCode::STORE_GLOBAL
        | OpCode::PUSH_BOOL => {  // Plan 318: 1 byte operand (0|1)
            let v = flash.read_u8(ip);
            // Plan 087/088: parameters encoded as 0x80 + param_index
            let operand = if v >= 0x80 {
                AbtOperand::ImmU8(v) // will be displayed as argN by formatter
            } else {
                AbtOperand::ImmU8(v)
            };
            (vec![operand], 1)
        }
        OpCode::LOAD_LOC_0 | OpCode::LOAD_LOC_1 | OpCode::LOAD_LOC_2
        | OpCode::STORE_LOC_0 | OpCode::STORE_LOC_1 => (vec![], 0),

        OpCode::LOAD_STR => {
            let v = flash.read_u16(ip);
            (vec![AbtOperand::StringIdx(v as usize)], 2)
        }
        OpCode::CALL_NAT => {
            let v = flash.read_u16(ip);
            (vec![AbtOperand::NatIdx(v)], 2)
        }
        OpCode::CAPTURE_VAR | OpCode::LOAD_CAPTURED | OpCode::STORE_CAPTURED => {
            let v = flash.read_u16(ip);
            (vec![AbtOperand::StringIdx(v as usize)], 2)
        }

        OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ => {
            let rel = i16::from_le_bytes([flash.read_u8(ip), flash.read_u8(ip + 1)]);
            let target = (ip + 2).wrapping_add(rel as usize);
            (vec![label(target)], 2)
        }
        OpCode::PUSH_HANDLER => {
            // handler_pc: u16 (relative offset to catch handler)
            let rel = i16::from_le_bytes([flash.read_u8(ip), flash.read_u8(ip + 1)]);
            let target = (ip + 2).wrapping_add(rel as usize);
            (vec![label(target)], 2)
        }
        OpCode::JMP_L | OpCode::JMP_FAR => {
            let rel = i32::from_le_bytes([
                flash.read_u8(ip),
                flash.read_u8(ip + 1),
                flash.read_u8(ip + 2),
                flash.read_u8(ip + 3),
            ]);
            let target = (ip + 4).wrapping_add(rel as usize);
            (vec![label(target)], 4)
        }

        OpCode::CALL => {
            let v = flash.read_u32(ip) as usize;
            (vec![label(v)], 4)
        }

        // Plan 321: CREATE_GENERATOR: u32 func_addr + u8 n_args
        OpCode::CREATE_GENERATOR => {
            let v = flash.read_u32(ip) as usize;
            (vec![label(v)], 5) // 4 bytes addr + 1 byte n_args
        }

        OpCode::CALL_SPEC => {
            let spec = flash.read_u16(ip);
            let method = flash.read_u16(ip + 2);
            (vec![AbtOperand::StringIdx(spec as usize), AbtOperand::StringIdx(method as usize)], 4)
        }

        OpCode::CREATE_ARRAY | OpCode::CREATE_TUPLE => {
            let v = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(v)], 1)
        }

        OpCode::CREATE_OBJ => {
            let key_index = flash.read_u16(ip);
            let field_count = flash.read_u8(ip + 2);
            (vec![AbtOperand::ImmU16(key_index), AbtOperand::ImmU8(field_count)], 3)
        }

        // Plan 369 Task 10: py-FFI call: u16 native_id + u8 arg_count
        OpCode::CALL_PY => {
            let native_id = flash.read_u16(ip);
            let arg_count = flash.read_u8(ip + 2);
            (vec![AbtOperand::NatIdx(native_id), AbtOperand::ImmU8(arg_count)], 3)
        }

        OpCode::BUILD_FSTR => {
            let part_count = flash.read_u8(ip);
            let mut tags = vec![AbtOperand::ImmU8(part_count)];
            for i in 0..part_count {
                tags.push(AbtOperand::ImmU8(flash.read_u8(ip + 1 + i as usize)));
            }
            (tags, 1 + part_count as usize)
        }

        OpCode::GET_FIELD => {
            let v = flash.read_u16(ip);
            (vec![AbtOperand::FieldIdx(v as usize)], 2)
        }

        OpCode::CREATE_NODE => {
            let name = flash.read_u16(ip);
            let argc = flash.read_u8(ip + 2);
            let id_idx = flash.read_u16(ip + 3);
            (
                vec![
                    AbtOperand::StringIdx(name as usize),
                    AbtOperand::ImmU8(argc),
                    AbtOperand::StringIdx(id_idx as usize),
                ],
                5,
            )
        }

        OpCode::CREATE_OK => {
            let type_tag = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(type_tag)], 1)
        }

        OpCode::TYPE_CAST_I32 | OpCode::TYPE_CAST_U32 | OpCode::TYPE_CAST_I64
        | OpCode::TYPE_CAST_U64 | OpCode::TYPE_CAST_F64 | OpCode::TYPE_TO_STR
        | OpCode::TYPE_TO_I32 | OpCode::TYPE_TO_F64 | OpCode::TYPE_F64_TO_STR
        | OpCode::TYPE_I64_TO_STR | OpCode::TYPE_U64_TO_STR | OpCode::TYPE_BOOL_TO_STR
        | OpCode::TYPE_F32_TO_STR => (vec![], 0),

        OpCode::CLOSURE => {
            let addr = flash.read_u32(ip) as usize;
            (vec![label(addr)], 4)
        }

        OpCode::NEW_INSTANCE => {
            let name_len = if offset >= 5 && flash.read_u8(offset - 5) == OpCode::CONST_I32 as u8 {
                flash.read_i32(offset - 4) as usize
            } else {
                0
            };
            let mut bytes = Vec::new();
            for i in 0..name_len {
                bytes.push(flash.read_u8(ip + i));
            }
            (vec![AbtOperand::Bytes(bytes)], name_len)
        }

        OpCode::GET_GENERIC_FIELD | OpCode::SET_GENERIC_FIELD => {
            let idx = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(idx)], 1)
        }

        OpCode::IS_VARIANT => {
            let len = flash.read_u16(ip);
            let mut bytes = Vec::new();
            for i in 0..len {
                bytes.push(flash.read_u8(ip + 2 + i as usize));
            }
            (vec![AbtOperand::Bytes(bytes)], 2 + len as usize)
        }

        OpCode::CREATE_FUTURE => {
            let offset_val = flash.read_u32(ip) as usize;
            (vec![label(offset_val)], 4)
        }

        // Already covered in the no-operands arm above

        OpCode::SOURCE_LINE => {
            let line = flash.read_u16(ip);
            (vec![AbtOperand::ImmU16(line)], 2)
        }

        OpCode::GET_TUPLE_FIELD => {
            let idx = flash.read_u8(ip);
            (vec![AbtOperand::ImmU8(idx)], 1)
        }

        OpCode::LOAD_REF | OpCode::STORE_REF | OpCode::LOAD_MUT_REF | OpCode::STORE_MUT_REF => {
            let v = flash.read_u32(ip);
            (vec![AbtOperand::ImmU32(v)], 4)
        }
    }
}

fn sanitize_label(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}
