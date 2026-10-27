# PLAN-841:r1 — merge receipt（merge-r2 轮，进行中）

stage: merge; plan state at entry: reviewed 10/12（R6 pass）。冻结入口：lang `ecc2f54a7`、UI `bfad5149`、Man `6330ce00`、down 固定 `c39ac7f9`。

## Checkpoints

| Checkpoint | State / evidence |
|---|---|
| prepared | R6 pass + GATE-05 修复（UI `4903bef`、Man pin `22cec06`，lang 代码未动）+ 独立受影响复审 pass（[repair-review-gate05.md](repair-review-gate05.md)）；冻结基线核验（三树 clean、HEAD 一致） |
| landed | （待填） |
| ledger_refreshed | （待填；store HTTP 不可达 → guarded offline projection，[store-channel-r2.json.txt](store-channel-r2.json.txt)） |
| archived | （待填） |
| cleaned | （待填） |

## 门禁记录

- Phase A 首跑（入口冻结基线）：ui-t 2203 运行 2202 过/1 败——plan492 金丝雀整档负载红 → 归因 [M841-GATE-05](M841-GATE-05/finding.md)（parked Init 段 resume 失败不回写需求相位，滞留 InFlight 被静默翻 Done；两处 resume 泵失败臂同类缺口）。修复=两臂回写 `mark_init_demand_failed_by_fn` + 确定性 park 回归测试；定向 plan841 快照 10/plan492 14/plan437 5/vm_bridge 55 全绿；独立受影响复审 pass（红-绿反证）。UI `bfad5149`→`4903bef`、Man pin→`22cec06`。
- Phase A2/A3（新冻结基线）：（待填——ui-t 金丝雀全档通过；两枚预存红按归因容忍，见 [preexisting-reds-attribution.md](preexisting-reds-attribution.md)：pre-841 主检配对同签名复现，T-08 closeout §5 既有基线债对）
- Phase D 777 strict：pass，0 violations / 4 grandfathered（[strict-777-r2.json.txt](strict-777-r2.json.txt)）。
- Phase B Man 干净 clone 门 + 发行 DST/FLR 14 例：（待填）
- Phase C 真实 792：（待填）

## 保护与保全

主检出他 session 文档 WIP 原样保全；合入时仅对与分支相交的 3 个簿记文件（00-intro/KNOWN-DEBT/INDEX）做 stash→merge→pop。组内外来 auto-down WIP 树与 scratch 目录不动。
