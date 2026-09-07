---
plan_id: PLAN-584
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B 桌面域搬迁设计文档（auto-lang → auto-os）
author: [zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 纯文档计划：产出在 auto-os 仓，本仓仅动 docs/plans/ 记账
current_step: 0
total_steps: 9
---

# [PLAN-584] Stage B 桌面域搬迁设计文档（auto-lang → auto-os）

## 变更摘要

L2「桌面域资产搬迁 auto-lang → auto-os（Stage B）」的第一步：产出架构设计文档，落
**auto-os 仓** `docs/design/01-stage-b-desktop-migration.md`，一次定案七件事——
①随迁资产清单 ②框架层留架清单 ③`auto desktop` 入口委托方式 ④两项技术清障方案
（rust-server 落点硬编码 / 桌面注册表默认指向框架仓）⑤在途桌面域计划处置裁定
（用户 2026-09-07 已裁定，本文固化入档）⑥验证矩阵 ⑦后续实施拆解（Stage B 拆 N 个
执行 plan 的原则与排序）。同时把 579/583 执行期发现的清障项与 VM 债族落入本仓
`docs/plans/KNOWN-DEBT-AND-RISKS.md` 台账。

**门禁分类：Category A（纯文档/计划跟踪）**——全程不触 `crates/` 下 Rust 源码，
严禁 `cargo t` 与 `docs_gen`（AGENTS.md 分级门禁）。

## 目标

- **G1** 设计文档落地：`D:/autostack/auto-os/docs/design/01-stage-b-desktop-migration.md`
  （auto-os 尚无 docs/design/ 目录，本计划创建之并建 `00-intro.md` 索引，循 L2 流程）。
- **G2** 资产清单定案：随迁项每项三要素（auto-lang 源路径 / auto-os 目标路径 / 依赖的
  框架 API 面）。已锚定的核心随迁资产：shell 四件
  `crates/auto-lang/assets/{shell,desktop,switcher,notification_center}.at`、
  `examples/ui/028-launcher/`、桌面程序台账 `docs/plans/autos-desktop-program.md`、
  examples/ui 0xx 系列逐项归属裁定、与 auto-os `apps.manifest` 伞形清单的对接面。
- **G3** 留架清单定案：框架层不动清单（VM/编译器/AutoUI 框架等，约 8 万行量级——以
  T3 实测盘点为准）及每项与桌面域的耦合点标注。
- **G4** 委托方式定案：auto-os 侧如何消费 auto-lang 框架跑桌面（`auto desktop` 入口
  归属 / manifest 供给方式），零 junction 红线 + 跨仓解析序（env → 组内 → 主检出）约束。
- **G5** 两项清障方案定案（Stage B 硬前置，含代码锚点、改造方案、向后兼容分析）。
- **G6** 处置裁定固化：在途桌面域计划「先收口 or 先随迁」的用户裁定入档（见需求分析
  §处置裁定表）。
- **G7** 验证矩阵定案：实机 + desktop_mcp 基线 + 双端（vue/vm）一致性抽查。
- **G8** 本仓 KNOWN-DEBT 落账：两项清障债 + VM 债族修复指引（583 台账 + scratch/p583
  复现器指引）。

## 架构方案

本计划的产出物**是**设计文档，故本节定义设计文档的章节规格与执行形态：

**执行形态裁定**（Category A 特例，区别于常规 L1/L2）：
- 不开 `lang-584` worktree——本计划对 auto-lang **零代码改动**，仅 plan 文件与
  KNOWN-DEBT 台账在主检出记账（master）。
- auto-os 侧写入循 **Plan 579 先例**直接在其 main 提交（该仓 git log 三笔 579 提交
  `b206780`/`71f01f8`/`f5d4a64` 均直接 main、未开 worktree；doc-only 不适用代码 worktree 纪律）。
- 跨仓红线延续：任何情况下不在 worktree 内建 junction/symlink；跨仓依赖只走解析序。

**设计文档章节规格**（§1–§7 对应 G2–G7，§7 为拆解）：

| 节 | 内容边界 | 定案产物 |
|---|---|---|
| §1 资产清单 | 随迁项三要素表 + apps.manifest 对接面 | 资产表（含源路径存在性对账） |
| §2 留架清单 | 框架层不动项 + 耦合点 | 留架表 |
| §3 委托方式 | 选项空间 a/b/c + 定案 + ADR 理由 | 委托裁定 |
| §4 两项清障 | 锚点/方案/兼容分析/验收命令 ×2 | 清障方案 |
| §5 处置裁定 | 用户裁定原文 + 处置表 + 迁移机制 | 裁定档案 |
| §6 验证矩阵 | 实机/desktop_mcp/双端基线 | 验证矩阵表 |
| §7 实施拆解 | Stage B 拆 N plan 的原则、排序、依赖 | 拆解蓝图 |

**搬迁总原则**（设计文档开篇定调）：框架留 auto-lang（单一真相源），桌面域资产与
计划台账迁 auto-os（应用轨道仓），跨仓依赖一律走既定解析序，绝不用链接。

## 需求分析与背景调查
（取材：docs/specs/overview.md 仓库地图 + 2026-09-07 实测盘点 + 579/583 归档台账）

### 1. Stage B 前提盘点（2026-09-07 实测）

579 注记的显式前置完成 **4/6**：525/526（旧 worktree 折叠）、566、576 已归档；
未完的 577（p534 债批，drafting）与 541（executing）均为桌面域在途——已并入下方
处置裁定，不再是独立阻塞项。

### 2. 在途计划实测与处置裁定表（用户 2026-09-07 裁定）

实测来源：`docs/plans/*.md` frontmatter + `git worktree list`。

| 计划 | 域 | 实测状态 | worktree | 处置 |
|---|---|---|---|---|
| 541-025-sys-monitor | 桌面 | **executing** | lang-541 在飞 | **auto-lang 收口**（已动工其一） |
| 582-playground-notes-explorer | 桌面/examples | **executing** | lang-582 在飞 | **auto-lang 收口**（已动工其二） |
| 535-desktop-ux-followups | 桌面 | drafting | 无 | **整体随迁 auto-os** |
| 554-clock-app | 桌面 | drafting | 无 | **整体随迁 auto-os** |
| 556-games-wave1 | 桌面 | drafting | 无 | **整体随迁 auto-os** |
| 557-tetris | 桌面 | drafting | 无 | **整体随迁 auto-os** |
| 558-klondike | 桌面 | drafting | 无 | **整体随迁 auto-os** |
| 577-p534-debt-batch-1 | 桌面债批 | drafting | 无 | **整体随迁 auto-os** |
| 578-desktop-gallery-apps | 桌面 | drafting | 无 | **整体随迁 auto-os** |
| 545-use-namespace-semantics | 语言域 | drafting | 无 | 留守 auto-lang |
| 570-py-subclass-factory | 语言域 | drafting | 无 | 留守 auto-lang |

**用户裁定原文**（2026-09-07，固化入档）：
> 我建议把已经动工的两个（541和另一个）完成，其他还没开始的，可以整个计划文件一起
> 搬迁过去，搬迁完毕后再在auto-os仓库执行。

即：已动工的 541/582 在本仓完成；未动工的桌面域 drafting 七项（535/554/556/557/558/
577/578）**计划文件整体迁往 auto-os**，搬迁完毕后在该仓执行；语言域 545/570 留守。
推论（写入设计文档 §5）：**搬迁窗口 = 541/582 收口之后、桌面域七项开工之前**——
七项随迁计划不存在双写冲突面。

### 3. 579/583 执行期发现的技术清障（Stage B 硬前置，583 归档台账在案）

1. **rust-server 后端落点硬编码**（583 台账债候选 R-D1）：`crates/auto-man/src/rust_ui.rs:354`
   `ensure_shared_workspace` + 调用点 `api_gen.rs:636/752/754/3179`——产物落点硬路由
   auto-lang 的 `examples/rust-workspace`。shell 迁到 auto-os 后若走 rust 后端，
   产物仍会污染框架仓。需要「落点可配 / 落 app 仓」改造。
2. **桌面注册表默认指向框架仓**：`crates/auto-lang/examples/ui_desktop.rs:31-34`
   `--apps-dir`（默认仓库 `examples/ui`）+ `crates/auto-man/src/vue.rs:5480` `--apps`
   同源——仓外 app（auto-kanban、未来的 auto-os/apps）要进桌面，注册表必须能指向
   auto-os 的 manifest/目录。这是搬迁的核心适配点（与 apps.manifest 伞形清单对接）。
3. **VM 债族（建议前置，非硬）**：`meta.source_root` 经 fn 返回字符串静默归 0，
   m12/m16 同族（CALL 结果内联作算术操作数 / 字符串跨 fn 返回特定形态）——583 台账
   「残留债候选」第 267 行在案，复现器留存 `scratch/p583`。shell.at 是重度 .at 逻辑
   代码，正是该形态的高密度用户——**搬迁回归前修掉比搬迁中踩雷便宜**。

### 4. auto-os 仓现状（2026-09-07 实测）

- 伞形 `apps.manifest` 已登记 kanban 首个真实 app（repo 形态，17100/17101）。
- auto-plan 脚手架就绪：`docs/plans/.next-id` 自 001 起号、`new-plan.sh` 适配
  `os-NNN` 组目录——随迁计划将重编号，映射策略见待澄清 #1。
- 无 `docs/design/` 目录（本计划创建）。

### 5. specs 取材

- overview 仓库地图：examples/ 51 项（ui 应用轨道 0xx 系列）、auto-cosmic（Smithay
  合成宿主线）、packages/ JS 四包——§1 资产盘点的边界与 §2 留架清单的取材源。
- GOAL-010「应用轨道出 examples——仓外首例」（579 注记）为本设计的目标锚。

## 详细设计

（= 设计文档各章的定案要求，由执行步骤 T1–T9 逐章落地；此处记录定案要求与选项空间）

- **§3 委托方式选项空间**：a) auto-os 直接依赖 auto-lang 二进制（env 解析序指路）；
  b) auto-lang CLI 保留 `auto desktop` 入口、auto-os 只供 manifest/注册表可指仓外；
  c) 双入口并存。定案标准：零 junction 红线、跨仓解析序、向后兼容既有 028-launcher
  启动方式、579 已验证的 vm 模式跳过路径。初值倾向见待澄清 #2。
