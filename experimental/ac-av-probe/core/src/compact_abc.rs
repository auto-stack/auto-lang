use crate::abc::AbcDecodeError;
use crate::contract::{
    ResourceKind, SpanInfo, AV_PROFILE, MAX_DECODED_MODULE_BYTES, MAX_INPUT_BYTES,
    MAX_TABLE_ENTRIES, MAX_VERIFY_WALL, MAX_VERIFY_WORK_STEPS,
};
use crate::mir::*;
use std::time::Instant;

pub const COMPACT_ABC_MAGIC: [u8; 4] = *b"AVAC";
pub const COMPACT_ABC_VERSION: u16 = 1;
pub const COMPACT_ABC_PROFILE: u16 = 1;
pub const COMPACT_ABC_SCHEMA: &str = "auto.abc.av-compact-core-i32/1";

const SECTION_MODULE: u8 = 0x01;
const SECTION_TYPES: u8 = 0x02;
const SECTION_CONSTS: u8 = 0x03;
const SECTION_IMPORTS: u8 = 0x04;
const SECTION_FUNCS: u8 = 0x05;
const SECTION_SOURCE: u8 = 0x7f;
const FIXED_HEADER_LEN: usize = 12;
const DIRECTORY_ENTRY_LEN: usize = 9;

const OP_CONST: u8 = 0x01;
const OP_COPY: u8 = 0x02;
const OP_BIN: u8 = 0x03;
const OP_UN: u8 = 0x04;
const OP_CALL: u8 = 0x05;
const OP_TRACE: u8 = 0x06;
const TERM_JUMP: u8 = 0x10;
const TERM_BRANCH: u8 = 0x11;
const TERM_RETURN: u8 = 0x12;

#[derive(Debug)]
pub struct DecodedCompactAbc {
    pub version: u16,
    pub profile: u16,
    pub module: MirModule,
}

fn format_err(code: &str, message: impl Into<String>) -> AbcDecodeError {
    AbcDecodeError::Format {
        code: code.to_string(),
        message: message.into(),
    }
}

