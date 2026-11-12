---
plan_id: PLAN-1007
status: executing
feature_name: ROS2 SDK平台、真实任务与provider合同
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 4
current_step: 2
total_steps: 17
depends_on: []
supersedes_spec_components: []
new_spec_components: [docs/specs/ros2-sdk/platform-and-provider.md, docs/specs/ros2-sdk/official-example-input.md]
touched_goals: []
---

# [PLAN-1007] ROS2 SDK平台、真实任务与provider合同

## 0. 变更摘要

2026-11-12 r4数据路径增补：用户要求针对ROS2数据专门适配并减少复制/序列化，纳入Design57 §12 DP-01..08。本轮只修合同，不运行新探针、不改ACE核心/上游，不放宽M1-TAG或当前不合并限制；原任务/AC与历史证据保留，旧功能绿不核销新增计量/生命周期义务。

2026-11-12 r3：仅规划修订，纳入官方示例原生库存/反例与新增1018/1019交接；原A01/A04完成证据保持，新增任务不倒算已完成；A-only/no-merge和B/M1-TAG门不变。

2026-11-12 r2：用户授权修订A/B两批并创建独立worktree开始1007-A；**暂不合并，不修改ACE核心，不绕过M1-TAG**。A仅独立合同、环境、原生ROS/provider/ROSIDL与Autonix可行性验证；B为ACE接缝与最终平台/业务合同核验，当前未授权实施。总体保持executing，A局部通过不等整Plan完成/交付。

设计权威：[Design57](../design/57-auto-ros2-sdk.md)；[专项路线](../design/strategy/ros2-sdk-delivery-roadmap.md)。负责R06-01/12输入冻结，保留r1全部required和AC-01..06；r1 T-01..04尚未执行，现拆为T-A01..06/T-B01..04（映射见§8），原T-05/06继续负责独立复审/merge。r2总数6→12；r3增T-A07及B05/06/07后总数16，当前r4增数据路径T-A08后总数17，不保留重复勾选项。

## 1. 目标

A提前消除宿主库、接口映射与部署风险；B消费正式ACE接口并交最终合同。工作场景已由用户选定为**传感器采集与数据处理**，设备/消息/采样率/处理流程与性能要求待补充；本机WSL Ubuntu22.04/ROS2 Humble用于A实验，不自动变成v0.6支持矩阵。

非目标：实现正式SDK、修改ACE/标准库/自动binding/调度、启动1008–1012、建新产品仓、改发行版配置或运行设备控制、签章发布。独立原生探针不是Auto SDK运行证据。

## 2. 架构方案与A/B进入门

A路径：冻结输入/原生ROS接口→原生rclrs或C++/Python对端→原生RMW→结果/状态/资源观察。A可在ACE M1 tag前实施，因为不消费/修改未交ACE接缝。

B路径：普通Auto源码→公共SDK→受检FFI/provider→真实ROS2。B进入须重新授权，核ACE M1正式交付/tag身份及所需967/968/969/978/Linux接口的已审已落主来源；M1 tag本身不证明这些能力齐备。

A允许：独立worktree中的交付设计/接口合同文档；统一材料scope中的原生探针、上游源码副本、Cargo/colcon外部工作区、日志和生成物；在既有WSL ROS环境运行有限原生互操作。无需先完成最终业务预算才能做功能探针，但探针临时规模不是产品预算，不能据观察值倒设阈值。

