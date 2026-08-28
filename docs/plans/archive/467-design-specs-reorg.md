---
plan_id: PLAN-467
status: archived
feature_name: docs/design 整理 + docs/specs v2 + auto-plan 范式收敛
author: [zcode]
created_at: 2026-08-28
updated_at: 2026-08-28

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components:
  - "docs/specs/README.md: v1 整体重写为 v2（五环 spec-sync → auto-plan 四技能）"
  - "docs/design/plan-spec-hybrid-model.md: 加 superseded 横幅（被 26 取代）"
new_spec_components:
  - "docs/design/26-autoplan-spec-ledger.md: 新增（v2 范式设计）"
  - "docs/specs/overview.md: 新增（全局总览，/auto-plan:new 取材源）"
  - "docs/specs/goals.md: 新增（GOAL-001..018 目标账本）"
  - "docs/specs/{auto-cosmic,shim-metadata,a2r-actor-tests,autoui-skill}/project.md: 新增 4 张项目卡"
touched_goals:
  - "GOAL-018: 开发范式与知识账本运转（本计划即首个完整走通的样板）"


current_step: 12
total_steps: 12
---

# [PLAN-467] docs/design 整理 + docs/specs v2 重建 + auto-plan 四技能范式收敛

## 变更摘要

一次纯文档/元数据任务（Category A/C，不动 `crates/` 下任何 Rust 源码）：

1. **docs/design/ 重新整理**：修复编号冲突（两个 19-）、解散 stray `new/` 目录、战略路线图归入 `strategy/` 子目录、plan 报告类文档归位 `docs/plans/reports/`、重写 `00-intro.md` 为全量索引（111 个文件逐一分类+状态标注）。
2. **docs/specs/ v2 重建**：保留 v1 的 project/module 知识树，新增全局 `overview.md`（auto-plan:new 的取材源）与 `goals.md`（全局目标账本），重写 `README.md` 为 v2 规约——对齐 auto-plan-new/work/review/merge 四技能的 6 段 spec ledger 范式（goals/architecture/designs/tests/reviews/reports），定义本仓路径映射与沉淀程序。
3. **范式收敛**：AGENTS.md 与 `scripts/new-plan.sh` 对齐新范式（worktree 统一 `.worktrees/plan-<NNN>-dev`、归档目录 `archive/` 与技能文档 `archived/` 的映射声明、新式 frontmatter）。
4. **specs 覆盖补全**：为代码盘点发现的新 crate/资源补项目卡（auto-cosmic、shim-metadata、a2r-actor-tests、autoui-skill），孤儿项（auto-xml、auto-lang-macros、ac-examples）在 overview.md 标注；spec-index.py 分组更新并重生成 INDEX.md。
5. 新写 `docs/design/26-autoplan-spec-ledger.md`（v2 体系设计，supersede `plan-spec-hybrid-model.md`，旧文加 superseded 横幅保留）。

## 目标

- G1: docs/design 树内任何文件都能从 `00-intro.md` 索引按分类+状态找到；无编号冲突、无 stray 目录。
- G2: docs/specs 成为 auto-plan 四技能范式的落地账本：`overview.md` 可直接作为 `/auto-plan:new` Step 2 的背景材料；`goals.md` 供 review 填 `touched_goals` 引用；README v2 写清 6 段映射与本仓路径适配。
- G3: 四技能在本仓可零歧义执行：路径映射（archived/→archive/、worktree 命名、specs.json 位置）在 AGENTS.md 与 specs README 双处声明。
- G4: specs INDEX 覆盖全部 workspace 成员与主要根资源目录；孤儿/废弃项有明确标注。
- G5: 所有被移动文件的仓内链接全部修复（grep 验证零残留）；跨仓引用（auto-down→raw/、auto-musk→plans/）不受影响（已核实不指向被移动文件）。

## 架构方案

三份资产、三层职责（目标态）：

