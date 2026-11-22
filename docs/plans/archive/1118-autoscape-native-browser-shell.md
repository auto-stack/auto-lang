---
plan_id: PLAN-1118
status: archived
feature_name: AutoScape 本地浏览器七区外壳
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 8
total_steps: 8
depends_on: [PLAN-1021, PLAN-1023, PLAN-1106, PLAN-1109]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/local-browser-shell.md]
touched_goals: [GOAL-010]
affects: [ase, age/autoui-runtime]
---

# [PLAN-1118] AutoScape 本地浏览器七区外壳

## 0. 变更摘要

用户 2026-11-20 确认先做本地七区外壳，以 Counter、Clock 和 Gallery 验收地址、历史与收藏，再实施 HTTPS 只读访问。承接 [外壳设计](../design/autoscape/browser-shell.md)，复用 1109 已交原生会话。当前 0/8，仅制定实施合同。

## 1. 目标

- 一个 Windows x64 原生 AutoScape 窗口具备标题栏与菜单、地址栏、右侧工具栏、多个页签、收藏栏、中央内容区及底部状态栏。每区有可操作或可验证的职责。
- 地址栏、Gallery 目录卡片、收藏和历史都能通过同一导航入口到达同一已登记应用。用真实 Counter 输入和 Clock 切回校时证明应用生效。Gallery 首页是两项应用的轻量目录。
- 验收每页历史、失败/取消、两页签独立会话、关闭回收、收藏进程重启保留。向 HTTPS 后继交浏览器导航请求、完成/取消、文档视口与状态的公共接缝。
- 首期是本地验证入口。采用 Counter、Clock、Gallery 和已登记根 `app://`；Todo、远程激活、HTTPS、整个 Gallery 产品切换及发行分别按后续合同。

## 2. 架构方案

`普通 AutoUI 外壳 → 导航动作（地址/目录/历史/收藏）→ ASE 解析与应用身份 → 公开原生宿主挂载 → 选中页签 AutoFrame 与状态`。外壳负责浏览器 UI 和用户配置，ASE 负责目标、代次及完成提交，ACE 继续执行 Counter/Clock 业务，AGE / AutoUI 持有视口与输入。Gallery 首页用目录元数据呈现，点击才装载 guest。1119 在同一导航接缝注册网络目标。

## 3. 技术栈

语言仓的 `experimental/ase-m0/src/{uri,registry,run}.rs` 提供现有解析/装载依据，`experimental/ase-m0/fixtures` 提供独立 Counter、Clock 源。AutoUI 当前 `crates/auto-ui-runtime/src/ui/ase_m0_sessions.rs` 有选中车道、会话控制和关闭审计，`examples/ase_m0_sessions_window.rs` 是可复跑真窗基线。普通 AutoUI authoring 入口、声明式外壳项目路径和公开接线由 T-01 用真窗口选择并登记。拟同组 `D:/autostack/.wt/lang-1118/{auto-lang,auto-ui}`，分支 `plan-1118-dev`。依赖只读树按资源闭包补齐，材料统一放 `D:/autostack/plan-artifacts/plan-1118/`。

## 4. 需求分析与背景调查

本次授权限起草两份计划及登记实施顺序，禁止启动实现、创建实施树、构建或运行验证。核查基线 auto-lang `c6544ee5e`、auto-ui `448ed614`（2026-11-20），工作阶段重钉实际版本和依赖。

**2026-11-22 实施授权**：用户 `/goal 完成计划1118` 经 /auto-plan-goal 授权全链（work→review→bounded repair→merge），目标/验收/范围/预算按本合同不变。工作树 `D:/autostack/.wt/lang-1118/{auto-lang,auto-ui}`，两仓分支 `plan-1118-dev`。依据 [模块总览](../specs/overview.md)、[原生会话](../specs/ase/m0-interactive-sessions.md)、[页签定时](../specs/ase/visible-tab-timers.md) 和 [1109 窗口收据](archive/1109-ase-clock-tab-lifecycle.md)。

