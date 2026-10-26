# PLAN-841 merge gate repair / affected review

stage: review; review_scope: final (affected); plan_revision: 1; outcome: pass.
Reviewed lang `4a879088519716a45c700588052a4019c3b405ba`; UI `8d6ad2bf362aa8293e58d2734fc54dd42db1da0f`; Man `f5e82892ed6b52bdfe7ca329ef64f0162567f4a8`. Worktree `D:/autostack/.wt/lang-841/auto-lang`, branch `codex/plan-841-dev`, committed and clean. Same Codex session implemented this test-only repair and reviewed its committed artifacts; this is not a fresh independent reviewer. Original independent R4 evidence remains applicable to unchanged production code/Spec contracts. Baseline/config hashes: [baseline](repair-review-baseline.json.txt).

## Failure and correction

Initial merge-owned `cargo t` at lang `693dba083fde69ff8b5ff7385387c1b43df526be`: 659 pass, 5 fail, 2 timeout (3183 selected; fail-fast left 2517 not run). Preserve [original log](lang-t.log.txt) and [gate record](gates.json.txt).

M841-GATE-01 (P2, AC-05 / T-07, T-08, T-09, T-10): five Plan367 view-function cases used Core-only `parse()` after the Plan moved UI grammar to explicit domain admission. Their source programs were valid UI. The helper now calls `parse_domain(&UiDialect)`, retaining all inputs/assertions. Sibling scan found two Plan408 negative cases accidentally passing on generic grammar rejection and one Plan711 targeted outlet-diagnostic case. All four call sites now enter the real UI grammar. No production parser, assertions, timeout, filter or expected output was weakened. Remaining Core-only parser uses in UI-related files were inspected: plain Core source/error-span probes and explicit Core controls remain Core.

Four nested experimental lockfiles gained only ecow's serde edge, matching the Plan's workspace feature; versions/checksums unchanged. Both original timeout tests build/use these probes. No native algorithm or probe limits changed.

## Reconstructed checks

- [32 targeted cases](parser-migration-targeted.log.txt): all pass, including positive view functions, component-only computed/msg rejection, illegal outlet diagnostic and init-demand siblings. Assertions inspected against preserved source.
- [Original two timeouts, serial replay](native-timeout-replay.log.txt): 2 pass (108.094s and 12.047s), unchanged 120s limit. This proves isolated replay only; cold compilation/contention is a hypothesis, not a verified cause. Required bare unified gate remains pending.
- [Formatting](repair-format.log.txt) and diff whitespace: pass.
- Committed repair changes only tests, four dependency lock feature edges, and portable evidence links. AC-01..04 production invariants and approved SD-01..05 are unchanged; R4's unrelated evidence is reused explicitly on this basis. AC-05 affected test selection/entry correctness passes; AC-06 review updated; AC-07 remains merge-owned pending.

M841-GATE-01 CLOSED. No new in-contract implementation blocker found in affected review. Original whole-gate failures remain diagnostics; they are not replaced by targeted totals. Return reviewed 10/12; next run the unified gates on this new frozen baseline, followed by strict 777, real 792 and three-repo distribution checks before landing.
