//! Target layout for the candidate profile + the FIRST layout cache
//! (plan §5.1, §5.4).
//!
//! Layout algorithm frozen in contract-decision.md §4 (mirrors the ABI
//! RFC §6 candidates): declaration order, natural alignment, struct
//! alignment = max field alignment, size rounded up to alignment. Bool is
//! a 1-byte 0/1; char is a u32 scalar-value code point; buffers are
//! {i64 ptr, u64 len} descriptor pairs (T-02 exercises them); padding
//! never carries data.
//!
//! The cache contract from day one (§5.4): layout queries are REAL
//! persisted artifacts in the merged 783 managed store under a
//! probe-owned namespace — the 783/787 layer registry is NOT modified.
//! Keys carry the semantic-ref snapshot hash, the record's canonical
//! form, the target and the ABI profile, so an input change is a
//! different key (real invalidation), and hits never skip the caller's
//! own verification. Nothing here caches a PASS verdict.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec};
use ac_artifact_probe::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};
use serde::{Deserialize, Serialize};

/// Candidate-profile tag (plan §5.1). "candidate" is load-bearing: the
/// freeze happens at merge, not here.
pub const ABI_PROFILE: &str = "autoabi.win-x64.v0.1-candidate";
/// First platform (plan §5.1): Windows x64, platform call convention.
pub const TARGET: &str = "x86_64-pc-windows-msvc";

/// Scalar kinds legal in record fields of the T-01/T-02 candidate set.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SemScalar {
    Bool,
    U8,
    I32,
    U32,
    I64,
    U64,
    F64,
    Char,
}

impl SemScalar {
    /// (size, align) in bytes on the frozen profile — Windows x64 natural
    /// alignment. Independent of the C header; the T-02 conformance check
    /// compares against C `sizeof`/`_Alignof`, not against this table.
    pub fn size_align(self) -> (u32, u32) {
        match self {
            SemScalar::Bool | SemScalar::U8 => (1, 1),
            SemScalar::I32 | SemScalar::U32 | SemScalar::Char => (4, 4),
            SemScalar::I64 | SemScalar::U64 | SemScalar::F64 => (8, 8),
        }
    }

