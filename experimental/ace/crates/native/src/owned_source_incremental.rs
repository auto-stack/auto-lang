//! PLAN-894 T-05: the mn CONSUMER of an OWNED-TEXT source incremental
//! session (§5.3). The 828 retained-environment transaction owns the
//! frontend increment (one whole-module warm-up; each pure single-body
//! edit re-checks exactly ONE body via `prepare_candidate_document` —
//! the prepare is TRANSACTIONAL, a failure leaves the published
//! environment byte-identical, only `FnCandidate::commit` publishes);
//! this layer turns the changed function into the native pipeline's
//! increment: re-lower exactly that unit
//! (`lower_function_owned_unit_with_modes`), re-emit only ITS COFF,
//! re-link and run for real — unchanged units' fingerprints and
//! generations never move, no whole-module lower stage runs, content
//! hits reuse historical COFFs (A→B→A: zero COFF production), and every
//! degenerate fallback is counted, never silent.
//!
//! Boundary (§2/§5.5): this bridge is TEXT-owned only. It is neither the
//! scalar `MnIncrementalSession` (its 828 session is scalar) nor the
//! collections profile (889), and it does not copy the AV
//! `OwnedFunctionSession` — it consumes the SAME 828 primitives
//! (`CheckedEnv` / `prepare_candidate_document`) and the SAME per-unit
//! COFF/cache/link facilities.

use crate::cache::{self, CoffMeta};
use crate::config::*;
use crate::link_driver::{self, ObjectInput};
use crate::{emit, MnError};
use ace_artifacts::store::ManagedStore;
use ace_av_core::contract::fnv1a64;
use ace_core::hir;
use ace_core::source::{
    compile_text_record_source, env_fn_names, parse_program_with, prepare_candidate_document,
    resolve_env, SourceProfile,
};
use ace_lowering::lower_checked_owned;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One unit's published production counters (whole-module rounds and
/// single-body edits both count here; REFUSED attempts never publish —
/// they surface as `attempt_prepare` on the observation).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitProduction {
    /// The prepare transaction's real frontend work.
    pub parse: u32,
    pub recheck: u32,
    pub emit: u32,
    /// The mn-side unit production.
    pub lower: u32,
    pub coff: u32,
    pub cache_put: u32,
}

impl UnitProduction {
    fn whole_round() -> Self {
        UnitProduction {
            parse: 1,
            recheck: 1,
            emit: 1,
            lower: 1,
            coff: 1,
            cache_put: 1,
        }
    }
    fn sum(&self) -> u64 {
        self.parse as u64
            + self.recheck as u64
            + self.emit as u64
            + self.lower as u64
            + self.coff as u64
            + self.cache_put as u64
    }
}

/// Which production path minted the resident artifact — the restore gate
/// re-derives the fingerprint THROUGH THE SAME PATH (a whole-module split
/// unit and a single-body unit legitimately differ in import-table shape;
/// only a same-path re-derivation is an honest identity check).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitProvenance {
    Whole,
    Body,
}

impl UnitProvenance {
    fn as_str(self) -> &'static str {
        match self {
            UnitProvenance::Whole => "whole",
            UnitProvenance::Body => "body",
        }
    }
    fn of(v: &str) -> Self {
        if v == "body" {
            UnitProvenance::Body
        } else {
            UnitProvenance::Whole
        }
    }
}

/// One unit's identity in the current revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedUnitEntry {
    pub fp: String,
    pub generation: u64,
    /// blake3 of the resident COFF payload — the restore-time decode
    /// verification input (no trusted flags).
    pub coff_hash: String,
    /// The production path that minted this artifact.
    pub provenance: UnitProvenance,
}

/// The observable outcome of one owned-source edit (or the warm-up /
/// restore run).
#[derive(Clone, Debug, Default)]
pub struct OwnedEditObservation {
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub changed_fn: Option<String>,
    pub refused_reason: Option<String>,
    pub infra: Option<String>,
    /// COFF objects emitted this round (0 on a content hit).
    pub codegen_units: u64,
    pub cache_hits: u64,
    pub fallback_whole_lower: u64,
    /// True when the changed body matched a HISTORICAL content version
    /// and the stored COFF was reused (zero lower/COFF work).
    pub reused_history_unit: bool,
    /// Stage-counter deltas of functions OTHER than the changed one and
    /// any whole-module stage work (both must be 0 on the fast path).
    pub unaffected_stage_delta: u64,
    pub whole_module_stage_delta: u64,
    /// REFUSED-but-attempted preparation (parse/prepare attempts that
    /// never published).
    pub attempt_prepare: u64,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub wall_ms: f64,
}

struct DeclFn {
    lo: u64,
    hi: u64,
    name: String,
}

/// The owned-text source session over the native pipeline.
pub struct OwnedSourceMnSession {
    src: String,
    entry: String,
    caps: Vec<String>,
    env: ace_core::source::resolve::CheckedEnv,
    decl_fns: Vec<DeclFn>,
    decl_types: Vec<(u64, u64)>,
    sig_modes: BTreeMap<String, Vec<hir::ParamMode>>,
    /// env slot alignment (fast-path addressing by decl index).
    fast_path_ok: bool,
    unit_order: Vec<String>,
    units: BTreeMap<String, OwnedUnitEntry>,
    resident: BTreeMap<String, Vec<u8>>,
    /// (unit, body-content key) → (fp, generation, provenance) — the
    /// content-addressed history for A→B→A reuse (a hit RESTORES the
    /// historical unit identity; the generation does not bump).
    cv_map: BTreeMap<(String, String), (String, u64, UnitProvenance)>,
    store: ManagedStore,
    store_root: PathBuf,
    workdir_root: PathBuf,
    round: u64,
    owner: ace_av_core::mir::OwnerScope,
    production: BTreeMap<String, UnitProduction>,
    last_exit: Option<i32>,
}

