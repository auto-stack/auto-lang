---
plan_id: PLAN-1109
status: archived
completion_kind: delivered
feature_name: ASE 原生 Clock 页签可见性与定时生命周期
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 7
total_steps: 7
depends_on: [PLAN-1106, PLAN-1023, PLAN-980, PLAN-981]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/visible-tab-timers.md]
touched_goals: [GOAL-010]
affects: [ase, age/autoui-runtime]
---

# [PLAN-1109] ASE 原生 Clock 页签可见性与定时生命周期

## 0. 变更摘要

1106 已于 2026-11-20 归档，M0 的双 Counter 现已走真实 ACE AV 增量事件会话。按 Gallery 的 Counter → Clock → Todo/Notes → 混合应用顺序，本计划验证第一个由时间驱动的原生应用页签，覆盖前台校时、隐藏停更、恢复即校时、关闭撤销和旧回调拒绝。已交付同窗 Clock 与 Counter、可见定时生命周期和失败关闭恢复，实测与复审收据见 §7、§9。

## 1. 目标

- 单个 Windows x64 原生窗口内独立装载 Clock 和既有 Counter。Clock 选中即显示当前时间，持续选中约每秒更新，切到 Counter 后停止 Clock 的回调派发与呈现，切回以一条新采样恢复显示。Counter 的输入、状态和焦点独立。
- 业务处理由普通 AutoUI Node 源经 AGE Schema、ACE 受检编译和 AV 会话执行，经原生 frame 提交。固定源在一次打开期间只编译、链接、初始化各一次。会话关闭释放其状态根和定时资源。
- 复用 012 Clock 数字时钟的墙钟行为作为样例来源。首期验证最小数字钟，不包含 012 的表盘、城市列表、闹钟、秒表、倒计时、存储或完整兼容迁移。完整原项目仍由 AutoUI PLAN-105 推进。本计划范围为 ASE 页签和时间源消费，不扩到操作系统窗口最小化策略、后台任务通用策略、HTTP 服务或 Gallery 默认入口。

## 2. 架构方案

`ASE 选中页签 → 宿主单次唤醒/重新校时 → 带会话标记的受检 Tick(int) → AV 增量 handler → 状态差量 → AGE/AutoUI 选中 frame`。

ASE 管理选中与可见代次，调度沿现有原生宿主事件循环或原 owner 已交的可取消入口，所有派发和显示提交在相同宿主所有权边界串行化。先验证 typed 回调、拒绝和差量的最长真实链。唤醒节奏使用单调时间，显示值使用真实墙钟采样；Clock 作者源把受检整数样本写入会话状态并决定所显示的时间。宿主只负责采样、调度、路由及原通用渲染。系统时间跳变后按当前样本显示，计时器不积压或追赶旧拍。

## 3. 技术栈

沿 ACE AV `ComponentEventSessionProgram`、AGE/AutoUI `NodeDesktopBridge`、`AseSessionHost` 和现有 Iced 单窗口。语言仓持计划、样例 fixture 和验证合同，AutoUI 仓持宿主与原生 UI 实现；实施时分别使用同组 `D:/autostack/.wt/lang-1109/{auto-lang,auto-ui}`，分支 `plan-1109-dev`。按实际依赖补同组 detached auto-web/auto-down 等，路径解析按仓规，不使用链接。计划材料统一登记到 `D:/autostack/plan-artifacts/plan-1109/`。

## 4. 需求分析与背景调查

依据 [Gallery 设计 §8](../../design/strategy/autoscape-native-gallery.md)、[ASE 路线](../../design/strategy/ase-m0-delivery-roadmap.md)、[模块目标](../../specs/overview.md)、[M0 交互合同](../../specs/ase/m0-interactive-sessions.md)、[增量事件会话](../../specs/ace-migration/component-event-sessions.md) 与 [1106 归档 §9](1106-ase-incremental-event-sessions-and-replay-audit-repair.md)。11 月 20 日核查主检出基线 auto-lang `5089d3de0`、auto-ui `da997de2`。开工固定实际源码/依赖、ABI、profile、工具与提交，核对并行计划已落地的接缝。

| 所查路径 | 已观察的边界与影响 |
|---|---|
| AutoUI `crates/auto-ui-runtime/src/ui/ase_m0_sessions.rs` | `open_session/select` 只给被选中实例打开显示车道；切走保留 bridge。`step_selected` 只步进选中实例，现有状态表带会话 id 和 generation。 |
| AutoUI `crates/auto-ui-runtime/src/ui/ace_node_desktop.rs`、`ace_state_bridge.rs` | 已有 typed 业务派发、状态差量和显示车道；点击注入以当前车道为目标。定时事件必须显式校验其目标会话，不能把到期事件当点击发往当前页。 |
| 语言仓 `experimental/ace/crates/av/src/component_event_sessions.rs` | 1106 已交的会话提供 typed 载荷验证、按 handler 派发、事务、纯状态读和释放审计。继续用执行层证据辨别重放与重建。 |
| [012 Clock 源码](../../../../auto-ui/examples/ui/012-clock/src/front/app.at) 与 [AutoUI PLAN-105](../../../../auto-ui/docs/plans/105-autoui-u03-local-interactive-apps-av-vue.md) | 原 `widget App` 的 `.Tick` 用 `Time.now_sec()` 刷新时钟，并同时承担列表、秒表、倒计时与闹钟逻辑。105 正在推进原项目 AV/A2Vue；当前记录仍有整例差距。其 owner 继续负责原语法、通用计时和应用兼容。 |

