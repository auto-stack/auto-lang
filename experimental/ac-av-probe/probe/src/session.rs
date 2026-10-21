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
    abc_unit_key, canonical_hir_function, caps_fingerprint, decode_unit_sidecar,
    encode_unit_sidecar, get_enveloped_opt, merge_spans, open_store, payload_fingerprint,
    put_enveloped, split_units, unit_sidecar_usable_for_current, unit_span_sidecar_key,
    unit_span_sidecar_repair_key, EnvelopeIdentity, UnitSidecar,
};
use crate::driver::front_door_admit;
use crate::function_queries::FnEnv;
use crate::hir_adapter::{lower_checked, lower_function_unit};
use crate::semrefs::content_version_of;
use ac_syntax_core_adapter::{project_decl_strict, project_strict, CoreKinds};
use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::edit::incremental_reparse;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::outline::outline;
use ac_syntax_rt::parse::{parse_snapshot, ParseOutput};
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::hir;
use auto_ac_prototype::source::ast::{FnItem, Item};
use auto_ac_prototype::source::{
    compile_scalar_source, parse_program_with, prepare_candidate_document, resolve_env, CheckedEnv,
    FnCandidate, SourceProfile,
};
use auto_ac_prototype::verify::{verify_function, CheckedFunction};
use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::abc_verify::verify_abc;
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits, ExecTrace};
use auto_av_core::contract::{ExecOutcome, SpanInfo};
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

    fn add(&mut self, key: &str, value: u64) {
        if value != 0 {
            *self.map.entry(key.to_string()).or_insert(0) += value;
        }
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
    /// The verified HIR body object — same `Arc` across revisions unless
    /// this function itself changed.
    hir_body: Arc<hir::Body>,
    /// Admitted execution code — same `Arc` across revisions unless this
    /// function itself changed.
    code: Arc<FunctionCode>,
    /// Current-source declaration origin (byte offset) of this function.
    origin: u64,
    /// Declaration origin in the source revision this function's UNIT
    /// spans were lowered from — diagnostics-only span mapping delta =
    /// `origin - mint_origin` (F-828-R1-04/R2: unchanged functions' trap
    /// spans must track the CURRENT source without touching code payloads).
    mint_origin: u64,
    /// Piecewise alignment map for this function's code: mint bytes →
    /// current offsets (F-828-R1-04/R8a). Decl-LOCAL coordinates; the
    /// absolute view adds the function's origins. A re-minted function
    /// resets to identity; reused code accumulates edits.
    span_map: auto_av_core::span_map::SpanSegments,
    /// In-session content-addressed reuse (PLAN-828 T-04): content version
    /// → (verified body, code, mint origin, the code's OWN accumulated
    /// span map). A body edited back to prior content reuses the SAME
    /// objects and their OWN mint baseline AND source map
    /// (F-828-R1-04/R3a).
    history: BTreeMap<
        String,
        (
            Arc<hir::Body>,
            Arc<FunctionCode>,
            u64,
            auto_av_core::span_map::SpanSegments,
            String,
        ),
    >,
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
    /// MINTING session's identity — a candidate handed to another session
    /// is refused even at equal revision numbers (F-828-R1-05).
    session_id: u64,
    /// PLAN-828 T-02: the ac-core candidate transaction object. The
    /// PreparedEdit OWNS the candidate — the environment slot publishes
    /// only at [`FunctionIncrementalSession::commit`], together with the
    /// root/source/revision advance (D-03 single publication point).
    candidate: FnCandidate,
    /// Re-minted (miss) vs reused-from-history (content hit): span
    /// baselines reset only on a mint (F-828-R1-04/R2a F04-b).
    reminted: bool,
    /// The origin baseline of the candidate's code object (its own mint
    /// source, whether fresh or from history).
    mint_origin: u64,
    /// The edit's own region (old-source start, replaced length, inserted
    /// length) — the source-map delta every non-reminted function applies.
    edit_region: (u64, u64, u64),
    edit_bytes: Vec<u8>,
    /// The changed function's code-object span map (history entry's own on
    /// a hit; empty on a miss).
    span_map: auto_av_core::span_map::SpanSegments,
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

/// Process-wide session counter for unforgeable cross-session candidate
/// binding (F-828-R1-05): a `PreparedEdit` minted by one session is
/// refused by another even at equal revision numbers.
static NEXT_SESSION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

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
    /// PLAN-828 T-02: the RETAINED checked environment (ac-core
    /// `CheckedEnv`). The hot edit path runs the real single-body
    /// re-check/emission transaction against it
    /// ([`prepare_candidate_document`]); its slot publishes only at commit
    /// (D-03) and close() releases it explicitly.
    env_core: CheckedEnv,
    /// The ac-core declaration-context fingerprint the outstanding body
    /// credentials were verified under; a drift invalidates them.
    context_revision: String,
    current: Arc<ProgramSnapshot>,
    revision: u64,
    /// Unique session identity bound into every prepared candidate.
    session_id: u64,
    counters: SessionCounters,
    /// Optional persistent backing (PLAN-828 T-07): ABC units + the session
    /// manifest ride the 783 store; a warm restart re-admits them instead
    /// of recompiling.
    store: Option<(ac_artifact_probe::store::ManagedStore, String)>,
    /// Terminal CLOSE state (PLAN-828 T-06/F-05b): a closed session
    /// refuses new work and has released its bounded content history and
    /// store handle; snapshots pinned elsewhere stay alive and correct.
    closed: bool,
}