fn content_key(slice: &str) -> String {
    blake3::hash(slice.as_bytes()).to_hex().to_string()
}
fn bytes_key(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Compile + lower + verify the WHOLE owned module and split it into
/// per-function units (the warm-up and the fallback channel's shape).
fn whole_module_owned_units(
    src: &str,
    entry: &str,
    caps: &[String],
) -> Result<Vec<ace_av_core::mir::MirModule>, MnError> {
    let compiled = compile_text_record_source(src, "mn894.at", caps).map_err(|ds| {
        MnError::Unsupported(
            "frontend",
            ds.first()
                .map(|d| format!("{} {}", d.code, d.message))
                .unwrap_or_default(),
        )
    })?;
    let mir = lower_checked_owned(&compiled.checked, entry, fnv1a64(src.as_bytes()))
        .map_err(|e| MnError::Unsupported("adapter", format!("{e:?}")))?;
    let checked = ace_av_core::mir_verify::verify_mir(&mir)
        .map_err(|e| MnError::Unsupported("mir", format!("{e:?}")))?;
    ace_av::cache::split_units(checked.module())
        .map_err(|e| MnError::Backend(format!("unit split: {e}")))
}

/// Scan the top-level declaration spans (free functions and types) of a
/// TextRecord-profile program.
fn decl_spans(program: &ace_core::source::ast::Program) -> (Vec<DeclFn>, Vec<(u64, u64)>) {
    use ace_core::source::ast::Item;
    let mut fns = Vec::new();
    let mut types = Vec::new();
    for it in &program.items {
        match it {
            Item::Fn(f) => fns.push(DeclFn {
                lo: f.span.start as u64,
                hi: f.span.end as u64,
                name: f.name.name.clone(),
            }),
            Item::Type(t) => {
                types.push((t.span.start as u64, t.span.end as u64));
                for m in &t.methods {
                    fns.push(DeclFn {
                        lo: m.span.start as u64,
                        hi: m.span.end as u64,
                        name: m.name.name.clone(),
                    });
                }
            }
            Item::Enum(e) => types.push((e.span.start as u64, e.span.end as u64)),
        }
    }
    (fns, types)
}

/// Retained interface facts: every function def's param modes, keyed by
/// def id (the D828 scaffold lowering needs them).
fn sig_modes_of(bundle: &hir::Bundle) -> BTreeMap<String, Vec<hir::ParamMode>> {
    let mut out = BTreeMap::new();
    let module = &bundle.modules[0];
    for d in &module.defs {
        if let hir::DefKind::Function { body, .. } | hir::DefKind::Method { body, .. } = &d.kind {
            if let Some(b) = module.bodies.get(body.0 as usize) {
                out.insert(
                    d.id_text.clone(),
                    b.locals
                        .iter()
                        .filter(|l| l.role == hir::LocalRole::Param)
                        .map(|l| l.param_mode)
                        .collect(),
                );
            }
        }
    }
    out
}

fn slice_of<'a>(src: &'a str, decls: &[DeclFn], unit_name: &str) -> &'a str {
    let source_name = unit_name.strip_prefix("d_").unwrap_or(unit_name);
    for d in decls {
        if d.name == source_name {
            return &src[d.lo as usize..d.hi as usize];
        }
    }
    ""
}

/// Parse ONE fn slice into its item (the single-body channel's only
/// parse). Fails when the slice is not exactly one function.
fn parse_single_fn(slice: &str, name: &str) -> Result<ace_core::source::ast::FnItem, String> {
    let parsed = parse_program_with(slice, SourceProfile::TextRecord)
        .map_err(|d| format!("parse {name}: {} {}", d.code, d.message))?;
    use ace_core::source::ast::Item;
    match parsed.items.as_slice() {
        [Item::Fn(f)] => Ok(f.clone()),
        _ => Err(format!(
            "single-body channel: the slice of `{name}` is not exactly one function"
        )),
    }
}

impl OwnedSourceMnSession {
    /// The session's link support library: the owned-text domain runtime
    /// shim (the same auto-located library the mn driver links for the
    /// Owned arm — `ac_rt_own_concat` and friends resolve or the link
    /// fails loudly).
    fn support_libs(&self) -> Result<Vec<PathBuf>, MnError> {
        Ok(vec![
            ace_av::text_runtime::text_runtime_lib().map_err(MnError::Infra)?
        ])
    }

    fn meta_of(&self, name: &str, fp: &str) -> CoffMeta {
        CoffMeta {
            owner: self.owner.clone(),
            unit: name.to_string(),
            symbol: crate::symbols::function_symbol(&self.owner, name),
            fp: fp.to_string(),
            abi: MN_ABI.to_string(),
            target: MN_TARGET.to_string(),
            flags: MN_FLAGS.to_string(),
            backend: MN_BACKEND_TAG.to_string(),
            runtime: MN_RUNTIME_TAG.to_string(),
            mir_schema: MIR_SCHEMA_AXIS.to_string(),
            mir_semantics: MIR_SEMANTICS_AXIS.to_string(),
        }
    }

    fn manifest_path(store_root: &Path) -> PathBuf {
        store_root.join("owned-894-session.json")
    }