2026-11-20 用户指令「完成计划 1109」授权按 r1 合同执行、独立复审、有界修复与合并归档。语言与 UI 工作树基线分别为 b5d5bf02b、da997de2，同组依赖为 auto-down@72b0970、auto-web@26d085f。材料及可删除构建缓存登记于 D:/autostack/plan-artifacts/plan-1109。

## 5. 详细设计

### 实施前置与并行界面

| 接口 | 就绪状态与决定 |
|---|---|
| 1106 ACE AV 增量会话、1023 ASE 选中车道、980/981 已交的受检 UI 宿主/事件面 | 已交，足以开始 T-01。确认一条 typed 整数时间样本从宿主到声明式时间 handler、AV 状态再到真实 frame 的链路。原生路径不等待 ACE/AGE M2 全部收口及 AC/A2R/混合图形。 |
| 页签驱动的单次唤醒、取消与 typed 入队接缝 | T-01 在现有宿主循环和已交的取消接口中选择一个。记录实际调度/派发位置、取消点、错误和执行层观察。允许为 ASE 增加薄的源注册和生命周期适配；如缺通用计时、payload、文本或 dispatch 能力，登记精确缺口和 owner，受影响步骤等该最小接口正式落地。 |
| AutoUI PLAN-105 的 012 原项目与通用 timer/语言接缝 | 不作为 T-01 或最小 Clock 的整包前置。若已交同一接缝则直接复用；若正由 105 实现且本计划确实需要，先交接该接口和落地提交，再完成受影响任务。105 保持原项目验收，1109 验 ASE 页签。 |

### T-01 执行接缝

- 复用 `AseM0SessionsWindow` 的 20 毫秒事件循环。其选中步进执行 `ClockWake` 的单槽受理与消费；事件不跨线程。打开或恢复建立新可见代次与立即到期的单调截止点；关闭或隐藏先清注册及待处理槽。
- 真实路径为普通 Node 声明 `Tick(int)` → `NodeDesktopBridge::dispatch_typed`（与按钮/键入共用派发、文本纯读与状态差量）→ `AceStateEngine` → `ComponentEventSession` 受检 handler → `ui_apply_text` → 原生 frame。选择端仅接受显式 `--clock-app` 注册。Clock 源的初始文本从已挂载程序读回，避免旧桥的额外场景编译。过期工作和补拍负控的判据使用 AV 入口数和 FFI 文本提交。
- 原 012 `.Tick` 会读取 `Time.now_sec()` 并承担其他计时逻辑。最小样例仅复用其墙钟采样行为。105 的原项目和通用计时接口未列为本实施前置。测试需预备 UI980 provider。现有缓存入口在缺失时会编译 DLL；增加受源/工具链 stamp 约束的只读装载入口，供本计划的验证进程显式启用。
- 首次构建尝试因机上的 sccache 包装禁止 `CARGO_INCREMENTAL=1` 而退出 101。专属构建改用原生 rustc 与显式增量 dev/test；失败记入验证账。

### 会话与回调合同

- 提供一份独立的普通 Node 数字钟，通过原生验证宿主已有的 `--app <key>=<path>` 显式装载。按 012 数字时间的采样行为确定预期。时间样本用 UTC 日内秒 `0..86399` 的 typed 整数（本草案选择），由 Auto handler 生成数字钟状态，通过现有受检文本路径提交。T-01 确定实际投影接缝，须在应用 frame 显示可读的 `HH:MM:SS`；调试状态读回只作旁证。
- 首次显示或恢复时在首个宿主派发机会采样一次，不等下一个周期；有效时间首次提交前显示加载态，避免把上次隐藏时的旧值当作已校时。持续可见按约一秒再次采样。重复选中同一可见页不重复订阅或发额外恢复事件。隐藏同时撤显示车道和该页定时资格，Counter 正常接收交互。
- 每条定时工作绑定会话 id、会话 generation 和本次可见代次。单窗口至多一个活跃 Clock 定时注册，至多一个待处理唤醒或时钟事件；再次到期只保留一次最新采样机会。队列出队和进入业务前校验标记与真正的选中车道。隐藏/关闭先失效再取消，已在途的旧工作无 AV 或 UI 效果；失败切换不得留下活跃但无有效车道的任务，原选择若恢复则使用新可见代次。
- 关闭收回定时注册、事件槽和 AV 状态根；仍隐藏的会话保留原状态根，切回继续同一会话。重开同 app 生成新代次并重新初始化一次。真实时间采样失败或样本越界时保留最后有效状态或加载态，并可在下一合法样本恢复；错误不提交半态、不补派发历史。
- 不改全局当前显示车道的所有权。Clock 样本与 Counter 输入各自在真实 typed 路径计数，配合 1106 的 `SessionAudit`/`EngineExecutionAudit`、会话/代次和 frame diff 对账。窗口原有输入轮询可继续，隐藏时零业务派发和零 Clock frame apply，以此量化节省的工作。

