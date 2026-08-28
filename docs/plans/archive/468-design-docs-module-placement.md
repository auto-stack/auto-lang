---
plan_id: PLAN-468
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: design 文档按模块归位（去序号化）
author: [zcode]
created_at: 2026-08-28
updated_at: 2026-08-28

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/design/00-intro.md: 467 版分类学重写（域级章/需求级/流程体系三层归位规则 + 封存号说明）"
new_spec_components:
  - "docs/design/autoui/: 新子目录（README + 8 篇归位文档）"
  - "docs/design/blocks/blocks-first-class.md: Blocks 域主设计归位"
  - "docs/design/autoplan-spec-ledger.md: 流程体系文档去号根级"
touched_goals:
  - "GOAL-018: 开发范式与知识账本运转（design 层归位规则入册）"
             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design]        # 受影响的 specs 路径
current_step: 8
total_steps: 8
---

# [PLAN-468] design 文档按模块归位（去序号化）

## 变更摘要

修正 Plan 467 遗留的结构问题：`docs/design/` 的编号系列（00–16）原本是**分门别类的域级章**，
16 号之后的新增文档（16a、17–26）实为"按需取号"的需求级设计，不应占据章号。
本计划审计后将其归位到模块子目录（`autoui/`、`blocks/`、根级），编号 17–26 **封存不复用**，
仅保留两个真正够格的域级新章：16（App 生成战略）与 20（AutoUI 分离架构）。

## 目标

- G1: 编号系列回归"域级章"语义：00–16 + 20；17–19、21–26 序号封存（历史文献"Design NN"仍可溯源）。
- G2: 每个被移动文档带**归位注记**横幅（保留原 Design NN 对应关系，供 grep 历史提及）。
- G3: 除 1 处登记的已知断链（用户在途文件）外，全仓 markdown 链接零断裂。
- G4: 00-intro.md 重写反映新结构，并写明"新增设计文档的归位规则"（域级章拿号 vs 需求级进子目录）。

## 架构方案

审计结论（哪些是"独立小需求设计"）：

| 原号 | 文档 | 定性 | 归位 |
|---|---|---|---|
| 16 | app-generation-and-ai-authoring | ✅ 域级战略章（Rung 1–5 伞形，AutoUI 域锚点） | **保留章号** |
| 20 | autoui-separation-architecture | ✅ 域级架构章（被 23/24/specs 引用为架构锚点） | **保留章号** |
| 16a | 025-gap-enumeration | 需求级（025 示例差距枚举，历史记录） | `autoui/025-gap-enumeration.md` |
| 17 | blocks-first-class | 需求级（Blocks 层设计；blocks/ 子目录本就为其而设） | `blocks/blocks-first-class.md` |
| 18 | shared-store | 需求级（Rung 4 特性，Plan 351 已交付） | `autoui/shared-store.md` |
| 19 | theming-and-dark-mode | 需求级（主题特性，Plan 458 落地线） | `autoui/theming-and-dark-mode.md` |
| 21 | examples-app-track | 需求级（examples/ui 轨道纲领） | `autoui/examples-app-track.md` |
| 22 | base-styles-and-visual-parity | 需求级（视觉一致性规范） | `autoui/base-styles-and-visual-parity.md` |
| 23 | autoui-virtual-desktop | 需求级（虚拟桌面架构方案，修订 20 的专题设计） | `autoui/virtual-desktop.md` |
| 24 | autoui-desktop-shell-and-launcher | 需求级（桌面 Shell M2–M4 落地设计） | `autoui/desktop-shell-and-launcher.md` |
| 25 | a2ui-composer-analysis | 需求级（外部技术研究输入） | `autoui/a2ui-composer-analysis.md` |
| 26 | autoplan-spec-ledger | 流程体系类（非设计章；与 plan-spec-hybrid-model 同类） | 根级 `autoplan-spec-ledger.md`（去号） |

目标结构：

