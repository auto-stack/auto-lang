//! auto-mir-lower — PLAN-868 extraction of the HIR→Typed MIR lowering.
//!
//! Owner of the single `HIR → Typed MIR` lowering implementation consumed by
//! the av probe (worker/session hosts), `ac-mir-native` (MIR→CLIF native
//! cross-check) and the CTE probe. Extracted verbatim from
//! `ac-av-probe/probe/src/hir_adapter.rs` (PLAN-798 §5.1/§5.2) as a pure
//! move: function signatures, module paths and semantics are unchanged.
//!
//! Dependency red line: this crate depends only on `auto-ac-prototype`
//! (HIR/verify front door) and `auto-av-core` (Typed MIR model/verifier).
//! It must never depend on probe host types (worker/session/cache), so the
//! execution core stays free of front-end pollution and no consumer can
//! grow a private lowering copy without forking this crate.
//!
//! Lowering contract identity: [`MIR_LOWERING_VERSION`] is the semantic
//! version of this lowering contract; consumers report it verbatim on the
//! `mir-lowering` report identity axis so "the lowering changed" stays
//! distinguishable from "a consumer crate version changed".

pub mod hir_adapter;

// Flatten the module at the crate root: consumers spell
// `ace_lowering::lower_checked` — the crate IS the lowering.
pub use hir_adapter::*;

/// Semantic contract version of the HIR→Typed MIR lowering. Bump ONLY when
/// lowering semantics change (mapping rules, admitted domains, error
/// taxonomy); pure refactors and consumer-side moves keep it frozen.
/// v2 (PLAN-881): record/text-record type-bound static/view methods lower
/// as ordinary MIR functions (`m.<Owner>.<name>` identity) in the
/// av.data/1 + av.owned/1 whole-module and single-function surfaces —
/// the previously rejected `lower.method` rows become admitted.
/// v3 (PLAN-863): independent console HIR -> typed host references, with no change to existing domain mappings.
/// v4 (PLAN-894): text-domain admission extension — text eq/ne lowers to
/// `StrCmp` over view operands (owned operands view first, BIN stays
/// scalar-only) and `TextConcat` operands view-coerce; plus the owned-Str
/// local full-root replacement (`var` targets, §5.1) rides the existing
/// owned local-assign `Move` mapping.
pub const MIR_LOWERING_VERSION: &str = "4";
