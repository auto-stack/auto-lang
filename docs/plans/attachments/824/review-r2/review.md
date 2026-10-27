# PLAN-824 T-07 R2 — needs_fix (affected repair review)

## Baseline and scope

- reviewer: Codex in this chat (same independent reviewer context as R1, not the implementation context; no additional agent/session identity claimed). No implementation edits.
- review_scope: final repair re-review; Plan r7, shared main Plan read before inspection. worktree D:/autostack/.wt/lang-824/auto-lang, branch plan-824-dev, reviewed_commit **6a68240e89bdfd141ecfb893f83c7718bf503056**; repair diff base **2093675731fb1fe1b9aaa7291cdbbdee672bf9b5**; integration base c6c5ddb256089e6b628d7651cb9759664d15f22a. Main baseline 81c32af56cd6e0f0dee1f618345adcc3aaa530ac.
- dependency revisions: consumed D821/D822/D828/D852 remain the inherited integration base; no dependency update in the 11-file repair commit. Inputs, current Spec sources, closure table and frozen SD-01..07 hashes recorded in inputs.json. Numeric collision with unlanded 823 remains an integration prerequisite.
- Worktree clean and HEAD unchanged at repeated checks. Review evidence written only under main docs/plans. Running builds in the existing Plan worktree required tool approval for its output directory under this session's filesystem permissions; it was approved. No rejected approval or remaining access blocker.

## Closure decisions

| Finding | R2 verdict | Evidence |
|---|---|---|
| F-824-R1-01 [P1] scalar kind mismatch | **closed on reviewed SHA** | Original recomputed-checksum Bool/I32 mutation now Invalid(type). expect_leaf_kind covers all 8 scalar tags. Permanent 8×8/root/nested valid/invalid controls inspected; targeted test passes. |
| F-824-R1-02 [P1] incomplete current-type admission | **closed on reviewed SHA** | Original empty List<unknown record> now unknown-type; same-name changed field type now layout-mismatch. Binding adapter now carries recursive field/payload types; root descriptor admission covers empty containers/inactive arms. 3 admission tests and real-adapter current-field-type test pass, including legal fresh index 77. |
| F-824-R1-03 [P1] unbudgeted type expansion/traversal | **partial, remains open** | Original 17-layer/721B sample now DynTypeNodes; node work is charged. But byte/name expansion remains invisible for transient operands, and portable type traversal has no effective cancellation polling. Durable counterexamples below. |
| R1 unused-import health issue | **closed** | Import removed; cargo check of the exact incremental test target succeeds without warnings. Package/harness fmt checks pass. |

R1 typed restoration closure is not revoked by the remaining budget defect. The repair closure table's F03 “余隙:无” is not supported by the actual implementation.

## F-824-R1-03 remaining invariant failure: temporary type bytes

Existing rule, not new scope: Plan §5.2 requires new type descriptors to request budget before allocation and includes execution data under the 8MiB / managed total 32MiB policy; §5.4 requires type/path analysis and temporary construction to count against the existing aggregate budget, including cancellation. R1 explicitly required type expansion budget before excess work/allocation, not just closure of its 17-level sample.

Bound input: whole-kind av.dynamic/1 module (real encode_abc → verify_abc_bytes → execute). It packs an i32 then DynIs against 15 nested two-arm variants, both arms referring to the preceding type. Variant names append 1024 ASCII x bytes (unique V0..V14 prefix), all admitted by the production verifier. Wire is **16011 bytes**, depth 16; expansion **65535** nodes, below MAX_DYN_NODES=65536. Execution max_steps=1000000, max_slot_bytes=4096.

Expected: charged temporary descriptor bytes cause a controlled resource refusal before exceeding the applicable budget (or a bounded representation avoids that expansion), with observable accounting. Observed: **Success Bool(false), steps=65546, peak_slot_bytes=240**. At the point resolve_dyn_type returns, the owned tree has at least **38796208 bytes (~37.0MiB)** of DynType cells plus repeated name suffix bytes, excluding vector/allocator overhead and other state. This is a structural lower bound, not measured process RSS. The tree alone exceeds even 32MiB, so this does not depend on interpreting 4096 slot bytes as a universal temporary-allocation limit.

Established cause: av.rs:588 resolves/materializes the entire owned tree with only depth/node checks; :684 charges work only AFTER resolution. dyn_type_bytes (contract.rs:694) counts descriptors only for values written to a Dyn slot. DynIs/DynTake comparison operands are temporary types and never enter that accounting. Repeating long names is a legal instance of the same DAG-expansion mechanism, not a new language capability. A node ceiling is not a byte ceiling.

Control: 3 layers, short names, 238B ABC, 15 nodes/~1200B, success Bool(false), steps26/peak240. Old 17-layer/721B input now returns DynTypeNodes. Thus old-example closure is preserved, full-invariant closure is not.

Affected siblings: all three Dyn resolution callsites use the same constructor; DynPack can fail only after temporary materialization/slot preparation. Binding construction and portable tydesc read/write also clone/emit type metadata and must be audited for the same before-allocation budget rule. No assertion is made that every sibling independently reproduces the precise same result.

## F-824-R1-03 remaining invariant failure: portable type cancellation

