# PLAN-829 T-11 merge-owned gates — executed on frozen `ddbde3e62857`

2026-10-22 (Asia/Shanghai), merge session. Worktree
`D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`, HEAD
`ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74` (R18 reviewed commit), base
`fdb1fad2ce7afa124adfa3080323496df384e670`. Tree clean before and after;
`git diff --name-only base..HEAD` is 127 files, all under
`experimental/autographics-prototype/` (experimental-only scope — no root
test tier / docs_gen triggered, per Plan §8 T-11 and R18). No leftover
`graphics-probe`/`iced_spike` processes before the windowed case (count 0).

All commands below ran serially inside the worktree with the frozen AGP lock
(SHA256 `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`).

## Gate results

| Gate | Command (AGP manifest) | Result |
|---|---|---|
| fmt | `cargo fmt --check` | exit 0 |
| check | `cargo check --locked --offline --all-targets` | exit 0, no warnings |
| full tests | `cargo test --locked --offline` (no filter) | **117 passed / 0 failed / 0 ignored** across 12 targets: lib 29 + prototype 16 + squash 17 + composite_gate 17 + iced_seam 5 + delivery_closure 2 + breakout 11 + threed 10 + cache 3 + verifier 7 + 2 empty bin unittests (graphics-probe, iced_spike) |
| probe matrix | `graphics-probe --case <id> --replay canonical --report scratch/829/merge-t11/<id>` × 10 | **10/10 exit 0** (see table below) |
| consumer relocation | `--case independent-consumer` (case-internal A→B relocate + real cold build) | `relocation.ok=true`, `case_success=true`, `mismatches=[]`, machine-specific fields separated |

Per-case exits (report + keyframes copied here; JSON kept as `.json.txt` per
repo ignore rules):

| Case | exit | frames |
|---|---|---|
| gpu-foundation (`--mode auto`) | 0 | 3 |
| shared-runtime (`--mode auto`) | 0 | 0 (logic case) |
| ag3d-risk-slice | 0 | 4 |
| ag2d-breakout | 0 | 4 |
| ag3d-squash | 0 | 4 |
| lifecycle | 0 | 0 (logic case) |
| metrics | 0 | 0 (logic case) |
| verifier-negative-controls | 0 | 0 (gate-only case) |
| independent-consumer | 0 | 0 (relocation case) |
| iced-shared-device (windowed, real input) | 0 | 15 |

`iced-shared-device/report.json.txt`: `body.composite.ok=true`, fingerprint
`agp_commit=ddbde3e62857`. All ten reports carry the same `ddbde3e62857`
fingerprint. Worktree-side raw output (stdout/stderr per case) remains at
`scratch/829/merge-t11/` in the worktree (gitignored).

Reproduction: the commands are exactly `experimental/autographics-prototype/README.md`
§"Build / test / run" (ten-case matrix), run from the worktree root.

## Not executed here (blocked)

- **Migration/write-surface handoff clearance** (`迁移完成交接门已明确放行合入`,
  Plan §8 T-11 precondition; §5.4 requires migration-owner receipts — partial
  814-A does not qualify). This is not a merge-executable gate; per R3/R5/R7/R18
  it is a user/migration-owner decision. Until granted: no landing, no
  canonical Spec deposit, no ledger refresh, no archive, no worktree cleanup.
- Consequently checkpoints `landed` / `ledger_refreshed` / `archived` /
  `cleaned` remain pending; the Plan stays `reviewed 10/11`, T-11 open.

Gate validity is bound to `ddbde3e62857` + lock `6cd34576…` + asset set
`983a78f540…` (per R18); any implementation/dependency/config/input change
invalidates the affected evidence and requires re-review/re-gates.
