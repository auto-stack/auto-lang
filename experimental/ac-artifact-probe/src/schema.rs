//! A00–A12 layer registry: the unified artifact-contract boundary
//! (PLAN-783 §5.1, Design 40 §3).
//!
//! Every layer registers its schema identity, producer, consumers, key
//! inputs, reference kinds, storage/residency policy and regenerability,
//! and — critically — how far the probe actually implements it:
//!
//! * `Codec`      — implemented end to end in this probe (real codec, real
//!                  store payload, real consumer);
//! * `ProbeSeam`  — the identity/edge/manifest machinery is exercised, but
//!                  the payload is probe-only (not a language product);
//! * `Future`     — schema seam and legal/illegal descriptions only; the
//!                  probe refuses to produce or consume these layers with a
//!                  controlled error.

use std::fmt;

/// How much of a layer this probe actually implements.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerCoverage {
    /// Real codec + real store payload + real consumer in this probe.
    Codec,
    /// Identity/edge/manifest machinery exercised with probe payloads.
    ProbeSeam,
    /// Schema seam and legal/illegal descriptions only.
    Future,
}

/// How a layer's payloads may be regenerated when absent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Regeneration {
    /// Requires the original user-owned inputs (source text); the cache
    /// never owns them.
    FromUserInputs,
    /// Recomputable from other retained artifacts plus a recorded recipe.
    FromUpstreamRecipe,
    /// Cannot be regenerated inside the store (user delivery output); the
    /// store only references provenance.
    NotRegeneratable,
}

impl fmt::Display for Regeneration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Regeneration::FromUserInputs => write!(f, "from-user-inputs"),
            Regeneration::FromUpstreamRecipe => write!(f, "from-upstream-recipe"),
            Regeneration::NotRegeneratable => write!(f, "not-regeneratable"),
        }
    }
}

/// One registered layer of the unified artifact contract.
pub struct LayerSpec {
    /// `"A00"` .. `"A12"`.
    pub id: &'static str,
    pub name: &'static str,
    /// Versioned schema identity string (domain-tag for content hashes).
    pub schema_id: &'static str,
    pub producer: &'static str,
    pub consumers: &'static [&'static str],
    /// Named key inputs (fingerprint components); order-independent.
    pub key_inputs: &'static [&'static str],
    /// Reference kinds this layer may carry to other artifacts.
    pub ref_kinds: &'static [&'static str],
    pub storage: &'static str,
    pub residency: &'static str,
    pub regeneration: Regeneration,
    pub coverage: LayerCoverage,
}

impl LayerSpec {
    pub fn is_implemented(&self) -> bool {
        matches!(self.coverage, LayerCoverage::Codec)
    }
}

