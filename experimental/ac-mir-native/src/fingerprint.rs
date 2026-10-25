//! Canonical, span-free semantic rendering of a MIR unit — the content
//! axis of A09' keys (contract-decision §9.2). Type indices and const
//! indices are resolved to SHAPES and VALUES so per-unit table remapping
//! (798 `split_units`) does not change the fingerprint; spans/snapshot
//! never enter. `fn:<name>` imports carry the callee's structural
//! signature, so a callee signature change flows into every caller's
//! fingerprint (callee-sig invalidation), while a body-only edit does not.

use crate::MnError;
use auto_av_core::contract::fnv1a64;
use auto_av_core::mir::*;

fn shape(m: &MirModule, idx: TypeIdx) -> &'static str {
    match m.type_shape(idx) {
        Some(ScalarShape::I32) => "i32",
        Some(ScalarShape::Bool) => "bool",
        None => "fn",
        // PLAN-820 shared-core widening: wide/record shapes never enter an
        // mn unit's fingerprint (rejected at admission) — a defensive
        // non-panicking label keeps the match total.
        Some(_) => "avdata",
    }
}

fn sig_shape(m: &MirModule, idx: TypeIdx) -> String {
    match m.sig_of(idx) {
        Some(s) => format!(
            "({})->{}",
            s.params
                .iter()
                .map(|p| shape(m, *p))
                .collect::<Vec<_>>()
                .join(","),
            shape(m, s.result)
        ),
        None => shape(m, idx).to_string(),
    }
}

fn const_value(m: &MirModule, idx: u32) -> String {
    match m.consts.get(idx as usize) {
        Some(c) => const_repr(c),
        None => "MISSING".to_string(),
    }
}

fn const_repr(c: &MirConst) -> String {
    match c {
        MirConst::I32(v) => format!("i32:{v}"),
        MirConst::Bool(b) => format!("bool:{}", if *b { 1 } else { 0 }),
        // PLAN-820 shared-core widening (defensive total match; wide
        // constants are rejected before fingerprinting).
        other => format!("avdata:{:?}", std::mem::discriminant(other)),
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n")
}

/// Canonical text rendering of one unit (single function).
pub fn render_unit(unit: &MirModule) -> Result<String, MnError> {
    if !unit.is_unit || unit.funcs.len() != 1 {
        return Err(MnError::Unsupported(
            "unit-shape",
            "fingerprint input must be a single-function unit".to_string(),
        ));
    }
    let f = &unit.funcs[0];
    let mut caps = unit.capabilities.clone();
    caps.sort();
    let mut s = String::new();
    s.push_str(&format!(
        "mir-schema {}\nprofile {}\nowner {}|{}\nunit {}\ncaps {}\n",
        escape(&unit.owner.schema),
        escape(&unit.profile),
        escape(&unit.owner.schema),
        escape(&unit.owner.module),
        escape(&f.name),
        caps.join(",")
    ));
    s.push_str(&format!("sig {}\n", sig_shape(unit, f.sig)));
    s.push_str(&format!(
        "slots {}\n",
        f.slots
            .iter()
            .map(|sl| format!("{}:{}", kind_tag(sl.kind), shape(unit, sl.ty)))
            .collect::<Vec<_>>()
            .join(",")
    ));
    s.push_str(&format!(
        "imports {}\n",
        unit.imports
            .iter()
            .map(|i| format!("{}|{}", escape(&i.symbol), sig_shape(unit, i.sig)))
            .collect::<Vec<_>>()
            .join(",")
    ));
    s.push_str(&format!(
        "consts {}\n",
        unit.consts
            .iter()
            .map(const_repr)
            .collect::<Vec<_>>()
            .join(",")
    ));
    for (bi, b) in f.blocks.iter().enumerate() {
        s.push_str(&format!("block {bi}\n"));
        for i in &b.instrs {
            s.push_str(&render_instr(unit, i)?);
            s.push('\n');
        }
        s.push_str(&render_term(&b.term)?);
        s.push('\n');
    }
    Ok(s)
}

fn kind_tag(k: SlotKind) -> &'static str {
    match k {
        SlotKind::Param => "p",
        SlotKind::Local => "l",
        SlotKind::Temp => "t",
    }
}

fn render_instr(m: &MirModule, i: &MirInstr) -> Result<String, MnError> {
    Ok(match i {
        MirInstr::Const { dst, value } => format!("const {} = {}", dst, const_value(m, *value)),
        MirInstr::Copy { dst, src } => format!("copy {} <- {}", dst, src),
        MirInstr::Bin { op, dst, lhs, rhs } => format!("bin {:?} {} {} {}", op, dst, lhs, rhs),
        MirInstr::Un { op, dst, operand } => format!("un {:?} {} {}", op, dst, operand),
        MirInstr::Call { callee, dst, args } => match callee {
            CalleeRef::Func(_) => {
                return Err(MnError::Unsupported(
                    "unit-callee",
                    "a unit must address callees through imports".to_string(),
                ))
            }
            CalleeRef::Import(ii) => {
                let Some(imp) = m.imports.get(*ii as usize) else {
                    return Err(MnError::Unsupported(
                        "import-ref",
                        format!("call references out-of-range import {ii}"),
                    ));
                };
                format!(
                    "call {} {} ({})",
                    escape(&imp.symbol),
                    dst,
                    args.iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        },
        MirInstr::Trace { event } => format!("trace {event:?}"),
        // PLAN-820 shared-core widening: the data-profile instructions are
        // outside mn's frozen instruction surface — a fingerprint render
        // (pure formatting) must not panic on them.
        MirInstr::FieldGet { dst, base, path } => {
            format!("fieldget {dst} {base} {:?}", path)
        }
        MirInstr::FieldSet { root, path, value } => {
            format!("fieldset {root} {:?} {value}", path)
        }
        MirInstr::Aggregate { dst, record_ty, operands } => {
            format!("aggregate {dst} {record_ty} {:?}", operands)
        }
    })
}

fn render_term(t: &Terminator) -> Result<String, MnError> {
    Ok(match t {
        Terminator::Jump { target } => format!("jump {target}"),
        Terminator::Branch { cond, then, else_ } => format!("branch {cond} {then} {else_}"),
        Terminator::Return { value } => format!("return {value}"),
    })
}

/// FNV-1a64 over the canonical rendering (change detector, 783/798 key
/// primitive; the object's own content binds separately via blake3).
pub fn unit_fingerprint(unit: &MirModule) -> Result<String, MnError> {
    Ok(format!("{:016x}", fnv1a64(render_unit(unit)?.as_bytes())))
}
