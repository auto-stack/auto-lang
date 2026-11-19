---
plan_id: PLAN-1100
status: drafting
feature_name: Platformer 2D 完整项目 AV 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-1075, PLAN-980, PLAN-981, PLAN-978, ag-examples/PLAN-024, ag-examples/PLAN-025]
supersedes_spec_components: [docs/specs/ace-migration/m2-age.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, age]
---

# [PLAN-1100] Platformer 2D 完整项目 AV 接管

## 0. 变更摘要

2026-11-19。用户要求将 1075、1097、1074 的交付按实际依赖拆分。继承原输入、行为预期、负控和资源约束。

## 1. 目标

同源运行完整 platformer-2d，交 G2 的 AV 行为、编辑和冷包证据及适用 IF-AGE-GFX-AV/<cap>。

## 2. 架构方案

从 age-examples/games/platformer-2d/ 固定原项目、关卡、资源锁、许可及独立 verifier。消费公共 AV 会话和 2D 服务，生产源、渲染和输入走原路径。

## 3. 技术栈

沿 981 原 Windows 目标、978 SDK、Schema、源和资源版本。实施用 D:/autostack/.wt/lang-1100/auto-lang、分支 plan-1100-dev。按写面开同组 auto-ui 或 age-examples 树。材料归 D:/autostack/plan-artifacts/plan-1100/。

## 4. 需求分析与背景调查

授权范围为规划拆分。来源为 [981 归档合同](archive/981-ace-m2-age.md) §5–8、[AGE 当前规范](../specs/ace-migration/m2-age.md) 及 [AGE 交付顺序](../design/strategy/age-m2-three-route-strategy.md)。规划基面 104074a5b。

## 5. 详细设计

**交付范围**。981 B-CONTENT 的 G2 行和 B-SESSION/LANGUAGE/PROOF/FUSION 的适用 AV 格。保 G2-01..09、V-B-CONTENT、V-B-SESSION、V-AGE-WAVE/TRANSFER 的 2D 行。

**实施入口**。确认1075 基础 AV、各项实际 2D 资源/服务和 978 SDK，在相应用例开工前钉正式版本、源码/资源哈希、SDK/ABI、原入口和运行参数。排在小 demo 和混合样例之后。同源 G2 AV 供 1104、1076 和最终认证。

### 规范增量

| delta_id | 操作 | 目标 | 变化与验收 |
|---|---|---|---|
| SD-01 | modify | docs/specs/ace-migration/m2-age.md | 按实际入口、目标、原 oracle 和正式身份登记本交付的支持范围。AC-01/02/03。 |

## 6. 测试设计

复用 age-examples/games/platformer-2d/tools/verify_game2d.py 和原 G2 行为检查。验输入、碰撞、胜败重开、音画、保存/SaveAs/搬迁、坏编辑保旧、关闭回收及仓外 AV 冷包。缺资源、漏原玩法行或用首帧替代运行使验收失败。 语言仓有对应代码改动时运行受影响固定组，例如 cargo t ace-source ace-av，图形验收走上述原入口。

复用同源独立预期，记录源码/资源、Schema、SDK、宿主、平台、实际命令及子终态。已有证据按同一身份和支持范围复用。Windows 的 Linux/WSL 腿按项目裁定登记。

## 7. 验收标准

| ID | owner_stage | 完成条件 |
|---|---|---|
| AC-01 | work | §5 原分母逐条映射到正式输入、实现入口和原独立预期，缺失依赖可定位到生产方及所需交付。 |
| AC-02 | work | §6 的实际后端、受影响原入口、适用图形行为及负控全部通过。故障后可按原合同恢复，资源完整且可回收。 |
| AC-03 | review | 独立复核冻结版本、原义务、真实行为轨迹和至少一个区分性反例。 |
| AC-04 | merge | 交付合入，发布具名收据，记录 Spec 支持范围及下游消费版本。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结输入和验收行（AC-01）**。核 §5 依赖、§6 命名入口和原分母，记录实际消费版本与运行方式。
- [ ] **T-02｜work｜接通正式入口（AC-02）**。在 §2 所列项目复用现有宿主、资源与执行链，必要的共享改动归实际 provider，保独立预期和错误语义。
- [ ] **T-03｜work｜跑实际行为与负控（AC-02）**。执行 §6 原入口和适用固定组，冻结目标、产物、原行为、失败与恢复证据。
- [ ] **T-04｜review｜独立复审（AC-03）**。逐行核原分母、身份、运行路径与负控，发现问题复验后再签。
- [ ] **T-05｜merge｜正式交付（AC-04）**。合入、沉淀、发布具名收据，归档及清理。

## 9. 复审记录

2026-11-19 stage: new。拆分规划；实现、运行与独立复审待执行。

## 10. 待澄清事项

T-01 核实当前工具参数、适用端口和正式版本。
