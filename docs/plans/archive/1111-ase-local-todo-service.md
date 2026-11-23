---
plan_id: PLAN-1111
status: archived
completion_kind: delivered
feature_name: ASE 本地 Todo 服务绑定与租约
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-23
plan_revision: 1
current_step: 6
total_steps: 7
depends_on: [PLAN-1109, PLAN-960, PLAN-968, PLAN-969, PLAN-1082]
supersedes_spec_components: [docs/specs/ace-migration/m2-modules.md]
new_spec_components: [docs/specs/ase/local-service-binding.md]
touched_goals: [GOAL-010]
affects: [ase]
---

# [PLAN-1111] ASE 本地 Todo 服务绑定与租约

## 0. 变更摘要

1109 已归档，Clock 验证了原生页签的定时、可见性与撤销。本计划先在无窗口宿主运行 Gallery 中 **013-todo 的真实 Auto 后端**，验证原来的业务操作能经本地绑定执行，省去每个 demo 的 HTTP 服务成本。原生前端与耐久数据交给 PLAN-1112，实施顺序见 [ASE 路线](../../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

在 Windows x64 的可信本地原型中，经 ACE AV 运行现有 013 Todo 的后台模块，按原签名验列表、新增、改文字/完成状态、删除和清除已完成项。两个同 Realm 的会话共享一个按需服务，另一 Realm 即使同名也有自己的实例。请求有界，通过显式租约关闭，且本地运行零 demo HTTP listener、零 demo 子进程。

此阶段保持 Todo 原有内存数据语义，最后租约释放后再打开走原始种子。范围限 013 声明的本地服务表和宿主观察入口。远程 RPC、任意第三方/不可信应用、磁盘保存、Gallery 默认入口、HTTP proxy 改造及前端迁移由后续或原主责承担。

## 2. 架构方案

提议的路径为 `原 API 类型和公开业务入口 → 受检 ACE 包制品 → 有界服务 owner → ASE ServiceBinding → typed 本地请求/结果`。ASE 负责身份、准入、租约、任务生命周期和完成路由；原 Auto 模块持有 Todo 状态和业务，ACE 负责受检执行，AutoWeb 保有 HTTP 路由与传输责任。生产演示和无窗口本地测试消费同一份业务代码。禁止把本地调用绕成构造 HTTP 文本、RPC 到 loopback、旧 VM 调用或宿主内存假 Todo。

沿 [旧 proxy 生命周期](../../specs/auto-lang/vm/back-proxy.md) 的按需激活、多窗共享与最后 owner 关闭语义，先对账现有 lazy/proxy/独立进程拓扑。本地路径继承已证的生命周期要求；原 HTTP 路径由原 owner 保持并提供可比业务对照。

## 3. 技术栈

语言仓 `experimental/ase-m0/src/{registry,run}.rs` 为现有无窗口解析与 AV 消费，拟在 `experimental/ase-m0` 加局部绑定模块与 `tests/local_todo_service.rs`。ACE 接缝候选为包制品与 AV 公开执行入口、既有宿主队列和可核验的运行限额。013 作者源码归 AutoUI，若需拆开 `#[api]` 路由声明和业务作用域，只在 AutoUI 原 owner 范围处理。拟工作树 `D:/autostack/.wt/lang-1111/{auto-lang,auto-ui}`，分支 `plan-1111-dev`。只读依赖按已交闭包取同组 detached 或既有解析顺序。材料登记 `D:/autostack/plan-artifacts/plan-1111/`。

## 4. 需求分析与背景调查

2026-11-23 用户以 `/goal 完成计划1111` 授权既定 r1 范围内的实施、独立复审、修复、合并、归档及守护清理。权威实施树 `D:/autostack/.wt/lang-1111/auto-lang`，分支 `plan-1111-dev`，基面 `9c0f5e0fc`。只读兄弟源码 auto-ui `4dd9696d`、auto-down `456c86ac`。HTTP 核查 auto-web `0cfc7c78`、AutoMan 检出 `4a30483a`（运行二进制按文件哈希单独标识）。

2026-11-20 核查主检出 auto-lang `5a76167cc`、auto-ui `448ed614`、auto-web `26d085f`、auto-os `efcfced`。本次授权限规划两个后继计划及更新设计路线。代码、运行验证、工作树、提交和归档属于后续阶段。以 [模块总览](../../specs/overview.md)、[Gallery §5–6、8–9](../../design/strategy/autoscape-native-gallery.md) 和 [1109 最终归档](1109-ase-clock-tab-lifecycle.md) 为范围来源。

| 正式来源 | 当前已观察的能力与打开点 |
|---|---|
| [Todo API](../../../../auto-ui/examples/ui/013-todo/src/back/api.at)、[Todo DB](../../../../auto-ui/examples/ui/013-todo/src/back/db.at) | `pub fn`/注解 API 共享 Todo 记录；DB 有模块 `List<Todo>` 与连续 `nextid`、初始 4 条。它是内存服务。真实源码含模块引用、动态列表、记录、`?Todo`；须试原闭包，不能用 1082 集合支持推断整包与跨界参数/结果已通。 |
| [960 包闭包](../../specs/ace-migration/m2-modules.md)、[969 宿主 facade](../../specs/ace-migration/m2-facade.md)、[968 调度](../../specs/ace-migration/m2-scheduler.md)、[1082 集合](1082-av-parity-collections-and-places.md) | 已有受检包编译、AV 及调度基础。E0 是单次执行，960 文档只确认特定入口结果和全局 cell 观察。完整 Todo 的多次受检调用与类型落地由 T-01 先证。 |
| [1106 事件会话](../../specs/ace-migration/component-event-sessions.md)、[983 Web](../../specs/ace-migration/m2-web.md) | 1106 单 Node 的 UI handler 接缝与包面分开，当前对宿主通道有拒绝边界。983 是 HTTP 生产路径的证据。两者不宣称已经交付本地 typed 完整包服务。 |
| [AutoUI PLAN-105 归档](../../../../auto-ui/docs/plans/archived/105-autoui-u03-local-interactive-apps-av-vue.md) | 013 的原生装载及 `ToggleAll` 已有 AV 交付证据。1111 的无窗口后台可独立先证，完整前端操作与本地回传由 1112 验收。 |

## 5. 详细设计

- **T-01 冻结真实接缝。** 钉主责、正式提交、源码哈希、Schema/profile/ABI、现有后端原入口、测试命令和手写的业务预期。优先选公开受检包与可驻留 AV owner，经同一数据根验证 `list → create → list`；另一候选仅限原 owner 已有的 checked-entry/provider 接缝。最多试两个。至少真实出现 `Todo` 列表、创建结果和拒绝错类型。在原包入口、跨模块类型、返回值或受限执行缺口时记录具名诊断、owner 和接入验证；受影响实现等具体缺口交付。
- **身份和准入。** 由已经解析、受信的 app 注册消费服务，键包含 app/Realm、逻辑服务、部署版本；运行代次独立。请求绑定租约、请求号和运行代次，通过静态操作签名和 payload gate。未知操作、错类型、越权、超预算、关闭、超时、业务缺失和服务执行错误可区分。原 `?Todo=None` / `bool=false` 保留业务语义；不向缺失记录自动补写。采用现有受检 typed 载体，必要的内部编码不能绕过参数/返回验证。
- **实例与资源。** 仅登记/打开页签不初始化服务，首个调用才启动；两个并发首请求合并初始化、同代只 link/init 一次，活实例内操作串行修改。一个租约结束只使该会话失去投递资格；最后释放后撤排队请求、收拢在途任务并释放执行根。初版预算为最多两个服务实例、每实例一个在途和最多八个排队，任一边界具名拒绝，前台不等待 IO。T-01 固定实际 AV fuel/期限、支持的载荷/列表边界和可独立观测的 owner。第 N 次合法修改不编译/重建或重放前 N−1 次。
- **失败与提交。** 初始化失败释放并允许下一次显式重试。排队中取消或超限不进入业务。执行已开始时，取消遵实际受控边界，过期结果不得发布到 UI；已成功提交的效果不因关页撤销。对无法确定是否提交的等待超时，调用端刷新真实状态，不自动重投变更。最后释放清理有明确终态，卡住的 owner 不能假报退出。此切片采用有限本地执行，不引入任意阻塞宿主任务。
- **主责边界。** 如需提取路由与业务的共享入口，只允许同源 Auto 业务、最小适配和旧消费者回归；不在 ASE 增设第二套模块编译、HTTP 路由或调度器。上游 ACE 缺口只由原 owner 补齐，先证独立工作继续。

### 规范增量

| ID | 变化 | 依据 |
|---|---|---|
| SD-01 | 新增 `docs/specs/ase/local-service-binding.md`，沉淀适用范围、typed 入口、租约、限额、终态及监听/进程的实证 | AC-01–03。上游引擎合同若有必要扩展，按其 owner 单独沉淀。 |
| SD-02 | 更新 `docs/specs/ace-migration/m2-modules.md`，记录普通 Auto 包的原 013 作者语法、类型依赖与 AV 列表事务接缝 | AC-01–03，旧运行时依赖环仍拒绝。 |

## 6. 测试设计

以下为拟议验证，尚未执行。业务断言以原 seeds、API 与独立脚本预期为准。测试先完成真生产入口，再接失败注入；测试替身只控制调度、失败与时钟。

| ID | 入口与判别断言 |
|---|---|
| V-01 | T-01 冻结原包、入口及实跑证据。在同一 checked owner 读原 4 条、创建包含汉字和 emoji 的 id=4、再次读到 5 条，验坏类型执行次数为 0、未发现 id 保持 `None/false`。换源负控用合法源码变更种子或行为，断言宿主的结果随真实执行变化。 |
| V-02 | 新增 `experimental/ase-m0/tests/local_todo_service.rs`，走原 013 包与绑定，实跑五个原 API 操作，覆盖空列表、已有/缺失 id、UTF-8、更新和清除。冻结现有 Gallery 该 demo 的实际后端命令、拓扑及同语义原 HTTP 结果，分别记录；不把 proxy 整体算作每 tab 单进程。 |
| V-03 | 同 holder 及双租约并发首调仅一次 init；关 A 后 B 可继续；另一 Realm、同名错版本及错代请求不串。用迟到/错代负控验证只比较最终列表会漏报；以 init/真实执行序号检测每调用重建，单独证明运行代内不复算历史。 |
| V-04 | 生产队列用可控门导致启动失败和撤销边界，核零副作用、重试可用、待执上限、执行预算和拒绝后的正确终态。20 轮打开/释放后队列、根、worker、租约归零；捕获真实进程树与 listener 账，本地腿进程、监听各零。 |
| V-05 | 冻结范围跑完整 `cargo t ase-m0`；若 1106 已知 nextest 配置问题仍在，留原错误并跑同 workspace 的 `cargo test --manifest-path experimental/ase-m0/Cargo.toml --locked`。若触及原示例或公开宿主，验原消费者及受影响 `ace-source/ace-av/ace-facade` 对应固定组。Windows 上 Linux/WSL 按仓规排除。 |

## 7. 验收标准

| ID | 责任与关闭条件 |
|---|---|
| AC-01 | work。真实 013 后端经受检链完成五项操作，类型错误和业务缺失分态，原 HTTP 对照可追溯。V-01/02。 |
| AC-02 | work。同 Realm 合并激活、独立 Realm 隔离，最后租约关闭，生产层可辨重建、旧请求及晚完成。V-03/04。 |
| AC-03 | work。活动/队列有界，失败恢复，20 轮资源收敛，零 demo listener/子进程；原触面验证和进程拓扑证据可复查。V-04/05。 |
| AC-04 | review。独立核验 AC-01–03、真实业务来源、资源终态与缺失项；检查永久正反控和新增告警。 |
| AC-05 | merge。冻结触面验证、落地并核验 SD-01；归档及清理。向 1112 提供可消费签名、版本、错误/取消协议和真业务证据。 |

### 实测记录（Windows x64，debug/test）

- V-01：原始 `api.at` 在普通包前门报 `source.unsupported`（命名记录旧拼写），后续探针分辨 `[]Todo`、符号导入和类型依赖。保留 API 的 Todo 类型与五项签名，仅前置 `use db`。普通包区分解析后的类型边和运行时依赖边，命名构造沿现有拥有文本的转换。受检接口为 `auto.source.language.av/v1`、`av-source-packages/1` 和 `ac.runtime.text.v1`，实际 AV 参数位使用 `StrView`。同根读 4 → 创建 id=4 `中😀` → 读 5，错误类型 0 次目标帧，缺失 ID 保持 None/false。改源种子及从同一导入清单加回值边的负控分别改变返回及触发 `source.module-cycle`。
- V-02：真实源码在 AutoUI `06e87993dd6bda844365a1000fe9ab0a8d92f832`。工作树 SHA256 为 API `332ba5437e084bd26a78c3c715ff3c046376d151c73b420ae9a84d12418941c2`、DB `28adc6417f80eebf0b47d395684885788a30eedd2ecabf27799b2fa5a3d4164f`。宿主读取声明的两份原作者源码，受检 AV 由每个代次的线程持有；同源 VM 对照为 `auto service --server vm --dir <013-copy> --back-port <ephemeral>`。Auto CLI SHA256 `eddb2cce50fd8afe08cd26959c2fe52e090eb833b5c8b9a3f27538b28d947fcf`。基线和候选均通过列表 4/5/0、两种 UTF-8 修改、缺失 null/false、已有删除。两者 `DELETE /api/todos/completed` 均因注册序先命中 `/:id` 得到 400。仅 scratch 重排两个 DELETE 声明的正控得到 true，剩 ID=1,2，表明业务可达；原路由行为留 AutoWeb owner。
- Gallery 拓扑：`auto run -r vm` 入口经 AutoUI `run_vm.rs → start_gallery_back_proxy`；`run_vue.rs` 给 013 登记 `src/back/api.at`，`lazy_sessions: true`，一个进程内共享 listener，首请求唤起该 app 的 VM session 线程。`back_proxy.rs` 使用同一 AutoWeb `match_route`。另测的裸 VM HTTP 样本由命令进程自行拥有一个监听，不能按每 tab 折算 Gallery 子进程。最终样本 PID 61196、端口 3988，测试后已回收。
- 原生产消费者：`auto build -r vue --gen-only` 的基线和最终候选均生成 26 文件，生成的 Rust `api.rs`、`types.rs` 与 TS `api.ts` 字节相同（TS SHA256 `596b175a7481b04c00e527bff07ba27fdd473bb92697d1cf4d082ef77da95d25`）。编译并执行原样生成的 Rust DB 和类型，五项业务、UTF-8、缺失、清空测试全过。原已经存在的生成代码 `done` 未使用告警保留。
- V-03/04：两个 Realm 上限、同 holder 两个首调及双租约并发首调、单代链接和初始化各一次。用实际 `ExecTrace.execution_seq` 和计数识别历史重放。队列 8 个待执 + 1 个在途，拒第九待执。无效代次、租约、请求号三个控制无投递。过期/取消的排队 mutation 不入业务；提交后丢失租约或取消的结果撤投，B 可读已提交状态。AV 指令燃料拒绝后显式重试；在两个全局写入后故意 trap，列表与 nextid 回滚。上限为 2 秒请求、500ms AV wall、100 万步、4096 UTF-8 字节和 256 条。
- Windows 进程账：`DETACHED_PROCESS` 启动冻结集成测试的单个 20 轮用例，psutil 按测试 PID 的进程树/各 PID 的 TCP LISTEN 采样。进程 23340，90 次快照，线程峰值 6，demo 子进程 0、TCP listener 0，2.066s；每轮真实 join/根/队列/租约归零，AV 制品目录清空。常规 Windows console 启动的对照发现 OS `conhost.exe`，未计入 demo 子进程。
- V-05：冻结候选使用 `cargo nextest run --offline --locked --build-jobs 2 --manifest-path experimental/ase-m0/Cargo.toml --test-threads 1 --no-fail-fast -E 'test(/todo_service::tests/) or binary(local_todo_service)'`。执行 10/10，无失败，28 个非定向未选。构建 10.930s，测试/清理合计 3.603s，发现未单列，整命令 15.213s。合并阶段须在冻结提交执行完整 `ase-m0 ace-source ace-av ace-facade`，按 Windows 裁定具名排除 Linux/WSL。
- 首次红保留：早期 probe Rust 临时实参报 E0716；未加受检文本能力时根帧拒绝；移动 Todo 类型的候选使旧生成 API 降级，已恢复原类型位置；`DivZero` / `av.resource.steps` 两个探针预期字面已纠正；较早源变体的插值语法不兼容，改用正常记录字段路径。无一次失败作为通过证据。

| 新增用例 | 判别断言 | debug/test 秒 |
|---|---|---:|
| actual_av_fuel_refusal_does_not_publish_and_explicit_retry_works | 真 AV 燃料拒绝、同代重试 | 0.224 |
| last_release_does_not_report_a_paused_worker_as_reaped | 活线程仍在时关闭等待超时，放行后 join | 0.145 |
| queue_refuses_ninth_waiter_and_withdraws_pending_work | 8 个待执，队列撤销、期限、关闭和副作用 | 0.186 |
| stale_and_late_completions_cannot_publish_but_success_stays_committed | 错三元组拒绝，晚取消保留提交 | 0.145 |
| actual_todo_signatures_and_crud | 真 CRUD、空列表、UTF-8 字节边界与类型 | 0.167 |
| changed_source_changes_results_and_bad_signature_releases_for_retry | 缺公开入口、混合模块环和真实改种 | 0.236 |
| full_list_refuses_before_dispatch_and_allows_create_after_delete | 256 条拒新增、删除后恢复 | 0.267 |
| same_realm_shares_other_realm_isolates_and_stale_generation_is_refused | 双租约/同 holder 首调、Realm 隔离与重开 | 0.291 |
| trap_after_global_writes_rolls_back_both_list_and_next_id | 真 Auto trap 发生于两项写入之后，验证回滚 | 0.153 |
| twenty_lifetimes_release_actual_workers_roots_and_store | 20 轮真实工作线程、状态及制品收敛 | 1.787 |

冻结证据在唯一 scope 内，`todo-freeze-nextest.log` SHA256 `41c156df80aa79b833cdc0ebf01f0365648e8ae6fd533553a9e1e7f64615b65d`，`process-observer-freeze.log` `20354b3a44da9b4cd8427a975294323150ab8f40189a270197ab06687d35d29d`，`http-oracle-frozen.log` `30371dc931ad4d7cc090ed7a095b6e2a4eba801587d6302a404d47256339081a`。生成 Rust 准备 5.880s，执行 0.066s；发现未单列。

## 8. 执行步骤

有原生任务工具时，开始实施前按以下 ID 登记任务。

- [x] T-01 先证原 Todo 后端与 typed AV
  - owner_stage: work。对照 013 的 `src/back/{api,db}.at`、ASE `src/run.rs`、960/969 AV 入口和 Gallery 的原 proxy/HTTP 行为。跑 V-01 真实长链，出开工/待接缝结论及配置。AC-01。
- [x] T-02 实现局部服务绑定并对照原业务
  - owner_stage: work；依赖 T-01。实验 workspace 内实现准入、真实编译/消费、同源业务注册与 typed 请求/结果。V-01/02，AC-01。
- [x] T-03 接入按需激活、请求与租约
  - owner_stage: work；依赖 T-02。在所选原 owner 上接可取消任务、限额、释放与晚完成处理。V-03/04 正例，AC-02/03。
- [x] T-04 固化真实正反控与回归
  - owner_stage: work；依赖 T-03。将 V-02–04 的负控和资源账固化到拟增测试入口，记录坏道失败和正常通过。完成 V-05 适用触面，准备集成版本。AC-01–03。
- [x] T-05 整理交接和规范候选
  - owner_stage: work；依赖 T-04。冻结 SD-01、全链证据与 1112 使用方式，未过项具名。AC-01–03。
- [x] T-06 独立复审
  - owner_stage: review；依赖 T-05。依据实际入口独立核证、重放关键负控、检查同源与遗漏。AC-04。
- [ ] T-07 正式落地
  - owner_stage: merge；依赖 T-06 pass。冻结职责/消费者并集，核规范、来源、归档与材料清理，更新路线和 1112 对交付件的引用。AC-05。

## 9. 复审记录

stage: new | plan_id: PLAN-1111 | plan_revision: r1 | outcome: pass（规划交接） | next: 获实施指令后进入 T-01，拟议验证均未执行。

起草检查通过。7 个任务、5 项验证、5 条验收、1 项规范候选均有责任阶段，本地链接和依赖方向已核对。

stage: goal | plan_id: PLAN-1111 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: in-progress | worktree: D:/autostack/.wt/lang-1111/auto-lang | commit: 9c0f5e0fc | next: T-01（用户已授权既定全链）

stage: work | plan_id: PLAN-1111 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1111/{auto-lang,auto-ui} | branch: plan-1111-dev | base_commit: lang 9c0f5e0fc / ui 4dd9696d | code_commit: lang 293d092d687fc454a2369cabec86e41924d92b37 / ui 06e87993dd6bda844365a1000fe9ab0a8d92f832 | dependency_revisions: auto-down 456c86a / auto-web 0cfc7c7 | task_ids: T-01..T-05 | evidence: §7 实测记录 | blockers: 无，冻结全组交合并阶段 | next_responsible_role: review

stage: goal | plan_id: PLAN-1111 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1111/{auto-lang,auto-ui} | commit: lang 293d092d6 / ui 06e87993 | next: review

stage: review | review_scope: final | reviewer_identity: 当前会话内从冻结工件独立重建，限于同会话 | plan_id: PLAN-1111 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1111/{auto-lang,auto-ui} | branch: plan-1111-dev | reviewed_commit: lang 293d092d687fc454a2369cabec86e41924d92b37 / ui 06e87993dd6bda844365a1000fe9ab0a8d92f832 | base: lang 9c0f5e0fc / ui 4dd9696d | dependencies: auto-down 456c86a / auto-web 0cfc7c7 | spec_inputs: SD-01 6cd0610cce0cb6e2acb328a3c42350ff1189d194f5104273a91e02c205d61aad, SD-02 b0fec7cc4b135f7a7741d309d578ded62c59f636926887c6fce9dea0351124cd | acceptance: AC-01..04 pass | findings: 旧 HTTP 路由顺序差异，原生产链基线同样可复现 | pending: 完整固定组、主线核对、规范落地、归档清理 | next: merge

- AC-01：从两个实际 Auto 文件跟踪到包绑定、已校验公开函数及 MIR 类型，再到 AV 单代与拥有型回值。重查 AutoUI 的 API diff 只搬移 import，旧生成三份契约文件同哈希。符号表、动态列表和类型边沿现有 parser/binder 检查，没有为 ASE 另造编译器或 HTTP 路线。
- AC-02：核代次在实例锁内保留，实际 owner 单线程串行；关闭撤投绑定请求而不误撤其他租约。直接重放 `stale_and_late_completions_cannot_publish_but_success_stays_committed`，1/1，测试 0.13s、总 0.243s，保留第一次工具误选 0 条的记录。
- AC-03：直接重放 `trap_after_global_writes_rolls_back_both_list_and_next_id`，1/1，测试 0.20s、总 0.233s。沿结构变更前 AV fork 和同代两份全局状态的恢复点复查；引用 §7 的 20 轮及 Windows 进程实测，检查队列停机和真实 join。无本地网络或 subprocess 入口。两个直接重放均使用已准备 test 二进制，未构建；发现/清理未单列。
- AC-04：两仓冻结 tree 无计划代码 WIP，新增 Rust 文件 rustfmt 与 `git diff --check` 通过。新增 10 个用例时间走 §7，最长 1.787s。Rust 告警来自基线共享依赖；生成 DB 的 `done` 参数告警已存在。旧 VM 静态 DELETE 被动态 DELETE 抢先，记录原 owner 债务，局部业务对照和原传输证据齐备。SD-01、SD-02 是当前合同的对应来源，待落地核验，不写遗留台账。

stage: goal | plan_id: PLAN-1111 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1111/{auto-lang,auto-ui} | commit: lang 293d092d6 / ui 06e87993 | next: merge

### 冻结合并门（2026-11-23）

PLAN-1122 合入共享编译路径后，中止旧基面的准备。补丁等价变基到 lang `2a2d82bbac7c2947a1c5582a3fea2e862a216baf`、ui `4cc04828ac12c42366f8b958344b27f18e230bd7`。同一 10 条受影响 Todo 用例复跑通过，构建 11.400s、测试及清理 2.857s、整命令 14.870s。主线后续仅文档变动，最终拟送达 lang `340a0be6c935e742f1c48e8eda3b7ea9b6a3e583`、ui `9f1f0fed12757e3d0fcaa03d028fa524ed3cb997`。两仓 `git range-diff` 均为 `=`，冻结执行输入和规范路径在最后变基前后零差异。

全机等 PLAN-096 及 PLAN-1122 编译、固定组和 Case 运行；等待未精确分列。开始准备、开始执行及串行重放前均从 Windows 进程表查无其它构建/测试。冻结命令 `cargo t ase-m0 ace-source ace-av ace-facade --debug-tools --prepare-only --out <scope>/rebased-gate-prepared`，准备成功，383.492s，清单 SHA256 `e5c8cb529d74ce2f73d528d0f25108fbb4fbf19349a1156eba58972a3af5da7d`。沿原固定组驱动执行 `--run-prepared`，Windows 临时包装仅附加 `and not test(/(?i)linux|wsl/)`，包装 SHA256 `17e7d73b3f50320ffff5c8407e061af618fd6812bc5b838cec863318ac7f6ab8`。唯一匹配 `stdlib_assembly_1043_linux_target_ignores_missing_windows_sdk`，只列举未执行，归专用 Linux 环境。

| 固定组 | 首跑 | 最终归因 | 发现 | 执行进程墙钟 |
|---|---|---|---:|---:|
| ase-m0 | 38 执行，38 PASS | 全绿，新增 10 条在内 | 2.416s | 2.591s |
| ace-source | 990 执行，985 PASS、5 FAIL、另 1 原条件排除 | 5 条 `text_record_source` 原红，既有 `verify.double-move` | 0.471s | 12.264s |
| ace-av | 1626 执行，1600 PASS、26 FAIL | 13 条原红，12 条补准备后绿，1 条隔离计时重放绿，Linux 排除 1 | 1.991s | 69.366s |
| ace-facade | 100 执行，99 PASS、1 FAIL | `p969_t02_console_lane_grants_and_capture` 原红，仍返回 `Success { value: I32(1), steps: 2 }`，对应 D-1098-01 | 7.984s | 11.625s |

首跑总墙钟 110.532s，退出 1，保留 32 条首红。准备共享、清理未单列。结果 SHA256 `29de860c07d6ca36755ffaad6511805e1512672c93ebc3ef4c5a5d28ac01db6a`。比对同冻结基面 PLAN-1122 完整证据，结果 SHA256 `be4610934970d46ad62ba788391946b624567d89d9a1719aff246ee864c1d188`；除进程 PID 外，source 5 条及 AV 13 条的失败全文一致。facade 与 [PLAN-1092](1092-av-parity-attributes-and-metadata.md) 和 [PLAN-1098](1098-ace-daily-regression-purpose-and-cost.md) 收据一致。AV 原红涉及 method modes/owned、owned first case/recipe、data profile、1106 text leaves、915 三条和 layout，原门状态持续可见。

恢复仅跑受影响 13 条。Python provider 7 条缺 `py.descriptor.json`，从同冻结 ACE debug 构建 `prepare-av-providers` 31.565s，显式使用本机 `C:/Python314` 准备 10.868s，DLL SHA256 `bfe26ddb6ef1e0bb8baeb35b3a129aba855053c50cef44d626667397927b6f05`。另 5 条原 `PREPARATION_REQUIRED`，用同 run 环境补 `runtime-warm.exe` 的 `prepare-debug` 2.733s，`run-debug` 再查三个库均为 builds=0。复用冻结归档，定向串行 12/12 PASS，测试/清理 8.124s、总 10.710s。A2R 64 KiB 首轮内计 11.493s，单项重放 1/1 PASS，测试 1.516s、总 3.158s。重放日志哈希分别 `4e9dfe7f355746d40ada4f28d1982258ad006f31b40fb1aa1ca776568ee37c86`、`f995124bddc92f2d6e9f8e0e2c71ffcdec3f1342e9d2666cdfb19e872f41fc9d`。最终 2754 条执行，对账后 2735 通过、19 主线原红、0 新增未闭失败。

stage: merge | checkpoint: prepared | plan_id: PLAN-1111 | plan_revision: r1 | accepted commits: lang 340a0be6c / ui 9f1f0fed | gate: §9 冻结合并门 | next: guarded ff-only

stage: merge | checkpoint: landed | plan_id: PLAN-1111 | plan_revision: r1 | delivery: lang 340a0be6c935e742f1c48e8eda3b7ea9b6a3e583 / ui 9f1f0fed12757e3d0fcaa03d028fa524ed3cb997 | evidence: 两仓 master 快进，三个预检 guard clean，正式 AutoUI 源哈希与 §7 一致，冻结二进制读取主线 AutoUI 源的真实 CRUD 1/1，测试 0.34s、总 0.459s

stage: merge | checkpoint: canonical_verified | plan_id: PLAN-1111 | landed source: docs/specs/ase/local-service-binding.md 6cd0610cce0cb6e2acb328a3c42350ff1189d194f5104273a91e02c205d61aad / docs/specs/ace-migration/m2-modules.md b0fec7cc4b135f7a7741d309d578ded62c59f636926887c6fce9dea0351124cd | next: archive and guarded cleanup

归档门核对状态、任务、链接及交付祖先后提交计划与 PLAN-1112 接口链接，T-07 在实际 cleanup 完成后关闭。

stage: merge | checkpoint: archived | plan_id: PLAN-1111 | completion_kind: delivered | archive_path: docs/plans/archive/1111-ase-local-todo-service.md | archive gate: 6/7 tasks, 16 links, 5 navigation/source references and both delivery ancestors | cleanup: pending

## 10. 待澄清事项

无必须先由用户裁定的选项。T-01 负责确认受检包的原源码兼容、typed 边界与可驻留执行。如发现最小上游缺口，给出失败用例、负责计划、落地/复验条件后再推进相应任务。