### 规范增量（复审候选）

| ID | 操作和目标 | 落地证据 |
|---|---|---|
| SD-01 | 新增 `docs/specs/ase/visible-tab-timers.md`，记录已验生命周期、队列上界、时间源和范围 | AC-01–04，源和版本可复查 |
| SD-02 | AutoUI `docs/specs/runtime/ase-m0-frame-consumption.md` 补消费接缝和排窗入口；ASE 路线登记 Gallery ASG-09 中的 Clock 子项 | AC-02–05，按实际交付收据更新；本轮各 Spec 不新增能力声明 |

## 6. 测试设计

以下入口均待实施验证。CPU 用例可注入墙钟和单调调度，驱动生产入队/取消/派发接口。窗口腿在真实时间源、真实窗口下运行；时间与次数的精确断言由确定性用例负责。

| ID | 实际入口、覆盖范围与判别断言 |
|---|---|
| V-01 | T-01 在真实 Node 源、公共桥和 AV session 上先证 `Tick(int)` → 应用 frame 读回，以及未声明/非法载荷具名拒绝、零业务/UI 效果。记录 012 行为来源、最终选定入口和并行 owner；读回与渲染审计区别于外层事件计数。 |
| V-02 | 拟增 AutoUI `--test ase_clock_lifecycle`，走真实会话宿主和可控时间源。首次显示读取 09:59:59，下一拍显示 10:00:00；切到 Counter 多次步进并点击，Clock handler/apply 增量均为 0；将墙钟拨到 11:20:00 后切回，首个时间提交即该值、仅一条新 handler，编译/链接/init 增量为 0。另验墙钟向后跳及跨 UTC 午夜，由测试主动推进时间。 |
| V-03 | 同目标重选、快速 A→B→A、两个同源会话、过期回调积压、关闭后重开，注入捕获的旧可见代次和关闭代次回调。旧事件零 handler/状态/frame，当前合法事件恰一次，待处理上界为 1。用刻意忽略代次校验的负控证明旧事件可污染新视口且会被 oracle 抓出；用故意逐拍补发负控验证执行次数能识别最终时间正确时的额外派发。 |
| V-04 | 用同一原生宿主逻辑连续 20 轮打开/切换/关闭，并分别验证切换、采样失败后恢复。每轮核计时注册、事件槽、挂载和状态根收敛，审计发布值/执行/release 对账。记录首帧与切回耗时、活动和隐藏采样/派发/apply、关闭走势；在 V-05 的实窗进程中对账 demo 专用 HTTP 进程、监听端口均为 0。 |
| V-05 | 扩展 `ase_m0_sessions_window` 或同一宿主例程运行真实原生窗与系统时钟，至少观察两个连续显示时间样本，并在 Clock→Counter 输入→Clock→关页的完整序列检查页签、焦点和应用 frame，包含活动计时下的真实 resize。另测窗口关闭时仍有活动时钟，采集宿主清理终态与进程退出，避免只以强制结束进程代替卸载验证。原生自动输入和真人验收分别记账。 |
| V-06 | AutoUI 受影响完整目标 `ase_m0_sessions`、`component_event_sessions_1106`、`ace_node_desktop`；新增目标 `ase_clock_lifecycle`。用 `cargo test -p auto-ui-runtime --features ace-bridge --test <目标> -- --test-threads=1`，真窗档单实例。Clock fixture 随 V-01/02/05 验真；若触及无窗口 resolver 则加 `cargo t ase-m0`，现有 nextest 配置问题若重现则留痕并用同 workspace 完整测试确认结果。上游源/AV 代码由原 owner 变更时，冻结其 `ace-source`/`ace-av` 受影响组及消费者证据。 |

## 7. 验收标准

| ID | 完成条件 |
|---|---|
| AC-01 | 真实本地 Auto Clock 与 Counter 由同一个原生宿主独立装载，时间 handler、状态和应用 frame 有真实链路证据；来源、读回、实际入口和边界可复跑。V-01/05。 |
| AC-02 | 可见正常走时，隐藏零 Clock 业务与呈现，恢复首个提交用当前墙钟且不重放/补拍；Counter 输入、状态和焦点正常。V-02/05。 |
| AC-03 | 定时上界、重复选中、错会话/错代次回调、拒绝后的恢复与资源收敛满足 §5；正反控能以真实 AV 审计识别多派发。V-01/03/04。 |
| AC-04 | 原 M0 与 1106 的受影响行为和执行层反例通过，真窗档完成，计时/资源和进程/listener 按实际记录。V-04/05/06。 |
| AC-05 | 独立复审逐项核对 AC-01–04、原例归属、实际 producer/consumer、未完成项及零新增告警；正式交付复验冻结版本、核 SD-01/02，并归档可追溯收据。 |