```
docs/design/    设计层：意图与方案（含历史素材 raw/）——"为什么这样设计"
docs/specs/     账本层：现状知识 + 6 段 ledger 映射——"现在是什么样"
docs/plans/     过程层：一次任务的时间线叙事（active + archive/）——"怎么做的"
.autoos/specs.json  机器账本（musk 后端 / merge 技能的 upsert 目标）
```

design 树目标结构：

```
docs/design/
├── 00-intro.md                 # 重写：全量索引 + 分类学 + 状态
├── 01..15-*.md                 # 语言核心/应用框架/AI（不动）
├── 16..24-*.md                 # AutoUI 时代主线（19 冲突消解后）
├── 25-a2ui-composer-analysis.md   # ← new/ 解散提级
├── 26-autoplan-spec-ledger.md     # ★ 新增：v2 体系设计
├── strategy/                   # ★ 战略路线图（4 篇迁入）
├── blocks/                     # Design 17 配套（不动）
├── forge/                      # AutoForge 附录（不动，索引标注外仓主题）
├── raw/                        # 历史草稿池（不动）
├── plan-spec-hybrid-model.md   # 保留 + superseded 横幅（被 26 取代）
├── ash-design-summary.md       # 保留（外仓 auto-shell 主题，含迁移注记）
├── dialect-extension-diagnosis.md  # 保留（被 6 处 specs 引用）
└── vm-debugging.md             # 保留（被 design/14 与 specs 引用）
```

specs 树目标结构（v1 树保留，新增 3 个全局件）：

```
docs/specs/
├── README.md       # v2 规约（重写）
├── INDEX.md        # 脚本重生成（覆盖补全后）
├── overview.md     # ★ 全局总览：仓库地图/模块状态/近期演进/孤儿清单
├── goals.md        # ★ 全局目标账本 GOAL-NNN
├── _archive/       # 保留
└── <project>/...   # v1 树保留 + 4 张新项目卡
```

## 技术栈

Markdown 文档 + `git mv` + 2 个 Python 脚本（spec-index.py / spec-lint.py）+ 1 个 bash 脚本（new-plan.sh）。不涉及 Rust 构建。

## 需求分析与背景调查

（2026-08-28 全仓调研结论，来源：docs/design 全量扫描、docs/plans 活跃+归档抽读、specs v1 树、.autoos/specs.json、四份 auto-plan 技能文档、代码盘点代理报告）

### design/ 现状问题（111 文件）

- 编号冲突：`16-appendix-025-gap-enumeration`（16 的附录，可接受）与 `19-plan-374-rust-type-compat` / `19-theming-and-dark-mode` 双 19 冲突（真问题）。
- `19-plan-374-*` 实为 plan 执行报告（标题"Plan 374: …第三轮进行中"），放错了层；零外部引用，可安全移动。
- stray 目录 `new/`（仅 1 篇 a2ui-composer-analysis，2026-08-25，实为 Design 16/21 同族的设计分析）。
- 4 篇战略路线图散在根目录（auto-as-rust-script-strategy / consumer-parity-strategy / python-parity-roadmap / rust-library-replication-roadmap），website 活页面 rust.md/python.md（含 zh）有链接。
- `00-intro.md` 停留在 2026-06-15 的"14 章 + forge 附录"视图，不覆盖 16-24 新章与后续新增。
- `plan-spec-hybrid-model.md`（2026-07-23）是 specs v1 的设计源，其五环流程已被 auto-plan 四技能范式实际取代（Plan 446/466 已按新范式走完全程并沉淀 specs.json）。

### plans/ 现状

- `.next-id`=467；active 26 个文件（多为 AutoUI 应用轨道 437-465 + a2r/lsp/build 线）；`archive/` 438 个。
- 高价值伴生资产：`KNOWN-DEBT-AND-RISKS.md`（维护良好，保留）、`plans-status-audit-2026-08-20.md`（最新审计，保留）、`reports/`（spike/bench 报告，保留）、`466-skill-backup/`（技能备份，保留）。
- 大量历史 plan 无 frontmatter（旧格式），不回溯改造；新 plan 一律走新范式 frontmatter。