| 当前接缝 | 事实和影响 |
|---|---|
| ASE `parse_app_uri`、`Registry::lookup`、无窗口 `run_app` | 已收严格根 `app://` 和固定登记闭包；未与原生页签直接接通。候选演示根为 `app://auto.demo.counter/`、`app://auto.demo.clock/`、`app://auto.gallery/`，前二指既有 demo，Gallery 为内建目录。T-01 冻结正式注册名及输入归一化规则。 |
| `AseSessionHost::{open_session,select,close_session,close_all}` 与 1106 AV 审计 | 已证真实事件、状态根和单显示所有权；原窗口按显式文件路径打开，有脚本和实际窗口事件。导航失败后的原子提交、历史和配置持久化属于本计划。 |
| AutoUI 普通作者入口与原生 `ace_native.rs` | 已有原生运行接缝；须实证外壳本身通过正常构建可承载菜单、地址输入、焦点、快捷键以及共享内容宿主。单个原型视图出现控件不算全部前置齐备。 |
| [PLAN-1111](1111-ase-local-todo-service.md) 与 [PLAN-1112](1112-ase-native-todo-and-persistence.md) | 1111 的无窗口后端证据和本计划可并行；1112 的原合同依赖 1111 及具体 013 AV 能力。本计划无 Todo 依赖。1112 若后续把真实 Todo 装进产品外壳，仅该接线任务消费本计划的公开宿主接口，原存储和 013 验收照既有顺序。 |

## 5. 详细设计

- **T-01 先证接缝。** 在独立树上用普通 AutoUI 外壳作者源至真原生窗口，先证一个菜单动作和一个地址提交共走真实 ASE 导航，且选中 frame 能显示一个原样例真实业务输出。固定宿主接口、Windows 版本、已验组件、进入/失败/撤销点及存储 owner。如果共享作者能力或嵌入接口缺失，交对应 ACE/AGE/AutoUI owner，具名影响任务和复验；不通过手写宿主控件绕过应由普通 AutoUI 定义的外壳。
- **七区的实际操作。** 标题采用 AutoScape 和系统窗口控制。文件/查看/导航/收藏菜单分别连新建关闭、收藏栏显示、后退前进刷新停止、添加删除等已实现动作。右侧工具栏连接收藏、可收起的右侧收藏面板和窗口菜单。地址栏支持 `Ctrl+L`、输入根地址或已登记应用名及回车。页签支持 `Ctrl+T/W`、点击切换、关闭焦点回落。收藏栏支持 `Ctrl+D`、打开、删除和重启保留。主区示范真实应用和轻量 Gallery。状态栏显示已提交来源、加载/错误阶段，悬停合法目录链接时显示目标。窄窗时主控件保持可达，按外壳设计收纳次要工具。
- **导航提交。** 每个页签有独立的进行中请求代次、已提交地址、历史游标与视口。正常导航在新目标可呈现时一次提交，成功分支导航才截断前进记录。后退/前进只移动游标；刷新当前目标不加历史，未保留的历史会话按原地址显式重开。失败、停止、关页和同页新请求使旧完成失效，保留正确的已提交地址与原记录。地址栏编辑、悬停提示和错误信息不能提前替换已提交来源。
- **资源与配置。** 切回仍驻留页签保留当前 AV 会话，Clock 恢复只发一条新校时。目录和空白页签是元数据，活 guest 上限沿 1109 的两份；准备中的挂载同样计入上限。容量不足具名提示释放后重试，保留已提交页面。收藏保存标题、规范地址和打开方式，位于宿主用户配置目录，独立于 app 的 Realm。通过既有配置/原子写入口保存，失败向外壳报告并保留旧提交。
- **交接边界。** 输出可独立消费的导航请求/代次/取消、已验证目标提交、视口/状态和配置接口。用内建 Gallery 先证非 guest 原生内容可进入同一视口，再记录给 1119 的文档内容、来源与失败状态接缝。1111 独占 Todo 服务/租约实现，1118 独占外壳导航和本地产品接入；并行工作在各自工作树。若都要修改同一公共文件/API，先约定唯一生产者，依赖其已落地交付再接消费者。

### 规范增量

| ID | 操作与目标 | 来源和验收 |
|---|---|---|
| SD-01 | 新增 `docs/specs/ase/local-browser-shell.md`，记录已验七区、目标与历史语义、收藏配置、资源上限和订阅接口 | AC-01–04 与冻结的真窗、CPU、重启证据。 |
| SD-02 | 由 AutoUI owner 在 `docs/specs/runtime/ase-m0-frame-consumption.md` 记录公共浏览器消费/输入和原生启动入口；ASE 路线标明 1119 的正式上游 | AC-01/03/06。当前仅候选，合入时逐条核对。 |

## 6. 测试设计

以下验证均未执行。T-01 固定两个实际窗口尺寸（常用和产品支持的窄窗近邻）及既有原生准备命令，禁止用脚本的最后文本替代真实 UI、状态与释放审计。