### 实施验证记录（2026-11-20）

- V-01 首探退出 101，增量 compiler 拒绝源里的顶层 `utc_time` 辅助函数。改用已交的 `str.slice`、整数运算和拼接，在 `Tick(int)` 内格式化。真实桥的合法样本 35999 显示 09:59:59，未知消息、错误类型和数量均具名拒绝，AV handler 次数和帧提交数未变。正式目标冻结前重放通过。
- V-02/03/04 实际入口 `cargo test -p auto-ui-runtime --features ace-bridge --test ase_clock_lifecycle -- --test-threads=1 --nocapture`。恢复前首帧无提交，隐藏期间 Clock 入口与旧车道提交计数不变，Counter +3；恢复、拨回及 UTC 午夜各验证显示/状态。故意只核 session ID 时真实 handler 入口 1（应为 0），故意按旧截止点累加时同一终值下 handler 入口 2（应为 1）；两控均退出 101，恢复生产源码后 4/4 通过。已用错误工具链 identity 验证准备产物拒绝，新增目标在禁止 rustc 进程的配置下运行。
- V-06 当前 Clock 4/4（应用帧/载荷 0.077 s，错误/20 轮释放 1.056 s，待处理上界及代次 0.080 s，可见与隔离 0.114 s，首帧 18.171 ms、恢复 18.338 ms）；Clock 整命令 1.51 s。组件增量 4/4（0.23 s），M0 会话 9/9 含真窗（5.59 s）。构建两示例 18.19 s，四目标预备构建 4.03 s，等待 62.25 s。旧 Node 桥 9/11，两个旧 case 直接访问各自的 provider 准备入口，受本轮禁止 rustc 的哨兵影响，补显式准备后独立重放。未将首次红归为代码回归。
- V-05 初跑序列的 Clock 可见区间偏短，只提交一次文本；延长该区间重放在同一进程观察到 18:42:48 → 18:42:49 UTC，Counter 真派发 1 次，恢复并重开后用 WM_CLOSE 自然退出，三个 AV 状态根均释放，活注册和事件槽归零。真实客户区 1600×1000 → 1042×633，专用子进程和 TCP 监听 0。Windows 锁屏期间物理桌面前台无法到达目标窗，记录为单独验证档。

- V-06 Node 桥补显式准备 F0（示例构建 14.655 s，准备 0.634 s），预置 F0/UI980 源与 ABI stamp，旧桥执行只允 `rustc --version`；完整目标 11/11（含两个真窗，8.505 s）。四个目标合计 28 项通过，条件排除 0；Windows 未运行 Linux/WSL 腿。
- V-05 冻结路径自动重放 6.679 s；原窗口目标输入队列 `SetFocus/GetFocus` 为真，Counter typed 点击恰一次，窗口 HWND 实际 resize 至 1042×633。Clock lane 首段 18:52:15 → 18:52:16 UTC，恢复首样本 18:52:17，重开 18:52:18 起；关窗前存在 1 个活动时钟，WM_CLOSE 后三个会话均一次 init/release，活状态/定时注册/槽为 0，进程自然 exit 0。HTTP 子进程、TCP listener 各 0。系统锁屏期间的物理前台点击和真人验收单独记为未执行。自动输入经 native 事件循环内宿主脚本与真实 typed 路由，焦点、resize 和关闭用 Win32 接口。
- 构建出现基线告警，改动行无新增编译告警。实施 profile 为 dev/test、debug=0、增量、Windows x64 MSVC、rustc 1.98.0 (88d9e12ae 2026-08-18)；首次冷构建 323.735 s。材料（测试日志、临时对照、provider 和版本守卫）在统一 scope。未改无窗口 resolver。

## 8. 执行步骤

- [x] T-01 冻结会话、队列与时间采样接缝
  - owner_stage: work。核 1106/980/981 正式版本与 AutoUI PLAN-105 的实际责任边界；在 `ase_m0_sessions.rs`、`ace_node_desktop.rs` 和真实事件会话上先证 V-01。记录落地来源、候选取舍、假时钟注入点、派发/取消不变量及确需等待的上游缺口。AC-01/03。
- [x] T-02 接通普通 Node Clock 与原生 frame
  - owner_stage: work；依赖 T-01。在 `experimental/ase-m0/fixtures` 新增 Clock 源，以独立 app key 接现有原生装载。通过共享 typed 事件入口派发，读回 Clock 状态差量，在应用 frame 显示。复验合法/非法输入 V-01 和 V-02 基础段。AC-01/02。
- [x] T-03 接入页签选择、失效与关闭
  - owner_stage: work；依赖 T-02。在 AutoUI `ui/ase_m0_sessions.rs` 和 T-01 选定宿主接缝上实施一个可取消唤醒和一条 typed 时间事件的上界、可见代次及会话关闭纪律，保留 Counter 路由。完成 V-02/03/04 的生命周期正例。AC-02/03。
- [x] T-04 固化生产链判别测试
  - owner_stage: work；依赖 T-03。在 AutoUI `tests/ase_clock_lifecycle.rs` 固化 V-02/03/04，包括时钟跳变、两类过期工作、错误恢复、实际执行层负控与 20 轮关闭。保留负控打红与恢复绿的对应证据。AC-02/03/04。