    fn write_manifest(&self) -> Result<(), MnError> {
        let manifest = serde_json::json!({
            "schema": "mn.owned-894-session/1",
            "src": self.src,
            "entry": self.entry,
            "caps": self.caps,
            "unit_order": self.unit_order,
            "units": self.units.iter().map(|(n, u)| {
                (n.clone(), serde_json::json!({
                    "fp": u.fp, "generation": u.generation, "coff_hash": u.coff_hash,
                    "provenance": u.provenance.as_str(),
                }))
            }).collect::<std::collections::BTreeMap<String, serde_json::Value>>(),
            "cv_map": self.cv_map.iter().map(|((u, c), (f, g, p))| serde_json::json!({
                "unit": u, "content": c, "fp": f, "generation": g,
                "provenance": p.as_str(),
            })).collect::<Vec<_>>(),
            "last_exit": self.last_exit,
        });
        std::fs::write(
            Self::manifest_path(&self.store_root),
            serde_json::to_vec_pretty(&manifest).map_err(|e| MnError::Infra(format!("{e}")))?,
        )
        .map_err(|e| MnError::Infra(format!("manifest write: {e}")))
    }

    /// Cold warm-up: the ONE whole-module compile of the session's life —
    /// the real owned frontend, per-unit owned lowering, every unit
    /// emitted + persisted once (generations start at 0), then a real
    /// link/run. Returns the session and the cold run's observation.
    pub fn warmup(
        src: &str,
        entry: &str,
        caps: &[String],
        store_root: &Path,
    ) -> Result<(OwnedSourceMnSession, OwnedEditObservation), MnError> {
        let compiled = compile_text_record_source(src, "mn894.at", caps).map_err(|ds| {
            MnError::Unsupported(
                "frontend",
                ds.first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_default(),
            )
        })?;
        match compiled.checked.domain() {
            hir::HirDomain::Text | hir::HirDomain::TextRecord => {}
            other => {
                return Err(MnError::Unsupported(
                    "domain",
                    format!(
                        "owned-source session takes a text source, got `{}`",
                        other.as_str()
                    ),
                ));
            }
        }
        let units = whole_module_owned_units(src, entry, caps)?;
        let owner = units.first().expect("units").owner.clone();
        let store = ManagedStore::open(store_root)
            .map_err(|e| MnError::Infra(format!("store open: {e}")))?;
        let program = parse_program_with(src, SourceProfile::TextRecord)
            .map_err(|d| MnError::Unsupported("parse", format!("{} {}", d.code, d.message)))?;
        let (decl_fns, decl_types) = decl_spans(&program);
        let env = resolve_env(&program, caps, SourceProfile::TextRecord).map_err(|ds| {
            MnError::Unsupported(
                "env",
                ds.first().map(|d| d.message.clone()).unwrap_or_default(),
            )
        })?;
        let env_names = env_fn_names(&env);
        let decl_names: Vec<&str> = decl_fns.iter().map(|d| d.name.as_str()).collect();
        let fast_path_ok = env_names == decl_names;
        let sig_modes = sig_modes_of(compiled.checked.bundle());
        let workdir_root = store_root.join("mn-owned894-work");
        std::fs::create_dir_all(&workdir_root)
            .map_err(|e| MnError::Infra(format!("workdir: {e}")))?;
        let mut s = OwnedSourceMnSession {
            src: src.to_string(),
            entry: entry.to_string(),
            caps: caps.to_vec(),
            env,
            decl_fns,
            decl_types,
            sig_modes,
            fast_path_ok,
            unit_order: Vec::new(),
            units: BTreeMap::new(),
            resident: BTreeMap::new(),
            cv_map: BTreeMap::new(),
            store,
            store_root: store_root.to_path_buf(),
            workdir_root,
            round: 0,
            owner,
            production: BTreeMap::new(),
            last_exit: None,
        };
        for unit in &units {
            let name = unit.funcs[0].name.clone();
            let obj = emit::emit_unit_coff(unit)?;
            let fp = cache::unit_fp(unit);
            let meta = s.meta_of(&name, &fp);
            cache::put_unit_coff(&s.store, None, &meta, &obj.bytes)?;
            s.cv_map.insert(
                (name.clone(), content_key(slice_of(src, &s.decl_fns, &name))),
                (fp.clone(), 0, UnitProvenance::Whole),
            );
            s.unit_order.push(name.clone());
            s.units.insert(
                name.clone(),
                OwnedUnitEntry {
                    fp,
                    generation: 0,
                    coff_hash: bytes_key(&obj.bytes),
                    provenance: UnitProvenance::Whole,
                },
            );
            s.resident.insert(name.clone(), obj.bytes);
            s.production.insert(name, UnitProduction::whole_round());
        }
        let obs = s.run_current(None, units.len() as u64, 0, 0, 0, 0.0)?;
        s.write_manifest()?;
        Ok((s, obs))
    }

