# PLAN-794 合入沉淀收据（scroll host provider）

- **stage**: merge
- **plan_id**: PLAN-794
- **plan_revision**: 1
- **outcome**: delivered
- **delivery_commit**: `5ddef1315818f363c9362cf8e7b6c53fe37c7837`
- **reviewed_commit（R2）**: `d027e5f617a3d33bb8b1897779569020f886f0e2`
- **base_commit**: `8b97d3c9bc9c86cb4d8ebde4a5d7856554e83b9c`
- **worktree**: `D:/autostack/.wt/lang-794/auto-lang` / branch `codex/plan-794-dev`
- **archive path**: `docs/plans/archive/794-scroll-host-provider.md`

## Checkpoint 回执

| Checkpoint | 状态 | 证据 |
|---|---|---|
| prepared | done | R2 pass@d027e5f61；冻结 SD SHA256=`84d2b2776faba28671437fbc91e708986408adb1240057639689923cca4c5bac`；Spec 目标=vm/overview + ui/overview + widgets/scroll-pane + ui/design/scroll-child-anchor |
| landed | done | ff-only `93c508d7d..5ddef1315`，无 merge commit；range-diff 等价：T-02 修复提交 `=` 全等，T-01/T-03 仅 Cargo.toml 并集上下文与 baseline 元数据差异（实现字节面不变） |
| ledger_refreshed | done | guarded offline projection（无 store writer）；`.autoos/specs.json` v35→v36；items P794-D1/D2/T/R2/M |
| archived | done | `docs/plans/archive/794-scroll-host-provider.md` + 本收据与 attachments/794 入库 |
| cleaned | done | wt-guard clean → worktree/分支/组目录移除 |

## Merge-owned 冻结门禁（最终 tip 5ddef1315）

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t` | **5335/5335 passed**（66.1s；sccache SCCACHE_SERVER_PORT=42917 绕开端口保留段） |
| `cargo tv` | **163/163 passed** |
| `cargo tu` | 不触发（ui_gen 零触面） |
| `cargo tf` | 批量档，见到期检查 |
| 777 strict | pass / 0 violations / 2 grandfathered（重锚后 declared_edges=48 @7e5225426） |

首轮冻结门禁在 `d027e5f61` 跑过 t 5335/5335 + tv 163/163；rebase 后交付身份变更（Cargo.toml 并集 + baseline 重锚），按规程在最终 tip 重跑，结果同绿。范围差异仅 master 并入的 exclude 成员（package-lock/diagnostics consumer）与基线元数据，本计划实现/测试字节面经 range-diff 确认不变。

## 规范沉淀

| SD | target | 落点 |
|---|---|---|
| SD-01 | docs/specs/auto-lang/vm/overview.md | `scroll native 宿主能力（PLAN-794）` 节 |
| SD-02 | docs/specs/auto-lang/ui/overview.md | `scroll controller adapter 宿主装配（PLAN-794）` 节 + 未迁族清单收窄 |
| SD-03 | docs/specs/widgets/scroll-pane.md | `宿主装配（PLAN-794）` 节 |
| SD-04 | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | Native 层调用边界迁 provider |

索引回写：`docs/specs/plans.md`、`auto-lang/vm/plans.md`、`auto-lang/ui/plans.md`、`goals.md`（GOAL-003/007）、`INDEX.md`（spec-index.py 再生成，无字节变化）。

## 验收映射（AC-01..05 由 R2 pass 覆盖，AC-06 本收据）

- AC-01..05：R2 `pass`（三 finding 闭合，VM 60/60，Vue 59/59 指纹复用，consumer 5/5）
- AC-06：本收据（冻结门禁+沉淀+归档+清理）
