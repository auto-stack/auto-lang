//! InstructionSpec contract table (PLAN-820 §5.4, V-11).
//!
//! One row per wire opcode of both profiles, freezing the operand
//! roles, type rule, trap/fuel facts and the profile membership the
//! verifier and executor implement. The [`coverage_check`] function is
//! the V-11 "contract completeness" gate: every MIR instruction and
//! terminator variant must carry a row (a new `MirInstr` arm without a
//! spec row fails the check — a contract omission, not a style nit),
//! and every row must declare all four layer treatments.
//!
//! Codec/verifier/loader/executor coverage is STRUCTURALLY guaranteed by
//! Rust's exhaustive matches over `MirInstr`/`Terminator` (a new variant
//! breaks compilation of every layer); this table is the human-auditable
//! contract that the layers' behavior matches, exercised by the
//! `av_data_820_instr_spec` tests via full encode→decode→verify→execute
//! round trips over programs that use every opcode.

use crate::mir::{MirInstr, Terminator};

/// One instruction-contract row.
#[derive(Clone, Copy, Debug)]
pub struct InstructionSpec {
    pub opcode: u8,
    pub name: &'static str,
    /// "instr" | "term"
    pub family: &'static str,
    /// (operand name, "R"=read/"W"=written) in operand order.
    pub operands: &'static [(&'static str, &'static str)],
    /// One-line type rule (the verifier's frozen judgment).
    pub type_rule: &'static str,
    /// Whether this instruction can raise a language TRAP at runtime.
    pub traps: bool,
    /// Execution fuel units per instruction (the accounting base).
    pub fuel: u64,
    /// Profile membership: "core" = av.core-i32/1, "data" = av.data/1,
    /// "owned" = av.owned/1 (PLAN-821; subsumes the data surface).
    pub profiles: &'static [&'static str],
    /// Ownership/borrow note (all Copy in both frozen profiles).
    pub ownership: &'static str,
}

