# PLAN-841:r1 — merge receipt（merge-r2 轮，已收口）

stage: merge; outcome: **pass**; entry state: reviewed 10/12（R6 pass）→ 终态 12/12 archived。

| Checkpoint | State / evidence |
|---|---|
| prepared | R6 pass + 本轮 GATE-05（UI 4903bef）与 GATE-06（lang 9d96d0f76）修复，各经独立受影响复审 pass（红-绿反证/反事实因果证明）；冻结基线逐门断言核验 |
| landed | lang `f16bb468b`（21 提交两轮 rebase——docs/plans 取主检出权威簿记、goals.md 并集、代码面 delta 哈希等价 `f6e6d7fd`、specs.json 零增量，ff-only；[range-diff](lang-range-diff.txt)）、UI `4903bef`（15 提交纯 ff-only）、Man `9a75d3b`（10 提交纯 ff-only）；有序 UI→lang→Man，master tip==delivery 逐仓核验；主检出冒烟 check 绿 |
| ledger_refreshed | `.autoos/specs.json` v100→v101 guarded offline projection（store HTTP 不可达存证 [store-channel-r2.json.txt](store-channel-r2.json.txt)；P841-D1/T/R6/M 四项；byte 备份 `.autoos/specs.json.bak-plan841-merge`；schema/读回/无关段零扰动守卫全过；live 词表镜像补 `active`） |
| archived | `docs/plans/archive/841-ui-domain-ast-token-lowering-finalization.md`，status: archived（收尾提交随本收据） |
| cleaned | 逐树 wt-guard + worktree/branch 移除（组内外来 auto-down WIP、auto-down-clean、scratch-792* 与 777 工件按既往保全） |

## 门禁总账（绑定最终交付身份）

- Phase A：ui-tool-build ✓；ui-t ✓（a4/a5 金丝雀绿、FAIL 精确⊆[预存对](preexisting-reds-attribution.md)——pre-841 主检配对同签名复现）；ui-tu ✓；ui-ti ✓；lang-t 3183/3183 ✓；lang-tv 163 ✓；lang-tt 3555 ✓；te-child --locked ✓；lang-tu=既有 B855-FEATURE-01（归因 821/827/822，fix-forward 859，841 未触碰）
- Phase B：clone 断言 ✓；man-ui-tool/child bootstrap ✓；man-t ✓；man-tcore ✓；man-ti 42/43（816-smoke 预存族，832 先例同集）；发行 **DST-1..7 + FLR-1..6 = 14/14 ✓**（[distribution-receipt-r2.json.txt](distribution-receipt-r2.json.txt)）
- Phase C：真实 792 **39/39 ✓**（[c-792-receipt.json](c-792-receipt.json)，绑定 lang `1c81c8f42`）
- Phase D：777 strict **0 violations** ✓（[strict-777-final.json.txt](strict-777-final.json.txt)；GATE-06 后基线显式重采样——stale_baseline 按设计拒绝默默通行）

## 修复链（本轮 merge-owned findings）

1. **M841-GATE-05**：parked Init 段 resume 失败不回写需求相位，滞留 InFlight 被下轮派发"已出册=收敛"静默翻 Done（整档负载下 plan492 金丝雀实录；隔离跑同步失败走 Err 臂故恒绿）。修复=两处 resume 失败臂回写 + `max_steps=1` 确定性回归测试；UI `4903bef` + Man pin `22cec06`；独立受影响复审 pass（[finding](M841-GATE-05/finding.md)/[复审](repair-review-gate05.md)）。
2. **M841-GATE-06**：AutoStr=ecow::EcoString 别名，auto-val serde feature 未自链 ecow/serde——安装态 Core SDK 独立闭包编译 auto-ast derive 346 错（主工作区被 workspace 级 ecow/serde 统一掩盖；SDK 工作区清单源自 Man 根 Cargo.toml）。修复=`serde = ["dep:serde","ecow/serde"]`（lang `9d96d0f76`）；附带 Man 主/Core 锁同步（`e32bed1`/`9a75d3b`）；独立受影响复审 pass 含反事实因果证明（[复审](repair-review-gate06.md)）。
3. 环境事实：Windows MAX_PATH（770 深层附件）→ 子进程注入 `core.longpaths`（仅 env）；磁盘耗尽一次（UI 树 target 48G 清偿重建）。

## 已知 flake 债项

M841-KNOWN-FLAKE-01（plan492 金丝雀恢复段 1/4 整档频率，见 KNOWN-DEBT 与 [归因附录](preexisting-reds-attribution.md)）；预存红族：ui-t 两枚、man-ti 816-smoke、lang-tu B855-FEATURE-01。

## 保护与保全

主检出他 session 文档 WIP 全程保全（stash→merge→pop；00-intro 冲突按并集解决、章号 48→49 衔接，仍为未提交 WIP）；KNOWN-DEBT 债项登记随其工作树；组内外来 auto-down WIP 未触碰。
