//! Logical layout descriptors for the `av.data/1` profile (PLAN-820,
//! contract-decision §6).
//!
//! A layout is DERIVED from the checked type graph — an input's own claim
//! about Copy/drop/offsets has no admission surface to forge (V-12: there
//! is nothing to trust on disk; the wire carries no descriptor). Scalars
//! have their exact logical width/alignment; records use incrementally
//! aligned C-style field placement with tail padding. All arithmetic is
//! CHECKED: a layout overflow is a verification rejection
//! (`mir.layout-overflow`), never a panic and never a published layout.
//!
//! These descriptors are LOGICAL: the executor stores values as the `Value`
//! enum and never reinterprets a Rust object at an offset. Offsets are
//! diagnostic/reporting facts (and the future native-ABI seam), not memory
//! addresses in this interpreter.
//!
//! Traversals are ITERATIVE with an explicit work budget — the type graph
//! is finite (acyclic by verification) but a hand-made bundle must not be
//! able to make this layer recurse on the host stack.

use crate::contract::{MAX_RECORD_FIELDS, MAX_VERIFY_WORK_STEPS};
use crate::mir::{MirModule, MirType, RecordTypeRef, TypeIdx};

/// Logical width/alignment of one scalar kind (bytes).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScalarLayout {
    pub size: u64,
    pub align: u64,
}

pub fn scalar_layout(t: &MirType) -> Option<ScalarLayout> {
    Some(match t {
        MirType::I32 => ScalarLayout { size: 4, align: 4 },
        MirType::Bool => ScalarLayout { size: 1, align: 1 },
        MirType::I64 => ScalarLayout { size: 8, align: 8 },
        MirType::U64 => ScalarLayout { size: 8, align: 8 },
        MirType::F32 => ScalarLayout { size: 4, align: 4 },
        MirType::F64 => ScalarLayout { size: 8, align: 8 },
        MirType::Fn(_) | MirType::Record(_) => return None,
    })
}

/// One field's logical placement inside its record.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FieldLayout {
    pub name: String,
    pub ty: TypeIdx,
    pub offset: u64,
    pub size: u64,
    pub align: u64,
}

/// A derived record layout. `is_copy` is `true` for every type of this
/// profile (pure values, no resources) — the flag exists so consumers can
/// not silently assume it; non-Copy kinds arrive with future profiles.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LayoutDesc {
    pub size: u64,
    pub align: u64,
    pub fields: Vec<FieldLayout>,
    pub is_copy: bool,
}

/// Layout derivation failure taxonomy. `Type` errors are invalid inputs
/// (invalid/unsupported classification); `Budget` is a resource rejection;
/// `Cycle` is by-value recursion (defense in depth behind `mir.type-cycle`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LayoutError {
    /// The type index does not name a non-empty record in the table.
    NotRecord(TypeIdx),
    /// A field count exceeds the frozen per-record guard.
    TooManyFields(usize),
    /// Checked size/align/offset arithmetic overflowed.
    Overflow,
    /// The record type graph is cyclic.
    Cycle,
    /// Derivation work exceeded the verification work-step ceiling.
    Budget,
}

impl LayoutError {
    pub fn code(&self) -> &'static str {
        match self {
            LayoutError::NotRecord(_) => "mir.layout-type",
            LayoutError::TooManyFields(_) => "mir.layout-fields",
            LayoutError::Overflow => "mir.layout-overflow",
            LayoutError::Cycle => "mir.type-cycle",
            LayoutError::Budget => "av.resource.verify-steps",
        }
    }
}

/// Compute the logical layout of `record_ty` within `module`'s type table.
/// Nested record fields contribute their OWN derived size/align — an
/// iterative post-order DFS (explicit `(node, field-cursor)` stack, three
/// colors) memoizes per type index; a GRAY revisit is a cycle.
pub fn compute_layout(module: &MirModule, record_ty: TypeIdx) -> Result<LayoutDesc, LayoutError> {
    let n = module.types.len();
    if record_ty as usize >= n {
        return Err(LayoutError::NotRecord(record_ty));
    }
    let mut steps: u64 = 0;
    // 0 = white, 1 = gray (on the DFS stack), 2 = black (memoized).
    let mut color = vec![0u8; n];
    let mut memo: Vec<Option<LayoutDesc>> = vec![None; n];
    color[record_ty as usize] = 1;
    let mut stack: Vec<(TypeIdx, usize)> = vec![(record_ty, 0)];
    while !stack.is_empty() {
        steps += 1;
        if steps > MAX_VERIFY_WORK_STEPS {
            return Err(LayoutError::Budget);
        }
        let cur = stack.last().expect("non-empty").0;
        let mut cursor = stack.last().expect("non-empty").1;
        let r = match module.types.get(cur as usize) {
            Some(MirType::Record(r)) => r,
            _ => return Err(LayoutError::NotRecord(cur)),
        };
        if r.fields.is_empty() {
            return Err(LayoutError::NotRecord(cur)); // empty records unsupported (§2)
        }
        if r.fields.len() > MAX_RECORD_FIELDS {
            return Err(LayoutError::TooManyFields(r.fields.len()));
        }
        // Advance the field cursor past scalars and black records; descend
        // into the first unresolved record-typed field.
        let mut descended = false;
        while cursor < r.fields.len() {
            let fty = r.fields[cursor].ty;
            match module.types.get(fty as usize) {
                Some(MirType::Record(_)) => match color[fty as usize] {
                    1 => return Err(LayoutError::Cycle),
                    2 => {
                        cursor += 1;
                    }
                    _ => {
                        color[fty as usize] = 1;
                        stack.last_mut().expect("non-empty").1 = cursor;
                        stack.push((fty, 0));
                        descended = true;
                        break;
                    }
                },
                Some(_) => {
                    cursor += 1;
                }
                None => return Err(LayoutError::NotRecord(fty)),
            }
        }
        if descended {
            continue;
        }
        // Every field is a scalar or a black (memoized) record: place them.
        steps += r.fields.len() as u64;
        if steps > MAX_VERIFY_WORK_STEPS {
            return Err(LayoutError::Budget);
        }
        let desc = place_fields(module, r, &memo)?;
        memo[cur as usize] = Some(desc);
        color[cur as usize] = 2;
        stack.pop();
    }
    match &memo[record_ty as usize] {
        Some(l) => Ok(l.clone()),
        None => Err(LayoutError::NotRecord(record_ty)),
    }
}