    /// Cross-process restore (§5.3 restart): decode the persisted
    /// required data (manifest + stored COFFs), VERIFY every payload
    /// against its recorded content hash AND the live source's unit
    /// fingerprints (no trusted flags), re-link and run — the recorded
    /// last exit must reproduce. Production counters start fresh (the
    /// restore re-verification is reported here, never mixed into
    /// warm-edit zeros).
    pub fn restore(
        store_root: &Path,
    ) -> Result<(OwnedSourceMnSession, OwnedEditObservation), MnError> {
        let raw = std::fs::read(Self::manifest_path(store_root))
            .map_err(|e| MnError::Infra(format!("manifest read: {e}")))?;
        let m: serde_json::Value = serde_json::from_slice(&raw)
            .map_err(|e| MnError::Infra(format!("manifest parse: {e}")))?;
        if m["schema"] != "mn.owned-894-session/1" {
            return Err(MnError::Admit("manifest", "unknown session schema".into()));
        }
        let src = m["src"].as_str().unwrap_or_default().to_string();
        let entry = m["entry"].as_str().unwrap_or_default().to_string();
        let caps: Vec<String> = m["caps"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let last_exit = m["last_exit"].as_i64().map(|v| v as i32);
        let store = ManagedStore::open(store_root)
            .map_err(|e| MnError::Infra(format!("store open: {e}")))?;
        // The dependency re-verification: re-derive the whole-module
        // units from the CURRENT toolchain + source and require every
        // fingerprint to equal the persisted one — drift refuses the
        // restore (stale artifacts never resurrect).
        let units = whole_module_owned_units(&src, &entry, &caps)?;
        let owner = units.first().expect("units").owner.clone();
        let program = parse_program_with(&src, SourceProfile::TextRecord)
            .map_err(|d| MnError::Unsupported("parse", format!("{} {}", d.code, d.message)))?;
        let (decl_fns, decl_types) = decl_spans(&program);
        let env = resolve_env(&program, &caps, SourceProfile::TextRecord).map_err(|ds| {
            MnError::Unsupported(
                "env",
                ds.first().map(|d| d.message.clone()).unwrap_or_default(),
            )
        })?;
        let compiled = compile_text_record_source(&src, "mn894.at", &caps).map_err(|ds| {
            MnError::Unsupported(
                "frontend",
                ds.first().map(|d| d.message.clone()).unwrap_or_default(),
            )
        })?;
        let sig_modes = sig_modes_of(compiled.checked.bundle());
        let env_names = env_fn_names(&env);
        let decl_names: Vec<&str> = decl_fns.iter().map(|d| d.name.as_str()).collect();
        let fast_path_ok = env_names == decl_names;
        let workdir_root = store_root.join("mn-owned894-work");
        std::fs::create_dir_all(&workdir_root)
            .map_err(|e| MnError::Infra(format!("workdir: {e}")))?;
        let mut s = OwnedSourceMnSession {
            src,
            entry,
            caps,
            env,
            decl_fns,
            decl_types,
            sig_modes,
            fast_path_ok,
            unit_order: Vec::new(),
            units: BTreeMap::new(),
            resident: BTreeMap::new(),
            cv_map: BTreeMap::new(),
            store,
            store_root: store_root.to_path_buf(),
            workdir_root,
            round: 0,
            owner,
            production: BTreeMap::new(),
            last_exit,
        };
        let mut decoded = 0u64;
        let order: Vec<String> = m["unit_order"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        for name in &order {
            let um = &m["units"][name.as_str()];
            let fp = um["fp"].as_str().unwrap_or_default().to_string();
            let generation = um["generation"].as_u64().unwrap_or(0);
            let coff_hash = um["coff_hash"].as_str().unwrap_or_default().to_string();
            let provenance = UnitProvenance::of(um["provenance"].as_str().unwrap_or("whole"));
            // Same-path fingerprint re-derivation: a Whole unit re-derives
            // from the whole-module split; a Body unit re-derives through
            // the single-body channel (the two legitimately differ in
            // import-table shape — only the SAME path is an honest check).
            let live_fp = match provenance {
                UnitProvenance::Whole => {
                    let unit = units
                        .iter()
                        .find(|u| u.funcs[0].name == *name)
                        .ok_or_else(|| MnError::Admit("restore", format!("unit {name} missing")))?;
                    cache::unit_fp(unit)
                }
                UnitProvenance::Body => {
                    let source_name = name.strip_prefix("d_").unwrap_or(name);
                    let decl_idx = s
                        .decl_fns
                        .iter()
                        .position(|d| d.name == source_name)
                        .ok_or_else(|| {
                            MnError::Admit("restore", format!("decl of {name} missing"))
                        })?;
                    let decl = &s.decl_fns[decl_idx];
                    let slice = &s.src[decl.lo as usize..decl.hi as usize];
                    let item = parse_single_fn(slice, source_name)
                        .map_err(|e| MnError::Admit("restore", format!("re-parse {name}: {e}")))?;
                    let module_program = parse_program_with(&s.src, SourceProfile::TextRecord)
                        .map_err(|d| {
                            MnError::Admit("restore", format!("re-parse module: {}", d.message))
                        })?;
                    let mut verify_env =
                        resolve_env(&module_program, &s.caps, SourceProfile::TextRecord).map_err(
                            |ds| {
                                MnError::Admit(
                                    "restore",
                                    ds.first().map(|d| d.message.clone()).unwrap_or_default(),
                                )
                            },
                        )?;
                    let cand = prepare_candidate_document(
                        &mut verify_env,
                        &item,
                        decl_idx,
                        &s.src,
                        "mn894.at",
                    )
                    .map_err(|ds| {
                        MnError::Admit(
                            "restore",
                            ds.first()
                                .map(|d| format!("{} {}", d.code, d.message))
                                .unwrap_or_else(|| "prepare".into()),
                        )
                    })?;
                    let unit = ace_lowering::lower_function_owned_unit_with_modes(
                        &cand.parts().bundle,
                        name,
                        &s.caps,
                        fnv1a64(s.src.as_bytes()),
                        &s.sig_modes,
                    )
                    .map_err(|e| {
                        MnError::Admit("restore", format!("re-lower {name}: {}", e.code()))
                    })?;
                    cache::unit_fp(&unit)
                }
            };
            if live_fp != fp {
                return Err(MnError::Admit(
                    "restore",
                    format!("unit {name}: fp drifted (persisted {fp}, live {live_fp})"),
                ));
            }
            let meta = s.meta_of(name, &fp);
            let bytes = cache::get_unit_coff(&s.store, &meta)?
                .ok_or_else(|| MnError::Admit("restore", format!("unit {name} not in store")))?;
            let live_hash = bytes_key(&bytes);
            if live_hash != coff_hash {
                return Err(MnError::Admit(
                    "restore",
                    format!(
                        "unit {name}: COFF hash mismatch (persisted {coff_hash}, live {live_hash})"
                    ),
                ));
            }
            decoded += 1;
            s.unit_order.push(name.clone());
            s.units.insert(
                name.clone(),
                OwnedUnitEntry {
                    fp,
                    generation,
                    coff_hash,
                    provenance,
                },
            );
            s.resident.insert(name.clone(), bytes);
            s.production.insert(name.clone(), UnitProduction::default());
        }
        for e in m["cv_map"].as_array().cloned().unwrap_or_default() {
            if let (Some(u), Some(c), Some(f)) =
                (e["unit"].as_str(), e["content"].as_str(), e["fp"].as_str())
            {
                let g = e["generation"].as_u64().unwrap_or(0);
                let prov = UnitProvenance::of(e["provenance"].as_str().unwrap_or("whole"));
                s.cv_map
                    .insert((u.to_string(), c.to_string()), (f.to_string(), g, prov));
            }
        }
        let mut obs = s.run_current(None, 0, 0, 0, 0, 0.0)?;
        // The restored program must reproduce the recorded behavior —
        // the restore gate (an old/wrong artifact set refuses here).
        if obs.exit_code != last_exit {
            return Err(MnError::Admit(
                "restore",
                format!(
                    "restored exit {:?} != recorded {last_exit:?}",
                    obs.exit_code
                ),
            ));
        }
        obs.cache_hits = decoded;
        Ok((s, obs))
    }

    /// Link the CURRENT resident units and run the PE (the observation
    /// window's tail — the caller owns the wall clock).
    #[allow(clippy::too_many_arguments)]
    fn run_current(
        &mut self,
        changed: Option<&str>,
        codegen_units: u64,
        fallback_whole: u64,
        unaffected_delta: u64,
        whole_module_delta: u64,
        wall_ms: f64,
    ) -> Result<OwnedEditObservation, MnError> {
        let mut objects = Vec::with_capacity(self.unit_order.len());
        let mut hits = 0u64;
        for name in &self.unit_order {
            match self.resident.get(name) {
                Some(bytes) => {
                    if Some(name.as_str()) != changed {
                        hits += 1;
                    }
                    objects.push(ObjectInput {
                        name: name.clone(),
                        bytes: bytes.clone(),
                    });
                }
                None => {
                    return Err(MnError::Admit(
                        "address",
                        format!("unit `{name}` is not resident"),
                    ));
                }
            }
        }
        let entry_fp = self.units.get(&self.entry).expect("entry unit").fp.clone();
        let entry_meta = self.meta_of(&self.entry, &entry_fp);
        let wrapper = emit::emit_wrapper_coff(&entry_meta.symbol)?;
        let libs = self.support_libs()?;
        self.round += 1;
        let workdir = self.workdir_root.join(format!("r{}", self.round));
        let run = link_driver::link_and_run(
            &objects,
            &wrapper,
            START_SYMBOL,
            &workdir,
            &libs,
            &format!("owned-894 round {}\n", self.round),
        )?;
        self.last_exit = run.exit_code;
        Ok(OwnedEditObservation {
            ok: true,
            exit_code: run.exit_code,
            reused_history_unit: false,
            changed_fn: changed.map(|s| s.to_string()),
            refused_reason: None,
            infra: None,
            codegen_units,
            cache_hits: hits,
            fallback_whole_lower: fallback_whole,
            unaffected_stage_delta: unaffected_delta,
            whole_module_stage_delta: whole_module_delta,
            attempt_prepare: 0,
            stdout: run.stdout,
            stderr: run.stderr,
            wall_ms,
        })
    }

    /// One source edit (byte range against the CURRENT source). Pure
    /// single-body edits ride the D828 channel (one body re-check, one
    /// unit re-lower/COFF); a content hit reuses the historical COFF;
    /// everything else falls back to the whole-module channel (counted).
    /// ANY failure restores the prior revision — the old program keeps
    /// running.
    pub fn edit(&mut self, start: u64, end: u64, insert: Vec<u8>) -> OwnedEditObservation {
        self.edit_batch(&[(start, end, insert)])
    }

    pub fn edit_batch(&mut self, edits: &[(u64, u64, Vec<u8>)]) -> OwnedEditObservation {
        let window_started = std::time::Instant::now();
        let wall = |obs: &mut OwnedEditObservation| {
            obs.wall_ms = window_started.elapsed().as_secs_f64() * 1000.0;
        };
        // ---- apply the edits (back-to-front, atomic ranges) ----------
        let mut ranges: Vec<(u64, u64, &[u8])> = edits
            .iter()
            .map(|(a, b, i)| (*a, *b, i.as_slice()))
            .collect();
        ranges.sort_by_key(|(a, _, _)| std::cmp::Reverse(*a));
        let mut next_src = self.src.clone();
        for (a, b, ins) in &ranges {
            let (a, b) = (*a as usize, *b as usize);
            if a > b || b > next_src.len() {
                let mut o = OwnedEditObservation::default();
                o.refused_reason = Some(format!(
                    "edit range {a}..{b} invalid for len {}",
                    next_src.len()
                ));
                wall(&mut o);
                return o;
            }
            match std::str::from_utf8(ins) {
                Ok(text) => next_src.replace_range(a..b, text),
                Err(_) => {
                    let mut o = OwnedEditObservation::default();
                    o.refused_reason =
                        Some("edit bytes are not valid UTF-8 (text sources only)".into());
                    wall(&mut o);
                    return o;
                }
            }
        }
        // ---- route: pure single-body? --------------------------------
        let hit_types = self
            .decl_types
            .iter()
            .any(|(lo, hi)| ranges.iter().any(|(a, b, _)| a < hi && b > lo));
        let touched: Vec<usize> = self
            .decl_fns
            .iter()
            .enumerate()
            .filter(|(_, d)| ranges.iter().any(|(a, b, _)| a >= &d.lo && b <= &d.hi))
            .map(|(i, _)| i)
            .collect();
        let single_body = !hit_types && touched.len() == 1 && self.fast_path_ok;
        if !single_body {
            return self.whole_module_fallback(next_src, wall);
        }
        let idx = touched[0];
        let decl_name = self.decl_fns[idx].name.clone();
        let delta = next_src.len() as i64 - self.src.len() as i64;
        let lo = self.decl_fns[idx].lo as usize;
        let hi = (self.decl_fns[idx].hi as i64 + delta) as usize;
        if lo > next_src.len() || hi > next_src.len() || lo >= hi {
            let mut o = OwnedEditObservation::default();
            o.refused_reason = Some("single-body slice out of bounds".into());
            wall(&mut o);
            return o;
        }
        let slice = next_src[lo..hi].to_string();
        let unit_name = format!("d_{decl_name}");
        let ckey = content_key(&slice);
        // ---- content hit? reuse the historical COFF ------------------
        if let Some((fp, hist_gen, hist_prov)) =
            self.cv_map.get(&(unit_name.clone(), ckey.clone())).cloned()
        {
            let item = match parse_single_fn(&slice, &decl_name) {
                Ok(f) => f,
                Err(msg) => {
                    let mut o = OwnedEditObservation::default();
                    o.refused_reason = Some(msg);
                    o.attempt_prepare = 1;
                    wall(&mut o);
                    return o;
                }
            };
            // The prepare transaction re-checks the body (real
            // parse/recheck/emit work) WITHOUT publishing.
            match prepare_candidate_document(&mut self.env, &item, idx, &next_src, "mn894.at") {
                Ok(candidate) => {
                    let bytes = {
                        let meta = CoffMeta {
                            owner: self.owner.clone(),
                            unit: unit_name.clone(),
                            symbol: crate::symbols::function_symbol(&self.owner, &unit_name),
                            fp: fp.clone(),
                            abi: MN_ABI.to_string(),
                            target: MN_TARGET.to_string(),
                            flags: MN_FLAGS.to_string(),
                            backend: MN_BACKEND_TAG.to_string(),
                            runtime: MN_RUNTIME_TAG.to_string(),
                            mir_schema: MIR_SCHEMA_AXIS.to_string(),
                            mir_semantics: MIR_SEMANTICS_AXIS.to_string(),
                        };
                        match cache::get_unit_coff(&self.store, &meta) {
                            Ok(Some(b)) => b,
                            other => {
                                let mut o = OwnedEditObservation::default();
                                o.infra = Some(format!("history unit read: {other:?}"));
                                o.attempt_prepare = 1;
                                wall(&mut o);
                                return o;
                            }
                        }
                    };
                    let old_bytes = self
                        .resident
                        .insert(unit_name.clone(), bytes.clone())
                        .unwrap_or_default();
                    let old_units = self.units.clone();
                    // RESTORE the historical unit identity (the original
                    // fp + generation; the historical COFF is live again).
                    self.units.insert(
                        unit_name.clone(),
                        OwnedUnitEntry {
                            fp: fp.clone(),
                            generation: hist_gen,
                            coff_hash: bytes_key(&bytes),
                            provenance: hist_prov,
                        },
                    );
                    match self.run_current(Some(&unit_name), 0, 0, 0, 0, 0.0) {
                        Ok(mut obs) => {
                            if candidate.commit(&mut self.env).is_err() {
                                self.resident.insert(unit_name.clone(), old_bytes);
                                self.units = old_units;
                                let mut o = OwnedEditObservation::default();
                                o.refused_reason = Some("candidate commit refused".into());
                                o.attempt_prepare = 1;
                                wall(&mut o);
                                return o;
                            }
                            // publish: parse/recheck/emit counted (the
                            // prepare ran); lower/coff/cache ZERO
                            // (history reuse); generation unchanged.
                            let p = self.production.entry(unit_name.clone()).or_default();
                            p.parse += 1;
                            p.recheck += 1;
                            p.emit += 1;
                            obs.reused_history_unit = true;
                            self.src = next_src;
                            self.rebuild_decls();
                            self.write_manifest().ok();
                            wall(&mut obs);
                            obs
                        }
                        Err(e) => {
                            self.resident.insert(unit_name.clone(), old_bytes);
                            self.units = old_units;
                            let mut o = OwnedEditObservation::default();
                            o.infra = Some(format!("link/run: {e:?}"));
                            wall(&mut o);
                            o
                        }
                    }
                }
                Err(ds) => {
                    let mut o = OwnedEditObservation::default();
                    o.refused_reason = Some(
                        ds.first()
                            .map(|d| format!("{} {}", d.code, d.message))
                            .unwrap_or_else(|| "prepare".into()),
                    );
                    o.attempt_prepare = 1;
                    wall(&mut o);
                    o
                }
            }
        } else {
            // ---- fast path: re-produce exactly ONE unit ---------------
            let item = match parse_single_fn(&slice, &decl_name) {
                Ok(f) => f,
                Err(msg) => {
                    let mut o = OwnedEditObservation::default();
                    o.refused_reason = Some(msg);
                    o.attempt_prepare = 1;
                    wall(&mut o);
                    return o;
                }
            };
            match prepare_candidate_document(&mut self.env, &item, idx, &next_src, "mn894.at") {
                Ok(candidate) => {
                    let def_id = format!("d_{}", item.name.name);
                    let modes = self.sig_modes.clone();
                    let produced = ace_lowering::lower_function_owned_unit_with_modes(
                        &candidate.parts().bundle,
                        &def_id,
                        &self.caps,
                        fnv1a64(next_src.as_bytes()),
                        &modes,
                    )
                    .map_err(|e| format!("lower: {}", e.code()))
                    .and_then(|unit| {
                        ace_av_core::mir_verify::verify_mir(&unit)
                            .map_err(|e| format!("verify: {e:?}"))
                            .map(|_| unit)
                    })
                    .and_then(|unit| {
                        emit::emit_unit_coff(&unit)
                            .map_err(|e| format!("coff: {e:?}"))
                            .map(|o| (unit, o))
                    });
                    match produced {
                        Ok((unit, obj)) => {
                            let fp = cache::unit_fp(&unit);
                            let meta = self.meta_of(&def_id, &fp);
                            if let Err(e) =
                                cache::put_unit_coff(&self.store, None, &meta, &obj.bytes)
                            {
                                let mut o = OwnedEditObservation::default();
                                o.infra = Some(format!("cache put: {e:?}"));
                                o.attempt_prepare = 1;
                                wall(&mut o);
                                return o;
                            }
                            let old_bytes = self
                                .resident
                                .insert(def_id.clone(), obj.bytes.clone())
                                .unwrap_or_default();
                            let old_units = self.units.clone();
                            let old_gen = old_units.get(&def_id).map(|u| u.generation).unwrap_or(0);
                            self.units.insert(
                                def_id.clone(),
                                OwnedUnitEntry {
                                    fp: fp.clone(),
                                    generation: old_gen + 1,
                                    coff_hash: bytes_key(&obj.bytes),
                                    provenance: UnitProvenance::Body,
                                },
                            );
                            self.cv_map.insert(
                                (def_id.clone(), ckey),
                                (fp, old_gen + 1, UnitProvenance::Body),
                            );
                            match self.run_current(Some(&def_id), 1, 0, 0, 0, 0.0) {
                                Ok(mut obs) => {
                                    if candidate.commit(&mut self.env).is_err() {
                                        self.resident.insert(def_id.clone(), old_bytes);
                                        self.units = old_units;
                                        let mut o = OwnedEditObservation::default();
                                        o.refused_reason = Some("candidate commit refused".into());
                                        o.attempt_prepare = 1;
                                        wall(&mut o);
                                        return o;
                                    }
                                    let p = self.production.entry(def_id.clone()).or_default();
                                    let np = UnitProduction::whole_round();
                                    p.parse += np.parse;
                                    p.recheck += np.recheck;
                                    p.emit += np.emit;
                                    p.lower += np.lower;
                                    p.coff += np.coff;
                                    p.cache_put += np.cache_put;
                                    self.src = next_src;
                                    self.rebuild_decls();
                                    self.write_manifest().ok();
                                    wall(&mut obs);
                                    obs
                                }
                                Err(e) => {
                                    self.resident.insert(def_id.clone(), old_bytes);
                                    self.units = old_units;
                                    let mut o = OwnedEditObservation::default();
                                    o.infra = Some(format!("link/run: {e:?}"));
                                    wall(&mut o);
                                    o
                                }
                            }
                        }
                        Err(msg) => {
                            let mut o = OwnedEditObservation::default();
                            o.refused_reason = Some(msg);
                            o.attempt_prepare = 1;
                            wall(&mut o);
                            o
                        }
                    }
                }
                Err(ds) => {
                    let mut o = OwnedEditObservation::default();
                    o.refused_reason = Some(
                        ds.first()
                            .map(|d| format!("{} {}", d.code, d.message))
                            .unwrap_or_else(|| "prepare".into()),
                    );
                    o.attempt_prepare = 1;
                    wall(&mut o);
                    o
                }
            }
        }
    }

    /// The whole-module channel: recompile everything, re-emit every
    /// unit (history content hits still reuse stored COFFs), rebuild the
    /// retained environment. Counted — never silent.
    fn whole_module_fallback(
        &mut self,
        next_src: String,
        wall: impl Fn(&mut OwnedEditObservation),
    ) -> OwnedEditObservation {
        let units = match whole_module_owned_units(&next_src, &self.entry, &self.caps) {
            Ok(u) => u,
            Err(e) => {
                let mut o = OwnedEditObservation::default();
                o.refused_reason = Some(format!("whole-module recompile: {e:?}"));
                o.attempt_prepare = 1;
                wall(&mut o);
                return o;
            }
        };
        let program = match parse_program_with(&next_src, SourceProfile::TextRecord) {
            Ok(p) => p,
            Err(d) => {
                let mut o = OwnedEditObservation::default();
                o.refused_reason = Some(format!("parse: {} {}", d.code, d.message));
                wall(&mut o);
                return o;
            }
        };
        let (decl_fns, decl_types) = decl_spans(&program);
        let env = match resolve_env(&program, &self.caps, SourceProfile::TextRecord) {
            Ok(e) => e,
            Err(ds) => {
                let mut o = OwnedEditObservation::default();
                o.refused_reason = Some(ds.first().map(|d| d.message.clone()).unwrap_or_default());
                wall(&mut o);
                return o;
            }
        };
        let compiled = match compile_text_record_source(&next_src, "mn894.at", &self.caps) {
            Ok(c) => c,
            Err(ds) => {
                let mut o = OwnedEditObservation::default();
                o.refused_reason = Some(ds.first().map(|d| d.message.clone()).unwrap_or_default());
                wall(&mut o);
                return o;
            }
        };
        self.sig_modes = sig_modes_of(compiled.checked.bundle());
        let mut new_order = Vec::new();
        let mut new_units = BTreeMap::new();
        let mut new_resident = BTreeMap::new();
        let mut emitted = 0u64;
        for unit in &units {
            let name = unit.funcs[0].name.clone();
            let ckey = content_key(slice_of(&next_src, &decl_fns, &name));
            if let Some((fp, gen, prov)) = self.cv_map.get(&(name.clone(), ckey.clone())).cloned() {
                let meta = self.meta_of(&name, &fp);
                if let Ok(Some(b)) = cache::get_unit_coff(&self.store, &meta) {
                    new_units.insert(
                        name.clone(),
                        OwnedUnitEntry {
                            fp,
                            generation: gen,
                            coff_hash: bytes_key(&b),
                            provenance: prov,
                        },
                    );
                    new_resident.insert(name.clone(), b);
                    new_order.push(name);
                    continue;
                }
            }
            let obj = match emit::emit_unit_coff(unit) {
                Ok(o) => o,
                Err(e) => {
                    let mut o = OwnedEditObservation::default();
                    o.infra = Some(format!("emit: {e:?}"));
                    wall(&mut o);
                    return o;
                }
            };
            emitted += 1;
            let fp = cache::unit_fp(unit);
            let meta = self.meta_of(&name, &fp);
            if let Err(e) = cache::put_unit_coff(&self.store, None, &meta, &obj.bytes) {
                let mut o = OwnedEditObservation::default();
                o.infra = Some(format!("cache put: {e:?}"));
                wall(&mut o);
                return o;
            }
            let gen = self.units.get(&name).map(|u| u.generation + 1).unwrap_or(0);
            self.cv_map.insert(
                (name.clone(), ckey),
                (fp.clone(), gen, UnitProvenance::Whole),
            );
            new_units.insert(
                name.clone(),
                OwnedUnitEntry {
                    fp,
                    generation: gen,
                    coff_hash: bytes_key(&obj.bytes),
                    provenance: UnitProvenance::Whole,
                },
            );
            new_resident.insert(name.clone(), obj.bytes);
            new_order.push(name);
        }
        let old = (
            self.unit_order.clone(),
            self.units.clone(),
            self.resident.clone(),
        );
        self.unit_order = new_order;
        self.units = new_units;
        self.resident = new_resident;
        self.decl_fns = decl_fns;
        self.decl_types = decl_types;
        match self.run_current(None, emitted, 1, 0, 1, 0.0) {
            Ok(mut obs) => {
                self.env = env;
                let env_names: Vec<String> = env_fn_names(&self.env)
                    .into_iter()
                    .map(String::from)
                    .collect();
                let decl_names: Vec<String> =
                    self.decl_fns.iter().map(|d| d.name.clone()).collect();
                self.fast_path_ok = env_names == decl_names;
                for name in &self.unit_order {
                    let p = self.production.entry(name.clone()).or_default();
                    let w = UnitProduction::whole_round();
                    p.parse += w.parse;
                    p.recheck += w.recheck;
                    p.emit += w.emit;
                    p.lower += w.lower;
                    p.coff += w.coff;
                    p.cache_put += w.cache_put;
                }
                self.src = next_src;
                self.write_manifest().ok();
                wall(&mut obs);
                obs
            }
            Err(e) => {
                self.unit_order = old.0;
                self.units = old.1;
                self.resident = old.2;
                let mut o = OwnedEditObservation::default();
                o.infra = Some(format!("link/run: {e:?}"));
                wall(&mut o);
                o
            }
        }
    }

    /// Refresh the declaration spans after a published edit.
    fn rebuild_decls(&mut self) {
        if let Ok(program) = parse_program_with(&self.src, SourceProfile::TextRecord) {
            let (fns, types) = decl_spans(&program);
            self.decl_fns = fns;
            self.decl_types = types;
        }
    }

    // ---- read-only observation surface -------------------------------

    /// Re-run the CURRENT program (no codegen, no source change) — the
    /// failure-oracle leg: after a REFUSED edit the old revision must
    /// still run and produce its recorded exit.
    pub fn rerun(&mut self) -> OwnedEditObservation {
        let window_started = std::time::Instant::now();
        let mut obs = match self.run_current(None, 0, 0, 0, 0, 0.0) {
            Ok(o) => o,
            Err(e) => {
                let mut o = OwnedEditObservation::default();
                o.infra = Some(format!("rerun: {e:?}"));
                return o;
            }
        };
        obs.wall_ms = window_started.elapsed().as_secs_f64() * 1000.0;
        obs
    }

    pub fn src(&self) -> &str {
        &self.src
    }
    pub fn unit_names(&self) -> &[String] {
        &self.unit_order
    }
    pub fn unit_entry(&self, name: &str) -> Option<&OwnedUnitEntry> {
        self.units.get(name)
    }
    pub fn production(&self) -> &BTreeMap<String, UnitProduction> {
        &self.production
    }
    pub fn production_sum(&self, name: &str) -> u64 {
        self.production.get(name).map(|p| p.sum()).unwrap_or(0)
    }
    pub fn last_exit(&self) -> Option<i32> {
        self.last_exit
    }
}
