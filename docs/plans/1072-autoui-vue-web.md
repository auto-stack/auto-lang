---
plan_id: PLAN-1072
status: drafting
feature_name: AutoUI Vue 与生产 TS/JS 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-976, PLAN-978]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1072] AutoUI Vue 与生产 TS/JS 接管

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

Vue 生产模式正式收据，含完整真实旧语料迁移、合法 TS/JS 模块闭包、浏览器行为和错误恢复。可提前给 Tauri 交已复核的 Vue 生成接口。

## 2. 架构方案

auto-ui 主责，auto-lang 与 AutoMan 按接口供受检语义、provider 和真 CLI。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。消费 980 DIALECT/NATIVE 实际使用的已交接口、981 EVENT-B 对应项、976/978。010 的列表字段按 1071 所交精确语义接入；场景不使用该字段时可独立推进。U0 联合收据 `2c9c5f228` 按原身份接收。

**接手范围**。接 980 B-WEB、完整生产 TS/JS 的 Vue 子面和 PHASE-LAMBDA-IF。包括 005、010、AutoNotes 跨文件件、quick_switcher/use、组件包和实际工程的旧新映射；已封存内容按原来源复用。

**PHASE-LAMBDA-IF 交付状态注记（2026-11-19 跨会话协调）**：该刀主体已与拆分裁定**同日在 980 名下交付落主**——lang `347540437` + auto-ui `05ed07c3`（T-LIF-1..6 全交、AC-LIF-01..06 闭、规范沉淀 `m2-autoui.md` §13.12；收据=980 §9「lambda-if 刀交付」）。**F-R21-2 亦已闭**（981 r21 指派项，同日 980 名下：ace_package_sfc 脚本重放落 `handler_stmts` 消费门=具名拒 `source.age.sfc-handler-stmt`，auto-ui `7b8456e5`；收据=980 §9「F-R21-2 收口」——981 T-BASE-ONIF 可据此复核）。1072 承接的**残余**：AC-LIF-07 的 1072 侧记账、原 §10-1 owned-wrap 统一决策、分支内调用形→通道调用承载（#6p 谱系）、005 整例 use/style/design 残墙、`n_assigns` 平铺计数的 census 漂移记账——不重做已交主体。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

与 982 联合验一个原样 Vue UI 工程的 `auto gen`（Vue/TS 与运行时输出）、`auto build`（实际构建）、`auto run`（真实浏览器、事件与状态）。1072 交生成/运行时及行为依据，982 验公开 CLI 与工具派发。U0 只作为既有 Run 子面按原身份复用。

原 V-B-WEB、V-EVT-WEB-CONSUME、V-TS-CONSUMER-UI Vue 格、AC/V-LIF 全部及正式语料 census。核实际 watch/setup/expose/ext_imports/style、绑定、更新/卸载、错误保旧、a2x_rt/TS 与真浏览器结果。旧发射或只观测生成文本不计通过。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜冻结语料**。复用 `auto-ui/crates/auto-ui-runtime/tests/ace_ts_provider.rs` 与 980 真实语料 census，定位旧源及独立期望。

- [ ] **T-02｜work｜完成受检生成**。在既有 `ace_ts_provider.rs`、SFC 和 tools 产品入口补完 005、010 及分母内的缺口，复用 980 的生产 SFC 与 `a2x_rt.mjs`。

- [ ] **T-03｜work｜验收 Vue 产品链**。跑原 V-B-WEB 及适用产品用例，从 `auto run --render vue` 到 pnpm/Vite/真实浏览器，对照页面事件、退出与清理。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
