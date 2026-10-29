//! Record/text data layout for the mn data+owned surface (PLAN-870 T-02
//! freeze §1): field offsets in DECLARATION order, the 12-byte/8-align Str
//! field shape `{u64 ptr @0, u32 len @8}` (bit-identical to the landed
//! 782 reference layout), and the Str-field path enumeration. All walks
//! are ITERATIVE (the frozen domain admits unbounded acyclic record
//! nesting — no host recursion anywhere in mn lowering).

use crate::MnError;
use auto_av_core::mir::{MirModule, MirType, TypeIdx};
use std::collections::BTreeMap;

/// One record type's frozen machine layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordLayout {
    pub size: u64,
    /// log2 of the record's alignment: 2 (4) or 3 (8) — the landed
    /// reference's stack-slot alignment ladder.
    pub align_log2: u32,
    pub field_off: Vec<u64>,
}

/// Byte size/alignment class of one field/slot type (the mn subset).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// I32/Bool/Char — 4 bytes, align 4.
    I32,
    /// Str / StrView — (ptr, len) pair: 12 bytes, align 8 in a record.
    Str,
}

pub fn shape_of(m: &MirModule, ty: TypeIdx) -> Result<Shape, MnError> {
    match m.types.get(ty as usize) {
        Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => Ok(Shape::I32),
        Some(MirType::Str) | Some(MirType::StrView) => Ok(Shape::Str),
        Some(MirType::I64) | Some(MirType::U64) | Some(MirType::F32) | Some(MirType::F64) => {
            Err(MnError::Unsupported(
                "type",
                format!("type {ty} is a wide numeric scalar outside the mn surface (R2-N)"),
            ))
        }
        other => Err(MnError::Unsupported(
            "type",
            format!("type {ty} ({other:?}) is outside the mn data/owned surface"),
        )),
    }
}

fn field_shape(
    m: &MirModule,
    ty: TypeIdx,
    cache: &mut BTreeMap<TypeIdx, RecordLayout>,
) -> Result<(u64, u64), MnError> {
    match m.types.get(ty as usize) {
        Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char) => Ok((4, 4)),
        Some(MirType::Str) | Some(MirType::StrView) => Ok((12, 8)),
        Some(MirType::Record(r)) => {
            let l = record_layout(m, ty, r, cache)?;
            Ok((l.size, 1u64 << l.align_log2))
        }
        Some(MirType::RecordView(r)) => {
            // A view field is the borrowed record's own bytes at rest; the
            // layout mirrors the underlying record (verify keeps views out
            // of record fields, so this arm is defensive only).
            let l = record_layout(m, ty, r, cache)?;
            Ok((l.size, 1u64 << l.align_log2))
        }
        Some(MirType::I64) | Some(MirType::U64) | Some(MirType::F32) | Some(MirType::F64) => {
            Err(MnError::Unsupported(
                "type",
                format!("type {ty} is a wide numeric scalar outside the mn surface (R2-N)"),
            ))
        }
        other => Err(MnError::Unsupported(
            "type",
            format!("type {ty} ({other:?}) is outside the mn data/owned surface"),
        )),
    }
}

/// Layout of one record type: declaration-order offsets, iterative DFS
/// over nested records with cycle detection (`mn.unsupported.layout`).
pub fn record_layout(
    m: &MirModule,
    ty: TypeIdx,
    _r: &auto_av_core::mir::RecordTypeRef,
    cache: &mut BTreeMap<TypeIdx, RecordLayout>,
) -> Result<RecordLayout, MnError> {
    if let Some(l) = cache.get(&ty) {
        return Ok(l.clone());
    }
    // In-progress marker: a record whose own construction is on the stack
    // of pending work is self-referential.
    let mut pending: Vec<TypeIdx> = vec![ty];
    let mut in_flight = std::collections::BTreeSet::from([ty]);
    while let Some(cur) = pending.last().copied() {
        // PLAN-881: the entry type may be a RecordView (a view base); its
        // layout equals the underlying record's.
        let Some(MirType::Record(cr) | MirType::RecordView(cr)) = m.types.get(cur as usize) else {
            return Err(MnError::Unsupported(
                "layout",
                format!("type {cur} is not a record but entered record layout"),
            ));
        };
        let mut blocked = false;
        for fd in &cr.fields {
            if let Some(MirType::Record(inner)) = m.types.get(fd.ty as usize) {
                let _ = inner;
                if !cache.contains_key(&fd.ty) {
                    if !in_flight.insert(fd.ty) {
                        return Err(MnError::Unsupported(
                            "layout",
                            format!("record type {ty} reaches itself (cyclic layout)"),
                        ));
                    }
                    pending.push(fd.ty);
                    blocked = true;
                    break;
                }
            }
        }
        if blocked {
            continue;
        }
        pending.pop();
        in_flight.remove(&cur);
        let mut off: u64 = 0;
        let mut align: u64 = 4;
        let mut field_off = Vec::with_capacity(cr.fields.len());
        for fd in &cr.fields {
            let (fsize, falign) = field_shape(m, fd.ty, cache)?;
            if fsize > u32::MAX as u64 || falign > 8 {
                return Err(MnError::Unsupported(
                    "layout",
                    format!(
                        "record type {ty} field `{}` shape ({fsize}/{falign}) out of range",
                        fd.name
                    ),
                ));
            }
            off = off.next_multiple_of(falign);
            field_off.push(off);
            off = off.saturating_add(fsize);
            align = align.max(falign);
        }
        let size = off.next_multiple_of(align);
        if size > u32::MAX as u64 {
            return Err(MnError::Unsupported(
                "layout",
                format!("record type {ty} size {size} above the u32 layout cap"),
            ));
        }
        let l = RecordLayout {
            size,
            align_log2: if align >= 8 { 3 } else { 2 },
            field_off,
        };
        cache.insert(cur, l.clone());
    }
    Ok(cache.get(&ty).expect("just computed").clone())
}

