---
plan_id: PLAN-1112
status: archived
completion_kind: delivered
feature_name: ASE 原生 Todo 页签与本地持久化
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-23
plan_revision: 2
current_step: 7
total_steps: 8
depends_on: [PLAN-1111, PLAN-1109, PLAN-973, PLAN-975, PLAN-1064, auto-ui/PLAN-105]
supersedes_spec_components: [docs/specs/ase/local-service-binding.md]
new_spec_components: [docs/specs/ase/native-todo-persistence.md]
touched_goals: [GOAL-010]
affects: [ase, ace/source, ace/av, age/autoui-runtime]
---

# [PLAN-1112] ASE 原生 Todo 页签与本地持久化

## 0. 变更摘要

消费 [PLAN-1111](1111-ase-local-todo-service.md) 的本地绑定，在 AutoScape 原生验证宿主运行 Gallery 原 013 Todo。Windows 真窗完整业务和独立进程重启复验通过，成本对照与消费者回归结果已记入 §9。两仓已合入并验证规范源，归档后清理计划目录，当前 7/8。

## 1. 目标

- 在 Windows x64 的一个原生窗口打开 013 Todo 与另一个已有 demo。Todo 的原 Store、列表与编辑输入经当前 ACE/AGE AV 正常运行，完成新增、编辑、切完成状态、筛选、删除和清除已完成项。
- 原生验证入口显式开启数据保存。关页只释放该会话和服务租约，最后关闭释放后端和存储资源；重开应用及退出宿主再启动仍读到最后成功提交的数据，同应用的两个页签各自保留输入/筛选草稿。
- 以同一 013 源版本和等价业务脚本与原 Gallery 路径对照，量化是否省掉 demo 服务、监听和重复启动，分开记录新增耐久写入的成本。
- 范围为受信本地 Todo、两个存活应用会话、单宿主进程。完整 Gallery 默认入口、Notes、混合图形、跨进程并发写、云同步、通用沙箱或 checkpoint、HTTP 兼容迁移由各自后继负责。

## 2. 架构方案

`Todo 原前端/Store → 受检服务操作 → 1111 本地绑定与租约 → 原 Todo Auto 业务 → ACE 文件/存储能力 → typed 完成 → 同一会话的状态更新 → 原生 frame`。

AutoUI 保有真实工程装载、Store 和呈现所有权。ASE 管理 tab、租约、可见性、完成代次与数据根授予。业务和 durable 提交走受检 Auto 源和 ACE 资源 owner。既有 HTTP 与默认内存模式继续消费同一业务合同；使用不同传输和显式存储配置。

## 3. 技术栈

复用 `AseSessionHost`、1109 的选中车道与窗口入口。AutoUI 改动面为原生宿主、`ace_package_bridge` 的受检包桥、原 013 前后端的最小可配置适配。AutoLang 改动面为 `experimental/ase-m0`、AGE 包绑定与 widget runtime 及原 013 所需的 AV 事件消费。沿用 `D:/autostack/.wt/lang-1112/{auto-lang,auto-ui}` 的 `plan-1112-dev`，兄弟依赖只读。AutoOS Gallery 作基线。材料在 `D:/autostack/plan-artifacts/plan-1112/`。

## 4. 需求分析与背景调查

本次用户要求规划后继计划，授权本草案及路线登记。基线 auto-lang `5a76167cc`、auto-ui `448ed614`、auto-web `26d085f`、auto-os `efcfced`，2026-11-20 读取。开工换为真实冻结版本与原始用例报告。依 [ASE 路线](../../design/strategy/ase-m0-delivery-roadmap.md)、[Gallery 从 G2 到 G3 的合同](../../design/strategy/autoscape-native-gallery.md)、[模块总览](../../specs/overview.md)。

2026-11-23，用户以 `/goal 完成计划1112` 授权执行、独立复审、合同内修复、合入与清理。授权沿用 r1 的范围和验收。

2026-11-23，用户明确授权 r2 在 AutoLang 和 AutoUI 补齐原 013 的跨单元 TodoStore 与 1111 本地服务结果的原生回传，并继续真窗验收、成本对照、复审和合并。原项目、同源样本量、运行模式和验收条件沿用 r1。

| 正式来源 | 对 1112 的意义 |
|---|---|
| [013 工程](../../../../auto-ui/examples/ui/013-todo/pac.at)、[TodoStore](../../../../auto-ui/examples/ui/013-todo/src/front/todo_store.at)、[013 后端](../../../../auto-ui/examples/ui/013-todo/src/back/db.at) | 原 Store 导入后台 API，列表修改和编辑在 Auto 里；原后端只有内存种子。持久化是 ASE 实验入口新增的显式模式，由真实后端写入。 |
| [原生页签定时规范](../../specs/ase/visible-tab-timers.md)、[1106 会话](../../specs/ace-migration/component-event-sessions.md)、[AutoUI 宿主](../../specs/ace-migration/m2-autoui.md) | 现有选中车道、会话代次与受检事件/执行审计可复用。1109 验的是单 Node Clock，原 013 所需的 Store、跨包调用和列表/记录完成链仍需专门证据。 |
| [AutoUI PLAN-105 归档](../../../../auto-ui/docs/plans/archived/105-autoui-u03-local-interactive-apps-av-vue.md) | 013 原生 AV 装载与 `ToggleAll` 在交付树复验通过。原收据保留文本输入及带载荷操作的边界。T-01 核当前宿主支持，T-03 仍须验证 013 所需完整操作和本地服务回传。 |
| [973/1064 IO 与存储](../../specs/ace-migration/m2-stdlib-io.md)、[现有 Gallery](../../../../auto-os/ui-gallery/README.md)、[1020 历史基线](1020-ase-m0-baseline-and-interface-readiness.md) | 已交 AV 真实 Win32 文件端口、原子替换与 ACE-KV 合同；尚需在 Todo 的实际执行 profile 和生命周期内证明。1020 是历史成本与拓扑参考，实测需跑当前版本。 |

