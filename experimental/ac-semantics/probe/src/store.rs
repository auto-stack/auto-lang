//! Isolated-namespace store consumer (PLAN-796 T-01/T-04 + R4 repairs).
//!
//! Namespace decisions (contract-decision D8): layer `S796` (never an
//! A-layer registration of the OLD producer), carrier `SnapshotRecord`
//! (never `Hir`), contract-page payload admitted on read (schema/
//! producer/codec/structure), and full CONSUMER identity admission:
//! page kind, target logical id, AND facts ownership (F-796-R4-09) —
//! a legal store frame carrying another declaration's facts is a
//! controlled rejection, at publish and at every read.
//!
//! Outcome model (F-796-R4-02): one query runs through
//! [`run_query_flow`] and yields a MUTUALLY-EXCLUSIVE typed
//! disposition — never a bare cache string that can masquerade as
//! success. Source provenance (F-796-R4-05) separates semantic-content
//! reuse from position freshness. Required data (F-796-R4-07) wires
//! real indirect refs and store edges so retained records keep their
//! referenced type pages alive across GC.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{ManagedStore, PutRequest, StoreError};
use ac_sema_contract::{
    decode_page, encode_page, CachedResult, InferFacts, ModuleIndex, PageBody, PageHeader,
    PageKind, RecordMeta, ResultFacts, SemaPage, TypeDesc, SEMA_PAGE_SCHEMA_V1,
};
use ac_sema_core::query::{current_fingerprints, freshness_class, QueryKind};
use ac_sema_core::{InferOutcome, InferSession, SemaInput, SemaQueryKey};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

/// The isolated semantic namespace layer (contract-decision D8).
pub const SEMA_LAYER: &str = "S796";

/// Carrier chunk kind: the generic versioned-record framing (never
/// `ChunkKind::Hir`).
pub const SEMA_CARRIER: ChunkKind = ChunkKind::SnapshotRecord;

/// Session wall for one in-process query flow (the RUNNER worker wall
/// is owned by the parent side; this bounds library callers).
pub const FLOW_WALL_MS: u128 = 5_000;

fn store_key(k: &SemaQueryKey) -> QueryKey {
    QueryKey {
        layer: SEMA_LAYER,
        components: k.components.clone(),
    }
}

/// What a lookup found (never computed here).
#[derive(Debug)]
pub enum FetchOutcome {
    /// Payload decoded, manifest fully fresh — positions valid as-is.
    Hit { page: SemaPage, artifact_id: String },
    /// Key matched and every dependency is fresh EXCEPT the module
    /// source fingerprint (F-796-R4-05): semantic CONTENT is reusable,
    /// the cached positions belong to the OLD source. Never served as
    /// plain positions — the caller must refresh the projection.
    HitStaleProvenance {
        page: SemaPage,
        artifact_id: String,
        reason: String,
    },
    /// A semantic dependency drifted (signature/body/sets/negative).
    Stale { reason: String },
    /// No artifact under the key.
    Miss,
}

