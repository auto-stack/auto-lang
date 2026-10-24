# PLAN-845 r2 — independent final review R2

**Verdict: pass.** F-845-R1-01 is independently closed on `e518f5fc778cfa6827f7de9eb99b4c0910ef294d`. No new finding. This approves work-owned acceptance and the proposed Spec delta; every complete merge gate remains pending. Plan becomes reviewed9/10, T09 complete and T10 open. No implementation repair, canonical publication, merge, archive or cleanup occurred.

## Identity and authority

stage: review | review_scope: final | plan_id: PLAN-845 | plan_revision: 2.
Reviewer: `/root/review_845_r2`, a fresh authorized subagent context independent of implementation and R1; no unexposed model/session identity is claimed. No further delegation. Read the complete requested auto-musk auto-plan-review skill, its verification-contract reference, repository AGENTS.md, current shared Plan, full R1 receipt and affected canonical Specs. G840's user-approved limitation remains historical and applies only to840; it is not an exemption from this independent845 review.

The only implementation tree is `D:/autostack/.wt/lang-845/auto-lang`, branch `plan-845-dev`; entry worktree inventory and final audit confirm the clean frozen HEAD above. Repair parent is `75277a7b7b8d1b43f840c8993721c02833e5657a`; full implementation base is `8b62db5cba54b9bfe7f759f4920b22ccb9b75e88`. The repair changes exactly six driver files: Host source selection, Storage identity metadata, permanent Host tests, handoff/proposal and its source manifest. No SDK/AGP/root/lock changes.

[Audit](audit.json.txt) binds all320 committed source objects (normalized according to existing transport rules; zero mismatches), work-r2's280 and work-r3's18 raw artifacts (zero hash mismatches), three locks, repository config, skill/reference fingerprints and unchanged canonical Spec inputs. Actual prepared SDK681 files pass independent [verify-only](sdk-verify.log.txt). SDK commit remains `932ad4877e5968f1aa7227d9dcd7fb69b4c69a21`. SDK root lock also matches the frozen source-lock copy. No dependency/configuration/input drift invalidates retained R1 evidence.

## F-845-R1-01 closure and distinguishing observations

Existing rule remains Plan §5.2/5.3/5.5, AC03/06/08 and T06/08: accepted unsaved bytes must have explicit memory/hash provenance, saved project bytes use appropriate verified blob provenance, and Run uses accepted bytes without reread. This is closure of the original requirement, not scope expansion or a new integrity guarantee.

I replayed the **unchanged** R1 external consumer from main:

`powershell -NoProfile -File docs/plans/attachments/845/review-r1/replay.ps1 -Case source-origin`

[Raw output](replay-source-origin.log.txt), [exit0](replay-source-origin-exit.txt). It rebuilds the frozen library, links the durable R1 consumer, and exercises actual Host/SourceSnapshot/Last Run/Auto callbacks. Original R1 exit101 and exact code/inputs remain in review-r1; they are retained red evidence, not counted as a new execution here.

- No saved path + sourceA: `<memory:0e02eeb8...>.at`; accepted control.
- Saved A: existing canonical `scripts/0e02eeb8...at`; accepted control.
- Saved scene + newly accepted unsaved B, original import removed: SourceSnapshot and Last Run both contain `<memory:b8b391fdff55bbebbce187916bdef7773ca3c469eb08cc030a6ed439be6a66fc>.at`. Actual `output/app_start` error contains the same origin and `API version/error code=1 node=999 argument=0`. No setup failure or fallback.
- Save B then Run: actual existing `scripts/b8b391...at`; third accepted control.

`RUST_MIN_STACK=16777216 cargo test --locked --test behavior_project_host bound_source_origin -- --nocapture` passes **2 selected tests / 2 passed**, no skips ([raw](permanent-origin.log.txt), [exit](permanent-origin-exit.txt)). I read the assertions, not only the totals: SourceSnapshot.path, Last Run path/hash and actual callback error are all asserted. The tests cover no-path document, first binding on a saved empty scene, geometry-only dirtiness, A/B update and Undo/Redo before/after save, Open, removed import/confirmed blob, failed and pending SaveAs, old-revision receipt and verified root change followed by Redo. Pending receipts assert Run rejection rather than attempting to run a forbidden state. Work's old-Host controlled mutation fails both at the actual path equality; its raw artifact identity was independently checked. That mutation was not repeated because review keeps implementation writes paused.

## Shared-state review and independent sibling cases

Production path inspected: Storage::load_project validates declared code before creating provenance; verify_locked checks entry + GLB + code and confirm_saved(epoch) before changing provenance; Host::start_selected_play validates current binding/payload then asks source_origin(identity); CodePayload::snapshot uses retained raw bytes; Play/Last Run/runtime diagnostics consume that snapshot. The record stores CodeRef values, not Arc payloads or VM owners; maximum8, missing/evicted entries conservatively use memory. Root changes clear old records, same-root verification retains valid identity history, old revisions confirm snapshot.code rather than current binding. Source identity includes format/hash/length; geometry dirtiness does not affect it. New/Open publication replaces Storage only after accepted candidate preparation. The temporary external/Builtin routes and their actor selection are unchanged.

I independently chose additional combinations for the changed shared invariant, linked a separate external consumer against the same frozen library and ran it ([source](holdout.rs), [replay](replay-holdout.ps1), [raw](holdout.log.txt), [exit0](holdout-exit.txt)):

