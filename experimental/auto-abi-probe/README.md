# auto-abi-probe — PLAN-797 first AutoABI candidate profile + interop probe

Standalone experimental workspace (own `Cargo.lock`; not a member of any
product build graph). It freezes the first **bounded, candidate** Windows
x64 AutoABI profile and proves three REAL native interop legs with
independently produced objects:

- **Leg A** AutoABI caller → AutoABI callee (two separately emitted
  Cranelift objects, one PE),
- **Leg B** AutoABI caller → C callee compiled by a real MSVC `cl.exe`,
- **Leg C** C caller (own entry, own oracle) → AutoABI callee.

Linking uses the merged ac-core entries (`find_rust_lld`,
`find_sdk_um_dir`, `link_object_staged`, `run_deadlined_split`); the
first layout/object cache is the merged 783 managed store under
probe-owned layers (`ABI-L0` …), with the 783/787 registry untouched.

## What this is NOT

- Not an Auto-source compiler: modules come from a frozen declaration IR
  (`src/decl.rs`), never through an Auto parser.
- Not a frozen ABI: `autoabi.win-x64.v0.1-candidate` becomes a freeze
  only after independent review + merge gates (plan §0).
- Not a C header importer, Rust shim product, loader, or hot-reload
  host; the PLAN-774 internal ABI-A–E surfaces are untouched and no
  compatibility with them is claimed.
- The semantic-ref snapshot (`src/semref.rs`) is a temporary 797-local
  adapter; it is retired at D-796 when PLAN-796's semantic contract
  lands (plan §5.6).

Frozen decisions live in
`docs/plans/attachments/797/contract-decision.md`. Scope/non-goals:
`docs/plans/797-autoabi-v01-bidirectional-probe.md`.

## Running

```
cargo test            # unit + T-01 feasibility legs (needs MSVC cl.exe)
```

A missing C toolchain is a controlled infra blocker, not a skip.
