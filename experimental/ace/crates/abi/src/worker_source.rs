//! Explicit source-import CERTIFICATION recipe on the existing bounded ABI
//! worker transport. Observed data is returned; only the parent owns oracles.
use crate::source_request_work as host_work;
use crate::{
    source_foreign::{CurrentAuthority, Edit, PrepareError, Session, Stored, Target},
    source_provider as provider,
    worker::{DiagOut, MeterOut, OkOut, StageSpan, StepOut, WorkerRequest},
};
use ace_artifacts::store::ManagedStore;
use ace_foreign_contract::manual::ManualManifest;
use ace_source_query::frontend::Frontend;
use ace_source_query::resources::{Charge, OutputLimits, Reservation, Resources};
use ace_source_query::work::{Ledger, Phase};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc, time::Instant};
pub const RECIPE: &str = "abi/source-foreign-certify-v1";
pub const DECK_SCHEMA: &str = "abi.source-import-deck/1";
pub const REPORT_SCHEMA: &str = "abi.source-import-report/2";
/// PLAN-939: the RESOURCE certification identities — a scalar-import
/// deck/report never settles them and vice versa (schema strings are the
/// version; the transport envelope is shared).
pub const RESOURCE_RECIPE: &str = "abi/resource-foreign-certify-v1";
pub const RESOURCE_DECK_SCHEMA: &str = "abi.resource-import-deck/1";
pub const RESOURCE_REPORT_SCHEMA: &str = "abi.resource-import-report/1";