### 两套流程并存（本次收敛对象）

| 维度 | 旧（AGENTS.md 现文） | 新（auto-plan 四技能，446/466 已用） |
|---|---|---|
| worktree | `.worktree/plan-<NNN>`（无 -dev） | `.worktrees/plan-<NNN>-dev` |
| 状态机 | draft/in-progress/complete | drafting→executing→execution_done→reviewed→archived |
| 归档目录 | `docs/plans/archive/` | 技能文档写 `docs/plans/archived/`（**本仓裁定：保留 archive/ 不改名**，理由：仓内 68 处引用 + archive-plan 技能 + auto-musk 跨仓引用均用 archive/；在 AGENTS.md/specs README 声明映射让技能适配） |
| spec 沉淀 | spec-sync 手工回写 module 树 | merge 技能 upsert `.autoos/specs.json`（P<seq>-<n> 条目） |

### 代码盘点要点（影响 specs 覆盖）

- 缺卡：auto-cosmic（4 子 crate，Linux 向 COSMIC 复刻实验，无消费者）、shim-metadata（rustdoc→FFI shim 管线，被 auto-lang/auto-cache 依赖）、a2r-actor-tests（actor parity 慢测试）、autoui-skill（AI 技能包，非 crate）。
- 孤儿/废弃：auto-xml（非 workspace 成员，Plan 325 注明已迁 ../auto-ai，Cargo.toml 却仍用 workspace 依赖，单建必失败）、auto-lang-macros（auto-macros 的逐字废弃副本）、ac-examples（无 Cargo.toml 的遗留目录）、根 auto-shell/（仅剩 Cargo.lock，crate 已迁独立仓）。
- 根资源目录：auto/（自举实验，=specs aavm 卡）、blocks/、packages/（4 JS 包）、parity/（独立 workspace）、website/、deploy/、schema/（aura.at 唯一声明源）、tools/、test/+tests/、stdlib/、examples/（51 项）。

## 详细设计

### D1 design 树移动清单（git mv + 链接修复）

| 源 | 目标 | 修复链接 |
|---|---|---|
| `docs/design/19-plan-374-rust-type-compat.md` | `docs/plans/reports/374-rust-type-compat.md`（文首加归属注记） | 无引用，零修复 |
| `docs/design/new/a2ui-composer-analysis.md` | `docs/design/25-a2ui-composer-analysis.md` | `archive/234-a3ui-a2vue-replica.md` 1 处 |
| `docs/design/auto-as-rust-script-strategy.md` | `docs/design/strategy/auto-as-rust-script-strategy.md` | website/rust.md、website/zh/rust.md、archive/359 |
| `docs/design/consumer-parity-strategy.md` | `docs/design/strategy/consumer-parity-strategy.md` | 同目录互链（文件名不变则无需改） |
| `docs/design/python-parity-roadmap.md` | `docs/design/strategy/python-parity-roadmap.md` | website/python.md、website/zh/python.md、archive/369、plans-360-369-status-summary |
| `docs/design/rust-library-replication-roadmap.md` | `docs/design/strategy/rust-library-replication-roadmap.md` | archive/347、archive/348 |

移动策略：**不留重定向存根**（仓内链接全量修复 + git `--follow` 可溯源；跨仓引用已核实不指向被移动文件）。website 链接修复时保持其相对路径形态（`../design/x` → `../design/strategy/x`，以实际形态为准）。

不动清单（被多处活文档引用）：dialect-extension-diagnosis（6 处 specs 引用）、vm-debugging（design/14 + specs runtime/vm plans.md）、ash-design-summary（design/11、15）、blocks/、forge/、raw/ 全部。

### D2 `00-intro.md` 重写要点