## 5. 详细设计

### 就绪与执行顺序

| 接口/责任 | 使用条件 |
|---|---|
| [1111 本地绑定规范](../../specs/ase/local-service-binding.md) | 已交付。注册 app://auto.demo.todo/ 与 todo 1.0.0，通过 ServiceBinding、Lease、Ticket 消费原 013。按代次和请求号路由完成，超时可能已提交时刷新。T-02/T-03 复用这份版本及 Auto 业务。 |
| AutoUI PLAN-105 的 013 AV 切片 | 105 的装载与 `ToggleAll` 收据用作兼容对照。r2 由 T-03 补齐共享 Store 与本地回传，完成后复验编辑输入和动态列表的原操作。 |
| 973/1064 文件与持久能力、[975 数据能力](../../specs/ace-migration/m2-stdlib-data.md)、1109 车道 | 已交。T-01 核授予 API、配额、存储路径及真实原生挂载。1111 就绪后 T-02 在实际 Todo AV profile 先证持久化，无需等 105 前端。装配缺口具名交原 owner。 |

### 数据与生命周期

- T-03 从 `bind_packages_age_mode`、`inline_store_decl` 与 `WidgetSession` 核对 Store 声明、包身份、受检模型及事件提交。原 013 的 App 和 TodoList 共享同一页签下的一个 TodoStore，TodoList 的筛选及编辑输入保留在相应会话；缺失、错型或歧义绑定具名拒绝。以原 013 和 Counter 正控跑实际包准入，再通过原 App 写、TodoList 读和第二页签的独立草稿区别重复 Store 副本与真正共享。
- T-03 在 AutoUI 受检包桥复用 `WidgetSession` 的 typed host input 机制。通过实际 1111 `Lease` 提交，`Ticket` 的完成以应用会话、租约、请求号、运行代次核对后送受检完成 handler。原 013 的操作和返回类型须依次穿过实际业务与受检帧提交；受控慢完成、关闭重开、带实参事件和坏完成正反控计入 V-01/V-03。宿主持有 Lease/Ticket，AV 运行原 Store/View 业务和完成处理。
- 沿 1111 的绑定继承有限队列和独立执行 owner。服务完成带会话/请求/运行代次，在交给会话前检查身份。隐藏时可让有效本地提交在服务侧结束，只给选中车道提交帧；切回刷新已提交状态，不重投写。关页后的完成不会改重开的会话或被误投给另一个实例。双页签同 Realm 读共享提交，输入草稿和筛选仍属 UI 会话。
- 宿主按已解析的可信 Realm 给预置的独立 demo 数据根，限定同一宿主串行写。T-02 探针证实 `std.storage` 在受检 AV 的请求预算内无法稳定读回较大的有效快照，采用 973/1064 的 `std.file.replace_atomic` 作为唯一提交点。宿主读取或编码一份版本化 UTF-8 JSON（列表和 `nextid` 同时提交），限 256 KiB、256 行、单行文本 4096 字节。宿主独占创建同目录暂存文件并同步数据，再由受检 Auto 入口通过限定源和目标路径的 Win32 能力做替换。只有授权目录内缺少快照时才加载原种子。坏文件、不可读、版本不兼容、超限与未授权保留原数据，并各自返回错误。原 HTTP API 与默认内存仍使用原种子和 CRUD。
- 数据改动在服务内按顺序计算，经现有原子存储成功后才允许发布成功结果与新状态。失败时旧文件和旧提交可重读，下一次显式修复/重试继续读受检业务。成功已落盘但应答被取消时，重开从提交态恢复。保留 1111 运行代次与提交观测，避免将资源生命周期冒充跨进程 exactly-once。
- 同业务比较至少包含冷宿主首次打开、同一宿主首次 Todo 首个业务结果、切回、最后关闭；另外单列耐久模式进程重开和写入成本。记录该 demo 的真实进程、监听、编译/link/init、首个可交互帧、首个业务结果、RSS 与资源终态。测前冻结输入/动作、样本数、机器和工具/缓存配置。比较用默认内存模式与原 Gallery 等价操作，耐久模式单列；原路径若提供不了同语义操作，就标缺项并报告实际可比行，不从别的 demo 推导倍数。

### 规范增量

| ID | 变化 | 依据 |
|---|---|---|
| SD-01 | 新增 `docs/specs/ase/native-todo-persistence.md`，记录原应用入口、绑定、数据归属、提交/错误与重启；更新 `docs/specs/ase/local-service-binding.md` 的可选 SetAll 与模式合同；同步路线中 ASG-05/06 对这个 Todo 切片的记录 | AC-01–04。保留其他原型能力与整库验收边界。 |
| SD-02 | AutoUI `docs/specs/runtime/ase-m0-frame-consumption.md` 追加原生包消费/typed 完成的已验事实，由原 owner 复审沉淀 | AC-01/03。105 持有原应用和引擎通用能力验收。 |
| SD-03 | AutoLang `docs/specs/ace-migration/m2-autoui.md` 追加实际交付的共享 Store 绑定、受检原生消费与作用域事实 | AC-01/03，覆盖 T-03 交付的软件接缝。 |

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

- [x] T-01 冻结可比行为与上游消费接缝
  - owner_stage: work。已冻结 013/Gallery 源及操作、可比样本条件与存储 profile。已编译对照 Counter 和原 013 包，验证当前 AGE 包无法装载 013 跨单元 Store，T-03 待上游。V-01，AC-01。见 §9。
- [x] T-02 接入 Realm 数据根与真实存储
  - owner_stage: work；依赖 T-01 存储条目及 1111 正式绑定。保留 1111 的默认内存路径，完成显式耐久模式和 V-02 的真实进程验证。AC-02 的原生窗口部分留 T-04。见 §9。
- [x] T-03 接通 013 原生包与本地完成
  - owner_stage: work；消费 T-01/T-02 和 1111。先用实际包入口复现并补齐跨单元 Store，确认 App/TodoList 同一会话共享及第二页签草稿隔离，再连带载荷输入、1111 typed 回传和代次隔离。V-01/03，AC-01/03。
