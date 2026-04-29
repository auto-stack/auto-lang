//! Plan 226: ABT text parser
//!
//! Parses ABT text into an `AbtProgram`.

use crate::vm::abt::{AbtInstruction, AbtOperand, AbtProgram};
use crate::vm::codegen::ObjectType;
use crate::vm::opcode::OpCode;


/// Parse ABT source text into an `AbtProgram`.
pub fn parse(source: &str) -> Result<AbtProgram, String> {
    let mut program = AbtProgram::default();
    let mut section = Section::None;
    let mut instr_idx = 0usize;

    for (line_no, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Section directives
        if line.starts_with('.') && !line.starts_with(".line") {
            section = match line {
                ".strings" => Section::Strings,
                ".exports" => Section::Exports,
                ".object_keys" => Section::ObjectKeys,
                ".object_types" => Section::ObjectTypes,
                ".code" => Section::Code,
                _ => return Err(format!("Unknown section '{}' at line {}", line, line_no + 1)),
            };
            continue;
        }

        match section {
            Section::Strings => {
                // Strip leading index prefix like "0: " and parse the quoted string
                let rest = line.find('"').map(|i| &line[i..]).unwrap_or(line);
                program.strings.push(parse_quoted_string(rest)?);
            }
            Section::Exports => {
                let parts: Vec<&str> = line.split("->").collect();
                if parts.len() != 2 {
                    return Err(format!("Invalid export line at {}: {}", line_no + 1, line));
                }
                let name = parts[0].trim().to_string();
                let target = parts[1].trim().to_string();
                // Strip @ prefix from label references in exports
                let target = if target.starts_with('@') {
                    target[1..].to_string()
                } else {
                    target
                };
                program.exports.push((name, target));
            }
            Section::ObjectKeys => {
                let keys = parse_array_line(line)?;
                program.object_keys.push(keys);
            }
            Section::ObjectTypes => {
                let types = parse_types_line(line)?;
                program.object_types.push(types);
            }
            Section::Code => {
                parse_code_line(line, line_no + 1, &mut program, &mut instr_idx)?;
            }
            Section::None => {
                return Err(format!("Line outside of section at {}: {}", line_no + 1, line));
            }
        }
    }

    // Convert export targets from strings to offsets
    // (offsets will be resolved during assembly)
    Ok(program)
}

#[derive(Debug, Clone, Copy)]
enum Section {
    None,
    Strings,
    Exports,
    ObjectKeys,
    ObjectTypes,
    Code,
}

fn parse_quoted_string(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') {
        Ok(s[1..s.len()-1].to_string())
    } else {
        Ok(s.to_string())
    }
}

fn parse_array_line(s: &str) -> Result<Vec<String>, String> {
    // Format: idx: ["a", "b", "c"]
    let bracket_start = s.find('[').ok_or_else(|| format!("Expected [ in: {}", s))?;
    let bracket_end = s.rfind(']').ok_or_else(|| format!("Expected ] in: {}", s))?;
    let inner = &s[bracket_start+1..bracket_end];
    let mut result = Vec::new();
    for item in inner.split(',') {
        let item = item.trim();
        if !item.is_empty() {
            result.push(parse_quoted_string(item)?);
        }
    }
    Ok(result)
}

fn parse_types_line(s: &str) -> Result<Vec<ObjectType>, String> {
    let bracket_start = s.find('[').ok_or_else(|| format!("Expected [ in: {}", s))?;
    let bracket_end = s.rfind(']').ok_or_else(|| format!("Expected ] in: {}", s))?;
    let inner = &s[bracket_start+1..bracket_end];
    let mut result = Vec::new();
    for item in inner.split(',') {
        let item = item.trim();
        if !item.is_empty() {
            result.push(parse_object_type(item)?);
        }
    }
    Ok(result)
}

fn parse_object_type(s: &str) -> Result<ObjectType, String> {
    match s.trim() {
        "Int" => Ok(ObjectType::Int),
        "Float" => Ok(ObjectType::Float),
        "Double" => Ok(ObjectType::Double),
        "Bool" => Ok(ObjectType::Bool),
        "String" => Ok(ObjectType::String),
        "Byte" => Ok(ObjectType::Byte),
        "Char" => Ok(ObjectType::Char),
        "Uint" => Ok(ObjectType::Uint),
        "Void" => Ok(ObjectType::Void),
        "NestedObject" => Ok(ObjectType::NestedObject),
        "Array" => Ok(ObjectType::Array),
        _ => Err(format!("Unknown ObjectType: {}", s)),
    }
}