- 分类学：语言核心（01-10）/ 应用框架与生态（11-15）/ AutoUI 与 App 生成（16-25）/ 流程与体系（26 + plan-spec-hybrid-model）/ 战略路线图（strategy/）/ 专题诊断（根级 3 篇）/ 附录（blocks/、forge/）/ 历史素材（raw/，70 篇）。
- 每篇带状态标注（现行 / 已被取代 / 历史记录 / 外仓主题），AutoUI 主线标注与 plans 的对应关系。
- 声明 design↔specs↔plans 三层职责（与 26 号文档一致）。

### D3 `26-autoplan-spec-ledger.md` 新设计文档要点

- 诊断：v1（plan-spec-hybrid-model）为何停摆——五环 spec-sync 依赖人工回写，437 后无一次完整回写；两套流程并存期（.worktree 旧 vs .worktrees 新）。
- 设计：三层资产模型 + 6 段 ledger 映射（见 D6）+ 沉淀程序 + 状态机 + 路径映射表。
- 取舍：为何不改名 archived/（68 处引用 + 跨仓 + archive-plan 技能）；为何 specs.json 与 markdown 树并存（机器账本 vs 人类视图，merge 技能 upsert 前者、本仓规约要求同步后者）。

### D4 specs README v2 规约要点

- 保留 v1 的 project/module 概念与文档类型（project/overview/architecture/design/plans）。
- 新增：6 段 ledger 映射表（goals→goals.md+project.md 目标节；architecture→module architecture.md ADR；designs→module design/；tests→module overview 测试节或 design/testing-*；reviews/reports→plan 文件 + specs.json + plans.md 行）。
- 新增：/auto-plan:merge 在本仓的手工回退程序扩展（specs.json upsert 之后，追加 module plans.md 行 + 必要时 overview/ADR 回写 + INDEX 重生成）。
- 新增：路径映射表（技能路径 → 本仓路径）。
- 保留：编号/引用规则、反模式清单（v1 §7 全部继承）。

### D5 overview.md / goals.md

- overview.md（≤150 行）：仓库地图（crates 表 + 根资源表）、auto-lang 九模块状态行、活跃开发线（AutoUI 桌面轨道 / a2r parity / LSP / 构建）、孤儿清单、入口链接（INDEX/goals/KNOWN-DEBT/审计）。
- goals.md：GOAL-NNN 条目（从 roadmap.md 纲领 + 359/347/369 等战略 plan + 16/23/24 号设计提炼 8-12 条），每条含一句话 + 状态 + 关联 plan/design 链接。

### D6 项目卡补缺 + INDEX

- 新建 `docs/specs/auto-cosmic/project.md`、`shim-metadata/project.md`、`a2r-actor-tests/project.md`、`autoui-skill/project.md`（v1 project.md 模板）。
- `spec-index.py` GROUPS 更新：工具链 += shim-metadata；外围/验证 += a2r-actor-tests；UI/Web 生态 += autoui-skill；实验组（新增"实验/沙盒"）= auto-cosmic。
- 重生成 INDEX.md，核对 4 张新卡出现。

### D7 AGENTS.md / new-plan.sh 对齐

- AGENTS.md：§1 L1 worktree 路径改 `.worktrees/plan-<NNN>-dev`（分支同名）；§4 归档段补状态机与新 frontmatter 说明 + archived/→archive/ 映射声明；§5 清理命令同步；新增一小节"auto-plan 范式"指向 specs README v2。测试门禁分级（Category A/B/C）与 L0/L1/L2 分诊**保持不变**。
- new-plan.sh：骨架 frontmatter 换新范式（plan_id/status: drafting/feature_name/created_at/updated_at/current_step/total_steps + 预留 review 字段），正文骨架对齐 auto-plan-new 的 11 节结构（变更摘要…待澄清事项）；防御检查同步把 `archived/` 路径按本仓改为仅 `archive/`、`old/`。

