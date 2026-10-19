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
    PageKind, RecordMeta, ResultFacts, SemaPage, TypeDesc, SEMA_PAGE_SCHEMA_V2,
};
use ac_sema_core::query::KeyEnv;
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

/// Number of full module SCC partitions built in this process. The
/// publication loop must partition ONCE (a per-record miss rebuilding
/// the whole graph was the 400s V-06 deep-chain cost); the counter is a
/// permanent regression control for that invariant.
pub static SCC_BUILDS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

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
    /// F-796-R6-01: a page reached under the CURRENT key carries a
    /// reference to a definition version that is not the current one.
    /// Never served, never overwritten — the key/closure invariant is
    /// broken and the controlled rejection names the stale ref.
    RefVersionMismatch {
        name: String,
        cached: String,
        current: String,
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
            ProbeError::RefVersionMismatch {
                name,
                cached,
                current,
            } => write!(
                f,
                "cached ref to `{name}` carries version {cached}, current definition is {current}"
            ),
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

/// Group page key (F-796-R6-01 + R7-01): membership, carrier topology
/// and the unit's TRANSITIVE effective version (member canonicals plus
/// external dependencies' effective versions). Any leaf edit, SCC
/// formation/split or external group change is a NEW key; old pages
/// stay immutable under old keys.
fn group_key(input: &SemaInput, member_names: &[String], env: &KeyEnv) -> QueryKey {
    let mut members = member_names.to_vec();
    members.sort();
    QueryKey {
        layer: SEMA_LAYER,
        components: vec![
            ("query".into(), "record-group".into()),
            ("package".into(), input.package.clone()),
            ("module".into(), input.module.clone()),
            ("members".into(), members.join("+")),
            ("algo".into(), "ac796-record-group/1".into()),
            (
                "schema".into(),
                ac_sema_contract::SEMA_PAGE_SCHEMA_V2.into(),
            ),
            ("producer".into(), ac_sema_contract::SEMA_PRODUCER_ID.into()),
            ("group-content-fp".into(), env.group_eff(&members)),
        ],
    }
}

/// Conservative reservation for one cache read + decode window
/// (F-796-R7-02): the raw payload allocation happens INSIDE the store
/// read, so the reservation must be taken BEFORE `get_artifact` (a
/// payload-size-based charge afterward would account memory that was
/// already live). The contract caps a page body at MAX_PAGE_BODY; the
/// decoded tree can be several times the payload, so reserve 3x.
const READ_RESERVE: usize = ac_sema_contract::MAX_PAGE_BODY * 3;

fn decode_loaded(
    loaded: ac_artifact_probe::store::LoadedArtifact,
) -> Result<(SemaPage, String), ProbeError> {
    let page = decode_page(&loaded.payload)?;
    Ok((page, loaded.artifact_id))
}

/// F-796-R6-01: a served page's embedded nominal refs must carry the
/// CURRENT definition version. The key now makes a mismatch structurally
/// unreachable for well-formed pages; this admission catches malformed/
/// conflicting pages instead of serving a stale ref as current.
fn admit_ref(input: &SemaInput, decl: &ac_sema_contract::DeclRef) -> Result<(), ProbeError> {
    decl.verify().map_err(|e| ProbeError::FactsMismatch {
        detail: format!("nominal ref failed verification: {e}"),
    })?;
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    if decl.package != pkg || decl.module != input.module {
        return Err(ProbeError::TargetMismatch {
            found: format!("{}::{}", decl.package.as_str(), decl.module),
            expected: format!("{}::{}", input.package, input.module),
        });
    }
    let Some(r) = input.lookup_record(&decl.name) else {
        return Err(ProbeError::RefVersionMismatch {
            name: decl.name.clone(),
            cached: decl.content.as_str().to_string(),
            current: "<record no longer declared>".into(),
        });
    };
    let current =
        ac_sema_contract::ContentVersion::of_canonical(&ac_sema_core::query::record_canonical(r));
    if current != decl.content {
        return Err(ProbeError::RefVersionMismatch {
            name: decl.name.clone(),
            cached: decl.content.as_str().to_string(),
            current: current.as_str().to_string(),
        });
    }
    Ok(())
}

fn admit_shape_refs(
    shape: &ac_sema_contract::RecordShape,
    input: &SemaInput,
) -> Result<(), ProbeError> {
    // the shape's OWN declared version is part of the read set — a page
    // carrying another revision of the same record is never served
    admit_ref(input, &shape.decl)?;
    for f in &shape.fields {
        if let TypeDesc::Record { decl } = &f.ty {
            admit_ref(input, decl)?;
        }
    }
    Ok(())
}

/// F-796-R12-01: full nominal admission for a method ref — the same
/// obligations `admit_ref` enforces for record refs, plus the owner:
/// package/module must match the CURRENT input, the method must be
/// currently declared under its owning record, and the content version
/// must equal the CURRENT signature canonical. A foreign/stale method
/// ref inside an otherwise-current page is a controlled rejection.
fn admit_method_ref(
    input: &SemaInput,
    method: &ac_sema_contract::FnMeta,
) -> Result<(), ProbeError> {
    let decl = &method.decl;
    decl.verify().map_err(|e| ProbeError::FactsMismatch {
        detail: format!("nominal ref failed verification: {e}"),
    })?;
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    if decl.package != pkg || decl.module != input.module {
        return Err(ProbeError::TargetMismatch {
            found: format!("{}::{}", decl.package.as_str(), decl.module),
            expected: format!("{}::{}", input.package, input.module),
        });
    }
    let owner = decl.owner.as_deref().unwrap_or("<ownerless>");
    let qualified = format!("{owner}.{}", decl.name);
    let Some(r) = input.lookup_record(owner) else {
        return Err(ProbeError::RefVersionMismatch {
            name: qualified,
            cached: decl.content.as_str().to_string(),
            current: "<record no longer declared>".into(),
        });
    };
    let Some(m) = r.methods.iter().find(|m| m.name == decl.name) else {
        return Err(ProbeError::RefVersionMismatch {
            name: qualified,
            cached: decl.content.as_str().to_string(),
            current: "<method no longer declared>".into(),
        });
    };
    let current =
        ac_sema_contract::ContentVersion::of_canonical(&ac_sema_core::query::fn_sig_canonical(m));
    if current != decl.content {
        return Err(ProbeError::RefVersionMismatch {
            name: qualified,
            cached: decl.content.as_str().to_string(),
            current: current.as_str().to_string(),
        });
    }
    Ok(())
}

/// The complete read-side version admission (F-796-R6-01 + R7-01):
/// page content must match the unit's CURRENT EFFECTIVE version (the
/// transitive dependency closure), the carrier topology must match the
/// current SCC partition, and every embedded nominal ref must be the
/// current declaration version. A page reached under the current key
/// that fails this is a controlled rejection, never a served stale
/// closure.
fn admit_page(
    page: &SemaPage,
    input: &SemaInput,
    query: Option<&QueryKind>,
    env: Option<&KeyEnv>,
) -> Result<(), ProbeError> {
    match &page.body {
        PageBody::RecordMeta(m) => {
            admit_shape_refs(&m.shape, input)?;
            // F-796-R11-01 + R12-01: every published method belongs to
            // THIS record (kind + owner), and carries the full nominal
            // admission — current package/module/owner/name/signature
            // content. A foreign or stale method ref is rejected.
            for method in &m.methods {
                if method.decl.kind != ac_sema_contract::DeclKind::Method
                    || method.decl.owner.as_deref() != Some(m.shape.decl.name.as_str())
                {
                    return Err(ProbeError::TargetMismatch {
                        found: format!(
                            "method `{}` with owner {:?} and kind `{}`",
                            method.decl.name,
                            method.decl.owner,
                            method.decl.kind.as_str()
                        ),
                        expected: format!("a Method decl owned by record `{}`", m.shape.decl.name),
                    });
                }
                admit_method_ref(input, method)?;
            }
            let Some(env) = env else {
                return Ok(());
            };
            let expect = env.effective(&m.shape.decl.name).ok_or_else(|| {
                ProbeError::RefVersionMismatch {
                    name: m.shape.decl.name.clone(),
                    cached: m.group_content.as_str().to_string(),
                    current: "<record no longer declared>".into(),
                }
            })?;
            if m.group_content.as_str() != expect {
                return Err(ProbeError::RefVersionMismatch {
                    name: m.shape.decl.name.clone(),
                    cached: m.group_content.as_str().to_string(),
                    current: expect.to_string(),
                });
            }
            Ok(())
        }
        PageBody::RecGroup(members) => {
            for m in members {
                admit_shape_refs(m, input)?;
            }
            let Some(env) = env else {
                return Ok(());
            };
            let Some(first) = members.first() else {
                return Err(ProbeError::TargetMismatch {
                    found: "empty recursive group".into(),
                    expected: "at least one member".into(),
                });
            };
            let expect =
                env.effective(&first.decl.name)
                    .ok_or_else(|| ProbeError::RefVersionMismatch {
                        name: first.decl.name.clone(),
                        cached: page.header.content.as_str().to_string(),
                        current: "<record no longer declared>".into(),
                    })?;
            if page.header.content.as_str() != expect {
                return Err(ProbeError::RefVersionMismatch {
                    name: first.decl.name.clone(),
                    cached: page.header.content.as_str().to_string(),
                    current: expect.to_string(),
                });
            }
            let mut got: Vec<String> = members.iter().map(|m| m.decl.name.clone()).collect();
            got.sort();
            let mut want: Vec<String> = env
                .members(&first.decl.name)
                .map(|m| m.to_vec())
                .unwrap_or_default();
            want.sort();
            if got != want {
                return Err(ProbeError::TargetMismatch {
                    found: format!("group members {}", got.join("+")),
                    expected: format!("current unit {}", want.join("+")),
                });
            }
            Ok(())
        }
        PageBody::FnMeta(m) => {
            // F-796-R13-02: full nominal admission for the embedded fn
            // decl — verify, current package/module, a top-level Fn decl
            // for a currently declared function, and the CURRENT
            // signature content version. Header equivalence cannot stand
            // in for the embedded fact.
            m.decl.verify().map_err(|e| ProbeError::FactsMismatch {
                detail: format!("nominal ref failed verification: {e}"),
            })?;
            let pkg = ac_sema_contract::PackageId::new(&input.package);
            if m.decl.package != pkg || m.decl.module != input.module {
                return Err(ProbeError::TargetMismatch {
                    found: format!("{}::{}", m.decl.package.as_str(), m.decl.module),
                    expected: format!("{}::{}", input.package, input.module),
                });
            }
            if m.decl.kind != ac_sema_contract::DeclKind::Fn || m.decl.owner.is_some() {
                return Err(ProbeError::TargetMismatch {
                    found: format!("{} `{}`", m.decl.kind.as_str(), m.decl.name),
                    expected: "a top-level Fn decl".into(),
                });
            }
            let Some(f) = input.lookup_fn(&m.decl.name) else {
                return Err(ProbeError::RefVersionMismatch {
                    name: m.decl.name.clone(),
                    cached: m.decl.content.as_str().to_string(),
                    current: "<fn no longer declared>".into(),
                });
            };
            let current = ac_sema_contract::ContentVersion::of_canonical(
                &ac_sema_core::query::fn_sig_canonical(f),
            );
            if current != m.decl.content {
                return Err(ProbeError::RefVersionMismatch {
                    name: m.decl.name.clone(),
                    cached: m.decl.content.as_str().to_string(),
                    current: current.as_str().to_string(),
                });
            }
            admit_header_content(page, input, query)
        }
        PageBody::ModuleIndex(ix) => {
            // F-796-R13-01: every published field is admitted against
            // the CURRENT input — package/module, fn candidates, record
            // candidates and each decl's signature version — not just
            // the header hash.
            let pkg = ac_sema_contract::PackageId::new(&input.package);
            if ix.package != pkg || ix.module != input.module {
                return Err(ProbeError::TargetMismatch {
                    found: format!("{}::{}", ix.package.as_str(), ix.module),
                    expected: format!("{}::{}", input.package, input.module),
                });
            }
            if ix.fn_names != input.fn_candidates() {
                return Err(ProbeError::RefVersionMismatch {
                    name: "module-index.fn_names".into(),
                    cached: ix.fn_names.join("\u{1}"),
                    current: input.fn_candidates().join("\u{1}"),
                });
            }
            if ix.record_names != input.record_candidates() {
                return Err(ProbeError::RefVersionMismatch {
                    name: "module-index.record_names".into(),
                    cached: ix.record_names.join("\u{1}"),
                    current: input.record_candidates().join("\u{1}"),
                });
            }
            let want = ac_sema_core::query::module_decl_versions(input);
            if ix.decl_versions != want {
                let join = |v: &[(
                    ac_sema_contract::LogicalId,
                    ac_sema_contract::ContentVersion,
                )]| {
                    v.iter()
                        .map(|(_, ver)| ver.as_str().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                };
                return Err(ProbeError::RefVersionMismatch {
                    name: "module-index.decl_versions".into(),
                    cached: join(&ix.decl_versions),
                    current: join(&want),
                });
            }
            admit_header_content(page, input, query)
        }
        _ => admit_header_content(page, input, query),
    }
}

/// The generic page-level freshness check: the header content must
/// equal the query target's CURRENT content version.
fn admit_header_content(
    page: &SemaPage,
    input: &SemaInput,
    query: Option<&QueryKind>,
) -> Result<(), ProbeError> {
    let Some(query) = query else {
        return Ok(());
    };
    let current = ac_sema_core::query::target_content(input, query);
    if page.header.content != current {
        return Err(ProbeError::RefVersionMismatch {
            name: query.as_str().to_string(),
            cached: page.header.content.as_str().to_string(),
            current: current.as_str().to_string(),
        });
    }
    Ok(())
}

/// Lookup WITHOUT any compute path. Decodes, admits kind/target/facts
/// ownership and current reference versions, and classifies freshness
/// INCLUDING the provenance-only-drift case.
pub fn lookup_only(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
) -> Result<FetchOutcome, ProbeError> {
    lookup_core(store, input, query, None, true, None)
}

/// Metered lookup (F-796-R6-05): the payload/decode window is charged to
/// the SAME session meter as the solve and publish.
pub fn lookup_only_metered<'m>(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    meter: Option<&'m mut crate::budget::SessionMeter>,
) -> Result<FetchOutcome, ProbeError> {
    lookup_core(store, input, query, meter, true, None)
}

/// Direct lookup with a PRECOMPUTED effective environment (publication
/// loop: one SCC partition per module).
fn lookup_direct_env(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    env: &KeyEnv,
) -> Result<FetchOutcome, ProbeError> {
    lookup_core(store, input, query, None, false, Some(env))
}

fn lookup_core(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    mut meter: Option<&mut crate::budget::SessionMeter>,
    allow_group: bool,
    env: Option<&KeyEnv>,
) -> Result<FetchOutcome, ProbeError> {
    let owned;
    let env = match env {
        Some(e) => Some(e),
        None if matches!(query, QueryKind::RecordFields { .. }) => {
            owned = key_env(input);
            Some(&owned)
        }
        None => None,
    };
    let key = SemaQueryKey::build_with_env(query, input, env);
    // F-796-R7-02: reserve the read/decode window BEFORE the payload is
    // allocated by the store read
    let mut read_guard = None;
    if let Some(m) = meter.as_deref_mut() {
        read_guard = Some(
            m.scratch(READ_RESERVE)
                .map_err(ProbeError::BudgetRejected)?,
        );
    }
    let loaded = match store.get_artifact(SEMA_LAYER, &store_key(&key))? {
        Some(l) => l,
        None => {
            if !allow_group {
                return Ok(FetchOutcome::Miss);
            }
            // F-796-R5 (F07) + R7-01: a RecordFields miss may be a member
            // of a recursive GROUP page; the CURRENT unit and effective
            // version decide the carrier and the key
            let QueryKind::RecordFields { record } = query else {
                return Ok(FetchOutcome::Miss);
            };
            let env = env.expect("record env");
            let Some(unit) = env.members(record) else {
                return Ok(FetchOutcome::Miss);
            };
            if !env.is_group(input, record) {
                return Ok(FetchOutcome::Miss);
            }
            match store.get_artifact(SEMA_LAYER, &group_key(input, unit, env))? {
                Some(l) => l,
                None => return Ok(FetchOutcome::Miss),
            }
        }
    };
    let (page, artifact_id) = decode_loaded(loaded)?;
    admit_page(&page, input, Some(query), env)?;
    drop(read_guard);
    let expected = expected_kind(query);
    // F-796-R5 (F07): a RecordFields query may be served by the
    // member's recursive GROUP page — admitted below by membership
    let group_ok_for_fields =
        page.header.kind == PageKind::RecGroup && matches!(query, QueryKind::RecordFields { .. });
    if page.header.kind != expected && !group_ok_for_fields {
        return Err(ProbeError::KindMismatch {
            expected: format!("{expected:?}"),
            found: format!("{:?}", page.header.kind),
        });
    }
    if group_ok_for_fields {
        let QueryKind::RecordFields { record } = query else {
            unreachable!()
        };
        if let PageBody::RecGroup(members) = &page.body {
            let member_target = ac_sema_contract::logical_id(
                &ac_sema_contract::PackageId::new(&input.package),
                &input.module,
                ac_sema_contract::DeclKind::Record,
                record,
            );
            if members.iter().any(|m| m.decl.logical == member_target) {
                return Ok(FetchOutcome::Hit { page, artifact_id });
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
            ac_sema_core::Freshness::Fresh => Ok(FetchOutcome::Hit { page, artifact_id }),
            ac_sema_core::Freshness::ProvenanceStale { reason } => {
                Ok(FetchOutcome::HitStaleProvenance {
                    page,
                    artifact_id,
                    reason,
                })
            }
            ac_sema_core::Freshness::Stale { reason } => Ok(FetchOutcome::Stale { reason }),
        }
    } else {
        Ok(FetchOutcome::Hit { page, artifact_id })
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

/// Deterministic solver staging estimate from the ACTUAL target body
/// (F-796-R6-05): charged BEFORE the solve, so an over-budget ledger
/// rejects before work — never a fixed post-hoc reservation.
fn solver_estimate(input: &SemaInput, fn_name: &str) -> usize {
    use ac_sema_core::input::{InputExpr, InputExprKind, InputStmt};
    // Conservative per-node costs calibrated against the R7 allocator
    // evidence (~660 live bytes per `let v = N`, i.e. ~330/node): one
    // statement 512 + one expression 256 + names, plus a 16 KiB base.
    // The estimate is a POLICY RESERVATION, not a measurement: it must
    // never undercount the real concurrent allocation.
    const SOLVER_BASE: usize = 16 * 1024;
    const STMT_COST: usize = 512;
    const EXPR_COST: usize = 256;
    let mut bytes = SOLVER_BASE;
    let mut exprs: Vec<&InputExpr> = Vec::new();
    let mut blocks: Vec<&ac_sema_core::input::InputBlock> = Vec::new();
    if let Some(f) = input.lookup_fn(fn_name) {
        for (n, _) in &f.params {
            bytes = bytes.saturating_add(96 + n.len().saturating_mul(4));
        }
        blocks.push(&f.body);
    }
    while let Some(b) = blocks.pop() {
        for st in &b.stmts {
            bytes = bytes.saturating_add(STMT_COST);
            match st {
                InputStmt::Let { name, value, .. } => {
                    bytes = bytes.saturating_add(name.len().saturating_mul(4));
                    exprs.push(value);
                }
                InputStmt::Assign { target, value, .. } => {
                    bytes = bytes.saturating_add(target.len().saturating_mul(4));
                    exprs.push(value);
                }
                InputStmt::Return { value, .. } => exprs.push(value),
                InputStmt::If {
                    cond, then, els, ..
                } => {
                    exprs.push(cond);
                    blocks.push(then);
                    if let Some(e) = els {
                        blocks.push(e);
                    }
                }
                InputStmt::While { cond, body, .. } => {
                    exprs.push(cond);
                    blocks.push(body);
                }
                InputStmt::Loop { body, .. } => blocks.push(body),
                InputStmt::Break { .. } | InputStmt::Continue { .. } => {}
            }
        }
    }
    while let Some(e) = exprs.pop() {
        bytes = bytes.saturating_add(EXPR_COST);
        match &e.kind {
            InputExprKind::Call { callee, args } => {
                bytes = bytes.saturating_add(callee.len().saturating_mul(4));
                exprs.extend(args.iter());
            }
            InputExprKind::Var(n) => {
                bytes = bytes.saturating_add(n.len().saturating_mul(4));
            }
            InputExprKind::Binary { lhs, rhs, .. } => {
                exprs.push(lhs);
                exprs.push(rhs);
            }
            InputExprKind::Unary { operand, .. } => exprs.push(operand),
            _ => {}
        }
    }
    bytes
}

/// Conservative size of the facts/manifest the publish path clones and
/// encodes (F-796-R7-02): charged BEFORE the clone.
fn facts_estimate(s: &ac_sema_core::InferSummary) -> usize {
    let mut b = 4 * 1024usize;
    for (n, _) in &s.facts.bindings {
        b = b.saturating_add(96 + n.len().saturating_mul(8));
    }
    b = b.saturating_add(s.facts.expr_types.len().saturating_mul(48));
    for d in &s.facts.diagnostics {
        b = b.saturating_add(
            128 + d.code.len().saturating_mul(4) + d.message.len().saturating_mul(2),
        );
    }
    for sig in &s.reads.sigs {
        b = b.saturating_add(96 + sig.len().saturating_mul(4));
    }
    for (_, n, set) in &s.reads.negatives {
        b = b.saturating_add(128 + n.len().saturating_mul(4) + set.len());
    }
    b
}

/// Solve under a scratch reservation derived from the actual body. The
/// guard is held for the WHOLE solve window, so the meter's peak
/// includes the real solver staging phase.
fn solve_charged(
    input: &SemaInput,
    fn_name: &str,
    meter: Option<&mut crate::budget::SessionMeter>,
) -> Result<InferOutcome, crate::budget::BudgetRejected> {
    let mut guard = None;
    if let Some(m) = meter {
        guard = Some(m.scratch(solver_estimate(input, fn_name))?);
    }
    let outcome = run_inference(input, fn_name);
    drop(guard);
    Ok(outcome)
}

fn resource_report(
    stages: Vec<(&'static str, u128)>,
    io_bytes: u64,
    compute_calls: u32,
    reason: String,
) -> QueryReport {
    QueryReport {
        disposition: Disposition::ResourceExhausted,
        compute_calls,
        bindings: Vec::new(),
        diagnostics: Vec::new(),
        stages,
        io_bytes,
        reason: Some(reason),
    }
}

/// The full staged query flow with typed dispositions and per-stage
/// timing — the ONE implementation behind the library callers, the
/// bin, the worker and the runner. The SAME session meter covers the
/// cache read/decode window, the solve, and the publish encode
/// (F-796-R6-05); a budget rejection is a typed resource state.
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
    let fetch = match lookup_only_metered(store, input, query, meter.as_deref_mut()) {
        Ok(f) => f,
        Err(ProbeError::BudgetRejected(e)) => {
            stages.push(("cache-read-validate", t.elapsed().as_nanos()));
            return Ok(resource_report(stages, io_bytes, 0, format!("{e}")));
        }
        Err(e) => return Err(e),
    };
    stages.push(("cache-read-validate", t.elapsed().as_nanos()));

    match fetch {
        FetchOutcome::Hit { page, artifact_id } => {
            // the reused page is REAL DATA for the current project: root
            // it (F-796-R6-02) — a hit must not be collected later
            root_reuse(store, project, &artifact_id)?;
            let encoded = encode_page(&page).map(|b| b.len() as u64).unwrap_or(0);
            io_bytes += encoded;
            let mut guard = None;
            if let Some(m) = meter.as_deref_mut() {
                match m.scratch(encoded as usize) {
                    Ok(g) => guard = Some(g),
                    Err(e) => return Ok(resource_report(stages, io_bytes, 0, format!("{e}"))),
                }
            }
            let (bindings, diagnostics) = match &page.body {
                PageBody::CachedResult(CachedResult {
                    facts: ResultFacts::Infer(f),
                    ..
                }) => (binding_strings(f), diag_strings(f)),
                _ => (Vec::new(), Vec::new()),
            };
            drop(guard);
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
        FetchOutcome::HitStaleProvenance {
            page,
            artifact_id,
            reason,
        } => {
            // F-796-R4-05: reuse semantic CONTENT, refresh the
            // projection. The refresh re-runs the deterministic solver
            // over the CURRENT source, verifies content equality with
            // the cache, and republishes current positions. A
            // divergence is a controlled error — never served, never
            // overwritten.
            root_reuse(store, project, &artifact_id)?;
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
            let outcome = match solve_charged(input, &fn_name, meter.as_deref_mut()) {
                Ok(o) => o,
                Err(e) => {
                    stages.push(("infer", t.elapsed().as_nanos()));
                    return Ok(resource_report(stages, io_bytes, 1, format!("{e}")));
                }
            };
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
                    let published = publish_infer_result_charged(
                        store,
                        input,
                        query,
                        &summary,
                        project,
                        meter.as_deref_mut(),
                    );
                    let art = match published {
                        Ok(a) => a,
                        Err(ProbeError::BudgetRejected(e)) => {
                            stages.push(("publish", t.elapsed().as_nanos()));
                            return Ok(resource_report(stages, io_bytes, 1, format!("{e}")));
                        }
                        Err(e) => return Err(e),
                    };
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
                InferOutcome::ResourceExhausted(e) => {
                    Ok(resource_report(stages, io_bytes, 1, format!("{e}")))
                }
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
            let outcome = match solve_charged(input, &fn_name, meter.as_deref_mut()) {
                Ok(o) => o,
                Err(e) => {
                    stages.push(("infer", t.elapsed().as_nanos()));
                    return Ok(resource_report(stages, io_bytes, 1, format!("{e}")));
                }
            };
            let infer_ns = t.elapsed().as_nanos();
            match outcome {
                InferOutcome::Ok(summary) => {
                    stages.push(("infer", infer_ns));
                    let t = Instant::now();
                    let published = publish_infer_result_charged(
                        store,
                        input,
                        query,
                        &summary,
                        project,
                        meter.as_deref_mut(),
                    );
                    let art = match published {
                        Ok(a) => a,
                        Err(ProbeError::BudgetRejected(e)) => {
                            stages.push(("publish", t.elapsed().as_nanos()));
                            return Ok(resource_report(stages, io_bytes, 1, format!("{e}")));
                        }
                        Err(e) => return Err(e),
                    };
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
                InferOutcome::ResourceExhausted(e) => {
                    Ok(resource_report(stages, io_bytes, 1, format!("{e}")))
                }
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
    publish_infer_result_charged(store, input, query, summary, project, None)
}

/// Conservative estimate of the ENCODED page body (F-796-R8-01): the
/// payload cap must be enforced as a typed resource BEFORE the facts
/// are cloned/encoded, not as a late codec error after the allocation.
fn estimated_page_bytes(
    s: &ac_sema_core::InferSummary,
    manifest: &ac_sema_contract::DepManifest,
) -> usize {
    let mut b = 4 * 1024usize;
    for (n, _) in &s.facts.bindings {
        b = b.saturating_add(n.len() + 8);
    }
    b = b.saturating_add(s.facts.expr_types.len().saturating_mul(12));
    for d in &s.facts.diagnostics {
        b = b.saturating_add(d.code.len() + d.message.len() + 16);
    }
    // the ACTUAL manifest is encoded into the page: config/source-map/
    // fn-body(origin = TARGET NAME)/fn-sig positives and negative names
    // are all part of the budget (F-796-R9-02 — a long target name is
    // encoded as the fn-body origin and was previously invisible here)
    for p in &manifest.positives {
        b = b.saturating_add(p.kind.len() + p.origin.len() + p.fp.len() + 24);
    }
    for n in &manifest.negatives {
        b = b.saturating_add(n.domain.len() + n.name.len() + n.set_fp.len() + 24);
    }
    b
}

/// Charged publish (F-796-R6-05 + R7-02): ONE conservative reservation
/// covers the facts clone, the manifest and the encoded page BEFORE the
/// clone happens; an over-budget publish is a typed resource rejection,
/// never a late codec error.
pub fn publish_infer_result_charged(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    summary: &ac_sema_core::InferSummary,
    project: Option<&str>,
    mut meter: Option<&mut crate::budget::SessionMeter>,
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
    // the pre-encode page ledger is checked FIRST: an unencodable page
    // must never drive the clone/encode allocations (typed resource,
    // recorded on the session meter). The estimate is derived from the
    // ACTUAL manifest that will be encoded.
    let page_want = estimated_page_bytes(summary, &manifest);
    if page_want > ac_sema_contract::MAX_PAGE_BODY {
        let err = match meter.as_deref_mut() {
            Some(m) => m.record_rejection("page", page_want, ac_sema_contract::MAX_PAGE_BODY),
            None => crate::budget::BudgetRejected {
                ledger: "page",
                wanted: page_want,
                budget: ac_sema_contract::MAX_PAGE_BODY,
            },
        };
        return Err(ProbeError::BudgetRejected(err));
    }
    let reserve = facts_estimate(summary)
        .saturating_add(page_want)
        .saturating_add(ac_sema_contract::MAX_PAGE_BODY);
    let mut guard = None;
    if let Some(m) = meter {
        guard = Some(m.scratch(reserve).map_err(ProbeError::BudgetRejected)?);
    }
    let cached = CachedResult {
        query: query.as_str(),
        target,
        facts: ResultFacts::Infer(summary.facts.clone()),
        manifest,
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V2.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::CachedResult,
            content: ac_sema_core::query::target_content(input, query),
        },
        body: PageBody::CachedResult(cached),
    };
    let art = publish_page(store, input, query, &page, project)?;
    drop(guard);
    Ok(art)
}

/// Publish a module index page (declaration candidate sets).
pub fn publish_module_index(
    store: &ManagedStore,
    input: &SemaInput,
    project: Option<&str>,
) -> Result<String, ProbeError> {
    let query = QueryKind::ListCandidates { domain: "fn" };
    // F-796-R13-01: header content and decl versions derive from the
    // SAME full-payload bytes the key and the read-side target content
    // use — a signature edit or a record addition republishes under a
    // NEW key (never a same-key Conflict), and a body-only edit is an
    // idempotent republication of the same artifact.
    let index = ModuleIndex {
        package: ac_sema_contract::PackageId::new(&input.package),
        module: input.module.clone(),
        fn_names: input.fn_candidates(),
        record_names: input.record_candidates(),
        decl_versions: ac_sema_core::query::module_decl_versions(input),
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V2.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::ModuleIndex,
            content: ac_sema_contract::ContentVersion::of_canonical(
                &ac_sema_core::query::module_index_bytes(input),
            ),
        },
        body: PageBody::ModuleIndex(index),
    };
    publish_page(store, input, &query, &page, project)
}

/// PURE record shape (no store access): ordered fields with REAL nominal
/// refs carrying the CURRENT referenced-definition content version
/// (F-796-R4-07 + F-796-R6-01). Unknown record names stay
/// descriptive-opaque. Because the refs are current by construction, the
/// shape content version changes exactly when any read dependency does.
fn shape_of(input: &SemaInput, name: &str) -> Result<ac_sema_contract::RecordShape, ProbeError> {
    let Some(r) = input.lookup_record(name) else {
        return Err(ProbeError::UnknownTarget {
            what: "record",
            name: name.to_string(),
        });
    };
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    let mut fields: Vec<ac_sema_contract::FieldDesc> = Vec::new();
    for (n, a) in &r.fields {
        let ty = match a {
            ac_sema_core::Ann::I32 => TypeDesc::I32,
            ac_sema_core::Ann::Bool => TypeDesc::Bool,
            ac_sema_core::Ann::Record(rec) => match input.lookup_record(rec) {
                Some(dep) => TypeDesc::Record {
                    decl: ac_sema_contract::DeclRef::new(
                        &pkg,
                        &input.module,
                        ac_sema_contract::DeclKind::Record,
                        rec,
                        ac_sema_contract::ContentVersion::of_canonical(
                            &ac_sema_core::query::record_canonical(dep),
                        ),
                    ),
                },
                None => TypeDesc::Opaque {
                    name: format!("record:{rec}"),
                },
            },
            ac_sema_core::Ann::Inferred => TypeDesc::Opaque {
                name: "?inferred".into(),
            },
            ac_sema_core::Ann::Unsupported(w) => TypeDesc::Opaque {
                name: format!("unsupported:{w}"),
            },
        };
        fields.push(ac_sema_contract::FieldDesc {
            name: n.clone(),
            ty,
        });
    }
    Ok(ac_sema_contract::RecordShape {
        decl: ac_sema_contract::DeclRef::new(
            &pkg,
            &input.module,
            ac_sema_contract::DeclKind::Record,
            name,
            ac_sema_contract::ContentVersion::of_canonical(&ac_sema_core::query::record_canonical(
                r,
            )),
        ),
        fields,
    })
}

/// Resolve the required-data artifact of every EXTERNAL referenced
/// record (outside `unit`) from the already-published dependency pages.
fn required_edges_of(
    store: &ManagedStore,
    input: &SemaInput,
    name: &str,
    unit: &std::collections::BTreeSet<String>,
    published: &BTreeMap<String, String>,
    env: &KeyEnv,
) -> Result<Vec<String>, ProbeError> {
    let mut out = Vec::new();
    for dep in ac_sema_core::query::record_dep_names(input, name) {
        if unit.contains(&dep) {
            continue;
        }
        // a dependency INSIDE an already-published recursive group has
        // no singleton page of its own — the publication map is the
        // authoritative artifact name (F-796-R6 group-of-DAG case)
        if let Some(art) = published.get(&dep) {
            out.push(art.clone());
            continue;
        }
        let dep_query = QueryKind::RecordFields {
            record: dep.clone(),
        };
        match lookup_direct_env(store, input, &dep_query, env)? {
            FetchOutcome::Hit { artifact_id, .. } => out.push(artifact_id),
            _ => {
                return Err(ProbeError::UnknownTarget {
                    what: "published record dependency",
                    name: dep,
                })
            }
        }
    }
    Ok(out)
}

/// SCC partition (iterative Kosaraju) of the record reference graph.
/// Returns units in DEPENDENCY ORDER (referenced groups first); a unit
/// is a singleton without self-reference or a recursive group.
/// Build the effective-dependency environment and count the partition
/// (the permanent deep-chain control: publication must partition ONCE).
fn key_env(input: &SemaInput) -> KeyEnv {
    SCC_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    KeyEnv::new(input)
}

/// Root a REUSED artifact under the current project (F-796-R6-02):
/// a hit is a real read of the current project — the page must be kept
/// alive exactly as if this project had published it. `put_artifact`
/// only roots NEW artifacts, so idempotent republication and pure reuse
/// need this explicit registration or GC reclaims a live result.
fn root_reuse(
    store: &ManagedStore,
    project: Option<&str>,
    artifact_id: &str,
) -> Result<(), ProbeError> {
    if let Some(p) = project {
        store.register_root(p, artifact_id, "soft")?;
    }
    Ok(())
}

/// Publish (or reuse) one record's page whose referenced pages ALL
/// exist already. Non-recursive: the caller resolves the DAG order.
fn ensure_record_page(
    store: &ManagedStore,
    input: &SemaInput,
    name: &str,
    unit: &std::collections::BTreeSet<String>,
    published: &BTreeMap<String, String>,
    project: Option<&str>,
    env: &KeyEnv,
) -> Result<String, ProbeError> {
    let query = QueryKind::RecordFields {
        record: name.to_string(),
    };
    if let FetchOutcome::Hit { artifact_id, .. } = lookup_direct_env(store, input, &query, env)? {
        root_reuse(store, project, &artifact_id)?;
        return Ok(artifact_id);
    }
    let Some(r) = input.lookup_record(name) else {
        return Err(ProbeError::UnknownTarget {
            what: "record",
            name: name.to_string(),
        });
    };
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    let shape = shape_of(input, name)?;
    let required_edges = required_edges_of(store, input, name, unit, published, env)?;
    let effective = env
        .effective(name)
        .ok_or_else(|| ProbeError::UnknownTarget {
            what: "record unit",
            name: name.to_string(),
        })?
        .to_string();
    let meta = RecordMeta {
        shape,
        methods: r
            .methods
            .iter()
            .map(|m| {
                let content = ac_sema_contract::ContentVersion::of_canonical(
                    &ac_sema_core::query::fn_sig_canonical(m),
                );
                // F-796-R11-01: a method's nominal identity includes its
                // OWNING record — A.get/B.get/top-level get are distinct
                ac_sema_contract::FnMeta {
                    decl: ac_sema_contract::DeclRef::new_method(
                        &pkg,
                        &input.module,
                        name,
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
        group_content: ac_sema_contract::ContentVersion(effective),
    };
    let page = SemaPage {
        header: PageHeader {
            schema: SEMA_PAGE_SCHEMA_V2.into(),
            producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
            codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
            kind: PageKind::RecordMeta,
            content: meta.group_content.clone(),
        },
        body: PageBody::RecordMeta(meta),
    };
    let key = SemaQueryKey::build_with_env(&query, input, Some(env));
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
    let art = store.put_artifact(req)?;
    root_reuse(store, project, &art)?;
    Ok(art)
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
    publish_page_charged(store, input, query, page, project, None)
}

fn publish_page_charged(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    page: &SemaPage,
    project: Option<&str>,
    meter: Option<&mut crate::budget::SessionMeter>,
) -> Result<String, ProbeError> {
    let payload = encode_page(page)?;
    let mut guard = None;
    if let Some(m) = meter {
        guard = Some(
            m.scratch(payload.len())
                .map_err(ProbeError::BudgetRejected)?,
        );
    }
    let key = SemaQueryKey::build(query, input);
    let req = PutRequest {
        layer: SEMA_LAYER,
        key: store_key(&key),
        kind: SEMA_CARRIER,
        payload,
        edges: Vec::new(),
        project: project.map(|s| s.to_string()),
    };
    let art = store.put_artifact(req)?;
    drop(guard);
    // idempotent republication returns the existing artifact WITHOUT a
    // new root row — register it so the current project really holds it
    root_reuse(store, project, &art)?;
    Ok(art)
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
/// (Kosaraju SCC units in dependency order, computed ONCE — the old
/// per-record miss rebuilt the full graph and made the 4096-chain V-06
/// pressure case take ~400s in debug). A recursive group publishes as
/// ONE page under its content-versioned group key; every reuse is
/// rooted under the CURRENT project (F-796-R6-02).
pub fn publish_module_records(
    store: &ManagedStore,
    input: &SemaInput,
    project: Option<&str>,
) -> Result<BTreeMap<String, String>, ProbeError> {
    let mut out = BTreeMap::new();
    // ONE effective-dependency partition per module (R7-01): the key,
    // page content and carrier topology all derive from this env
    let env = key_env(input);
    for unit in env.units() {
        let unit_set: std::collections::BTreeSet<String> = unit.iter().cloned().collect();
        let is_group = env.is_group(input, &unit[0]);
        if !is_group {
            // singleton without self-reference: single page (reused if
            // a page for exactly this record already exists)
            let name = &unit[0];
            if let Ok(FetchOutcome::Hit { artifact_id, .. }) = lookup_direct_env(
                store,
                input,
                &QueryKind::RecordFields {
                    record: name.clone(),
                },
                &env,
            ) {
                root_reuse(store, project, &artifact_id)?;
                out.insert(name.clone(), artifact_id);
                continue;
            }
            let art = ensure_record_page(store, input, name, &unit_set, &out, project, &env)?;
            out.insert(name.clone(), art);
            continue;
        }
        // recursive group: ONE page for the whole SCC (internal refs
        // are plain DeclRefs inside the page; external refs must be
        // published already and become required-data edges)
        let mut required_edges: Vec<String> = Vec::new();
        let mut members: Vec<ac_sema_contract::RecordShape> = Vec::new();
        for name in unit {
            members.push(shape_of(input, name)?);
            required_edges.extend(required_edges_of(
                store, input, name, &unit_set, &out, &env,
            )?);
        }
        required_edges.sort();
        required_edges.dedup();
        let key = group_key(input, unit, &env);
        // an unchanged group under the SAME effective-versioned key is
        // reused (and rooted), never rewritten; an edited member or a
        // topology change is a DIFFERENT key
        if let Some(loaded) = store.get_artifact(SEMA_LAYER, &key)? {
            let page = decode_page(&loaded.payload)?;
            admit_page(&page, input, None, Some(&env))?;
            root_reuse(store, project, &loaded.artifact_id)?;
            for name in unit {
                out.insert(name.clone(), loaded.artifact_id.clone());
            }
            continue;
        }
        let page = SemaPage {
            header: PageHeader {
                schema: SEMA_PAGE_SCHEMA_V2.into(),
                producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
                codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
                kind: PageKind::RecGroup,
                content: ac_sema_contract::ContentVersion(env.group_eff(unit)),
            },
            body: PageBody::RecGroup(members),
        };
        let payload = encode_page(&page)?;
        let edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = required_edges
            .into_iter()
            .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData))
            .collect();
        let req = PutRequest {
            layer: SEMA_LAYER,
            key,
            kind: SEMA_CARRIER,
            payload,
            edges,
            project: project.map(|s| s.to_string()),
        };
        let art = store.put_artifact(req)?;
        root_reuse(store, project, &art)?;
        for name in unit {
            out.insert(name.clone(), art.clone());
        }
    }
    Ok(out)
}
