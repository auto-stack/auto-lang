//! Plan 226: ABT (Auto Byte Text) — human-readable bytecode format
//!
//! ABT provides bidirectional conversion between text and binary bytecode (ABC).

pub mod asm;
pub mod disasm;
pub mod parser;

#[cfg(test)]
mod tests;

use crate::vm::opcode::OpCode;
use crate::vm::codegen::ObjectType;
use std::collections::HashMap;

/// In-memory representation of an ABT program
#[derive(Debug, Clone, Default)]
pub struct AbtProgram {
    /// String constant pool
    pub strings: Vec<String>,
    /// Export table: name -> target label or offset string
    pub exports: Vec<(String, String)>,
    /// Object literal key tables
    pub object_keys: Vec<Vec<String>>,
    /// Object literal field type tables
    pub object_types: Vec<Vec<ObjectType>>,
    /// Sequence of instructions
    pub code: Vec<AbtInstruction>,
    /// Label name -> byte offset (populated during assembly / after disassembly)
    pub labels: HashMap<String, usize>,
}

/// A single ABT instruction
#[derive(Debug, Clone)]
pub struct AbtInstruction {
    /// Bytecode offset (for disassembly reference)
    pub offset: usize,
    /// Opcode
    pub opcode: OpCode,
    /// Operands
    pub operands: Vec<AbtOperand>,
    /// Source line number (from SOURCE_LINE pseudo-op)
    pub source_line: Option<u32>,
}

/// An ABT operand
#[derive(Debug, Clone)]
pub enum AbtOperand {
    /// Immediate i32
    ImmI32(i32),
    /// Immediate i64
    ImmI64(i64),
    /// Immediate u64
    ImmU64(u64),
    /// Immediate f32
    ImmF32(f32),
    /// Immediate f64
    ImmF64(f64),
    /// Immediate u8
    ImmU8(u8),
    /// Immediate u16
    ImmU16(u16),
    /// Immediate u32
    ImmU32(u32),
    /// Label reference (for jumps and calls)
    Label(String),
    /// String pool index
    StringIdx(usize),
    /// Field name string pool index
    FieldIdx(usize),
    /// Native function index
    NatIdx(u16),
    /// Raw bytes (for NEW_INSTANCE inline mono_name)
    Bytes(Vec<u8>),
}

impl AbtProgram {
    /// Render the ABT program to a string and produce a source map that maps
    /// each output line to the Auto source line that produced it (when known).
    pub fn to_string_with_source_map(&self) -> (String, Vec<crate::trans::SourceMapEntry>) {
        use std::fmt::Write;
        let mut out = String::new();
        let mut map = Vec::new();
        let mut line_no: usize = 1;
        let mut current_line: Option<u32> = None;

        let mut emit_line = |s: &str, src_line: Option<u32>, out: &mut String, map: &mut Vec<crate::trans::SourceMapEntry>, line_no: &mut usize| {
            out.push_str(s);
            out.push('\n');
            if let Some(sl) = src_line {
                map.push(crate::trans::SourceMapEntry {
                    source_line: sl as usize,
                    output_line: *line_no,
                    source_file: None,
                });
            }
            *line_no += 1;
        };

        if !self.strings.is_empty() {
            emit_line(".strings", None, &mut out, &mut map, &mut line_no);
            for (i, s) in self.strings.iter().enumerate() {
                emit_line(&format!("  {}: {:?}", i, s), None, &mut out, &mut map, &mut line_no);
            }
            emit_line("", None, &mut out, &mut map, &mut line_no);
        }

        if !self.exports.is_empty() {
            emit_line(".exports", None, &mut out, &mut map, &mut line_no);
            for (name, target) in &self.exports {
                let text = if target.starts_with("0x") || target.parse::<usize>().is_ok() {
                    format!("  {} -> {}", name, target)
                } else {
                    format!("  {} -> @{}", name, target)
                };
                emit_line(&text, None, &mut out, &mut map, &mut line_no);
            }
            emit_line("", None, &mut out, &mut map, &mut line_no);
        }

        if !self.object_keys.is_empty() {
            emit_line(".object_keys", None, &mut out, &mut map, &mut line_no);
            for (i, keys) in self.object_keys.iter().enumerate() {
                emit_line(&format!("  {}: {:?}", i, keys), None, &mut out, &mut map, &mut line_no);
            }
            emit_line("", None, &mut out, &mut map, &mut line_no);
        }

        if !self.object_types.is_empty() {
            emit_line(".object_types", None, &mut out, &mut map, &mut line_no);
            for (i, types) in self.object_types.iter().enumerate() {
                let type_names: Vec<String> = types.iter().map(|t| format!("{:?}", t)).collect();
                emit_line(&format!("  {}: [{}]", i, type_names.join(", ")), None, &mut out, &mut map, &mut line_no);
            }
            emit_line("", None, &mut out, &mut map, &mut line_no);
        }

        emit_line(".code", None, &mut out, &mut map, &mut line_no);
        for instr in &self.code {
            for (label, &offset) in &self.labels {
                if offset == instr.offset {
                    emit_line(&format!("\n{}:", label), None, &mut out, &mut map, &mut line_no);
                }
            }

            if let Some(line) = instr.source_line {
                if current_line != Some(line) {
                    emit_line(&format!("  .line {}", line), Some(line), &mut out, &mut map, &mut line_no);
                    current_line = Some(line);
                }
            }

            if instr.opcode == OpCode::SOURCE_LINE {
                continue;
            }

            let text = match instr.opcode {
                OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL => {
                    if let Some(AbtOperand::ImmU8(v)) = instr.operands.first() {
                        if *v >= 0x80 {
                            format!("  {} arg{}", instr.opcode.to_mnemonic(), v - 0x80)
                        } else {
                            format!("  {} {}", instr.opcode.to_mnemonic(), v)
                        }
                    } else {
                        format!("  {}", instr.opcode.to_mnemonic())
                    }
                }
                OpCode::LOAD_LOC_0 | OpCode::LOAD_LOC_1 | OpCode::LOAD_LOC_2
                | OpCode::STORE_LOC_0 | OpCode::STORE_LOC_1 => {
                    format!("  {}", instr.opcode.to_mnemonic())
                }
                _ => {
                    let ops = instr
                        .operands
                        .iter()
                        .map(|o| format!("{}", o))
                        .collect::<Vec<_>>()
                        .join(", ");
                    if ops.is_empty() {
                        format!("  {}", instr.opcode.to_mnemonic())
                    } else {
                        format!("  {} {}", instr.opcode.to_mnemonic(), ops)
                    }
                }
            };
            emit_line(&text, current_line, &mut out, &mut map, &mut line_no);
        }

        (out, map)
    }
}