| ID | 入口及可判别预期 |
|---|---|
| V-01 | T-01 在普通外壳源和真实窗口先证最短路径，随后真实窗口验齐七区及每区对应动作或状态；打开菜单、操作输入、切页、收藏面板、状态反馈，覆盖窄窗、快捷键、焦点与真实内容。 |
| V-02 | 拟增 AutoUI `--test autoscape_shell_navigation`，从地址栏、Gallery 卡片、收藏触发同一 resolver，观察目标、业务源、真实 Counter handler 与 Clock。只进入目录/新建首页时编译、guest init 和 demo 进程/监听均为零；三种打开入口都能到达实际 app。 |
| V-03 | 用同一原生导航生产路径执行 Gallery→Counter→Clock→后退→前进、退回后访问新目标、刷新及停止。断言每页完整历史、游标与视口；同页 A→B→A 注入旧完成，坏地址、错代、失败重试、关页晚完成均不污染地址或目标。用故意先提交地址或放过旧完成的受控负例，证明验收能识别错误。 |
| V-04 | 两个并存页签、两份同源 Counter 隔离、达到上限再重试；切回 Clock 前隐藏零业务/呈现，恢复只见一次校时。对账 1106 执行层和原 1109 释放证据，核最后关窗状态根、显示车道、Clock 唤醒和导航回调全部收敛。 |
| V-05 | 使用隔离浏览器配置目录，在两个真实进程中添加、打开、删除收藏并检查重启；写入失败与坏配置分别记录，旧有效写入可读。正常窗口菜单/收藏栏和内容的输入路由独立。 |
| V-06 | 沿 T-01 确定的真实启动入口排真窗档，原生自动与物理/真人分账。AutoUI 用 `cargo test -p auto-ui-runtime --features ace-bridge --test <目标> -- --test-threads=1` 跑新增目标和 `ase_m0_sessions`、`ase_clock_lifecycle` 及实际改及的 1106 审计目标。解析/注册改面跑 `cargo t ase-m0`；既知 nextest 配置问题若仍在，记录原错误并以同 workspace 的完整测试核实，不减覆盖。 |

## 7. 验收标准

| ID | owner_stage 与验收 |
|---|---|
| AC-01 | work。Windows 真窗有七个有效区域，支持所列主动作、窄窗和键盘焦点；外壳源和真实宿主入口可复跑。V-01/06。 |
| AC-02 | work。同一地址导航入口覆盖地址、目录、历史、收藏，后退前进、刷新/停止正确；坏请求或晚完成不改变已提交态。V-02/03。 |
| AC-03 | work。真实 Counter/Clock 业务、tab 隔离、驻留上限和最后 owner 释放通过；空目录不启动 guest/HTTP。V-02/04/06。 |
| AC-04 | work。添加、打开和删除的收藏经重启一致，存储故障可观测且保留最后有效配置。V-05。 |
| AC-05 | review。独立核 AC-01–04、普通 AutoUI 源、真实导航/执行/持久化观测、跨仓写面和零新增告警。 |
| AC-06 | merge。复验冻结范围、落地规范和归档，登记 1119 接口/源码与复跑入口并清理。 |

## 8. 执行步骤

有原生任务工具时，开始实施前按 ID 登记任务。

- [x] T-01 冻结普通外壳和原生接缝
  - owner_stage: work。核 1021/1106/1109 正式版和实际宿主；固定示例身份、最低原生能力和 T-01 的窗口正例。V-01，AC-01。
  - [✅ 2026-11-22 冻结] 基线重钉：auto-lang `c5fd06592` / auto-ui `d045f4a4`（组 `D:/autostack/.wt/lang-1118/{auto-lang,auto-ui}`，两仓分支 `plan-1118-dev`）。**正式注册表（冻结）**：`app://auto.demo.counter/`（显示名 Counter，源 auto-lang `experimental/ase-m0/fixtures/counter-a/main.at`，按钮序 0=-/1=Reset/2=+）、`app://auto.demo.clock/`（Clock，`fixtures/clock/main.at`，Tick(int) 校时）、`app://auto.gallery/`（内建目录，零 guest）。**输入归一化（冻结）**：trim → 显示名大小写不敏感匹配 / 裸 identity → `app://<identity>/`（根路由缺省补 `/`）→ ase-m0 `parse_app_uri` 严格解析（单一语法源，auto-ui 经组内兄弟 path dep 复用，不复制解析）→ 冻结表 lookup，未登记具名拒且已提交地址不变。**最低原生能力（已核源）**：`View::input(on_text/on_submit)` 地址栏；确定性 input id `auto_input_{placeholder}_{width}_{password}` + `Component::take_focus_request`（crate 内 `FocusRequest::new`）承载 `Ctrl+L` 聚焦；`View::MouseArea(on_enter/on_exit)` 承载状态栏悬停目标；`AUTO_VM_WINDOW=WxH` 播种窗尺寸；`style::theme::window_width()` 窄窗判据（<720 收纳）。**共享缺口（本计划内修复，affects 含 age/autoui-runtime）**：`TickWrap::keyboard_subscription` 现丢弃 Ctrl 修饰键——扩展为组件 `key_bindings` 声明了 `Ctrl+<Key>` 名才投递（增量，未声明组件零影响）。**存储 owner（冻结）**：收藏/配置落 `~/.config/autoos/apps/autoscape/bookmarks.json`，env `AUTOSCAPE_HOME` 覆盖根（隔离测试），原子写经既有 `auto_lang_core::state_file::atomic_write`（与 desktop_config 同入口）。**窗口正例（冻结）**：常用 1280x800（默认）/ 窄窗 520x700（`AUTO_VM_WINDOW=520x700`）；准备档 `--prep-only`；真窗证据 `--evidence <json>`。**会话层**：原样复用 `AseSessionHost`（两活上限/选中车道/close 代次/Clock 唤醒），外壳只拥有 tab/历史/收藏/地址元数据。
