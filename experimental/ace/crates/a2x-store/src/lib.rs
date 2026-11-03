//! PLAN-892 T-04 (P-private part) + PLAN-902 T-05: the A2X layered
//! cache/session over the PLAN-783 `ManagedStore`/CAS — no second database
//! (plan §5.4).
//!
//! Layer registry (FROZEN here; the Design 40 A00–A12 ids are deliberately
//! NOT squatted — these are A2X-owned namespaces registered beside them):
//!
//! | layer   | name                   | chunk kind        | contents                     |
//! |---------|------------------------|-------------------|------------------------------|
//! | `AXS1`  | a2x-function-semantic  | Hir               | per-function semantic plan bytes (target-neutral) |
//! | `AXR1`  | a2x-rust-fragment      | Object            | one printed Rust fragment (target layer) |
//! | `AXM1`  | a2x-fileset-manifest   | ObjectManifest    | an assembly manifest binding its fragments (required edges) |
//! | `AXS2`  | a2x-rt-function-semantic | Hir             | PLAN-902 record/text semantics (modes/nominal/callee digests ride the fingerprint) |
//! | `AXR2`  | a2x-rt-rust-fragment   | Object            | one printed record-text fragment (rt runtime content keyed) |
//!
//! The 902 layers are SEPARATE namespaces (`a2902/*` schemas): scalar
//! keys, bytes and hits are untouched, and the two families never
//! cross-hit. Key layering (§5.4): the semantic key reads NO target
//! configuration; every result that reads a [`TargetProfile`] lands in the
//! target layer whose key ADDS the target fields (lang/edition/backend/
//! version/runtime content). Absolute spans, snapshots and table
//! reordering never enter a key — only the semantic fingerprint and
//! declared identities do.

use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::target::{FilesetRequest, FragmentText, TargetProvider};
use ace_a2x_core::{semantic_fingerprint, A2xReject, FnFacts, SemanticView};
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::{QueryKey, RefKind};
use ace_artifacts::schema::LayerSpec;
use ace_artifacts::store::{gc_config_default, run_gc, ManagedStore, PutRequest, SpaceReport};
use ace_core::source::{
    compile_collections_source, compile_record_source, compile_scalar_source, compile_source,
    compile_text_record_source, compile_text_source, parse_function_fragment,
    parse_method_fragment, parse_program_with, prepare_function_candidate, resolve_env,
    SourceProfile,
};
use ace_core::text_resources::{param_exclusions_fixpoint, ParamExclusions};
use ace_core::verify::{verify_function_ref, verify_text_function};
use std::collections::BTreeMap;
use std::path::Path;

pub mod anchor;

pub use anchor::{Anchor, AnchorFn, ANCHOR_SCHEMA};

// PLAN-921: the shared session layer no longer statically owns Rust-private
// lowering/profile items — every target-specific operation (fragment key
// identity, production, runtime source, prelude inputs, file-set assembly)
// is delegated to the [`TargetProvider`] the session is opened with. The
// Rust provider lives in `a2x-rust` (`RustTargetProvider`), the C provider
// in `a2x-c`; the semantic layers stay shared and target-neutral.

