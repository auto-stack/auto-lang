# 生态版图总战略（Ecosystem Portfolio Strategy）

**日期**: 2026-09-06
**状态**: Draft（伞形纲领；各分战略为骨架，"待扩充"节为后续填充区）
**关联**: 各分战略/母纲文档（见 §1 总表）；
[GOAL-003/004/005/012/013](../../specs/goals.md)

---

## 0. 统一透镜：动静结合第一优势、三大市场、寄生期合作层原则

### 核心战略优势：动静结合（2026-09-06 定为全生态设计原点）

**开发态（动态）**：VM 解释执行——Python 级开发效率（1s 周转、热重载、REPL/LSP、AI 原生
交互）。**发布态（静态）**：转译到 C/Rust——系统级性能与部署形态。**一份源码、两态同源**，
parity 机器保证行为一致。

竞争位形：Python 只有前半段，C/Rust 只有后半段；Auto 两者原生兼得。这是仓库既有教义
（"脚本开发→转译发布"双形态，GOAL-003）从**语言形态**升格为**全生态竞争论**。

**进入任何生态的设计原点**——立项前必须回答两问：该生态里 ① 动态态怎么跑（开发调试）？
② 静态态怎么发（性能/部署）？**两问都有答案才立项。** 各生态的兑现方式：

| 生态 | 动态态（开发） | 静态态（发布） |
|---|---|---|
| 语言主线 | VM 直跑 .at | a2r→Rust / a2c→C |
| AutoUI | VM/iced 与 Vue dev 实时预览 | 转译发布（a2r 桌面 / a2ts Web 构建） |
| C 系统消费（消费面一） | VM host shim 直调 libc | a2c 直发 C |
| C 嵌入式（消费面二） | VM 即模拟器 + QEMU | a2c 交叉编译烧录 |
| ROS2 | VM 直调 / 生成 Python（rclpy） | a2c/a2r 对接 rcl C 接口 |
| 第三代互联网（Godot） | 发射 GDScript（编辑器热改） | 静态 C/Rust 缝合引擎 |
| Python AI 线 | use.py 调 PyTorch（本身即动态态） | 编排层经 a2r 发布服务 |

### 三大市场（版图的最终组织原则）

Auto 生态（AI·Lang·OS）瞄准未来三大市场：**AI + 机器人 + 第三代互联网**。

**需求侧叙事（2026-09-06）**：以人作比——**AI 替代白领（认知劳动），机器人替代蓝领
（物理劳动），第三代互联网容纳被替代的所有人**（注意力/身份/生活的去处）。这是三市场
选择的需求侧对偶论证（供给侧 = 三根技术支柱一一对应），并揭示三者构成闭环飞轮：AI+机器人
产出富余（认知与体力成本趋零）→ 释放的人类注意力流入第三代互联网 → 第三代互联网
（仿真/数字孪生）同时是 AI 与机器人的训练场——**人的去处与机器的训练场在此交汇**。
时序上 AI 已至、机器人爬坡、第三代互联网需求随替代规模而来，与各轨道热度（Python/AI 线
最活跃、C/ROS2 等触发、Godot 等窗口）互为映射；而编程本身即被替代的白领工种之一——
Auto 作 AI 原生语言，是为"替代者"造笔，产品论点与市场论点在此咬合。

**市场更名裁决（2026-09-06）**：第三市场定名**第三代互联网**（曾用名/俗名"元宇宙"
"WEB3.0"）——"元宇宙"被区块链/VR 绑定过窄（区块链已排除，VR 仅为接入设备之一）；
不用"下一代互联网"（移动靶命名，且与 CNGI/IPv6 历史撞名）。定义与更名理由见
[thirdgen-internet-strategy](thirdgen-internet-strategy.md) §0。

| 市场 | 承担轨道 |
|---|---|
| AI | Python 消费轨（PyTorch）+ AI 原生语言内核 + AutoOS/AI 基础设施（Design 15/16） |
| 机器人 | [ROS2 绑定包](ros2-ecosystem-strategy.md) + C 嵌入式消费面（MCU / micro-ROS） |
| 第三代互联网（AI+XR 沉浸式；曾用名"元宇宙/WEB3.0"） | [Godot/3D 引擎入口](thirdgen-internet-strategy.md) |

母生态（Rust，自举基座）与 AutoUI（横向渲染能力）是三个市场共享的底座，不单独属于某个
市场；native 编译是三个市场共同的终局执行底座。

