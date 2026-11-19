---
plan_id: PLAN-1073
status: drafting
feature_name: AutoUI Tauri 工程接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-980, PLAN-978, PLAN-1072]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1073] AutoUI Tauri 工程接管

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

真实 Tauri 工程的 ACE 前端与 Rust 后端生成、构建、启动、权限、失败和恢复收据。

## 2. 架构方案

auto-ui tools 主责，按实际桥接需求消费语言与 AutoMan 接口。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

AutoMan v0.6 的通用 UI CLI 验收聚焦 Vue Web 与 A2R/Iced。本计划沿 Tauri 自有实际工程入口验收，按实际使用的已有接口接 AutoMan；Tauri 真实产品收据单独交 985。

**前置**。消费 1072 的 Vue 生成接口、978 构建入口及真实后端使用的 Rust 接口。若该工程确实消耗 1074 的新增接口，记录具体接口及交付版本，独立的 Tauri 面可先施工。

**接手范围**。接 980 B-TAURI 与完整生产 TS/JS 的 Tauri 子面，保持前后端 invoke/事件/资源和权限，原生 Rust 后端生成闭包随原工程实际验证。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

沿原 V-B-TAURI、V-TS-CONSUMER-UI 的 Tauri 格，跑真实产物生成、编译、窗口启动、invoke/事件、拒权、坏源/坏工程、关闭重开与旧 emitter 零回退。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结工程**。从原 980 Tauri 工程、`auto-ui/crates/auto-ui-tools/src/tauri.rs` 和 `tauri_backend.rs` 核对调用路径与工具链。

- [ ] **T-02｜work｜完成生成与桥接**。接 Vue 生成接口和真实 Rust 后端，复用已有权限、诊断和工程生成事务。

- [ ] **T-03｜work｜实跑产品窗口**。执行原 V-B-TAURI 工程级命令，收集真实 invoke、事件、失败保旧、退出重开和闭包证据。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