- [x] T-02 搭齐七区与 Gallery 首页
  - owner_stage: work；依赖 T-01。编写普通 AutoUI 外壳、菜单/工具/页签/状态和无执行的目录，接真原生宿主。V-01/02，AC-01/03。
  - [✅ 2026-11-22] `crates/auto-ui-runtime/src/ui/ase_browser_shell.rs`（`AutoscapeWindow` 七区：菜单栏+右侧工具/地址栏（后退/前进/刷新-停止/输入）/页签行/收藏栏/内容视口/状态栏；`View::input(on_text/on_submit)`、`View::MouseArea(on_enter/on_exit)` 悬停目标、窄窗 <720px 收藏栏自动收纳+标题截断+⋯ 菜单兜底）；Gallery 首页为元数据卡片（点击才装载）。真窗证据：`p1118_window_e2e_navigation_normal_and_narrow` 常用 1280x800 + 窄窗 `AUTO_VM_WINDOW=520x700` 两档 7.88s PASS（frames≥20、零错误、历史/收藏断言）。
- [x] T-03 接通本地地址、历史与发布边界
  - owner_stage: work；依赖 T-02。接现有 registry/parser 与真实会话，完成导航提交、错误、取消和资源上限；固定给 1119 的接口。V-02/03/04，AC-02/03。
  - [✅ 2026-11-22] 会话层扩展（ase_m0_sessions.rs 增量）：`begin_open`（真实装载成票据）+`mount_open`（权威容量复检+车道落位；`open_session`=两者组合，既有行为不变）+`deselect`（切无 guest 页签撤车道）。外壳导航状态机：每页签 `nav_generation`，受理=代次推进+分段装载，提交=下一 step 核代次后一次提交（新目标截断前进/后退前进只移游标/未驻留历史显式重开/刷新不加历史换新会话）；停止/抢占/关页整票失效（票据 drop=引擎释放零 UI 效果）；容量拒保留已提交页面。负控在案：`p1118_v03_stop_invalidates_pending`、`p1118_v03_superseded_request_only_newest_commits`、`p1118_v03_close_tab_invalidates_pending`。给 1119 的接口冻结于 SD-02（导航请求/代次/取消、已验证目标提交、视口/状态、配置接口）。
- [x] T-04 接入本地收藏和配置
  - owner_stage: work；依赖 T-03。菜单、工具、收藏栏走同一导航，固定重启、关闭、配置失败。V-02/05，AC-02/04。
  - [✅ 2026-11-22] `BookmarkStore`：`~/.config/autoos/apps/autoscape/bookmarks.json`（env `AUTOSCAPE_HOME` 隔离），`auto_lang_core::state_file::atomic_write` 原子写；同地址去重；菜单/工具/收藏栏/地址栏共用 `navigate`。证据：`p1118_v05_favorites_roundtrip_across_restart`（两代实例一致）、`p1118_v05_corrupt_config_named_error_old_write_readable`（坏配置具名+空表可用+修复性保存+旧有效写读回）、`p1118_v05_write_failure_observable`（阻断根具名失败+盘上有效写未被破坏）。
