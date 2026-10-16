//! Isolated-namespace store consumer (PLAN-796 T-01/T-04).
//!
//! Namespace decisions (contract-decision D8):
//! * layer `S796` — clearly distinct from every 783 A-layer, so a
//!   semantic artifact can never be mistaken for an A06 registration of
//!   the OLD producer ("resolve/typecheck");
//! * carrier chunk kind `SnapshotRecord` — the generic versioned-record
//!   framing; NEVER `Hir` (that would make a HIR kind impersonate a
//!   semantic value);
//! * the payload is always a contract page (`ac796/sema-page/v1`),
//!   admitted on read (schema/producer/codec/structure) — a foreign or
//!   corrupt payload is a controlled error, never a reinterpretation.
//!
//! Hit discipline (§5.4): a lookup NEVER computes — `lookup_only` has
//! no compute path at all (禁调反控 is structural). The full query flow
//! (`query`) fetches, validates the whole dependency manifest for
//! freshness, and only on miss invokes the caller's compute closure,
//! recording the invocation count.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{ManagedStore, PutRequest, StoreError};
use ac_sema_contract::{
    decode_page, encode_page, CachedResult, ModuleIndex, PageBody,
    PageHeader, PageKind, RecordMeta, ResultFacts, SemaPage, SEMA_PAGE_SCHEMA_V1,
};
use ac_sema_core::{
    current_fingerprints, freshness_of, infer::InferOutcome, infer::InferSession,
    manifest_from, query::QueryKind, Freshness, InferSummary, SemaInput, SemaQueryKey,
};
use std::path::Path;

/// The isolated semantic namespace layer (contract-decision D8).
pub const SEMA_LAYER: &str = "S796";

/// Carrier chunk kind: the generic versioned-record framing (see module
/// docs — never `ChunkKind::Hir`).
pub const SEMA_CARRIER: ChunkKind = ChunkKind::SnapshotRecord;

/// Map a semantic key onto the 783 store's query-key shape.
fn store_key(k: &SemaQueryKey) -> QueryKey {
    QueryKey { layer: SEMA_LAYER, components: k.components.clone() }
}

/// What a lookup found (never computed here).
pub enum FetchOutcome {
    /// Payload decoded AND the full manifest re-validated fresh.
    Hit { page: SemaPage, artifact_id: String },
    /// No artifact under the key.
    Miss,
    /// Artifact present but a dependency drifted — the miss reason is
    /// preserved for the audit trail.
    Stale { reason: String },
}

/// Result of the full query flow.
pub struct QueryFlow {
    pub outcome: FetchedOrComputed,
    /// How many times the compute closure ran (0 on a clean hit — the
    /// 禁调反控 assertion).
    pub compute_calls: u32,
}

pub enum FetchedOrComputed {
    Hit { page: SemaPage },
    Computed { summary: InferSummary },
}

/// Lookup WITHOUT any compute path (structural 禁调反控). Decodes and
/// fully re-validates the dependency manifest against the CURRENT
/// input.
pub fn lookup_only(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
) -> Result<FetchOutcome, ProbeError> {
    let key = SemaQueryKey::build(query, input);
    let Some(loaded) = store.get_artifact(SEMA_LAYER, &store_key(&key))? else {
        return Ok(FetchOutcome::Miss);
    };
    let page = decode_page(&loaded.payload)?;
    // consumer identity validation: the cached body must be the right
    // KIND and the right TARGET before any bytes are trusted further
    let CachedResult { target, .. } = match &page.body {
        PageBody::CachedResult(c) => c,
        _ => {
            return Err(ProbeError::KindMismatch {
                expected: "CachedResult",
                found: format!("{:?}", page.header.kind),
            })
        }
    };
    let expect_target = ac_sema_core::query::target_logical(input, query);
    if *target != expect_target {
        return Err(ProbeError::TargetMismatch {
            found: target.as_str().to_string(),
            expected: expect_target.as_str().to_string(),
        });
    }
    if let PageBody::CachedResult(c) = &page.body {
        let current = current_fingerprints(input);
        match freshness_of(&c.manifest, &current) {
            Freshness::Fresh => Ok(FetchOutcome::Hit { page, artifact_id: loaded.artifact_id }),
            Freshness::Stale { reason } => Ok(FetchOutcome::Stale { reason }),
        }
    } else {
        unreachable!("checked above")
    }
}

