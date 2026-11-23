---
plan_id: PLAN-1112
status: drafting
feature_name: ASE 原生 Todo 页签与本地持久化
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: [PLAN-1111, PLAN-1109, PLAN-973, PLAN-975, PLAN-1064, auto-ui/PLAN-105]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/native-todo-persistence.md]
touched_goals: [GOAL-010]
affects: [ase, age/autoui-runtime]
---

# [PLAN-1112] ASE 原生 Todo 页签与本地持久化

## 0. 变更摘要

消费 [PLAN-1111](archive/1111-ase-local-todo-service.md) 的本地绑定，把 Gallery 的真实 013 Todo 项目接进 AutoScape 原生验证宿主。验前后台经受检执行、关闭和重开后的数据、同业务对照成本。当前 0/8。

## 1. 目标

- 在 Windows x64 的一个原生窗口打开 013 Todo 与另一个已有 demo。Todo 的原 Store、列表与编辑输入经当前 ACE/AGE AV 正常运行，完成新增、编辑、切完成状态、筛选、删除和清除已完成项。
- 原生验证入口显式开启数据保存。关页只释放该会话和服务租约，最后关闭释放后端和存储资源；重开应用及退出宿主再启动仍读到最后成功提交的数据，同应用的两个页签各自保留输入/筛选草稿。
- 以同一 013 源版本和等价业务脚本与原 Gallery 路径对照，量化是否省掉 demo 服务、监听和重复启动，分开记录新增耐久写入的成本。
- 范围为受信本地 Todo、两个存活应用会话、单宿主进程。完整 Gallery 默认入口、Notes、混合图形、跨进程并发写、云同步、通用沙箱或 checkpoint、HTTP 兼容迁移由各自后继负责。

## 2. 架构方案

`Todo 原前端/Store → 受检服务操作 → 1111 本地绑定与租约 → 原 Todo Auto 业务 → ACE 文件/存储能力 → typed 完成 → 同一会话的状态更新 → 原生 frame`。

AutoUI 保有真实工程装载、Store 和呈现所有权。ASE 管理 tab、租约、可见性、完成代次与数据根授予。业务和 durable 提交走受检 Auto 源和 ACE 资源 owner。既有 HTTP 与默认内存模式继续消费同一业务合同；使用不同传输和显式存储配置。

## 3. 技术栈

复用 `AseSessionHost`、1109 的选中车道/撤销与窗口验证入口，以及 AutoUI/ACE 对真实工程的公开宿主包入口。受影响路径为 AutoUI 的 `crates/auto-ui-runtime/src/ui/{ase_m0_sessions,ace_node_desktop,ace_native}.rs` 中 T-01 确认的消费接缝，以及原 `examples/ui/013-todo/src/{front,back}/` 的最小可配置适配。语言仓 `experimental/ase-m0` 消费 1111 和存储契约。拟工作树 `D:/autostack/.wt/lang-1112/{auto-lang,auto-ui}`、分支 `plan-1112-dev`，使用正式版本的只读依赖闭包。现行 Gallery 在 AutoOS 作基线，实际切换产品入口另在该仓立项。材料统一登记到 `D:/autostack/plan-artifacts/plan-1112/`。

## 4. 需求分析与背景调查

本次用户要求规划后继计划，授权本草案及路线登记。基线 auto-lang `5a76167cc`、auto-ui `448ed614`、auto-web `26d085f`、auto-os `efcfced`，2026-11-20 读取。开工换为真实冻结版本与原始用例报告。依 [ASE 路线](../design/strategy/ase-m0-delivery-roadmap.md)、[Gallery 从 G2 到 G3 的合同](../design/strategy/autoscape-native-gallery.md)、[模块总览](../specs/overview.md)。