/// Controlled probe errors.
#[derive(Debug)]
pub enum ProbeError {
    Store(StoreError),
    Codec(ac_sema_contract::CodecError),
    KindMismatch {
        expected: String,
        found: String,
    },
    TargetMismatch {
        found: String,
        expected: String,
    },
    /// F-796-R4-09: the cached/published facts do not belong to the
    /// queried declaration.
    FactsMismatch {
        detail: String,
    },
    UnknownTarget {
        what: &'static str,
        name: String,
    },
    NotCacheable(&'static str),
    /// F-796-R4-05: a projection refresh diverged from the cached
    /// semantic content — reported, never silently overwritten.
    CacheDivergence {
        detail: String,
    },
    /// F-796-R4-07: mutually-referential record groups need group
    /// publication, which this probe version does not implement.
    CyclicRecordGroup {
        cycle: String,
    },
    BudgetRejected(crate::budget::BudgetRejected),
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbeError::Store(e) => write!(f, "store: {e}"),
            ProbeError::Codec(e) => write!(f, "codec: {e}"),
            ProbeError::KindMismatch { expected, found } => {
                write!(f, "page kind mismatch: expected {expected}, found {found}")
            }
            ProbeError::TargetMismatch { found, expected } => {
                write!(
                    f,
                    "cached target mismatch: found {found}, expected {expected}"
                )
            }
            ProbeError::FactsMismatch { detail } => write!(f, "facts ownership mismatch: {detail}"),
            ProbeError::UnknownTarget { what, name } => write!(f, "unknown {what} `{name}`"),
            ProbeError::NotCacheable(w) => write!(f, "outcome not cacheable: {w}"),
            ProbeError::CacheDivergence { detail } => {
                write!(
                    f,
                    "projection refresh diverged from cached content: {detail}"
                )
            }
            ProbeError::CyclicRecordGroup { cycle } => {
                write!(
                    f,
                    "cyclic record group needs group publication (unsupported): {cycle}"
                )
            }
            ProbeError::BudgetRejected(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ProbeError {}

impl From<StoreError> for ProbeError {
    fn from(e: StoreError) -> Self {
        ProbeError::Store(e)
    }
}

impl From<ac_sema_contract::CodecError> for ProbeError {
    fn from(e: ac_sema_contract::CodecError) -> Self {
        ProbeError::Codec(e)
    }
}

/// Expected page kind per query kind (consumer admission).
fn expected_kind(query: &QueryKind) -> PageKind {
    match query {
        QueryKind::InferBody { .. } => PageKind::CachedResult,
        QueryKind::FnSig { .. } => PageKind::FnMeta,
        QueryKind::RecordFields { .. } => PageKind::RecordMeta,
        QueryKind::ListCandidates { .. } => PageKind::ModuleIndex,
    }
}

/// Consumer-side ownership admission for a CachedResult body
/// (F-796-R4-09): query tag, target logical id and facts variant must
/// all belong to THIS query.
fn admit_cached(
    c: &CachedResult,
    query: &QueryKind,
    expect_target: &ac_sema_contract::LogicalId,
    input: &SemaInput,
) -> Result<(), ProbeError> {
    if c.query != query.as_str() {
        return Err(ProbeError::FactsMismatch {
            detail: format!(
                "page query tag `{}` does not match `{}`",
                c.query,
                query.as_str()
            ),
        });
    }
    if c.target != *expect_target {
        return Err(ProbeError::TargetMismatch {
            found: c.target.as_str().to_string(),
            expected: expect_target.as_str().to_string(),
        });
    }
    match (&c.facts, query) {
        (ResultFacts::Infer(f), QueryKind::InferBody { fn_name }) => {
            let facts_target = ac_sema_contract::logical_id(
                &ac_sema_contract::PackageId::new(&input.package),
                &input.module,
                ac_sema_contract::DeclKind::Fn,
                fn_name,
            );
            if f.fn_logical != facts_target {
                return Err(ProbeError::FactsMismatch {
                    detail: format!(
                        "page facts fn_logical `{}` does not belong to `{}`",
                        f.fn_logical.as_str(),
                        facts_target.as_str()
                    ),
                });
            }
            Ok(())
        }
        (ResultFacts::FnSig(_), QueryKind::FnSig { .. })
        | (ResultFacts::Fields(_), QueryKind::RecordFields { .. })
        | (ResultFacts::Candidates(_), QueryKind::ListCandidates { .. }) => Ok(()),
        _ => Err(ProbeError::FactsMismatch {
            detail: "facts variant does not match the query kind".into(),
        }),
    }
}

/// Producer-side ownership admission (F-796-R4-09): the summary being
/// published must BELONG to the query target.
fn admit_summary_ownership(
    summary: &ac_sema_core::InferSummary,
    query: &QueryKind,
    target: &ac_sema_contract::LogicalId,
) -> Result<(), ProbeError> {
    match query {
        QueryKind::InferBody { .. } => {
            if summary.facts.fn_logical != *target {
                return Err(ProbeError::FactsMismatch {
                    detail: format!(
                        "summary fn_logical `{}` does not match the query target `{}`",
                        summary.facts.fn_logical.as_str(),
                        target.as_str()
                    ),
                });
            }
            Ok(())
        }
        _ => Err(ProbeError::NotCacheable(
            "only infer-body summaries publish through this API",
        )),
    }
}

/// Lookup WITHOUT any compute path. Decodes, admits kind/target/facts
/// ownership, and classifies freshness INCLUDING the
/// provenance-only-drift case.
pub fn lookup_only(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
) -> Result<FetchOutcome, ProbeError> {
    let key = SemaQueryKey::build(query, input);
    let loaded = match store.get_artifact(SEMA_LAYER, &store_key(&key))? {
        Some(l) => l,
        None => {
            // F-796-R5 (F07): a RecordFields miss may be a member of a
            // recursive GROUP page published under the group's canonical
            // key — derive the record's unit from the CURRENT input
            // (deterministic SCC partition) and retry through the group
            // key with member admission
            let QueryKind::RecordFields { record } = query else {
                return Ok(FetchOutcome::Miss);
            };
            let Some(unit) = scc_units(input)
                .into_iter()
                .find(|u| u.iter().any(|n| n == record))
            else {
                return Ok(FetchOutcome::Miss);
            };
            let is_group = unit.len() > 1
                || record_deps(input, record).iter().any(|d| d == record);
            if !is_group {
                return Ok(FetchOutcome::Miss);
            }
            let mut member_names = unit.clone();
            member_names.sort();
            let group_key = QueryKey {
                layer: SEMA_LAYER,
                components: vec![
                    ("query".into(), "record-group".into()),
                    ("package".into(), input.package.clone()),
                    ("module".into(), input.module.clone()),
                    ("members".into(), member_names.join("+")),
                ],
            };
            match store.get_artifact(SEMA_LAYER, &group_key)? {
                Some(l) => l,
                None => return Ok(FetchOutcome::Miss),
            }
        }
    };
    let page = decode_page(&loaded.payload)?;
    let expected = expected_kind(query);
    // F-796-R5 (F07): a RecordFields query may be served by the
    // member's recursive GROUP page — admitted below by membership
    let group_ok_for_fields = page.header.kind == PageKind::RecGroup
        && matches!(query, QueryKind::RecordFields { .. });
    if page.header.kind != expected && !group_ok_for_fields {
        return Err(ProbeError::KindMismatch {
            expected: format!("{expected:?}"),
            found: format!("{:?}", page.header.kind),
        });
    }
    if group_ok_for_fields {
        let QueryKind::RecordFields { record } = query else { unreachable!() };
        if let PageBody::RecGroup(members) = &page.body {
            let member_target = ac_sema_contract::logical_id(
                &ac_sema_contract::PackageId::new(&input.package),
                &input.module,
                ac_sema_contract::DeclKind::Record,
                record,
            );
            if members.iter().any(|m| m.decl.logical == member_target) {
                return Ok(FetchOutcome::Hit { page, artifact_id: loaded.artifact_id });
            }
            return Err(ProbeError::TargetMismatch {
                found: "a recursive group not containing the queried record".into(),
                expected: member_target.as_str().to_string(),
            });
        }
        unreachable!("group_ok_for_fields implies a RecGroup body");
    }
    let expect_target = ac_sema_core::query::target_logical(input, query);
    match &page.body {
        PageBody::CachedResult(c) => admit_cached(c, query, &expect_target, input)?,
        PageBody::FnMeta(m) => {
            if m.decl.logical != expect_target {
                return Err(ProbeError::TargetMismatch {
                    found: m.decl.logical.as_str().to_string(),
                    expected: expect_target.as_str().to_string(),
                });
            }
        }
        PageBody::RecordMeta(m) => {
            if m.shape.decl.logical != expect_target {
                return Err(ProbeError::TargetMismatch {
                    found: m.shape.decl.logical.as_str().to_string(),
                    expected: expect_target.as_str().to_string(),
                });
            }
        }
        PageBody::ModuleIndex(_) | PageBody::InferBody(_) | PageBody::RecGroup(_) => {}
    }
    if let PageBody::CachedResult(c) = &page.body {
        let current = current_fingerprints(input);
        match freshness_class(&c.manifest, &current) {
            ac_sema_core::Freshness::Fresh => Ok(FetchOutcome::Hit {
                page,
                artifact_id: loaded.artifact_id,
            }),
            ac_sema_core::Freshness::ProvenanceStale { reason } => {
                Ok(FetchOutcome::HitStaleProvenance {
                    page,
                    artifact_id: loaded.artifact_id,
                    reason,
                })
            }
            ac_sema_core::Freshness::Stale { reason } => Ok(FetchOutcome::Stale { reason }),
        }
    } else {
        Ok(FetchOutcome::Hit {
            page,
            artifact_id: loaded.artifact_id,
        })
    }
}

/// Mutually-exclusive typed disposition of one query flow
/// (F-796-R4-02): exactly one outcome; `cache` words never double as
/// success claims.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Disposition {
    /// Clean store hit; zero compute; positions valid.
    Hit,
    /// Semantic content reused after a projection refresh (positions
    /// republished for the current source).
    Reprojected,
    /// Fresh compute published under a new key (miss).
    PublishedMiss,
    /// Fresh compute published after a semantic-dependency drift.
    PublishedStale,
    /// Definitive type diagnostics (NOT published through this flow).
    Diags,
    /// Infrastructure resource exhaustion (never cached).
    ResourceExhausted,
    /// Cancellation (never cached).
    Cancelled,
}

impl Disposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Disposition::Hit => "hit",
            Disposition::Reprojected => "reprojected",
            Disposition::PublishedMiss => "published-miss",
            Disposition::PublishedStale => "published-stale",
            Disposition::Diags => "diags",
            Disposition::ResourceExhausted => "resource",
            Disposition::Cancelled => "cancelled",
        }
    }

    /// The typed success/error split the RUNNER judges on: these are
    /// the only language-success dispositions.
    pub fn is_ok(self) -> bool {
        matches!(
            self,
            Disposition::Hit
                | Disposition::Reprojected
                | Disposition::PublishedMiss
                | Disposition::PublishedStale
        )
    }
}

