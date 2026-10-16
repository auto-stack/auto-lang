# ROS2 生态兼容：通信先行与 v0.6 阶段边界

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