pub const LAYER_SEMANTIC: LayerSpec = LayerSpec {
    id: "AXS1",
    name: "a2x-function-semantic",
    schema_id: "a2892/axs-fn-sem/v1",
    producer: "auto-a2x-core prepare",
    // PLAN-921: the C target consumes the SAME semantic facts (keys and
    // bytes unchanged; additive consumer registration only).
    consumers: &["auto-a2x-rust", "auto-a2x-c"],
    key_inputs: &[
        "owner",
        "decl",
        "profile",
        "hir_schema",
        "signature",
        "sem_fp",
        "caps",
        "pass",
    ],
    ref_kinds: &[],
    storage: "one semantic-plan artifact per function",
    residency: "restored on demand; a restored fragment is re-verified, never trusted blind",
    regeneration: ace_artifacts::schema::Regeneration::FromUserInputs,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

pub const LAYER_FRAGMENT: LayerSpec = LayerSpec {
    id: "AXR1",
    name: "a2x-rust-fragment",
    schema_id: "a2892/axr-rust-fn/v1",
    producer: "auto-a2x-rust",
    consumers: &["a2x session assembly"],
    key_inputs: &[
        "owner",
        "decl",
        "sem_fp",
        "lang",
        "edition",
        "backend",
        "backend_version",
        "runtime_sha256",
        "print_version",
    ],
    ref_kinds: &["provenance"],
    storage: "one printed Rust fragment artifact per function",
    residency: "resident fragments are the warm reuse unit; store hits re-verify by content hash",
    regeneration: ace_artifacts::schema::Regeneration::FromUpstreamRecipe,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

pub const LAYER_MANIFEST: LayerSpec = LayerSpec {
    id: "AXM1",
    name: "a2x-fileset-manifest",
    schema_id: "a2892/axm-fileset/v1",
    producer: "a2x session",
    consumers: &["certification"],
    key_inputs: &[
        "owner",
        "entry",
        "source_sha256",
        "lang",
        "edition",
        "backend",
        "fileset_schema",
    ],
    ref_kinds: &["required"],
    storage: "one manifest artifact per assembly",
    residency: "manifests pin their fragment set with required edges",
    regeneration: ace_artifacts::schema::Regeneration::FromUpstreamRecipe,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-902 T-05: the record/text/text-record semantic layer — same key
/// shape as AXS1, separate namespace (modes, nominal identities, callee
/// signatures and record-shape digests ride `sem_fp`; the 902 fingerprint
/// serializes all of them).
pub const LAYER_SEMANTIC_RT: LayerSpec = LayerSpec {
    id: "AXS2",
    name: "a2x-rt-function-semantic",
    schema_id: "a2902/axs-rt-sem/v1",
    producer: "auto-a2x-core prepare",
    consumers: &["auto-a2x-rust"],
    key_inputs: &[
        "owner",
        "decl",
        "profile",
        "hir_schema",
        "signature",
        "sem_fp",
        "caps",
        "pass",
    ],
    ref_kinds: &[],
    storage: "one record/text semantic-plan artifact per function",
    residency: "restored on demand; a restored fragment is re-verified, never trusted blind",
    regeneration: ace_artifacts::schema::Regeneration::FromUserInputs,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-902 T-05: the record/text fragment layer — the runtime content
/// input is the RECORD-TEXT runtime (resource helpers), so a scalar hit
/// can never satisfy an rt key or vice versa.
pub const LAYER_FRAGMENT_RT: LayerSpec = LayerSpec {
    id: "AXR2",
    name: "a2x-rt-rust-fragment",
    schema_id: "a2902/axr-rt-fn/v1",
    producer: "auto-a2x-rust",
    consumers: &["a2x session assembly"],
    key_inputs: &[
        "owner",
        "decl",
        "sem_fp",
        "lang",
        "edition",
        "backend",
        "backend_version",
        "runtime_sha256",
        "print_version",
    ],
    ref_kinds: &["provenance"],
    storage: "one printed record-text Rust fragment per function",
    residency: "resident fragments are the warm reuse unit; store hits re-verify by content hash",
    regeneration: ace_artifacts::schema::Regeneration::FromUpstreamRecipe,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-904 T-06: the collections semantic layer — same key shape, its
/// own namespace (list/variant nominal digests ride `sem_fp`; keys/bytes
/// of AXS1/AXS2 never move).
pub const LAYER_SEMANTIC_COLL: LayerSpec = LayerSpec {
    id: "AXS3",
    name: "a2x-collections-function-semantic",
    schema_id: "a2904/axs-coll-sem/v1",
    producer: "auto-a2x-core prepare",
    consumers: &["auto-a2x-rust"],
    key_inputs: &[
        "owner",
        "decl",
        "profile",
        "hir_schema",
        "signature",
        "sem_fp",
        "caps",
        "pass",
    ],
    ref_kinds: &[],
    storage: "one collections semantic-plan artifact per function",
    residency: "restored on demand; a restored fragment is re-verified, never trusted blind",
    regeneration: ace_artifacts::schema::Regeneration::FromUserInputs,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-904 T-06: the collections fragment layer — the runtime content
/// input is the COLLECTIONS runtime (list/billing/cleanup helpers), so a
/// scalar or record-text hit can never satisfy a collections key.
pub const LAYER_FRAGMENT_COLL: LayerSpec = LayerSpec {
    id: "AXR3",
    name: "a2x-collections-rust-fragment",
    schema_id: "a2904/axr-coll-fn/v1",
    producer: "auto-a2x-rust",
    consumers: &["a2x session assembly"],
    key_inputs: &[
        "owner",
        "decl",
        "sem_fp",
        "lang",
        "edition",
        "backend",
        "backend_version",
        "runtime_sha256",
        "print_version",
    ],
    ref_kinds: &["provenance"],
    storage: "one printed collections Rust fragment per function",
    residency: "resident fragments are the warm reuse unit; store hits re-verify by content hash",
    regeneration: ace_artifacts::schema::Regeneration::FromUpstreamRecipe,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-921 T-04: the C (HIR→C11 scalar) fragment layer — its own
/// namespace beside the Rust layers. The runtime content input is the C
/// runtime, so a Rust hit can never satisfy a C key or vice versa (the
/// backend/standard/print fields differ besides).
pub const LAYER_FRAGMENT_C: LayerSpec = LayerSpec {
    id: "AXC1",
    name: "a2x-c-fragment",
    schema_id: "a2921/axc-c-fn/v1",
    producer: "auto-a2x-c",
    consumers: &["a2x session assembly"],
    key_inputs: &[
        "owner",
        "decl",
        "sem_fp",
        "lang",
        "standard",
        "backend",
        "backend_version",
        "runtime_sha256",
        "print_version",
    ],
    ref_kinds: &["provenance"],
    storage: "one printed C fragment artifact per function",
    residency: "resident fragments are the warm reuse unit; store hits re-verify by content hash",
    regeneration: ace_artifacts::schema::Regeneration::FromUpstreamRecipe,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-925 CA-B: the C (HIR→C11 record-text) fragment layer — its own
/// namespace beside the Rust record-text layer (AXR2) and the C scalar
/// one (AXC1). The runtime content input is the C typed resource
/// runtime, so neither a Rust hit nor a C scalar hit can satisfy an AXC2
/// key or vice versa (the runtime hash differs besides).
pub const LAYER_FRAGMENT_C_RT: LayerSpec = LayerSpec {
    id: "AXC2",
    name: "a2x-c-rt-fragment",
    schema_id: "a2925/axc-c-rt-fn/v1",
    producer: "auto-a2x-c",
    consumers: &["a2x session assembly"],
    key_inputs: &[
        "owner",
        "decl",
        "sem_fp",
        "lang",
        "standard",
        "backend",
        "backend_version",
        "runtime_sha256",
        "print_version",
    ],
    ref_kinds: &["provenance"],
    storage: "one printed C record-text fragment artifact per function",
    residency: "resident fragments are the warm reuse unit; store hits re-verify by content hash",
    regeneration: ace_artifacts::schema::Regeneration::FromUpstreamRecipe,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// PLAN-921 T-06: the source-bound anchor layer — the per-function source
/// binding (full-source sha256 → per-function origins/slice hashes/sem_fps
/// + the assembly file-set receipt) that makes NEW-PROCESS pure restore
/// (P0/R>0) possible without re-running the whole-source frontend. One
/// anchor artifact per published assembly state.
pub const LAYER_SOURCE_ANCHOR: LayerSpec = LayerSpec {
    id: "AXP1",
    name: "a2x-source-anchor",
    schema_id: "a2921/axp-source-anchor/v1",
    producer: "a2x session",
    consumers: &["a2x session restore"],
    key_inputs: &["owner", "profile", "source_sha256", "backend"],
    ref_kinds: &["required"],
    storage: "one source-bound anchor artifact per assembly state",
    residency:
        "restored artifacts are re-verified against the caller's source, never trusted blind",
    regeneration: ace_artifacts::schema::Regeneration::FromUserInputs,
    coverage: ace_artifacts::schema::LayerCoverage::Codec,
};

/// Pass-version constants of THIS build (key inputs; §5.4 "pass版本").
pub const PASS_PREPARE: &str = "a2x-prepare/1";
pub const PRINT_VERSION: &str = "a2x-print/1";
/// PLAN-902: the record-text prepare pass (fingerprint carries modes,
/// nominal identities, callee signatures and record-shape digests).
pub const PASS_PREPARE_RT: &str = "a2x-prepare-rt/1";
/// PLAN-904: the collections prepare pass (the fingerprint carries the
/// list/variant nominal digests and callee signatures).
pub const PASS_PREPARE_COLL: &str = "a2x-prepare-coll/1";

/// Real per-function production accounting (§5.4 "真实producer按function计…
/// 次数/字节与whole fallback"). Every counter is a fact the probes assert.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProducerStats {
    pub parse: u64,
    pub recheck: u64,
    pub emit_body: u64,
    pub prepare_candidate: u64,
    pub verify: u64,
    pub lower: u64,
    pub print: u64,
    pub persist: u64,
    pub bytes_persisted: u64,
    pub store_hit_semantic: u64,
    pub store_hit_fragment: u64,
    pub fragments_reused_resident: u64,
    pub whole_fallback: u64,
    /// PLAN-921 T-06: pure-restore accounting - lookup/decode of the
    /// source-bound anchor (M), per-function source-binding re-verifies
    /// (R), semantic/fragment payload re-verifies (R), file-set receipt
    /// re-verify (R). Restore NEVER touches the production counters.
    pub restore_lookup: u64,
    pub restore_binding_checks: u64,
    pub restore_payload_verifies: u64,
    pub restore_fileset_verifies: u64,
    pub restore_anchors_published: u64,
    /// Whole-source env rebuilds enabling warm edits after a pure
    /// restore (M-class management; the parse/resolve cost rides the
    /// parse counter).
    pub env_rebuilds: u64,
}

/// One resident per-function fragment (the warm-session reuse unit).
#[derive(Clone, Debug)]
pub struct ResidentFragment {
    pub source_name: String,
    pub def_index: usize,
    pub sem_fp: String,
    /// The printed Rust fragment text.
    pub text: String,
}

/// An assembly: the full emit file set + its manifest.
#[derive(Clone, Debug)]
pub struct Assembly {
    pub files: Vec<(String, String)>,
    pub manifest: String,
}

/// A controlled A2X resource/quota rejection (§5.6): the store keeps its
/// previous state; nothing partial lands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceReject {
    pub code: &'static str,
    pub detail: String,
}

impl std::fmt::Display for ResourceReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

/// The channel family of the session's profile (layer-set selection) —
/// PLAN-921: shared via `ace_a2x_core::target::Family` (re-exported here
/// for source compatibility).
pub use ace_a2x_core::target::Family;

/// The A2X session: cold init, warm per-function edits with RESIDENT reuse,
/// layered persistence and quota gating over one `ManagedStore`.
pub struct A2xSession {
    store: ManagedStore,
    project: String,
    owner: String,
    quota_bytes: u64,
    pub stats: ProducerStats,
    /// PLAN-921: the emit target this session produces through. Switching
    /// targets on ONE session (and one store) is the mixed-target
    /// contract — fragment identities isolate the namespaces, residents
    /// are per-backend.
    provider: Box<dyn TargetProvider>,
    /// (backend, source name) → resident fragment (the last PUBLISHED
    /// semantics per fn under that target). Keyed by source name — def
    /// indices are NOT stable across whole-module and single-body bundles
    /// (F-828-R1). Methods are owner-qualified residents (`Counter_read`).
    resident: BTreeMap<(String, String), ResidentFragment>,
    /// The retained checked environment (None until cold init).
    env: Option<ace_core::source::resolve::CheckedEnv>,
    /// PLAN-902: the retained whole-module pairwise exclusivity map for
    /// text-family profiles (the single-body credential's seed).
    retained_excl: Option<ParamExclusions>,
    profile: SourceProfile,
    caps: Vec<String>,
    /// PLAN-904: sha256 of the CURRENT FULL SOURCE the collections
    /// channel binds its manifest to (cold init and every ranged warm
    /// edit rebind it).
    source_sha256: Option<String>,
    /// PLAN-921 T-06: (backend, source name) -> fragment artifact id of
    /// the resident set (anchor required-edge inputs).
    resident_ids: BTreeMap<(String, String), String>,
    /// The last published source-bound anchor (the rebinding base of
    /// ranged warm edits). None until a scalar-family cold init.
    last_anchor: Option<Anchor>,
    last_anchor_id: Option<String>,
}

impl A2xSession {
    /// Open a session on a (new or existing) store root with an explicit
    /// emit target provider (PLAN-921: the Rust provider is
    /// `ace_a2x_rust::RustTargetProvider::v1()`). `quota_bytes` caps the
    /// NAMESPACE total (blocks+db+staging/trash/locks+logs) for puts from
    /// THIS session — a put that would cross it is a controlled
    /// `a2x.resource` rejection leaving prior state intact.
    pub fn open<P: TargetProvider + 'static>(
        root: &Path,
        project: &str,
        owner: &str,
        quota_bytes: u64,
        provider: P,
    ) -> Result<Self, String> {
        let store = ManagedStore::open(root).map_err(|e| format!("open store: {:?}", e))?;
        Ok(A2xSession {
            store,
            project: project.to_string(),
            owner: owner.to_string(),
            quota_bytes,
            stats: ProducerStats::default(),
            provider: Box::new(provider),
            resident: BTreeMap::new(),
            env: None,
            retained_excl: None,
            profile: SourceProfile::Core,
            caps: vec![],
            source_sha256: None,
            resident_ids: BTreeMap::new(),
            last_anchor: None,
            last_anchor_id: None,
        })
    }

    /// PLAN-921: switch the session's emit target (the mixed-target
    /// contract — same session, same store; fragment keys/residents stay
    /// isolated per backend). The retained environment and source binding
    /// are target-neutral and keep serving the next target's cold/warm
    /// ops.
    pub fn set_target<P: TargetProvider + 'static>(&mut self, provider: P) {
        self.provider = Box::new(provider);
    }

    /// PLAN-921: the current target's backend identity.
    pub fn current_backend(&self) -> &str {
        self.provider.descriptor().backend
    }

    pub fn space(&self) -> SpaceReport {
        self.store.space()
    }

    pub fn stats(&self) -> &ProducerStats {
        &self.stats
    }

    /// The channel family of the current profile (layer-set + fileset
    /// selection; scalar keys/bytes never move).
    pub fn family(&self) -> Family {
        match self.profile {
            SourceProfile::Record | SourceProfile::Text | SourceProfile::TextRecord => {
                Family::RecordText
            }
            SourceProfile::Collections => Family::Collections,
            _ => Family::Scalar,
        }
    }

    fn layer_semantic(&self) -> &'static LayerSpec {
        match self.family() {
            Family::Scalar => &LAYER_SEMANTIC,
            Family::RecordText => &LAYER_SEMANTIC_RT,
            Family::Collections => &LAYER_SEMANTIC_COLL,
        }
    }

    fn layer_fragment(&self) -> &'static LayerSpec {
        // PLAN-921: target-owned layer selection (AXR1/2/3 for Rust, AXC1
        // for C). Unsupported families are NAMED rejections validated at
        // cold/warm entry (`ensure_target_supports`); an id missing from
        // the registry is an internal wiring error.
        match self.provider.fragment_layer_id(self.family()) {
            Ok("AXR1") => &LAYER_FRAGMENT,
            Ok("AXR2") => &LAYER_FRAGMENT_RT,
            Ok("AXR3") => &LAYER_FRAGMENT_COLL,
            Ok("AXC1") => &LAYER_FRAGMENT_C,
            Ok("AXC2") => &LAYER_FRAGMENT_C_RT,
            other => panic!("provider returned unregistered fragment layer {:?}", other),
        }
    }

    /// PLAN-921/925: named rejection when the current target does not
    /// serve the session's family (C serves the scalar/record-text
    /// channels).
    fn ensure_target_supports(&self) -> Result<(), A2xReject> {
        self.provider.fragment_layer_id(self.family()).map(|_| ())
    }

    fn pass_version(&self) -> &'static str {
        match self.family() {
            Family::Scalar => PASS_PREPARE,
            Family::RecordText => PASS_PREPARE_RT,
            Family::Collections => PASS_PREPARE_COLL,
        }
    }

    /// Semantic-layer key of a function (NO target fields). `signature`
    /// is the caller-computed canonical signature — the frozen scalar
    /// rendering on the scalar family (byte-identical AXS1 keys), the
    /// view-aware rendering (modes + nominal names) on the 902 family.
    fn semantic_key(
        &self,
        f: &FnFacts<'_>,
        sem_fp: &str,
        hir_schema: &str,
        signature: &str,
    ) -> QueryKey {
        QueryKey::new(
            self.layer_semantic(),
            vec![
                ("owner".into(), self.owner.clone()),
                ("decl".into(), f.source_name.to_string()),
                ("profile".into(), self.profile_label().into()),
                ("hir_schema".into(), hir_schema.to_string()),
                ("signature".into(), signature.to_string()),
                ("sem_fp".into(), sem_fp.to_string()),
                ("caps".into(), self.caps.join(",")),
                ("pass".into(), self.pass_version().into()),
            ],
        )
    }

    /// Canonical signature key input for `f` under the current family.
    fn signature_input(&self, view: &SemanticView<'_>, f: &FnFacts<'_>) -> String {
        match self.family() {
            Family::Scalar => ace_a2x_core::canonical_signature(f),
            Family::RecordText => view.canonical_signature(f),
            Family::Collections => view.canonical_signature(f),
        }
    }

    /// Target-layer key: semantic key inputs + the provider's target
    /// fields (§5.4). The Rust provider reproduces the frozen AXR1/2/3
    /// field order byte-identically; other targets spell their own set.
    fn fragment_key(&self, f: &FnFacts<'_>, sem_fp: &str, runtime_sha: &str) -> QueryKey {
        let mut inputs = vec![
            ("owner".into(), self.owner.clone()),
            ("decl".into(), f.source_name.to_string()),
            ("sem_fp".into(), sem_fp.to_string()),
        ];
        inputs.extend(self.provider.fragment_key_inputs(runtime_sha));
        QueryKey::new(self.layer_fragment(), inputs)
    }

    fn profile_label(&self) -> &'static str {
        match self.profile {
            SourceProfile::Core => "core",
            SourceProfile::Scalar => "scalar",
            SourceProfile::Record => "record",
            SourceProfile::Text => "text",
            SourceProfile::TextRecord => "text-record",
            _ => "unsupported",
        }
    }

    /// Quota gate: projected namespace total after a put of `payload`.
    fn quota_guard(&self, payload_len: u64) -> Result<(), ResourceReject> {
        let total = self.space().total;
        if total.saturating_add(payload_len) > self.quota_bytes {
            return Err(ResourceReject {
                code: "a2x.resource",
                detail: format!(
                    "persisting {}B would take the namespace to {}B, over the {}B quota",
                    payload_len,
                    total + payload_len,
                    self.quota_bytes
                ),
            });
        }
        Ok(())
    }

    /// COLD initialization over a whole source (§5.4 allows whole-source
    /// init). Compiles once, persists per-function semantic+fragment
    /// artifacts, assembles the manifest and populates the resident set.
    /// The scalar-family entry (892 contract, unchanged).
    pub fn cold_assemble(
        &mut self,
        src: &str,
        scalar: bool,
        caps: &[String],
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let profile = if scalar {
            SourceProfile::Scalar
        } else {
            SourceProfile::Core
        };
        self.cold_assemble_profile(src, profile, caps, entry, read_only)
    }

    /// PLAN-902: cold initialization for ANY supported profile — the
    /// record/text/text-record domains ride the record-text channel
    /// (AXS2/AXR2 layers, rt fileset) with the same resident loop.
    pub fn cold_assemble_profile(
        &mut self,
        src: &str,
        profile: SourceProfile,
        caps: &[String],
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        self.profile = profile;
        self.caps = caps.to_vec();
        self.source_sha256 = Some(sha256_hex(src.as_bytes()));
        // PLAN-921: the target must serve this family (a named rejection —
        // e.g. the C target serves the scalar channel only)
        self.ensure_target_supports()
            .map_err(A2xRejectOrResource::Reject)?;
        self.stats.parse += 1;
        let compiled = match profile {
            SourceProfile::Core => compile_source(src, "a2x", caps),
            SourceProfile::Scalar => compile_scalar_source(src, "a2x", caps),
            SourceProfile::Record => compile_record_source(src, "a2x", caps),
            SourceProfile::Text => compile_text_source(src, "a2x", caps),
            SourceProfile::TextRecord => compile_text_record_source(src, "a2x", caps),
            SourceProfile::Collections => compile_collections_source(src, "a2x", caps),
            other => {
                return Err(A2xRejectOrResource::Reject(A2xReject::domain(format!(
                    "unsupported profile {:?}",
                    other
                ))))
            }
        }
        .map_err(|ds| {
            A2xRejectOrResource::Reject(A2xReject::new("a2x.compile", render_diag(&ds)))
        })?;
        let view =
            SemanticView::from_checked(&compiled.checked).map_err(A2xRejectOrResource::Reject)?;

        // PLAN-902: retain the whole-module exclusivity map for the text
        // families (the single-body credential seed of warm edits)
        self.retained_excl = if self.family() == Family::RecordText
            && matches!(profile, SourceProfile::Text | SourceProfile::TextRecord)
        {
            Some(param_exclusions_fixpoint(compiled.checked.bundle()))
        } else {
            None
        };

        // retained environment for warm edits
        let program = parse_program_with(src, profile)
            .map_err(|d| A2xRejectOrResource::Reject(A2xReject::new("a2x.parse", d.message)))?;
        self.stats.parse += 1;
        let env = resolve_env(&program, caps, profile).map_err(|ds| {
            A2xRejectOrResource::Reject(A2xReject::new("a2x.resolve", render_diag(&ds)))
        })?;
        self.env = Some(env);

        // warm-check the whole thing through the resident loop; the cold
        // walk is one whole-fallback counter (§5.4 observation)
        self.stats.whole_fallback += 1;
        self.assemble_from_view(&view, &compiled.input_hash, entry, src, read_only)
            .map_err(|e| e)
    }

    /// Assemble from a view, reusing resident fragments and store hits;
    /// persists only what is new (§5.4 "未改函数生产/工件写入均0").
    fn assemble_from_view(
        &mut self,
        view: &SemanticView<'_>,
        input_hash: &str,
        entry: &str,
        _src: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let family = self.family();
        let backend = self.current_backend().to_string();
        let runtime = self
            .provider
            .runtime_source(view, family)
            .map_err(A2xRejectOrResource::Reject)?;
        let runtime_sha = sha256_hex(runtime.as_bytes());
        let mut fragments: Vec<(usize, ResidentFragment)> = Vec::new();
        // store misses, collected for ONE batched whole-view production
        // pass through the provider (per-function lowering would re-lower
        // the module O(n) times at load scale). `miss_slots` preserves
        // each miss's LOOP POSITION so the assembled fragment order (and
        // therefore the generated-file bytes) is exactly the frozen one.
        let mut misses: Vec<(&FnFacts<'_>, String)> = Vec::new();
        let mut miss_slots: Vec<usize> = Vec::new();

        for f in view.functions() {
            self.stats.verify += 1;
            let sem_fp = semantic_fingerprint(view, f).map_err(A2xRejectOrResource::Reject)?;

            // 1) resident reuse (per-target): unchanged function/method →
            //    ZERO production
            if let Some(r) = self
                .resident
                .get(&(backend.clone(), f.source_name.to_string()))
            {
                if r.sem_fp == sem_fp {
                    self.stats.fragments_reused_resident += 1;
                    fragments.push((f.def_index, r.clone()));
                    continue;
                }
            }

            // 2) store hit on the TARGET layer: reuse the checked payload
            //    (A→B→A revisits land here with real historical bytes)
            let frag_key = self.fragment_key(f, &sem_fp, &runtime_sha);
            if let Ok(Some(hit)) = self.store.get_artifact(self.layer_fragment().id, &frag_key) {
                let text = String::from_utf8(hit.payload.clone()).map_err(|_| {
                    A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.internal",
                        "fragment payload not utf8",
                    ))
                })?;
                self.stats.store_hit_fragment += 1;
                self.resident_ids.insert(
                    (backend.clone(), f.source_name.to_string()),
                    hit.artifact_id.clone(),
                );
                fragments.push((
                    f.def_index,
                    ResidentFragment {
                        source_name: f.source_name.to_string(),
                        def_index: f.def_index,
                        sem_fp,
                        text,
                    },
                ));
                continue;
            }

            // 3) miss: reserve the slot now (order), produce in batch below
            miss_slots.push(fragments.len());
            misses.push((f, sem_fp.clone()));
            fragments.push((
                f.def_index,
                ResidentFragment {
                    source_name: f.source_name.to_string(),
                    def_index: f.def_index,
                    sem_fp,
                    text: String::new(),
                },
            ));
        }

        if !misses.is_empty() {
            // produce: lower + print (+ persist), ONE provider batch
            self.stats.lower += misses.len() as u64;
            self.stats.print += misses.len() as u64;
            let fns: Vec<&FnFacts<'_>> = misses.iter().map(|(f, _)| *f).collect();
            let texts = self
                .provider
                .produce_fragments(view, &fns, family)
                .map_err(A2xRejectOrResource::Reject)?;
            for (i, ((f, sem_fp), text)) in misses.iter().zip(texts).enumerate() {
                if !read_only {
                    // semantic layer first
                    let sig_input = self.signature_input(view, f);
                    let sem_key = self.semantic_key(f, sem_fp, view.domain.schema(), &sig_input);
                    let sem_payload = format!(
                        "{{\"schema\":\"{}\",\"decl\":\"{}\",\"signature\":\"{}\",\"sem_fp\":\"{}\"}}",
                        self.layer_semantic().schema_id,
                        f.source_name,
                        view.canonical_signature(f),
                        sem_fp
                    );
                    let sem_id = self.quota_put(
                        self.layer_semantic().id,
                        sem_key,
                        ChunkKind::Hir,
                        sem_payload.as_bytes(),
                        vec![],
                    )?;
                    // fragment layer (provenance edge to its semantic input)
                    let frag_key = self.fragment_key(f, sem_fp, &runtime_sha);
                    let frag_id = self.quota_put(
                        self.layer_fragment().id,
                        frag_key,
                        ChunkKind::Object,
                        text.as_bytes(),
                        vec![(sem_id, RefKind::Provenance)],
                    )?;
                    self.resident_ids
                        .insert((backend.clone(), f.source_name.to_string()), frag_id);
                }
                fragments[miss_slots[i]].1.text = text;
            }
        }

        // publish residents (current target's namespace) + assemble the
        // file set through the provider
        self.resident.retain(|(b, _), _| b != &backend);
        for (_, r) in &fragments {
            self.resident
                .insert((backend.clone(), r.source_name.clone()), r.clone());
        }
        let prelude_seed = if family == Family::RecordText || family == Family::Collections {
            self.provider
                .capture_prelude_seed(view, family)
                .map_err(A2xRejectOrResource::Reject)?
        } else {
            None
        };
        let frags: Vec<FragmentText<'_>> = fragments
            .iter()
            .map(|(_, r)| FragmentText {
                def_index: r.def_index,
                source_name: r.source_name.as_str(),
                sem_fp: r.sem_fp.as_str(),
                text: r.text.as_str(),
            })
            .collect();
        let fileset = self
            .provider
            .assemble_fileset(FilesetRequest {
                entry,
                input_hash,
                family,
                fragments: &frags,
                view: Some(view),
                runtime: &runtime,
                prelude_seed: prelude_seed.as_deref(),
                origins: None,
                source_sha256: self.source_sha256.as_deref(),
            })
            .map_err(A2xRejectOrResource::Reject)?;
        let assembly = Assembly {
            files: fileset.files,
            manifest: fileset.manifest,
        };
        // PLAN-921 T-06: publish the source-bound anchor (the SCALAR
        // family; its file set re-assembles from fragments alone). The
        // anchor is the COMMIT POINT of the cold publication: its
        // required edges keep every fragment alive and are
        // liveness-checked atomically at this put - a crash before it
        // leaves a store restore rejects, never a half-restorable one.
        if family == Family::Scalar && !read_only {
            let src_sha = self
                .source_sha256
                .clone()
                .unwrap_or_else(|| sha256_hex(_src.as_bytes()));
            let mut fns = Vec::with_capacity(fragments.len());
            for (_, r) in &fragments {
                let origin = view
                    .fn_by_def(r.def_index)
                    .map(|f| f.origin)
                    .unwrap_or((0, 0));
                let slice = _src.get(origin.0 as usize..origin.1 as usize).unwrap_or("");
                // the SEMANTIC-KEY signature input (the frozen scalar
                // rendering on this family — NOT the view-aware one)
                let signature = view
                    .fn_by_def(r.def_index)
                    .map(ace_a2x_core::canonical_signature)
                    .unwrap_or_default();
                fns.push(anchor::AnchorFn {
                    decl: r.source_name.clone(),
                    def_index: r.def_index as u32,
                    origin,
                    slice_sha256: sha256_hex(slice.as_bytes()),
                    sem_fp: r.sem_fp.clone(),
                    signature,
                });
            }
            let fileset_receipt: Vec<(String, String)> = assembly
                .files
                .iter()
                .map(|(p, c)| (p.clone(), sha256_hex(c.as_bytes())))
                .collect();
            let a = anchor::Anchor {
                schema: anchor::ANCHOR_SCHEMA.to_string(),
                owner: self.owner.clone(),
                profile: self.profile_label().to_string(),
                backend: self.current_backend().to_string(),
                entry: entry.to_string(),
                source_sha256: src_sha,
                input_hash: input_hash.to_string(),
                hir_schema: view.domain.schema().to_string(),
                caps: self.caps.clone(),
                runtime_sha256: runtime_sha,
                runtime_text: runtime,
                functions: fns,
                fileset: fileset_receipt,
            };
            let edges: Vec<(String, RefKind)> = fragments
                .iter()
                .filter_map(|(_, r)| {
                    self.resident_ids
                        .get(&(backend.clone(), r.source_name.clone()))
                        .map(|id| (id.clone(), RefKind::RequiredData))
                })
                .collect();
            let id = self.publish_anchor(&a, edges)?;
            self.last_anchor = Some(a);
            self.last_anchor_id = Some(id);
        }
        Ok(assembly)
    }

    /// Publish an anchor IDEMPOTENTLY: a byte-identical anchor under the
    /// same key is reused (no put, no quota, no persist counter) — the
    /// common re-cold path; a genuinely new state publishes atomically
    /// with its required edges (the liveness-checked commit point).
    fn publish_anchor(
        &mut self,
        a: &anchor::Anchor,
        edges: Vec<(String, RefKind)>,
    ) -> Result<String, A2xRejectOrResource> {
        let key = self.anchor_key(&a.source_sha256, &a.entry);
        let encoded = a.encode();
        if let Ok(Some(hit)) = self.store.get_artifact(LAYER_SOURCE_ANCHOR.id, &key) {
            if hit.payload == encoded.as_bytes() {
                return Ok(hit.artifact_id);
            }
            // different content under one key is a store-level conflict —
            // a re-derived anchor for the same state must be byte-stable
        }
        let id = self.quota_put(
            LAYER_SOURCE_ANCHOR.id,
            key,
            ChunkKind::ObjectManifest,
            encoded.as_bytes(),
            edges,
        )?;
        self.stats.restore_anchors_published += 1;
        Ok(id)
    }

    /// PLAN-921 T-06: the anchor-layer key of one assembly state.
    fn anchor_key(&self, source_sha256: &str, entry: &str) -> QueryKey {
        QueryKey::new(
            &LAYER_SOURCE_ANCHOR,
            vec![
                ("owner".into(), self.owner.clone()),
                ("profile".into(), self.profile_label().into()),
                ("source_sha256".into(), source_sha256.to_string()),
                ("backend".into(), self.current_backend().to_string()),
                ("entry".into(), entry.to_string()),
            ],
        )
    }

    fn quota_put(
        &mut self,
        layer: &str,
        key: QueryKey,
        kind: ChunkKind,
        payload: &[u8],
        edges: Vec<(String, RefKind)>,
    ) -> Result<String, A2xRejectOrResource> {
        self.quota_guard(payload.len() as u64)?;
        self.stats.persist += 1;
        self.stats.bytes_persisted += payload.len() as u64;
        let req = PutRequest {
            layer: static_layer(layer),
            key,
            kind,
            payload: payload.to_vec(),
            edges,
            project: Some(self.project.clone()),
        };
        // count only when the put actually happens
        self.store.put_artifact(req).map_err(|e| {
            A2xRejectOrResource::Resource(ResourceReject {
                code: "a2x.resource",
                detail: format!("store put rejected: {:?}", e),
            })
        })
    }

    /// WARM pure body edit of ONE function: prepare → verify → (reuse or)
    /// re-produce exactly that fragment; every other function comes from
    /// the resident set with ZERO production (§5.4). PLAN-902: text-family
    /// edits verify through the single-body text credential (the retained
    /// whole-module exclusivity map is the seed); method bodies are
    /// residents but their edits ride the cold path (a method edit is a
    /// declaration-context change, not a pure free-function body edit).
    #[allow(clippy::too_many_arguments)]
    pub fn warm_edit(
        &mut self,
        fn_name: &str,
        edited_src: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        // PLAN-904: the collections channel binds its manifest to the
        // REPLACED FULL SOURCE — a fragment-only edit cannot establish
        // that identity, so the ranged authorization is the collections
        // warm contract (a fragment-only request is a NAMED boundary, not
        // a silent stale binding).
        if self.family() == Family::Collections {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                "collections warm edits take the ranged authorization (warm_edit_ranged): the manifest binds the replaced full source, which a fragment alone cannot establish",
            )));
        }
        self.warm_edit_core(fn_name, edited_src, entry, read_only)
    }

    /// PLAN-904 (§5.2): the base-bound ranged warm edit. The candidate
    /// carries the EDIT AUTHORIZATION — the current full source snapshot
    /// (`base_src`, checked against the session's bound hash), a UTF-8
    /// `range` covering exactly the target function's declaration, and
    /// the candidate `fragment`. The manifest binds sha256 of the
    /// REPLACED full source (never a fragment-only hash).
    pub fn warm_edit_ranged(
        &mut self,
        fn_name: &str,
        base_src: &str,
        range: (usize, usize),
        fragment: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        // PLAN-921: the scalar family ALSO takes the ranged
        // authorization — it is a strict superset of the fragment-only
        // path (base + range + single-decl validated) and it is what the
        // source-bound anchor chain rebinding needs (the C channel rides
        // it exclusively). The record-text family keeps its frozen
        // fragment path.
        if !matches!(self.family(), Family::Collections | Family::Scalar) {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                "ranged authorization is the collections/scalar warm contract; the record-text family keeps its frozen fragment path",
            )));
        }
        // the 828 close contract FIRST: a closed environment rejects every
        // warm edit before any lookup
        if let Some(env) = self.env.as_ref() {
            if env.closed() {
                return Err(A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.prepare",
                    "source.closed: the checked environment is closed; warm edits are a rejection",
                )));
            }
        }
        // 1) base binding: the caller's snapshot must be the CURRENT
        //    published source (a stale base is a named rejection)
        let cur = self.source_sha256.clone().ok_or_else(|| {
            A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.session",
                "cold init required before warm edit",
            ))
        })?;
        let base_sha = sha256_hex(base_src.as_bytes());
        if cur != base_sha {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.stale-base",
                format!(
                    "base source sha256 {base_sha} does not bind the session's current source {cur}"
                ),
            )));
        }
        // PLAN-921: a session restored without an env rebuilds it from
        // the bound base source NOW (before the next-source
        // rebinding below switches the binding forward)
        self.ensure_env(base_src)?;
        // 2) range validation: in-bounds, whole UTF-8 scalars, and it
        //    covers EXACTLY one function declaration — the target's
        let (start, end) = range;
        if start > end || end > base_src.len() {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                format!(
                    "range {start}..{end} escapes the base source (len {})",
                    base_src.len()
                ),
            )));
        }
        if !base_src.is_char_boundary(start) || !base_src.is_char_boundary(end) {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                "range cuts a multi-byte UTF-8 scalar",
            )));
        }
        let replaced = &base_src[start..end];
        let old_item =
            parse_function_fragment(replaced, 0..replaced.len(), self.profile).map_err(|d| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.range",
                    format!(
                        "range does not cover exactly one function declaration: {}",
                        d.message
                    ),
                ))
            })?;
        if old_item.name.name != fn_name {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                format!(
                    "range covers `{}`, expected `{}`",
                    old_item.name.name, fn_name
                ),
            )));
        }
        // 3) the candidate fragment is a single same-named declaration
        let frag_item = parse_function_fragment(fragment, 0..fragment.len(), self.profile)
            .map_err(|d| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.parse",
                    format!("fragment: {}", d.message),
                ))
            })?;
        if frag_item.name.name != fn_name {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.parse",
                format!(
                    "edited fragment is `{}`, expected `{}`",
                    frag_item.name.name, fn_name
                ),
            )));
        }
        // 4) the replaced full source identity binds the NEXT manifest
        let full_after = format!("{}{}{}", &base_src[..start], fragment, &base_src[end..]);
        let next_sha = sha256_hex(full_after.as_bytes());
        self.source_sha256 = Some(next_sha.clone());
        // 5) the ordinary warm machinery (prepare → verify → produce →
        //    swap → assemble) runs on the fragment; the manifest built
        //    under it carries the replaced-full-source identity
        // PLAN-921: a session restored without an env rebuilds it from
        // the bound base now (the ranged path carries the base source)
        let assembly = self.warm_edit_core(fn_name, fragment, entry, read_only)?;
        // PLAN-921 T-06: rebind the source-bound anchor to the NEXT full
        // source (per-function origins shift past the edit; the edited
        // function re-binds its slice sha + sem_fp). The anchor publishes
        // only on scalar (the anchored channel) and non-read-only edits.
        if self.family() == Family::Scalar && !read_only {
            let full_after = format!("{}{}{}", &base_src[..start], fragment, &base_src[end..]);
            let delta = fragment.len() as i64 - (end - start) as i64;
            let prev = self.last_anchor.clone();
            if let Some(prev) = prev {
                let mut fns: Vec<anchor::AnchorFn> = Vec::with_capacity(prev.functions.len());
                for mut af in prev.functions {
                    if af.decl == fn_name {
                        af.origin = (start as u32, (start + fragment.len()) as u32);
                        af.slice_sha256 = sha256_hex(fragment.as_bytes());
                    } else if af.origin.0 as i64 >= end as i64 {
                        let s0 = (af.origin.0 as i64 + delta).max(0) as u32;
                        let s1 = (af.origin.1 as i64 + delta).max(0) as u32;
                        af.origin = (s0, s1);
                    }
                    // re-verify + re-bind the slice against the NEXT
                    // source (the binding never trusts the shift alone)
                    let slice = full_after
                        .get(af.origin.0 as usize..af.origin.1 as usize)
                        .ok_or_else(|| {
                            A2xRejectOrResource::Reject(A2xReject::new(
                                "a2x.range",
                                "shifted origin escapes the next source",
                            ))
                        })?;
                    af.slice_sha256 = sha256_hex(slice.as_bytes());
                    // the edited function's NEW sem_fp rides the resident
                    // set (the unchanged keep theirs)
                    if af.decl == fn_name {
                        if let Some(r) = self
                            .resident
                            .get(&(self.current_backend().to_string(), af.decl.clone()))
                        {
                            af.sem_fp = r.sem_fp.clone();
                        }
                    }
                    fns.push(af);
                }
                let fileset_receipt: Vec<(String, String)> = assembly
                    .files
                    .iter()
                    .map(|(p, c)| (p.clone(), sha256_hex(c.as_bytes())))
                    .collect();
                let a = anchor::Anchor {
                    schema: anchor::ANCHOR_SCHEMA.to_string(),
                    owner: self.owner.clone(),
                    profile: self.profile_label().to_string(),
                    backend: self.current_backend().to_string(),
                    entry: entry.to_string(),
                    source_sha256: next_sha.clone(),
                    input_hash: prev.input_hash.clone(),
                    hir_schema: prev.hir_schema.clone(),
                    caps: self.caps.clone(),
                    runtime_sha256: prev.runtime_sha256.clone(),
                    runtime_text: prev.runtime_text.clone(),
                    functions: fns,
                    fileset: fileset_receipt,
                };
                let backend = self.current_backend().to_string();
                let edges: Vec<(String, RefKind)> = a
                    .functions
                    .iter()
                    .filter_map(|af| {
                        self.resident_ids
                            .get(&(backend.clone(), af.decl.clone()))
                            .map(|id| (id.clone(), RefKind::RequiredData))
                    })
                    .collect();
                let id = self.publish_anchor(&a, edges)?;
                self.last_anchor = Some(a);
                self.last_anchor_id = Some(id);
            }
        }
        Ok(assembly)
    }

    /// PLAN-921 T-06: PURE source-bound restore (the new-process entry).
    ///
    /// The caller supplies the COMPLETE source and its declaration
    /// context (profile/caps/entry); the anchor is looked up by the
    /// source sha on the CURRENT target, decoded STRICTLY, and every
    /// claim is re-verified against the caller's source (per-function
    /// slice hashes) and the store payloads (semantic fields, fragment
    /// texts, the assembled file-set bytes vs the receipt). NOTHING in
    /// the store is trusted blind; NOTHING is produced (P0: no
    /// lower/print/persist); the verification work is counted (R>0).
    /// On any mismatch: a NAMED rejection, and NOTHING is installed.
    /// A missing anchor is a rejection (a cold fallback would defeat the
    /// P0 contract), as is a non-scalar family (its file set needs
    /// view-derived prelude inputs a restore cannot re-derive).
    pub fn restore_source_bound(
        &mut self,
        src: &str,
        profile: SourceProfile,
        caps: &[String],
        entry: &str,
    ) -> Result<Assembly, A2xRejectOrResource> {
        // a restore must not disturb session state on failure: verify
        // into locals first, install last.
        let family = match profile {
            SourceProfile::Record | SourceProfile::Text | SourceProfile::TextRecord => {
                Family::RecordText
            }
            SourceProfile::Collections => Family::Collections,
            _ => Family::Scalar,
        };
        if family != Family::Scalar {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.restore",
                "pure restore serves the scalar channel only (view-derived file sets cannot re-assemble blind)",
            )));
        }
        let src_sha = sha256_hex(src.as_bytes());
        self.stats.restore_lookup += 1;
        let key = QueryKey::new(
            &LAYER_SOURCE_ANCHOR,
            vec![
                ("owner".into(), self.owner.clone()),
                ("profile".into(), profile_label_of(profile).into()),
                ("source_sha256".into(), src_sha.clone()),
                ("backend".into(), self.current_backend().to_string()),
                ("entry".into(), entry.to_string()),
            ],
        );
        let hit = self
            .store
            .get_artifact(LAYER_SOURCE_ANCHOR.id, &key)
            .map_err(|e| {
                A2xRejectOrResource::Resource(ResourceReject {
                    code: "a2x.resource",
                    detail: format!("anchor lookup: {:?}", e),
                })
            })?
            .ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.restore.no-anchor",
                    format!(
                        "no source-bound anchor for source sha256 {} on backend {} (a cold fallback would defeat the P0 contract)",
                        &src_sha[..16],
                        self.current_backend()
                    ),
                ))
            })?;
        let anchor_text = String::from_utf8(hit.payload).map_err(|_| {
            A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.restore.bad-anchor",
                "anchor payload not utf8",
            ))
        })?;
        let a = anchor::Anchor::decode(&anchor_text).ok_or_else(|| {
            A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.restore.bad-anchor",
                "anchor failed strict decode (schema/fields/counts)",
            ))
        })?;
        // caller context must match the anchored context exactly
        if a.profile != profile_label_of(profile)
            || a.caps != caps
            || a.entry != entry
            || a.backend != self.current_backend()
            || a.source_sha256 != src_sha
        {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.restore.context",
                "anchor context (profile/caps/entry/backend/source) does not match the caller's",
            )));
        }
        // per-function SOURCE binding: slice existence + byte hash
        for f in &a.functions {
            self.stats.restore_binding_checks += 1;
            let (s0, s1) = (f.origin.0 as usize, f.origin.1 as usize);
            let slice = src.get(s0..s1).ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.restore.binding",
                    format!(
                        "origin {}..{} escapes the caller's source for `{}`",
                        s0, s1, f.decl
                    ),
                ))
            })?;
            let slice_sha = sha256_hex(slice.as_bytes());
            if slice_sha != f.slice_sha256 {
                return Err(A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.restore.binding",
                    format!(
                        "source slice sha256 {} != anchored {} for `{}` — the source differs from the anchored state",
                        &slice_sha[..16],
                        &f.slice_sha256[..16],
                        f.decl
                    ),
                )));
            }
        }
        if !a.functions.iter().any(|f| f.decl == entry) {
            return Err(A2xRejectOrResource::Reject(A2xReject::entry(
                "entry missing from the anchored function set",
            )));
        }
        // per-function payload re-verification: semantic fields + the
        // fragment text under the anchored identities
        let mut fragments: Vec<(usize, ResidentFragment)> = Vec::with_capacity(a.functions.len());
        for f in &a.functions {
            // semantic layer: key reconstructed from ANCHOR data only
            let sem_key = QueryKey::new(
                self.layer_semantic(),
                vec![
                    ("owner".into(), self.owner.clone()),
                    ("decl".into(), f.decl.clone()),
                    ("profile".into(), a.profile.clone()),
                    ("hir_schema".into(), a.hir_schema.clone()),
                    ("signature".into(), f.signature.clone()),
                    ("sem_fp".into(), f.sem_fp.clone()),
                    ("caps".into(), a.caps.join(",")),
                    ("pass".into(), pass_version_of(family).into()),
                ],
            );
            if let Ok(Some(sem)) = self.store.get_artifact(self.layer_semantic().id, &sem_key) {
                self.stats.restore_payload_verifies += 1;
                let text = String::from_utf8_lossy(&sem.payload).into_owned();
                let want_schema = format!("\"schema\":\"{}\"", self.layer_semantic().schema_id);
                let want_decl = format!("\"decl\":\"{}\"", f.decl);
                let want_fp = format!("\"sem_fp\":\"{}\"", f.sem_fp);
                if !text.contains(&want_schema)
                    || !text.contains(&want_decl)
                    || !text.contains(&want_fp)
                {
                    return Err(A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.restore.payload",
                        format!(
                            "semantic payload fields disagree with the anchor for `{}`",
                            f.decl
                        ),
                    )));
                }
            }
            // fragment layer: the anchored target identity + runtime hash
            let mut inputs = vec![
                ("owner".into(), self.owner.clone()),
                ("decl".into(), f.decl.clone()),
                ("sem_fp".into(), f.sem_fp.clone()),
            ];
            inputs.extend(self.provider.fragment_key_inputs(&a.runtime_sha256));
            let frag_key = QueryKey::new(self.layer_fragment(), inputs);
            let fh = self
                .store
                .get_artifact(self.layer_fragment().id, &frag_key)
                .map_err(|e| {
                    A2xRejectOrResource::Resource(ResourceReject {
                        code: "a2x.resource",
                        detail: format!("fragment lookup: {:?}", e),
                    })
                })?
                .ok_or_else(|| {
                    A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.restore.fragment",
                        format!(
                            "anchored fragment missing for `{}` (collected by GC?) — re-cold to republish",
                            f.decl
                        ),
                    ))
                })?;
            let text = String::from_utf8(fh.payload).map_err(|_| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.restore.fragment",
                    format!("fragment payload not utf8 for `{}`", f.decl),
                ))
            })?;
            self.stats.restore_payload_verifies += 1;
            let frag_artifact_id = fh.artifact_id.clone();
            fragments.push((
                f.def_index as usize,
                ResidentFragment {
                    source_name: f.decl.clone(),
                    def_index: f.def_index as usize,
                    sem_fp: f.sem_fp.clone(),
                    text,
                },
            ));
            self.resident_ids.insert(
                (self.current_backend().to_string(), f.decl.clone()),
                frag_artifact_id,
            );
        }
        fragments.sort_by_key(|(d, _)| *d);
        // the anchored runtime TEXT is re-verified by content hash (a
        // restore has no view, so the provider cannot re-derive marks)
        self.stats.restore_payload_verifies += 1;
        if sha256_hex(a.runtime_text.as_bytes()) != a.runtime_sha256 {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.restore.bad-anchor",
                "anchored runtime text hash disagrees with the anchored runtime sha256",
            )));
        }
        let runtime = a.runtime_text.clone();
        // the anchor's origins ride the request so the reassembled
        // manifest is byte-identical to the cold one
        let anchor_origins: Vec<(usize, (u32, u32))> = a
            .functions
            .iter()
            .map(|f| (f.def_index as usize, f.origin))
            .collect();
        // re-assemble through the provider (scalar: no view, no prelude
        // seed) and verify EVERY file byte against the anchor receipt
        let frags: Vec<FragmentText<'_>> = fragments
            .iter()
            .map(|(_, r)| FragmentText {
                def_index: r.def_index,
                source_name: r.source_name.as_str(),
                sem_fp: r.sem_fp.as_str(),
                text: r.text.as_str(),
            })
            .collect();
        let fileset = self
            .provider
            .assemble_fileset(FilesetRequest {
                entry,
                input_hash: &a.input_hash,
                family,
                fragments: &frags,
                view: None,
                runtime: &runtime,
                prelude_seed: None,
                origins: Some(&anchor_origins),
                source_sha256: Some(&a.source_sha256),
            })
            .map_err(A2xRejectOrResource::Reject)?;
        for (path, want_sha) in &a.fileset {
            self.stats.restore_fileset_verifies += 1;
            let got = fileset
                .files
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, c)| sha256_hex(c.as_bytes()))
                .unwrap_or_default();
            if got != *want_sha {
                return Err(A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.restore.fileset",
                    format!(
                        "reassembled file `{}` sha256 {} != anchored receipt {} — store payloads drifted",
                        path,
                        &got[..got.len().min(16)],
                        &want_sha[..want_sha.len().min(16)],
                    ),
                )));
            }
        }
        if fileset.files.len() != a.fileset.len() {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.restore.fileset",
                "reassembled file set size differs from the anchored receipt",
            )));
        }
        // ALL verified: install (residents under the current backend,
        // context fields, NO env — a later warm edit rebuilds it on
        // demand, counted as env_rebuilds)
        let backend = self.current_backend().to_string();
        self.resident.retain(|(b, _), _| b != &backend);
        for (_, r) in &fragments {
            self.resident
                .insert((backend.clone(), r.source_name.clone()), r.clone());
        }
        self.profile = profile;
        self.caps = caps.to_vec();
        self.source_sha256 = Some(src_sha);
        self.env = None;
        // the restored anchor becomes the session's rebinding base (the
        // next ranged warm edit rebinds from it)
        self.last_anchor = Some(a);
        self.last_anchor_id = Some(hit.artifact_id.clone());
        Ok(Assembly {
            files: fileset.files,
            manifest: fileset.manifest,
        })
    }

    /// PLAN-921 T-06: rebuild the retained environment from the bound
    /// source so warm edits work after a pure restore (M-class
    /// management: one whole-source parse+resolve, counted; never
    /// production). Called lazily by the warm paths.
    fn ensure_env(&mut self, base_src: &str) -> Result<(), A2xRejectOrResource> {
        if self.env.is_some() {
            return Ok(());
        }
        let bound = self.source_sha256.clone().ok_or_else(|| {
            A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.session",
                "cold init required before warm edit",
            ))
        })?;
        if sha256_hex(base_src.as_bytes()) != bound {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.stale-base",
                "base source does not bind the session's restored source",
            )));
        }
        let profile = self.profile;
        self.stats.parse += 1;
        let program = parse_program_with(base_src, profile)
            .map_err(|d| A2xRejectOrResource::Reject(A2xReject::new("a2x.parse", d.message)))?;
        self.stats.parse += 1;
        let env = resolve_env(&program, &self.caps.clone(), profile).map_err(|ds| {
            A2xRejectOrResource::Reject(A2xReject::new("a2x.resolve", render_diag(&ds)))
        })?;
        self.env = Some(env);
        self.stats.env_rebuilds += 1;
        Ok(())
    }

    fn warm_edit_core(
        &mut self,
        fn_name: &str,
        edited_src: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        self.warm_edit_targeted(&WarmTarget::Free { name: fn_name }, edited_src, entry, read_only)
    }

    /// PLAN-925 (§5.4): WARM body edit of ONE method — the narrow method
    /// fragment (`static fn ..` / `fn ..` with caller-provided owner
    /// binding) through the same single-body credential chain. The graft
    /// keeps the retained receiver/owner shape (PLAN-881); the method def
    /// of the single-method document is `d_f{slot}_{name}`.
    pub fn warm_edit_method(
        &mut self,
        owner: &str,
        fn_name: &str,
        edited_src: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        if self.family() == Family::Collections {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                "collections warm edits take the ranged authorization (warm_edit_ranged)",
            )));
        }
        self.warm_edit_targeted(
            &WarmTarget::Method {
                owner,
                name: fn_name,
            },
            edited_src,
            entry,
            read_only,
        )
    }

    fn warm_edit_targeted(
        &mut self,
        target: &WarmTarget<'_>,
        edited_src: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let fn_name = target.name();
        let profile = self.profile;
        let backend = self.current_backend().to_string();
        // produce the candidate fragment text + identity while the
        // candidate is alive (SourceParts is not Clone — borrowed use only)
        let (frag_def, frag_name, frag_sem_fp, frag_text, input_hash, runtime, prelude_seed) = {
            let env = self.env.as_mut().ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.session",
                    "cold init required before warm edit",
                ))
            })?;
            self.stats.parse += 1;
            // FRAGMENT parse only (§5.4: a warm pure body edit parses just
            // the changed declaration — never the whole source; whole
            // parses are the cold path's business and its cost).
            // PLAN-925: method edits take the narrow method fragment with
            // the caller-provided owner binding context.
            let item = match target {
                WarmTarget::Free { .. } => {
                    parse_function_fragment(edited_src, 0..edited_src.len(), profile)
                }
                WarmTarget::Method { owner, .. } => {
                    parse_method_fragment(edited_src, 0..edited_src.len(), profile, owner)
                }
            }
            .map_err(|d| A2xRejectOrResource::Reject(A2xReject::new("a2x.parse", d.message)))?;
            if item.name.name != fn_name {
                return Err(A2xRejectOrResource::Reject(A2xReject::entry(format!(
                    "edited fragment is `{}`, expected `{}`",
                    item.name.name, fn_name
                ))));
            }
            // the 828 close contract FIRST: a closed environment rejects
            // every warm edit before any lookup (close also clears the fn
            // table, so ordering matters for the rejection's identity)
            if env.closed() {
                return Err(A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.prepare",
                    "source.closed: the checked environment is closed; warm edits are a rejection",
                )));
            }
            let slot = match target {
                WarmTarget::Free { .. } => env_slot(env, fn_name),
                WarmTarget::Method { owner, .. } => {
                    ace_core::source::env_method_index(env, owner, fn_name)
                }
            }
            .ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::entry(format!(
                    "fn `{}` not resident in env",
                    fn_name
                )))
            })?;
            self.stats.prepare_candidate += 1;
            let cand =
                prepare_function_candidate(env, &item, slot, edited_src, "a2x").map_err(|ds| {
                    A2xRejectOrResource::Reject(A2xReject::new("a2x.prepare", render_diag(&ds)))
                })?;
            let parts = cand.parts();
            self.stats.verify += 1;
            // PLAN-902: family-dispatched single-body verification — text
            // domains carry their resource plan through the credential
            // (seeded by the RETAINED whole-module exclusivity map).
            let cred = match self.profile {
                SourceProfile::Text | SourceProfile::TextRecord => {
                    let seed = self
                        .retained_excl
                        .clone()
                        .unwrap_or_else(|| param_exclusions_fixpoint(&parts.bundle));
                    verify_text_function(
                        &parts.bundle,
                        def_index_in(&parts.bundle, target, slot),
                        &seed,
                    )
                }
                _ => verify_function_ref(&parts.bundle, def_index_in(&parts.bundle, target, slot)),
            }
            .map_err(|ds| {
                A2xRejectOrResource::Reject(A2xReject::new("a2x.verify", render_diag(&ds)))
            })?;
            let view =
                SemanticView::from_parts(parts, &cred).map_err(A2xRejectOrResource::Reject)?;
            let family = self.family();
            let runtime = self
                .provider
                .runtime_source(&view, family)
                .map_err(A2xRejectOrResource::Reject)?;
            let runtime_sha = sha256_hex(runtime.as_bytes());
            let f = view.functions().iter().next().ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new("a2x.internal", "empty fragment view"))
            })?;
            let sem_fp = semantic_fingerprint(&view, f).map_err(A2xRejectOrResource::Reject)?;
            // reuse by store hit / resident; else produce now
            let text = if let Some(r) = self
                .resident
                .get(&(backend.clone(), f.source_name.to_string()))
            {
                if r.sem_fp == sem_fp {
                    self.stats.fragments_reused_resident += 1;
                    r.text.clone()
                } else {
                    self.produce_fragment(&view, f, &sem_fp, &runtime_sha, read_only)?
                        .text
                }
            } else {
                self.produce_fragment(&view, f, &sem_fp, &runtime_sha, read_only)?
                    .text
            };
            // the rt/coll prelude needs the (record-carrying) fragment view
            // AFTER the borrow ends — capture the provider-opaque seed now
            let prelude_seed = if family == Family::RecordText || family == Family::Collections {
                self.provider
                    .capture_prelude_seed(&view, family)
                    .map_err(A2xRejectOrResource::Reject)?
            } else {
                None
            };
            (
                f.def_index,
                f.source_name.to_string(),
                sem_fp,
                text,
                parts.input_hash.clone(),
                runtime,
                prelude_seed,
            )
        };
        let def_index = frag_def;
        let fragment = ResidentFragment {
            source_name: frag_name,
            def_index,
            sem_fp: frag_sem_fp,
            text: frag_text,
        };

        // swap the edited fragment into the CURRENT target's resident set
        // (other targets' residents are untouched) and assemble
        let mut frags: Vec<ResidentFragment> = self
            .resident
            .iter()
            .filter(|((b, _), _)| b == &backend)
            .map(|(_, r)| r.clone())
            .collect();
        frags.retain(|r| r.source_name != fragment.source_name);
        frags.push(fragment);
        frags.sort_by_key(|r| r.def_index);
        self.resident.retain(|(b, _), _| b != &backend);
        self.resident.extend(
            frags
                .into_iter()
                .map(|r| ((backend.clone(), r.source_name.clone()), r)),
        );

        // assemble from residents (no extra production; manifest built
        // here) — the prelude re-derives from the captured seed
        let mut residents: Vec<(usize, ResidentFragment)> = self
            .resident
            .iter()
            .filter(|((b, _), _)| b == &backend)
            .map(|(_, r)| (r.def_index, r.clone()))
            .collect();
        // declaration order (BTreeMap iteration is name-alphabetical;
        // multi-function modules must assemble in def order — the cold
        // path's byte shape)
        residents.sort_by_key(|(d, _)| *d);
        let family = self.family();
        let frags: Vec<FragmentText<'_>> = residents
            .iter()
            .map(|(_, r)| FragmentText {
                def_index: r.def_index,
                source_name: r.source_name.as_str(),
                sem_fp: r.sem_fp.as_str(),
                text: r.text.as_str(),
            })
            .collect();
        let fileset = self
            .provider
            .assemble_fileset(FilesetRequest {
                entry,
                input_hash: &input_hash,
                family,
                fragments: &frags,
                view: None,
                runtime: &runtime,
                prelude_seed: prelude_seed.as_deref(),
                origins: None,
                source_sha256: self.source_sha256.as_deref(),
            })
            .map_err(A2xRejectOrResource::Reject)?;
        Ok(Assembly {
            files: fileset.files,
            manifest: fileset.manifest,
        })
    }

    fn produce_fragment(
        &mut self,
        view: &SemanticView<'_>,
        f: &FnFacts<'_>,
        sem_fp: &str,
        runtime_sha: &str,
        read_only: bool,
    ) -> Result<ResidentFragment, A2xRejectOrResource> {
        let frag_key = self.fragment_key(f, sem_fp, runtime_sha);
        if let Ok(Some(hit)) = self.store.get_artifact(self.layer_fragment().id, &frag_key) {
            if let Ok(text) = String::from_utf8(hit.payload) {
                self.stats.store_hit_fragment += 1;
                return Ok(ResidentFragment {
                    source_name: f.source_name.to_string(),
                    def_index: f.def_index,
                    sem_fp: sem_fp.to_string(),
                    text,
                });
            }
        }
        self.stats.lower += 1;
        self.stats.print += 1;
        let texts = self
            .provider
            .produce_fragments(view, &[f], self.family())
            .map_err(A2xRejectOrResource::Reject)?;
        let text = texts.into_iter().next().ok_or_else(|| {
            A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.internal",
                format!("fragment for `{}` missing after lowering", f.source_name),
            ))
        })?;
        if !read_only {
            let sig_input = self.signature_input(view, f);
            let sem_key = self.semantic_key(f, sem_fp, view.domain.schema(), &sig_input);
            let sem_payload = format!(
                "{{\"schema\":\"{}\",\"decl\":\"{}\",\"signature\":\"{}\",\"sem_fp\":\"{}\"}}",
                self.layer_semantic().schema_id,
                f.source_name,
                view.canonical_signature(f),
                sem_fp
            );
            let sem_id = self.quota_put(
                self.layer_semantic().id,
                sem_key,
                ChunkKind::Hir,
                sem_payload.as_bytes(),
                vec![],
            )?;
            self.quota_put(
                self.layer_fragment().id,
                frag_key,
                ChunkKind::Object,
                text.as_bytes(),
                vec![(sem_id, RefKind::Provenance)],
            )?;
        }
        Ok(ResidentFragment {
            source_name: f.source_name.to_string(),
            def_index: f.def_index,
            sem_fp: sem_fp.to_string(),
            text,
        })
    }

    /// RESTART re-admission: a fresh session over the same store must HIT
    /// every previously persisted fragment (§5.4 "重建后下一restart必须真hit").
    pub fn restart_assemble(
        &mut self,
        src: &str,
        scalar: bool,
        caps: &[String],
        entry: &str,
    ) -> Result<Assembly, A2xRejectOrResource> {
        // a restart drops the CURRENT target's residents and rebuilds
        // purely from store hits (other targets' residents stay valid)
        let backend = self.current_backend().to_string();
        self.resident.retain(|(b, _), _| b != &backend);
        self.cold_assemble(src, scalar, caps, entry, false)
    }

    /// PLAN-902: the restart entry for explicit profiles (the record-text
    /// channel restarts through the same store-hit rebuild).
    pub fn restart_assemble_profile(
        &mut self,
        src: &str,
        profile: SourceProfile,
        caps: &[String],
        entry: &str,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let backend = self.current_backend().to_string();
        self.resident.retain(|(b, _), _| b != &backend);
        self.cold_assemble_profile(src, profile, caps, entry, false)
    }

    /// Close the retained checked environment (the 828 close contract):
    /// every later prepare/recheck/commit is a controlled `source.closed`
    /// rejection; the resident set and the store keep serving assemblies.
    pub fn close_env(&mut self) {
        if let Some(env) = self.env.as_mut() {
            env.close();
        }
    }

    pub fn env_closed(&self) -> bool {
        self.env.as_ref().map(|e| e.closed()).unwrap_or(true)
    }

    /// Lease the CURRENT assembly against GC. PLAN-921: when a
    /// source-bound anchor exists it is the leased artifact — its
    /// required edges keep the whole fragment set alive with it.
    pub fn lease_current(&self, purpose: &str) -> Result<ace_artifacts::store::Lease, String> {
        self.store
            .acquire_lease(purpose, 3600, self.last_anchor_id.as_deref())
            .map_err(|e| format!("lease: {:?}", e))
    }

    /// PLAN-921: the last published source-bound anchor (observable for
    /// tests/receipts; None before a scalar cold init).
    pub fn last_anchor(&self) -> Option<&Anchor> {
        self.last_anchor.as_ref()
    }

    /// Run the namespace GC (783 default config) and return the receipt.
    pub fn gc(&self) -> Result<String, String> {
        let receipt =
            run_gc(&self.store, gc_config_default()).map_err(|e| format!("gc: {:?}", e))?;
        Ok(format!("{:?}", receipt))
    }
}

