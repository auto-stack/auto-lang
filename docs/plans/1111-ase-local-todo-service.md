---
plan_id: PLAN-1111
status: drafting
feature_name: ASE 本地 Todo 服务绑定与租约
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 0
total_steps: 7
depends_on: [PLAN-1109, PLAN-960, PLAN-968, PLAN-969, PLAN-1082]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/local-service-binding.md]
touched_goals: [GOAL-010]
affects: [ase]
---

# [PLAN-1111] ASE 本地 Todo 服务绑定与租约

## 0. 变更摘要

1109 已归档，Clock 验证了原生页签的定时、可见性与撤销。本计划先在无窗口宿主运行 Gallery 中 **013-todo 的真实 Auto 后端**，验证原来的业务操作能经本地绑定执行，省去每个 demo 的 HTTP 服务成本。原生前端与耐久数据交给 PLAN-1112，实施顺序见 [ASE 路线](../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

在 Windows x64 的可信本地原型中，经 ACE AV 运行现有 013 Todo 的后台模块，按原签名验列表、新增、改文字/完成状态、删除和清除已完成项。两个同 Realm 的会话共享一个按需服务，另一 Realm 即使同名也有自己的实例。请求有界，通过显式租约关闭，且本地运行零 demo HTTP listener、零 demo 子进程。

此阶段保持 Todo 原有内存数据语义，最后租约释放后再打开走原始种子。范围限 013 声明的本地服务表和宿主观察入口。远程 RPC、任意第三方/不可信应用、磁盘保存、Gallery 默认入口、HTTP proxy 改造及前端迁移由后续或原主责承担。

## 2. 架构方案

提议的路径为 `原 API 类型和公开业务入口 → 受检 ACE 包制品 → 有界服务 owner → ASE ServiceBinding → typed 本地请求/结果`。ASE 负责身份、准入、租约、任务生命周期和完成路由；原 Auto 模块持有 Todo 状态和业务，ACE 负责受检执行，AutoWeb 保有 HTTP 路由与传输责任。生产演示和无窗口本地测试消费同一份业务代码。禁止把本地调用绕成构造 HTTP 文本、RPC 到 loopback、旧 VM 调用或宿主内存假 Todo。

沿 [旧 proxy 生命周期](../specs/auto-lang/vm/back-proxy.md) 的按需激活、多窗共享与最后 owner 关闭语义，先对账现有 lazy/proxy/独立进程拓扑。本地路径继承已证的生命周期要求；原 HTTP 路径由原 owner 保持并提供可比业务对照。

## 3. 技术栈

语言仓 `experimental/ase-m0/src/{registry,run}.rs` 为现有无窗口解析与 AV 消费，拟在 `experimental/ase-m0` 加局部绑定模块与 `tests/local_todo_service.rs`。ACE 接缝候选为包制品与 AV 公开执行入口、既有宿主队列和可核验的运行限额。013 作者源码归 AutoUI，若需拆开 `#[api]` 路由声明和业务作用域，只在 AutoUI 原 owner 范围处理。拟工作树 `D:/autostack/.wt/lang-1111/{auto-lang,auto-ui}`，分支 `plan-1111-dev`。只读依赖按已交闭包取同组 detached 或既有解析顺序。材料登记 `D:/autostack/plan-artifacts/plan-1111/`。

## 4. 需求分析与背景调查

2026-11-20 核查主检出 auto-lang `5a76167cc`、auto-ui `448ed614`、auto-web `26d085f`、auto-os `efcfced`。本次授权限规划两个后继计划及更新设计路线。代码、运行验证、工作树、提交和归档属于后续阶段。以 [模块总览](../specs/overview.md)、[Gallery §5–6、8–9](../design/strategy/autoscape-native-gallery.md) 和 [1109 最终归档](archive/1109-ase-clock-tab-lifecycle.md) 为范围来源。

| 正式来源 | 当前已观察的能力与打开点 |
|---|---|
| [Todo API](../../../auto-ui/examples/ui/013-todo/src/back/api.at)、[Todo DB](../../../auto-ui/examples/ui/013-todo/src/back/db.at) | `pub fn`/注解 API 共享 Todo 记录；DB 有模块 `List<Todo>` 与连续 `nextid`、初始 4 条。它是内存服务。真实源码含模块引用、动态列表、记录、`?Todo`；须试原闭包，不能用 1082 集合支持推断整包与跨界参数/结果已通。 |
| [960 包闭包](../specs/ace-migration/m2-modules.md)、[969 宿主 facade](../specs/ace-migration/m2-facade.md)、[968 调度](../specs/ace-migration/m2-scheduler.md)、[1082 集合](archive/1082-av-parity-collections-and-places.md) | 已有受检包编译、AV 及调度基础。E0 是单次执行，960 文档只确认特定入口结果和全局 cell 观察。完整 Todo 的多次受检调用与类型落地由 T-01 先证。 |
| [1106 事件会话](../specs/ace-migration/component-event-sessions.md)、[983 Web](../specs/ace-migration/m2-web.md) | 1106 单 Node 的 UI handler 接缝与包面分开，当前对宿主通道有拒绝边界。983 是 HTTP 生产路径的证据。两者不宣称已经交付本地 typed 完整包服务。 |
| [AutoUI PLAN-105](../../../auto-ui/docs/plans/105-autoui-u03-local-interactive-apps-av-vue.md) | 原 013 前端是 Store、列表、索引/记录链和 API 导入，105 在处理原应用兼容。1111 的无窗口后台可以独立进入先证；1112 的原前端受实际公开接口约束。 |

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

## 8. 执行步骤

有原生任务工具时，开始实施前按以下 ID 登记任务。

- [ ] T-01 先证原 Todo 后端与 typed AV
  - owner_stage: work。对照 013 的 `src/back/{api,db}.at`、ASE `src/run.rs`、960/969 AV 入口和 Gallery 的原 proxy/HTTP 行为。跑 V-01 真实长链，出开工/待接缝结论及配置。AC-01。
- [ ] T-02 实现局部服务绑定并对照原业务
  - owner_stage: work；依赖 T-01。实验 workspace 内实现准入、真实编译/消费、同源业务注册与 typed 请求/结果。V-01/02，AC-01。
- [ ] T-03 接入按需激活、请求与租约
  - owner_stage: work；依赖 T-02。在所选原 owner 上接可取消任务、限额、释放与晚完成处理。V-03/04 正例，AC-02/03。
- [ ] T-04 固化真实正反控与回归
  - owner_stage: work；依赖 T-03。将 V-02–04 的负控和资源账固化到拟增测试入口，记录坏道失败和正常通过。完成 V-05 适用触面，准备集成版本。AC-01–03。
- [ ] T-05 整理交接和规范候选
  - owner_stage: work；依赖 T-04。冻结 SD-01、全链证据与 1112 使用方式，未过项具名。AC-01–03。
- [ ] T-06 独立复审
  - owner_stage: review；依赖 T-05。依据实际入口独立核证、重放关键负控、检查同源与遗漏。AC-04。
- [ ] T-07 正式落地
  - owner_stage: merge；依赖 T-06 pass。冻结职责/消费者并集，核规范、来源、归档与材料清理，更新路线和 1112 对交付件的引用。AC-05。

## 9. 复审记录

stage: new | plan_id: PLAN-1111 | plan_revision: r1 | outcome: pass（规划交接） | next: 获实施指令后进入 T-01，拟议验证均未执行。

起草检查通过。7 个任务、5 项验证、5 条验收、1 项规范候选均有责任阶段，本地链接和依赖方向已核对。

## 10. 待澄清事项

无必须先由用户裁定的选项。T-01 负责确认受检包的原源码兼容、typed 边界与可驻留执行。如发现最小上游缺口，给出失败用例、负责计划、落地/复验条件后再推进相应任务。