- [x] T-04 实测真窗与完整重开链
  - owner_stage: work；依赖 T-02/03。在通常启动入口验证操作、关闭、重开与持久结果，长期负控挂真实路径。V-03/04，AC-01–03。
- [x] T-05 同业务对照与受影响组验证
  - owner_stage: work；依赖 T-04。完成 V-05/06，记录可比行及异常，整理数据、服务、窗口证据。
- [x] T-06 冻结交接与规范候选
  - owner_stage: work；依赖 T-05。逐条对账 AC-01–04 与 SD-01/02，明确其它 demo、完整 G2/G3 和默认接线的边界。
- [x] T-07 独立复审
  - owner_stage: review；依赖 T-06。查原工程消费、磁盘/执行器终态、拒绝/恢复与可比口径，复验关键正反控。AC-05。
- [ ] T-08 正式落地
  - owner_stage: merge；依赖 T-07 pass。核冻结组和正式规范、归档、清理及路线链接。AC-06。

## 9. 复审记录

stage: new | plan_id: PLAN-1112 | plan_revision: r1 | outcome: pass（规划交接） | next: 获实施指令后冻结 013 原行为；真实产品接线等 1111 与 AutoUI 105 所用 AV 切片按 §5 落地。

起草检查通过。8 个任务、6 项验证、6 条验收、2 项规范候选均有责任阶段，存储和前端各自的进入条件已对账。

stage: work | plan_id: PLAN-1112 | plan_revision: r1 | outcome: blocked | worktree_path: 待创建 | branch: plan-1112-dev（拟） | base_commit: 预检 AutoLang `dca52e196`、AutoUI `1fd87b97` | code_commit: 无 | task_ids: T-01 开工前 | evidence: `git status --short --untracked-files=all -- .wt`、`git worktree list --porcelain`、`git diff --no-index` | blockers: 主检出有 1095 未交接的测试文件副本，见 §10 | next_responsible_role: 用户或文件所有者

stage: goal | plan_id: PLAN-1112 | plan_revision: r1 | round: 1 | from: work | to: user | status: drafting | outcome: blocked | worktree: 待创建 | commit: 无 | next: user | authorization: 2026-11-23 `/goal 完成计划1112`，覆盖执行、独立复审、合同内修复与合入清理

2026-11-23，复核 `git status --porcelain=v1 --untracked-files=all -- crates experimental scripts tests examples` 和 1095 副本差异。实际正式源码路径无 WIP，副本差异仅为标题与末尾换行，位于未注册的独立目录。维持原文件，撤回开工阻塞结论。专用组 `D:/autostack/.wt/lang-1112/` 已核对全路径和 HEAD：AutoLang `dca52e196a64f123ca057e1442c71c8f3e95ab30`、AutoUI `1fd87b970e2463d63595f0efec175a652547d709` 分支均为 `plan-1112-dev`，只读兄弟 AutoDown `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`、AutoWeb `0cfc7c78446650631d244a9ea442d28635072958` 为 detached。材料登记在 `D:/autostack/plan-artifacts/.retention/policy.json`，scope `plan-1112`。

T-01 基线：AutoUI 013 目录树 `2dc06351304bba29c1a7644f4129c88ce85a9bce`。AutoOS `cdb4883812229576b1f21e6124cd900bfad7c630` 的 Gallery 树 `978315e2d5fb05b682ede7e51d91858fccf1ce49`。原 Gallery 的 VM 内嵌 app、TodoStore、列表和后端各有版本差异。只有在原 Gallery 拓扑下装载冻结 AutoUI 013 才能记同源耗时。原 Gallery 自带的非同源行单独报告。可比脚本冻结原四条种子、新增、编辑取消与完成、筛选、删除与清除、Counter 切回、最后关闭，以及持久模式的重启和 ID，冷/暖各至少五次。脚本和输入清单 `D:/autostack/plan-artifacts/plan-1112/t01-baseline.md` SHA-256 `fa5ed1905255a34883e253cc667840bda4961ebcf4845256692cdee57a031a74`。

T-01 正式 AGE 包入口核查：以和 `NodeDesktopBridge::load_package` 相同的 `compile_age_package_with_scenario` 准入调用原 013 App、TodoList、TodoStore 和其闭包，Counter 正控通过。013 返回 `source.age.component-unsupported`，span 4488..4500，缺少 `store` 字段并要求在同一包单元声明 Store。工具 `todo-package.exe` 源于本计划暂存区，输出 `todo-package-probe.log` SHA-256 `6c9b264605775ed2dffc8586c00001eefa3ce43d4d616457cbdaacb68bc985cb`。105 的 `P105_MEM_BACKEND` 用 Store 内的替代 CRUD 改写原调用，归档收据覆盖装载和 ToggleAll。1112 必须经 1111 的真实操作与 typed 完成，接线余项见 §10。

T-02 实现检查点为 AutoLang `f4fb6ac68951073eff8d0bf34a9adf88f0a33722` 和 AutoUI `0a6f8ac7dbcc20f560ca1b2e9fa777d4b87e0016`。显式授予已存在的绝对私有目录，禁止数据文件 reparse point，不同 Realm 无法借用绑定，同一宿主内另一活动绑定无法占用同一根。首次激活时读取快照，按检查过的两个内部 Auto 入口恢复行和 ID；真实 CRUD 在现有 AV 事务中执行，磁盘替换完成后才发布。暂存创建、写入、同步或替换失败时回滚候选状态，保留旧文件。最后租约 join 后释放活根。默认内存绑定不启用存储。