```
docs/design/
├── 00-intro.md                  # 重写（新分类学 + 归位规则）
├── 01..15-*.md                  # 原始域级章（不动）
├── 16-app-generation….md        # 域级章（AutoUI/AppGen 战略）
├── 20-autoui-separation….md     # 域级章（AutoUI 架构）
├── autoui/                      # ★ AutoUI 域需求级设计（8 篇 + README）
├── blocks/                      # Blocks 域（4 篇自洽）
├── strategy/ · forge/ · raw/    # 不动
├── autoplan-spec-ledger.md      # 流程体系（去号）
├── plan-spec-hybrid-model.md    # 流程体系（v1，已取代）
└── dialect-extension-diagnosis.md · vm-debugging.md · ash-design-summary.md
```

## 需求分析与背景调查

（引用面扫描：2026-08-28，grep 全仓）

- 引用需修复的文件（非用户在途）：00-intro、blocks/README、examples/{ui,blocks-gallery,capability-tests}/README、
  .agents/skills/autoui-verifier/SKILL.md、docs/guides/autoui-verification-and-mcp-guide.md、
  活跃 plans 439/440/441/458/462/autos-desktop-program、归档 plans 234/345/342/343/351/354/357/365/438/445/452/453/459、
  specs goals/overview/README/INDEX（经 spec-index.py）、AGENTS.md、design 内部互链（20↔23、21↔18 等）。
- **已知断链登记（不可修复）**：`docs/plans/437-024-charts.md:4` 的 `[Design 21 §5](../design/21-examples-app-track.md)`
  ——该文件属用户未提交在途修改（437/463/archive-466/Cargo.toml/3×snap.new），**一律不动**；
  463 对 24 仅为纯文字提及（非链接，不因移动而断）。
  待用户提交后由后续计划顺手修复（一行）。
- 跨仓引用复核：auto-down→raw/、auto-musk→plans/，均不涉及本次移动文件。

## 详细设计

1. **移动**（git mv + 去号重命名）：上表 11 项；新建 `autoui/README.md`（8 篇一览 + 指向 16/20 域级章）。
2. **归位注记横幅**：每个移动文件 H1 下方加一行（原号 + Plan 468 + 新位置），保"Design NN"可溯源。
3. **链接修复**：python 映射表批量替换三种形态——`(NN-file.md)`（design 同域互链）、`design/NN-file.md`、
   `../design/NN-file.md`；修复集排除用户在途 5 文件与历史归档 467。
4. **00-intro 重写**：新分类学（域级章 00–16+20 / autoui/ / blocks/ / 流程体系根级 / strategy/ / 专题诊断 /
   forge/ / raw/）；**封存号说明**（17–19、21–26 已用并封存，下一章号 27）；**新增文档归位规则**。
5. **specs 侧同步**：goals.md/overview.md/README.md 中指向 21/24/25/26 的链接更新；spec-index.py 头部链接
   26 → autoplan-spec-ledger.md 后重生成 INDEX；AGENTS.md 中 26 链接更新。
6. plan-spec-hybrid-model.md 的 superseded 横幅同步指向新路径。

## 测试设计

- `python scripts/spec-lint.py` 无 ERROR；
- 自写链接检查脚本扫全部被修文件 + 移动后文件：相对链接目标存在；
- 残留 grep：旧文件名路径形引用仅剩 437（登记）与 467 归档（历史）；
- 编号复查：`ls docs/design/[0-9]*.md` 仅 00–16、20。

## 验收标准

- [x] AC1: design 根级编号文件仅 00–16 与 20；autoui/ 含 8 篇 + README；blocks/ 含 4 篇；26 去号在根级。
- [x] AC2: 11 个移动文件均带归位注记横幅（含原 Design NN）。
- [x] AC3: 00-intro 重写且含封存号说明与归位规则；文件总数对账（111 = 索引 1 + 章 17 + autoui 9 + blocks 4 + 流程 2 + strategy 4 + 诊断 3 + forge 5 + raw 67 → 对账公式写入索引）。
- [x] AC4: AGENTS.md/specs/{README,overview,goals,INDEX}/spec-index.py 的 26 引用全部指向新路径；lint 0 error。
- [x] AC5: 437 断链在计划与本说明中双登记（不修不改在途文件）。

## 执行步骤

1. 建 worktree `.worktrees/plan-468-dev`；master 上 commit 计划骨架与 .next-id(469)。
   [✅ 已完成] master 74d7ee1d4 立项 + 469 取号 + .worktrees/plan-468-dev