- [x] T-05 固化正反控与排窗验收
  - owner_stage: work；依赖 T-03/04。用实际 UI 和数据路径完整跑 V-01–06，绑定独立正反控和执行层证据。AC-01–04。
  - [✅ 2026-11-22 复审修复后重开→完成] RF-1 修复：`p1118_v01_component_message_wiring_all_regions`（七区动作组件消息面全覆盖——地址编辑/回车、frame 点击、菜单展开/后退/前进/刷新/停止、刷新-停止双臂、查看/收藏菜单、工具☆/面板、收藏打开/删除、页签新建/切换/关闭、悬停/退出、失败重试，逐臂断言宿主真实态）；RF-3 修复：`p1118_v03_mount_capacity_refusal_keeps_committed`（双 staged 票据→先提交落位 live=2→后提交 mount 具名 session.limit 拒+已提交 Gallery 保留+释放后重试可达）。全目标 21 case CPU 档 2.00s 全绿（9fc41dd4）。
  - [✅ 2026-11-22 第一轮证据] 新目标 `tests/autoscape_shell_navigation.rs` 19 case CPU 档 1.91s 全绿（归一化语料含 ase-m0 同语法拒绝面、Gallery 零装载、三入口达真 Counter（真实 handler 3 次）、历史/截断/刷新/停止/抢占/关页、容量拒+释放重试、双页签同源隔离 3/7+切换零装载、Clock 隐藏零派发+恢复恰一条校时、编辑/悬停不动已提交、快捷键绑定面+Ctrl+L 聚焦一次性消费）；排窗档 `AUTO_UI_ASE_M0_E2E=1` 单实例真窗两档 7.88s 绿。V-06 回归并集：`ase_m0_sessions` 9✓（**预存红修复**：PLAN-108 增设 `DispatchOutcome::InFlight` 后测试 match 不穷尽——门禁修复为 Ok 全拒臂，语义不变）、`ase_clock_lifecycle` 5✓（prep envs `ACE_NODE_DESKTOP_ROOT`/`ACE_NODE_DESKTOP_PREPARED_RUSTC`）、`component_event_sessions_1106` 4✓；`cargo t ase-m0` 未跑——auto-lang 零代码改面（ase-m0 crate 原样 path-dep 消费，仅文档）。计时：新 case 全部 ≤10s（CPU 档 1.91s/19 case；真窗 7.88s/2 窗）；准备=示例冷构建 4m13s+provider prep（rustc×2 缓存），日常档零 rustc。真鼠标/键盘 OS 注入与真人走查归手工档（1022/1023/1109 同界分账）。