/// Every Str-field path of a record type in strict declaration-order DFS
/// (iterative — a nested record's paths interleave IN PLACE between its
/// siblings' paths). A path is a chain of declaration-order field
/// indices; `[0]` is the first field. Empty for a pure (resource-free)
/// record.
pub fn str_field_paths(m: &MirModule, ty: TypeIdx) -> Result<Vec<Vec<u16>>, MnError> {
    enum Item {
        Emit(Vec<u16>),
        Expand(TypeIdx, Vec<u16>),
    }
    let mut out: Vec<Vec<u16>> = Vec::new();
    let mut stack: Vec<Item> = vec![Item::Expand(ty, Vec::new())];
    while let Some(item) = stack.pop() {
        match item {
            Item::Emit(p) => out.push(p),
            Item::Expand(cur, prefix) => {
                let Some(MirType::Record(r)) = m.types.get(cur as usize) else {
                    return Err(MnError::Unsupported(
                        "layout",
                        format!("type {cur} is not a record but entered Str-path enumeration"),
                    ));
                };
                // Push children in REVERSE so they pop in declaration
                // order (a nested record's paths surface between its
                // earlier and later siblings', exactly like recursion).
                let mut children: Vec<Item> = Vec::new();
                for (i, fd) in r.fields.iter().enumerate() {
                    let mut p = prefix.clone();
                    p.push(i as u16);
                    match m.types.get(fd.ty as usize) {
                        Some(MirType::Str) => children.push(Item::Emit(p)),
                        Some(MirType::Record(_)) => children.push(Item::Expand(fd.ty, p)),
                        _ => {}
                    }
                }
                stack.extend(children.into_iter().rev());
            }
        }
    }
    Ok(out)
}

/// Offset of a field path within the record at `ty`.
pub fn path_offset(
    m: &MirModule,
    ty: TypeIdx,
    path: &[u16],
    cache: &mut BTreeMap<TypeIdx, RecordLayout>,
) -> Result<u64, MnError> {
    let mut off = 0u64;
    let mut cur = ty;
    for seg in path {
        // PLAN-881: a RecordView base (an owned-profile view param or a
        // method receiver) has the same field layout as its record.
        let Some(MirType::Record(r) | MirType::RecordView(r)) = m.types.get(cur as usize) else {
            return Err(MnError::Unsupported(
                "layout",
                format!("path segment {seg} escapes a non-record type {cur}"),
            ));
        };
        let l = record_layout(m, cur, r, cache)?;
        let i = *seg as usize;
        let Some(o) = l.field_off.get(i) else {
            return Err(MnError::Unsupported(
                "layout",
                format!("path segment {seg} out of range for type {cur}"),
            ));
        };
        off = off.saturating_add(*o);
        cur = r.fields[i].ty;
    }
    Ok(off)
}

/// The type index a field path resolves to.
pub fn path_ty(m: &MirModule, ty: TypeIdx, path: &[u16]) -> Result<TypeIdx, MnError> {
    let mut cur = ty;
    for seg in path {
        // PLAN-881: RecordView bases carry their record's field table.
        let Some(MirType::Record(r) | MirType::RecordView(r)) = m.types.get(cur as usize) else {
            return Err(MnError::Unsupported(
                "layout",
                format!("path segment {seg} escapes a non-record type {cur}"),
            ));
        };
        let i = *seg as usize;
        let Some(fd) = r.fields.get(i) else {
            return Err(MnError::Unsupported(
                "layout",
                format!("path segment {seg} out of range for type {cur}"),
            ));
        };
        cur = fd.ty;
    }
    Ok(cur)
}
