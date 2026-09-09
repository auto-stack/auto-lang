# auto-lang 计划索引（Stage B 随迁指针）

> **性质**：桌面域资产随迁（Stage B，[auto-os Design 01](
> ../../auto-os/docs/design/01-stage-b-desktop-migration.md)，Plan 584 定案）
> 产生的**去向指针页**——随迁计划的单一事实源在 auto-os 侧，本页只留指针行
> （防腐：单一事实源，Design 01 §5 迁移机制 4）。本仓活跃计划清单仍以
> `docs/plans/` 目录与 `.autoos/specs.json` 为准，本页不承载其维护。

## 桌面域计划随迁（P-1 批，2026-09-07，auto-os PLAN-001）

| 原 auto-lang 计划 | 去向（auto-os docs/plans/） | 迁移时状态 |
|---|---|---|
| 535-desktop-ux-followups.md | 002-desktop-ux-followups.md（origin PLAN-535） | drafting 原状 |
| 554-clock-app.md | 003-clock-app.md（origin PLAN-554） | drafting 原状 |
| 556-games-wave1.md | 004-games-wave1.md（origin PLAN-556） | drafting 原状 |
| 557-tetris.md | 005-tetris.md（origin PLAN-557） | drafting 原状 |
| 558-klondike.md | 006-klondike.md（origin PLAN-558） | drafting 原状 |
| 577-p534-debt-batch-1.md | 007-p534-debt-batch-1.md（origin PLAN-577） | 📦 已交付归档（2026-09-09） |
| 578-desktop-gallery-apps.md | 008-desktop-gallery-apps.md（origin PLAN-578） | drafting 原状 |

注：

1. **编号 577 消歧**——随迁对象为 p534 债批一（577-p534-debt-batch-1），
   与本仓 archive 既有 577-emitter-gaps-batch（trans 域，已归档）无关；
   活动区编号冲突随本次 git rm 消解（补账提交 42219a602 即为保全其
   git 历史而生）。
2. **留守未迁**：545-use-namespace-semantics、570-py-subclass-factory
   （语言域，Design 01 §5 处置表裁定）。
3. **本仓收口不随迁**：541（025 系）、582（notes explorer）——产出资产
   随 P-5 本体批迁入 auto-os。

## 桌面域资产随迁（P-5 本体批，2026-09-07，auto-lang PLAN-590）

| 原 auto-lang 资产 | 去向（auto-os） | 备注 |
|---|---|---|
| docs/plans/autos-desktop-program.md（**桌面程序台账**） | docs/plans/autos-desktop-program.md | 活账接棒——桌面域计划状态变更此后只记 auto-os 侧台账（Design 01 §5 迁移机制 4） |
| examples/ui/028-launcher/ | apps/028-launcher/ | 注册表型特权 app；经 P-3 容器探测注册 |
| examples/ui/025-sys-monitor/ | apps/025-sys-monitor/ | 541 终态主目录 |
| examples/ui/038-minesweeper/ | apps/038-minesweeper/ | games-wave1 基底 |
| examples/ui/common/settings/ | apps/common/settings/ | 唯一消费方 ui-gallery 随迁同批 |
| examples/ui-gallery/ | ui-gallery/（顶层） | 收割语料=框架 examples/ui（留架），`../auto-lang` 兄弟探测/`AUTO_GALLERY_APPS` |
| examples/widgets-gallery/ | widgets-gallery/（顶层） | 框架 docs/schema 管线语料锚改 `resolve_os_top_dir` 解析序（PLAN-590） |

注：B3（clock/tetris/klondike）无实物（计划已随 P-1 迁）；B5 前提修正——
582 产出=website playground，非 examples/ui 资产，无实物随迁（枚举定案
详表见 PLAN-590 与 auto-os Design 01 §1-B）。