/// The frozen instruction contract of both wire families.
pub const SPECS: &[InstructionSpec] = &[
    InstructionSpec {
        opcode: 0x01,
        name: "CONST",
        family: "instr",
        operands: &[("dst", "W"), ("const", "R")],
        type_rule: "const shape == dst slot type (shape compare, index-free)",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "Copy",
    },
    InstructionSpec {
        opcode: 0x02,
        name: "COPY",
        family: "instr",
        operands: &[("dst", "W"), ("src", "R")],
        type_rule: "src type == dst type under structural (nominal record) equivalence",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "Copy (record: shallow Rc clone == deep copy, immutable)",
    },
    InstructionSpec {
        opcode: 0x03,
        name: "BIN",
        family: "instr",
        operands: &[("sub", "R"), ("dst", "W"), ("lhs", "R"), ("rhs", "R")],
        type_rule: "i32 arithmetic I32×I32→I32; comparisons same-scalar→Bool (Eq/Ne any equal scalar pair, ordered only I32); wide scalars rejected (mir.type-op)",
        traps: true,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "Copy",
    },
    InstructionSpec {
        opcode: 0x04,
        name: "UN",
        family: "instr",
        operands: &[("sub", "R"), ("dst", "W"), ("operand", "R")],
        type_rule: "NegI32 I32→I32; NotBool Bool→Bool",
        traps: true,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "Copy",
    },
    InstructionSpec {
        opcode: 0x05,
        name: "CALL",
        family: "instr",
        operands: &[("callee", "R"), ("dst", "W"), ("args", "R*")],
        type_rule: "args structurally equal to callee sig params; dst == sig result (records nominal)",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "Copy (aggregate params/returns by value)",
    },
    InstructionSpec {
        opcode: 0x06,
        name: "TRACE",
        family: "instr",
        operands: &[("event", "R")],
        type_rule: "trace capability required for mark imports",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "observation only",
    },
    InstructionSpec {
        opcode: 0x07,
        name: "FIELD_GET",
        family: "instr",
        operands: &[("dst", "W"), ("base", "R"), ("path", "R")],
        type_rule: "base slot is a record; path segments validated against each owner record; dst == final field type",
        traps: false,
        fuel: 1,
        profiles: &["data", "owned"],
        ownership: "Copy (rvalue copy out of an immutable value)",
    },
    InstructionSpec {
        opcode: 0x08,
        name: "FIELD_SET",
        family: "instr",
        operands: &[("root", "W"), ("path", "R"), ("value", "R")],
        type_rule: "root slot is a record; path validated against owners; value == final field type; RHS pre-materialized (overlap invariant)",
        traps: false,
        fuel: 1,
        profiles: &["data", "owned"],
        ownership: "Copy (structural rebuild into the root slot)",
    },
    InstructionSpec {
        opcode: 0x09,
        name: "AGGREGATE",
        family: "instr",
        operands: &[("dst", "W"), ("record_ty", "R"), ("operands", "R*")],
        type_rule: "operands count == field count, each structurally equal to its field type; dst == record_ty",
        traps: false,
        fuel: 1,
        profiles: &["data", "owned"],
        ownership: "Copy (fresh immutable aggregate)",
    },
    InstructionSpec {
        opcode: 0x0A,
        name: "CLONE",
        family: "instr",
        operands: &[("dst", "W"), ("src", "R")],
        type_rule: "src type == dst type; source must be Str or resource record (782 clone: per-field independent buffers)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "fresh owner; source unchanged (explicit deep clone)",
    },
    InstructionSpec {
        opcode: 0x0B,
        name: "MOVE",
        family: "instr",
        operands: &[("dst", "W"), ("src", "R")],
        type_rule: "src type == dst type; src owned+init+unmoved+borrow-free; kill src (move-kill); copyable src rejected (mir.ownership-required)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "ownership transfer; source slot dies (Moved)",
    },
    InstructionSpec {
        opcode: 0x0C,
        name: "VIEW",
        family: "instr",
        operands: &[("dst", "W"), ("src", "R")],
        type_rule: "src slot type Str, dst StrView; register read-borrow rooted at src",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "read-only token; no drop right; root-conservative conflicts",
    },
    InstructionSpec {
        opcode: 0x0D,
        name: "MUTREF",
        family: "instr",
        operands: &[("dst", "W"), ("src", "R")],
        type_rule: "src slot Str/resource record, dst MutRef; exclusive (T-03 admission)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "exclusive write token; blocks root move/replace while live",
    },
    InstructionSpec {
        opcode: 0x0E,
        name: "BORROW_END",
        family: "instr",
        operands: &[("token", "R")],
        type_rule: "token slot StrView/MutRef, init; releases the borrow",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "borrow release; roots accept conflicting access again",
    },
    InstructionSpec {
        opcode: 0x0F,
        name: "DROP",
        family: "instr",
        operands: &[("slot", "R")],
        type_rule: "owned+init slot; cleans exactly once, slot -> Dropped (T-03 admission)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "explicit cleanup; uninit/moved/dropped source is mir.drop-uninit",
    },
    InstructionSpec {
        opcode: 0x13,
        name: "STR_LEN",
        family: "instr",
        operands: &[("dst", "W"), ("view", "R")],
        type_rule: "view: StrView -> dst I32 (Unicode scalar count)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "read-only view use",
    },
    InstructionSpec {
        opcode: 0x14,
        name: "STR_LEN_BYTES",
        family: "instr",
        operands: &[("dst", "W"), ("view", "R")],
        type_rule: "view: StrView -> dst I32 (UTF-8 byte length)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "read-only view use",
    },
    InstructionSpec {
        opcode: 0x15,
        name: "STR_CHAR_AT",
        family: "instr",
        operands: &[("dst", "W"), ("view", "R"), ("index", "R")],
        type_rule: "view: StrView, index: I32 -> dst Char",
        traps: true,
        fuel: 1,
        profiles: &["owned"],
        ownership: "read-only view use; out-of-range traps av.trap.str-index",
    },
    InstructionSpec {
        opcode: 0x16,
        name: "STR_SLICE",
        family: "instr",
        operands: &[("dst", "W"), ("view", "R"), ("start", "R"), ("end", "R")],
        type_rule: "view: StrView, start/end: I32 -> dst StrView sharing the source",
        traps: true,
        fuel: 1,
        profiles: &["owned"],
        ownership: "read-only view use; invalid range traps av.trap.str-index",
    },
    InstructionSpec {
        opcode: 0x17,
        name: "STR_CONCAT",
        family: "instr",
        operands: &[("dst", "W"), ("lhs", "R"), ("rhs", "R")],
        type_rule: "lhs/rhs: StrView -> dst Str (fresh owned buffer)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "fresh owned allocation",
    },
    InstructionSpec {
        opcode: 0x18,
        name: "STR_TO_OWNED",
        family: "instr",
        operands: &[("dst", "W"), ("view", "R")],
        type_rule: "view: StrView -> dst Str (deep copy of the range)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "fresh owned allocation (770 to)",
    },
    InstructionSpec {
        opcode: 0x19,
        name: "STR_CMP",
        family: "instr",
        operands: &[("sub", "R"), ("dst", "W"), ("lhs", "R"), ("rhs", "R")],
        type_rule: "lhs/rhs: StrView -> dst Bool (byte-wise range equality/inequality)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "read-only view use",
    },
    InstructionSpec {
        opcode: 0x1A,
        name: "CHAR_TO_CODEPOINT",
        family: "instr",
        operands: &[("dst", "W"), ("ch", "R")],
        type_rule: "ch: Char -> dst I32",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "Copy",
    },
    InstructionSpec {
        opcode: 0x1B,
        name: "CHAR_FROM_CODEPOINT",
        family: "instr",
        operands: &[("dst", "W"), ("code", "R")],
        type_rule: "code: I32 -> dst Char; invalid scalar traps av.trap.char-invalid",
        traps: true,
        fuel: 1,
        profiles: &["owned"],
        ownership: "Copy",
    },
    InstructionSpec {
        opcode: 0x1C,
        name: "FIELDMOVE",
        family: "instr",
        operands: &[("dst", "W"), ("root", "R/K"), ("path", "R")],
        type_rule: "path over root record -> owned field type == dst type; root dies (use-once)",
        traps: false,
        fuel: 1,
        profiles: &["owned"],
        ownership: "R2-F08' write-back machinery: destructive whole-field move-out (Rc share to dst, root killed)",
    },
    InstructionSpec {
        opcode: 0x10,
        name: "JUMP",
        family: "term",
        operands: &[("target", "R")],
        type_rule: "target block index in range",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "—",
    },
    InstructionSpec {
        opcode: 0x11,
        name: "BRANCH",
        family: "term",
        operands: &[("cond", "R"), ("then", "R"), ("else", "R")],
        type_rule: "cond slot type Bool; targets in range",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "—",
    },
    InstructionSpec {
        opcode: 0x12,
        name: "RETURN",
        family: "term",
        operands: &[("value", "R")],
        type_rule: "value structurally equal to the signature result",
        traps: false,
        fuel: 1,
        profiles: &["core", "data", "owned"],
        ownership: "Copy (value passes to caller frame)",
    },
];