存储方案实证：`std.storage` 正控能经 Win32 写回 Unicode/NUL，`storage-probe-relink.log` SHA-256 `b1d8f1ec93bc5d5d253ac0c305116a5624d1f8b65c709c1527bad1332494cb33`。约 57 KiB 的读回相继命中指令、槽和时间上限，约 7 KiB 的正控在 1800 万步内仍失败（`storage-expansion-8k.log` SHA-256 `6b33a48a8536df329c56def75ba4ba7a81725ec509ddcb77f1e554272463d70f`）。部署选定 §5 的 checked `std.file.replace_atomic`，保留 1111 请求时限和业务限额。用 64 行、约 255 KiB 真读写正控与超过 256 KiB 的扩容反例验证限额和回滚。

Windows debug/test 定向验证：`cargo test --locked --offline -j 3 --manifest-path experimental/ase-m0/Cargo.toml --target-dir <scope>/storage-probe-target --test local_todo_service --no-run` 准备 4.32 秒，11 条各以同一 exe 的 `--exact` 串行执行，11/11 通过，外部计时合计 14.636 秒。冻结提交复跑同命令的 `--lib --test local_todo_service -- --test-threads=1`，6/6 单元与 11/11 集成通过，准备 13.90 秒、测试分别 1.16 与 25.03 秒、整命令 41.166 秒，发现和清理未单列。Linux/WSL 排除。仍有上游 ACE 既有告警，改动范围无新增。单例日志 `durable-json-cases.log` SHA-256 `605306efd91e20c6cb98f177f5d5c2a27a2503873d56d10916a7bfd67b37dc0b`。冻结日志 `durable-json-commit-run.log` SHA-256 `bd9d447756a10b05730a806eeafe7db95dfd92b00dc9b069c76435c37ed84648`。

| 新用例 | 实际判别 | 含准备与清理 |
|---|---|---:|
| `durable_snapshot_reopens_with_exact_rows_and_next_id` | 双租约共享，异 Realm 和同目录竞争拒绝。Unicode 编辑、完成、删除、清除、重开与后续 ID | 2.469 s |
| `durable_snapshot_failed_replace_preserves_disk_rows_and_next_id` | 持有源文件句柄拒绝真实替换，旧字节和行保留，释放后下一个 ID 正确 | 1.290 s |
| `durable_snapshot_rejects_corruption_and_versions_then_recovers` | 损坏、版本、超限各自拒绝，坏文件字节原样，明确清理后恢复种子 | 1.447 s |
| `durable_snapshot_expansion_refusal_leaves_a_readable_commit` | 约 255 KiB 正控可提交，越界扩容保留该提交 | 1.320 s |
| `durable_snapshot_survives_process_exit_and_detects_a_memory_only_write` | 写进程退出再读，独立根内存写入负控被第二进程识别 | 4.036 s |

stage: work | plan_id: PLAN-1112 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1112/{auto-lang,auto-ui} | branch: plan-1112-dev | base_commit: AutoLang `dca52e196` / AutoUI `1fd87b97` | code_commit: AutoLang `f4fb6ac68` / AutoUI `0a6f8ac7` | dependency_revisions: AutoDown `456c86ac` / AutoWeb `0cfc7c78` | task_ids: T-01/T-02 完成，T-03 等上游 | evidence: 冻结提交 17/17，原包准入反例及 Counter 正控 | blockers: AGE 包跨单元 Store 与受检本地回传宿主接缝，见 §10 | next_responsible_role: ACE/AGE 包面与 AutoUI 原生桥 owner

stage: goal | plan_id: PLAN-1112 | plan_revision: r1 | round: 2 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1112 | commit: AutoLang `f4fb6ac68` / AutoUI `0a6f8ac7` | next: user

stage: goal | plan_id: PLAN-1112 | plan_revision: r1 | round: 3 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1112 | commit: AutoLang `f4fb6ac68` / AutoUI `0a6f8ac7` | next: user

第 3 轮核查：AutoLang 主线仍为 `dca52e196`。AutoUI 主线已到 `ee222ac0`，由 1112 基面以来的增量仅为 PLAN-117 文档。AutoUI PLAN-119 的跨单元 MinesweeperStore 仍依赖共享实现交付。两份 1112 检查点工作树干净，§10 暂无授权调整或上游交付版本，T-03 继续等待。

stage: goal | plan_id: PLAN-1112 | plan_revision: r1 | round: 4 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1112 | commit: AutoLang `f4fb6ac68` / AutoUI `0a6f8ac7` | next: user

第 4 轮核查：两仓主线与检查点身份未变，§10 无上游交付版本或范围决定。T-03 的同一前置已连续三轮阻断，目标进入 blocked，计划仍为 executing 2/8。上游交付后重验 013 原包，或经用户授权后修订实施范围。

stage: new | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | changes: T-03 承接 ACE/AGE 包 Store 与 AutoUI 原生回传，SD-03 记录共享接缝 | retained: T-01/T-02，原 AC-01–06、V-01–06 | code_commit: AutoLang `f4fb6ac68` / AutoUI `0a6f8ac7` | next: work

stage: work | plan_id: PLAN-1112 | plan_revision: r2 | outcome: checkpoint | code_commit: AutoLang `169ff92fa88ed398aeb8e3e8c24ec1eff403e144` / AutoUI `ce2b2146020e977b5124b42a140501cabf7f7263` | task_ids: T-03/T-04 | next: T-05

T-03：从 013 原工程路径加载 App、TodoList、TodoStore、types 四个前端单元。AGE 以同一包的实际声明与导入绑定 Store。受检 AV 以会话根模型统一 Store 状态，保留子组件自己的筛选和各页签的草稿。兼容分支在包的原作用域静态选择，未改写传入源文件。跨单元调用诊断锚到导入位置。服务租约和请求票据归原生挂载 owner，核对 Realm、运行代次、租约及请求身份。成功与不确定提交均以同代次读回对账。三条 AV 定向测试检查父子双向共享、两挂载隔离、错误类型及跨绑定拒绝。原 013 的 checked package 和 widget runtime 均通过，Unicode 受检行经实际 TodoList 视图显示，日志 SHA-256 `abdae2b514396ac4c5b63b90fe1a656177a49e5db359c000b6e5589873e7de01`。