/// Warm-up: the one whole-module compile of the session's life (cold path;
/// NOT part of any edit's hot-path counters). `entry` follows the pipeline's
/// def-id convention (`"d_main"`), same as `Driver::request`.
pub fn warmup(
    src: &str,
    entry_def_id: &str,
    caps: &[String],
) -> Result<FunctionIncrementalSession, String> {
    // Input admission BEFORE any allocation (F-828-R1-09): the same frozen
    // size/nesting front door the 798 driver runs, not a session exemption.
    front_door_admit(src).map_err(|d| d.message)?;
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
    let mir =
        lower_checked(&checked, &entry_def_id, snapshot_hash).map_err(|e| format!("{e:?}"))?;
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
        let cf = verify_function(bundle.clone(), entry_idx).map_err(|ds| render_diags(src, &ds))?;
        cf.context_revision().to_string()
    };
    // PLAN-828 T-02: the RETAINED checked environment — the hot edit path's
    // single-body re-check/emission transaction runs against it. Built once
    // at warm-up (cold path) and published only through candidate commits.
    let core_program =
        parse_program_with(src, SourceProfile::Scalar).map_err(|d| d.render("probe.at", src))?;
    let env_core = resolve_env(&core_program, caps, SourceProfile::Scalar)
        .map_err(|ds| render_diags(src, &ds))?;
    let projected = project_strict(&grammar, &kinds, &parse.tree, &snap)
        .map_err(|e| format!("project: {e:?}"))?;
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
        // Intrinsic declarations (trace marks) are part of the preserved
        // support surface (F-828-R1-08): they ride the unit/lower import
        // convention and hold no function record of their own.
        let hir::DefKind::Function { body, .. } = &d.kind else {
            if matches!(&d.kind, hir::DefKind::Intrinsic { .. }) {
                continue;
            }
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
        history.insert(warm_cv, history_entry(body_arc.clone(), code.clone()));
        fns.push(FnRecord {
            def_id: d.id_text.clone(),
            name: name.clone(),
            hir_body: body_arc,
            code: code.clone(),
            origin: 0,
            mint_origin: 0,
            span_map: auto_av_core::span_map::SpanSegments::default(),
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
    // ...and with the retained checked environment's slots: the edit path
    // addresses env slots by declaration index (T-02 binding, verified
    // once here; the re-check's own name guard is the runtime backstop).
    let env_order = auto_ac_prototype::source::env_fn_names(&env_core);
    if env_order != def_order {
        return Err(format!(
            "checked-env slot order {env_order:?} != declaration order {def_order:?}"
        ));
    }

    // Per-function declaration origins of the accepted source (span
    // mapping baseline; decl order == fn order was asserted above).
    for (i, d) in parse.tree.decls.iter().enumerate() {
        let (d0, d1) = parse.tree.abs_range(d.node);
        fns[i].origin = d0;
        fns[i].mint_origin = d0;
        let fresh_map = auto_av_core::span_map::SpanSegments::identity_range(d0, d1, d0);
        let text = src[d0 as usize..d1 as usize].to_string();
        fns[i].span_map = fresh_map.clone();
        if let Some(e) = fns[i].history.values_mut().next() {
            e.2 = d0;
            e.3 = fresh_map;
            e.4 = text;
        }
    }

    let snapshot = Arc::new(
        ProgramSnapshot::link(codes, &entry_def_id, 0).map_err(|e| format!("warm-up link: {e}"))?,
    );

    let session_id = NEXT_SESSION_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        env_core,
        context_revision,
        current: snapshot,
        revision: 0,
        session_id,
        counters: SessionCounters::default(),
        store: None,
        closed: false,
    })
}

