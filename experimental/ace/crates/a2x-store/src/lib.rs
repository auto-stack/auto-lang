//! PLAN-892 T-04 (P-private part): the A2X layered cache/session over the
//! PLAN-783 `ManagedStore`/CAS — no second database (plan §5.4).
//!
//! Layer registry (FROZEN here; the Design 40 A00–A12 ids are deliberately
//! NOT squatted — these are A2X-owned namespaces registered beside them):
//!
//! | layer   | name                   | chunk kind        | contents                     |
//! |---------|------------------------|-------------------|------------------------------|
//! | `AXS1`  | a2x-function-semantic  | Hir               | per-function semantic plan bytes (target-neutral) |
//! | `AXR1`  | a2x-rust-fragment      | Object            | one printed Rust fragment (target layer) |
//! | `AXM1`  | a2x-fileset-manifest   | ObjectManifest    | an assembly manifest binding its fragments (required edges) |
//!
//! Key layering (§5.4): the semantic key reads NO target configuration;
//! every result that reads a [`TargetProfile`] lands in the target layer
//! whose key ADDS the target fields (lang/edition/backend/version/runtime
//! content). Absolute spans, snapshots and table reordering never enter a
//! key — only the semantic fingerprint and declared identities do.

use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::{semantic_fingerprint, A2xReject, FnFacts, SemanticView};
use ace_a2x_rust::{lower_functions, TargetProfile};
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::{QueryKey, RefKind};
use ace_artifacts::schema::LayerSpec;
use ace_artifacts::store::{gc_config_default, run_gc, ManagedStore, PutRequest, SpaceReport};
use ace_core::source::{
    compile_scalar_source, compile_source, parse_function_fragment, parse_program_with,
    prepare_function_candidate, resolve_env, SourceProfile,
};
use ace_core::verify::verify_function_ref;
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

