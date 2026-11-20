//! MIR verifier — the sole constructor of [`CheckedMir`] (PLAN-798 §5.2).
//!
//! Stage order (frozen): identity/budget → shape/range/quota (ALL blocks,
//! reachable or not) → control flow → definite initialization over the
//! entry-reachable CFG → call/return signature admission → capability
//! admission. Unreachable code is not a hiding place for unknown shapes:
//! every table and instruction is structure-checked; only the init dataflow
//! restricts itself to the reachable subgraph.
//!
//! Initialization uses a conservative must-worklist: block entry state is
//! the INTERSECTION of predecessor exit states (params initialized at
//! entry). Loops converge from the optimistic bound, which keeps legal
//! zero-iteration loops sound — an initialization inside a loop body never
//! escapes to the loop exit — while `def-then-use` inside one iteration is
//! accepted. Legal CFG cycles are dataflow, not errors.

use crate::contract::{
    ResourceKind, SpanInfo, MAX_DECODED_MODULE_BYTES, MAX_TABLE_ENTRIES, MAX_VERIFY_ANALYSIS_BYTES,
    MAX_VERIFY_WALL, MAX_VERIFY_WORK_STEPS, TRACE_CAPABILITY,
};
use crate::mir::*;
use std::time::Instant;

mod init;

/// Verifier rejection taxonomy. `Resource` breaches are budget outcomes;
/// `Unsupported` is out-of-profile content; `Invalid` is every structural,
/// typing, control-flow or initialization violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MirVerifyError {
    Invalid {
        code: String,
        message: String,
        span: Option<SpanInfo>,
    },
    Unsupported {
        code: String,
        message: String,
        span: Option<SpanInfo>,
    },
    Resource {
        kind: ResourceKind,
        message: String,
    },
}

impl MirVerifyError {
    fn inv(code: &str, message: String) -> Self {
        MirVerifyError::Invalid {
            code: code.to_string(),
            message,
            span: None,
        }
    }

    fn uns(code: &str, message: String) -> Self {
        MirVerifyError::Unsupported {
            code: code.to_string(),
            message,
            span: None,
        }
    }

    pub fn code(&self) -> String {
        match self {
            MirVerifyError::Invalid { code, .. } | MirVerifyError::Unsupported { code, .. } => {
                code.clone()
            }
            MirVerifyError::Resource { kind, .. } => kind.code().to_string(),
        }
    }
}

/// Statistics recorded on the credential for observability.
#[derive(Clone, Copy, Debug, Default)]
pub struct VerifyStats {
    pub work_steps: u64,
    pub reachable_blocks: usize,
    pub total_blocks: usize,
    /// Upper bound of the analysis allocations this verification reserved
    /// (init-dataflow bitsets etc.) — folded into the staging partition
    /// ledger by consumers (F798-R3-08).
    pub analysis_bytes: u64,
}

/// Verified MIR. Constructed only here; the inner module is private so a
/// stale credential cannot survive a rewrite.
#[derive(Clone, Debug)]
pub struct CheckedMir {
    inner: MirModule,
    stats: VerifyStats,
}

impl CheckedMir {
    pub fn module(&self) -> &MirModule {
        &self.inner
    }

    pub fn stats(&self) -> &VerifyStats {
        &self.stats
    }

    /// Provenance consumed by the codec (identity only, never trust).
    pub fn profile(&self) -> &str {
        &self.inner.profile
    }
}

/// Verify an unchecked MIR module under the frozen budgets.
pub fn verify_mir(module: &MirModule) -> Result<CheckedMir, MirVerifyError> {
    verify_mir_with_deadline(module, Instant::now() + MAX_VERIFY_WALL)
}

pub fn verify_mir_with_deadline(
    module: &MirModule,
    deadline: Instant,
) -> Result<CheckedMir, MirVerifyError> {
    let stats = validate_mir_with_deadline(module, deadline)?;
    Ok(CheckedMir {
        inner: module.clone(),
        stats,
    })
}

/// The same deep validation for hosts that need a verdict and keep their
/// immutable module, rather than constructing a second owning credential.
/// This returns no executable credential and cannot outlive a module rewrite.
pub fn validate_mir(module: &MirModule) -> Result<VerifyStats, MirVerifyError> {
    validate_mir_with_deadline(module, Instant::now() + MAX_VERIFY_WALL)
}

fn validate_mir_with_deadline(
    module: &MirModule,
    deadline: Instant,
) -> Result<VerifyStats, MirVerifyError> {
    validate_mir_with_analysis_reservation_and_deadline(module, deadline, &mut |_| Ok(()))
}

/// The same verifier with pre-allocation admission of its initialization
/// analysis. This reserves the bases/CFG/dataflow workspace; input ownership,
/// diagnostics, owning credentials and ownership-profile analyses are separate.
pub fn validate_mir_with_analysis_reservation(
    module: &MirModule,
    mut reserve: impl FnMut(usize) -> Result<(), MirVerifyError>,
) -> Result<VerifyStats, MirVerifyError> {
    validate_mir_with_analysis_reservation_and_deadline(
        module,
        Instant::now() + MAX_VERIFY_WALL,
        &mut reserve,
    )
}

fn validate_mir_with_analysis_reservation_and_deadline(
    module: &MirModule,
    deadline: Instant,
    reserve: &mut dyn FnMut(usize) -> Result<(), MirVerifyError>,
) -> Result<VerifyStats, MirVerifyError> {
    let mut v = Verifier {
        module,
        reserve,
        steps: 0,
        deadline,
        analysis_bytes: 0,
        is_data: module.is_data_profile(),
        is_owned: module.is_owned_profile() || module.is_host_console_profile(),
        is_collections: module.is_collections_profile(),
        is_closures: module.is_closures_profile(),
        is_dynamic: module.is_dynamic_profile(),
        is_entry_unit: module.is_entry_unit_profile() || module.is_host_console_profile(),
    };
    v.run()?;
    Ok(v.stats_snapshot())
}

struct Verifier<'a, 'r> {
    module: &'a MirModule,
    reserve: &'r mut dyn FnMut(usize) -> Result<(), MirVerifyError>,
    steps: u64,
    deadline: Instant,
    analysis_bytes: u64,
    /// Whether the module declares the `av.data/1` profile (PLAN-820): the
    /// feature-set gate for types/consts/instructions and the entry
    /// contract. `false` keeps the frozen core-i32 surface byte-for-byte.
    is_data: bool,
    /// Whether the module declares the `av.owned/1` profile (PLAN-821):
    /// subsumes the data feature set and adds the owned surface (types,
    /// str ops, move/borrow/cleanup analysis).
    is_owned: bool,
    /// Whether the module declares the `av.collections/1` profile
    /// (PLAN-822): subsumes the owned surface and adds the List/Variant
    /// surface.
    is_collections: bool,
    /// Whether the module declares the `av.closures/1` profile
    /// (PLAN-823): subsumes the collections surface and adds the closure
    /// surface (owned envs, checked indirect calls, capture prepare/
    /// commit). Drives the closure-site registry below.
    is_closures: bool,
    /// PLAN-823 r8: the closure-body function indices (functions with a
    /// declared env layout), for the entry-body and direct-call gates.
    /// Whether the module declares the `av.dynamic/1` profile (PLAN-824):
    /// subsumes the collections surface and adds the checked-erasure Dyn
    /// surface.
    is_dynamic: bool,
    /// Whether the module declares the `av.entry-unit/1` profile
    /// (PLAN-862): the scalar feature surface plus the Unit RESULT channel
    /// and the entry-kind contract.
    is_entry_unit: bool,
}

impl<'a, 'r> Verifier<'a, 'r> {
    /// The collections feature surface (List/Variant/Place and their
    /// instructions): admitted by `av.collections/1` and its dynamic
    /// successor (PLAN-824 subsumption).
    fn language_values(&self) -> bool {
        self.is_closures
            && self
                .module
                .capabilities
                .iter()
                .any(|c| c == crate::contract::LANGUAGE_VALUES_CAPABILITY)
    }

    fn dynamic_values(&self) -> bool {
        self.is_dynamic || self.language_values()
    }

