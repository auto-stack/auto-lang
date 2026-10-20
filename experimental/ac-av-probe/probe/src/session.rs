//! Resident function-incremental session (PLAN-828 T-01 first case).
//!
//! The bounded full-path slice over the D798 pipeline: one resident session
//! holds the source snapshot (ac-syntax), the per-function declaration
//! records, and the current [`ProgramSnapshot`]. A single body edit goes
//! local-rescan/graft (ac-syntax) → changed-decl detection → per-function
//! frontend (real parser over the changed decl, unchanged decls carry only
//! their O(1) signature stubs) → [`auto_ac_prototype::verify::verify_function`]
//! → [`crate::hir_adapter::lower_function_unit`] → unit `verify_mir` → ABC
//! encode/decode/`verify_abc` → [`FunctionCode`] → snapshot re-link. Every
//! stage bump is a per-function counter; unchanged functions keep their
//! `Arc` HIR bodies and `Arc<FunctionCode>` — no re-read, re-decode,
//! re-verify or re-load runs for them.
//!
//! First-case bounds frozen by `docs/plans/attachments/828/contract-decision.md`:
//! single-edit-per-request body edits inside one existing top-level `fn`,
//! decl order and signatures unchanged, scalar profile, no store
//! persistence (warm session only — restore/GC arrive with T-07).

use crate::cache::{
    abc_unit_key, canonical_hir_function, caps_fingerprint, get_enveloped_opt, open_store,
    payload_fingerprint, put_enveloped, split_units, EnvelopeIdentity,
};
use crate::function_queries::FnEnv;
use crate::semrefs::content_version_of;
use crate::hir_adapter::{lower_checked, lower_function_unit};
use ac_syntax_core_adapter::{project_decl_strict, project_strict, CoreKinds};
use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::edit::incremental_reparse;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::outline::outline;
use ac_syntax_rt::parse::{parse_snapshot, ParseOutput};
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use auto_ac_prototype::atom_text::{Diagnostic, Span};
use auto_ac_prototype::hir;
use auto_ac_prototype::source::ast::{Block, FnItem, Item, Program as AstProgram};
use auto_ac_prototype::source::{
    compile_program_parts, compile_scalar_source, SourceProfile,
};
use auto_ac_prototype::verify::{verify_function, CheckedFunction};
use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::abc_verify::verify_abc;
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits, ExecTrace};
use auto_av_core::contract::ExecOutcome;
use auto_av_core::function_code::FunctionCode;
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::program_snapshot::ProgramSnapshot;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

/// Per-stage counter axes (PLAN-828 §5.5 names; the single `verify` axis
/// splits by layer as equivalent names, frozen in contract-decision.md):
/// `project|resolve|typecheck|hir_build|mir_lower|abc_emit|decode|
/// verify_hir|verify_mir|verify_abc|load` × function.
/// Per-function content-history bound (V-07: bounded reclamation — a
/// long edit session's history cannot grow without limit; the resident
/// revision's entry is never evicted).
pub const HISTORY_CAP: usize = 32;

pub const STAGES: [&str; 11] = [
    "project",
    "resolve",
    "typecheck",
    "hir_build",
    "mir_lower",
    "abc_emit",
    "decode",
    "verify_hir",
    "verify_mir",
    "verify_abc",
    "load",
];

/// Real per-request counters. Every bump happens at the actual call site of
/// the corresponding entry; nothing is pre-filled.
#[derive(Clone, Debug, Default)]
pub struct SessionCounters {
    map: BTreeMap<String, u64>,
}

impl SessionCounters {
    fn bump(&mut self, key: &str) {
        *self.map.entry(key.to_string()).or_insert(0) += 1;
    }