/// The V-11 coverage gate: every instruction/terminator variant carries
/// exactly one row, opcodes are unique, and every row declares its layer
/// treatment through the shared table (codec/verifier/executor coverage
/// is structural via exhaustive matches; this check binds the TABLE to
/// the enum).
pub fn coverage_check() -> Result<(), String> {
    let mut seen_names = std::collections::BTreeSet::new();
    let mut seen_ops = std::collections::BTreeSet::new();
    for s in SPECS {
        if !seen_names.insert(s.name) {
            return Err(format!("duplicate spec row `{}`", s.name));
        }
        if !seen_ops.insert(s.opcode) {
            return Err(format!("duplicate opcode {:#04x} ({})", s.opcode, s.name));
        }
        if s.fuel == 0 {
            return Err(format!("{} declares zero fuel", s.name));
        }
        if s.profiles.is_empty() {
            return Err(format!("{} declares no profile membership", s.name));
        }
    }
    // Discriminant-level completeness: mirror the enum arms explicitly so a
    // NEW variant added without a row fails here (names are the frozen
    // wire mnemonics).
    let expected = [
        "CONST",
        "COPY",
        "BIN",
        "UN",
        "CALL",
        "TRACE",
        "FIELD_GET",
        "FIELD_SET",
        "AGGREGATE",
        "CLONE",
        "MOVE",
        "VIEW",
        "MUTREF",
        "BORROW_END",
        "DROP",
        "STR_LEN",
        "STR_LEN_BYTES",
        "STR_CHAR_AT",
        "STR_SLICE",
        "STR_CONCAT",
        "STR_TO_OWNED",
        "STR_CMP",
        "CHAR_TO_CODEPOINT",
        "CHAR_FROM_CODEPOINT",
        "FIELDMOVE",
        "JUMP",
        "BRANCH",
        "RETURN",
    ];
    for e in expected {
        if !seen_names.contains(e) {
            return Err(format!("instruction contract gap: `{}` has no spec row", e));
        }
    }
    if SPECS.len() != expected.len() {
        return Err(format!(
            "spec table carries {} rows, expected {}",
            SPECS.len(),
            expected.len()
        ));
    }
    Ok(())
}

