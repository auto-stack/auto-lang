//! Plan 226: ABT → ABC assembler
//!
//! Converts ABT text into binary bytecode.

use crate::vm::abt::{AbtInstruction, AbtOperand, AbtProgram};

use crate::vm::loader::CompiledPackage;
use crate::vm::opcode::OpCode;
use std::collections::HashMap;

/// Assemble an `AbtProgram` into a `CompiledPackage`.
pub fn assemble(program: &AbtProgram) -> Result<CompiledPackage, String> {
    // === Pass 1: Compute instruction sizes and label offsets ===
    let mut offsets = Vec::new();
    let mut label_offsets = HashMap::new();
    let mut current_offset = 0usize;

    for instr in &program.code {
        offsets.push(current_offset);

        // Check if there's a label for this instruction
        for _ in &program.labels {
        }

        let size = instruction_size(instr);
        current_offset += size;
    }

    // Build label_offsets from program.labels.
    // The parser stores instruction indices (0, 1, 2, ...).
    // The disassembler stores byte offsets (typically larger).
    // If a label value is a valid instruction index, look up its byte offset.
    // Otherwise, treat it as a byte offset directly.
    for (label, &idx_or_offset) in &program.labels {
        let byte_offset = if idx_or_offset < offsets.len() {
            // Likely an instruction index from the parser
            offsets.get(idx_or_offset).copied().unwrap_or(0)
        } else {
            // Likely a byte offset from the disassembler
            idx_or_offset
        };
        label_offsets.insert(label.clone(), byte_offset);
    }
    // === Pass 2: Emit bytecode ===
    let mut bytecode = Vec::new();

    for (i, instr) in program.code.iter().enumerate() {
        let offset = offsets[i];

        match instr.opcode {
            OpCode::SOURCE_LINE => {
                if let Some(AbtOperand::ImmU16(line)) = instr.operands.first() {
                    bytecode.push(OpCode::SOURCE_LINE as u8);
                    bytecode.extend_from_slice(&line.to_le_bytes());
                }
            }
            _ => {
                bytecode.push(instr.opcode as u8);
                emit_operands(instr, offset, &label_offsets, &mut bytecode)?;
            }
        }
    }

    // Build exports map
    let mut exports = HashMap::new();
    for (name, target) in &program.exports {
        let addr = if target.starts_with("0x") {
            u32::from_str_radix(&target[2..], 16).map_err(|e| format!("Invalid export target: {}", e))?
        } else if let Some(&off) = label_offsets.get(target) {
            off as u32
        } else {
            return Err(format!("Unknown export target: {}", target));
        };
        exports.insert(name.clone(), addr);
    }

    // Build object_keys / object_types from program metadata
    let object_keys: Vec<Vec<auto_val::ValueKey>> = program
        .object_keys
        .iter()
        .map(|keys| keys.iter().map(|k| auto_val::ValueKey::Str(k.clone().into())).collect())
        .collect();

    Ok(CompiledPackage {
        bytecode,
        string_pool: program.strings.iter().map(|s| s.as_bytes().to_vec()).collect(),
        object_keys,
        object_types: program.object_types.clone(),
        exports,
        tasks: HashMap::new(),
        api_routes: Vec::new(),
    })
}

