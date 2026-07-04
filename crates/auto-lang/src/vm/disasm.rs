//! Plan 199: Bytecode disassembler for debugging
//!
//! Converts bytecode to human-readable mnemonics with source line annotations.

use crate::vm::opcode::OpCode;
use crate::vm::virt_memory::VirtualFlash;

/// A single disassembled line
#[derive(Debug, Clone)]
pub struct DisasmLine {
    pub offset: usize,
    pub mnemonic: &'static str,
    pub operands: String,
    pub line: Option<u32>,
}

/// Bytecode disassembler
pub struct Disassembler<'a> {
    flash: &'a VirtualFlash,
}

impl<'a> Disassembler<'a> {
    pub fn new(flash: &'a VirtualFlash) -> Self {
        Self { flash }
    }

    /// Disassemble a range of bytecode
    pub fn disassemble_range(&self, start: usize, end: usize) -> Vec<DisasmLine> {
        let mut lines = Vec::new();
        let mut ip = start;
        let mut current_line = None;

        while ip < end {
            let offset = ip;
            let op_byte = self.flash.read_u8(ip);
            ip += 1;

            if !OpCode::is_valid(op_byte) {
                lines.push(DisasmLine {
                    offset,
                    mnemonic: "???",
                    operands: format!("0x{:02x}", op_byte),
                    line: current_line,
                });
                continue;
            }

            let op: OpCode = op_byte.into();

            if op == OpCode::SOURCE_LINE {
                let line = self.flash.read_u16(ip);
                ip += 2;
                current_line = Some(line as u32);
                lines.push(DisasmLine {
                    offset,
                    mnemonic: ".line",
                    operands: line.to_string(),
                    line: current_line,
                });
                continue;
            }

            let mnemonic = op.to_mnemonic();
            let (operands, advance) = self.decode_operands(op, ip, offset);
            ip += advance;

            lines.push(DisasmLine {
                offset,
                mnemonic,
                operands,
                line: current_line,
            });
        }
        lines
    }

