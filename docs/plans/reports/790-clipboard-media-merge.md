# PLAN-790 合入收据（文件/图像剪贴板宿主能力依赖倒置）

2026-10-16，plan_revision=1，stage=merge。

## 交付链（ff-only，无 merge commit）

`f9234e0c4`（实现：四 shim 依赖倒置+provider+adapter+fixture）→ `9a6cae26a`（R1 修复：Adler32 范围+严格 PNG 自校验+命名/fmt）→ `b57617392`（SD-01/02 沉淀：vm/ui overview）→ `e69639875`（ledger v31→v32 投影+plans.md 双索引）→ `249a80f2c`（777 基线重锚）= master tip。

rebase 哈希映射（master 并发推进=789 词法抽取合入）：`be5dfea0c→f9234e0c4`、`8c906b126→9a6cae26a`、`c2fbd245e→b57617392`；`git range-diff` 3 提交中 2/3 全等（`=`），commit1 仅 root Cargo.toml exclude 上下文适配（789 lex-consumer 行成为邻接上下文），补丁语义等价。

## 冻结门禁（merge-owned，绑定实际提交）

- 首基线 `8c906b126`（R2 审查冻结）：裸 `cargo t` **5390/5390**（首轮唯一红 plan730_raw_receive_commit_conflict_roundtrip 经 solo 绿+全量复跑绿归因为并行负载 flake）；`cargo tv` **163/163**；clipboard OS 定向 **30/30**（真 OS 往返含 plan485/785/790 族）。
- rebase 后基线 `249a80f2c`（基面并入 789 后重跑）：裸 `cargo t` **5326/5326**（计数差=789 迁移的测试面变化，两基线各自全绿）；`cargo tv` **163/163**；clipboard OS 定向 **30/30**（一次真 OS 剪贴板争用瞬态红，同提交即时复跑绿+首基线同测试绿——归因记录，未静默豁免）。
- 主检出落地后：`modularity-audit --mode strict` **pass 0 violations**（基线锚 e69639875 落 master 可解析）；scoped 冒烟 clipboard 族 30/30。

## 规范/账本

- SD-01/02 按 R2 冻结稿（03C8EB39...）沉淀入 `docs/specs/auto-lang/vm/overview.md` 与 `ui/overview.md`（ui 两处旧未迁清单同步更新，非只追加新节）。
- ledger v31→v32 受管离线投影（Regime B：字节级备份/程序化外科修改/存量逐字节等价仅 version 行变更/六 section schema 校验/回读 ID 验证）：P790-D1/D2（designs）、P790-T（tests）、P790-R2（reviews）、P790-M（reports，本文件），全部带 file+tags。
- plans.md 双索引加行。

## 事故记录（已修复，无数据损失）

rebase 冲突续行时编辑器异常将 commit message 写入 Cargo.toml（`cargo metadata` 解析失败即暴露），且 rerere 记忆污染解在二次 rebase 重放；修复=逐行剥离污染+`git -c core.editor=true rebase --continue`+清除共享 rr-cache（82 条——纯冲突解缓存，非数据，损失仅为后续 rebase 手工解冲突）。最终链逐提交验证零污染+manifest 可解析。

## 待办交接

- **批量回归到期**：790%5==0 且 789 收据已预告"下一触发=790 落地"→ `/auto-plan:regress` 主检出单实例执行 tf 档。
- 产物观察项（landing≠deployment）：本批为 VM/测试/docs 面，未改 UI 视觉/协议——auto 桌面 release 二进制与 vue dist 如有常驻生产部署需按需重建（与 785 同口径）。