- **§4 清障一（rust 落点）**：锚点 `rust_ui.rs:354`；方向——落点由调用方/配置注入
  （env 或参数），默认值保持现状（向后兼容）；验收命令以「auto-os 仓外跑 rust 后端，
  产物零落 auto-lang」为准。T5 须先盘点 583 fold 后 R-D1 烟测是否已补（归档台账
  229/262 行）。
- **§4 清障二（注册表）**：锚点 `ui_desktop.rs:31-34` / `vue.rs:5480`；方向——
  `--apps-dir`/`--apps` 支持指向仓外目录与 apps.manifest 伞形清单（repo 形态 app），
  与 579 已落地的 manifest 结构对接。
- **§5 迁移机制**：计划文件整文件迁移（git 历史留痕：本仓 `git rm` + auto-os `git add`），
  编号映射按待澄清 #1 默认建议执行；本仓 INDEX/specs 不为随迁计划另留镜像，只在
  处置裁定表留一行去向。
- **§7 实施拆解排序**（承接用户裁定的推荐顺序）：设计文档（本计划）→ 541/582 收口 →
  桌面域七项随迁（纯文件迁移，一个 plan）→ 两项技术清障（可立小 plan，不依赖时机）→
  VM 债族修复 → 资产搬迁执行（拆 N plan）→ 随迁计划在 auto-os 逐个执行。