fn parse_code_line(line: &str, line_no: usize, program: &mut AbtProgram, instr_idx: &mut usize) -> Result<(), String> {
    // Label?
    if line.ends_with(':') {
        let label = line[..line.len()-1].trim().to_string();
        program.labels.insert(label, *instr_idx); // instruction index, resolved to byte offset during assembly
        return Ok(());
    }

    // .line pseudo-op
    if line.starts_with(".line") {
        let num: u32 = line[5..].trim().parse().map_err(|e| format!("Invalid .line at {}: {}", line_no, e))?;
        program.code.push(AbtInstruction {
            offset: 0,
            opcode: OpCode::SOURCE_LINE,
            operands: vec![AbtOperand::ImmU16(num as u16)],
            source_line: Some(num),
        });
        *instr_idx += 1;
        return Ok(());
    }

    // Instruction
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    let mnemonic = parts[0];
    let opcode = OpCode::from_mnemonic(mnemonic)
        .ok_or_else(|| format!("Unknown mnemonic '{}' at line {}", mnemonic, line_no))?;

    let mut operands = Vec::new();
    if parts.len() > 1 {
        let rest = line[mnemonic.len()..].trim();
        for op_str in rest.split(',') {
            let op_str = op_str.trim();
            if !op_str.is_empty() {
                operands.push(parse_operand(op_str)?);
            }
        }
    }

    program.code.push(AbtInstruction {
        offset: 0,
        opcode,
        operands,
        source_line: None,
    });
    *instr_idx += 1;

    Ok(())
}

fn parse_operand(s: &str) -> Result<AbtOperand, String> {
    let s = s.trim();

    // Label reference: @name
    if s.starts_with('@') {
        return Ok(AbtOperand::Label(s[1..].to_string()));
    }

    // String index: str[N]
    if s.starts_with("str[") && s.ends_with(']') {
        let inner = &s[4..s.len()-1];
        let idx: usize = inner.parse().map_err(|e| format!("Invalid str idx: {}", e))?;
        return Ok(AbtOperand::StringIdx(idx));
    }

    // Field index: field[N]
    if s.starts_with("field[") && s.ends_with(']') {
        let inner = &s[6..s.len()-1];
        let idx: usize = inner.parse().map_err(|e| format!("Invalid field idx: {}", e))?;
        return Ok(AbtOperand::FieldIdx(idx));
    }

    // Native index: nat#N
    if s.starts_with("nat#") {
        let inner = &s[4..];
        let idx: u16 = inner.parse().map_err(|e| format!("Invalid nat idx: {}", e))?;
        return Ok(AbtOperand::NatIdx(idx));
    }

    // Parameter reference: argN (encoded as 0x80 + N)
    if s.starts_with("arg") {
        let num: u8 = s[3..].parse().map_err(|e| format!("Invalid arg ref: {}", e))?;
        if num > 127 {
            return Err(format!("arg index too large: {}", num));
        }
        return Ok(AbtOperand::ImmU8(0x80 + num));
    }

    // Hex: 0xNN
    if s.starts_with("0x") || s.starts_with("0X") {
        let val = u32::from_str_radix(&s[2..], 16).map_err(|e| format!("Invalid hex: {}", e))?;
        return Ok(AbtOperand::ImmU32(val));
    }

    // Float (contains .)
    if s.contains('.') {
        if let Ok(f) = s.parse::<f64>() {
            return Ok(AbtOperand::ImmF64(f));
        }
        if let Ok(f) = s.parse::<f32>() {
            return Ok(AbtOperand::ImmF32(f));
        }
    }

    // Integer
    if let Ok(v) = s.parse::<i32>() {
        return Ok(AbtOperand::ImmI32(v));
    }
    if let Ok(v) = s.parse::<i64>() {
        return Ok(AbtOperand::ImmI64(v));
    }
    if let Ok(v) = s.parse::<u64>() {
        return Ok(AbtOperand::ImmU64(v));
    }
    if let Ok(v) = s.parse::<u32>() {
        return Ok(AbtOperand::ImmU32(v));
    }
    if let Ok(v) = s.parse::<u16>() {
        return Ok(AbtOperand::ImmU16(v));
    }
    if let Ok(v) = s.parse::<u8>() {
        return Ok(AbtOperand::ImmU8(v));
    }

    // Quoted string
    if s.starts_with('"') && s.ends_with('"') {
        return Ok(AbtOperand::Bytes(s[1..s.len()-1].as_bytes().to_vec()));
    }

    Err(format!("Cannot parse operand: {}", s))
}
