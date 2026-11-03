//! PLAN-921 CA-A + PLAN-925 CA-B: the C emit target (HIR → C11, scalar
//! and record-text channels).
//!
//! Lowers the shared [`SemanticView`] to deterministic C11 source —
//! explicit checked arithmetic with named traps via an inline runtime
//! header, source-order single evaluation of call arguments pinned into
//! temporaries, short-circuit `and`/`or` as branch-guarded temporaries,
//! `for(;;)` loops with C `break`/`continue`, and hygienic `fn_`/`mt_`/`lN_`
//! naming — then assembles the full emit file set (`main.c` wrapper +
//! runtime header + `generated.c` + manifest). The record-text channel
//! (PLAN-925) adds the typed resource runtime, private record structs
//! with per-record deep-clone/free helpers, view/mut/move param modes,
//! the verifier's buffer-free schedule and the prelude-seed record table
//! (pure-restore re-assembly).
//!
//! Contract: each C channel is its own namespace (scalar:
//! `ac-a2c/hir-scalar-emit-v1` / `auto.a2x.c.fileset/1` / AXC1;
//! record-text: `ac-a2c/hir-record-text-emit-v1` /
//! `auto.a2x.c.record-text.fileset/1` / AXC2) — a Rust or cross-channel
//! fragment/cert/fileset can never satisfy a C identity or vice versa.
//! The semantic facts (AXS1/AXS2 keys, fingerprints) are SHARED with the
//! Rust target; only the fragment layers are C-private.

pub mod cert;
pub mod lower;
pub mod lower_rt;
pub mod provider;
pub mod rt_runtime;
pub mod runtime;
pub mod seed;

pub use cert::{BoundFile, CCertDraft, CToolchainIdentity, C_CERT_SCHEMA};
pub use lower::{c_fn_name, c_method_name, lower_functions_c, CFn};
pub use lower_rt::{lower_functions_rt, CFnRt};
pub use provider::{
    CTargetProvider, C_BACKEND_VERSION, C_FILESET_SCHEMA, C_PRINT_VERSION, C_RECIPE,
    C_RT_FILESET_SCHEMA, C_RT_RECIPE, C_STANDARD,
};
pub use rt_runtime::record_text_runtime_source;
pub use runtime::runtime_source;
