---
plan_id: PLAN-1071
status: drafting
feature_name: AutoUI 示例与 Widgets 的 AV 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 1
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-979]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# [PLAN-1071] AutoUI 示例与 Widgets 的 AV 接管

## 0. 变更摘要

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

`UI-W1-AV`，绑定完整 U01–U06 与 Widgets、同版源/Schema/宿主 ABI、AV 窗口、原正负对照和资源终态。可按组件组先交正式子面。

## 2. 架构方案

auto-ui 主责，auto-lang 维护共享语义，auto-os 提供 Widgets 真实源与宿主证据。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [980 原合同](980-ace-m2-autoui.md#0-变更摘要) · [981 原合同](981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。M2A-08 已正式交付。消费 980/981 已交基础和对应组真实 EVENT-B、typed 宿主及 Widgets 输入。向 982 先交 VM 工程 `run` 所需入口与真实收据。独立组不等待全组过线。

**接手范围**。接 980 B-AV 全批，981 SESSION/LANGUAGE/PROOF/FUSION/CONTENT 中传统 UI、Widgets 与 UI 基线混合见证所需 AV 格。牵头确定 010 列表字段的共享组件 state/仅视图数据边界，给 1072 交所需共享接口。游戏与扩展场景的完整 AV 由 1075 接。

每组开工确认精确输入源、对应原用例、接口 owner 及身份。共享语义和 ABI 由原 owner 串行写入，变化时重验实际受影响格。已有阶段收据保留其验收边界。

## 6. 测试设计

接 980 V-UI-W1、V-UI-SOURCE/FULL、V-EVT-AV-CONSUME、V-NODE-DESKTOP 与 981 V-B-SESSION/V-B-LANGUAGE/V-AGE-WAVE/V-EVT 中对应 AV 格。按原分组实跑真实 Iced 窗及 001/002、输入/IME/捕获、主题、媒体/terminal、坏编辑保旧、双实例、资源归零；统一记录错误、关闭/恢复和无旧解释回退。

沿原入口记录源码/目标/SDK、命令、子终态、独立 oracle、资源、条件排除与恢复。材料归本计划的统一目录。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [x] **T-01｜work｜冻结输入**。对照 980 §9 的 U01–U06 和 981 分组交接，建立原源、Widgets canonical 路径、GUI oracle 与旧检测对应表。对应表与生产入口见 §9（2026-11-19 T-01 收据）。

- [ ] **T-02｜work｜接 AV 组件与状态**。复用 `auto-ui/crates/auto-ui-runtime/src/ui/ace_node_desktop.rs` 与现有组件/Store/事件入口；共享列表语义按现有 AGE/collections owner 单写。

- [ ] **T-03｜work｜实跑完整 AV**。沿原 V-UI-W1 真窗、`ace_node_desktop` 和 `authoring_session` 等生产测试入口逐组关闭，提供可独立验收的 VM 工程运行收据。

- [ ] **T-04｜review｜独立复审**。逐项核 AC-01..03、原任务和实际反例，修复发现后复验。

- [ ] **T-05｜merge｜正式交付**。按 owner 合入、沉淀规范，记录可消费版本，按项目规约归档及清理。

## 9. 复审记录

### 2026-11-19 stage: work（T-01 输入冻结）| PLAN-1071 r1 | outcome: pass | worktree: `D:/autostack/.wt/lang-1071/{auto-lang,auto-down,auto-ui}` | branch: plan-1071-dev×2 + auto-down detached | base: lang `f02b34b74` / ui `d2e7d7a4` / down `72b09700`

**U01–U06 原源分母（`auto-ui/examples/ui/` 36 目录实测定案）**

| 组 | 示例目录 | Widgets 页（`auto-os/widgets-gallery/src/front/pages/`） | 每波独立关闭 oracle |
|---|---|---|---|
| U01 | 001-helloworld、002-counter、003-converter、011-calculator | button、label、input、form | 原值/事件轨迹、中文/非法输入、两实例隔离（980 #14 fold 已交：`ace_node_desktop` 11/11、`p980_u01_window_input_e2e` 真窗 i0 `你好Auto`/42·`abc` 具名拒·i1 `乙侧`/-5、`p980_node_desktop_permission_revocation_refuses_before_dispatch`） |
| U02 | 004-profile-card、005-login、006-hero-section、007-stats-board、008-pricing-table、009-article-feed、010-contact-form | form/select/combobox/radiogroup/checkbox/switch/datepicker/dialog/alertdialog/popover/sheet/drawer/hovercard/tooltip/dropdownmenu/contextmenu/menubar/tabs/card/avatar/badge/breadcrumb/alert/separator/skeleton | 主题/可访问输入/布局与原独立预期；001 选择复制完整保留 |
| U03 | 012-clock、013-todo、015-notes、016-calendar、018-book-reader、022-kanban | calendar/scroll/scrollarea/datatable/table/accordion/collapsible/carousel/pagination/sidebar/toast/sonner/progress/slider | timer 不倍速、编辑/关闭/取消、列表与焦点隔离 |
| U04 | 014-weather、017-chat、023-realworld、025-sys-monitor、026-database、027-file-manager | datatable/filetree/treeview/table | 真服务/文件/DB oracle 及故障/资源 |
| U05 | 019-video-app、020-music-player、024-charts、029-photo-gallery、030-video-player、031-image-viewer、031-paint | area-chart/bar-chart/donut-chart/line-chart/flow-diagram/carousel/code-editor | 实际输入/ROI/播放状态/坏资源恢复，原平台条件保持 |
| U06 | 032-handler-async-probe、038-minesweeper、041-auto-edit、021-blog-viewer、028-launcher | kitchen-sink/command/navigationmenu/absolute/alignment/center/col/flex/grid/grid-span/position/responsive/row/sizing/spacing | 原状态/事件/卸载/资源与复杂组合、全部分母余行=0 |

**Widgets canonical 路径**：`auto-os/widgets-gallery/src/front/{pages,components}/`（70 页 + 18 组件；components 为 pages 的 `use { package: official }` 依赖闭包）。ui-gallery Demo/组件为副本，建立映射去重、不双计（980 §5 :489 口径）。跨仓归 OS064 owner，本计划只消费规范源不反卡 OS 组装。

**GUI oracle 与旧检测（981 W1 五组证据，绑定 `3047b3481`，980 §9 :1070 入账）**：G-INSTANCE 双实例 2:1（`plan981_event_b_av`/`plan981_cross_module_nodes`）；G-LIFECYCLE 关开（`plan981_gfx_host`/`plan981_gfx_hooks`）；G-VIEW/FRAME/INPUT（`plan980_*`/`plan979_*` AV 家族 75/75）；G-BADEDIT 坏编辑保旧（源侧 core 三 case；会话侧 auto-ui R64-1 为正式来源）；G-RESOURCE 资源归零（会话腿 `plan981_resource_ledger` 2 例；AGE 节点资源面归 UI058/1068）。

**生产测试入口与命令（冻结，沿 980 §6 :756/:838 与 #14 收据）**：UI CPU `cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1`（11/11，串行必需）；UI 真窗 `AUTO_UI_ACE_NODE_E2E=1` + `examples/ace_node_desktop_window --prep-only`（`p980_u01_window_input_e2e` ≈6.4s、`p980_node_desktop_window_e2e_counter_full_script` ≈2.2s）；UI Hello `cargo test -p auto-ui-runtime --features ace-bridge --test ace_ui_hello -- --test-threads=1`；UI 会话 `experimental/autographics-auto-driver/tests/authoring_session.rs`（981 SESSION 批入口）；lang `cargo t ace-source ace-av`（980/981 各轮已验）。V-UI-W1 判据（980 §6 :834）：全部分母对独立交互/错误/资源 oracle，目录空选/漏页/只截图/只CPU/手写 Rust 业务红，不要求 AC/A2R。

**跨仓来源身份**：lang `f02b34b74`、ui `d2e7d7a4`、down `72b09700`（各自主检出 tip，1071 组分支基面恒钉 master）。

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

T-01 已核实：原用例位置（auto-ui/examples/ui 36 目录 + auto-os/widgets-gallery 70 页）、实际依赖接口（ace_node_desktop BridgeEngine、authoring_session、auto-lang-ace-host）与生产入口均已冻结（§9 T-01 收据）。待办：010 列表字段共享组件 state/仅视图数据边界由 T-02 牵头确认并交 1072；U02–U06 逐组 AV 实跑发现的具名墙按 §10 在此追加。