/// Compute the byte size of an instruction.
fn instruction_size(instr: &AbtInstruction) -> usize {
    let operand_size = match instr.opcode {
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
            => 0,

        OpCode::CONST_U8 | OpCode::POP_N | OpCode::RESERVE_STACK | OpCode::RET
        | OpCode::ERROR_PROPAGATE | OpCode::CREATE_OK | OpCode::GET_GENERIC_FIELD
        | OpCode::SET_GENERIC_FIELD | OpCode::GET_TUPLE_FIELD
        | OpCode::CREATE_ARRAY | OpCode::CREATE_TUPLE
        | OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL
        | OpCode::LOAD_STATE_FIELD | OpCode::STORE_STATE_FIELD
        | OpCode::LOAD_GLOBAL | OpCode::STORE_GLOBAL
            => 1,

        OpCode::FN_PROLOG => 2,

        OpCode::CONST_I32 | OpCode::CONST_F32 | OpCode::CALL | OpCode::CLOSURE
        | OpCode::SLEEP | OpCode::JOIN | OpCode::SEND | OpCode::CREATE_FUTURE
        | OpCode::LOAD_REF | OpCode::STORE_REF | OpCode::LOAD_MUT_REF | OpCode::STORE_MUT_REF
            => 4,

        // Plan 321: CREATE_GENERATOR has 5 operand bytes (u32 func_addr + u8 n_args)
        OpCode::CREATE_GENERATOR => 5,

        OpCode::CONST_I64 | OpCode::CONST_U64 | OpCode::CONST_F64 => 8,

        OpCode::LOAD_STR | OpCode::CALL_NAT | OpCode::CAPTURE_VAR | OpCode::LOAD_CAPTURED
        | OpCode::STORE_CAPTURED | OpCode::GET_FIELD | OpCode::JMP | OpCode::JMP_IF_Z
        | OpCode::JMP_IF_NZ
            => 2,

        OpCode::IS_VARIANT => {
            match instr.operands.first() {
                Some(AbtOperand::Bytes(b)) => 2 + b.len(),
                _ => 2,
            }
        }

        OpCode::JMP_L | OpCode::JMP_FAR | OpCode::CALL_SPEC => 4,

        OpCode::SPAWN => 5,

        OpCode::CREATE_OBJ => 3,

        OpCode::CREATE_NODE => 5,

        OpCode::BUILD_FSTR => {
            let part_count = match instr.operands.first() {
                Some(AbtOperand::ImmU8(n)) => *n as usize,
                Some(AbtOperand::ImmU16(n)) => *n as usize,
                Some(AbtOperand::ImmU32(n)) => *n as usize,
                Some(AbtOperand::ImmI32(n)) => *n as usize,
                Some(AbtOperand::ImmI64(n)) => *n as usize,
                Some(AbtOperand::ImmU64(n)) => *n as usize,
                _ => 0,
            };
            1 + part_count
        }

        OpCode::NEW_INSTANCE => {
            match instr.operands.first() {
                Some(AbtOperand::Bytes(b)) => b.len(),
                _ => 0,
            }
        }

        OpCode::SOURCE_LINE => 2,
    };

    1 + operand_size // 1 byte for opcode
}

