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
///  2. GATE — the EXACT REPRESENTATION bytes the materialized layout
///     will own are computed BEFORE any allocation: every flattened
///     FieldLayout struct PLUS the real path strings (`parent.child`
///     concatenation — path length grows with depth AND field-name
///     length), plus the shared prefix buffer's doubling slack. A table
///     whose representation exceeds the staging budget is refused with
///     `StagingTooLarge` (F-797-R4-10 R8: the R5 gate counted leaves ×
///     (struct + 24) and let a 64-byte-name fan materialize ~19.4MB of
///     strings past the 8MiB budget);
///  3. FLATTEN — only for tables that passed the gate: ONE vector with
///     exact capacity, names built with exact-capacity Strings against a
///     shared prefix — actual owned bytes equal the predicted gate, no
///     nested intermediate vectors.
pub fn layout_record_owned(
    table: &std::collections::BTreeMap<String, RecordDefOwned>,
    name: &str,
) -> Result<RecordLayout, LayoutError> {
    let mut sizes: std::collections::BTreeMap<String, (u32, u32)> =
        std::collections::BTreeMap::new();
    // F-797-R4-10 (R8): the staging budget is the EXACT representation
    // the MATERIALIZED layout will own — struct bytes + REAL path-string
    // bytes (not a per-leaf constant): leaf counts AND total path bytes
    // AND the longest path are memoized in the size phase.
    let mut leaves: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut path_bytes: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut max_path: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    // F-797-R4-10 (R12): longest FLATTEN path name cost per record —
    // unlike the size-walk chain peak, this is memoized per RECORD, so
    // a SHARED subgraph's full flatten path is counted even though the
    // size walk hits the memo and never walks it again
    let mut fpath: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut chain: Vec<String> = Vec::new();
    // F-797-R4-10 (R10): the memo/index working state is budgeted
    // INCREMENTALLY, charged BEFORE each allocation it covers (record
    // name copies in the four maps; chain/seen name copies while live),
    // so a table that would exceed 8MiB of staging is refused AT the
    // crossing record — never after the whole memo was built — and a
    // tiny table with one long name is charged only what it actually
    // uses (no fictional max-depth/max-name multiplication).
    let mut budget_used: u64 = 0;
    // F-797-R4-10 (R11): peak OUTSTANDING chain charge — the flatten
    // phase re-spends the same shape (its `seen` vector re-copies every
    // name along the deepest path) while ALL FOUR memo maps stay alive,
    // so the final gate must budget that phase explicitly instead of
    // relying on chain credits returned during the size walk.
    let mut chain_acc: (u64, u64) = (0, 0);
    compute_size(
        table,
        name,
        &mut chain,
        0,
        &mut sizes,
        &mut leaves,
        &mut path_bytes,
        &mut max_path,
        &mut fpath,
        &mut budget_used,
        &mut chain_acc,
    )?;
    let (root_size, root_align) = sizes
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    let root_leaves = *leaves
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    let root_paths = *path_bytes
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    let root_max = *max_path
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    let per_leaf = std::mem::size_of::<FieldLayout>() as u64;
    let root_fpath = *fpath
        .get(name)
        .ok_or_else(|| LayoutError::UnknownRecord(name.to_string()))?;
    // representation + the FLATTEN-phase live set: all FIVE memo maps
    // (budget_used after the walk = maps only, chain credited) PLUS the
    // `seen` name copies along the LONGEST FLATTEN PATH — the memoized
    // per-record path cost, NOT the size-walk chain peak (a shared
    // subgraph truncates the walk via memo hits but flatten still
    // unfolds the full path) — Vec-doubling worst case 2x, PLUS the
    // materialized representation and prefix
    let staging_bytes = root_leaves
        .saturating_mul(per_leaf)
        .saturating_add(root_paths)
        // the shared prefix buffer: worst-case String doubling
        .saturating_add(root_max.saturating_mul(2))
        .saturating_add(budget_used)
        .saturating_add(root_fpath.saturating_mul(2));
    if staging_bytes > MAX_STAGING_BYTES {
        return Err(LayoutError::StagingTooLarge {
            record: name.to_string(),
            size: staging_bytes,
        });
    }
    let mut fields = Vec::with_capacity(root_leaves as usize);
    let mut prefix = String::new();
    let mut seen = Vec::new();
    flatten_into(
        table,
        name,
        &sizes,
        0,
        &mut prefix,
        &mut fields,
        &mut seen,
        0,
    )?;
    Ok(RecordLayout {
        record: name.to_string(),
        fields,
        size: *root_size,
        align: *root_align,
    })
}