/// One staged query report: disposition + facts + non-overlapping stage
/// spans (request / cache-read-validate / infer / publish) + compute
/// counter (F-796-R4-10 evidence surface).
#[derive(Clone, Debug)]
pub struct QueryReport {
    pub disposition: Disposition,
    pub compute_calls: u32,
    pub bindings: Vec<(String, String)>,
    pub diagnostics: Vec<(String, String)>,
    pub stages: Vec<(&'static str, u128)>,
    pub io_bytes: u64,
    /// Miss/reproject audit reason.
    pub reason: Option<String>,
}

/// Span-free semantic content of an inference result (the reusable
/// part under position drift): binding types + diagnostic codes and
/// messages, in program order.
fn content_signature(f: &InferFacts) -> Vec<u8> {
    let mut out = Vec::new();
    for (n, t) in &f.bindings {
        out.extend_from_slice(n.as_bytes());
        out.push(0);
        out.extend_from_slice(format!("{t:?}").as_bytes());
        out.push(0);
    }
    out.push(b'#');
    for d in &f.diagnostics {
        out.extend_from_slice(d.code.as_bytes());
        out.push(0);
        out.extend_from_slice(d.message.as_bytes());
        out.push(0);
    }
    out
}

fn binding_strings(f: &InferFacts) -> Vec<(String, String)> {
    f.bindings
        .iter()
        .map(|(n, t)| (n.clone(), desc_name(t)))
        .collect()
}

fn diag_strings(f: &InferFacts) -> Vec<(String, String)> {
    f.diagnostics
        .iter()
        .map(|d| (d.code.to_string(), d.message.clone()))
        .collect()
}

fn desc_name(t: &TypeDesc) -> String {
    match t {
        TypeDesc::I32 => "i32".into(),
        TypeDesc::Bool => "bool".into(),
        TypeDesc::Opaque { name } => format!("opaque:{name}"),
        other => format!("{other:?}"),
    }
}

/// The full staged query flow with typed dispositions and per-stage
/// timing — the ONE implementation behind the library callers, the
/// bin, the worker and the runner.
pub fn run_query_flow(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    project: Option<&str>,
    mut meter: Option<&mut crate::budget::SessionMeter>,
) -> Result<QueryReport, ProbeError> {
    let mut stages: Vec<(&'static str, u128)> = Vec::new();
    let mut io_bytes = 0u64;

    let t = Instant::now();
    let _key = SemaQueryKey::build(query, input);
    stages.push(("request", t.elapsed().as_nanos()));

    let t = Instant::now();
    let fetch = lookup_only(store, input, query)?;
    stages.push(("cache-read-validate", t.elapsed().as_nanos()));

    match fetch {
        FetchOutcome::Hit { page, .. } => {
            io_bytes += encode_page(&page).map(|b| b.len() as u64).unwrap_or(0);
            let (bindings, diagnostics) = match &page.body {
                PageBody::CachedResult(CachedResult {
                    facts: ResultFacts::Infer(f),
                    ..
                }) => (binding_strings(f), diag_strings(f)),
                _ => (Vec::new(), Vec::new()),
            };
            Ok(QueryReport {
                disposition: Disposition::Hit,
                compute_calls: 0,
                bindings,
                diagnostics,
                stages,
                io_bytes,
                reason: None,
            })
        }
        FetchOutcome::HitStaleProvenance { page, reason, .. } => {
            // F-796-R4-05: reuse semantic CONTENT, refresh the
            // projection. The refresh re-runs the deterministic solver
            // over the CURRENT source, verifies content equality with
            // the cache, and republishes current positions. A
            // divergence is a controlled error — never served, never
            // overwritten.
            let cached_sig = match &page.body {
                PageBody::CachedResult(CachedResult {
                    facts: ResultFacts::Infer(f),
                    ..
                }) => content_signature(f),
                _ => Vec::new(),
            };
            let fn_name = match query {
                QueryKind::InferBody { fn_name } => fn_name.clone(),
                _ => {
                    return Err(ProbeError::NotCacheable(
                        "provenance refresh is infer-body only",
                    ))
                }
            };
            let t = Instant::now();
            let outcome = run_inference(input, &fn_name);
            let infer_ns = t.elapsed().as_nanos();
            match outcome {
                InferOutcome::Ok(summary) | InferOutcome::Diags(summary) => {
                    stages.push(("infer", infer_ns));
                    let fresh_sig = content_signature(&summary.facts);
                    if fresh_sig != cached_sig {
                        return Err(ProbeError::CacheDivergence {
                            detail: format!(
                                "refreshed content differs from the cached page ({} vs {} bytes)",
                                fresh_sig.len(),
                                cached_sig.len()
                            ),
                        });
                    }
                    let t = Instant::now();
                    let art = publish_infer_result(store, input, query, &summary, project)?;
                    let _ = art;
                    stages.push(("publish", t.elapsed().as_nanos()));
                    Ok(QueryReport {
                        disposition: Disposition::Reprojected,
                        compute_calls: 1,
                        bindings: binding_strings(&summary.facts),
                        diagnostics: diag_strings(&summary.facts),
                        stages,
                        io_bytes,
                        reason: Some(reason),
                    })
                }
                InferOutcome::ResourceExhausted(_) => Ok(QueryReport {
                    disposition: Disposition::ResourceExhausted,
                    compute_calls: 1,
                    bindings: Vec::new(),
                    diagnostics: Vec::new(),
                    stages,
                    io_bytes,
                    reason: None,
                }),
                InferOutcome::Cancelled => Ok(QueryReport {
                    disposition: Disposition::Cancelled,
                    compute_calls: 1,
                    bindings: Vec::new(),
                    diagnostics: Vec::new(),
                    stages,
                    io_bytes,
                    reason: None,
                }),
            }
        }
        FetchOutcome::Stale { .. } | FetchOutcome::Miss => {
            let missed = matches!(fetch, FetchOutcome::Miss);
            let reason = match &fetch {
                FetchOutcome::Stale { reason } => reason.clone(),
                _ => String::new(),
            };
            let fn_name = match query {
                QueryKind::InferBody { fn_name } => fn_name.clone(),
                _ => {
                    return Ok(QueryReport {
                        disposition: Disposition::PublishedMiss,
                        compute_calls: 0,
                        bindings: Vec::new(),
                        diagnostics: Vec::new(),
                        stages,
                        io_bytes,
                        reason: None,
                    })
                }
            };
            let t = Instant::now();
            let outcome = run_inference(input, &fn_name);
            let infer_ns = t.elapsed().as_nanos();
            match outcome {
                InferOutcome::Ok(summary) => {
                    stages.push(("infer", infer_ns));
                    // the publish scratch reservation is part of the
                    // REAL flow: an over-budget ledger rejects HERE as
                    // a typed resource state, never a silent skip
                    let _guard = match meter.as_mut() {
                        None => None,
                        Some(m) => match m.scratch(64 * 1024) {
                            Ok(g) => Some(g),
                            Err(e) => {
                                return Ok(QueryReport {
                                    disposition: Disposition::ResourceExhausted,
                                    compute_calls: 1,
                                    bindings: Vec::new(),
                                    diagnostics: Vec::new(),
                                    stages,
                                    io_bytes,
                                    reason: Some(format!("{e}")),
                                })
                            }
                        },
                    };
                    let t = Instant::now();
                    let art = publish_infer_result(store, input, query, &summary, project)?;
                    drop(_guard);
                    let _ = art;
                    stages.push(("publish", t.elapsed().as_nanos()));
                    Ok(QueryReport {
                        disposition: if missed {
                            Disposition::PublishedMiss
                        } else {
                            Disposition::PublishedStale
                        },
                        compute_calls: 1,
                        bindings: binding_strings(&summary.facts),
                        diagnostics: diag_strings(&summary.facts),
                        stages,
                        io_bytes,
                        reason: if missed { None } else { Some(reason) },
                    })
                }
                // Definitive diagnostics are their own typed
                // disposition and are NOT published through this flow
                // (F-796-R4-02).
                InferOutcome::Diags(summary) => {
                    stages.push(("infer", infer_ns));
                    Ok(QueryReport {
                        disposition: Disposition::Diags,
                        compute_calls: 1,
                        bindings: binding_strings(&summary.facts),
                        diagnostics: diag_strings(&summary.facts),
                        stages,
                        io_bytes,
                        reason: if missed { None } else { Some(reason) },
                    })
                }
                InferOutcome::ResourceExhausted(_) => Ok(QueryReport {
                    disposition: Disposition::ResourceExhausted,
                    compute_calls: 1,
                    bindings: Vec::new(),
                    diagnostics: Vec::new(),
                    stages,
                    io_bytes,
                    reason: None,
                }),
                InferOutcome::Cancelled => Ok(QueryReport {
                    disposition: Disposition::Cancelled,
                    compute_calls: 1,
                    bindings: Vec::new(),
                    diagnostics: Vec::new(),
                    stages,
                    io_bytes,
                    reason: None,
                }),
            }
        }
    }
}

/// Publish a body-inference result page. Ownership is admitted BEFORE
/// any store write (F-796-R4-09).
pub fn publish_infer_result(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    summary: &ac_sema_core::InferSummary,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let target = ac_sema_core::query::target_logical(input, query);
    admit_summary_ownership(summary, query, &target)?;
    let manifest = ac_sema_core::manifest_from(
        &summary.reads,
        input,
        &match query {
            QueryKind::InferBody { fn_name } => fn_name.clone(),
            _ => String::new(),
        },
    );
    let cached = CachedResult {
        query: query.as_str(),
        target,
        facts: ResultFacts::Infer(summary.facts.clone()),
        manifest,
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V1.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::CachedResult,
            content: ac_sema_core::query::target_content(input, query),
        },
        body: PageBody::CachedResult(cached),
    };
    publish_page(store, input, query, &page, project)
}

/// Publish a module index page (declaration candidate sets).
pub fn publish_module_index(
    store: &ManagedStore,
    input: &SemaInput,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let query = QueryKind::ListCandidates { domain: "fn" };
    let index = ModuleIndex {
        package: ac_sema_contract::PackageId::new(&input.package),
        module: input.module.clone(),
        fn_names: input.fn_candidates(),
        record_names: input.record_candidates(),
        decl_versions: {
            let mut v = Vec::new();
            for f in &input.fns {
                v.push((
                    ac_sema_contract::logical_id(
                        &ac_sema_contract::PackageId::new(&input.package),
                        &input.module,
                        ac_sema_contract::DeclKind::Fn,
                        &f.name,
                    ),
                    ac_sema_contract::ContentVersion::of_canonical(
                        &ac_sema_core::query::fn_sig_canonical(f),
                    ),
                ));
            }
            v.sort_by(|a, b| a.0.cmp(&b.0));
            v
        },
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V1.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::ModuleIndex,
            content: ac_sema_contract::ContentVersion::of_canonical(
                &index.fn_names.join("\u{1}").into_bytes(),
            ),
        },
        body: PageBody::ModuleIndex(index),
    };
    publish_page(store, input, &query, &page, project)
}

