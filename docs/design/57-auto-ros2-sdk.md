# Design 57：面向实际工作的 Auto ROS2 SDK

2026-11-12 · proposed r3。用户确认目标为“能用于实际工作的 Auto ROS2 SDK”，要求专项设计、分阶段路线、ACE M2/AGE M2/Linux 依赖及 v0.6/后续边界；**本次仅规划**，不授权实施、探针、生成接口/SDK、构建、GUI/设备运行、提交、打标或发布。下文 required 表示拟交付义务，不表示已实现。执行拆包见[专项路线](strategy/ros2-sdk-delivery-roadmap.md)，旧[ROS2 战略](strategy/ros2-ecosystem-strategy.md)保留历史。

> 2026-11-12执行增补：用户已授权PLAN-1007 r4的A批提前实施、建独立worktree并暂不合并；不修改ACE核心、不绕M1-TAG。上方“仅规划”为初稿授权历史，当前以1007 r4为准；1007-B和其他SDK计划仍未实施。用户选定传感器采集与数据处理，设备未定、先通用数据路径。

## 1. 目标与版本身份

Auto 成为机器人应用开发语言：消费已有 ROS2 驱动、导航、机械臂及感知节点，用普通 Auto 模块编写消息处理、设备/业务集成、异步任务与操作界面。首版交付 SDK、工程工具、文档和真实工作应用，不以双方 mock、单个桥接 demo 或手写内置消息作为完成。

v0.6 本专项完成谓词：§3 的 R06-01..12 全部通过，绑定同一 SDK/源码/依赖候选；一个已冻结真实工作任务、AutoUI 控制台和 Autonix 安装运行完成独立复审。缺设备、预算、依赖或证据时保持 pending，不自动降为 bridge 发布。既有 ACE/AGE/Linux 各自总门保留，ROS2 收据不能代替它们，反之亦然。不新增日期承诺。

非目标：重写 DDS/RMW、导航/SLAM/感知算法、高频硬实时控制、完整机器人操作系统、任意 C++ 插件导入、完整 RViz/Gazebo 替代、所有 ROS 版本/架构/中间件排列、micro-ROS。调用现有算法 Action/Service 属本版允许的工作流，不等于开发算法。

## 2. 现状、来源与未决事实

2026-11-12 只读调查语言 HEAD `e6fb5855bab3fa103e41502833d37c6b440c00be`；共享主检出已有他方大量设计/计划 WIP，未纳入本专项交付证明。现行规范优先：