/// COLD warm-up + persistent backing (PLAN-828 T-07): after the one
/// whole-module compile, every function's ABC unit (spanless payload +
/// unit-bound span sidecar, PLAN-828 F-10) and the session manifest are
/// deposited in the store under the same identity scheme the 798 driver
/// uses — a later [`restore`] re-admits them.
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
        .persist_manifest()
        .map_err(|e| format!("persist manifest: {e:?}"))?;
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
    // `anchors` collects each unit's mint→current SPAN MAP plus its true
    // mint origin/text (F11-S01) for the record/snapshot seeding below.
    let mut restored: Vec<Arc<FunctionCode>> = Vec::with_capacity(session.fns.len());
    let mut anchors: Vec<(auto_av_core::span_map::SpanSegments, u64, Vec<u8>)> =
        Vec::with_capacity(session.fns.len());
    for (idx, rec) in session.fns.iter().enumerate() {
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
        let mut decoded = match auto_av_core::abc::decode_abc(&payload) {
            Ok(d) => d,
            Err(_) => return cold(),
        };
        // F10-S01: a CONTROLLED shape check before any indexing — a
        // zero-function payload decodes fine and must fail closed here,
        // not panic; `verify_abc` below still rejects deeper shape
        // issues authoritatively.
        let Some(shape) = decoded.module.funcs.first().cloned() else {
            return cold();
        };
        // F11-S01 provenance evidence: the session's own cold compile of
        // the CURRENT source (the resident code until the swap below) —
        // its span table and decl slice are exactly what a same-source
        // row must agree with.
        let (d0, d1) = session
            .parse
            .tree
            .abs_range(session.parse.tree.decls[idx].node);
        let cur_decl = src[d0 as usize..d1 as usize].as_bytes();
        let cur_fn = rec.code.unit().module().funcs.first().cloned();
        // Re-attach the unit's mint span table (PLAN-828 F-10): the
        // payload row is spanless BY DESIGN. The sidecar is probed
        // primary → repair (F10-S02) under the FULL applicability
        // standard — decode, span coverage, and provenance binding to
        // the CURRENT source (mint UTF-8/frame/re-anchoring for
        // generation 2; exact span-table equality with the current
        // compile for the legacy spans-only generation) — a row failing
        // ANY of it is skipped, and only then does the restore fail
        // closed; never a span-blind or stale-coordinate restore.
        let sid = EnvelopeIdentity::unit_span_sidecar(&owner, &rec.def_id, &body_cv);
        let skey = unit_span_sidecar_key(&owner, &rec.def_id, &body_cv);
        let rkey = unit_span_sidecar_repair_key(&owner, &rec.def_id, &body_cv);
        let row_usable = |b: &[u8]| match &cur_fn {
            Some(cur) => unit_sidecar_usable_for_current(b, &shape, cur, d0, cur_decl),
            None => false,
        };
        let sidecar_bytes = match get_enveloped_opt(&store, &skey, "A09", &sid) {
            Ok(Some(b)) if row_usable(&b) => b,
            _ => match get_enveloped_opt(&store, &rkey, "A09", &sid) {
                Ok(Some(b)) if row_usable(&b) => b,
                _ => return cold(),
            },
        };
        let sc = match decode_unit_sidecar(&sidecar_bytes) {
            Ok(sc) => sc,
            Err(_) => return cold(), // usable implies decodable; fail closed anyway
        };
        merge_spans(&mut decoded.module, &sc.spans);
        // F11-S01: re-anchor the unit's MINT spans to the CURRENT decl —
        // a generation-2 sidecar carries its mint range/text (a
        // repositioned source still restores CURRENT coordinates); the
        // legacy spans-only generation passed the exact span-table
        // evidence check above, so its coordinates ARE current-source
        // coordinates (identity).
        let (map, mint_origin, mint_text) = match sc.mint {
            Some((m_d0, m_d1, m_text)) => {
                match auto_av_core::span_map::SpanSegments::from_texts(
                    m_d0, m_d1, d0, &m_text, cur_decl,
                ) {
                    Some(m) => (m, m_d0, m_text),
                    None => return cold(), // usable implies re-anchorable; fail closed anyway
                }
            }
            None => (
                auto_av_core::span_map::SpanSegments::identity_range(d0, d1, d0),
                d0,
                cur_decl.to_vec(),
            ),
        };
        // The mint text seeds future content-hit re-anchoring — it must
        // round-trip as a source string.
        if std::str::from_utf8(&mint_text).is_err() {
            return cold();
        }
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
            Ok(code) => {
                restored.push(Arc::new(code));
                anchors.push((map, mint_origin, mint_text));
            }
            Err(_) => return cold(),
        }
    }
    // Swap in the RESTORED code (the cold compile's units are discarded —
    // only their content versions/bodies seed the history).
    let codes: Vec<Arc<FunctionCode>> = restored;
    // The per-function anchors back BOTH the served snapshot and the
    // resident records: the first run after restore maps through its own
    // snapshot (PLAN-828 F-10), never the empty-map accident of bare
    // `link`.
    let maps: Vec<auto_av_core::span_map::SpanSegments> =
        anchors.iter().map(|a| a.0.clone()).collect();
    let snapshot =
        match ProgramSnapshot::link_with_maps(codes.clone(), entry_def_id, 0, maps.clone()) {
            Ok(sn) => Arc::new(sn),
            Err(_) => return cold(),
        };
    // F-828-R1-07: the RESTORED code objects become the resident ones —
    // records AND the content-history entries — so a later commit keeps
    // every unchanged function's restored object identity instead of
    // swapping back to the cold-compiled ones. History entries carry the
    // unit's TRUE mint origin/text (F11-S01), not the current slice.
    for (i, code) in codes.iter().enumerate() {
        let (map, mint_d0, mint_text) = (anchors[i].0.clone(), anchors[i].1, anchors[i].2.clone());
        let rec = &mut session.fns[i];
        let body = rec.hir_body.clone();
        rec.span_map = map.clone();
        if let Some(entry) = rec.history.iter_mut().next() {
            let Ok(text) = String::from_utf8(mint_text) else {
                return cold();
            };
            *entry.1 = (body.clone(), code.clone(), mint_d0, map, text);
        }
        rec.code = code.clone();
    }
    session.current = snapshot;
    session
        .counters
        .set("restore_units_admitted", session.fns.len() as u64);
    session.counters.set("restore_materialize", 1);
    session.store = Some((store, project.to_string()));
    Ok(session)
}

