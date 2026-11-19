---
plan_id: PLAN-1071
status: drafting
feature_name: AutoUI 示例与 Widgets 的 AV 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-979]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1071] AutoUI 示例与 Widgets 的 AV 接管

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

`UI-W1-AV`，绑定完整 U01–U06 与 Widgets、同版源/Schema/宿主 ABI、AV 窗口、原正负对照和资源终态。可按组件组先交正式子面。

## 2. 架构方案

auto-ui 主责，auto-lang 维护共享语义，auto-os 提供 Widgets 真实源与宿主证据。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要) · [981 原合同](981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。M2A-08 已正式交付。消费 980/981 已交基础和对应组真实 EVENT-B、typed 宿主及 Widgets 输入。向 982 先交 VM 工程 `run` 所需入口与真实收据。独立组不等待全组过线。

**接手范围**。接 980 B-AV 全批，981 SESSION/LANGUAGE/PROOF/FUSION/CONTENT 中传统 UI、Widgets 与 UI 基线混合见证所需 AV 格。牵头确定 010 列表字段的共享组件 state/仅视图数据边界，给 1072 交所需共享接口。游戏与扩展场景的完整 AV 由 1075 接。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

接 980 V-UI-W1、V-UI-SOURCE/FULL、V-EVT-AV-CONSUME、V-NODE-DESKTOP 与 981 V-B-SESSION/V-B-LANGUAGE/V-AGE-WAVE/V-EVT 中对应 AV 格。按原分组实跑真实 Iced 窗及 001/002、输入/IME/捕获、主题、媒体/terminal、坏编辑保旧、双实例、资源归零；统一记录错误、关闭/恢复和无旧解释回退。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结输入**。对照 980 §9 的 U01–U06 和 981 分组交接，建立原源、Widgets canonical 路径、GUI oracle 与旧检测对应表。

- [ ] **T-02｜work｜接 AV 组件与状态**。复用 `auto-ui/crates/auto-ui-runtime/src/ui/ace_node_desktop.rs` 与现有组件/Store/事件入口；共享列表语义按现有 AGE/collections owner 单写。

- [ ] **T-03｜work｜实跑完整 AV**。沿原 V-UI-W1 真窗、`ace_node_desktop` 和 `authoring_session` 等生产测试入口逐组关闭，提供可独立验收的 VM 工程运行收据。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
