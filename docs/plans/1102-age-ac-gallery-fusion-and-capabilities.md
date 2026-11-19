---
plan_id: PLAN-1102
status: drafting
feature_name: AGE Gallery、混合场景与扩展能力 AC
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-1097, PLAN-1075, PLAN-1099, PLAN-981, PLAN-978, PLAN-1078]
supersedes_spec_components: [docs/specs/ace-migration/m2-age.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, age]
---

# [PLAN-1102] AGE Gallery、混合场景与扩展能力 AC

## 0. 变更摘要

2026-11-19。用户要求将 1075、1097、1074 的交付按实际依赖拆分。继承原输入、行为预期、负控和资源约束。

## 1. 目标

以真实 AC guest 接同一 typed 宿主，交 Gallery、混合场景、MAP/ASSET/PBR/CLIP 对应的 IF-AGE-GFX-AC/<cap>。

## 2. 架构方案

消费已交 UI-W2-AC、各组同源 AV 及 typed 服务/资源。取 MAP ← auto-ui/PLAN-099、ASSET ← PLAN-100、PBR ← PLAN-101、CLIP ← PLAN-102 的各自 AV 正式来源，独立按 cap 落结果。适用公共语言用 1078 同源 Case 加 AC 实际动作和独立检查。

## 3. 技术栈

沿 981 原 Windows 目标、978 SDK、Schema、源和资源版本。实施用 D:/autostack/.wt/lang-1102/auto-lang、分支 plan-1102-dev。按写面开同组 auto-ui 或 age-examples 树。材料归 D:/autostack/plan-artifacts/plan-1102/。

## 4. 需求分析与背景调查

授权范围为规划拆分。来源为 [981 归档合同](archive/981-ace-m2-age.md) §5–8、[AGE 当前规范](../specs/ace-migration/m2-age.md) 及 [AGE 交付顺序](../design/strategy/age-m2-three-route-strategy.md)。规划基面 104074a5b。

## 5. 详细设计

**交付范围**。接 1097 原 T-05/T-06 的公共 AGE 分母、981 LANGUAGE/FUSION/PROOF/AUDIT 的公共 AC 格及 V-AC-DESKTOP-PRODUCT/V-AGE-CAP 的适用行。两个完整游戏的 AC 归 1104、1105。

与 AG029 国风和 AG030 光影等原产品共享的宿主能力，按其真实用例和原支持格交对应能力结果。产品侧行为和最终验收仍由各产品及 AG031 按原合同负责。共享语料只接本计划实际用到的 Case，按后端逐 check 恢复原验收。

**实施入口**。确认1097 的 W2、1075/1099 的同源 AV、各扩展实际 AV 交付与 1078 所用协议，在相应用例开工前钉正式版本、源码/资源哈希、SDK/ABI、原入口和运行参数。逐 cap 可独立复审、合入、交 1103 或直接消费者。W2 归 1097，G2/G3 独立出件。

### 规范增量

| delta_id | 操作 | 目标 | 变化与验收 |
|---|---|---|---|
| SD-01 | modify | docs/specs/ace-migration/m2-age.md | 按实际入口、目标、原 oracle 和正式身份登记本交付的支持范围。AC-01/02/03。 |

## 6. 测试设计

运行 Gallery 与原 Showcase 图形入口，四项扩展取各自原 AV/GPU oracle 的 AC 对应格。核机器 handler、实际窗口和 GPU、资源卸载、错 ABI 拒绝、恢复及仓外冷包。切断机器 dispatch 或换成 AV fallback 必须失败。 语言仓有对应代码改动时运行受影响固定组，例如 cargo t ace-native ace-abi，图形验收走上述原入口。

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