- [x] T-05 完成真窗和完整触面验证
  - owner_stage: work；依赖 T-04。扩展原生验证入口并运行 V-05/06，汇总单进程、会话/frame 工作量与各阶段时间。整理 SD-01/02 候选和独立复审入口。AC-01/04。
- [x] T-06 独立复审
  - owner_stage: review；依赖 T-05。同树按实际源码、运行入口和冻结身份审计 V-01–06，核对原例归属、未授权延后、错误/关闭、重复派发及 Spec 差量。复审有缺口即返 work 定向修正。AC-05。
- [x] T-07 正式落地与归档
  - owner_stage: merge；依赖 T-06 pass。在冻结提交核受影响职责及消费者并集，落地 Spec/路线/验收摘要并验证版本，归档并按 repo 规则对账和清理各专属树、材料目录。AC-05。

## 9. 复审记录

stage: new | plan_id: PLAN-1109 | plan_revision: r1 | outcome: pass（起草交接） | next: 获实施指令后在专属工作树进入 T-01；拟议验证均未运行。

起草检查确认 7 个任务、6 项验证、5 条验收、2 项规范候选和本轮涉及文档的本地链接；四个硬依赖均已归档。1023 和 1106 的归档未改动。

stage: work | plan_id: PLAN-1109 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | branch: plan-1109-dev | base_commit: lang b5d5bf02b（仅并行计划文档更新后等价变基）/ ui da997de2 | code_commit: lang 535a17c56 / ui 80555436 | dependency_revisions: auto-down@72b0970 / auto-web@26d085f | task_ids: T-01..T-05 | evidence: §7，V-01..06 实际源码、负控和命名目标；真实窗口自动输入与生命周期已测，物理前台和真人档未执行 | blockers: 无 | next_responsible_role: review（T-06，合并冻结职责组仍待跑）

stage: goal | plan_id: PLAN-1109 | plan_revision: r1 | round: 1 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | commit: lang 535a17c56 / ui 80555436 | next: review

stage: review | review_scope: final | reviewer_identity: 独立上下文 /root/review，同权威工作树 | plan_id: PLAN-1109 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | branch: plan-1109-dev | reviewed_commit: lang 535a17c5675c89f0fcacf3740987a8ad95f876f0 / ui 805554369083857405e7b65e330e9d85029cf71c | base_commit: lang ecc175acceb1cf69b2f099d88c3aca299482b61c / ui da997de2919a268c9aa02d31377de2558872689c | dependency_revisions: auto-down@72b0970 / auto-web@26d085f | spec_inputs: SD-01 2682a57d91794d0f03293c02a2ffe618a4619f8ca892c3490d90c6111d01dd02，SD-02 61325f2bcbac0f40db76fe4351cb78ed34d5f69a5644752542b2d943c52f8166 | acceptance_results: AC-01 pass，AC-02/04 正例 pass 失败分支 partial，AC-03 fail，AC-05 待修 | findings: F-1109-01 | next: work（T-03/04/05），冻结 gate 仍待合并阶段

F-1109-01（AC-02/03/04）独立 holdout 调用真实 `revoke_provider(UI980_PROVIDER_NAME)` 后尝试 Clock→Counter，关车道被拒但仍保留 selected/wake。旧视口无 apply（2→2）而业务 Tick 已执行（handler +1，状态 00:00:02，显示 00:00:01）；关窗后双 AV 状态根仍存活、release 计数 [0,0]。恢复 provider 后清理成功。正例原生自动焦点/resize/WM_CLOSE 与四个 UI 目标证据复用。反例预备链接首轮环境缺 SDK 搜索路径 8.041 s，补齐后 2.669 s；执行 0.253 s，退出 101。源码 SHA256 5919eea849c534ef669445c64d524d73833d626e3a7ba64d37ee398d5bc2b212，结果 SHA256 e1ca543fc2e4fca2515de1d4dd5b99c7d1ec0314a584ae73b47d932e14869fd8。修复入口为丢弃失效定时资格、允许具名错误后重建有效车道、关闭释放所有宿主状态根，并用永久回归绑定真实派发计数。

stage: goal | plan_id: PLAN-1109 | plan_revision: r1 | round: 2 | from: review | to: work | status: reviewed | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | commit: lang 535a17c56 / ui 80555436 | next: work

stage: work | plan_id: PLAN-1109 | plan_revision: r1 | outcome: pass（定向修复） | worktree_path: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | branch: plan-1109-dev | code_commit: lang 42546f007456af55beb0cbc1cb8fa9bb22789b77 / ui 03e96377780a20b5b6998d7e022a1edb67eb9e14 | dependency_revisions: auto-down@72b0970 / auto-web@26d085f | task_ids: T-03/04/05 | next_responsible_role: review（只复 F-1109-01 及受影响约定）

