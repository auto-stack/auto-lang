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
use ace_a2x_core::{semantic_fingerprint, A2xReject, FnFacts, SemanticView};
use ace_a2x_rust::{lower_functions, TargetProfile};
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::{QueryKey, RefKind};
use ace_artifacts::schema::LayerSpec;
use ace_artifacts::store::{gc_config_default, run_gc, ManagedStore, PutRequest, SpaceReport};
use ace_core::hir::HirDomain;
use ace_core::source::{
    compile_collections_source, compile_record_source, compile_scalar_source, compile_source,
    compile_text_record_source, compile_text_source, parse_function_fragment, parse_program_with,
    prepare_function_candidate, resolve_env, SourceProfile,
};
use ace_core::text_resources::{param_exclusions_fixpoint, ParamExclusions};
use ace_core::verify::{verify_function_ref, verify_text_function};
use std::collections::BTreeMap;
use std::path::Path;

pub const LAYER_SEMANTIC: LayerSpec = LayerSpec {
    id: "AXS1",
    name: "a2x-function-semantic",
    schema_id: "a2892/axs-fn-sem/v1",
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

/// The channel family of the session's profile (layer-set selection).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Scalar,
    RecordText,
    /// PLAN-904: the collections (List/Variant) channel — AXS3/AXR3
    /// namespaces, the collections runtime and fileset.
    Collections,
}

/// The A2X session: cold init, warm per-function edits with RESIDENT reuse,
/// layered persistence and quota gating over one `ManagedStore`.
pub struct A2xSession {
    store: ManagedStore,
    project: String,
    owner: String,
    quota_bytes: u64,
    pub stats: ProducerStats,
    /// source name → resident fragment (the last PUBLISHED semantics per
    /// fn). Keyed by source name — def indices are NOT stable across
    /// whole-module and single-body bundles (F-828-R1). Methods are
    /// owner-qualified residents (`Counter_read`).
    resident: BTreeMap<String, ResidentFragment>,
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
}

