# Auto ROS2 SDK：v0.6 分阶段交付路线

2026-11-12 · r4 · 1007-A已授权实施，其余仍规划。目标/required矩阵与架构权威为[Design57](../57-auto-ros2-sdk.md)。当前仅PLAN-1007 r4的A批可在独立worktree进行原生ROS/provider/ROSIDL与部署研究；不修改ACE核心、不绕M1-TAG、不合并。1007-B、1008–1012及1018/1019仍未授权实施，正式SDK/设备部署与发布未启动。用户要求的“实际工作SDK”替代旧bridge首版上限，历史[ROS2战略](ros2-ecosystem-strategy.md)不再决定本路线排序。

## 1. 版本定位

v0.6 在拆分重构M1、ACE M2、AGE M2和Linux发行版主线基础上追加 ROS2 SDK 专项。SDK须满足Design57 R06-01..12，不以桥接demo收尾。增加此项不降低原四线的发布门，也不自动重启Python/算法/MCU等历史候选。未冻结可信工期，不从旧版本日历推导发布日期。

“依赖ACE能力”表示消费正式交付的接口；“依赖ACE最终候选”表示最后发行身份核验，两者分开。AGE不是核心SDK前置，Autonix不是Ubuntu别名。ROS专项签章、AGE签章、ACE M2和发行版签章分别保留。

## 2. 阶段与正式计划

| 阶段 / Plan | 独立结果 | 实际进入条件 | 出口 / 下游 | 状态 |
|---|---|---|---|---|
| R0 [1007 环境/provider合同](../../plans/1007-ros2-sdk-platform-and-provider-contract.md) | A原生验证/类型与部署研究，B正式ACE接缝和最终业务平台合同；原required保留 | A已授权且不依赖M1 tag；B等新授权/M1正式tag及所需正式接口，不改ACE核心 | A保留独立分支不合并，不作为下游已交依赖；官方示例可消费B05/06/07有限正式来源，完整R0仍保留 | executing r4，2/17；进度见Plan |
| R1 [1008 接口生成](../../plans/1008-ros2-sdk-interface-generation.md) | 标准/项目接口包→受检Auto类型、类型支持、缓存/拒绝；R06-02 | 1007官方示例输入正式切片；960/962/963相关能力切片 | 接口生成和双向原生值oracle；供1009/1010 | drafting r3，0/6 |
| R2 [1009 运行时语义](../../plans/1009-ros2-sdk-runtime-and-task-semantics.md) | Node/Topic/Service/Action/参数/时间/资源语义；R06-03..07 | 1007官方示例输入正式切片；967/968/969及库的所需切片；类型合同可与1008并行，最终联调等其正式来源 | 有界队列、取消/close/重启等正负控通过；供1010 | drafting r3，0/6 |
| R3 [1010 双端与工具](../../plans/1010-ros2-sdk-ace-dual-backend-and-tooling.md) | AV开发、新A2R安装发布、colcon/ament/run/launch/录放；R06-01/08/09 | 1008/1009正式来源，967 A2R/978装配及Linux目标接口 | 干净工程同源两端真实运行，原生CLI/工具检查；供1018/1019及1011/1012 | drafting r3，0/6 |
| E1 [1018 官方基础示例](../../plans/1018-ros2-sdk-official-examples-conformance.md) | OF-01..05：公开Auto SDK对接官方双向Topic/Service/Action、参数与干净工程，两端教程 | 1010正式来源；间接核1007有限输入/1008类型/1009运行时，不等真实设备或AGE | 原版独立节点与普通Auto源码双端互验；供1019 | drafting r1，0/9 |
| E2 [1019 turtlesim综合教程](../../plans/1019-ros2-sdk-turtlesim-tutorial-and-interoperability.md) | TS-01..05：运动/Pose、服务、RotateAbsolute/取消、实例与恢复、AV/A2R教程 | 1018已交；官方Qt/显示环境实际准入，普通CLI不等AutoUI/AGE | 数据/错误/必要画面分别验；供1011，不代传感器生产 | drafting r1，0/8 |
| R4 [1011 工作流与控制台](../../plans/1011-ros2-sdk-workflow-and-autoui-console.md) | 一个真实工作任务＋AutoUI、项目所需tf2等；R06-10 | 1010/1019正式来源、980/auto-ui的所需Linux UI/事件切片、1007冻结业务/设备 | 真实对端任务/反馈/取消/失败恢复/回放；供1012 | drafting r3，0/6 |
| R5 [1012 Autonix与终验](../../plans/1012-ros2-sdk-autonix-release-and-certification.md) | 本地profile安装运行/更新恢复、长稳/性能、全矩阵独立验收；R06-11/12 | 1007–1011及1018/1019交付；发行owner候选与ACE M2最终SDK/无legacy来源 | R06全齐、候选身份/安装产物/文档一致，专项签章 | drafting r3，0/6 |

