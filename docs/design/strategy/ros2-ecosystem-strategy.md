# ROS2 生态：Auto SDK 当前方向与历史方案

## 当前方向（2026-11-12）

数据路径增补：按[Design57 §12](../57-auto-ros2-sdk.md#12-ros2传感器数据的专门适配与低复制合同2026-11-12仅规划)，ROS传感器的连续buffer/元数据、借用与owner、低复制/无多余重编码及实际计量是v0.6基础义务；RMW loan/shared-memory仍需按实际消息和部署证实。本轮仅修合同，未运行新探针。

官方首组已纳入路线：[1018官方基础示例](../../plans/1018-ros2-sdk-official-examples-conformance.md)→[1019 turtlesim完整Auto教程](../../plans/1019-ros2-sdk-turtlesim-tutorial-and-interoperability.md)→1011真实传感器工作流。原生预验证复用1007，不另建探针计划；正式示例必须经同一公共SDK/类型生成与AV/A2R，不作为生产传感器/Autonix验收替代。

执行增补：PLAN-1007已修订r4，用户授权A批在独立worktree提前验证原生ROS2/provider/ROSIDL与部署研究，暂不合并、不改ACE核心、不绕M1-TAG；B、1008–1012及1018/1019未实施。下面“仅规划/均drafting”描述初稿，实际进度以1007当前合同为准。

用户要求v0.6交付“能用于实际工作的Auto ROS2 SDK”，本轮仅专项设计和分阶段规划，不实施。权威目标与验收见[Design57](../57-auto-ros2-sdk.md)，执行拆包见[SDK交付路线](ros2-sdk-delivery-roadmap.md)与PLAN-1007..1012（均drafting）。普通Auto程序直接作为ROS2节点，消费ACE通用FFI/调度/构建；AutoOS系统通信adapter为另一入口，不再作为所有SDK工作的总前置。机器人新算法仍另议。

v0.6必做包括类型生成、Topic/Service/Action、参数/QoS/时间、AV开发/A2R发布、ROS工程工具、真实任务与AutoUI、Autonix安装运行和质量验收。AGE增强为可选消费者；Linux实际底座为NixOS＋COSMIC，不能用Ubuntu跑通代发行验收。未冻结设备/版本/provider/预算由1007承接，不宣称已有SDK。

以下2026-10-16及更早内容为历史讨论；“通信框架→有限bridge”不再限定新版SDK的全部范围/开工顺序，“C轨暂缓期间不正式投入”不覆盖本轮新目标。其他系统通信与算法边界仍按各自有效合同执行。

## 当前方向与阶段合同（2026-10-16）

AutoOS 自己拥有应用通信/能力模型，以 adapter 兼容 ROS2；ROS2 不作为所有应用的强制底层。[系统阶段合同](system-foundations-v06.md)复用 auto-os 的通信/DevTools RFC。下面2026-09-06的“C绑定包”是历史接入方案，不再穷尽整个通信战略；rcl/rclpy等仍可作为 bridge 实现候选。

用户最新约束：先做 AutoOS 跨进程通信框架，再开展具体机器人算法；本轮不把 BEV/SLAM、训练、板级模型部署或机器人算法示例列为 v0.6 本次整理的工作项。机器人市场长期定位保留。

分期为：F0/F1本地消息/调用合同与两进程实测→F2任务/取消/事件/流→ROS2语义映射及兼容bridge→另行规划机器人算法。ROS2映射调研可以先记录，不以此提前启动算法。
既有“ROS2生态完整支持”目标保留为历史已确认版本意图；具体兼容支持集尚未冻结，不能由单个bridge demo冒称完成。用户此次已重排前置，本轮不单方面删掉兼容目标，也不让未选算法成为框架验收条件。

| 阶段 | 验收/待冻结范围 | 拆分边界 |
|---|---|---|
| 通信基础 | 真实进程发现/调用、版本/权限拒绝、重启失效、截止与取消/回执，按系统阶段文档逐项验收 | 独立DTO/client/server先做；旧Task/Msg及VM接线协调语言SDK |
| ROS2映射调研 | Topic/Service/Action、参数、类型/codec、QoS、发现、取消/反馈逐项说明可映射与缺口；相同transport不视为天然兼容 | 不改旧parser或强迫所有App依赖ROS；ABI/binding按公开合同 |
| 有限兼容原型 | 通信基础成立后，独立AutoOS端与真实ROS2端互通一组Topic/Service/Action；检查值/顺序/超时/取消、断线与不支持的QoS。不能用双方mock代替 | 固定ROS/平台/中间件版本；资源与工具兼容矩阵由Plan冻结 |

后续“完整支持”的实际矩阵与版本容量需再确认；跨节点Actor集群、micro-ROS与具体机器人算法未自动纳入首个原型。以下原文保留其日期，实施不直接沿用旧排序。

**日期**: 2026-09-06
**状态**: Draft（骨架，待扩充）
**关联**: [ecosystem-portfolio-strategy](ecosystem-portfolio-strategy.md)（伞形总战略·三市场
框架中"机器人"市场的主轨之一）、[c-ecosystem-strategy](c-ecosystem-strategy.md) §3/§4（消费面一/二）

---

## 0. 定性

ROS2 支持**不是新发射臂/新后端**，而是 C 战略消费面一（Linux 系统库消费者）的一个
**生态绑定包 + 代码生成器**：

- ROS2 核心库 **rcl 是纯 C**（rclcpp/rclpy 皆其封装）→ Auto 经 `#[c]`/auto-bindgen
  直接绑定 rcl，消费面一的产能直接复用。
- `.msg` / `.srv` / `.action` IDL → Auto struct/comptime 代码生成，与 auto-gen 契合。
- **micro-ROS**（XRCE-DDS）把 ROS2 概念延伸到 MCU → 又接消费面二（嵌入式）。
- 具身 AI 载体与 Auto 的 AI 原生定位共振：机器人应用层（行为树/任务编排/监控面板——
  后者可吃 AutoUI 资产）是 Auto 的天然场景。

### 双形态兑现（动静结合在本生态的落地，2026-09-06 补）

- **动态态（开发）**：Auto 直接调用——VM 经 rcl host shim 直调；或生成 Python
  （a2py→rclpy）——快速开发调试，node/topic 即时起停。
- **静态态（发布）**：转成 C/Rust 对接 ROS 的 C 接口（rcl）——降低系统消耗、提高 ROS
  模块运行效率。

## 1. 已定裁决（2026-09-06 讨论）

1. ROS2 立为 **C 轨重启触发条件之一**；预定为 C Stage 1 后的**首个外部生态绑定试点**。
2. C 轨暂缓期间：最多用 use.py 调 rclpy 做**需求原型**，不做正式投入。

## 2. 待扩充

- [ ] rcl 绑定面清单（node/topic/service/param/action 各域 C API 覆盖）
- [ ] 开发态桥选型：VM rcl shim vs a2py→rclpy（双臂成本与调试体验对拍）
- [ ] .msg/.srv → Auto struct 生成管线设计（auto-gen 集成）
- [ ] DDS QoS 语义到 Auto 的映射约定
- [ ] 旗舰叙事：机器人脑编排（Auto 编排 + PyTorch 感知模型 + a2r/a2c 发布）
- [ ] micro-ROS 与消费面二（MCU）的交汇验证场景