impl A2xSession {
    /// Open a session on a (new or existing) store root. `quota_bytes` caps
    /// the NAMESPACE total (blocks+db+staging+trash+locks+logs) for puts
    /// from THIS session — a put that would cross it is a controlled
    /// `a2x.resource` rejection leaving prior state intact.
    pub fn open(root: &Path, project: &str, owner: &str, quota_bytes: u64) -> Result<Self, String> {
        let store = ManagedStore::open(root).map_err(|e| format!("open store: {:?}", e))?;
        Ok(A2xSession {
            store,
            project: project.to_string(),
            owner: owner.to_string(),
            quota_bytes,
            stats: ProducerStats::default(),
            resident: BTreeMap::new(),
            env: None,
            retained_excl: None,
            profile: SourceProfile::Core,
            caps: vec![],
            source_sha256: None,
        })
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
        match self.family() {
            Family::Scalar => &LAYER_FRAGMENT,
            Family::RecordText => &LAYER_FRAGMENT_RT,
            Family::Collections => &LAYER_FRAGMENT_COLL,
        }
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

    /// Target-layer key: semantic key inputs + target fields (§5.4).
    fn fragment_key(&self, f: &FnFacts<'_>, sem_fp: &str, runtime_sha: &str) -> QueryKey {
        let t = TargetProfile::v1();
        QueryKey::new(
            self.layer_fragment(),
            vec![
                ("owner".into(), self.owner.clone()),
                ("decl".into(), f.source_name.to_string()),
                ("sem_fp".into(), sem_fp.to_string()),
                ("lang".into(), t.lang.into()),
                ("edition".into(), t.edition.into()),
                ("backend".into(), t.backend.into()),
                ("backend_version".into(), t.backend_version.into()),
                ("runtime_sha256".into(), runtime_sha.into()),
                ("print_version".into(), PRINT_VERSION.into()),
            ],
        )
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
        let runtime = fragment_runtime_source(view);
        let runtime_sha = sha256_hex(runtime.as_bytes());
        let mut fragments: Vec<(usize, ResidentFragment)> = Vec::new();
        // ONE whole-module lowering pass memoized for all store misses —
        // lowering per function would re-lower the module O(n) times at
        // load scale (O(n^2) total).
        let mut whole: Option<ace_a2x_rust::RustProgram> = None;

        for f in view.functions() {
            self.stats.verify += 1;
            let sem_fp = semantic_fingerprint(view, f).map_err(A2xRejectOrResource::Reject)?;

            // 1) resident reuse: unchanged function/method → ZERO production
            if let Some(r) = self.resident.get(f.source_name) {
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
                let text = String::from_utf8(hit.payload).map_err(|_| {
                    A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.internal",
                        "fragment payload not utf8",
                    ))
                })?;
                self.stats.store_hit_fragment += 1;
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

            // 3) produce: lower + print (+ persist)
            self.stats.lower += 1;
            self.stats.print += 1;
            let text = if self.family() == Family::Collections {
                let program = ace_a2x_rust::lower_coll::lower_coll_functions(view)
                    .map_err(A2xRejectOrResource::Reject)?;
                let rust_name = rust_fragment_name(f);
                let rust_fn = program
                    .functions
                    .iter()
                    .find(|rf| rf.name == rust_name)
                    .cloned()
                    .ok_or_else(|| {
                        A2xRejectOrResource::Reject(A2xReject::new(
                            "a2x.internal",
                            format!("fragment for `{}` missing after lowering", f.source_name),
                        ))
                    })?;
                print_coll_fragment_text(&program.records, &program.variants, &rust_fn)
            } else {
                if whole.is_none() {
                    whole = Some(lower_functions(view).map_err(A2xRejectOrResource::Reject)?);
                }
                let program = whole.as_ref().unwrap();
                let rust_name = rust_fragment_name(f);
                let rust_fn = program
                    .functions
                    .iter()
                    .find(|rf| rf.name == rust_name)
                    .cloned()
                    .ok_or_else(|| {
                        A2xRejectOrResource::Reject(A2xReject::new(
                            "a2x.internal",
                            format!("fragment for `{}` missing after lowering", f.source_name),
                        ))
                    })?;
                print_fragment_text(whole.as_ref().unwrap().records.as_slice(), &rust_fn)
            };

            if !read_only {
                // semantic layer first
                let sig_input = self.signature_input(view, f);
                let sem_key = self.semantic_key(f, &sem_fp, view.domain.schema(), &sig_input);
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
                let frag_id = self.quota_put(
                    self.layer_fragment().id,
                    frag_key,
                    ChunkKind::Object,
                    text.as_bytes(),
                    vec![(sem_id, RefKind::Provenance)],
                )?;
                let _ = frag_id;
            }
            fragments.push((
                f.def_index,
                ResidentFragment {
                    source_name: f.source_name.to_string(),
                    def_index: f.def_index,
                    sem_fp,
                    text,
                },
            ));
        }

        // publish residents + assemble the file set
        self.resident = fragments
            .iter()
            .map(|(_, r)| (r.source_name.clone(), r.clone()))
            .collect();
        let rt_records = if self.family() == Family::RecordText {
            Some(ace_a2x_rust::lower::lower_records(view).map_err(A2xRejectOrResource::Reject)?)
        } else {
            None
        };
        self.build_fileset(
            entry,
            input_hash,
            &fragments,
            Some(view),
            rt_records.as_deref(),
            runtime,
            None,
        )
    }

