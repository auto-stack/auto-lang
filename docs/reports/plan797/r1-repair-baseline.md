# PLAN-797 R1 修复基线（2026-10-16）

本目录是 T-07 `docs/reports/plan797/` 的工作产物。本文件记录 R1
needs_fix 后 **可独立完成项** 的修复基线；T-06（D-796/D-788）仍打开。

## 1. R1 findings 处置

| finding | 级别 | 处置 | 状态 |
|---|---|---|---|
| F-797-R1-03 | P2 | `experimental/auto-abi-probe/Cargo.lock` force-add 入提交；`--locked --offline` 解析/检查/测试通过 | **本轮关闭** |
| F-797-R1-02 | P2 | 补齐可独立准备材料：verify 脚本、本目录报告、spec-delta 草稿、freeze proposal、support-matrix、来源/并行审计 | **部分关闭**（终稿等 T-06） |
| F-797-R1-01 | P1 | 真实 semantic contract / 正式 AutoDown / 性能复测 | **仍打开**（等 D-796/D-788） |

## 2. Cargo.lock 冻结证据（F-797-R1-03）

| 项 | 值 |
|---|---|
| 路径 | `experimental/auto-abi-probe/Cargo.lock` |
| size | 27959 |
| SHA256 | `C7EC8DD5A8B49B0DDF510C5B470FC5C6DA873B2AB66B73009CCACED32F93ABC6` |
| git blob | `598833812a800c9d71574b5cf345f8583c082b88` |
| 入库方式 | `git add -f`（对抗根 `.gitignore:8` `Cargo.lock`） |
| 校验 | `git ls-files` 可见；`git hash-object` == index blob；`cargo metadata/check/test --locked --offline` exit 0 |

说明：manifest `0.126` 等是兼容约束，**不能**替代提交内解析锁。本机
锁与提交 blob 全等后，才允许在依赖未变前提下复用历史定向测试证据；
若后续依赖或锁变化，受影响定向验收须重跑。

## 3. 健康检查补丁

- `src/emit.rs`：删除 `add_data` 闭包多余 `mut`（`unused_mut`），
  使 probe crate `cargo check --locked` 零告警（path 依赖 `shim-metadata`
  的既有 doc-comment 告警不在本 crate 触面）。

## 4. T-07 可独立准备材料（本轮提交）

| 路径 | 内容 |
|---|---|
| `scripts/verify-autoabi-797.ps1` | `-Scope acceptance\|closeout` 验收门禁 |
| `docs/reports/plan797/freeze-proposal.md` | 候选 v0.1 freeze 提案 |
| `docs/reports/plan797/support-matrix.md` | 支持/拒绝矩阵 |
| `docs/plans/attachments/797/spec-delta.md` | SD-01–05 正文草稿 |
| `docs/plans/attachments/797/source-parallel-audit.md` | 来源/并行边界审计 |

**明确未完成（不得当作 T-07 关闭）**：V-01–11 最终收口、SD 终稿 hash
冻结、正式 AutoDown 执行与 AC-05 复测、快照退役收据。

## 5. 测试 / 门禁时序

- 本轮 work 修复只跑定向：`cargo check/test --locked --offline`（probe
  workspace）与 verify 脚本 acceptance 面。
- 不触发整档 `cargo t/tv/tt/tb/tu/taa`（2026-10-14 裁定：开发/修复提交
  不得触发整档）。
- merge-owned 门禁保持 pending：最终 review 通过后在冻结提交执行 ABI
  完整套件/真实 C 三腿 + 实际触面兼容；根 cargo t / tv/tt 由 merge 按
  diff 定界。

## 6. 下一步

1. 依赖门满足后：T-06 同树切换真实 semantic contract、退役快照、正式
   recipe/worker、全重测。
2. T-07 终稿：V-01–11 收口 + SD/支持矩阵/freeze 重绑 hash。
3. 再申请完整 final review（T-08）。