    fn set(&mut self, key: &str, value: u64) {
        self.map.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> u64 {
        self.map.get(key).copied().unwrap_or(0)
    }

    /// A stage's count for one function (0 = untouched).
    pub fn stage_of(&self, stage: &str, function: &str) -> u64 {
        self.get(&format!("{stage}:{function}"))
    }

    /// The structural gate view for one function: every stage's count. For
    /// an unchanged function on the pure body fast path this map must be
    /// all-zero (`unaffected_* == 0`).
    pub fn stages_of(&self, function: &str) -> BTreeMap<String, u64> {
        STAGES
            .iter()
            .map(|s| (s.to_string(), self.stage_of(s, function)))
            .collect()
    }

    /// Whole-module stage zeros: the session structurally never calls a
    /// whole-module frontend/lower/closure-roundtrip on the edit path, and
    /// these counters prove it (they only ever move off zero if that
    /// invariant is broken by a future change).
    pub fn whole_module(&self) -> BTreeMap<String, u64> {
        [
            "whole_module_frontend",
            "whole_module_lower",
            "whole_module_closure_roundtrip",
        ]
        .iter()
        .map(|k| (k.to_string(), self.get(k)))
        .collect()
    }
}

/// One function's resident record across revisions.
struct FnRecord {
    /// The def id text (bundle def identity, e.g. `d_main`).
    def_id: String,
    /// The source-level function name (e.g. `main`).
    name: String,
    /// Signature-only stub item for the synth program: the projected fn item
    /// with an empty body and all spans zeroed (stubs carry no real source
    /// positions; re-anchoring never sees out-of-range stub spans).
    stub_item: Item,
    /// The verified HIR body object — same `Arc` across revisions unless
    /// this function itself changed.
    hir_body: Arc<hir::Body>,
    /// Admitted execution code — same `Arc` across revisions unless this
    /// function itself changed.
    code: Arc<FunctionCode>,
    /// In-session content-addressed reuse (PLAN-828 T-04): canonical-HIR
    /// content version → the immutable verified (body, code) pair. A body
    /// edited back to a prior content reuses the SAME objects — no
    /// re-verify, re-lower or re-admission (disk layers arrive with T-07).
    history: BTreeMap<String, (Arc<hir::Body>, Arc<FunctionCode>)>,
}

/// Outcome of one edit request.
#[derive(Clone, Debug)]
pub struct EditOutcome {
    /// Whether the edit installed a new program snapshot revision.
    pub ok: bool,
    /// The session's current (post-decision) revision.
    pub revision: u64,
    /// Source name of the single changed function when the fast path ran.
    pub changed_fn: Option<String>,
    /// Exactly the def ids whose code was installed this edit.
    pub install_set: Vec<String>,
    /// Rendered diagnostics for rejected candidates.
    pub diagnostics: Vec<String>,
    /// Why a non-fast-path edit was refused (recorded, never silently
    /// degraded to a whole-module recompute).
    pub non_fast_path_reason: Option<String>,
}

impl EditOutcome {
    fn rejected(revision: u64, reason: String, diagnostics: Vec<String>) -> Self {
        EditOutcome {
            ok: false,
            revision,
            changed_fn: None,
            install_set: Vec::new(),
            diagnostics,
            non_fast_path_reason: Some(reason),
        }
    }
}

/// A fully prepared candidate (PLAN-828 T-06): the changed function's
/// verified artifacts plus the session-state advance, held OUT of the
/// session until [`FunctionIncrementalSession::commit`] accepts it.
pub struct PreparedEdit {
    base_revision: u64,
    changed_idx: usize,
    changed_name: String,
    content_version: String,
    hir_body: Arc<hir::Body>,
    code: Arc<FunctionCode>,
    new_src: String,
    new_snap: Snapshot,
    new_parse: ParseOutput,
}

/// The resident session. Single-threaded (ac-syntax state is `Rc`).
pub struct FunctionIncrementalSession {
    /// The entry function's def id text (e.g. `d_main`) — def-order identity
    /// used by lower/link.
    entry_def_id: String,
    caps: Vec<String>,
    src: String,
    grammar: GrammarProgram,
    kinds: CoreKinds,
    scanner: CompiledScanner,
    snap: Snapshot,
    parse: ParseOutput,
    fns: Vec<FnRecord>,
    /// Interface environment of the accepted revision (signatures + def
    /// ids + owner scope; PLAN-828 T-03).
    env: FnEnv,
    /// The ac-core declaration-context fingerprint the outstanding body
    /// credentials were verified under; a drift invalidates them.
    context_revision: String,
    current: Arc<ProgramSnapshot>,
    revision: u64,
    counters: SessionCounters,
    /// Optional persistent backing (PLAN-828 T-07): ABC units + the session
    /// manifest ride the 783 store; a warm restart re-admits them instead
    /// of recompiling.
    store: Option<(ac_artifact_probe::store::ManagedStore, String)>,
}

/// Warm-up: the one whole-module compile of the session's life (cold path;
/// NOT part of any edit's hot-path counters). `entry` follows the pipeline's
/// def-id convention (`"d_main"`), same as `Driver::request`.
pub fn warmup(
    src: &str,
    entry_def_id: &str,
    caps: &[String],
) -> Result<FunctionIncrementalSession, String> {
    let compiled =
        compile_scalar_source(src, "probe.at", caps).map_err(|ds| render_diags(src, &ds))?;
    let checked = &compiled.checked;
    let bundle = checked.bundle();
    let m = bundle
        .modules
        .first()
        .ok_or_else(|| "bundle has no module".to_string())?;

    // The whole-module lower + link entries key on the def id text
    // (`d_<name>`); `DefKind::Function::name` is the emitter's internal
    // symbol (`sf_<name>`), not the source name.
    m.defs
        .iter()
        .find(|d| d.id_text == entry_def_id)
        .ok_or_else(|| format!("entry def `{entry_def_id}` not found"))?;

    // Whole-module lower + per-unit admission (cold; mirrors the 798 path).
    let snapshot_hash = fnv64(src.as_bytes());
    let mir = lower_checked(&checked, &entry_def_id, snapshot_hash).map_err(|e| format!("{e:?}"))?;
    let units = split_units(&mir).map_err(|e| e)?;

    // ac-syntax resident state over the same source, under the versioned
    // scalar grammar (PLAN-828 T-02). The projected AST is the session's
    // name/declaration view (parity with the ac-core parser is pinned by the
    // scalar_projection_ differential suite and the frozen-load check).
    let grammar_src = std::fs::read_to_string(format!(
        "{}/../../ac-syntax/grammars/auto-scalar.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .map_err(|e| format!("read auto-scalar grammar: {e}"))?;
    let grammar = compile_grammar(&grammar_src).map_err(|e| format!("compile grammar: {e:?}"))?;
    let kinds = CoreKinds::from_program(&grammar);
    let scanner = CompiledScanner::compile(&grammar).map_err(|e| format!("scanner: {e:?}"))?;
    let store = Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, src);
    let parse = parse_snapshot(&grammar, &scanner, &snap, &Budget::default())
        .map_err(|e| format!("syntax parse: {e:?}"))?;
    if !parse.tree.diags.is_empty() {
        return Err(format!("syntax diags: {:?}", parse.tree.diags));
    }
    // Interface environment + the declaration-context fingerprint the
    // per-function credentials bind to (T-03). The context revision is
    // minted by ONE warm-up function verification against the same bundle —
    // every later edit must reproduce it or its credentials are stale.
    let env = FnEnv::from_bundle(bundle)?;
    let context_revision = {
        let entry_idx = m
            .defs
            .iter()
            .position(|d| d.id_text == entry_def_id)
            .ok_or_else(|| format!("entry def `{entry_def_id}` not found"))?;
        let cf = verify_function(bundle.clone(), entry_idx)
            .map_err(|ds| render_diags(src, &ds))?;
        cf.context_revision().to_string()
    };
    let projected =
        project_strict(&grammar, &kinds, &parse.tree, &snap).map_err(|e| format!("project: {e:?}"))?;
    let ast_fns: Vec<(String, FnItem)> = projected
        .items
        .into_iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some((f.name.name.clone(), f)),
            _ => None,
        })
        .collect();

