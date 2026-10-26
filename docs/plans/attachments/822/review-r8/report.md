# PLAN-822 final repair review R8 — needs_fix

2026-10-26; final review; Plan revision 5. Reviewer: current chat Codex, did not implement the repair, reconstructed the verdict from committed code, the main-checkout contract and targeted behavior. No subagent.

## Baseline and R7 closure

Existing tree D:/autostack/.wt/lang-822/auto-lang, branch plan-822-dev, HEAD 952c2b586a6e040fd1637cbbb983c633f1cb0b78; repair base 3afbcbd3f74ed3d59e562dd901c419678f92f58e; implementation base de93bacc0ad16622140b0adb4d36eee86fe7fc83. Entry/final HEAD match, tracked implementation clean. Pre-existing untracked review_822_r3.rs and review_822_r8.rs were preserved. This review's additional temporary clone test was saved as replay-clone.rs and removed.

Cargo.lock SHA256 37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873. Repair changes only ownership.rs, the semantics test and the 822 scope required-ID list; dependency manifests/configuration, ABC schema and executor unchanged. input-hashes.txt binds main Plan/contract/Spec inputs. Frozen proposed SD SHA256 4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739, copied as sd-frozen.md. No canonical Spec/ledger publication.

F-822-R7-01 CLOSED: independently reran the allocator probe on the production ownership function. 64 Places: MIR/ABC accepted, charge 19,648B, measured incremental live peak 20,126B. 12,000 Places: MIR/ABC refused; ownership charge 110,628,928B; incremental live peak before refusal only 131,202B, below frozen 16,777,216B; steps=0. Place slots now enter the preallocation root-matrix bound before state allocation, without discarding roots or raising cap. See memory.log.txt and replay.rs. The probe predates this active turn; inspected assertions and reran it, not claimed as newly authored.

F-822-R7-02 CLOSED: obsolete method removed. Fresh clone-probe compilation and targeted builds emit no product warning. Changed committed Rust files pass rustfmt --check --edition 2021; git diff --check 3afbcbd3f HEAD clean.

Affected semantic controls independently rerun: R6 filter 5/5 green (local Jump escape rejected, external Jump return/read and direct/copied Place update supported); mixed local/external join 1/1 green with three modes (local in either predecessor rejects, external-only executes 7/live0). Existing replay-r6.log.txt retained; this turn confirmed same values. No full tier, workspace suite, acceptance aggregate or unrelated performance campaign.

## F-822-R8-01 — P1: explicit collection Clone aliases the source buffer

Classification: original collection integration gap, exposed while finishing the previously pending AC/V audit; not introduced by the three-file R7 repair. Affects AC-02/03/05, T-03/T-04/T-06, V-03/11 and SD-01/02. Plan §1 explicitly includes move/clone/drop, §5.4 requires charged large clone/construction/cleanup, V-03 requires clone nth-item failure, V-11 requires large-clone cancellation. Existing CLONE instruction is documented and implemented as an explicit deep copy.

Bound input and production path: replay-clone.rs::review_r8_clone_list_is_independent takes the existing legal beta_unit(7), uses whole-module collections profile, builds List<i32>[7], CLONEs slot0 into same-typed slot4, writes 9 into original slot0[0], then returns cloned slot4[0]. No resource edge, invalid handle, forged credential or unsupported type. Real verify_mir and encode_abc→verify_abc_bytes both accept. Real execute returns Success I32(9), steps=130, live=0. Expected Success I32(7): mutating the source cannot mutate an explicit deep copy. Independent test fails (exit 101), repeated with the same outcome; see clone.log.txt.

Established cause: av.rs::clone_owned_value (:656 onward) traverses and reconstructs only Record and Str. List and Variant fall through `other => other.clone()` (:732); ListHandle::clone shares its Rc buffer (contract.rs:471). mir_verify.rs:1672 accepts matching owned collection operands for CLONE, so this fallback is reachable through trusted production admission. av.rs:1531 calls this helper for the actual CLONE instruction. Copy/share policy is a separate operation and does not discharge deep Clone semantics.

The T-01 contract-decision §4 claim that CLONE/MOVE/DROP over collections is rejected with mir.type.owned-op-on-collection is contradicted by code and existing accepted move/drop tests. This freeze document cannot silently remove the approved clone requirement; reconcile the supporting contract/SD with the original Plan while repairing. Absence of any clone/cancel/strip execution in the 822 committed test family also means test totals do not close the required V-03/V-11 clone/cancellation/strip oracles. This is a coverage boundary, not a demand for aggregate reruns.

Minimal reproduction (from the same implementation tree): copy replay-clone.rs to experimental/ac-av-probe/probe/tests/review_822_r8_clone.rs, then:

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r8_clone review_r8_clone_list_is_independent -- --nocapture

Closing oracle: true independent collection clone through both gates and executor, original/clone mutation isolation; List<owned record/Str> and Variant payload sibling coverage; per-element allocation/fuel/cancel and nth-item failure rollback/cleanup assertions required by existing V-03/11, plus legal controls. Preserve ordinary COPY/MOVE sharing and existing reverse-cleanup behavior. Do not disable CLONE or weaken the original Plan without explicit contract revision. Use explicit bounded traversal consistent with §5.1.6. Run only affected checks in work/re-review; full regression remains merge-owned.

## Acceptance and knowledge disposition

AC-01: retain previous first-case/parent-oracle evidence, no complete fresh pass inferred. AC-02/03 fail on explicit deep clone; Place R7 controls pass within their stated scope. AC-04: R7 admission-memory blocker closed; other prior rejection evidence retained, no new whole-matrix pass. AC-05: verifier memory repaired, clone resource/failure oracle remains incomplete. AC-06/09: existing cache/recipe/compatibility/incremental evidence retained within prior boundaries; no unrelated replay. AC-07 fails due to current finding; AC-08 remains merge-owned pending.

V-03/11 now have a concrete behavioral failure and missing closing oracles, rather than an unexamined pending claim. Other V evidence retains its earlier boundary; remaining full AC/V/SD audit is not falsely marked complete on known-blocked code. SD-01/02 cannot claim collection deep clone/lifecycle complete. SD-03..06 retain prior disposition; metadata paths and touched_goals=[] explanation remain unchanged. Prior R7 note about guard must-intersect wording is not itself established as a new defect: current ownership.rs really intersects predecessor guard sets; a semantic contract change is not inferred from old work notes.

Omission/workaround scan: new owned collection kinds reached CLONE's accepted operand domain without extending its deep-clone visitor. This is in-contract required work, not deferrable debt. Record in KNOWN-DEBT-AND-RISKS, reopen T-03/T-04/T-06, return executing 4/9; preserve T-01/T-02/T-05/T-09 and historical evidence. Repair diagnosis/coverage must include all consumers of the newly supported owned kinds, not only the old example.

Next: same-tree work, commit targeted repairs, re-review affected criteria and finish remaining AC/V/SD audit. Pending closeout: after final pass, frozen bare cargo t + cargo tv + independent workspace finite-contract gates and any actual additional surface tier by merge. cargo tf remains main-checkout batch-only. No implementation fixes, merge, archive or cleanup performed.