/// The certification policy a request selects by its recipe string. The
/// two flows share the bounded transport and the session machinery; the
/// policy owns EVERY identity divergence (review registry, provider
/// bytes, reference oracle, profile, pointer namespace).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CertPolicy {
    Scalar938,
    Resource939,
}
impl CertPolicy {
    pub fn of_recipe(recipe: &str) -> Option<Self> {
        match recipe {
            RECIPE => Some(Self::Scalar938),
            RESOURCE_RECIPE => Some(Self::Resource939),
            _ => None,
        }
    }
    pub fn recipe(self) -> &'static str {
        match self {
            Self::Scalar938 => RECIPE,
            Self::Resource939 => RESOURCE_RECIPE,
        }
    }
    pub fn deck_schema(self) -> &'static str {
        match self {
            Self::Scalar938 => DECK_SCHEMA,
            Self::Resource939 => RESOURCE_DECK_SCHEMA,
        }
    }
    pub fn report_schema(self) -> &'static str {
        match self {
            Self::Scalar938 => REPORT_SCHEMA,
            Self::Resource939 => RESOURCE_REPORT_SCHEMA,
        }
    }
    pub fn input_profile(self) -> &'static str {
        match self {
            Self::Scalar938 => ace_core::source::SCALAR_SOURCE_PROFILE,
            Self::Resource939 => ace_core::source::TEXT_RECORD_SOURCE_PROFILE,
        }
    }
    pub fn module_id(self) -> &'static str {
        match self {
            Self::Scalar938 => "source938",
            Self::Resource939 => "source939",
        }
    }
    pub fn pointer_name(self, leg: &str) -> String {
        format!("{}-current-{}.json", self.module_id(), leg)
    }
    pub fn header_bytes(self) -> &'static [u8] {
        match self {
            Self::Scalar938 => provider::HEADER,
            Self::Resource939 => crate::resource_provider::HEADER,
        }
    }
    pub fn table_bytes(self) -> &'static [u8] {
        match self {
            Self::Scalar938 => provider::TABLE,
            Self::Resource939 => crate::resource_provider::TABLE,
        }
    }
    pub fn review_id(self) -> &'static str {
        match self {
            Self::Scalar938 => provider::REVIEW,
            Self::Resource939 => crate::resource_provider::REVIEW,
        }
    }
    /// The reviewed contract digest FOR ONE REFERENCE (938 has two
    /// reviewed contract shapes selected by reference; 939 has one).
    pub fn contract_digest(self, reference: Option<&str>) -> String {
        match self {
            Self::Scalar938 => provider::Review::for_reference(
                reference.unwrap_or("add"),
            )
            .map(|r| r.digest().to_string())
            .unwrap_or_else(|| provider::ADD_CONTRACT_DIGEST.into()),
            Self::Resource939 => crate::resource_provider::reviewed_resource_manifest()
                .contract_digest()
                .expect("reviewed resource manifest"),
        }
    }
    /// The reviewed reference-table digests this policy accepts.
    pub fn accepts_digest(self, digest: &str) -> bool {
        match self {
            Self::Scalar938 => provider::Review::for_digest(digest).is_some(),
            Self::Resource939 => digest == self.contract_digest(None),
        }
    }
    pub fn accepts_reference(self, reference: &str) -> bool {
        match self {
            Self::Scalar938 => provider::Review::for_reference(reference).is_some(),
            // PLAN-939: one reviewed reference per DECK — the independent
            // oracle mirrors that deck's exact provider call sequence.
            Self::Resource939 => matches!(
                reference,
                "utf8-owned"
                    | "utf8-bytes"
                    | "utf8-scalars"
                    | "record-mut"
                    | "record-mut-fail"
                    | "handle-lifecycle"
                    | "handle-stale"
                    | "txn-commit"
                    | "txn-seed-fail"
                    | "txn-laststep-fail"
                    | "callback"
                    | "callback-fail"
                    | "alias-pair-ok"
                    | "alias-pairv-ok"
                    | "alias-pair-reject"
                    | "alias-pairv-reject"
            ),
        }
    }
    pub fn qualify(
        self,
        manifest: &ManualManifest,
        caps: &[String],
        reference: &str,
    ) -> Result<(), String> {
        match self {
            Self::Scalar938 => provider::Review::for_reference(reference)
                .ok_or_else(|| "unreviewed reference".to_string())?
                .qualify(manifest, caps),
            Self::Resource939 => crate::resource_provider::qualify_resource(manifest, caps),
        }
    }
    /// The RESTORE qualification's authority: derived from the STORED
    /// pointer's ACCEPTED contract digest (the old binding), never the
    /// incoming deck's reference — a review transition restores the old
    /// binding first and only then transacts to the new contract.
    pub fn authority_for_digest(
        self,
        provider_digest: &str,
        generation: u64,
        caps: &[String],
        contract_digest: &str,
    ) -> Result<ace_foreign_contract::manual::HostAuthority, String> {
        match self {
            Self::Scalar938 => Ok(provider::Review::for_digest(contract_digest)
                .ok_or_else(|| "unreviewed accepted contract".to_string())?
                .authority(provider_digest, generation, caps)),
            Self::Resource939 => Ok(crate::resource_provider::resource_authority(
                provider_digest,
                generation,
                caps,
            )),
        }
    }
    pub fn authority(
        self,
        provider_digest: &str,
        generation: u64,
        caps: &[String],
        reference: &str,
    ) -> ace_foreign_contract::manual::HostAuthority {
        match self {
            Self::Scalar938 => provider::Review::for_reference(reference)
                .ok_or_else(|| "unreviewed reference".to_string())
                .expect("checked by accepts_reference")
                .authority(provider_digest, generation, caps),
            Self::Resource939 => crate::resource_provider::resource_authority(
                provider_digest,
                generation,
                caps,
            ),
        }
    }
    /// The provider C translation unit (with bias/events defines applied
    /// by the caller) and its include name.
    pub fn provider_source(self) -> &'static str {
        match self {
            Self::Scalar938 => include_str!("../cside/source_foreign938.c"),
            Self::Resource939 => include_str!("../cside/resource939.c"),
        }
    }
    pub fn provider_header_name(self) -> &'static str {
        match self {
            Self::Scalar938 => "source_foreign938.h",
            Self::Resource939 => "resource939.h",
        }
    }
    pub fn provider_events(self) -> &'static str {
        match self {
            Self::Scalar938 => provider::EVENT_RUNTIME,
            Self::Resource939 => crate::resource_provider::EVENT_RUNTIME,
        }
    }
    /// The certification runtime policy identity (the session's recorded
    /// runtime identity must equal this).
    pub fn certification_runtime_policy(self, target: Target) -> String {
        match (self, target) {
            // PLAN-939: the resource C leg's runtime page is the
            // record-text runtime (no trace marks in the fixtures). The
            // page digest inside the identity formula is SHA-256 like
            // every other term (the store's page ids are sha256; the
            // blake3 `provider::hash` digests the provider OBJECT, a
            // different identity space).
            (Self::Resource939, Target::C) => crate::source_foreign::runtime_identity(
                target,
                &ace_a2x_core::sha256::sha256_hex(
                    ace_a2x_c::rt_runtime::record_text_runtime_source(&[]).as_bytes(),
                ),
            ),
            _ => crate::source_foreign::certification_runtime_policy(target),
        }
    }
    pub fn provider_define(self) -> &'static str {
        match self {
            Self::Scalar938 => "SOURCE938_EVENTS",
            Self::Resource939 => "SOURCE939_EVENTS",
        }
    }
    /// The reference-oracle translation unit for one deck reference.
    pub fn reference_source(self, reference: &str) -> Result<String, String> {
        match self {
            Self::Scalar938 => {
                let case = provider::reference_case(reference)
                    .ok_or_else(|| "unreviewed reference".to_string())?;
                Ok(format!(
                    "#define SOURCE938_SCENARIO {}
{}",
                    case,
                    include_str!("../cside/source_foreign938_reference.c")
                ))
            }
            Self::Resource939 => {
                if !self.accepts_reference(reference) {
                    return Err("unreviewed reference".into());
                }
                let guard: String = reference
                    .bytes()
                    .map(|b| if b == b'-' { b'_' } else { b })
                    .map(|b| b as char)
                    .collect();
                Ok(format!(
                    "#define R939_S_{}
{}",
                    guard,
                    include_str!("../cside/resource939_reference.c")
                ))
            }
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDiagnostic {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span: Option<(usize, usize)>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deck {
    pub schema: String,
    pub source: String,
    pub module: String,
    pub manifest: ManualManifest,
    pub capabilities: Vec<String>,
    pub provider_bias: u8,
    pub reference: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReport {
    pub source_work: Ledger,
    pub host_work: host_work::RequestWork,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub schema: String,
    pub direction: String,
    pub input_profile: String,
    pub abi_domain: String,
    pub abi_version: String,
    pub abi_profile: String,
    pub callconv: String,
    pub target: String,
    pub consumer: String,
    pub source_sha256: String,
    pub compiler: String,
    pub c_toolchain: String,
    pub contract_digest: String,
    pub provider_digest: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub review_id: String,
    pub actual_exit: Option<i32>,
    pub reference_exit: Option<i32>,
    pub actual_events: Vec<String>,
    pub reference_events: Vec<String>,
    pub actual_stderr: String,
    pub reference_stderr: String,
    pub source_parse: u64,
    pub solver: u64,
    pub hir: u64,
    pub mir: u64,
    pub native: u64,
    pub c_fragment: u64,
    pub diagnostic_stage: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    module: String,
    #[serde(default = "provider::legacy_add_contract")]
    contract_digest: String,
    provider_bias: u8,
    binding_generation: u64,
    stored: Stored,
}
#[derive(Serialize)]
struct PublicationRef<'a> {
    module: &'a str,
    #[serde(skip_serializing_if = "is_add_contract")]
    contract_digest: &'a str,
    provider_bias: u8,
    binding_generation: u64,
    stored: &'a Stored,
}
fn is_add_contract(digest: &&str) -> bool {
    *digest == provider::ADD_CONTRACT_DIGEST
}
fn err(code: &str, message: impl ToString) -> DiagOut {
    DiagOut {
        code: code.into(),
        message: message.to_string(),
    }
}
fn budget_error(code: &str, resources: &Resources, message: impl ToString) -> DiagOut {
    err(
        if resources.output_stats().is_some_and(|s| s.refused) {
            "foreign.resource"
        } else {
            code
        },
        message,
    )
}
fn step(
    outcome: &'static str,
    diagnostics: Vec<DiagOut>,
    compute: u32,
    store: &ManagedStore,
    started: Instant,
    exit: Option<i32>,
) -> Result<StepOut, DiagOut> {
    let gc = crate::source_store::StorePolicy::worker()
        .collect(store)
        .map_err(|e| {
            err(
                "store.cleanup-pending",
                format!("source operation completed; {e}"),
            )
        })?;
    if let Some(backoff) = &gc.backoff {
        // This is maintenance after the selected operation, which may have
        // already published. Never disguise it as a rejected source candidate
        // whose current root supposedly stayed unchanged.
        return Err(err(
            "store.cleanup-pending",
            format!("source operation completed; store backoff: {backoff}"),
        ));
    }
    Ok(StepOut {
        op: "query",
        outcome,
        cache: if outcome == "resource" {
            "-".into()
        } else if outcome == "diags" {
            "diags".into()
        } else if compute == 0 {
            "hit".into()
        } else {
            "published".into()
        },
        compute_calls: compute,
        diagnostics,
        stages: vec![StageSpan {
            name: "source-import".into(),
            wall_ns: started.elapsed().as_nanos(),
        }],
        meter: MeterOut {
            scratch_peak_disk_bytes: host_work::scratch_peak(),
            hot_bytes: store.space().total,
            rejections: if matches!(outcome, "diags" | "resource") {
                1
            } else {
                0
            },
            gc_runs: u32::from(gc.ran),
            gc_deleted_chunks: gc.deleted_chunks.try_into().unwrap_or(u32::MAX),
            gc_freed_bytes: gc.freed_bytes,
        },
        pe_exit: exit,
    })
}
fn events(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|s| {
            s.strip_prefix("source938.call=")
                .or_else(|| s.strip_prefix("source939.call="))
                .map(String::from)
        })
        .collect()
}
fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), String> {
    let bytes = bytes.as_ref();
    std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    host_work::io("tool-input", 0, bytes.len() as u64);
    Ok(())
}
fn prepare_reference(
    run_dir: &Path,
    reference: &str,
    provider_bytes: &[u8],
    crt: &crate::source_crt::Toolchain,
    policy: CertPolicy,
) -> Result<std::path::PathBuf, DiagOut> {
    let work = host_work::begin("reference-compile", Phase::Production);
    let reference_dir = run_dir.join("reference");
    std::fs::create_dir_all(&reference_dir).map_err(|e| err("infra", e))?;
    write(
        &reference_dir.join(policy.provider_header_name()),
        policy.header_bytes(),
    )
    .map_err(|e| err("infra", e))?;
    write(
        &reference_dir.join("reference.c"),
        policy
            .reference_source(reference)
            .map_err(|e| err("deck", e))?,
    )
    .map_err(|e| err("infra", e))?;
    write(&reference_dir.join("events.c"), policy.provider_events())
        .map_err(|e| err("infra", e))?;
    write(&reference_dir.join("provider.obj"), provider_bytes).map_err(|e| err("infra", e))?;
    let exe = reference_dir.join("reference.exe");
    crt.compile(
        &reference_dir,
        &["reference.c", "events.c"],
        &[reference_dir.join("provider.obj")],
        &exe,
    )
    .map_err(|e| {
        err(
            if e.0.starts_with("foreign.resource:") {
                "foreign.resource"
            } else {
                "infra"
            },
            e,
        )
    })?;
    host_work::complete(work);
    Ok(exe)
}
fn decode(
    req: &WorkerRequest,
    resources: &Resources,
    policy: CertPolicy,
) -> Result<(Deck, Reservation), DiagOut> {
    if req.deck_b64.len() > crate::worker::MAX_LINE {
        return Err(err("deck", "source deck transport budget"));
    }
    let decoded_bound = req
        .deck_b64
        .len()
        .saturating_div(4)
        .saturating_mul(3)
        .saturating_add(3);
    let _input = resources
        .reserve(Charge::io(decoded_bound))
        .map_err(|e| err("foreign.resource", e))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.deck_b64)
        .map_err(|e| err("deck", e))?;
    if bytes.len() > crate::worker::MAX_LINE {
        return Err(err("deck", "source deck budget"));
    }
    let (deck, model): (Deck, _) =
        resources
            .decode_foreign_request(&bytes)
            .map_err(|e| match e {
                ace_source_query::resources::ResourceError::Budget(_) => err("foreign.resource", e),
                ace_source_query::resources::ResourceError::Decode(_) => err("deck", e),
            })?;
    if deck.schema != policy.deck_schema()
        || deck.source.is_empty()
        || deck.module.is_empty()
        || deck.provider_bias > 1
        || !policy.accepts_reference(&deck.reference)
    {
        return Err(err(
            "deck",
            "unsupported source deck schema/module/provider/reference",
        ));
    }
    Ok((deck, model))
}
fn output_limits(resources: &Resources) -> Result<OutputLimits, DiagOut> {
    const NAME: &[u8] = b"ABI_TEST_PRODUCED_BUDGET_BYTES";
    let mut name = [0u16; NAME.len() + 1];
    for (wide, byte) in name.iter_mut().zip(NAME) {
        *wide = *byte as u16;
    }
    // Reuse the original host-only tightening seam, sizing its native value
    // before owning it. Request/deck data never selects these allowances.
    let value =
        crate::ctool_environment::read(&name, resources).map_err(|e| err("foreign.resource", e))?;
    let maximum = OutputLimits::default();
    let request = value
        .as_ref()
        .and_then(|v| v.value.to_str())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(maximum.request)
        .min(maximum.request);
    Ok(OutputLimits { request, ..maximum })
}
pub fn execute(req: &WorkerRequest) -> Result<OkOut, DiagOut> {
    let policy = match CertPolicy::of_recipe(&req.recipe) {
        Some(p) => p,
        None => {
            return Err(err("recipe","source certification requires its recipe, explicit native/c consumer and one operation"));
        }
    };
    if !matches!(req.leg.as_str(), "native" | "c") || req.steps.len() != 1 {
        return Err(err("recipe","source certification requires its recipe, explicit native/c consumer and one operation"));
    }
    if req.steps[0].from.is_some() || req.steps[0].to.is_some() {
        return Err(err("op", "query/restart cannot carry ignored edit fields"));
    }
    if req.steps[0].op == "restart" {
        return Ok(OkOut {
            recipe: policy.recipe(),
            steps: vec![],
            restart_pending: true,
            worker_version: crate::worker::WORKER_VERSION.into(),
            source_import: None,
        });
    }
    if req.steps[0].op != "query" {
        return Err(err(
            "op",
            "source certification operation must be query/restart",
        ));
    }
    let started = Instant::now();
    let store = ManagedStore::open(Path::new(&req.store_root)).map_err(|e| err("store", e))?;
    let target = if req.leg == "native" {
        Target::Native
    } else {
        Target::C
    };
    let resources = Resources::default();
    resources
        .enable_output_budget(output_limits(&resources)?)
        .map_err(|e| err("foreign.resource", e))?;
    let request_work =
        host_work::RequestWork::new(&resources).map_err(|e| err("foreign.resource", e))?;
    let _work_scope = request_work.enter();
    let mut report = SourceReport {
        source_work: Ledger::default(),
        host_work: request_work,
        diagnostics: vec![],
        schema: policy.report_schema().into(),
        direction: "source-import".into(),
        input_profile: policy.input_profile().into(),
        abi_domain: ace_foreign_contract::plan::DOMAIN.into(),
        abi_version: ace_foreign_contract::plan::DOMAIN_VERSION.into(),
        callconv: "windows-x64".into(),
        abi_profile: ace_foreign_contract::plan::PROFILE.into(),
        target: ace_foreign_contract::plan::TARGET.into(),
        consumer: req.leg.clone(),
        compiler: target.identity(),
        source_sha256: String::new(),
        c_toolchain: String::new(),
        contract_digest: policy.contract_digest(None),
        provider_digest: String::new(),
        header_digest: provider::hash(policy.header_bytes()),
        physical_table_digest: provider::hash(policy.table_bytes()),
        review_id: policy.review_id().into(),
        actual_exit: None,
        reference_exit: None,
        actual_events: vec![],
        reference_events: vec![],
        actual_stderr: String::new(),
        reference_stderr: String::new(),
        source_parse: 0,
        solver: 0,
        hir: 0,
        mir: 0,
        native: 0,
        c_fragment: 0,
        diagnostic_stage: String::new(),
    };
    // Reap dead tool/provider scopes even when this request ends with an early
    // source/declaration diagnostic. Active scopes are protected by OS locks.
    crate::source_tool_scratch::ToolScratch::sweep(&store, &resources).map_err(|e| {
        err(
            "store.cleanup-pending",
            format!("before source operation: {e}"),
        )
    })?;
    let (deck, _request_model) = match decode(req, &resources, policy) {
        Ok(d) => d,
        Err(d) => {
            report.diagnostic_stage = "parse".into();
            let outcome = if d.code == "foreign.resource" {
                "resource"
            } else {
                "diags"
            };
            let s = step(outcome, vec![d], 0, &store, started, None)?;
            return Ok(output(s, report, policy));
        }
    };
    if !policy.accepts_reference(&deck.reference) {
        return Err(err("deck", "unreviewed reference"));
    }
    report.contract_digest = policy.contract_digest(Some(&deck.reference));
    if let Err(e) = target.admit_source_bytes(deck.source.len()) {
        report.diagnostic_stage = "parse".into();
        return Ok(output(
            step(
                "resource",
                vec![err("foreign.resource", e)],
                0,
                &store,
                started,
                None,
            )?,
            report,
            policy,
        ));
    }
    report.source_sha256 = ace_a2x_core::sha256::sha256_hex(deck.source.as_bytes());
    // Reject unauthorized ABI/declaration candidates before compiling/linking
    // or calling any C provider. No candidate supplies this policy digest.
    if let Err(e) = policy.qualify(&deck.manifest, &deck.capabilities, &deck.reference) {
        report.diagnostic_stage = "resolve".into();
        let code = e.split(':').next().unwrap_or("foreign.authority");
        return Ok(output(
            step("diags", vec![err(code, &e)], 0, &store, started, None)?,
            report,
            policy,
        ));
    }
    let (bytes, hit) = prepare_provider(&store, &resources, deck.provider_bias, policy)?;
    let mut provider_productions = (!hit) as u64;
    report.c_toolchain = crate::ctool::toolchain_fingerprint().map_err(|e| err("infra", e))?;
    report.provider_digest = provider::hash(&bytes);
    let current = Path::new(&req.store_root).join(policy.pointer_name(&req.leg));
    let (previous_record, expected_pointer) =
        match crate::source_foreign::read_pointer_bytes_reserved(&current, &resources).map_err(
            |e| {
                let code = if e.starts_with("foreign.resource:") {
                    "foreign.resource"
                } else {
                    "store"
                };
                err(code, e)
            },
        )? {
            Some(bytes) => {
                let digest = ace_a2x_core::sha256::sha256_hex(bytes.bytes());
                let (previous, model) = resources
                    .decode_metadata_json::<Publication>(bytes.bytes())
                    .map_err(|e| match e {
                        ace_source_query::resources::ResourceError::Budget(_) => {
                            err("foreign.resource", e)
                        }
                        ace_source_query::resources::ResourceError::Decode(_) => err("restore", e),
                    })?;
                // Tuple field order releases the typed model before its
                // reservation, including every early-return/error path.
                (Some((previous, model)), Some(digest))
            }
            None => (None, None),
        };
    let previous = previous_record.as_ref().map(|(record, _ticket)| record);
    let changed_contract = previous.is_some_and(|p| p.contract_digest != policy.contract_digest(Some(&deck.reference)));
    let binding_generation = match &previous {
        Some(p) if p.provider_bias != deck.provider_bias || changed_contract => p
            .binding_generation
            .checked_add(1)
            .ok_or_else(|| err("foreign.binding", "generation overflow"))?,
        Some(p) => p.binding_generation,
        None => 1,
    };
    let (mut session, edit) = if let Some(previous) = &previous {
        if previous.module != deck.module {
            return Err(err(
                "restore",
                "changed module requires a separate source session",
            ));
        }
        // R admits the accepted old binding first. A different requested body
        // then uses the explicit provider transaction, never a cold fallback.
        let restore_digest = if previous.provider_bias != deck.provider_bias {
            let (old_bytes, old_hit) =
                prepare_provider(&store, &resources, previous.provider_bias, policy)?;
            provider_productions += (!old_hit) as u64;
            provider::hash(&old_bytes)
        } else {
            report.provider_digest.clone()
        };
        let restore_generation = previous.binding_generation;
        if !policy.accepts_digest(&previous.contract_digest) {
            return Err(err("restore", "unreviewed accepted contract"));
        }
        let mut restore_current = || {
            Ok(CurrentAuthority {
                foreign: policy
                    .authority_for_digest(
                        &restore_digest,
                        restore_generation,
                        &deck.capabilities,
                        &previous.contract_digest,
                    )
                    .map_err(|e| format!("foreign.restore.authority: {e}"))?,
                runtime_digest: policy.certification_runtime_policy(target),
                emitter: target.identity(),
            })
        };
        let mut restored = Session::restore_stored_current_with_resources(
            &store,
            &previous.stored,
            target,
            &deck.capabilities,
            &mut restore_current,
            resources.clone(),
        )
        .map_err(|e| err("restore", e))?;
        report.source_work = restored.frontend().work();
        let changed_source = restored.frontend().source() != deck.source;
        let changed_provider = previous.provider_bias != deck.provider_bias;
        let result = if changed_provider || changed_contract {
            let authority = policy.authority(
                &report.provider_digest,
                binding_generation,
                &deck.capabilities,
                &deck.reference,
            );
            let (environment, interface_work) = restored.frontend().work().observe_foreign(|| {
                if changed_contract {
                    resources.admit_foreign(deck.manifest, &authority)
                } else {
                    resources.rebind_foreign(restored.frontend().foreign(), &authority)
                }
            });
            let environment =
                environment.map_err(|e| budget_error("foreign.authority", &resources, e))?;
            restored
                .prepare_foreign_source_checked(
                    environment,
                    bytes.clone(),
                    changed_source.then_some(deck.source.as_str()),
                )
                .map(|candidate| Edit::Foreign(candidate.include_interface_work(interface_work)))
        } else if changed_source {
            restored
                .prepare_source_checked(&deck.source)
                .map(Edit::Body)
        } else {
            Ok(Edit::Current)
        };
        let edit = match result {
            Ok(candidate) => candidate,
            Err(PrepareError::Source(ace_source_query::frontend::Error::Diagnostics(ds))) => {
                let fm = restored.frontend().meter();
                return source_diagnostics(
                    report,
                    ds,
                    fm.whole_parse_attempts + fm.body_parse_attempts,
                    restored.frontend().query_counters().solver_runs,
                    provider_productions,
                    &store,
                    started,
                    policy,
                );
            }
            Err(e) => return Err(budget_error("source", &resources, e)),
        };
        (restored, edit)
    } else {
        let mut observation = ace_source_query::frontend::ColdObservation::default();
        report.source_work = observation.work.clone();
        let (environment, interface_work) = observation.work.observe_foreign(|| {
            resources.admit_foreign(
                deck.manifest,
                &policy.authority(
                    &report.provider_digest,
                    binding_generation,
                    &deck.capabilities,
                    &deck.reference,
                ),
            )
        });
        let environment =
            environment.map_err(|e| budget_error("foreign.authority", &resources, e))?;
        let (front, checked) = match Frontend::cold_observed_with_resources(
            &deck.source,
            &deck.module,
            policy.module_id(),
            &deck.capabilities,
            environment,
            &mut observation,
            resources.clone(),
        ) {
            Ok(c) => c,
            Err(ace_source_query::frontend::Error::Diagnostics(ds)) => {
                return source_diagnostics(
                    report,
                    ds,
                    observation.whole_parse_attempts,
                    observation.queries.solver_runs,
                    provider_productions,
                    &store,
                    started,
                    policy,
                );
            }
            Err(e) => return Err(budget_error("source", &resources, e)),
        };
        let session = Session::from_frontend_stored(front, checked, bytes.clone(), target, &store)
            .map_err(|e| budget_error("source", &resources, e))?;
        for work in interface_work {
            work.accept();
        }
        (session, Edit::Current)
    };
    let fm = session.frontend().meter();
    let tm = session.meter();
    report.source_parse = fm.whole_parse_attempts + fm.body_parse_attempts;
    report.solver = session.frontend().query_counters().solver_runs;
    report.hir = fm.hir_produced;
    report.mir = tm.mir_produced;
    report.native = tm.native_produced;
    report.c_fragment = tm.c_produced;
    let scratch = crate::source_tool_scratch::ToolScratch::create(&store, &resources)
        .map_err(|e| err("store.cleanup-pending", e))?;
    let mut publication_committed = false;
    // Keep compiler-context metadata charged until the request hands its
    // typed result to the bounded worker transport, including cleanup/GC.
    let mut crt_owner = None;
    let operation: Result<(), DiagOut> = (|| {
        let run_dir = scratch.path();
        let digest = report.provider_digest.clone();
        // PLAN-939: the runtime digest derives from the SESSION's recorded
        // identity (the resource C runtime depends on the view's trace
        // marks; the scalar lanes keep the frozen policy identity).
        let runtime_policy = session
            .runtime_identity()
            .to_string();
        let mut current_authority = || {
            Ok(CurrentAuthority {
                foreign: policy.authority(
                    &digest,
                    binding_generation,
                    &deck.capabilities,
                    &deck.reference,
                ),
                runtime_digest: runtime_policy.clone(),
                emitter: target.identity(),
            })
        };
        let crt_work = host_work::begin("crt-context", Phase::Metadata);
        let crt = crt_owner.insert(crate::source_crt::Toolchain::resolve(&resources).map_err(
            |e| {
                err(
                    if e.0.starts_with("foreign.resource:") {
                        "foreign.resource"
                    } else {
                        "infra"
                    },
                    e,
                )
            },
        )?);
        host_work::complete(crt_work);
        report.c_toolchain = crt.fingerprint();
        let prepared = session
            .prepare_publication_with_crt(edit, run_dir, &mut current_authority, None, Some(crt))
            .map_err(|e| {
                err(
                    if resources.output_stats().is_some_and(|s| s.refused)
                        || e.starts_with("bridge: foreign.resource:")
                    {
                        "foreign.resource"
                    } else {
                        "publication"
                    },
                    e,
                )
            })?;
        let exe = prepare_reference(run_dir, &deck.reference, bytes.as_slice(), crt, policy)?;
        let actual_work = host_work::begin("actual-run", Phase::Production);
        let actual = prepared
            .run(&mut current_authority, None)
            .map_err(|e| err("infra", e))?;
        host_work::io(
            "actual-run",
            (actual.stdout.len() + actual.stderr.len()) as u64,
            0,
        );
        host_work::complete(actual_work);
        let reference_work = host_work::begin("reference-run", Phase::Production);
        let reference = ace_native::link_driver::run_pe(
            &exe,
            crate::bridge::BRIDGE_RUN_DEADLINE,
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
        )
        .map_err(|e| err("infra", e))?;
        host_work::io(
            "reference-run",
            (reference.stdout.len() + reference.stderr.len()) as u64,
            0,
        );
        host_work::complete(reference_work);
        report.actual_exit = actual.exit_code;
        report.reference_exit = reference.exit_code;
        report.actual_events = events(&actual.stderr);
        report.reference_events = events(&reference.stderr);
        report.actual_stderr = String::from_utf8_lossy(&actual.stderr).into();
        report.reference_stderr = String::from_utf8_lossy(&reference.stderr).into();
        session
            .publish_prepared(
                prepared,
                &store,
                &current,
                expected_pointer.as_deref(),
                previous.as_ref().map(|p| &p.stored),
                |stored, encoder| {
                    encoder.json(&PublicationRef {
                        module: &deck.module,
                        contract_digest: &policy.contract_digest(Some(&deck.reference)),
                        provider_bias: deck.provider_bias,
                        binding_generation,
                        stored,
                    })
                },
                &mut current_authority,
                None,
            )
            .map_err(|e| budget_error("store", &resources, e))?;
        publication_committed = true;
        if let Some(error) = &session.publication_maintenance().cleanup_error {
            return Err(err("store.cleanup-pending", error));
        }
        Ok(())
    })();
    if let Err(cleanup) = scratch.finish() {
        let original = operation
            .as_ref()
            .err()
            .map(|d| format!("{}: {}", d.code, d.message))
            .unwrap_or_else(|| "none".into());
        return Err(err("store.cleanup-pending", format!(
            "tool cleanup; publication committed={publication_committed}; original={original}; {cleanup}"
        )));
    }
    operation?;
    // Linking/publication may produce target outputs after the initial source
    // edit. Read the same live meters after that work, rather than reporting
    // a pre-link snapshot beside the complete ledger.
    let fm = session.frontend().meter();
    let tm = session.meter();
    report.source_parse = fm.whole_parse_attempts + fm.body_parse_attempts;
    report.solver = session.frontend().query_counters().solver_runs;
    report.hir = fm.hir_produced;
    report.mir = tm.mir_produced;
    report.native = tm.native_produced;
    report.c_fragment = tm.c_produced;
    let compute = report.solver
        + report.hir
        + report.mir
        + report.native
        + report.c_fragment
        + provider_productions;
    Ok(output(
        step(
            "ok",
            vec![],
            compute as u32,
            &store,
            started,
            report.actual_exit,
        )?,
        report,
        policy,
    ))
}
fn output(step: StepOut, mut report: SourceReport, policy: CertPolicy) -> OkOut {
    if report.diagnostics.is_empty() {
        report.diagnostics = step
            .diagnostics
            .iter()
            .map(|d| SourceDiagnostic {
                stage: report.diagnostic_stage.clone(),
                code: d.code.clone(),
                message: d.message.clone(),
                span: None,
            })
            .collect();
    }
    OkOut {
        recipe: policy.recipe(),
        steps: vec![step],
        restart_pending: false,
        worker_version: crate::worker::WORKER_VERSION.into(),
        source_import: Some(report),
    }
}