    let mut fns: Vec<FnRecord> = Vec::new();
    let mut codes: Vec<Arc<FunctionCode>> = Vec::new();
    let mut ast_i = 0usize;
    for (di, d) in m.defs.iter().enumerate() {
        let hir::DefKind::Function { body, .. } = &d.kind else {
            return Err(format!("def `{}` is not a plain function", d.id_text));
        };
        // Function defs and AST fn items share source order; the record's
        // source name comes from the AST item.
        let item = ast_fns
            .get(ast_i)
            .cloned()
            .ok_or_else(|| format!("fewer AST items than function defs (def {})", d.id_text))?;
        ast_i += 1;
        let _ = di;
        let name = item.0;
        let unit = units
            .iter()
            .find(|u| u.funcs[0].name == d.id_text.as_str())
            .ok_or_else(|| format!("no MIR unit for def `{}`", d.id_text))?;
        // Unit admission is real even at warm-up: encode→decode→verify_abc.
        let flat = encode_abc(unit)
            .map_err(|e| format!("warm-up encode: {e:?}"))?
            .to_flat();
        let decoded = decode_abc(&flat).map_err(|e| format!("warm-up decode: {e:?}"))?;
        let loaded =
            verify_abc(decoded, &unit.owner).map_err(|e| format!("warm-up admit: {e:?}"))?;
        let code = Arc::new(
            FunctionCode::admit_unit(loaded, 0)
                .map_err(|e| format!("warm-up function code: {e}"))?,
        );
        let body_ref = m
            .bodies
            .get(body.0 as usize)
            .ok_or_else(|| format!("body {} out of range", body.0))?;
        let warm_dump = canonical_hir_function(m, *body);
        let warm_cv = content_version_of(&warm_dump);
        // ONE Arc shared by the resident record and the content history —
        // a later content hit must return the very same object.
        let body_arc = Arc::new(body_ref.clone());
        let mut history = BTreeMap::new();
        history.insert(warm_cv, (body_arc.clone(), code.clone()));  // first entry: within cap
        fns.push(FnRecord {
            def_id: d.id_text.clone(),
            name: name.clone(),
            stub_item: Item::Fn(stubify(item.1)),
            hir_body: body_arc,
            code: code.clone(),
            history,
        });
        codes.push(code);
    }

    // Def order must agree with the source decl order (first-case contract).
    let def_order: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
    let ast_order: Vec<&str> = ast_fns.iter().map(|(n, _)| n.as_str()).collect();
    if def_order != ast_order {
        return Err(format!(
            "first-case contract: def order {def_order:?} != source decl order {ast_order:?}"
        ));
    }

    let snapshot = Arc::new(
        ProgramSnapshot::link(codes, &entry_def_id, 0).map_err(|e| format!("warm-up link: {e}"))?,
    );

    Ok(FunctionIncrementalSession {
        kinds,
        entry_def_id: entry_def_id.to_string(),
        caps: caps.to_vec(),
        src: src.to_string(),
        grammar,
        scanner,
        snap,
        parse,
        fns,
        env,
        context_revision,
        current: snapshot,
        revision: 0,
        counters: SessionCounters::default(),
        store: None,
    })
}