## 测试设计

- `python scripts/spec-lint.py`：无 ERROR（链接断链为 ERROR 级）。
- 移动残留 grep：`grep -rn "design/new/"`, `"19-plan-374"`, `"strategy" 旧路径形态`（website 与 docs 内）均零命中（archive 历史正文中的叙述性提及允许，路径型链接不允许）。
- `python scripts/spec-index.py` 后 INDEX.md 含 4 张新卡且分组正确。
- `git status` 干净（全部提交）；`git log --follow` 抽查 1 个移动文件可溯源。

## 验收标准

- [x] AC1: `docs/design/` 无编号冲突；`new/` 目录不存在；`strategy/` 含 4 篇；19-plan-374 位于 `docs/plans/reports/`。
- [x] AC2: `00-intro.md` 索引覆盖 design/ 下全部文件（数量对账：111 - 1 迁出 + 2 新增 = 112）。
- [x] AC3: `docs/specs/` 存在 README v2 + overview.md + goals.md + 4 张新项目卡；INDEX.md 重生成且含新卡。
- [x] AC4: AGENTS.md worktree/归档/范式段更新；new-plan.sh 产出的骨架含新范式 frontmatter（bash -n 语法过）。
- [x] AC5: spec-lint 无 ERROR；移动残留 grep 零命中；工作树提交干净。
- [x] AC6: `26-autoplan-spec-ledger.md` 存在且 plan-spec-hybrid-model.md 带 superseded 横幅。

## 执行步骤

1. `git mv docs/design/19-plan-374-rust-type-compat.md docs/plans/reports/374-rust-type-compat.md`，文首加归属注记。验证：`ls docs/plans/reports/374-*`。
   [✅ 已完成] f1586a72c；归属注记已加，零外部引用
2. `git mv docs/design/new/a2ui-composer-analysis.md docs/design/25-a2ui-composer-analysis.md`，删除空 `new/`；修 `archive/234` 内链接。验证：`ls docs/design/new` 报不存在。
   [✅ 已完成] f1586a72c；archive/234 链接已修，new/ 已删
3. `mkdir docs/design/strategy` + 4 个 `git mv`；修复 website/{rust,python}.md、website/zh/{rust,python}.md、archive/359、369、347、348、plans-360-369-status-summary 内路径链接。验证：对每个旧文件名 grep 路径形链接零命中。
   [✅ 已完成] f1586a72c；偏差（已核实）：website/{rust,python}.md 的链接经查为站点路由（/docs/design/...）且 website/ 内无文件拷贝，与仓库路径解耦，未改动；实际修复 archive/359、369、347、348、plans-360-369-status-summary 与 consumer-parity 互链
4. `plan-spec-hybrid-model.md` 文首加 superseded 横幅（指向 26）。
   [✅ 已完成] f1586a72c；横幅指向 26
5. 重写 `docs/design/00-intro.md`（全量索引）。验证：索引文件数与 `find docs/design -type f | wc -l` 对账。
   [✅ 已完成] db5eade45 + 2291e2eeb；数量对账修正：实际 111 篇（raw 实为 67，计划期估算 70 有误），索引已按实际数
6. 新写 `docs/design/26-autoplan-spec-ledger.md`。验证：文件存在且含 6 段映射表与路径映射表。
   [✅ 已完成] db5eade45；含 6 段映射表（§2.3）与路径映射表（§4）
7. 重写 `docs/specs/README.md`（v2 规约）。验证：含路径映射表与 merge 扩展程序。
   [✅ 已完成] aa290cff2；含路径映射表（§5）与 merge 扩展程序（§4）
8. 新写 `docs/specs/overview.md` 与 `docs/specs/goals.md`。验证：两文件存在，overview ≤150 行。
   [✅ 已完成] aa290cff2；overview 128 行 ≤150
