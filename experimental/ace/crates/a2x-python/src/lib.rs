//! PLAN-1042 minimal A2X Python target provider & stdlib/runtime port readiness.
//!
//! Consumes ONLY the shared Checked-HIR [`ace_a2x_core::SemanticView`] and
//! verifier-computed resource plans (`BodyResourcePlan` / `CollectionsBodyFacts`),
//! lowering admitted modules (including multi-module packages with `std.math`
//! and `std.text`) to deterministic Python 3 modules (`generated.py` +
//! `a2x_main.py` + versioned `a2x_py_runtime.py` + `source_map.json` +
//! `manifest.json`).
//!
//! Contract guarantees:
//! * Strictly independent identities (`auto-a2x-python`, `AXPY1`/`AXPY2`/`AXPY3`,
//!   `ac-a2py/*`) — never aliased onto Rust/C namespaces, legacy `PythonTrans`,
//!   or AutoVM fallback.
//! * Explicit checked `i32` arithmetic & truncation-toward-zero `/` / `%`
//!   (never Python floor division for signed operands).
//! * Byte-exact UTF-8 (`str` as `bytes`, `Str` as explicit-lifecycle buffer with
//!   embedded NUL preserved) and Unicode scalar `char` semantics.
//! * Explicit verifier-scheduled `free` / port resource `close` lifecycle with
//!   deterministic state transitions and ledger trace — never relying on Python GC.
//! * Standard-operation port bindings keyed by contract operation ID, semantic
//!   version, parameter modes, and runtime dependency SHA-256; helpers never
//!   hardcode test expectations.

pub mod cert;
pub mod foreign_bridge;
pub mod lower;
pub mod ports;
pub mod print;
pub mod provider;
pub mod runtime;
pub mod seed;

pub use cert::{
    verify_python_fileset_manifest, PythonExecExpectation, PythonExecOutcome,
    PythonRuntimeIdentity, PY_CERT_SCHEMA, PY_COLL_CERT_SCHEMA, PY_RT_CERT_SCHEMA,
};
pub use lower::{
    lower_collections_fn_py, lower_functions_py, lower_rt_fn_py, py_fn_name, py_method_name,
    py_record_name, py_variant_name, sanitize, PyFn,
};
pub use ports::{
    default_stdlib_port_catalog, ParamDirection, PortContract, PortParamSpec, StdlibPortCatalog,
};
pub use print::{print_program_py, SourceMapEntry, PY_PRINT_VERSION};
pub use provider::{
    PythonTargetProvider, PY_BACKEND, PY_BACKEND_VERSION, PY_COLL_FILESET_SCHEMA, PY_COLL_RECIPE,
    PY_FILESET_SCHEMA, PY_PRODUCER, PY_RECIPE, PY_RT_FILESET_SCHEMA, PY_RT_RECIPE, PY_STANDARD,
};
pub use runtime::{python_runtime_source, runtime_body_sha256, PY_RUNTIME_VERSION};
