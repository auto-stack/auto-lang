# ROS2 生态战略（C 消费面的生态绑定包）

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
