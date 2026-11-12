---
plan_id: PLAN-1009
status: drafting
feature_name: ROS2节点、任务与运行时语义
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 3
current_step: 0
total_steps: 6
depends_on: [PLAN-1007]
supersedes_spec_components: []
new_spec_components: [docs/specs/ros2-sdk/runtime-and-task-semantics.md]
touched_goals: []
---

# [PLAN-1009] ROS2节点、任务与运行时语义

## 0. 变更摘要

2026-11-12 r3数据路径增补：用户要求针对ROS2数据专门适配并减少复制/序列化，纳入Design57 §12 DP-01..08。本轮只修合同，不运行新探针、不改ACE核心/上游，不放宽M1-TAG或当前不合并限制；原任务/AC与历史证据保留，旧功能绿不核销新增计量/生命周期义务。

2026-11-12 r2：仅规划修订，接入官方基础示例/1018和turtlesim/1019的职责、类型与验收接洽；原T-01..06、AC-01..06、SD-01及全部R06义务保留，未实施。

2026-11-12 r1，仅规划，0/6未执行。用户要求面向实际工作的SDK专项设计与阶段路线，暂不实施；不授权建仓/worktree、探针、接口生成/SDK、构建、设备/GUI运行、提交、merge或发布。本次只写设计/计划与导航，未来任务的命令/证据均为拟议。

设计权威：[Design57](../design/57-auto-ros2-sdk.md)；[R0–R5路线](../design/strategy/ros2-sdk-delivery-roadmap.md)。本包负责R06-03..07及R06-12运行资源腿；不能以bridge demo代required，也不削ACE/AGE/Linux原门。

## 1. 目标

交付§8中本包独立结果，以§6可观察oracle验证。非目标：本包之外的其他阶段、所有平台排列、新机器人算法、自制DDS、legacy兜底、未经批准跨仓代码改动。所有义务仍required，未执行/环境限制不计PASS。

## 2. 架构方案

1007的正式依赖可由r3 T-B05/B06/B07已审已落主的官方示例输入切片满足；不能用A未合并分支或B05候选代替。最终设备/生产预算/Autonix合同保留在1007/1011/1012，不作为本包基础能力的额外总前置。下文“1007正式合同”据此具名核对。

1007正式合同；967 Linux FFI/资源/callback、968任务/调度、969宿主及972/973所需库。可与1008按已冻结类型合同并行，最终联调必须消费1008已审已落主来源。

普通Auto源码→公共SDK→受检FFI/provider→真实ROS2；类型/线程/资源/错误语义与Design57一致。SDK承载目录为新目标，1007冻结前不伪造现存路径。分包协作复用同一合同，独立review前后只消费已审已落主正式接口，不消费他方分支tip。

## 3. 技术栈

Auto/ACE AV、新A2R、Rust/C-compatible宿主shim、ROSIDL与原生ROS工具；Autonix为NixOS＋COSMIC。具体ROS/RMW/toolchain/provider版本及runner由1007固定。非代码构建输出/外部消费者均显式指定统一材料scope。

## 4. 需求分析与背景调查

授权：2026-11-12本轮只规划、暂无实施授权，未指定工期、模型或预算。调查语言HEAD e6fb5855bab3fa103e41502833d37c6b440c00be；主检出存在他方文档WIP，不能将其统称本Plan改动或已交来源。

取材：[Specs总览](../specs/overview.md)、[ABI事实](../specs/auto-abi/shared-interface-and-scalar-bridge.md)、[装配现状](../specs/stdlib/design/ace-binding-and-assembly.md)、[M2分母](../specs/ace-migration/m2-denominator.md)。现状仅有限模块/Windows ABI；Linux/ROS完整能力不能从旧Spec推导。已实读宿主入口experimental/ace/crates/av-core/src/host.rs（console/1）和experimental/ace/crates/a2x-rust/src/provider.rs（a2r.foreign-unsupported）；它们是未来接入锚，不宣称今天已接ROS。