**定性裁决（2026-09-06）**：三者是**跨行业生态，不是行业**——各自横切所有行业，且与
Auto 的三根技术支柱一一对应。行业级扩展（航天、生物医药、智能汽车、量化金融等）**暂不
纳入**：这些行业的软件需求可由现有支柱分解覆盖，无新增支柱则不新增市场；未来某行业出现
Auto 切入路径清晰的真实拉力时，再以"行业垂直"形态评估（挂靠对应轨道），而非扩市场框架。

**供给侧"+1"（2026-09-06，同级候选扫描后裁定）**：三大市场是需求侧框架；扫描同级候选后
唯一认真的候选——**半导体/芯片设计自动化（EDA/HDL/RISC-V）——定性为供给侧底座而非第四
市场**：它是三大市场的算力上游，且与既有裁定连续（native 必做理由 d、GPU/NPU 的
Triton/TileLang 路线、自有硬件/RISC-V 倾向），产业锚点落在
[native-backend-strategy](native-backend-strategy.md)。框架由此定型为"**需求侧 3 +
供给侧 1**"。其余候选裁决：**量子计算**=远期观察项（语言维度真实、经典-量子混合架构
与双形态天然映射，但时间线十年以上）；**智能合约/区块链**=不纳入（裁定：区块链不是
下一代互联网的基础，见第三代互联网定义）；**科学计算**=并入 AI 市场子面——Julia 的"双语言
问题"与动静结合同款、证明痛点真实，但其十年未能撼动 Python 的教训（生态引力 > 语言
优势，语言优势不自动转化为生态位）正是寄生期策略"先消费后自主"的正面依据；
**云原生/基础设施**=当下互联网的底座、寄生期宿主，Rust/C 线天然服务，不升格。

### 寄生期合作层原则（阶段裁决，非终局）

**现阶段（寄生期）**，Auto 在每个平台的接入层选在**厂商/官方工具链拥有的最高稳定抽象层**
——源码级发射（官方编译器负责向下）或库级消费（FFI 调官方库）；不碰厂商内部 IR/字节码、
不自己做机器码发射、不建并行 LLVM 后端。每下沉一层 = 放弃继承整个工具链生态
（调试/预览/热重载/认证/组件库），自己背负一个编译器后端。

**终局（自主生态期）native 直发必做**——系统语言资格、全链路优化空间、独立生态、自有
硬件四条理由，及路线选型（LLVM/自研 emitter/cranelift/MIR；GPU/NPU 的 Triton/TileLang
模式）见 [native-backend-strategy](native-backend-strategy.md)。合作层原则是寄生期的
最优解，不是对 native 路线的否定。

### 生态战略函数（每个生态对 Auto 的作用，可多重）

| 函数 | 含义 | 当前承担者 |
|---|---|---|
| 自举基座 | 编译器/VM 实现语言、自举参照实现 | Rust |
| 发布轨 | 性能/部署形态的转译目标 | Rust（a2r）；C（a2c，暂缓） |
| AI 调用面 | 调用 AI/科学计算生态 | Python（PyTorch 方向） |
| UI 渲染臂 | AutoUI 的发射目标 | Vue、VM/iced、ArkTS（待重启）、Jetpack（待重启）、LVGL（远期） |
| 市场入口 | 三大市场之一的生态切入点 | Godot（第三代互联网）、ROS2+嵌入式（机器人） |
| 市场增量 | 新窗口期生态 | 鸿蒙 |
| 自主底座 | 独立生态的原生执行能力（终局必做） | 无（远期：[native-backend-strategy](native-backend-strategy.md)） |

---

## 1. 版图总表（优先级序）