/// Phase 1: memoized (size, align) with cycle/depth/budget ceilings, plus
/// the representation memos (leaf count, total path bytes, longest path)
/// the staging gate needs. `budget_used` carries the INCREMENTAL staging
/// charge (F-797-R4-10 R10): every name copy the engine is about to
/// allocate is charged before it exists, and the walk refuses at the
/// crossing record.
#[allow(clippy::too_many_arguments)]
fn compute_size(
    table: &std::collections::BTreeMap<String, RecordDefOwned>,
    name: &str,
    chain: &mut Vec<String>,
    depth: usize,
    memo: &mut std::collections::BTreeMap<String, (u32, u32)>,
    leaves: &mut std::collections::BTreeMap<String, u64>,
    path_bytes: &mut std::collections::BTreeMap<String, u64>,
    max_path: &mut std::collections::BTreeMap<String, u64>,
    fpath: &mut std::collections::BTreeMap<String, u64>,
    budget_used: &mut u64,
    chain_acc: &mut (u64, u64),
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
    // charge the chain/seen name copies BEFORE the push allocates them
    // (credited back on pop — they are transient)
    let name_len = name.len() as u64;
    let chain_charge = 2u64.saturating_mul(name_len + 24);
    if budget_used.saturating_add(chain_charge) > MAX_STAGING_BYTES {
        return Err(LayoutError::StagingTooLarge {
            record: name.to_string(),
            size: budget_used.saturating_add(chain_charge),
        });
    }
    *budget_used += chain_charge;
    chain_acc.0 = chain_acc.0.saturating_add(chain_charge);
    chain_acc.1 = chain_acc.1.max(chain_acc.0);
    chain.push(name.to_string());
    let mut offset: u32 = 0;
    let mut align: u32 = 1;
    for f in &def.fields {
        let (size, fa) = match &f.ty {
            FieldTyOwned::Scalar(s) => s.size_align(),
            FieldTyOwned::Record(n) => compute_size(
                table,
                n,
                chain,
                depth + 1,
                memo,
                leaves,
                path_bytes,
                max_path,
                fpath,
                budget_used,
                chain_acc,
            )?,
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
    // representation memos (F-797-R4-10 R8): scalar leaves carry their
    // own name; a record-typed field prefixes EVERY leaf below it with
    // `<field-name>.` — counted exactly, so long field names are first
    // class in the gate.
    let mut leaf_n: u64 = 0;
    let mut path_sum: u64 = 0;
    let mut path_max: u64 = 0;
    for f in &def.fields {
        match &f.ty {
            FieldTyOwned::Scalar(_) => {
                leaf_n += 1;
                let n = f.name.len() as u64;
                path_sum += n;
                path_max = path_max.max(n);
            }
            FieldTyOwned::Record(n) => {
                let sub_leaves = *leaves
                    .get(n)
                    .ok_or_else(|| LayoutError::UnknownRecord(n.clone()))?;
                let sub_paths = *path_bytes
                    .get(n)
                    .ok_or_else(|| LayoutError::UnknownRecord(n.clone()))?;
                let sub_max = *max_path
                    .get(n)
                    .ok_or_else(|| LayoutError::UnknownRecord(n.clone()))?;
                let prefix = f.name.len() as u64 + 1; // '.'
                leaf_n += sub_leaves;
                path_sum += sub_paths + sub_leaves.saturating_mul(prefix);
                path_max = path_max.max(sub_max + prefix);
            }
        }
    }
    // F-797-R4-10 (R12): longest FLATTEN path cost — the `seen` vector
    // during flatten re-copies every RECORD name along the recursion
    // path; for a DAG this is the per-record memoized maximum (own name
    // + deepest child path), so a shared subgraph's full unfold is
    // budgeted even when the size walk memo-hits and skips it
    let own = name_len + 24; // String header + heap name
    let mut best_child: u64 = 0;
    for f in &def.fields {
        if let FieldTyOwned::Record(n) = &f.ty {
            let sub = *fpath
                .get(n)
                .ok_or_else(|| LayoutError::UnknownRecord(n.clone()))?;
            best_child = best_child.max(sub);
        }
    }
    let fpath_v = own.saturating_add(best_child);
    // the five memo maps duplicate the RECORD name — charge BEFORE the
    // inserts allocate them (permanent for this walk)
    let map_charge = 5u64.saturating_mul(name_len + 24 + 96);
    if budget_used.saturating_add(map_charge) > MAX_STAGING_BYTES {
        return Err(LayoutError::StagingTooLarge {
            record: name.to_string(),
            size: budget_used.saturating_add(map_charge),
        });
    }
    *budget_used += map_charge;
    leaves.insert(name.to_string(), leaf_n);
    path_bytes.insert(name.to_string(), path_sum);
    max_path.insert(name.to_string(), path_max);
    fpath.insert(name.to_string(), fpath_v);
    memo.insert(name.to_string(), (size, align));
    // the transient chain copies are gone with the pop — credit back
    chain.pop();
    *budget_used = budget_used.saturating_sub(chain_charge);
    chain_acc.0 = chain_acc.0.saturating_sub(chain_charge);
    Ok((size, align))
}

/// Phase 3: flatten into ONE exact-capacity vector (only reached past
/// the gate). Names build against a shared prefix with exact-capacity
/// Strings — the materialized representation IS the predicted one; no
/// nested intermediate vectors exist.
fn flatten_into(
    table: &std::collections::BTreeMap<String, RecordDefOwned>,
    name: &str,
    sizes: &std::collections::BTreeMap<String, (u32, u32)>,
    base: u32,
    prefix: &mut String,
    out: &mut Vec<FieldLayout>,
    seen: &mut Vec<String>,
    depth: usize,
) -> Result<(), LayoutError> {
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
    let mut offset: u32 = 0;
    for f in &def.fields {
        let (size, fa) = match &f.ty {
            FieldTyOwned::Scalar(s) => s.size_align(),
            FieldTyOwned::Record(n) => *sizes
                .get(n)
                .ok_or_else(|| LayoutError::UnknownRecord(n.clone()))?,
        };
        offset = offset
            .checked_add(fa - 1)
            .and_then(|v| v.checked_sub(v % fa))
            .ok_or_else(|| LayoutError::OffsetOverflow {
                record: name.to_string(),
                field: f.name.clone(),
            })?;
        match &f.ty {
            FieldTyOwned::Scalar(_) => {
                let mut nm = String::with_capacity(prefix.len() + f.name.len());
                nm.push_str(prefix);
                nm.push_str(&f.name);
                out.push(FieldLayout {
                    name: nm,
                    offset: base + offset,
                    size,
                });
            }
            FieldTyOwned::Record(n) => {
                let mark = prefix.len();
                prefix.push_str(&f.name);
                prefix.push('.');
                flatten_into(table, n, sizes, base + offset, prefix, out, seen, depth + 1)?;
                prefix.truncate(mark);
            }
        }
        offset = offset
            .checked_add(size)
            .ok_or_else(|| LayoutError::OffsetOverflow {
                record: name.to_string(),
                field: f.name.clone(),
            })?;
    }
    seen.pop();
    Ok(())
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