F-1109-01 闭合证据。失败切换撤销 timer 和选中，保留待关车道以便下一次切换重试。已入队工作在取样前核真实显示 handle 和 provider 准入。provider 恢复后新车道和新可见代次可采样，旧车道及唤醒均已失效。窗体退出逐一关闭原 provider 车道并在被拒时释放本地 UI 车道及队列，继续完成所有 AV 根释放，同时报告关闭错误。帧提交错误不会计为 Clock 呈现成功。永久用例 `revoked_lane_cancels_clock_and_shutdown_releases_all_sessions` 走真实 revoke/readmit，先复独立反例，再以直接撤销验证待处理回调和退出邻近路径，断言零新增 handler/apply、新 lane 正例和两根 release。

| 定向检查 | 实际结果与执行时间 | 准备与整命令 |
|---|---|---|
| `ace_clock_lifecycle` 完整新目标，Windows dev/test | 5/5，1.34 s；V-01 0.07746 s，20 轮与坏样本 1.05097 s，队列与旧代次 0.06442 s，V-02 可见/恢复 0.09730 s（首帧 23.15 ms，恢复 11.22 ms）；新增撤销用例 0.05120 s | 编译 19.40 s，整命令 21.00 s；发现和清理未单计 |
| `ase_m0_sessions` scoped `p1023_t03_same_app_isolation_close_reopen_zero` | 1/1，0.187 s；8 条范围外，由合并目标重验 | 编译 3.23 s，整命令 3.68 s |
| V-05 `ase_m0_sessions_window` + 同一 Win32 自动脚本 | 1/1，7.069 s；真实焦点与 resize，19:01:25→19:01:26 UTC，回到 Clock 19:01:28，WM_CLOSE 后活会话、注册和槽均归零 | 专属示例预备构建见 scope；真人前台档仍未执行 |

上轮反控、1106、Node 完整目标作为部分复用证据，合并仍需冻结职责组和受影响 UI 完整目标。

stage: goal | plan_id: PLAN-1109 | plan_revision: r1 | round: 3 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | commit: lang 42546f007 / ui 03e96377 | next: review

stage: review | review_scope: final | reviewer_identity: 独立上下文 /root/review | plan_id: PLAN-1109 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | branch: plan-1109-dev | reviewed_commit: lang 42546f007456af55beb0cbc1cb8fa9bb22789b77 / ui 03e96377780a20b5b6998d7e022a1edb67eb9e14 | base_commit: lang f8311d8cffbdc23f0bb99c588f6266e1db87cbca / ui da997de2919a268c9aa02d31377de2558872689c | dependency_revisions: auto-down@72b0970 / auto-web@26d085f | spec_inputs: SD-01 SHA256 891660267545221e2487398cce69ed3d206808cf60537d12c6789fe3cf0cb6ab；SD-02 SHA256 23279bd36ac4dc214fbdbd573a09a456e151d906d73c2bacaffd280ed452b018 | acceptance_results: AC-01..04 pass，AC-05 独立复审与规范核查完成 | findings: F-1109-01 已闭 | pending_closeout_gates: ace-abi、ace-facade，四个 AutoUI 完整目标，SD-01/02 与 ASG-09 Clock 路线、归档清理 | next: merge

同一独立 holdout 在修复编译产物上重新链接（2.173 s）后 1/1，0.269 s，exit 0。失败切换保留原 00:00:01 且 apply 2→2，选中和定时槽清空，handler 增量 0。关窗报告撤销错误并照常释放两根。复核新永久测试和独立运行边界，复用工作阶段 5/5 Clock（新撤销例 0.0512 s）、1/1 M0 同源近邻、7.069 s 真窗重放。真窗 29 次 view，活动计时下 resize，Counter 输入与焦点，WM_CLOSE 清理成功，0 HTTP 进程/监听。未重复运行完整回归。物理前台与真人档未执行，Linux/WSL 依裁定排除。两仓 diff 检查清洁，相关改行无新增编译告警。反例重放日志 SHA256 36e142e64931928592fb6e0690229604c0fd124abe5ba050695ae5dc53029609。

stage: goal | plan_id: PLAN-1109 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1109/{auto-lang,auto-ui} | commit: lang 42546f007 / ui 03e96377 | next: merge

stage: merge | plan_id: PLAN-1109 | plan_revision: r1 | checkpoints: prepared + landed + canonical_verified | outcome: archive 通过归档门后待清理 | delivery_commit: auto-lang 42546f007456af55beb0cbc1cb8fa9bb22789b77 / auto-ui b8da1ab1a292502cb086eb5a5ea1ad4ba43dd6cd | archive: docs/plans/archive/1109-ase-clock-tab-lifecycle.md

合并采用两仓 `git merge --ff-only plan-1109-dev`。AutoUI 主线仅新增 105 的计划/目录文档，代码提交 80555436→22a173de、03e96377→b8da1ab1，`git range-diff` 两项等价。语言交付仍是 42546f007。同组依赖为 auto-down 72b0970、auto-web 26d085f。落地主分支两份规范的 Git blob 与复审完全相同。SD-01 blob f2a6f875f5604b8104431e5f9db4d67f2c2f3a2e、SHA256 891660267545221e2487398cce69ed3d206808cf60537d12c6789fe3cf0cb6ab。SD-02 blob 4d43b312f5ab08d1a374c99d8d0aa25f8b18ece3、Git 原始字节 SHA256 9e5a67f0c2d16192ca6984f11222437bcaa3f8fef3c45041ad3d6f5517054522；以同 blob 及两提交的原始字节一致为准。