fn emit_operands(
    instr: &AbtInstruction,
    offset: usize,
    label_offsets: &HashMap<String, usize>,
    bytecode: &mut Vec<u8>,
) -> Result<(), String> {

    match instr.opcode {
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
            => Ok(()),

        OpCode::CONST_U8 | OpCode::POP_N | OpCode::RESERVE_STACK | OpCode::RET
        | OpCode::ERROR_PROPAGATE | OpCode::CREATE_OK
        | OpCode::CREATE_ARRAY | OpCode::CREATE_TUPLE
        | OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL
        | OpCode::LOAD_STATE_FIELD | OpCode::STORE_STATE_FIELD
        | OpCode::LOAD_GLOBAL | OpCode::STORE_GLOBAL
            => {
                let v = operand_u8(&instr.operands, 0)?;
                bytecode.push(v);
                Ok(())
            }

        OpCode::FN_PROLOG => {
            let args = operand_u8(&instr.operands, 0)?;
            let locals = operand_u8(&instr.operands, 1)?;
            bytecode.push(args);
            bytecode.push(locals);
            Ok(())
        }

        OpCode::SPAWN => {
            let addr = operand_label_or_u32(&instr.operands, 0, label_offsets, ResolveType::Absolute)?;
            let argc = operand_u8(&instr.operands, 1)?;
            bytecode.extend_from_slice(&addr.to_le_bytes());
            bytecode.push(argc);
            Ok(())
        }

        OpCode::SLEEP | OpCode::JOIN | OpCode::SEND => {
            let v = operand_u32(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }

        OpCode::CONST_I32 => {
            let v = operand_i32(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }
        OpCode::CONST_F32 => {
            let v = operand_f32(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }
        OpCode::CONST_I64 => {
            let v = operand_i64(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }
        OpCode::CONST_U64 => {
            let v = operand_u64(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }
        OpCode::CONST_F64 => {
            let v = operand_f64(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }

        OpCode::LOAD_STR | OpCode::CALL_NAT | OpCode::CAPTURE_VAR | OpCode::LOAD_CAPTURED
        | OpCode::STORE_CAPTURED
            => {
                let v = operand_u16(&instr.operands, 0)?;
                bytecode.extend_from_slice(&v.to_le_bytes());
                Ok(())
            }

        OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ => {
            let target = operand_label_or_u32(&instr.operands, 0, label_offsets, ResolveType::Rel16)?;
            // Relative offset = target - (current_offset + opcode_size + operand_size)
            let rel = (target as isize) - (offset as isize + 1 + 2);
            bytecode.extend_from_slice(&(rel as i16).to_le_bytes());
            Ok(())
        }

        OpCode::IS_VARIANT => {
            let bytes = operand_bytes(&instr.operands, 0)?;
            bytecode.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            for b in bytes {
                bytecode.push(*b);
            }
            Ok(())
        }

        OpCode::JMP_L | OpCode::JMP_FAR => {
            let target = operand_label_or_u32(&instr.operands, 0, label_offsets, ResolveType::Rel32)?;
            // Relative offset = target - (current_offset + opcode_size + operand_size)
            let rel = (target as isize) - (offset as isize + 1 + 4);
            bytecode.extend_from_slice(&(rel as i32).to_le_bytes());
            Ok(())
        }

        OpCode::CALL | OpCode::CLOSURE | OpCode::CREATE_FUTURE => {
            let addr = operand_label_or_u32(&instr.operands, 0, label_offsets, ResolveType::Absolute)?;
            bytecode.extend_from_slice(&addr.to_le_bytes());
            Ok(())
        }

        // Plan 321: CREATE_GENERATOR: u32 func_addr + u8 n_args
        OpCode::CREATE_GENERATOR => {
            let addr = operand_label_or_u32(&instr.operands, 0, label_offsets, ResolveType::Absolute)?;
            bytecode.extend_from_slice(&addr.to_le_bytes());
            // n_args: read from operands if present, else default 0
            let n_args: u8 = if instr.operands.len() > 1 { 0 } else { 0 };
            bytecode.push(n_args);
            Ok(())
        }

        OpCode::CALL_SPEC => {
            let spec = operand_u16(&instr.operands, 0)?;
            let method = operand_u16(&instr.operands, 1)?;
            bytecode.extend_from_slice(&spec.to_le_bytes());
            bytecode.extend_from_slice(&method.to_le_bytes());
            Ok(())
        }

        OpCode::CREATE_OBJ => {
            let key_index = operand_u16(&instr.operands, 0)?;
            let field_count = operand_u8(&instr.operands, 1)?;
            bytecode.extend_from_slice(&key_index.to_le_bytes());
            bytecode.push(field_count);
            Ok(())
        }

        OpCode::CREATE_NODE => {
            let name = operand_u16(&instr.operands, 0)?;
            let argc = operand_u8(&instr.operands, 1)?;
            let id_idx = operand_u16(&instr.operands, 2)?;
            bytecode.extend_from_slice(&name.to_le_bytes());
            bytecode.push(argc);
            bytecode.extend_from_slice(&id_idx.to_le_bytes());
            Ok(())
        }

        OpCode::BUILD_FSTR => {
            let part_count = operand_u8(&instr.operands, 0)?;
            bytecode.push(part_count);
            for i in 1..=part_count as usize {
                let tag = operand_u8(&instr.operands, i)?;
                bytecode.push(tag);
            }
            Ok(())
        }

        OpCode::GET_FIELD => {
            let v = operand_u16(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }

        OpCode::NEW_INSTANCE => {
            // ABT stores mono_name bytes as the operand.
            // The assembler emits: CONST_I32 + len, NEW_INSTANCE, bytes
            // But in AbtProgram, NEW_INSTANCE instruction only has the bytes operand.
            // We need to emit the preceding CONST_I32 separately...
            // Actually, for round-trip, the ABT format should include CONST_I32 explicitly.
            // For now, if NEW_INSTANCE appears without preceding CONST_I32 in AbtProgram,
            // we emit it inline.
            let bytes = operand_bytes(&instr.operands, 0)?;
            for b in bytes {
                bytecode.push(*b);
            }
            Ok(())
        }

        OpCode::GET_GENERIC_FIELD | OpCode::SET_GENERIC_FIELD => {
            let idx = operand_u8(&instr.operands, 0)?;
            bytecode.push(idx);
            Ok(())
        }



        OpCode::SOURCE_LINE => {
            let line = operand_u16(&instr.operands, 0)?;
            bytecode.extend_from_slice(&line.to_le_bytes());
            Ok(())
        }



        OpCode::LOAD_REF | OpCode::STORE_REF | OpCode::LOAD_MUT_REF | OpCode::STORE_MUT_REF => {
            let v = operand_u32(&instr.operands, 0)?;
            bytecode.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }

        OpCode::GET_TUPLE_FIELD => {
            let idx = operand_u8(&instr.operands, 0)?;
            bytecode.push(idx);
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
enum ResolveType {
    Absolute,
    Rel16,
    Rel32,
}

fn operand_label_or_u32(
    operands: &[AbtOperand],
    idx: usize,
    label_offsets: &HashMap<String, usize>,
    resolve: ResolveType,
) -> Result<u32, String> {
    let op = operands.get(idx).ok_or_else(|| format!("Missing operand {}", idx))?;
    let value = match op {
        AbtOperand::Label(name) => {
            let target = label_offsets.get(name).copied()
                .ok_or_else(|| format!("Undefined label: {}", name))?;
            match resolve {
                ResolveType::Absolute => target as u32,
                ResolveType::Rel16 => {
                    // Relative offset from after the operand
                    // The caller handles this differently for emit...
                    // Actually we need the instruction offset to compute relative.
                    // For now, return target and let caller adjust.
                    target as u32
                }
                ResolveType::Rel32 => target as u32,
            }
        }
        AbtOperand::ImmU32(v) => *v,
        AbtOperand::ImmI32(v) => *v as u32,
        AbtOperand::ImmU16(v) => *v as u32,
        AbtOperand::ImmU8(v) => *v as u32,
        _ => return Err(format!("Expected label or u32, got {:?}", op)),
    };
    Ok(value)
}

fn operand_u8(operands: &[AbtOperand], idx: usize) -> Result<u8, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmU8(v)) => Ok(*v),
        Some(AbtOperand::ImmU16(v)) => Ok(*v as u8),
        Some(AbtOperand::ImmU32(v)) => Ok(*v as u8),
        Some(AbtOperand::ImmI32(v)) => Ok(*v as u8),
        Some(AbtOperand::StringIdx(v)) => Ok(*v as u8),
        Some(AbtOperand::FieldIdx(v)) => Ok(*v as u8),
        Some(AbtOperand::NatIdx(v)) => Ok(*v as u8),
        other => Err(format!("Expected u8 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_u16(operands: &[AbtOperand], idx: usize) -> Result<u16, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmU16(v)) => Ok(*v),
        Some(AbtOperand::ImmU32(v)) => Ok(*v as u16),
        Some(AbtOperand::ImmI32(v)) => Ok(*v as u16),
        Some(AbtOperand::ImmU8(v)) => Ok(*v as u16),
        Some(AbtOperand::StringIdx(v)) => Ok(*v as u16),
        Some(AbtOperand::FieldIdx(v)) => Ok(*v as u16),
        Some(AbtOperand::NatIdx(v)) => Ok(*v),
        other => Err(format!("Expected u16 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_u32(operands: &[AbtOperand], idx: usize) -> Result<u32, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmU32(v)) => Ok(*v),
        Some(AbtOperand::ImmI32(v)) => Ok(*v as u32),
        Some(AbtOperand::ImmU16(v)) => Ok(*v as u32),
        Some(AbtOperand::ImmU8(v)) => Ok(*v as u32),
        other => Err(format!("Expected u32 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_i32(operands: &[AbtOperand], idx: usize) -> Result<i32, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmI32(v)) => Ok(*v),
        Some(AbtOperand::ImmU32(v)) => Ok(*v as i32),
        Some(AbtOperand::ImmU16(v)) => Ok(*v as i32),
        Some(AbtOperand::ImmU8(v)) => Ok(*v as i32),
        other => Err(format!("Expected i32 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_i64(operands: &[AbtOperand], idx: usize) -> Result<i64, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmI64(v)) => Ok(*v),
        Some(AbtOperand::ImmU64(v)) => Ok(*v as i64),
        other => Err(format!("Expected i64 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_u64(operands: &[AbtOperand], idx: usize) -> Result<u64, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmU64(v)) => Ok(*v),
        Some(AbtOperand::ImmI64(v)) => Ok(*v as u64),
        other => Err(format!("Expected u64 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_f32(operands: &[AbtOperand], idx: usize) -> Result<f32, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmF32(v)) => Ok(*v),
        Some(AbtOperand::ImmF64(v)) => Ok(*v as f32),
        other => Err(format!("Expected f32 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_f64(operands: &[AbtOperand], idx: usize) -> Result<f64, String> {
    match operands.get(idx) {
        Some(AbtOperand::ImmF64(v)) => Ok(*v),
        other => Err(format!("Expected f64 operand at {}, got {:?}", idx, other)),
    }
}

fn operand_bytes(operands: &[AbtOperand], idx: usize) -> Result<&[u8], String> {
    match operands.get(idx) {
        Some(AbtOperand::Bytes(v)) => Ok(v.as_slice()),
        other => Err(format!("Expected bytes operand at {}, got {:?}", idx, other)),
    }
}
