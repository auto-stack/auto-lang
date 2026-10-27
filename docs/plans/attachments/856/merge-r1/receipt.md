# PLAN-856:r1 合入收据

## prepared

独立 final R2 通过的代码为 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`，基面 `05fdc81c546aff735eefdb6c39fbc5fb35dc546a`，SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。冻结 Spec delta SHA256 `9daf99d4cfbc5f482ea41e1c223e9a5a8625cca91a058a01693d9245f3f260f3`。规范草稿和输入哈希见 canonical-draft.patch/json；来源限定driver，源码仅落 autographics-dev。

## T-08 gate pass

生产端与独立消费端完整测试各80入口、300例、0 failed/ignored，实际ID相同；继承855全部288项并增加12项。21个生成文件与853基线逐字节一致，两端各7项相关控制通过。两端原新OS/GPU、内容、项目、sweep/motion与资源所有权门通过，原AGP 23槽逐项证明输入未变后复用。原始日志和二进制哈希见 evidence-index.json.txt，审计见 validated.json.txt、selected-ids.json.txt 和 agp-reuse.json.txt。

历史失败及恢复见 failure-closures.json.txt：消费端第一次中断原因未知，既有预算测试对切片时序的文案断言不稳定且SDK片内错误误指HTTP；这些未声称修复。窗口遮挡和phase0超时现场均保留，用户确认桌面可用后同一绑定场景与完整15项均通过。

## landed and ledger_refreshed

源码 `autographics-dev` 从 `05fdc81c546aff735eefdb6c39fbc5fb35dc546a` 快进至 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`。主检出规范提交 `2334f2326c3d46a9f82cbb8349af316197439784`，派生台账提交 `186965390aee93bdefbb52b85b32c46775a9565e`；八个ledger目标、版本99→100、哈希和守护式离线投影见 ledger-projection.json.txt。规范索引生成结果仅换行差异，未覆盖主检出的既有用户更改。

注意：本次执行遗漏了 landing 前应有的 wt-guard 扫描；源码和规范落地后补扫 `D:/autostack/.wt/lang-856/auto-lang` 结果 clean。此补扫不冒称事前扫描；删除工作树之前必须重新运行并通过 guard。

## archived

实际归档路径 `docs/plans/archive/856-autographics-2d-viewport-grid-and-navigation.md`，归档提交 `cf2477516eb4ca4ae39417672ecbe74c03c090b7`。

## cleaned

删除工作树前，确认实际绝对路径和分支归属，并立即运行 Git Bash 的 `wt-guard.sh`，结果 clean；工作树、已落地的开发及规范沉淀分支、空组目录均已移除。五处计划专属 Cargo target 已验证 CACHEDIR.TAG、无 reparse 并回收，见 cleanup.json.txt；SDK/research 的 966 个文件保留在 `D:/autostack/plan-artifacts/plan-856/recovery/` 并有逐文件 SHA 清单。原始日志和实际执行文件继续保留。归档任务10/10。批量回归到期状态单独核查。