/// The error surface of session operations: named A2X rejections or
/// controlled resource failures (never each other, §5.6).
#[derive(Debug)]
pub enum A2xRejectOrResource {
    Reject(A2xReject),
    Resource(ResourceReject),
}

impl From<A2xReject> for A2xRejectOrResource {
    fn from(r: A2xReject) -> Self {
        A2xRejectOrResource::Reject(r)
    }
}

impl From<ResourceReject> for A2xRejectOrResource {
    fn from(r: ResourceReject) -> Self {
        A2xRejectOrResource::Resource(r)
    }
}

fn render_diag(ds: &[ace_core::atom_text::Diagnostic]) -> String {
    ds.iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn env_slot(env: &ace_core::source::resolve::CheckedEnv, name: &str) -> Option<usize> {
    // PLAN-925: the FREE entry — a same-named method (which shares the
    // fn table) must never be grafted as the free function (the 881
    // receiver/owner guard would reject it, but the lookup must not even
    // aim there)
    ace_core::source::env_free_fn_index(env, name)
}

/// Def lookup of a single-body document by TARGET SHAPE: a free function
/// is `d_{name}`; a METHOD (PLAN-925) is `d_f{slot}_{name}`. The shapes
/// are never tried in sequence — a same-named free-function scaffold must
/// not be verified in a method's place (or vice versa).
fn def_index_in(bundle: &ace_core::hir::Bundle, target: &WarmTarget<'_>, slot: usize) -> usize {
    let want = match target {
        WarmTarget::Free { name } => format!("d_{}", name),
        WarmTarget::Method { name, .. } => format!("d_f{}_{}", slot, name),
    };
    bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == want)
        .unwrap_or_else(|| panic!("def {} missing", want))
}