fn resource_err(kind: ResourceKind, message: impl Into<String>) -> AbcDecodeError {
    AbcDecodeError::Resource {
        kind,
        message: message.into(),
    }
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_uleb(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), AbcDecodeError> {
    let len = u32::try_from(value.len())
        .map_err(|_| format_err("compact.encode.string-length", "string exceeds u32 length"))?;
    put_uleb(out, len);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn check_module_tables(count: usize) -> Result<(), AbcDecodeError> {
    if count > MAX_TABLE_ENTRIES {
        Err(resource_err(
            ResourceKind::TableEntries,
            format!("module table entries {count} exceed {MAX_TABLE_ENTRIES}"),
        ))
    } else {
        Ok(())
    }
}

fn check_function_tables(f: &MirFunction) -> Result<(), AbcDecodeError> {
    let count = f
        .slots
        .len()
        .saturating_add(f.blocks.len())
        .saturating_add(f.flat_instr_count() as usize);
    if count > MAX_TABLE_ENTRIES {
        Err(resource_err(
            ResourceKind::TableEntries,
            format!("function table entries {count} exceed {MAX_TABLE_ENTRIES}"),
        ))
    } else {
        Ok(())
    }
}

fn encode_instr(out: &mut Vec<u8>, instr: &MirInstr) -> Result<(), AbcDecodeError> {
    match instr {
        MirInstr::Const { dst, value } => {
            out.push(OP_CONST);
            put_uleb(out, *dst);
            put_uleb(out, *value);
        }
        MirInstr::Copy { dst, src } => {
            out.push(OP_COPY);
            put_uleb(out, *dst);
            put_uleb(out, *src);
        }
        MirInstr::Bin { op, dst, lhs, rhs } => {
            out.push(OP_BIN);
            out.push(match op {
                BinOp::AddI32 => 1,
                BinOp::SubI32 => 2,
                BinOp::MulI32 => 3,
                BinOp::DivI32 => 4,
                BinOp::RemI32 => 5,
                BinOp::LtI32 => 6,
                BinOp::LeI32 => 7,
                BinOp::GtI32 => 8,
                BinOp::GeI32 => 9,
                BinOp::EqI32 => 10,
                BinOp::NeI32 => 11,
            });
            put_uleb(out, *dst);
            put_uleb(out, *lhs);
            put_uleb(out, *rhs);
        }
        MirInstr::Un { op, dst, operand } => {
            out.push(OP_UN);
            out.push(match op {
                UnOp::NegI32 => 1,
                UnOp::NotBool => 2,
            });
            put_uleb(out, *dst);
            put_uleb(out, *operand);
        }
        MirInstr::Call { callee, dst, args } => {
            let argc = u32::try_from(args.len())
                .map_err(|_| format_err("compact.encode.arity", "call arity exceeds u32"))?;
            out.push(OP_CALL);
            match callee {
                CalleeRef::Func(index) => {
                    out.push(0);
                    put_uleb(out, *index);
                }
                CalleeRef::Import(index) => {
                    out.push(1);
                    put_uleb(out, *index);
                }
            }
            put_uleb(out, *dst);
            put_uleb(out, argc);
            for arg in args {
                put_uleb(out, *arg);
            }
        }
        MirInstr::Trace { event } => {
            out.push(OP_TRACE);
            out.push(match event {
                TraceEvent::MarkA => 1,
                TraceEvent::MarkB => 2,
            });
        }
        _ => {
            return Err(format_err(
                "compact.encode.profile",
                "instruction is outside av.core-i32/1",
            ))
        }
    }
    Ok(())
}

fn encode_terminator(out: &mut Vec<u8>, term: &Terminator) {
    match term {
        Terminator::Jump { target } => {
            out.push(TERM_JUMP);
            put_uleb(out, *target);
        }
        Terminator::Branch { cond, then, else_ } => {
            out.push(TERM_BRANCH);
            put_uleb(out, *cond);
            put_uleb(out, *then);
            put_uleb(out, *else_);
        }
        Terminator::Return { value } => {
            out.push(TERM_RETURN);
            put_uleb(out, *value);
        }
    }
}

fn encode_function(f: &MirFunction) -> Result<Vec<u8>, AbcDecodeError> {
    check_function_tables(f)?;
    // PLAN-823 r8 env contract refresh (PLAN-826 T-02): a function with a
    // DECLARED closure env layout is a closure body — outside the scalar
    // av.core-i32/1 subset this wire carries. Refused at encode, never
    // silently dropped: the compact wire has no env section.
    if f.env.is_some() {
        return Err(format_err(
            "compact.encode.profile",
            "closure env layout is outside av.core-i32/1",
        ));
    }
    let mut out = Vec::new();
    put_uleb(&mut out, f.sig);
    put_uleb(
        &mut out,
        u32::try_from(f.slots.len())
            .map_err(|_| format_err("compact.encode.slots", "slot count exceeds u32"))?,
    );
    for slot in &f.slots {
        out.push(match slot.kind {
            SlotKind::Param => 0,
            SlotKind::Local => 1,
            SlotKind::Temp => 2,
        });
        put_uleb(&mut out, slot.ty);
    }
    put_uleb(&mut out, f.entry);
    put_uleb(
        &mut out,
        u32::try_from(f.blocks.len())
            .map_err(|_| format_err("compact.encode.blocks", "block count exceeds u32"))?,
    );
    for block in &f.blocks {
        put_uleb(
            &mut out,
            u32::try_from(block.instrs.len()).map_err(|_| {
                format_err("compact.encode.instrs", "instruction count exceeds u32")
            })?,
        );
        for instr in &block.instrs {
            encode_instr(&mut out, instr)?;
        }
        encode_terminator(&mut out, &block.term);
    }
    Ok(out)
}

fn assemble_sections(sections: &[(u8, Vec<u8>)]) -> Result<Vec<u8>, AbcDecodeError> {
    let section_count = u16::try_from(sections.len())
        .map_err(|_| format_err("compact.encode.sections", "section count exceeds u16"))?;
    let directory_len = sections
        .len()
        .checked_mul(DIRECTORY_ENTRY_LEN)
        .ok_or_else(|| format_err("compact.encode.length", "directory length overflow"))?;
    let mut offset = FIXED_HEADER_LEN
        .checked_add(directory_len)
        .ok_or_else(|| format_err("compact.encode.length", "header length overflow"))?;
    let total = sections.iter().try_fold(offset, |acc, (_, bytes)| {
        acc.checked_add(bytes.len())
            .ok_or_else(|| format_err("compact.encode.length", "section length overflow"))
    })?;
    if total > MAX_INPUT_BYTES {
        return Err(resource_err(
            ResourceKind::InputTooLarge,
            format!("compact ABC length {total} exceeds {MAX_INPUT_BYTES}"),
        ));
    }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&COMPACT_ABC_MAGIC);
    out.extend_from_slice(&COMPACT_ABC_VERSION.to_le_bytes());
    out.extend_from_slice(&COMPACT_ABC_PROFILE.to_le_bytes());
    out.extend_from_slice(&section_count.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    for (id, bytes) in sections {
        let len = u32::try_from(bytes.len())
            .map_err(|_| format_err("compact.encode.section-length", "section exceeds u32"))?;
        let offset_u32 = u32::try_from(offset)
            .map_err(|_| format_err("compact.encode.offset", "section offset exceeds u32"))?;
        out.push(*id);
        put_u32(&mut out, offset_u32);
        put_u32(&mut out, len);
        offset += bytes.len();
    }
    for (_, bytes) in sections {
        out.extend_from_slice(bytes);
    }
    Ok(out)
}

pub fn encode_compact_abc(m: &MirModule) -> Result<Vec<u8>, AbcDecodeError> {
    if m.profile != AV_PROFILE {
        return Err(format_err(
            "compact.encode.profile",
            format!("profile `{}` is not {AV_PROFILE}", m.profile),
        ));
    }
    check_module_tables(
        m.types
            .len()
            .saturating_add(m.consts.len())
            .saturating_add(m.imports.len())
            .saturating_add(m.funcs.len()),
    )?;

    let mut module = Vec::new();
    module.push(u8::from(m.is_unit));
    put_uleb(&mut module, m.entry);
    put_uleb(
        &mut module,
        u32::try_from(m.capabilities.len())
            .map_err(|_| format_err("compact.encode.capabilities", "too many capabilities"))?,
    );
    for capability in &m.capabilities {
        put_string(&mut module, capability)?;
    }

    let mut types = Vec::new();
    put_uleb(
        &mut types,
        u32::try_from(m.types.len())
            .map_err(|_| format_err("compact.encode.types", "type count exceeds u32"))?,
    );
    for ty in &m.types {
        match ty {
            MirType::I32 => types.push(0),
            MirType::Bool => types.push(1),
            MirType::Fn(sig) => {
                types.push(2);
                put_uleb(
                    &mut types,
                    u32::try_from(sig.params.len()).map_err(|_| {
                        format_err("compact.encode.sig", "signature arity exceeds u32")
                    })?,
                );
                for param in &sig.params {
                    put_uleb(&mut types, *param);
                }
                put_uleb(&mut types, sig.result);
            }
            _ => {
                return Err(format_err(
                    "compact.encode.profile",
                    "type is outside av.core-i32/1",
                ))
            }
        }
    }

    let mut consts = Vec::new();
    put_uleb(
        &mut consts,
        u32::try_from(m.consts.len())
            .map_err(|_| format_err("compact.encode.consts", "const count exceeds u32"))?,
    );
    for constant in &m.consts {
        match constant {
            MirConst::I32(value) => {
                consts.push(0);
                put_u32(&mut consts, *value as u32);
            }
            MirConst::Bool(value) => {
                consts.push(1);
                consts.push(u8::from(*value));
            }
            _ => {
                return Err(format_err(
                    "compact.encode.profile",
                    "constant is outside av.core-i32/1",
                ))
            }
        }
    }

    let mut imports = Vec::new();
    put_uleb(
        &mut imports,
        u32::try_from(m.imports.len())
            .map_err(|_| format_err("compact.encode.imports", "import count exceeds u32"))?,
    );
    for import in &m.imports {
        // PLAN-823 r8 env contract refresh (PLAN-826 T-02): an import
        // binding a closure body declares an env layout — outside the
        // scalar subset; refused at encode.
        if import.env.is_some() {
            return Err(format_err(
                "compact.encode.profile",
                "closure env import is outside av.core-i32/1",
            ));
        }
        put_string(&mut imports, &import.symbol)?;
        imports.push(match import.effect {
            Effect::Observable => 0,
            Effect::Pure => 1,
        });
        put_uleb(&mut imports, import.sig);
    }

    let mut funcs = Vec::new();
    put_uleb(
        &mut funcs,
        u32::try_from(m.funcs.len())
            .map_err(|_| format_err("compact.encode.funcs", "function count exceeds u32"))?,
    );
    for function in &m.funcs {
        let body = encode_function(function)?;
        put_u32(
            &mut funcs,
            u32::try_from(body.len())
                .map_err(|_| format_err("compact.encode.body-length", "body exceeds u32"))?,
        );
        funcs.extend_from_slice(&body);
    }

    let mut sections = vec![
        (SECTION_MODULE, module),
        (SECTION_TYPES, types),
        (SECTION_CONSTS, consts),
        (SECTION_IMPORTS, imports),
        (SECTION_FUNCS, funcs),
    ];
    if m.funcs
        .iter()
        .any(|f| f.source_snapshot != 0 || !f.spans.is_empty())
    {
        let mut source = Vec::new();
        for function in &m.funcs {
            source.extend_from_slice(&function.source_snapshot.to_le_bytes());
            put_uleb(
                &mut source,
                u32::try_from(function.spans.len())
                    .map_err(|_| format_err("compact.encode.spans", "span count exceeds u32"))?,
            );
            for (instr, span) in &function.spans {
                put_uleb(&mut source, *instr);
                put_uleb(&mut source, span.func);
                put_uleb(&mut source, span.start);
                put_uleb(&mut source, span.end);
            }
        }
        sections.push((SECTION_SOURCE, source));
    }
    assemble_sections(&sections)
}

struct DecodeBudget {
    steps: u64,
    decoded_est: usize,
    deadline: Instant,
}

impl DecodeBudget {
    fn tick(&mut self) -> Result<(), AbcDecodeError> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > MAX_VERIFY_WORK_STEPS {
            return Err(resource_err(
                ResourceKind::VerifySteps,
                "compact decode exceeded the work-step budget",
            ));
        }
        if self.steps & 0x3ff == 0 && Instant::now() > self.deadline {
            return Err(resource_err(
                ResourceKind::VerifyWall,
                "compact decode exceeded the wall budget",
            ));
        }
        Ok(())
    }

    fn charge(&mut self, amount: usize, what: &str) -> Result<(), AbcDecodeError> {
        self.decoded_est = self.decoded_est.saturating_add(amount);
        if self.decoded_est > MAX_DECODED_MODULE_BYTES {
            Err(resource_err(
                ResourceKind::DecodeMemory,
                format!(
                    "decoding {what} pushes the estimate to {} bytes, above {MAX_DECODED_MODULE_BYTES}",
                    self.decoded_est
                ),
            ))
        } else {
            Ok(())
        }
    }
}

struct Cursor<'a, 'b> {
    data: &'a [u8],
    pos: usize,
    budget: &'b mut DecodeBudget,
}

impl<'a, 'b> Cursor<'a, 'b> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], AbcDecodeError> {
        self.budget.tick()?;
        let end = self.pos.checked_add(len).ok_or_else(|| {
            format_err(
                "compact.decode.length-overflow",
                "length arithmetic overflow",
            )
        })?;
        if end > self.data.len() {
            return Err(format_err(
                "compact.decode.truncated",
                format!("need {len} bytes at {}, have {}", self.pos, self.data.len()),
            ));
        }
        let result = &self.data[self.pos..end];
        self.pos = end;
        Ok(result)
    }

    fn u8(&mut self) -> Result<u8, AbcDecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, AbcDecodeError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    fn u64(&mut self) -> Result<u64, AbcDecodeError> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes(bytes.try_into().expect("eight bytes")))
    }

    fn uleb(&mut self) -> Result<u32, AbcDecodeError> {
        let mut value = 0u32;
        for index in 0..5 {
            let byte = self.u8()?;
            if index == 4 && byte > 0x0f {
                return Err(format_err(
                    "compact.decode.uleb-overflow",
                    "u32 ULEB exceeds five bytes or its fourth-bit boundary",
                ));
            }
            value |= u32::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                if index > 0 && byte == 0 {
                    return Err(format_err(
                        "compact.decode.uleb-noncanonical",
                        "ULEB has a redundant zero group",
                    ));
                }
                return Ok(value);
            }
        }
        Err(format_err(
            "compact.decode.uleb-overflow",
            "u32 ULEB exceeds five bytes",
        ))
    }

    fn count(&mut self) -> Result<usize, AbcDecodeError> {
        Ok(self.uleb()? as usize)
    }

    fn finish(&self, what: &str) -> Result<(), AbcDecodeError> {
        if self.pos == self.data.len() {
            Ok(())
        } else {
            Err(format_err(
                "compact.decode.trailing-bytes",
                format!("{} trailing bytes in {what}", self.data.len() - self.pos),
            ))
        }
    }

    fn string(&mut self, what: &str) -> Result<String, AbcDecodeError> {
        let len = self.count()?;
        self.budget
            .charge(len + std::mem::size_of::<String>(), what)?;
        let bytes = self.take(len)?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| format_err("compact.decode.utf8", format!("{what} is not UTF-8")))?;
        Ok(text.to_string())
    }
}