| 正式来源 | 对 1112 的意义 |
|---|---|
| [013 工程](../../../auto-ui/examples/ui/013-todo/pac.at)、[TodoStore](../../../auto-ui/examples/ui/013-todo/src/front/todo_store.at)、[013 后端](../../../auto-ui/examples/ui/013-todo/src/back/db.at) | 原 Store 导入后台 API，列表修改和编辑在 Auto 里；原后端只有内存种子。持久化是 ASE 实验入口新增的显式模式，由真实后端写入。 |
| [原生页签定时规范](../specs/ase/visible-tab-timers.md)、[1106 会话](../specs/ace-migration/component-event-sessions.md)、[AutoUI 宿主](../specs/ace-migration/m2-autoui.md) | 现有选中车道、会话代次与受检事件/执行审计可复用。1109 验的是单 Node Clock，原 013 所需的 Store、跨包调用和列表/记录完成链仍需专门证据。 |
| [AutoUI PLAN-105](../../../auto-ui/docs/plans/105-autoui-u03-local-interactive-apps-av-vue.md) | 正在推进原 013 的原生 AV 与 A2Vue。已记录原 Store handler 控制流、列表下标/记录读等缺口工作，105 负责共享语言与原示例兼容。 |
| [973/1064 IO 与存储](../specs/ace-migration/m2-stdlib-io.md)、[现有 Gallery](../../../auto-os/ui-gallery/README.md)、[1020 历史基线](archive/1020-ase-m0-baseline-and-interface-readiness.md) | 已交 AV 真实 Win32 文件端口、原子替换与 ACE-KV 合同；尚需在 Todo 的实际执行 profile 和生命周期内证明。1020 是历史成本与拓扑参考，实测需跑当前版本。 |

## 5. 详细设计

### 就绪与执行顺序

| 接口/责任 | 使用条件 |
|---|---|
| [1111 本地绑定规范](../specs/ase/local-service-binding.md) | 已交付。注册 app://auto.demo.todo/ 与 todo 1.0.0，通过 ServiceBinding、Lease、Ticket 消费原 013。按代次和请求号路由完成，超时可能已提交时刷新。T-02/T-03 复用这份版本及 Auto 业务。 |
| AutoUI PLAN-105 的 013 AV 切片 | 需要实际交付能装载原 `pac.at` 及闭包、原 Store 与动态列表、编辑输入，以及本地操作/typed 完成可消费的原生宿主接口。只等这些接口的正式来源与复跑记录。105 的整批示例、A2Vue 及完整 ACE/AGE M2 不作为原生接线前置。通用能力或原例源缺口交原 owner；1112 可先冻结源、对照脚本和存储方案。 |
| 973/1064 文件与持久能力、[975 数据能力](../specs/ace-migration/m2-stdlib-data.md)、1109 车道 | 已交。T-01 核授予 API、配额、存储路径及真实原生挂载。1111 就绪后 T-02 在实际 Todo AV profile 先证持久化，无需等 105 前端。装配缺口具名交原 owner。 |

### 数据与生命周期

- 沿 1111 的绑定继承有限队列和独立执行 owner。服务完成带会话/请求/运行代次，在交给会话前检查身份。隐藏时可让有效本地提交在服务侧结束，只给选中车道提交帧；切回刷新已提交状态，不重投写。关页后的完成不会改重开的会话或被误投给另一个实例。双页签同 Realm 读共享提交，输入草稿和筛选仍属 UI 会话。
- 宿主按已解析的可信 Realm 给预置的独立 demo 数据根，限定同一宿主串行写。首选已交 973/1064 `std.storage` 配合 975 已有确定性编码，只提交一份版本化快照（Todo 列表和 `nextid` 一起）。T-01 固定合法 UTF-8 和大小域、默认种子、重拒策略、路径约束以及唯一调用点。真正空缺的根才从种子开始；坏文件、不可读、版本不兼容、超限与未授权有明确错误，保留原数据。原 HTTP 和原例默认内存模式保持原语义。
- 数据改动在服务内按顺序计算，经现有原子存储成功后才允许发布成功结果与新状态。失败时旧文件和旧提交可重读，下一次显式修复/重试继续读受检业务。成功已落盘但应答被取消时，重开从提交态恢复。保留 1111 运行代次与提交观测，避免将资源生命周期冒充跨进程 exactly-once。
- 同业务比较至少包含冷宿主首次打开、同一宿主首次 Todo 首个业务结果、切回、最后关闭；另外单列耐久模式进程重开和写入成本。记录该 demo 的真实进程、监听、编译/link/init、首个可交互帧、首个业务结果、RSS 与资源终态。测前冻结输入/动作、样本数、机器和工具/缓存配置。比较用默认内存模式与原 Gallery 等价操作，耐久模式单列；原路径若提供不了同语义操作，就标缺项并报告实际可比行，不从别的 demo 推导倍数。

