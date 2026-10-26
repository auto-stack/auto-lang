# PLAN-850 r1 proposed canonical delta

Frozen code: `1db29917b7f7b26925cc1432fb3a74a0f4c01129`.

| Target | Change | SHA-256 |
|---|---|---|
| docs/specs/auto-lang/project.md | Formal cross-owner full aliases, actual ID/required selection checks, process failures, single-instance discipline, root identity and WIP report preservation | 254e88d5df3d1a8ae13e9c60941b527ecd46359490659588685a0ba1e8e9faa6 |
| docs/specs/auto-ui/project.md | Three-package UI full ownership, compiler/ui unification, full configuration and ui_gen coverage; daily t is not a full substitute | 638cdf50083f6e1a079e3b3cc3b962d596cfecc94a33b29956ae8d5016d36eb1 |
| docs/specs/auto-lang/plans.md | New module Plan index, currently executing; merge updates actual archive reference/status | d924c90529b9f9e8cbafb0a3ee09d2bde8979c4641e3f6eb5655dcc45f6fd3a2 |
| docs/specs/auto-ui/plans.md | New module Plan index, currently executing; merge updates actual archive reference/status | a247df87558c547e54c93141573ed0b81936aeb420ac9a323dcad90e087d8aa2 |

No product behavior, test oracle, support reduction, threshold or input-scale
change. The canonical delta is prepared only in the implementation worktree;
it has not been published to main. `spec-index.py` ran and produced no net
index delta. Live ledger publication is merge-owned and has not occurred.