/// PLAN-925: the warm edit's target shape.
enum WarmTarget<'a> {
    Free { name: &'a str },
    Method { owner: &'a str, name: &'a str },
}

impl WarmTarget<'_> {
    fn name(&self) -> &str {
        match self {
            WarmTarget::Free { name } | WarmTarget::Method { name, .. } => name,
        }
    }
}

fn profile_label_of(profile: SourceProfile) -> &'static str {
    match profile {
        SourceProfile::Core => "core",
        SourceProfile::Scalar => "scalar",
        SourceProfile::Record => "record",
        SourceProfile::Text => "text",
        SourceProfile::TextRecord => "text-record",
        _ => "unsupported",
    }
}

fn pass_version_of(family: Family) -> &'static str {
    match family {
        Family::Scalar => PASS_PREPARE,
        Family::RecordText => PASS_PREPARE_RT,
        Family::Collections => PASS_PREPARE_COLL,
    }
}

fn static_layer(id: &str) -> &'static str {
    match id {
        "AXS1" => LAYER_SEMANTIC.id,
        "AXR1" => LAYER_FRAGMENT.id,
        "AXM1" => LAYER_MANIFEST.id,
        "AXS2" => LAYER_SEMANTIC_RT.id,
        "AXR2" => LAYER_FRAGMENT_RT.id,
        "AXS3" => LAYER_SEMANTIC_COLL.id,
        "AXR3" => LAYER_FRAGMENT_COLL.id,
        "AXC1" => LAYER_FRAGMENT_C.id,
        "AXC2" => LAYER_FRAGMENT_C_RT.id,
        "AXP1" => LAYER_SOURCE_ANCHOR.id,
        other => panic!("unknown layer {}", other),
    }
}