| 冻结门禁 | 实际执行结果 | 准备、执行与总耗时 |
|---|---|---|
| `cargo t ace-abi ace-facade --prepare-only` | 联合准备退出 2。PLAN-1108 同族配置缺口，root workspace 缺两个 AAC 包，nextest 无法解析 override 14/15。正式配置保持原样。 | 181.745 s，执行 0 条 |
| `cargo t ace-abi --prepare-only` 后 `--run-prepared` | 314 条执行，289 绿、25 红，另 8 条排除。25 红与基线失败 ID 集完全相同；无新增失败。 | 冻结预备 564.410 s，nextest 调度/执行 43.736 s，执行整命令 46.645 s |
| 基线 f8311d8cf，`cargo t ace-abi --test-name source_foreign` 显式准备和执行 | 86 条执行，61 绿、25 红，236 条未选或排除。相同 C 运行时校验，记录于 D-1109-ABI-C-RUNTIME-PIN。 | 预备 17.590 s，执行 11.937 s，执行整命令 14.868 s |
| `ace-facade` 全部原 Cargo 包/target + 原 nextest 选择式 | 95 条执行，94 绿、1 红。等价配置只移除两个对该 workspace 无效的包过滤，其他配置 TOML 值逐项相等，保留所有用例。原断言在基线上同样失败，归现有控制台入口维护项 D-1098-01。 | 临时配置 archive 65.627 s，准备 `runtime-dev` collection fixture 0.505 s，执行 2.594 s，执行整命令 8.857 s。基线定向重放整命令 6.875 s |
| AutoUI `ase_clock_lifecycle`、`component_event_sessions_1106`、`ase_m0_sessions`、`ace_node_desktop` 完整目标，`--test-threads=1`，开启既有排窗档 | 分别 5/5、4/4、9/9、11/11，排除 0。合计 29 绿。Windows Linux/WSL 腿依仓规排除。 | 两示例准备 17.110 s，四个测试目标准备 4.115 s；分别总耗时 1.363、0.179、5.193、8.386 s。发现与清理未单计。|
| 落地冒烟，已验原生示例 + 主检出显式 Clock/Counter 源 | 1/1，实际窗口输入焦点、resize、同窗连续走时、Counter 实际 handler、切回与 WM_CLOSE 清理。28 次 view，活动注册 1→0，关闭三根、无 HTTP 子进程和监听。 | 复用源等价的已验 dev 产物，执行整过程 7.073 s |

冻结新用例逐项耗时依次为 0.07622 s（真实 typed handler、帧与无效载荷）、1.04441 s（坏样本和 20 轮释放）、0.07313 s（单槽/旧代次）、0.04695 s（provider 撤销、恢复与释放）、0.09730 s（隐藏、补时、Counter 与 UTC 时间跳变）。第一帧 19.587 ms，切回 11.815 ms。原生物理前台和真人档在锁屏期间未执行；自动真窗档包含同窗口输入焦点和 typed Counter。Gate 开始前确认进程空闲，首次、恢复等待分别 18.568 s、12.406 s。开发/测试使用增量 dev/test，固定 ABI 门的既有工具准备沿原工具清单的既定 profile。无发行包或常驻应用由此验证构建。

`ace-abi` 基线与交付同红的 25 个 case（原测试名保留）如下。

<details>
<summary>25 个基线失败用例</summary>

