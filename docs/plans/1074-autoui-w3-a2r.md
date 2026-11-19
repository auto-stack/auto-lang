---
plan_id: PLAN-1074
status: drafting
feature_name: AutoUI 与 AGE 公共 A2R 生成和宿主链
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-978, PLAN-1071, PLAN-1068]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1074] AutoUI 与 AGE 公共 A2R 生成和宿主链

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

`UI-W3-A2R` 及 `IF-AGE-GFX-A2R/<cap>` 对应正式源，另可先交 982 所需 Rust 工程 `run` 的独立已验接口。

## 2. 架构方案

auto-ui 与 auto-lang A2R owner 共用一套生成与 typed AGE 宿主，UI 验收由 auto-ui 主责。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要) · [981 原合同](981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。新增图形端到端必须消费 1068/T-04 正式 `UI-W2-AC`，以及 1071 同源 UI/Widgets、已交 AGE 公共语义和实际图形能力所需 AV/AC 收据。早期非图形工程路径按其真实接口先行。1068 T-06 最终汇总和两游戏发行属于下游。

**接手范围**。接 980 B-RUST、981 B-RUST 的公共生成发行链及 LANGUAGE/PROOF/FUSION 适用 A2R 格。全 UI/Widgets、传统 UI 与普通 Node 的真实 A2R；共同消费端口只实现一次。两游戏最终 ZIP 归 1076/1077。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

与 982 联合验一个原样 Iced UI 工程的 `auto gen`（A2R Rust 产物）、`auto build`（干净目录编译链接）、`auto run`（真实 Iced 窗口、事件及关闭）。1074 交生成、宿主及行为依据，982 验公开 CLI 与工具派发。

原 V-UI-W3、V-UI-AXR0、V-UI-SOURCE/FULL、两计划的 V-B-RUST、V-R-FAIL 公共部分及 AGE 对应 W3 格。冷生成→真 rustc→运行；同独立事件/资源/GPU oracle；坏源保旧、错宿主身份拒绝、真实 dispatch、无 VM/AV/旧 emitter 回退。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜核公共源与发行链**。核 980 AXR0 与 981 RUST 的多模块源、原 API、source/SDK/ABI、978 构建及 AGE typed 宿主接口；在 `auto-lang/experimental/ace/crates/a2x-rust` 与 `auto-ui/experimental/autographics-auto-driver/tests/standalone_game_player.rs` 核正式 W2 来源与现有运行入口。

- [ ] **T-02｜work｜完成 A2R 组件与宿主**。统一生成/编译/调度、侧车、props/msg/route、direct/RQ 及资源；在已有 provider 写面实现并向游戏发行交可消费接口。

- [ ] **T-03｜work｜验收 W3 与工程运行**。沿原 V-UI-W3 和 V-B-RUST 真构建、真运行逐条对证，分别记录 W3 与 982 工程运行的正式收据。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
