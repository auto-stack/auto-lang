# Proposed PLAN-837 r1 Spec delta — review freezes, merge deposits

SD-01 ADD docs/specs/auto-graphics/scene-hierarchy.md (AC02/03/04/08):
Finite authoring forest: shared nonzero u64 ID namespace, total128 nodes, same
2D/3D single group parent or null root, complete references and no cycles. Groups
inherit translation and visibility only; no group TRS/resource/material. Stable
parent-first traversal uses group declaration order then object declaration order
among siblings; original objects array independently preserves2D draw order.
No semantic depth truncation within128 nodes. Both local and derived world
coordinates finite abs<=100000, including hidden nodes;2D group z0. Auto profile4
receives local nodes plus checked parent ordinals, computes per-layer f32 addition
and local AND parent visibility. Rust checks all IDs/kinds/dimensions/parents/counts,
numbers and semantics; renderer/pick/outline consume actual returned values.
Any bad packet/resource leaves accepted content/history/selection and preview intact.

SD-02 MODIFY docs/specs/auto-graphics/scene-document.md (AC02/06/08):
Current save schema autographics.scene/2 adds groups plus explicit nullable parent
to each group/object. Group fields id/name/dimension(2d|3d)/parent/translation/visible;
leaf fields unchanged plus parent. Unknown/missing/duplicate fields and invalid
forest fail before consumers. v1 strictly migrates into root leaves/empty groups,
same IDs/local fields/order, without rewriting the source. Storage retains raw
original bytes for conflicts; saved canonical v2 semantic hash makes import clean.
Explicit Save/Save As writes v2, derived values and UI/history/runtime never persist.
Old readers do not read v2. New-process inspect rebuilds profile4, clean with no
history/selection.1MiB/128 Unicode/64 history steps/16MiB budgets retain meaning,
but the128 budget now includes groups and leaves. No old leaf is dropped to add groups.
Original local single-writer owned lock/temp/commit/readback recovery remains.

SD-03 MODIFY docs/specs/auto-graphics/authoring-session.md (AC03/04/06/08):
Replace/CreateGroup/Reparent/Delete-subtree are stamped transactions. Target identity,
selection and expected old parent gate structure commands. Reparent preserves only
world position: old world minus new parent world, unchanged descendant local fields
and own visible; effective visibility recomputed. Reject cycle/cross-dimension/leaf
parent/missing/bad numeric candidate. Recompute target+descendants and reject if
any coordinate moves beyond max(1e-3,8*f32EPSILON*max(1,abs(before),abs(after))).
No descendant compensation, clamping or relaxed precision. Group delete UI shows
descendant count and confirms; cancellation changes nothing. Undo/redo restore exact
subtree IDs/local fields/order in one before/after step. New edit clears redo;
created ID high-water does not retreat on delete/undo and exhaustion only prevents
new creation. Selection/drafts reset as before; stale target/parent/epoch/revision
or pending structure commands cannot mutate current content. Source profile3 remains
available as the unchanged original sample; current new/import editing explicitly
uses profile4. It never guesses profiles from payload shape.

SD-04 MODIFY docs/specs/auto-graphics/viewport-interaction.md (AC04/05/07/08):
Real stock-Iced2D/3D tree selects stable group/leaf IDs, explicit parent target/root
buttons and Keep world attach; local properties plus read-only actual world/effective.
Leaf geometry picking still selects leaves. Tree-selected group aggregates actual
visible descendant geometry bounds with one outline and origin translation handle.
2D origin cross10px hit radius,3D existing54px/7px X/Y/Z handles. Empty/hidden/alpha0
descendants or clipped/degenerate origin disable group handles with displayed reason;
tree properties remain. All overlay commands within4096. Indentation display capped8
without dropping deep nodes. Translation-only ancestors make target local delta equal
the world pointer delta while preserving parent offset; group drag changes group local
only, leaf drag changes leaf local only. Actual presented snapshot/camera/stamp drives
render/pick/outline.836 capture, bounded samples, last endpoint, failure/cancel/noop,
single-history and busy/stale/DPI/focus/resize/late-release rules remain.

SD-05 MODIFY docs/specs/auto-graphics/{auto-source-driver,prototype-profile,project,integration-contract}.md (AC01/07/08/10):
Document explicit profile4, current v2 finite forest and exact experimental CLI:
--hierarchy-check, --wrong-parent, --drop-release, --frames 1 incomplete, --inspect.
Self-check completion is resolved before serialization; incomplete/fault exits nonzero
and passed=false, full positive exits0/passed=true. New-process reconstruction stays independently checked.
Actual final-composition endpoint/origin pixels are compared to independent initial
leaf-color/background controls at temporary/move/undo/redo/hidden/show/saved states;
missing readback rejects. Native failure is serialized before nonzero exit and the
aggregate requires all seven state records. A test-only native copy injecting real
transparent endpoint or an old-position ghost must fail that same gate despite
valid Auto/state/commands, before paired reopen hashes can hide shared rendering
faults. These are finite2D fixture controls, not a whole-image/3D pixel-equivalence
claim. F-837-R1-01 corrected the earlier unimplemented visible-state oracle claim. Source identity is
plan837-identity.json; tools/verify_plan837.py --controls, check_hierarchy.py, check_hierarchy_pixel_faults.py and
package_consumer.py provide reproducible inputs. Fixed SDK9576b4d, locks, original
profile1/2 games, AGP/assets/goldens unchanged.7 resident meshes/14 initial writes
136952B, target/pipeline/mesh warm reuse; actual20 hierarchy rounds and retained-owner/
fresh-upload controls. CPU compile/frame/working-set measurements explicitly exclude
GPU timestamps, driver residency and Iced internal allocations. Source integration
only autographics-dev; master knowledge after reviewed landing. I1 remains delivered,
I2 production AutoUI/RQHost and I3 formal Forge/distribution remain pending. Finite
scene organization does not establish general TRS/Prefab/assets/game editing, public
stable engine APIs, production migration, Linux/Shader/CAP/ACE or full engine completion.
Final complete driver/AGP/native/composite/source/consumer gates remain merge-owned
after independent review. No delivery claim before actual T-10.