外部/跨仓来源、Autonix ROADMAP blob及有效范围见Design57 §2。跨仓仅只读，不修改ACE/AGE/发行owner已批准在途合同。正式开工重核当前commit/模块与当地AGENTS。

## 5. 详细设计

采用Design57 §3/5规定的类型身份、状态、资源/时钟不变量。所有公开入口必须可命名诊断缺失provider/错版本/超预算，错误不能退成成功空值。以下任务对可执行路径和证据闭合负责。

### 官方客户端/服务端语义接洽（r2增补）

1018将独立验证Auto客户端→官方server与官方client→Auto server；1019消费turtlesim官方服务端。T-02/03必须提供通用双向Service/Action、参数、remap/graph/关闭契约，T-04补迟响应/断线/取消状态与重复资源关闭的反例。官方Fibonacci默认accept-all，不把负order自动当拒绝；turtlesim服务callback的失败可能没有统一领域错误字段，明确原response/超时/诊断与业务判断。公共API保留cancel发送/ack/终态区分，未确认不报停止。不得手写turtlesim调用native或在示例代码吞掉当前provider的use_sim_time只读限制。新增样本的完整应用收据归1018/1019，本包不反向等待它们，避免运行时依赖消费者整包。

### ROS数据路径专门适配（本轮仅规划）

provider/运行时：把客户端C/Rust表示转换一并计入，优先保留可拥有的native/RMW消息或借用视图，跨FFI/队列转移owner handle/descriptor，不能默认再复制一份。callback借用不能跨await，除非底层明确提供可保留owner/loan；否则显式owned物化并计量。多consumer共享只读帧，写入要求唯一owner或显式copy；按帧数和驻留字节限制队列与loan，丢帧/cancel/close/池耗尽归还恰好一次。typed路径不为过FFI重编码；raw转发按类型/编码身份处理。loan不可用时安全owned路径保留且模式可查。