T-03/V-02/V-03：AutoLang 6/6 单元及 12/12 本地服务集成测试通过，分别执行 0.40 s 和 22.22 s（Linux/WSL 排除）。新 ToggleAll 正控核四行 ID 与原文字不变、一次 durable commit、重开无额外写。服务日志 SHA-256 `e70336be5436ff5493cc33da9004c9b2c08a55855e3e3ff407a7e9bd1073130a`。AutoUI `ase_native_todo` 1/1，构建 6.31 s，执行 6.59 s，核真实 Counter 完成事件时 Todo 的业务回传落在隐藏挂载、同 Realm 共享、双页签草稿隔离、编辑取消和保存、删清及最终租约/worker/owner 均归零。首次运行因验收根据已提交写入和仍显示的编辑输入提前继续，修正为等 AV 编辑已退出后通过。日志 SHA-256 `d88e3b71ebadd57b5622036430335a5808bf93c64624a0152a79603444de480f`。

T-04/V-04：实际 Windows 原生窗口通过脚本输入和原生关闭消息验收。脚本向真实受检 renderer 所用带 revision 队列提交键入及按钮动作。单次完整运行 50 帧、13 次点击、27 次受检派发、6 次耐久写，执行新增、编辑取消和保存、勾选、筛选、删除、清除、Counter 切入切回，以及关页后重开。关闭后 `todo.json` rows 为 1、2、5，Unicode `重启保留 🌱` 原样，next_id=6。另一个宿主进程重开同一授予根，追加 ID 6 并保存 next_id=7。两次采样均见零 demo 子进程、零 TCP 监听，关窗终态 leases/workers/owners/queued/active=0。首次 smoke 传入 rustc 路径导致 ui980 准备态身份不匹配，改用 `rustc --version` 的编译器身份后复验通过。完整和重启窗口收据 SHA-256 分别为 `89e57be105246bafb6e3fdb1f9b6f285a65f1a2b56b6858e64f0aad6bdf7b098`、`31dcf0edbcfbaa70212991b29546b827fefdac9f850a2c25c8b99b723815b213`。真人逐项操作未执行，原生自动化已通过。

T-05/V-05：Windows x64，debug/incremental，预构建带来源和编译器身份的 ui980，原生窗单实例。默认内存模式用同一套业务动作跑 5 个新宿主及同一宿主预热后 5 轮切页关闭重开。冷指进程和服务实例冷启动，沿同一 Windows 文件缓存执行。测量采样脚本首次一个冷试次在初读完成前送到新增，后续找不到正确业务行，保留为失败；给首读留足节拍后获得下表样本。

| 阶段 | 冷中位数［范围］ | 暖中位数［范围］ |
|---|---:|---:|
| 启动到首个 Win32 窗口可见 | 948.2 ms［891.9–1028.1］ | 同一窗，适用于冷启动 |
| 013 受检包装载 | 768.3 ms［747.7–772.5］ | 748.4 ms［679.9–897.2］ |
| 安装挂载与提交初帧 | 48.2 ms［47.0–49.9］ | 17.4 ms［16.6–17.8］ |
| 挂载后首个真实列表完成 | 265.7 ms［135.0–906.6］ | 234.0 ms［79.7–519.8］ |
| Counter 切回 Todo | 14.9 ms［14.7–19.8］ | 14.8 ms［14.7–21.8］ |
| 宿主 RSS 峰值 | 249.4 MiB［247.2–259.6］ | 274.0 MiB（暖宿主整段峰值） |
| demo 子进程 / TCP 监听峰值 | 0 / 0 | 0 / 0 |

每轮日志均有八次受检列表完成。各冷宿主和暖组最后一轮核对最终帧仅余种子 1、2。窗口关闭后服务终态归零。ASE 冷暖汇总 SHA-256 `18a0bad57c25bd4f4ffe75004fe45110bc698d802cf28e3c34bde8129cdf1f80`。同样的五笔新增另测从 AddTodo handler 到列表完成，内存 503.3 ms［491.3–737.9］，耐久 503.4 ms［491.8–505.6］（含事件调度、校验和读回，采样无法分辨纯磁盘成本）。两者分别为 0/5 次耐久提交，耐久产物恢复 IDs 0–8、next_id=9。SHA-256 `0340f9cffaace15b17847367615157b8c226bfcd9e50df9a82d383a3fbd758e4`。

Gallery 对照采用 AutoOS `cdb4883` 的原 viewport 和从同一 013 路径导入的源，固定 Windows `auto 0.1.0+plan-890-verified-r7-t19-116-gc879b3e`。Vue 实际浏览器 mounted=0，`natives.ts` 缺 `editorBridge` 且 013 SFC 的 `__autoThemeRoot` 重复，HTTP 500。VM 真窗显示四条种子，3 项未完成，MCP 经实参 SetInput 后走真实 AddTodo submit，业务状态变为一行 ID 0，复现 013 原测试 B12(b) 的模块全局重置。两行都无完成等价业务的有效样本，无 Gallery 性能倍数。Vue/VM 反例日志 SHA-256 分别为 `51ff56a12be26b830d5def7c0772cf584cc135b9ef1aab49fe976a0e5e65f202`、`c02aee5b06711c959448a8d73d3aae1b5d62fd7633e1a807a65e97704db43242`。ASG-05/06 的 Todo 子项具备证据；ASG-10 的证据限于有效异步请求与 Counter 输入并行以及本切片的终态回收。

013 冻结树 `fe4388361fbe5bca06907548b579ec66631ca1f0`。逐文件核对 Gallery 生成输入，manifest、App、TodoList、TodoStore、API 和 DB 与冻结源 SHA-256 一致。types.at 的差异为对照副本尾部多一个空行，声明一致。

