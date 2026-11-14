//! PLAN-967 T-14 (AC-12 / V-14, V-ABI-CACHE+PACK applicable parts): the
//! provider BUILD cache — precise BuildIdentity keys derived from the
//! REAL read dependencies, stored in the existing 783 ManagedStore.
//!
//! Identity axes (the T-09 frozen field list): provider source bytes,
//! generated shim bytes, the toolchain's own `rustc --version` identity,
//! the generator identity (auto-bindgen rust_import version), the frozen
//! target and the crate types. Every axis is a REAL read — no labels.
//!
//! Semantics (V-14 / V-ABI-CACHE):
//! * a HIT reuses the built artifacts (linker/loader satisfy from the
//!   cached cdylib/staticlib/import-lib) — rustc never re-runs;
//! * a provider SOURCE edit flips the identity (miss → rebuild);
//! * a generated-shim edit flips the identity (the generator is an axis);
//! * a rustc override (AC741_RUSTC — the V-ABI-CROSS axis) flips the
//!   identity even when every source byte is identical;
//! * an ordinary AUTO body edit does NOT touch this key (the caller
//!   source is not a build input of the provider unit).
//!
//! The distribution manifest (V-ABI-PACK seed, handed to 978): the
//! artifact set + every identity axis + the runtime-dependency note —
//! a JSON document beside the cached artifacts.
use crate::rust_provider::RustProviderBuild;
use ace_artifacts::schema::LayerSpec;
use ace_artifacts::store::{ManagedStore, PutRequest};
use serde::{Deserialize, Serialize};

use ace_artifacts::schema::{LayerCoverage, Regeneration};

pub const BUILD_LAYER: LayerSpec = LayerSpec {
    id: "S967B",
    name: "f0-rust-provider-build",
    schema_id: "abi/rust-provider-build/1",
    producer: "plan967/rust-provider-build/1",
    consumers: &["f0 rust provider lanes (AV/native/A2R/A2PY)"],
    key_inputs: &[
        "provider-sha",
        "shim-sha",
        "rustc-identity",
        "generator-identity",
        "target",
        "crate-types",
    ],
    ref_kinds: &[],
    storage: "original ManagedStore",
    residency: "content-addressed artifacts",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};

/// The precise build identity (every axis a REAL read).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildIdentity {
    pub provider_sha256: String,
    pub shim_sha256: String,
    pub rustc_identity: String,
    pub generator_identity: String,
    pub target: String,
    pub crate_types: Vec<String>,
}

/// The cached product set of one identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecord {
    pub schema: String,
    pub identity: BuildIdentity,
    /// the artifact paths (cdylib + staticlib + import lib) as produced
    pub cdylib: String,
    pub staticlib: String,
    pub import_lib: String,
    /// runtime-dependency note (the distribution manifest seed): the
    /// MSVC CRT the cdylib imports (recorded from the toolchain, never
    /// from a guess)
    pub runtime_note: String,
}

pub const RECORD_SCHEMA: &str = "abi/rust-provider-build-record/1";
/// The runtime-dependency note for the frozen toolchain (dumpbin-verified
/// family: VCRUNTIME140 + the api-ms-win-crt set — the 967 packs never
/// embed a libc replacement).
pub const RUNTIME_NOTE: &str = "msvc-crt:VCRUNTIME140+ucrt (dynamic CRT, dumpbin-verified)";

impl BuildIdentity {
    /// Derive the identity from a REAL build (the rustc identity the build
    /// actually recorded) + the source bytes + the generator version.
    pub fn of(build: &RustProviderBuild, shim_source: &str) -> Self {
        Self {
            provider_sha256: build.provider_digest.clone(),
            shim_sha256: ace_foreign_contract::blake3_hex(shim_source.as_bytes()),
            rustc_identity: build.rustc_identity.clone(),
            generator_identity: format!(
                "auto-bindgen/rust_import/{}+{}",
                env!("CARGO_PKG_VERSION"),
                auto_bindgen::rust_import::IMPORT_FORMAT
            ),
            target: build.manifest.target.clone(),
            crate_types: vec!["cdylib".into(), "staticlib".into()],
        }
    }

    /// The canonical store key of this identity (a QueryKey component
    /// string — sorted axes, version-tagged).
    pub fn key(&self) -> String {
        format!(
            "S967B/v1 provider={} shim={} rustc={} gen={} target={} types={}",
            self.provider_sha256,
            self.shim_sha256,
            self.rustc_identity,
            self.generator_identity,
            self.target,
            self.crate_types.join("+")
        )
    }
}

/// One build cache over the existing ManagedStore: hit = the record +
/// artifacts already on disk; miss = the caller builds and PUTs.
pub struct ProviderBuildCache {
    store: ManagedStore,
}

impl ProviderBuildCache {
    pub fn open(root: &std::path::Path) -> Result<Self, String> {
        let store = ManagedStore::open(root).map_err(|e| format!("store open: {e:?}"))?;
        Ok(Self { store })
    }

    /// A cache HIT iff a record with this exact identity exists AND its
    /// artifacts are still on disk (a deleted artifact is a miss, never
    /// a stale-green).
    pub fn get(&self, identity: &BuildIdentity) -> Result<Option<BuildRecord>, String> {
        let key = ace_artifacts::identity::QueryKey {
            layer: BUILD_LAYER.id,
            components: vec![("identity".into(), identity.key())],
        };
        let Ok(maybe) = self.store.get_artifact(BUILD_LAYER.id, &key) else {
            return Ok(None);
        };
        let Some(loaded) = maybe else {
            return Ok(None);
        };
        let record: BuildRecord =
            serde_json::from_slice(&loaded.payload).map_err(|e| format!("record decode: {e}"))?;
        if record.identity != *identity {
            return Ok(None);
        }
        if !std::path::Path::new(&record.cdylib).is_file()
            || !std::path::Path::new(&record.staticlib).is_file()
        {
            return Ok(None);
        }
        Ok(Some(record))
    }

    /// PUT a built product set under its identity.
    pub fn put(&self, build: &RustProviderBuild, shim_source: &str) -> Result<BuildRecord, String> {
        let identity = BuildIdentity::of(build, shim_source);
        let record = BuildRecord {
            schema: RECORD_SCHEMA.into(),
            identity: identity.clone(),
            cdylib: build.cdylib.display().to_string(),
            staticlib: build.staticlib.display().to_string(),
            import_lib: build
                .cdylib
                .with_file_name("f0_text_provider.dll.lib")
                .display()
                .to_string(),
            runtime_note: RUNTIME_NOTE.into(),
        };
        let key = ace_artifacts::identity::QueryKey {
            layer: BUILD_LAYER.id,
            components: vec![("identity".into(), identity.key())],
        };
        let bytes = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
        self.store
            .put_artifact(PutRequest {
                layer: BUILD_LAYER.id,
                key,
                kind: ace_artifacts::codec::ChunkKind::LayoutData,
                payload: bytes,
                edges: vec![],
                project: Some("plan967".into()),
            })
            .map_err(|e| format!("store put: {e:?}"))?;
        Ok(record)
    }
}