## 测试设计

Category A 门禁：**严禁 `cargo t` / `docs_gen`**。验证手段全为文档级：

1. **结构自查**：设计文档 §1–§7 齐全、零 TBD/TODO 占位（`grep -n "TBD\|TODO"` 零命中）。
2. **路径对账**：文档引用的所有 auto-lang / auto-os 路径逐条 `test -e` 存在性对账
   （bash 一行循环）。
3. **裁定对账**：§5 处置表与本仓 `docs/plans/*.md` frontmatter 实测一致
   （executing×2 / drafting 桌面×7 / 留守×2）。
4. **台账落账检查**：KNOWN-DEBT 新条目含代码锚点（rust_ui.rs:354 / ui_desktop.rs:31 /
   vue.rs:5480）与 583 台账、scratch/p583 复现器指引。
5. **红线检查**：`git status` 全程无 `crates/` 改动。

## 验收标准

- **C1** 设计文档存在于
  `D:/autostack/auto-os/docs/design/01-stage-b-desktop-migration.md` + `00-intro.md`
  索引注册，§1–§7 齐全、零 TBD/TODO。
- **C2** §1 资产清单每项源路径实测存在；shell 四件 / 028-launcher / 程序台账
  autos-desktop-program.md 在列；examples/ui 0xx 系列逐项有归属裁定。
- **C3** §4 两项清障方案含上述代码锚点，各带向后兼容分析与验收命令；R-D1 fold 后
  烟测状态盘点结论在案。
- **C4** §5 处置裁定表与本仓 frontmatter 实测一致，用户裁定原文逐字入档。
- **C5** KNOWN-DEBT-AND-RISKS.md 新增：清障债×2 + VM 债族指引（含 583 归档路径
  `docs/plans/archive/583-vm-heaprc-fix-batch.md` 与 `scratch/p583` 复现器指引）。