- [x] T-06 冻结交接与规范候选
  - owner_stage: work；依赖 T-05。绑定 SD-01/02、导航/取消和只读文档挂载接口、真实入口、已知边界。AC-01–04。
  - [✅ 2026-11-22 复审修复后重开→完成] RF-2 修复：SD-01（auto-lang 6f2bfaf80）与 SD-02（auto-ui 9fc41dd4）Ctrl 投递机制文本更正为准确机制（订阅对非空 key_bindings 组件投递 `Ctrl+<字母>` 组合名、`key_message` 分派过滤、未映射组合零效果）；语义无变更。
  - [✅ 2026-11-22 第一轮] SD-01 候选已落 auto-lang worktree `docs/specs/ase/local-browser-shell.md`；SD-02 已落 auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md`。真实入口 `examples/autoscape_window`。已知边界：跨页签并发导航=活动页签单导航；只读文档挂载接口（Gallery 同视口）已验，https 文档内容接缝归 1119。
- [x] T-07 独立复审
  - owner_stage: review；依赖 T-06。实际核源码与关键正反控，核验七区、原生输入和资源终态。AC-05。
  - [✅ 2026-11-22] 第一轮 final needs_fix（RF-1/2/3，见 §9）；修复轮 auto-ui 9fc41dd4 / auto-lang 6f2bfaf80 后复审：修复 diff 仅测试+文档（实现零改，既有真窗/回归证据不失效）；RF-1 用例 36 处 `window.on(ShellMsg::…)` + 30 断言逐臂钉住七区动作接线；RF-3 双 staged 票据序列断言先提交落位/后提交 mount 具名拒/已提交保留/释放重试；RF-2 两处文本与实现机制一致（语义零变更）。冻结树重跑全目标 21 case 2.11s 全绿；本计划文件零源码告警（唯一 warning=MSVC 链接器信息性输出）。**final pass**。
- [x] T-08 正式落地
  - owner_stage: merge；依赖 T-07 pass。复验冻结职责并集、落地规范和 1119 上游接缝，归档、更新引用并清理。AC-06。
  - [✅ 2026-11-22 merge] 落地：auto-ui `9fc41dd4`（==复审冻结树，master 零移动直 ff）；auto-lang rebase 干净后 ff（映射 4341cdcf1→c06c7b1ad、6f2bfaf80→63120e372，docs-only 无等价性风险）。canonical_verified：SD-01 `docs/specs/ase/local-browser-shell.md`（master sha256 a4631e69e84966a5…）、SD-02 `docs/specs/runtime/ase-m0-frame-consumption.md`（master sha256 22f5b47daa7c7a17…，PLAN-1118 节在案）。冻结门禁复用：V-06 并集（新目标 21✓/sessions 9✓/clock 5✓/1106-4✓）绑定 9fc41dd4 落地树，输入同一（修复轮仅测试+文档，回归目标输入未变）；auto-lang 侧纯文档 → 内容/链接检查。1119 上游接缝登记于 SD-02（导航请求/代次/取消、目标提交、视口/状态、配置接口）。清理：组内四树（auto-lang/auto-ui/auto-web/auto-down）wt-guard clean 后移除，分支 plan-1118-dev 删除，组目录删除；plan-artifacts/plan-1118 证据已折叠入本文件后整目录删除，登记表刷新。

## 9. 复审记录

stage: new | plan_id: PLAN-1118 | plan_revision: r1 | outcome: pass（起草交接） | next: 取得实施指令后进入 T-01。本轮验证未执行。

stage: goal | plan_id: PLAN-1118 | plan_revision: r1 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1118/{auto-lang,auto-ui} | commit: base auto-lang c5fd06592 / auto-ui d045f4a4 | next: work（/goal 完成计划1118 授权全链 work→review→bounded repair→merge；授权记录入 §4）

stage: work | plan_id: PLAN-1118 | plan_revision: r1 | outcome: pass（execution_done） | worktree_path: D:/autostack/.wt/lang-1118/auto-ui（实现）+ D:/autostack/.wt/lang-1118/auto-lang（SD-01 文档；依赖只读树补齐 auto-web 0cfc7c7d / auto-down 456c86a detached） | branch: plan-1118-dev（两仓） | base_commit: auto-ui d045f4a4 / auto-lang c5fd06592 | code_commit: auto-ui 82881477 / auto-lang 4341cdcf1 | dependency_revisions: ase-m0 crate 原样 path-dep 消费（auto-lang 零代码改面）；ui980 provider=target/ace-node-desktop（rustc 1.98.0 缓存） | task_ids: T-01..T-06 全部完成 | evidence: 新目标 autoscape_shell_navigation 19 case CPU 1.91-2.18s 全绿 + AUTO_UI_ASE_M0_E2E=1 真窗两档 7.88s 绿；V-06 并集 ase_m0_sessions 9✓/ase_clock_lifecycle 5✓/component_event_sessions_1106 4✓；ase-m0 组未跑（auto-lang 零代码改面，裁定记录于 T-05 证据）；新 case 全部 ≤10s；准备档=示例冷构建 4m13s+provider prep，日常零 rustc；本机并行另有他人 cargo 构建（-j2 auto-ui-tools/ace-bridge），未冲突 | blockers: 无 | next_responsible_role: review（T-07 独立复审；AC-05）

**门禁修复登记（预存红，非本计划引入）**：`tests/ase_m0_sessions.rs` 的 match 因 PLAN-108 增设 `DispatchOutcome::InFlight` 不再穷尽（master 基线即编译失败，diff 佐证本计划未触该枚举）；修复为 Ok 全拒臂（语义不变：单元消息带参派发不允许任何 Ok），随本计划提交（82881477）。

stage: goal | plan_id: PLAN-1118 | plan_revision: r1 | round: 2 | from: work | to: review | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1118/{auto-lang,auto-ui} | commit: auto-ui 82881477 / auto-lang 4341cdcf1 | next: work（RF-1/2/3 修复）

stage: review | review_scope: final | reviewer_identity/limitations: 实现同会话复审（无独立上下文可用；结论自冻结工件独立重构——重跑+源码/断言巡检，不信执行叙述） | plan_id: PLAN-1118 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1118/auto-ui（实现）+ auto-lang（文档） | branch: plan-1118-dev | reviewed_commit: auto-ui 82881477（树 clean）/ auto-lang 4341cdcf1 | base_commit: auto-ui d045f4a4 / auto-lang c5fd06592 | dependency_revisions: ase-m0 原样 path-dep；ui980 provider 缓存 rustc 1.98.0 | spec_inputs: SD-01 候选 local-browser-shell.md（4341cdcf1）+ SD-02 ase-m0-frame-consumption.md 节（82881477） | acceptance_results: AC-01 partial、AC-02 pass（host 面全证）、AC-03 partial、AC-04 pass、AC-05 本轮 | pending_closeout_gates: 冻结后 merge 档统一回归（本仓 V-06 并集已绿，merge 复用） | findings: RF-1（major，AC-01/T-02/T-05）组件消息面零验证——`AutoscapeWindow::on` 的 AddressEdited→AddressSubmit/菜单/工具/收藏/页签/悬停/重试接线无任何用例（grep 证零命中；脚本与 CPU 档均直达 host），七区动作接线错误现有门不可见；最小修复=组件级用例全覆盖各动作臂（安装宿主→发 ShellMsg→断言宿主态），复验=该用例+受影响断言。RF-2（text，T-06/SD-01/SD-02）规范文本"key_bindings 声明 Ctrl+<Key> 才投递"与实现不符（实际=订阅对非空 key_bindings 组件投递 Ctrl+<字母>名，key_message 分派过滤；可观察保证"未声明组件零影响"成立）；修两处文本后复核，语义无变更不需升版。RF-3（minor，AC-03/T-05）`mount_open` 容量复检路径零覆盖——两页签各持 staged 票据→先后提交→后者 mount 具名拒且已提交页保留（合同"准备中的挂载同样计入上限"）；最小修复=双 staged 提交序列用例 | evidence: 复核重跑（82881477 冻结树）autoscape_shell_navigation 19✓ 2.33s；巡检=测试断言/负控逐条读+diff 边界核（fmt 溢出已清洗，最终 diff 139+/9- 全部本计划触面）；新 case 全部 ≤10s（计时表见 work 收据）；重用证据：真窗两档 7.88s、回归并集 9/5/4✓（输入未变，本轮未重跑） | next: work（修复 RF-1/2/3 → 复审受影响面）

stage: work | plan_id: PLAN-1118 | plan_revision: r1 | outcome: pass（复审修复轮：RF-1/2/3 全闭） | worktree_path: D:/autostack/.wt/lang-1118/auto-ui + auto-lang | branch: plan-1118-dev | base_commit: auto-ui 82881477 / auto-lang 4341cdcf1 | code_commit: auto-ui 9fc41dd4 / auto-lang 6f2bfaf80 | dependency_revisions: 不变 | task_ids: T-05/T-06 重开→完成 | evidence: RF-1→`p1118_v01_component_message_wiring_all_regions`（逐臂断言宿主态）；RF-3→`p1118_v03_mount_capacity_refusal_keeps_committed`（先提交落位/后提交 mount 具名拒/已提交保留/释放重试）；RF-2→SD-01/SD-02 文本更正（机制描述与实现一致，语义零变更）；全目标 21 case CPU 档 2.00s 全绿；实现代码零改动（仅测试+文档），既有证据（真窗/回归并集）不受影响 | blockers: 无 | next_responsible_role: review（受影响面复审）

stage: goal | plan_id: PLAN-1118 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: pass（修复完成） | worktree: D:/autostack/.wt/lang-1118/{auto-lang,auto-ui} | commit: auto-ui 9fc41dd4 / auto-lang 6f2bfaf80 | next: review（受影响面复审）

stage: review | review_scope: final（修复复审） | reviewer_identity/limitations: 同会话修复复审（限制同前；对修复 diff 逐项独立核——RF-1 用例 36 派发/30 断言、RF-3 序列断言、RF-2 文本对照实现） | plan_id: PLAN-1118 | plan_revision: r1 | outcome: **pass（final）** | worktree_path: D:/autostack/.wt/lang-1118/auto-ui + auto-lang（两树 clean） | branch: plan-1118-dev | reviewed_commit: auto-ui 9fc41dd4 / auto-lang 6f2bfaf80 | base_commit: auto-ui d045f4a4 / auto-lang c5fd06592 | dependency_revisions: ase-m0 原样；ui980 provider 缓存 rustc 1.98.0 | spec_inputs: SD-01 local-browser-shell.md（6f2bfaf80）+ SD-02 ase-m0-frame-consumption.md 节（9fc41dd4）——内容与实现一致，语义零变更（RF-2 仅机制描述更正，不升版） | acceptance_results: AC-01 pass（七区+主动作+窄窗+键盘焦点：真窗两档+组件消息面全覆盖+Ctrl+L 聚焦）、AC-02 pass（三入口/历史/提交/负控全证）、AC-03 pass（真业务/隔离/上限含 mount 复检/释放对账）、AC-04 pass（重启一致/坏配置/写失败）、AC-05 pass（本轮 final） | pending_closeout_gates: merge 档——冻结基线复用本仓 V-06 并集证据（实现自 82881477 起未变，修复轮仅测试+文档）；周期 `cargo t all` 批量回归按到期规则另行触发（非本计划门） | findings: 无遗留（RF-1/2/3 闭合验证）；边界记录：真鼠标/键盘 OS 注入与真人走查归手工档、窄窗收纳为代码巡检+真窗渲染稳定证（无像素断言）、地址输入 id 格式与 derive_input_id 的一致性为注释+巡检锚定（未加钉死测试） | evidence: 冻结树 9fc41dd4 全目标 21 case 2.11s 全绿；计时：CPU 档 21 case 2.00-2.33s（新增 2 case 各 <1s），真窗两档 7.88s（输入未变重用），回归并集 sessions 9/clock 5/1106-4 ✓（实现未变重用） | next: merge（T-08）

stage: goal | plan_id: PLAN-1118 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1118/{auto-lang,auto-ui} | commit: auto-ui 9fc41dd4 / auto-lang 6f2bfaf80 | next: merge

stage: merge | plan_id: PLAN-1118 | plan_revision: r1 | outcome: pass（delivered） | completion_kind: delivered | checkpoints: prepared（reviewed r1 final pass + 冻结 Spec delta SD-01/02 在 worktree 已提交并随复审落地）；landed（auto-ui master tip=9fc41dd4 `--ff-only` 零移动直进；auto-lang master tip=63120e372 rebase 后 `--ff-only`，hash 映射 4341cdcf1→c06c7b1ad/6f2bfaf80→63120e372，docs-only 干净 rebase 无冲突）；canonical_verified（SD-01 sha256 a4631e69e84966a5、SD-02 sha256 22f5b47daa7c7a17，均验于 master 提交对象；无 ledger/索引写入）；archived（本文件 git mv 至 docs/plans/archive/，status: archived，docs 提交在默认分支）；cleaned（组内 auto-lang/auto-ui/auto-web/auto-down 四树 wt-guard clean → worktree remove → 分支删除 → 组目录删除；plan-artifacts/plan-1118 全部证据已折叠入本文件后整目录删除，.retention/policy.json 刷新） | delivery_commit: auto-ui 9fc41dd4 / auto-lang 63120e372 | spec_paths: docs/specs/ase/local-browser-shell.md（auto-lang）+ docs/specs/runtime/ase-m0-frame-consumption.md §AutoScape（auto-ui） | gates: 冻结并集复用（同树同输入）：autoscape_shell_navigation 21✓ 2.11s、ase_m0_sessions 9✓、ase_clock_lifecycle 5✓、component_event_sessions_1106 4✓；真窗两档 7.88s；auto-lang 纯文档=内容/链接检查；新 case 全部 ≤10s | deployment_observation: 无生产面工件（实验外壳+测试；不触发发行/重启） | next: done

## 完成摘要

本地浏览器七区外壳交付：冻结身份表（counter/clock/gallery）+ 地址归一化（ase-m0 单一语法源）+ 导航状态机（分段装载票据/代次失效/历史截断/显式重开/容量双检）+ 收藏原子持久化 + `AseSessionHost` 增量（begin_open/mount_open/deselect）+ 共享 Ctrl 组合投递扩展；21 CPU case + 真窗两档 + 回归并集全绿；一处预存红门禁修复（PLAN-108 InFlight 非穷尽 match）随行。1119 上游接缝登记于 SD-02。

**复审覆盖说明**：RF 外巡检通过项——归一化语料含同语法拒绝面锚定；导航状态机负控（停止/抢占/关页）与提交不变式有真实断言；会话层 begin/mount/deselect 为纯增量（open_session 组合复现原序列，sessions 9✓ 回归证）；收藏三态（重启/坏配置/写失败）断言到盘上字节；预存红修复（InFlight）diff 佐证非本计划引入且语义不变；本计划文件零新增告警；主检出零代码 WIP（仅计划簿记）；.wt/lang-1095 为他会话遗留未注册目录（不属本计划，不动）。窄窗收纳规则（<720px 收藏栏隐藏/标题截断）经视图代码巡检+520x700 真窗渲染稳定证明，未做像素断言（边界记录）。

## 10. 待澄清事项

无起草阻塞。T-01 先证普通外壳的公开原生承载、配置 owner 与 AutoFrame 接口；共享缺口按精确接口及原主责解决。
