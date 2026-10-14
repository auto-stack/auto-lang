# PLAN-771 final review R2 — needs_fix

2026-10-14; plan_revision 2; scope final. Reviewed commit 86f3a91a0cabfafede9b9b69579d368bdf3df522, base 199d6509e60821c3c0a6414ecd0bedf484105e4d, prior review ecd9530d0086d0bdae35ef09cf229f2fb6f91468. Same worktree D:/autostack/.wt/lang-771/auto-lang, branch plan-771-dev, clean before/after review. Current Codex review context did not implement the repair; no additional agent/model identity claimed. Main HEAD 37ad632900f7467b7af975a422c725f46f996061. 772/774 archived; 770 active, not merged; cross-repo dependency code unchanged.

## Prior findings: distinguish examples from full invariant

| Finding | Reproduced closure |
|---|---|
| F-771-R1-01 | Original alias-ctor/alias-sret/alias-control attachments now all build and PE exit0. Destination disjointness repair is effective for those cases. Full record value/evaluation-order invariant remains open: F-771-R2-01 below. |
| F-771-R1-02 | Original positional.at now builds and executes exit0; arity/type/mixed-named controls inspected and gated. Closed. |
| F-771-R1-03 | Original actual Bundle mutation harness rebuilt against new rlib: binding-arg and expression-ref both controlled reject; unmodified control verifies. Boundary/missing binding regression assertions inspected. Closed. |
| F-771-R1-04 | r01 success0 vs failures11/12; r02 forced failure control PE exit11; r03 nested-copy/return/alias oracle has distinct failure codes. Native tests and gate check distinct outcomes. Closed for the identified indistinguishable failure. |

V09 malformed 2000 nested ctor regression executed in full suite; iterative CtorOut resolver and complete ResolvedExpr child detach inspected. Root test tiers not run (no crates/root changes). One-off patch helpers removed. Native forward_safe guards writes into existing destination; fresh locals/out slot use root=None. That solves early target corruption but does not freeze value operands that retain a live source pointer.

## F-771-R2-01 [P1, blocking] Constructor record operands are copied after later side effects

Original requirement gap / full-invariant closure failure, not newly added scope. AC-02/04/05/06; T-04/05/06/07. Plan §5.2: constructor arguments evaluated in SOURCE order; record values have copy semantics. V02/V07 and SD01/03 apply.

native.rs:1256-1259 emits ReadLocal(record) as the local's storage pointer with no snapshot. CtorOut (:1437) executes only after every eval_arg. Its :1482 obtains that pointer, and :1495-1498 finally copies the field. Thus a later field expression can mutate the source before an earlier record value is captured. The repair's mention/disjointness analysis does not help because the enclosing let destination is fresh.

Attached counterexample ctor-snapshot.at:

    type Q { v int }
    type P { a Q; b int }
    fn bump(mut q Q) int { q.v = 2; return 0 }
    fn main() int {
      var q = Q(1)
      let p = P { a: q, b: bump(q.mut) }
      if q.v != 2 { return 10 }
      if p.a.v != 1 { return 11 }
      return 0
    }

Expected: q.v becomes2, p.a holds the earlier value1, exit0. Actual: successful source→bind→verify→native→PE, exit **11**. No borrow escapes: q is a constructor value operand; its copy is finished before the later independent bump call by the agreed value/source-order contract. A view-call parameter is a different context and must retain its declared borrow semantics.

Sibling positional-snapshot.at uses `P(q, bump(q.mut))`: build success, exit **11**. snapshot-control.at explicitly does `let saved=q` before constructing with saved and mutating q: build success, exit **0**. These distinguish deferred value capture from a wrong oracle, unsupported syntax or destination overwrite. Original R1 frozen inputs unchanged and replayed as above.

Repair: separate value materialization from view/mut pointer passing; ensure each record-valued constructor operand is captured at its source-order evaluation point before later effects. Preserve nonaliasing copy elision/deep constructor performance. Audit all record rvalue consumption contexts with later effects, including named field reorder, positional form, nested constructor, local and field-value roots, constructor returned from a function, explicit mut parameter. Add source and direct-HIR runtime controls, with distinct failure outcomes; do not globally copy BorrowView or reject legal value construction as a workaround.

## Coverage and SD review

| AC | Result | Evidence |
|---|---|---|
| AC01 | pass | identity/dispatch/isolation/receipt gate and existing assertions |
| AC02 | fail | prior aliases/positional/fresh-copy controls fixed; new constructor snapshot fails |
| AC03 | pass | actual Bundle harness and full record_hir family, graphs/receiver/reference mutations |
| AC04 | fail | existing ABI/view/mut/return tests pass; constructor combination with legal mut call fails value contract |
| AC05 | fail | ba/aab/order/deep/drop tests pass, but earlier record argument changes after later effect |
| AC06 | fail | all advertised gates pass but cannot establish value/order contract or all-green SD claim |
| AC07 | fail | independent review performed; F-R2-01 blocks acceptance |
| AC08 | pending | merge/Spec/ledger/index/archive/guard owned by closeout |

SD01 positional support now substantiated; SD02 trusted reference bounds repair verified; SD03/04 value/ordering and V01..V10 complete claim remain unapproved. SD05 experimental ACC subset does not close Gen1/GOAL017. Canonical sources rehashed on main, frozen proposal included; no canonical/ledger publication. touched_goals=[] remains valid because this experimental slice closes no formal goal.

Nonblocking documentation hygiene: verification.md §4 still mixes old 187/9 totals, old timing and an old bullet `exit 1 = 快乐路径` with new 199/13/22-step claims; its R1 report relative link uses ../../docs/plans (resolves to docs/docs/plans). Reconcile the current receipt and link in T07; historical evidence must be labelled as historical.

Two failed final reviews now exist. Before another final handoff, work must provide a diagnosis and coverage adjustment for record value capture vs borrowed parameter contexts, including permanent source/HIR mutation-order controls; an old-example closure table alone is insufficient.

## Replay

In Plan worktree:

    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-record-771.ps1

Use target/debug/auto-ac-prototype.exe under experimental/ac-core to build each attached .at with `build-source <absolute input> --source-profile auto.source.record.draft --entry main --output experimental/ac-core/target/<unique-name>.exe`; execute PE and compare codes above. Final gate results and product/Spec/input hashes accompany this report. Original R1 Bundle harness replay uses its documented rustc invocation, linked to the latest ac-core rlib.

Next: same-tree auto-plan:work; T04..07 reopen, T01..03 retain completion (executing 3/9), then committed repair and independent final review. No implementation changes, merge, archive or cleanup performed by review.

Final gate outcome reproduced: 22/22 PASS, 19 cases, full ac-core 199/199, record_native 13/13; fmt/all-targets zero warnings, old 741/745/767 all pass. Durable gate-summary.txt and gate-context.txt retained; context captures entry binary while final-products.sha256 captures actual final rebuilt products. Source HEAD and clean worktree rechecked; no dependency/config drift.