    /// Decode operands for an opcode, returning (operand_string, bytes_consumed)
    fn decode_operands(&self, op: OpCode, ip: usize, offset: usize) -> (String, usize) {
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
                => (String::new(), 0),

            // 1-byte operand
            OpCode::CONST_U8 => {
                let v = self.flash.read_u8(ip);
                (format!("{}", v), 1)
            }
            // Plan 336: PUSH_BOOL byte operand (0|1)
            OpCode::PUSH_BOOL => {
                let v = self.flash.read_u8(ip);
                (if v != 0 { "true".into() } else { "false".into() }, 1)
            }
            OpCode::POP_N => {
                let v = self.flash.read_u8(ip);
                (format!("{}", v), 1)
            }
            OpCode::RESERVE_STACK => {
                let v = self.flash.read_u8(ip);
                (format!("{}", v), 1)
            }
            OpCode::RET => {
                let v = self.flash.read_u8(ip);
                (format!("n_args={}", v), 1)
            }
            OpCode::ERROR_PROPAGATE => {
                let v = self.flash.read_u8(ip);
                (format!("n_args={}", v), 1)
            }
            OpCode::FN_PROLOG => {
                let n_args = self.flash.read_u8(ip);
                let n_locals = self.flash.read_u8(ip + 1);
                (format!("args={}, locals={}", n_args, n_locals), 2)
            }
            OpCode::SPAWN => {
                let func = self.flash.read_u32(ip);
                let argc = self.flash.read_u8(ip + 4);
                (format!("func=0x{:04x}, argc={}", func, argc), 5)
            }
            OpCode::SLEEP => {
                let ms = self.flash.read_u32(ip);
                (format!("{}ms", ms), 4)
            }
            OpCode::JOIN | OpCode::SEND => {
                let v = self.flash.read_u32(ip);
                (format!("{}", v), 4)
            }

            // 4-byte operand (i32 / f32)
            OpCode::CONST_I32 => {
                let v = self.flash.read_i32(ip);
                (format!("{}", v), 4)
            }
            OpCode::CONST_F32 => {
                let v = self.flash.read_f32(ip);
                (format!("{}", v), 4)
            }

            // 8-byte operand (i64 / u64 / f64)
            OpCode::CONST_I64 => {
                let v = self.flash.read_i64(ip);
                (format!("{}", v), 8)
            }
            OpCode::CONST_U64 => {
                let v = self.flash.read_u64(ip);
                (format!("{}", v), 8)
            }
            OpCode::CONST_F64 => {
                let v = self.flash.read_f64(ip);
                (format!("{}", v), 8)
            }

            // 1-byte local index
            OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL
            | OpCode::LOAD_STATE_FIELD | OpCode::STORE_STATE_FIELD
            | OpCode::LOAD_GLOBAL | OpCode::STORE_GLOBAL => {
                let v = self.flash.read_u8(ip);
                (format!("{}", v), 1)
            }
            OpCode::LOAD_LOC_0 => ("0".to_string(), 0),
            OpCode::LOAD_LOC_1 => ("1".to_string(), 0),
            OpCode::LOAD_LOC_2 => ("2".to_string(), 0),
            OpCode::STORE_LOC_0 => ("0".to_string(), 0),
            OpCode::STORE_LOC_1 => ("1".to_string(), 0),

            // 2-byte operand (u16)
            OpCode::LOAD_STR => {
                let v = self.flash.read_u16(ip);
                (format!("str[{}]", v), 2)
            }
            OpCode::CALL_NAT => {
                let v = self.flash.read_u16(ip);
                (format!("nat#{}", v), 2)
            }
            OpCode::CAPTURE_VAR | OpCode::LOAD_CAPTURED | OpCode::STORE_CAPTURED => {
                let v = self.flash.read_u16(ip);
                (format!("str[{}]", v), 2)
            }

            // Jump: i16 offset
            OpCode::JMP | OpCode::JMP_IF_Z | OpCode::JMP_IF_NZ | OpCode::PUSH_HANDLER => {
                let v = i16::from_le_bytes([self.flash.read_u8(ip), self.flash.read_u8(ip + 1)]);
                let target = (ip + 2) as isize + v as isize;
                (format!("-> 0x{:04x}", target), 2)
            }
            OpCode::JMP_FAR => {
                let v = self.flash.read_i32(ip);
                let target = (ip + 4) as isize + v as isize;
                (format!("-> 0x{:04x}", target), 4)
            }
            OpCode::JMP_L => {
                let v = i32::from_le_bytes([
                    self.flash.read_u8(ip),
                    self.flash.read_u8(ip + 1),
                    self.flash.read_u8(ip + 2),
                    self.flash.read_u8(ip + 3),
                ]);
                let target = (ip + 4) as isize + v as isize;
                (format!("-> 0x{:04x}", target), 4)
            }

            // CALL: u32 target
            OpCode::CALL => {
                let v = self.flash.read_u32(ip);
                (format!("0x{:04x}", v), 4)
            }

            // Plan 321: CREATE_GENERATOR: func_addr:u32, n_args:u8
            OpCode::CREATE_GENERATOR => {
                let addr = self.flash.read_u32(ip);
                let n_args = self.flash.read_u8(ip + 4);
                (format!("addr=0x{:04x}, n_args={}", addr, n_args), 5)
            }

            // CALL_SPEC: u16 spec_name, u16 method_name
            OpCode::CALL_SPEC => {
                let spec = self.flash.read_u16(ip);
                let method = self.flash.read_u16(ip + 2);
                (format!("spec={}, method={}", spec, method), 4)
            }

            // CREATE_ARRAY/CREATE_TUPLE: u8 count
            OpCode::CREATE_ARRAY | OpCode::CREATE_TUPLE => {
                let v = self.flash.read_u8(ip);
                (format!("count={}", v), 1)
            }

            // CREATE_OBJ: u16 key_index, u8 field_count
            OpCode::CREATE_OBJ => {
                let key_index = self.flash.read_u16(ip);
                let field_count = self.flash.read_u8(ip + 2);
                (format!("keys={}, fields={}", key_index, field_count), 3)
            }

            // BUILD_FSTR: u8 part_count, then part_count * u8 type_tags
            OpCode::BUILD_FSTR => {
                let part_count = self.flash.read_u8(ip);
                let mut tags = Vec::new();
                for i in 0..part_count {
                    tags.push(self.flash.read_u8(ip + 1 + i as usize));
                }
                (format!("parts={}, tags={:?}", part_count, tags), 1 + part_count as usize)
            }

            // GET_FIELD: u16 field_idx
            OpCode::GET_FIELD => {
                let v = self.flash.read_u16(ip);
                (format!("field[{}]", v), 2)
            }

            // CREATE_NODE: u16 name_idx, u8 argc, u16 id_idx
            OpCode::CREATE_NODE => {
                let name = self.flash.read_u16(ip);
                let argc = self.flash.read_u8(ip + 2);
                let id_idx = self.flash.read_u16(ip + 3);
                (format!("name[{}], argc={}, id={}", name, argc, id_idx), 5)
            }

            // CREATE_OK: type_tag u8
            OpCode::CREATE_OK => {
                let type_tag = self.flash.read_u8(ip);
                let name = if type_tag == 1 { "f64" } else { "i32" };
                (format!("type={}", name), 1)
            }

            // Type casts and conversions (no operands)
            OpCode::TYPE_CAST_I32 | OpCode::TYPE_CAST_U32 | OpCode::TYPE_CAST_I64
            | OpCode::TYPE_CAST_U64 | OpCode::TYPE_CAST_F64 | OpCode::TYPE_TO_STR
            | OpCode::TYPE_TO_I32 | OpCode::TYPE_TO_F64 | OpCode::TYPE_F64_TO_STR
            | OpCode::TYPE_I64_TO_STR | OpCode::TYPE_U64_TO_STR | OpCode::TYPE_BOOL_TO_STR
            | OpCode::TYPE_F32_TO_STR
            | OpCode::POP_HANDLER => (String::new(), 0),

            // Closures
            OpCode::CLOSURE => {
                let addr = self.flash.read_u32(ip);
                (format!("addr=0x{:04x}", addr), 4)
            }

            // Generic instance
            OpCode::NEW_INSTANCE => {
                // NEW_INSTANCE reads name_len from stack, then name_bytes from bytecode.
                // To disassemble, look backward for the preceding CONST_I32 that pushed the length.
                let name_len = if offset >= 5 && self.flash.read_u8(offset - 5) == OpCode::CONST_I32 as u8 {
                    self.flash.read_i32(offset - 4) as usize
                } else {
                    0
                };
                let mut bytes = Vec::new();
                for i in 0..name_len {
                    bytes.push(self.flash.read_u8(ip + i));
                }
                let name = String::from_utf8_lossy(&bytes);
                (format!("\"{}\"", name), name_len)
            }

            OpCode::GET_GENERIC_FIELD | OpCode::SET_GENERIC_FIELD => {
                let idx = self.flash.read_u8(ip);
                (format!("field={}", idx), 1)
            }

            // Enum variant check
            OpCode::IS_VARIANT => {
                let len = self.flash.read_u16(ip);
                let mut bytes = Vec::new();
                for i in 0..len {
                    bytes.push(self.flash.read_u8(ip + 2 + i as usize));
                }
                let name = String::from_utf8_lossy(&bytes);
                (format!("\"{}\"", name), 2 + len as usize)
            }

            // Async
            OpCode::CREATE_FUTURE => {
                let offset_val = self.flash.read_u32(ip);
                (format!("body=0x{:04x}", offset_val), 4)
            }

            // Source line (handled above, but needed for exhaustiveness)
            OpCode::SOURCE_LINE => {
                let line = self.flash.read_u16(ip);
                (format!("{}", line), 2)
            }

            // Tuple
            OpCode::GET_TUPLE_FIELD => {
                let idx = self.flash.read_u8(ip);
                (format!("idx={}", idx), 1)
            }

            // References: u32 var_index
            OpCode::LOAD_REF | OpCode::STORE_REF | OpCode::LOAD_MUT_REF | OpCode::STORE_MUT_REF => {
                let v = self.flash.read_u32(ip);
                (format!("{}", v), 4)
            }
        }
    }
}

impl std::fmt::Display for DisasmLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line.is_some() {
            write!(f, "  {:04x}: {:16} {}", self.offset, self.mnemonic, self.operands)?;
            if let Some(line) = self.line {
                write!(f, "  ; line {}", line)?;
            }
        } else {
            write!(f, "  {:04x}: {:16} {}", self.offset, self.mnemonic, self.operands)?;
        }
        Ok(())
    }
}
