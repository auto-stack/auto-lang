//! Real vertical compile driver (PLAN-783 §5.3, T-04): source →
//! parse → HIR → verify → COFF → staged link → run, with per-stage
//! content-addressed artifacts in the managed store and three resume
//! paths:
//!
//! * **object hit** — skip frontend/verify/lower; still real link + run;
//! * **HIR hit**    — binary restore → REAL `verify` of the restored
//!                    bundle → lower → link + run (persisted payloads
//!                    never carry trust; the verifier is the only way in);
//! * **full miss**  — real `compile_parts_with` (the source frontend
//!                    seam), then the same verify/lower/link/run.
//!
//! Stage counters + failure injection (source file removed) prove the
//! skip claims: a hit path that cannot possibly recompile still produces
//! the same semantic oracle.

use crate::codec::{self, ChunkKind, ObjectManifest};
use crate::identity::{
    self, decl_summary_key, hir_bundle_key, QueryKey, SourceSnapshotId, BACKEND_ID,
    SUPPORTED_TARGET,
};
use crate::store::{ManagedStore, PutRequest, StoreError};
use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::hir::HirDomain;
use auto_ac_prototype::link;
use auto_ac_prototype::native;
use auto_ac_prototype::source::{compile_parts_with, parse_program_with, SourceProfile};
use std::fmt;
use std::path::{Path, PathBuf};

/// Per-stage execution counts — the skip oracle.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, PartialEq, Eq)]
pub struct StageCounters {
    pub parse: u32,
    pub resolve_typecheck: u32,
    pub bind: u32,
    pub verify: u32,
    pub hir_encode: u32,
    pub hir_restore: u32,
    pub lower: u32,
    pub link: u32,
    pub run: u32,
    pub decl_summary_build: u32,
    pub store_puts: u32,
}

/// Per-layer hit/miss outcome of one build.
#[derive(Clone, Debug, Default, serde::Serialize, PartialEq, Eq)]
pub struct HitReport {
    pub snapshot_present: bool,
    pub summary: &'static str, // hit | computed | skipped
    pub hir: &'static str,     // hit(restored) | built | skipped(object-hit)
    pub object: &'static str,  // hit | lowered
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct VerticalOutcome {
    pub project: String,
    pub counters: StageCounters,
    pub hits: HitReport,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub exe: String,
    pub artifacts: Vec<(String, String)>, // (layer, artifact id)
    pub snapshot_id: String,
    pub link_ms: u64,
    pub total_ms: u64,
}

#[derive(Debug)]
pub enum DriverError {
    UnsupportedTarget(String),
    SourceUnavailable {
        path: PathBuf,
        detail: String,
    },
    Store(StoreError),
    Codec(codec::CodecError),
    /// Pre-rendered against the CONSUMING project's source path/text (R1
    /// F-11: diagnostics bind to the current project, never to blank
    /// context; content identity is unaffected).
    Frontend(String),
    Verify(String),
    Backend(String),
    Link(String),
    Run(String),
    Internal(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverError::UnsupportedTarget(t) => {
                write!(
                    f,
                    "unsupported target {} (probe lowers {} only)",
                    t, SUPPORTED_TARGET
                )
            }
            DriverError::SourceUnavailable { path, detail } => write!(
                f,
                "source unavailable at {} and no cache hit covers it: {}",
                path.display(),
                detail
            ),
            DriverError::Store(e) => write!(f, "store: {}", e),
            DriverError::Codec(e) => write!(f, "codec: {}", e),
            DriverError::Frontend(s) => write!(f, "frontend: {}", s),
            DriverError::Verify(s) => write!(f, "verify: {}", s),
            DriverError::Backend(s) => write!(f, "backend: {}", s),
            DriverError::Link(e) => write!(f, "link: {}", e),
            DriverError::Run(e) => write!(f, "run: {}", e),
            DriverError::Internal(e) => write!(f, "internal: {}", e),
        }
    }
}

/// Render diagnostics bound to the current project's source path (and
/// text when in hand — a restored-payload verification without source
/// still names the consuming path).
fn render_diags(ds: &[Diagnostic], path: &str, src: &str) -> String {
    ds.iter()
        .map(|d| d.render(path, src))
        .collect::<Vec<_>>()
        .join("; ")
}

impl std::error::Error for DriverError {}

