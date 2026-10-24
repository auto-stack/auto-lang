# PLAN-840 r3 / Phase 2 work closure

Frozen commit: 7778f3192d801e67ce95dc1f8fc1dfa47b50803d; base 980a6708dacf843af2a84dd390da6ac4dce1619b; same lang-840 tree and auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8.

| Finding / invariant | Repair / affected controls | Result / boundary |
|---|---|---|
| CTE imports optional Case dependencies in feature-off test build | tests.rs feature guard matches AV/Case/probe; no Cargo/feature defaults changed | default --tests and book-feature --tests checks pass; 5 CTE cases execute, including wrong exit, unknown recipe, bogus worker, stale identity |
| Parallel wall-clock oracles observe unrelated test scheduling | exact two IDs reserve num-test-threads in default/full/t3; input, assertions and product code identical | same 8 scoped cases pass at -j6; census full513ms/window390ms, IPC 30 samples P95=263ms; window tiling accepted control passes. Whole-tier confirmation remains merge-owned |
| Selection support must not shrink | all old aliases/filters/config preserved; only scheduler weights added; current on-feature CTE IDs 5/5 | three configs parse and select identical8; historical814 nine-tier mappings and actual deletion controls pass. This is not a claim current nine tiers executed |

Evidence: scoped-results.json.txt, scoped-warm-result.json.txt, scoped-warm-stderr.txt, turnaround-ipc.json.txt, selection-audit.json.txt, config-parity.json.txt. Both original red runs and aborted cold-worker run retained. Exact root and standalone worker locks reused; no new dependency resolution accepted.

Health: diff whitespace clean; tests.rs rustfmt --check --config skip_children=true passes (two preexisting neighboring comment indentations normalized). Existing compile warnings are outside changed executable code; no new warning-producing executable symbols added. No debug output or dropped/ignored tests. Proposed canonical delta is the two committed project.md additions.

Original Phase 1 AC/support and graphics/SDK evidence reused only because all product source, SDK pin, fixtures and relevant graphics configuration are unchanged; autographics-dev stays8b62db5cba54b9bfe7f759f4920b22ccb9b75e88. No new graphics tests run.

No work-owned open finding. Full cargo t, touched configuration tiers and main-only batch tf/tt/tb remain pending, merge-owned. No delivered/all-green claim from scoped results.
