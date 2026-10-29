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

fn shape(m: &MirModule, idx: TypeIdx) -> String {
    match m.types.get(idx as usize) {
        Some(MirType::I32) => "i32".into(),
        Some(MirType::Bool) => "bool".into(),
        // PLAN-870 (freeze §6): the data/owned shapes render DISTINCT,
        // nominal labels — the old defensive "fn"/"avdata" labels collided
        // (a Record slot and an Fn param hashed identically), which would
        // have made unit cache keys unsound for owned units.
        Some(MirType::Char) => "char".into(),
        Some(MirType::Str) => "str".into(),
        Some(MirType::StrView) => "strview".into(),
        Some(MirType::Fn(_)) => "fn".into(),
        Some(MirType::MutRef) => "mutref".into(),
        Some(MirType::Record(r)) => record_shape(m, r),
        Some(MirType::RecordView(r)) => format!("view:{}", record_shape(m, r)),
        // Wide numerics stay outside the mn surface; a defensive distinct
        // label keeps the render total for rejected units.
        Some(
            MirType::I64
            | MirType::U64
            | MirType::F32
            | MirType::F64
            | MirType::List(_)
            | MirType::Variant(_)
            | MirType::Place(_)
            | MirType::Closure(_)
            | MirType::Dyn
            // PLAN-862's entry-unit Unit type: outside mn's three admitted
            // profiles; a defensive distinct label keeps the render total.
            | MirType::Unit,
        ) => "unsupported".into(),
        None => "missing".into(),
    }
}

/// Nominal record shape: identity is (owner, name, ordered field shapes)
/// — same-width different-name records never render alike (Design44).
fn record_shape(m: &MirModule, r: &RecordTypeRef) -> String {
    let fields = r
        .fields
        .iter()
        .map(|f| format!("{}:{}", f.name, shape(m, f.ty)))
        .collect::<Vec<_>>()
        .join(",");
    format!("rec({}|{}:{})", r.owner_module, r.name, fields)
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
        // PLAN-870 (freeze §6): Char/Str consts render by VALUE — the old
        // discriminant label made every string literal hash identically.
        MirConst::Char(ch) => format!("char:{}", *ch as u32),
        MirConst::Str(bytes) => format!("str:{}:{:016x}", bytes.len(), fnv1a64(bytes)),
        // Wide numeric constants stay rejected before fingerprinting; the
        // discriminant label remains a defensive total arm.
        other => format!("wide:{:?}", std::mem::discriminant(other)),
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n")
}