T-05/V-06 消费者完成记录：`cargo test --locked --offline -j 2 -p auto-ui-runtime --features ace-bridge --test ase_native_todo --test ase_clock_lifecycle --test ase_m0_sessions --test plan114_widget_packages -- --test-threads=1 --nocapture`，prepared provider，冻结 AutoUI `b8659d282a989d954e1d31746131493520212380`。17.53 s 准备、81.485 s 整命令，发现与清理未独立计时。Clock 5/5（2.13 s）、会话 8 条实际执行和一条原可选 OS resize/focus 排窗排除（2.12 s）、Todo 1/1（6.90 s）、受检包桥 46/46（51.90 s）。日志 SHA-256 `13057bb2e1d71f6fa571c467876b0db22627c91cf0c5878ca1c0b56d0949115f`。AutoLang 冻结受影响固定组 `ace-source ace-av ase-m0` 交给合入档，指定组由编译器 import/lowering、AV model/host-input、ASE binding 的实际改面选取。

stage: work | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1112/{auto-lang,auto-ui} | branch: plan-1112-dev | base_commit: AutoLang `dca52e196a64f123ca057e1442c71c8f3e95ab30` / AutoUI `1fd87b970e2463d63595f0efec175a652547d709` | code_commit: AutoLang `169ff92fa88ed398aeb8e3e8c24ec1eff403e144` / AutoUI `b8659d282a989d954e1d31746131493520212380` | dependency_revisions: AutoDown `456c86ac886f9781e56f6153ce9aa41cd59e5c8e` / AutoWeb `0cfc7c78446650631d244a9ea442d28635072958` | task_ids: T-01–06 | spec_delta: `D:/autostack/plan-artifacts/plan-1112/spec-delta.md` SHA-256 `cda6b701cc175eff6bad256ae542c64e6b344991da41979e44615166e4357324` | pending: 合入档 Windows 固定组，Linux/WSL 按裁定排除 | next: final review

stage: review | review_scope: final | reviewer_identity: 同会话复审，按冻结 diff、源、用例和原验收收据独立重建结论 | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1112/{auto-lang,auto-ui} | branch: plan-1112-dev | reviewed_commit: AutoLang `169ff92fa88ed398aeb8e3e8c24ec1eff403e144` / AutoUI `b8659d282a989d954e1d31746131493520212380` | base_commit/dependency_revisions: 同前条冻结收据，两树复验干净 | spec_inputs: local-service-binding `7acb1ffbf2350eec76bbccd127170daf7f86b32e`；m2-autoui `f0e04bb81c60d4e01c364ccf5bee5f51cb3fe098`；AutoUI frame-consumption `e605a2e265c240c9651d4620ff2158556dc523a6`；规范增量 SHA-256 同工作收据 | pending_closeout_gates: `ace-source ace-av ase-m0`、规范落地、归档和清理 | next: merge

| 验收 | 复审核对 | 结论 |
|---|---|---|
| AC-01 | source binder 导入对应实际 Store，受检 AV 子调用同步到根；原 013 的正常 manifest 路径、原 API/DB、动态编辑与列表、真实窗口版本全部可对照。受检消息的 typed 回传和普通帧事件入口分开，坏载荷与跨绑定反例维持命名拒绝。 | pass |
| AC-02 | 显式存储授予限 Realm，默认内存仍从种子使用。检查 IO 预算、同目录原子发布、AV 事务回滚、真实 Win32 失败及新进程恢复；原生重启核对 IDs 1、2、5、6 与 next_id=7。 | pass |
| AC-03 | 活页签持有租约/票据，切页保留挂载。读回以同代次消费；晚完成只能进入所属挂载。独立草稿、只选中车道显示、Counter 在 Todo 等待结果期间更新、最后清理和端口进程采样均有相应 oracle。 | pass |
| AC-04 | 检查 5 个新进程与同一宿主 5 个暖循环，以及内存/耐久各五笔新增的时间与业务判别。Gallery Vue 无挂载、VM 新增后旧行消失，按合同行为不同单列。 | pass |
| AC-05 | 复验已准备的 durable ToggleAll，test body 2.39 s，整命令 2.565 s。移除 prepared 编译器身份跑新 AutoUI case 故意失败，0.00 s 拒绝并给出准备指示。复核全部 V 项和规范候选，确认未越界改变等价业务阈值。 | pass |
| AC-06 | 固定组、最终落地和清理必须由合入档取得终态。 | pending |

复审重用之前的受检包、AutoUI、真窗、成本和 18 条本地服务验证，由冻结提交、实际源码和 prepared tooling 身份约束。新 AutoUI case 6.90 s，两个新 AV 针在三例定向复跑共 0.11 s，新 durable ToggleAll 单例 2.39 s。旧耐久单例的各自计时已见 T-02 表。事实或环境限制已具名，依现有范围核对无被动延后的 work-owned 验收。已有编译告警及 MSVC 产物消息按原基面保留，两树 diff whitespace 检查和新增 Rust 文件格式检查通过。

stage: merge | plan_id: PLAN-1112 | plan_revision: r2 | outcome: needs_fix | reviewed_commit: AutoLang `169ff92fa` / AutoUI `b8659d28` | next: 同步主线后的兼容性修复及受影响复验

首轮准备在 2 GiB 磁盘门槛 fail-closed，退出 2，墙 168.749 s，清理 1112 自有旧编译缓存后，在无其它活动 Cargo 工具时重试。准备三组共 433.287 s、exit 0。执行 80.246 s、exit 1。ace-source 990 条实跑，985 PASS、5 基线红（12.417 s）及 1 忽略。ace-av 1625 条实跑，1605 PASS、20 红（54.373 s），27 未选；其中 13 条按 PLAN-1092 的已知资源/文本/接口基线，另外 7 条实际因提供者准备漏项 `foreign.prepared` 提前拒绝，下一轮显式准备并复验。ase-m0 44/44（5.999 s）。Linux 目标相关候选按 Windows 裁定排除。