/// Real indirect type reference for a record annotation (F-796-R4-07):
/// nominal identity via DeclRef — never an opaque name string for a
/// KNOWN record; unknown names stay descriptive-opaque. The referenced
/// record's page must ALREADY exist (module-order publishing resolves
/// the DAG; this function never recurses).
fn record_typedesc(
    store: &ManagedStore,
    input: &SemaInput,
    name: &str,
) -> Result<(TypeDesc, Option<String>), ProbeError> {
    let Some(r) = input.lookup_record(name) else {
        return Ok((
            TypeDesc::Opaque {
                name: format!("record:{name}"),
            },
            None,
        ));
    };
    let content =
        ac_sema_contract::ContentVersion::of_canonical(&ac_sema_core::query::record_canonical(r));
    let decl = ac_sema_contract::DeclRef::new(
        &ac_sema_contract::PackageId::new(&input.package),
        &input.module,
        ac_sema_contract::DeclKind::Record,
        name,
        content,
    );
    let dep_query = QueryKind::RecordFields {
        record: name.to_string(),
    };
    let artifact = match lookup_only(store, input, &dep_query)? {
        FetchOutcome::Hit { artifact_id, .. } => Some(artifact_id),
        _ => None,
    };
    Ok((TypeDesc::Record { decl }, artifact))
}