impl From<StoreError> for DriverError {
    fn from(e: StoreError) -> Self {
        DriverError::Store(e)
    }
}
impl From<codec::CodecError> for DriverError {
    fn from(e: codec::CodecError) -> Self {
        DriverError::Codec(e)
    }
}
impl From<std::io::Error> for DriverError {
    fn from(e: std::io::Error) -> Self {
        DriverError::Internal(e.to_string())
    }
}

pub struct VerticalCfg {
    pub project: String,
    /// Source file; may be absent when a snapshot override is supplied.
    pub source: PathBuf,
    /// Pre-registered snapshot identity (resume without source).
    pub snapshot_override: Option<SourceSnapshotId>,
    pub logical_name: String,
    pub profile: SourceProfile,
    pub target: String,
    /// User entry def (selects the `ac_start` wrapper target).
    pub entry: String,
    pub toolchain_fp: String,
    pub work_dir: PathBuf,
}

impl Default for VerticalCfg {
    fn default() -> Self {
        VerticalCfg {
            project: "proj".into(),
            source: PathBuf::from("main.at"),
            snapshot_override: None,
            logical_name: "main".into(),
            profile: SourceProfile::Core,
            target: SUPPORTED_TARGET.into(),
            entry: "main".into(),
            toolchain_fp: "default-lld".into(),
            work_dir: std::env::temp_dir(),
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// One full build/resume through the real pipeline. Never bypasses the
/// verifier; object bytes always come from real `lower_object`.
pub fn vertical(store: &ManagedStore, cfg: &VerticalCfg) -> Result<VerticalOutcome, DriverError> {
    let t0 = now_ms();
    if cfg.target != SUPPORTED_TARGET {
        return Err(DriverError::UnsupportedTarget(cfg.target.clone()));
    }
    std::fs::create_dir_all(&cfg.work_dir)?;

    let mut c = StageCounters::default();
    let mut hits = HitReport {
        snapshot_present: false,
        ..Default::default()
    };
    let mut artifacts: Vec<(String, String)> = Vec::new();

    // -- inputs ------------------------------------------------------------
    let (snapshot, source_bytes): (SourceSnapshotId, Option<Vec<u8>>) = match &cfg.snapshot_override
    {
        Some(id) => (id.clone(), None),
        None => match std::fs::read(&cfg.source) {
            Ok(b) => {
                hits.snapshot_present = true;
                (identity::source_snapshot(&b, &cfg.logical_name), Some(b))
            }
            Err(_) => (identity::source_snapshot(&[], &cfg.logical_name), None),
        },
    };
    let source_text = || -> Result<String, DriverError> {
        source_bytes
            .clone()
            .map(|b| {
                String::from_utf8(b).map_err(|_| DriverError::Internal("source not utf-8".into()))
            })
            .unwrap_or_else(|| {
                Err(DriverError::SourceUnavailable {
                    path: cfg.source.clone(),
                    detail:
                        "front-end recompilation requires the source; no HIR/object hit covered it"
                            .into(),
                })
            })
    };

    // A00: source snapshot artifact (raw bytes; user source stays
    // user-owned — the store keeps its own immutable content copy).
    if let Some(bytes) = &source_bytes {
        let key = QueryKey::new(
            crate::schema::layer("A00").unwrap(),
            vec![("snapshot".into(), snapshot.0.clone())],
        );
        let art = store.put_artifact(PutRequest {
            layer: "A00",
            key,
            kind: ChunkKind::SnapshotRecord,
            payload: bytes.clone(),
            edges: vec![],
            project: None,
        })?;
        artifacts.push(("A00".into(), art));
        c.store_puts += 1;
    }

    // -- layer keys ----------------------------------------------------------
    let capabilities: Vec<String> = Vec::new(); // core/scalar corpus needs none
    let hir_key = hir_bundle_key(
        cfg.profile.as_str(),
        cfg.profile.hir_domain().as_str(),
        &capabilities,
        &snapshot,
        codec::HIR_CODEC_VERSION,
    );
    let obj_key = object_key_by_hash(&hir_key.hash(), &cfg.target, BACKEND_ID, &cfg.entry);

    // -- A09 object hit: skip everything up to link --------------------------
    if let Some(loaded) = store.get_artifact("A09", &obj_key)? {
        hits.object = "hit";
        hits.hir = "skipped(object-hit)";
        // The link receipt for THIS toolchain fingerprint is created when
        // absent (object hit ⇒ only the link inputs changed). The receipt
        // content is a pure function of the object chunk + link inputs;
        // upstream HIR linkage rides PROVENANCE EDGES (metadata), not the
        // receipt bytes, so cold upstreams never fork the receipt content.
        let lkey = link_key(&obj_key, &cfg.toolchain_fp);
        let a10_id = match store.get_artifact("A10", &lkey)? {
            Some(m) => m.artifact_id,
            None => {
                let manifest = ObjectManifest {
                    object_chunk: loaded.chunk_id.clone(),
                    object_len: loaded.payload.len() as u64,
                    target: cfg.target.clone(),
                    backend: BACKEND_ID.into(),
                    entry: cfg.entry.clone(),
                    produced_by: crate::PROBE_ID.into(),
                };
                let mpayload = serde_json::to_vec(&manifest)
                    .map_err(|e| DriverError::Internal(e.to_string()))?;
                let mut edges = vec![(loaded.artifact_id.clone(), identity::RefKind::RequiredData)];
                if let Some(hir) = store.get_artifact("A07", &hir_key)? {
                    edges.push((hir.artifact_id, identity::RefKind::Provenance));
                }
                store.put_artifact(PutRequest {
                    layer: "A10",
                    key: lkey,
                    kind: ChunkKind::ObjectManifest,
                    payload: mpayload,
                    edges,
                    project: None,
                })?
            }
        };
        artifacts.push(("A10".into(), a10_id));
        artifacts.push(("A09".into(), loaded.artifact_id.clone()));
        let (exe, link_ms) = link_only(&loaded.payload, cfg, &mut c)?;
        // A hit consumer still owns its current-version roots: a consumer
        // that only resumes must not leave its deliverables unrooted and
        // GC-able (PLAN-783 §5.4: project roots follow consumption).
        set_current_roots(store, &cfg.project, &artifacts)?;
        return Ok(finish(
            cfg, c, hits, artifacts, &exe, &snapshot, link_ms, t0,
        ));
    }

    // -- HIR: restore (binary) or build (real front end), then REAL verify ----
    let path_disp = cfg.source.display().to_string();
    let checked = if let Some(loaded) = store.get_artifact("A07", &hir_key)? {
        hits.hir = "hit(restored)";
        c.hir_restore += 1;
        let bundle = codec::decode_bundle_chunk(&loaded.payload)?;
        // Restored-payload verification: bind diagnostics to the CURRENT
        // project path (text in hand when the source is present).
        let text_ctx = source_bytes
            .as_ref()
            .map(|b| String::from_utf8_lossy(b).to_string());
        let checked = verify_real(bundle, &path_disp, text_ctx.as_deref().unwrap_or(""))?;
        c.verify += 1;
        artifacts.push(("A07".into(), loaded.artifact_id.clone()));
        checked
    } else {
        hits.hir = "built";
        let text = source_text()?;
        let program = parse_program_with(&text, cfg.profile)
            .map_err(|d| DriverError::Frontend(render_diags(&[d], &path_disp, &text)))?;
        c.parse += 1;

        // A04: declaration summary from the REAL AST — content-identical
        // across body-only edits (same signature ⇒ same artifact bytes).
        let summary = codec::build_decl_summary(&program, &cfg.logical_name);
        c.decl_summary_build += 1;
        let summary_payload = codec::encode_decl_summary(&summary)?;
        let skey = decl_summary_key(cfg.profile.as_str(), &snapshot);
        let summary_art = store.put_artifact(PutRequest {
            layer: "A04",
            key: skey,
            kind: ChunkKind::DeclSummary,
            payload: summary_payload,
            edges: vec![],
            project: None,
        })?;
        artifacts.push(("A04".into(), summary_art));
        c.store_puts += 1;
        hits.summary = "computed";

        // The adapter's own front end runs inside compile_parts_with (one
        // extra parse; counted as one resolve/typecheck + bind here).
        let parts = compile_parts_with(&text, &cfg.logical_name, &capabilities, cfg.profile)
            .map_err(|ds| DriverError::Frontend(render_diags(&ds, &path_disp, &text)))?;
        c.resolve_typecheck += 1;
        c.bind += 1;
        let checked = verify_real(parts.bundle, &path_disp, &text)?;
        c.verify += 1;

        // A07: persist the binary bundle (provenance: the snapshot).
        let bundle = checked.bundle().clone();
        let chunk = codec::encode_bundle_chunk(&bundle)?;
        c.hir_encode += 1;
        let snap_art = artifacts
            .iter()
            .find(|(l, _)| l == "A00")
            .map(|(_, id)| id.clone());
        let art = store.put_artifact(PutRequest {
            layer: "A07",
            key: hir_key.clone(),
            kind: ChunkKind::Hir,
            payload: chunk,
            edges: snap_art
                .map(|a| vec![(a, identity::RefKind::Provenance)])
                .unwrap_or_default(),
            project: None,
        })?;
        artifacts.push(("A07".into(), art));
        c.store_puts += 1;
        checked
    };
    // -- real lowering → A09 ---------------------------------------------------
    // The source adapter names defs `d_<name>` (ac-core CLI contract).
    let sel = native::find_entry(&checked, &format!("d_{}", cfg.entry))
        .map_err(|ds| DriverError::Backend(render_diags(&ds, &path_disp, "")))?;
    let obj_bytes = native::lower_object(&checked, &sel)
        .map_err(|ds| DriverError::Backend(render_diags(&ds, &path_disp, "")))?;
    c.lower += 1;
    let obj_art = store.put_artifact(PutRequest {
        layer: "A09",
        key: obj_key.clone(),
        kind: ChunkKind::Object,
        payload: obj_bytes.clone(),
        // Provenance only: the object stays linkable when the HIR is gone.
        edges: artifacts
            .iter()
            .find(|(l, _)| l == "A07")
            .map(|(_, id)| vec![(id.clone(), identity::RefKind::Provenance)])
            .unwrap_or_default(),
        project: None,
    })?;
    artifacts.push(("A09".into(), obj_art.clone()));
    c.store_puts += 1;
    hits.object = "lowered";

    // -- link + publish + run ---------------------------------------------------
    let obj_path = cfg.work_dir.join("app.obj");
    std::fs::write(&obj_path, &obj_bytes)
        .map_err(|e| DriverError::Internal(format!("write obj: {}", e)))?;
    let (exe, link_ms) = link_only(&obj_bytes, cfg, &mut c)?;

    // A10: link receipt — content is a pure function of the object chunk +
    // link inputs; the upstream HIR linkage rides a PROVENANCE EDGE
    // (metadata), so cold upstreams never fork the receipt content.
    let manifest = ObjectManifest {
        object_chunk: codec::framed_chunk_id(ChunkKind::Object, &obj_bytes).0,
        object_len: obj_bytes.len() as u64,
        target: cfg.target.clone(),
        backend: BACKEND_ID.into(),
        entry: cfg.entry.clone(),
        produced_by: crate::PROBE_ID.into(),
    };
    let mpayload =
        serde_json::to_vec(&manifest).map_err(|e| DriverError::Internal(e.to_string()))?;
    let mut receipt_edges = vec![(obj_art.clone(), identity::RefKind::RequiredData)];
    if let Some((_, hir_id)) = artifacts.iter().find(|(l, _)| l == "A07") {
        receipt_edges.push((hir_id.clone(), identity::RefKind::Provenance));
    }
    let mart = store.put_artifact(PutRequest {
        layer: "A10",
        key: link_key(&obj_key, &cfg.toolchain_fp),
        kind: ChunkKind::ObjectManifest,
        payload: mpayload,
        edges: receipt_edges,
        project: None,
    })?;
    artifacts.push(("A10".into(), mart));
    c.store_puts += 1;

    set_current_roots(store, &cfg.project, &artifacts)?;
    Ok(finish(
        cfg, c, hits, artifacts, &exe, &snapshot, link_ms, t0,
    ))
}

/// Real verification in the bundle's OWN domain — the only door to a
/// Checked credential, fresh or restored.
fn verify_real(
    bundle: auto_ac_prototype::hir::Bundle,
    path: &str,
    src: &str,
) -> Result<auto_ac_prototype::verify::CheckedModule, DriverError> {
    match bundle.domain {
        HirDomain::Core => auto_ac_prototype::verify::verify(bundle)
            .map_err(|ds| DriverError::Verify(render_diags(&ds, path, src))),
        HirDomain::Scalar => auto_ac_prototype::verify::verify_scalar(bundle)
            .map_err(|ds| DriverError::Verify(render_diags(&ds, path, src))),
        d => Err(DriverError::Internal(format!(
            "domain {} outside probe subset",
            d.as_str()
        ))),
    }
}

/// Object key from the HIR lookup-key hash: computable from source
/// identity + profile WITHOUT compiling — required for object-hit resume
/// with the source absent, and a body-only edit changes the snapshot and
/// thus this key (same signature or not).
pub fn object_key_by_hash(
    hir_key_hash: &str,
    target: &str,
    backend: &str,
    entry: &str,
) -> QueryKey {
    identity::object_key(
        &identity::ArtifactId(format!("hirkey:{}", hir_key_hash)),
        target,
        backend,
        entry,
    )
}

fn link_key(obj_key: &QueryKey, toolchain_fp: &str) -> QueryKey {
    QueryKey::new(
        crate::schema::layer("A10").unwrap(),
        vec![
            ("object-key".into(), obj_key.hash()),
            ("toolchain".into(), toolchain_fp.into()),
        ],
    )
}

/// Real staged link of object bytes + publish to the work dir. The PE
/// entry is always the `ac_start` wrapper (ac-core contract).
fn link_only(
    obj_bytes: &[u8],
    cfg: &VerticalCfg,
    c: &mut StageCounters,
) -> Result<(PathBuf, u64), DriverError> {
    let t0 = now_ms();
    let obj_path = cfg.work_dir.join("link.obj");
    std::fs::write(&obj_path, obj_bytes)
        .map_err(|e| DriverError::Internal(format!("stage obj: {}", e)))?;
    let exe_path = cfg.work_dir.join("app.exe");
    let _ = std::fs::remove_file(&exe_path);
    let lld = link::find_rust_lld()
        .map_err(|d| DriverError::Link(d.render("", "").trim_end().to_string()))?;
    let sdk = link::find_sdk_um_dir()
        .map_err(|d| DriverError::Link(d.render("", "").trim_end().to_string()))?;
    let staged =
        link::link_object_staged(&lld, &sdk, &obj_path, &exe_path, native::START_SYMBOL, &[])
            .map_err(|d| DriverError::Link(d.render("", "").trim_end().to_string()))?;
    std::fs::rename(&staged.tmp_exe, &exe_path)
        .map_err(|e| DriverError::Internal(format!("publish staged exe: {}", e)))?;
    c.link += 1;
    Ok((exe_path, now_ms().saturating_sub(t0)))
}

fn finish(
    cfg: &VerticalCfg,
    c: StageCounters,
    hits: HitReport,
    artifacts: Vec<(String, String)>,
    exe: &Path,
    snapshot: &SourceSnapshotId,
    link_ms: u64,
    t0: u64,
) -> VerticalOutcome {
    VerticalOutcome {
        project: cfg.project.clone(),
        counters: c,
        hits,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        exe: exe.display().to_string(),
        artifacts,
        snapshot_id: snapshot.0.clone(),
        link_ms,
        total_ms: now_ms().saturating_sub(t0),
    }
}

/// Replace the project's soft roots with this version's artifacts (old
/// versions become unreachable → GC-able; that is what makes churn
/// converge — PLAN-783 §5.4).
pub fn set_current_roots(
    store: &ManagedStore,
    project: &str,
    artifacts: &[(String, String)],
) -> Result<(), DriverError> {
    store.remove_project_roots(project)?;
    for (_, id) in artifacts {
        store.register_root(project, id, "soft")?;
    }
    Ok(())
}

/// Run a produced exe and record semantics (exit/stdout/stderr). The
/// deadline defaults to the probe's 60 s; tests tighten it via
/// `AC_PROBE_RUN_DEADLINE_MS` (shortening is test-scope only — production
/// runs never extend it here).
pub fn run_and_record(
    exe: &Path,
    c: &mut StageCounters,
) -> Result<(Option<i32>, String, String), DriverError> {
    let deadline_ms: u64 = std::env::var("AC_PROBE_RUN_DEADLINE_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|ms| *ms > 0)
        .unwrap_or(crate::SUBPROC_DEADLINE_SECS * 1000);
    let mut cmd = std::process::Command::new(exe);
    cmd.current_dir(exe.parent().unwrap_or(Path::new(".")));
    let r = crate::subproc::run_deadlined(cmd, std::time::Duration::from_millis(deadline_ms))
        .map_err(|e| DriverError::Run(e.to_string()))?;
    c.run += 1;
    if r.timed_out {
        return Err(DriverError::Run("exe exceeded deadline".into()));
    }
    Ok((r.code, r.stdout, r.stderr))
}

/// Convenience wrapper: vertical + run, filling the outcome's semantic
/// fields (the shape every test and the node bin consumes).
pub fn vertical_and_run(
    store: &ManagedStore,
    cfg: &VerticalCfg,
) -> Result<VerticalOutcome, DriverError> {
    let mut out = vertical(store, cfg)?;
    let exe = PathBuf::from(&out.exe);
    let (code, stdout, stderr) = run_and_record(&exe, &mut out.counters)?;
    out.exit_code = code;
    out.stdout = stdout;
    out.stderr = stderr;
    Ok(out)
}
