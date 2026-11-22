---
plan_id: PLAN-1074
status: drafting
feature_name: AutoUI W3 与公共 A2R 工程宿主链
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 5
current_step: 0
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-978, PLAN-1071, PLAN-1097, PLAN-1078]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1074] AutoUI W3 与公共 A2R 工程宿主链

## 0. 变更摘要

2026-11-19。本计划独立签发 UI-W3-A2R 与 Iced 工程生成/宿主交付。AGE 公共图形 A2R 格迁 [1103](1103-age-a2r-gallery-fusion-and-capabilities.md)。原 Iced 三命令联合验收由 982 接收本计划交付后在生产 CLI 完成。

### 2026-11-19 r4 · 接收AV-ONLY阶段与非AV检查恢复

用户已明确1078–1096只验AV，公共语料中AC/A2X等检查先以检查级#ignore暂缓。其未实现/未运行/失败不阻AV计划，不充PASS；本包进入对应后端开发/验证阶段后，逐check接手、移除本阶段标记并真实执行。共享源码、原oracle/条件/owner和本包完整验收门保持；不得要求AV计划先交非AVPASS。本轮仅修订规划。

### 2026-11-19 r3 · 公共语言语料按特性共源（仅规划）

按用户要求接[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)与1078的`IF-ACE-FEATURE-CORPUS/1`。A2R公共语言测试复用language按特性Case，在同Case加生成/真实rustc构建/运行检查；Rust emit金样单独保留。UI/AGE完整原源、GUI/GPU/资源oracle仍读auto-ui/age-examples等canonical来源并保本包原门，不把demo另抄进language或用语言例代W3。 原任务ID/数量/进度与已有证据边界保持，本轮不施工、不迁实际测试。

2026-11-19。图形 AC 的 W2 与 AGE 域证据取 [1097](1097-ace-ac-autoui-w2-and-age-host.md)，正式三端验收仍由 1068 消费本计划的 W3。

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

交 UI-W3-A2R 和可由 982 与 AGE 消费的同版 A2R 工程生成、构建与运行宿主，记录独立 Iced 工程证据。

## 2. 架构方案