/// Records each record references (known record names only).
fn record_deps(input: &SemaInput, name: &str) -> Vec<String> {
    let Some(r) = input.lookup_record(name) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (_, a) in &r.fields {
        if let ac_sema_core::Ann::Record(rec) = a {
            if input.lookup_record(rec).is_some() && !out.iter().any(|d| d == rec) {
                out.push((*rec).to_string());
            }
        }
    }
    out
}

/// Build one record's shape with REAL indirect refs; `unit` is the set
/// of records published in the SAME unit (group members or the
/// singleton itself) — references inside the unit stay plain DeclRefs
/// (they live in the same page); references outside must already be
/// published and become required-data edges.
fn build_shape(
    store: &ManagedStore,
    input: &SemaInput,
    name: &str,
    unit: &std::collections::BTreeSet<String>,
    required_edges: &mut Vec<String>,
) -> Result<ac_sema_contract::RecordShape, ProbeError> {
    let Some(r) = input.lookup_record(name) else {
        return Err(ProbeError::UnknownTarget { what: "record", name: name.to_string() });
    };
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    let mut fields: Vec<ac_sema_contract::FieldDesc> = Vec::new();
    for (n, a) in &r.fields {
        let ty = match a {
            ac_sema_core::Ann::I32 => TypeDesc::I32,
            ac_sema_core::Ann::Bool => TypeDesc::Bool,
            ac_sema_core::Ann::Record(rec) => {
                let (inner, artifact) = record_typedesc(store, input, rec)?;
                if let (TypeDesc::Record { .. }, Some(art)) = (&inner, artifact) {
                    if !unit.contains::<str>(rec) {
                        required_edges.push(art);
                    }
                }
                inner
            }
            ac_sema_core::Ann::Inferred => TypeDesc::Opaque { name: "?inferred".into() },
            ac_sema_core::Ann::Unsupported(w) => {
                TypeDesc::Opaque { name: format!("unsupported:{w}") }
            }
        };
        fields.push(ac_sema_contract::FieldDesc { name: n.clone(), ty });
    }
    Ok(ac_sema_contract::RecordShape {
        decl: ac_sema_contract::DeclRef::new(
            &pkg,
            &input.module,
            ac_sema_contract::DeclKind::Record,
            name,
            ac_sema_contract::ContentVersion::of_canonical(&ac_sema_core::query::record_canonical(r)),
        ),
        fields,
    })
}

