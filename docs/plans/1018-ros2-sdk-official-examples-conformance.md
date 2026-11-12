---
plan_id: PLAN-1018
status: drafting
feature_name: ROS2官方基础示例的Auto SDK双端验收
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 0
total_steps: 9
depends_on: [PLAN-1010]
supersedes_spec_components: []
new_spec_components: [docs/specs/ros2-sdk/official-examples.md]
touched_goals: []
---

# [PLAN-1018] ROS2官方基础示例的Auto SDK双端验收

## 0. 变更摘要

2026-11-12 r1，仅规划，0/9未执行。用户确认官方基础示例→turtlesim→传感器工作流的顺序，要求第一组计划并纳入ROS2路线。此次授权只含设计/计划/导航，不含运行/安装/GUI/接口生成、建实施树、提交、merge或发布。1007既有A-only/no-merge授权独立保留，不因本包草案扩大。

权威：[Design57 §11](../design/57-auto-ros2-sdk.md#11-官方示例首组合同2026-11-12增补仅规划)、[专项路线](../design/strategy/ros2-sdk-delivery-roadmap.md)。本包负责OF-01..05，作为R06-02..09具体消费者验证；不替R06-10..12。未完成官方示例不宣称本组SDK支持完成，示例通过也不代真实生产验收。

## 1. 目标

用普通Auto公共SDK实现官方基础示例对应源码，形成双向Topic/Service/Action、参数/remap和干净工程的可执行教学与兼容性套件。连接未经修改的官方C++/Python节点，独立核结果；只做CLI不证明Auto支持，原生1007探针不核销本包。

非目标：复刻官方仿真器/算法、修改官方节点来适配SDK、UI/AGE增强、所有ROS发行版/RMW/教程、生产传感器/Autonix全门、新Auto语法或私有ABI。例子与真实业务走同一公共SDK，不按示例名称在编译器/宿主里特判。

## 2. 架构方案

1010公共SDK、AV与新A2R/打包已审已落主；间接消费1007已审已落主的官方示例输入切片来源（B05/B06/B07）、1008接口生成、1009运行时。M1 tag和967/968/978等仍按原Plan进入；本包不补ACE核心来绕依赖，不等待AGE或全部AutoUI。

普通Auto源码→1008生成类型→1009公共ROS API/资源/调度→1010已交AV或A2R/provider→真实官方节点/工具。生成声明/emit/CLI shell外包不能代实际Auto求值和宿主调用。SDK公共缺陷回所属owner修复、交正式来源后定向复验，不在示例包做workaround。

## 3. 技术栈与归属

Auto/ACE AV、新A2R、公共ROS SDK、原生ROS官方包及ROSIDL；1007固定SDK实际承载仓、接口/版本和runner。拟新增交付路径为SDK owner内 examples/ros2/official-basics/ 与相应教程/测试入口，尚未创建。真正路径/CLI须在T-01对照已交SDK冻结，不假定今天已有auto ros2命令；语言仓只承载协调Plan和必要通用修复，跨仓施工须当地AGENTS/owner/Plan先具名。

唯一语言协调实施树拟为D:/autostack/.wt/lang-1018/auto-lang、分支plan-1018-dev，本轮不建。跨仓同组一仓一权威树、不建junction/symlink。材料/原始官方副本/日志/截图/外置target仅D:/autostack/plan-artifacts/plan-1018/，开工登记owner/用途/在途状态；colcon必须显式--log-base/build-base/install-base和产品输出，避免默认log/latest链接进入检出。

## 4. 需求分析与背景调查

本轮主检出HEAD 7b2f2e95967cb3434429093988fae429c05e865f；共享检出有其他会话文档WIP，不纳为本包已交。仅修文档，无ROS/GUI/cargo/新worktree操作。用户未指定工期或模型，不派agent。

读[Specs总览](../specs/overview.md)、[ACE装配现状](../specs/stdlib/design/ace-binding-and-assembly.md)、[M2分母](../specs/ace-migration/m2-denominator.md)，它们不证明当前已能运行完整Auto ROS程序。1007 r2的原生13项通过/1项限制、WSL Humble和rclrs候选是历史可行性输入，不是本包Auto证据；1007未合并分支不满足正式依赖。

官方源码及有限结论记录于Design57 §11，实施前锁commit/package/interface hash而不是只引用移动分支。Humble demo_nodes_cpp注册talker/listener/add_two_ints_server/client；官方examples_rclpy_minimal_action_server/server用example_interfaces/Fibonacci，其反馈字段sequence，与action_tutorials_interfaces同名类型不同；本包不得继承1007的错误接口假设。官方Action server默认accept-all，负order不是通用拒绝oracle。turtlesim的Pose/Spawn/RotateAbsolute使用float32；GUI依Qt事件循环，需要真实运行环境。

## 5. 详细设计

样本/不变量唯一分母为Design57 OF-01..05，本包§8逐项实施。T-01先冻结原生名字/命令、过程身份、数据和数值容差/超时，后续不能据跑出来的值倒改expected或放宽阈值。有限功能样本预算不是产品性能预算。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ros2-sdk/official-examples.md（拟新增） | 当前无本组Auto公共SDK交付规范；真实验收后记录支持样本/调用方向/后端/平台/拒绝集/复现入口 | 用官方独立对端证明公共SDK可用，不能把规划或原生结果写成已支持 | AC-01..09 |

若实际SDK另仓拥有规范，T-01据1007决定修订SD-01为当地canonical路径，本仓只保指针，不双份维护。当前不改Specs/legacy JSON。没有经goals.md核实的ROS目标ID，touched_goals保留空。

## 6. 测试设计

内部自测由实现者负责，正式review在同一唯一树独立上下文进行；本轮不分配模型/agent。拟议用例由§8任务建立可运行入口，目前全未执行。

| Case / AC | input / invariant | producer→consumer / entrypoint | distinguishing oracle | coverage boundary |
|---|---|---|---|---|
| V-01 / AC-02 | 官方talker→Auto，Auto→官方listener；合法QoS和故意不匹配 | T-02普通Auto源码→1010公开AV/A2R执行入口；官方ros2 run demo_nodes_cpp talker/listener | 两独立端实际收到/发出冻结样本；不匹配不交付且正控仍通 | 官方String有限样本，非大消息性能 |
| V-02 / AC-03 | AddTwoInts固定41+1、-17+5及服务缺席/超时 | T-03两方向实际server/client | 得到42/-12；迟响应不串新请求，超时不宣称远端未执行 | 算术业务无副作用，不由此承诺exactly-once |
| V-03 / AC-04 | 官方Fibonacci order4及执行中取消；Auto fixture具名拒绝策略 | T-04官方server/client/client_cancel与Auto反方向 | 官方Humble server按源码产生[0,1,1,2,3]；反馈/goal/result匹配；cancel request/ack/canceled分列；拒绝只对声明策略核 | 版本变动先冻结等义oracle，不改官方实现 |
| V-04 / AC-05 | 两个namespace、错remap、参数错类型 | T-05原生参数服务/CLI和SDK实例 | 读回/日志/端点明确；错实例不收到目标数据，错误不变成功 | 锁定平台/接口 |
| V-05 / AC-06/07 | 干净工作区、缺包/错type support、正常关闭及子进程非零 | T-06/07公开教程/构建安装/AV/A2R入口 | 只用发行SDK与依赖即可重建运行；零选择/子项失败不能整体PASS，关闭进程可核 | 不验证整套Autonix，留1012 |

未来原生检查入口包括ros2 pkg executables/pkg prefix/interface show/node info/topic echo/service call/action send_goal/param，版本对应命令由T-01冻结；公开SDK源码和AV/A2R执行命令必须基于1010实际交付填写。没有可执行入口的用例保持pending，不虚构cargo t ros2/auto ros2或只造测试名。

SDK样例/ROS系统测试使用1007已冻结owner runner；语言通用修复才按实际修改选完整固定组，例如cargo t ace-source ace-foreign-contract ace-abi ace-host ace-runtime ace-a2x ace-rust-lowering的必要并集，不机械全跑，不用legacy vm/a2r组代ACE新路径。UI/Qt环境用所属owner入口，不跑全AG/全仓。记录列用例/实际执行/排除、首次红/环境归因/重放绿、准备/发现/运行/清理与整命令耗时，核子项终态及零选择。

## 7. 验收标准

| AC | owner_stage | observable result / verification |
|---|---|---|
| AC-01 | work | T-01结果符合§8，按§6对应真实入口/正负控可重放；source/环境manifest与OF逐行输入输出合同，不能以勾选/编译/截图替运行 |
| AC-02 | work | T-02结果符合§8，按§6对应真实入口/正负控可重放；OF-01：原生端与Auto端各自读回，AV/A2R同源，不能以勾选/编译/截图替运行 |
| AC-03 | work | T-03结果符合§8，按§6对应真实入口/正负控可重放；OF-02：精确和/响应相关性；不能只依赖父进程exit0，不能以勾选/编译/截图替运行 |
| AC-04 | work | T-04结果符合§8，按§6对应真实入口/正负控可重放；OF-03：按example_interfaces实际序列oracle，避免混同action_tutorials_interfaces，不能以勾选/编译/截图替运行 |
| AC-05 | work | T-05结果符合§8，按§6对应真实入口/正负控可重放；OF-04：参数/实例隔离和无错态吞没，不能以勾选/编译/截图替运行 |
| AC-06 | work | T-06结果符合§8，按§6对应真实入口/正负控可重放；OF-05：公开SDK前门与安装后的消费者实际可用，不能以勾选/编译/截图替运行 |
| AC-07 | work | T-07结果符合§8，按§6对应真实入口/正负控可重放；OF-01..05全分母，两后端分列结果，任何required未执行保持pending，不能以勾选/编译/截图替运行 |
| AC-08 | review | 独立逐项核work AC、OF-01..05、实际diff/提交与SD-01；检查遗漏/未经批准延期/workaround、warnings/format/debug；不以原生probe或本地自测代正式review |
| AC-09 | merge | 后续获准merge时核已审同源合入、规范/教程/Plan摘要与交接来源，完整路径对账和guard/清理归档；当前未授权 |

## 8. 执行步骤

| Task | owner_stage | dependencies | action / new paths / outcome | AC / verification |
|---|---|---|---|---|
| [ ] T-01 | work | §2正式前置及实施授权 | 冻结官方包/commit/接口内容、SDK和两后端身份、命令/namespace/remap/隔离域、有限消息数/每例deadline/日志上限、安装和清理配方；核官方消息/Action真实语义及fixture归属；新增样例/教程/runner路径按§3/T-01冻结 | AC-01；source/环境manifest与OF逐行输入输出合同 |
| [ ] T-02 | work | T-01 | 实现Auto订阅官方talker以及官方listener消费Auto发布的普通源码样例，独立进程校验文本/序列和QoS负控，保留节点发现/连接失败结果；新增样例/教程/runner路径按§3/T-01冻结 | AC-02；OF-01：原生端与Auto端各自读回，AV/A2R同源 |
| [ ] T-03 | work | T-02 | 实现Auto客户端→官方AddTwoInts server、官方client→Auto server，固定正负整数精确结果；补服务未就绪、本地timeout/迟响应关联反例，不自动重试有副作用请求；新增样例/教程/runner路径按§3/T-01冻结 | AC-03；OF-02：精确和/响应相关性；不能只依赖父进程exit0 |
| [ ] T-04 | work | T-03 | 实现Auto Action client→官方Fibonacci server及官方client/client_cancel→Auto server；逐goal核反馈/结果与cancel回执/终态。需要拒绝策略时用明示的公共SDK fixture，不强迫原版accept-all server拒绝负order；新增样例/教程/runner路径按§3/T-01冻结 | AC-04；OF-03：按example_interfaces实际序列oracle，避免混同action_tutorials_interfaces |
| [ ] T-05 | work | T-04 | 补Auto参数get/set/类型拒绝与原生工具读回、namespace/remap/两实例、graph查询和close。use_sim_time只读/重建政策按SDK明确声明，不把不支持映射为成功；新增样例/教程/runner路径按§3/T-01冻结 | AC-05；OF-04：参数/实例隔离和无错态吞没 |
| [ ] T-06 | work | T-05 | 整理普通Auto项目源码/运行和生成发布recipe、依赖锁、ROS run/launch/失败排查说明；不手填函数ABI或示例专用native。产物进入1010已交包机制，不由样例另造SDK构建链；新增样例/教程/runner路径按§3/T-01冻结 | AC-06；OF-05：公开SDK前门与安装后的消费者实际可用 |
| [ ] T-07 | work | T-06 | 同源AV/A2R执行全部OF及必需反例；在干净消费者按教程运行，核节点/子进程真实退出、缺包/错类型支持拒绝和可追溯报告；复用未变有效证据，提交实现供独立复审；新增样例/教程/runner路径按§3/T-01冻结 | AC-07；OF-01..05全分母，两后端分列结果，任何required未执行保持pending |
| [ ] T-08 | review | 所有work任务完成且冻结提交 | 在同一权威树独立审查全样本/AC/SD-01；必要反例定向核验，具名遗漏/延期/workaround记docs/plans/KNOWN-DEBT-AND-RISKS.md | AC-08；独立revision/commit绑定收据 |
| [ ] T-09 | merge | 独立pass及未来merge授权 | 合入来源、规范/教程/Plan摘要交下游；材料全路径对账，guard clean后删除唯一树/分支/scope并按本仓archive归档 | AC-09；来源/hash/清理收据 |

提交、review、merge前不重复无关全目录测试；复用身份/输入未变有效证据，修复只使实际受影响腿失效。下游只消费已审已落主来源；不把分支PASS当完成交付。

## 9. 复审记录

2026-11-12：stage=new，revision=1；outcome=pass（草案合同/链接/依赖DAG/任务与AC覆盖/表格核对），非独立功能review或开工许可。execution_authorized=false，next=保持drafting，按§2正式依赖及实施指令进入work。T/AC/SD均为新合同，全部未执行，不声明功能PASS/签章。

## 10. 待澄清事项

1007官方示例输入切片的SDK仓、API/CLI、ROS实验矩阵未定；T-01消费其已审已落主决定才写精确Auto命令，不等待最终传感器设备/生产预算或Autonix合同。现有WSL Humble只是已调查输入，不自动扩大支持平台。官方包缺失要具名依赖，不能改成自制mock并报告通过。

这些未决点由T-01具名关闭；不因未决就默认弃掉required。新平台/验收语义改变须修约，保留原输入/结果。正式实施需要另行指令；本次不向用户追加无必要确认。