    fn collections_surface(&self) -> bool {
        self.is_collections || self.is_dynamic || self.is_closures
    }
    /// PLAN-961: the typed numeric surface (narrow/wide/float arithmetic,
    /// bit ops, shifts, conversions) — admitted by the data profile and
    /// every successor (owned/collections/dynamic/closures), exactly the
    /// family that admits the wide scalar TYPES today. The frozen
    /// `av.core-i32/1` profile keeps its i32-only surface.
    fn numeric_surface(&self) -> bool {
        self.is_data || self.is_owned || self.collections_surface()
    }
    fn tick(&mut self, n: u64) -> Result<(), MirVerifyError> {
        self.steps += n;
        if self.steps > MAX_VERIFY_WORK_STEPS {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifySteps,
                message: format!("verify exceeded {} work steps", MAX_VERIFY_WORK_STEPS),
            });
        }
        if self.steps & 0x3ff == 0 && Instant::now() > self.deadline {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifyWall,
                message: "verify exceeded wall budget".to_string(),
            });
        }
        Ok(())
    }

    fn stats_snapshot(&self) -> VerifyStats {
        VerifyStats {
            work_steps: self.steps,
            reachable_blocks: 0,
            total_blocks: self.module.funcs.iter().map(|f| f.blocks.len()).sum(),
            analysis_bytes: self.analysis_bytes,
        }
    }

    fn run(&mut self) -> Result<(), MirVerifyError> {
        crate::host::validate(self.module).map_err(|e| MirVerifyError::inv(e.code, e.message))?;
        if self.module.is_host_console_profile() && self.module.is_unit {
            return Err(MirVerifyError::uns("mir.host-unit", "host-console/1 is a whole-module profile; host function-unit linking is not admitted".into()));
        }

        if self.module.is_host_console_profile() {
            for t in &self.module.types {
                if let MirType::Fn(sig) = t {
                    if sig
                        .params
                        .iter()
                        .any(|p| self.module.types.get(*p as usize) == Some(&MirType::Unit))
                    {
                        return Err(MirVerifyError::inv(
                            "mir.unit-placement",
                            "Unit cannot be a parameter type".into(),
                        ));
                    }
                }
            }
        }

        // ---- identity -------------------------------------------------
        if self.module.profile != crate::contract::AV_PROFILE
            && self.module.profile != crate::contract::AV_DATA_PROFILE
            && self.module.profile != crate::contract::AV_OWNED_PROFILE
            && self.module.profile != crate::contract::AV_COLLECTIONS_PROFILE
            && self.module.profile != crate::contract::AV_CLOSURES_PROFILE
            && self.module.profile != crate::contract::AV_DYNAMIC_PROFILE
            // PLAN-862: the entry-unit profile (scalar surface + Unit result).
            && self.module.profile != crate::contract::AV_HOST_CONSOLE_PROFILE
            && self.module.profile != crate::contract::AV_ENTRY_UNIT_PROFILE
        {
            return Err(MirVerifyError::uns(
                "mir.profile",
                format!(
                    "profile `{}` is not one of the supported profiles (`{}` / `{}` / `{}` / `{}` / `{}` / `{}` / `{}`)",
                    self.module.profile,
                    crate::contract::AV_PROFILE,
                    crate::contract::AV_DATA_PROFILE,
                    crate::contract::AV_OWNED_PROFILE,
                    crate::contract::AV_COLLECTIONS_PROFILE,
                    crate::contract::AV_CLOSURES_PROFILE,
                    crate::contract::AV_DYNAMIC_PROFILE,
                    crate::contract::AV_ENTRY_UNIT_PROFILE
                ),
            ));
        }
        if self.is_data && self.module.owner.schema != crate::contract::MIR_DATA_SCHEMA {
            return Err(MirVerifyError::inv(
                "mir.schema-profile",
                format!(
                    "data-profile module owner schema `{}` is not `{}`",
                    self.module.owner.schema,
                    crate::contract::MIR_DATA_SCHEMA
                ),
            ));
        }
        if self.is_owned
            && !self.module.is_host_console_profile()
            && self.module.owner.schema != crate::contract::MIR_OWNED_SCHEMA
        {
            return Err(MirVerifyError::inv(
                "mir.schema-profile",
                format!(
                    "owned-profile module owner schema `{}` is not `{}`",
                    self.module.owner.schema,
                    crate::contract::MIR_OWNED_SCHEMA
                ),
            ));
        }
        if self.is_collections
            && self.module.owner.schema != crate::contract::MIR_COLLECTIONS_SCHEMA
        {
            return Err(MirVerifyError::inv(
                "mir.schema-profile",
                format!(
                    "collections-profile module owner schema `{}` is not `{}`",
                    self.module.owner.schema,
                    crate::contract::MIR_COLLECTIONS_SCHEMA
                ),
            ));
        }
        if self.is_closures && self.module.owner.schema != crate::contract::MIR_CLOSURES_SCHEMA {
            return Err(MirVerifyError::inv(
                "mir.schema-profile",
                format!(
                    "closures-profile module owner schema `{}` is not `{}`",
                    self.module.owner.schema,
                    crate::contract::MIR_CLOSURES_SCHEMA
                ),
            ));
        }
        if self.is_dynamic && self.module.owner.schema != crate::contract::MIR_DYNAMIC_SCHEMA {
            return Err(MirVerifyError::inv(
                "mir.schema-profile",
                format!(
                    "dynamic-profile module owner schema `{}` is not `{}`",
                    self.module.owner.schema,
                    crate::contract::MIR_DYNAMIC_SCHEMA
                ),
            ));
        }
        if self.is_entry_unit
            && self.module.owner.schema
                != (if self.module.is_host_console_profile() {
                    crate::contract::MIR_HOST_CONSOLE_SCHEMA
                } else {
                    crate::contract::MIR_ENTRY_UNIT_SCHEMA
                })
        {
            return Err(MirVerifyError::inv(
                "mir.schema-profile",
                format!(
                    "entry-unit-profile module owner schema `{}` is not `{}`",
                    self.module.owner.schema,
                    crate::contract::MIR_ENTRY_UNIT_SCHEMA
                ),
            ));
        }
        if self.module.is_unit && self.module.funcs.len() != 1 {
            return Err(MirVerifyError::inv(
                "mir.unit-shape",
                format!(
                    "a unit carries exactly one function, found {}",
                    self.module.funcs.len()
                ),
            ));
        }
        // Every function unit must live in the module's owner scope — a
        // foreign-owner function table entry is an identity violation, not
        // a shape quirk (F798-R3-01).
        for (fi, f) in self.module.funcs.iter().enumerate() {
            if f.owner != self.module.owner {
                return Err(MirVerifyError::inv(
                    "mir.owner",
                    format!(
                        "function #{} `{}` owner {}/{} differs from module owner {}/{}",
                        fi,
                        f.name,
                        f.owner.schema,
                        f.owner.module,
                        self.module.owner.schema,
                        self.module.owner.module
                    ),
                ));
            }
        }

        // ---- budget: table entries BEFORE any deep allocation ----------
        // Scope (contract-decision.md §6 r1.1): the 65_536 cap is a PER-TABLE
        // allocation guard — module tables (types/consts/imports/functions)
        // and EACH function's slot/block/instruction table independently.
        // Whole-module totals are governed by the request-level work-step
        // and wall budgets (and the 4 MiB input cap), which is what makes
        // the frozen 100-function reference load representable without
        // weakening any single-table guard.
        let module_tables = self.module.types.len()
            + self.module.consts.len()
            + self.module.imports.len()
            + self.module.funcs.len();
        if module_tables > MAX_TABLE_ENTRIES {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::TableEntries,
                message: format!(
                    "module table entries {} exceed the frozen budget {}",
                    module_tables, MAX_TABLE_ENTRIES
                ),
            });
        }
        for f in &self.module.funcs {
            let fn_entries = f.slots.len() + f.blocks.len() + f.flat_instr_count() as usize;
            if fn_entries > MAX_TABLE_ENTRIES {
                return Err(MirVerifyError::Resource {
                    kind: ResourceKind::TableEntries,
                    message: format!(
                        "function `{}` table entries {} exceed the frozen budget {}",
                        f.name, fn_entries, MAX_TABLE_ENTRIES
                    ),
                });
            }
        }
        // Structural heap estimate BEFORE deep verification: wire length is
        // not a working-set proxy (F798-R3-08).
        let heap_est = self.module.heap_estimate();
        if heap_est > MAX_DECODED_MODULE_BYTES {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::DecodeMemory,
                message: format!(
                    "decoded module heap estimate {} bytes exceeds the frozen budget {}",
                    heap_est, MAX_DECODED_MODULE_BYTES
                ),
            });
        }

        // ---- shape: type table -----------------------------------------
        for (ti, t) in self.module.types.iter().enumerate() {
            if let MirType::Fn(sig) = t {
                for p in sig.params.iter().chain(std::iter::once(&sig.result)) {
                    if *p as usize >= self.module.types.len() {
                        return Err(MirVerifyError::inv(
                            "mir.type-ref",
                            format!("type #{ti} references out-of-range type {}", p),
                        ));
                    }
                }
                if sig.params.len() > u8::MAX as usize {
                    return Err(MirVerifyError::inv(
                        "mir.sig-arity",
                        format!("type #{ti} has more than {} params", u8::MAX),
                    ));
                }
            }
            // Profile feature gate: the wide scalars and records exist ONLY
            // in the data/owned profiles (owned subsumes the data surface;
            // collections subsumes owned); char/Str/StrView/MutRef exist
            // ONLY in the owned-or-collections profiles (PLAN-821/822
            // contract-decision §2).
            match t {
                MirType::I64 | MirType::U64 | MirType::F32 | MirType::F64 | MirType::Record(_)
                    if !self.is_data && !self.is_owned && !self.collections_surface() =>
                {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} ({:?} …) exists only in the `{}`/`{}` profiles",
                            t,
                            crate::contract::AV_DATA_PROFILE,
                            crate::contract::AV_OWNED_PROFILE
                        ),
                    ));
                }
                // PLAN-961: the narrow integers ride the same data-family
                // gate as the wide scalars.
                MirType::I8 | MirType::U8 | MirType::U32 if !self.numeric_surface() => {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} ({:?}) exists only in the data-family profiles",
                            t
                        ),
                    ));
                }
                MirType::Char
                | MirType::Str
                | MirType::StrView
                | MirType::MutRef
                | MirType::RecordView(_)
                    if !self.is_owned && !self.collections_surface() =>
                {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} ({:?}) exists only in the `{}`/`{}` profiles",
                            t,
                            crate::contract::AV_OWNED_PROFILE,
                            crate::contract::AV_COLLECTIONS_PROFILE
                        ),
                    ));
                }
                // PLAN-822: the List/Variant/Place surface is
                // collections-only (and its dynamic successor).
                MirType::List(_) | MirType::Variant(_) | MirType::Place(_)
                    if !self.collections_surface() =>
                {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} ({:?} …) exists only in the `{}`+ profiles",
                            t,
                            crate::contract::AV_COLLECTIONS_PROFILE
                        ),
                    ));
                }
                // PLAN-823: the Closure surface is closures-profile-only.
                MirType::Closure(_) if !self.is_closures => {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} (Closure …) exists only in the `{}` profile",
                            crate::contract::AV_CLOSURES_PROFILE
                        ),
                    ));
                }
                // PLAN-862: the Unit RESULT type is entry-unit-only.
                // PLAN-964: a unit-RESULT closure (block lambda whose body
                // is an effect statement) is the closures profile's Unit.
                MirType::Unit if !self.is_entry_unit && !self.is_closures => {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{} (Unit) exists only in the `{}` profile",
                            ti,
                            crate::contract::AV_ENTRY_UNIT_PROFILE
                        ),
                    ));
                }
                // PLAN-824: the erased Dyn type is dynamic-profile-only.
                MirType::Dyn if !self.dynamic_values() => {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} (Dyn) exists only in the `{}` profile",
                            crate::contract::AV_DYNAMIC_PROFILE
                        ),
                    ));
                }
                _ => {}
            }
            // PLAN-822 admissible-closure checks (contract-decision §2):
            // list elements and variant payloads share the frozen element
            // domain (borrowed views / Fn / nested lists rejected); records
            // never carry containers in their fields.
            if let MirType::List(l) = t {
                if l.elem as usize >= self.module.types.len() {
                    return Err(MirVerifyError::inv(
                        "mir.type-ref",
                        format!("list type #{ti} references out-of-range element {}", l.elem),
                    ));
                }
                self.check_element_domain(l.elem, ti, "mir.type.list-elem")?;
            }
            // F822-R4-01: the Place referent is a module TypeIdx exactly
            // like a List element — an unused table entry is still
            // untrusted input. Out-of-range AND out-of-domain referents
            // (including self/cyclic and container referents) are
            // rejected HERE, before any checked credential exists; the
            // snapshot merger indexes the referent unguarded.
            if let MirType::Place(l) = t {
                if l.elem as usize >= self.module.types.len() {
                    return Err(MirVerifyError::inv(
                        "mir.type-ref",
                        format!(
                            "place type #{ti} references out-of-range element {}",
                            l.elem
                        ),
                    ));
                }
                self.check_element_domain(l.elem, ti, "mir.type.place-elem")?;
            }
            if let MirType::Variant(v) = t {
                if v.arms.is_empty() {
                    return Err(MirVerifyError::inv(
                        "mir.variant-empty",
                        format!("variant type #{ti} `{}` has no arms", v.name),
                    ));
                }
                if v.arms.len() > crate::contract::MAX_RECORD_FIELDS {
                    return Err(MirVerifyError::Resource {
                        kind: ResourceKind::TableEntries,
                        message: format!(
                            "variant type #{ti} `{}` has {} arms, above the frozen per-type guard {}",
                            v.name,
                            v.arms.len(),
                            crate::contract::MAX_RECORD_FIELDS
                        ),
                    });
                }
                if v.owner_module.is_empty() || v.name.is_empty() {
                    return Err(MirVerifyError::inv(
                        "mir.variant-identity",
                        format!("variant type #{ti} has an empty owner module or name"),
                    ));
                }
                let mut seen = std::collections::BTreeSet::new();
                for a in &v.arms {
                    if !seen.insert(a.name.as_str()) {
                        return Err(MirVerifyError::inv(
                            "mir.variant-arm-dup",
                            format!(
                                "variant type #{ti} `{}` declares arm `{}` twice",
                                v.name, a.name
                            ),
                        ));
                    }
                    if let Some(pt) = a.payload {
                        if pt as usize >= self.module.types.len() {
                            return Err(MirVerifyError::inv(
                                "mir.type-ref",
                                format!(
                                    "variant type #{ti} `{}` arm `{}` references out-of-range payload type {}",
                                    v.name, a.name, pt
                                ),
                            ));
                        }
                        self.check_element_domain(pt, ti, "mir.type.variant-payload")?;
                    }
                }
            }
            // PLAN-822: records stay container-free (a List/Variant field is
            // `mir.type.nested-container`, first-batch rejection).
            // PLAN-962: the aggregates admission capability lifts this —
            // composite record fields are legal on that surface only.
            if let MirType::Record(r) = t {
                for f in &r.fields {
                    if matches!(
                        self.module.types.get(f.ty as usize),
                        Some(MirType::List(_)) | Some(MirType::Variant(_))
                    ) && !self.aggregates_admission()
                    {
                        return Err(MirVerifyError::uns(
                            "mir.type.nested-container",
                            format!(
                                "record type #{ti} `{}` field `{}` is a container type (not a record field kind in this profile)",
                                r.name, f.name
                            ),
                        ));
                    }
                }
            }
            if let MirType::Record(r) = t {
                if r.fields.is_empty() {
                    return Err(MirVerifyError::inv(
                        "mir.record-empty",
                        format!(
                            "record type #{ti} `{}` has no fields (empty records are not a supported capability)",
                            r.name
                        ),
                    ));
                }
                if r.fields.len() > crate::contract::MAX_RECORD_FIELDS {
                    return Err(MirVerifyError::Resource {
                        kind: ResourceKind::TableEntries,
                        message: format!(
                            "record type #{ti} `{}` has {} fields, above the frozen per-record guard {}",
                            r.name,
                            r.fields.len(),
                            crate::contract::MAX_RECORD_FIELDS
                        ),
                    });
                }
                if r.owner_module.is_empty() || r.name.is_empty() {
                    return Err(MirVerifyError::inv(
                        "mir.record-identity",
                        format!("record type #{ti} has an empty owner module or name"),
                    ));
                }
                let mut seen = std::collections::BTreeSet::new();
                for f in &r.fields {
                    if !seen.insert(f.name.as_str()) {
                        return Err(MirVerifyError::inv(
                            "mir.record-field-dup",
                            format!(
                                "record type #{ti} `{}` declares field `{}` twice",
                                r.name, f.name
                            ),
                        ));
                    }
                    match self.module.types.get(f.ty as usize) {
                        Some(MirType::Fn(_)) => {
                            return Err(MirVerifyError::inv(
                                "mir.record-field-type",
                                format!(
                                    "record type #{ti} `{}` field `{}` is a function type",
                                    r.name, f.name
                                ),
                            ));
                        }
                        // 782 mirror: a borrowed view / token is not a
                        // field kind — owned Str fields are; views are
                        // place values only (contract-decision §2).
                        Some(MirType::StrView)
                        | Some(MirType::MutRef)
                        | Some(MirType::RecordView(_))
                        | Some(MirType::Place(_)) => {
                            return Err(MirVerifyError::inv(
                                "mir.field-borrow",
                                format!(
                                    "record type #{ti} `{}` field `{}` is a borrowed view/token kind",
                                    r.name, f.name
                                ),
                            ));
                        }
                        Some(_) => {}
                        None => {
                            return Err(MirVerifyError::inv(
                                "mir.type-ref",
                                format!(
                                    "record type #{ti} `{}` field `{}` references out-of-range type {}",
                                    r.name, f.name, f.ty
                                ),
                            ));
                        }
                    }
                }
            }
        }
        // Record type graph must be ACYCLIC (by-value records; iterative
        // color DFS, work-step charged — contract-decision §2).
        self.check_record_acyclic()?;
        // Every record type's logical layout must DERIVE without overflow
        // (checked arithmetic; the wire never carries a descriptor). Layout
        // rejections are `mir.layout-*` (contract-decision §6). The owned
        // profile derives layouts too — its records carry Str fields
        // (12/8 logical placement) and a DERIVED is_copy (PLAN-821).
        if self.is_data || self.is_owned {
            // F820-R3-01: ONE memo/color pair and the VERIFIER'S OWN step
            // counter serve every record — layout work shares the frozen
            // verification budget (no per-record reset, no quadratic
            // ancestor re-walk), and an over-ceiling derivation is a
            // RESOURCE rejection (never a silent accept, never mislabeled
            // invalid).
            let n = self.module.types.len();
            let mut layout_memo: Vec<Option<crate::layout::LayoutDesc>> = vec![None; n];
            let mut layout_color = vec![0u8; n];
            for ti in 0..n {
                if matches!(self.module.types[ti], MirType::Record(_)) {
                    self.tick(1)?;
                    let derived = crate::layout::compute_layout_shared(
                        self.module,
                        ti as TypeIdx,
                        &mut self.steps,
                        &mut layout_memo,
                        &mut layout_color,
                    );
                    if let Err(e) = derived {
                        return Err(match e {
                            crate::layout::LayoutError::Budget => MirVerifyError::Resource {
                                kind: ResourceKind::VerifySteps,
                                message: format!(
                                    "layout derivation of type {ti} exceeded the shared {} work-step budget",
                                    MAX_VERIFY_WORK_STEPS
                                ),
                            },
                            other => MirVerifyError::inv(other.code(), format!("{:?}", other)),
                        });
                    }
                }
            }
        }
        // ---- shape: imports --------------------------------------------
        // v1 support surface: ONLY the two approved test-trace intrinsics
        // plus — in single-function UNITS — the internal `fn:<name>`
        // linkage imports the closure assembler resolves into direct calls
        // (deferred resolution for recursive/mutual units). Anything else
        // is an UNSUPPORTED rejection at admission — never a runtime
        // "unresolved import" infra outcome (F798-R3-01).
        for (ii, imp) in self.module.imports.iter().enumerate() {
            let known = match imp.symbol.strip_prefix("fn:") {
                Some(name) => self.module.is_unit && !name.is_empty(),
                // PLAN-968: scheduler intrinsics ride the import seam as
                // def-side identity only — their call sites rewrite to the
                // SCHED operations at lowering; runtime never dispatches
                // them as imports.
                None => {
                    matches!(imp.symbol.as_str(), "hir.test.mark_a" | "hir.test.mark_b")
                        || imp.symbol.starts_with("__sched_")
                        // PLAN-960/981：模块全局访问子与 AGE typed 端口构造器
                        // 走 import 缝只是 **def 侧身份**——调用点在 lowering 已
                        // 改写为 GlobalGet/GlobalSet 与端口 record 的 Aggregate，
                        // 运行时从不按 import 派发（与 `__sched_` 同一理由）。
                        // 符号注册表在 ace-core `source::age_resources`（本 crate
                        // 不依赖它，字面量与注册表冻结同步）。
                        || imp.symbol.starts_with("module.global.")
                        || matches!(
                            imp.symbol.as_str(),
                            "age.material2d.color.v1"
                                | "age.mesh2d.quad.v1"
                                | "age.mesh2d.circle.v1"
                                // PLAN-981（IF-981-GFX-AV-SEAM）：纯值构造器
                                //（其余 gfx 符号在调用点车道具名拒，不会成为
                                // 存活 import）。
                                | "age.frame2d.at.v1"
                        )
                        // PLAN-981（IF-981-GFX-HOST-PORT）：AGE 图形服务的宿主
                        // 派发 op 以**存活 import 调用**到达 AV——调用点在 callee
                        // 解析前按 symbol 分流到注册 provider（未注册即具名拒），
                        // 所以这里必须承认它们是受检的 import 身份。
                        || crate::age_gfx::is_host_symbol(&imp.symbol)
                }
            };
            if !known {
                return Err(MirVerifyError::uns(
                    "mir.import-unknown",
                    format!(
                        "import #{} `{}` is outside the supported v1 surface (test trace intrinsics; unit-internal fn: linkage)",
                        ii, imp.symbol
                    ),
                ));
            }
            let Some(sig) = self.module.sig_of(imp.sig) else {
                return Err(MirVerifyError::inv(
                    "mir.import-sig",
                    format!(
                        "import #{} `{}` does not reference a function type",
                        ii, imp.symbol
                    ),
                ));
            };
            self.check_sig_indices(sig)?;
        }
        // ---- capability admission (allowlist) ----------------------------
        // `trace` is the ONLY known capability; anything else — minted by a
        // future producer or hand-crafted — is unsupported at admission,
        // not a runtime grant (F798-R3-01).
        for c in &self.module.capabilities {
            let known = c == TRACE_CAPABILITY
                // PLAN-962: the aggregates admission capability (keys the
                // widened composite element domain in this verifier).
                || c == "av.aggregates/1"
                || (self.is_closures && c == crate::contract::LANGUAGE_VALUES_CAPABILITY)
                || (self.is_closures && c == crate::contract::GLOBAL_TAKE_CAPABILITY)
                || c == ace_foreign_contract::manual::SOURCE_CAP
                || self.module.funcs.iter().any(|f| f.blocks.iter().any(|b| b.instrs.iter().any(|i|
                    matches!(i, MirInstr::ForeignCall { declaration, .. } if declaration.capabilities.contains(c)))))
                || (self.is_owned && c == crate::contract::TEXT_RUNTIME_CAPABILITY)
                // PLAN-939 S06: a foreign source module's units carry the
                // module-level foreign caps even when THIS unit's function
                // has no foreign call of its own (e.g. a callback target
                // fn); the grant was admitted with the environment.
                || (self.is_owned && c == ace_foreign_contract::resource::RESOURCE_CAP)
                // PLAN-981 B-W1（IF-981-PROOF-AV）：AGE 资源服务能力（2D/3D）
                // ——AGE 包声明资源端口（g01 的 `resources.*`）即携带；观察
                // 车道把端口构造 lower 成 record Aggregate（无 IO/宿主授予）。
                || (self.is_owned && matches!(c.as_str(), "age2d.resource" | "age3d.resource"))
                // PLAN-968: scheduler-surface modules admit the console
                // capabilities (task-handler stdout observation).
                || (self.module.host_imports.iter().any(|h| h.symbol.starts_with("auto.console."))
                    && matches!(
                        c.as_str(),
                        crate::host::STDOUT | crate::host::STDERR
                    ))
                // PLAN-982（B-CONFIG-I）：配置文档原语的能力键是已知准入项
                // （声明必须真的带同类别 host import；运行期会话另核）。
                || (c == crate::config_document::CAPABILITY
                    && self
                        .module
                        .host_imports
                        .iter()
                        .any(|h| crate::config_document::is_config_symbol(&h.symbol)))
                // PLAN-981（IF-981-GFX-HOST-PORT）：AGE 图形能力键在模块真的
                // 携带图形宿主 import 时是已知准入项（provider 注册/预算仍由
                // `ace_av::age_gfx` 会话另核）。
                || (crate::age_gfx::is_gfx_capability(c)
                    && self
                        .module
                        .imports
                        .iter()
                        .any(|i| crate::age_gfx::is_host_symbol(&i.symbol)))
                || (self.module.is_host_console_profile()
                    && matches!(
                        c.as_str(),
                        crate::host::STDOUT
                            | crate::host::STDERR
                            | crate::platform_io::CAP_STDOUT
                            | crate::platform_io::CAP_STDERR
                            | crate::platform_io::CAP_WRITE
                            | crate::platform_io::CAP_MEM
                            | crate::platform_io::CAP_PROBE
                    ))
                // Scheduler units share the closures family and may carry
                // these capabilities transitively. Individual calls still
                // require the precise declaration and capability.
                || ((self.module.is_host_console_profile()
                    || self.module.profile == crate::contract::AV_CLOSURES_PROFILE)
                    && matches!(
                        c.as_str(),
                        crate::platform_io::CAP_NET_CONNECT
                            | crate::platform_io::CAP_NET_WRITE
                            | crate::platform_io::CAP_NET_READ
                            | crate::platform_io::CAP_NET_CLOSE
                            | crate::platform_io::CAP_NET_LISTEN
                    ))
            // PLAN-973: the system-resource capability spellings are
            // KNOWN admissions (units may carry them transitively from
            // package binding without owning a platform import) — the
            // run-time grant ledger stays the authority.
            || {
                matches!(
                    c.as_str(),
                    crate::platform_io::CAP_STDIN
                        | crate::platform_io::CAP_FILE
                        | crate::platform_io::CAP_PATH
                        | crate::platform_io::CAP_ENV
                        | crate::platform_io::CAP_CLOCK
                        | crate::platform_io::CAP_ENTROPY
                        | crate::platform_io::CAP_PROC
                        // PLAN-982 (Q-982-2 (a) step 1): the narrow argv-read
                        // grant (the per-run argv op; the cmdline-read imports
                        // accept it at the run-time grant ledger).
                        | crate::platform_io::CAP_PROC_ARGV
                        // PLAN-982 (Q-982-2 step 2): the sys script lane's
                        // console print/eprint/say imports (stream grants
                        // checked at the run-time ledger).
                        | crate::host::STDOUT
                        | crate::host::STDERR
                        // PLAN-982（aggregates 文本入口裁定，2026-11-19）：
                        // 受检文本入口能力（车道门在入口带借用 `str` 形参时
                        // 写入受检产物；运行端从入口所属单元核对能力与
                        // `StrView` 签名——`execute_program_text_platform`）。
                        | crate::contract::TEXT_RUNTIME_CAPABILITY
                        // sys-library units stage through io/mem primitives
                        | crate::platform_io::CAP_WRITE
                        | crate::platform_io::CAP_MEM
                )
            };
            if !known {
                return Err(MirVerifyError::uns(
                    "mir.capability-unknown",
                    format!(
                        "capability `{}` is not in the supported set (`{}`{})",
                        c,
                        TRACE_CAPABILITY,
                        if self.is_owned {
                            format!(" / `{}`", crate::contract::TEXT_RUNTIME_CAPABILITY)
                        } else {
                            String::new()
                        }
                    ),
                ));
            }
        }
        // Trace USE still requires the declared capability.
        // ---- capability admission for trace intrinsics ------------------
        let uses_trace = self
            .module
            .imports
            .iter()
            .any(|i| i.symbol == "hir.test.mark_a" || i.symbol == "hir.test.mark_b")
            || self.module.funcs.iter().any(|f| {
                f.blocks
                    .iter()
                    .any(|b| b.instrs.iter().any(|i| matches!(i, MirInstr::Trace { .. })))
            });
        if uses_trace
            && !self
                .module
                .capabilities
                .iter()
                .any(|c| c == TRACE_CAPABILITY)
        {
            return Err(MirVerifyError::uns(
                "mir.capability",
                format!(
                    "module uses test trace intrinsics but does not require `{}`",
                    TRACE_CAPABILITY
                ),
            ));
        }

        // ---- entry ------------------------------------------------------
        if self.module.entry as usize >= self.module.funcs.len() {
            return Err(MirVerifyError::inv(
                "mir.entry",
                format!("entry function index {} out of range", self.module.entry),
            ));
        }
        let entry_sig_idx = self.module.funcs[self.module.entry as usize].sig;
        let Some(entry_sig) = self.module.sig_of(entry_sig_idx) else {
            return Err(MirVerifyError::inv(
                "mir.entry-sig",
                "entry function does not reference a function type".to_string(),
            ));
        };
        // The frozen entry contract: MODULE closures — core-i32 requires
        // zero-param i32; the data profile admits all six scalars for params
        // and result but NO record at the entry boundary (contract-decision
        // §6). Single-function units verify their one function as-is —
        // closure assembly selects and re-checks the real module entry.
        if !self.module.is_unit {
            if self.is_owned && !self.module.is_host_console_profile() {
                // PLAN-821 entry contract (contract-decision §6): params
                // are the six scalars plus Char; the result may ADD owned
                // Str and records (the host receives the owned result with
                // the single release responsibility). Views/tokens never
                // cross the entry boundary.
                let ok = |t: &FnSig| -> bool {
                    t.params.iter().all(|p| self.entry_param_ok(*p))
                        && self.entry_result_ok(t.result)
                };
                if !ok(entry_sig) {
                    return Err(MirVerifyError::inv(
                        "mir.entry-sig",
                        "owned-profile module entry params must be scalar/char; result may additionally be owned Str/record (views never cross the entry)"
                            .to_string(),
                    ));
                }
            } else if self.is_data {
                let scalar_ok = |t: &FnSig| -> bool {
                    t.params
                        .iter()
                        .all(|p| self.module.type_shape(*p).is_some())
                        && self.module.type_shape(t.result).is_some()
                };
                if !scalar_ok(entry_sig) {
                    return Err(MirVerifyError::inv(
                        "mir.entry-sig",
                        "data-profile module entry params/result must be scalar (records do not cross the entry boundary)"
                            .to_string(),
                    ));
                }
            } else if self.is_dynamic {
                // PLAN-824 entry contract: params as owned; the result may
                // ADDITIONALLY be a Dyn — the host receives the checked
                // container with single release responsibility (containers
                // cross the boundary via Dyn, not bare).
                let ok = |t: &FnSig| -> bool {
                    t.params.iter().all(|p| self.entry_param_ok(*p))
                        && (self.entry_result_ok(t.result)
                            || matches!(
                                self.module.types.get(t.result as usize),
                                Some(MirType::Dyn)
                            ))
                };
                if !ok(entry_sig) {
                    return Err(MirVerifyError::inv(
                        "mir.entry-sig",
                        "dynamic-profile module entry params must be scalar/char; result may additionally be owned Str/record/Dyn"
                            .to_string(),
                    ));
                }
            } else if self.is_entry_unit {
                // PLAN-862 entry contract (entry-contract.md §5): the entry
                // is zero-param and its result follows the declared KIND:
                // `Int` <=> i32, `Unit` <=> the Unit type. A kind-less
                // module, a kind/signature mismatch in EITHER direction and
                // a non-zero-param entry are all controlled rejections —
                // the signature is never used to GUESS the kind.
                //
                // PLAN-982（983 组合入口，用户裁定）：唯一例外——**host-console
                // 文本入口**：模块声明 `ac.runtime.text.v1` 且形参**全部为
                // `StrView`**（借用 `str`）时放行（`execute_entry_text_platform`
                // 的文本实参通道）；entry-unit 的冻结合同不变。
                if !entry_sig.params.is_empty() {
                    let text_entry = self.module.is_host_console_profile()
                        && self.module.has_text_entry_capability()
                        && entry_sig.params.iter().all(|p| {
                            matches!(
                                self.module.types.get(*p as usize),
                                Some(crate::mir::MirType::StrView)
                            )
                        });
                    if !text_entry {
                        return Err(MirVerifyError::inv(
                            "mir.entry-sig",
                            "the entry-unit profile's entry takes no parameters".to_string(),
                        ));
                    }
                }
                let result_is_i32 =
                    self.module.type_shape(entry_sig.result) == Some(ScalarShape::I32);
                let result_is_unit = matches!(
                    self.module.types.get(entry_sig.result as usize),
                    Some(MirType::Unit)
                );
                match self.module.entry_kind {
                    Some(crate::mir::EntryKind::Int) if result_is_i32 => {}
                    Some(crate::mir::EntryKind::Unit) if result_is_unit => {}
                    Some(crate::mir::EntryKind::Int) => {
                        return Err(MirVerifyError::inv(
                            "mir.entry-kind",
                            "entry kind `int` requires an i32 entry result".to_string(),
                        ));
                    }
                    Some(crate::mir::EntryKind::Unit) => {
                        return Err(MirVerifyError::inv(
                            "mir.entry-kind",
                            "entry kind `unit` requires a Unit entry result".to_string(),
                        ));
                    }
                    None => {
                        return Err(MirVerifyError::inv(
                            "mir.entry-kind",
                            "the entry-unit profile requires a declared entry kind (kind-less modules are not admissible)".to_string(),
                        ));
                    }
                }
            } else if self.collections_surface() {
                // PLAN-961: the collections/closures family (the numeric
                // source surface rides it) admits every SCALAR as entry
                // params/result — the frozen closure-profile i32-entry
                // contract is superseded for this family by the numeric
                // entry matrix (spec delta SD-01); records still never
                // cross the entry boundary.
                let scalar_ok = entry_sig
                    .params
                    .iter()
                    .all(|p| self.module.type_shape(*p).is_some())
                    && (self.module.type_shape(entry_sig.result).is_some()
                        // PLAN-1080 (D-04)：普通 Auto 语言的 void main——
                        // 零参 Unit 结果入口放行（ABC 布局不为 closures 家
                        // 族携带 entry kind，按签名形状收纳；冻结窄面的
                        // **源路径**产不出该形状——注解必选，故仅 MIR 准入
                        // 面放宽，可审计）。
                        || (entry_sig.params.is_empty() && self.language_values()
                            && matches!(self.module.types.get(entry_sig.result as usize), Some(MirType::Dyn)))
                        || (entry_sig.params.is_empty()
                            && matches!(
                                self.module.types.get(entry_sig.result as usize),
                                Some(MirType::Unit)
                            )));
                if !scalar_ok {
                    return Err(MirVerifyError::inv(
                        "mir.entry-sig",
                        "collections/closures-family module entry params/result must be scalar"
                            .to_string(),
                    ));
                }
            } else if !entry_sig.params.is_empty()
                || self.module.type_shape(entry_sig.result) != Some(ScalarShape::I32)
            {
                return Err(MirVerifyError::inv(
                    "mir.entry-sig",
                    "module entry must be zero-param i32 (frozen entry contract)".to_string(),
                ));
            }
        }
        let _ = entry_sig;

        // ---- PLAN-823 closure-site pre-pass ------------------------------
        // The UNIQUE CREATE_CLOSURE site per callee defines the env layout
        // (capture-order source slot TYPES) the body's ENV_GET
        // instructions type against (first-case rule, contract-decision
        // r2 §5; T-03 generalizes the env-layout binding).
        if self.is_closures {
            // r8 env contract: every CREATE_CLOSURE site's captures must
            // equal the callee's DECLARED env layout position-by-position
            // (a BORROW capture's declared entry is the StrView type).
            // Whole modules keep the unique-site rule; units address
            // closure bodies through imports carrying the same
            // declaration (link-time equivalence pins the binding).
            // The StrView type is only REQUIRED when a Borrow capture is
            // present (a unit whose reached-type closure has no view type
            // must not fail on an unused lookup).
            let has_borrow = self.module.funcs.iter().any(|f| {
                f.blocks.iter().any(|b| {
                    b.instrs.iter().any(|i| {
                        matches!(i, MirInstr::CreateClosure { captures, .. } if captures
                            .iter()
                            .any(|c| c.kind == crate::mir::CaptureKind::Borrow))
                    })
                })
            });
            let strview_ty = if has_borrow {
                Some(self.strview_type_idx()?)
            } else {
                None
            };
            // PLAN-1081: the env-need test reads each callee's declared
            // env directly; a precomputed body set is no longer kept.
            let mut sites: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
            for f in self.module.funcs.iter() {
                for b in f.blocks.iter() {
                    for instr in b.instrs.iter() {
                        if let MirInstr::CreateClosure {
                            callee, captures, ..
                        } = instr
                        {
                            let declared: Option<&Vec<u32>> = match callee {
                                crate::mir::CalleeRef::Func(idx) => {
                                    if self.module.is_unit {
                                        return Err(MirVerifyError::inv(
                                            "mir.unit-callee",
                                            "a unit must address closure bodies through imports"
                                                .to_string(),
                                        ));
                                    }
                                    // PLAN-1081: the single-site (first-case)
                                    // rule protects per-site ENV ownership;
                                    // an EMPTY-env callee (a named fn item
                                    // referenced as a value) has no per-site
                                    // state, so multiple value references of
                                    // the same item are legal.
                                    let empty_env_callee =
                                        self.module.funcs.get(*idx as usize).is_some_and(|t| {
                                            t.env.as_ref().is_some_and(|e| e.is_empty())
                                        });
                                    if !empty_env_callee && !sites.insert(*idx) {
                                        return Err(MirVerifyError::inv(
                                            "mir.closure.site-duplicate",
                                            format!(
                                                "function #{} has more than one CREATE_CLOSURE site (first-case rule)",
                                                idx
                                            ),
                                        ));
                                    }
                                    self.module
                                        .funcs
                                        .get(*idx as usize)
                                        .and_then(|t| t.env.as_ref())
                                }
                                crate::mir::CalleeRef::Import(idx) => {
                                    if !self.module.is_unit {
                                        return Err(MirVerifyError::inv(
                                            "mir.closure.callee-kind",
                                            "a whole-module CREATE_CLOSURE must target a function, not an import"
                                                .to_string(),
                                        ));
                                    }
                                    self.module
                                        .imports
                                        .get(*idx as usize)
                                        .and_then(|i| i.env.as_ref())
                                }
                            };
                            let Some(declared) = declared else {
                                return Err(MirVerifyError::inv(
                                    "mir.closure.env-contract",
                                    format!(
                                        "CREATE_CLOSURE callee {:?} lacks a declared env layout",
                                        callee
                                    ),
                                ));
                            };
                            if captures.len() != declared.len() {
                                return Err(MirVerifyError::inv(
                                    "mir.closure.env-contract",
                                    format!(
                                        "captures ({}) do not match the declared env layout ({})",
                                        captures.len(),
                                        declared.len()
                                    ),
                                ));
                            }
                            for (ci, c) in captures.iter().enumerate() {
                                let entry = match c.kind {
                                    crate::mir::CaptureKind::MoveFromEnv => {
                                        // R1-F01: the transferred field's
                                        // declared type comes from the
                                        // PARENT's own env layout.
                                        match f.env.as_ref().and_then(|e| e.get(c.src as usize)) {
                                            Some(t) => *t,
                                            None => continue, // typing rejects the range
                                        }
                                    }
                                    crate::mir::CaptureKind::Borrow => {
                                        strview_ty.expect("has_borrow implies the lookup ran")
                                    }
                                    _ => match f.slots.get(c.src as usize) {
                                        Some(sl) => sl.ty,
                                        None => continue, // typing rejects the range
                                    },
                                };
                                if entry != declared[ci] {
                                    return Err(MirVerifyError::inv(
                                        "mir.closure.env-contract",
                                        format!(
                                            "capture #{} type {} does not match the declared env field type {}",
                                            ci, entry, declared[ci]
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            // The entry can never be a closure body (no env binding at
            // the root frame). A UNIT's single function is not an
            // executable entry (re-verifiable only, linked via imports),
            // so unit bodies are exempt.
            if !self.module.is_unit && self.module.funcs[self.module.entry as usize].env.is_some() {
                return Err(MirVerifyError::inv(
                    "mir.closure.entry-body",
                    format!(
                        "module entry #{} is a closure body (env-bound bodies are reachable only through CALL_CLOSURE)",
                        self.module.entry
                    ),
                ));
            }
        }
        // ---- per-function deep checks -----------------------------------
        for (fi, f) in self.module.funcs.iter().enumerate() {
            self.check_function(fi as u32, f)?;
        }
        Ok(())
    }

    fn entry_param_ok(&self, ty: TypeIdx) -> bool {
        match self.module.types.get(ty as usize) {
            Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::I64) | Some(MirType::U64)
            | Some(MirType::F32) | Some(MirType::F64) | Some(MirType::Char) => true,
            // PLAN-982（983 文本面）：**只**在模块声明了受检文本入口能力时，入口
            // 形参才可以是借用 `StrView`（请求内借用；view 无 drop 权）。
            Some(MirType::StrView) => self.module.has_text_entry_capability(),
            _ => false,
        }
    }

    fn entry_result_ok(&self, ty: TypeIdx) -> bool {
        self.entry_param_ok(ty)
            || matches!(
                self.module.types.get(ty as usize),
                Some(MirType::Str) | Some(MirType::Record(_))
            )
    }

    fn want_strview_i32(
        &self,
        vt: TypeIdx,
        dt: TypeIdx,
        err: &dyn Fn(&str, String) -> MirVerifyError,
        sp: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let _ = sp;
        if !matches!(self.module.types.get(vt as usize), Some(MirType::StrView)) {
            return Err(err(
                "mir.type-op",
                format!("str operand {} is not a view", vt),
            ));
        }
        if self.module.type_shape(dt) != Some(ScalarShape::I32) {
            return Err(err(
                "mir.type-op",
                format!("str-op dst type {} is not i32", dt),
            ));
        }
        Ok(())
    }

    fn check_sig_indices(&mut self, sig: &FnSig) -> Result<(), MirVerifyError> {
        for p in sig.params.iter().chain(std::iter::once(&sig.result)) {
            if *p as usize >= self.module.types.len() {
                return Err(MirVerifyError::inv(
                    "mir.type-ref",
                    format!("signature references out-of-range type {}", p),
                ));
            }
        }
        Ok(())
    }

    /// FROZEN type-equivalence rule (contract-decision §2 / V-11): structural
    /// `MirType` equality — records compare nominally (owner + name + full
    /// field list), so equal-width different-owner records never match, while
    /// a wire table's duplicate-but-equal entries DO match. Every typing
    /// check in this verifier routes through here; index identity is never
    /// the rule.
    fn ty_equiv(&self, a: TypeIdx, b: TypeIdx) -> bool {
        match (
            self.module.types.get(a as usize),
            self.module.types.get(b as usize),
        ) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// Iterative three-color cycle check over the record type graph. A cycle
    /// means by-value recursion — rejected before any layout derivation.
    /// PLAN-962: whether this module carries the aggregates admission
    /// capability (`av.aggregates/1`, stamped by the aggregates source
    /// profile) — the key that widens the frozen 822 element domain.
    fn aggregates_admission(&self) -> bool {
        self.module
            .capabilities
            .iter()
            .any(|c| c == "av.aggregates/1")
    }

    /// PLAN-822: the frozen admissible element/payload domain (contract-
    /// decision §2) — scalars (incl. wide + char), owned `Str`, records.
    /// Rejected: borrowed views/tokens, `Fn`, and containers (List-of-List
    /// / container payloads keep the first batch one layer deep).
    /// PLAN-962: a module carrying `av.aggregates/1` admits the composite
    /// closure (nested List, narrow numerics i8/u8/u32) — frozen-profile
    /// documents keep the strict domain byte-for-byte.
    fn check_element_domain(
        &mut self,
        elem: TypeIdx,
        ti: usize,
        code: &'static str,
    ) -> Result<(), MirVerifyError> {
        let aggregates = self.aggregates_admission();
        let ok = match self.module.types.get(elem as usize) {
            Some(
                MirType::I32
                | MirType::Bool
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64
                | MirType::Char
                | MirType::Str
                | MirType::Record(_)
                | MirType::Variant(_),
            ) => true,
            // PLAN-962 aggregates admission: the narrow numerics and the
            // nested container itself join the element closure.
            Some(MirType::I8 | MirType::U8 | MirType::U32) if aggregates => true,
            Some(MirType::List(_)) if aggregates => true,
            Some(MirType::Dyn) if code == "mir.type.list-elem" && self.language_values() => true,
            // PLAN-964: an OBJECT element rides the collections family (the
            // closures profile represents a shared object capture as a
            // List<Object> env cell).
            Some(MirType::Object { .. }) if self.collections_surface() => true,
            // PLAN-1081: a CLOSURE element rides the collections family —
            // a closure/fn value is an owned payload (its handle owns the
            // env), exactly like a record; the value crosses container
            // slots under the source move/clone discipline. `Fn` (the
            // borrowed callable view) stays rejected.
            Some(MirType::Closure(_)) if self.collections_surface() => true,
            _ => false,
        };
        if !ok {
            let what = match self.module.types.get(elem as usize) {
                Some(MirType::List(_)) => "a nested List",
                Some(MirType::Fn(_)) => "a function type",
                Some(MirType::StrView) | Some(MirType::RecordView(_)) => "a borrowed view",
                Some(MirType::MutRef) => "a borrow token",
                _ => "an unsupported kind",
            };
            return Err(MirVerifyError::uns(
                code,
                format!("type #{} uses {} as an element/payload kind", ti, what),
            ));
        }
        Ok(())
    }

    /// PLAN-822: the element type of the List-typed slot `base`.
    fn list_elem_of(
        &self,
        f: &MirFunction,
        base: SlotId,
        sp: Option<SpanInfo>,
    ) -> Result<TypeIdx, MirVerifyError> {
        let bt = slot_ty_of(f, base, sp, "list base")?;
        match self.module.types.get(bt as usize) {
            Some(MirType::List(l)) => Ok(l.elem),
            _ => Err(MirVerifyError::inv(
                "mir.list-type",
                format!("slot type {} is not a List", bt),
            )),
        }
    }

    /// PLAN-824: whether a type index lies inside the Dyn erasable closure
    /// — scalars/char/Str/records/lists/variants, checked RECURSIVELY (Fn,
    /// borrows, Places and Dyn itself are rejected at any depth). Cycle
    /// detection is PATH-based (a type table is a DAG — two fields may
    /// share one child type; only a reference back into the CURRENT path
    /// is a cycle, and such recursive types have no finite value). The
    /// frozen 256 depth bound keeps the walk stack-safe.
    ///
    /// F-824-R1-03: verdicts are memoized per `(idx, depth)` — a shared
    /// child is visited once per distinct depth context, never once per
    /// path, so a compact DAG (e.g. nested two-arm variants) costs linear
    /// verify work instead of expanding its 2^n path count.
    fn dyn_whitelisted(&self, root: TypeIdx) -> bool {
        use crate::dynamic::MAX_DYN_DEPTH;
        use std::collections::HashMap;
        fn go(
            m: &MirModule,
            language_values: bool,
            idx: TypeIdx,
            depth: u32,
            path: &mut Vec<TypeIdx>,
            memo: &mut HashMap<(TypeIdx, u32), bool>,
        ) -> bool {
            if depth > MAX_DYN_DEPTH {
                return false;
            }
            if path.contains(&idx) {
                return false;
            }
            if let Some(&cached) = memo.get(&(idx, depth)) {
                return cached;
            }
            path.push(idx);
            let ok = match m.types.get(idx as usize) {
                Some(MirType::Bool) | Some(MirType::I32) | Some(MirType::I64)
                | Some(MirType::U64) | Some(MirType::F32) | Some(MirType::F64)
                | Some(MirType::Char) | Some(MirType::Str) => true,
                Some(MirType::Object { .. } | MirType::Nil) => language_values,
                Some(MirType::Record(r)) => r
                    .fields
                    .iter()
                    .all(|f| go(m, language_values, f.ty, depth + 1, path, memo)),
                Some(MirType::List(l)) => {
                    (language_values && matches!(m.types.get(l.elem as usize), Some(MirType::Dyn)))
                        || go(m, language_values, l.elem, depth + 1, path, memo)
                }
                Some(MirType::Variant(v)) => v.arms.iter().all(|a| match a.payload {
                    Some(p) => go(m, language_values, p, depth + 1, path, memo),
                    None => true,
                }),
                _ => false,
            };
            path.pop();
            memo.insert((idx, depth), ok);
            ok
        }
        go(
            self.module,
            self.language_values(),
            root,
            1,
            &mut Vec::new(),
            &mut HashMap::new(),
        )
    }

    /// PLAN-822: index operands are i32-typed slots (domain checked at
    /// runtime before any buffer access).
    fn check_list_index_slot(
        &self,
        f: &MirFunction,
        index: SlotId,
        sp: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let it = slot_ty_of(f, index, sp, "list index")?;
        if !matches!(self.module.types.get(it as usize), Some(MirType::I32)) {
            return Err(MirVerifyError::inv(
                "mir.list-type",
                format!("list index slot type {} is not i32", it),
            ));
        }
        Ok(())
    }

    /// PLAN-822: disc in range and the payload-slot presence + type match
    /// the arm's declaration.
    fn check_variant_arm(
        &self,
        f: &MirFunction,
        variant_ty: TypeIdx,
        disc: u32,
        payload: Option<&SlotId>,
        sp: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let Some(MirType::Variant(v)) = self.module.types.get(variant_ty as usize) else {
            return Err(MirVerifyError::inv(
                "mir.variant-type",
                format!("type {} is not a Variant", variant_ty),
            ));
        };
        let Some(arm) = v.arms.get(disc as usize) else {
            return Err(MirVerifyError::inv(
                "mir.variant.arm-mismatch",
                format!("discriminant {} out of range ({} arms)", disc, v.arms.len()),
            ));
        };
        match (payload, arm.payload) {
            (Some(slot), Some(want)) => {
                let pt = slot_ty_of(f, *slot, sp, "variant payload")?;
                if !self.ty_equiv(pt, want) {
                    return Err(MirVerifyError::inv(
                        "mir.variant.arm-mismatch",
                        format!("payload type {} != arm payload type {}", pt, want),
                    ));
                }
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(MirVerifyError::inv(
                    "mir.variant.arm-mismatch",
                    format!(
                        "arm `{}` carries no payload but a payload slot is given",
                        arm.name
                    ),
                ));
            }
            (None, Some(_)) => {
                return Err(MirVerifyError::inv(
                    "mir.variant.arm-mismatch",
                    format!("arm `{}` requires a payload but none is given", arm.name),
                ));
            }
        }
        Ok(())
    }

    fn check_record_acyclic(&mut self) -> Result<(), MirVerifyError> {
        const WHITE: u8 = 0;
        const GRAY: u8 = 1;
        const BLACK: u8 = 2;
        let n = self.module.types.len();
        // Scalar/Fn tables have no by-value edges. Retain the original one
        // work step per root, but do not allocate colors or a per-root DFS
        // stack before shared function-analysis admission. Record/Variant
        // graphs continue through the same cycle checker below.
        if self
            .module
            .types
            .iter()
            .all(|t| !matches!(t, MirType::Record(_) | MirType::Variant(_)))
        {
            for _ in 0..n {
                self.tick(1)?;
            }
            return Ok(());
        }
        let mut color = vec![WHITE; n];
        // Explicit (node, next-field-cursor) stack — no host recursion.
        // PLAN-822: Variant payload edges are BY-VALUE like record fields,
        // so they join the same cycle check.
        for root in 0..n {
            if color[root] != WHITE {
                continue;
            }
            let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
            color[root] = GRAY;
            while let Some((node, cur)) = stack.last_mut() {
                self.tick(1)?;
                // F822-R9-02: the EDGE SET drives every node kind — the
                // old Record-only filter here blacked Variants out before
                // this match could extract their payload edges, so a
                // by-value Variant cycle passed the gate.
                let edges: Vec<TypeIdx> = match &self.module.types[*node] {
                    MirType::Record(r) => r.fields.iter().map(|f| f.ty).collect(),
                    MirType::Variant(v) => v.arms.iter().filter_map(|a| a.payload).collect(),
                    _ => vec![],
                };
                if *cur >= edges.len() {
                    color[*node] = BLACK;
                    stack.pop();
                    continue;
                }
                let next = edges[*cur] as usize;
                *cur += 1;
                if next >= n {
                    // Out-of-range refs are reported by the shape pass; skip.
                    continue;
                }
                match color[next] {
                    GRAY => {
                        let name = match &self.module.types[next] {
                            MirType::Record(r) => r.name.clone(),
                            MirType::Variant(v) => v.name.clone(),
                            _ => format!("#{}", next),
                        };
                        return Err(MirVerifyError::inv(
                            "mir.type-cycle",
                            format!(
                                "by-value type graph is cyclic (cycle through `{}`); by-value recursion is unsupported",
                                name
                            ),
                        ));
                    }
                    WHITE => {
                        color[next] = GRAY;
                        stack.push((next, 0));
                    }
                    BLACK => {}
                    _ => unreachable!("color domain is 0..=2"),
                }
            }
        }
        Ok(())
    }

    fn span_of(&self, f: &MirFunction, flat: u32) -> Option<SpanInfo> {
        let mut lo = 0usize;
        let mut hi = f.spans.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            match f.spans[mid].0.cmp(&flat) {
                std::cmp::Ordering::Less => lo = mid + 1,
                _ => hi = mid,
            }
        }
        if lo < f.spans.len() && f.spans[lo].0 == flat {
            Some(f.spans[lo].1)
        } else {
            None
        }
    }

    /// Shared FIELD_GET/FIELD_SET path admission: length cap and base/root
    /// slot must be a record type (segment-by-segment nominal owner checks
    /// happen via `resolve_field_path` at the instruction arms).
    fn check_field_path(
        &mut self,
        f: &MirFunction,
        base: SlotId,
        path: &[u16],
        sp: Option<SpanInfo>,
        what: &str,
    ) -> Result<(), MirVerifyError> {
        if path.is_empty() {
            return Err(MirVerifyError::Invalid {
                code: "mir.field-path".to_string(),
                message: format!("{} carries an empty field path", what),
                span: sp,
            });
        }
        if path.len() > crate::contract::MAX_FIELD_PATH_LEN {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::TableEntries,
                message: format!(
                    "{} field path depth {} exceeds the frozen guard {}",
                    what,
                    path.len(),
                    crate::contract::MAX_FIELD_PATH_LEN
                ),
            });
        }
        let bt = slot_ty_of(f, base, sp, &format!("{} base", what))?;
        match self.module.types.get(bt as usize) {
            // R1-F08: a borrowed record view base reads fields like its
            // record (FieldGet only; FieldSet roots stay owned records).
            Some(MirType::Record(_)) | Some(MirType::RecordView(_)) => Ok(()),
            _ => Err(MirVerifyError::Invalid {
                code: "mir.field-path".to_string(),
                message: format!("{} base slot type {} is not a record", what, bt),
                span: sp,
            }),
        }
    }

    /// The StrView type-table index a BORROW capture's env-field layout
    /// entry uses (the field holds a share view).
    fn strview_type_idx(&self) -> Result<u32, MirVerifyError> {
        self.module
            .types
            .iter()
            .position(|t| matches!(t, MirType::StrView))
            .map(|p| p as u32)
            .ok_or_else(|| {
                MirVerifyError::inv(
                    "mir.capture.borrow-view-type",
                    "a Borrow capture needs a StrView type in the table (the env field is a \
                     share view)"
                        .to_string(),
                )
            })
    }

    /// PLAN-823 T-03 (§5.1.3/§5.1.6): the conservative BORROW-capture
    /// discipline. A borrow-capturing closure is SYNCHRONOUS-ONLY:
    /// * its value never escapes the creating function — the only allowed
    ///   consumers are CALL_CLOSURE on it and DROP of it; frame-exit cleanup
    ///   also releases an unreturned closure (`mir.borrow-closure.escape` /
    ///   `mir.borrow-closure.synchronous`);
    /// * every borrow SOURCE slot is PINNED for the whole function — its
    ///   only allowed uses are the capture itself and VIEW minting
    ///   (`mir.borrow-source-pinned`).
    /// Deliberately a per-function scan (the conservative explicit allow
    /// set of §5.1.6), not a dataflow proof; refinement belongs to a
    /// later revision with the full borrow analysis.
    fn check_borrow_discipline(&self, f: &MirFunction) -> Result<(), MirVerifyError> {
        // (src slot, site position) pins and (dst slot, site) escapes.
        let mut pins: Vec<(u32, (usize, usize))> = Vec::new();
        let mut escs: Vec<(u32, (usize, usize))> = Vec::new();
        for (bi, b) in f.blocks.iter().enumerate() {
            for (ii, instr) in b.instrs.iter().enumerate() {
                if let MirInstr::CreateClosure { dst, captures, .. } = instr {
                    if captures
                        .iter()
                        .any(|c| c.kind == crate::mir::CaptureKind::Borrow)
                    {
                        escs.push((*dst, (bi, ii)));
                        for c in captures {
                            if c.kind == crate::mir::CaptureKind::Borrow {
                                // Pin only the admissible borrow domain (a
                                // Str-typed source): a non-Str borrow is
                                // the typing arm's `borrow-domain`
                                // rejection — pinning it here would mask
                                // the right error code.
                                let src_is_str = matches!(
                                    f.slots
                                        .get(c.src as usize)
                                        .and_then(|s| self.module.types.get(s.ty as usize)),
                                    Some(MirType::Str)
                                );
                                if src_is_str {
                                    pins.push((c.src, (bi, ii)));
                                }
                            }
                        }
                    }
                }
            }
        }
        if pins.is_empty() {
            return Ok(());
        }
        // Let-bound closures are first produced in a temporary and then
        // transferred into their HIR local. Follow those ownership moves.
        let mut aliases: Vec<(u32, usize)> = escs
            .iter()
            .enumerate()
            .map(|(root, (slot, _))| (*slot, root))
            .collect();
        loop {
            let mut changed = false;
            for block in &f.blocks {
                for instr in &block.instrs {
                    if let MirInstr::Move { dst, src } = instr {
                        if let Some((_, root)) = aliases.iter().find(|(slot, _)| slot == src) {
                            if !aliases.iter().any(|(slot, _)| slot == dst) {
                                aliases.push((*dst, *root));
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let mut calls = vec![0usize; escs.len()];
        let mut drops = vec![0usize; escs.len()];
        for (bi, b) in f.blocks.iter().enumerate() {
            for (ii, instr) in b.instrs.iter().enumerate() {
                let uses = instr.use_slots();
                for (slot, k) in &aliases {
                    match instr {
                        MirInstr::CallClosure { closure, .. } if closure == slot => {
                            calls[*k] += 1;
                        }
                        MirInstr::Drop { slot: dropped } if dropped == slot => {
                            drops[*k] += 1;
                        }
                        MirInstr::Move { src, .. } if src == slot => {}
                        _ => {
                            if uses.contains(slot) {
                                return Err(MirVerifyError::inv(
                                    "mir.borrow-closure.escape",
                                    format!(
                                        "slot #{} of `{}` is a borrow-bound closure: its only \
                                         allowed consumers are CALL_CLOSURE and DROP in the \
                                         creating function (synchronous non-escaping use)",
                                        slot, f.name
                                    ),
                                ));
                            }
                        }
                    }
                }
                for (src, site) in pins.iter() {
                    if uses.contains(src) {
                        let is_site = (bi, ii) == *site;
                        let is_view = matches!(instr, MirInstr::View { src: s, .. } if s == src);
                        if !is_site && !is_view {
                            return Err(MirVerifyError::inv(
                                "mir.borrow-source-pinned",
                                format!(
                                    "slot #{} of `{}` is a borrow-capture source: pinned for \
                                     the whole function (only the capture itself and VIEW \
                                     minting may use it)",
                                    src, f.name
                                ),
                            ));
                        }
                    }
                }
            }
            // Terminators are uses too: RETURNING a borrow-bound closure is
            // the canonical escape (Branch conditions are typing-rejected
            // but land here first).
            let term_uses: Vec<u32> = match &b.term {
                crate::mir::Terminator::Return { value } => vec![*value],
                crate::mir::Terminator::Branch { cond, .. } => vec![*cond],
                crate::mir::Terminator::Jump { .. } => vec![],
                crate::mir::Terminator::LangFail { message } => vec![*message],
            };
            for (slot, _) in aliases.iter() {
                if term_uses.contains(slot) {
                    return Err(MirVerifyError::inv(
                        "mir.borrow-closure.escape",
                        format!(
                            "slot #{} of `{}` is a borrow-bound closure: RETURN carries it out \
                             of the creating function (synchronous non-escaping use only)",
                            slot, f.name
                        ),
                    ));
                }
            }
            if matches!(b.term, crate::mir::Terminator::Return { .. }) {
                // RETURN drains owned closure slots through the runtime's
                // frame cleanup path. Treat that as the implicit DROP for
                // every borrow-bound closure that was not returned above.
                for (_, root) in &aliases {
                    drops[*root] += 1;
                }
            }
        }
        for (k, (dst, _)) in escs.iter().enumerate() {
            if calls[k] == 0 || drops[k] == 0 {
                return Err(MirVerifyError::inv(
                    "mir.borrow-closure.synchronous",
                    format!(
                        "borrow-bound closure slot #{} of `{}` must be CALLED and released in \
                         the creating function (found {} calls, {} drops)",
                        dst, f.name, calls[k], drops[k]
                    ),
                ));
            }
        }
        Ok(())
    }

    /// R1-F01 (§5.1.4/§5.5): the conservative env-capture discipline.
    /// (a) a slot whose value originates from ENV_GET is an ENV ALIAS (a
    ///     share of the parent's field): a MOVE capture of an OWNED-typed
    ///     alias never transfers the parent's responsibility — the
    ///     dedicated MoveFromEnv op is the only transfer path
    ///     (`mir.capture.env-alias`);
    /// (b) a field moved out by MoveFromEnv may not be re-read (ENV_GET)
    ///     or re-transferred anywhere in the same body
    ///     (`mir.env.field-moved`).
    /// Deliberately a per-function conservative scan (§5.1.6's allow-set).
    fn check_env_capture_discipline(&mut self, f: &MirFunction) -> Result<(), MirVerifyError> {
        fn alias(bits: &[u64], slot: u32) -> bool {
            bits.get(slot as usize / 64)
                .map(|w| w & (1u64 << (slot % 64)) != 0)
                .unwrap_or(false)
        }
        fn assign(bits: &mut [u64], slot: u32, value: bool) {
            if let Some(w) = bits.get_mut(slot as usize / 64) {
                let mask = 1u64 << (slot % 64);
                if value {
                    *w |= mask;
                } else {
                    *w &= !mask;
                }
            }
        }
        fn transfer(bits: &mut [u64], instr: &MirInstr) {
            match instr {
                MirInstr::EnvGet { dst, .. } => assign(bits, *dst, true),
                MirInstr::Clone { dst, .. } => assign(bits, *dst, false),
                MirInstr::Move { dst, src } | MirInstr::Copy { dst, src } => {
                    let source = alias(bits, *src);
                    assign(bits, *dst, source);
                }
                MirInstr::FieldMove { dst, root, .. }
                | MirInstr::VariantProject { dst, root, .. } => {
                    let source = alias(bits, *root);
                    assign(bits, *dst, source);
                }
                MirInstr::FieldGet { dst, base, .. }
                | MirInstr::ListGet { dst, base, .. }
                | MirInstr::ListFieldGet { dst, base, .. } => {
                    let source = alias(bits, *base);
                    assign(bits, *dst, source);
                }
                // In-place field writes retain the root's provenance.
                MirInstr::FieldSet { .. } => {}
                _ => {
                    if let Some(dst) = instr.def_slot() {
                        assign(bits, dst, false);
                    }
                }
            }
        }

        let blocks = f.blocks.len();
        let words = (f.slots.len().saturating_add(63)) / 64;
        let instructions = f
            .blocks
            .iter()
            .fold(0usize, |n, b| n.saturating_add(b.instrs.len()));
        let capture_count = f
            .blocks
            .iter()
            .flat_map(|b| &b.instrs)
            .fold(0usize, |n, instr| match instr {
                MirInstr::CreateClosure { captures, .. } => n.saturating_add(captures.len()),
                _ => n,
            });
        let max_operands = f
            .blocks
            .iter()
            .flat_map(|b| &b.instrs)
            .map(|instr| match instr {
                MirInstr::Aggregate { operands, .. }
                | MirInstr::Call { args: operands, .. }
                | MirInstr::ForeignCall { args: operands, .. }
                | MirInstr::CallClosure { args: operands, .. } => operands.len(),
                MirInstr::CreateClosure { captures, .. } => captures.len(),
                _ => 1,
            })
            .max()
            .unwrap_or(0);
        let analysis_bound = blocks
            .saturating_mul(words)
            .saturating_mul(std::mem::size_of::<u64>())
            .saturating_mul(2)
            .saturating_add(blocks.saturating_mul(80))
            .saturating_add(instructions.saturating_mul(16))
            .saturating_add(capture_count.saturating_mul(40))
            .saturating_add(max_operands.saturating_mul(std::mem::size_of::<u32>()));
        self.analysis_bytes = self.analysis_bytes.max(analysis_bound as u64);
        if analysis_bound > MAX_VERIFY_ANALYSIS_BYTES {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifyMemory,
                message: format!(
                    "environment alias analysis for `{}` needs up to {} bytes, above {}",
                    f.name, analysis_bound, MAX_VERIFY_ANALYSIS_BYTES
                ),
            });
        }
        if f.entry as usize >= blocks {
            return Err(MirVerifyError::inv(
                "mir.block-ref",
                format!("invalid entry block in `{}`", f.name),
            ));
        }
        let mut moved_fields = std::collections::HashSet::<u32>::new();
        let mut successors = Vec::with_capacity(blocks);
        for b in &f.blocks {
            let targets = match b.term {
                Terminator::Jump { target } => vec![target],
                Terminator::Branch { then, else_, .. } if then != else_ => vec![then, else_],
                Terminator::Branch { then, .. } => vec![then],
                Terminator::Return { .. } | Terminator::LangFail { .. } => Vec::new(),
            };
            if targets.iter().any(|target| *target as usize >= blocks) {
                return Err(MirVerifyError::inv(
                    "mir.block-ref",
                    format!("invalid branch target in `{}`", f.name),
                ));
            }
            successors.push(targets);
            for instr in &b.instrs {
                self.tick(1)?;
                match instr {
                    MirInstr::EnvGet { dst, .. } if *dst as usize >= f.slots.len() => {
                        return Err(MirVerifyError::inv(
                            "mir.slot-ref",
                            format!("invalid ENV_GET destination slot #{dst} in `{}`", f.name),
                        ));
                    }
                    MirInstr::Move { dst, src } | MirInstr::FieldMove { dst, root: src, .. }
                        if *dst as usize >= f.slots.len() || *src as usize >= f.slots.len() =>
                    {
                        return Err(MirVerifyError::inv(
                            "mir.slot-ref",
                            format!("invalid MOVE slots in `{}`", f.name),
                        ));
                    }
                    MirInstr::CreateClosure { captures, .. } => {
                        self.tick(captures.len() as u64)?;
                        for c in captures {
                            if c.kind == crate::mir::CaptureKind::MoveFromEnv
                                && !moved_fields.insert(c.src)
                            {
                                return Err(MirVerifyError::inv(
                                    "mir.env.field-moved",
                                    format!(
                                        "env field {} of `{}` is transferred more than once",
                                        c.src, f.name
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        // May-alias dataflow: union at branch joins, strong updates at each
        // definition. Empty entry state; loops converge by monotone union.
        let mut inputs = vec![vec![0u64; words]; blocks];
        let mut outputs = vec![vec![0u64; words]; blocks];
        let mut seen = vec![false; blocks];
        let mut queued = vec![false; blocks];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(f.entry as usize);
        queued[f.entry as usize] = true;
        while let Some(bi) = queue.pop_front() {
            queued[bi] = false;
            self.tick(1 + words as u64)?;
            let mut live = inputs[bi].clone();
            for instr in &f.blocks[bi].instrs {
                self.tick(1)?;
                transfer(&mut live, instr);
            }
            if !seen[bi] || live != outputs[bi] {
                seen[bi] = true;
                outputs[bi] = live;
                for target in &successors[bi] {
                    let ti = *target as usize;
                    let mut changed = !seen[ti];
                    for (dst, src) in inputs[ti].iter_mut().zip(&outputs[bi]) {
                        let merged = *dst | *src;
                        changed |= merged != *dst;
                        *dst = merged;
                    }
                    if changed && !queued[ti] {
                        queued[ti] = true;
                        queue.push_back(ti);
                    }
                }
            }
        }
        // Check the converged states, never a transient under-approximation.
        for (bi, b) in f.blocks.iter().enumerate() {
            if !seen[bi] {
                continue;
            }
            let mut live = inputs[bi].clone();
            for instr in &b.instrs {
                self.tick(1)?;
                if let MirInstr::EnvGet { field, .. } = instr {
                    if moved_fields.contains(&(*field as u32)) {
                        return Err(MirVerifyError::inv(
                            "mir.env.field-moved",
                            format!(
                                "env field {} of `{}` was transferred out and re-read",
                                field, f.name
                            ),
                        ));
                    }
                }
                let consuming: Vec<u32> = match instr {
                    MirInstr::Aggregate { operands, .. }
                    | MirInstr::Call { args: operands, .. }
                    | MirInstr::ForeignCall { args: operands, .. }
                    | MirInstr::CallClosure { args: operands, .. } => operands.clone(),
                    MirInstr::ListPush { value, .. }
                    | MirInstr::ListInsert { value, .. }
                    | MirInstr::ListSet { value, .. }
                    | MirInstr::ListFieldSet { value, .. }
                    | MirInstr::FieldSet { value, .. }
                    | MirInstr::PlaceSet { value, .. } => vec![*value],
                    MirInstr::VariantPack { payload, .. }
                    | MirInstr::VariantSwitch { payload, .. } => payload.iter().copied().collect(),
                    MirInstr::Drop { slot } => vec![*slot],
                    MirInstr::FieldMove { root, .. } => vec![*root],
                    MirInstr::CreateClosure { captures, .. } => captures
                        .iter()
                        .filter(|c| c.kind == crate::mir::CaptureKind::Move)
                        .map(|c| c.src)
                        .collect(),
                    _ => Vec::new(),
                };
                self.tick(consuming.len() as u64)?;
                for src in consuming {
                    if alias(&live, src)
                        && f.slots
                            .get(src as usize)
                            .map(|slot| crate::ownership::type_is_owned(self.module, slot.ty))
                            .unwrap_or(false)
                    {
                        return Err(MirVerifyError::inv("mir.capture.env-alias", format!("owned environment alias slot #{} of `{}` cannot be transferred by this instruction", src, f.name)));
                    }
                }
                transfer(&mut live, instr);
            }
            if let Terminator::Return { value } = b.term {
                if alias(&live, value)
                    && f.slots
                        .get(value as usize)
                        .map(|slot| crate::ownership::type_is_owned(self.module, slot.ty))
                        .unwrap_or(false)
                {
                    return Err(MirVerifyError::inv(
                        "mir.capture.env-alias",
                        format!(
                            "owned environment alias slot #{} of `{}` cannot be returned",
                            value, f.name
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// PLAN-961 typed-numeric Bin typing — cold frame (F-941-15): the deep
    /// verify path must fit the frozen 256 KiB small-stack contract, so the
    /// numeric-only temporaries live in their own #[inline(never)] frame;
    /// non-numeric deep chains never pay for them.
    #[inline(never)]
    fn check_bin_numeric(
        &self,
        f: &MirFunction,
        op: crate::mir::NumOp,
        ty: crate::mir::NumTy,
        dst: SlotId,
        lhs: SlotId,
        rhs: SlotId,
        sp: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let err = |code: &str, msg: String| MirVerifyError::Invalid {
            code: code.to_string(),
            message: msg,
            span: sp,
        };
        if !self.numeric_surface() {
            return Err(err(
                "mir.profile-type",
                "typed numeric operations exist only in the data-family profiles".to_string(),
            ));
        }
        let lt = slot_ty_of(f, lhs, sp, "bin lhs")?;
        let rt = slot_ty_of(f, rhs, sp, "bin rhs")?;
        let dt = slot_ty_of(f, dst, sp, "bin dst")?;
        let want = ty.shape_of();
        if lt != rt || self.module.type_shape(lt) != Some(want) {
            return Err(err(
                "mir.bin-type",
                format!(
                    "numeric {:?} on {:?} requires same-type {:?} operands (types {},{})",
                    op, ty, want, lt, rt
                ),
            ));
        }
        let dst_shape = self.module.type_shape(dt);
        let ok = if matches!(
            op,
            crate::mir::NumOp::Lt
                | crate::mir::NumOp::Le
                | crate::mir::NumOp::Gt
                | crate::mir::NumOp::Ge
                | crate::mir::NumOp::Eq
                | crate::mir::NumOp::Ne
        ) {
            dst_shape == Some(ScalarShape::Bool)
        } else if matches!(
            op,
            crate::mir::NumOp::And
                | crate::mir::NumOp::Or
                | crate::mir::NumOp::Xor
                | crate::mir::NumOp::Shl
                | crate::mir::NumOp::Shr
        ) && ty.is_float()
        {
            false
        } else {
            dst_shape == Some(want)
        };
        if !ok {
            return Err(err(
                "mir.bin-type",
                format!(
                    "numeric {:?} on {:?} does not type check for dst type {}",
                    op, ty, dt
                ),
            ));
        }
        Ok(())
    }

    /// F-941-15 cold half A of the per-instruction typing walk (union frame halved; see check_function).
    #[inline(never)]
    fn check_instr_typing_a(
        &mut self,
        f: &MirFunction,
        instr: &MirInstr,
        sp: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let err = |code: &str, msg: String| MirVerifyError::Invalid {
            code: code.to_string(),
            message: msg,
            span: sp,
        };
        let module = self.module;
        match instr {
            MirInstr::Const { dst, value } => {
                let dt = slot_ty_of(f, *dst, sp, "const dst")?;
                // PLAN-821: Char/Str consts are owned-profile-only
                // and carry NO ScalarShape — they type against the
                // slot's MirType directly (below).
                let owned_const = match self.module.consts.get(*value as usize) {
                    Some(MirConst::Char(_)) | Some(MirConst::Str(_)) => true,
                    _ => false,
                };
                if owned_const && !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-const",
                        "char/Str constants exist only in the owned profile".to_string(),
                    ));
                }
                let want_shape = match self.module.consts.get(*value as usize) {
                    Some(MirConst::I32(_)) => Some(ScalarShape::I32),
                    Some(MirConst::Bool(_)) => Some(ScalarShape::Bool),
                    Some(MirConst::I64(_)) => Some(ScalarShape::I64),
                    Some(MirConst::U64(_)) => Some(ScalarShape::U64),
                    Some(MirConst::F32Bits(_)) => Some(ScalarShape::F32),
                    Some(MirConst::F64Bits(_)) => Some(ScalarShape::F64),
                    // PLAN-961: narrow-integer consts.
                    Some(MirConst::I8(_)) => Some(ScalarShape::I8),
                    Some(MirConst::U8(_)) => Some(ScalarShape::U8),
                    Some(MirConst::U32(_)) => Some(ScalarShape::U32),
                    Some(MirConst::Char(_)) | Some(MirConst::Str(_)) => None,
                    // PLAN-964 (L-21): the nil const matches the Nil type
                    // only (checked through the shape lane below).
                    Some(MirConst::Nil) => None,
                    None => {
                        return Err(err(
                            "mir.const-ref",
                            format!(
                                "const instruction references out-of-range constant {}",
                                value
                            ),
                        ))
                    }
                };
                // PLAN-961: wide/narrow numeric constants exist in
                // every numeric-surface family (the data gate below
                // keeps the core-i32 profile frozen).
                if let (Some(shape), false) = (want_shape, self.numeric_surface()) {
                    if !matches!(shape, ScalarShape::I32 | ScalarShape::Bool) {
                        return Err(err(
                            "mir.profile-const",
                            format!(
                                "constant of shape {:?} exists only in the data profile",
                                shape
                            ),
                        ));
                    }
                }
                // Shape comparison, not index equality: a wire table
                // may carry duplicate scalar type entries.
                if let Some(c) = self.module.consts.get(*value as usize) {
                    if let Some(want_ty) = match c {
                        MirConst::Char(_) => Some(MirType::Char),
                        // PLAN-964 (L-21): a Nil const admits only a
                        // Nil-typed destination (zero-width).
                        MirConst::Nil => Some(MirType::Nil),
                        // A Str const materializes by DESTINATION
                        // kind: an owned Str slot gets a fresh
                        // allocation; a StrView slot gets the
                        // immortal static view (770 literal rule).
                        MirConst::Str(_) => Some(match self.module.types.get(dt as usize) {
                            Some(MirType::StrView) => MirType::StrView,
                            _ => MirType::Str,
                        }),
                        _ => None,
                    } {
                        if self.module.types.get(dt as usize) != Some(&want_ty) {
                            return Err(err(
                                "mir.const-type",
                                format!(
                                    "const into slot {} of type {}; constant has another kind",
                                    dst, dt
                                ),
                            ));
                        }
                    }
                }
                if self.module.type_shape(dt) != want_shape {
                    return Err(err(
                        "mir.const-type",
                        format!(
                            "const into slot {} of type {}; constant has another shape",
                            dst, dt
                        ),
                    ));
                }
            }
            MirInstr::Copy { dst, src } => {
                let dt = slot_ty_of(f, *dst, sp, "copy dst")?;
                let st = slot_ty_of(f, *src, sp, "copy src")?;
                if !self.ty_equiv(dt, st) {
                    return Err(err(
                        "mir.copy-type",
                        format!("copy from type {} into type {}", st, dt),
                    ));
                }
                // PLAN-821: copying an owned value or a write token
                // is a borrow of ownership semantics the profile
                // forbids — explicit CLONE / token transfer only.
                // A StrView copy is the legal multi-view share.
                if self.is_owned {
                    match self.module.types.get(st as usize) {
                        Some(MirType::Str) | Some(MirType::MutRef) | Some(MirType::Record(_))
                            if crate::ownership::type_is_owned(self.module, st) =>
                        {
                            return Err(err(
                                "mir.copy-owned",
                                format!(
                                    "COPY of owned type {}; use CLONE/MOVE (or the value path)",
                                    st
                                ),
                            ));
                        }
                        Some(MirType::MutRef) => {
                            return Err(err(
                                "mir.copy-owned",
                                "COPY of a MutRef token breaks exclusivity".to_string(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
            MirInstr::Bin { op, dst, lhs, rhs } => {
                let dt = slot_ty_of(f, *dst, sp, "bin dst")?;
                let lt = slot_ty_of(f, *lhs, sp, "bin lhs")?;
                let rt = slot_ty_of(f, *rhs, sp, "bin rhs")?;
                let shapes = (
                    self.module.type_shape(dt),
                    self.module.type_shape(lt),
                    self.module.type_shape(rt),
                );
                // PLAN-961: the typed numeric surface replaces the
                // PLAN-820 "no wide-scalar arithmetic" gate. The
                // structural `Num { op, ty }` form requires BOTH
                // operands to have exactly `ty`'s shape (no implicit
                // promotion) and dst = ty (or Bool for comparisons);
                // bit/shift ops are int-only. The legacy `*I32`
                // subcodes keep their frozen i32-only typing, and
                // EqI32/NeI32 no longer admit wide/float pairs —
                // those MUST ride Num{Eq/Ne, ty} (the old admission
                // let through shapes the interpreter could not
                // execute; closing that seam is part of this plan).
                if let BinOp::Num { op: nop, ty } = op {
                    return self
                        .check_bin_numeric(f, *nop, *ty, *dst, *lhs, *rhs, sp)
                        .map(|()| ());
                } else {
                    use ScalarShape::*;
                    let ok = if op.result_is_bool() {
                        match op {
                            // Tightened by PLAN-961: the legacy
                            // equality subcodes admit i32/bool only;
                            // every other scalar pair rides Num.
                            BinOp::EqI32 | BinOp::NeI32 => {
                                shapes.0 == Some(Bool)
                                    && (shapes.1 == Some(I32) && shapes.2 == Some(I32)
                                        || shapes.1 == Some(Bool) && shapes.2 == Some(Bool))
                            }
                            _ => shapes == (Some(Bool), Some(I32), Some(I32)),
                        }
                    } else {
                        shapes == (Some(I32), Some(I32), Some(I32))
                    };
                    if !ok {
                        return Err(err(
                            "mir.bin-type",
                            format!(
                                "binary {:?} does not type check for dst/lhs/rhs types ({},{},{})",
                                op, dt, lt, rt
                            ),
                        ));
                    }
                }
            }
            MirInstr::Un { op, dst, operand } => {
                let dt = slot_ty_of(f, *dst, sp, "un dst")?;
                let ot = slot_ty_of(f, *operand, sp, "un operand")?;
                let ok = match op {
                    UnOp::NegI32 => {
                        self.module.type_shape(dt) == Some(ScalarShape::I32)
                            && self.module.type_shape(ot) == Some(ScalarShape::I32)
                    }
                    UnOp::NotBool => {
                        self.module.type_shape(dt) == Some(ScalarShape::Bool)
                            && self.module.type_shape(ot) == Some(ScalarShape::Bool)
                    }
                    // PLAN-961: typed negation — signed or float
                    // only (unsigned negation is typed away before
                    // lowering), same shape for operand and result.
                    UnOp::NegT { ty } => {
                        self.numeric_surface()
                            && !matches!(
                                ty,
                                crate::mir::NumTy::U8
                                    | crate::mir::NumTy::U32
                                    | crate::mir::NumTy::U64
                                    | crate::mir::NumTy::I32
                            )
                            && self.module.type_shape(ot) == Some(ty.shape_of())
                            && self.module.type_shape(dt) == Some(ty.shape_of())
                    }
                };
                if !ok {
                    return Err(err(
                        "mir.un-type",
                        format!(
                            "unary {:?} does not type check for types ({},{})",
                            op, dt, ot
                        ),
                    ));
                }
            }
            MirInstr::Conv { dst, src } => {
                // PLAN-961: the frozen conversion matrix — both
                // slots numeric, any pair admitted (identity pairs
                // are pointless but legal); the runtime checks
                // narrowing/float->int value bounds.
                let dt = slot_ty_of(f, *dst, sp, "conv dst")?;
                let st = slot_ty_of(f, *src, sp, "conv src")?;
                if !self.numeric_surface() {
                    return Err(err(
                        "mir.profile-type",
                        "numeric conversions exist only in the data-family profiles".to_string(),
                    ));
                }
                let ok = matches!(
                    (self.module.type_shape(dt), self.module.type_shape(st)),
                    (Some(a), Some(b)) if a.num_ty().is_some() && b.num_ty().is_some()
                );
                if !ok {
                    return Err(err(
                        "mir.conv-type",
                        format!("conversion requires numeric slots (types {},{})", st, dt),
                    ));
                }
            }
            MirInstr::ForeignCall {
                declaration,
                dst,
                args,
                callbacks,
            } => {
                declaration
                    .validate()
                    .map_err(|m| err("mir.foreign-contract", m))?;
                if !self
                    .module
                    .capabilities
                    .iter()
                    .any(|c| c == ace_foreign_contract::manual::SOURCE_CAP)
                    || declaration
                        .capabilities
                        .iter()
                        .any(|c| !self.module.capabilities.contains(c))
                {
                    return Err(err(
                        "mir.foreign-capability",
                        "foreign requirements are missing".into(),
                    ));
                }
                let matches_lane = |ty: u32, lane: ace_foreign_contract::LaneKind| match (
                    self.module.types.get(ty as usize),
                    lane,
                ) {
                    (Some(MirType::I32), ace_foreign_contract::LaneKind::I32)
                    | (Some(MirType::Bool), ace_foreign_contract::LaneKind::Bool) => true,
                    _ => false,
                };
                // PLAN-939 S06: every Callback lane carries EXACTLY
                // ONE named target (a reviewed identifier naming a
                // free source fn; the backend emits the static
                // trampoline); its args slot is a placeholder that
                // no reader may consume.
                let res_params = declaration
                    .resource()
                    .map(|r| r.params.as_slice())
                    .unwrap_or_default();
                {
                    let mut seen = std::collections::BTreeSet::new();
                    for (lane, name) in callbacks {
                        if (*lane as usize) >= res_params.len()
                            || !seen.insert(*lane)
                            || !matches!(
                                res_params[*lane as usize],
                                ace_foreign_contract::resource::ResKind::Callback { .. }
                            )
                            || name.is_empty()
                            || name.len() > 64
                            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                        {
                            return Err(err(
                                "mir.foreign-callback",
                                "callback lane target is outside the reviewed surface".into(),
                            ));
                        }
                    }
                    let want = res_params
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            matches!(p, ace_foreign_contract::resource::ResKind::Callback { .. })
                        })
                        .count();
                    if callbacks.len() != want {
                        return Err(err(
                            "mir.foreign-callback",
                            "every callback lane needs exactly one named target".into(),
                        ));
                    }
                }
                if args.len() != declaration.arity() {
                    return Err(err(
                        "mir.foreign-arity",
                        "foreign argument count mismatch".into(),
                    ));
                }
                for (arg, lane) in args.iter().zip(declaration.params()) {
                    let ty = slot_ty_of(f, *arg, sp, "foreign arg")?;
                    if !matches_lane(ty, *lane) {
                        return Err(err(
                            "mir.foreign-type",
                            "foreign argument logical type mismatch".into(),
                        ));
                    }
                }
                let ty = slot_ty_of(f, *dst, sp, "foreign dst")?;
                if !matches_lane(ty, declaration.result()) {
                    return Err(err(
                        "mir.foreign-type",
                        "foreign result logical type mismatch".into(),
                    ));
                }
            }
            MirInstr::Call { callee, dst, args } => {
                let dt = slot_ty_of(f, *dst, sp, "call dst")?;
                let callee_sig: &FnSig = match callee {
                    CalleeRef::Func(idx) => {
                        if self.module.is_unit {
                            return Err(err(
                                "mir.unit-callee",
                                "a unit must address callees through imports".to_string(),
                            ));
                        }
                        // PLAN-823: a closure body executes ONLY with
                        // an env binding — a direct call can never
                        // provide one. PLAN-1081: an EMPTY-env body (a
                        // named fn item also referenced as a value) needs
                        // no binding, so direct calls stay legal — the
                        // item is simultaneously callable and a value.
                        let env_needs_binding = self
                            .module
                            .funcs
                            .get(*idx as usize)
                            .is_some_and(|t| t.env.as_ref().is_some_and(|e| !e.is_empty()));
                        if env_needs_binding {
                            return Err(err(
                                "mir.closure.body-direct-call",
                                format!(
                                    "direct call targets function #{} which is a \
                                             closure body (reachable only through CALL_CLOSURE)",
                                    idx
                                ),
                            ));
                        }
                        match module.funcs.get(*idx as usize) {
                            Some(target) => match module.sig_of(target.sig) {
                                Some(s) => s,
                                None => {
                                    return Err(err(
                                        "mir.callee-sig",
                                        format!("callee #{} has no function type", idx),
                                    ))
                                }
                            },
                            None => {
                                return Err(err(
                                    "mir.callee-ref",
                                    format!("call references out-of-range function {}", idx),
                                ))
                            }
                        }
                    }
                    CalleeRef::Import(idx) => match module.imports.get(*idx as usize) {
                        Some(imp) => match module.sig_of(imp.sig) {
                            Some(s) => s,
                            None => {
                                return Err(err(
                                    "mir.import-sig",
                                    format!("import #{} has no function type", idx),
                                ))
                            }
                        },
                        None => {
                            return Err(err(
                                "mir.import-ref",
                                format!("call references out-of-range import {}", idx),
                            ))
                        }
                    },
                };
                if callee_sig.params.len() != args.len() {
                    return Err(err(
                        "mir.call-arity",
                        format!(
                            "call passes {} args, callee signature takes {}",
                            args.len(),
                            callee_sig.params.len()
                        ),
                    ));
                }
                for (ai, (a, want)) in args.iter().zip(callee_sig.params.iter()).enumerate() {
                    let at = slot_ty_of(f, *a, sp, &format!("call arg #{}", ai))?;
                    // R1-F08 borrowing coercion: an owned record arg
                    // satisfies a RecordView (borrowed) param — one
                    // way only (a view never satisfies an owned
                    // param; .move semantics stay explicit).
                    let borrow_ok = matches!(
                        (
                            self.module.types.get(at as usize),
                            self.module.types.get(*want as usize),
                        ),
                        (
                            Some(MirType::Record(ra)),
                            Some(MirType::RecordView(rw)),
                        ) if ra == rw
                    );
                    if !self.ty_equiv(at, *want) && !borrow_ok {
                        return Err(err(
                            "mir.call-arg-type",
                            format!("call arg #{} has type {}, signature wants {}", ai, at, want),
                        ));
                    }
                }
                // R2-F08': a mut-param callee returns the t_MutRet
                // wrapper — the call dst legitimately holds the
                // WRAPPER (the adapter unwraps right after).
                let dst_is_wrapper = (|| {
                    matches!(
                        self.module.types.get(callee_sig.result as usize),
                        Some(MirType::Record(r)) if r.name.starts_with("t_MutRet_")
                    )
                })();
                if !self.ty_equiv(dt, callee_sig.result) && !dst_is_wrapper {
                    return Err(err(
                        "mir.call-result-type",
                        format!(
                            "call dst has type {}, callee returns {}",
                            dt, callee_sig.result
                        ),
                    ));
                }
            }
            // PLAN-960: global cells are checked against the module's
            // global table (index range + slot type == cell type).
            MirInstr::GlobalGet { dst, global } | MirInstr::GlobalTake { dst, global } => {
                let take = matches!(instr, MirInstr::GlobalTake { .. });
                if take
                    && (!self.is_closures
                        || !self
                            .module
                            .capabilities
                            .iter()
                            .any(|cap| cap == crate::contract::GLOBAL_TAKE_CAPABILITY))
                {
                    return Err(err(
                        "global.take-capability",
                        "global take requires its AV capability".into(),
                    ));
                }
                // The decoded-unit path carries no global table (cells
                // are program-scoped at link, never on the wire): the
                // range/type check runs when the table is present and
                // the executor guards the decoded case.
                if !self.module.globals.is_empty() {
                    let g = self.module.globals.get(*global as usize).ok_or_else(|| {
                        err("global.reference", "global index out of range".into())
                    })?;
                    if take && !g.mutable {
                        return Err(err(
                            "global.immutable",
                            "cannot take an immutable global".into(),
                        ));
                    }
                    let dst_ty = slot_ty_of(f, *dst, sp, "global dst")?;
                    if !self.ty_equiv(dst_ty, g.ty) {
                        return Err(err(
                            "global.type",
                            "global read into a slot of another type".into(),
                        ));
                    }
                }
            }
            MirInstr::GlobalSet { dst, global, src } => {
                if !self.module.globals.is_empty() {
                    let g = self.module.globals.get(*global as usize).ok_or_else(|| {
                        err("global.reference", "global index out of range".into())
                    })?;
                    let src_ty = slot_ty_of(f, *src, sp, "global src")?;
                    if !self.ty_equiv(src_ty, g.ty) {
                        return Err(err(
                            "global.type",
                            "global write from a slot of another type".into(),
                        ));
                    }
                }
                let _ = dst;
            }
            MirInstr::HostCall { host, arg } => {
                // PLAN-968: scheduler modules carry the same console
                // HostCall seam as the host-console profile.
                let sched_console = self
                    .module
                    .host_imports
                    .iter()
                    .any(|h| h.symbol.starts_with("auto.console."));
                if !self.module.is_host_console_profile() && !sched_console {
                    return Err(err("host.profile", "host call in a legacy profile".into()));
                }
                let h = self
                    .module
                    .host_imports
                    .get(*host as usize)
                    .ok_or_else(|| err("host.reference", "host index out of range".into()))?;
                let sig = self
                    .module
                    .sig_of(h.sig)
                    .ok_or_else(|| err("host.signature", "invalid host signature".into()))?;
                let at = slot_ty_of(f, *arg, sp, "host arg")?;
                if !self.ty_equiv(at, sig.params[0]) {
                    return Err(err("host.signature", "host operand type mismatch".into()));
                }
            }
            MirInstr::PlatformCall { dst, host, args } => {
                // PLAN-973: packages-lane system-library modules lower
                // platform ops on the closures family (the frozen op table
                // is the signature/capability authority).
                let sys_platform_module = self.module.host_imports.iter().all(|h| {
                    h.symbol.starts_with("auto.platform.") || h.symbol.starts_with("auto.console.")
                }) && self
                    .module
                    .host_imports
                    .iter()
                    .any(|h| h.symbol.starts_with("auto.platform."));
                if !self.module.is_host_console_profile() && !sys_platform_module {
                    return Err(err(
                        "host.profile",
                        "platform call requires host-console profile".into(),
                    ));
                }
                let h = self
                    .module
                    .host_imports
                    .get(*host as usize)
                    .ok_or_else(|| err("host.reference", "platform index out of range".into()))?;
                if !crate::platform_io::is_platform_symbol(&h.symbol)
                    && !crate::config_document::is_config_symbol(&h.symbol)
                {
                    return Err(err(
                        "host.category",
                        "PlatformCall must target a platform or configuration operation".into(),
                    ));
                }
                let sig = self
                    .module
                    .sig_of(h.sig)
                    .ok_or_else(|| err("host.signature", "invalid platform signature".into()))?;
                if args.len() != sig.params.len() {
                    return Err(err(
                        "host.signature",
                        "platform operand count mismatch".into(),
                    ));
                }
                for (arg, param_ty) in args.iter().zip(sig.params.iter()) {
                    let at = slot_ty_of(f, *arg, sp, "platform arg")?;
                    if !self.ty_equiv(at, *param_ty) {
                        return Err(err(
                            "host.signature",
                            "platform operand type mismatch".into(),
                        ));
                    }
                }
                let dt = slot_ty_of(f, *dst, sp, "platform dst")?;
                if !self.ty_equiv(dt, sig.result) {
                    return Err(err(
                        "host.signature",
                        "platform result type mismatch".into(),
                    ));
                }
            }
            MirInstr::Trace { .. } => {}
            MirInstr::FieldGet { dst, base, path } => {
                if !self.is_data && !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "FIELD_GET exists only in the data/owned profiles".to_string(),
                    ));
                }
                self.check_field_path(f, *base, path, sp, "field-get")?;
                let dt = slot_ty_of(f, *dst, sp, "field-get dst")?;
                let bt = slot_ty_of(f, *base, sp, "field-get base")?;
                let Some((final_ty, _owner)) = self.module.resolve_field_path(bt, path) else {
                    return Err(err(
                        "mir.field-path",
                        format!(
                            "field path {:?} does not resolve from base type {}",
                            path, bt
                        ),
                    ));
                };
                if self.is_owned {
                    // PLAN-821: a Str field read is a BORROW VIEW
                    // rooted at the base place (782: 字段读取=视图);
                    // a resource-record field never implicitly
                    // copies its owner out.
                    match self.module.types.get(final_ty as usize) {
                        Some(MirType::Str) => {
                            if !matches!(self.module.types.get(dt as usize), Some(MirType::StrView))
                            {
                                return Err(err(
                                            "mir.field-type",
                                            format!(
                                                "field-get of a Str field must land in a StrView slot (dst {})",
                                                dt
                                            ),
                                        ));
                            }
                        }
                        Some(t) if crate::ownership::type_is_owned(self.module, final_ty) => {
                            let _ = t;
                            return Err(err(
                                        "mir.ownership-required",
                                        format!(
                                            "field-get of resource-record field type {} never copies the owner (views only)",
                                            final_ty
                                        ),
                                    ));
                        }
                        _ => {
                            if !self.ty_equiv(dt, final_ty) {
                                return Err(err(
                                    "mir.field-type",
                                    format!(
                                        "field-get dst type {} does not match path final type {}",
                                        dt, final_ty
                                    ),
                                ));
                            }
                        }
                    }
                } else if !self.ty_equiv(dt, final_ty) {
                    return Err(err(
                        "mir.field-type",
                        format!(
                            "field-get dst type {} does not match path final type {}",
                            dt, final_ty
                        ),
                    ));
                }
            }
            MirInstr::FieldSet { root, path, value } => {
                if !self.is_data && !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "FIELD_SET exists only in the data/owned profiles".to_string(),
                    ));
                }
                self.check_field_path(f, *root, path, sp, "field-set")?;
                let rt_slot = slot_ty_of(f, *root, sp, "field-set root")?;
                let Some((final_ty, _owner)) = self.module.resolve_field_path(rt_slot, path) else {
                    return Err(err(
                        "mir.field-path",
                        format!(
                            "field path {:?} does not resolve from root type {}",
                            path, rt_slot
                        ),
                    ));
                };
                let vt = slot_ty_of(f, *value, sp, "field-set value")?;
                if !self.ty_equiv(vt, final_ty) {
                    return Err(err(
                        "mir.field-type",
                        format!(
                            "field-set value type {} does not match path final type {}",
                            vt, final_ty
                        ),
                    ));
                }
                // The root must be a place the frame owns across the
                // write: a named local or a param (temps are lowered
                // away by the adapter; a hand-made unit writing a
                // temp root is legal-by-shape but pointless — still
                // admitted, the rebuild is well-defined).
            }
            MirInstr::Aggregate {
                dst,
                record_ty,
                operands,
            } => {
                if !self.is_data && !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "AGGREGATE exists only in the data/owned profiles".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "aggregate dst")?;
                let want_fields = match self.module.types.get(*record_ty as usize) {
                    Some(MirType::Record(r)) => r,
                    _ => {
                        return Err(err(
                            "mir.aggregate-type",
                            format!("aggregate references non-record type {}", record_ty),
                        ))
                    }
                };
                if !self.ty_equiv(dt, *record_ty) {
                    return Err(err(
                        "mir.aggregate-type",
                        format!(
                            "aggregate dst type {} does not match record type {}",
                            dt, record_ty
                        ),
                    ));
                }
                if operands.len() != want_fields.fields.len() {
                    return Err(err(
                        "mir.aggregate-arity",
                        format!(
                            "aggregate passes {} operands, record declares {} fields",
                            operands.len(),
                            want_fields.fields.len()
                        ),
                    ));
                }
                for (fi_, (op_slot, field)) in
                    operands.iter().zip(want_fields.fields.iter()).enumerate()
                {
                    let ot = slot_ty_of(f, *op_slot, sp, &format!("aggregate operand #{}", fi_))?;
                    if self.is_owned
                        && matches!(
                            self.module.types.get(ot as usize),
                            Some(MirType::StrView) | Some(MirType::MutRef)
                        )
                    {
                        return Err(err(
                            "mir.type-op",
                            format!(
                                "aggregate operand #{} is a borrow kind; fields own their values",
                                fi_
                            ),
                        ));
                    }
                    if !self.ty_equiv(ot, field.ty) {
                        return Err(err(
                            "mir.aggregate-field-type",
                            format!(
                                "aggregate operand #{} has type {}, field `{}` wants {}",
                                fi_, ot, field.name, field.ty
                            ),
                        ));
                    }
                }
            }
            // -- PLAN-821 owned surface (av.owned/1) -------------------
            MirInstr::Clone { dst, src } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "CLONE exists only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "clone dst")?;
                let st = slot_ty_of(f, *src, sp, "clone src")?;
                // D-981-GLOBAL-CLONE-GAP closure: CLONE is also the unique
                // view→owned crossing — a `RecordView(r)` source deep-clones
                // (for a scalar record: value-copies) into a fresh owned
                // `Record(r)` of the SAME nominal record. COPY cannot cross a
                // view (`mir.copy-type`), so this form carries resource and
                // scalar records alike; every other mismatch stays
                // `mir.clone-type`.
                let view_to_owned = match (
                    self.module.types.get(dt as usize),
                    self.module.types.get(st as usize),
                ) {
                    (Some(MirType::Record(want)), Some(MirType::RecordView(got))) => want == got,
                    _ => false,
                };
                if !view_to_owned && !self.ty_equiv(dt, st) {
                    return Err(err(
                        "mir.clone-type",
                        format!("clone from type {} into type {}", st, dt),
                    ));
                }
                // Clone is the EXPLICIT deep copy of owned data;
                // cloning a copyable is a plain COPY. PLAN-823: a
                // Closure is deep-cloneable too (the env's fields
                // deep-clone under a fresh store handle). The view→owned
                // form is exempt: the crossing is only expressible via
                // CLONE, and for a scalar record the deep clone degenerates
                // to a value copy.
                let is_closure_ty = matches!(
                    self.module.types.get(st as usize),
                    Some(MirType::Closure(_))
                );
                if !view_to_owned
                    && !is_closure_ty
                    && !crate::ownership::type_is_owned(self.module, st)
                {
                    return Err(err(
                        "mir.ownership-required",
                        format!("CLONE of copyable type {} (use COPY)", st),
                    ));
                }
            }
            MirInstr::Move { dst, src } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "MOVE exists only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "move dst")?;
                let st = slot_ty_of(f, *src, sp, "move src")?;
                if !self.ty_equiv(dt, st) {
                    return Err(err(
                        "mir.move-type",
                        format!("move from type {} into type {}", st, dt),
                    ));
                }
                if !crate::ownership::type_is_owned(self.module, st) {
                    return Err(err(
                        "mir.ownership-required",
                        format!(
                            "MOVE of copyable type {} (ownership transfer is for owned data)",
                            st
                        ),
                    ));
                }
            }
            MirInstr::View { dst, src } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "VIEW exists only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "view dst")?;
                let st = slot_ty_of(f, *src, sp, "view src")?;
                if !matches!(self.module.types.get(st as usize), Some(MirType::Str)) {
                    return Err(err(
                        "mir.type-op",
                        format!("VIEW source type {} is not an owned Str slot", st),
                    ));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::StrView)) {
                    return Err(err(
                        "mir.type-op",
                        format!("VIEW destination type {} is not StrView", dt),
                    ));
                }
            }
            MirInstr::MutRef { dst, src } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "MUTREF exists only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "mutref dst")?;
                let st = slot_ty_of(f, *src, sp, "mutref src")?;
                if !matches!(self.module.types.get(dt as usize), Some(MirType::MutRef)) {
                    return Err(err(
                        "mir.type-op",
                        format!("MUTREF destination type {} is not MutRef", dt),
                    ));
                }
                if !crate::ownership::type_is_rootable(self.module, st) {
                    return Err(err(
                        "mir.type-op",
                        format!(
                            "MUTREF source type {} is not an ownable root (Str / resource record)",
                            st
                        ),
                    ));
                }
            }
            MirInstr::Drop { slot } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "DROP exists only in the owned profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *slot, sp, "drop slot")?;
                // PLAN-823: a Closure slot drops too — the release
                // path is the env-store release (its fields own the
                // resources).
                let is_closure_ty = matches!(
                    self.module.types.get(st as usize),
                    Some(MirType::Closure(_))
                );
                if !is_closure_ty && !crate::ownership::type_is_owned(self.module, st) {
                    return Err(err(
                        "mir.type-op",
                        format!(
                            "DROP of type {} (only owned Str / resource records drop)",
                            st
                        ),
                    ));
                }
            }
            MirInstr::FieldMove { dst, root, path } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "FIELDMOVE exists only in the owned profile".to_string(),
                    ));
                }
                self.check_field_path(f, *root, path, sp, "field-move")?;
                let dt = slot_ty_of(f, *dst, sp, "field-move dst")?;
                let rt = slot_ty_of(f, *root, sp, "field-move root")?;
                let Some((final_ty, _owner)) = self.module.resolve_field_path(rt, path) else {
                    return Err(err(
                        "mir.field-path",
                        format!(
                            "field path {:?} does not resolve from root type {}",
                            path, rt
                        ),
                    ));
                };
                // The extracted field must be OWNED (a resource
                // record — Str fields ride FIELD_GET views; the
                // write-back wrapper only carries records).
                if !crate::ownership::type_is_owned(self.module, final_ty) {
                    return Err(err(
                        "mir.ownership-required",
                        format!(
                            "field-move of non-owned field type {} (use FIELD_GET)",
                            final_ty
                        ),
                    ));
                }
                if !self.ty_equiv(dt, final_ty) {
                    return Err(err(
                        "mir.field-type",
                        format!("field-move dst {} != field type {}", dt, final_ty),
                    ));
                }
            }
            // -- PLAN-822 collections surface (typing arms) --------
            MirInstr::ListNew { dst, list_ty, cap } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_NEW exists only in the collections profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "list-new dst")?;
                if !self.ty_equiv(dt, *list_ty) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-new dst {} != declared list type {}", dt, list_ty),
                    ));
                }
                if !matches!(
                    self.module.types.get(*list_ty as usize),
                    Some(MirType::List(_))
                ) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-new type {} is not a List type", list_ty),
                    ));
                }
                if *cap as usize > crate::contract::MAX_LIST_ELEMS {
                    return Err(MirVerifyError::Resource {
                        kind: ResourceKind::ListElems,
                        message: format!(
                            "list-new capacity {} above the frozen guard {}",
                            cap,
                            crate::contract::MAX_LIST_ELEMS
                        ),
                    });
                }
            }
            MirInstr::ListLen { dst, src } | MirInstr::ListCap { dst, src } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_LEN/LIST_CAP exist only in the collections profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "list-op source")?;
                if !matches!(self.module.types.get(st as usize), Some(MirType::List(_))) {
                    return Err(err(
                        "mir.list-type",
                        format!("slot type {} is not a List", st),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "list-op dst")?;
                if !matches!(self.module.types.get(dt as usize), Some(MirType::I32)) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-op dst {} is not i32", dt),
                    ));
                }
            }
            MirInstr::ListGet { dst, base, index } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_GET exists only in the collections profile".to_string(),
                    ));
                }
                let elem = self.list_elem_of(f, *base, sp)?;
                self.check_list_index_slot(f, *index, sp)?;
                let dt = slot_ty_of(f, *dst, sp, "list-get dst")?;
                if !self.ty_equiv(dt, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-get dst {} != element type {}", dt, elem),
                    ));
                }
                if crate::ownership::type_is_owned(self.module, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!(
                            "list-get of owned element type {} (use LIST_REMOVE / LIST_FIELD_*)",
                            elem
                        ),
                    ));
                }
            }
            MirInstr::ListPush { root, value } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_PUSH exists only in the collections profile".to_string(),
                    ));
                }
                let elem = self.list_elem_of(f, *root, sp)?;
                let vt = slot_ty_of(f, *value, sp, "list-push value")?;
                if !self.ty_equiv(vt, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-push value {} != element type {}", vt, elem),
                    ));
                }
            }
            MirInstr::ListInsert { root, index, value }
            | MirInstr::ListSet { root, index, value } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_INSERT/LIST_SET exist only in the collections profile".to_string(),
                    ));
                }
                let elem = self.list_elem_of(f, *root, sp)?;
                self.check_list_index_slot(f, *index, sp)?;
                let vt = slot_ty_of(f, *value, sp, "list-op value")?;
                if !self.ty_equiv(vt, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-op value {} != element type {}", vt, elem),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// F-941-15 cold half B of the per-instruction typing walk (union frame halved; see check_function).
    fn object_elem_of(
        &self,
        f: &MirFunction,
        slot: SlotId,
        span: Option<SpanInfo>,
    ) -> Result<TypeIdx, MirVerifyError> {
        let ty = slot_ty_of(f, slot, span, "object")?;
        match self.module.types.get(ty as usize) {
            Some(MirType::Object { elem }) if self.collections_surface() => Ok(*elem),
            Some(MirType::Dyn) if self.language_values() => Ok(ty),
            _ => Err(MirVerifyError::Invalid {
                code: "mir.object-type".into(),
                message: "object instruction requires an object slot".into(),
                span,
            }),
        }
    }

    fn check_object_key(
        &self,
        f: &MirFunction,
        slot: SlotId,
        view_allowed: bool,
        span: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let ty = slot_ty_of(f, slot, span, "object key")?;
        match self.module.types.get(ty as usize) {
            Some(MirType::Str | MirType::I32 | MirType::Bool) => Ok(()),
            Some(MirType::StrView) if view_allowed => Ok(()),
            _ => Err(MirVerifyError::Invalid {
                code: "mir.object-key".into(),
                message: "object key requires string, i32 or bool".into(),
                span,
            }),
        }
    }

    #[inline(never)]
    fn check_instr_typing_b(
        &mut self,
        f: &MirFunction,
        instr: &MirInstr,
        sp: Option<SpanInfo>,
    ) -> Result<(), MirVerifyError> {
        let err = |code: &str, msg: String| MirVerifyError::Invalid {
            code: code.to_string(),
            message: msg,
            span: sp,
        };
        match instr {
            MirInstr::ListRemove { dst, root, index } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_REMOVE exists only in the collections profile".to_string(),
                    ));
                }
                let elem = self.list_elem_of(f, *root, sp)?;
                self.check_list_index_slot(f, *index, sp)?;
                let dt = slot_ty_of(f, *dst, sp, "list-remove dst")?;
                if !self.ty_equiv(dt, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!("list-remove dst {} != element type {}", dt, elem),
                    ));
                }
            }
            MirInstr::ListFieldGet {
                dst,
                base,
                index,
                path,
            } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_FIELD_GET exists only in the collections profile".to_string(),
                    ));
                }
                let elem = self.list_elem_of(f, *base, sp)?;
                self.check_list_index_slot(f, *index, sp)?;
                let Some((final_ty, _owner)) = self.module.resolve_field_path(elem, path) else {
                    return Err(err(
                        "mir.field-path",
                        format!(
                            "field path {:?} does not resolve from element type {}",
                            path, elem
                        ),
                    ));
                };
                let dt = slot_ty_of(f, *dst, sp, "list-field-get dst")?;
                if !self.ty_equiv(dt, final_ty) {
                    return Err(err(
                        "mir.field-type",
                        format!("list-field-get dst {} != field type {}", dt, final_ty),
                    ));
                }
                if crate::ownership::type_is_owned(self.module, final_ty) {
                    return Err(err(
                                "mir.field-type",
                                format!(
                                    "list-field-get of owned field type {} (fields ride records; use LIST_REMOVE first)",
                                    final_ty
                                ),
                            ));
                }
            }
            MirInstr::ListFieldSet {
                root,
                index,
                path,
                value,
            } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_FIELD_SET exists only in the collections profile".to_string(),
                    ));
                }
                let elem = self.list_elem_of(f, *root, sp)?;
                self.check_list_index_slot(f, *index, sp)?;
                let Some((final_ty, _owner)) = self.module.resolve_field_path(elem, path) else {
                    return Err(err(
                        "mir.field-path",
                        format!(
                            "field path {:?} does not resolve from element type {}",
                            path, elem
                        ),
                    ));
                };
                let vt = slot_ty_of(f, *value, sp, "list-field-set value")?;
                if !self.ty_equiv(vt, final_ty) {
                    return Err(err(
                        "mir.field-type",
                        format!("list-field-set value {} != field type {}", vt, final_ty),
                    ));
                }
            }
            MirInstr::VariantPack {
                dst,
                variant_ty,
                disc,
                payload,
            } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "VARIANT_PACK exists only in the collections profile".to_string(),
                    ));
                }
                let vt = slot_ty_of(f, *dst, sp, "variant-pack dst")?;
                if !self.ty_equiv(vt, *variant_ty) {
                    return Err(err(
                        "mir.variant-type",
                        format!(
                            "variant-pack dst {} != declared variant type {}",
                            vt, variant_ty
                        ),
                    ));
                }
                self.check_variant_arm(f, *variant_ty, *disc, payload.as_ref(), sp)?;
            }
            MirInstr::VariantDisc { dst, src } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "VARIANT_DISC exists only in the collections profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "variant source")?;
                if !matches!(
                    self.module.types.get(st as usize),
                    Some(MirType::Variant(_))
                ) {
                    return Err(err(
                        "mir.variant-type",
                        format!("slot type {} is not a Variant", st),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "variant-disc dst")?;
                if !matches!(self.module.types.get(dt as usize), Some(MirType::I32)) {
                    return Err(err(
                        "mir.variant-type",
                        format!("variant-disc dst {} is not i32", dt),
                    ));
                }
            }
            MirInstr::VariantGuard { dst, src, disc } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "VARIANT_GUARD exists only in the collections profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "variant source")?;
                let Some(MirType::Variant(v)) = self.module.types.get(st as usize) else {
                    return Err(err(
                        "mir.variant-type",
                        format!("slot type {} is not a Variant", st),
                    ));
                };
                if *disc as usize >= v.arms.len() {
                    return Err(err(
                        "mir.variant.arm-mismatch",
                        format!("guard discriminant {} out of range", disc),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "variant-guard dst")?;
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Bool)) {
                    return Err(err(
                        "mir.variant-type",
                        format!("variant-guard dst {} is not bool", dt),
                    ));
                }
            }
            MirInstr::VariantProject { dst, root, disc } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "VARIANT_PROJECT exists only in the collections profile".to_string(),
                    ));
                }
                let rt = slot_ty_of(f, *root, sp, "variant root")?;
                let Some(MirType::Variant(v)) = self.module.types.get(rt as usize) else {
                    return Err(err(
                        "mir.variant-type",
                        format!("slot type {} is not a Variant", rt),
                    ));
                };
                let Some(arm) = v.arms.get(*disc as usize) else {
                    return Err(err(
                        "mir.variant.arm-mismatch",
                        format!("project discriminant {} out of range", disc),
                    ));
                };
                let Some(pt) = arm.payload else {
                    return Err(err(
                        "mir.variant.arm-mismatch",
                        format!("project of no-payload arm {}", disc),
                    ));
                };
                let dt = slot_ty_of(f, *dst, sp, "variant-project dst")?;
                if !self.ty_equiv(dt, pt) {
                    return Err(err(
                        "mir.variant-type",
                        format!("variant-project dst {} != payload type {}", dt, pt),
                    ));
                }
            }
            MirInstr::VariantSwitch {
                root,
                disc,
                payload,
            } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "VARIANT_SWITCH exists only in the collections profile".to_string(),
                    ));
                }
                let rt = slot_ty_of(f, *root, sp, "variant root")?;
                if !matches!(
                    self.module.types.get(rt as usize),
                    Some(MirType::Variant(_))
                ) {
                    return Err(err(
                        "mir.variant-type",
                        format!("slot type {} is not a Variant", rt),
                    ));
                }
                self.check_variant_arm(f, rt, *disc, payload.as_ref(), sp)?;
            }
            // -- PLAN-822 R2 index-place surface (typing arms) -----
            MirInstr::ListPlace { dst, base, index } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "LIST_PLACE exists only in the collections profile".to_string(),
                    ));
                }
                // F822-R3-01: the referent element type BINDS at
                // the mint — dst must be Place{elem == base's
                // element}. A Place over a different element is a
                // typed-admission rejection.
                let elem = self.list_elem_of(f, *base, sp)?;
                self.check_list_index_slot(f, *index, sp)?;
                let dt = slot_ty_of(f, *dst, sp, "list-place dst")?;
                match self.module.types.get(dt as usize) {
                    Some(MirType::Place(want)) => {
                        if want.elem != elem {
                            return Err(err(
                                        "mir.list-type",
                                        format!(
                                            "list-place dst Place refers to element {}, but the base list holds {}",
                                            want.elem, elem
                                        ),
                                    ));
                        }
                    }
                    _ => {
                        return Err(err(
                            "mir.list-type",
                            format!("list-place dst {} is not a Place type", dt),
                        ))
                    }
                }
            }
            MirInstr::PlaceGet { dst, src } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "PLACE_GET exists only in the collections profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "place source")?;
                let elem = match self.module.types.get(st as usize) {
                    Some(MirType::Place(want)) => want.elem,
                    _ => {
                        return Err(err(
                            "mir.list-type",
                            format!("place-get source {} is not a Place type", st),
                        ))
                    }
                };
                // F822-R3-01: the destination must BE the bound
                // element type — a bool destination over a
                // List<i32> place is rejected at typing.
                let dt = slot_ty_of(f, *dst, sp, "place-get dst")?;
                if !self.ty_equiv(dt, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!(
                            "place-get dst {} != the place's bound element type {}",
                            dt, elem
                        ),
                    ));
                }
                if crate::ownership::type_is_owned(self.module, elem) {
                    // PLAN-962: the aggregates surface reads a whole owned
                    // element through a place as a BORROW — admitted ONLY
                    // when the destination is a compiler TEMP: temps are
                    // never part of a facts free schedule, so the alias
                    // cannot double-free; the place's epoch revalidation
                    // traps any stale use. A named-local destination keeps
                    // the frozen rejection (mutated-bundle safety).
                    let dst_temp = f
                        .slots
                        .get(*dst as usize)
                        .is_some_and(|s| s.kind == crate::mir::SlotKind::Temp);
                    if !(self.aggregates_admission() && dst_temp) {
                        return Err(err(
                                    "mir.list-type",
                                    format!(
                                        "place-get of owned element type {} (places read Copy elements; use LIST_REMOVE)",
                                        elem
                                    ),
                                ));
                    }
                }
            }
            MirInstr::PlaceSet { place, value } => {
                if !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "PLACE_SET exists only in the collections profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *place, sp, "place source")?;
                let elem = match self.module.types.get(st as usize) {
                    Some(MirType::Place(want)) => want.elem,
                    _ => {
                        return Err(err(
                            "mir.list-type",
                            format!("place-set source {} is not a Place type", st),
                        ))
                    }
                };
                // F822-R3-01: the value must BE the bound element
                // type — writing a bool into a List<i32> through a
                // place is rejected at typing, before any access.
                let vt = slot_ty_of(f, *value, sp, "place-set value")?;
                if !self.ty_equiv(vt, elem) {
                    return Err(err(
                        "mir.list-type",
                        format!(
                            "place-set value {} != the place's bound element type {}",
                            vt, elem
                        ),
                    ));
                }
            }
            // -- PLAN-823 closure surface (av.closures/1) ----------
            MirInstr::CreateClosure {
                dst,
                callee,
                captures,
            } => {
                if !self.is_closures {
                    return Err(err(
                        "mir.profile-instr",
                        "CREATE_CLOSURE exists only in the closures profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "create-closure dst")?;
                let callee_sig: crate::mir::FnSig = match callee {
                    crate::mir::CalleeRef::Func(idx) => {
                        let target = match self.module.funcs.get(*idx as usize) {
                            Some(t) => t,
                            None => {
                                return Err(err(
                                    "mir.callee-ref",
                                    format!(
                                        "create-closure references out-of-range \
                                                 function {}",
                                        idx
                                    ),
                                ))
                            }
                        };
                        match self.module.sig_of(target.sig) {
                            Some(s) => s.clone(),
                            None => {
                                return Err(err(
                                    "mir.callee-sig",
                                    format!("closure callee #{} has no function type", idx),
                                ))
                            }
                        }
                    }
                    crate::mir::CalleeRef::Import(idx) => {
                        let imp = match self.module.imports.get(*idx as usize) {
                            Some(t) => t,
                            None => {
                                return Err(err(
                                    "mir.import-ref",
                                    format!(
                                        "create-closure references out-of-range \
                                                 import {}",
                                        idx
                                    ),
                                ))
                            }
                        };
                        match self.module.sig_of(imp.sig) {
                            Some(s) => s.clone(),
                            None => {
                                return Err(err(
                                    "mir.import-sig",
                                    format!("closure import #{} has no function type", idx),
                                ))
                            }
                        }
                    }
                };
                // dst must be EXACTLY the callee's Closure type: an
                // Fn slot never accepts an arbitrary closure
                // (§5.1.1) — the bound signature is structural.
                match self.module.types.get(dt as usize) {
                    Some(MirType::Closure(s)) if s == &callee_sig => {}
                    other => {
                        return Err(err(
                            "mir.closure.type",
                            format!(
                            "create-closure dst type {} is {:?}, must be the callee's Closure type",
                            dt, other
                        ),
                        ))
                    }
                }
                for (ci, c) in captures.iter().enumerate() {
                    if c.kind == crate::mir::CaptureKind::MoveFromEnv {
                        // R1-F01: src is a FIELD index of the
                        // CURRENT frame's env — typed against the
                        // parent's own declared layout by the
                        // pre-scan and the alias/moved gates.
                        if f.env.is_none() {
                            return Err(err(
                                        "mir.capture.env-field",
                                        format!(
                                            "capture #{} moves an env field but `{}` declares no env layout",
                                            ci, f.name
                                        ),
                                    ));
                        }
                        if c.src as usize >= f.env.as_ref().unwrap().len() {
                            return Err(err(
                                "mir.capture.env-field",
                                format!(
                                    "capture #{} env field {} out of range ({} declared)",
                                    ci,
                                    c.src,
                                    f.env.as_ref().unwrap().len()
                                ),
                            ));
                        }
                        continue;
                    }
                    let st = slot_ty_of(f, c.src, sp, &format!("capture #{}", ci))?;
                    if c.kind == crate::mir::CaptureKind::Borrow {
                        // The delivered borrow domain: an owned Str
                        // source; the env field is a share view.
                        if !matches!(self.module.types.get(st as usize), Some(MirType::Str)) {
                            return Err(err(
                                "mir.capture.borrow-domain",
                                format!("borrow capture #{} type {} is not an owned Str", ci, st),
                            ));
                        }
                        continue;
                    }
                    if c.kind == crate::mir::CaptureKind::Shared {
                        // PLAN-909 cell domain: a handle share is
                        // admitted ONLY over the cell representation
                        // — `List<T>` with a Copy scalar element
                        // (i32/bool/char). Sharing any other owned
                        // aggregate (Str, record, nested list,
                        // variant) is not a cell contract and is
                        // rejected by name; an ordinary list copy
                        // never stands in for a cell.
                        let cell_ok = match self.module.types.get(st as usize) {
                            Some(MirType::List(l)) => matches!(
                                self.module.types.get(l.elem as usize),
                                Some(MirType::I32 | MirType::Bool | MirType::Char)
                            ),
                            _ => false,
                        };
                        if !cell_ok {
                            return Err(err(
                                "mir.capture.shared-domain",
                                format!(
                                    "shared capture #{} type {} is not a scalar cell \
                                             (List<i32|bool|char>); shared captures exist only \
                                             over the single-element cell representation",
                                    ci, st
                                ),
                            ));
                        }
                        continue;
                    }
                    if matches!(
                        self.module.types.get(st as usize),
                        Some(MirType::StrView)
                            | Some(MirType::MutRef)
                            | Some(MirType::Place(_))
                            | Some(MirType::Closure(_))
                            | Some(MirType::Fn(_))
                    ) {
                        return Err(err(
                            "mir.capture.kind",
                            format!(
                                "capture #{} type {} is not escape-capturable \
                                         (borrow/token/place/closure kinds are rejected)",
                                ci, st
                            ),
                        ));
                    }
                }
            }
            MirInstr::CallClosure { dst, closure, args } => {
                if !self.is_closures {
                    return Err(err(
                        "mir.profile-instr",
                        "CALL_CLOSURE exists only in the closures profile".to_string(),
                    ));
                }
                let ct = slot_ty_of(f, *closure, sp, "call-closure operand")?;
                let sig = match self.module.types.get(ct as usize) {
                    Some(MirType::Closure(s)) => s.clone(),
                    other => {
                        return Err(err(
                            "mir.closure.type",
                            format!(
                                "call-closure operand type {} is {:?}, not a Closure",
                                ct, other
                            ),
                        ))
                    }
                };
                if sig.params.len() != args.len() {
                    return Err(err(
                        "mir.closure.arity",
                        format!(
                            "closure call passes {} args, signature takes {}",
                            args.len(),
                            sig.params.len()
                        ),
                    ));
                }
                for (ai, (a, want)) in args.iter().zip(sig.params.iter()).enumerate() {
                    let at = slot_ty_of(f, *a, sp, &format!("closure arg #{}", ai))?;
                    if !self.ty_equiv(at, *want) {
                        return Err(err(
                            "mir.closure.arg-type",
                            format!(
                                "closure arg #{} has type {}, signature wants {}",
                                ai, at, want
                            ),
                        ));
                    }
                }
                let dt = slot_ty_of(f, *dst, sp, "call-closure dst")?;
                if !self.ty_equiv(dt, sig.result) {
                    return Err(err(
                        "mir.closure.result-type",
                        format!(
                            "closure call dst type {} does not match signature result {}",
                            dt, sig.result
                        ),
                    ));
                }
            }
            MirInstr::EnvGet { dst, field } => {
                if !self.is_closures {
                    return Err(err(
                        "mir.profile-instr",
                        "ENV_GET exists only in the closures profile".to_string(),
                    ));
                }
                // r8: the env layout is the function's OWN
                // declaration — a body unit verifies standalone.
                let layout = match &f.env {
                    Some(l) => l,
                    None => {
                        return Err(err(
                            "mir.env.not-closure-body",
                            format!(
                                "ENV_GET in `{}` which declares no env layout \
                                         (env reads live only in closure bodies)",
                                f.name
                            ),
                        ))
                    }
                };
                if *field as usize >= layout.len() {
                    return Err(err(
                        "mir.env.field-range",
                        format!(
                            "env field {} out of range (env layout has {} captures)",
                            field,
                            layout.len()
                        ),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "env-get dst")?;
                let want = layout[*field as usize];
                if !self.ty_equiv(dt, want) {
                    return Err(err(
                        "mir.env.field-type",
                        format!(
                            "env-get dst type {} does not match captured field type {}",
                            dt, want
                        ),
                    ));
                }
            }
            MirInstr::ObjectNew { dst, keys, vals } => {
                let elem = self.object_elem_of(f, *dst, sp)?;
                if keys.len() != vals.len() {
                    return Err(err(
                        "mir.object-arity",
                        "object key and value counts differ".into(),
                    ));
                }
                for key in keys {
                    self.check_object_key(f, *key, false, sp)?;
                }
                for val in vals {
                    let ty = slot_ty_of(f, *val, sp, "object value")?;
                    if !self.ty_equiv(ty, elem) {
                        return Err(err(
                            "mir.object-type",
                            "object value type does not match the element type".into(),
                        ));
                    }
                }
            }
            MirInstr::ObjectGet {
                dst,
                obj,
                key,
                elem,
            } => {
                let declared = self.object_elem_of(f, *obj, sp)?;
                self.check_object_key(f, *key, true, sp)?;
                let result = slot_ty_of(f, *dst, sp, "object result")?;
                if !self.ty_equiv(*elem, declared) || !self.ty_equiv(result, declared) {
                    return Err(err(
                        "mir.object-type",
                        "object read type does not match the element type".into(),
                    ));
                }
            }
            MirInstr::ObjectSet { obj, key, value } => {
                let elem = self.object_elem_of(f, *obj, sp)?;
                self.check_object_key(f, *key, true, sp)?;
                let ty = slot_ty_of(f, *value, sp, "object value")?;
                if !self.ty_equiv(ty, elem) {
                    return Err(err(
                        "mir.object-type",
                        "object value type does not match the element type".into(),
                    ));
                }
            }
            // -- PLAN-824 dynamic surface (typing arms) ----------
            MirInstr::DynPack { dst, src } => {
                if !self.dynamic_values() {
                    return Err(err(
                        "mir.profile-instr",
                        "DYN_PACK exists only in the dynamic profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "dyn-pack dst")?;
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Dyn)) {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_PACK dst {} is not the Dyn type", dt),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "dyn-pack src")?;
                if !self.dyn_whitelisted(st) {
                    return Err(err(
                        "mir.dyn.whitelist",
                        format!(
                            "DYN_PACK src type {} is outside the erasable closure \
                                     (Fn/borrows/Places rejected at any depth)",
                            st
                        ),
                    ));
                }
            }
            MirInstr::DynIs { dst, src, ty } => {
                if !self.dynamic_values() {
                    return Err(err(
                        "mir.profile-instr",
                        "DYN_IS exists only in the dynamic profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "dyn-is dst")?;
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Bool)) {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_IS dst {} is not bool", dt),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "dyn-is src")?;
                if !matches!(self.module.types.get(st as usize), Some(MirType::Dyn)) {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_IS src {} is not the Dyn type", st),
                    ));
                }
                if self.module.types.get(*ty as usize).is_none() {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_IS type operand {} is out of bounds", ty),
                    ));
                }
                if !self.dyn_whitelisted(*ty) {
                    return Err(err(
                        "mir.dyn.whitelist",
                        format!("DYN_IS type operand {} is outside the erasable closure", ty),
                    ));
                }
            }
            MirInstr::DynTake { dst, src, ty } => {
                if !self.dynamic_values() {
                    return Err(err(
                        "mir.profile-instr",
                        "DYN_TAKE exists only in the dynamic profile".to_string(),
                    ));
                }
                let st = slot_ty_of(f, *src, sp, "dyn-take src")?;
                if !matches!(self.module.types.get(st as usize), Some(MirType::Dyn)) {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_TAKE src {} is not the Dyn type", st),
                    ));
                }
                if self.module.types.get(*ty as usize).is_none() {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_TAKE type operand {} is out of bounds", ty),
                    ));
                }
                if !self.dyn_whitelisted(*ty) {
                    return Err(err(
                        "mir.dyn.whitelist",
                        format!(
                            "DYN_TAKE type operand {} is outside the erasable closure",
                            ty
                        ),
                    ));
                }
                // dst receives exactly the requested type.
                let dt = slot_ty_of(f, *dst, sp, "dyn-take dst")?;
                if !self.ty_equiv(dt, *ty) {
                    return Err(err(
                        "mir.dyn.type",
                        format!("DYN_TAKE dst {} != the requested type {}", dt, ty),
                    ));
                }
            }
            MirInstr::BorrowEnd { token } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "BORROW_END exists only in the owned profile".to_string(),
                    ));
                }
                let tt = slot_ty_of(f, *token, sp, "borrow-end token")?;
                if !matches!(
                    self.module.types.get(tt as usize),
                    Some(MirType::StrView) | Some(MirType::MutRef)
                ) {
                    return Err(err(
                        "mir.type-op",
                        format!("BORROW_END operand type {} is not a borrow token", tt),
                    ));
                }
            }
            MirInstr::StrLen { dst, view } | MirInstr::StrLenBytes { dst, view } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "str-op dst")?;
                let vt = slot_ty_of(f, *view, sp, "str-op view")?;
                self.want_strview_i32(vt, dt, &err, sp)?;
            }
            MirInstr::StrCharAt { dst, view, index } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "char_at dst")?;
                let vt = slot_ty_of(f, *view, sp, "char_at view")?;
                let it = slot_ty_of(f, *index, sp, "char_at index")?;
                if !matches!(self.module.types.get(vt as usize), Some(MirType::StrView)) {
                    return Err(err(
                        "mir.type-op",
                        format!("char_at operand {} is not a view", vt),
                    ));
                }
                if self.module.type_shape(it) != Some(ScalarShape::I32) {
                    return Err(err(
                        "mir.type-op",
                        format!("char_at index type {} is not i32", it),
                    ));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Char)) {
                    return Err(err(
                        "mir.type-op",
                        format!("char_at dst type {} is not char", dt),
                    ));
                }
            }
            MirInstr::StrSlice {
                dst,
                view,
                start,
                end,
            } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "slice dst")?;
                let vt = slot_ty_of(f, *view, sp, "slice view")?;
                let stt = slot_ty_of(f, *start, sp, "slice start")?;
                let ent = slot_ty_of(f, *end, sp, "slice end")?;
                if !matches!(self.module.types.get(vt as usize), Some(MirType::StrView)) {
                    return Err(err(
                        "mir.type-op",
                        format!("slice operand {} is not a view", vt),
                    ));
                }
                if self.module.type_shape(stt) != Some(ScalarShape::I32)
                    || self.module.type_shape(ent) != Some(ScalarShape::I32)
                {
                    return Err(err("mir.type-op", "slice bounds must be i32".to_string()));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::StrView)) {
                    return Err(err(
                        "mir.type-op",
                        format!("slice dst type {} is not StrView", dt),
                    ));
                }
            }
            MirInstr::StrConcat { dst, lhs, rhs } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "concat dst")?;
                let lt = slot_ty_of(f, *lhs, sp, "concat lhs")?;
                let rt = slot_ty_of(f, *rhs, sp, "concat rhs")?;
                if !matches!(self.module.types.get(lt as usize), Some(MirType::StrView))
                    || !matches!(self.module.types.get(rt as usize), Some(MirType::StrView))
                {
                    return Err(err(
                        "mir.type-op",
                        "concat operands must be views".to_string(),
                    ));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Str)) {
                    return Err(err(
                        "mir.type-op",
                        format!("concat dst type {} is not Str", dt),
                    ));
                }
            }
            MirInstr::StrToOwned { dst, view } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "to-owned dst")?;
                let vt = slot_ty_of(f, *view, sp, "to-owned view")?;
                if !matches!(self.module.types.get(vt as usize), Some(MirType::StrView)) {
                    return Err(err(
                        "mir.type-op",
                        "to-owned operand must be a view".to_string(),
                    ));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Str)) {
                    return Err(err(
                        "mir.type-op",
                        format!("to-owned dst type {} is not Str", dt),
                    ));
                }
            }
            MirInstr::StrFromChar { dst, ch } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "from-char dst")?;
                let ct = slot_ty_of(f, *ch, sp, "from-char char")?;
                if !matches!(self.module.types.get(ct as usize), Some(MirType::Char)) {
                    return Err(err(
                        "mir.type-op",
                        "from_char operand must be char".to_string(),
                    ));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Str)) {
                    return Err(err(
                        "mir.type-op",
                        format!("from_char dst type {} is not Str", dt),
                    ));
                }
            }
            MirInstr::StrCmp { dst, lhs, rhs, .. } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "str operations exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "cmp dst")?;
                let lt = slot_ty_of(f, *lhs, sp, "cmp lhs")?;
                let rt = slot_ty_of(f, *rhs, sp, "cmp rhs")?;
                if !matches!(self.module.types.get(lt as usize), Some(MirType::StrView))
                    || !matches!(self.module.types.get(rt as usize), Some(MirType::StrView))
                {
                    return Err(err("mir.type-op", "cmp operands must be views".to_string()));
                }
                if self.module.type_shape(dt) != Some(ScalarShape::Bool) {
                    return Err(err(
                        "mir.type-op",
                        format!("cmp dst type {} is not bool", dt),
                    ));
                }
            }
            MirInstr::CharToCodepoint { dst, ch } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "char conversions exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "to-codepoint dst")?;
                let ct = slot_ty_of(f, *ch, sp, "to-codepoint char")?;
                if !matches!(self.module.types.get(ct as usize), Some(MirType::Char)) {
                    return Err(err(
                        "mir.type-op",
                        "to_codepoint operand must be char".to_string(),
                    ));
                }
                if self.module.type_shape(dt) != Some(ScalarShape::I32) {
                    return Err(err(
                        "mir.type-op",
                        "to_codepoint dst must be i32".to_string(),
                    ));
                }
            }
            MirInstr::CharFromCodepoint { dst, code } => {
                if !self.is_owned && !self.collections_surface() {
                    return Err(err(
                        "mir.profile-instr",
                        "char conversions exist only in the owned profile".to_string(),
                    ));
                }
                let dt = slot_ty_of(f, *dst, sp, "from-codepoint dst")?;
                let ct = slot_ty_of(f, *code, sp, "from-codepoint code")?;
                if self.module.type_shape(ct) != Some(ScalarShape::I32) {
                    return Err(err(
                        "mir.type-op",
                        "from_codepoint operand must be i32".to_string(),
                    ));
                }
                if !matches!(self.module.types.get(dt as usize), Some(MirType::Char)) {
                    return Err(err(
                        "mir.type-op",
                        "from_codepoint dst must be char".to_string(),
                    ));
                }
            }

            _ => {}
        }
        Ok(())
    }
    fn check_function(&mut self, _fi: u32, f: &MirFunction) -> Result<(), MirVerifyError> {
        if !(self.is_owned || self.collections_surface()) {
            let bytes = init::init_analysis_bound(f);
            if bytes > MAX_VERIFY_ANALYSIS_BYTES {
                return Err(MirVerifyError::Resource {
                    kind: ResourceKind::VerifyMemory,
                    message: format!("init analysis for `{}` needs {bytes} bytes, above the frozen {MAX_VERIFY_ANALYSIS_BYTES} budget", f.name),
                });
            }
            self.analysis_bytes = self.analysis_bytes.max(bytes as u64);
            (self.reserve)(bytes)?;
        }
        // Precomputed block bases (flat instruction offsets): the naive
        // per-call prefix sum made chain-shaped functions quadratic.
        let bases: Vec<u32> = {
            let mut v = Vec::with_capacity(f.blocks.len() + 1);
            let mut acc = 0u32;
            v.push(0);
            for b in &f.blocks {
                acc += b.instrs.len() as u32;
                v.push(acc);
            }
            v
        };
        // PLAN-823 T-03: the conservative borrow-capture discipline (§5.1.3
        // /§5.1.6) runs before the per-instruction walk so an escaping or
        // unpinned borrow reports before any other shape complaint.
        if self.is_closures {
            self.check_borrow_discipline(f)?;
            self.check_env_capture_discipline(f)?;
        }
        // shape: sig
        let module = self.module;
        let Some(sig) = module.sig_of(f.sig) else {
            return Err(MirVerifyError::inv(
                "mir.fn-sig",
                format!("function `{}` does not reference a function type", f.name),
            ));
        };
        // shape: slots
        for (si, s) in f.slots.iter().enumerate() {
            if s.ty as usize >= self.module.types.len() {
                return Err(MirVerifyError::inv(
                    "mir.slot-type",
                    format!(
                        "slot #{} of `{}` references out-of-range type {}",
                        si, f.name, s.ty
                    ),
                ));
            }
            // PLAN-862 `mir.unit-placement`: a Unit-typed slot is ZERO-WIDTH
            // and exists only as the return operand of the ENTRY function of
            // a unit entry (the entry kind says the result is Unit). No
            // other slot may be Unit — it would be a value of unit type.
            if matches!(self.module.types.get(s.ty as usize), Some(MirType::Unit)) {
                if self.module.is_host_console_profile() && (s.kind != SlotKind::Temp || !f.blocks.iter().any(|b| matches!(b.term, Terminator::Return { value } if value == si as u32))) {
                    return Err(MirVerifyError::inv("mir.unit-placement", "Unit slots may only be zero-width return temporaries".into()));
                }
                let entry_unit = self.is_entry_unit
                    && self.module.entry_kind == Some(crate::mir::EntryKind::Unit)
                    && _fi == self.module.entry
                    || (self.module.is_host_console_profile()
                        && self
                            .module
                            .sig_of(f.sig)
                            .and_then(|sig| self.module.types.get(sig.result as usize))
                            == Some(&MirType::Unit));
                // PLAN-964: the closures profile's Unit TEMP is a closure
                // call's discarded result slot (`bump()` effect statements)
                // or a unit-result closure's own return operand.
                let closures_call_temp = self.is_closures && s.kind == SlotKind::Temp;
                if !entry_unit && !closures_call_temp {
                    return Err(MirVerifyError::inv(
                        "mir.unit-placement",
                        format!(
                            "slot #{} of `{}` is `unit`; Unit is the entry result and its return operand only",
                            si, f.name
                        ),
                    ));
                }
                if s.kind == SlotKind::Param {
                    return Err(MirVerifyError::inv(
                        "mir.unit-placement",
                        format!(
                            "slot #{} of `{}` is a `unit` PARAMETER; Unit is a result type",
                            si, f.name
                        ),
                    ));
                }
            }
        }
        // params first and in signature order
        let mut param_iter = f
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == SlotKind::Param);
        for (want_ty, _p) in sig.params.iter().enumerate() {
            match param_iter.next() {
                Some((si, s)) => {
                    if s.ty != sig.params[want_ty] {
                        return Err(MirVerifyError::inv(
                            "mir.param-type",
                            format!(
                                "param slot #{} of `{}` has type {}, signature wants {}",
                                si, f.name, s.ty, sig.params[want_ty]
                            ),
                        ));
                    }
                }
                None => {
                    return Err(MirVerifyError::inv(
                        "mir.param-count",
                        format!(
                            "function `{}` has fewer param slots than its signature ({})",
                            f.name,
                            sig.params.len()
                        ),
                    ));
                }
            }
        }
        if param_iter.next().is_some() {
            return Err(MirVerifyError::inv(
                "mir.param-count",
                format!(
                    "function `{}` has more param slots than its signature",
                    f.name
                ),
            ));
        }
        // shape: blocks / entry
        if f.entry as usize >= f.blocks.len() {
            return Err(MirVerifyError::inv(
                "mir.fn-entry",
                format!("entry block {} of `{}` out of range", f.entry, f.name),
            ));
        }
        // shape: spans sorted & in range
        let flat_total = f.flat_instr_count();
        for w in f.spans.windows(2) {
            if w[0].0 >= w[1].0 {
                return Err(MirVerifyError::inv(
                    "mir.span-order",
                    format!("span map of `{}` is not strictly sorted", f.name),
                ));
            }
        }
        if let Some(last) = f.spans.last() {
            if last.0 >= flat_total {
                return Err(MirVerifyError::inv(
                    "mir.span-range",
                    format!(
                        "span map of `{}` references instruction {} beyond count {}",
                        f.name, last.0, flat_total
                    ),
                ));
            }
        }

        // instruction/terminator typing over ALL blocks (reachable or not)
        for (bi, b) in f.blocks.iter().enumerate() {
            for (ii, instr) in b.instrs.iter().enumerate() {
                self.tick(1)?;
                let flat = bases[bi] + ii as u32;
                let sp = self.span_of(f, flat);
                // F-941-15: the 51-arm typing union split into two cold
                // half-walks — the debug frame unions halve; the deep chain's
                // 256 KiB contract no longer pays for unreachable arms
                // (PLAN-961 regression fix). PLAN-965 F-965-R1-09: the old
                // `match instr { _ => .. }` wrapper and its unused `_err`
                // closure were dead code.
                self.check_instr_typing_a(f, instr, sp)?;
                self.check_instr_typing_b(f, instr, sp)?;
            }
            // terminator
            self.tick(1)?;
            let sp = self.span_of(f, bases[bi + 1]);
            let err = |code: &str, msg: String| MirVerifyError::Invalid {
                code: code.to_string(),
                message: msg,
                span: sp,
            };
            match &b.term {
                Terminator::Jump { target } => {
                    if *target as usize >= f.blocks.len() {
                        return Err(err(
                            "mir.jump-target",
                            format!("jump target {} out of range", target),
                        ));
                    }
                }
                Terminator::Branch { cond, then, else_ } => {
                    let ct = slot_ty_of(f, *cond, sp, "branch cond")?;
                    if self.module.type_shape(ct) != Some(ScalarShape::Bool) {
                        return Err(err(
                            "mir.branch-cond-type",
                            format!("branch condition slot type {} is not bool", ct),
                        ));
                    }
                    for (w, t) in [("then", then), ("else", else_)] {
                        if *t as usize >= f.blocks.len() {
                            return Err(err(
                                "mir.branch-target",
                                format!("branch {} target {} out of range", w, t),
                            ));
                        }
                    }
                }
                Terminator::Return { value } => {
                    let vt = slot_ty_of(f, *value, sp, "return value")?;
                    if !self.ty_equiv(vt, sig.result) {
                        return Err(err(
                            "mir.return-type",
                            format!(
                                "return value type {} does not match signature result {}",
                                vt, sig.result
                            ),
                        ));
                    }
                }
                Terminator::LangFail { message } => {
                    let _ = slot_ty_of(f, *message, sp, "lang fail message")?;
                }
            }
        }
        // ---- definite initialization over the reachable CFG -------------
        if self.is_owned || self.collections_surface() {
            // PLAN-821: the ownership dataflow SUBSUMES the must-init
            // analysis (init ⊂ its state lattice) and adds move-kill,
            // borrow tokens, reservation and escape gates. PLAN-822: the
            // collections profile rides the SAME analysis (its lattice
            // carries the guard-fact and payload-moved states).
            let module = self.module;
            let steps = &mut self.steps;
            let analysis_bytes = &mut self.analysis_bytes;
            let fclone = f.clone();
            let deadline = self.deadline;
            let mut res = crate::ownership::verify_owned_function(
                module,
                f,
                steps,
                analysis_bytes,
                &|flat| span_of_fn(&fclone, flat),
            );
            if res.is_ok() && Instant::now() > deadline {
                res = Err(MirVerifyError::Resource {
                    kind: ResourceKind::VerifyWall,
                    message: "verify exceeded wall budget (ownership analysis)".to_string(),
                });
            }
            res
        } else {
            init::check_init(self, f, &bases)
        }
    }
}

