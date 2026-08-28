---
plan_id: PLAN-468
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: design 文档按模块归位（去序号化）
author: [zcode]
created_at: 2026-08-28
updated_at: 2026-08-28

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design]        # 受影响的 specs 路径
current_step: 0
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

- [ ] AC1: design 根级编号文件仅 00–16 与 20；autoui/ 含 8 篇 + README；blocks/ 含 4 篇；26 去号在根级。
- [ ] AC2: 11 个移动文件均带归位注记横幅（含原 Design NN）。
- [ ] AC3: 00-intro 重写且含封存号说明与归位规则；文件总数对账（111 = 索引 1 + 章 17 + autoui 9 + blocks 4 + 流程 2 + strategy 4 + 诊断 3 + forge 5 + raw 67 → 对账公式写入索引）。
- [ ] AC4: AGENTS.md/specs/{README,overview,goals,INDEX}/spec-index.py 的 26 引用全部指向新路径；lint 0 error。
- [ ] AC5: 437 断链在计划与本说明中双登记（不修不改在途文件）。

## 执行步骤

1. 建 worktree `.worktrees/plan-468-dev`；master 上 commit 计划骨架与 .next-id(469)。
2. `mkdir docs/design/autoui` + 11 项 `git mv`（含去号重命名）+ `autoui/README.md`。
3. 归位注记横幅批量注入（python）。
4. 链接映射表批量修复（排除 437/463/archive-466/Cargo.toml/snap.new 与 archive/467）。
5. 重写 `00-intro.md`。
6. specs 侧同步（goals/overview/README/AGENTS.md/plan-spec-hybrid 横幅/spec-index.py + INDEX 重生成）。
7. 全量验证（lint + 链接脚本 + 残留 grep + 编号复查 + 对账）。
8. 复审回填 → fold 合并 → 账本沉淀（P468 条目）→ 归档终态。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

（无）