/// Publish a body-inference result page under the query key. The
/// manifest records exactly what the run read (positives + negatives).
pub fn publish_infer_result(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    summary: &InferSummary,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let target = ac_sema_core::query::target_logical(input, query);
    let manifest = manifest_from(&summary.reads, input, &match query {
        QueryKind::InferBody { fn_name } => fn_name.clone(),
        _ => String::new(),
    });
    let cached = CachedResult {
        query: query.as_str(),
        target: target.clone(),
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

/// Publish a module index page (declaration candidate sets — the input
/// of negative dependencies).
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

/// Publish a record metadata page (shape + methods + group version).
pub fn publish_record_meta(
    store: &ManagedStore,
    input: &SemaInput,
    record: &str,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let query = QueryKind::RecordFields { record: record.to_string() };
    let Some(r) = input.lookup_record(record) else {
        return Err(ProbeError::UnknownTarget { what: "record", name: record.to_string() });
    };
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    let shape = ac_sema_contract::RecordShape {
        decl: ac_sema_contract::DeclRef::new(
            &pkg,
            &input.module,
            ac_sema_contract::DeclKind::Record,
            record,
            ac_sema_contract::ContentVersion::of_canonical(
                &ac_sema_core::query::record_canonical(r),
            ),
        ),
        fields: r
            .fields
            .iter()
            .map(|(n, a)| ac_sema_contract::FieldDesc {
                name: n.clone(),
                ty: match a {
                    ac_sema_core::Ann::I32 => ac_sema_contract::TypeDesc::I32,
                    ac_sema_core::Ann::Bool => ac_sema_contract::TypeDesc::Bool,
                    ac_sema_core::Ann::Record(rec) => ac_sema_contract::TypeDesc::Opaque {
                        name: format!("record:{rec}"),
                    },
                    ac_sema_core::Ann::Inferred => {
                        ac_sema_contract::TypeDesc::Opaque { name: "?inferred".into() }
                    }
                    ac_sema_core::Ann::Unsupported(w) => {
                        ac_sema_contract::TypeDesc::Opaque { name: format!("unsupported:{w}") }
                    }
                },
            })
            .collect(),
    };
    let group = ac_sema_contract::RecGroup { members: vec![shape.clone()] };
    let meta = RecordMeta {
        shape,
        methods: r
            .methods
            .iter()
            .map(|m| ac_sema_contract::FnMeta {
                decl: ac_sema_contract::DeclRef::new(
                    &pkg,
                    &input.module,
                    ac_sema_contract::DeclKind::Fn,
                    &m.name,
                    ac_sema_contract::ContentVersion::of_canonical(
                        &ac_sema_core::query::fn_sig_canonical(m),
                    ),
                ),
                params: m
                    .params
                    .iter()
                    .map(|(n, a)| {
                        (n.clone(), ann_to_desc(a), ac_sema_contract::Mode::View)
                    })
                    .collect(),
                ret: ann_to_desc(&m.ret),
                receiver: "view",
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
    publish_page(store, input, &query, &page, project)
}

fn ann_to_desc(a: &ac_sema_core::Ann) -> ac_sema_contract::TypeDesc {
    match a {
        ac_sema_core::Ann::I32 => ac_sema_contract::TypeDesc::I32,
        ac_sema_core::Ann::Bool => ac_sema_contract::TypeDesc::Bool,
        ac_sema_core::Ann::Record(r) => {
            ac_sema_contract::TypeDesc::Opaque { name: format!("record:{r}") }
        }
        ac_sema_core::Ann::Inferred => {
            ac_sema_contract::TypeDesc::Opaque { name: "?inferred".into() }
        }
        ac_sema_core::Ann::Unsupported(w) => {
            ac_sema_contract::TypeDesc::Opaque { name: format!("unsupported:{w}") }
        }
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

/// The full query flow: fetch → validate → (miss?) compute → publish.
/// The compute closure runs ONLY on a miss/stale; its invocation count
/// is returned for the no-compute-on-hit assertion.
pub fn query<F>(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    project: Option<&str>,
    compute: F,
) -> Result<QueryFlow, ProbeError>
where
    F: FnOnce() -> InferSummary,
{
    let mut compute_calls = 0;
    match lookup_only(store, input, query)? {
        FetchOutcome::Hit { page, .. } => {
            return Ok(QueryFlow { outcome: FetchedOrComputed::Hit { page }, compute_calls });
        }
        FetchOutcome::Miss => {}
        FetchOutcome::Stale { .. } => {}
    }
    compute_calls += 1;
    let summary = compute();
    publish_infer_result(store, input, query, &summary, project)?;
    Ok(QueryFlow { outcome: FetchedOrComputed::Computed { summary }, compute_calls })
}

/// Convenience: run the REAL inference (new solver, never the old
/// typecheck) and hand back the outcome.
pub fn run_inference(input: &SemaInput, fn_name: &str) -> InferOutcome {
    InferSession::new(input).infer_function(fn_name)
}

/// Controlled probe errors.
#[derive(Debug)]
pub enum ProbeError {
    Store(StoreError),
    Codec(ac_sema_contract::CodecError),
    KindMismatch { expected: &'static str, found: String },
    TargetMismatch { found: String, expected: String },
    UnknownTarget { what: &'static str, name: String },
    NotCacheable(&'static str),
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
                write!(f, "cached target mismatch: found {found}, expected {expected}")
            }
            ProbeError::UnknownTarget { what, name } => write!(f, "unknown {what} `{name}`"),
            ProbeError::NotCacheable(w) => write!(f, "outcome not cacheable: {w}"),
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

/// Open (creating when absent) the managed store at `root`. The probe
/// never modifies 783 code and never accepts a reparse point as root.
pub fn open_store(root: &Path) -> Result<ManagedStore, ProbeError> {
    Ok(ManagedStore::open(root)?)
}