- [全局总览](../specs/overview.md)为取材入口，部分路径/旧引擎状态是历史快照，不作 ACE M2 完成证书。
- [共享接口/标量 ABI](../specs/auto-abi/shared-interface-and-scalar-bridge.md)只证明有限 Windows x64 合同；[装配](../specs/stdlib/design/ace-binding-and-assembly.md)只证明有限模块。不能推导 Linux、ROS2 聚合类型或完整 importer 已支持。
- [M2 分母](../specs/ace-migration/m2-denominator.md)、[Design56](56-ace-m2-ffi-and-native-stdlib.md)、[M2 路线](strategy/ace-m2-delivery-roadmap.md)分别定义已冻结义务、目标 FFI/库与依赖；本专项消费通用成果，不替 967/968/978 等实现其原职责。
- 实读 `experimental/ace/crates/av-core/src/host.rs` 仍有 `console/1` 有限宿主声明；`experimental/ace/crates/a2x-rust/src/provider.rs` 明确存在 `a2r.foreign-unsupported` 拒绝。ROS2 AV/A2R 路径是新目标，不是现有可用能力声明。
- AGE 来源为[两阶段路线](../../../auto-ui/docs/design/autographics/age-three-milestones-and-ace-m2.md)，UI 与图形职责仍归 auto-ui；不得把 M1 旧 VM 内容包充 ACE ROS2 应用。
- Linux 权威为 [Autonix ROADMAP](https://github.com/auto-stack/auto-os-distro/blob/main/ROADMAP.md)。通过已连接 GitHub 只读取得 `ROADMAP.md` blob `808985be33acfdb017356026a58309c9bbbac0ab`：NixOS＋COSMIC、M1–M4、首基线 x86_64/VMware/UEFI/NAT；不是 Ubuntu 衍生发行版。没有据此核验其 ISO/设备或后续阶段完成。

外部依据（2026-11-12 读取；实现前锁定 commit/package/image digest）：

| 来源 | 有限结论 / 设计影响 |
|---|---|
| [ROS2 发行表](https://raw.githubusercontent.com/ros2/ros2_documentation/rolling/source/Releases.rst) | Jazzy 到 2029-05、Lyrical 到 2031-05；首版锁一个 ROS 版本，不承诺跨发行互通 |
| [Jazzy 安装](https://raw.githubusercontent.com/ros2/ros2_documentation/jazzy/source/Installation/Ubuntu-Install-Debs.rst)、[Lyrical 平台](https://raw.githubusercontent.com/ros2/ros2_documentation/rolling/source/Releases/lyrical/supported-platforms.rst) | 候选参考环境是 Ubuntu24.04/Jazzy 或 Ubuntu26.04/Lyrical；不能由此推导 NixOS 原生安装可用 |
| [rclrs README](https://raw.githubusercontent.com/ros2-rust/ros2_rust/main/README.md) | 已列 Topic/Service/Action/参数/时间；项目仍声明无稳定保证。只据此列为首选探针对象，不宣称 Auto 可直接使用 |
| [客户端层](https://raw.githubusercontent.com/ros2/ros2_documentation/jazzy/source/Concepts/Basic/About-Client-Libraries.rst)、[executor](https://raw.githubusercontent.com/ros2/ros2_documentation/jazzy/source/Concepts/Intermediate/About-Executors.rst) | rcl 绑定外仍需类型生成、执行模型、参数与资源合同 |
| [QoS](https://raw.githubusercontent.com/ros2/ros2_documentation/jazzy/source/Concepts/Intermediate/About-Quality-of-Service-Settings.rst)、[Action](https://raw.githubusercontent.com/ros2/ros2_documentation/jazzy/source/Concepts/Basic/About-Actions.rst) | 匹配和目标状态必须保真；Auto 超时/取消不能隐式替代远端结果 |

业务方向已定传感器采集/处理；设备、正式ROS/RMW/ARM64、性能阈值和Autonix形态由1007/T-A02/B03/B04冻结，数据路径调查由A08负责；官方示例有限输入由B05/06/07正式交接。未知不等于不需要。

## 3. v0.6 必做矩阵

以下为全专项唯一验收分母；各 Plan 的 AC 引用这些 ID。1007 只能细化平台/样本/数值预算，不能删 required 行。类型、版本和资源限制必须先声明，再实现/验证。

| ID | required 范围 | 可观察验收 | owner |
|---|---|---|---|
| R06-01 | 一套明确版本的 Auto SDK/API、宿主 provider、ROS/OS/arch/RMW 及工具链矩阵 | 可查询支持/拒绝项；锁定依赖和重建输入；缺 provider、错 ABI/类型/版本在使用前命名拒绝 | 1007、1010 |
| R06-02 | `.msg/.srv/.action` 与 ROSIDL 解析模型衔接；标准及项目自定义接口包 | 数值宽度/符号、bool、float32/64、UTF-8 string（含有界形态）、固定数组、有界/无界 sequence、嵌套消息、常量/默认值和包依赖双向保真；超界/未知类型明确拒绝；wstring/额外 IDL 构造先列拒绝集 | 1008 |
| R06-03 | context/node、名称/namespace/remap、ROS 日志、graph 查询、幂等 close | 两节点不串状态，关闭释放资源；原生 CLI 可发现节点/端点；错误/回调不冒成功 | 1009 |
| R06-04 | Topic 双向 pub/sub；有界队列与 QoS | reliable/best_effort、keep_last/depth、volatile/transient_local 和 default/sensor_data profile 实测；晚加入/不匹配有可观察结果，溢出可诊断 | 1009 |
| R06-05 | Service 客户端/服务端，异步完成及本地超时 | 与真实 C++/Python 对端双向；超时后晚响应不串请求，不将请求自动重发成重复领域效果 | 1009 |
| R06-06 | Action 客户端及基础服务端 | goal ID、拒绝、反馈、成功/失败/取消终态；取消拒绝和完成竞争；超时保留结果未知，不声称远端停止 | 1009 |
| R06-07 | 参数声明/get/set/校验、YAML 启动配置、参数事件；timer/system/ROS time/use_sim_time | 原生参数工具交互；无效参数不生效；仿真时钟暂停、前后跳变不混淆单调 deadline | 1009 |
| R06-08 | 同源 ACE AV 开发＋新 A2R 独立发布，公共 SDK 行为一致 | 同接口/业务/反例两端实际运行；无旧 AutoVM/A2X fallback，无仅 emit/编译冒运行；关闭后无旧回调进入新 session | 1010 |
| R06-09 | 包/工具闭环 | package.xml/ament 索引、colcon 构建/install、ros2 run/launch、remap/YAML、rosbag2 录放及 RViz2 原工具互通；干净消费者无源码树路径依赖 | 1010 |
| R06-10 | 一个冻结的真实业务＋AutoUI 工作控制台 | 状态→提交→反馈→取消/失败恢复→记录回放贯通；使用既有真实设备/业务节点，不能仅用 SDK 作者的 mock；UI 不阻塞、不重复提交 | 1011 |
| R06-11 | Autonix 上可安装、运行、升级和恢复的机器人开发/运行 profile | 真实安装环境的依赖、网络发现、目标设备权限、GUI与CLI、更新/回退实际核验；Ubuntu通过不代此项，VM/NAT不代真实设备跨机路径 | 1012 |
| R06-12 | 性能/资源与可维护交付 | 按§12 DP-01..08冻结数据路径/负载/阈值后测复制/分配/编码账及latency/throughput/loss/CPU/RSS、长时运行/重连/close；文档、示例、来源、独立复审与发布矩阵一致；所有 required 非 pending | 1007 定预算、各实现包测、1012 汇总 |

QoS deadline/lifespan/liveliness、keep_all 等超出首版保证的策略必须明确报不支持，或在 1007 冻结为所选业务的必需项并由 1009 承接；禁止静默替换。float NaN/Inf/-0 的接受与保真规则按原生类型与选定 codec 在 1008 冻结，JSON 不作为二进制/宽整数通用传输层。无界 sequence 表示类型形态受支持，不承诺无限内存；消息字节、元素数、队列及资源上限由 1007 定量冻结，超限与类型错误分开。

## 4. 架构与职责

### 4.1 公共源码与两个入口

主路径：普通 Auto 业务模块 → Auto ROS2 SDK → 受检 Foreign/宿主合同 → ROS2 client/provider → rcl/RMW → 原生 ROS2 节点。开发 AV、发布 A2R 共享类型/错误/资源/权限事实。无需新 ROS 语法、独立解释器或第三套执行引擎；SDK 公共包名/具体函数签名由 1007 冻结，本文不伪造现存 API。

系统路径：AutoOS client → 已批准系统通信合同 → ROS2 adapter → 同一 SDK/provider。只为需要暴露的能力建立映射；普通 Auto ROS2 节点不强制绕 broker。原“先通信框架再机器人算法”继续约束系统集成与新算法工作，不再把完整 AutoOS F0–F2 当作所有 SDK 包的总开工前置。系统 adapter 本版为可选扩展；R06-10 可由应用进程直接消费 SDK 完成。

### 4.2 provider 选择

1007 有界验证 rclrs＋受检 Rust/C-compatible shim 为首选；比较必要的直接 rcl/补充原生库方案，检查 Action、参数、ROS time、tf2、消息生成、Linux 动态链接和退出。若 rclrs 不满足，记录失败反例并正式选择一种实现，不能削 SDK 验收；不长期维护多个同义 provider。Python/rclpy 允许作为需求/oracle 对照，不作为正式 SDK 必经进程或旧引擎 fallback。

复用 967 自动 binding/受检 manifest/shim，不用 ROS 专用数字 native ID 绕通用 FFI；ROS 对象以 opaque handle＋generation＋owner context 进入资源合同，Rust 私有布局/trait object/指针不暴露为 Auto 值。A2R 可以消费合同等价的直接 crate adapter，但须与 AV 路径对拍，不声称实现 ABI 天然相同。

### 4.3 代码与仓库边界

拟设独立 SDK 源包/宿主/接口生成器所有权（逻辑名 auto-ros2，**本次不建仓或目录**）；1007 冻结真实承载仓/路径。若选新仓，先由所属仓分配本地实施计划，不拿语言计划号冒该仓号。当前六份语言 Plan 为专题协调/执行合同，跨仓实际代码写面必须在开工前明确同组路径、owner、当地 AGENTS 和门禁；未具名之前不跨仓施工。

| owner | 拥有 | 不由本专项重复承担 |
|---|---|---|
| auto-lang/ACE | 公共绑定/Foreign/资源/调度/错误/目标与构建能力 | DDS、自制 ROS executor 语义或应用算法 |
| SDK owner（1007 冻结） | ROS API、接口生成与类型映射、provider、教程/样例、ROS 工程 adapter | 新语言引擎/另一套标准库 |
| auto-ui / 应用 owner | AutoUI 组件、UI线程/会话接口；工作控制台业务 | 把 ROS 加成所有 UI 应用的依赖 |
| AGE / 图形 owner | 已有图形能力和可选机器人可视化 adapter | 首版重新实现 RViz 或仿真器 |
| auto-os-distro | Nix/profile/镜像/容器装配、网络设备权限、安装升级回退 | SDK 内重写 Nix 或自行更换发行底座 |
| auto-os | 需要时实现系统通信与授权 adapter | 直接 SDK 的必经 broker |

## 5. 关键运行语义

| 规则 | producer → consumer / owner | 不变量与异常结果 |
|---|---|---|
| 类型身份 | ROSIDL/interface package → 1008 生成类型 → provider/AV/A2R | 包名/类型名、接口内容 hash、依赖闭包、目标及 generator 版本参与缓存；ABI 错配不因同名被接受；反序列化前校验长度与资源上限 |
| executor 交接 | 原生 executor → 1009 有界事件入口 → ACE task / UI dispatcher | 原生回调不在未知线程重入 AV；UI只在所属线程修改。明确服务响应/future唤醒机制，不能把阻塞等待写成异步；并发、重入和有序保证逐项声明，不承诺跨 publisher 全局顺序 |
| close / session | context/node/endpoint → owner generation → executor join/drop | 停接收→取消/排空策略→join→释放；超时返回关闭未完成而非成功。双close安全、旧handle拒绝；首版热更允许显式停止重建，禁止暗中重复订阅/重发目标 |
| Service | request ID → response matcher → task | deadline 到达只结束本地等待；远端效果可能已发生；迟响应不匹配新请求。不承诺 exactly-once，不默认重试有副作用调用 |
| Action | goal ID → goal state → feedback/result/cancel | 发送取消、远端接受取消、终态 canceled 分开；断线/未知单列。基础 server 允许显式拒绝并发 goal，不声称自动实现抢占 |
| QoS/队列 | RMW profile → endpoint → ACE队列 | 匹配失败、deadline事件与本地溢出分开报告；不能因 DDS reliable 就声称 UI 消费无丢帧。控制结果不得被传感器最新帧淘汰策略覆盖 |
| 时间 | ROS clock / monotonic clock → timer/deadline | ROS时间可暂停/回跳；本地资源deadline使用单调时间。区分消息stamp、接收时间和clock域；回放无效时不继续发真实设备控制命令 |
| 大数据 | ROS buffer → bounded owned/borrowed buffer → consumer | 按§12专门适配ROS数据：元数据与载荷分开，有效owner下借用/转移缓冲区；不增加无必要的深复制/编码；不可借用路径只有显式计量的物化，不得默认JSON或逐元素动态值膨胀 |

系统 adapter 的 request/goal/receipt、取消和错误只做显式语义映射，不因同 transport、同 Zenoh 或同 Topic 名就宣称兼容。授权、版本与实例失效按原系统合同验证。

## 6. 平台、Linux 与设备合同

参考环境候选优先 Ubuntu24.04/Jazzy/x86_64；若现有工作设备要求 Humble/其他版，或发行/驱动栈适合 Ubuntu26.04/Lyrical，由 1007 以实际软件包与设备证据选择**一个主要 ROS 发行版和一个主要 RMW**。其他组合为 not-tested，不自动宣称不支持，更不据命名推定兼容。

Autonix 必须另交安装 profile。1007 比较锁定 ROS userspace 容器/OCI运行与 Nix 原生封装的可行性、更新/回退、网络/设备/图形行为；优先评估隔离 ROS userspace 的方案，不提前宣称能在 NixOS 上使用 Ubuntu 动态库。宿主 Auto SDK 与容器 ROS 依赖不能任意混链接：可将 ROS node/host 整体放入容器，原生 UI 经受检进程边界连接，或将整个 GUI 应用放入受验容器；两者均需保留用户编写 Auto 节点和 AV/A2R 行为，不降级为通用转发桥演示。

远端 Ubuntu 机器人可作为真实工作拓扑的一部分，但“Autonix仅SSH启动远端程序”不满足 R06-11。Autonix 必须具备受验的本地开发/运行/操作入口、版本身份和恢复路径；设备侧/工作站侧角色由 1007 冻结。

首架构为 x86_64 候选。1007 必须核实工作设备是否 ARM64；若首个真实任务依赖 ARM64，把相应 target/toolchain/provider/部署实测加入 required 矩阵，不能记为后续绕过。GPU、相机、雷达、串口/USB 等仅对选定设备逐项准入，不由VM通过泛化。Autonix既有VM/NAT镜像测试继续保留，ROS跨机发现/网络必须补对应拓扑实测。

## 7. ACE、AGE、Linux 依赖合同

| 所需能力/owner | 本专项消费者与进入条件 | 不形成的倒置依赖 |
|---|---|---|
| ACE 960/962/963、967 | 1008需要模块/组合类型；1009真实源FFI需要967资源/回调与Linux目标事实 | ROS专项不回填成ACE M1或967/F0首UI Hello新增前置；967/F0有限文本不足ROS完整运行 |
| ACE 968/969、972/973 | 1009异步task、timer/close；1010执行宿主、bytes/时间/进程 | 消费已审已落主的具名切片，不拿分支tip作已交，不等待所有App替换才能首次接入 |
| ACE 978、967 A2R腿 | 1010同源AV/A2R、预编译host/依赖锁与干净包 | 不能由旧A2R或生成源码代新路径运行 |
| ACE 980 / auto-ui | 1011只等Linux平台所需普通UI/事件与AV/A2R切片 | 不等所有传统UI demo/AGE增强；不向980原合同暗加ROS职责 |
| ACE 985/986/987 | 1012最终候选须消费完整M2/实际SDK发行/无legacy身份 | ROS成果独立签章；既有M2分母不被扩大/削减，本专项新增义务独立登记；联合版本仍按各门合取 |
| AGE M2 / 981 | 可选空间展示等其实际Node/图形/会话切片 | 核心SDK、普通控制台与SDK签章不等待完整AGE M2；整体v0.6仍等待既定AGE发布门 |
| Autonix发行owner | 1007锁profile合同；1012依赖候选镜像/安装更新回退接口的具名正式来源 | 不指定“必须M4才能测SDK”；ROS专项不凭自身收据签整条发行路线完成 |
| AutoOS F0–F2 | 可选系统adapter用能力/错误/取消/receipt合同 | 直接节点不等完整系统通信；新机器人算法仍另行设计 |

并行原则：1007批准实施后可先调查/原生provider有界可行性；依赖未交时可交合同但不能交ACE运行PASS。1008与1009可在1007的接口合同冻结后并行，联调各依赖正式来源；1010消费两者；1011消费1010与UI切片；1012消费全部结果及最终发行候选。完整ACE M2是最终发布前置，不是所有前探的总开工前置。

## 8. 验证与发布

三层证据分别报告：纯类型/生命周期反例、原生ROS端到端互操作、真实业务/设备/Autonix交付。正控与负控详见各Plan；真实对端至少使用原生C++和Python用例，不能两端共用Auto codec作为唯一oracle。

1007在看结果前冻结工作集：消息大小/频率/数量、并发节点/goal、运行时长、p50/p95/p99、吞吐/丢失、CPU/RSS峰值与稳态、close/重连上限及环境；阈值依据工作要求和原生基线，不凭本次估计填数。后续修改预算按语义修约、保留原红，不把跑得慢改成条件排除。

本轮只检查文档/链接/合同，不运行 cargo 或 ROS。未来实现按实际职责选择现有固定组：`cargo t ace-foreign-contract ace-abi ace-host ace-runtime ace-a2x ace-rust-lowering ace-stdlib`为候选集合，各包选完整受影响职责组和必要下游；不能按此串机械全跑，更不能用 legacy `a2r/vm`组代新ACE路径。SDK专属ROS用例的包名/runner/命令在1007定，未来新增入口在所属Plan创建，当前不虚构已存在的`cargo t ros2`。UI/AGE、发行测试使用所属仓owner入口；无关AG全量不为此启动。

未执行、环境红、重放绿、条件排除分别记账；测试列出/实际执行/排除数及准备/发现/运行/清理耗时分别保留。正式gate验证子项真实终态和零选择，不能只看父exit0。

每包独立review后交来源与受验矩阵；1012复核R06全分母与候选同源，再做安装/升级/恢复和联合签章。Specs仅在实际交付后按reviewed delta沉淀，当前不写已支持。原始计划材料只允许 `D:/autostack/plan-artifacts/plan-<NNN>/`，开工登记owner/用途/在途；收尾折叠摘要到Plan并按政策核路径、清理，不入attachments。

## 9. 后续扩展

| 扩展 | 启动依据 | v0.6关系 |
|---|---|---|
| tf2、Lifecycle管理、Nav2/MoveIt业务封装 | 实际工作需要的API/平台/错误语义 | 若首业务需要，1007冻结为1011必做切片（复用原生库）；其余API后续，不把监听/tf算完整tf2 |
| AGE姿态/轨迹/点云/图像展示 | SDK与AGE所需图形切片正式交付 | 选定展示可选；不替RViz互通或普通UI必做项 |
| 多ROS版本/多RMW/跨发行桥接 | 逐组合真实设备与工具证据 | 后续扩面，不默认跨版本wire兼容 |
| ARM64与边缘GPU | 工作设备/供应商SDK支持矩阵 | 首任务必要则前移必做，否则后续 |
| 零拷贝/loaned/shared memory | §12数据路径、实际消息/RMW/部署与ownership支持 | SDK低复制缓冲区适配与计量属v0.6必做；RMW loan/shared-memory在支持矩阵成立时验证，跨部署/GPU/全消息零拷贝为后续扩面；不以标量loan成功推导Image/PointCloud2全链零拷贝 |
| AutoOS能力/Agent adapter | F0–F2、授权与goal/receipt映射 | 可独立追加；不开放未经授权的全图控制 |
| DDS安全/SROS2、多机/多机器人运维 | 实际部署信任/网络与身份要求 | 默认后续；若首任务环境强制要求，1007必须前移并具名修约，不能先不安全部署 |
| 原生控制/新算法/micro-ROS | 独立实时/设备/算法合同 | 另议，非SDK首版旁带义务 |

## 10. 本轮交接

初稿六份Plan均drafting r1、0/6；当前1007已修订r4并保留A批执行，唯一树D:/autostack/.wt/lang-1007/auto-lang，其余1008–1012仍drafting且未实施。1007首先冻结业务、版本、RMW、平台、部署与预算；1008–1012不自行猜测这些决定。除已授权1007-A外，其余实施需要另行授权；届时按既有Plan/worktree/review/merge流程，不重开已归档M1或擅改ACE/AGE/发行owner合同。

## 11. 官方示例首组合同（2026-11-12增补，仅规划）

用户同意“官方基础示例→turtlesim综合示例→传感器工作流”的顺序，要求建立计划并纳入路线。复用1007的原生预验证，新增[1018官方基础示例](../plans/1018-ros2-sdk-official-examples-conformance.md)和[1019 turtlesim](../plans/1019-ros2-sdk-turtlesim-tutorial-and-interoperability.md)。本次只修订文档，不新增运行/GUI/安装授权，不启动新Plan/worktree；1007已有A-only/no-merge授权保持，B及正式SDK实现仍按进入门。

三层结果必须分开：1007-A的原生ROS对照是可行性证据；1018/1019是实际Auto公共SDK同源AV/A2R消费者；1011/1012是传感器真实工作/负载/部署。官方节点是独立oracle，复用原版ROS节点和工具，Auto实现其客户端/业务逻辑，不在SDK内增加turtlesim特判、私有native或另一套类型映射。两份新Plan是R06-02..09的具体消费者验证，不替代R06-10..12。

| sample ID / owner | 最小案例和支持范围 | oracle / 反例 | 关闭边界 |
|---|---|---|---|
| OF-01 / 1018 | Auto订阅官方talker；官方listener订阅Auto发布，标准String | 双向独立进程得到匹配载荷/序列；本地未连通不冒通过；QoS不匹配独立负控 | AV/A2R同源；有限样本，不承诺全局有序 |
| OF-02 / 1018 | Auto调用官方AddTwoInts server；官方client调用Auto server | 已冻结整数输入与精确和，服务缺席/本地超时/迟响应不串请求 | 不能双方都用自制Auto端自证 |
| OF-03 / 1018 | Auto对接官方Fibonacci server；官方client/client_cancel对接Auto server | 独立核goal/feedback/result/cancel回执与终态；拒绝由具名策略fixture补，不假设官方server拒绝负order | 按实际接口包/版本生成；example_interfaces与action_tutorials_interfaces的同名类型不混用 |
| OF-04 / 1018 | 参数get/set/错类型、namespace/remap、两实例与节点查询 | 原生参数服务/CLI读回、错类型拒绝、实例不串状态；use_sim_time只读限制明确保留 | 不改上游示例实现来迁就Auto |
| OF-05 / 1018 | 干净工程的公开SDK源码、run/launch与开发/发行教程 | 新安装环境按文档重建并实际运行AV/A2R，关闭/超时清理可核；缺包/错类型支持不静默fallback | 不要求全部官方教程或全RMW组合 |
| TS-01 / 1019 | turtlesim原版节点＋Auto的cmd_vel发布/Pose订阅 | 选安全初态，正向指令造成可观察位移，零指令/错误remap负控可区分；Pose中的float32真实生成 | 数值容差/截止先冻结；画面变化不是唯一oracle |
| TS-02 / 1019 | Spawn、Kill、SetPen及Clear/Reset的教学流程 | Spawn返回名与对应Pose端点一致；Kill后端点/流停止；请求相关性；重复名结果按原版语义冻结 | 不假定所有Service都有标准错误响应；画笔外观另作可视走查 |
| TS-03 / 1019 | RotateAbsolute goal/feedback/result/cancel | 实际角度/remaining/delta与终态相符；取消接受与canceled分开；完成竞争如实报告 | 不是看到旋转就算Action全支持 |
| TS-04 / 1019 | namespaced双实例、节点重启、参数与失败恢复 | 不串海龟/实例，旧goal引用不冒新结果；未知结果不显示成功或自动重试有副作用请求 | 不承诺所有网络拓扑或旧目标恢复 |
| TS-05 / 1019 | 一份普通Auto源码的CLI教程、AV开发和A2R安装发布 | 新用户按说明连接官方节点、完成上述流程；GUI/数据/错误结果分别记录 | 不依赖AutoUI/AGE增强；官方Qt运行环境仍须实际验证 |

Humble作为已调查输入，正式ROS矩阵仍由1007冻结；实施前固定包版本或源码commit/接口hash、官方默认名字及命令参数。只允许等义CLI选项差异，不修改SDK契约或删样本来迁就版本。

外部依据（本轮只读）：[demo_nodes_cpp构建目标](https://raw.githubusercontent.com/ros2/demos/humble/demo_nodes_cpp/CMakeLists.txt)列出talker/listener/AddTwoInts入口；[官方Fibonacci server](https://raw.githubusercontent.com/ros2/examples/humble/rclpy/actions/minimal_action_server/examples_rclpy_minimal_action_server/server.py)使用example_interfaces并接受goal、处理取消；[官方client入口](https://raw.githubusercontent.com/ros2/examples/humble/rclpy/actions/minimal_action_client/setup.py)提供client/client_cancel。[Pose](https://raw.githubusercontent.com/ros/ros_tutorials/humble/turtlesim/msg/Pose.msg)、[Spawn](https://raw.githubusercontent.com/ros/ros_tutorials/humble/turtlesim/srv/Spawn.srv)、[RotateAbsolute](https://raw.githubusercontent.com/ros/ros_tutorials/humble/turtlesim/action/RotateAbsolute.action)为turtlesim接口输入；[原版frame实现](https://raw.githubusercontent.com/ros/ros_tutorials/humble/turtlesim/src/turtle_frame.cpp)包含Qt事件循环及重复名字处理。未检查本机是否安装turtlesim/GUI，不把上游源码可读当运行证据。

进入序为1007官方示例输入切片（B05工作/B06独立review/B07未来获准落主）→1008/1009→1010公共SDK双端/工程能力→1018→1019→1011真实传感器应用→1012终验。1007-A官方原生调查与传感器类型/数据研究可独立并行；其他Plan的准备仍各守自身门。1010不反等1018/1019，避免SDK与样例循环依赖；AGE不在官方CLI例子关键路径。1007完整设备/预算/Autonix合同仍归原任务并供1011/1012，不能把有限输入签成完整R0。设备未定可推进示例与通用功能，但官方样例通过不能替真实传感器平台/负载/Autonix验证。


## 12. ROS2传感器数据的专门适配与低复制合同（2026-11-12，仅规划）

用户明确要求针对ROS2数据专门适配和优化，尽量减少复制与序列化。本节把它设为v0.6基础设施义务，落实到1007–1012；本轮只修设计/计划，不运行探针、不修改ACE/标准库或上游ROS/provider，不改变M1-TAG、A-only/no-merge及其他计划实施门。R06-02/04/08/10/12验收须引用DP-01..08；不能仅用小消息功能绿声称传感器路径已优化。

### 12.1 路径和计量边界

为每个收发方向分别冻结并追踪：原生RMW take/borrow或publish缓冲区 → 客户端/类型支持的C与Rust表示 → SDK宿主/FFI → AV或A2R缓冲区/任务队列 → 算法或应用；反向发布按相反方向记录。rclrs的原生消息到Rust惯用消息转换必须纳入，不得只数shim调用处。采集驱动、压缩/解压/算法变换、DDS网络/共享内存传输及UI/GPU上传分别列出，不能混入或从端到端总量隐去。

复制分“元数据小量复制”和“payload深复制”；共享handle/引用计数不等于复制payload。每个观察窗口同时记消息数、输入有效字节、payload复制次数/字节、分配/重分配及容量、序列化/反序列化次数/字节、排队驻留/丢帧/延迟与峰值资源；不可观测项标unknown，不能填0。批量计量必须同时给分布/最坏样本，不能平均掩盖偶发巨量复制。

### 12.2 义务与owner

| ID | v0.6约束与实现方向 | 可观察正反验收 | owner |
|---|---|---|---|
| DP-01 | 每种消息/方向/执行目标/部署有一份数据路径与能力表，固定SDK、ROS/RMW、provider/type support、QoS、缓冲区来源/借用或owned模式 | 额外一层C→Rust/FFI转换或重序列化不会漏记；每个边界有owner与计量方法，未观察不冒0 | 1007冻结；1008/1009/1010落实；1012汇总 |
| DP-02 | Image、PointCloud2等采用类型化元数据＋连续payload缓冲区/视图；支持合法stride/padding/endian/field offset，不自动展开为每字节/像素/点的动态对象 | 非紧密布局、空载荷、大payload、float字段和有效padding正确；长度/乘法溢出/偏移越界/无效alignment被控制处理；不把CDR bytes直接当本地结构指针 | 1008 |
| DP-03 | SDK普通热路径优先安全借用/owner转移，使ABI与队列只转移描述符/handle。不能借用时，普通无变换路径的目标为最多一次payload物化；多次必要复制须逐边界说明并获审，不能隐藏 | 借用与owned路径各自有实际copy/alloc证据；强制copy、缺借用能力、错mode/过期generation必须可区分；未达目标保持优化差额，不改统计口径过门 | 1009/1010；1007核预算 |
| DP-04 | 原生loan/borrow只在其有效期内访问；跨async/队列需可保留的owner或一次owned物化。close/cancel/丢帧/多订阅者只释放各自持有，最终loan恰好归还一次；有界队列同时限制帧数和字节/loan驻留，慢consumer不钉死池 | callback返回后访问、await/取消竞争、旧view、共享帧写入、队列淘汰、borrow池耗尽等负控；拒绝或显式物化，不UAF/重复归还；处理结束资源实际释放 | 1009；967/968若有通用缺口回原owner，禁止本包旁路 |
| DP-05 | typed路径在已有原生消息上直接处理，不为穿越FFI额外JSON/CDR编码再解码。只做转发/录制的raw路径可保留已序列化envelope，显式携带类型/编码/版本/endianness；需读字段或变换时才选择正确解码路径 | 透明转发的payload/类型身份匹配，无额外encode/decode；错类型/编码/封装拒绝。算法变换/跨机DDS必要编码单列，不误报“全链无序列化” | 1008/1009/1011 |
| DP-06 | RMW loan/shared-memory为条件能力，按实际类型/版本/拓扑验证。Image/PointCloud2含可变长序列，不能由固定POD或Float64 loan例子推定同样支持。不可用时安全owned路径仍可用且能力/复制代价可查询 | 对实际大消息测试支持与force-disable/不支持两腿；源buffer有效期、数据保真、drop/return可核。共享内存已启用不等于全链零拷贝 | 1007定矩阵；1009实现；1012部署核 |
| DP-07 | 模式及copy/serialization账必须能由分配/调用追踪、buffer来源/identity和受控加一次复制/编码的负控交叉检查，不能靠实现自己声明zero_copy=true | 有意多复制/重序列化负控使对应计量增量可见并导致目标核验失败；合法baseline继续通过；不能有效测到的计数保持unknown | 各实现包；1012独立核 |
| DP-08 | 同输入/语义/QoS/负载的原生基线、Auto AV及A2R分别测正确性、额外复制/编码、p50/p95/p99、吞吐、CPU/RSS/驻留；借用路径与安全owned路径分别登记 | payload hash/数值与独立expected一致；在看结果前冻结预算。额外复制/编码须解释，SDK优化不能换来数据截断、减样本、丢可靠结果或扩大队列 | 1007预算；1010/1011测；1012验 |

“最多一次物化”是普通SDK路径的设计目标，范围包含provider C↔Rust转换与Auto边界，并非所有驱动/算法/网络的全链承诺；如所选表示或生命周期迫使多次复制，必须保留实测差额、原因及修复/具名例外，不能借“理论C FFI零开销”自行认定达成。独立review的原因说明不自动批准偏离已冻结预算；需要例外时按Plan语义修约确认。小型元数据复制单独计量，不为消除它制造不安全借用。

### 12.3 Auto/FFI表达与实施顺序

公共概念为受检owner handle、只读ByteView/TypedView、owned buffer、显式clone/materialize和释放/归还合同；这些是待1007/967核定的概念，不宣称现有API/语法已经可用。AV不直接接收Rust Vec/String私有布局或任意裸指针；需批量访问连续数据而非每字节一轮FFI。TypedView在对齐/endianness/offset符合时才可复用，不满足时受控读取或显式转换。用户可选择保留帧，但必须按预算持有owner；不能把callback借用塞进跨帧全局变量。

A2R可使用直接crate适配，只要类型/owner/错误合同等价且复制/编码单独计量；不能用A2R优化绿掩盖AV动态值装箱或借用问题。发送侧从用户填充的owned/borrowed buffer到publish的生命周期也必须验证，不能只优化订阅方向。

1007-A08先规划原生数据路径/配置调查与有限计量，不接ACE、不启用新RMW或改系统配置；具体原生试验仍按原已授范围和环境具名核定。本轮新增任务未执行。B批冻结可消费的描述符/owner/能力及可观测性合同；1008实现通用类型/buffer映射；1009实现provider/队列/loan回退；1010测同源双端；1011以传感器实际流程验证；1012核最终部署与全账。无需为此另造ROS解释器或维护第二套SDK，官方1018/1019仍走同一公共入口，但小消息/turtlesim结果不核销DP大数据腿。

v0.6必须交安全低复制表示、批量FFI访问、无无谓重编码、计量/负控和实际支持矩阵；RMW零拷贝在成立的profile逐项证实，跨进程/跨容器/跨机/GPU或所有消息零拷贝仍不普遍承诺。旧§9“零拷贝后续”只约束这些扩面，不再把SDK缓冲区设计与消除重复转换推迟。

依据：[ROS2 loan设计](https://design.ros2.org/articles/zero_copy.html)区分borrow/归还与用户内存持有；[loaned messages指南](https://raw.githubusercontent.com/ros2/ros2_documentation/jazzy/source/How-To-Guides/Configure-ZeroCopy-loaned-messages.rst)说明RMW支持及非POD限制；[FastDDS RMW配置](https://raw.githubusercontent.com/ros2/rmw_fastrtps/jazzy/README.md)区分Data Sharing配置、发布模式与队列复制。后两份为Jazzy设计输入，不推定本机Humble相同配置/能力，更不证明当前已启用任何零拷贝。所有计量/模式均尚未实施。