impl MirInstr {
    /// The slot this instruction initializes (its definition), if any.
    pub fn def_slot(&self) -> Option<SlotId> {
        match self {
            MirInstr::GlobalGet { dst, .. }
            | MirInstr::GlobalTake { dst, .. }
            | MirInstr::GlobalSet { dst, .. }
            | MirInstr::Const { dst, .. }
            | MirInstr::Copy { dst, .. }
            | MirInstr::Bin { dst, .. }
            | MirInstr::Un { dst, .. }
            | MirInstr::Conv { dst, .. }
            | MirInstr::Call { dst, .. }
            | MirInstr::ForeignCall { dst, .. }
            | MirInstr::FieldGet { dst, .. }
            | MirInstr::Aggregate { dst, .. }
            | MirInstr::Clone { dst, .. }
            | MirInstr::Move { dst, .. }
            | MirInstr::View { dst, .. }
            | MirInstr::MutRef { dst, .. }
            | MirInstr::StrLen { dst, .. }
            | MirInstr::StrLenBytes { dst, .. }
            | MirInstr::StrCharAt { dst, .. }
            | MirInstr::StrSlice { dst, .. }
            | MirInstr::StrConcat { dst, .. }
            | MirInstr::StrToOwned { dst, .. }
            | MirInstr::StrFromChar { dst, .. }
            | MirInstr::StrCmp { dst, .. }
            | MirInstr::CharToCodepoint { dst, .. }
            | MirInstr::CharFromCodepoint { dst, .. } => Some(*dst),
            MirInstr::FieldMove { dst, .. } => Some(*dst),
            // PLAN-822: value-producing collections instructions define
            // their dst; structural mutations (PUSH/INSERT/SET/FIELD_SET/
            // SWITCH) require an ALREADY-initialized root and never
            // re-initialize it.
            // PLAN-968: value-producing scheduler ops define their dst
            // (handles/futures are opaque i32 ids).
            MirInstr::SchedSpawn { dst, .. }
            | MirInstr::SchedAsk { dst, .. }
            | MirInstr::SchedAwait { dst, .. }
            | MirInstr::SchedRecv { dst, .. }
            | MirInstr::SchedPort { dst, .. }
            | MirInstr::SchedStopping { dst, .. } => Some(*dst),
            // PLAN-968: effect-only scheduler ops define no slot.
            MirInstr::SchedSend { .. }
            | MirInstr::SchedReply { .. }
            | MirInstr::SchedStop { .. }
            | MirInstr::SchedSleep { .. }
            | MirInstr::SchedGo { .. } => None,
            MirInstr::ListNew { dst, .. }
            | MirInstr::ListLen { dst, .. }
            | MirInstr::ListCap { dst, .. }
            | MirInstr::ListGet { dst, .. }
            | MirInstr::ListRemove { dst, .. }
            | MirInstr::ListFieldGet { dst, .. }
            | MirInstr::VariantPack { dst, .. }
            | MirInstr::VariantDisc { dst, .. }
            | MirInstr::VariantGuard { dst, .. }
            | MirInstr::VariantProject { dst, .. }
            | MirInstr::ListPlace { dst, .. }
            | MirInstr::PlaceGet { dst, .. } => Some(*dst),
            // PLAN-824: all three Dyn ops define their dst (TAKE on a
            // mismatch traps — nothing after it executes).
            MirInstr::DynPack { dst, .. }
            | MirInstr::DynIs { dst, .. }
            | MirInstr::DynTake { dst, .. } => Some(*dst),
            // PLAN-964: OBJECT_NEW/GET and TEXT_DISPLAY define their dst;
            // OBJECT_SET is a structural mutation (never re-initializes).
            MirInstr::ObjectNew { dst, .. }
            | MirInstr::ObjectGet { dst, .. }
            | MirInstr::TextDisplay { dst, .. } => Some(*dst),
            MirInstr::FieldSet { root, .. } => Some(*root),
            // PLAN-823: closure-surface value producers define their dst.
            MirInstr::CreateClosure { dst, .. }
            | MirInstr::CallClosure { dst, .. }
            | MirInstr::EnvGet { dst, .. }
            | MirInstr::PlatformCall { dst, .. } => Some(*dst),
            MirInstr::HostCall { .. }
            | MirInstr::Trace { .. }
            | MirInstr::BorrowEnd { .. }
            | MirInstr::Drop { .. }
            | MirInstr::PlaceSet { .. } => None,
            MirInstr::ListPush { .. }
            | MirInstr::ListInsert { .. }
            | MirInstr::ListSet { .. }
            | MirInstr::ListFieldSet { .. }
            | MirInstr::VariantSwitch { .. } => None,
            // PLAN-964: OBJECT_SET is a structural mutation (no dst).
            MirInstr::ObjectSet { .. } => None,
        }
    }