    /// Canonical spelling used in keys and payloads.
    pub fn spell(self) -> &'static str {
        match self {
            SemScalar::Bool => "bool",
            SemScalar::U8 => "u8",
            SemScalar::I32 => "i32",
            SemScalar::U32 => "u32",
            SemScalar::I64 => "i64",
            SemScalar::U64 => "u64",
            SemScalar::F64 => "f64",
            SemScalar::Char => "char",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FieldDef {
    pub name: &'static str,
    pub ty: SemScalar,
}

#[derive(Clone, Copy, Debug)]
pub struct RecordDef {
    pub name: &'static str,
    pub fields: &'static [FieldDef],
}

/// The T-01 reference record (mirrors semref snapshot type `RecMix`).
pub const REC_MIX: RecordDef = RecordDef {
    name: "RecMix",
    fields: &[
        FieldDef {
            name: "flag",
            ty: SemScalar::Bool,
        },
        FieldDef {
            name: "count",
            ty: SemScalar::I32,
        },
        FieldDef {
            name: "tag",
            ty: SemScalar::Char,
        },
        FieldDef {
            name: "big",
            ty: SemScalar::I64,
        },
    ],
};

/// Same shape as REC_MIX under a different nominal name (nominal-identity
/// fixture; same-layout different-name records must not alias).
pub const REC_OTHER: RecordDef = RecordDef {
    name: "RecOther",
    fields: &[
        FieldDef {
            name: "flag",
            ty: SemScalar::Bool,
        },
        FieldDef {
            name: "count",
            ty: SemScalar::I32,
        },
        FieldDef {
            name: "tag",
            ty: SemScalar::Char,
        },
        FieldDef {
            name: "big",
            ty: SemScalar::I64,
        },
    ],
};

/// REC_MIX with the first two fields swapped (field-order fixture).
pub const REC_MIX_SWAPPED: RecordDef = RecordDef {
    name: "RecMix",
    fields: &[
        FieldDef {
            name: "count",
            ty: SemScalar::I32,
        },
        FieldDef {
            name: "flag",
            ty: SemScalar::Bool,
        },
        FieldDef {
            name: "tag",
            ty: SemScalar::Char,
        },
        FieldDef {
            name: "big",
            ty: SemScalar::I64,
        },
    ],
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldLayout {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordLayout {
    pub record: String,
    pub fields: Vec<FieldLayout>,
    pub size: u32,
    pub align: u32,
}

/// Owned field type of a descriptor record: a scalar or a named record
/// reference (nested records — declaration order, acyclic).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldTyOwned {
    Scalar(SemScalar),
    Record(String),
}

impl FieldTyOwned {
    pub fn spell(&self) -> String {
        match self {
            FieldTyOwned::Scalar(s) => s.spell().to_string(),
            FieldTyOwned::Record(n) => format!("rec {n}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDefOwned {
    pub name: String,
    pub ty: FieldTyOwned,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordDefOwned {
    pub name: String,
    pub fields: Vec<FieldDefOwned>,
}

impl From<&RecordDef> for RecordDefOwned {
    fn from(def: &RecordDef) -> Self {
        RecordDefOwned {
            name: def.name.to_string(),
            fields: def
                .fields
                .iter()
                .map(|f| FieldDefOwned {
                    name: f.name.to_string(),
                    ty: FieldTyOwned::Scalar(f.ty),
                })
                .collect(),
        }
    }
}

/// Controlled layout failures (V-07): these are rejections and resource
/// states, never panics and never wrapped answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutError {
    UnknownRecord(String),
    RecursiveRecord(Vec<String>),
    NestingTooDeep(usize),
    RecordTooLarge {
        record: String,
        size: u64,
    },
    /// F-797-R4-10: the flattened layout exceeds the staging budget —
    /// refused before materialization (bounded memory, resource state).
    StagingTooLarge {
        record: String,
        size: u64,
    },
    OffsetOverflow {
        record: String,
        field: String,
    },
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::UnknownRecord(n) => write!(f, "unknown record '{n}'"),
            LayoutError::RecursiveRecord(p) => write!(f, "recursive record chain {}", p.join("->")),
            LayoutError::NestingTooDeep(d) => write!(f, "nesting depth {d} exceeds limit"),
            LayoutError::RecordTooLarge { record, size } => {
                write!(f, "record '{record}' size {size} exceeds budget")
            }
            LayoutError::StagingTooLarge { record, size } => {
                write!(
                    f,
                    "record '{record}' flattened size {size} exceeds the staging budget"
                )
            }
            LayoutError::OffsetOverflow { record, field } => {
                write!(f, "offset overflow in '{record}' at field '{field}'")
            }
        }
    }
}

/// Resource ceilings for layout computation (contract-decision.md §6):
/// a record beyond these is a controlled resource state, distinct from
/// any ABI-compatibility rejection.
pub const MAX_RECORD_BYTES: u64 = 64 * 1024 * 1024;
/// Nested-record chains must stay bounded; >=4096 depth works (plan
/// §5.5), deeper is a controlled rejection.
pub const MAX_NESTING_DEPTH: usize = 4096;
/// F-797-R4-10: the layout STAGING budget (contract-decision §6:
/// layout/query staging + index <=8MiB). Materializing a flattened
/// layout whose bytes exceed this is a controlled RESOURCE state — the
/// engine refuses BEFORE allocating the flattened field vector, so a
/// huge-but-legal record never becomes a giant staging allocation.
pub const MAX_STAGING_BYTES: u64 = 8 * 1024 * 1024;

/// Resolved record table (descriptor type set) with full layout.
///
/// F-797-R4-10 two-phase engine:
///  1. SIZE phase — memoized (size, align) computation per record with
///     cycle detection, depth and absolute-size ceilings; no flattened
///     vector is built here, so a wide/deep table costs TABLE space,
///     not leaf space;
///  2. GATE — the ROOT's size must fit the STAGING budget before any
///     materialization (a huge-but-legal record is refused with
///     `StagingTooLarge`, never a giant allocation);
///  3. FLATTEN — only for tables that passed the gate.
pub fn layout_record_owned(
    table: &std::collections::BTreeMap<String, RecordDefOwned>,
    name: &str,
) -> Result<RecordLayout, LayoutError> {
    let mut sizes: std::collections::BTreeMap<String, (u32, u32)> =
        std::collections::BTreeMap::new();
    let mut chain: Vec<String> = Vec::new();
    compute_size(table, name, &mut chain, 0, &mut sizes)?;
    let (root_size, _) = sizes
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    if *root_size as u64 > MAX_STAGING_BYTES {
        return Err(LayoutError::StagingTooLarge {
            record: name.to_string(),
            size: *root_size as u64,
        });
    }
    let mut seen = Vec::new();
    flatten(table, name, &mut seen, 0)
}

/// Phase 1: memoized (size, align) with cycle/depth/budget ceilings.
fn compute_size(
    table: &std::collections::BTreeMap<String, RecordDefOwned>,
    name: &str,
    chain: &mut Vec<String>,
    depth: usize,
    memo: &mut std::collections::BTreeMap<String, (u32, u32)>,
) -> Result<(u32, u32), LayoutError> {
    if let Some(hit) = memo.get(name) {
        return Ok(*hit);
    }
    if depth > MAX_NESTING_DEPTH {
        return Err(LayoutError::NestingTooDeep(depth));
    }
    if chain.iter().any(|s| s == name) {
        let mut path = chain.clone();
        path.push(name.to_string());
        return Err(LayoutError::RecursiveRecord(path));
    }
    let def = table
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    chain.push(name.to_string());
    let mut offset: u32 = 0;
    let mut align: u32 = 1;
    for f in &def.fields {
        let (size, fa) = match &f.ty {
            FieldTyOwned::Scalar(s) => s.size_align(),
            FieldTyOwned::Record(n) => compute_size(table, n, chain, depth + 1, memo)?,
        };
        offset = offset
            .checked_add(fa - 1)
            .and_then(|v| v.checked_sub(v % fa))
            .ok_or_else(|| LayoutError::OffsetOverflow {
                record: name.to_string(),
                field: f.name.clone(),
            })?;
        align = align.max(fa);
        offset = offset
            .checked_add(size)
            .ok_or_else(|| LayoutError::OffsetOverflow {
                record: name.to_string(),
                field: f.name.clone(),
            })?;
        if offset as u64 > MAX_RECORD_BYTES {
            chain.pop();
            return Err(LayoutError::RecordTooLarge {
                record: name.to_string(),
                size: offset as u64,
            });
        }
    }
    chain.pop();
    let size = offset
        .checked_add(align - 1)
        .and_then(|v| v.checked_sub(v % align))
        .ok_or_else(|| LayoutError::OffsetOverflow {
            record: name.to_string(),
            field: "<tail>".into(),
        })?;
    if size as u64 > MAX_RECORD_BYTES {
        return Err(LayoutError::RecordTooLarge {
            record: name.to_string(),
            size: size as u64,
        });
    }
    memo.insert(name.to_string(), (size, align));
    Ok((size, align))
}

/// Phase 3: flatten (only reached past the gate; every visited record
/// fits the staging budget, so the flattened vector is bounded).
fn flatten(
    table: &std::collections::BTreeMap<String, RecordDefOwned>,
    name: &str,
    seen: &mut Vec<String>,
    depth: usize,
) -> Result<RecordLayout, LayoutError> {
    if depth > MAX_NESTING_DEPTH {
        return Err(LayoutError::NestingTooDeep(depth));
    }
    if seen.iter().any(|s| s == name) {
        let mut chain = seen.clone();
        chain.push(name.to_string());
        return Err(LayoutError::RecursiveRecord(chain));
    }
    let def = table
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    seen.push(name.to_string());
    let mut fields = Vec::with_capacity(def.fields.len());
    let mut offset: u32 = 0;
    let mut align: u32 = 1;
    for f in &def.fields {
        let (size, fa, nested) = match &f.ty {
            FieldTyOwned::Scalar(s) => (s.size_align().0, s.size_align().1, None),
            FieldTyOwned::Record(n) => {
                let l = flatten(table, n, seen, depth + 1)?;
                (l.size, l.align, Some(l))
            }
        };
        offset = offset
            .checked_add(fa - 1)
            .and_then(|v| v.checked_sub(v % fa))
            .ok_or_else(|| LayoutError::OffsetOverflow {
                record: name.to_string(),
                field: f.name.clone(),
            })?;
        align = align.max(fa);
        if let Some(nested_layout) = nested {
            for nf in nested_layout.fields {
                fields.push(FieldLayout {
                    name: format!("{}.{}", f.name, nf.name),
                    offset: offset + nf.offset,
                    size: nf.size,
                });
            }
        } else {
            fields.push(FieldLayout {
                name: f.name.clone(),
                offset,
                size,
            });
        }
        offset = offset
            .checked_add(size)
            .ok_or_else(|| LayoutError::OffsetOverflow {
                record: name.to_string(),
                field: f.name.clone(),
            })?;
    }
    seen.pop();
    let size = offset
        .checked_add(align - 1)
        .and_then(|v| v.checked_sub(v % align))
        .ok_or_else(|| LayoutError::OffsetOverflow {
            record: name.to_string(),
            field: "<tail>".into(),
        })?;
    Ok(RecordLayout {
        record: name.to_string(),
        fields,
        size,
        align,
    })
}

/// Canonical textual form of an owned record definition (the key input;
/// nested references spelled `rec <name>`).
pub fn canonical_record_owned(def: &RecordDefOwned) -> String {
    let mut s = format!("rec {} {{", def.name);
    for f in &def.fields {
        s.push_str(&format!(" {} {};", f.ty.spell(), f.name));
    }
    s.push_str(" }");
    s
}

/// Compute the record layout with the frozen algorithm. Offsets are
/// checked arithmetic: an overflowing layout is a bug in the definer, not
/// a wrapped silent answer.
pub fn layout_record(def: &RecordDef) -> Result<RecordLayout, String> {
    let mut fields = Vec::with_capacity(def.fields.len());
    let mut offset: u32 = 0;
    let mut align: u32 = 1;
    for f in def.fields {
        let (size, fa) = f.ty.size_align();
        offset = offset
            .checked_add(fa - 1)
            .and_then(|v| v.checked_sub(v % fa))
            .ok_or_else(|| format!("offset overflow before field {}", f.name))?;
        align = align.max(fa);
        fields.push(FieldLayout {
            name: f.name.to_string(),
            offset,
            size,
        });
        offset = offset
            .checked_add(size)
            .ok_or_else(|| format!("size overflow after field {}", f.name))?;
    }
    let size = offset
        .checked_add(align - 1)
        .and_then(|v| v.checked_sub(v % align))
        .ok_or_else(|| format!("size rounding overflow for {}", def.name))?;
    Ok(RecordLayout {
        record: def.name.to_string(),
        fields,
        size,
        align,
    })
}

/// Canonical textual form of a record definition — the key input. Two
/// definitions with the same canonical form and same semantic-ref
/// snapshot are the same layout query by definition; nominal identity
/// (record name) is part of the form, so same-layout different-name
/// records do NOT alias (plan §5.1: 不同逻辑名义类型即使布局相同不自动互换).
pub fn canonical_record(def: &RecordDef) -> String {
    let mut s = format!("rec {} {{", def.name);
    for f in def.fields {
        s.push_str(&format!(" {} {};", f.ty.spell(), f.name));
    }
    s.push_str(" }");
    s
}

// ---------------------------------------------------------------------------
// first layout cache — 783 managed store, probe-owned namespace
// ---------------------------------------------------------------------------

/// Probe-owned layer spec for persisted layout artifacts. Registered
/// here, not in the 783/787 `LAYERS` registry (plan §1/§5.4: the store's
/// registry and GC stay untouched; the namespace is separate by id).
pub const LAYER_LAYOUT_V01: LayerSpec = LayerSpec {
    id: "ABI-L0",
    name: "autoabi-target-layout",
    schema_id: "auto797/abi-l0-layout/v1",
    producer: "auto-abi-probe layout engine",
    consumers: &["auto-abi-probe emit/build", "conformance checks"],
    key_inputs: &["sem-ref", "record", "target", "abi-profile"],
    ref_kinds: &["provenance"],
    storage: "small canonical layout records (JSON payload)",
    residency: "hot in index; tiny payloads",
    regeneration: ac_artifact_probe::schema::Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::ProbeSeam,
};

/// Query key for a record layout. Inputs: the semantic-ref snapshot hash
/// (semantic identity of the logical types), the record's canonical form
/// (nominal identity + field order + representations), the target and the
/// ABI profile (physical rules). Changing any input changes the key —
/// source-identity alone never reuses a layout.
pub fn layout_key(def: &RecordDef) -> QueryKey {
    QueryKey::new(
        &LAYER_LAYOUT_V01,
        vec![
            ("sem-ref".into(), crate::semref::binding_hash().to_string()),
            ("record".into(), canonical_record(def)),
            ("target".into(), TARGET.into()),
            ("abi-profile".into(), ABI_PROFILE.into()),
        ],
    )
}

/// Persist a computed layout. Returns the artifact id.
pub fn put_layout(
    store: &ManagedStore,
    def: &RecordDef,
    layout: &RecordLayout,
) -> Result<String, StoreError> {
    let payload = serde_json::to_vec(layout).map_err(|e| StoreError::Internal(e.to_string()))?;
    store.put_artifact(PutRequest {
        layer: LAYER_LAYOUT_V01.id,
        key: layout_key(def),
        kind: ChunkKind::LayoutData,
        payload,
        edges: vec![],
        project: None,
    })
}

/// Look up a persisted layout.
pub fn get_layout(
    store: &ManagedStore,
    def: &RecordDef,
) -> Result<Option<RecordLayout>, StoreError> {
    match store.get_artifact(LAYER_LAYOUT_V01.id, &layout_key(def))? {
        None => Ok(None),
        Some(LoadedArtifact { payload, .. }) => {
            let layout: RecordLayout = serde_json::from_slice(&payload)
                .map_err(|e| StoreError::Internal(format!("layout payload decode: {e}")))?;
            Ok(Some(layout))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-derived oracle for REC_MIX (bool@0, i32@4, char@8, i64@16,
    /// size 24, align 8) — derived on paper from the frozen algorithm,
    /// independent of the code under test.
    #[test]
    fn rec_mix_layout_matches_hand_derivation() {
        let l = layout_record(&REC_MIX).expect("layout");
        assert_eq!(l.fields[0].name, "flag");
        assert_eq!(l.fields[0].offset, 0);
        assert_eq!(l.fields[1].name, "count");
        assert_eq!(l.fields[1].offset, 4);
        assert_eq!(l.fields[2].name, "tag");
        assert_eq!(l.fields[2].offset, 8);
        assert_eq!(l.fields[3].name, "big");
        assert_eq!(l.fields[3].offset, 16); // i64 align 8: 12 -> 16
        assert_eq!(l.size, 24); // 24 already 8-aligned
        assert_eq!(l.align, 8);
    }

    #[test]
    fn canonical_record_is_nominal() {
        // Same shape, different nominal name -> different canonical form.
        assert_ne!(canonical_record(&REC_MIX), canonical_record(&REC_OTHER));
        // Field order is part of the form.
        assert_ne!(
            canonical_record(&REC_MIX),
            canonical_record(&REC_MIX_SWAPPED)
        );
    }
}