### 规范增量

| ID | 变化 | 依据 |
|---|---|---|
| SD-01 | 新增 `docs/specs/ase/native-todo-persistence.md`，记录原应用入口、绑定、数据归属、提交/错误与重启节奏；同步路线中 ASG-05/06 对这个 Todo 切片的记录 | AC-01–04。保留其他原型能力与整库验收边界。 |
| SD-02 | AutoUI `docs/specs/runtime/ase-m0-frame-consumption.md` 追加原生包消费/typed 完成的已验事实，由原 owner 复审沉淀 | AC-01/03。105 持有原应用和引擎通用能力验收。 |

## 6. 测试设计

以下用例和入口均待执行。确定性失败由 provider/队列接缝注入，业务、存储、ACE 与原生 UI 路径使用真实实现。

| ID | 入口与判别断言 |
|---|---|
| V-01 | T-01 冻结原 013 项目、原行为与可比 Gallery 行。核对 1111 包签名、文件授予与刷新路径；T-03 用生产入口走真实 TodoStore → 1111 → Auto 创建/读回 → 原生 frame。证据钉版本和 API 生产者，混合分支、旧 VM 回退、手填列表均不计通过。 |
| V-02 | 拟在 `experimental/ase-m0/tests/todo_persistence.rs` 验真实 Win32 存储，使用两份隔离根，验证 Unicode、新增/编辑、完成与删除、重启后的精确列表和 nextid，隔离提交与默认内存路径。原子发布失败、坏文件与错误版本、错误权限、扩容超限分别具名；用一次故意跳过写入的负控证明第二进程能查出只存缓存。试验数据只用计划独立根。 |
| V-03 | 拟增 AutoUI `--test ase_native_todo`，沿生产宿主双页签、关开与慢完成。验证同 Realm 数据共享、草稿隔离、后端故意延迟时仍可切到 Counter 并完成输入，且 Todo 的旧结果不改当前帧；关一个后另一可用，关窗/最后租约归零。用代次错配、同路径不同授权、关闭前后两个真实提交边界识别隐蔽串态与重复写。 |
| V-04 | 扩展同一原生窗口入口或同 owner 的真实验证例程。实际输入 Todo 文本，走新增、编辑/保存与取消、勾选、筛选、删除和清除，再切页/关闭/重开。采集 AV、已提交存储、frame、进程树与监听账。重新启动正常宿主检查持久记录；原生自动和物理/真人验收分别记，未执行项保持原状态。 |
| V-05 | T-05 在 Windows 对照原 Gallery Vue 和原生/VM 各实际可用等价行与 ASE，核同源、确实挂载、相同动作与结果；各冷/暖至少 5 次有效同条件样本，记录中位数和范围、失败、阶段成本与进程/listener 峰值。耐久写入单列。先以原生实测验证 ASE 的零 demo listener/子进程与隐藏换页行为。 |
| V-06 | 冻结 `cargo t ase-m0`（已知 nextest 装载问题若仍在，保留原错误并跑同 workspace 完整测试）。AutoUI 实跑新增目标及 `ase_clock_lifecycle`、`ase_m0_sessions`、实际改动所及原生包桥的完整测试目标，沿 `cargo test -p auto-ui-runtime --features ace-bridge --test <目标> -- --test-threads=1`，真窗单实例。触及上游源、AV 或 IO 时按实际改面跑 `ace-source`、`ace-av` 或 `ace-stdlib` 相关完整组；原用户路径保持适用回归，Linux/WSL 在 Windows 按仓规排除。 |