auto-ui 与 auto-lang A2R owner 共用生成和工程宿主接口，UI 验收由 auto-ui 主责。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](archive/980-ace-m2-autoui.md#0-变更摘要) · [981 原合同](archive/981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

### AV-ONLY接收与本目标启用

1078的IF-ACE-FEATURE-CORPUS/1和1096的AV语言证书允许携带非AV#ignore/NOT_RUN行，接收时只核AV声明和迁移义务未丢，不以非AVpending回卡AV独立出口。本包实际验证自己的后端/代际时，由原生产任务逐check去掉AV阶段#ignore，保原平台/ignore与oracle，按当前同源/工具/SDK身份运行；失败属于本目标任务，必须修到本目标required齐才签其完整出口。1068/985的最终多端门及AAC独立门不接受忽略充PASS；AV阶段通过从不冒多端通过。

规范影响：公共作者/引用/选择/迁移协议由1078统一在拟`docs/specs/auto-test-case/feature-organized-corpus.md`沉淀；本消费者只在自身原SD/模块Spec记录实际Case引用、目标动作和正式证据，不再各写一套公共协议。AC-FC属于本包验收增量，原SD与目标义务全部保留。

### FC · canonical语言输入与旧义务承接

A2R公共语言测试复用language按特性Case，在同Case加生成/真实rustc构建/运行检查；Rust emit金样单独保留。UI/AGE完整原源、GUI/GPU/资源oracle仍读auto-ui/age-examples等canonical来源并保本包原门，不把demo另抄进language或用语言例代W3。

由原T-01/T-02/T-03按其职责消费/迁移对应行，保原source/expected/check及条件，重复仅在完整输入闭包/语义等价时合并，原义务→canonical Case/check映射不得丢；既有固定组/suite/worker读者按1078接口切换，不新建按后端/代际维护的Auto副本。缺字段/目标接口由1078和实际producer在原框架有界补接，不要求本包先等全AV/全AAC完成。

**前置**。新增图形端到端需 1097 正式 UI-W2-AC，使用 1071 与 [AutoUI 逐例交付表](../../../auto-ui/docs/plans/README.md#u04u06-逐例交付) 所定的同源 UI/Widgets。非图形工具能力可按已交接口先行。

**接手范围**。接 980 B-RUST 的 UI/Widgets、普通 Node 及跨工程共用生成、编译、调度端口，保真实窗口、传统 UI 和全分母。[1103](1103-age-a2r-gallery-fusion-and-capabilities.md) 消费公共接口并拥有 981 B-RUST 的 AGE 共用宿主接线、域格与原公共失败验证。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

### V-FC-CONSUMER（work / 原T-01/T-02/T-03，未执行）

同一canonical source变化必须使本包实际目标/代际依赖失效并用独立oracle重验；单目标oracle变异、遗留旧活入口、删原义务映射、零选中/缺required检查和emit冒run分别判红。公共语言输入去重不减少旧资源/错误/恢复/平台条件；typed IR/后端专项及外仓完整产品保原owner。实际结果按Case＋check＋编译器/目标/动作＋source/SDK/provider身份记录，不以AV或其它代结果回填。

先用原样 Iced UI 工程验证生成、冷构建和真实窗口，并交付可消费版本。982 接受该来源后联合验公开 CLI 的 `auto gen`（A2R Rust 产物）、`auto build`（干净目录编译链接）、`auto run`（真实 Iced 窗口、事件及关闭）。1074 交生成、宿主及行为依据，982 验公开 CLI 与工具派发，联合结果供 985/B。

原 V-UI-W3、V-UI-AXR0、V-UI-SOURCE/FULL、980 V-B-RUST、V-R-FAIL 的 UI/通用工程部分。冷生成→真 rustc→运行；同独立事件/资源/GPU oracle；坏源保旧、错宿主身份拒绝、真实 dispatch、无 VM/AV/旧 emitter 回退。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

- [ ] **AC-FC｜owner_stage: work**：V-FC-CONSUMER及§5 FC逐原义务满足，公共语言唯一作者/本目标真实动作与独立结果/迁移映射均可追溯；原独立review复核，merge只接真实落主来源。未交required目标保pending，不因共享目录、少文件或另一端通过而降门。

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [ ] **T-01｜work｜核公共源与发行链**。核 980 AXR0 与已交 981 的多模块源、原 API、source/SDK/ABI、978 构建及 AGE typed 宿主接口；在 `auto-lang/experimental/ace/crates/a2x-rust` 与 `auto-ui/experimental/autographics-auto-driver/tests/standalone_game_player.rs` 核正式 W2 来源与现有运行入口。

- [ ] **T-02｜work｜完成 UI A2R 和公共工程宿主**。统一生成/编译/调度、侧车、props/msg/route、direct/RQ 及资源；在已有 provider 写面实现并向 982 和 AGE 能力计划交可消费接口。

- [ ] **T-03｜work｜验收 W3 与工程运行**。沿原 V-UI-W3 和 V-B-RUST 的 UI/通用工程格真构建、真运行逐条对证，分别记录 W3 和 Iced 工程输入/产物/宿主的正式交付，由 982 用于真实 CLI 联验。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

- 2026-11-19 r4规划交接：明确AV路线只选AV，非AV#ignore由本目标实施阶段逐项恢复；原任务/状态/进度、完整门与原预期保持，本轮没有实现或运行。

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（规划静态预检），`next: work`待后续指令/所需来源。新增FC/V-FC-CONSUMER/AC-FC，原T-01/T-02/T-03补接同源语言/迁移合同，任务/进度不变；未移动用例、修改expected、运行或合入。

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

由 T-01 核实原用例位置、实际依赖接口与待冻结的生产入口。