1007 r4为A八项＋B七项（含有限输入work/review/未来merge）＋原T-05/06，共17项；1008–1012各6项，1018九项、1019八项，八包共64项，目前仅1007的A01/A04两项完成。1007实际进度见唯一Plan，A部分通过不构成R0已交；计划勾数不替验收。

R0的A批有界原生验证现已授权，B的ACE接缝仍未启动。R1/R2可在R0的B05/06/07官方示例输入切片已审已落主后并行，不能用A分支或B05候选代替；联调必须消费正式依赖。正式消费者按R3→E1官方基础→E2 turtlesim→R4传感器工作流→R5发行终验串接；传感器类型/业务调查可在1007内并行，不等官方教程。Autonix profile原生/容器可行性须在R0完整合同解决，不能到R5才发现安装路线不存在；它与设备/生产预算不反向卡官方示例有限输入。每包仍遵守上游Plan内部进入门，不替945–948放宽M1-TAG。

## 3. 与三条主线的交接

| 主线 | 本专项取用 | 本专项交回 | 发布边界 |
|---|---|---|---|
| ACE M2-A | 967 FFI/资源/callback、968任务调度、969宿主、972/973库、978构建；1008另需模块/组合/泛型相关能力 | 类型/异步/生命周期与真实ROS场景反例；通用缺陷回原owner | 不把ROS新义务偷塞冻结分母；依赖接口分窗，最终1012绑定完整M2候选 |
| ACE M2-B | 实际SDK/产品安装发行、旧执行链退出 | 同源AV/A2R SDK消费者和安装结果 | SDK不使用旧VM临时兜底；ROS失败保持本专项pending，不伪造ACE原门通过/失败 |
| AGE M2 | 可选空间可视化所需Node/图形/session能力 | 机器人姿态/轨迹/传感器领域消费者 | 默认不在核心SDK关键路径；若追加为首业务必需须明确修约，AGE完整门仍独立 |
| Linux / Autonix | NixOS＋COSMIC候选、安装/profile/网络/设备/回退合同 | 固定版本机器人开发/运行profile和原生ROS工具互通证据 | 不用Ubuntu单机结果代Autonix，不用VM/NAT替真实机器人网络；不自行把发行M1–M4重排 |

AutoUI控制台依赖UI而非AGE增强。若使用系统级能力代理，另加AutoOS通信adapter及授权映射；普通SDK节点不受此全局前置约束。

## 4. R0 必须冻结的决定

| 决定 | 当前候选 / 未知 | owner与关闭条件 |
|---|---|---|
| 首业务 | 已选传感器采集与处理，设备未定先通用路径 | 1007/T-A02与B03冻结真实任务/设备/预算；官方例子输入由B05/06/07独立交接，turtlesim不替生产 |
| ROS与RMW | Ubuntu24.04/Jazzy/x86_64优先候选；可能随设备选Lyrical/其他 | 1007/T-A02/B03核供应商SDK/驱动、接口包和依赖版本，锁一个主要组合；未测试组合列not-tested |
| ARM64 | 是否首设备必需未知 | 1007/T-A02/B03，必需则加入全部相关实现与发行行，不自动后移 |
| provider/API | rclrs首选，稳定性和Linux/Action/tf2能力待验证 | 1007/T-A03与B01/02/03给一个正式选择、受检ABI、公共接口、关闭/错误/线程规则；无可行选择不得放行R2 |
| Autonix部署 | 整体ROS node/host容器化优先评估，原生Nix封装作比较 | 1007/T-A05与B03/04核本地运行/设备/发现/GUI/可恢复更新；冻结workstation/device角色和边界；Ubuntu库不能直接混链接 |
| 资源/性能 | 无已批准数字 | 1007/T-A02/B03在实现结果前冻结消息/频率/并发/长稳时长/延迟/内存等预算、测量与oracle；各包沿用 |
| SDK仓/runner | 独立auto-ros2逻辑owner，物理承载未定 | 1007/T-B05先定示例切片的仓/路径/当地Plan/测试命令，B03核完整业务合同与同组布局，才允许跨仓写代码；语言Plan作协调合同 |

这些是实施进入条件，不阻本次设计草拟。所有未知保留具名owner与关闭条件，不用无数据的“性能良好”验收。

### 首组计划的可消费顺序

1. 1007-A03/A07：原生官方对端/接口库存，补客户端方向，仍不证明Auto支持；当前不新增GUI或运行授权。
2. 1007-B05/06/07：未来B进入后先交示例所需SDK/API/平台的正式输入，完整设备/Autonix合同留原任务；当前不授权B/merge。
3. 1008/1009/1010：交共用类型、运行时和同源AV/A2R/包入口，明确承担示例需要的浮点/Action类型事实，不靠特判。
4. 1018→1019：分别交官方基础套件与turtlesim完整普通Auto CLI教程，独立复审后按未来授权落主，成为1011消费者输入。
5. 1011→1012：真实传感器工作流、AutoUI、负载/长稳与Autonix终验；保留原R06-10..12。