/// SCC partition (iterative Kosaraju) of the record reference graph.
/// Returns units in DEPENDENCY ORDER (referenced groups first); a unit
/// is a singleton without self-reference or a recursive group.
fn scc_units(input: &SemaInput) -> Vec<Vec<String>> {
    use std::collections::BTreeSet;
    let names: Vec<String> = input.records.iter().map(|r| r.name.clone()).collect();
    let name_set: BTreeSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let adj = |n: &str| -> Vec<String> {
        record_deps(input, n)
            .into_iter()
            .filter(|d| name_set.contains(d.as_str()))
            .collect()
    };
    // 1st pass: DFS finish order (iterative)
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    for start in &names {
        if visited.contains(start) {
            continue;
        }
        let mut stack: Vec<(String, usize)> = vec![(start.clone(), 0)];
        visited.insert(start.clone());
        while let Some((node, idx)) = stack.pop() {
            let succ = adj(&node);
            if idx < succ.len() {
                stack.push((node.clone(), idx + 1));
                let next = succ[idx].clone();
                if visited.insert(next.clone()) {
                    stack.push((next, 0));
                }
            } else {
                order.push(node.clone());
            }
        }
    }
    // 2nd pass on the REVERSE graph in reverse finish order — the
    // reverse adjacency is BUILT ONCE (a per-node scan is O(V^2) and
    // made the deep-chain lookups quadratic; this is the fix)
    let mut reverse: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for n in &names {
        reverse.insert(n.clone(), Vec::new());
    }
    for n in &names {
        for d in adj(n) {
            reverse.entry(d).or_default().push(n.clone());
        }
    }
    let radj = |n: &str| -> &[String] {
        reverse.get(n).map(|v| v.as_slice()).unwrap_or(&[])
    };
    let mut assigned: BTreeSet<String> = BTreeSet::new();
    let mut units: Vec<Vec<String>> = Vec::new();
    for start in order.iter().rev() {
        if assigned.contains(start) {
            continue;
        }
        let mut comp: Vec<String> = Vec::new();
        let mut stack = vec![start.clone()];
        assigned.insert(start.clone());
        while let Some(node) = stack.pop() {
            comp.push(node.clone());
            for next in radj(&node) {
                if assigned.insert(next.clone()) {
                    stack.push(next.clone());
                }
            }
        }
        units.push(comp);
    }
    // Kosaraju emits SCCs in reverse topological condensation order;
    // reverse so a unit's dependencies come FIRST
    units.reverse();
    units
}