fn prepare_provider(
    store: &ManagedStore,
    resources: &Resources,
    bias: u8,
    policy: CertPolicy,
) -> Result<(Arc<Vec<u8>>, bool), DiagOut> {
    let cache_work = host_work::begin("provider-cache", Phase::Metadata);
    let scratch = crate::source_tool_scratch::ToolScratch::provider(store, resources, bias)
        .map_err(|e| err("store.cleanup-pending", e))?;
    let operation = (|| {
        let inputs = scratch.path();
        let src = inputs.join("provider.c");
        let object = inputs.join("provider.obj");
        write(&inputs.join(policy.provider_header_name()), policy.header_bytes())
            .map_err(|e| err("store", e))?;
        write(
            &src,
            format!(
                "#define {}\n#define BODY_BIAS {}\n{}",
                policy.provider_define(),
                bias,
                policy.provider_source()
            ),
        )
        .map_err(|e| err("store", e))?;
        // This ticket lives outside the producer closure: the dependency
        // vector remains owned by the cache through hashing/generation.
        let mut dependencies_allocation = resources
            .reserve(ace_source_query::resources::Charge::index(0))
            .map_err(|e| err("foreign.resource", e))?;
        let result = crate::cache::produce_c_object_with_resources(
            store,
            &src,
            &inputs,
            &[object.clone()],
            resources,
            || {
                let compile_work = host_work::begin("provider-compile", Phase::Production);
                let deps = crate::ctool::compile_c_with_deps_admitted(
                    &src,
                    inputs,
                    &object,
                    Some(resources),
                    |bytes| {
                        dependencies_allocation
                            .resize(ace_source_query::resources::Charge::index(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
                .map_err(|e| match e {
                    crate::ctool::CToolFailure::Tool(error) => {
                        ace_artifacts::store::StoreError::Internal(format!("producer: {error}"))
                    }
                    crate::ctool::CToolFailure::Resource(message) => {
                        ace_artifacts::store::StoreError::ResourceExhausted(format!(
                            "C dependency producer: {message}"
                        ))
                    }
                })?;
                let bytes = crate::cache::read_c_object_file(&object, resources)?;
                host_work::complete(compile_work);
                Ok((bytes, deps))
            },
        )
        .map_err(|e| {
            let code = if matches!(&e, ace_artifacts::store::StoreError::ResourceExhausted(_)) {
                "foreign.resource"
            } else {
                "infra"
            };
            err(code, e)
        })?;
        Ok(result)
    })();
    if let Err(cleanup) = scratch.finish() {
        let original = operation
            .as_ref()
            .err()
            .map(|d: &DiagOut| d.message.as_str())
            .unwrap_or("none");
        return Err(err(
            "store.cleanup-pending",
            format!(
                "provider preparation; publication committed=false; original={original}; {cleanup}"
            ),
        ));
    }
    if let Ok((bytes, hit)) = &operation {
        host_work::io("provider-cache", bytes.len() as u64, 0);
        if *hit {
            host_work::reuse("provider-cache");
        }
        host_work::complete(cache_work);
    }
    operation
}

fn source_diagnostics(
    mut report: SourceReport,
    ds: Vec<ace_core::atom_text::Diagnostic>,
    source_parse: u64,
    solver: u64,
    provider_productions: u64,
    store: &ManagedStore,
    started: Instant,
    policy: CertPolicy,
) -> Result<OkOut, DiagOut> {
    report.source_parse = source_parse;
    report.solver = solver;
    report.diagnostic_stage = match ds.first().map(|d| d.stage) {
        Some(ace_core::atom_text::Stage::Text) => "parse",
        Some(ace_core::atom_text::Stage::Verify) => "verify",
        _ => "resolve",
    }
    .into();
    report.diagnostics = ds
        .iter()
        .map(|d| SourceDiagnostic {
            stage: report.diagnostic_stage.clone(),
            code: d.code.clone(),
            message: d.message.clone(),
            span: Some((d.span.start, d.span.end)),
        })
        .collect();
    Ok(output(
        step(
            "diags",
            ds.into_iter().map(|d| err(&d.code, d.message)).collect(),
            (provider_productions + solver) as u32,
            store,
            started,
            None,
        )?,
        report,
        policy,
    ))
}

#[cfg(test)]
mod request_budget_tests {
    use super::*;
    use ace_source_query::resources::Limits;

    #[test]
    fn host_output_allowance_only_tightens_and_never_comes_from_the_deck() {
        const NAME: &str = "ABI_TEST_PRODUCED_BUDGET_BYTES";
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let wide: Vec<_> = NAME.encode_utf16().chain(Some(0)).collect();
        let saved = crate::ctool_environment::read(&wide, &resources).unwrap();
        struct Restore(Option<crate::ctool_environment::Value>);
        impl Drop for Restore {
            fn drop(&mut self) {
                if let Some(value) = &self.0 {
                    std::env::set_var(NAME, &value.value);
                } else {
                    std::env::remove_var(NAME);
                }
            }
        }
        let restore = Restore(saved);
        for (text, expected) in [
            ("0", 0),
            ("123", 123),
            ("18446744073709551615", 128 << 20),
            ("invalid", 128 << 20),
        ] {
            std::env::set_var(NAME, text);
            let limits = output_limits(&resources).unwrap();
            assert_eq!(limits.request, expected);
            assert_eq!(limits.artifact, 64 << 20);
        }
        drop(restore);
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn source_step_runs_original_hot_collector_and_reports_its_actual_receipt() {
        use ace_artifacts::{codec::ChunkKind, identity::QueryKey, store::PutRequest};
        let dir =
            std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests").join(format!(
                "source-worker-gc-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let store = ManagedStore::open(&dir).unwrap();
        let layer = crate::cache::LAYER_OBJECT_V01;
        let mut kept = String::new();
        for revision in 0..5 {
            let id = store
                .put_artifact(PutRequest {
                    layer: layer.id,
                    key: QueryKey::new(&layer, vec![("revision".into(), revision.to_string())]),
                    kind: ChunkKind::Object,
                    payload: if revision == 0 {
                        vec![0; 1024]
                    } else {
                        vec![revision; 3 << 20]
                    },
                    edges: vec![],
                    project: None,
                })
                .unwrap();
            if revision == 0 {
                store
                    .register_root("source-gc-control", &id, "current")
                    .unwrap();
                kept = id;
            }
        }
        assert!(store.space().total > crate::source_store::HOT_HIGH);
        let result = step("ok", vec![], 0, &store, Instant::now(), Some(42)).unwrap();
        assert_eq!(result.outcome, "ok");
        assert_eq!(result.meter.gc_runs, 1);
        assert!(result.meter.gc_deleted_chunks > 0 && result.meter.gc_freed_bytes > 0);
        assert!(result.meter.hot_bytes <= crate::source_store::HOT_LOW);
        assert!(store.get_artifact_by_id(&kept).unwrap().is_some());
        // A collector error is a visible maintenance failure, not a source
        // rejection promising that a completed publication was rolled back.
        for revision in 5u8..8 {
            store
                .put_artifact(PutRequest {
                    layer: layer.id,
                    key: QueryKey::new(&layer, vec![("revision".into(), revision.to_string())]),
                    kind: ChunkKind::Object,
                    payload: vec![revision; 3 << 20],
                    edges: vec![],
                    project: None,
                })
                .unwrap();
        }
        assert!(store.space().total > crate::source_store::HOT_HIGH);
        store.db().execute_batch("PRAGMA query_only=ON").unwrap();
        let error = step("ok", vec![], 0, &store, Instant::now(), Some(42))
            .err()
            .unwrap();
        assert_eq!(error.code, "store.cleanup-pending");
        assert!(error.message.contains("source operation completed"));
        store.db().execute_batch("PRAGMA query_only=OFF").unwrap();
        assert!(step("ok", vec![], 0, &store, Instant::now(), Some(42)).is_ok());
        for revision in 8u8..11 {
            let id = store
                .put_artifact(PutRequest {
                    layer: layer.id,
                    key: QueryKey::new(&layer, vec![("revision".into(), revision.to_string())]),
                    kind: ChunkKind::Object,
                    payload: vec![revision; 3 << 20],
                    edges: vec![],
                    project: None,
                })
                .unwrap();
            store
                .register_root("source-live-floor", &id, "live")
                .unwrap();
        }
        let backoff = step("ok", vec![], 0, &store, Instant::now(), Some(42))
            .err()
            .unwrap();
        assert_eq!(backoff.code, "store.cleanup-pending");
        assert!(backoff.message.contains("backoff"));
        assert!(store.get_artifact_by_id(&kept).unwrap().is_some());
        assert_eq!(
            store
                .root_artifacts("source-live-floor", "live")
                .unwrap()
                .len(),
            3
        );
    }

    fn request(value: &impl Serialize) -> WorkerRequest {
        WorkerRequest {
            recipe: RECIPE.into(),
            nonce: "budget".into(),
            request_hash: "a".repeat(64),
            deck_b64: base64::engine::general_purpose::STANDARD
                .encode(serde_json::to_vec(value).unwrap()),
            leg: "native".into(),
            store_root: String::new(),
            steps: vec![],
        }
    }
    fn deck() -> Deck {
        Deck {
            schema: DECK_SCHEMA.into(),
            source: "fn main() int { return foreign_add(20,22) }".into(),
            module: "source938".into(),
            manifest: provider::reviewed_add_manifest(),
            capabilities: vec![ace_foreign_contract::manual::SOURCE_CAP.into()],
            provider_bias: 0,
            reference: "add".into(),
        }
    }
    #[test]
    fn source_request_reserves_base64_model_and_structured_manifest_before_decode() {
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let input = request(&deck());
        let (decoded, model) = decode(&input, &resources, CertPolicy::Scalar938).unwrap();
        assert_eq!(decoded.manifest, provider::reviewed_add_manifest());
        assert!(resources.stats().current.index > baseline.index);
        assert!(resources.stats().current.staging >= decoded.source.capacity());
        assert_eq!(resources.stats().current.io, 0);
        drop(decoded);
        drop(model);
        assert_eq!(resources.stats().current, baseline);

        let tight = Resources::limited(Limits {
            index: 1024,
            ..Limits::default()
        })
        .unwrap();
        assert_eq!(
            decode(&input, &tight, CertPolicy::Scalar938).err().unwrap().code,
            "foreign.resource"
        );
        assert_eq!(tight.stats().current.io, 0);
        let io = Resources::limited(Limits {
            io: 128,
            ..Limits::default()
        })
        .unwrap();
        assert_eq!(decode(&input, &io, CertPolicy::Scalar938).err().unwrap().code, "foreign.resource");
        assert_eq!(io.stats().current.io, 0);
        let mut malformed = input;
        malformed.deck_b64 = "%%%".into();
        assert_eq!(decode(&malformed, &resources, CertPolicy::Scalar938).err().unwrap().code, "deck");
        assert_eq!(resources.stats().current, baseline);
        let mut unknown = serde_json::to_value(deck()).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("safe".into(), true.into());
        assert_eq!(
            decode(&request(&unknown), &resources, CertPolicy::Scalar938).err().unwrap().code,
            "deck"
        );
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn source_target_limits_keep_original_native_and_neutral_frontend_caps() {
        let native = ace_native::config::MAX_INPUT_BYTES;
        let neutral = ace_source_query::frontend::MAX_SOURCE_BYTES;
        assert_eq!(native, 4 << 20);
        assert_eq!(neutral, 8 << 20);
        assert!(Target::Native.admit_source_bytes(native).is_ok());
        assert!(Target::Native
            .admit_source_bytes(native + 1)
            .unwrap_err()
            .contains("foreign.resource.source"));
        assert!(Target::C.admit_source_bytes(native + 1).is_ok());
        assert!(Target::C.admit_source_bytes(neutral).is_ok());
        assert!(Target::C.admit_source_bytes(neutral + 1).is_err());
    }
}