同步期间主线分别到了 AutoLang `674eda6d742a27016772a98fd094d5e45c76417c`（PLAN-1127 新 PySlice、ABI 与测试准备）及 AutoUI `77efdc9ebae9571bd9ddd5d566f5e9570f4e0871`（PLAN-117 源准备）。AutoUI 四提交 range-diff 等价，交付候选 `843271669d90c318e89b4c40050c1add6e1fedb0`；AutoLang 持久化提交等价，AGE span visitor 与 PySlice 形态需一个集成修复。修复提交 `f74077eec1dc2867d79037612288fc4b08319293` 把 PySlice 的基值和边界交同一个位置映射，保留 1127 的语义和 1112 的导入诊断锚点。合同和 r2 不变。重新验证两个当前主线基础上的相关组。

stage: review | review_scope: final | reviewer_identity: 同一会话执行单独的冻结证据复审 | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | branch: plan-1112-dev | reviewed_commit: AutoLang `f74077eec1dc2867d79037612288fc4b08319293` / AutoUI `752806ed33eb6925bbbf78468e9e8df23d2f9513` | base_commit: AutoLang `674eda6d742a27016772a98fd094d5e45c76417c` / AutoUI `77efdc9ebae9571bd9ddd5d566f5e9570f4e0871` | dependency_revisions: AutoDown `456c86ac886f9781e56f6153ce9aa41cd59e5c8e` / AutoWeb `0cfc7c78446650631d244a9ea442d28635072958` | spec_inputs: local-service-binding `7acb1ffbf2350eec76bbccd127170daf7f86b32e`；m2-autoui `f0e04bb81c60d4e01c364ccf5bee5f51cb3fe098`；AutoUI frame-consumption `e605a2e265c240c9651d4620ff2158556dc523a6` | spec_delta SHA-256 `5c533e8592b65006c523ba724c8f5794ecad4898cf08f3511a1248b05dd85e68` | next: merge

复审批定 AC-01–05 均通过。AutoLang 的 PySlice 遍历沿同一诊断跨度映射访问基值与边界，1112 的导入锚点与 1127 的语义均在冻结组复验。AutoUI 复审发现通常启动缺省会在观察器退出条件满足后提早结束，`752806ed` 使带 Todo 工程且未提供脚本时默认等待窗口关闭。最新宿主构建 13.14 s；无 `--script` 和 `--await-window-close` 的真实窗口在 1 s 观察预算后保持打开至约 4.85 s 的 Win32 关闭，终态无错误、资源归零。窗口收据 SHA-256 `bc1d7dd59164d0069c492e8026f661c18cf447ee47c7e22d00903278bf705688`。

最终真窗首次回放在初读待完成时过早发出首笔新增，未满足测试前置，数据和动作断言失败。以干净根并给初读更多节拍后重跑，退出 0，墙 16.311 s，57 帧、13 次点击、27 次派发、6 次耐久写，快照 IDs 1、2、5，next_id=6。独立 Windows 进程退出 0，墙 8.550 s，27 帧、1 次耐久写，保持 `重启保留 🌱` 并以 ID 6 新增，next_id=7。两窗均无观测到的 demo 子进程或 TCP 监听，关闭后租约、worker、owner、排队和活动项归零。两份窗口收据 SHA-256 `bc23ae26fb46723d4dbbda70d18b99438207186b8b9cd254476ffab660471e1b`、`96415fb359703dccd857fd85e2a7900514f6992de9c6429db8e5813d947a21a8`。上轮冻结的同业务成本样本保留其原提交身份；两次集成修复无成本口径改变。真人逐项操作未执行。

AutoUI 用上述四个目标及当前基线的准备态串行实际执行 60 条，全部通过。Clock 5（2.09 s），会话 8（2.09 s），Todo 1（6.53 s），受检包桥 46（48.26 s）。会话的原可选 OS resize/focus 1 条按原条件未调度。冷准备 421.913 s，测试整命令 60.053 s，退出 0，发现与清理未独立计时。日志 SHA-256 `8f01074d41c22bcb4f2e402015adc46bf6a5f9bfab2db76eda39d778e0a4097b`。

合入冻结组：`cargo t ace-source ace-av ase-m0 ace-python --prepare-only --debug-tools --out <scope>/rebased-fixed-prep`，准备 268.499 s，退出 0。显式从同一提交的 ace-python 准备结果接入 ace-av 所需 Python provider，先前 7 条准备失败的用例全部通过。按 Windows 裁定具名排除 7 条 Linux 目标候选和全部 WSL；其余原选择和 ignore 保留。执行 `cargo t ace-source ace-av ase-m0 ace-python --run-prepared <scope>/rebased-fixed-prep/prepared-windows.json --out <scope>/rebased-fixed-run`，整命令 85.148 s，原始退出 1。清理未独立计时。准备清单 SHA-256 `cbc82a4c02e458b627e2238d6e6e34240a097ac060101afd9d78f8bc1cd40991`，结果 SHA-256 `5e204a44e12ae26261aa4c2a259e5f7193902d1c600050fbdb94c7bcebcb1eac`，基线逐项对照 SHA-256 `ddae246d9476ea4ac35f6b30c3f80c01d95b4fa555d8997c676192c37d632524`。

| 固定组 | 实际执行 / 通过 / 失败 | 原条件未执行 | 发现 / 执行墙时 |
|---|---:|---:|---:|
| ace-source | 994 / 989 / 5 | 1 忽略 | 0.516 / 8.737 s |
| ace-av | 1625 / 1612 / 13 | 27 未选 | 1.806 / 54.174 s |
| ase-m0 | 44 / 44 / 0 | 0 | 0.726 / 4.801 s |
| ace-python | 12 / 12 / 0 | 218 未选 | 0.515 / 8.424 s |

18 个失败名及所属组逐项匹配归档 PLAN-1092 与 PLAN-1127 的已知输出，无新增失败。新的 AV 同挂载 Store 与独立挂载隔离、typed 完成/错误隔离、持久 ToggleAll 重开定向用例分别约 0.105、0.075、3.290 s。格式和差异检查通过，原编译告警照实保留。周期批量回归最近收据覆盖 PLAN-1126，时间为 2026-11-23 00:03:39 UTC，本计划未触发该门。

