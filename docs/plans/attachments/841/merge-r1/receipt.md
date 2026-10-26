# PLAN-841:r1 — merge receipt

stage: merge; outcome: needs_fix; plan state: executing 6/12. No source has landed, no archive/cleanup or U841 release has occurred.

| Checkpoint | State / evidence |
|---|---|
| prepared | Original independent R4 passes at lang ecfa187f72d3fcf7ba662ae6cd3e1170aad75735, UI eb50ce67930179f51db041b74faa6d1fbc72c211, Man ebfbb37903d24bcea3cbd81c6a0a5602da605982. [Frozen R4](../review-r4/review.md). Current implementation/config identities: [frozen input trees](frozen-inputs-attempt-3.json.txt). |
| landed | pending; only ff-only landing is permitted |
| ledger_refreshed | pending; tracked workspace ledger .autoos/specs.json remains unchanged, schema v96 input; HTTP-first, guarded offline projection only if no store writer is reachable |
| archived | pending; active Plan remains in docs/plans |
| cleaned | pending; three owned development trees remain intact |

## Reviewed-source mapping and scope

Language/UI implementation branches rebased onto current master without code conflicts. [Language range mapping](lang-range-diff.txt), [UI mapping](ui-range-diff.txt); UI patches all equivalent, language final doc patch omitted only the identical already-landed closeout artifact. Canonical source contracts retain reviewed SD-01..05; prepared docs add provenance, module plans/navigation and unchanged overall goal statuses. [Canonical hashes](canonical-source-inventory.json.txt), [new-link check](spec-link-check.json.txt). Future archive links will be verified after archival.

Current language frozen code `cced32bc7d1eb09a419147e7e01e49a2a8f71de0`, UI `8d6ad2bf362aa8293e58d2734fc54dd42db1da0f`, Man `f5e82892ed6b52bdfe7ca329ef64f0162567f4a8`; fixed down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`. Man's release-gate UI source pin is synchronized to that UI commit. Subsequent docs-only delivery descendants must prove unchanged code/dependencies/configuration/fixtures before reusing gates.

## Failed gates and repairs (preserved, never waived)

- [Attempt 1](gates.json.txt), [raw log](lang-t.log.txt): 659 pass, five view-function failures and two native timeouts; fail-fast left remaining tests pending. [Test-entry/lock-feature repair and affected review](repair-review.md), 32 scoped pass plus two isolated timeout replays. Reviewed repair commit 4a879088519716a45c700588052a4019c3b405ba.
- [Attempt 2](attempt-2-gates.json.txt), [raw log](attempt-2-lang-t.log.txt): 1125 pass, one stale Plan806 diagnostic-table oracle, 2057 not run. Both original timeout tests passed in this whole invocation. [Returned-error/recovery oracle repair and affected review](repair-review-2.md), 39 scoped pass, commit cced32bc7d1eb09a419147e7e01e49a2a8f71de0. No product parser/runtime behavior, input, timeout, assertion threshold or selection weakened. Same implementing session reviewed these test-only changes; this limitation is explicit, original independent R4 remains applicable to unchanged production/SD.
- [Attempt 3 frozen unified gates](attempt-3-gates.json.txt): evolving receipt, not an all-pass claim until every required row succeeds. Language bare t: 3183/3183; tv: 163/163; tt and UI tiers pending at this checkpoint. Man unified gates will use the exact fixed clean release clones, before fresh-lock changes, to exclude pre-existing unrelated down WIP.
- [777 strict](strict-777.json.txt): pass, zero violations / four grandfathered baseline entries; recheck final affected inputs before landing.
- Real 792 and 14-case Man distribution: pending; historical worker results are not substituted for current frozen-source checks.

## Preservation, deployment and due check

Main checkout contains other sessions' document WIP and new document commits. Preserve all unrelated content and record any overlapping-file recovery; never include it in this Plan's commits. A dirty detached auto-down tree existed before merge in the shared group; do not modify/remove it. Release tests use a clean fixed clone instead.

[Artifact observations](artifact-observations.json.txt) record absent main release binaries and an older web artifact. This source merge does not claim production restart/deployment. Fresh isolated delivery artifacts are covered separately by release gates. Batch due check remains pending until cleanup; prior receipt covers Plan850 at 2026-10-26T00:02:10Z.


## Final checkpoint — merge not landed

Three automatic work/review repairs completed; M841-GATE-01/02/03 closed with 32 / 39 / 3+11 scoped cases. [Third affected review](repair-review-3.md) binds UI 7f270e51ea10a4c584a96a3eb8101e57dff35908 and Man 4ad2208c5e624f5c031cebf5694eec16981d7ad0 with unchanged language code cced32bc7d1eb09a419147e7e01e49a2a8f71de0. UI test-source fingerprint changed, so old language whole-tier results are retained as historical evidence, not a final-source pass.

[Attempt 4](attempt-4-gates.json.txt) exposed a system-reserved default port; [supported child-env resolution](environment-port-resolution.md) proved the original lifecycle test passes at 54480 without code or oracle changes. [Attempt 5](attempt-5-gates.json.txt) UI daily: **2147 pass / 1 fail, 273 not run**. [M841-GATE-04 finding](M841-GATE-04/finding.md) and [targeted replay](plan492-targeted-diagnosis.log.txt) establish healthy1 pass / canary1 fail before Init. Further diagnosis must distinguish probe validity from production regression; no new contract is silently accepted.

Current work handoff: executing 6/12, T-07..10 reopened. Automatic work/review repair cap reached (three); no fourth implementation repair and no further global gate run on known incomplete acceptance. Three development trees and reviewed recovery branches retained. No default-branch code, canonical Spec, ledger, archive, deployment or cleanup operation has been performed. Main Plan/evidence-only commits preserve the diagnostic chain. All canonical/ledger scripts are drafts; derived ledger remains byte-identical to main tracked input. Man clean-source unified/distribution and 792 have not run because the serial prerequisite gate failed; no historical results substitute for them.