9. 新建 4 张项目卡（auto-cosmic / shim-metadata / a2r-actor-tests / autoui-skill）。验证：4 个 project.md 存在。
   [✅ 已完成] aa290cff2；4 卡齐
10. 更新 `scripts/spec-index.py` GROUPS，运行 `python scripts/spec-index.py` 重生成 INDEX。验证：INDEX 含 4 新卡。
   [✅ 已完成] aa290cff2；INDEX 26 项目含 4 新卡（含 aavm Status 解析修复）
11. 更新 `AGENTS.md`（§1/§4/§5 + 范式小节）与 `scripts/new-plan.sh`（新 frontmatter 骨架；`bash -n` 过）。
   [✅ 已完成] 8bc65a7bd；探针运行验证 v2 frontmatter 后清理
12. 全量验证：`python scripts/spec-lint.py`；残留 grep；`git add -A && git commit`。验证：AC1-AC6 逐条勾选。
   [✅ 已完成] 2291e2eeb；spec-lint 0 error/1 warn（aavm/data 既有）、链接检查 ALL OK、git status 0

## 复审记录

**复审人**：zcode（/auto-plan:review 范式，2026-08-28）　**结论**：✅ 通过（reviewed）

逐条核验（不信勾选框，全部对实际文件重跑）：

| 项 | 证据 |
|---|---|
| AC1 编号/目录 | find 全列：16 族仅 16+16a（附录，有意）；双 19 消解；new/ 不存在；strategy/ 4 篇；reports/374 就位 |
| AC2 索引对账 | find=111；00-intro 分区合计 1+10+5+11+2+4+3+3+5+67=111 ✓（计划期写 112 系 raw 估算 70 vs 实际 67，已修正） |
| AC3 specs v2 | README v2 / overview(128 行) / goals(18 条) / 4 卡齐；INDEX 重生成 26 项目，实验/沙盒组就位 |
| AC4 范式对齐 | AGENTS.md 范式块 + §1/2/4/5 路径更新；new-plan.sh bash -n 过 + 探针生成 v2 frontmatter 验证后清理 |
| AC5 验证 | spec-lint 0 error（1 warn=aavm/data 既有，数据目录非 module）；新建/改写 10 文件相对链接脚本检查 ALL OK；残留 grep 仅命中计划自身与归属注记；worktree 提交干净 |
| AC6 体系文档 | 26 存在（6 段映射+路径映射+取舍记录）；plan-spec-hybrid 横幅 1 处 |

**遗漏/延后/workaround 扫描**：
- 无未批准延后。两处与计划文本的偏差均显式记录（步骤 3 的 website 路由裁定、步骤 5 的数量修正），非静默缩水。
- 有意取舍（Design 26 §6 决议，非本计划遗漏）：① 438 归档计划不回溯入账，账本向前生长；② specs module 树对 437–466 的深度回填不做，现状由 overview.md 承载。
- 范围外遗留（记录为后续候选，不阻塞）：docs/ 根级 26 个散文件的归类（v1 设计 §7 曾规划未行）；仓库根杂物清理（tmp/nul/日志）。

**git 溯源**：git log --follow 抽查 strategy/python-parity-roadmap.md 可溯至迁移前历史 ✓

## 待澄清事项

（无——路径映射与不改名裁定已在需求分析节给出依据，如用户不认可 archive/ 保留裁定可在复审时翻转）

---

## spec-sync 回写记录（2026-08-28，/auto-plan:merge）

- `.autoos/specs.json`：P467-1..P467-7 七条幂等入库（reports/goals/architecture/designs/tests/reviews×2）。
- `docs/specs/goals.md`：新增 GOAL-018（开发范式与知识账本运转）。
- `docs/specs/INDEX.md`：随本计划重生成（26 项目，新增实验/沙盒组）。
- module 树：本计划无单一受影响 module（元任务）；overview.md/goals.md 即全局回写载体。
- 归档：`git mv` → `docs/plans/archive/`，`status: archived`（终态）。
