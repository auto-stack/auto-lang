# Parity Verification Guide

The `auto-parity` tool runs three-way consistency checks across AutoVM, a2r
(transpiled Rust), and native Rust for each replicated library. For a
categorized index of every library and category, see the
[top-level README](../README.md).

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

Phase mapping (Plans 347/358/367/368/369 — source of truth is the
`discover_libraries_by_phase` table in `crates/auto-parity/src/main.rs`):
- `p0`: `_dummy` (framework smoke test)
- `p1`: `base64`, `url`
- `p2`: `serde_json`, `regex`
- `p3`: `sha2`, `rusqlite`
- `p4`: `tokio`, `tokio_stream`
- `p5`: `py_math`, `py_random`
- `p6`: `py_datetime`, `py_struct`, `py_uuid`
- `p7`: `py_configparser`, `py_hashlib`, `py_json`, `py_list`, `py_os`,
  `py_re`, `py_string`, `py_sys`
- `d1`: `cli_app`
- `d2`: `trait_advanced`, `generators`
- `d4`: `string_utils`
- `d5`: `c_fs_app`, `c_env_app`, `c_process_app`, `c_text_app`, `c_json_app`
- `d6`: `http_client_sync`, `c_http_get`, `c_wget`, `c_crawler`

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

### AAVM four-way matrix (Plan 433)

Runs the AAVM self-hosting matrix over the v2 execution-layer corpus
(`crates/auto-lang/test/vm/aavm2/corpus_m4`, 30 cases):

| col | backend | meaning |
|---|---|---|
| 1 reference | `auto <case.at>` | auto-lang native implementation (oracle) |
| 2 aavm_rust | built binary | auto/lib v2 transpiled by a2r `--merge` + compiled (zero a2r_std) |
| 3 aavm_vm | `auto` (merged lib + ev_run wrapper) | AutoVM interpreting the AAVM `.at` pipeline |
| 4 golden | `<case>.expected.out` | corpus golden; falls back to 1 when absent (noted) |

```
cd parity
cargo run -- --root . --auto-binary <abs path to auto.exe> aavm
cargo run -- --root . --auto-binary <abs> aavm --html docs/aavm-matrix.html
```
The 2nd backend binary is built on demand (cargo, content-hash cached under
the system temp dir). Exit code 0 iff the whole matrix is green. Requires a
Rust toolchain.

## How to add a new library

Libraries live in categorized two-level layout `libs/<category>/<name>/`
(Plan 460). Existing categories: `framework` (runner smoke), `lang` (language
features), `python` (Python stdlib parity), `consumer` (consumer apps),
`rust` (crate replication). A new category is just a new directory — no tool
changes needed. The **library identity is the leaf name** `<name>`: it is the
CLI argument, the TAP/test name, and the module name under `auto/`.

1. Create `libs/<category>/<name>/` with:
   - `auto/<name>.at` — Auto replication
   - `tests/auto/<scenario>.at` — Auto test cases (TAP output)
   - `tests/rust/Cargo.toml` + `tests/rust/tests/<scenario>.rs` — Rust native tests
   - `tests/python/test_<name>.py` — Python oracle (python-parity libs only;
     its presence switches the lib to Python parity mode)
   - `mock-server/` — standalone HTTP fixture (HTTP consumer libs only)
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
   library root (`libs/<category>/<name>/`), so the library at
   `./auto/<name>.at` resolves as the module path `auto.<name>`.

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