impl FunctionIncrementalSession {
    /// The current program snapshot (clone the `Arc` to pin a revision for
    /// a top-level run).
    /// The current program snapshot (clone the `Arc` to pin a revision for
    /// a top-level run). After [`Self::close`] this is a lightweight
    /// VACANT root — the closed session's real program was released
    /// (F05b-C01); holders that pinned a revision keep their own Arc.
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

    /// Run the CURRENT snapshot's entry. Span-bearing outcomes (traps)
    /// are mapped into CURRENT-source coordinates: each function's unit
    /// keeps the spans of the revision it was lowered from, and the
    /// session shifts them by that function's origin delta — code
    /// payloads are never touched. Runs against a PINNED older snapshot
    /// keep that revision's coordinates (its own source view).
    pub fn run(&self, limits: &ExecLimits, cancel: &CancelFlag) -> (ExecOutcome, ExecTrace) {
        if self.closed {
            return (
                ExecOutcome::Infra("session closed".to_string()),
                ExecTrace::default(),
            );
        }
        execute_program(&self.current, limits, cancel)
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

    /// CANCEL a prepared candidate (PLAN-828 T-06/F-05b): the candidate is
    /// consumed and NEVER installs — the session stays on its current
    /// revision, source and result, and keeps accepting new edits. A
    /// candidate minted by another session is refused exactly like a
    /// commit would be (the minting session can still dispose of its own).
    pub fn cancel_prepared(&mut self, prepared: PreparedEdit) -> EditOutcome {
        if self.closed {
            return EditOutcome::rejected(self.revision, "session closed".to_string(), Vec::new());
        }
        if prepared.session_id != self.session_id {
            return EditOutcome::rejected(
                self.revision,
                "cross-session candidate refused: prepared by a different session".to_string(),
                Vec::new(),
            );
        }
        self.counters.bump("candidates_cancelled");
        EditOutcome::rejected(
            self.revision,
            format!(
                "candidate cancelled (base revision {}); session remains at revision {}",
                prepared.base_revision, self.revision
            ),
            Vec::new(),
        )
    }

    /// CLOSE the session (PLAN-828 T-06/F-05b): a terminal, idempotent
    /// state. New runs/edits/prepares/commits/cancels are refused, and the
    /// session drops its OWNED strong references — every function's
    /// current HIR body and code object (plus their bounded histories),
    /// the store handle, and the current execution root. WITHOUT an
    /// external pin those resources die HERE, not at `drop(session)`
    /// (F05b-C01); snapshots pinned elsewhere and PausedRuns keep
    /// themselves alive and correct (V-05: in-flight chains are never
    /// yanked). `current()` afterwards hands back a lightweight VACANT
    /// root (never the released program, never a panic); `resident()`
    /// returns None.
    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        // PLAN-828 T-02/D-03: EXPLICIT release of the retained checked
        // environment's owned body entries (never implied by field drops).
        self.env_core.close();
        self.store = None;
        self.fns.clear();
        self.current = Arc::new(ProgramSnapshot::vacant(&self.entry_def_id, self.revision));
    }

    /// Whether this session has been closed (terminal state, F-05b).
    pub fn closed(&self) -> bool {
        self.closed
    }

    /// Bounded content-history insert (V-07): never evict the resident
    /// revision's own entry; evict an arbitrary oldest-ordered key when
    /// over the cap.
    #[allow(clippy::type_complexity)]
    fn history_entry(
        body: Arc<hir::Body>,
        code: Arc<FunctionCode>,
    ) -> (
        Arc<hir::Body>,
        Arc<FunctionCode>,
        u64,
        auto_av_core::span_map::SpanSegments,
        String,
    ) {
        (
            body,
            code,
            0,
            auto_av_core::span_map::SpanSegments::default(),
            String::new(),
        )
    }