/// The full A00–A12 registry (Design 40 §3 ordering; PLAN-783 §5.1).
pub const LAYERS: &[LayerSpec] = &[
    LayerSpec {
        id: "A00",
        name: "compile-inputs",
        schema_id: "ac783/a00-inputs/v1",
        producer: "session/tooling",
        consumers: &["every downstream stage key"],
        key_inputs: &["source-bytes", "module-logical-name", "target", "features", "profile", "toolchain", "env-fingerprint"],
        ref_kinds: &["provenance"],
        storage: "small config records + raw source as independent blobs; user source is NOT owned by the store",
        residency: "config in index; source bytes cold, one blob per snapshot",
        regeneration: Regeneration::FromUserInputs,
        coverage: LayerCoverage::Codec,
    },
    LayerSpec {
        id: "A01",
        name: "grammar-tables",
        schema_id: "ac783/a01-grammar/v1",
        producer: "language-definition pipeline",
        consumers: &["parser", "tree-schema consumers"],
        key_inputs: &["grammar-id", "grammar-revision", "scanner-abi"],
        ref_kinds: &["required-data"],
        storage: "shared per language generation, not per project",
        residency: "pinned per generation; replaced wholesale on grammar upgrade",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Future,
    },
    LexSnapLayer(),
    LayerSpec {
        id: "A03",
        name: "tolerant-cst",
        schema_id: "ac783/a03-cst/v1",
        producer: "parser (future grammar-driven)",
        consumers: &["incremental reparsing", "AST projection", "tooling"],
        key_inputs: &["source-snapshot", "grammar-id", "parser-version"],
        ref_kinds: &["required-data", "provenance"],
        storage: "immutable subtree pages; synthetic-node population in this probe",
        residency: "hot pages mapped on demand; full-tree residency is never required",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::ProbeSeam,
    },
    LayerSpec {
        id: "A04",
        name: "ast-decl-summary",
        schema_id: "ac783/a04-summary/v1",
        producer: "source frontend (real AST)",
        consumers: &["HIR driver", "cross-module interface lookup"],
        key_inputs: &["profile", "schema-revision", "signature-content"],
        ref_kinds: &["required-data", "provenance"],
        storage: "per-module summary artifact (binary)",
        residency: "small; hot in index",
        regeneration: Regeneration::FromUserInputs,
        coverage: LayerCoverage::Codec,
    },
    LayerSpec {
        id: "A05",
        name: "macro-comptime",
        schema_id: "ac783/a05-macro/v1",
        producer: "expansion pipeline",
        consumers: &["HIR driver"],
        key_inputs: &["expansion-input-ids", "env-fingerprint"],
        ref_kinds: &["required-data", "provenance"],
        storage: "per expansion instance; uncontrollable side effects disable sharing",
        residency: "cold",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Future,
    },
    SemInfoLayer(),
    LayerSpec {
        id: "A07",
        name: "public-hir",
        schema_id: "ac783/a07-hir/v1",
        producer: "descriptor binder (real)",
        consumers: &["verifier (re-verify on restore)", "lowering"],
        key_inputs: &["hir-domain", "schema-revision", "profile", "capabilities", "source-snapshot", "codec-version"],
        ref_kinds: &["required-data", "provenance"],
        storage: "one binary bundle artifact per module",
        residency: "restored only on demand; a restored bundle is NEVER trusted — it re-enters the real verifier",
        regeneration: Regeneration::FromUserInputs,
        coverage: LayerCoverage::Codec,
    },
    LayerSpec {
        id: "A08",
        name: "semantic-lowering-passes",
        schema_id: "ac783/a08-pass/v1",
        producer: "pass pipeline",
        consumers: &["backend"],
        key_inputs: &["hir-artifact", "pass-chain-version"],
        ref_kinds: &["required-data", "provenance"],
        storage: "per pass artifact (only implemented passes)",
        residency: "short-lived",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Future,
    },
    ObjectLayer(),
    LayerSpec {
        id: "A10",
        name: "link-publish",
        schema_id: "ac783/a10-link/v1",
        producer: "staged linker driver (real)",
        consumers: &["delivery"],
        key_inputs: &["object-artifact", "entry", "toolchain-fingerprint", "libs"],
        ref_kinds: &["required-data", "provenance"],
        storage: "link receipts in the store; delivered exe/lib is user output, NOT store-owned",
        residency: "receipt metadata only",
        regeneration: Regeneration::NotRegeneratable,
        coverage: LayerCoverage::Codec,
    },
    LayerSpec {
        id: "A11",
        name: "diagnostics-tooling",
        schema_id: "ac783/a11-diag/v1",
        producer: "any stage",
        consumers: &["ide", "cli"],
        key_inputs: &["stage-key", "revision"],
        ref_kinds: &["provenance"],
        storage: "versioned records; failure results only reused under the full input key",
        residency: "bounded",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Future,
    },
    LayerSpec {
        id: "A12",
        name: "hot-reload-generations",
        schema_id: "ac783/a12-generations/v1",
        producer: "runtime",
        consumers: &["reload orchestrator"],
        key_inputs: &["generation", "state-schema"],
        ref_kinds: &["required-data"],
        storage: "live runtime state; NOT ordinary cache LRU",
        residency: "held until safety point + lease release",
        regeneration: Regeneration::NotRegeneratable,
        coverage: LayerCoverage::Future,
    },
];