    /// Slots this instruction reads, in operand order.
    pub fn use_slots(&self) -> Vec<SlotId> {
        match self {
            MirInstr::Const { .. } | MirInstr::Trace { .. } => vec![],
            MirInstr::Copy { src, .. } => vec![*src],
            MirInstr::FieldMove { root, .. } => vec![*root],
            MirInstr::Bin { lhs, rhs, .. } => vec![*lhs, *rhs],
            MirInstr::Un { operand, .. } => vec![*operand],
            // PLAN-961: a conversion reads its source, defines its dst.
            MirInstr::Conv { src, .. } => vec![*src],
            MirInstr::Call { args, .. } => args.clone(),
            // PLAN-939 S06: a callback lane's args entry is a PLACEHOLDER
            // (the named target rides `callbacks`); it reads no slot.
            MirInstr::ForeignCall {
                args,
                callbacks,
                declaration: _,
                dst: _,
            } => {
                let mut lanes = callbacks
                    .iter()
                    .map(|(l, _)| *l as usize)
                    .collect::<Vec<_>>();
                lanes.sort_unstable();
                let mut out = Vec::with_capacity(args.len());
                let mut li = 0usize;
                for (i, a) in args.iter().enumerate() {
                    if li < lanes.len() && lanes[li] == i {
                        li += 1;
                        continue;
                    }
                    out.push(*a);
                }
                out
            }
            MirInstr::HostCall { arg, .. } => vec![*arg],
            // PLAN-968: scheduler operand reads.
            MirInstr::SchedSpawn { args, .. } | MirInstr::SchedGo { args, .. } => args.clone(),
            MirInstr::SchedSend { handle, msg } => vec![*handle, *msg],
            MirInstr::SchedAsk { handle, msg, .. } => vec![*handle, *msg],
            MirInstr::SchedReply { value } => vec![*value],
            MirInstr::SchedStop { handle } | MirInstr::SchedSleep { ms: handle } => {
                vec![*handle]
            }
            MirInstr::SchedAwait { future, .. } => vec![*future],
            MirInstr::SchedRecv { .. }
            | MirInstr::SchedPort { .. }
            | MirInstr::SchedStopping { .. } => vec![],
            MirInstr::PlatformCall { args, .. } => args.clone(),
            MirInstr::GlobalGet { .. } | MirInstr::GlobalTake { .. } => vec![],
            MirInstr::GlobalSet { src, .. } => vec![*src],
            MirInstr::FieldGet { base, .. } => vec![*base],
            // FieldSet READS the root record (structural rebuild source) and
            // the already-materialized value slot.
            MirInstr::FieldSet { root, value, .. } => vec![*root, *value],
            MirInstr::Aggregate { operands, .. } => operands.clone(),
            MirInstr::Clone { src, .. }
            | MirInstr::Move { src, .. }
            | MirInstr::View { src, .. }
            | MirInstr::MutRef { src, .. } => vec![*src],
            // PLAN-824: every Dyn op reads its source slot (PACK clones —
            // the source stays valid; IS/TAKE never consume).
            MirInstr::DynPack { src, .. }
            | MirInstr::DynIs { src, .. }
            | MirInstr::DynTake { src, .. } => vec![*src],
            MirInstr::BorrowEnd { token } => vec![*token],
            MirInstr::Drop { slot } => vec![*slot],
            MirInstr::StrLen { view, .. }
            | MirInstr::StrLenBytes { view, .. }
            | MirInstr::StrToOwned { view, .. }
            | MirInstr::CharToCodepoint { ch: view, .. } => vec![*view],
            MirInstr::StrFromChar { ch, .. } => vec![*ch],
            MirInstr::StrCharAt { view, index, .. } => vec![*view, *index],
            MirInstr::StrSlice {
                view, start, end, ..
            } => vec![*view, *start, *end],
            MirInstr::StrConcat { lhs, rhs, .. } | MirInstr::StrCmp { lhs, rhs, .. } => {
                vec![*lhs, *rhs]
            }
            MirInstr::CharFromCodepoint { code, .. } => vec![*code],
            // PLAN-822 collections reads (payload transfer slots included;
            // LIST_* roots are reads because the value stays initialized).
            MirInstr::ListNew { .. } => vec![],
            MirInstr::ListLen { src, .. } | MirInstr::ListCap { src, .. } => vec![*src],
            MirInstr::ListGet { base, index, .. } => vec![*base, *index],
            MirInstr::ListPush { root, value, .. } => vec![*root, *value],
            MirInstr::ListInsert {
                root, index, value, ..
            }
            | MirInstr::ListSet {
                root, index, value, ..
            } => vec![*root, *index, *value],
            MirInstr::ListRemove { root, index, .. } => vec![*root, *index],
            MirInstr::ListFieldGet { base, index, .. } => vec![*base, *index],
            MirInstr::ListFieldSet {
                root, index, value, ..
            } => vec![*root, *index, *value],
            MirInstr::VariantPack { payload, .. } => match payload {
                Some(p) => vec![*p],
                None => vec![],
            },
            MirInstr::VariantDisc { src, .. } | MirInstr::VariantGuard { src, .. } => vec![*src],
            MirInstr::VariantProject { root, .. } => vec![*root],
            MirInstr::VariantSwitch { payload, .. } => match payload {
                Some(p) => vec![*p],
                None => vec![],
            },
            MirInstr::ListPlace { base, index, .. } => vec![*base, *index],
            MirInstr::PlaceGet { src, .. } => vec![*src],
            MirInstr::PlaceSet { place, value } => vec![*place, *value],
            // PLAN-823 closure reads: capture sources (clone reads, move
            // reads-then-kills), the closure operand + call args, and the
            // env (not a slot — ENV_GET reads nothing).
            MirInstr::CreateClosure { captures, .. } => captures.iter().map(|c| c.src).collect(),
            MirInstr::CallClosure { closure, args, .. } => std::iter::once(*closure)
                .chain(args.iter().copied())
                .collect(),
            MirInstr::EnvGet { .. } => vec![],
            // PLAN-964 object reads: NEW reads keys+vals (source order);
            // GET reads obj+key; SET reads obj+key+value; DISPLAY reads src.
            MirInstr::ObjectNew { keys, vals, .. } => {
                keys.iter().copied().chain(vals.iter().copied()).collect()
            }
            MirInstr::ObjectGet { obj, key, .. } => vec![*obj, *key],
            MirInstr::ObjectSet { obj, key, value } => vec![*obj, *key, *value],
            MirInstr::TextDisplay { src, .. } => vec![*src],
        }
    }
}

