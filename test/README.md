# Repository test assets

`test/` contains feature fixtures, retained regression exports, and language-level
test runners. Rust crate-specific golden suites remain under
`crates/auto-lang/test/`; Rust tests remain in their crates.

Run the language-level entrypoints **from the repository root** so their golden
suite paths keep their meaning:

```sh
auto test --dir test/auto
auto test --dir test/runners/a2c_tests.at
auto test --dir test/runners/a2r_tests.at
auto test --dir test/runners/a2ts_tests.at
auto test --dir test/runners/vm_tests.at
```

These runners select their own suites; they are not a replacement for the
change-scoped gates in AGENTS.md. `ui/vue-gen-gap/` documents its generated Vue
fixture workflow. Plan 446 exports live in `fixtures/plan446/` and their optional
MCP probes in `scripts/repro/p446/`.

Before PLAN-761, the runners and `auto/` were in root `tests/`, and the Plan 446
exports in root `tmp-corpus/`. Historical receipts may still describe those paths.