| 序 | 生态/轨道 | 战略函数 | 分战略/母纲文档 | 状态 | 近期动作 |
|---|---|---|---|---|---|
| 1 | Rust | 自举基座 + 发布轨 | [auto-as-rust-script-strategy](auto-as-rust-script-strategy.md) + [rust-library-replication-roadmap](rust-library-replication-roadmap.md) | ✅ 现行主线 | 按 532/539 等在途 plan 推进；守"自举后单头演进"纪律（§2.1） |
| 2 | AutoUI Web(Vue) + Desktop | UI 主臂 | 本文档 §2.2 + GOAL-007 线 | 交付主线 | GOAL-007 收口优先于一切新臂 |
| 3 | 鸿蒙 ArkTS | UI 渲染臂 + 市场增量 | [harmonyos-ecosystem-strategy](harmonyos-ecosystem-strategy.md) | 📝 Draft 骨架 | a2ark rebaseline 恢复跑通（该线首个 plan） |
| 4 | Python（PyTorch 消费） | AI 调用面 | [python-parity-roadmap](python-parity-roadmap.md)（定位更新见本文档 §2.3） | ✅ 现行 | 539/569/570 线推进；旗舰叙事=三生态协作应用 |
| 5 | Mobile（Jetpack 统一） | UI 渲染臂 | [mobile-jetpack-strategy](mobile-jetpack-strategy.md) | 📝 Draft 骨架 | 排鸿蒙后；a2jet rebaseline + CMP 跟进 |
| 6 | Web React 臂 | UI 渲染臂扩展 | [web-ecosystem-strategy](web-ecosystem-strategy.md) | 📝 Draft 骨架 | GOAL-007 收口后立项 |
| 7 | C 生态 | 发布轨（暂缓） | [c-ecosystem-strategy](c-ecosystem-strategy.md) | 📝 Draft（暂缓裁决在案） | 不动；三重启触发条件（其 §11） |
| 8 | ROS2 | C 消费面绑定包 | [ros2-ecosystem-strategy](ros2-ecosystem-strategy.md) | 📝 Draft 骨架 | C 轨重启触发条件之一 |
| 9 | Godot（第三代互联网入口） | 市场入口 | [thirdgen-internet-strategy](thirdgen-internet-strategy.md) | 📝 Draft 骨架 | 定性已升级（非 showcase）；投入时序与近期主线权衡、季度规划定；立项前置=缝合性能实测 |
| 10 | Native 编译 | 自主底座（终局必做） | [native-backend-strategy](native-backend-strategy.md) | 📝 Draft 骨架 | 选型届时战略级分析；触发条件在案（其 §2） |

---

## 2. 各生态摘要（详细裁决在各分战略）

### 2.1 Rust：母生态，双角色，无需新裁决

- **实现语言与自举基座**：编译器/VM 为 Rust 实现；GOAL-017（AAVM，`auto/lib/` .at 实现）
  在途，自举完成后 Rust 编译器退为 canonical reference。
- **第一发布轨**：a2r（23k 行、20+ pass）+ a2r-std + 三方 parity 机器（VM/a2r/原生 Rust），
  GOAL-003/004 双线。
- **纪律**：自举完成后防止 Rust 编译器与 `auto/lib/` 的 .at 实现长期双头演进（参照单头）。
- **原型模板**：a2r 走过的路（parity 补课→语料驱动→消费者模式）是每个新发射臂的标准路径。

### 2.2 Web：契约独立、发射臂自由；Vue 主臂不动摇

- **升格为原则**：AutoUI 契约（schema/design tokens/行为规范，527 清单驱动路线）独立于任何
  宿主框架；Vue/VM-iced 是当前两臂，React/ArkTS 等为臂的增删而非生态重建。
- Vue 主臂地位维持（中国市场论据 + 沉没投资最深）；React 臂 = GOAL-007 收口后的**发射臂
  扩展**（shadcn 本为 React 系、schema 复用）；Svelte 观察。
- 自研浏览器运行时（VM→WASM）不做，保留为远期"VM 第三发射域"选项。详见
  [web-ecosystem-strategy](web-ecosystem-strategy.md)。

### 2.3 Python：消费桥定性，聚焦 AI/科学计算

- 定性边界：Python 轨是**消费桥**（use.py 调 PyTorch/numpy）而非发布目标；a2py 反向转译
  维持次要（语料迁移/教学）；不做全面 Python parity（Web 框架等不投资）。
- 旗舰叙事：三生态协作应用——Auto 编排 + PyTorch 计算（use.py）+ a2r 发布 Rust 服务，
  即"AI 原生多目标"的可执行证明。
- 定位更新待扩充期吸收进 [python-parity-roadmap](python-parity-roadmap.md)。

### 2.4 鸿蒙：下一主战场，ArkTS 主线

四条路线裁决：ArkTS 主线（合作层原则 + 与 a2ts 机制共享）；仓颉观察不做；ABC 字节码不做；
机器码不做。OpenHarmony 独立 OS = 产品发行版决策（技术同栈 ArkUI），登记终态方向不立项。
三线交汇（UI 轨 / C 轨 Lite 设备 / GOAL-009 shell 肌肉）是优先级高于 Mobile 的技术论据。
详见 [harmonyos-ecosystem-strategy](harmonyos-ecosystem-strategy.md)。

### 2.5 Mobile：Android+iOS 统一 Jetpack Compose

一份 a2jet 发射臂经 Compose Multiplatform 覆盖 Android/iOS/JVM 桌面；不做 SwiftUI 臂。
风险在案：CMP on iOS 成熟度（局部原生桥补缺）；"iOS 中国份额下降"为前提假设（全球化时
重审）。详见 [mobile-jetpack-strategy](mobile-jetpack-strategy.md)。

### 2.6 C：见分战略（暂缓裁决与三重启触发条件在其 §11）

