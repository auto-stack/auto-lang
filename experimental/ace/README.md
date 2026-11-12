# ACE workspace

ACE implementations live in this independent Cargo workspace. Member crates keep their existing responsibilities and dependency directions; the language workspace has no implicit ACE test build.

- From the repository root, `cargo t ace` runs the regular ACE collection and the language SDK's private protocol controls.
- `cargo t ace-perf` explicitly selects formal timing, sustained scale/GC and measurement controls using the serial `ace-perf` nextest profile (`test-threads = 1`). Ordinary package groups exclude `::performance::`. Samples, sizes, assertions, ignored and platform conditions remain intact.
- The language `cargo t all` covers the language fixed groups only; every ACE group here is excluded from `all` (opt-in `all: false`) and is entered explicitly through the `ace`/`ace-*` groups above. `all` also excludes external UI/Man ownership.
- `cargo t --list` reads the fixed group configuration without discovery or building.

Each package has a small number of integration targets with semantic Rust modules. Private unit tests remain in the implementation modules. `tests/conformance` owns the former language-host ACE consumers. Five no_std static libraries (`ace-collections-runtime`, `ace-data-runtime`, `ace-text-runtime`, `ace-trace-runtime`, `ace-typed-entry-runtime`) are built separately and checked through their real ABI consumers.

Normal dev/test keeps line tables for workspace members (`debug = "line-tables-only"`) while non-workspace dependencies build with debug off (`profile.*.package."*"` `debug = false`; the wildcard never matches workspace members). This adds member line-table debug information relative to the previous `debug = 0` — a deliberate diagnostics gain, not a size reduction. `full-debug` opts members into `debug = 2`; dependency debug stays off under it until the wildcard override is explicitly adjusted — no single environment variable overrides the wildcard. `syntax-release` preserves the former syntax release debug information. `runtime-dev` / `runtime-release` preserve the native static libraries' panic-abort contract; `runtime-dev` inherits the new dev line tables. Regular tests/workers share this tree's `target`; batch workers use `target-batch`. No writable target is shared across worktrees.

Migration, original obligations and authorized identical-source aggregation are recorded in the archived plan `docs/plans/archive/896-merged-invocations-and-consumer-closure-audit.md`; this is not a runtime test catalog.
