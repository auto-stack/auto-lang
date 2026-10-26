# PLAN-822 independent final review R2 — needs_fix

- Date: 2026-10-26; scope: final repair review plus R1 outstanding contract audit. Reviewer: this chat Codex; did not implement either repair. No delegated reviewer or invented model identity.
- Plan r5; tree D:/autostack/.wt/lang-822/auto-lang; branch plan-822-dev; frozen HEAD 81c33e0918468bc21f6bd756fb8bbd6b4616ab57; diff base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Repair commits 717e9c208 and 81c33e091. Tree clean before/after (temporary review tests removed).
- Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873; canonical Spec / contract / SD hashes in spec-hashes.txt. These are observed sources, not an approval of the proposed SD.

## Prior findings

F-822-R1-01: closed for the reported ListSet swap-before-budget defect. Static order now prechecks before replacing/dropping; four new transaction tests independently replayed, 4 pass (resources.log.txt). This does not approve all lifecycle and resource criteria; gaps below remain.

F-822-R1-02: partial, not closed. New CollectionsUpdateSession really admits/links/executes units and carries unchanged objects; existing affected tests independently replayed (excluding unchanged baseline campaign), 3 pass (incremental.log.txt). The replacement creates a failed-install regression and the script does not test mandatory identity/case coverage reliably. Signature rejection followed by submitting another valid beta does not prove state preservation.

## Blocking findings

### F-822-R2-01 — P1: failed candidate becomes current session (repair regression)

Rule: Plan §5 T-09 / AC-09 / D828 install transaction: failed/cancelled candidates do not install; prior snapshot remains usable. Canonical resident-function-install.md requires candidate build-aside and atomic successful commit.

Location: probe/src/collections_session.rs:242–248. apply_revision increments revision and overwrites units/identity BEFORE snapshot/link and execution. Either link Err or execution failure leaves rejected state installed. Proven cause, not hypothesis.

Replay: copy replay.rs to experimental/ac-av-probe/probe/tests/review_r2_822.rs; run `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_r2_822 review_r2_rejected_revision_preserves_snapshot -- --nocapture`. Install main=beta()+5 with beta returning 2; submit a separately valid bool-returning beta. Expected signature rejection AND original beta Arc/current result 7. Actual signature rejection followed by changed beta Arc; assertion fails (replay.log.txt). Existing repair test replaces the bad beta before examining current snapshot, hiding the mutation.

Closure scope: reject signature candidate without changing revision/current units/identity; immediately query/execute current snapshot. Also test execution trap/resource failure after successful link and earlier held snapshot. No global suite required for this repair.

### F-822-R2-02 — P1: required list Index Place/token mechanism absent (original contract gap)

Rule: §5.1.3, §5.5, V-02/11/12 and contract-decision §4 require index Places, old token rejection on remove/realloc, fresh-index positive controls and per-element generation on replacement.

Inspection: core/src/contract.rs ListBuf has id/epoch/cap/elems only, no element generation or token snapshot. `rg -n 'StalePlace|\.epoch' experimental/ac-av-probe/core/src` finds only trap declaration/name and epoch increments (av.rs push/insert/remove); no epoch comparison or StalePlace construction. MIR collection instructions address root+fresh index directly. No list Place/token creation exists. The fresh-index remove test is legitimate but cannot exercise retained old Place. Thus do not invent a runtime counterexample against an unavailable input path; the missing required path itself is the finding.

Closure scope: deliver the contracted index Place/token path and both old-token/fresh-index controls through verify/ABC/AV and applicable restore, including realloc and remove. Add per-element replacement validity controls. If reducing the approved surface is intended, use explicit replan; silent omission is not completion. This also blocks SD-01/02's epoch/Place claims.

### F-822-R2-03 — P1: list elements clean up forward, violating reverse cleanup (original contract gap)

Rule: V-06 and contract-decision §4 require reverse element responsibility/event order on list drop. This is observable existing contract, not a new scale or user-destructor promise.

