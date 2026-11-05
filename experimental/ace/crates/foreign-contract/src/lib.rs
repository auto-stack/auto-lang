//! PLAN-937 CA-D1: the backend-free foreign-interface contract.
//!
//! This crate owns the NEUTRAL half of the CA-D1 bridge — the half that is
//! consumed identically by two real consumers that must stay independent:
//!
//! * the **797 native ABI side** (`ace-abi`): Cranelift machine-code
//!   emission/admission reading the same physical projection it always
//!   did, now expressible from these facts;
//! * the **A2C C side** (`ace-a2x-c`): a thin public wrapper/export layer
//!   whose generated C carries exactly the physical form spelled here.
//!
//! What lives here (937 plan §2/§5.1):
//! * [`lane`] — the LOGICAL public scalar lanes plus their PHYSICAL
//!   projection per lane, including the bool/char lane decisions and the
//!   explicit no-f32 rule;
//! * [`facts`] — logical Interface Facts: nominal identity + content
//!   version, per-function logical signature forms, effects, the explicit
//!   error model (status table), capabilities, and the FACTS SOURCE
//!   (typed candidate vs checked source) — the identity separation the
//!   plan pins (a typed candidate is not a checked-source export);
//! * [`typed`] — explicit typed-declaration candidates: validation and
//!   normalization rules (pure requires recorded totality evidence;
//!   ordinary checked arithmetic is fallible), plus per-VALUE pre-call
//!   validation (bool 0/1, char = Unicode scalar value, finite-domain
//!   rules for the frozen fixtures);
//! * [`plan`] — the target/domain PHYSICAL plan projection (Windows x64
//!   register/stack classification spelled per parameter);
//! * [`codec`] — strict versioned codecs (`deny_unknown_fields`, wrong
//!   format rejected, bounded size) so persisted bridge artifacts decode
//!   fail-closed.
//!
//! What deliberately does NOT live here (937 plan §1 non-goals / §5.5):
//! no Auto-source foreign IMPORT model (938), no resource/callback/handle
//! bridge (939), no descriptor/3 wire change (the 797 descriptor stays
//! owned by `ace-abi`; T-03 asserts this projection is CONSISTENT with
//! it, it does not replace it), and no process, store, or tool access —
//! this crate is a pure model library.
//!
//! Private-vs-public boundary pinned in module docs of [`lane`]: the
//! internal mn/A2C private ABI (bool = I32 lane, text = ptr+len pair)
//! is a DIFFERENT domain and is never aliased onto these public lanes.

pub mod codec;
pub mod facts;
pub mod lane;
pub mod manual;
pub mod plan;
pub mod typed;
pub mod work;

pub use facts::{ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, FACTS_FORMAT};
pub use lane::{LaneKind, LaneValue, PhysLane};
pub use plan::{PhysFn, PhysForm, PhysParam, PhysRet, PhysicalPlan, SlotClass, PLAN_FORMAT};
pub use typed::{TotalityEvidence, TypedBodyFamily, TypedDecl, TypedDecls, TYPED_DECLS_FORMAT};

/// blake3, hex-encoded — the digest form shared with the 797 probe's
/// semref/descriptor fingerprints (same 64-char shape).
pub fn blake3_hex(bytes: &[u8]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(bytes);
    h.finalize().to_hex().to_string()
}
