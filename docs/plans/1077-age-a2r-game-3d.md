---
plan_id: PLAN-1077
status: drafting
feature_name: AGE 3D 游戏的 A2R 独立发行
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-981, PLAN-1074, PLAN-1075, ag-examples/PLAN-028, auto-ui/PLAN-072]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1077] AGE 3D 游戏的 A2R 独立发行

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

原 3D 游戏 Windows x64 A2R exe、GLB/纹理与其它资源 ZIP、许可、来源、SHA256、启动说明及完整产品验收收据。

## 2. 架构方案

age-examples 3D 产品与发行 owner，消费 auto-ui 与公共 A2R 交付。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [981 原合同](981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。消费 1075 同源 3D AV、1068/T-04 对应 AC 域证据以及 1074 UI-W3/实际 A2R 宿主接口。具名引用 981 原 AG028 工具与现有 3D 源资产。按 3D 自身收据开工。

**接手范围**。接 981 B-RUST3D 的全部 G3-01..08、相机/音画输入、失败与独立发行义务。与 2D 各自出件，公共生成链由 1074 持有。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

原 V-B-RUST3D、V-R3-RULE/MEDIA、V-R-PACK/FAIL 的 3D 格。真 A2R 生成/编译、仓外无开发工具链冷启动、鼠标/键盘/音频、相机、GLB 纹理闭包、恢复与无解释器分派。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结 3D 原源与 oracle**。从 981 RUST3D 合同核 G3-01..08、原 AG028 packager `age-examples/tools/package_game_release.py`、`age-examples/games/platformer-3d/tools/verify_game3d.py`、GLB/纹理和其它资源边界。

- [ ] **T-02｜work｜接真实 release 打包**。复用原 packager，钉公共 A2R 工具与宿主身份，生成 exe、完整资源 ZIP 和发布清单。

- [ ] **T-03｜work｜验仓外发行包**。执行原 V-R3-* 和 3D V-R-PACK/FAIL，验离线试玩、退出和与同源 AV 的独立语义对照。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