// -- bitset helpers -----------------------------------------------------------

/// Span lookup free of the Verifier borrow (ownership analysis callback).
fn span_of_fn(f: &crate::mir::MirFunction, flat: u32) -> Option<SpanInfo> {
    let mut lo = 0usize;
    let mut hi = f.spans.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if f.spans[mid].0 < flat {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    f.spans
        .get(lo)
        .filter(|(i, _)| *i == flat)
        .map(|(_, sp)| *sp)
}

/// Slot type lookup with the shared out-of-range rejection.
fn slot_ty_of(
    f: &MirFunction,
    s: SlotId,
    sp: Option<SpanInfo>,
    what: &str,
) -> Result<TypeIdx, MirVerifyError> {
    f.slots
        .get(s as usize)
        .map(|x| x.ty)
        .ok_or_else(|| MirVerifyError::Invalid {
            code: "mir.slot-ref".to_string(),
            message: format!("{} references out-of-range slot {}", what, s),
            span: sp,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::MIR_SCHEMA;

    fn owner() -> OwnerScope {
        OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: "m".to_string(),
        }
    }

    fn entry_fn(m: &mut MirModule, name: &str) -> u32 {
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let f = MirFunction {
            owner: owner(),
            name: name.to_string(),
            sig,
            slots: vec![],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        };
        m.funcs.push(f);
        (m.funcs.len() - 1) as u32
    }

    #[test]
    fn mir_admission_minimal_entry_passes() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        // one slot holding the return value
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![MirSlot {
            kind: SlotKind::Local,
            ty: t,
            name: Some("r".into()),
        }];
        let c = m.intern_const(MirConst::I32(7));
        m.funcs[fi as usize].blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: c }];
        m.entry = fi;
        let checked = verify_mir(&m).expect("verified");
        assert_eq!(checked.module().funcs.len(), 1);
    }

    #[test]
    fn mir_admission_uninitialized_read_rejected() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t,
                name: Some("r".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t,
                name: Some("dead".into()),
            },
        ];
        // return slot 1 without initializing it
        m.funcs[fi as usize].blocks[0].term = Terminator::Return { value: 1 };
        m.entry = fi;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.uninitialized-read")
        );
    }

    #[test]
    fn mir_admission_branch_merge_needs_both_arms() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let ti = m.intern_type(MirType::I32);
        let tb = m.intern_type(MirType::Bool);
        m.funcs[fi as usize].slots = vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: tb,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: Some("r".into()),
            },
        ];
        let c_true = m.intern_const(MirConst::Bool(true));
        let c1 = m.intern_const(MirConst::I32(1));
        let f = &mut m.funcs[fi as usize];
        // b0: r = 1 (slot1); branch on slot0 (uninit bool → also invalid, so init it first)
        f.blocks[0].instrs = vec![
            MirInstr::Const {
                dst: 0,
                value: c_true,
            },
            MirInstr::Const { dst: 1, value: c1 },
        ];
        f.blocks[0].term = Terminator::Branch {
            cond: 0,
            then: 1,
            else_: 2,
        };
        // b1: jump b2 ; b2: return slot1 — all init: fine
        f.blocks.push(MirBlock {
            instrs: vec![],
            term: Terminator::Jump { target: 2 },
        });
        f.blocks.push(MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 1 },
        });
        m.entry = fi;
        verify_mir(&m).expect("both arms keep r initialized");

        // now make the then-arm reinit-only path: remove init from entry
        let f = &mut m.funcs[fi as usize];
        f.blocks[0]
            .instrs
            .retain(|i| matches!(i, MirInstr::Const { dst: 0, .. }));
        // slot1 initialized in NEITHER arm now → rejection
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.uninitialized-read")
        );
    }

    #[test]
    fn mir_admission_loop_init_does_not_escape() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let ti = m.intern_type(MirType::I32);
        let tb = m.intern_type(MirType::Bool);
        m.funcs[fi as usize].slots = vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: tb,
                name: None,
            }, // 0 cond
            MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: Some("r".into()),
            }, // 1
        ];
        let c_true = m.intern_const(MirConst::Bool(true));
        let c1 = m.intern_const(MirConst::I32(1));
        let f = &mut m.funcs[fi as usize];
        f.blocks[0].instrs = vec![MirInstr::Const {
            dst: 0,
            value: c_true,
        }];
        f.blocks[0].term = Terminator::Branch {
            cond: 0,
            then: 1,
            else_: 3,
        };
        // b1 (loop body): r = 1; jump b2
        f.blocks.push(MirBlock {
            instrs: vec![MirInstr::Const { dst: 1, value: c1 }],
            term: Terminator::Jump { target: 2 },
        });
        // b2 (latch): branch cond → body : exit
        f.blocks.push(MirBlock {
            instrs: vec![],
            term: Terminator::Branch {
                cond: 0,
                then: 1,
                else_: 3,
            },
        });
        // b3 (exit): return r — r only initialized INSIDE the loop → reject
        f.blocks.push(MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 1 },
        });
        m.entry = fi;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.uninitialized-read"),
            "got {err:?}"
        );
    }

    #[test]
    fn mir_admission_dangling_jump_target() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let f = &mut m.funcs[fi as usize];
        f.blocks[0].term = Terminator::Jump { target: 9 };
        m.entry = fi;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.jump-target")
        );
    }

    #[test]
    fn mir_admission_wrong_entry_result_type() {
        let mut m = MirModule::new(owner());
        let tb = m.intern_type(MirType::Bool);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: tb,
        }));
        let f = MirFunction {
            owner: owner(),
            name: "main".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Local,
                ty: tb,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        };
        m.funcs.push(f);
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.entry-sig"));
    }

    #[test]
    fn mir_admission_table_budget_is_resource() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        // A function with many empty blocks to blow the 65_536 budget.
        let blocks: Vec<MirBlock> = (0..70_000)
            .map(|_| MirBlock {
                instrs: vec![],
                term: Terminator::Jump { target: 0 },
            })
            .collect();
        m.funcs.push(MirFunction {
            owner: owner(),
            name: "big".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: None,
            }],
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(matches!(
            err,
            MirVerifyError::Resource {
                kind: ResourceKind::TableEntries,
                ..
            }
        ));
    }

    // ---- F798-R3-01 admissions: owner / capability / import identity ----

    #[test]
    fn mir_admission_foreign_function_owner_rejected() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![MirSlot {
            kind: SlotKind::Local,
            ty: t,
            name: Some("r".into()),
        }];
        let c = m.intern_const(MirConst::I32(7));
        m.funcs[fi as usize].blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: c }];
        m.entry = fi;
        // Tamper: the function claims a foreign owner scope.
        m.funcs[fi as usize].owner = OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: "other-module".to_string(),
        };
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.owner"),
            "got {err:?}"
        );
        // Legal control: same owner verifies.
        m.funcs[fi as usize].owner = owner();
        verify_mir(&m).expect("same-owner module verifies");
    }

    #[test]
    fn mir_admission_type_ref_equal_len_rejected() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        // An Fn type whose param points at index == types.len() AFTER the
        // Fn itself is pushed (out of range by exactly one — the old `>`
        // check let it through, F798-R3-01).
        let idx_after_push = (m.types.len() + 1) as u32;
        let _bad = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti, idx_after_push],
            result: ti,
        }));
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Invalid { ref code, .. } if code == "mir.type-ref"),
            "got {err:?}"
        );
    }

    #[test]
    fn mir_admission_unknown_capability_rejected() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let _ = fi;
        m.capabilities = vec!["unknown.superuser".to_string()];
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Unsupported { ref code, .. } if code == "mir.capability-unknown"),
            "got {err:?}"
        );
    }

    #[test]
    fn mir_admission_unknown_import_rejected() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti],
            result: ti,
        }));
        let fi = entry_fn(&mut m, "main");
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![MirSlot {
            kind: SlotKind::Local,
            ty: t,
            name: Some("r".into()),
        }];
        let c7 = m.intern_const(MirConst::I32(7));
        m.funcs[fi as usize].blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: c7 }];
        m.imports.push(MirImport {
            symbol: "unknown.native.ffi".to_string(),
            effect: Effect::Pure,
            sig,
            env: None,
        });
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Unsupported { ref code, .. } if code == "mir.import-unknown"),
            "got {err:?}"
        );
        // The approved trace intrinsics with the trace capability stay
        // admissible in a module (unused import is a shape, not a grant).
        m.imports.clear();
        m.imports.push(MirImport {
            symbol: "hir.test.mark_a".to_string(),
            effect: Effect::Observable,
            sig,
            env: None,
        });
        m.capabilities = vec![TRACE_CAPABILITY.to_string()];
        verify_mir(&m).expect("trace import with capability verifies");
    }

    #[test]
    fn mir_admission_unit_fn_linkage_import_allowed() {
        let mut m = MirModule::unit(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti],
            result: ti,
        }));
        m.imports.push(MirImport {
            symbol: "fn:helper".to_string(),
            effect: Effect::Pure,
            sig,
            env: None,
        });
        let f = MirFunction {
            owner: owner(),
            name: "caller".into(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Param,
                ty: ti,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        };
        m.funcs.push(f);
        m.entry = 0;
        verify_mir(&m).expect("unit fn: linkage import verifies");
        // The same linkage import is NOT admissible in a whole module.
        m.is_unit = false;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(err, MirVerifyError::Unsupported { ref code, .. } if code == "mir.import-unknown"),
            "got {err:?}"
        );
    }

    // ---- F798-R3-08: composite analysis memory BEFORE allocation ----

    #[test]
    fn mir_verify_dense_analysis_memory_pre_rejected() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        // 16384 slots + 8192 empty blocks: 24581 table entries — under every
        // per-table budget, but the init bitsets need 3 × 16 MiB.
        let slots: Vec<MirSlot> = (0..16_384)
            .map(|_| MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: None,
            })
            .collect();
        let blocks: Vec<MirBlock> = (0..8192)
            .map(|i| MirBlock {
                instrs: vec![],
                term: Terminator::Jump {
                    target: (i + 1) % 8192,
                },
            })
            .collect();
        m.funcs.push(MirFunction {
            owner: owner(),
            name: "dense".into(),
            sig,
            slots,
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        let err = verify_mir(&m).unwrap_err();
        assert!(
            matches!(
                err,
                MirVerifyError::Resource {
                    kind: ResourceKind::VerifyMemory,
                    ..
                }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn mir_verify_analysis_bytes_recorded_on_credential() {
        let mut m = MirModule::new(owner());
        let fi = entry_fn(&mut m, "main");
        let t = m.intern_type(MirType::I32);
        m.funcs[fi as usize].slots = vec![MirSlot {
            kind: SlotKind::Local,
            ty: t,
            name: Some("r".into()),
        }];
        let c = m.intern_const(MirConst::I32(7));
        m.funcs[fi as usize].blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: c }];
        m.entry = fi;
        let checked = verify_mir(&m).expect("verified");
        assert!(checked.stats().analysis_bytes > 0);
    }

    /// D-981-GLOBAL-CLONE-GAP closure（PLAN-981）：CLONE 是唯一的
    /// 视图→拥有跨接指令——`RecordView(r)` 源深拷贝进同名 `Record(r)`
    /// 的 owned 槽。含资源叶子（Str）与非资源（纯标量）record 都合法；
    /// 非同名 record 仍是 `mir.clone-type`。本组只验证 verify 面；
    /// 执行/深拷贝独立性在 av.rs 的执行测试里。
    fn owned_module_with_view_clone(
        view_name: &str,
        rec_name: &str,
        fields: &[(&str, MirType)],
    ) -> MirModule {
        let mod_owner = || OwnerScope {
            schema: crate::contract::MIR_OWNED_SCHEMA.to_string(),
            module: "m".to_string(),
        };
        let mut m = MirModule::new(mod_owner());
        m.profile = crate::contract::AV_OWNED_PROFILE.to_string();
        let ti = m.intern_type(MirType::I32);
        let tstr = m.intern_type(MirType::Str);
        let field_defs: Vec<MirFieldDef> = fields
            .iter()
            .map(|(fname, ft)| MirFieldDef {
                name: fname.to_string(),
                ty: m.intern_type(ft.clone()),
            })
            .collect();
        let ref_of = |name: &str| crate::mir::RecordTypeRef {
            owner_module: "m".to_string(),
            name: name.to_string(),
            fields: field_defs.clone(),
        };
        let view_ti = m.intern_type(MirType::RecordView(ref_of(view_name)));
        let rec_ti = m.intern_type(MirType::Record(ref_of(rec_name)));
        let takes_sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![view_ti],
            result: ti,
        }));
        // Non-entry helper: a borrowed `p RecordView(r)` param, CLONED into a
        // fresh owned `Record(r)` local (the view→owned crossing under test).
        let takes = MirFunction {
            owner: mod_owner(),
            name: "takes".to_string(),
            sig: takes_sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: view_ti,
                    name: Some("p".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: rec_ti,
                    name: Some("owned".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: ti,
                    name: Some("r".into()),
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::Clone { dst: 1, src: 0 },
                    MirInstr::Const {
                        dst: 2,
                        value: m.intern_const(MirConst::I32(0)),
                    },
                ],
                term: Terminator::Return { value: 2 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        };
        m.funcs.push(takes);
        let takes_idx = (m.funcs.len() - 1) as u32;
        // Entry builds an OWNED record (Aggregate over owned operands) and
        // passes it to the view param — the legal call-site form for a
        // RecordView param (owned args satisfy borrowed params). One Const
        // per declared field keeps the operand list field-shape-agnostic.
        let mut main_instrs = Vec::new();
        let mut main_slots = Vec::new();
        let mut operands = Vec::new();
        for (fi, fd) in field_defs.iter().enumerate() {
            let (slot_ty, konst) = match m.types.get(fd.ty as usize) {
                Some(MirType::Str) => (tstr, MirConst::Str(b"abc".to_vec().into())),
                _ => (ti, MirConst::I32(3 + fi as i64 as i32)),
            };
            main_slots.push(MirSlot {
                kind: SlotKind::Local,
                ty: slot_ty,
                name: Some(format!("f{fi}")),
            });
            main_instrs.push(MirInstr::Const {
                dst: fi as u32,
                value: m.intern_const(konst),
            });
            operands.push(fi as u32);
        }
        main_slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: rec_ti,
            name: Some("rec".into()),
        });
        let rec_slot = main_slots.len() as u32 - 1;
        main_slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: ti,
            name: Some("out".into()),
        });
        let out_slot = main_slots.len() as u32 - 1;
        main_instrs.push(MirInstr::Aggregate {
            dst: rec_slot,
            record_ty: rec_ti,
            operands,
        });
        main_instrs.push(MirInstr::Call {
            callee: crate::mir::CalleeRef::Func(takes_idx),
            dst: out_slot,
            args: vec![rec_slot],
        });
        let main_sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let main = MirFunction {
            owner: mod_owner(),
            name: "main".to_string(),
            sig: main_sig,
            slots: main_slots,
            blocks: vec![MirBlock {
                instrs: main_instrs,
                term: Terminator::Return { value: out_slot },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        };
        m.funcs.push(main);
        m.entry = takes_idx + 1;
        m
    }

    #[test]
    fn mir_clone_view_to_owned_resource_record_admitted() {
        // Str-carrying record (resource leaf) — the deep-clone form.
        let m = owned_module_with_view_clone(
            "Img",
            "Img",
            &[("sha", MirType::Str), ("w", MirType::I32)],
        );
        let checked = verify_mir(&m).expect("view→owned CLONE of a resource record verifies");
        assert_eq!(checked.module().funcs.len(), 2);
    }

    #[test]
    fn mir_clone_view_to_owned_scalar_record_admitted() {
        // Pure-scalar record: COPY cannot cross a view, so CLONE carries the
        // value copy — legal (the deep clone degenerates).
        let m = owned_module_with_view_clone(
            "Quad",
            "Quad",
            &[("w", MirType::I32), ("h", MirType::I32)],
        );
        verify_mir(&m).expect("view→owned CLONE of a scalar record verifies");
    }

    #[test]
    fn mir_clone_view_to_owned_wrong_record_rejected() {
        // A DIFFERENT nominal record as dst — still mir.clone-type.
        let m = owned_module_with_view_clone("A", "B", &[("x", MirType::I32)]);
        let err = verify_mir(&m).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("mir.clone-type"), "got {msg}");
    }
}