/// Publish (or reuse) one record's page whose referenced pages ALL
/// exist already. Non-recursive: the caller resolves the DAG order.
fn ensure_record_page(
    store: &ManagedStore,
    input: &SemaInput,
    name: &str,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let query = QueryKind::RecordFields {
        record: name.to_string(),
    };
    if let FetchOutcome::Hit { artifact_id, .. } = lookup_only(store, input, &query)? {
        return Ok(artifact_id);
    }
    let Some(r) = input.lookup_record(name) else {
        return Err(ProbeError::UnknownTarget {
            what: "record",
            name: name.to_string(),
        });
    };
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    let mut required_edges: Vec<String> = Vec::new();
    let mut shape_fields: Vec<ac_sema_contract::FieldDesc> = Vec::new();
    for (n, a) in &r.fields {
        let ty = match a {
            ac_sema_core::Ann::I32 => TypeDesc::I32,
            ac_sema_core::Ann::Bool => TypeDesc::Bool,
            ac_sema_core::Ann::Record(rec) => {
                let (inner, artifact) = record_typedesc(store, input, rec)?;
                // the referenced page is REQUIRED DATA for this page: a
                // retained record keeps its referenced type pages alive
                // across GC (F-796-R4-07)
                if let (TypeDesc::Record { .. }, Some(art)) = (&inner, artifact) {
                    required_edges.push(art);
                }
                inner
            }
            ac_sema_core::Ann::Inferred => TypeDesc::Opaque {
                name: "?inferred".into(),
            },
            ac_sema_core::Ann::Unsupported(w) => TypeDesc::Opaque {
                name: format!("unsupported:{w}"),
            },
        };
        shape_fields.push(ac_sema_contract::FieldDesc {
            name: n.clone(),
            ty,
        });
    }
    let shape = ac_sema_contract::RecordShape {
        decl: ac_sema_contract::DeclRef::new(
            &pkg,
            &input.module,
            ac_sema_contract::DeclKind::Record,
            name,
            ac_sema_contract::ContentVersion::of_canonical(&ac_sema_core::query::record_canonical(
                r,
            )),
        ),
        fields: shape_fields,
    };
    let group = ac_sema_contract::RecGroup {
        members: vec![shape.clone()],
    };
    let meta = RecordMeta {
        shape,
        methods: r
            .methods
            .iter()
            .map(|m| {
                let content = ac_sema_contract::ContentVersion::of_canonical(
                    &ac_sema_core::query::fn_sig_canonical(m),
                );
                ac_sema_contract::FnMeta {
                    decl: ac_sema_contract::DeclRef::new(
                        &pkg,
                        &input.module,
                        ac_sema_contract::DeclKind::Fn,
                        &m.name,
                        content,
                    ),
                    params: m
                        .params
                        .iter()
                        .map(|(n, a)| (n.clone(), plain_desc(a), ac_sema_contract::Mode::View))
                        .collect(),
                    ret: plain_desc(&m.ret),
                    receiver: "view",
                }
            })
            .collect(),
        group_content: group.group_content(),
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V1.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::RecordMeta,
            content: meta.group_content.clone(),
        },
        body: PageBody::RecordMeta(meta),
    };
    let key = SemaQueryKey::build(&query, input);
    let payload = encode_page(&page)?;
    let edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = required_edges
        .into_iter()
        .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData))
        .collect();
    let req = PutRequest {
        layer: SEMA_LAYER,
        key: store_key(&key),
        kind: SEMA_CARRIER,
        payload,
        edges,
        project: project.map(|s| s.to_string()),
    };
    Ok(store.put_artifact(req)?)
}