## 7. 验收标准

| ID | 责任与关闭条件 |
|---|---|
| AC-01 | work。013 原工程经正式 ACE/AGE AV 运行，原用户操作及受检结果可复查，版本和原示例写面已对账。V-01/04。 |
| AC-02 | work。默认内存路径仍沿原行为，独立数据根的持久模式可在宿主重启后恢复，失败不破坏旧提交，计数/ID 与列表一致。V-02/04。 |
| AC-03 | work。关开、切页与慢完成不串会话或复写，两个会话共享同 Realm 数据且草稿独立，两侧清理终态正确，零 demo listener/子进程。V-03/04/06。 |
| AC-04 | work。同业务成本样本可复跑，耐久模式单列，不可比或未跑具名。实测表按已验证覆盖映射 ASG-05/06/10 的 Todo 子项。V-05/06。 |
| AC-05 | review。独立核 AC-01–04、原项目来源、写入原子性和真实重启、回调隔离、适用范围及告警。 |
| AC-06 | merge。冻结受影响职责和消费者，高效复用有效证据，落地规范、路线、归档和清理。 |

## 8. 执行步骤

有原生任务工具时，开始实施前按以下 ID 登记任务。

- [ ] T-01 冻结可比行为与上游消费接缝
  - owner_stage: work。先冻结原源、旧链路、对照脚本与存储 profile，记录 1111 和 105 两个接口的已交/待交状态；真实长链由 T-03 复核。V-01，AC-01。
- [ ] T-02 接入 Realm 数据根与真实存储
  - owner_stage: work；依赖 T-01 存储条目及 1111 正式绑定。复用 1111 业务、已交 IO 与原子 API，实现显式耐久存储并固化 V-02。AC-02。
- [ ] T-03 接通 013 原生包与本地完成
  - owner_stage: work；依赖 T-01、1111 及 105 的 013 AV 切片；与 T-02 接口对齐后联调。连接真实 Store 到本地绑定和受检完成，复用原生挂载以及已交生命周期。V-01/03，AC-01/03。
- [ ] T-04 实测真窗与完整重开链
  - owner_stage: work；依赖 T-02/03。在通常启动入口验证操作、关闭、重开与持久结果，长期负控挂真实路径。V-03/04，AC-01–03。
- [ ] T-05 同业务对照与受影响组验证
  - owner_stage: work；依赖 T-04。完成 V-05/06，记录可比行及异常，整理数据、服务、窗口证据。
- [ ] T-06 冻结交接与规范候选
  - owner_stage: work；依赖 T-05。逐条对账 AC-01–04 与 SD-01/02，明确其它 demo、完整 G2/G3 和默认接线的边界。
- [ ] T-07 独立复审
  - owner_stage: review；依赖 T-06。查原工程消费、磁盘/执行器终态、拒绝/恢复与可比口径，复验关键正反控。AC-05。
- [ ] T-08 正式落地
  - owner_stage: merge；依赖 T-07 pass。核冻结组和正式规范、归档、清理及路线链接。AC-06。

## 9. 复审记录

stage: new | plan_id: PLAN-1112 | plan_revision: r1 | outcome: pass（规划交接） | next: 获实施指令后冻结 013 原行为；真实产品接线等 1111 与 AutoUI 105 所用 AV 切片按 §5 落地。

起草检查通过。8 个任务、6 项验证、6 条验收、2 项规范候选均有责任阶段，存储和前端各自的进入条件已对账。

## 10. 待澄清事项

无必须先由用户裁定的选项。T-01 确认 105 的正式消费点、支持的原输入/视图操作、1111 持久 owner 接缝和确需提交的存储预算。超出签名或涉及通用能力的缺口由对应 owner 给出范围与复验，不扩大或弱化本计划验收。