按[Design57 §12](../design/57-auto-ros2-sdk.md#12-ros2传感器数据的专门适配与低复制合同2026-11-12仅规划)核DP-01..08，元数据/payload、provider转换/FFI/队列/应用变换/RMW编码分账；unobserved标unknown。借用不能牺牲所有权/取消安全，RMW零拷贝仅对实际消息/profile证明，不普遍承诺。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ros2-sdk/runtime-and-task-semantics.md（拟新增） | 当前无此已交SDK规范；交付后记录本包受验行为/支持与拒绝集、owner/真实入口/证据 | 仅沉淀实际通过能力，不把规划伪装current-state | AC-01..06 |
| SD-DATA | add | docs/specs/ros2-sdk/runtime-and-task-semantics.md（拟新增部分） | 增加本包负责的DP数据表示/复制编码账/所有权/能力与拒绝模式，仅实际交付后沉淀 | 防SDK适配退化成重复转换或不安全借用 | AC-DATA |

目前不修改canonical Specs。若1007最终另定SDK承载仓，开工前修订SD-01为实际owner的canonical目标，保留本仓引用指针，不能双份手维护；review核最终增量与来源。touched_goals留空：未在现有goals.md核到ROS正式目标ID，不凭旧伞形文档的GOAL-021写不存在关联。

## 6. 测试设计

内部工作验证由实现者负责；T-05独立上下文复审，不把自测当独立review；最终角色/工具在实施前记录，本次不派agent。拟补充用例由T-02..04建立可运行入口，目前全部未执行：

| Case ID / AC | invariant / input | entrypoint / producer→consumer | observable oracle | coverage boundary |
|---|---|---|---|---|
| V-DATA / AC-DATA | callback后/await后旧view、cancel与drop竞争、slow consumer耗尽池、共享帧修改、force-disable loan、错编码/类型；无UAF/重复归还/无界驻留，必要物化有真实账，类型/流量保真 | T-02原生provider/有界队列/owner→T-03任务/取消/关闭，T-04故障与计量 | DP规定的实际数据/资源/调用与独立负控观察；禁止只自报计数或模式flag | 当前未执行；不以既有小消息/原生绿核销 |
| V-01 / AC-02 | 异步和资源：原生回调到达/close并发、两个node/session、借用失效 | T-02宿主资源→ACE task/UI dispatcher | 不跨线程非法重入、不串实例、不UAF；关闭终态/join可核对，close超时不报成功 | 首版允许显式停重建，不保证无缝热更 |
| V-02 / AC-03 | 请求状态：Service晚响应、Action拒绝/失败/取消被拒/取消与成功竞争 | T-03真实客户端/服务端 | ID匹配；每goal唯一合法终态；本地timeout保结果未知，未确认不报canceled | 不承诺exactly-once/自动抢占 |
| V-03 / AC-04 | QoS和时间：晚订阅/不匹配/饱和、use_sim_time暂停前后跳 | T-04真实RMW+ROS clock | QoS结果可观察，溢出独立计数；单调deadline不随ROS clock回退 | 只覆盖冻结QoS支持集 |
| V-07 / AC-07 | 官方双向Service/Action角色、accept-all策略、取消/迟响应/错实例 | T-02/03通用SDK与原生对照，T-04负控 | 双向角色可用、策略按真实server解释，取消ack不冒终态，旧请求不串新实例；错误不变成功 | 能力/反例验证；完整官方教程由1018/1019验 |

职责组候选：ace-foreign-contract ace-abi ace-host ace-runtime ace-stdlib，实际修改目标时补相应组。现有名称来自scripts/test-groups.toml；开工按实际修改选完整职责组及必要下游，命令形式cargo t <组名...>，不机械执行全部候选。SDK专属原生/设备用例命令由1007冻结，本包T-01消费并校验；未创建命令不虚报已存在。父exit0不代子项终态，零选择失败；保留列用例/实际执行/条件排除、环境红/重放绿及准备/发现/运行/清理时间。本轮只做文档检查，无cargo/ROS运行。

## 7. 验收标准

| AC | owner_stage | outcome / method |
|---|---|---|
| AC-DATA | work | T-02/03/04落实本包§5/§6与DP责任；有真实payload/复制分配编码/生命周期/模式及正负控证据；未测项不写0，多次复制差额保留，阶段review与最终review均核覆盖 |
| AC-01 | work | T-01前置、真实路径/命令、版本/拒绝集与分母已冻结；有具名可消费来源，无未决事实被默认成通过 |
| AC-02 | work | T-02结果按§8产生真实输入→输出/状态证据，V-01及相应正控可重放 |
| AC-03 | work | T-03结果按§8及V-02实际消费者核验，不能只schema/emit/mock通过 |
| AC-04 | work | T-04/V-03及本包所有必需反例、资源预算/完整职责组与必要集成实际通过；未执行项不计PASS |
| AC-05 | review | 独立核AC-01..04与所承R06行、实际diff/候选，遗漏/延期/workaround、warning/format/debug print及SD-01逐项有结论；未批准缺项不得pass |
| AC-06 | merge | 合入来源与受验身份一致，规范和Plan摘要可追踪，产物路径对账/guard/清理完成；不以merge阶段补做实现正确性 |
| AC-07 | work | §5官方首组接洽与V-07在原T-02/03/04范围通过；T-05独立核它与原AC，原职责不缩减 |

本包原work任务同时承担§5/§6/AC-DATA，具体owner为T-02/03/04；review逐项核新增数据义务，不改变未受影响证据的适用性。

## 8. 执行步骤

| Task | owner_stage | dependencies | action / affected paths / result | AC / verification |
|---|---|---|---|---|
| [ ] T-01 | work | §2正式前置；未来实施授权 | 冻结API/线程/owner generation/queue/close/Service与Action状态机；核967/F0仅文本不够资源和异步接入，按正式切片列齐来源，不对尚未交付的调度写替代实现；新SDK路径以1007冻结为准，通用接入锚见§4 | AC-01；核合同/实际路径/命令/正式来源 |
| [ ] T-02 | work | T-01 | 实现context/node、名称/namespace/remap、日志/graph、Topic/QoS/有界事件队列、资源关闭与回调交接；原生executor不在未知线程重入AV或修改UI，借用不跨生命周期；新SDK路径以1007冻结为准，通用接入锚见§4 | AC-02；§6对应真实入口与V-01；本行结果不凭勾选核销 |
| [ ] T-03 | work | T-02 | 实现双向Service、Action基础server/client和goal/feedback/result/cancel；参数声明/校验/YAML事件、timer/ROS time/use_sim_time；不默认重试有副作用请求，不把deadline变成远端取消成功；新SDK路径以1007冻结为准，通用接入锚见§4 | AC-03；§6对应真实入口与V-02；本行结果不凭勾选核销 |
| [ ] T-04 | work | T-03 | 与1008生成类型/原生C++Python节点联调并跑反例：并发/迟响应/取消竞争/断线重启/队列饱和/close join/旧session回调/时钟暂停跳变；按1007负载测资源与延迟；新SDK路径以1007冻结为准，通用接入锚见§4 | AC-04；§6对应真实入口与V-03；本行结果不凭勾选核销 |
| [ ] T-05 | review | T-01..04 | 在同一权威实施树独立审查全部AC/R06责任与SD-01；扫描未经批准延期/workaround，新增具名债候选记入docs/plans/KNOWN-DEBT-AND-RISKS.md；运行必要定向反例，记录冻结revision/commit | AC-05；独立review收据，不能由实现者自测替代 |
| [ ] T-06 | merge | T-05 pass及后续merge授权 | 核已审来源合入、沉淀受验规范/Plan摘要，交下游；材料全路径对账，guard clean后清理唯一树/分支与scope，按本仓archive路径归档 | AC-06；来源/摘要/hash/清理收据 |

未来唯一语言worktree：D:/autostack/.wt/lang-1009/auto-lang，分支plan-1009-dev；本轮不创建。跨仓若需实现，由1007绑定同组真实仓/当地Plan与owner后再建，禁止junction/symlink。测试/生成代码等计划材料仅D:/autostack/plan-artifacts/plan-1009/，开工登记owner/用途/在途；正常构建可在专属worktree，确需外置target只在该scope。收尾摘要折叠进Plan后删scope，未知归属fail-closed，不创建attachments或其他证据目录。

## 9. 复审记录

2026-11-12数据合同交接：stage=new，revision=3，仅文档修订；新增V-DATA/AC-DATA/SD-DATA，全部未执行。保留历史完成标记/原红/限制；不声明新计量、零拷贝或安全借用通过。

2026-11-12规划交接：stage=new，revision=2；本轮仅修合同，未运行新增样例。保留原进度/原证据，新增官方覆盖未执行，功能独立review/merge仍pending。next=按当前授权/正式依赖继续，未授予新Plan实施或GUI权限。


2026-11-12：stage=new，revision=1，outcome=pass（仅草案合同完整性/文档交接，非功能独立review或可开工PASS），execution_authorized=false；next=保持drafting，取得实施指令后先核§2/§10和T-01前置。新增T-01..06/AC-01..06/SD-01，全部未执行。正式T-05/06无证据，未reviewed/archived。

## 10. 待澄清事项

精确API/资源预算沿1007；1008只在合同层并行，最终联调依赖不能省；必要QoS扩项由1007显式前移。

这些决定有具名任务owner，不为本次规划追加确认流程；影响依赖工作的事实未定则不执行相应腿。首阶段不得据“范围冻结”自行删Design57 required；新增平台/验收语义改变须修约，保留旧结果与理由。
