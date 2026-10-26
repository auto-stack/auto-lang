# PLAN-852 r2 — T-07 final review R5

**Outcome: pass.** Code, work-owned acceptance and proposed knowledge delta pass. **Merge-owned gates remain pending; no merge or full-M1 delivery claim.**

## Baseline and independence

Reviewer: same independent R1–R4 review context, not implementation context; no agents, implementation edits or merge. Existing tree `D:/autostack/.wt/lang-852/auto-lang`, branch `codex/plan-852-dev`, actual reviewed HEAD `826e3472c04b4e20b5db7fea5e8c109d694eb2db`. User-frozen implementation `d3b4fe90e`; affected code/script/auto-test-case drafts identical between that commit and actual HEAD. Repair comparison baseline `0cc4e2f7e808ed4f3396db954546cc4e05d682b7`. Dependency commits unchanged: auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`, auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. Tracked implementation clean before/final baseline checks; original untracked probe-t01 retained.

Reviewed the main Plan's mechanism diagnosis: R3/R4 had followed listed examples rather than inventorying all declared scalar fields; current repair adds explicit string guards and a field-list-driven matrix, and extracts one daily validator shared by real snapshot and a direct diagnostic seam. This is an actual coverage adjustment, not just another original-example patch.

## Targeted evidence and closure

1. Replayed unchanged review-r4/replay-array.ps1 against current tree (r4-replay.txt): case/module single-element array receipt rejected by Rust exit1 and full real PS5.1 snapshot exit1. **F-852-R4-01 closed.**
2. Independently constructed validator-matrix.ps1. It parses the exact production verifier AST and loads unmodified Load-Binding/Sha256File/Sha256Text/Invoke-ReceiptValidation function bodies. Only Fail is replaced by a throwing unit-test boundary to collect outcomes; no fabricated suite report or acceptance banner is used. Uses an isolated COPY of the corpus.
3. Matrix covers20 declared string fields ×5 wrong types, including arrays containing the ORIGINAL matching scalar (to exercise PowerShell collection-filter hazards, not merely wrong values), objects/null/number/bool. Five additional regressions cover negative created_at,string exit,flags null element,external null/number entries,unknown produced_by. **105/105 reject; unchanged valid control accepts10 certificates** (ps-matrix.txt). Production script SHA256 `1b1fb3ba955f91980d98d78d655d42b0eb943f95a95f401b46d7c0b35a0a2652`.
4. Rust test `test_cases::ace_certify::tests::invalid_receipts_are_rejected_in_memory` selected exactly1/pass1; inspected its per-field matrix and prior negative cases (rust-schema.txt). No reliance on a zero-test success.
5. Normal snapshot on committed corpus under documented Windows PowerShell5.1 passes:26 a2r checks,10 strictly valid receipts,97 full-suite checks,exit0 (valid-snapshot.txt). Real daily entry calls the exact shared validator tested above. Snapshot and receipts do not dispatch certification.
6. Added Rust formatting check exits0; changed implementation diff whitespace clean (fmt.txt). No new production Rust behavior beyond comments in this repair. The previous stream/identity/cleanup implementations and prior valid evidence are unchanged.

Scope is F-R4-01/shared strict scalar checks and delta correction. Not a new global audit cycle. Existing R2/R3 actual compiler identity, scratch early-error cleanup, R2 stream-order/overflow/wall, R1 implicit certification removal, fixed oracle/source/lifecycle/denominator evidence remain reusable: affected code/deps/input/config identities inspected unchanged. Previous raw logs are historical claims, not substitutes for these checks.

## Findings ledger / scope boundaries

R1 receipt identity/strictness,implicit certification,stream governance and durable handoff findings: closed through their recorded R2–R5 evidence chain. R2 actual compiler labeling,wrong-type parsing,early cleanup and formatting: closed. R3 produced_by nested unknown-field rejection: closed and retained in current matrix. R4 scalar collection bypass: closed. No remaining in-contract blocking findings identified on the reviewed baseline.

Sampling observer remains explicitly limited to periodic CIM ancestry observation; not claimed lossless. Build-default profile and explicitly measured certification compiler identity are distinct. The diagnostic receipts scope requires an externally supplied identity and is not standalone proof of execution/M1 completeness. Certification only proves the frozen supported profile. Unsupported native/collection/closure/Dyn/online-HMR capabilities remain downstream M1 gaps, not silently counted green or newly demanded of this testing Plan.

## Acceptance map

| AC | Result | Revision-bound evidence |
|---|---|---|
| AC-01 | pass | current full snapshot97 checks; retained fixed58+5 source/typed identities and four-scalar baseline; corrected actual compiler certification evidence from R3 |
| AC-02 | pass | R2 independently inspected43 values/9 diagnostics/6 traps and all sources/references,unchanged; explicit legacy semantic/capability gaps retained |
| AC-03 | pass | real emitted actual==golden,actual/reference separate runs; R3 nightly counterexample closed; current10 receipts valid on the frozen supported context |
| AC-04 | pass | shared daily validator105 negative controls + valid10; full daily snapshot pass; no implicit certification; process sampling limitation explicit |
| AC-05 | pass |63 corpus+40 scenario registry intact; base-layer/M1-completeness conclusions remain separate; selectors do not reduce denominator |
| AC-06 | pass | existing v1/v2 decoder/recipe contracts unchanged; new exact-key/type checks inspected and negative matrices through both consumers |
| AC-07 | pass | retained real120/720 session/cache observations; R2 streams3pass and R3 identity/cleanup1pass; unaffected implementation/input identities unchanged |
| AC-08 | pass | independent revision-bound chain,omission/defer/workaround scan,closed findings and frozen SD; explicit pending gates below |
| AC-09 | pending merge | frozen-commit gates,canonical deposits/ledger/index/archive/guard/cleanup not performed by review |

## Reviewed knowledge delta and closeout

Frozen final delta: spec-delta-reviewed.md,including precise SD-03/04/05 deposit text and diagnostic receipts/navigation/sampling clarifications. Frozen draft copies beside this report: SD-01 SHA256 `9f668f89f179ee63979d40b11eea0c106f124302d0a1ea5808248f6bc2f405b6`; corrected SD-02 SHA256 `9dbc7d704ff12e3e99a2eda1d46b305acc711b9122fa53e9d53eaadf23772af4`. contract-decision hash `c0f6fe811c57adfe28fddd7d481440c88aa0865a6414d5af5df49496d186b75ea`. Current canonical target hashes: canonical-input-hashes.txt. Main Plan frontmatter exact paths and empty-goal rationale reviewed; no semantic-contract change or revision bump. The stale SD-02 resource paragraph and BUILD_TOOLCHAIN_IDENTITY comment now accurately separate actual compiler measurement from harness default side axis. No canonical Spec or live ledger publication by review.

Next: T-08 merge. **Pending merge-owned gates: bare cargo t on the frozen implementation plus any surface tier required by the actual final diff.** The852 Rust changes are testing/certifier/build identity plumbing,not changes to VM/compiler/transpiler/UI generation/aavm implementations; no extra whole tier is inferred solely from invoking these paths in tests. Merge owner verifies final diff,dependency/config/input identities and applies repository gate rules. cargo tf remains batch-only main/single-instance when due. No whole-tier t/tv/tt/tb/tu/taa/tf or docs_gen executed by this review. If code/dependencies/config/inputs change,the affected evidence requires reconciliation/re-review before landing. Do not repeat already valid targeted evidence merely because ownership changes.