#[derive(Clone, Copy)]
struct SectionDesc {
    id: u8,
    offset: usize,
    len: usize,
}

fn read_directory(
    bytes: &[u8],
    budget: &mut DecodeBudget,
) -> Result<[Option<SectionDesc>; 256], AbcDecodeError> {
    if bytes.len() < FIXED_HEADER_LEN {
        return Err(format_err(
            "compact.decode.truncated",
            "fixed header is truncated",
        ));
    }
    if bytes[..4] != COMPACT_ABC_MAGIC {
        return Err(format_err(
            "compact.decode.magic",
            "not an AVAC compact stream",
        ));
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != COMPACT_ABC_VERSION {
        return Err(format_err(
            "compact.decode.version",
            format!("unsupported compact version {version}"),
        ));
    }
    let profile = u16::from_le_bytes([bytes[6], bytes[7]]);
    if profile != COMPACT_ABC_PROFILE {
        return Err(format_err(
            "compact.decode.profile",
            format!("unsupported compact profile {profile}"),
        ));
    }
    if bytes[10] != 0 || bytes[11] != 0 {
        return Err(format_err(
            "compact.decode.reserved",
            "reserved header bits must be zero",
        ));
    }
    let count = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    if count > 256 {
        return Err(format_err(
            "compact.decode.sections",
            "section count exceeds the 256-id domain",
        ));
    }
    let directory_len = count
        .checked_mul(DIRECTORY_ENTRY_LEN)
        .ok_or_else(|| format_err("compact.decode.length-overflow", "directory overflow"))?;
    let directory_end = FIXED_HEADER_LEN
        .checked_add(directory_len)
        .ok_or_else(|| format_err("compact.decode.length-overflow", "directory overflow"))?;
    if directory_end > bytes.len() {
        return Err(format_err(
            "compact.decode.truncated",
            "section directory is truncated",
        ));
    }
    budget.charge(
        count * std::mem::size_of::<SectionDesc>(),
        "section directory",
    )?;
    let mut sections = [None; 256];
    let mut ordered = Vec::with_capacity(count);
    for index in 0..count {
        budget.tick()?;
        let at = FIXED_HEADER_LEN + index * DIRECTORY_ENTRY_LEN;
        let id = bytes[at];
        let offset = u32::from_le_bytes(bytes[at + 1..at + 5].try_into().expect("offset")) as usize;
        let len = u32::from_le_bytes(bytes[at + 5..at + 9].try_into().expect("length")) as usize;
        if sections[id as usize].is_some() {
            return Err(format_err(
                "compact.decode.duplicate-section",
                format!("duplicate section {id:#x}"),
            ));
        }
        if id < 0x80
            && !matches!(
                id,
                SECTION_MODULE
                    | SECTION_TYPES
                    | SECTION_CONSTS
                    | SECTION_IMPORTS
                    | SECTION_FUNCS
                    | SECTION_SOURCE
            )
        {
            return Err(format_err(
                "compact.decode.unknown-required-section",
                format!("unknown required section {id:#x}"),
            ));
        }
        let desc = SectionDesc { id, offset, len };
        sections[id as usize] = Some(desc);
        ordered.push(desc);
    }
    ordered.sort_by_key(|section| section.offset);
    let mut expected = directory_end;
    for section in ordered {
        let end = section.offset.checked_add(section.len).ok_or_else(|| {
            format_err("compact.decode.length-overflow", "section range overflow")
        })?;
        if section.offset != expected || end > bytes.len() {
            return Err(format_err(
                "compact.decode.section-layout",
                format!(
                    "section {:#x} range {}..{} is not contiguous from {expected} within {}",
                    section.id,
                    section.offset,
                    end,
                    bytes.len()
                ),
            ));
        }
        expected = end;
    }
    if expected != bytes.len() {
        return Err(format_err(
            "compact.decode.trailing-bytes",
            "section payload does not cover the complete input",
        ));
    }
    for id in [
        SECTION_MODULE,
        SECTION_TYPES,
        SECTION_CONSTS,
        SECTION_IMPORTS,
        SECTION_FUNCS,
    ] {
        if sections[id as usize].is_none() {
            return Err(format_err(
                "compact.decode.missing-section",
                format!("required section {id:#x} is missing"),
            ));
        }
    }
    Ok(sections)
}

fn section_cursor<'a, 'b>(
    bytes: &'a [u8],
    desc: SectionDesc,
    budget: &'b mut DecodeBudget,
) -> Cursor<'a, 'b> {
    Cursor {
        data: &bytes[desc.offset..desc.offset + desc.len],
        pos: 0,
        budget,
    }
}

fn add_module_entries(current: &mut usize, count: usize) -> Result<(), AbcDecodeError> {
    *current = current.saturating_add(count);
    check_module_tables(*current)
}

fn decode_function(
    bytes: &[u8],
    budget: &mut DecodeBudget,
    owner: &OwnerScope,
) -> Result<MirFunction, AbcDecodeError> {
    let mut c = Cursor {
        data: bytes,
        pos: 0,
        budget,
    };
    let sig = c.uleb()?;
    let slot_count = c.count()?;
    let mut function_entries = slot_count;
    c.budget.charge(
        slot_count * std::mem::size_of::<MirSlot>() + std::mem::size_of::<Vec<MirSlot>>(),
        "slot table",
    )?;
    let mut slots = Vec::with_capacity(slot_count.min(4096));
    for _ in 0..slot_count {
        let kind = match c.u8()? {
            0 => SlotKind::Param,
            1 => SlotKind::Local,
            2 => SlotKind::Temp,
            other => {
                return Err(format_err(
                    "compact.decode.slot-kind",
                    format!("unknown slot kind {other}"),
                ))
            }
        };
        let ty = c.uleb()?;
        slots.push(MirSlot {
            kind,
            ty,
            name: None,
        });
    }
    let entry = c.uleb()?;
    let block_count = c.count()?;
    function_entries = function_entries.saturating_add(block_count);
    if function_entries > MAX_TABLE_ENTRIES {
        return Err(resource_err(
            ResourceKind::TableEntries,
            "function slots and blocks exceed the table budget",
        ));
    }
    c.budget.charge(
        block_count * std::mem::size_of::<MirBlock>() + std::mem::size_of::<Vec<MirBlock>>(),
        "block table",
    )?;
    let mut blocks = Vec::with_capacity(block_count.min(4096));
    for _ in 0..block_count {
        let instr_count = c.count()?;
        function_entries = function_entries.saturating_add(instr_count);
        if function_entries > MAX_TABLE_ENTRIES {
            return Err(resource_err(
                ResourceKind::TableEntries,
                "function slots, blocks and instructions exceed the table budget",
            ));
        }
        c.budget.charge(
            instr_count * std::mem::size_of::<MirInstr>() + std::mem::size_of::<Vec<MirInstr>>(),
            "instruction table",
        )?;
        let mut instrs = Vec::with_capacity(instr_count.min(4096));
        for _ in 0..instr_count {
            let opcode = c.u8()?;
            let instr = match opcode {
                OP_CONST => MirInstr::Const {
                    dst: c.uleb()?,
                    value: c.uleb()?,
                },
                OP_COPY => MirInstr::Copy {
                    dst: c.uleb()?,
                    src: c.uleb()?,
                },
                OP_BIN => {
                    let op = match c.u8()? {
                        1 => BinOp::AddI32,
                        2 => BinOp::SubI32,
                        3 => BinOp::MulI32,
                        4 => BinOp::DivI32,
                        5 => BinOp::RemI32,
                        6 => BinOp::LtI32,
                        7 => BinOp::LeI32,
                        8 => BinOp::GtI32,
                        9 => BinOp::GeI32,
                        10 => BinOp::EqI32,
                        11 => BinOp::NeI32,
                        other => {
                            return Err(format_err(
                                "compact.decode.bin-subop",
                                format!("unknown binary sub-op {other}"),
                            ))
                        }
                    };
                    MirInstr::Bin {
                        op,
                        dst: c.uleb()?,
                        lhs: c.uleb()?,
                        rhs: c.uleb()?,
                    }
                }
                OP_UN => {
                    let op = match c.u8()? {
                        1 => UnOp::NegI32,
                        2 => UnOp::NotBool,
                        other => {
                            return Err(format_err(
                                "compact.decode.un-subop",
                                format!("unknown unary sub-op {other}"),
                            ))
                        }
                    };
                    MirInstr::Un {
                        op,
                        dst: c.uleb()?,
                        operand: c.uleb()?,
                    }
                }
                OP_CALL => {
                    let callee = match c.u8()? {
                        0 => CalleeRef::Func(c.uleb()?),
                        1 => CalleeRef::Import(c.uleb()?),
                        other => {
                            return Err(format_err(
                                "compact.decode.callee-kind",
                                format!("unknown callee kind {other}"),
                            ))
                        }
                    };
                    let dst = c.uleb()?;
                    let argc = c.count()?;
                    if argc > u8::MAX as usize {
                        return Err(format_err(
                            "compact.decode.call-arity",
                            "call arity exceeds the core verifier's u8 limit",
                        ));
                    }
                    c.budget.charge(
                        argc * std::mem::size_of::<SlotId>() + std::mem::size_of::<Vec<SlotId>>(),
                        "call arguments",
                    )?;
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(c.uleb()?);
                    }
                    MirInstr::Call { callee, dst, args }
                }
                OP_TRACE => MirInstr::Trace {
                    event: match c.u8()? {
                        1 => TraceEvent::MarkA,
                        2 => TraceEvent::MarkB,
                        other => {
                            return Err(format_err(
                                "compact.decode.trace-event",
                                format!("unknown trace event {other}"),
                            ))
                        }
                    },
                },
                other => {
                    return Err(format_err(
                        "compact.decode.opcode",
                        format!("unknown compact opcode {other:#x}"),
                    ))
                }
            };
            instrs.push(instr);
        }
        let term = match c.u8()? {
            TERM_JUMP => Terminator::Jump { target: c.uleb()? },
            TERM_BRANCH => Terminator::Branch {
                cond: c.uleb()?,
                then: c.uleb()?,
                else_: c.uleb()?,
            },
            TERM_RETURN => Terminator::Return { value: c.uleb()? },
            other => {
                return Err(format_err(
                    "compact.decode.terminator",
                    format!("unknown terminator {other:#x}"),
                ))
            }
        };
        blocks.push(MirBlock { instrs, term });
    }
    c.finish("function body")?;
    Ok(MirFunction {
        owner: owner.clone(),
        name: String::new(),
        sig,
        slots,
        blocks,
        entry,
        // PLAN-826 T-02 (PLAN-823 r8 env refresh): the scalar wire carries
        // no closure env layout — decoded functions are ordinary bodies.
        env: None,
        source_snapshot: 0,
        spans: Vec::new(),
    })
}

