# ACE workspace

ACE implementations live in this independent Cargo workspace. Member crates keep their existing responsibilities and dependency directions; the language workspace has no implicit ACE test build.

- From the repository root, `cargo t ace` runs the regular ACE collection and the language SDK's private protocol controls.
- `cargo t ace-performance` explicitly selects the original formal timing, sustained scale/GC and measurement controls. Samples, sizes, assertions, ignored and platform conditions remain intact.
- Named complete package groups and `cargo t all` retain the complete applicable package tests, including that namespace. `all` also includes the language groups; it excludes external UI/Man ownership.
- `cargo t --list` reads the fixed group configuration without discovery or building.

Each package has a small number of integration targets with semantic Rust modules. Private unit tests remain in the implementation modules. `tests/conformance` owns the former language-host ACE consumers. Four no_std static libraries are built separately and checked through their real ABI consumers.

Normal dev/test uses debug=0; `full-debug` opts into debug=2. `syntax-release` preserves the former syntax release debug information. `runtime-dev` / `runtime-release` preserve the native static libraries' panic-abort contract. Regular tests/workers share this tree's `target`; batch workers use `target-batch`. No writable target is shared across worktrees.

Migration, original obligations and authorized identical-source aggregation are recorded in `docs/plans/attachments/896/ace-migration-summary.md`; this is not a runtime test catalog.