    #[allow(clippy::type_complexity)]
    fn history_insert(
        rec: &mut FnRecord,
        cv: String,
        entry: (
            Arc<hir::Body>,
            Arc<FunctionCode>,
            u64,
            auto_av_core::span_map::SpanSegments,
            String,
        ),
    ) {
        if rec.history.len() >= HISTORY_CAP && !rec.history.contains_key(&cv) {
            if let Some(victim) = rec
                .history
                .keys()
                .find(|k| {
                    !rec.history
                        .get(*k)
                        .map(|v| Arc::ptr_eq(&v.1, &rec.code))
                        .unwrap_or(false)
                })
                .cloned()
            {
                rec.history.remove(&victim);
            }
        }
        rec.history.entry(cv).or_insert(entry);
    }

    /// Resident function source-names in declaration order.
    pub fn function_names(&self) -> Vec<String> {
        self.fns.iter().map(|f| f.name.clone()).collect()
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
            .find(|(_, v)| Arc::ptr_eq(&v.1, &rec.code))
            .map(|(cv, _)| cv.clone())
    }

    /// Persist every function's ABC unit under its content version (same
    /// identity scheme as the 798 driver's A09 rows). The payload row is
    /// SPANLESS by design — byte-equal to the 798 scheme — and the unit's
    /// mint span table plus its mint decl range/text ride a separate
    /// unit-bound sidecar row (PLAN-828 F-10/F11-S01), so a later
    /// [`restore`] re-attaches and RE-ANCHORS it instead of publishing
    /// `span: None` or stale absolute positions.
    fn persist_all_units(&self) -> Result<(), String> {
        let Some((store, project)) = self.store.as_ref() else {
            return Err("no store".to_string());
        };
        let owner = self.fns_owner();
        let caps_fp = caps_fingerprint(&self.caps);
        for (idx, rec) in self.fns.iter().enumerate() {
            // The resident entry keys the row AND carries the code's true
            // mint range/text — the F11-S01 re-anchoring basis.
            let Some((cv, mint_d0, mint_text)) = rec
                .history
                .iter()
                .find(|(_, v)| Arc::ptr_eq(&v.1, &rec.code))
                .map(|(cv, v)| (cv.clone(), v.2, v.4.clone()))
            else {
                continue;
            };
            let mut spanless = rec.code.unit().module().clone();
            let Some(f) = spanless.funcs.first_mut() else {
                continue;
            };
            let spans = std::mem::take(&mut f.spans);
            let bytes = auto_av_core::abc::encode_abc(&spanless)
                .map_err(|e| format!("encode {}: {e:?}", rec.def_id))?
                .to_flat();
            let key = abc_unit_key(&owner, &rec.def_id, &cv, &caps_fp);
            let expect = EnvelopeIdentity::abc_bound(&owner, &rec.def_id, &cv);
            if let Err(e) =
                put_enveloped(store, &key, "A09", &expect, &bytes, vec![], Some(project))
            {
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
            let skey = unit_span_sidecar_key(&owner, &rec.def_id, &cv);
            let sid = EnvelopeIdentity::unit_span_sidecar(&owner, &rec.def_id, &cv);
            let spayload = encode_unit_sidecar(&UnitSidecar {
                spans,
                mint: Some((
                    mint_d0,
                    mint_d0 + mint_text.len() as u64,
                    mint_text.as_bytes().to_vec(),
                )),
            });
            if let Err(e) =
                put_enveloped(store, &skey, "A09", &sid, &spayload, vec![], Some(project))
            {
                if !format!("{e:?}").contains("Conflict") {
                    return Err(format!("put sidecar {}: {e:?}", rec.def_id));
                }
                // F10-S02: a Conflict must not silently keep an INVALID
                // survivor as the only restore reference. The store has
                // no per-artifact rewrite, so an inapplicable survivor is
                // repaired by publishing the re-derived payload under the
                // repair identity, which restore probes after the primary
                // key — the same rederive-AND-rewrite contract as the 798
                // source sidecar (cache-and-conformance §11). The
                // survivor is held to the SAME full applicability
                // standard restore selects with — decode, coverage, and
                // provenance binding to the CURRENT source (mint
                // UTF-8/frame/re-anchoring; legacy rows must equal the
                // current-source compile's own span table).
                let (cd0, cd1) = self.parse.tree.abs_range(self.parse.tree.decls[idx].node);
                let cur_decl = &self.src[cd0 as usize..cd1 as usize];
                // The survivor standard is the SAME mapped-source evidence
                // restore selects with: the resident's spans expressed in
                // CURRENT coordinates — through the session's live span
                // map (identity on a fresh cold compile, the composed map
                // after edits).
                let resident_f = rec.code.unit().module().funcs.first();
                let cur_ev = resident_f.map(|f| {
                    let mut ev = f.clone();
                    ev.spans = f
                        .spans
                        .iter()
                        .map(|(i, sp)| {
                            let (s, e) = rec.span_map.map_span(sp.start as u64, sp.end as u64);
                            (
                                *i,
                                auto_av_core::contract::SpanInfo {
                                    func: sp.func,
                                    start: s as u32,
                                    end: e as u32,
                                },
                            )
                        })
                        .collect();
                    ev
                });
                let row_usable = |b: &[u8]| match (resident_f, &cur_ev) {
                    (Some(f), Some(ev)) => {
                        unit_sidecar_usable_for_current(b, f, ev, cd0, cur_decl.as_bytes())
                    }
                    _ => false,
                };
                let survivor_usable = matches!(
                    get_enveloped_opt(store, &skey, "A09", &sid),
                    Ok(Some(b)) if row_usable(&b)
                );
                if !survivor_usable {
                    let rkey = unit_span_sidecar_repair_key(&owner, &rec.def_id, &cv);
                    if let Err(e) =
                        put_enveloped(store, &rkey, "A09", &sid, &spayload, vec![], Some(project))
                    {
                        if !format!("{e:?}").contains("Conflict") {
                            return Err(format!("put sidecar repair {}: {e:?}", rec.def_id));
                        }
                        // The repair row is held too: a VALID holder still
                        // serves future restores; an invalid one means the
                        // store is genuinely stuck on this unit — surface
                        // the error instead of accepting a permanently
                        // broken sidecar.
                        let repair_usable = matches!(
                            get_enveloped_opt(store, &rkey, "A09", &sid),
                            Ok(Some(b)) if row_usable(&b)
                        );
                        if !repair_usable {
                            return Err(format!(
                                "sidecar `{}`: primary and repair rows both hold invalid payloads",
                                rec.def_id
                            ));
                        }
                    }
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
                (
                    "producer".into(),
                    auto_av_core::contract::PRODUCER_ID.into(),
                ),
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

    /// This session's identity token.
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// The miss path of the content cache (T-02: verification itself runs
    /// at the caller, deferred by the candidate-document prepare so a
    /// content hit reuses without re-verifying): take the ALREADY-VERIFIED
    /// credential, currency-check it against the resident interface
    /// environment, lower the single-function unit, admit it through the
    /// ABC admission and wrap the resident code handle. Counters bump at
    /// each real entry. Inner `Err` = candidate rejection (diagnostics);
    /// outer `Err` = infrastructure failure.
    #[allow(clippy::type_complexity)]
    fn lower_and_admit(
        &mut self,
        bundle: &hir::Bundle,
        checked_fn: CheckedFunction,
        changed_name: &str,
        new_src_hash: u64,
    ) -> Result<Result<(Arc<hir::Body>, Arc<FunctionCode>), Vec<String>>, String> {
        // Credential currency: the body was verified under the SAME
        // declaration context as the outstanding ones — a drift here means
        // the interface gate above missed a context change (defense in
        // depth; reject rather than re-stamp).
        if !self.env.credential_current(&checked_fn) {
            return Ok(Err(vec![
                "body credential minted under a different declaration context".to_string(),
            ]));
        }

        let def_idx = bundle.modules[0]
            .defs
            .iter()
            .position(|d| d.id_text == checked_fn.def_id_text())
            .ok_or_else(|| "verified def missing in candidate bundle".to_string())?;
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
        if self.closed {
            return Err("session closed".to_string());
        }
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
        self.counters.add("syntax_rescan_bytes", inc.rescan_bytes);
        self.counters
            .add("syntax_reparsed_decls", inc.reparsed_decls as u64);
        self.counters
            .add("syntax_reused_decls", inc.reused_decls as u64);
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
        // The edited source must pass the same front door (a growing edit
        // cannot push the program past the frozen input budget).
        front_door_admit(&new_src).map_err(|d| d.message)?;

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

        // PLAN-828 T-02: the hot path runs the REAL single-body transaction
        // against the retained checked environment — re-check (resolve +
        // typecheck of ONE body) and single-body emission with body-less
        // callee scaffolds. No synth program, no whole-program frontend:
        // unchanged functions contribute no body work (the producer window
        // gate proves it). Current is restored before the candidate is
        // returned; the environment slot publishes only at commit (D-03).
        let candidate = match prepare_candidate_document(
            &mut self.env_core,
            &projected_fn,
            changed_idx,
            &new_src,
            "probe.at",
        ) {
            Ok(c) => c,
            Err(ds) => {
                // The re-check's signature-drift guard (`source.incremental`)
                // IS the session's explicit non-fast path — an interface
                // edit, never silently re-checked. Anything else is a
                // candidate rejection carrying real diagnostics.
                if ds.iter().any(|d| d.code == "source.incremental") {
                    return Ok(Err(EditOutcome::rejected(
                        self.revision,
                        "non-fast-path: signature change refused by the retained environment's interface gate"
                            .to_string(),
                        Vec::new(),
                    )));
                }
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

        let def_idx = candidate
            .parts()
            .bundle
            .modules
            .first()
            .ok_or_else(|| "candidate bundle has no module".to_string())?
            .defs
            .iter()
            .position(|d| d.id_text == self.fns[changed_idx].def_id)
            .ok_or_else(|| {
                format!(
                    "def {} missing in candidate bundle",
                    self.fns[changed_idx].def_id
                )
            })?;

        // Fast-path gate through the interface environment (T-03): the
        // changed function's interface AND the whole declaration context
        // must be identical to the resident environment.
        let candidate_env = FnEnv::from_bundle(&candidate.parts().bundle)?;
        if !self.env.interface_unchanged(&candidate_env, &changed_name) {
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
        let cand_m = candidate
            .parts()
            .bundle
            .modules
            .first()
            .ok_or_else(|| "candidate bundle has no module".to_string())?;
        let def_body = match &cand_m.defs[def_idx].kind {
            hir::DefKind::Function { body, .. } => *body,
            _ => return Err("internal: changed def is not a function".to_string()),
        };
        let cv = content_version_of(&canonical_hir_function(cand_m, def_body));
        let mut was_hit = false;
        let mut candidate_mint_origin: u64 = 0;
        let mut candidate_span_map = auto_av_core::span_map::SpanSegments::default();
        let (new_hir_body, new_code): (Arc<hir::Body>, Arc<FunctionCode>) =
            match self.fns[changed_idx].history.get(&cv) {
                Some((body, code, mint_o, map, mint_text)) => {
                    self.counters.bump(&format!("reuse_hit:{changed_name}"));
                    was_hit = true;
                    candidate_mint_origin = *mint_o;
                    // Prefer a full-text re-anchoring when the entry's
                    // mint text re-embeds in the new text (recovers
                    // rewritten anchors a geometric chain cannot);
                    // otherwise keep the accumulated map and compose this
                    // edit into it.
                    let (nd0, nd1) = inc.tree.abs_range(inc.tree.decls[changed_idx].node);
                    let new_text = &new_src[nd0 as usize..nd1 as usize];
                    candidate_span_map = match auto_av_core::span_map::SpanSegments::from_texts(
                        *mint_o,
                        *mint_o + mint_text.len() as u64,
                        nd0,
                        mint_text.as_bytes(),
                        new_text.as_bytes(),
                    ) {
                        Some(m) => m,
                        None => {
                            let mut m = map.clone();
                            let old_bytes = self.src[edit.start as usize..edit.end as usize]
                                .as_bytes()
                                .to_vec();
                            m.apply_aligned(edit.start, edit.end, &old_bytes, &edit.insert);
                            m
                        }
                    };
                    (body.clone(), code.clone())
                }
                None => {
                    let new_src_hash = fnv64(new_src.as_bytes());
                    // The miss path verifies the candidate body NOW: the
                    // document prepare defers verification so a content HIT
                    // can reuse the stored pair with zero re-verify (T-04).
                    let checked_fn =
                        match verify_function(candidate.parts().bundle.clone(), def_idx) {
                            Ok(c) => c,
                            Err(ds) => {
                                return Ok(Err(EditOutcome::rejected(
                                    self.revision,
                                    "candidate rejected: verification diagnostics".to_string(),
                                    ds.iter().map(|d| d.render("probe.at", &new_src)).collect(),
                                )));
                            }
                        };
                    self.counters.bump(&format!("verify_hir:{changed_name}"));
                    match self.lower_and_admit(
                        &candidate.parts().bundle,
                        checked_fn,
                        &changed_name,
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

        if !was_hit {
            candidate_mint_origin = inc.tree.abs_range(inc.tree.decls[changed_idx].node).0;
        }
        Ok(Ok(PreparedEdit {
            candidate,
            edit_region: (start, end.saturating_sub(start), edit.insert.len() as u64),
            edit_bytes: edit.insert.clone(),
            session_id: self.session_id,
            reminted: !was_hit,
            mint_origin: candidate_mint_origin,
            span_map: candidate_span_map,
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
        if self.closed {
            return EditOutcome::rejected(self.revision, "session closed".to_string(), Vec::new());
        }
        if prepared.session_id != self.session_id {
            return EditOutcome::rejected(
                self.revision,
                "cross-session candidate refused: prepared by a different session".to_string(),
                Vec::new(),
            );
        }
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
        let reminted = prepared.reminted;
        // Refresh per-function origins BEFORE linking: only a RE-MINTED
        // function resets its span baseline; a content hit keeps the
        // original mint baseline and gains a body-region refinement when
        // its body text moved internally (F-828-R1-04/R2a F04-b/c).
        let old_src = self.src.clone();
        let mut span_maps: Vec<auto_av_core::span_map::SpanSegments> =
            Vec::with_capacity(self.fns.len());
        for (i, d) in prepared.new_parse.tree.decls.iter().enumerate() {
            let o = prepared.new_parse.tree.abs_range(d.node).0;
            let rec = &mut self.fns[i];
            rec.origin = o;
            span_maps.push(auto_av_core::span_map::SpanSegments::default()); // replaced below
            if i == changed_idx {
                rec.mint_origin = prepared.mint_origin;
                rec.span_map = prepared.span_map.clone();
            }
            let changed_hit = !reminted && i == changed_idx;
            let (c, old_len, new_len) = prepared.edit_region;
            if reminted && i == changed_idx {
                // Fresh code mints against the NEW source: identity over
                // its own decl range.
                let (d0, d1) = prepared
                    .new_parse
                    .tree
                    .abs_range(prepared.new_parse.tree.decls[i].node);
                rec.span_map = auto_av_core::span_map::SpanSegments::identity_range(d0, d1, d0);
            } else if changed_hit {
                // A content hit's map was re-anchored to the NEW text at
                // prepare time — applying the edit again would double it.
            } else {
                // Content-aware: equivalent rewrites keep their mint
                // anchors (F-828-R1-04/R8a-check).
                let old_bytes = old_src.as_bytes()[c as usize..(c + old_len) as usize].to_vec();
                rec.span_map
                    .apply_aligned(c, c + old_len, &old_bytes, &prepared.edit_bytes);
                for e in rec.history.values_mut() {
                    e.3.apply_aligned(c, c + old_len, &old_bytes, &prepared.edit_bytes);
                }
                continue;
            }
            // EVERY stored code object of this function (history entries)
            // experiences the same source change; the deposited current
            // entry is overwritten below with the post-edit map.
            for e in rec.history.values_mut() {
                let old_bytes = old_src.as_bytes()[c as usize..(c + old_len) as usize].to_vec();
                e.3.apply_aligned(c, c + old_len, &old_bytes, &prepared.edit_bytes);
            }
        }
        span_maps = self.fns.iter().map(|r| r.span_map.clone()).collect();
        let mut codes: Vec<Arc<FunctionCode>> = Vec::with_capacity(self.fns.len());
        for (i, rec) in self.fns.iter().enumerate() {
            if i == changed_idx {
                codes.push(prepared.code.clone());
            } else {
                codes.push(rec.code.clone());
            }
        }
        let next = match ProgramSnapshot::link_with_maps(
            codes,
            &self.entry_def_id,
            next_revision,
            span_maps,
        ) {
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
            candidate,
            session_id,
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
        let _ = (session_id, base_revision);
        // D-03 single publication point: the environment slot publishes
        // TOGETHER with the root/source/revision advance — the candidate's
        // commit validates owner/base/closed and installs its own entry.
        // A refusal is unreachable by construction (this session is the
        // env's sole publisher and the base-revision guard above matches
        // the env base); surface it honestly rather than ignore.
        if let Err(ds) = candidate.commit(&mut self.env_core) {
            return EditOutcome::rejected(
                self.revision,
                format!(
                    "env install refused: {}",
                    ds.iter()
                        .map(|d| d.render("probe.at", &self.src))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
                Vec::new(),
            );
        }
        let changed_rec = &mut self.fns[changed_idx];
        // The deposited map is the POST-commit one (installed baseline
        // composed with this edit) — captured after the origin loop.
        let deposit_map = changed_rec.span_map.clone();
        let (td0, td1) = new_parse
            .tree
            .abs_range(new_parse.tree.decls[changed_idx].node);
        let mint_text = new_src[td0 as usize..td1 as usize].to_string();
        Self::history_insert(
            changed_rec,
            content_version,
            (
                hir_body.clone(),
                code.clone(),
                prepared.mint_origin,
                deposit_map,
                mint_text,
            ),
        );
        changed_rec.hir_body = hir_body;
        changed_rec.code = code;

        // T-07: durable deposit of the installed revision (units + the
        // session manifest). A store failure is recorded but never
        // rolls the installed revision back — memory state is primary.
        if self.store.is_some() {
            let persist = self
                .persist_all_units()
                .and_then(|()| self.persist_manifest());
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

fn render_diags(src: &str, ds: &[Diagnostic]) -> String {
    ds.iter()
        .map(|d| d.render("probe.at", src))
        .collect::<Vec<_>>()
        .join("\n")
}

fn history_entry(
    body: Arc<hir::Body>,
    code: Arc<FunctionCode>,
) -> (
    Arc<hir::Body>,
    Arc<FunctionCode>,
    u64,
    auto_av_core::span_map::SpanSegments,
    String,
) {
    (
        body,
        code,
        0,
        auto_av_core::span_map::SpanSegments::default(),
        String::new(),
    )
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