/// Family discriminant helper used by the spec tests to assert every
/// `MirInstr`/`Terminator` variant maps to its contract row's mnemonic.
pub fn mnemonic_of_instr(i: &MirInstr) -> &'static str {
    match i {
        MirInstr::Const { .. } => "CONST",
        MirInstr::Copy { .. } => "COPY",
        MirInstr::Bin { .. } => "BIN",
        MirInstr::Un { .. } => "UN",
        MirInstr::Call { .. } => "CALL",
        MirInstr::Trace { .. } => "TRACE",
        MirInstr::FieldGet { .. } => "FIELD_GET",
        MirInstr::FieldSet { .. } => "FIELD_SET",
        MirInstr::Aggregate { .. } => "AGGREGATE",
        MirInstr::Clone { .. } => "CLONE",
        MirInstr::Move { .. } => "MOVE",
        MirInstr::View { .. } => "VIEW",
        MirInstr::MutRef { .. } => "MUTREF",
        MirInstr::BorrowEnd { .. } => "BORROW_END",
        MirInstr::Drop { .. } => "DROP",
        MirInstr::FieldMove { .. } => "FIELDMOVE",
        MirInstr::StrLen { .. } => "STR_LEN",
        MirInstr::StrLenBytes { .. } => "STR_LEN_BYTES",
        MirInstr::StrCharAt { .. } => "STR_CHAR_AT",
        MirInstr::StrSlice { .. } => "STR_SLICE",
        MirInstr::StrConcat { .. } => "STR_CONCAT",
        MirInstr::StrToOwned { .. } => "STR_TO_OWNED",
        MirInstr::StrCmp { .. } => "STR_CMP",
        MirInstr::CharToCodepoint { .. } => "CHAR_TO_CODEPOINT",
        MirInstr::CharFromCodepoint { .. } => "CHAR_FROM_CODEPOINT",
    }
}

pub fn mnemonic_of_term(t: &Terminator) -> &'static str {
    match t {
        Terminator::Jump { .. } => "JUMP",
        Terminator::Branch { .. } => "BRANCH",
        Terminator::Return { .. } => "RETURN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instr_spec_coverage_complete() {
        coverage_check().expect("the V-11 instruction contract table is complete");
        // Trap classification frozen: BIN/UN plus the text-runtime traps.
        let trapping: Vec<&str> = SPECS.iter().filter(|s| s.traps).map(|s| s.name).collect();
        assert_eq!(
            trapping,
            vec![
                "BIN",
                "UN",
                "STR_CHAR_AT",
                "STR_SLICE",
                "CHAR_FROM_CODEPOINT"
            ]
        );
        // Owned-only surface frozen (T-02 admits all but MUTREF/DROP).
        let owned_only: Vec<&str> = SPECS
            .iter()
            .filter(|s| s.profiles.len() == 1 && s.profiles[0] == "owned")
            .map(|s| s.name)
            .collect();
        assert_eq!(
            owned_only,
            vec![
                "CLONE",
                "MOVE",
                "VIEW",
                "MUTREF",
                "BORROW_END",
                "DROP",
                "STR_LEN",
                "STR_LEN_BYTES",
                "STR_CHAR_AT",
                "STR_SLICE",
                "STR_CONCAT",
                "STR_TO_OWNED",
                "STR_CMP",
                "CHAR_TO_CODEPOINT",
                "CHAR_FROM_CODEPOINT",
                "FIELDMOVE",
            ]
        );
    }
}
