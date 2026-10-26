# PLAN-841 — F841-R5-01 repair readiness

stage: work; plan_revision: 1; outcome: pass (implementation verified, affected acceptance review pending). UI `bfad5149f8f1d1981465d06d759e95cc8127ab75`, lang `ecc2f54a7` (candidate Spec only), Man `6330ce0` (source pin only). Reviewed baselines were UI `2f311868f082bc4ab6fac9d484f71f18d13b1c95`, lang `88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80`, Man `9b4302fa585e6cda4a91738a6e9057fae12c9402`; down remains fixed read-only `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`.

## Diagnosis and actual path

The exact R5 producer probe was replayed before editing: direct `[.source]` passed; `[[.source]]` and `[{inner: .source}]` failed after the parent field was replaced, with the accepted nested `VmRef` left dangling ([red replay](red-replay.log.txt)). Earlier direct-element tests missed the recursive producer shapes and did not prove container destruction after replay.

The repaired path is actual parser/lowering and `AuraViewBuilder::resolve_expr_to_value` → queued snapshot → `ensure_child_state` replay → real Init → last field/container release. `visit_init_refs` now recursively accounts for accepted Array/Object values. Replay recursively materializes arrays as `ListData` and objects as `ObjectData`; every new container owns direct typed children under the existing Plan 419 protocol. This preserves reference identity and avoids both borrowed nested IDs and recursive-release leaks. Scalar `Int` classification, queue order, failure states, cancellation and scheduling are unchanged.

## Finding closure

| Finding / invariant | Affected paths/states | Fix and permanent oracle | Current result / gap |
|---|---|---|---|
| F841-R5-01 nested array queue owner | actual `[[.source]]`; parent overwrite, forced reap | recursive queue ownership; exact source ID and `[7]` readable; real Init and last release | pass; none in work scope |
| F841-R5-01 object-in-array queue owner | actual `[{inner: .source}]`; same transitions | same recursive ownership plus ObjectData replay and field identity assertion | pass; none in work scope |
| Direct control and replayed-container ownership | `[.source]`, nested list/object, Init Done, queue release, last field removal | three permanent producer tests assert queued liveness, replayed heap shape, `done=true`, source readability and final Weak expiry | 3 pass; none |
| Cancel/missing/drop and aliases | compound snapshots containing two references to one source | permanent compound exit test asserts two queue shares and zero owners after each exit; prior shared-owner replay retained | 1 permanent test (3 states) + 3 prior cases pass; none |
| Existing direct/scalar/failure behavior | direct refs, integer ID collision, failed Init, duplicate/replacement | existing Plan 841 ownership tests unchanged and rerun | pass; none |

## Scoped verification on the frozen repair

- [Runtime ownership](runtime.log.txt): **9 pass** — five prior controls plus three real producer lifecycle tests and compound cancel/missing/drop/shared-alias coverage.
- [Affected UI controls](affected-ui.log.txt): **9 pass** — original M1 healthy/canary, M4 three, domain admission two, chained state and dynamic SVG neighbors two. Canary remains healthy 36 / runtime 14 with BarChart failed and siblings Done.
- [R5 owner replay](owner-replay.log.txt): **3 pass** — cancellation with another owner, aliased fields/same-value reseed, parked Init replacement.
- [Demand lifecycle](demand-lifecycle.log.txt): **6 pass** — once-per-generation, missing/failed terminal states, identity cancellation, dependent CPU ordering and outlet diagnostic. The first direct nextest invocation omitted the repository's required `test-cases` feature and did not compile unrelated AV tests; the standard scoped `cargo t plan711_init_` invocation is the recorded successful check, not a product failure or a whole-tier run.
- Total **27 distinct passing directed tests**, zero selected-case skips. Formatting and diff whitespace checks pass; temporary review probes were removed. Existing baseline warnings remain; no new debug output or product configuration was added.

## Spec, source identity and handoff

SD-03's existing runtime rule now describes recursive accepted compound values, heap materialization and final release: [candidate-only delta](spec-delta-r5.diff), SHA256 `2ff00020fa7de68fd51041356cedc3f1efbbd7f0d3cc2f6b4013fe6233423708`. No canonical publication or live-ledger edit. Man's actual UI source pin parses and equals `bfad5149f8f1d1981465d06d759e95cc8127ab75`; full distribution is not rerun in this repair. Exact hashes and environment: [baseline](baseline.json.txt).

Work-owned AC-03/04/05 affected behavior is ready for affected review; unchanged AC-01/02, codec, generator and cold-import evidence is reused only outside this repair's code path. No open in-contract finding, coverage gap or unresolved semantic decision remains in this finding scope. T-07/T-08/T-09 are work-complete; T-10 remains review-owned.

Unified bare `cargo t`, applicable `tv`/`tt`/`tu`, UI/Man tiers, frozen 777/792/distribution and landing remain **merge-owned pending** after acceptance review. No whole-tier gate, merge, archive, ledger publication, worktree cleanup or U841 completion is claimed.