#[allow(non_snake_case)]
const fn LexSnapLayer() -> LayerSpec {
    LayerSpec {
        id: "A02",
        name: "tokens-checkpoints",
        schema_id: "ac783/a02-lex/v1",
        producer: "lexer/scanner",
        consumers: &["parser", "incremental lexing"],
        key_inputs: &["source-snapshot", "scanner-abi", "lexer-version"],
        ref_kinds: &["required-data", "provenance"],
        storage: "read-only positioned blocks per lex region",
        residency: "cold; states converge per region",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Future,
    }
}

#[allow(non_snake_case)]
const fn SemInfoLayer() -> LayerSpec {
    LayerSpec {
        id: "A06",
        name: "semantic-interface-meta",
        schema_id: "ac783/a06-sem/v1",
        producer: "resolve/typecheck",
        consumers: &["HIR driver", "interface queries"],
        key_inputs: &["profile", "summary-ids", "typecheck-version"],
        ref_kinds: &["required-data", "provenance"],
        storage: "typed id tables + records; interface layer separate from confirmed semantics",
        residency: "module summaries hot",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Future,
    }
}

#[allow(non_snake_case)]
const fn ObjectLayer() -> LayerSpec {
    LayerSpec {
        id: "A09",
        name: "backend-object",
        schema_id: "ac783/a09-object/v1",
        producer: "native lowering (real Cranelift path)",
        consumers: &["linker driver"],
        key_inputs: &[
            "hir-artifact",
            "target",
            "backend-id",
            "lowering-options",
            "entry",
        ],
        ref_kinds: &["provenance"], // upstream HIR is NOT required once the object exists
        storage: "raw native object bytes as one immutable chunk",
        residency: "independently linkable; HIR/CST need not stay resident",
        regeneration: Regeneration::FromUpstreamRecipe,
        coverage: LayerCoverage::Codec,
    }
}

/// Look up a layer by id.
pub fn layer(id: &str) -> Option<&'static LayerSpec> {
    LAYERS.iter().find(|l| l.id == id)
}

/// Layers with real codec coverage in this probe.
pub fn codec_layers() -> Vec<&'static LayerSpec> {
    LAYERS
        .iter()
        .filter(|l| l.coverage == LayerCoverage::Codec)
        .collect()
}

/// One legal/illegal payload description sample for a layer seam. For
/// `Codec` layers these are backed by real fixtures/tests; for `Future`
/// layers they are the frozen seam descriptions the future implementation
/// must satisfy (and the probe's refusal to produce them is tested).
pub struct LayerSample {
    pub layer_id: &'static str,
    pub legal: &'static [&'static str],
    pub illegal: &'static [&'static str],
}

