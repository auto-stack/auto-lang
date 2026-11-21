//! PLAN-976: shared Checked-HIR TypeScript / JavaScript target provider.
//!
//! Consumes ONLY `ace_a2x_core::SemanticView` (the verifier's read-only,
//! target-neutral semantic view) and the verifier-computed resource plans, and
//! emits deterministic ES modules:
//!
//! * `a2x_rt.mjs` — the versioned runtime helper module (shared by both legs);
//! * `a2x_rt.d.mts` — its TypeScript declaration (TypeScript leg only);
//! * `generated.<ts|mjs>` — every Auto function as an exported script function;
//! * `a2x_main.<ts|mjs>` — the entry driver (`a2x_run_main`);
//! * `source_map.json` — declaration → original span → generated lines;
//! * `manifest.json` — producer/backend/runtime/print/port-catalog identity,
//!   function list and per-file sha256.
//!
//! Contract guarantees (T-01 freeze, plan §5):
//! * No legacy `TypeScriptTrans` / `JavaScriptTrans` call, no AutoVM fallback;
//! * `i64`/`u64` are `bigint` end to end (`Number` cannot carry >2^53);
//! * text is UTF-8 bytes (`Uint8Array`), never JS UTF-16 strings, so
//!   `len`/`len_bytes`/`char_at`/slice/NUL fidelity match the Auto contract;
//! * checked arithmetic traps (`trap:Overflow`), C truncating `/` and `%`;
//! * owned `Str` buffers are freed from the verifier's resource plan (no GC
//!   reliance) with a deterministic ledger; composite values are GC-managed
//!   (named target difference, see plan §5 T-01 F);
//! * foreign native declarations and scheduler intrinsics are NAMED rejections
//!   (`a2ts.foreign-unsupported` / `a2ts.sched-unsupported`), never silent
//!   passthroughs.

pub mod cert;
pub mod fragment;
pub mod lower;
pub mod ports;
pub mod print;
pub mod provider;
pub mod runtime;
pub mod seed;
pub mod target;

pub use cert::{
    verify_script_fileset_manifest, ScriptExecExpectation, ScriptExecOutcome,
    ScriptRuntimeIdentity, TS_CERT_SCHEMA, TS_COLL_CERT_SCHEMA, TS_RT_CERT_SCHEMA,
};
pub use fragment::{
    emit_expr_fragment, emit_function_body_fragment, emit_stmt_fragment, innermost_expr_in_span,
    FragmentLocal, FragmentOut, FragmentProgramContext,
};
pub use lower::{
    lower_collections_fn, lower_functions, lower_rt_fn, print_fragment, sanitize, ts_fn_name,
    ts_method_name, ts_record_name, ts_variant_name, ScriptFn,
};
pub use ports::{
    default_stdlib_port_catalog, ParamDirection, PortContract, PortParamSpec, StdlibPortCatalog,
};
pub use print::{print_program, SourceMapEntry, TS_PRINT_VERSION};
pub use provider::{JavaScriptTargetProvider, ScriptProvider, TypeScriptTargetProvider};
pub use runtime::{
    runtime_body_sha256, runtime_export_names, script_runtime_body, script_runtime_declaration,
    script_widget_runtime_body,
    A2TS_RUNTIME_VERSION,
};
pub use target::{
    ScriptLang, BACKEND_VERSION, JS_BACKEND, JS_COLL_RECIPE, JS_RECIPE, JS_RT_RECIPE, JS_STANDARD,
    TS_BACKEND, TS_COLL_FILESET_SCHEMA, TS_COLL_RECIPE, TS_PRODUCER, TS_RECIPE,
    TS_RT_FILESET_SCHEMA, TS_RT_RECIPE, TS_SCALAR_FILESET_SCHEMA, TS_STANDARD,
};
