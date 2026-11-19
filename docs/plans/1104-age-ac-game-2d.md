---
plan_id: PLAN-1104
status: drafting
feature_name: Platformer 2D 完整项目原生 AC
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-1097, PLAN-1100, PLAN-981, PLAN-978, PLAN-1078, auto-ui/PLAN-072, ag-examples/PLAN-028]
supersedes_spec_components: [docs/specs/ace-migration/m2-age.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, age]
---

# [PLAN-1104] Platformer 2D 完整项目原生 AC

## 0. 变更摘要

2026-11-19。用户要求将 1075、1097、1074 的交付按实际依赖拆分。继承原输入、行为预期、负控和资源约束。

## 1. 目标

Windows 原生 AC 跑同源 G2 完整游戏，交 G2 的 IF-AGE-GFX-AC/<cap> 和仓外原生证据。

## 2. 架构方案

复用 1097 的 typed AC 宿主及 1100 的 2D 游戏、资源和 AV 行为。实际用到 1102 的公共端口时按具名接口消费。共用语言切片接 1078 同源 Case。

## 3. 技术栈

沿 981 原 Windows 目标、978 SDK、Schema、源和资源版本。实施用 D:/autostack/.wt/lang-1104/auto-lang、分支 plan-1104-dev。按写面开同组 auto-ui 或 age-examples 树。材料归 D:/autostack/plan-artifacts/plan-1104/。

## 4. 需求分析与背景调查

授权范围为规划拆分。来源为 [981 归档合同](archive/981-ace-m2-age.md) §5–8、[AGE 当前规范](../specs/ace-migration/m2-age.md) 及 [AGE 交付顺序](../design/strategy/age-m2-three-route-strategy.md)。规划基面 104074a5b。

## 5. 详细设计

**交付范围**。接 1097 原 T-05/T-06 的 G2 全部 AC 行，以及 981 LANGUAGE/FUSION/PROOF/AUDIT 的 G2 原生格。

**实施入口**。确认1097 的 W2/AC 宿主、1100 正式 G2 AV、实际 2D 公共端口及 1078 所用协议，在相应用例开工前钉正式版本、源码/资源哈希、SDK/ABI、原入口和运行参数。交 1076、AG031 和 1068/985/B；用到公共能力时只消费对应正式交付。

### 规范增量

| delta_id | 操作 | 目标 | 变化与验收 |
|---|---|---|---|
| SD-01 | modify | docs/specs/ace-migration/m2-age.md | 按实际入口、目标、原 oracle 和正式身份登记本交付的支持范围。AC-01/02/03。 |

## 6. 测试设计

以 age-examples/games/platformer-2d/tools/verify_game2d.py 的独立 G2-01..09 oracle 和原 V-AC-DESKTOP-PRODUCT/V-AGE-CAP 核机器行为、GPU/输入、资源、错 ABI、关闭及仓外 AC 冷包。禁用机器 dispatch、换成 AV 或漏资源必红。 语言仓有对应代码改动时运行受影响固定组，例如 cargo t ace-native ace-abi，图形验收走上述原入口。

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