/// Publish a record metadata page with REAL indirect refs and
/// required-data edges. The module's reference DAG is resolved
/// ITERATIVELY (Kahn order — arbitrary depth, no native recursion);
/// mutually-referential groups are a controlled rejection (group
/// publication is not implemented in this probe).
pub fn publish_record_meta(
    store: &ManagedStore,
    input: &SemaInput,
    record: &str,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let all = publish_module_records(store, input, project)?;
    all.get(record)
        .cloned()
        .ok_or_else(|| ProbeError::UnknownTarget {
            what: "record",
            name: record.to_string(),
        })
}

fn plain_desc(a: &ac_sema_core::Ann) -> TypeDesc {
    match a {
        ac_sema_core::Ann::I32 => TypeDesc::I32,
        ac_sema_core::Ann::Bool => TypeDesc::Bool,
        ac_sema_core::Ann::Record(r) => TypeDesc::Opaque {
            name: format!("record:{r}"),
        },
        ac_sema_core::Ann::Inferred => TypeDesc::Opaque {
            name: "?inferred".into(),
        },
        ac_sema_core::Ann::Unsupported(w) => TypeDesc::Opaque {
            name: format!("unsupported:{w}"),
        },
    }
}

fn publish_page(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    page: &SemaPage,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let payload = encode_page(page)?;
    let key = SemaQueryKey::build(query, input);
    let req = PutRequest {
        layer: SEMA_LAYER,
        key: store_key(&key),
        kind: SEMA_CARRIER,
        payload,
        edges: Vec::new(),
        project: project.map(|s| s.to_string()),
    };
    Ok(store.put_artifact(req)?)
}

/// Run the REAL inference (the new solver) with the flow wall applied
/// as a hard backstop for library callers without a parent wall.
pub fn run_inference(input: &SemaInput, fn_name: &str) -> InferOutcome {
    let started = Instant::now();
    let outcome = InferSession::new(input).infer_function(fn_name);
    let elapsed = started.elapsed().as_millis();
    if elapsed > FLOW_WALL_MS {
        return InferOutcome::ResourceExhausted(ac_sema_core::BudgetError::DeadlineExceeded {
            elapsed_ms: elapsed,
            max_ms: FLOW_WALL_MS,
        });
    }
    outcome
}

/// Open (creating when absent) the managed store at `root`.
pub fn open_store(root: &Path) -> Result<ManagedStore, ProbeError> {
    Ok(ManagedStore::open(root)?)
}

/// Map record names to their published page artifacts for a whole
/// module. Publish order resolves the reference DAG ITERATIVELY
/// (Kahn): a record publishes when every referenced record's page
/// already exists; no progress with records remaining = a reference
/// cycle, which needs group publication and is a controlled rejection.
pub fn publish_module_records(
    store: &ManagedStore,
    input: &SemaInput,
    project: Option<&str>,
) -> Result<BTreeMap<String, String>, ProbeError> {
    let mut out = BTreeMap::new();
    for unit in scc_units(input) {
        let unit_set: std::collections::BTreeSet<String> = unit.iter().cloned().collect();
        let is_group =
            unit.len() > 1 || unit.iter().any(|n| record_deps(input, n).iter().any(|d| d == n));
        if !is_group {
            // singleton without self-reference: single page (reused if
            // a page for exactly this record already exists)
            let name = &unit[0];
            if let Ok(FetchOutcome::Hit { artifact_id, .. }) = lookup_only(
                store,
                input,
                &QueryKind::RecordFields { record: name.clone() },
            ) {
                out.insert(name.clone(), artifact_id);
                continue;
            }
            let art = ensure_record_page(store, input, name, project)?;
            out.insert(name.clone(), art);
            continue;
        }
        // recursive group: ONE page for the whole SCC (internal refs
        // are plain DeclRefs inside the page; external refs must be
        // published already and become required-data edges)
        let mut required_edges: Vec<String> = Vec::new();
        let mut members: Vec<ac_sema_contract::RecordShape> = Vec::new();
        for name in &unit {
            members.push(build_shape(store, input, name, &unit_set, &mut required_edges)?);
        }
        let group = ac_sema_contract::RecGroup { members: members.clone() };
        let page = SemaPage {
            header: PageHeader {
                schema: SEMA_PAGE_SCHEMA_V1.into(),
                producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
                codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
                kind: PageKind::RecGroup,
                content: group.group_content(),
            },
            body: PageBody::RecGroup(members),
        };
        // key: the group's canonical identity (sorted member names)
        let mut member_names = unit.clone();
        member_names.sort();
        let key = ac_sema_core::SemaQueryKey {
            components: vec![
                ("query".into(), "record-group".into()),
                ("package".into(), input.package.clone()),
                ("module".into(), input.module.clone()),
                ("members".into(), member_names.join("+")),
            ],
        };
        let payload = encode_page(&page)?;
        let edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = required_edges
            .into_iter()
            .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData))
            .collect();
        let req = PutRequest {
            layer: SEMA_LAYER,
            key: QueryKey { layer: SEMA_LAYER, components: key.components.clone() },
            kind: SEMA_CARRIER,
            payload,
            edges,
            project: project.map(|s| s.to_string()),
        };
        let art = store.put_artifact(req)?;
        for name in &unit {
            out.insert(name.clone(), art.clone());
        }
    }
    Ok(out)
}