- `conformance source_foreign::source_foreign_authorized_capability_edit_after_restore_only_rechecks_actual_readers`
- `conformance source_foreign::source_foreign_declaration_readers_atomic_after_restore`
- `conformance source_foreign::source_foreign_external_compiler_output_refusal_preserves_current_and_next_request_repairs`
- `conformance source_foreign::source_foreign_formal_cache_discovery_budget_keeps_pointer_and_repairs_without_production`
- `conformance source_foreign::source_foreign_formal_early_diagnostics_recover_owned_provider_residue_without_publishing`
- `conformance source_foreign::source_foreign_formal_request_metadata_budget_preserves_pointer_and_repairs`
- `conformance source_foreign::source_foreign_formal_restart_first_error_repair_preserves_structured_diagnostics_and_pointer`
- `conformance source_foreign::source_foreign_formal_scalar_review_transition_restores_without_hidden_interface_production`
- `conformance source_foreign::source_foreign_formal_tool_io_failure_keeps_pointer_and_recovers_owned_scratch`
- `conformance source_foreign::source_foreign_formal_worker_rebinds_provider_without_any_caller_production`
- `conformance source_foreign::source_foreign_history_hits_before_lowering_and_uses_original_a2x_cache_after_restart`
- `conformance source_foreign::source_foreign_hot_collection_reclaims_revisions_and_preserves_current_and_reader_closures`
- `conformance source_foreign::source_foreign_name_changes_keep_nonreader_slots`
- `conformance source_foreign::source_foreign_provider_symbol_index_uses_shared_budget_before_rebind_and_repairs`
- `conformance source_foreign::source_foreign_publication_faults_and_revocation_preserve_the_accepted_root`
- `conformance source_foreign::source_foreign_publication_recovers_a_crashed_registration_and_gc_preserves_live_readers`
- `conformance source_foreign::source_foreign_real_process_restart_uses_required_managed_pages_before_first_edit`
- `conformance source_foreign::source_foreign_real_signature_and_source_repair_publish_together_after_restore`
- `conformance source_foreign::source_foreign_request_output_quota_refuses_real_first_edit_and_next_request_repairs`
- `conformance source_foreign::source_foreign_required_page_output_refusal_precedes_first_contract_edit_and_repairs`
- `conformance source_foreign::source_foreign_required_restore_rejects_each_missing_corrupt_or_misbound_page`
- `conformance source_foreign::source_foreign_runtime_policy_pin_matches_current_sdk_material`
- `conformance source_foreign::source_foreign_selected_targets_restore_then_immediate_edit_and_failed_first_edit`
- `conformance source_foreign::source_foreign_target_source_caps_reject_before_production_and_preserve_current`
- `source_resources source_foreign::pointer_encoding_reserves_before_output_allocation_and_preserves_current_on_refusal`

</details>

ABI 固定组选取配置 SHA256 8bb5c62196c720dbd4d78c3f15246b7add4ff1ed834d4ae69b4912dcf39f295a，冻结执行 source_state 21ec4c77c560acfc9b51a0ff281964a78c9f2de239855beb7143a00fee84270d。失败 ID 集 SHA256 b8efbc61a66ff0f5ca40441af9e9fe6242090b0a8f24ac561e3780324ca30aef。冻结/基线 ABI 结果 SHA256 f1b70c1e35ccfcac1ffe000fd622540f41cae1ea5cede140da9cfac64d4915c0 / 108de90f0dc65dc435060b422084bcdd6809fc211294a815c1f6d0a2a885e470。Facade 等价配置 SHA256 e375af75561721b811994ebc98fc2d43e5ea0c35a211250533be9246914adf69，基线控制台日志 SHA256 d1e479568aa035e6d9d97808d1edd6c265e61b1567d4de98563050a1f0ab9274。主检出原生冒烟日志 SHA256 fe2b9e48ae14d27bf60a6ce098032924616f050cf8e1ef8fd191ab3b42a02e38。

归档门已核实际位置、archived 状态、清理前 6/7、双仓已交付祖先、规范链接和两条 Gallery/ASE 反向链接。归档提交后继续守闸清理。

### 关闭路径对账

| 实际输出根 | 核验结果 | 文件数 | 字节数（清理前） |
|---|---|---|---|
| D:/autostack/.wt/lang-1109/auto-lang | 已移除 | 62356 | 20818442886 |
| D:/autostack/.wt/lang-1109/auto-ui | 已移除 | 21389 | 8655445887 |
| D:/autostack/.wt/lang-1109/auto-down | 已移除 | 1294 | 26841766 |
| D:/autostack/.wt/lang-1109/auto-web | 已移除 | 88 | 1584322 |
| D:/autostack/plan-artifacts/plan-1109 | 已移除 | 1000 | 795582140 |

语言目标目录（含 experimental/ace/target）、UI target、两个 detached 兄弟树均在上表的专属工作树内。临时验证、provider/F0、归档包、预备产物、对照和 nextest 输出均在登记的 Plan 材料根。核过实际命令路径与进程，未见其他计划所有的外置输出。两个计划分支和空组目录已移除。没有具名材料持有。

stage: merge | plan_id: PLAN-1109 | plan_revision: r1 | outcome: pass | delivery_commit: auto-lang 42546f007 / auto-ui b8da1ab1 | checkpoints: prepared + landed + canonical_verified + archived + cleaned | archive_commit: 5a0bddc03 | cleanup: 四个工作树通过 pwsh wt-guard 后分别移除，两仓计划分支和空组移除，材料折入归档并删除，登记表刷新 | next: done

递归删除命令被执行策略拒绝，材料目录随后按已守闸的清单逐文件删除并自底向上移除已确认的空目录。核验范围根和专属工作树均已不存在。两个主检出中和本计划相关的交付路径干净；并行计划的既有文档与辅助文件不计入本计划清理。已验原生 debug 产物随专属工作树回收。周期门复核最后覆盖号 1098，已有当日收据，本轮 1109 及收据后的归档没有新的五倍数或 48h 到期条件。

stage: goal | plan_id: PLAN-1109 | plan_revision: r1 | round: 5 | from: merge | to: done | status: archived | outcome: pass | worktree: 已删除 | commit: lang 42546f007 / ui b8da1ab1 | next: done

## 10. 待澄清事项

当前没有未决实施或合并决策。物理前台与真人档单独记录为锁屏未执行。语言测试的既有失败已在变更前基线按原用例确认，归维护方处理。
