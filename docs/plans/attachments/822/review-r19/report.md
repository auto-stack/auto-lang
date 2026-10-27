# PLAN-822 R19 independent final review — pass

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施修复，独立从合同、源码和实际测试重建，无代理 | plan_revision: 5
reviewed_commit: 98b4589ba427195d09419237e69ef1aae5ffed81
bookkeeping_commit: 423f9b232c89fc5074434b3b3a5bbcaf9812bdaa
base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83
repair_base: 7de289d47e92c4866857c4ae38579dab5cc4e16b
worktree: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev

## Finding closure and independent checks

F-822-R18-01 CLOSED; F-822-R17-01 CLOSED (R18's layout evidence retained, authorization completed here).
The complete candidate name set is formed before production. For each name the exact same first submitted module selection used by production supplies requirements; absent a replacement the admitted carried module supplies them. Both loops use the same set and selection. Every missing grant returns before produce_unit, link, execute or commit. Replacements correctly supply their own requirements. Previously published snapshots remain bound to their earlier admission.

Original R18 replay: 2/2 pass (one rejection test records caller-only and empty submission; one live-grant control). Both revoked shapes return Err naming beta/hir.test.trace.v1. Permanent incremental controls plus independent sibling control: 13/13 pass, 0 ignored, 17.77s, warning-free. This includes existing signature/body/layout/forced gate/session rollback and baseline campaign controls; it is one integration binary, not aggregate acceptance or a whole-tier gate. The baseline campaign was incidentally part of that binary, not a separate performance re-run requirement. Repeated focused commands are not extra unique cases.

Independent sibling test review_r19_candidate_preflight_order_and_replacement covers two submitted orderings (main/beta and beta/main), a later ungranted beta after an otherwise legal submitted main, zero verified/encoded/admitted/store_writes, unchanged main Arc and executable prior truth 11. Removing the restricted beta through a legitimate replacement under no grants succeeds to 12/live0; adding an unrelated grant while executing a no-capability program succeeds to 22/live0. Permanent carried test additionally covers live grant, re-grant, carried Arc preservation and zero production on rejection. Source ordering corroborates zero production rather than relying solely on instrumentation. No new defect found in this repair.

Commands (in recorded worktree):
- cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r19 -- --nocapture (13 pass; source preserved as replay.rs)
- Same command with filter review_r19 (1 pass; independent-controls.log.txt)
- Original review-r18/replay.rs copied temporarily as review_822_r18_recheck.rs, cargo test with --test review_822_r18_recheck review_r18 -- --nocapture (2 pass; replay-r18.log.txt)
- rustfmt --check --edition 2021 on the two changed Rust files; git diff --check 7de289d47 HEAD: pass.

## Acceptance / task evidence reconciliation

This repair changes only collections_session authorization, incremental tests and one required-case list entry. Core executor/verifier/codec/fixture/recipes, dependencies and test configuration are unchanged from the applicable preceding evidence. Do not interpret this review as fresh global execution of those families.

| AC → tasks | Result | Evidence retained / inspected; current repair impact |
|---|---|---|
| AC-01 → T-01/02/06 | pass | first_case actual verify/ABC/AV, fixed face and 3 alloc/free assertions, real recipe/parent independent expected and negative controls; work closeout receipt and R2 inspection; no path changed in R18 |
| AC-02 → T-03/04/06 | pass | semantics fixtures and V-01/02/04/05/06/11/12; Place referent/frame/escape/mutation fixes R3–R7, clone/cycle fixes R8–R11, replacement/drain R12–R14 closure records; supported domains remain contract-decision §2 |
| AC-03 → T-03/04/06 | pass | transfer/shared ownership and reverse-release oracles R2/R3; explicit deep cleanup R12/R13 and partial rollback closure R14; current session carries exact admitted objects |
| AC-04 → T-03/04/06 | pass | real MIR+ABC invalid-type/owner/state and Place/cycle/analysis-budget negative controls, R3–R9; current authorization rejects before production, with positive controls |
| AC-05 → T-04/06 | pass | resource both-sides/attempt/live/peak assertions, worker parent-sampled memory work evidence; transaction repairs R1, analysis allocation R7, fuel/wall R11, explicit cleanup/partial rollback R12–R14; no resource implementation changed by this repair |
| AC-06 → T-02/05/06 | pass | real store restart and generation separation, old consumer/parent fault evidence retained; decoded types/entry/snapshot/profile bindings independently closed R15–R17; no cache/consumer implementation changed here |
| AC-09 → T-09/06 | pass | V-13 source baseline and honest/forced structural controls, actual collections helper body/signature/layout controls; R18 layout 2/2 retained; current 13-case run and original R18 replay close required vs granted candidate authorization invariant |
| AC-07 → T-07 | pass | this revision-bound reconciliation, no open in-contract finding, metadata and SD audit below |
| AC-08 → T-08 | pending merge | frozen closeout gates, canonical deposition, actual merge/archive/index/guard/cleanup not performed |

Historical pending language was preserved as historical scope, not treated as an open named defect after its subsequent closure. Runtime evidence above is reused only where the implementing code and inputs are unchanged; retained worker/latency figures are not newly measured here. Cancel/wall seam limitations recorded in R10/R11 remain explicit; no new production-mid-cancel timing claim. No unfinished requirement was converted to debt or N/A. No added debt candidate.

## Spec delta

Frozen original sd-draft: SHA256 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739; copy sd-frozen.md. All six exact docs/specs targets and existing metadata reconciled against main; current target hashes in input-hashes.txt. New files remain proposed additions. Empty touched_goals rationale remains finite experimental expansion, not production/bootstrap goal completion.

SD-01/02 supported with contract-decision's exact domain: one List layer, arbitrary acyclic Variant nesting within budget, no List payload or container record fields; “一层容器” must not be rendered as a Variant depth cap. Current ownership.rs does intersect guard facts at predecessor joins and clear on calls/reassignment; old execution diary's block-local-only wording is historical, not the current Spec rule. SD-03 supported with actual decoded profile/types/entry/snapshot binding, owner restamping rather than invented payload self-authentication. SD-04/05 limited fixture-only profile and real recipe identities accepted. SD-06 accepted with complete candidate required-capabilities vs current host-grants check before production, submitted and carried units alike; prior snapshots retain earlier admission. These are verified deposition constraints, not new scope or canonical edits. No live ledger or canonical Spec published.

## Frozen state / routing

Final HEAD remains reviewed_commit; tracked implementation clean; temporary R19/R18-recheck files removed after preservation. Existing untracked historical probes unchanged. Dependency lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Manifest/config not changed by repair; source/spec/contract hashes attached. Diff and changed-file formatting clean, tested product compilation warning-free.

Outcome final pass → reviewed, T-07 complete/current_step 8 of 9; T-08 pending. Merge must run frozen bare cargo t + cargo tv + experimental workspace finite contract checks and actual additional surface gates, resolving stale historical untracked probe build inputs before selecting checks (do not mistake untracked review files for committed product tests). Any code change requires affected re-review and new frozen gates. cargo tf remains main-only batch regression when due. No full t/tv/tt/tb/tu/taa, aggregate acceptance, implementation repair, merge, archive or worktree cleanup performed here.