| Check → invariant | Actual mutation/control and observed result |
|---|---|
| H1 → root identity | SaveAs to another entry name in the same root retains confirmed sourceA's canonical blob origin; actual Run and Last Run agree. |
| H2 → failed verification cannot publish root/source proof | SaveAs to a different root with AfterReplaceRead gives pending. Corrupt the actual new blob: Verify fails at behavior closure, receipt remains pending, old-root provenance remains, Run rejects. Restore exact bytes: Verify succeeds and Run adopts the new-root blob. Corrupt that blob and attempt Open: source-identity rejection keeps accepted source/path, and Run still consumes accepted bytes. |
| H3 → no-code save must invalidate old-root proof | Detach A, SaveAs to a third root without source, Undo restores A. No blob exists there; actual Run and Last Run use explicit memory/hash, without inheriting proof from the former root. |

No new requirement for arbitrary disk freshness is inferred. A previously verified blob removed/tampered after confirmation can still be attributed to its prior verified project origin while Run executes accepted bytes; subsequent Open/Save/Verify detect the disk discrepancy. This matches the retained-source contract. Source origin cannot prove present disk readability. Public data structures are assessed on actual supported Host/Storage paths, not invented direct field corruption.

## Acceptance map and retained evidence

| AC → task | Result and evidence boundary |
|---|---|
| AC01 → T01/08 | pass; fresh tree/commit/source/SDK/lock/config/raw identity audit above; retain R1 full source/consumer boundary findings. |
| AC02 → T03 | pass retained R1; document/decode/migration/actor implementation, fixtures and assertions unchanged. |
| AC03 → T02/03/04/06 | pass; F01 origin closed through real callbacks and permanent assertions. Raw-byte admission/compiler/no callback at bind/Open unchanged; retain R1 H05 and permanent bounds/AST/compile controls. |
| AC04 → T04 | pass; retain R1 transactions/owners/dirty/noop/reference/stale evidence; fresh history/source-origin tests and H3 verify the repair's observational impact. No added payload owner. |
| AC05 → T05 | pass; existing transaction/locks/quotas/closure code unchanged apart from post-verification identity metadata. Fresh H2 and permanent pending/old-revision/root tests cover affected writes. Reuse work-r3 whole_path_move_open_run_and_before_after_commit_failure1/1 and reserved_targets_conflicts_and_old_revision_receipt1/1 after inspecting assertions and verifying raw identity. R1 actual corrupt-code/GLB/snapshot/reserved-alias evidence retained. |
| AC06 → T06/07 | pass; default actual Host/Auto source and diagnostics repaired and observed. Existing actor/override/busy/Stop and native OS/GPU logic/input unchanged; R1 independently rebuilt move/bad-source/partial and work relocation evidence retained. No native campaign rerun. |
| AC07 → T04/08 | pass retained R1; pool/current/history/save source owners, budgets and GPU/VM upload/release mechanisms unchanged. New metadata contains only bounded CodeRef, no raw-byte/VM/GPU owner. No GC/owner campaign rerun. |
| AC08 → T07/08 | pass; source provenance, coverage handoff and SD01/03 now accurate; remaining same-binary/new-process/OS/GPU/control evidence unchanged and retained from R1/work-r2. |
| AC09 → T09 | pass; independent final acceptance/Spec/health review completed here, outcome bound to frozen revision2. |
| AC10 → T10 | pending, merge-owned; no landing/delivery claim. |

R1's per-AC assertion mapping and raw reports remain linked in [R1 receipt](../review-r1/review.md). Fresh repair behavior is established above, while unaffected old evidence remains applicable because only the six enumerated repair files changed, all dependencies/configuration/input identities match, and the shared Storage/Host invariant has targeted new failure/control coverage. A new HEAD alone did not trigger a full rerun.

## Spec delta, health and omissions

Frozen [proposal](spec-delta-frozen.md), SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`, comes directly from the reviewed Git object. SD01 adds behavior-project.md; SD02 modifies scene-document/assets; SD03 modifies authoring-session/play/behavior-api; SD04 modifies auto-source-driver/project/integration-contract. All eight existing target hashes still equal R1's canonical versions; new target absent until merge. Proposed current-root/identity/no-reread clarification follows the already-approved §5.2 memory-origin requirement and verified code, without changing API semantics or weakening unsaved provenance. GOAL-007/008 and impact metadata remain valid. No canonical Spec/live ledger modified.

Omission/deferral/workaround scan found no open in-contract requirement, parser/compiler fork, byte reread/existence workaround, new source owner, unapproved production scope or hidden native gate deletion. SaveAs changes root only after full successful verification; failure and old-epoch exits precede metadata publication. Bounded identity eviction can only conservatively use memory; it does not affect byte ownership/admission. R1 F01 is closed, not deferred as debt.

Fresh targeted library builds and package fmt pass ([fmt](fmt.log.txt), [exit0](fmt-exit.txt)); reviewed diff passes git diff --check, and touched production files have no stray debug prints. Only immutable SDK's146 existing warnings occur, no new driver warnings. Work's necessary locked lib check evidence has verified identity and remains applicable. No full package/workspace, root cargo tier/docs_gen, complete native/owner/consumer campaign was run.

## Pending gates and routing

All complete gates remain **merge-owned pending on this frozen commit**: original63 invocations, locked complete driver/AGP tests/builds, original16 AGP cases and dual games, authoring/hierarchy/assets/Play/source/owner/counter AND actual-content gates, producer and fresh own-SDK/own-target consumer, all15 new project native cases plus actual UI-saved output relocation replay. Use committed behavior-project-handoff.md for exact commands/denominators. Root tiers remain untriggered by the driver-only diff; tf remains main single-instance batch-only with845 due check at successful landing.

Next: merge owner executes required frozen gates, diagnoses any failure, returns semantic repairs for affected re-review, then handles autographics-dev-only landing, canonical/ledger/index, archive and guarded cleanup. No source into master; no delivery before T10 closes. Final audit confirms HEAD unchanged and implementation clean.