/// COLD warm-up + persistent backing (PLAN-828 T-07): after the one
/// whole-module compile, every function's spanless ABC unit and the
/// session manifest are deposited in the store under the same identity
/// scheme the 798 driver uses — a later [`restore`] re-admits them.
pub fn warmup_persisted(
    src: &str,
    entry_def_id: &str,
    caps: &[String],
    store_root: &Path,
    project: &str,
) -> Result<FunctionIncrementalSession, String> {
    let mut session = warmup(src, entry_def_id, caps)?;
    let store = open_store(store_root).map_err(|e| format!("store open: {e:?}"))?;
    session.store = Some((store, project.to_string()));
    session
        .persist_all_units()
        .map_err(|e| format!("persist units: {e:?}"))?;
    session
        .persist_manifest().map_err(|e| format!("persist manifest: {e:?}"))?;
    Ok(session)
}

/// WARM RESTART (PLAN-828 T-07 / V-07): re-admit every function's code from
/// the store — each unit's envelope identity and payload name are
/// re-verified (`verify_abc`), nothing is trusted from disk — and link the
/// runnable snapshot. The declaration context (interface env, stub items,
/// content history seeds) is then materialized by ONE compile, counted as
/// `restore_materialize`, never as edit-path work. Any miss/tamper falls
/// back to a cold `warmup_persisted` (recorded in `restore_fallback`).
pub fn restore(
    src: &str,
    entry_def_id: &str,
    caps: &[String],
    store_root: &Path,
    project: &str,
) -> Result<FunctionIncrementalSession, String> {
    let cold = || warmup_persisted(src, entry_def_id, caps, store_root, project);
    let store = match open_store(store_root) {
        Ok(s) => s,
        Err(_) => return cold(),
    };
    let mut session = match warmup(src, entry_def_id, caps) {
        Ok(s) => s,
        Err(_) => return cold(),
    };
    let owner = session.fns_owner();
    let caps_fp = caps_fingerprint(caps);
    // Restore candidates keyed by the CURRENT source's content versions —
    // a source that changed since the units were written simply misses.
    let mut restored: Vec<Arc<FunctionCode>> = Vec::with_capacity(session.fns.len());
    for rec in session.fns.iter() {
        // Content version of the resident body (history seeds hold it).
        let body_cv = match session.resident_content_version(&rec.name) {
            Some(v) => v,
            None => return cold(),
        };
        let key = abc_unit_key(&owner, &rec.def_id, &body_cv, &caps_fp);
        let expect = EnvelopeIdentity::abc_bound(&owner, &rec.def_id, &body_cv);
        let payload = match get_enveloped_opt(&store, &key, "A09", &expect) {
            Ok(Some(b)) => b,
            _ => return cold(),
        };
        let decoded = match auto_av_core::abc::decode_abc(&payload) {
            Ok(d) => d,
            Err(_) => return cold(),
        };
        let unit_owner = decoded.module.owner.clone();
        let loaded = match verify_abc(decoded, &unit_owner) {
            Ok(l) => l,
            Err(_) => return cold(),
        };
        // Payload-name admission: the manifest-free restore derives names
        // from THIS source; a store row whose function is another def is a
        // fail-closed miss.
        if loaded.module().funcs[0].name != rec.def_id {
            return cold();
        }
        match FunctionCode::admit_unit(loaded, 0) {
            Ok(code) => restored.push(Arc::new(code)),
            Err(_) => return cold(),
        }
    }
    // Swap in the RESTORED code (the cold compile's units are discarded —
    // only their content versions/bodies seed the history).
    let codes: Vec<Arc<FunctionCode>> = restored;
    let snapshot = match ProgramSnapshot::link(codes, entry_def_id, 0) {
        Ok(sn) => Arc::new(sn),
        Err(_) => return cold(),
    };
    session.current = snapshot;
    session.counters.set("restore_units_admitted", session.fns.len() as u64);
    session.counters.set("restore_materialize", 1);
    session.store = Some((store, project.to_string()));
    Ok(session)
}