### 2.7 ROS2：C 消费面的生态绑定包

rcl 为纯 C → ROS2 支持是 C 战略消费面一的绑定包 + IDL 代码生成，非新发射臂；micro-ROS
接消费面二（MCU）；具身 AI 与 Auto 的 AI 原生定位共振。详见
[ros2-ecosystem-strategy](ros2-ecosystem-strategy.md)。

### 2.8 Godot：第三代互联网生态入口（2026-09-06 定性修正 + 更名）

Godot（或其他 2D/3D 引擎）是三大市场之一**第三代互联网**（曾用名"元宇宙"）的入口点——
Auto→GDScript 不是 showcase，而是生态切入。Auto 在该生态的差异化：**开发期发射动态
GDScript、发布期发射静态 C/Rust 缝合**，双形态是 GDScript/C#/C++ 单一路径都给不了的。
关键工程前提是引擎侧配合（静态缝合而非 plugin 接口，需调配 Godot 本体）。详见
[thirdgen-internet-strategy](thirdgen-internet-strategy.md)。

### 2.9 Native：终局必做的自主执行底座

寄生期"C/Rust 即 IR"；终局须直发 native 二进制——系统语言资格、全链路优化空间、独立
生态、自有硬件四条理由。选型（LLVM/自研 emitter/cranelift/MIR，GPU/NPU 的
Triton/TileLang 模式）届时做战略级分析，主战场先行锁定 1–2 个架构方向（如 RISC-V/Arm）。
详见 [native-backend-strategy](native-backend-strategy.md)。

---

## 3. 不做清单与重启条件

| 事项 | 裁决 | 重启/重审条件 |
|---|---|---|
| 仓颉语言发射臂 | 观察，不做 | 仓颉成为 NEXT 事实默认或出现华为侧合作拉力 |
| 鸿蒙 ABC 字节码 / 机器码发射 | 不做 | 无（合作层原则否定，见鸿蒙分战略） |
| SwiftUI 臂 | 不做 | iOS 高保真需求出现→先局部原生桥；全球化战略确立→重审 |
| Svelte 臂 | 观察 | React 臂落地且验证后 |
| 自研浏览器运行时（VM→WASM） | 不做，留远期选项 | 出现"不依赖宿主框架"的硬需求（如极端包体/沙箱） |
| native 后端 | 寄生期不建（C/Rust 即 IR）；**终局必做** | native 分战略 §2 触发条件（自举收口 + 主战场成立） |
| OpenHarmony 独立 OS（深开鸿模式） | 产品决策，登记终态方向 | AutoUI on HarmonyOS rich-device 完整覆盖 + shell 肌肉平移条件成熟 |
| Godot 重投资 | 维持示例轨道 | 真实游戏项目拉力 |

---

## 4. 目标账本映射

- 本次已挂链：GOAL-012（Web）← [web-ecosystem-strategy](web-ecosystem-strategy.md)。
- 待立项时新增（"新目标先入战略再登记"流程，本节即"入战略"动作）：
  GOAL-019 鸿蒙（ArkTS 主线）／GOAL-020 移动统一（Jetpack）／GOAL-021 ROS2（绑定包）／
  预留 GOAL-022 第三代互联网（Godot 入口）／GOAL-023 Native（终局执行底座）——后两者为
  远期线，号位预留、立项时再登记。
- GOAL-003/004（Rust）、GOAL-005（Python）、GOAL-013（C）维持现有关联；
  Python 的"消费桥"定位更新待 roadmap 扩充时吸收，不改账本行。

---

## 5. 扩充路线（各分战略"待扩充"节汇总）

| 分战略 | 首批待扩充 |
|---|---|
| web | React 臂立项前提与范围；a2ts 与 ArkTS 机制共享清单；第二组件库；WASM 选项触发条件 |
| harmonyos | rebaseline plan 范围；widget 覆盖矩阵；DevEco 版本锚定；vm/vue/arkui 三臂对拍闸门；Lite 设备与 C 轨交汇 |
| mobile | a2jet rebaseline 范围；CMP 版本选型；iOS 真机验证通道；发布管线 |
| ros2 | rcl 绑定面清单；.msg/.srv→Auto struct 生成管线；DDS QoS 映射；旗舰叙事 |
| godot | GDExtension vs 静态缝合性能实测（立项前置）；引擎调配策略（fork vs 上游）；双形态切换契约；第三代互联网旗舰场景 |
| native | 选型评估框架与决策时点；GPU/NPU 对标 Triton/TileLang；与 AAVM 自举闭环关系；中间表示选择；主战场架构选择 |
| c | 见其自身 §10 待决策点（Stage 0 立项时裁决） |