    /// Build the full file set from resident fragments (the shared cold and
    /// warm assembly tail). Scalar output is byte-identical to the frozen
    /// 892 assembly; the record-text channel adds the nominal-record and
    /// clone-helper prelude derived from the view + fragment texts.
    #[allow(clippy::too_many_arguments)]
    fn build_fileset(
        &self,
        entry: &str,
        input_hash: &str,
        fragments: &[(usize, ResidentFragment)],
        view: Option<&SemanticView<'_>>,
        rt_records: Option<&[ace_a2x_rust::RustRecord]>,
        runtime: String,
        coll_variants: Option<&[ace_a2x_rust::lower::RustVariant]>,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let mut generated = String::new();
        generated.push_str(&format!(
            "// AUTO-GENERATED by auto-a2x-rust (PLAN-892 AX-A) — session assembly, entry `{}`\n\n",
            entry
        ));
        generated.push_str("use crate::a2x_runtime::*;\n\n");
        if self.family() == Family::RecordText {
            let records = rt_records.ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.internal",
                    "record-text assembly needs its record table",
                ))
            })?;
            let texts: Vec<&str> = fragments.iter().map(|(_, r)| r.text.as_str()).collect();
            generated.push_str(&ace_a2x_rust::print::print_rt_prelude_from(records, &texts));
        }
        if self.family() == Family::Collections {
            // the type tables re-derive from the view (cold) or the
            // session's fragment-view snapshot (warm); free/clone/take
            // helpers and elem impls follow the same tables
            let records = match rt_records {
                Some(r) => r.to_vec(),
                None => match view {
                    Some(v) => ace_a2x_rust::lower::lower_records(v)
                        .map_err(A2xRejectOrResource::Reject)?,
                    None => {
                        return Err(A2xRejectOrResource::Reject(A2xReject::new(
                            "a2x.internal",
                            "collections assembly needs its type tables",
                        )))
                    }
                },
            };
            let variants = match coll_variants {
                Some(vs) => vs.to_vec(),
                None => match view {
                    Some(v) => ace_a2x_rust::lower_coll::lower_coll_variants(v),
                    None => Vec::new(),
                },
            };
            generated.push_str(&ace_a2x_rust::print::print_coll_prelude_from(
                &records, &variants,
            ));
        }
        for (_, r) in fragments {
            generated.push_str(&r.text);
            generated.push('\n');
        }
        if !fragments.iter().any(|(_, r)| r.source_name == entry) {
            return Err(A2xRejectOrResource::Reject(A2xReject::entry(
                "entry missing",
            )));
        }
        let entry_rust = entry_rust_name(entry, self.family());
        let wrapper = format!(
            "// AUTO-GENERATED (PLAN-892 AX-A) — DO NOT EDIT.\nmod a2x_runtime;\nmod generated;\n\nuse a2x_runtime::A2xResult;\n\nfn main() {{\n    let outcome: A2xResult<i32> = generated::{}();\n    match outcome {{\n        Ok(v) => println!(\"value:{{}}\", v),\n        Err(t) => println!(\"trap:{{}}\", t.name()),\n    }}\n}}\n",
            entry_rust
        );
        let files = vec![
            ("main.rs".to_string(), wrapper),
            ("a2x_runtime.rs".to_string(), runtime),
            ("generated.rs".to_string(), generated),
        ];
        let schema = match self.family() {
            Family::Scalar => "auto.a2x.rust.fileset/1",
            Family::RecordText => "auto.a2x.rust.record-text.fileset/1",
            Family::Collections => ace_a2x_rust::COLL_FILESET_SCHEMA,
        };
        let mut manifest = String::new();
        manifest.push_str(&format!("{{\n  \"schema\": \"{}\",\n", schema));
        manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
        if self.family() == Family::Collections {
            // PLAN-904: the manifest binds the REPLACED FULL SOURCE hash
            // (never a fragment-only hash) and the execution-config
            // identity
            let sha = self.source_sha256.clone().unwrap_or_default();
            manifest.push_str(&format!("  \"source_sha256\": \"{}\",
", sha));
            manifest.push_str(&format!(
                "  \"execution_config\": {},
",
                ace_a2x_rust::coll::canonical_execution_config(
                    "debug",
                    16_777_216,
                    None,
                    false,
                    false
                )
            ));
        }
        manifest.push_str(&format!(
            "  \"entry\": \"{}\",\n  \"functions\": [\n",
            entry
        ));
        for (i, (d, r)) in fragments.iter().enumerate() {
            let comma = if i + 1 < fragments.len() { "," } else { "" };
            // origins are included whenever the assembly carries its view
            // (the cold path); warm assemblies list declarations only —
            // frozen per-family behavior
            if let Some(v) = view {
                let origin = v.fn_by_def(*d).map(|f| f.origin).unwrap_or((0, 0));
                manifest.push_str(&format!(
                    "    {{ \"decl\": \"{}\", \"sem_fp\": \"{}\", \"origin\": [{}, {}] }}{}\n",
                    r.source_name, r.sem_fp, origin.0, origin.1, comma
                ));
            } else {
                manifest.push_str(&format!(
                    "    {{ \"decl\": \"{}\", \"sem_fp\": \"{}\" }}{}\n",
                    r.source_name, r.sem_fp, comma
                ));
            }
        }
        manifest.push_str("  ],\n  \"files\": [\n");
        for (i, (path, content)) in files.iter().enumerate() {
            let comma = if i + 1 < files.len() { "," } else { "" };
            manifest.push_str(&format!(
                "    {{ \"path\": \"{}\", \"sha256\": \"{}\" }}{}\n",
                path,
                sha256_hex(content.as_bytes()),
                comma
            ));
        }
        manifest.push_str("  ]\n}\n");
        Ok(Assembly { files, manifest })
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
        if self.family() != Family::Collections {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                "ranged authorization is the collections warm contract; the other families keep their frozen fragment path",
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
        // 2) range validation: in-bounds, whole UTF-8 scalars, and it
        //    covers EXACTLY one function declaration — the target's
        let (start, end) = range;
        if start > end || end > base_src.len() {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                format!("range {start}..{end} escapes the base source (len {})", base_src.len()),
            )));
        }
        if !base_src.is_char_boundary(start) || !base_src.is_char_boundary(end) {
            return Err(A2xRejectOrResource::Reject(A2xReject::new(
                "a2x.range",
                "range cuts a multi-byte UTF-8 scalar",
            )));
        }
        let replaced = &base_src[start..end];
        let old_item = parse_function_fragment(replaced, 0..replaced.len(), self.profile).map_err(
            |d| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.range",
                    format!(
                        "range does not cover exactly one function declaration: {}",
                        d.message
                    ),
                ))
            },
        )?;
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
        let frag_item = parse_function_fragment(fragment, 0..fragment.len(), self.profile).map_err(
            |d| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.parse",
                    format!("fragment: {}", d.message),
                ))
            },
        )?;
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
        self.warm_edit_core(fn_name, fragment, entry, read_only)
    }

    fn warm_edit_core(
        &mut self,
        fn_name: &str,
        edited_src: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let profile = self.profile;
        // produce the candidate fragment text + identity while the
        // candidate is alive (SourceParts is not Clone — borrowed use only)
        let (frag_def, frag_name, frag_sem_fp, frag_text, input_hash, runtime, frag_view_records, frag_view_variants) = {
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
            let item = parse_function_fragment(edited_src, 0..edited_src.len(), profile)
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
            let slot = env_slot(env, fn_name).ok_or_else(|| {
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
                    verify_text_function(&parts.bundle, def_index_of(&parts.bundle, fn_name), &seed)
                }
                _ => verify_function_ref(&parts.bundle, def_index_of(&parts.bundle, fn_name)),
            }
            .map_err(|ds| {
                A2xRejectOrResource::Reject(A2xReject::new("a2x.verify", render_diag(&ds)))
            })?;
            let view =
                SemanticView::from_parts(parts, &cred).map_err(A2xRejectOrResource::Reject)?;
            let runtime = fragment_runtime_source(&view);
            let runtime_sha = sha256_hex(runtime.as_bytes());
            let f = view.functions().iter().next().ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new("a2x.internal", "empty fragment view"))
            })?;
            let sem_fp = semantic_fingerprint(&view, f).map_err(A2xRejectOrResource::Reject)?;
            // reuse by store hit / resident; else produce now
            let text = if let Some(r) = self.resident.get(f.source_name) {
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
            // the rt prelude needs the (record-carrying) fragment view
            // AFTER the borrow ends — snapshot the lowered record table
            let records_snapshot = if self.family() == Family::RecordText
                || self.family() == Family::Collections
            {
                Some(
                    ace_a2x_rust::lower::lower_records(&view)
                        .map_err(A2xRejectOrResource::Reject)?,
                )
            } else {
                None
            };
            let variants_snapshot = if self.family() == Family::Collections {
                Some(ace_a2x_rust::lower_coll::lower_coll_variants(&view))
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
                records_snapshot,
                variants_snapshot,
            )
        };
        let def_index = frag_def;
        let fragment = ResidentFragment {
            source_name: frag_name,
            def_index,
            sem_fp: frag_sem_fp,
            text: frag_text,
        };

        // swap the edited fragment into the resident set and assemble
        let mut frags: Vec<ResidentFragment> = self.resident.values().cloned().collect();
        frags.retain(|r| r.source_name != fragment.source_name);
        frags.push(fragment);
        frags.sort_by_key(|r| r.def_index);
        self.resident = frags
            .into_iter()
            .map(|r| (r.source_name.clone(), r))
            .collect();

        // assemble from residents (no extra production; manifest built
        // here) — the rt prelude re-derives from the snapshot record table
        let mut residents: Vec<(usize, ResidentFragment)> = self
            .resident
            .values()
            .map(|r| (r.def_index, r.clone()))
            .collect();
        // declaration order (BTreeMap iteration is name-alphabetical;
        // multi-function modules must assemble in def order — the cold
        // path's byte shape)
        residents.sort_by_key(|(d, _)| *d);
        self.build_fileset(
            entry,
            &input_hash,
            &residents,
            None,
            frag_view_records.as_deref(),
            runtime,
            frag_view_variants.as_deref(),
        )
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
        let text = if self.family() == Family::Collections {
            let program = ace_a2x_rust::lower_coll::lower_coll_functions(view)
                .map_err(A2xRejectOrResource::Reject)?;
            let rust_name = rust_fragment_name(f);
            let rust_fn = program
                .functions
                .iter()
                .find(|rf| rf.name == rust_name)
                .cloned()
                .ok_or_else(|| {
                    A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.internal",
                        format!("fragment for `{}` missing after lowering", f.source_name),
                    ))
                })?;
            print_coll_fragment_text(&program.records, &program.variants, &rust_fn)
        } else {
            let program = lower_functions(view).map_err(A2xRejectOrResource::Reject)?;
            let rust_name = rust_fragment_name(f);
            let rust_fn = program
                .functions
                .iter()
                .find(|rf| rf.name == rust_name)
                .cloned()
                .ok_or_else(|| {
                    A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.internal",
                        format!("fragment for `{}` missing after lowering", f.source_name),
                    ))
                })?;
            let rt_records = if self.family() == Family::RecordText {
                Some(
                    ace_a2x_rust::lower::lower_records(view)
                        .map_err(A2xRejectOrResource::Reject)?,
                )
            } else {
                None
            };
            match &rt_records {
                Some(records) => print_fragment_text(records, &rust_fn),
                None => ace_a2x_rust::print::print_fn(&rust_fn),
            }
        };
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
        // a restart drops residents and rebuilds purely from store hits
        self.resident.clear();
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
        self.resident.clear();
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

    /// Lease the CURRENT assembly manifest artifact against GC.
    pub fn lease_current(&self, purpose: &str) -> Result<ace_artifacts::store::Lease, String> {
        self.store
            .acquire_lease(purpose, 3600, None)
            .map_err(|e| format!("lease: {:?}", e))
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
    ace_core::source::env_fn_names(env)
        .iter()
        .position(|n| *n == name)
}