A禁止：修改crates/**、experimental/ace/**、stdlib/**、根Cargo/测试分组或ACE依赖；使用未合入M1/M2分支作为正式接口；借1007实施945–948；手写Auto native数字ID/manifest绕967；GUI/ISO/真实传感器部署未具名之前不自动执行；全机升级、全局安装Rust/ROS替换、真实设备控制不在此次范围。

A停止条件：发现必须补ACE能力→记录B待办并停该腿；缺业务/设备/阈值→询问用户并继续原生通用探针；ROS/Nix依赖缺失→如实记录，不把静态检查/模拟当运行绿；跨仓产品改动→保留方案，不直接写其他仓。

## 3. 技术栈与路径

原生rclrs首选候选、C++/rclpy独立oracle、ROSIDL parser、已有WSL Humble/RMW；rclrs具体版本/来源先锁定再验证。ROSIDL研究只生成/读取原生类型，不生成正式Auto类型（1008职责）。Autonix=NixOS＋COSMIC，Nix可用不等于Autonix环境可用。

唯一语言实施树D:/autostack/.wt/lang-1007/auto-lang，分支plan-1007-dev；共享Plan簿记在主检出。本轮不合并、不清理该实施树。实施材料统一D:/autostack/plan-artifacts/plan-1007/，登记owner=PLAN-1007、用途=1007-A原生验证/上游副本/外置target、status=in-flight；原始探针源码与结果不进仓。禁止junction/symlink。Cargo/colcon输出显式位于scope，绝不从语言根构建。

拟交付文档路径（worktree内新文件）：docs/design/ros2/provider-feasibility.md。它描述目标合同/研究结论；原始证据只在scope，摘要折叠到本Plan。SDK产品承载仓留待正式决定，本次不建新仓。

## 4. 需求分析与背景调查

授权历史：r1仅设计规划；当前用户明确授权A提前实施/独立worktree，并限定不合并、不改ACE核心、不绕M1门，需要业务设备时及时询问。未指定工期或模型；不派agent。

当前主检出HEAD 1169f135940e38e6a567826ce406785134e356e3；存在其他会话文档WIP，代码路径preflight无WIP。仅消费该master已合入基线，不加入他方未提交设计/代码。唯一树创建后记录实际base/branch。

只读确认：本机WSL Ubuntu-22.04为Ubuntu22.04.5 x86_64，/opt/ros/humble、rclpy/rosidl_parser/colcon、Rust1.97.1和Nix2.35.3存在；test_msgs未安装。用户确认本机WSL有ROS2，并选择传感器采集与数据处理。RMW实际实现、上游版本和可运行性由A记录；不据安装目录判互操作PASS。

取材：[总览](../specs/overview.md)、[ABI事实](../specs/auto-abi/shared-interface-and-scalar-bridge.md)、[装配现状](../specs/stdlib/design/ace-binding-and-assembly.md)、[M2分母](../specs/ace-migration/m2-denominator.md)。Design55 §1.1明确945–948（含前探）等待M1-TAG；A不承接其任何任务。其余来源及Autonix ROADMAP blob见Design57 §2。B开工时重核当前Spec/接口，不继承A原生探针为ACE证据。

## 5. 详细设计

传感器方向优先研究sensor_msgs/Image、PointCloud2、Imu及header/time/QoS与二进制缓冲区；实际首消息待设备决定。研究类型长度/字节序/stride/field offsets和bounded sequence，不预先承诺特定传感器/零拷贝/频率。Service/Action探针只验证通用任务能力，不能代真实传感器业务验收。

### 官方示例原生预验证（r3增补）

本轮用户要求新增第一组计划/路线，仅修改文档；原A-only/no-merge授权保持，不在本轮运行新例子，不新增GUI/安装权限。T-A07负责官方包/节点/接口/调用方向库存和反例合同；T-A03在后续获准的原生验证中优先选官方headless对端补Rust客户端方向。turtlesim的Pose/Spawn/RotateAbsolute可先作接口调查；实际Qt节点启动必须先有具名运行环境与对应授权，不能绕当前GUI停止点。T-A04已完成的五种传感器研究证据保留；它不包含新增官方类型，也不被新任务冒充覆盖。

### 官方示例输入的有限交接

设备/生产负载/Autonix最终合同不能卡住官方例子。B新增T-B05先冻结官方例子所需ROS/RMW/实验平台/SDK仓与公共API/类型身份/资源/功能样本预算及实际ACE接缝来源，形成有限输入切片；T-B06独立复审其实现/反例与原required的边界，T-B07在未来获得merge授权后核同源落主交接。下游1008/1009可消费该具名切片，不必等A02/A05/B03/B04的最终设备/部署合同。阶段交接不把整个1007标完成，不改变AC-01..06或R06-10..12最终义务；整体Plan仍executing，唯一树保留。

B05/06/07均属B，当前未授权实施或合并，仍需M1正式tag和B01/B02所需正式ACE接口。未经review/正式落主的A分支、草案或B05候选不能作为下游已交来源。1008/1009 frontmatter的1007依赖按此具名切片核对；1011/1012的真实业务和最终发行仍等待完整相关合同。

### ROS数据路径专门适配（本轮仅规划）

数据路径调查与预算：新增T-A08只处理原生路径/类型支持表示、源buffer/loan能力/隐式C↔Rust转换的调查与有限计量配方；本轮未执行，不把原A04已完成的schema/值检查当复制计量。B05先冻结官方例子所需受检buffer/owner/模式与账目接口；B03/B04补实际传感器负载、额外复制/编码预算和原生基线。需改ACE、启用新RMW/系统shared-memory配置、安装工具或超出已授试验范围时停该腿，不借A实施。

按[Design57 §12](../design/57-auto-ros2-sdk.md#12-ros2传感器数据的专门适配与低复制合同2026-11-12仅规划)核DP-01..08，元数据/payload、provider转换/FFI/队列/应用变换/RMW编码分账；unobserved标unknown。借用不能牺牲所有权/取消安全，RMW零拷贝仅对实际消息/profile证明，不普遍承诺。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ros2-sdk/platform-and-provider.md（拟新增） | 目前无已交SDK平台规范；B最终核验/独立review/获准merge后只沉淀受验事实和拒绝集 | A不合并、不发布canonical支持声明 | AC-01..06 |
| SD-02 | add | docs/specs/ros2-sdk/official-example-input.md（拟新增） | 当前无正式输入切片；未来独立review且获准落主后只记官方例子平台/API/来源及明确排除 | 不让设备/发行合同反向阻塞教程，也不假称完整SDK交付 | AC-08..10 |
| SD-DATA | add | docs/specs/ros2-sdk/platform-and-provider.md（拟新增部分） | 增加本包负责的DP数据表示/复制编码账/所有权/能力与拒绝模式，仅实际交付后沉淀 | 防SDK适配退化成重复转换或不安全借用 | AC-DATA |

A只写研究/目标合同与Plan摘要，不更新现状Specs。touched_goals仍为空，不凭历史伞形文档生成未存在目标ID。

## 6. 测试设计

| Case / owner | input→entrypoint | distinguishing oracle | boundary |
|---|---|---|---|
| V-DATA / AC-DATA | 不同大小Image/PointCloud2、原生转换/borrow开关、源buffer生命周期；先冻结计量窗口和样本，受控加一次复制/编码须可见；未知计数不填0；入口：原生客户端/类型支持→宿主buffer的路径/来源与所有权，借用和owned模式，必要复制/编码及可测方法 | DP规定的实际数据/资源/调用与独立负控观察；禁止只自报计数或模式flag | 当前未执行；不以既有小消息/原生绿核销 |
| V-A01 / A01 | 已有WSL/工具/ROS库→只读preflight | 报实际版本/来源/缺项，不以缺可选包冒整个ROS失败 | 环境事实，不是SDK通过 |
| V-A02 / A03 | 原生Rust节点↔独立rclpy/C++节点 | Topic载荷一致、Service响应相关、Action反馈/结果/取消状态独立核对；不只节点内自发自收 | 原生provider，非Auto/ACE |
| V-A03 / A03 | QoS不匹配、晚加入、有限队列、退出/重复关闭、参数/ROS时钟 | 合法正控仍可用，负控产生预期状态/拒绝；资源释放以进程/线程终态核，不只timeout | 选定WSL/RMW小规模功能，不报性能基准 |
| V-A04 / A04 | 已安装sensor_msgs/项目原生接口→ROSIDL parser | 字段/嵌套/sequence/数据缓冲布局与原生消息类型一致；缺接口明确记录 | 不生成Auto SDK/binding |
| V-A05 / A05 | Autonix候选、Nix/容器依赖和网络设备边界→profile研究 | 区分源码审查/求值/真实安装运行；没有镜像或设备不得称部署已通过 | 不更改发行配置、不构建ISO |
| V-A07 / A07、A03 | 官方talker/listener/AddTwoInts/Fibonacci及turtlesim接口库存→原生CLI/schema/provider | 包名/接口hash/角色与上游语义一致；先库存再验证，记录未装/未运行；原生Rust客户端接官方server不代Auto | A07合同研究；A03原生运行另列，Qt GUI仍守进入门 |
| V-B01 / B01..04 | 真实Auto源→受检Linux FFI→AV/A2R | ABI/provider错配命名拒绝，合法两端行为等价；资源线程/取消与负控按Design57 | B未授权/未实施，禁止拿A核销 |

A runner在材料scope内显式限定进程超时、消息次数、ROS_DOMAIN_ID及本机loopback，不接真实控制网络；probe退出码不代子项断言。保原红、环境归因和重放绿；原生样本小规模仅功能，不称R06-12负载验收。A零语言代码触面不跑cargo t/ACE/AG全套；Rust探针独立Cargo manifest/target，不加入语言workspace。B职责组候选仍为ace-foreign-contract/ace-abi及实际影响组，留待B进入核定。

## 7. 验收标准

| AC | owner_stage | outcome / method |
|---|---|---|
| AC-DATA | work | A08/B05/B03/B04落实本包§5/§6与DP责任；有真实payload/复制分配编码/生命周期/模式及正负控证据；未测项不写0，多次复制差额保留，阶段review与最终review均核覆盖 |
| AC-01 | work/B最终 | 原T-01完整平台/业务/API/预算合同冻结；A可先冻结实验基线，未知项具名，不替代最终决定 |
| AC-02 | work/A+B | 原T-02原生provider及ACE接缝分别验证；A只能交原生部分，Linux ABI/Auto前门留B |
| AC-03 | work/A+B | 原T-03部署可行性与本地运行合同保留；A源码/环境研究不替真实Autonix部署运行 |
| AC-04 | work/A+B | 原T-04反例/预算/命令与依赖交接齐；A临时探针规模不能替业务预算，未合并A不放行下游 |
| AC-05 | review | T-05独立核完整AC/R06责任、diff、遗漏/延期/workaround、warning/format/debug和SD-01；A自测不代最终review |
| AC-06 | merge | T-06按后续授权合入/沉淀/来源核对/清理；当前禁止执行 |
| AC-07 | work/A | T-A07官方包/接口/角色/停止点合同完整，缺项与原生/Auto两层覆盖分列；不由静态库存宣称样例运行通过 |
| AC-08 | work/B05 | 官方例子输入切片的实际SDK/API/ABI/类型/平台/资源与正负控来源齐，设备/产品预算/发行pending显式保留 |
| AC-09 | review/B06 | 独立核B05有限输入、来源、反例及未完成分母，无偷偷降低原required |
| AC-10 | merge/B07 | 未来获准落主后身份/规范/phase收据可核，下游只消费该切片；无授权不执行 |
| AC-A | work | A每项据V-A观察结果独立标完成或pending；交接无ACE写面、无M1门绕过、无merge、无虚报运行通过 |

本包原work任务同时承担§5/§6/AC-DATA，具体owner为A08/B05/B03/B04；review逐项核新增数据义务，不改变未受影响证据的适用性。

## 8. 执行步骤（唯一任务表）

| Task | batch / owner_stage | dependencies | action / result | AC / verification |
|---|---|---|---|---|
| [x] T-A01 | A/work | 当前授权 | 建唯一worktree、核branch/base、登记材料scope并盘点WSL/ROS/工具/包/RMW；不碰ACE/根构建 | AC-A/01；V-A01 |
| [ ] T-A02 | A/work | 用户业务/设备信息 | 冻结传感器任务、现有节点/设备/架构、首消息/处理流程、最终预算依据与缺项；用户信息不足保持pending | AC-01；具名业务合同 |
| [ ] T-A03 | A/work | A01；不依赖A02最终设备决定 | 锁定rclrs候选并实现原生Topic/Service/Action/参数/时间/关闭及QoS正负控，与独立原生对端运行；依赖缺失准确记账 | AC-A/02/04；V-A02/03 |
| [x] T-A04 | A/work | A01；可与A03并行 | 研究ROSIDL与sensor_msgs类型/缓冲布局，形成生成输入/类型身份/拒绝/复制预算候选，不生成正式Auto类型 | AC-A/01/04；V-A04 |
| [ ] T-A07 | A/work | A01；本轮只规划 | 冻结官方包/可执行/真实接口与调用方向库存、源/版本/类型hash、OF/TS样本反例及GUI停止点；传感器研究不被替代，原生运行留A03后续，不生成Auto类型/不改ACE | AC-A/07；V-A07 |
| [ ] T-A08 | A/work | A01及A03已有原生来源；本轮仅规划 | 调查并设计Image/PointCloud2原生buffer/表示转换/loan能力与copy/alloc/encode计量及加一次复制负控；原生有限试验依原授权，超范围/改ACE/系统配置停该腿；不以现有schema/小消息结果核销 | AC-DATA；V-DATA、DP-01/06/07 |
| [ ] T-A05 | A/work | A01；不依赖ACE | 核Autonix锁定ROS userspace/原生Nix方案、host/container/设备/网络/GUI与更新恢复，具名指出哪些只有静态证据及所缺候选 | AC-A/03；V-A05 |
| [ ] T-A06 | A/work | A01..05/A07/A08结果逐项核对 | 汇总来源/命令/反例/结果、API与sensor数据路径候选、缺项和B进入条件；分支保留，不交正式已合入依赖，不将未完成A项核销 | AC-A；范围/路径与证据检查 |
| [ ] T-B01 | B/work | 新授权、M1正式tag/交付、所需967正式来源 | 检验Linux自动binding/shim、目标/ABI/资源/callback合同，不手写旁路 | AC-02；V-B01 |
| [ ] T-B02 | B/work | B01、968/969/978等已交切片 | 真实Auto源接缝与AV/A2R必要路径验证；没有能力不得补进ACE核心绕原owner | AC-02/04；V-B01 |
| [ ] T-B05 | B/work | B01/B02、A03/A07；仍需B实施授权 | 冻结官方样例所需实验平台/公共API/SDK仓/类型/资源和ACE正式来源；形成有限输入，不等待最终设备/Autonix合同 | AC-08；实际接口/普通消费者与反例 |
| [ ] T-B06 | B/review | B05冻结提交 | 对该有限切片独立审查实际接口/来源/反例，保留完整R0及真实业务未决分母；不冒最终T-05完成 | AC-09；阶段revision/commit收据 |
| [ ] T-B07 | B/merge | B06 pass及未来merge授权 | 核已审同源落主、SD-02与phase交接；保持executing/唯一树，刷新受影响身份；不归档/不删树，不冒最终T-06完成 | AC-10；下游输入来源；当前禁止 |
| [ ] T-B03 | B/work | A02/03/04/05及B02 | 冻结最终平台/provider/API/支持拒绝集、真实任务预算与SDK仓/命令；设备或部署未验不报完整 | AC-01/03/04 |
| [ ] T-B04 | B/work | B03 | 核全反例和Autonix真实可行性/职责组、形成1008–1012交接；未merge前仅待交付 | AC-01..04 |
| [ ] T-05 | review | A+B work完整 | 同一树独立复审；遗漏/延期/workaround具名记债，校验SD-01与冻结来源 | AC-05 |
| [ ] T-06 | merge | 独立pass及未来merge授权 | 核已审来源合入、规范沉淀与Plan摘要；全路径对账、guard后清理/归档 | AC-06；本轮禁止 |

原T-01→A01/A02/A04/B03；原T-02→A03/B01/B02；原T-03→A05/B04；原T-04→A06/B03/B04。原AC/required不减少，A/B不增加Plan状态。不把后续B/T-05/06勾上来制造A完成。

## 9. 复审记录与执行证据

2026-11-12数据合同交接：stage=new，revision=4，仅文档修订；新增V-DATA/AC-DATA/SD-DATA及T-A08，全部未执行。保留历史完成标记/原红/限制；不声明新计量、零拷贝或安全借用通过。

2026-11-12规划交接：stage=new，revision=3；本轮仅修合同，未运行新增样例。保留原进度/原证据，新增官方覆盖未执行，功能独立review/merge仍pending。next=按当前授权/正式依赖继续，未授予新Plan实施或GUI权限。


r1历史：2026-11-12仅规划，drafting0/6、未建树/运行；stage=new草案检查pass不代表功能复审。

r2授权：本轮开始A；stage=work，outcome=in_progress，execution_authorized=A-only，merge_authorized=false，ACE_core_writes=false；Plan保持executing。角色为当前实现者进行内部检查，正式独立review后续由T-05负责，本次不声称独立review已完成。

### 1007-A 首轮执行收据（2026-11-12）

- stage=work；outcome=in_progress；worktree_path=D:/autostack/.wt/lang-1007/auto-lang；branch=plan-1007-dev；base_commit=1169f135940e38e6a567826ce406785134e356e3；code_commit=394af104d94ffb17c6a9c021cf3931704bd242c4。提交仅2份设计/索引文档，未合并、未push；核心crates/experimental/stdlib/scripts/根Cargo零diff。无auto-down兄弟树：本批只运行外部独立原生Cargo manifest，不构建语言仓，不需要该依赖；如后续需要语言构建，必须按进入门另行核定并补齐依赖。
- T-A01完成：已核唯一树/branch/base，材料登记在主检出.retention/policy.json；scope为D:/autostack/plan-artifacts/plan-1007/（in-flight，外置targets/native与cargo-home均为可删除计划缓存）。WSL Ubuntu22.04.5、x86_64、ROS Humble、rmw_fastrtps_cpp；Rust/cargo1.97.1、Python3.10.12、colcon-core0.21.0、rosidl-parser3.1.9、Nix2.35.3。rcl5.3.13/rclpy3.3.21/rmw_fastrtps_cpp6.2.10/sensor_msgs4.9.2/rosidl_generator_rs0.4.12由现有dpkg包提供；test_msgs未装，不为此全局安装。
- 原生来源：rclrs=0.8.0、ros-env=0.3.0、rosidl_runtime_rs=0.7.0、async-std=1.13.2，Cargo.lock SHA256 `ca0051411065cb1cc1a615851a65bde4b068c1c38f8d72fa7999ca97de054be5`；官方examples源码副本commit a3940266c160c285c8d7dacb35d7b0f67365be99，仅参考，未修改其仓。全部探针源码/下载/targets只在统一scope。
- T-A03部分完成：`wsl -d Ubuntu-22.04 --exec sh /mnt/d/autostack/plan-artifacts/plan-1007/native/build.sh` 最终exit0、无编译warning；该入口显式外部manifest/target/CARGO_HOME。`.../native/run.sh` exit0；Rust原生节点与独立rclpy进程经真实FastDDS互通，ROS_DOMAIN_ID207、ROS_LOCALHOST_ONLY1，没有连接真实控制设备。14项观察=13项功能正负控通过＋1项明确unsupported，不能报14项SDK通过。
- 13项功能观察：transient_local晚加入；Image二进制4字节与4096字节；PointCloud2原始bytes；QoS不兼容端点可发现/无数据（Rust零计数核对）；Service关联响应42和-12；gain参数设置/类型拒绝；仿真时钟精确12s+345ns→1s+345ns回跳；Action成功/反馈/结果、拒绝goal、取消回执与最终canceled；QoS负控/停止命令及Rust真实exit0。部分计数是分层断言，非独立ROS功能族数；两个退出相关观察共用一次正常退出。Rust日志：NATIVE_PASS images=8 mismatch=0 graceful_stop=true。当前只测Rust服务/Action服务端与Python客户端方向；未测反向客户端、长稳/饱和、进程内重复close/泄漏、真实设备性能。时钟用例只证明观测到精确stamp及回跳，不声称完整暂停/跳变竞争覆盖。
- unsupported原始结果保留：rclrs的use_sim_time运行期切换被只读参数拒绝。首次请求动态启用失败留logs/attempt2-native-results.json；之后以启动参数use_sim_time:=true验证原生clock随动，同时断言运行期切换仍被拒绝。未改上游、未改SDK验收为成功。动态切换/明确重建node政策留B/1009选择，不将该缺口静默核销。
- T-A04完成（研究边界）：`.../native/schema.sh` exit0，实际ROSIDL Message.structure与原生Python字段顺序核对Image/PointCloud2/PointField/Imu/LaserScan五种接口，3项有效值/拒绝控制通过、1项语义缺口观察。原生Image允许step×height与data长度不符，已写入候选语义helper责任；uint32负值拒绝、pointcloud float32/endianness样本正确。没有生成Auto接口。交付设计位于唯一树docs/design/ros2/provider-feasibility.md，定义bytes/布局/QoS/线程/时钟与停止条件；文档检查pass。
- T-A05部分完成：Nix只读求值builtins.currentSystem→x86_64-linux；本机为Ubuntu而非NixOS，未发现docker/podman/proot/bwrap/qemu-system-x86_64命令。只读Autonix main ROADMAP blob808985be33acfdb017356026a58309c9bbbac0ab；main的flake.nix不存在。按ROADMAP所示研究plan-001-dev的flake.nix blob7ce5260732006ae003e234932ec7fc2e05f0ff1e、profiles/target.nix blob5ba06a225fbe84a0d6067f0a7d2c9b990ca2a2e3（仅研究，不作正式已交来源）：锁NixOS26.05、COSMIC/VMware目标；没有据此证明ROS容器/原生Nix安装可行。未安装运行时、未构建ISO/启动GUI、未写发行仓。
- 首红/修正保留：WSL继承localhost10809代理不可达→本任务命令禁用代理后Git/Cargo下载成功，未改全局配置；shell换行、ROS setup与nounset、非login PATH准备问题已修正；Humble Action反馈字段实际为partial_sequence而非其他接口的sequence，按原生类型修正；Humble无新版本rclpy.parameter_client便利模块，oracle使用实际SetParameters服务；原生spin短超时是正常idle事件，只用timeout_ok过滤明确Timeout，其他错误仍传播。首次Rust idle timeout错误收据在logs/attempt1-*。ROSIDL最初按根Structure取元素为空，按真实Message.structure模型更正后通过；未改ROS库。
- 路径纠正：两次colcon extensions查询默认写到主检出log与worktree/log，并产生内部latest链接。已核本次唯一extensions目录、无活动colcon及全部链接目标在其log内，仅unlink4个链接；4个普通文件（合计1112字节）迁移至scope/logs/colcon-discovery-main与colcon-discovery-worktree，逐文件SHA256前后一致，两个仓内log目录均已移走。迁移清单logs/colcon-relocation.json；后续colcon必须显式--log-base至scope。`wt-guard.sh D:/autostack/.wt/lang-1007/auto-lang` exit0/clean，无reparse point；未删除worktree。
- 证据入口均为scope相对路径：logs/native-results.json SHA256 `79eefa89c9229e4ae287ed36d4f34309ead84248fbd9b708c590f23b3665709d`；logs/rosidl-schema.json SHA256 `e977e5bf7016fec60000932e2be19352bb28f239f925b5a865b1cec49c5d0003`；logs/environment.json、logs/artifact-index.json及attempt1/2原红。原始材料未入git；记录只证明A原生探索，不证明Auto/ACE/正式SDK/产品负载通过。
- next_responsible_role=当前work实现者继续A。A02等待设备/负载等事实，但用户明确先通用数据路径，不追加阻塞确认；A03反向/资源/队列反例与A05真实部署环境仍pending。A06已有首轮摘要，完整A交接尚未完成。B/T-05/T-06未实施，不进入execution_done/reviewed，不合并、不签章。

## 10. 待澄清事项与停止点

用户已确认本机WSL ROS2，并选择传感器采集与数据处理。待补：传感器/数据类型、实际硬件/架构/ROS环境、采集/处理目的及频率/体积/延迟、可用样本/现有节点、Autonix目标角色。A02负责询问与冻结；A03/A04通用功能研究不依赖这些答案。缺实际候选/设备的A05只交有界结论，最终部署/业务预算不冒PASS。

B未授权且依赖未齐，暂不实施；945–948的M1-TAG门保持。其他计划仍只消费已审已落主来源，A分支即使内部验证绿也不能放行1008–1012。无合并/归档/清理请求，保留唯一worktree与材料scope供继续。
