---
plan_id: PLAN-1075
status: drafting
feature_name: AGE Gallery 和公共场景会话的 AV 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 2
current_step: 0
total_steps: 5
depends_on: [PLAN-981, PLAN-980, PLAN-978, ag-examples/PLAN-024, ag-examples/PLAN-025]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1075] AGE Gallery 和公共场景会话的 AV 接管

## 0. 变更摘要

2026-11-19。本计划独立交 Gallery 小 demo 与公共 AV 入口。原混合场景迁 [1099](1099-age-av-mixed-showcase.md)，完整 G2/G3 迁 [1100](1100-age-av-game-2d.md)、[1101](1101-age-av-game-3d.md)，保原验收义务。

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

使 age-examples/examples/ 的 G01–G12 通过 ACE AV 原入口运行，交 Gallery 及共享会话、事件和资源的正式证据。

## 2. 架构方案

AGE 场景及 auto-ui 宿主 owner，与 age-examples 产品 owner 配对。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [981 原合同](archive/981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。取 981 已交 SOURCE/EVENT-B、980 typed 宿主与 978 SDK。从样例实际使用的组件和服务取正式来源。先跑依赖少的图形、层级和查询，再推进角色、动画、相机和音频等入口。

**交付边界**。承接 981 B-CONTENT 的 Gallery 行、B-SESSION/LANGUAGE/PROOF/FUSION 公共 AV 格与原 AGE-L/AG-A 的适用行。冻结 G01–G12、22 入口及原适用变体、资源与 oracle。公共场景、会话和资源接线由本计划交付。MAP/ASSET/PBR/CLIP 的 AV 实现与自身 oracle 由 AutoUI 099–102 交付。

**产物**。交 Gallery 基线及适用的 IF-AGE-GFX-AV/<cap>，附源码/资源、SDK、Schema、独立预期、错误和恢复。混合样例及完整游戏在对应迁入计划验收。

## 6. 测试设计

从 age-examples/tools/gallery_node_authoring.py 和原 examples/ 固定入口、变体及独立 oracle。沿 auto-ui/experimental/autographics-auto-driver/tests/authoring_session.rs 核实际 AV 会话、真 GPU/输入/音频（适用时）、热更新、坏编辑保旧、保存/SaveAs/搬迁、仓外冷包和资源回收。保 V-B-CONTENT/SESSION、V-L-*、V-AGE-WAVE/TRANSFER 与 V-B-FUSION 的 Gallery/公共接面适用行。记录源/SDK、实际命令和失败条件。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结 Gallery 分母**。清点 G01–G12、22 入口、原变体、资源和实际依赖。
- [ ] **T-02｜work｜交公共 AV 会话接线**。沿 authoring_session 与原入口验证 typed 事件/资源、生命周期及坏编辑恢复。
- [ ] **T-03｜work｜验收原 Gallery**。跑各项原入口和独立预期，交适用图形能力的正式结果与源/资源指纹。
- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原入口及实际反例。
- [ ] **T-05｜merge｜正式交付**。合入、发布 Gallery/公共 AV 交付、归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
