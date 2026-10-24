# PLAN-845 proposed Spec delta (r2)

Proposed only; canonical publication is merge-owned after final review and complete gates.

| ID | Operation/target | Before → after | Acceptance |
|---|---|---|---|
| SD-01 | add docs/specs/auto-graphics/behavior-project.md | No bound project → scene4 exact-byte CodeRef, behavior/1, nullable stable leaf IDs, immutable payload ownership and quotas, explicit bind/update/retarget/detach/cancel and temporary overrides. One source blob, no packages or path search. | AC-02/03/04/06/08 |
| SD-02 | modify docs/specs/auto-graphics/scene-document.md; scene-assets.md | scene3 GLB-only → strict scene4 required behavior/null; v1–3 strict memory migration without disk rewrite. Source hash/length checked before parser/compiler. Single scene rename commits full GLB+scripts closure under original root lock, code budget8/2MiB, reserved destination rejection before old receipt Verify, precommit retention and postcommit unverified owner/recovery. | AC-02/05/06/07 |
| SD-03 | modify docs/specs/auto-graphics/authoring-session.md; scene-play-session.md; behavior-api.md | File-only temporary sources → accepted binding/current/pending/history/save receipts, compile without callback execution, default project source/roles, explicit override/builtin, dirty/Undo exact bytes, delete reference rejection, busy/stale protection and Stop restoration. Existing behavior API/budgets unchanged. | AC-03/04/06/07 |
| SD-04 | modify docs/specs/auto-graphics/auto-source-driver.md; project.md; integration-contract.md | Manual script accompaniment → portable local scene/assets/scripts projects, same runner/new process real OS/GPU evidence and finite trusted limitations. Graphics source only autographics-dev. Production I2/I3 and full engine/distribution remain pending. | AC-01/06/08/10 |

Canonical source API and bridge declarations unchanged; two original locks and SDK932 retained. Root Rust/AGP/SDK source read-only. Source fence extends every delivered source/test/tool/sample and retained prior provenance. Final merge cannot claim delivery using work-only targeted checks.

Source origin clarification for SD-01/03 (F-845-R1-01 repair; existing §5.2 rule):
Default Run attributes accepted bytes to an explicit `<memory:SHA256>.at` until
that identity has been confirmed by project load or successful complete save
verification in the current project root. Geometry-only dirtiness does not change
source provenance; Undo/Redo use source identity. Retained identity-only records
are bounded to eight and own no payload bytes. Save As changes the root only when
verified and cannot transfer old-root source proof; an old-revision receipt
confirms its own source, not a newer accepted binding. Pending receipts still block
Run. Attribution records prior verified provenance, not a new disk-integrity or
readability promise: Run never rereads a confirmed blob; Open/Save/Verify detect
subsequent disk changes. Missing/unconfirmed records conservatively use memory.