- ace-source 的 `text_record_source`：`flagship_combination_compiles_and_verifies`、`generated_atom_carries_the_combination_surface`、`move_arguments_transfer_whole_roots_not_views`、`named_ctor_source_order_is_the_written_order`、`positional_ctor_and_owning_contexts_promote_literals`。
- ace-av：`layout_rejects_non_record_and_empty`、`use_after_move_of_method_arg_rejected`、`v03_flagship_value_thirty`、`v03_token_microprogram_fifty_seven`、`v10_method_field_borrow_return_zero_alloc`、`v10_second_alloc_failure_rolls_back_partially_built_output`、`av_owned_first_case_whole_path_and_live_zero`、`av_owned_recipe_fixture_face_matches_hand_audit`、`c_data_profile_keeps_its_scalar_entry_boundary`、`v02b_text_leaves_are_pure_reads`、`resource_915_r02_l04_arena_sweep_is_complete`、`resource_915_r02_l04_value_sweep_is_complete`、`resource_915_r02_l05_v05_sweeps`。
- Windows 裁定排除的 7 个含 Linux 目标用例：`vload_1041_in_axis_with_io_runs_on_a2py_both_targets`、`vload_1041_in_axis_selection_negatives`、`p966_ct_statements_and_macro_run_on_av_native_rust_c_py`、`p966_ct_generated_declarations_change_end_results`、`stdlib_assembly_1043_selection_runs_on_av_native_rust_and_c`、`stdlib_assembly_1043_linux_target_ignores_missing_windows_sdk`、`stdlib_assembly_1043_store_roundtrip_keeps_assembly_identity`。

stage: merge | checkpoint: prepared | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | reviewed_commit: AutoLang `f74077eec` / AutoUI `752806ed` | delivery_commit: AutoLang `a882176b442ec8a2fb2730f9f5a2e911a143e9fc` / AutoUI `0add6c56cd751e5b272ecfc725c13c57ac26015c` | spec_delta SHA-256 `5c533e8592b65006c523ba724c8f5794ecad4898cf08f3511a1248b05dd85e68` | windows_regression: 2675 实跑 / 2657 通过 / 18 已知基线失败 / 0 新失败；AutoUI 60/60 | next: landed

stage: merge | checkpoint: landed | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | delivery_commit: AutoLang `a882176b442ec8a2fb2730f9f5a2e911a143e9fc` / AutoUI `0add6c56cd751e5b272ecfc725c13c57ac26015c` | method: 双仓 `git merge --ff-only` | next: canonical_verified

两仓从复审记录的主线基面快进，交付提交对受检源码提交的差异仅为各自规范和路线。AutoLang 主检出的 Gallery 路线另有未提交草案，已按单路径保存、恢复并核对原差异，其他工作区改动未纳入本计划提交。合入后用同一宿主二进制从两仓主检出装载原 013 运行并关窗，退出 0，墙 7.257 s、27 帧、一次真实列表结果，demo 进程、监听与关窗活动资源均为零。收据 SHA-256 `273da9c9b173e0854c64e3c452a82f80f89068af0770ccff16f95d5a9d7e6795`。

stage: merge | checkpoint: canonical_verified | plan_id: PLAN-1112 | plan_revision: r2 | outcome: pass | source_identity: AutoLang native-todo-persistence `9538949e8eda55a00be02c707cf6c0d42e85a8c9`，local-service-binding `84b332dc45b68088d9e144ed6e26b8aaa7dae1df`，m2-autoui `4977ffbd0844709acbba17f9189c462e9a050edd`，Gallery 路线 `af3b52fd67c6f0836246b8a409135cc286719ead`；AutoUI ase-m0-frame-consumption `81670870c0bf1a3bed2c8bf972ebbc275ec144ce` | next: archive

stage: merge | checkpoint: archived | plan_id: PLAN-1112 | plan_revision: r2 | completion_kind: delivered | archive_path: `docs/plans/archive/1112-ase-native-todo-and-persistence.md` | next: guarded cleanup

## 10. 待澄清事项

无必须先由用户裁定的选项。T-01 确认 105 的正式消费点、支持的原输入/视图操作、1111 持久 owner 接缝和确需提交的存储预算。超出签名或涉及通用能力的缺口由对应 owner 给出范围与复验，不扩大或弱化本计划验收。

2026-11-23 开工前置阻塞：主检出的 `.wt/lang-1095/auto-lang/crates/auto-lang/test/cases/language/composition/ordinary-projects.case.ad` 未被跟踪。`git -C .wt/lang-1095/auto-lang rev-parse --show-toplevel` 返回 `D:/autostack/auto-lang`，工作树登记无 1095。副本 Git blob 为 `4344fb60e6c627ad23bacf12a1bff70bfd833c58`，与当前主线的 `8c56c9fc7ba0f55db3be77ac4bafb929ca713ba8` 有标题及末尾换行差异。请文件所有者将其交接至正确工作树或移出主检出，或由用户明确授权移出保存。该路径从主检出状态中消失后即可继续创建 1112 工作树并开展 T-01。

同日复核解除上述开工阻塞，依据及工作树见 §9。

T-03 的上游阻塞已复现。当前正式 AGE 包准入拒绝 013 的 App/TodoList 使用跨单元 `TodoStore`。AutoUI 105 收据只覆盖其内存后端改写入口，输入和载荷行为仍有具名边界。按 §5 的 owner 约定，需要 ACE/AGE 包面和 AutoUI 原生桥交付并复验原 013 的共享 Store、带实参输入/列表事件及 1111 本地结果的消费接缝。若由 1112 承接这些生产者通用能力，需要用户授权修订实施范围。两仓检查点保持在专用分支，后续真窗、成本对照、复审与合并仍未执行。

r2 已获用户授权，r1 的实施范围阻塞解除。T-03 准入反例作为待修输入保留。
