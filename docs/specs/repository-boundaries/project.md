# repository-boundaries

> **Status**: active（PLAN-784 SD-01）
> 范围：auto-lang / auto-ui / auto-man 三仓身份、旧实现保全与 v0.6 拆分迁移准备状态

模块子页：[generated-ui-distribution](generated-ui-distribution.md)（PLAN-792 SD-01：
生成式 Rust UI 交付布局/消费根 patch/应用独立锁/资源 allowlist 与 producer
身份合同——G/H 搬仓硬门的首个可执行发行面）。

v0.6 模块化（[Design 37](../../design/37-v06-modularization-and-repo-boundaries.md)）复用既有
auto-ui / auto-man 仓身份与远端 URL；不另建新仓。本页登记三仓**当前真实状态**与
legacy 保全锚，目标态拆分不在本页冒称已完成。

## 三仓身份（2026-10-15，PLAN-784 T01 冻结）

| 仓 | 主检出 | 默认/当前分支 | 保全时 tip | 远端 |
|---|---|---|---|---|
| auto-lang | `D:/autostack/auto-lang` | master | （活跃开发仓，不入 legacy 保全面） | gitcode + gitee `auto-stack/auto-lang` |
| auto-ui | `D:/autostack/auto-ui` | master | `f7d92eef`（2026-02-28 AURA pipeline） | origin = gitee `auto-stack/auto-ui` |
| auto-man | `D:/autostack/auto-man` | master | `51fafe82`（2026-01-21） | origin = gitee `auto-stack/auto-man` |

auto-ui 主检出带未提交 `M Cargo.lock`（原始字节已保全）；auto-man 工作区干净、
其根 `Cargo.lock` 与 `.vscode/iar-vsc.json` 为 ignored 文件（均已按需保全）。
两仓各自 CLAUDE.md 为适用 agent 指令；auto-ui CLAUDE.md 中"master 已被清空"表述
与实际满树不符（过时注记，以实际 `ls-tree HEAD` 为准）。

## legacy 保全锚（PLAN-784 交付，本地 refs，未 push）

两旧仓各建立（同 tip 幂等复用、异 tip 拒绝移动，工具
`scripts/legacy-preservation.py`）：

- 分支 `legacy/pre-v06-extraction` —— 指向保全时旧 tip；
- 标签 `legacy-pre-v06-<tip短SHA>`（auto-ui `legacy-pre-v06-f7d92eef`，
  auto-man `legacy-pre-v06-51fafe82`）—— 不可挪动的归档锚；
- 全量 git bundle（含全部 refs+HEAD）+ 未提交/ignored 载荷的原始字节备份 +
  SHA256 清单，位于仓外独立恢复位置（路径与 hash 见
  [PLAN-784 保全收据](../../plans/reports/784-repo-preservation.md)）；
  保全为**状态寻址快照**（`snapshots/<snap-id>/`，snap-id 由 tip+**完整
  refs→objectID 映射**+WIP+保留内容 hash 派生）：同态重跑幂等复用、任何状态
  变化（含仅 refs 变化）新增独立快照、既有快照永不被覆盖；创建/复用/复核三处
  均断言 bundle 实际承载 refs 覆盖冻结映射，verify 对全部保留原文件（含
  ignored）做漂移检测；恢复演练（disposable clone）验证 refs/tree/tag 完整可恢复。

## 迁移准备状态

- 准备分支 `codex/v06-extraction`（两旧仓各一，自保全 tip 建立）检出于
  `D:/autostack/.wt/lang-784/{auto-ui,auto-man}` 隔离树；**当前未替换任何代码树**。
- auto-lang 侧计划执行树：`D:/autostack/.wt/lang-784/auto-lang`（`plan-784-dev`）。
- 组内无 junction/symlink（wt-guard 验证）。

## 治理规则（现行）

- 旧仓主分支在迁移验收前**维持旧版本**：不 reset / orphan / force-push /
  清空默认分支；新实现以新提交进入准备分支，验收后才推进默认分支。
- 默认分支更名（master→main）与远端 push 需单独授权，不属拆分前置。
- 旧实现保留在 legacy refs 与历史中；新默认代码树不同时保留另一套生产源码。
- 跨仓依赖按 env → 组内兄弟 → 主检出解析；worktree 内禁建 junction/symlink。
- 新增跨界依赖由 [模块边界门](../design/repository-layout.md)
  （`.autoos/module-boundaries.toml` + 冻结基线）确定性判红；基线当前锚
  `7d51845ff`（39 声明边，PLAN-784 T04 刷新），过期基线显式判红不允许默默通行。

## 迁出入口清单

迁出候选路径 → 未来 owner → 资源/生成物/测试/消费者 → 在途计划的重放基线表
见 [PLAN-784 迁出清单附件](../../plans/attachments/784/t04/migration-inventory.md)；
编排顺序见[正式拆分与搬迁路线](../../plans/reports/v06-extraction-execution-roadmap.md)。