impl std::fmt::Display for AbtProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // .strings section
        if !self.strings.is_empty() {
            writeln!(f, ".strings")?;
            for (i, s) in self.strings.iter().enumerate() {
                writeln!(f, "  {}: {:?}", i, s)?;
            }
            writeln!(f)?;
        }

        // .exports section
        if !self.exports.is_empty() {
            writeln!(f, ".exports")?;
            for (name, target) in &self.exports {
                if target.starts_with("0x") || target.parse::<usize>().is_ok() {
                    writeln!(f, "  {} -> {}", name, target)?;
                } else {
                    writeln!(f, "  {} -> @{}", name, target)?;
                }
            }
            writeln!(f)?;
        }

        // .object_keys section
        if !self.object_keys.is_empty() {
            writeln!(f, ".object_keys")?;
            for (i, keys) in self.object_keys.iter().enumerate() {
                writeln!(f, "  {}: {:?}", i, keys)?;
            }
            writeln!(f)?;
        }

        // .object_types section
        if !self.object_types.is_empty() {
            writeln!(f, ".object_types")?;
            for (i, types) in self.object_types.iter().enumerate() {
                let type_names: Vec<String> = types.iter().map(|t| format!("{:?}", t)).collect();
                writeln!(f, "  {}: [{}]", i, type_names.join(", "))?;
            }
            writeln!(f)?;
        }

        // .code section
        writeln!(f, ".code")?;
        let mut current_line: Option<u32> = None;
        for instr in &self.code {
            // Check if there's a label at this offset (print before .line)
            for (label, &offset) in &self.labels {
                if offset == instr.offset {
                    writeln!(f, "\n{}:", label)?;
                }
            }

            if let Some(line) = instr.source_line {
                if current_line != Some(line) {
                    writeln!(f, "  .line {}", line)?;
                    current_line = Some(line);
                }
            }

            // Skip SOURCE_LINE pseudo-op — it's already printed above via source_line
            if instr.opcode == OpCode::SOURCE_LINE {
                continue;
            }

            // Special-case opcodes with custom formatting
            match instr.opcode {
                OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL => {
                    if let Some(AbtOperand::ImmU8(v)) = instr.operands.first() {
                        if *v >= 0x80 {
                            writeln!(f, "  {} arg{}", instr.opcode.to_mnemonic(), v - 0x80)?;
                        } else {
                            writeln!(f, "  {} {}", instr.opcode.to_mnemonic(), v)?;
                        }
                    } else {
                        writeln!(f, "  {}", instr.opcode.to_mnemonic())?;
                    }
                }
                OpCode::LOAD_LOC_0 | OpCode::LOAD_LOC_1 | OpCode::LOAD_LOC_2
                | OpCode::STORE_LOC_0 | OpCode::STORE_LOC_1 => {
                    writeln!(f, "  {}", instr.opcode.to_mnemonic())?;
                }
                _ => {
                    let ops = instr
                        .operands
                        .iter()
                        .map(|o| format!("{}", o))
                        .collect::<Vec<_>>()
                        .join(", ");

                    if ops.is_empty() {
                        writeln!(f, "  {}", instr.opcode.to_mnemonic())?;
                    } else {
                        writeln!(f, "  {} {}", instr.opcode.to_mnemonic(), ops)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for AbtOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbtOperand::ImmI32(v) => write!(f, "{}", v),
            AbtOperand::ImmI64(v) => write!(f, "{}", v),
            AbtOperand::ImmU64(v) => write!(f, "{}", v),
            AbtOperand::ImmF32(v) => write!(f, "{}", v),
            AbtOperand::ImmF64(v) => write!(f, "{}", v),
            AbtOperand::ImmU8(v) => write!(f, "{}", v),
            AbtOperand::ImmU16(v) => write!(f, "{}", v),
            AbtOperand::ImmU32(v) => write!(f, "{}", v),
            AbtOperand::Label(name) => write!(f, "@{}", name),
            AbtOperand::StringIdx(idx) => write!(f, "str[{}]", idx),
            AbtOperand::FieldIdx(idx) => write!(f, "field[{}]", idx),
            AbtOperand::NatIdx(idx) => write!(f, "nat#{}", idx),
            AbtOperand::Bytes(bytes) => {
                let s = String::from_utf8_lossy(bytes);
                write!(f, "{:?}", s)
            }
        }
    }
}