- **C6** `git status`/`git diff` 证明全程未触 `crates/`（Category A 红线）；
  未运行 cargo t / docs_gen。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1** auto-os 建设计目录骨架：创建 `D:/autostack/auto-os/docs/design/00-intro.md`
  （索引页）与 `docs/design/01-stage-b-desktop-migration.md`（§1–§7 空节架）。
  验证：`grep -c "^## " D:/autostack/auto-os/docs/design/01-stage-b-desktop-migration.md` ≥ 7。
- **T2** 落 §1 资产清单：盘点并三要素落表（核心锚点：
  `crates/auto-lang/assets/{shell,desktop,switcher,notification_center}.at`、
  `examples/ui/028-launcher/`、`docs/plans/autos-desktop-program.md`、examples/ui
  0xx 逐项、`D:/autostack/auto-os/apps.manifest` 对接面）。
  验证：对文档内源路径逐条 `test -e` 对账（bash 循环零 miss）。
- **T3** 落 §2 留架清单：以 `docs/specs/overview.md` 仓库地图为源，列框架层不动项
  （VM/编译器/AutoUI 框架等）+ 耦合点标注。验证：`grep -n "TBD\|TODO"` 该文件零命中。
- **T4** 落 §3 委托方式：写选项空间 a/b/c + 定案 + ADR 式理由（含对 028-launcher
  现有启动方式的兼容分析）。验证：节内含明确「定案」标记行。
- **T5** 落 §4 两项清障方案：先盘点 R-D1 fold 后烟测状态（对照
  `docs/plans/archive/583-vm-heaprc-fix-batch.md` 229/262 行与 master 近期提交），
  再写方案（锚点 `rust_ui.rs:354`、`ui_desktop.rs:31-34`、`vue.rs:5480` + 兼容分析 +
  验收命令）。验证：`grep -n "ensure_shared_workspace\|apps-dir" 01-*.md` 锚点在案。
- **T6** 落 §5 处置裁定：裁定表 + 用户裁定原文逐字 + 迁移机制（整文件迁移 + 编号
  映射建议）。验证：与本仓 `grep -m1 '^status:' docs/plans/{541,582,535,554,556,557,558,577,578,545,570}-*.md` 实测对账一致。
- **T7** 落 §6 验证矩阵 + §7 实施拆解：矩阵表（实机/desktop_mcp/双端基线）+ 拆解
  蓝图（排序按详细设计 §7 推荐顺序）。验证：`grep -n "TBD\|TODO"` 该文件零命中。
- **T8** 本仓 KNOWN-DEBT 落账：`docs/plans/KNOWN-DEBT-AND-RISKS.md` 新增三条目
  （清障一 rust 落点 / 清障二注册表 / VM 债族指引），各含代码锚点与 583 台账 +
  `scratch/p583` 复现器指引。验证：`grep -n "rust_ui.rs:354\|source_root" docs/plans/KNOWN-DEBT-AND-RISKS.md` 命中。
- **T9** 提交与收尾：auto-os 侧 main 提交设计文档（循 579 先例，
  `git -C D:/autostack/auto-os add docs/design && git commit`）；本仓 master 提交
  plan 记账与 KNOWN-DEBT。验证：`git -C D:/autostack/auto-os log --oneline -1` 见
  设计文档提交；`git -C D:/autostack/auto-lang status` 无 crates/ 改动。

## 复审记录

## 待澄清事项

- **#1 随迁计划编号策略**：auto-os plans 自 001 起号，随迁七项（535/554/556/557/558/
  577/578）编号必然变化。默认建议：auto-os 侧重编 `os-NNN`，frontmatter 加
  `origin: PLAN-5xx` 溯源注记，本仓处置裁定表留去向一行；备选：保留 5xx 原号（两仓
  号段混用，不推荐）。按默认执行，除非用户改裁。
- **#2 委托方向初值**：T4 定案时默认倾向选项 b（auto-lang 保留 `auto desktop` 入口 +
  注册表可指仓外 manifest），理由：改动面最小、延续 579 已验证的 vm 模式路径；定案
  在设计文档 §3 留 ADR 记录，复审时可翻案。
- **#3 rust 落点改造的第一验收用例**：是否定为 auto-kanban（583 已有 vm 模式 586 卡
  对账基线；rust 路径 E2E 因 R-D1 顺延）。默认：是。