pub fn decode_compact_abc(bytes: &[u8]) -> Result<DecodedCompactAbc, AbcDecodeError> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(resource_err(
            ResourceKind::InputTooLarge,
            format!("input {} exceeds {MAX_INPUT_BYTES}", bytes.len()),
        ));
    }
    let owner = OwnerScope {
        schema: COMPACT_ABC_SCHEMA.to_string(),
        module: String::new(),
    };
    let mut budget = DecodeBudget {
        steps: 0,
        decoded_est: std::mem::size_of::<MirModule>()
            + AV_PROFILE.len()
            + owner.schema.len()
            + owner.module.len()
            + 3 * std::mem::size_of::<String>(),
        deadline: Instant::now() + MAX_VERIFY_WALL,
    };
    let sections = read_directory(bytes, &mut budget)?;

    let desc = sections[SECTION_MODULE as usize].expect("required module");
    let mut c = section_cursor(bytes, desc, &mut budget);
    let is_unit = match c.u8()? {
        0 => false,
        1 => true,
        other => {
            return Err(format_err(
                "compact.decode.kind",
                format!("unknown module kind {other}"),
            ))
        }
    };
    let entry = c.uleb()?;
    let capability_count = c.count()?;
    c.budget.charge(
        capability_count * std::mem::size_of::<String>() + std::mem::size_of::<Vec<String>>(),
        "capability table",
    )?;
    let mut capabilities = Vec::with_capacity(capability_count.min(64));
    for _ in 0..capability_count {
        capabilities.push(c.string("capability")?);
    }
    c.finish("module section")?;

    let desc = sections[SECTION_TYPES as usize].expect("required types");
    let mut c = section_cursor(bytes, desc, &mut budget);
    let type_count = c.count()?;
    let mut module_entries = type_count;
    add_module_entries(&mut module_entries, 0)?;
    c.budget.charge(
        type_count * std::mem::size_of::<MirType>() + std::mem::size_of::<Vec<MirType>>(),
        "type table",
    )?;
    let mut types = Vec::with_capacity(type_count.min(1024));
    for _ in 0..type_count {
        let ty = match c.u8()? {
            0 => MirType::I32,
            1 => MirType::Bool,
            2 => {
                let argc = c.count()?;
                if argc > u8::MAX as usize {
                    return Err(format_err(
                        "compact.decode.sig-arity",
                        "signature arity exceeds the core verifier's u8 limit",
                    ));
                }
                c.budget.charge(
                    argc * std::mem::size_of::<TypeIdx>() + std::mem::size_of::<Vec<TypeIdx>>(),
                    "signature parameters",
                )?;
                let mut params = Vec::with_capacity(argc);
                for _ in 0..argc {
                    params.push(c.uleb()?);
                }
                let result = c.uleb()?;
                MirType::Fn(FnSig { params, result })
            }
            other => {
                return Err(format_err(
                    "compact.decode.type-tag",
                    format!("unknown core type tag {other}"),
                ))
            }
        };
        types.push(ty);
    }
    c.finish("type section")?;

    let desc = sections[SECTION_CONSTS as usize].expect("required consts");
    let mut c = section_cursor(bytes, desc, &mut budget);
    let const_count = c.count()?;
    add_module_entries(&mut module_entries, const_count)?;
    c.budget.charge(
        const_count * std::mem::size_of::<MirConst>() + std::mem::size_of::<Vec<MirConst>>(),
        "constant table",
    )?;
    let mut consts = Vec::with_capacity(const_count.min(1024));
    for _ in 0..const_count {
        let constant = match c.u8()? {
            0 => MirConst::I32(c.u32()? as i32),
            1 => match c.u8()? {
                0 => MirConst::Bool(false),
                1 => MirConst::Bool(true),
                other => {
                    return Err(format_err(
                        "compact.decode.bool",
                        format!("boolean byte {other} is not 0/1"),
                    ))
                }
            },
            other => {
                return Err(format_err(
                    "compact.decode.const-tag",
                    format!("unknown core constant tag {other}"),
                ))
            }
        };
        consts.push(constant);
    }
    c.finish("constant section")?;

    let desc = sections[SECTION_IMPORTS as usize].expect("required imports");
    let mut c = section_cursor(bytes, desc, &mut budget);
    let import_count = c.count()?;
    add_module_entries(&mut module_entries, import_count)?;
    c.budget.charge(
        import_count * std::mem::size_of::<MirImport>() + std::mem::size_of::<Vec<MirImport>>(),
        "import table",
    )?;
    let mut imports = Vec::with_capacity(import_count.min(64));
    for _ in 0..import_count {
        let symbol = c.string("import symbol")?;
        let effect = match c.u8()? {
            0 => Effect::Observable,
            1 => Effect::Pure,
            other => {
                return Err(format_err(
                    "compact.decode.effect",
                    format!("unknown effect {other}"),
                ))
            }
        };
        let sig = c.uleb()?;
        imports.push(MirImport {
            symbol,
            effect,
            sig,
            // PLAN-826 T-02: scalar-wire imports never bind closure envs.
            env: None,
        });
    }
    c.finish("import section")?;

    let desc = sections[SECTION_FUNCS as usize].expect("required funcs");
    let mut c = section_cursor(bytes, desc, &mut budget);
    let function_count = c.count()?;
    add_module_entries(&mut module_entries, function_count)?;
    c.budget.charge(
        function_count
            * (std::mem::size_of::<MirFunction>()
                + owner.schema.len()
                + owner.module.len()
                + 2 * std::mem::size_of::<String>())
            + std::mem::size_of::<Vec<MirFunction>>(),
        "function table and owner scopes",
    )?;
    let mut funcs = Vec::with_capacity(function_count.min(1024));
    for _ in 0..function_count {
        let body_len = c.u32()? as usize;
        let body = c.take(body_len)?;
        funcs.push(decode_function(body, c.budget, &owner)?);
    }
    c.finish("function section")?;

    if let Some(desc) = sections[SECTION_SOURCE as usize] {
        let mut c = section_cursor(bytes, desc, &mut budget);
        for function in &mut funcs {
            function.source_snapshot = c.u64()?;
            let span_count = c.count()?;
            c.budget.charge(
                span_count * std::mem::size_of::<(u32, SpanInfo)>()
                    + std::mem::size_of::<Vec<(u32, SpanInfo)>>(),
                "source span table",
            )?;
            let mut spans = Vec::with_capacity(span_count.min(4096));
            for _ in 0..span_count {
                spans.push((
                    c.uleb()?,
                    SpanInfo {
                        func: c.uleb()?,
                        start: c.uleb()?,
                        end: c.uleb()?,
                    },
                ));
            }
            function.spans = spans;
        }
        c.finish("source section")?;
    }

    Ok(DecodedCompactAbc {
        version: COMPACT_ABC_VERSION,
        profile: COMPACT_ABC_PROFILE,
        module: MirModule {
            profile: AV_PROFILE.to_string(),
            owner,
            is_unit,
            types,
            consts,
            imports,
            funcs,
            entry,
            capabilities,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{MIR_SCHEMA, TRACE_CAPABILITY};
    use crate::mir_verify::verify_mir;

    fn owner() -> OwnerScope {
        OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: "compact-test".to_string(),
        }
    }

    fn function(
        m: &MirModule,
        name: &str,
        sig: TypeIdx,
        slots: Vec<MirSlot>,
        blocks: Vec<MirBlock>,
    ) -> MirFunction {
        MirFunction {
            owner: m.owner.clone(),
            name: name.to_string(),
            sig,
            slots,
            blocks,
            entry: 0,
            env: None,
            source_snapshot: 0,
            spans: Vec::new(),
        }
    }

    fn minimal_module() -> MirModule {
        let mut m = MirModule::new(owner());
        let i32_ty = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: i32_ty,
        }));
        let constant = m.intern_const(MirConst::I32(42));
        let f = function(
            &m,
            "main",
            sig,
            vec![MirSlot {
                kind: SlotKind::Local,
                ty: i32_ty,
                name: Some("answer".into()),
            }],
            vec![MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: 0,
                    value: constant,
                }],
                term: Terminator::Return { value: 0 },
            }],
        );
        m.funcs.push(f);
        m
    }

    fn sections_of(bytes: &[u8]) -> Vec<(u8, Vec<u8>)> {
        let count = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        (0..count)
            .map(|index| {
                let at = FIXED_HEADER_LEN + index * DIRECTORY_ENTRY_LEN;
                let id = bytes[at];
                let offset = u32::from_le_bytes(bytes[at + 1..at + 5].try_into().unwrap()) as usize;
                let len = u32::from_le_bytes(bytes[at + 5..at + 9].try_into().unwrap()) as usize;
                (id, bytes[offset..offset + len].to_vec())
            })
            .collect()
    }

    fn uleb_len(value: u32) -> usize {
        let mut out = Vec::new();
        put_uleb(&mut out, value);
        out.len()
    }

    fn wide_len(value: u32) -> usize {
        if value < 255 {
            1
        } else {
            5
        }
    }

    #[test]
    fn av_wire_826_candidate_comparison_and_density() {
        let values = [
            0,
            1,
            127,
            128,
            255,
            256,
            16383,
            16384,
            65535,
            65536,
            u32::MAX,
        ];
        let expected = [1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 5];
        for (value, len) in values.into_iter().zip(expected) {
            assert_eq!(uleb_len(value), len, "value {value}");
        }
        let uleb_total: usize = values.into_iter().map(uleb_len).sum();
        let wide_total: usize = values.into_iter().map(wide_len).sum();
        assert_eq!((uleb_total, wide_total), (25, 39));
        let minimal = encode_compact_abc(&minimal_module()).unwrap();
        assert_eq!(minimal.len(), 12 + 5 * 9 + 3 + 5 + 6 + 1 + 17);
        for value in [0, 1, 127] {
            let mut bin = vec![OP_BIN, 1];
            for operand in [value; 3] {
                put_uleb(&mut bin, operand);
            }
            let mut copy = vec![OP_COPY];
            for operand in [value; 2] {
                put_uleb(&mut copy, operand);
            }
            let mut ret = vec![TERM_RETURN];
            put_uleb(&mut ret, value);
            assert!(bin.len() <= 5);
            assert!(copy.len() <= 3);
            assert!(ret.len() <= 2);
            assert_eq!(bin, [3, 1, value as u8, value as u8, value as u8]);
            assert_eq!(copy, [2, value as u8, value as u8]);
            assert_eq!(ret, [0x12, value as u8]);
        }
    }

    #[test]
    fn av_wire_826_real_mir_roundtrip_and_verify() {
        let mut m = minimal_module();
        let i32_ty = 0;
        let helper_sig = m.types[1].clone();
        let helper_ty = m.intern_type(helper_sig);
        let forty = m.intern_const(MirConst::I32(40));
        let two = m.intern_const(MirConst::I32(2));
        let helper = function(
            &m,
            "helper",
            helper_ty,
            vec![MirSlot {
                kind: SlotKind::Local,
                ty: i32_ty,
                name: None,
            }],
            vec![MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: 0,
                    value: forty,
                }],
                term: Terminator::Return { value: 0 },
            }],
        );
        m.funcs.push(helper);
        m.funcs[0].slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: i32_ty,
            name: None,
        });
        m.funcs[0].slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: i32_ty,
            name: None,
        });
        m.funcs[0].blocks[0].instrs = vec![
            MirInstr::Const { dst: 1, value: two },
            MirInstr::Call {
                callee: CalleeRef::Func(1),
                dst: 0,
                args: vec![],
            },
            MirInstr::Bin {
                op: BinOp::AddI32,
                dst: 2,
                lhs: 0,
                rhs: 1,
            },
        ];
        m.funcs[0].blocks[0].term = Terminator::Return { value: 2 };
        let wire = encode_compact_abc(&m).unwrap();
        let decoded = decode_compact_abc(&wire).unwrap();
        verify_mir(&decoded.module).expect("real verifier accepts roundtrip");
        let reencoded = encode_compact_abc(&decoded.module).unwrap();
        assert_eq!(reencoded, wire);
    }

    #[test]
    fn av_wire_826_cfg_trace_unit_and_debug_roundtrip() {
        let mut m = minimal_module();
        let bool_ty = m.intern_type(MirType::Bool);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }));
        for symbol in ["hir.test.mark_a", "hir.test.mark_b"] {
            m.imports.push(MirImport {
                symbol: symbol.into(),
                effect: Effect::Observable,
                sig,
                env: None,
            });
        }
        m.capabilities.push(TRACE_CAPABILITY.into());
        m.funcs[0].slots.push(MirSlot {
            kind: SlotKind::Temp,
            ty: bool_ty,
            name: None,
        });
        let yes = m.intern_const(MirConst::Bool(true));
        m.funcs[0].blocks[0].instrs = vec![
            MirInstr::Const { dst: 1, value: yes },
            MirInstr::Trace {
                event: TraceEvent::MarkA,
            },
            MirInstr::Trace {
                event: TraceEvent::MarkB,
            },
        ];
        m.funcs[0].blocks[0].term = Terminator::Branch {
            cond: 1,
            then: 1,
            else_: 2,
        };
        m.funcs[0].blocks.push(MirBlock {
            instrs: vec![],
            term: Terminator::Jump { target: 2 },
        });
        m.funcs[0].blocks.push(MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: 0 }],
            term: Terminator::Return { value: 0 },
        });
        m.funcs[0].source_snapshot = 0x1234;
        m.funcs[0].spans = vec![(
            0,
            SpanInfo {
                func: 0,
                start: 10,
                end: 20,
            },
        )];
        let wire = encode_compact_abc(&m).unwrap();
        let decoded = decode_compact_abc(&wire).unwrap();
        verify_mir(&decoded.module).unwrap();
        assert_eq!(decoded.module.funcs[0].source_snapshot, 0x1234);
        assert_eq!(
            decoded.module.funcs[0].spans,
            vec![(
                0,
                SpanInfo {
                    func: 0,
                    start: 10,
                    end: 20
                }
            )]
        );
        assert_eq!(encode_compact_abc(&decoded.module).unwrap(), wire);
        m.funcs[0].source_snapshot = 0;
        m.funcs[0].spans.clear();
        let stripped = decode_compact_abc(&encode_compact_abc(&m).unwrap()).unwrap();
        verify_mir(&stripped.module).unwrap();
        assert_eq!(stripped.module.funcs[0].source_snapshot, 0);
        assert!(stripped.module.funcs[0].spans.is_empty());

        let mut unit = MirModule::unit(owner());
        let i32_ty = unit.intern_type(MirType::I32);
        let unit_sig = unit.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: i32_ty,
        }));
        unit.imports.push(MirImport {
            symbol: "fn:helper".into(),
            effect: Effect::Pure,
            sig: unit_sig,
            env: None,
        });
        unit.funcs.push(function(
            &unit,
            "only",
            unit_sig,
            vec![MirSlot {
                kind: SlotKind::Local,
                ty: i32_ty,
                name: None,
            }],
            vec![MirBlock {
                instrs: vec![MirInstr::Call {
                    callee: CalleeRef::Import(0),
                    dst: 0,
                    args: vec![],
                }],
                term: Terminator::Return { value: 0 },
            }],
        ));
        let decoded_unit = decode_compact_abc(&encode_compact_abc(&unit).unwrap()).unwrap();
        assert!(decoded_unit.module.is_unit);
        verify_mir(&decoded_unit.module).unwrap();
    }

    #[test]
    fn av_wire_826_integer_boundaries_and_invalid_refs() {
        let cases: &[(u32, &[u8])] = &[
            (0, &[0]),
            (127, &[127]),
            (128, &[0x80, 1]),
            (255, &[0xff, 1]),
            (256, &[0x80, 2]),
            (65535, &[0xff, 0xff, 3]),
            (65536, &[0x80, 0x80, 4]),
            (u32::MAX, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        ];
        for (value, expected) in cases {
            let mut out = Vec::new();
            put_uleb(&mut out, *value);
            assert_eq!(&out, expected);
            let mut budget = DecodeBudget {
                steps: 0,
                decoded_est: 0,
                deadline: Instant::now() + MAX_VERIFY_WALL,
            };
            let mut c = Cursor {
                data: &out,
                pos: 0,
                budget: &mut budget,
            };
            assert_eq!(c.uleb().unwrap(), *value);
        }
        for invalid in [
            &[0x80, 0][..],
            &[0xff, 0xff, 0xff, 0xff, 0x1f][..],
            &[0x80][..],
        ] {
            let mut budget = DecodeBudget {
                steps: 0,
                decoded_est: 0,
                deadline: Instant::now() + MAX_VERIFY_WALL,
            };
            let mut c = Cursor {
                data: invalid,
                pos: 0,
                budget: &mut budget,
            };
            assert!(c.uleb().is_err(), "accepted {invalid:x?}");
        }
        let mut m = minimal_module();
        m.funcs[0].blocks[0]
            .instrs
            .push(MirInstr::Copy { dst: 0, src: 65536 });
        let decoded = decode_compact_abc(&encode_compact_abc(&m).unwrap()).unwrap();
        let err = verify_mir(&decoded.module).unwrap_err();
        assert_eq!(err.code(), "mir.slot-ref");
    }

    #[test]
    fn av_wire_826_large_real_indices_do_not_truncate() {
        let mut m = MirModule::new(owner());
        let i32_ty = m.intern_type(MirType::I32);
        for _ in 0..256 {
            m.types.push(MirType::I32);
        }
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: i32_ty,
        }));
        let constant = m.intern_const(MirConst::I32(9));
        let slots = (0..=256)
            .map(|index| MirSlot {
                kind: SlotKind::Local,
                ty: index,
                name: None,
            })
            .collect::<Vec<_>>();
        let mut instrs = (0..=256)
            .map(|dst| MirInstr::Const {
                dst,
                value: constant,
            })
            .collect::<Vec<_>>();
        instrs.push(MirInstr::Copy { dst: 0, src: 256 });
        m.funcs.push(function(
            &m,
            "wide",
            sig,
            slots,
            vec![MirBlock {
                instrs,
                term: Terminator::Return { value: 0 },
            }],
        ));
        let wire = encode_compact_abc(&m).unwrap();
        let decoded = decode_compact_abc(&wire).unwrap();
        verify_mir(&decoded.module).unwrap();
        assert_eq!(decoded.module.funcs[0].slots[256].ty, 256);
        assert_eq!(encode_compact_abc(&decoded.module).unwrap(), wire);
    }

    #[test]
    fn av_wire_826_corrupt_sections_and_opcode_rejected() {
        let wire = encode_compact_abc(&minimal_module()).unwrap();
        for mut bad in [
            wire[..5].to_vec(),
            wire[..wire.len() - 1].to_vec(),
            [wire.clone(), vec![0]].concat(),
        ] {
            assert!(decode_compact_abc(&bad).is_err());
            bad.clear();
        }
        let mut duplicate = wire.clone();
        duplicate[FIXED_HEADER_LEN + DIRECTORY_ENTRY_LEN] = SECTION_MODULE;
        assert_eq!(
            decode_compact_abc(&duplicate).unwrap_err().code(),
            "compact.decode.duplicate-section"
        );
        let mut overlap = wire.clone();
        let at = FIXED_HEADER_LEN + DIRECTORY_ENTRY_LEN + 1;
        let old = u32::from_le_bytes(overlap[at..at + 4].try_into().unwrap());
        overlap[at..at + 4].copy_from_slice(&(old - 1).to_le_bytes());
        assert_eq!(
            decode_compact_abc(&overlap).unwrap_err().code(),
            "compact.decode.section-layout"
        );
        let mut sections = sections_of(&wire);
        sections.retain(|(id, _)| *id != SECTION_IMPORTS);
        assert_eq!(
            decode_compact_abc(&assemble_sections(&sections).unwrap())
                .unwrap_err()
                .code(),
            "compact.decode.missing-section"
        );
        let mut sections = sections_of(&wire);
        sections.push((0x40, vec![1, 2, 3]));
        assert_eq!(
            decode_compact_abc(&assemble_sections(&sections).unwrap())
                .unwrap_err()
                .code(),
            "compact.decode.unknown-required-section"
        );
        let mut sections = sections_of(&wire);
        sections.push((0x80, vec![1, 2, 3]));
        let accepted = decode_compact_abc(&assemble_sections(&sections).unwrap()).unwrap();
        verify_mir(&accepted.module).unwrap();
        let mut sections = sections_of(&wire);
        let funcs = sections
            .iter_mut()
            .find(|(id, _)| *id == SECTION_FUNCS)
            .unwrap();
        funcs.1.push(0);
        assert_eq!(
            decode_compact_abc(&assemble_sections(&sections).unwrap())
                .unwrap_err()
                .code(),
            "compact.decode.trailing-bytes"
        );
        let mut sections = sections_of(&wire);
        let funcs = sections
            .iter_mut()
            .find(|(id, _)| *id == SECTION_FUNCS)
            .unwrap();
        let body_len = u32::from_le_bytes(funcs.1[1..5].try_into().unwrap());
        funcs.1[1..5].copy_from_slice(&(body_len + 1).to_le_bytes());
        funcs.1.push(0);
        assert_eq!(
            decode_compact_abc(&assemble_sections(&sections).unwrap())
                .unwrap_err()
                .code(),
            "compact.decode.trailing-bytes"
        );
        let mut bad_opcode = wire.clone();
        let count = u16::from_le_bytes([wire[8], wire[9]]) as usize;
        for index in 0..count {
            let at = FIXED_HEADER_LEN + index * DIRECTORY_ENTRY_LEN;
            if wire[at] == SECTION_FUNCS {
                let offset = u32::from_le_bytes(wire[at + 1..at + 5].try_into().unwrap()) as usize;
                bad_opcode[offset + 12] = 0x7e;
            }
        }
        assert_eq!(
            decode_compact_abc(&bad_opcode).unwrap_err().code(),
            "compact.decode.opcode"
        );
    }

    #[test]
    fn av_wire_826_noncanonical_input_and_identity_rejected() {
        let wire = encode_compact_abc(&minimal_module()).unwrap();
        let mut sections = sections_of(&wire);
        let module = sections
            .iter_mut()
            .find(|(id, _)| *id == SECTION_MODULE)
            .unwrap();
        module.1.splice(1..2, [0x80, 0x00]);
        assert_eq!(
            decode_compact_abc(&assemble_sections(&sections).unwrap())
                .unwrap_err()
                .code(),
            "compact.decode.uleb-noncanonical"
        );
        let mut old = wire.clone();
        old[..4].copy_from_slice(b"AVAB");
        assert_eq!(
            decode_compact_abc(&old).unwrap_err().code(),
            "compact.decode.magic"
        );
        old[..4].copy_from_slice(b"AVAD");
        assert_eq!(
            decode_compact_abc(&old).unwrap_err().code(),
            "compact.decode.magic"
        );
        let data = MirModule::new_data(owner());
        assert_eq!(
            encode_compact_abc(&data).unwrap_err().code(),
            "compact.encode.profile"
        );
    }

    #[test]
    fn av_wire_826_decode_is_untrusted_until_verify() {
        let mut m = minimal_module();
        m.funcs[0].blocks[0].term = Terminator::Jump { target: 99 };
        let decoded = decode_compact_abc(&encode_compact_abc(&m).unwrap()).unwrap();
        assert_eq!(
            verify_mir(&decoded.module).unwrap_err().code(),
            "mir.jump-target"
        );
        let mut wrong_type = minimal_module();
        wrong_type.types.push(MirType::I64);
        assert!(encode_compact_abc(&wrong_type).is_err());
    }

    #[test]
    fn av_wire_826_preallocation_budgets() {
        assert_eq!(
            decode_compact_abc(&vec![0; MAX_INPUT_BYTES + 1])
                .unwrap_err()
                .code(),
            "av.resource.input-size"
        );
        let wire = encode_compact_abc(&minimal_module()).unwrap();
        let mut sections = sections_of(&wire);
        let types = sections
            .iter_mut()
            .find(|(id, _)| *id == SECTION_TYPES)
            .unwrap();
        types.1 = Vec::new();
        put_uleb(&mut types.1, 65537);
        assert_eq!(
            decode_compact_abc(&assemble_sections(&sections).unwrap())
                .unwrap_err()
                .code(),
            "av.resource.table-entries"
        );

        let mut m = MirModule::new(owner());
        let i32_ty = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: i32_ty,
        }));
        for _ in 0..60 {
            m.funcs.push(function(
                &m,
                "large",
                sig,
                vec![],
                vec![
                    MirBlock {
                        instrs: vec![],
                        term: Terminator::Jump { target: 0 }
                    };
                    6000
                ],
            ));
        }
        let large_wire = encode_compact_abc(&m).unwrap();
        assert!(large_wire.len() < MAX_INPUT_BYTES);
        assert_eq!(
            decode_compact_abc(&large_wire).unwrap_err().code(),
            "av.resource.decode-memory"
        );
    }
}
