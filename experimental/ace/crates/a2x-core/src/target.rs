//! PLAN-921 T-03: the shared, target-neutral emit-target contract.
//!
//! The A2X shared layer (a2x-store sessions, the semantic layers AXS1..3)
//! must not own Rust-private lowering/profile items: every target-specific
//! operation — fragment key identity, fragment production, runtime source,
//! prelude inputs and file-set assembly — is delegated to a
//! [`TargetProvider`] supplied by the target crate (`a2x-rust`, `a2x-c`).
//!
//! Contract rules (plan §5.2):
//! - the SEMANTIC half stays target-neutral: semantic keys carry no target
//!   fields and are shared by every provider;
//! - fragment/fileset/cert identities are FULLY target-qualified — a hit in
//!   one target's namespace can never satisfy another target's key;
//! - providers are deterministic and side-effect free (no process spawns,
//!   no store access, no clock reads): identical inputs produce identical
//!   bytes, which is what makes content-addressed reuse sound;
//! - a provider serves only the families it actually implements —
//!   unsupported families are NAMED rejections, never silent fallbacks.

use crate::reject::A2xReject;
use crate::semantic_view::{FnFacts, SemanticView};

/// The channel family of a source profile (layer-set + fileset selection).
/// Shared (was a2x-store-local) so providers and sessions agree on one
/// vocabulary; a2x-store re-exports it for source compatibility.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Scalar,
    RecordText,
    /// PLAN-904: the collections (List/Variant) channel — AXS3/AXR3
    /// namespaces, the collections runtime and fileset.
    Collections,
}

/// The identity of one emit target. Mirrors the fields the Rust target
/// already binds into every fragment key and manifest (lang / edition /
/// backend / backend_version); `standard` is the C-side spelling of the
/// edition slot (`c11`). Byte-frozen: providers return constants.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TargetDescriptor {
    /// Source language of the emitted artifacts (`rust`, `c`).
    pub lang: &'static str,
    /// Language standard/edition (`2021` for Rust, `c11` for C).
    pub standard: &'static str,
    /// Backend identity (`auto-a2x-rust`, `auto-a2x-c`).
    pub backend: &'static str,
    /// Backend version — bumped when emitted semantics or file-set shape
    /// change; part of every fragment key and manifest.
    pub backend_version: &'static str,
}

/// One fragment handed to file-set assembly (the session's resident/store
/// text plus its identity — enough for manifests and prelude scanning).
pub struct FragmentText<'a> {
    pub def_index: usize,
    pub source_name: &'a str,
    pub sem_fp: &'a str,
    pub text: &'a str,
}

/// An assembled emit file set: the full deterministic file list plus its
/// manifest. Same shape the a2x-rust fileset assemblers already return.
pub struct FileSet {
    pub files: Vec<(String, String)>,
    pub manifest: String,
}

/// Everything a provider needs to assemble the final file set. `view` is
/// the WHOLE-module view on the cold path (origins in manifests, prelude
/// derivation) and `None` on the warm path, where `prelude_seed` carries
/// the provider-opaque snapshot captured while the fragment view was
/// alive (see [`TargetProvider::capture_prelude_seed`]).
pub struct FilesetRequest<'a> {
    pub entry: &'a str,
    pub input_hash: &'a str,
    pub family: Family,
    pub fragments: &'a [FragmentText<'a>],
    pub view: Option<&'a SemanticView<'a>>,
    pub runtime: &'a str,
    pub prelude_seed: Option<&'a str>,
    /// PLAN-921: def-index -> source origin for view-less assemblies
    /// (pure restore) — the manifest carries origins exactly like the
    /// cold path's view-derived ones.
    pub origins: Option<&'a [(usize, (u32, u32))]>,
    /// sha256 of the current full source when the family binds its
    /// manifest to the replaced source (collections); `None` otherwise.
    pub source_sha256: Option<&'a str>,
}

/// The narrow emit-target seam (plan §5.2/D-01): one implementation per
/// target crate. Providers are stateless value types.
pub trait TargetProvider {
    /// Current process-local admission before either a cache hit or production.
    fn validate_view(&self, _view: &SemanticView<'_>) -> Result<(), A2xReject> {
        Ok(())
    }
    /// The frozen identity of this target (fragment keys, manifests).
    fn descriptor(&self) -> TargetDescriptor;

    /// Target-identity inputs of the FRAGMENT-layer key, appended after
    /// the shared (owner, decl, sem_fp) inputs, order-stable. The Rust
    /// provider returns exactly the frozen AXR1/2/3 fields
    /// (lang/edition/backend/backend_version/runtime_sha256/print_version)
    /// so existing keys are byte-identical; other targets spell their own
    /// equivalent set (never fewer fields than the runtime + printer
    /// identity).
    fn fragment_key_inputs(&self, runtime_sha256: &str) -> Vec<(String, String)>;

    /// The registered fragment-layer id for `family` (resolved against the
    /// a2x-store static registry). Unsupported families are named
    /// rejections.
    fn fragment_layer_id(&self, family: Family) -> Result<&'static str, A2xReject>;

    /// The runtime source text for a view — its content hash keys the
    /// fragment layer, so a target with a different runtime never
    /// cross-hits.
    fn runtime_source(&self, view: &SemanticView<'_>, family: Family) -> Result<String, A2xReject>;

    /// Lower + print `fns` from ONE view (the whole-module view on the
    /// cold path, the single-function fragment view on the warm path) to
    /// fragment texts, in request order. Batched so a whole-view lowering
    /// pass amortizes across every store miss (O(n), not O(n^2)).
    fn produce_fragments(
        &self,
        view: &SemanticView<'_>,
        fns: &[&FnFacts<'_>],
        family: Family,
    ) -> Result<Vec<String>, A2xReject>;

    /// Capture the provider-opaque module-level prelude seed from a view
    /// while it is alive (the warm path cannot re-derive it later). The
    /// seed round-trips losslessly through the provider's own codec and
    /// feeds [`FilesetRequest::prelude_seed`]; `None` when the family
    /// needs no prelude inputs (the scalar families).
    fn capture_prelude_seed(
        &self,
        view: &SemanticView<'_>,
        family: Family,
    ) -> Result<Option<String>, A2xReject>;

    /// Assemble the deterministic emit file set (wrapper, runtime,
    /// generated body, manifest) from the final fragment set. Byte-stable:
    /// the same request produces the same files.
    fn assemble_fileset(&self, req: FilesetRequest<'_>) -> Result<FileSet, A2xReject>;
}