/// Canonical text rendering of one unit (single function).
pub fn render_unit(unit: &MirModule) -> Result<String, MnError> {
    if unit.is_host_console_profile() || !unit.host_imports.is_empty() {
        return Err(MnError::Unsupported(
            "host-console",
            "console declarations are outside the native fingerprint contract".into(),
        ));
    }

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
        MirInstr::HostCall { .. } => {
            return Err(MnError::Unsupported(
                "host-console",
                "host calls are outside the native profile".into(),
            ))
        }
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
        MirInstr::Aggregate {
            dst,
            record_ty,
            operands,
        } => {
            format!("aggregate {dst} {record_ty} {:?}", operands)
        }
        // PLAN-821 shared-core widening: the owned-profile instructions
        // render (pure formatting — a fingerprint must not panic on
        // them); the lower surface rejects them below.
        MirInstr::Clone { dst, src } => format!("clone {dst} {src}"),
        MirInstr::Move { dst, src } => format!("move {dst} {src}"),
        MirInstr::View { dst, src } => format!("view {dst} {src}"),
        MirInstr::MutRef { dst, src } => format!("mutref {dst} {src}"),
        MirInstr::BorrowEnd { token } => format!("borrowend {token}"),
        MirInstr::Drop { slot } => format!("drop {slot}"),
        MirInstr::StrLen { dst, view } => format!("strlen {dst} {view}"),
        MirInstr::StrLenBytes { dst, view } => format!("strlenbytes {dst} {view}"),
        MirInstr::StrCharAt { dst, view, index } => {
            format!("strcharat {dst} {view} {index}")
        }
        MirInstr::StrSlice {
            dst,
            view,
            start,
            end,
        } => format!("strslice {dst} {view} {start} {end}"),
        MirInstr::StrConcat { dst, lhs, rhs } => format!("strconcat {dst} {lhs} {rhs}"),
        MirInstr::StrToOwned { dst, view } => format!("toowned {dst} {view}"),
        MirInstr::StrCmp { eq, dst, lhs, rhs } => format!("strcmp {eq} {dst} {lhs} {rhs}"),
        MirInstr::CharToCodepoint { dst, ch } => format!("char2cp {dst} {ch}"),
        MirInstr::CharFromCodepoint { dst, code } => format!("cp2char {dst} {code}"),
        // PLAN-821 R2-F08' widening: the write-back extraction renders;
        // the lower surface rejects it below.
        MirInstr::FieldMove { dst, root, path } => {
            format!("fieldmove {dst} {root} {:?}", path)
        }
        // PLAN-822 shared-core widening: the collections-profile
        // instructions render (pure formatting — a fingerprint must not
        // panic on them); the lower surface rejects them.
        MirInstr::ListNew { dst, list_ty, cap } => {
            format!("listnew {dst} t{list_ty} c{cap}")
        }
        MirInstr::ListLen { dst, src } => format!("listlen {dst} {src}"),
        MirInstr::ListCap { dst, src } => format!("listcap {dst} {src}"),
        MirInstr::ListGet { dst, base, index } => {
            format!("listget {dst} {base} {index}")
        }
        MirInstr::ListPush { root, value } => format!("listpush {root} {value}"),
        MirInstr::ListInsert { root, index, value } => {
            format!("listinsert {root} {index} {value}")
        }
        MirInstr::ListSet { root, index, value } => {
            format!("listset {root} {index} {value}")
        }
        MirInstr::ListRemove { dst, root, index } => {
            format!("listremove {dst} {root} {index}")
        }
        MirInstr::ListFieldGet {
            dst,
            base,
            index,
            path,
        } => format!("listfieldget {dst} {base} {index} {:?}", path),
        MirInstr::ListFieldSet {
            root,
            index,
            path,
            value,
        } => format!("listfieldset {root} {index} {:?} {value}", path),
        MirInstr::VariantPack {
            dst,
            variant_ty,
            disc,
            payload,
        } => format!(
            "variantpack {dst} t{variant_ty} d{disc} p{}",
            payload.map(|p| p.to_string()).unwrap_or_else(|| "-".into())
        ),
        MirInstr::VariantDisc { dst, src } => format!("variantdisc {dst} {src}"),
        MirInstr::VariantGuard { dst, src, disc } => {
            format!("variantguard {dst} {src} d{disc}")
        }
        MirInstr::VariantProject { dst, root, disc } => {
            format!("variantproject {dst} {root} d{disc}")
        }
        MirInstr::VariantSwitch {
            root,
            disc,
            payload,
        } => format!(
            "variantswitch {root} d{disc} p{}",
            payload.map(|p| p.to_string()).unwrap_or_else(|| "-".into())
        ),
        MirInstr::ListPlace { dst, base, index } => {
            format!("listplace {dst} {base} {index}")
        }
        MirInstr::PlaceGet { dst, src } => format!("placeget {dst} {src}"),
        MirInstr::PlaceSet { place, value } => format!("placeset {place} {value}"),
        MirInstr::CreateClosure { .. } | MirInstr::CallClosure { .. } | MirInstr::EnvGet { .. } => {
            return Err(MnError::Unsupported(
                "instruction",
                "closure instructions are outside the scalar native surface".into(),
            ));
        }
        // PLAN-824 shared-core widening: the dynamic-profile instructions
        // render (pure formatting — a fingerprint must not panic on
        // them); the lower surface rejects them below.
        MirInstr::DynPack { dst, src } => format!("dynpack {dst} {src}"),
        MirInstr::DynIs { dst, src, ty } => format!("dynis {dst} {src} t{ty}"),
        MirInstr::DynTake { dst, src, ty } => format!("dyntake {dst} {src} t{ty}"),
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
