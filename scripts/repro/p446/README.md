# Plan 446 historical UI repros

The preserved b3check/j2check exports are in `test/fixtures/plan446/`.
The five probes default to the tracked b3check Auto project and the `auto`
command from PATH. Set `AUTO_BIN` to an explicit build and
`P446_CORPUS_ROOT` to another exported Auto project when comparing versions.

From the repository root:

```sh
node scripts/repro/p446/vm-b3-check.mjs --check
node scripts/repro/p446/vm-b3-check.mjs 9385
```

`--check` validates and reports the resolved corpus without starting an app.
Normal runs start a VM window and drive its MCP interface; use a free MCP port.
These are historical investigation tools, not a substitute for the daily tests.
The headless Rust repro is `plan446_j1_repro::b3_child_widget_expansion` (explicitly
ignored); run it with `--ignored --nocapture` when inspecting this fixture.
