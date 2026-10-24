# PLAN-840 proposed Spec delta (r2)

Proposal only. Stable delta IDs and targets match Plan840 revision2, section5.
Canonical Specs, overview/plans/goals and the derived ledger are merge-owned.
This repair changes no acceptance or production support boundary.

| Delta | Operation and exact target | Before → after rule | Rationale / acceptance | Implementation / evidence |
|---|---|---|---|---|
| SD-01 | add `docs/specs/auto-graphics/behavior-api.md` | No user behavior API → behavior/1 callbacks, run-scoped ordinal handles (0 valid,−1 absent), graph/input reads, translation/visibility writers, source identity/diagnostics/budgets/limits | Independent business source; AC-02/03/04/08 | api/behavior_v1.at and api/README.md signatures; behavior.rs admission; actual Auto/API tests |
| SD-02 | modify `docs/specs/auto-graphics/scene-play-session.md` | Builtin6 only → explicit UserV1/profile7 alongside Builtin6; full configuration before app_start once, app_update once/tick; immutable source snapshot; original frame/Pause/Step/Stop/fault/ownership contract retained | Preserve839 lifecycle; AC-04/05/07/08 | play.rs, behavior.rs, typed output consumer; source/init/tick/owner tests |
| SD-03 | modify `docs/specs/auto-graphics/authoring-session.md` | No source selection → Editing-only UI and ordinary --behavior CLI; actual scene-root resolution, recovery/new Run; schema3/Save do not persist/copy source selection | Explicit authoring/persistence boundary; AC-05/06/08 | host.rs and ordinary CLI, SaveAs/root/draft/busy tests |
| SD-04 | modify `docs/specs/auto-graphics/auto-source-driver.md`; `docs/specs/auto-graphics/prototype-profile.md`; `docs/specs/auto-graphics/project.md`; `docs/specs/auto-graphics/integration-contract.md` | Fixed experiment → finite public API and two portable source-only examples on one runner; document corrected SDK pin, profile7 and user-source limits while preserving all old profiles/denominators and production gates | No general engine/SDK or I2/I3 delivery claim; AC-01/06/07/08/10 | Move/Patrol, same-binary speed mutation/relocation, source closure/consumer; inherited full merge matrix retained |
| SD-05 | modify `docs/specs/language-sdk/project.md` | Inert comment/string lines could be pre-scanned as dependencies → shared lexical-state code-line recognition; real use/dep forms and order/parameters retained | Reconcile AST and SDK dependency scan; AC-01/02/03/07, T-01a/T-01b | Four-file canonical scanner fix; scanner38 and fresh/cache/module source cases; exact SDK artifact932ad487 |

## Canonical target versions for review

Hashes are SHA256 of the actual main-checkout UTF-8 source with CRLF→LF transport
normalization; the last path commit is additional provenance, not a substitute for
the source hash. Before merge, reconcile any changed target instead of replacing
newer canonical rules. For the new component the baseline is explicitly absent.

| Delta | Operation | Canonical target | Source SHA256 (LF) | Last path commit |
|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-graphics/behavior-api.md` | `absent` | `not yet created` |
| SD-02 | modify | `docs/specs/auto-graphics/scene-play-session.md` | `9f356527f9d4097466c7fa40085f9edeb3ca06680b5fa1a5c6260cfe2f08fb88` | `c14fc97fe8c83a571d6cf7a249cba74a76e218c7` |
| SD-03 | modify | `docs/specs/auto-graphics/authoring-session.md` | `86c430779635a1e1738f7f7da3e51d6e364cc9e6bd24efdf4f4fa3537769d8ca` | `c14fc97fe8c83a571d6cf7a249cba74a76e218c7` |
| SD-04 | modify | `docs/specs/auto-graphics/auto-source-driver.md` | `e1a2a70e6f478375b2e70f5f61c0ba7e1635b0907225786834b9a21cd393ac9d` | `c14fc97fe8c83a571d6cf7a249cba74a76e218c7` |
| SD-04 | modify | `docs/specs/auto-graphics/prototype-profile.md` | `575d8dbbf2573023d84f8cd7dd2ffcadc701fc33ec6a4e5e08ba58f8badd1555` | `c14fc97fe8c83a571d6cf7a249cba74a76e218c7` |
| SD-04 | modify | `docs/specs/auto-graphics/project.md` | `74f0a1febf3cb01712748b9161ab82ff1692fb8dfd956dbfe627a70764dcf24e` | `c14fc97fe8c83a571d6cf7a249cba74a76e218c7` |
| SD-04 | modify | `docs/specs/auto-graphics/integration-contract.md` | `47364f2de460181020f2c6289a448bef9b4fef761fcac7db4c335effa5cbf2bf` | `c14fc97fe8c83a571d6cf7a249cba74a76e218c7` |
| SD-05 | modify | `docs/specs/language-sdk/project.md` | `b2d721bf27aa2904ae62a512d073f45152fe0f802bc66300044b9175ece88aec` | `f6005ed3a671b8cbc7062ffb7e389faa9af9e7fe` |

Public source is one regular `.at` file, strict UTF-8 without BOM, at most256KiB.
Managed units are at most64KiB and the complete unit at most320KiB. Runtime
supports128 ordered nodes. Handles belong only to one run: zero is valid,
negative one means absent actor/parent. Invalid write/query arguments set the
first error, reject that frame, close the VM and preserve last-good state.
User source is snapshotted with raw/API/bridge/combined hashes, source regions,
resolved path and base scene/assets/actors/stamp. SDK byte spans are mapped to
original source; absent spans are explicitly reported, never invented.

No scene schema or package format change, external dependency loading, hot reload,
per-object script persistence, rotation/scale writer, native allocations hard quota
or sandbox guarantee is introduced. Ordinary Core helpers/state/control flow and
inert comments/strings remain supported. Bounds are source/host queue/frame/time
bounds, not a general VM heap accounting claim. See API documentation for exact
signature and errors. Both existing and new merge-owned matrices must pass on
the reviewed frozen revision before depositing these changes.

Callbacks require Unit returns throughout the complete expression traversal, not
just top-level statements. Nested functions/lambdas/closures have their own return
policy; their ordinary value returns remain legal. The public signatures remain
app_start() and app_update(dt float), not an inferred payload-based profile.

Finite pure initializers include literal/structural values, pure operators,
constant formatted/backtick strings and references to already initialized const
or immutable let bindings. Mutable, forward/unbound/cyclic reads, assignment or
compound assignment, user/API calls and private access remain rejected. User bytes
are never rewritten or evaluated by the admission classifier; the actual SDK VM
initializes the admitted values. Existing source/VM/native allocation limits and
trusted synchronous compilation caveats remain unchanged.