官方样本矩阵和源码依据见[Design57 §11](../57-auto-ros2-sdk.md#11-官方示例首组合同2026-11-12增补仅规划)。1010不依赖1018/1019；1011依赖1019；1012同时核两份官方收据，不形成consumer→SDK倒置。

## 5. v0.6与后续

v0.6必做以Design57 R06-01..12为准：标准/自定义类型、Topic/Service/Action、参数/QoS/时间/资源、AV/A2R、ROS工具/包、真实任务＋AutoUI、Autonix与可量化运行质量。tf2/Lifecycle/业务封装若为首工作流所必需，由1007显式指定1011切片；不是要求v0.6完整重写这些库。

后续候选：多发行版/多RMW、非首验ARM/边缘GPU、更多设备、完整tf2/Lifecycle封装、AGE空间工具、零拷贝/共享内存、系统/Agent代理、多机器人运维、安全扩面、算法与micro-ROS。首业务强制要求的安全/硬件能力不能机械标后续。后续不自动承诺v0.7全交。

### ROS数据专门适配（v0.6必做）

用户要求尽量减少ROS数据复制与序列化，按[Design57 §12 DP-01..08](../57-auto-ros2-sdk.md#12-ros2传感器数据的专门适配与低复制合同2026-11-12仅规划)实施：1007新增A08原生数据路径/计量调查（本轮未执行）；1008负责类型化metadata＋连续buffer/视图，1009负责provider/FFI/队列owner与安全loan/owned模式，1010负责AV/A2R批量访问和同源copy/encode账，1011测传感器工作流，1012核实际部署与独立负控。

普通热路径优先安全借用/owner转移，不为穿越FFI额外JSON/CDR往返，不逐像素/点动态展开；不可借用时显式计量物化。普通无变换SDK路径以最多一次payload物化为目标，多次必要复制需保留差额和具名原因/审查。callback借用不能越过有效期进入await/队列，slow consumer/取消/drop/close必须有界且正确归还资源。所有provider的C↔Rust转换计入，未知计数不能写0。

SDK的低复制表示、计量和消除重复转换不再推迟到后续版本。RMW loan/shared-memory按实际Image/PointCloud2及配置/部署验证；不支持时安全owned模式仍可用并披露代价，不能由固定POD例子声称传感器端到端零拷贝。完整跨部署/GPU零拷贝继续是扩面。原native小消息结果和OF/TS绿不核销DP大数据/资源门；不因此本轮修改ACE/系统配置或运行探针。

## 6. 验收和执行纪律

各Plan §6/7以真实对端、类型oracle、状态/资源反例与固定职责组验证；历史r1只做文档检查；当前1007-A按r4保留原已授范围，本轮新增官方任务只规划；后续可依原授权在统一材料scope运行独立原生ROS/Rust探针，禁止从语言根运行Cargo/ACE/AG门或生成正式Auto接口代码；GUI/设备/ISO仍未启动。没有review/merge/正式来源的候选不能被下游登记已交。

实施时一Plan一worktree：`D:/autostack/.wt/lang-<NNN>/auto-lang`，分支`plan-<NNN>-dev`；SDK/UI/发行仓使用同组、当地规则与所属owner计划。材料仅`D:/autostack/plan-artifacts/plan-<NNN>/`，merge折叠摘要到Plan后按现行保留政策核路径清理；不建junction/symlink。

最终专项收据包含R06逐行结果、源/接口/工具链/镜像hash、实际平台设备与not-tested组合、用例数与排除、性能/长稳数据、真实安装更新恢复、review结论。缺一required行不得报告“实际工作SDK完成”。当前无实施/运行/签章证据。

r1历史文档检查：8份新增设计/路线/Plan的本地引用、6个唯一计划号与依赖DAG、36项任务/36项AC、12行required矩阵、9个引用固定组名称均核对通过；相关已跟踪文档diff whitespace检查通过。该记录仅为文档一致性核验，不是SDK测试或独立功能复审。共享检出的他方WIP及后续分配计划号未改写、未纳入本专项提交；本轮未提交。

2026-11-12 r3文档检查：8份Plan的63项任务与metadata计数一致，1007原2项完成保留；11个标准章节、本地引用、Markdown表格列、唯一计划号和依赖DAG通过；R06-01..12与OF/TS各5行完整。新增1018/1019均drafting、0进度；1007仍executing。仅检查文档，未运行ROS/cargo/GUI、未创建新实施树、未合并或发布。

2026-11-12 r4数据合同检查：8份Plan共64任务，保留1007原2项完成；R06十二行、DP八行、各基础计划AC-DATA/V-DATA/SD-DATA映射、本地引用/表格/依赖DAG通过。新增A08及数据计量/借用/loan负控均未执行；1007 worktree保持原干净提交，无ACE核心或上游改动。本轮仅规划，不合并。
