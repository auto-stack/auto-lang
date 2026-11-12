---
plan_id: PLAN-1019
status: drafting
feature_name: turtlesim完整Auto教程与公共SDK互操作
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: [PLAN-1018]
supersedes_spec_components: []
new_spec_components: [docs/specs/ros2-sdk/turtlesim-workflow.md]
touched_goals: []
---

# [PLAN-1019] turtlesim完整Auto教程与公共SDK互操作

## 0. 变更摘要

2026-11-12 r1，仅规划，0/8未执行。用户确认官方基础示例→turtlesim→传感器工作流的顺序，要求第一组计划并纳入ROS2路线。此次授权只含设计/计划/导航，不含运行/安装/GUI/接口生成、建实施树、提交、merge或发布。1007既有A-only/no-merge授权独立保留，不因本包草案扩大。

权威：[Design57 §11](../design/57-auto-ros2-sdk.md#11-官方示例首组合同2026-11-12增补仅规划)、[专项路线](../design/strategy/ros2-sdk-delivery-roadmap.md)。本包负责TS-01..05，作为R06-02..09具体消费者验证；不替R06-10..12。未完成官方示例不宣称本组SDK支持完成，示例通过也不代真实生产验收。

## 1. 目标

交付一个可按文档复现的普通Auto CLI项目：驱动原版turtlesim，读取Pose、发送cmd_vel、调用服务、执行/取消RotateAbsolute，处理多实例与退出恢复；同源AV开发和新A2R安装发布。它是综合教程与SDK消费者，不代传感器生产应用。

非目标：复刻官方仿真器/算法、修改官方节点来适配SDK、UI/AGE增强、所有ROS发行版/RMW/教程、生产传感器/Autonix全门、新Auto语法或私有ABI。例子与真实业务走同一公共SDK，不按示例名称在编译器/宿主里特判。

## 2. 架构方案

1018官方基础套件正式来源；间接消费1010/1008/1009及1007官方示例输入切片的正式来源，1007-A未合并成果不算前置齐备。Qt/WSLg或Linux显示环境须实际准入；无显示时只有经验证的原版Qt offscreen路径可作自动化补充，不能换mock。无需AutoUI/AGE增强；普通UI工作台仍归1011。

普通Auto源码→1008生成类型→1009公共ROS API/资源/调度→1010已交AV或A2R/provider→真实官方节点/工具。生成声明/emit/CLI shell外包不能代实际Auto求值和宿主调用。SDK公共缺陷回所属owner修复、交正式来源后定向复验，不在示例包做workaround。

## 3. 技术栈与归属

Auto/ACE AV、新A2R、公共ROS SDK、原生ROS官方包及ROSIDL；1007固定SDK实际承载仓、接口/版本和runner。拟新增交付路径为SDK owner内 examples/ros2/turtlesim/ 与相应教程/测试入口，尚未创建。真正路径/CLI须在T-01对照已交SDK冻结，不假定今天已有auto ros2命令；语言仓只承载协调Plan和必要通用修复，跨仓施工须当地AGENTS/owner/Plan先具名。

唯一语言协调实施树拟为D:/autostack/.wt/lang-1019/auto-lang、分支plan-1019-dev，本轮不建。跨仓同组一仓一权威树、不建junction/symlink。材料/原始官方副本/日志/截图/外置target仅D:/autostack/plan-artifacts/plan-1019/，开工登记owner/用途/在途状态；colcon必须显式--log-base/build-base/install-base和产品输出，避免默认log/latest链接进入检出。

## 4. 需求分析与背景调查

本轮主检出HEAD 7b2f2e95967cb3434429093988fae429c05e865f；共享检出有其他会话文档WIP，不纳为本包已交。仅修文档，无ROS/GUI/cargo/新worktree操作。用户未指定工期或模型，不派agent。

读[Specs总览](../specs/overview.md)、[ACE装配现状](../specs/stdlib/design/ace-binding-and-assembly.md)、[M2分母](../specs/ace-migration/m2-denominator.md)，它们不证明当前已能运行完整Auto ROS程序。1007 r2的原生13项通过/1项限制、WSL Humble和rclrs候选是历史可行性输入，不是本包Auto证据；1007未合并分支不满足正式依赖。

官方源码及有限结论记录于Design57 §11，实施前锁commit/package/interface hash而不是只引用移动分支。Humble demo_nodes_cpp注册talker/listener/add_two_ints_server/client；官方examples_rclpy_minimal_action_server/server用example_interfaces/Fibonacci，其反馈字段sequence，与action_tutorials_interfaces同名类型不同；本包不得继承1007的错误接口假设。官方Action server默认accept-all，负order不是通用拒绝oracle。turtlesim的Pose/Spawn/RotateAbsolute使用float32；GUI依Qt事件循环，需要真实运行环境。

## 5. 详细设计

样本/不变量唯一分母为Design57 TS-01..05，本包§8逐项实施。T-01先冻结原生名字/命令、过程身份、数据和数值容差/超时，后续不能据跑出来的值倒改expected或放宽阈值。有限功能样本预算不是产品性能预算。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ros2-sdk/turtlesim-workflow.md（拟新增） | 当前无本组Auto公共SDK交付规范；真实验收后记录支持样本/调用方向/后端/平台/拒绝集/复现入口 | 用官方独立对端证明公共SDK可用，不能把规划或原生结果写成已支持 | AC-01..08 |

若实际SDK另仓拥有规范，T-01据1007决定修订SD-01为当地canonical路径，本仓只保指针，不双份维护。当前不改Specs/legacy JSON。没有经goals.md核实的ROS目标ID，touched_goals保留空。

## 6. 测试设计

内部自测由实现者负责，正式review在同一唯一树独立上下文进行；本轮不分配模型/agent。拟议用例由§8任务建立可运行入口，目前全未执行。

| Case / AC | input / invariant | producer→consumer / entrypoint | distinguishing oracle | coverage boundary |
|---|---|---|---|---|
| V-01 / AC-02 | 冻结spawn初态/运动指令；零指令或错topic/remap | T-02普通Auto→官方/turtle*/cmd_vel；原版Pose/原生CLI观察 | 合法指令位移/方向符合预定容差，负控可区分；不按帧精确bit比较 | 锁定初态和运动时间，非物理仿真精度保证 |
| V-02 / AC-03 | Spawn不同名/重复名，Kill存在/不存在实体，SetPen/Clear/Reset | T-03真实官方Service，生成的turtlesim/std_srvs类型 | 返回名与新增流一致，Kill后流停止；重复名未新建，失败按原版response/timeout/日志分别报告而非假定标准错误码 | 不把空响应统一当成功，视觉画笔项另验 |
| V-03 / AC-04 | RotateAbsolute(theta=1.0候选)及执行中取消/完成竞争 | T-04普通Auto Action client→官方Action server | 归一化角度、remaining/delta/终态在先冻结容差内；cancel ack不等终态，竞争完成不伪报取消 | 源字段float32真实生成；不保证每次取消获胜 |
| V-04 / AC-05 | 双namespace/实例、node重启/缺服务/旧goal | T-05原版两个节点及Auto客户端 | 端点/数据/goal不串线，旧实例结果不接到新请求；自动恢复不重复领域效果 | 本机隔离网络，不代表多机全部QoS |
| V-05 / AC-06 | 干净安装AV/A2R、无Qt插件/显示、文档错误命令 | T-06公开项目/1010产物与ros2 run turtlesim turtlesim_node | 普通源码实际求值并调用SDK；缺环境明确pending或拒绝；画面不是唯一oracle，offscreen不得冒人工可视走查 | 不是AutoUI/AGE集成或R06-10..12 |

未来原生检查入口包括ros2 pkg executables/pkg prefix/interface show/node info/topic echo/service call/action send_goal/param，版本对应命令由T-01冻结；公开SDK源码和AV/A2R执行命令必须基于1010实际交付填写。没有可执行入口的用例保持pending，不虚构cargo t ros2/auto ros2或只造测试名。

SDK样例/ROS系统测试使用1007已冻结owner runner；语言通用修复才按实际修改选完整固定组，例如cargo t ace-source ace-foreign-contract ace-abi ace-host ace-runtime ace-a2x ace-rust-lowering的必要并集，不机械全跑，不用legacy vm/a2r组代ACE新路径。UI/Qt环境用所属owner入口，不跑全AG/全仓。记录列用例/实际执行/排除、首次红/环境归因/重放绿、准备/发现/运行/清理与整命令耗时，核子项终态及零选择。

## 7. 验收标准

| AC | owner_stage | observable result / verification |
|---|---|---|
| AC-01 | work | T-01结果符合§8，按§6对应真实入口/正负控可重放；TS全分母与显示环境实际合同；未具名GUI运行不得自动启动，不能以勾选/编译/截图替运行 |
| AC-02 | work | T-02结果符合§8，按§6对应真实入口/正负控可重放；TS-01：数值变化+原生观察，不只看画面，不能以勾选/编译/截图替运行 |
| AC-03 | work | T-03结果符合§8，按§6对应真实入口/正负控可重放；TS-02：原版服务结果和graph/数据；画笔外观用可视走查补充，不能以勾选/编译/截图替运行 |
| AC-04 | work | T-04结果符合§8，按§6对应真实入口/正负控可重放；TS-03：目标/反馈/结果/取消逐项核对；整数/浮点误映射负控，不能以勾选/编译/截图替运行 |
| AC-05 | work | T-05结果符合§8，按§6对应真实入口/正负控可重放；TS-04：无串实例、未知结果不冒成功，原生Qt节点仍是独立oracle，不能以勾选/编译/截图替运行 |
| AC-06 | work | T-06结果符合§8，按§6对应真实入口/正负控可重放；TS-05与TS-01..04：新用户可复现，自动化与可视走查分别明示，不能以勾选/编译/截图替运行 |
| AC-07 | review | 独立逐项核work AC、TS-01..05、实际diff/提交与SD-01；检查遗漏/未经批准延期/workaround、warnings/format/debug；不以原生probe或本地自测代正式review |
| AC-08 | merge | 后续获准merge时核已审同源合入、规范/教程/Plan摘要与交接来源，完整路径对账和guard/清理归档；当前未授权 |

## 8. 执行步骤

| Task | owner_stage | dependencies | action / new paths / outcome | AC / verification |
|---|---|---|---|---|
| [ ] T-01 | work | §2正式前置及实施授权 | 冻结turtlesim/接口包版本或源码commit、SDK候选/后端、Qt/显示运行方式、命令/namespace/实例名、运动安全初态、角度/位移容差/反馈时限/进程deadline和清理配方；核原版服务失败/Action取消语义；新增样例/教程/runner路径按§3/T-01冻结 | AC-01；TS全分母与显示环境实际合同；未具名GUI运行不得自动启动 |
| [ ] T-02 | work | T-01 | 交付普通Auto Pose订阅/cmd_vel发布源码，生成geometry_msgs/Twist与turtlesim/Pose真实类型；固定无碰墙初态验证指令→运动→姿态，零指令/错remap为可区分负控；新增样例/教程/runner路径按§3/T-01冻结 | AC-02；TS-01：数值变化+原生观察，不只看画面 |
| [ ] T-03 | work | T-02 | 增加Spawn/Kill/SetPen/Clear/Reset工作流和教学说明；返回名、目标Pose端点/流与请求相关；重复名/不存在对象按原版行为报告，不能凭SDK空值宣布成功；新增样例/教程/runner路径按§3/T-01冻结 | AC-03；TS-02：原版服务结果和graph/数据；画笔外观用可视走查补充 |
| [ ] T-04 | work | T-03 | 增加RotateAbsolute客户端，读取remaining/delta/终态；在确认执行中请求取消并核ack/canceled，完成竞争如实处理。Action接口由1008通用生成，不手写float32桥；新增样例/教程/runner路径按§3/T-01冻结 | AC-04；TS-03：目标/反馈/结果/取消逐项核对；整数/浮点误映射负控 |
| [ ] T-05 | work | T-04 | 编排命名空间双实例、参数读回、节点退出/重启、旧goal及服务缺席恢复；隔离状态和错误，不自动重发Spawn或其他有副作用调用；退出时停止发指令并关闭本项目资源；新增样例/教程/runner路径按§3/T-01冻结 | AC-05；TS-04：无串实例、未知结果不冒成功，原生Qt节点仍是独立oracle |
| [ ] T-06 | work | T-05 | 整理完整Auto教程与同源AV/A2R项目；通过1010包机制从干净消费者启动、运行TS全部正反例并核退出；Qt画面/数据/错误结果分别记录；不复制新引擎或单建GUI框架；新增样例/教程/runner路径按§3/T-01冻结 | AC-06；TS-05与TS-01..04：新用户可复现，自动化与可视走查分别明示 |
| [ ] T-07 | review | 所有work任务完成且冻结提交 | 在同一权威树独立审查全样本/AC/SD-01；必要反例定向核验，具名遗漏/延期/workaround记docs/plans/KNOWN-DEBT-AND-RISKS.md | AC-07；独立revision/commit绑定收据 |
| [ ] T-08 | merge | 独立pass及未来merge授权 | 合入来源、规范/教程/Plan摘要交下游；材料全路径对账，guard clean后删除唯一树/分支/scope并按本仓archive归档 | AC-08；来源/hash/清理收据 |

提交、review、merge前不重复无关全目录测试；复用身份/输入未变有效证据，修复只使实际受影响腿失效。下游只消费已审已落主来源；不把分支PASS当完成交付。

## 9. 复审记录

2026-11-12：stage=new，revision=1；outcome=pass（草案合同/链接/依赖DAG/任务与AC覆盖/表格核对），非独立功能review或开工许可。execution_authorized=false，next=保持drafting，按§2正式依赖及实施指令进入work。T/AC/SD均为新合同，全部未执行，不声明功能PASS/签章。

## 10. 待澄清事项

Qt/显示/offscreen方式与turtlesim安装状态本轮未检查；T-01核现有环境并冻结。不需要用户选真实传感器才能规划本包；实际设备/性能与Autonix仍留1011/1012，不用turtlesim替代。

这些未决点由T-01具名关闭；不因未决就默认弃掉required。新平台/验收语义改变须修约，保留原输入/结果。正式实施需要另行指令；本次不向用户追加无必要确认。