Bound input: a legally packed empty List<Variant V>, with V having 1024 distinct arms, each payload I32 (1026 total type nodes including List). Options.cancel is a callback returning true and counting invocations. Wire is 8153B. The full current VariantBinding is present for decode.

Expected: once type traversal crosses the documented 1024 work/node polling interval, freeze and unfreeze return Cancelled and preserve source/input usability. Observed: **freeze Ok(8153 bytes), cancel_polls=0; unfreeze Ok(DynValue), cancel_polls=0**. Default-options freeze remains a legal control; original bool roundtrip remains valid.

Established cause: write_tydesc/read_tydesc/admit_tydesc/ty_ident traverse these type nodes, but only enter_node polls cancellation, and that counts actual VALUE nodes (one empty list here). charge_steps increments work but never polls; write_tydesc does not carry traversal context at all. Newly added charged type admission therefore can cross the interval without any callback. This is not a request for immediate cancellation of a tiny operation: the input has >1024 real type nodes and exercises the explicitly promised type traversal. Existing value-node cancellation tests do not cover it.

## Reproducible command and raw output

From the existing implementation worktree:

```powershell
cargo run --quiet --manifest-path D:/autostack/auto-lang/docs/plans/attachments/824/review-r2/probes/Cargo.toml --target-dir D:/autostack/.wt/lang-824/auto-lang/experimental/ac-av-probe/target/review824-r2
```

Source and raw: probes/src/main.rs, replay.txt. Path dependency compiles exact reviewed core sources. The old probe was adapted only for the changed binding API; remaining controls were independently selected. Probe prints observations, exit0 is not an acceptance verdict. The 512-name trial also succeeded with ~21MiB; durable final input uses 1024 to exceed aggregate 32MiB unambiguously. An initial restricted build attempt returned no diagnostic; the approved worktree build produced the durable results. No product claim depends on that environment attempt.

## Independent checks and retained evidence

All commands in D:/autostack/.wt/lang-824/auto-lang:

| Check | Result | Mapping |
|---|---|---|
| cargo test --locked --manifest-path experimental/ac-av-probe/parallel/824/Cargo.toml --lib av_dynamic_824_wire_scalar_kind_equivalence | 1 passed, 0 ignored | F01 |
| same manifest, --lib av_dynamic_824_admission_ | 3 passed, 0 ignored | F02 |
| cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_dynamic_824_semantics av_dynamic_824_semantics_restore_revalidates_current_field_types | 1 passed | F02 real adapter |
| same probe manifest, --test av_dynamic_824_resources av_dynamic_824_resources_type_dag_expansion_bounded_and_charged | 1 passed | F03 original sample/node controls only |
| cargo check --locked --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_dynamic_824_incremental | pass, no warnings | R1 health |
| cargo fmt of core/probe packages and parallel/824 -- --check; git diff --check 209367573..HEAD | pass | repair health |

No aggregate acceptance/full workspace tests or whole-tier t/tv/etc run during this repair review. Unchanged R1 semantic/incremental evidence retained at its original coverage boundary. Work's repair semantic/compatibility receipts inspected; no new claim that all compatibility/performance gates independently passed. Work's 798 timing attribution is not used to waive any required gate; merge-owned frozen gates and required compatibility adjudication remain pending. No expensive unrelated campaigns repeated on known-red code.

AC map: AC-02/03/04/06 typed identity portions now pass for F01/F02; their resource/cancel portions remain partial/fail under F03. AC-05 fails; AC-10/11 fail because same-source and mapped resource/cancel invariants still fail. AC-01/09 retained with R1/repair boundary, not re-certified broadly; AC-07 final pass withheld; AC-08 merge pending. V-05/08/12 admission affected entries updated to closed where above; V-06/07/11/14/15 resource/cancel axes remain open. F03 maps to T-03/T-04/T-05/T-10; T-06/T-11 dependent aggregation evidence invalidated. Retain T-01, bounded T-02, T-09; reopen the six affected tasks (3/11).

## Next work and review boundary

- Keep F01/F02 and health closure rows at 6a68240e8; repair F03's actual invariant. Before next handoff, add a diagnosis/coverage adjustment explicitly explaining why node-only plus slot-only accounting missed transient name/type bytes and why value-only polling missed type work (two final reviews have now failed).
- Closing oracle: execute the saved long-name/under-node-cap sample and small positive control; enforce applicable bytes/work/cancel BEFORE allocations/traversal pass their limits; verify cleanup/source preservation. Cover shared constructor callsites and portable descriptor/adapter paths with representative valid/reject/cancel siblings. Type work must participate in cancellation even when the value tree is tiny. Tight-fuel refusal alone after an already-built large tree is insufficient.
- Re-review only this affected mechanism plus actual repair regressions; do not replay unrelated full suites/campaigns. Future bare cargo t + applicable tv + finite new-workspace closeout gates are merge-owned after final review pass; tf batch only on main. Renumbered post-review code needs affected revalidation.
- SD-01..07 frozen in spec-delta-frozen.md; current paths/metadata preserved, touched_goals [] rationale from R1 retained. Resource/cancellation claims cannot be published until F03 closes; no canonical Spec/ledger changed here.

**Outcome needs_fix; status executing; current_step 3/11; plan_revision 7 unchanged.** Implementation untouched, no merge or cleanup.
