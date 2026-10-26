# PLAN-854 work readiness

Outcome: pass for work-owned AC-01..05, T-01..05; execution_done5/9. Independent review and merge gates remain pending. Authoritative tree D:/autostack/.wt/lang-854/auto-lang; branch plan-854-dev; base c1961665d1a90a8705994085fd4344a1d6a9a564; final523d27b863dbe306c89af0ea546334225a29f8e9. SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 /681files, AGP and both locks/API/samples unchanged. No root Rust/production code changes. Implementation tree clean and frozen; source only eventually lands autographics-dev.

## Acceptance and distinguishing evidence

| AC / V | Actual path / oracle | Result and boundary |
|---|---|---|
| AC-01 / V-05 | Delivered853 -> own pre-change build -> current447-file verifier -> package export with own681 SDK | 21 original output files identical before refactor; complete current closure accepts, missing assembly/SDK/lock/asset, added source and wrong SDK reject, restored accepts. Source tools also retain root build/config/auto-target rejection. Historical manifests/verifiers unchanged. |
| AC-02 / V-01 | Api -> exhaustive assemble -> admission/namespace -> snapshot raw/credential/region hashes | Ten frozen actual853 LF/CRLF Unicode/local-reuse fixtures equal all serialized fields/profile/lengths and independent combined/user hashes. Whitespace drift changes oracle. Arbitrary legal inputs remain limited to supported profile, not universally proven by ten fixtures. |
| AC-03 / V-02 | Private trusted Inputs -> assembly before admission/compile/Play | Four permanent methods cover missing/duplicated selected components; altered/duplicated transport/setter/segment/entry boundaries; derive axis/visibility unit changes/duplication; wrong order; compact literals; actual restoration. No fallback/partial success. No production mutation API or new public feature. |
| AC-04 / V-03/04 | Original actual Parser, Session, Play, GPU Host/project consumers | 43 tests in8 named binaries +8 project Host/session tests; existing callback/constant/local/nested-return/diagnostic/bytes/selection, token/motion,128-wide/deep, budget/sticky/last-good/Stop/newRun, binding/history/origin and20 owner-cycle assertions retained. Byte-identical final runtime reuses those inputs after cfg(test)-only followup. |
| AC-05 / V-05 | Exported own-SDK/initially empty target -> actual tests | Consumer original behavior12 +identity1 +motion API4 (including all five actual VM headers) +assembly4,21 unique cases. No producer target/rlib copied. Final cfg(test)-only module+identity update independently verified/recompiled in same consumer; update proof in consumer-update.json.txt. |
| AC-05 / V-05 | Same853 baseline tool -> frozen runner50 real processes1500 warmticks | All50 semantic/source/pixels/resources/owner records equal853 and observer off/on. Ten generated exports equal originals. Same machine/adapter/toolchain/locks/flags/inputs; raw timings, memory and named boundaries are paired in performance-comparison.json.txt. No speed claim. |

Actual commands/selected IDs/log SHA: scoped-receipts.json.txt. Full code/dependency/source binding and all logs: handoff.json.txt. Baseline raw originals, reports, immutable runner and consumer target: D:/autostack/plan-artifacts/plan-854/. All capture/target/SDK data stays outside Git. New source tests7 pass; final source controls/check/package fmt/diff check pass. Fixed SDK146 existing warnings, driver0new.

## Performance and resource comparison

Same debug Windows configuration, five fresh processes per mode/workload, five warmup +30 accepted measured ticks per process; nearest-rank quantiles. New process does not imply cold disk cache. Working set is sampled10ms, with short missed-peak limitation and OS peaks retained separately. Different time-of-day/background/caches limit attribution. CPU boundaries overlap and must not be added. Compile_us units remain microseconds in JSON.

| Workload | Tick P50 old/new ms (off) | Tick P95 old/new ms (off) | Generate P50 old/new ms | Compile P50 old/new ms |
|---|---|---|---|---|
| move | 2.577/2.612 | 3.636/3.730 | 8.475/8.919 | 30.896/31.529 |
| maze2d | 3.941/4.006 | 5.165/6.078 | 48.113/48.877 | 113.331/118.461 |
| corridor3d | 4.175/4.214 | 5.268/5.449 | 48.181/50.727 | 114.714/120.255 |
| wide128 | 35.520/35.645 | 38.077/38.520 | 50.789/52.460 | 111.724/125.241 |
| deep128 | 32.937/34.052 | 34.834/54.133 | 50.611/52.697 | 113.784/117.517 |

Observed timings are slightly higher; this work claims maintainability/compatibility, not acceleration. All original hard budgets pass. Working set raw per-process values, read/init/Run/Stop, renderer/pump/admission timing and resource/owner records remain in the linked JSON; logical VM storage is unchanged. No tolerance or budget raised.

## Evidence reuse and remaining stage ownership

Measured commit16fea18a8 -> final523d27b86 adds only a cfg(test) derive counterexample and its identity hash. Verified production prefix before cfg(test) is byte-identical, all other447-file identities, SDK/config/locks/samples fixed; handoff.json.txt binds that proof. Thus measured original runner/50 reports/1500 ticks, identity and43 behavior/8 project cases remain valid, with new test run on final code. Runtime unchanged does not claim future complete gates ran.

Proposed SD-01/02 is research/behavior-assembly-spec-delta.md, SHA bound in handoff.json.txt. Canonical files/ledger are not published by work. Independent final review must verify all work AC/V/SD and omission/workaround/warnings/fmt. Merge owns full producer/fresh consumer and original native/owner/content/AGP/current-source gates per common contract; original85-group mapping remains853's durable input, not a new test total. Nothing is merged/archived/cleaned here.

Open in-contract findings: none. Required work coverage gaps: none. Unresolved contract decisions: none. No independent review pass claimed.

## Attempts and resolved local execution issues

The first entry replacement used return1 after installing profile10 and was rejected by exact-count check (assembly-first.log); full function-declaration boundaries fixed it, then all identities passed. cargo fmt --all hit SDK nested workspaces; package-only fmt is correct and passed. Initial closure generation used an unresolved relative parent, so identity admission correctly failed until the path was resolved; final source closure/controls/consumer all pass. Two local evidence-script edits failed before writing (substring/argument mismatch); corrected before their evidence was accepted. No SDK/lock/algorithm/budget/oracle weakening or test deferral resolved these issues. Original failing logs that exist are indexed, and tooling failures above are recorded as summaries rather than manufactured raw logs.
