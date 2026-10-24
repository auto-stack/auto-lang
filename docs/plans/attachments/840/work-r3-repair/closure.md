# PLAN-840 r2 — review R1 repair handoff

Work outcome: pass, execution_done10/12. Code `1f19395254d7e676bd188e43e3d8f734d57aaa12`, same plan-840-dev
worktree. Contract remains revision2; no support reduction or SDK/AGP/lock upgrade.
Independent affected re-review and complete merge-owned gates remain pending.

## Finding closure and actual path

| Finding | Invariant / affected states | Mechanism and actual oracle | Result / remaining gap |
|---|---|---|---|
| F-840-R1-01; T03/T05, AC03/05 | Both app_start and app_update require Unit, including operand/initializer/control-flow/type expressions; nested function/lambda/closure is a return-policy boundary | Removed partial unit_body visitor. Existing reference traversal carries Rules and checks Return once in every visited body. Child expressions/blocks retain policy; nested functions reset it. Loop initializer bindings precede condition/body validation. Permanent18counterexamples across bothcallbacks;4legal helper/closure/loop/Unit programs execute init+tick, verify value3/owner/Editor; actual Host rejects bad disk source then recovers through corrected source and all-part pixels | all pass; no required gap |
| F-840-R1-02; T03, AC02/03/06 | Literal/pure initializers retain ordinary immutable references/strings without executing user calls or reading mutable state | Track already initialized const/immutable let names in source order; recurse pure AST values/FStr/data operators, explicitly reject assignments and dynamic/private/unbound/cyclic/mutable reads. Four realVM programs observe initialized value3 including actual string lengths and constant indexing;7negative controls; original read→snapshot→Play replay controls | all pass; no required gap |
| F-840-R1-03; T08, AC08 | Stable proposal IDs, exact add/modify paths, canonical before versions | SD01 adds behavior-api;SD02 modifies scene-play;SD03 authoring;SD04 all4original source/profile/project/integration targets;SD05 language-sdk scanner. Compare actual Plan table/5IDs/8target versions, including absent new component, SHA256LF and last-path commits. Frozen proposal retained | pass; canonical/ledger publication remains merge-owned |

The original10-case review replay fails6/10 on a1da1a00b and passes10/10 on the
repair; the wrapper returns1/0 respectively. Inputs and validated wrapper are the
unchanged review-r1 artifacts, not rewritten expected values. Closure details and
hashes are in receipt.json.txt; normative current output is frozen-replay.

The prior tests missed these invariants because Unit checking was a second partial
visitor (direct returns only), initializer tests exercised literals rather than
immutable binding chains/FStr, and the proposal had no exact stable-ID/target check.
The repair removes the parallel traversal and binds scope policy to the existing
AST walk; it does not add text/regex exceptions, evaluate user code during admission
or change user bytes. SDK Parser/VM remain the only language pipeline.

## Scoped validation and reuse

Current producer:12behavior+4behavior_host+5feasibility pass; locked all-target
check/fmt/source299/SDK681/479packages/17locals/29edges and eight source controls
pass. Existing SDK146warnings, newdriver0. Producer4host includes actual20A/B VM/
GPU cycles,600tick, source-root/SaveAs/busy checks and new rejected-source→repaired
constant-source/GPU/Stop/Editor restoration. Actual return/constant controls and
private/reference diagnostics remain active. Source set retains all old278paths.

The independent source package received only the exact5changed sources/docs/tests
and updated identity; own681fileSDK/target retained. Its12behavior tests and1new
actual Host/GPU recovery test pass. No producer build artifacts were copied. The
first rerun omitted the already documented16MiB worker stack and overflowed; after
setting RUST_MIN_STACK=16777216 the same scoped tests pass. Preserve both logs;
this is neither a product pass nor a new SDK fix or reduced support promise.

An exploratory positive used for-init syntax. The pinned raw SDK independently
rejects both a plain name and transport-like name with Undefined variable, while
for-in compiles. This was a probe assumption, not introduced behavior: the permanent
positive uses the actually supported for-in control. The init AST negative still
checks Unit before any compilation and makes no execution-support claim. Both
rawSDK controls and the original failed test log remain. No broader SDK/compiler
change was made or asserted by this repair.

Play/Host/bin/probe/render/API.at/managed assembly/SDK/two locks and input/report
configuration are unchanged. The final code emits byte-identical complete guest
units for the four oldnative positives (Move,Patrol,speed-only,relocation), checked
against every original tick trace's user/combined/API/bridge/version identity.
Those raw13native reports and unchanged ownership/CLI/original-family results are
reused for their unaffected paths; **not** presented as reruns on a new executable.
All complete post-review merge gates in user-behavior-handoff.md and inherited
handoffs stay pending. No full campaign/rootwhole-tier/docs_gen was run.

## Readiness

Work-owned acceptance from review: AC02/03/05/06/08 affected gaps now closed by the
above actual oracles; AC01/04/07 evidence retained with the source/SDK/guest identity
checks. Open in-contract work findings, mandatory gaps and unresolved decisions:
empty. T03/T05/T08 work marks may close; T09 independent affected acceptance/Spec
review and T10 complete frozen gates/landing remain open. Implementation committed
and clean; tree retained. No finalpass, canonical publication, merge, archive or
cleanup is claimed. Re-review should replay the Unit/constant findings and sibling
lexical/value controls, compare the proposal targets/versions, and reuse unaffected
evidence; complete unified gates belong after review pass.
