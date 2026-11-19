---
plan_id: PLAN-1075
status: drafting
feature_name: AGE 内容与融合场景的 AV 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-981, PLAN-980, PLAN-978, ag-examples/PLAN-024, ag-examples/PLAN-025]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1075] AGE 内容与融合场景的 AV 接管

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

两游戏、Gallery、混合场景和实际 profile 的同源 AV 证据及适用 `IF-AGE-GFX-AV/<cap>`；可按游戏与 capability 独立交接。

## 2. 架构方案

AGE 场景及 auto-ui 宿主 owner，与 age-examples 产品 owner 配对。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [981 原合同](981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。消费 981 已交 source、包身份、事件/会话事实以及 980 已交 NATIVE/typed 宿主。仅对实际使用 1071 新组件的场景接其正式子接口与 UI oracle。W1 的 UI 基线签发不等待完整游戏或新增 capability；自身可独立交已有接口覆盖的 AV 格。

**接手范围**。接 981 B-CONTENT 在 UI 基线之外的全部 AV/编辑/冷包，及 B-SESSION/LANGUAGE/PROOF/FUSION 的 AGE 产品与扩展场景 AV 格。原 AGE-L、AG-A、两游戏/Gallery/MIX 适用分母和新增 cap 收据保留。实体验收使用已有 renderer/会话，不另建引擎。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

原 V-B-CONTENT、V-B-SESSION、V-B-LANGUAGE、V-L-*、V-AGE-WAVE/TRANSFER 与 V-B-FUSION 的产品 AV 格。真 GPU/音频/输入、同源结果、双实例/帧原子、同 PID 更新、坏编辑保旧、保存/SaveAs/搬迁、离线 AV 包、错误恢复、退出及资源回收。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结 AV 产品分母**。从 `age-examples/showcases/ui-2d-3d/tools/verify.py`、`age-examples/tools/gallery_node_authoring.py` 与 981 §5/§6 原游戏、Gallery 和混合场景表定位 canonical 源、资源和独立预期。

- [ ] **T-02｜work｜接 AGE 场景与会话**。沿 `auto-ui/experimental/autographics-auto-driver/tests/authoring_session.rs` 及原产品入口覆盖编辑、融合、音画输入及热更新。

- [ ] **T-03｜work｜验收游戏、Gallery 与混合场景**。分别冻结 2D/3D/跨场景及实际 cap 的 AV 收据，附仓外 AV 冷包和异常恢复，不代 A2R 发行。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