pub const SAMPLES: &[LayerSample] = &[
    LayerSample {
        layer_id: "A00",
        legal: &["(source-bytes, module-logical-name) snapshot record referencing a raw blob"],
        illegal: &[
            "snapshot claiming absolute path identity as content key",
            "snapshot embedding the user's only source copy as store-owned data",
        ],
    },
    LayerSample {
        layer_id: "A01",
        legal: &["grammar-id/revision pinned table set shared across projects"],
        illegal: &["per-project copy of grammar tables keyed by project path"],
    },
    LayerSample {
        layer_id: "A02",
        legal: &["region-scoped token block with scanner checkpoint"],
        illegal: &[
            "token block keyed by mtime",
            "checkpoint storing process pointers",
        ],
    },
    LayerSample {
        layer_id: "A03",
        legal: &["immutable subtree page shared between two snapshot versions"],
        illegal: &[
            "mutable in-place CST node",
            "full-file CST residency as a hit requirement",
        ],
    },
    LayerSample {
        layer_id: "A04",
        legal: &["signature-only summary content-identical across body-only edits"],
        illegal: &["summary keyed by full source hash so body edits invalidate interfaces"],
    },
    LayerSample {
        layer_id: "A05",
        legal: &["expansion instance recording actual file/env/tool inputs"],
        illegal: &["shared expansion cache with uncontrolled side-effect macros"],
    },
    LayerSample {
        layer_id: "A06",
        legal: &["interface summary consumable without loading function bodies"],
        illegal: &["interface hit that requires all downstream bodies resident"],
    },
    LayerSample {
        layer_id: "A07",
        legal: &["restored bundle that re-enters the real verifier before any use"],
        illegal: &[
            "persisted trusted flag constructing a Checked credential",
            "bundle restored from text roundtrip claimed as binary restore",
        ],
    },
    LayerSample {
        layer_id: "A08",
        legal: &["cached pass output keyed by (hir-artifact, pass-chain-version)"],
        illegal: &["pass cache reused across pass-chain version bump"],
    },
    LayerSample {
        layer_id: "A09",
        legal: &["object chunk linkable while its upstream HIR is cold/absent"],
        illegal: &[
            "object hit after body-only edit (same signature)",
            "fake blob standing in for real lowering",
        ],
    },
    LayerSample {
        layer_id: "A10",
        legal: &["link receipt bound to real persisted object bytes"],
        illegal: &[
            "store GC deleting the user's delivered exe",
            "PE nondeterminism treated as semantic failure",
        ],
    },
    LayerSample {
        layer_id: "A11",
        legal: &["diagnostics pinned to the exact stage key + revision"],
        illegal: &["failure diagnostics reused under a partial key"],
    },
    LayerSample {
        layer_id: "A12",
        legal: &["old generation held until safety point + lease release"],
        illegal: &["runtime generation reclaimed by ordinary LRU"],
    },
];

/// Controlled error type for registry violations (e.g. producing a
/// `Future` layer through probe machinery).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaError(pub String);

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "schema error: {}", self.0)
    }
}

impl std::error::Error for SchemaError {}

/// Assert a layer may be produced by probe code; `Future` layers are
/// refused with a controlled error naming the seam.
pub fn require_producible(spec: &LayerSpec) -> Result<LayerCoverage, SchemaError> {
    match spec.coverage {
        LayerCoverage::Future => Err(SchemaError(format!(
            "layer {} ({}): future seam only — the probe refuses to produce or consume this layer",
            spec.id, spec.name
        ))),
        c => Ok(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_complete_and_unique() {
        assert_eq!(LAYERS.len(), 13);
        for (i, l) in LAYERS.iter().enumerate() {
            assert_eq!(l.id, format!("A{:02}", i), "layer order/id");
            assert!(!l.schema_id.is_empty());
            assert!(!l.key_inputs.is_empty(), "{} needs key inputs", l.id);
            assert!(!l.consumers.is_empty(), "{} needs consumers", l.id);
            assert!(!l.ref_kinds.is_empty(), "{} needs ref kinds", l.id);
        }
        let ids: Vec<_> = LAYERS.iter().map(|l| l.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "duplicate layer ids");
    }

    #[test]
    fn every_layer_has_legal_and_illegal_samples() {
        for l in LAYERS {
            let s = SAMPLES
                .iter()
                .find(|s| s.layer_id == l.id)
                .unwrap_or_else(|| panic!("{} missing samples", l.id));
            assert!(!s.legal.is_empty(), "{} legal sample", l.id);
            assert!(!s.illegal.is_empty(), "{} illegal sample", l.id);
        }
        for s in SAMPLES {
            assert!(
                layer(s.layer_id).is_some(),
                "unknown sample layer {}",
                s.layer_id
            );
        }
    }

    #[test]
    fn codec_coverage_is_the_declared_subset() {
        let ids: Vec<&str> = codec_layers().iter().map(|l| l.id).collect();
        assert_eq!(ids, vec!["A00", "A04", "A07", "A09", "A10"]);
    }

    #[test]
    fn future_layers_are_refused() {
        let a01 = layer("A01").unwrap();
        assert!(matches!(a01.coverage, LayerCoverage::Future));
        let err = require_producible(a01).unwrap_err();
        assert!(err.0.contains("future seam"), "{err}");
        assert!(require_producible(layer("A07").unwrap()).is_ok());
    }
}
