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
use std::collections::VecDeque;
use std::time::Instant;

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
    let mut v = Verifier {
        module,
        steps: 0,
        deadline,
        analysis_bytes: 0,
        is_data: module.is_data_profile(),
        is_owned: module.is_owned_profile(),
        is_collections: module.is_collections_profile(),
    };
    v.run()?;
    let stats = v.stats_snapshot();
    Ok(CheckedMir {
        inner: module.clone(),
        stats,
    })
}

struct Verifier<'a> {
    module: &'a MirModule,
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
}

impl<'a> Verifier<'a> {
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
        // ---- identity -------------------------------------------------
        if self.module.profile != crate::contract::AV_PROFILE
            && self.module.profile != crate::contract::AV_DATA_PROFILE
            && self.module.profile != crate::contract::AV_OWNED_PROFILE
            && self.module.profile != crate::contract::AV_COLLECTIONS_PROFILE
        {
            return Err(MirVerifyError::uns(
                "mir.profile",
                format!(
                    "profile `{}` is not one of the supported profiles (`{}` / `{}` / `{}` / `{}`)",
                    self.module.profile,
                    crate::contract::AV_PROFILE,
                    crate::contract::AV_DATA_PROFILE,
                    crate::contract::AV_OWNED_PROFILE,
                    crate::contract::AV_COLLECTIONS_PROFILE
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
        if self.is_owned && self.module.owner.schema != crate::contract::MIR_OWNED_SCHEMA {
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
                    if !self.is_data && !self.is_owned && !self.is_collections =>
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
                MirType::Char
                | MirType::Str
                | MirType::StrView
                | MirType::MutRef
                | MirType::RecordView(_)
                    if !self.is_owned && !self.is_collections =>
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
                // PLAN-822: the List/Variant surface is collections-only.
                MirType::List(_) | MirType::Variant(_) if !self.is_collections => {
                    return Err(MirVerifyError::uns(
                        "mir.profile-type",
                        format!(
                            "type #{ti} ({:?} …) exists only in the `{}` profile",
                            t,
                            crate::contract::AV_COLLECTIONS_PROFILE
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
            if let MirType::Record(r) = t {
                for f in &r.fields {
                    if matches!(
                        self.module.types.get(f.ty as usize),
                        Some(MirType::List(_)) | Some(MirType::Variant(_))
                    ) {
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
                        | Some(MirType::RecordView(_)) => {
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
                None => matches!(imp.symbol.as_str(), "hir.test.mark_a" | "hir.test.mark_b"),
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
                || (self.is_owned && c == crate::contract::TEXT_RUNTIME_CAPABILITY);
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
            if self.is_owned {
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
    /// PLAN-822: the frozen admissible element/payload domain (contract-
    /// decision §2) — scalars (incl. wide + char), owned `Str`, records.
    /// Rejected: borrowed views/tokens, `Fn`, and containers (List-of-List
    /// / container payloads keep the first batch one layer deep).
    fn check_element_domain(
        &mut self,
        elem: TypeIdx,
        ti: usize,
        code: &'static str,
    ) -> Result<(), MirVerifyError> {
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
                let fields = match &self.module.types[*node] {
                    MirType::Record(r) => &r.fields,
                    _ => {
                        color[*node] = BLACK;
                        stack.pop();
                        continue;
                    }
                };
                let edges: Vec<TypeIdx> = match &self.module.types[*node] {
                    MirType::Record(r) => r.fields.iter().map(|f| f.ty).collect(),
                    MirType::Variant(v) => v.arms.iter().filter_map(|a| a.payload).collect(),
                    _ => vec![],
                };
                let _ = fields;
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

    fn check_function(&mut self, fi: u32, f: &MirFunction) -> Result<(), MirVerifyError> {
        let _ = fi;
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
        // shape: sig
        let Some(sig) = self.module.sig_of(f.sig).cloned() else {
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
                let err = |code: &str, msg: String| MirVerifyError::Invalid {
                    code: code.to_string(),
                    message: msg,
                    span: sp,
                };
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
                        if owned_const && !self.is_owned && !self.is_collections {
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
                            Some(MirConst::Char(_)) | Some(MirConst::Str(_)) => None,
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
                        if let (Some(shape), false) = (want_shape, self.is_data) {
                            // Wide-scalar constants are data-profile-only.
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
                                // A Str const materializes by DESTINATION
                                // kind: an owned Str slot gets a fresh
                                // allocation; a StrView slot gets the
                                // immortal static view (770 literal rule).
                                MirConst::Str(_) => {
                                    Some(match self.module.types.get(dt as usize) {
                                        Some(MirType::StrView) => MirType::StrView,
                                        _ => MirType::Str,
                                    })
                                }
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
                                Some(MirType::Str)
                                | Some(MirType::MutRef)
                                | Some(MirType::Record(_))
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
                        // Wide scalars have NO arithmetic in this profile: a
                        // Bin touching i64/u64/f32/f64 is an out-of-profile
                        // operation regardless of shapes otherwise matching
                        // (contract-decision §4 — no new Bin subcodes).
                        if self.is_data
                            && matches!(
                                shapes.1.or(shapes.2),
                                Some(ScalarShape::I64)
                                    | Some(ScalarShape::U64)
                                    | Some(ScalarShape::F32)
                                    | Some(ScalarShape::F64)
                            )
                        {
                            return Err(err(
                                "mir.type-op",
                                format!(
                                    "binary {:?} on wide-scalar operands (types {},{}); the data profile has no wide-scalar arithmetic",
                                    op, lt, rt
                                ),
                            ));
                        }
                        use ScalarShape::*;
                        let ok = if op.result_is_bool() {
                            match op {
                                BinOp::EqI32 | BinOp::NeI32 => {
                                    shapes.1.is_some()
                                        && shapes.1 == shapes.2
                                        && shapes.0 == Some(Bool)
                                }
                                _ => shapes == (Some(Bool), Some(I32), Some(I32)),
                            }
                        } else {
                            shapes == (Some(I32), Some(I32), Some(I32))
                        };
                        if !ok {
                            return Err(err(
                                "mir.bin-type",
                                format!("binary {:?} does not type check for dst/lhs/rhs types ({},{},{})", op, dt, lt, rt),
                            ));
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
                    MirInstr::Call { callee, dst, args } => {
                        let dt = slot_ty_of(f, *dst, sp, "call dst")?;
                        let callee_sig: FnSig = match callee {
                            CalleeRef::Func(idx) => {
                                if self.module.is_unit {
                                    return Err(err(
                                        "mir.unit-callee",
                                        "a unit must address callees through imports".to_string(),
                                    ));
                                }
                                match self.module.funcs.get(*idx as usize) {
                                    Some(target) => match self.module.sig_of(target.sig) {
                                        Some(s) => s.clone(),
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
                                            format!(
                                                "call references out-of-range function {}",
                                                idx
                                            ),
                                        ))
                                    }
                                }
                            }
                            CalleeRef::Import(idx) => {
                                match self.module.imports.get(*idx as usize) {
                                    Some(imp) => match self.module.sig_of(imp.sig) {
                                        Some(s) => s.clone(),
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
                                }
                            }
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
                        for (ai, (a, want)) in args.iter().zip(callee_sig.params.iter()).enumerate()
                        {
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
                                    format!(
                                        "call arg #{} has type {}, signature wants {}",
                                        ai, at, want
                                    ),
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
                    MirInstr::Trace { .. } => {}
                    MirInstr::FieldGet { dst, base, path } => {
                        if !self.is_data && !self.is_owned && !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "FIELD_GET exists only in the data/owned profiles".to_string(),
                            ));
                        }
                        self.check_field_path(f, *base, path, sp, "field-get")?;
                        let dt = slot_ty_of(f, *dst, sp, "field-get dst")?;
                        let bt = slot_ty_of(f, *base, sp, "field-get base")?;
                        let Some((final_ty, _owner)) = self.module.resolve_field_path(bt, path)
                        else {
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
                                    if !matches!(
                                        self.module.types.get(dt as usize),
                                        Some(MirType::StrView)
                                    ) {
                                        return Err(err(
                                            "mir.field-type",
                                            format!(
                                                "field-get of a Str field must land in a StrView slot (dst {})",
                                                dt
                                            ),
                                        ));
                                    }
                                }
                                Some(t)
                                    if crate::ownership::type_is_owned(self.module, final_ty) =>
                                {
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
                        if !self.is_data && !self.is_owned && !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "FIELD_SET exists only in the data/owned profiles".to_string(),
                            ));
                        }
                        self.check_field_path(f, *root, path, sp, "field-set")?;
                        let rt_slot = slot_ty_of(f, *root, sp, "field-set root")?;
                        let Some((final_ty, _owner)) =
                            self.module.resolve_field_path(rt_slot, path)
                        else {
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
                        if !self.is_data && !self.is_owned && !self.is_collections {
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
                            let ot = slot_ty_of(
                                f,
                                *op_slot,
                                sp,
                                &format!("aggregate operand #{}", fi_),
                            )?;
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
                        if !self.is_owned && !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "CLONE exists only in the owned profile".to_string(),
                            ));
                        }
                        let dt = slot_ty_of(f, *dst, sp, "clone dst")?;
                        let st = slot_ty_of(f, *src, sp, "clone src")?;
                        if !self.ty_equiv(dt, st) {
                            return Err(err(
                                "mir.clone-type",
                                format!("clone from type {} into type {}", st, dt),
                            ));
                        }
                        // Clone is the EXPLICIT deep copy of owned data;
                        // cloning a copyable is a plain COPY.
                        if !crate::ownership::type_is_owned(self.module, st) {
                            return Err(err(
                                "mir.ownership-required",
                                format!("CLONE of copyable type {} (use COPY)", st),
                            ));
                        }
                    }
                    MirInstr::Move { dst, src } => {
                        if !self.is_owned && !self.is_collections {
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
                                format!("MOVE of copyable type {} (ownership transfer is for owned data)", st),
                            ));
                        }
                    }
                    MirInstr::View { dst, src } => {
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "DROP exists only in the owned profile".to_string(),
                            ));
                        }
                        let st = slot_ty_of(f, *slot, sp, "drop slot")?;
                        if !crate::ownership::type_is_owned(self.module, st) {
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
                        if !self.is_owned && !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "FIELDMOVE exists only in the owned profile".to_string(),
                            ));
                        }
                        self.check_field_path(f, *root, path, sp, "field-move")?;
                        let dt = slot_ty_of(f, *dst, sp, "field-move dst")?;
                        let rt = slot_ty_of(f, *root, sp, "field-move root")?;
                        let Some((final_ty, _owner)) = self.module.resolve_field_path(rt, path)
                        else {
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
                        if !self.is_collections {
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
                        if !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "LIST_LEN/LIST_CAP exist only in the collections profile"
                                    .to_string(),
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
                        if !self.is_collections {
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
                        if !self.is_collections {
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
                        if !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "LIST_INSERT/LIST_SET exist only in the collections profile"
                                    .to_string(),
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
                    MirInstr::ListRemove { dst, root, index } => {
                        if !self.is_collections {
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
                        if !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "LIST_FIELD_GET exists only in the collections profile".to_string(),
                            ));
                        }
                        let elem = self.list_elem_of(f, *base, sp)?;
                        self.check_list_index_slot(f, *index, sp)?;
                        let Some((final_ty, _owner)) = self.module.resolve_field_path(elem, path)
                        else {
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
                        if !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "LIST_FIELD_SET exists only in the collections profile".to_string(),
                            ));
                        }
                        let elem = self.list_elem_of(f, *root, sp)?;
                        self.check_list_index_slot(f, *index, sp)?;
                        let Some((final_ty, _owner)) = self.module.resolve_field_path(elem, path)
                        else {
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
                        if !self.is_collections {
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
                        if !self.is_collections {
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
                        if !self.is_collections {
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
                        if !self.is_collections {
                            return Err(err(
                                "mir.profile-instr",
                                "VARIANT_PROJECT exists only in the collections profile"
                                    .to_string(),
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
                        if !self.is_collections {
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
                    MirInstr::BorrowEnd { token } => {
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                    MirInstr::StrCmp { dst, lhs, rhs, .. } => {
                        if !self.is_owned && !self.is_collections {
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
                            return Err(err(
                                "mir.type-op",
                                "cmp operands must be views".to_string(),
                            ));
                        }
                        if self.module.type_shape(dt) != Some(ScalarShape::Bool) {
                            return Err(err(
                                "mir.type-op",
                                format!("cmp dst type {} is not bool", dt),
                            ));
                        }
                    }
                    MirInstr::CharToCodepoint { dst, ch } => {
                        if !self.is_owned && !self.is_collections {
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
                        if !self.is_owned && !self.is_collections {
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
                }
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
            }
        }
        // ---- definite initialization over the reachable CFG -------------
        if self.is_owned || self.is_collections {
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
            self.check_init(f, &bases)
        }
    }

    fn check_init(&mut self, f: &MirFunction, bases: &[u32]) -> Result<(), MirVerifyError> {
        let n = f.slots.len();
        let words = words_for(n);
        let blocks = f.blocks.len();
        // Composite analysis budget BEFORE allocation (F798-R3-08): the
        // init dataflow holds defs/in/out bitsets of blocks × words each
        // (plus per-block successor/predecessor vectors). A dense unit
        // (16384 slots × 8192 blocks ≈ 3 × 16 MiB) previously allocated
        // ~51.7 MiB BEFORE the work-step budget rejected it — the guard
        // must fire first, as a resource classification.
        let per_matrix = blocks * words * std::mem::size_of::<u64>();
        let analysis_bound = per_matrix.saturating_mul(3)
            + blocks
                * (std::mem::size_of::<Vec<u64>>()
                    + 2 * std::mem::size_of::<Vec<u32>>()
                    + 2 * std::mem::size_of::<u32>());
        self.analysis_bytes = self.analysis_bytes.max(analysis_bound as u64);
        if analysis_bound > MAX_VERIFY_ANALYSIS_BYTES {
            return Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifyMemory,
                message: format!(
                    "init analysis for `{}` needs ~{} bytes ({} blocks × {} words × 3 matrices), above the frozen {} budget",
                    f.name,
                    analysis_bound,
                    blocks,
                    words,
                    MAX_VERIFY_ANALYSIS_BYTES
                ),
            });
        }
        // Def sets per block (fixed after the typing pass).
        let defs: Vec<Vec<u64>> = f
            .blocks
            .iter()
            .map(|b| {
                let mut m = vec![0u64; words];
                for instr in &b.instrs {
                    if let Some(d) = instr.def_slot() {
                        set_bit(&mut m, d as usize);
                    }
                }
                m
            })
            .collect();
        // Successors.
        let succs: Vec<Vec<u32>> = f
            .blocks
            .iter()
            .map(|b| match &b.term {
                Terminator::Jump { target } => vec![*target],
                Terminator::Branch { then, else_, .. } => {
                    if then == else_ {
                        vec![*then]
                    } else {
                        vec![*then, *else_]
                    }
                }
                Terminator::Return { .. } => vec![],
            })
            .collect();
        // Predecessors (reachable ones only, computed on the fly below).
        let mut preds: Vec<Vec<u32>> = vec![Vec::new(); blocks];
        // Reachability from entry.
        let mut reachable = vec![false; blocks];
        let mut stack = vec![f.entry];
        reachable[f.entry as usize] = true;
        while let Some(b) = stack.pop() {
            for s in &succs[b as usize] {
                if !reachable[*s as usize] {
                    reachable[*s as usize] = true;
                    stack.push(*s);
                }
            }
        }
        for b in 0..blocks {
            if !reachable[b] {
                continue;
            }
            for s in &succs[b] {
                if !preds[*s as usize].contains(&(b as u32)) {
                    preds[*s as usize].push(b as u32);
                }
            }
        }

        // In/out states; non-entry blocks start optimistic (all ones).
        let mut in_state: Vec<Vec<u64>> = (0..blocks)
            .map(|b| {
                if (b as u32) == f.entry {
                    let mut m = vec![0u64; words];
                    for (si, s) in f.slots.iter().enumerate() {
                        if s.kind == SlotKind::Param {
                            set_bit(&mut m, si);
                        }
                    }
                    m
                } else {
                    vec![u64::MAX; words]
                }
            })
            .collect();
        let mut out_state: Vec<Vec<u64>> = in_state
            .iter()
            .zip(defs.iter())
            .map(|(i, d)| union(i, d))
            .collect();
        // Terminator reads.
        let term_reads: Vec<Option<(u32, u32)>> = f
            .blocks
            .iter()
            .enumerate()
            .map(|(bi, b)| {
                let flat = bases[bi + 1];
                match &b.term {
                    Terminator::Branch { cond, .. } => Some((*cond, flat)),
                    Terminator::Return { value } => Some((*value, flat)),
                    Terminator::Jump { .. } => None,
                }
            })
            .collect();

        // Reverse postorder for a fast convergence seed.
        let mut rpo: Vec<u32> = Vec::with_capacity(blocks);
        {
            let mut seen = vec![false; blocks];
            let mut stack: Vec<(u32, usize)> = vec![(f.entry, 0)];
            seen[f.entry as usize] = true;
            while let Some((b, ci)) = stack.pop() {
                let ss = &succs[b as usize];
                if ci < ss.len() {
                    stack.push((b, ci + 1));
                    let s = ss[ci];
                    if !seen[s as usize] {
                        seen[s as usize] = true;
                        stack.push((s, 0));
                    }
                } else {
                    rpo.push(b);
                }
            }
            rpo.reverse();
        }

        let mut queue: VecDeque<u32> = rpo.iter().copied().collect();
        let mut queued = vec![false; blocks];
        for b in &queue {
            queued[*b as usize] = true;
        }
        let mut visits: u64 = 0;
        while let Some(b) = queue.pop_front() {
            queued[b as usize] = false;
            visits += 1;
            self.tick(1 + words as u64)?;
            // in = ∩ preds (reachable) out
            let mut new_in = if b == f.entry {
                in_state[b as usize].clone()
            } else {
                match preds[b as usize]
                    .iter()
                    .filter(|p| reachable[**p as usize])
                    .next()
                {
                    Some(first) => out_state[*first as usize].clone(),
                    None => vec![0u64; words], // unreachable-defensive
                }
            };
            if b != f.entry {
                for p in preds[b as usize]
                    .iter()
                    .filter(|p| reachable[**p as usize])
                    .skip(1)
                {
                    intersect(&mut new_in, &out_state[*p as usize]);
                }
            }
            // Reads must be initialized in `in ∪ defs-so-far`: walk instructions
            // in order, checking uses then applying the def.
            let mut live = new_in.clone();
            let mut fail: Option<(u32, u32)> = None;
            'blk: for (ii, instr) in f.blocks[b as usize].instrs.iter().enumerate() {
                let flat = bases[b as usize] + ii as u32;
                for u in instr.use_slots() {
                    if !bit_is_set(&live, u as usize) {
                        fail = Some((u, flat));
                        break 'blk;
                    }
                }
                if let Some(d) = instr.def_slot() {
                    set_bit(&mut live, d as usize);
                }
            }
            if fail.is_none() {
                if let Some((slot, flat)) = term_reads[b as usize] {
                    if !bit_is_set(&live, slot as usize) {
                        fail = Some((slot, flat));
                    }
                }
            }
            if let Some((slot, flat)) = fail {
                let sp = self.span_of(f, flat);
                let name = f
                    .slots
                    .get(slot as usize)
                    .and_then(|s| s.name.clone())
                    .unwrap_or_else(|| format!("slot#{}", slot));
                return Err(MirVerifyError::Invalid {
                    code: "mir.uninitialized-read".to_string(),
                    message: format!(
                        "read of uninitialized slot `{}` (kind {:?}) in `{}`",
                        name,
                        f.slots
                            .get(slot as usize)
                            .map(|s| s.kind)
                            .unwrap_or(SlotKind::Temp),
                        f.name
                    ),
                    span: sp,
                });
            }
            // out = in ∪ defs
            let mut new_out = new_in.clone();
            union_into(&mut new_out, &defs[b as usize]);
            if new_in != in_state[b as usize] {
                in_state[b as usize] = new_in;
                out_state[b as usize] = new_out;
                for s in &succs[b as usize] {
                    if !queued[*s as usize] {
                        queued[*s as usize] = true;
                        queue.push_back(*s);
                    }
                }
            } else if new_out != out_state[b as usize] {
                out_state[b as usize] = new_out;
            }
            if visits > (blocks as u64) * 16 + 1024 {
                // Defensive: the lattice guarantees termination; this only
                // catches implementation bugs without looping forever.
                return Err(MirVerifyError::Resource {
                    kind: ResourceKind::VerifySteps,
                    message: "init worklist exceeded its defensive visit bound".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl MirInstr {
    /// The slot this instruction initializes (its definition), if any.
    pub fn def_slot(&self) -> Option<SlotId> {
        match self {
            MirInstr::Const { dst, .. }
            | MirInstr::Copy { dst, .. }
            | MirInstr::Bin { dst, .. }
            | MirInstr::Un { dst, .. }
            | MirInstr::Call { dst, .. }
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
            | MirInstr::StrCmp { dst, .. }
            | MirInstr::CharToCodepoint { dst, .. }
            | MirInstr::CharFromCodepoint { dst, .. } => Some(*dst),
            MirInstr::FieldMove { dst, .. } => Some(*dst),
            // PLAN-822: value-producing collections instructions define
            // their dst; structural mutations (PUSH/INSERT/SET/FIELD_SET/
            // SWITCH) require an ALREADY-initialized root and never
            // re-initialize it.
            MirInstr::ListNew { dst, .. }
            | MirInstr::ListLen { dst, .. }
            | MirInstr::ListCap { dst, .. }
            | MirInstr::ListGet { dst, .. }
            | MirInstr::ListRemove { dst, .. }
            | MirInstr::ListFieldGet { dst, .. }
            | MirInstr::VariantPack { dst, .. }
            | MirInstr::VariantDisc { dst, .. }
            | MirInstr::VariantGuard { dst, .. }
            | MirInstr::VariantProject { dst, .. } => Some(*dst),
            MirInstr::FieldSet { root, .. } => Some(*root),
            MirInstr::Trace { .. } | MirInstr::BorrowEnd { .. } | MirInstr::Drop { .. } => None,
            MirInstr::ListPush { .. }
            | MirInstr::ListInsert { .. }
            | MirInstr::ListSet { .. }
            | MirInstr::ListFieldSet { .. }
            | MirInstr::VariantSwitch { .. } => None,
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
            MirInstr::Call { args, .. } => args.clone(),
            MirInstr::FieldGet { base, .. } => vec![*base],
            // FieldSet READS the root record (structural rebuild source) and
            // the already-materialized value slot.
            MirInstr::FieldSet { root, value, .. } => vec![*root, *value],
            MirInstr::Aggregate { operands, .. } => operands.clone(),
            MirInstr::Clone { src, .. }
            | MirInstr::Move { src, .. }
            | MirInstr::View { src, .. }
            | MirInstr::MutRef { src, .. } => vec![*src],
            MirInstr::BorrowEnd { token } => vec![*token],
            MirInstr::Drop { slot } => vec![*slot],
            MirInstr::StrLen { view, .. }
            | MirInstr::StrLenBytes { view, .. }
            | MirInstr::StrToOwned { view, .. }
            | MirInstr::CharToCodepoint { ch: view, .. } => vec![*view],
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

fn words_for(bits: usize) -> usize {
    bits.div_ceil(64)
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

fn set_bit(m: &mut [u64], i: usize) {
    m[i / 64] |= 1u64 << (i % 64);
}

fn bit_is_set(m: &[u64], i: usize) -> bool {
    m.get(i / 64)
        .map(|w| *w >> (i % 64) & 1 == 1)
        .unwrap_or(false)
}

fn union(a: &[u64], b: &[u64]) -> Vec<u64> {
    a.iter().zip(b.iter()).map(|(x, y)| x | y).collect()
}

fn union_into(a: &mut [u64], b: &[u64]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x |= *y;
    }
}

fn intersect(a: &mut [u64], b: &[u64]) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x &= *y;
    }
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
}