fn def_index_of(bundle: &ace_core::hir::Bundle, fn_name: &str) -> usize {
    bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == format!("d_{}", fn_name))
        .unwrap_or_else(|| panic!("def d_{} missing", fn_name))
}

/// Print one fragment: the record-text family renders against the module
/// record table (nominal names); the scalar family keeps the frozen path.
fn print_fragment_text(
    records: &[ace_a2x_rust::RustRecord],
    rust_fn: &ace_a2x_rust::RustFn,
) -> String {
    if records.is_empty() {
        ace_a2x_rust::print::print_fn(rust_fn)
    } else {
        ace_a2x_rust::print::print_fn_records(records, rust_fn)
    }
}

/// PLAN-904: the collections fragment text (types render through the
/// record AND variant tables).
fn print_coll_fragment_text(
    records: &[ace_a2x_rust::RustRecord],
    variants: &[ace_a2x_rust::lower::RustVariant],
    rust_fn: &ace_a2x_rust::RustFn,
) -> String {
    ace_a2x_rust::print::print_coll_fn(records, variants, rust_fn)
}

/// The fragment's Rust function name (free functions and owner-qualified
/// methods alike — the lowered `RustFn::name`).
fn rust_fragment_name(f: &FnFacts<'_>) -> String {
    match f.kind {
        ace_a2x_core::FnKind::Free => ace_a2x_rust::lower::rust_fn_name(f.source_name),
        ace_a2x_core::FnKind::Method { .. } => ace_a2x_rust::lower::rust_method_name(f.source_name),
    }
}

fn entry_rust_name(entry: &str, family: Family) -> String {
    // the entry is a free function in every admitted program
    let _ = family;
    ace_a2x_rust::lower::rust_fn_name(entry)
}

/// The minimal typed runtime text for a view (same frozen content as the
/// artifacts.rs emission; marks come from the view). PLAN-902: the
/// record-text family binds the RESOURCE runtime (its content hash keys
/// the rt fragment layer).
fn fragment_runtime_source(view: &SemanticView<'_>) -> String {
    let marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
    match view.domain {
        HirDomain::Text | HirDomain::Record | HirDomain::TextRecord => {
            ace_a2x_rust::record_text_runtime_source(&marks)
        }
        HirDomain::Collections => ace_a2x_rust::collections_runtime_source(&marks),
        _ => ace_a2x_rust::runtime_source(&marks),
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
        other => panic!("unknown layer {}", other),
    }
}
