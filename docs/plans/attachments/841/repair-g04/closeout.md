# PLAN-841 — M841-GATE-04 repair readiness

stage: work; plan_revision: 1; outcome: pass (implementation verified, affected acceptance review pending). UI `2f311868f082bc4ab6fac9d484f71f18d13b1c95`, Man `9b4302fa585e6cda4a91738a6e9057fae12c9402`. Language Rust code unchanged from `34733e23cb6d6436de24258e435a5f3407c7202b`; language candidate Spec/evidence commit follows. Worktree group D:/autostack/.wt/lang-841, branches codex/plan-841-dev. No new agent/review role, whole-tier gate, merge, ledger publication or cleanup.

## Diagnosis, scope and contract

The [original frozen finding](../merge-r1/M841-GATE-04/finding.md) is reproduced unchanged: [red replay](red.log.txt), healthy one pass / old canary one failure. `fstr`, `rhs_expr` and `check_symbol` are byte-identical after newline normalization to pre-Plan language base 100172980efb0343fe7016a05eb40f18d0014d74; [method origin hashes](parser-origin.json.txt). The undefined name is already invalid at parser admission. Previously a package failure could be ignored by the UI construction path; the independently required R1 import rejection now makes that old fault-injection setup stop before Init. No grammar relaxation or swallowing of module errors is appropriate.

A parse-valid data-dependent integer division inside the same band f-string reaches the real BarChart.Init and raises DivisionByZero. It exposed an actual ownership gap: [runtime trace](runtime-trace.log.txt), [heap trace](rc-trace.log.txt), [field identification](field-trace.log.txt). Initial `bands` is VmRef 4000016. Rendered Init snapshots copied that ID without acquiring an owner. Another graph could replace/release the root value; later replay could reinstall the borrowed stale ID, and failure before overwriting bands left it dangling. Healthy Inits normally overwrote it, hiding the gap. This is a production lifetime repair within AC-03/04/05 and the existing owned-consumer contract, not a new syntax or weaker negative oracle.

Actual fixed path: AuraViewBuilder's resolved prop snapshot -> register_init_demand (queue acquires typed reference ownership) -> generation/dispatch -> ensure_child_state (new root fields acquire ownership before old fields release) -> queue release -> real Init. Missing, duplicate, cancellation and bridge destruction balance queue ownership. Inline array props use the existing typed VmRef/ListData representation and own their direct typed child references; ordinary Int values never acquire ownership merely by matching an allocated ID. No Core/VM algorithm, dependency/feature/profile, scheduling order or timeout changed.

## Finding closure and permanent controls

| Finding / invariant | Affected paths | Correction and observed oracle | Remaining gap |
|---|---|---|---|
| M841-GATE-04: original invalid input cannot prove Init isolation | existing package harness and exact original undefined-variable patch | Preserve old test ID and literal input; assert direct domain parse rejects and formal component construction reports child parse failure, then repair the same file and recover the healthy package count | none in work scope; review pending |
| Valid runtime fault must reach Init and preserve siblings | package construction -> frame demand pump -> renderer | New parse-valid integer zero-division f-string; actual Init Failed and DivisionByZero, Line/Area/Donut Done. Healthy MouseArea=36, canary=14; original canary<healthy and floor>=12 preserved | none in work scope; review pending |
| Delayed snapshot owns heap data through overwrites/failure | queued replay, root replacement, failed Init | New permanent state-overwrite/forced-reap failure control: queue keeps object alive, replayed state retains exactly its owner, last removal reclaims it | none in exercised representation |
| All exit/identity paths release owned data | missing handler, cancellation, repeated identity, replacement, bridge drop | Permanent weak-owner/reclamation and refcount controls pass; replacing a sole queue owner remains live, duplicates add no lasting owner | I/O/CPU scheduling implementation unchanged; existing CPU/demand lifecycle controls reused/replayed |
| Array reference transfer and scalar classification | inline array -> ListData; numeric scalar same bits as heap ID | Typed child lives after replay and is reclaimed with the list; matching Int does not keep an unrelated object alive and still reads the same numeric value | no new generic container support claimed |

## Checks mapped to scope

- [Runtime controls](runtime-final.log.txt): **12 pass**, comprising original M1 two, same package-harness M4 three, new ownership five, existing R1 domain/import rejection/recovery two. Actual assertions and source patch readback remain active.
- [State neighbors](state-neighbors.log.txt): **2 pass**, existing chained-reference mutation and real dynamic SVG props consumer. Chosen because prop seeding and VmRef representation changed.
- [Demand lifecycle](demand-lifecycle.log.txt): **6 pass**, existing once-per-generation, failed/missing terminal states, identity cancellation and dependent CPU Init ordering, plus existing outlet diagnostic. Chosen because queue registration/replay/cancel changed.
- Total **20 distinct directed tests**, no skips within the selected cases. The earlier candidate logs are diagnostics, not added to the total. Formatting and diff whitespace checks pass. Existing compiler/linker warnings remain baseline signatures; no new debug instrumentation remains (the retained canary diagnostic reports its actual assertion values).
- Man source pin parses and equals the repaired UI commit; Man product code/manifests/locks unchanged. No broad Man suite is run during this repair.

## Spec delta and readiness

SD-03's existing runtime target gains the concrete delayed-snapshot ownership/exit rule: [candidate-only delta](spec-delta-g04.diff), SHA256 `14cc305bfe2db4ca0b0aed6c6d892f5ae1e6e11296fada9a46a65aefd4950718`. Contract revision stays 1: this fixes the existing lifetime/fault-isolation obligation; it adds no new backend, supported syntax or general IR mechanism. No canonical publication or live-ledger edit. Exact source/config identities and test receipts: [baseline](baseline.json.txt).

Work-owned AC-01/02 use unaffected Core/parser classification evidence; AC-03 affected owner lifetime and AC-04/05 package/control paths now pass the directed controls above. Codecs, the four generators and tool cold-import semantics are unchanged from prior reviewed production evidence. M841-GATE-04 is fixed on the implementation side; no open work-owned findings, test gaps or unresolved semantic decision remain. T-07/T-08/T-09 can close; T-10 remains pending affected independent acceptance review of the new production ownership path and candidate Spec.

Next: affected review on the committed three-repo baseline. Unified bare t and applicable tv/tt/tu plus UI/Man tiers, real 777/792 and distribution remain merge-owned pending. Historical gate passes do not cover this new production revision. No source has landed, no archive/ledger/cleanup/U841 completion is claimed.