impl FunctionIncrementalSession {
    /// The current program snapshot (clone the `Arc` to pin a revision for
    /// a top-level run).
    pub fn current(&self) -> Arc<ProgramSnapshot> {
        self.current.clone()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn counters(&self) -> &SessionCounters {
        &self.counters
    }

    /// The resident source text at the accepted revision.
    pub fn src(&self) -> &str {
        &self.src
    }

    /// Resident per-function views for identity checks: (name, HIR body
    /// Arc, code Arc).
    pub fn resident(&self, name: &str) -> Option<(String, Arc<hir::Body>, Arc<FunctionCode>)> {
        self.fns
            .iter()
            .find(|f| f.name == name)
            .map(|f| (f.name.clone(), f.hir_body.clone(), f.code.clone()))
    }

    /// Run the CURRENT snapshot's entry.
    pub fn run(&self, limits: &ExecLimits, cancel: &CancelFlag) -> (ExecOutcome, ExecTrace) {
        execute_program(&self.current, limits, cancel)
    }

    /// The miss path of the content cache: verify the changed body
    /// (verifier-constructed credential, currency-checked), lower the
    /// single-function unit, admit it through the ABC admission and wrap
    /// the resident code handle. Counters bump at each real entry.
    /// Inner `Err` = candidate rejection (diagnostics); outer `Err` =
    /// infrastructure failure.
    #[allow(clippy::type_complexity)]
    fn compile_and_admit_changed(
        &mut self,
        bundle: &hir::Bundle,
        def_idx: usize,
        changed_name: &str,
        new_src: &str,
        new_src_hash: u64,
    ) -> Result<Result<(Arc<hir::Body>, Arc<FunctionCode>), Vec<String>>, String> {
        let checked_fn: CheckedFunction = match verify_function(bundle.clone(), def_idx) {
            Ok(c) => c,
            Err(ds) => {
                return Ok(Err(ds
                    .iter()
                    .map(|d| d.render("probe.at", new_src))
                    .collect()));
            }
        };
        // Credential currency: the body was verified under the SAME
        // declaration context as the outstanding ones — a drift here means
        // the interface gate above missed a context change (defense in
        // depth; reject rather than re-stamp).
        if !self
            .env
            .credential_current(&checked_fn, &self.context_revision)
        {
            return Ok(Err(vec![
                "body credential minted under a different declaration context".to_string(),
            ]));
        }
        self.counters.bump(&format!("verify_hir:{changed_name}"));

        let def_id = bundle.modules[0].defs[def_idx].id_text.clone();
        let unit = lower_function_unit(bundle, &def_id, &self.caps, new_src_hash)
            .map_err(|e| format!("lower: {e:?}"))?;
        self.counters.bump(&format!("mir_lower:{changed_name}"));
        let _verified = verify_mir(&unit).map_err(|e| format!("mir verify: {e:?}"))?;
        self.counters.bump(&format!("verify_mir:{changed_name}"));
        let flat = encode_abc(&unit)
            .map_err(|e| format!("abc encode: {e:?}"))?
            .to_flat();
        self.counters.bump(&format!("abc_emit:{changed_name}"));
        let decoded = decode_abc(&flat).map_err(|e| format!("abc decode: {e:?}"))?;
        self.counters.bump(&format!("decode:{changed_name}"));
        let loaded = verify_abc(decoded, &unit.owner).map_err(|e| format!("abc verify: {e:?}"))?;
        self.counters.bump(&format!("verify_abc:{changed_name}"));
        let code = Arc::new(
            FunctionCode::admit_unit(loaded, self.revision + 1)
                .map_err(|e| format!("function code: {e}"))?,
        );
        self.counters.bump(&format!("load:{changed_name}"));
        Ok(Ok((Arc::new(checked_fn.into_body()), code)))
    }

    /// Apply one source edit (old-source byte coordinates) and, if it is a
    /// pure single-function body edit, install the next program revision.
    /// A rejected edit leaves the session at its last accepted revision.
    /// Fused prepare+commit; the split flow is [`prepare_edit`] + [`commit`].
    pub fn edit(&mut self, start: u64, end: u64, insert: Vec<u8>) -> Result<EditOutcome, String> {
        let prepared = match self.prepare_edit(start, end, insert)? {
            Ok(p) => p,
            Err(out) => return Ok(out),
        };
        Ok(self.commit(prepared))
    }

    /// Bounded content-history insert (V-07): never evict the resident
    /// revision's own entry; evict an arbitrary oldest-ordered key when
    /// over the cap.
    fn history_insert(rec: &mut FnRecord, cv: String, pair: (Arc<hir::Body>, Arc<FunctionCode>)) {
        if rec.history.len() >= HISTORY_CAP && !rec.history.contains_key(&cv) {
            if let Some(victim) = rec
                .history
                .keys()
                .find(|k| {
                    !rec
                        .history
                        .get(*k)
                        .map(|(_, c)| Arc::ptr_eq(c, &rec.code))
                        .unwrap_or(false)
                })
                .cloned()
            {
                rec.history.remove(&victim);
            }
        }
        rec.history.entry(cv).or_insert_with(|| pair);
    }

    /// Total resident content-history entries (bounded by HISTORY_CAP ×
    /// functions; V-07 soak oracle).
    pub fn history_len(&self) -> usize {
        self.fns.iter().map(|f| f.history.len()).sum()
    }

    /// Source position mapping: absolute byte offset → (1-based line,
    /// 1-based column) over the accepted source (diagnostics-only axis).
    pub fn line_col(&self, offset: u64) -> (u64, u64) {
        ac_syntax_rt::position::LineIndex::build(&self.snap).line_col_utf8(&self.snap, offset)
    }

    /// The module owner id of the resident environment.
    fn fns_owner(&self) -> String {
        self.env.owner.clone()
    }

    /// The content version of a function's CURRENT code object (its
    /// history entry), if seeded.
    fn resident_content_version(&self, name: &str) -> Option<String> {
        let rec = self.fns.iter().find(|f| f.name == name)?;
        rec.history
            .iter()
            .find(|(_, (_, code))| Arc::ptr_eq(code, &rec.code))
            .map(|(cv, _)| cv.clone())
    }

    /// Persist every function's spanless ABC unit under its content
    /// version (same identity scheme as the 798 driver's A09 rows).
    fn persist_all_units(&self) -> Result<(), String> {
        let Some((store, project)) = self.store.as_ref() else {
            return Err("no store".to_string());
        };
        let owner = self.fns_owner();
        let caps_fp = caps_fingerprint(&self.caps);
        for rec in self.fns.iter() {
            let Some(cv) = self.resident_content_version(&rec.name) else {
                continue;
            };
            let mut spanless = rec.code.unit().module().clone();
            std::mem::take(&mut spanless.funcs[0].spans);
            let bytes = auto_av_core::abc::encode_abc(&spanless)
                .map_err(|e| format!("encode {}: {e:?}", rec.def_id))?
                .to_flat();
            let key = abc_unit_key(&owner, &rec.def_id, &cv, &caps_fp);
            let expect = EnvelopeIdentity::abc_bound(&owner, &rec.def_id, &cv);
            if let Err(e) = put_enveloped(store, &key, "A09", &expect, &bytes, vec![], Some(project)) {
                // Same tolerance as the 798 driver's stale-rebuild path
                // (F798-R12-03): the unit payload embeds source_snapshot,
                // so the same content version re-persisted across source
                // revisions can collide; the surviving row still serves
                // future restores (its own admission re-verifies).
                if format!("{e:?}").contains("Conflict") {
                } else {
                    return Err(format!("put {}: {e:?}", rec.def_id));
                }
            }
        }
        Ok(())
    }

    /// Persist the session manifest (the per-function directory a
    /// [`restore`] consults). Integrity is fail-closed downstream: units
    /// are re-verified against their own content-bound identities, and a
    /// wrong row name is a miss.
    fn persist_manifest(&self) -> Result<(), String> {
        let Some((store, project)) = self.store.as_ref() else {
            return Err("no store".to_string());
        };
        let owner = self.fns_owner();
        let units: Vec<serde_json::Value> = self
            .fns
            .iter()
            .map(|rec| {
                serde_json::json!({
                    "def_id": rec.def_id,
                    "name": rec.name,
                    "cv": self.resident_content_version(&rec.name),
                })
            })
            .collect();
        let body = serde_json::to_vec(&serde_json::json!({
            "format": 1,
            "owner": owner,
            "entry": self.entry_def_id,
            "caps_fp": caps_fingerprint(&self.caps),
            "units": units,
        }))
        .map_err(|e| e.to_string())?;
        let fp = payload_fingerprint(&body);
        let key = ac_artifact_probe::identity::QueryKey::new(
            &crate::cache::ABC_LAYER,
            vec![
                ("producer".into(), auto_av_core::contract::PRODUCER_ID.into()),
                ("owner".into(), owner.clone()),
                ("kind".into(), "session-manifest".into()),
                ("format".into(), "1".into()),
                ("entry".into(), self.entry_def_id.clone()),
                ("fp".into(), fp),
            ],
        );
        let expect = EnvelopeIdentity::manifest(&owner, 0);
        put_enveloped(store, &key, "A09", &expect, &body, vec![], Some(project))
            .map_err(|e| format!("manifest put: {e:?}"))?;
        Ok(())
    }

    /// PREPARE (PLAN-828 T-06): run the whole candidate chain (syntax
    /// increment → per-decl projection → frontend → verify → reuse-or-miss
    /// chain) WITHOUT touching the session's current revision. The returned
    /// [`PreparedEdit`] commits only via [`commit`]; a stale candidate
    /// (its base revision no longer current) never overwrites a newer
    /// result. Inner `Err` = candidate/non-fast-path rejection.
    pub fn prepare_edit(
        &mut self,
        start: u64,
        end: u64,
        insert: Vec<u8>,
    ) -> Result<Result<PreparedEdit, EditOutcome>, String> {
        let edit = Edit { start, end, insert };
        let new_snap = self
            .snap
            .edit(&EditRequest {
                base_revision: self.snap.revision(),
                edits: vec![edit.clone()],
            })
            .map_err(|e| format!("snapshot edit: {e:?}"))?;

        // Local rescan + declaration graft. A recorded fallback is an
        // explicit non-fast-path edit: refused (recorded), never silently
        // degraded to whole-module recompute.
        let inc = incremental_reparse(
            &self.grammar,
            &self.scanner,
            &self.parse,
            &self.snap,
            &new_snap,
            &[edit.clone()],
            &Budget::default(),
        )
        .map_err(|e| format!("incremental reparse: {e:?}"))?;
        self.counters.set("syntax_rescan_bytes", inc.rescan_bytes);
        self.counters
            .set("syntax_reparsed_decls", inc.reparsed_decls as u64);
        self.counters
            .set("syntax_reused_decls", inc.reused_decls as u64);
        if let Some(reason) = &inc.fallback {
            return Ok(Err(EditOutcome::rejected(
                self.revision,
                format!("non-fast-path: syntax fallback {reason:?}"),
                Vec::new(),
            )));
        }

        // Changed-decl detection: the (single) decl whose old span meets the
        // edit interval. Anything else — zero, multiple, or a decl-count or
        // name-sequence change — is out of the first-case fast path.
        let lo = start.min(end);
        let hi = (start.max(end)).max(lo + 1);
        let mut changed: Vec<usize> = Vec::new();
        for (i, d) in self.parse.tree.decls.iter().enumerate() {
            let (dlo, dhi) = self.parse.tree.abs_range(d.node);
            if dlo < hi && lo < dhi {
                changed.push(i);
            }
        }
        if changed.len() != 1 {
            return Ok(Err(EditOutcome::rejected(
                self.revision,
                format!(
                    "non-fast-path: edit meets {} declarations (single body edit supported)",
                    changed.len()
                ),
                Vec::new(),
            )));
        }
        let changed_idx = changed[0];
        let old_outline = outline(&self.grammar, &self.parse.tree, &self.snap)
            .map_err(|e| format!("old outline: {e:?}"))?;
        let new_outline = outline(&self.grammar, &inc.tree, &new_snap)
            .map_err(|e| format!("new outline: {e:?}"))?;
        let old_names: Vec<&str> = old_outline.iter().map(|o| o.name.as_str()).collect();
        let new_names: Vec<&str> = new_outline.iter().map(|o| o.name.as_str()).collect();
        if old_names != new_names {
            return Ok(Err(EditOutcome::rejected(
                self.revision,
                "non-fast-path: declaration set changed (add/remove/rename)".to_string(),
                Vec::new(),
            )));
        }
        if changed_idx >= inc.tree.decls.len() {
            return Ok(Err(EditOutcome::rejected(
                self.revision,
                "non-fast-path: changed decl missing in new tree".to_string(),
                Vec::new(),
            )));
        }
        let changed_name = new_outline[changed_idx].name.clone();
        if changed_name != self.fns[changed_idx].name {
            return Err(format!(
                "internal: decl {} name `{changed_name}` != record `{}`",
                changed_idx, self.fns[changed_idx].name
            ));
        }
        let new_src = String::from_utf8(
            new_snap
                .read_bytes(0, new_snap.len())
                .map_err(|e| format!("new source read: {e:?}"))?,
        )
        .map_err(|e| format!("utf8: {e:?}"))?;

        // Project ONLY the changed declaration (PLAN-828 T-02): the
        // incremental CST is the source of truth and unchanged declarations
        // are never re-projected. Spans come out in absolute new-source
        // coordinates.
        let projected_fn = match project_decl_strict(
            &self.grammar,
            &self.kinds,
            &inc.tree,
            &new_snap,
            changed_idx,
        ) {
            Ok(f) => f,
            Err(e) => {
                return Ok(Err(EditOutcome::rejected(
                    self.revision,
                    format!("candidate rejected: projection `{e:?}`"),
                    Vec::new(),
                )));
            }
        };
        if projected_fn.name.name != self.fns[changed_idx].name {
            return Err(format!(
                "internal: projected name `{}` != record `{}`",
                projected_fn.name.name, self.fns[changed_idx].name
            ));
        }
        self.counters.bump(&format!("project:{changed_name}"));

        // Synth program: the changed fn's projected item plus the unchanged
        // fns' signature-only stubs (O(1) per function — no parse, no
        // projection, no body work for them).
        let mut items: Vec<Item> = Vec::with_capacity(self.fns.len());
        for (i, rec) in self.fns.iter().enumerate() {
            if i == changed_idx {
                items.push(Item::Fn(projected_fn.clone()));
            } else {
                items.push(rec.stub_item.clone());
            }
        }
        let synth = AstProgram { items };

        // Per-function frontend over the synth program: the REAL resolve /
        // typecheck / emit+bind entries, carrying real work only for the
        // changed function.
        let parts = match compile_program_parts(&synth, &new_src, "probe.at", &self.caps, SourceProfile::Scalar)
        {
            Ok(p) => p,
            Err(ds) => {
                return Ok(Err(EditOutcome::rejected(
                    self.revision,
                    "candidate rejected: frontend diagnostics".to_string(),
                    ds.iter().map(|d| d.render("probe.at", &new_src)).collect(),
                )));
            }
        };
        for stage in ["resolve", "typecheck", "hir_build"] {
            self.counters.bump(&format!("{stage}:{changed_name}"));
        }
        self.counters.bump("synth_stub_walks");

        let def_idx = parts
            .bundle
            .modules
            .first()
            .ok_or_else(|| "synth bundle has no module".to_string())?
            .defs
            .iter()
            .position(|d| d.id_text == self.fns[changed_idx].def_id)
            .ok_or_else(|| {
                format!("def {} missing in synth bundle", self.fns[changed_idx].def_id)
            })?;

        // Fast-path gate through the interface environment (T-03): the
        // changed function's interface AND the whole declaration context
        // must be identical to the resident environment.
        let candidate_env = FnEnv::from_bundle(&parts.bundle)?;
        if !self
            .env
            .interface_unchanged(&candidate_env, &changed_name)
        {
            return Ok(Err(EditOutcome::rejected(
                self.revision,
                "non-fast-path: interface or declaration context changed".to_string(),
                Vec::new(),
            )));
        }

        // In-session content-addressed reuse (T-04): the candidate body's
        // canonical-HIR content version decides. A hit reuses the stored
        // VERIFIED (body, code) pair — the interface/context gates above
        // already bound it to this exact declaration context — with zero
        // re-verification, re-lowering or re-admission. A miss runs the
        // full per-function chain and deposits the new immutable pair.
        let synth_m = parts
            .bundle
            .modules
            .first()
            .ok_or_else(|| "synth bundle has no module".to_string())?;
        let def_body = match &synth_m.defs[def_idx].kind {
            hir::DefKind::Function { body, .. } => *body,
            _ => return Err("internal: changed def is not a function".to_string()),
        };
        let cv = content_version_of(&canonical_hir_function(synth_m, def_body));
        let (new_hir_body, new_code): (Arc<hir::Body>, Arc<FunctionCode>) =
            match self.fns[changed_idx].history.get(&cv) {
                Some((body, code)) => {
                    self.counters.bump(&format!("reuse_hit:{changed_name}"));
                    (body.clone(), code.clone())
                }
                None => {
                    let new_src_hash = fnv64(new_src.as_bytes());
                    match self.compile_and_admit_changed(
                        &parts.bundle,
                        def_idx,
                        &changed_name,
                        &new_src,
                        new_src_hash,
                    )? {
                        Ok(pair) => pair,
                        Err(diags) => {
                            return Ok(Err(EditOutcome::rejected(
                                self.revision,
                                "candidate rejected: verification diagnostics".to_string(),
                                diags,
                            )));
                        }
                    }
                }
            };

        Ok(Ok(PreparedEdit {
            base_revision: self.revision,
            changed_idx,
            changed_name,
            content_version: cv,
            hir_body: new_hir_body,
            code: new_code,
            new_src,
            new_snap,
            new_parse: ParseOutput {
                tree: inc.tree,
                tokens: inc.tokens,
                lex_diags: inc.lex_diags,
                grafted_decls: inc.reused_decls as usize,
            },
        }))
    }

    /// COMMIT (PLAN-828 T-06): install a prepared candidate as the next
    /// program revision. A candidate whose base revision is no longer
    /// current is refused — a late/slow candidate never overwrites a newer
    /// committed result (its diagnostics carry the refusal reason).
    pub fn commit(&mut self, prepared: PreparedEdit) -> EditOutcome {
        if prepared.base_revision != self.revision {
            return EditOutcome::rejected(
                self.revision,
                format!(
                    "stale candidate refused: prepared at revision {} but current revision is {}",
                    prepared.base_revision, self.revision
                ),
                Vec::new(),
            );
        }
        let next_revision = self.revision + 1;
        let changed_idx = prepared.changed_idx;
        let mut codes: Vec<Arc<FunctionCode>> = Vec::with_capacity(self.fns.len());
        for (i, rec) in self.fns.iter().enumerate() {
            if i == changed_idx {
                codes.push(prepared.code.clone());
            } else {
                codes.push(rec.code.clone());
            }
        }
        let next = match ProgramSnapshot::link(codes, &self.entry_def_id, next_revision) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                return EditOutcome::rejected(
                    self.revision,
                    format!("candidate rejected: link failure `{e}`"),
                    Vec::new(),
                );
            }
        };
        let PreparedEdit {
            base_revision,
            changed_name,
            content_version,
            hir_body,
            code,
            new_src,
            new_snap,
            new_parse,
            ..
        } = prepared;
        let _ = base_revision;
        let changed_rec = &mut self.fns[changed_idx];
        Self::history_insert(changed_rec, content_version, (hir_body.clone(), code.clone()));
        changed_rec.hir_body = hir_body;
        changed_rec.code = code;