Replay: copy drop-replay.rs to probe/tests/review_r2_drop_822.rs; run `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_r2_drop_822 review_r2_list_drop_reverse_elements -- --nocapture`. Valid List<Str> with two owned strings, normal function exit after pushing first then second. Expected second string freed before first; actual free ids [0,1,2] (list id0, first id1, second id2). Failure in drop-replay.log.txt. ListHandle::drop only reports buffer free; Vec then drops elements in forward order. Existing nested test removes both elements before exit and only asserts scalar result, so it is not this oracle.

Closure scope: retained multi-element list on normal exit/explicit drop and representative trap/cancel exits, fixed reverse event oracle and live=0; nested owned record/Variant sibling. Do not hide incorrect order by reaping everything.

### F-822-R2-04 — P2: acceptance guard does not enforce required-case completeness

Rule: T-06 / V-10 require no missing/skip/duplicate required subcases and nonzero aggregate failure. Script's comment claims required-id guard, but Assert-Selected only checks result-line existence and all-zero count. It never rejects ignored cases or checks required identities exactly once.

Control: extract Assert-Selected unchanged from scripts/verify-av-collections-822.ps1 and invoke with `test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`. Actual returns normally (skip-control.log.txt); expected reject. This is a validator control, not a claim that committed tests are currently ignored. Removing a mandatory sibling while retaining one passing test is likewise undetected. Re-run minimal valid/skip/missing/duplicate controls and demonstrate actual aggregation exit failure; no full acceptance replay needed during repair.

## AC/V/SD map and health

| Acceptance | Result | Evidence / remaining boundary |
|---|---|---|
| AC-01 | partial | first-case/typed input gate/recipe parent fixed oracle inspected; no new whole-path runtime replay this repair round; work evidence not independently promoted to pass |
| AC-02/03 | fail | F-R2-02/03; ListSet repair independently passed but does not cover missing Place and reverse lifecycle |
| AC-04 | partial | admission dispatch separation and type checks inspected; full required V-11/12 not delivered via old-token path |
| AC-05 | partial/fail | four transaction tests pass; lifecycle reverse failure and matrix gaps prevent complete claim; unchanged memory campaign not rerun |
| AC-06 | partial/fail | cache reverify and recipe judge inspected; tests named new-worker call restore in-process and alone do not establish fresh-process restoration. Required V-11/12 recovery blocked by missing Place. Script coverage oracle fails F-R2-04 |
| AC-09 | fail | existing affected tests 3 pass; independent rejected-candidate control fails F-R2-01; no repeat of unchanged D828 scalar campaign |
| AC-07 | fail | unresolved findings; no pass |
| AC-08 | pending merge | only after subsequent review pass and frozen gates |

V-01/04/05 basic fixture/type/guard/switch cases inspected; V-02/11/12 missing old Place path; V-03/07 transaction subset replayed; V-06 independent negative demonstrates false lifecycle claim; V-08 cache source audit does not establish all fresh-worker/dependency controls; V-09 prior compatibility evidence retained as work evidence, not rerun globally; V-10 completeness validator control fails; V-13 replacement failure path reproduced. No claim of complete behavioral pass for unexecuted families.

SD-01/02 cannot deposit claimed Place/epoch/lifecycle rules until fixes above. SD-06 still says T-09 pending and cannot describe current session as transactional. SD-03/04/05 targets exist and profile/recipe identity additions align in outline; drafts are summaries rather than complete verified after-text. Existing metadata paths and empty touched_goals rationale retained; no canonical Spec or ledger changed. Frozen SD hash: F86CF08D8D8A645C209FF138E10CF5D9F87B69876500B745291D483D51FDDA2B.

Health: targeted resources compile emits unused-variable warnings at lines 151/202/443; semantics source also has unused s0 at 813. Clean these per zero-unhandled-warning rule. Plan T-04/T-09 table rows contain duplicated stale trailing text; update bookkeeping without changing semantic revision. Original historical review receipts remain intact.

No aggregate acceptance or whole-tier cargo t/tv/tt/tb/tu/taa run. Merge-owned gates remain pending: bare cargo t + applicable cargo tv + experimental workspace finite contract checks and any additional actual surface tiers, only after final review pass at frozen HEAD. tf remains main-checkout batch only. Next: work fixes in the same tree, targeted checks/commit, then re-review affected findings and remaining evidence.