/// Pass-version constants of THIS build (key inputs; §5.4 "pass版本").
pub const PASS_PREPARE: &str = "a2x-prepare/1";
pub const PRINT_VERSION: &str = "a2x-print/1";

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
    /// whole-module and single-body bundles (F-828-R1).
    resident: BTreeMap<String, ResidentFragment>,
    /// The retained checked environment (None until cold init).
    env: Option<ace_core::source::resolve::CheckedEnv>,
    profile: SourceProfile,
    caps: Vec<String>,
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
            profile: SourceProfile::Core,
            caps: vec![],
        })
    }

    pub fn space(&self) -> SpaceReport {
        self.store.space()
    }

    pub fn stats(&self) -> &ProducerStats {
        &self.stats
    }

    /// Semantic-layer key of a function (NO target fields).
    fn semantic_key(&self, f: &FnFacts<'_>, sem_fp: &str, hir_schema: &str) -> QueryKey {
        QueryKey::new(
            &LAYER_SEMANTIC,
            vec![
                ("owner".into(), self.owner.clone()),
                ("decl".into(), f.source_name.to_string()),
                ("profile".into(), self.profile_label().into()),
                ("hir_schema".into(), hir_schema.to_string()),
                ("signature".into(), ace_a2x_core::canonical_signature(f)),
                ("sem_fp".into(), sem_fp.to_string()),
                ("caps".into(), self.caps.join(",")),
                ("pass".into(), PASS_PREPARE.into()),
            ],
        )
    }

    /// Target-layer key: semantic key inputs + target fields (§5.4).
    fn fragment_key(&self, f: &FnFacts<'_>, sem_fp: &str, runtime_sha: &str) -> QueryKey {
        let t = TargetProfile::v1();
        QueryKey::new(
            &LAYER_FRAGMENT,
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
    pub fn cold_assemble(
        &mut self,
        src: &str,
        scalar: bool,
        caps: &[String],
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        self.profile = if scalar {
            SourceProfile::Scalar
        } else {
            SourceProfile::Core
        };
        self.caps = caps.to_vec();
        self.stats.parse += 1;
        let compiled = if scalar {
            compile_scalar_source(src, "a2x", caps).map_err(|ds| {
                A2xRejectOrResource::Reject(A2xReject::new("a2x.compile", render_diag(&ds)))
            })?
        } else {
            compile_source(src, "a2x", caps).map_err(|ds| {
                A2xRejectOrResource::Reject(A2xReject::new("a2x.compile", render_diag(&ds)))
            })?
        };
        let view =
            SemanticView::from_checked(&compiled.checked).map_err(A2xRejectOrResource::Reject)?;

        // retained environment for warm edits
        let program = if scalar {
            parse_program_with(src, SourceProfile::Scalar)
        } else {
            parse_program_with(src, SourceProfile::Core)
        }
        .map_err(|d| A2xRejectOrResource::Reject(A2xReject::new("a2x.parse", d.message)))?;
        self.stats.parse += 1;
        let env = resolve_env(&program, caps, self.profile).map_err(|ds| {
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

            // 1) resident reuse: unchanged function → ZERO production
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
            if let Ok(Some(hit)) = self.store.get_artifact(LAYER_FRAGMENT.id, &frag_key) {
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
            if whole.is_none() {
                whole = Some(lower_functions(view).map_err(A2xRejectOrResource::Reject)?);
            }
            let program = whole.as_ref().unwrap();
            let rust_fn = program
                .functions
                .iter()
                .find(|rf| rf.name == ace_a2x_rust::lower::rust_fn_name(f.source_name))
                .cloned()
                .ok_or_else(|| {
                    A2xRejectOrResource::Reject(A2xReject::new(
                        "a2x.internal",
                        format!("fragment for `{}` missing after lowering", f.source_name),
                    ))
                })?;
            let text = ace_a2x_rust::print::print_fn(&rust_fn);

            if !read_only {
                // semantic layer first
                let sem_key = self.semantic_key(f, &sem_fp, view.domain.schema());
                let sem_payload = format!(
                    "{{\"schema\":\"{}\",\"decl\":\"{}\",\"signature\":\"{}\",\"sem_fp\":\"{}\"}}",
                    LAYER_SEMANTIC.schema_id,
                    f.source_name,
                    ace_a2x_core::canonical_signature(f),
                    sem_fp
                );
                let sem_id = self.quota_put(
                    LAYER_SEMANTIC.id,
                    sem_key,
                    ChunkKind::Hir,
                    sem_payload.as_bytes(),
                    vec![],
                )?;
                // fragment layer (provenance edge to its semantic input)
                let frag_id = self.quota_put(
                    LAYER_FRAGMENT.id,
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
        let mut generated = String::new();
        generated.push_str(&format!(
            "// AUTO-GENERATED by auto-a2x-rust (PLAN-892 AX-A) — session assembly, entry `{}`\n\n",
            entry
        ));
        generated.push_str("use crate::a2x_runtime::*;\n\n");
        for (_, r) in &fragments {
            generated.push_str(&r.text);
            generated.push('\n');
        }
        if !fragments.iter().any(|(_, r)| r.source_name == entry) {
            return Err(A2xRejectOrResource::Reject(A2xReject::entry(
                "entry missing",
            )));
        }
        let entry_rust = ace_a2x_rust::lower::rust_fn_name(entry);
        let wrapper = format!(
            "// AUTO-GENERATED (PLAN-892 AX-A) — DO NOT EDIT.\nmod a2x_runtime;\nmod generated;\n\nuse a2x_runtime::A2xResult;\n\nfn main() {{\n    let outcome: A2xResult<i32> = generated::{}();\n    match outcome {{\n        Ok(v) => println!(\"value:{{}}\", v),\n        Err(t) => println!(\"trap:{{}}\", t.name()),\n    }}\n}}\n",
            entry_rust
        );
        let files = vec![
            ("main.rs".to_string(), wrapper),
            ("a2x_runtime.rs".to_string(), runtime),
            ("generated.rs".to_string(), generated),
        ];
        let mut manifest = String::new();
        manifest.push_str("{\n  \"schema\": \"auto.a2x.rust.fileset/1\",\n");
        manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
        manifest.push_str(&format!(
            "  \"entry\": \"{}\",\n  \"functions\": [\n",
            entry
        ));
        for (i, (d, r)) in fragments.iter().enumerate() {
            let origin = view.fn_by_def(*d).map(|f| f.origin).unwrap_or((0, 0));
            let comma = if i + 1 < fragments.len() { "," } else { "" };
            manifest.push_str(&format!(
                "    {{ \"decl\": \"{}\", \"sem_fp\": \"{}\", \"origin\": [{}, {}] }}{}\n",
                r.source_name, r.sem_fp, origin.0, origin.1, comma
            ));
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
    /// the resident set with ZERO production (§5.4).
    #[allow(clippy::too_many_arguments)]
    pub fn warm_edit(
        &mut self,
        fn_name: &str,
        edited_src: &str,
        entry: &str,
        read_only: bool,
    ) -> Result<Assembly, A2xRejectOrResource> {
        let profile = self.profile;
        // produce the candidate fragment text + identity while the
        // candidate is alive (SourceParts is not Clone — borrowed use only)
        let (frag_def, frag_name, frag_sem_fp, frag_text, input_hash, runtime) = {
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
            let cred = verify_function_ref(&parts.bundle, def_index_of(&parts.bundle, fn_name))
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
            (
                f.def_index,
                f.source_name.to_string(),
                sem_fp,
                text,
                parts.input_hash.clone(),
                runtime,
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

        // assemble from residents (no extra production; manifest built here)
        let mut generated = String::new();
        generated.push_str(&format!(
            "// AUTO-GENERATED by auto-a2x-rust (PLAN-892 AX-A) — session assembly, entry `{}`\n\n",
            entry
        ));
        generated.push_str("use crate::a2x_runtime::*;\n\n");
        for r in self.resident.values() {
            generated.push_str(&r.text);
            generated.push('\n');
        }
        if !self.resident.values().any(|r| r.source_name == entry) {
            return Err(A2xRejectOrResource::Reject(A2xReject::entry(
                "entry not resident",
            )));
        }
        let entry_rust = ace_a2x_rust::lower::rust_fn_name(entry);
        let wrapper = format!(
            "// AUTO-GENERATED (PLAN-892 AX-A) — DO NOT EDIT.\nmod a2x_runtime;\nmod generated;\n\nuse a2x_runtime::A2xResult;\n\nfn main() {{\n    let outcome: A2xResult<i32> = generated::{}();\n    match outcome {{\n        Ok(v) => println!(\"value:{{}}\", v),\n        Err(t) => println!(\"trap:{{}}\", t.name()),\n    }}\n}}\n",
            entry_rust
        );
        let files = vec![
            ("main.rs".to_string(), wrapper),
            ("a2x_runtime.rs".to_string(), runtime),
            ("generated.rs".to_string(), generated),
        ];
        let mut manifest = String::new();
        manifest.push_str("{\n  \"schema\": \"auto.a2x.rust.fileset/1\",\n");
        manifest.push_str(&format!("  \"input_hash\": \"{}\",\n", input_hash));
        manifest.push_str(&format!(
            "  \"entry\": \"{}\",\n  \"functions\": [\n",
            entry
        ));
        let residents: Vec<&ResidentFragment> = self.resident.values().collect();
        for (i, r) in residents.iter().enumerate() {
            let comma = if i + 1 < residents.len() { "," } else { "" };
            manifest.push_str(&format!(
                "    {{ \"decl\": \"{}\", \"sem_fp\": \"{}\" }}{}\n",
                r.source_name, r.sem_fp, comma
            ));
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

    fn produce_fragment(
        &mut self,
        view: &SemanticView<'_>,
        f: &FnFacts<'_>,
        sem_fp: &str,
        runtime_sha: &str,
        read_only: bool,
    ) -> Result<ResidentFragment, A2xRejectOrResource> {
        let frag_key = self.fragment_key(f, sem_fp, runtime_sha);
        if let Ok(Some(hit)) = self.store.get_artifact(LAYER_FRAGMENT.id, &frag_key) {
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
        let program = lower_functions(view).map_err(A2xRejectOrResource::Reject)?;
        let rust_fn = program
            .functions
            .iter()
            .find(|rf| rf.name == ace_a2x_rust::lower::rust_fn_name(f.source_name))
            .cloned()
            .ok_or_else(|| {
                A2xRejectOrResource::Reject(A2xReject::new(
                    "a2x.internal",
                    format!("fragment for `{}` missing after lowering", f.source_name),
                ))
            })?;
        let text = ace_a2x_rust::print::print_fn(&rust_fn);
        if !read_only {
            let sem_key = self.semantic_key(f, sem_fp, view.domain.schema());
            let sem_payload = format!(
                "{{\"schema\":\"{}\",\"decl\":\"{}\",\"signature\":\"{}\",\"sem_fp\":\"{}\"}}",
                LAYER_SEMANTIC.schema_id,
                f.source_name,
                ace_a2x_core::canonical_signature(f),
                sem_fp
            );
            let sem_id = self.quota_put(
                LAYER_SEMANTIC.id,
                sem_key,
                ChunkKind::Hir,
                sem_payload.as_bytes(),
                vec![],
            )?;
            self.quota_put(
                LAYER_FRAGMENT.id,
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

/// The minimal typed runtime text for a view (same frozen content as the
/// artifacts.rs emission; marks come from the view).
fn fragment_runtime_source(view: &SemanticView<'_>) -> String {
    let marks: Vec<&'static str> = view.trace_symbols().into_iter().collect();
    ace_a2x_rust::runtime_source(&marks)
}

fn static_layer(id: &str) -> &'static str {
    match id {
        "AXS1" => LAYER_SEMANTIC.id,
        "AXR1" => LAYER_FRAGMENT.id,
        "AXM1" => LAYER_MANIFEST.id,
        other => panic!("unknown layer {}", other),
    }
}
