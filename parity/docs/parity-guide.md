# Parity Verification Guide

The `auto-parity` tool runs three-way consistency checks across AutoVM, a2r
(transpiled Rust), and native Rust for each replicated library.

## How to run parity checks

`--root` and `--auto-binary` are global flags and must come **before** the
subcommand (`run` / `phase` / `all` / `list`). From the `parity/` directory:

### Single library
```
cd parity
cargo run -- --root . --auto-binary ../../target/debug/auto.exe run _dummy
```

### By phase
```
cargo run -- --root . --auto-binary ../../target/debug/auto.exe phase p0
```

Phase mapping (Plan 347):
- `p0`: `_dummy` (framework smoke test)
- `p1`: `base64`, `url`
- `p2`: `serde_json`, `regex`
- `p3`: `sha2`, `rusqlite`
- `p4`: `reqwest`, `tokio`

### All libraries
```
cargo run -- --root . --auto-binary ../../target/debug/auto.exe all
```
Note: `all` skips `_dummy`, which is a framework smoke test, not a real
library under test. Use `phase p0` to run `_dummy` explicitly.

### List discovered libraries
```
cargo run -- --root . list
```

## How to add a new library

1. Create `libs/<name>/` with:
   - `auto/<name>.at` — Auto replication
   - `tests/auto/<scenario>.at` — Auto test cases (TAP output)
   - `tests/rust/Cargo.toml` + `tests/rust/tests/<scenario>.rs` — Rust native tests
   - `README.md` — replication scope, upstream version, known divergences

2. The `tests/rust/Cargo.toml` must keep itself out of the parity workspace by
   including an empty `[workspace]` table:
   ```toml
   [package]
   name = "<name>-tests"
   version = "0.1.0"
   edition = "2021"

   [dependencies]

   [workspace]
   ```

3. Auto tests must import the library via `use auto.<name>: ...` and print TAP:
   - Success: `ok N - test_name`
   - Failure: `not ok N - test_name # got X expected Y`

   The parity runner executes the test with the working directory set to the
   library root (`libs/<name>/`), so the library at `./auto/<name>.at` resolves
   as the module path `auto.<name>`.

4. Run:
   ```
   cargo run -- --root . --auto-binary ../../target/debug/auto.exe run <name>
   ```

## Bug classification

Per design spec §2.2.5, each test case is classified from the three-way
(AutoVM, a2r, Rust) pass/fail result:

| AutoVM | a2r | Rust | Classification |
|--------|-----|------|---------------|
| pass | pass | pass | consistent |
| pass | pass | FAIL | replication bug |
| pass | FAIL | pass | a2r transpiler bug |
| FAIL | pass | pass | AutoVM bug |
| FAIL | FAIL | pass | replication bug (VM and a2r agree but wrong) |
| FAIL | FAIL | FAIL | test case issue (manual review) |

Any combination with a missing backend (a backend that produced no result for a
test case) is classified as a **test case issue** for manual review.