2. `mkdir docs/design/autoui` + 11 项 `git mv`（含去号重命名）+ `autoui/README.md`。
   [✅ 已完成] 68df23ea3；实际 10 项移动 + autoui/README.md；**25-autoshell 排除**（见复审记录 R1）
3. 归位注记横幅批量注入（python）。
   [✅ 已完成] 68df23ea3；10 个移动文件横幅齐（含原 Design NN）
4. 链接映射表批量修复（排除 437/463/archive-466/Cargo.toml/snap.new 与 archive/467）。
   [✅ 已完成] 68df23ea3；48 文件映射表修复，排除 437/466/Cargo.toml/snap.new 与 archive/467
5. 重写 `00-intro.md`。
   [✅ 已完成] 68df23ea3；新分类学 + 封存号（17-19、21-26）+ 归位规则三行表 + 对账公式 113
6. specs 侧同步（goals/overview/README/AGENTS.md/plan-spec-hybrid 横幅/spec-index.py + INDEX 重生成）。
   [✅ 已完成] 68df23ea3；spec-index.py 头部已换 autoplan-spec-ledger.md 并重生成 INDEX；AGENTS/specs 四件同步
7. 全量验证（lint + 链接脚本 + 残留 grep + 编号复查 + 对账）。
   [✅ 已完成] lint 0 error；全仓 1073 md 链接检查——新增断链 0（BROKEN 列表全为 docs/ 根既有债务，467 已登记）；残留路径形引用 0（登记项除外）；编号复查 00-16+20+25在途；对账 113
8. 复审回填 → fold 合并 → 账本沉淀（P468 条目）→ 归档终态。
   [✅ 已完成] 本节 + 合并/归档执行见后

## 复审记录

**复审人**：zcode（/auto-plan:review 范式，2026-08-28）　**结论**：✅ 通过（reviewed，含 1 项显式偏差裁定）

| 项 | 证据 |
|---|---|
| AC1 编号 | `ls docs/design/[0-9]*`：00–16、20 + 25-autoshell（在途登记）；autoui/ 8+README；blocks/ 4；autoplan-spec-ledger.md 根级 |
| AC2 横幅 | 10 个移动文件 H1 下均含"归位注记（2026-08-28，Plan 468）+ 原 Design NN" |
| AC3 索引 | 00-intro 重写：归位规则表、封存号说明、对账公式 113 = find 实数 |
| AC4 同步 | AGENTS.md/specs{README,overview,goals,INDEX}/spec-index.py 全部指向新路径；lint 0 error |
| AC5 断链登记 | 437:4 链接保持原样（在途文件未动），计划 §需求分析 + 本节双登记 |

**偏差与裁定**：
- **R1（计划外发现）**：执行中发现 `docs/design/25-autoshell-dsl-unified-shell.md`——立项后被并行会话提交（8120afe49 14:28 + aed3f0042 14:35 连续修订），且与旧 25-a2ui-composer 形成**双 25 撞号**（master 上真实存在过）。裁定：**排除出本次移动**（活跃在途文档，避免与并行会话撞车），在 autoui/README 与 00-intro 登记其稳定后归位路径；本次 a2ui-composer 的归位已顺带消解撞号。定性上它属需求级（细化 Design 23/24 的 shell 层工程分解），归位只是时间问题。
- 463 于执行期间被并行会话提交（不再在途），其反引号路径提及按标准合并语义随本计划更新；437 仍在途未动。
- 顺手修复：blocks/README 两处指向已归档 plan 的既有断链（本就在修复集内）。

**遗漏/延后/workaround 扫描**：无未批准延后；docs/ 根级既有断链（README/docs 散文件）属 467 已登记的范围外债务，未扩散。

**并发风险复核**：master 在本计划执行期间前进 5+ 提交（446 批五/design25），与本计划改动面（design 归位 + 链接）零文件重叠于代码；463 为纯文本行更新，合并可自动裁决。

## 待澄清事项

（无）

---

## spec-sync 回写记录（2026-08-28，/auto-plan:merge）

- `.autoos/specs.json`：P468-1..2 入库（reports/designs，幂等）。
- `docs/design/autoui/README.md`：新子目录索引（含 25-autoshell 在途登记）。
- 归档：`git mv` → `docs/plans/archive/`，`status: archived`（终态）。