fn place_fields(
    module: &MirModule,
    r: &RecordTypeRef,
    memo: &[Option<LayoutDesc>],
) -> Result<LayoutDesc, LayoutError> {
    let mut offset: u64 = 0;
    let mut max_align: u64 = 1;
    let mut fields_out: Vec<FieldLayout> = Vec::with_capacity(r.fields.len());
    for f in &r.fields {
        let (fsize, falign) = match module.types.get(f.ty as usize) {
            Some(MirType::Record(_)) => {
                let d = memo
                    .get(f.ty as usize)
                    .and_then(|m| m.as_ref())
                    .ok_or(LayoutError::NotRecord(f.ty))?;
                (d.size, d.align)
            }
            Some(t) => {
                let s = scalar_layout(t).ok_or(LayoutError::NotRecord(f.ty))?;
                (s.size, s.align)
            }
            None => return Err(LayoutError::NotRecord(f.ty)),
        };
        // offset rounded up to the field's alignment (checked).
        let rem = offset.checked_rem(falign).ok_or(LayoutError::Overflow)?;
        let pad = if rem == 0 { 0 } else { falign - rem };
        offset = offset.checked_add(pad).ok_or(LayoutError::Overflow)?;
        fields_out.push(FieldLayout {
            name: f.name.clone(),
            ty: f.ty,
            offset,
            size: fsize,
            align: falign,
        });
        offset = offset.checked_add(fsize).ok_or(LayoutError::Overflow)?;
        max_align = max_align.max(falign);
    }
    // Tail padding to the record's own alignment.
    let rem = offset.checked_rem(max_align).ok_or(LayoutError::Overflow)?;
    let pad = if rem == 0 { 0 } else { max_align - rem };
    let size = offset.checked_add(pad).ok_or(LayoutError::Overflow)?;
    Ok(LayoutDesc {
        size,
        align: max_align,
        fields: fields_out,
        is_copy: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::MIR_DATA_SCHEMA;
    use crate::mir::{MirFieldDef, OwnerScope, RecordTypeRef};

    fn owner() -> OwnerScope {
        OwnerScope {
            schema: MIR_DATA_SCHEMA.to_string(),
            module: "m".to_string(),
        }
    }

    #[test]
    fn layout_scalar_and_nested_checked() {
        let mut m = MirModule::new_data(owner());
        let t_i32 = m.intern_type(MirType::I32);
        let t_bool = m.intern_type(MirType::Bool);
        let t_i64 = m.intern_type(MirType::I64);
        let inner = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "m".into(),
            name: "Inner".into(),
            fields: vec![
                MirFieldDef {
                    name: "x".into(),
                    ty: t_i32,
                },
                MirFieldDef {
                    name: "b".into(),
                    ty: t_bool,
                },
            ],
        }));
        let outer = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "m".into(),
            name: "Outer".into(),
            fields: vec![
                MirFieldDef {
                    name: "inner".into(),
                    ty: inner,
                },
                MirFieldDef {
                    name: "wide".into(),
                    ty: t_i64,
                },
            ],
        }));
        let li = compute_layout(&m, inner).expect("inner layout");
        // i32 at 0 (size 4), bool padded to 4, size padded to align 4.
        assert_eq!(li.fields[0].offset, 0);
        assert_eq!(li.fields[1].offset, 4);
        assert_eq!(li.size, 8);
        assert_eq!(li.align, 4);
        let lo = compute_layout(&m, outer).expect("outer layout");
        assert_eq!(lo.fields[0].offset, 0);
        assert_eq!(lo.fields[0].size, 8);
        // i64 field aligned to 8 after the 8-byte inner record.
        assert_eq!(lo.fields[1].offset, 8);
        assert_eq!(lo.size, 16);
        assert_eq!(lo.align, 8);
        assert!(lo.is_copy);
    }

    #[test]
    fn layout_rejects_non_record_and_empty() {
        let mut m = MirModule::new_data(owner());
        let t_i32 = m.intern_type(MirType::I32);
        assert_eq!(
            compute_layout(&m, t_i32).unwrap_err().code(),
            "mir.layout-type"
        );
        let empty = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "m".into(),
            name: "E".into(),
            fields: vec![],
        }));
        assert_eq!(
            compute_layout(&m, empty).unwrap_err().code(),
            "mir.layout-type"
        );
    }

    #[test]
    fn layout_cycle_rejected_defensively() {
        let mut m = MirModule::new_data(owner());
        let a = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "m".into(),
            name: "A".into(),
            fields: vec![MirFieldDef {
                name: "b".into(),
                ty: 1,
            }],
        }));
        let _b = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "m".into(),
            name: "B".into(),
            fields: vec![MirFieldDef {
                name: "a".into(),
                ty: a,
            }],
        }));
        assert_eq!(compute_layout(&m, a).unwrap_err().code(), "mir.type-cycle");
    }
}
