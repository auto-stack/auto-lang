# PLAN-837 r1 final R2 — PASS

stage: review | review_scope: final | outcome: **pass** | 2026-10-23 (Asia/Shanghai)

**F-837-R1-01/P2 CLOSED. No new blocking finding.** Plan may enter
**reviewed9/10**; the complete frozen merge gates and actual T-10 closeout remain
**pending**. This review does not establish delivery or permit landing before
those gates pass.

Reviewed commit: `2ffc92d34143bc97a98f2684406160b183fd2ff3`.
Prior reviewed commit: `abe682c5a1144f7e9b72f4ffeb95ece19dd02841`.
Base: `52ec4c7c13b337992278b343ef89d8aa17102ff6`.
Same authoritative tree: `D:/autostack/.wt/lang-837/auto-lang`, `plan-837-dev`.
Plan revision remains1: this restores the existing V-10 requirement without
changing the supported profile or acceptance contract.

Reviewer: Codex in this existing review context, independent of the repair handoff.
No extra agent/chat was created; no unexposed model/session identity is asserted.
Source, test assertions and fresh native/GPU results were checked independently.
Entry and verdict checks show unchanged HEAD and clean implementation tree.
[Baseline and Spec/dependency/input hashes](baseline.json.txt).

## Finding closure and repair regression checks

| Check | Independently observed result | Evidence |
|---|---|---|
| Original R1 counterexample | Same GPU mutation/input body, original SHA256 `60727913a42181040dd0b9041d6799899930386e4a1a7efb38b1f9757dddbd7a`; only the expected HEAD guard rebound. Native now exits1, passed=false, phase9 at the move-pixel comparison; the old positive expectation/aggregate rejects | [Replay entry](replay-original.py), [result](original-replay/result.json.txt), [raw failure](original-replay/replay.log.txt) |
| Correct normal workflow | phase16,204frames, exit0/passed=true; exactly7 actual pixel records: temporary/move/undo/redo/hidden/show/saved | [Native results](native/results.json.txt), [positive report](native/positive.json.txt) |
| Actual transparent endpoint | Correct Auto/state/inspected commands preserved; GPU endpoint is background. Native/report/aggregate reject at phase9, `move: composed endpoint/origin pixels mismatch` | [Permanent control results](pixel-faults/results.json.txt), [transparent report](pixel-faults/transparent/native/positive.json.txt) |
| Actual vacated-position ghost | Endpoint retains correct leaf color, but origin also has leaf color. The other side of the same pixel oracle rejects, phase9/native1/reportfalse/aggregate1 | [Ghost report](pixel-faults/ghost/native/positive.json.txt), [actual GPU capture](pixel-faults/ghost/native/positive.png) |
| Existing negatives | wrong-parent phase5, missing release phase9 timeout, frame1 incomplete; all exit1/passed=false with expected diagnostics | [Native results](native/results.json.txt) |
| New-process saved reconstruction | Exact document/hash/Auto world and unselected2D+3D captures match; history[0,0,0], selectionnull, dirtyfalse | [Reopened result](native/reopen.json.txt) |
| Completion/CLI regression | 3 passed,0 failed,0 ignored; incomplete/normal-return/report/exit checks remain valid | [Scoped tests](scoped.log.txt) |
| Health/source | all-targets check, fmt and diff whitespace pass; source247/SDK680/closure479/17local/29edges verified in producer and existing independent repair consumer | [check](check.log.txt), [fmt](fmt.log.txt), [source](source.log.txt), [external source](external-source.log.txt) |

`Probe::pixels` reads the final composed top texture at both actual fixture
coordinates. It requires nonempty, distinct initial leaf/background controls,
compares each readback with `Some(expected)` (so missing readback fails), and
records actual/expected RGBA before reporting failure. Each of the seven call
sites was inspected: Undo reverses the two colors; hidden requires background at
both; other visible moved states require origin background and endpoint leaf color.
The aggregate requires exactly those seven distinct records and retains saved
new-process verification. It cannot substitute matching hashes for these pixel
comparisons.

The records were also audited with stage expectations independently reconstructed
from initial colors, rather than trusting the reported expected fields alone.
Leaf RGBA `[243,149,124,255]`, background `[39,48,56,255]`; all7 states agree.
[Independent assertion audit](assertion-audit.json.txt). Normal and injected ghost
captures were visually inspected. Transparent/ghost controls assert actual GPU
mutation before restoring inspected commands; raw aggregate errors show native
exit1, not merely a failed compilation or missing report. No producer source was
modified by these test-only copies.

Fresh commands, from the authoritative driver directory:

```powershell
$env:RUST_MIN_STACK="16777216"
cargo test --locked --bin autographics-authoring --test authoring_probe_exit -- --test-threads=1
cargo check --locked --all-targets
cargo fmt -- --check
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/check_hierarchy.py D:/autostack/auto-lang/docs/plans/attachments/837/review-r2/native
python tools/check_hierarchy_pixel_faults.py D:/autostack/auto-lang/docs/plans/attachments/837/review-r2/pixel-faults
python D:/autostack/auto-lang/docs/plans/attachments/837/review-r2/replay-original.py D:/autostack/.wt/lang-837/auto-lang/experimental/autographics-auto-driver
python tools/verify_source.py
python tools/verify_plan837.py
```

## Acceptance and evidence reuse

The complete nine-file repair diff was inspected. It changes only the native
hierarchy probe, two checking tools, the source manifest and explanatory documents.
There is no authoring library/test/sample, AGP, SDK, root production code, dependency
lock, game or asset change. **191** unchanged library/test/sample/AGP inputs were
compared byte-for-byte after documented newline normalization; hashes are in the
baseline. R1 independently executed38 tests remain applicable to their unchanged
graph/codec/Auto/transaction/gesture/storage/numeric/resource assertions. SDK and
locks are unchanged and the current source/closure verification passes. This is
explicit reuse, not a claim to have rerun38 tests in R2. R1's defective visible-pixel
coverage claim is not reused; fresh evidence above replaces it.

| AC | Final result and basis |
|---|---|
| AC-01 | pass — same unique tree/base/G836, fixed SDK/source/locks/write scope; fresh producer/external source checks |
| AC-02 | pass — strict v1/v2 forest/limits/identity/deep-chain assertions and implementation unchanged; R1 evidence reused with hashes |
| AC-03 | pass — actual profile4 calculation, complete output admission and actual render/pick consumption unchanged; R1 five output mutations/all-mesh3D matrix evidence retained |
| AC-04 | pass — graph commands, keep-world precision, atomic history, subtree/ID ordering, group/leaf gestures and rollback unchanged; R1 evidence retained |
| AC-05 | pass — original UI/geometry evidence plus fresh native7-state pixels and original/new GPU failure controls close V-10 |
| AC-06 | pass — unchanged Storage/epoch evidence plus fresh saved new-process exact content/world/captures/clean state; paired hashes now complement actual content assertions |
| AC-07 | pass — unchanged game/resource/numeric inputs; actual owner/mesh/target/warm-upload and last-owner R1 evidence retained |
| AC-08 | pass — README/handoff/feasibility explicitly correct old coverage claim; accurate permanent commands, source manifest and SD-05 reviewed; finite experimental/I2/I3 limits retained |
| AC-09 | pass — this independent final R2, bound to Plan r1 and current commit |
| AC-10 | pending — merge-owned full gates and actual closeout, not a work correctness deferral |

No unapproved scope reduction, deferred work-owned criterion, renderer workaround
or stray authored debug output was identified. Test-only mutation witnesses are
intentional diagnostics. SDK146 pre-existing warnings remain; no driver warning
was emitted. Final review history is R1 needs_fix then R2 pass; the work self-check
did not replace or reset that history.

## Spec delta and pending closeout

[Frozen current Spec proposal](spec-delta-frozen.md), SHA256
`0b69bfe9091bbe72294f834011362e04f57cd59298b886cb184056eed514b874`, is approved
for the inspected implementation. SD01..04 retain the R1-verified graph/storage/
transaction/gesture rules. Revised SD05 now accurately states the seven-state
final-composition oracle, missing-readback refusal and actual transparent/ghost
controls, with the explicit finite2D fixture boundary. Keep finding-history prose
as provenance when depositing the enduring rules. All seven superseded canonical
paths remain current and hash-identical to R1; `scene-hierarchy.md` is the intended
new component; GOAL-007/008 are valid. No canonical or live-ledger write occurred
during review.

**Merge-owned pending gates:** complete locked driver and AGP test/build; all16
AGP official cases; both original games and835/836 native positives/negatives;
hierarchy normal/wrong-parent/drop-release/partial/reopen and pixel-fault controls;
both distinct composition counter/content gates with their controls; source/asset
and fresh independent consumer build/native checks. Follow the frozen handoff's
exact matrix. Root whole tiers are outside this experimental-only Plan; `tf` is
main-checkout batch-only when due.

Only after those pass may T-10 land source into **autographics-dev**, deposit
canonical/module/goals/ledger knowledge on main, archive, record the batch due
decision and perform guarded cleanup. Preserve the same tree until then. Code,
dependencies or verification-input changes invalidate affected evidence and require
reconciliation/re-review. Next responsible role: **auto-plan-merge**. No landing,
archive, cleanup or full merge-tier execution is claimed here.
