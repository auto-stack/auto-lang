//! PLAN-921 CA-A: the C emit target (HIR → C11, scalar channel only).
//!
//! Lowers the shared [`SemanticView`] to deterministic C11 source —
//! explicit checked arithmetic with named traps via an inline runtime
//! header, source-order single evaluation of call arguments pinned into
//! temporaries, short-circuit `and`/`or` as branch-guarded temporaries,
//! `for(;;)` loops with C `break`/`continue`, and hygienic `fn_`/`mt_`/`lN_`
//! naming — then assembles the full emit file set (`main.c` wrapper +
//! `a2x_c_runtime.h` + `generated.c` + manifest).
//!
//! Contract (plan §5.1/§5.2): the C channel is its own namespace
//! (`ac-a2c/hir-scalar-emit-v1`, `auto.a2x.c.fileset/1`, layer AXC1) —
//! a Rust fragment/cert/fileset can never satisfy a C identity or vice
//! versa. The semantic facts (AXS1 keys, fingerprints) are SHARED with the
//! Rust target; only the fragment layer is C-private.

pub mod lower;
pub mod provider;
pub mod runtime;

pub use lower::{c_fn_name, c_method_name, lower_functions_c, CFn};
pub use provider::{CTargetProvider, C_FILESET_SCHEMA, C_PRINT_VERSION, C_RECIPE, C_STANDARD};
pub use runtime::runtime_source;