        // T-07: durable deposit of the installed revision (units + the
        /// session manifest). A store failure is recorded but never
        /// rolls the installed revision back — memory state is primary.
        if self.store.is_some() {
            let persist = self.persist_all_units().and_then(|()| self.persist_manifest());
            if let Err(e) = persist {
                self.counters.set("persist_error", 1);
                self.counters.bump(&format!("persist_error_log:{e}"));
            }
        }

        // Advance the resident syntax state to the ACTUAL incremental
        // artifacts (sustained session, not a fresh rebuild).
        self.src = new_src;
        self.parse = new_parse;
        self.snap = new_snap;
        self.current = next;
        self.revision = next_revision;

        EditOutcome {
            ok: true,
            revision: next_revision,
            changed_fn: Some(changed_name),
            install_set: vec![self.fns[changed_idx].def_id.clone()],
            diagnostics: Vec::new(),
            non_fast_path_reason: None,
        }
    }
}

/// Signature-only stub for the synth program: empty body, all spans zeroed
/// (stubs carry no real source positions — their spans are never consumed
/// and re-anchoring never sees out-of-range stub spans).
fn stubify(mut f: FnItem) -> FnItem {
    let zero = Span { start: 0, end: 0 };
    f.name.span = zero;
    for p in &mut f.params {
        p.name.span = zero;
        p.ty.span = zero;
        p.span = zero;
    }
    f.ret.span = zero;
    f.body = Block {
        stmts: Vec::new(),
        span: zero,
    };
    f.span = zero;
    f
}

fn render_diags(src: &str, ds: &[Diagnostic]) -> String {
    ds.iter()
        .map(|d| d.render("probe.at", src))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
