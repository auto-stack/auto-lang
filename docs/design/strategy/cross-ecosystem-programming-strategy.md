# Auto 跨生态编程战略：面向场景编程与 CAP

> **Draft 0.2（讨论重建）｜target-state｜2026-10-22**。未找到此前独立 Draft 0.2 原件，按《3D引擎方案比较》讨论重建；不是原文恢复，也不是实现或交付证明。
> 技术权威入口：[Design46：CAP System](../46-context-profile-capability-cross-ecosystem.md)；图形依赖：[AutoGraphics/Forge](autographics-forge-v06.md)、[Design45](../45-autographics-wgpu-engine-and-game-prototypes.md)；编译器依赖：[ACE 总路线](auto-native-backend-evolution.md)。

## 1. 战略定位

Auto 从“跨操作系统/跨目标语言”扩展为**跨计算生态的语言**：CPU（ACE native/a2r）、MCU（a2c 的后续受检路线）、GPU shader/compute，以及未来 accelerator。共享的是语言核心、静态算法、类型/接口和工具链；设备内存、执行模型与准入条件由场景显式表达。

核心体验为：开发者学习 Auto 与场景能力合同，编译器推导代码适用范围，并在不满足时解释原因。Profile 不是 backend 别名，动态类型不是应用于所有生态的默认语义。Auto 原有“动静结合”优势继续保留；开发态工具可动态交互，但 system/MCU/device 的程序语义始终受自身 profile 约束。热重载不等于动态类型，模拟器运行成功也不代表设备执行成功。

CAP 是 Context-Aware Programming 的缩写；Context、Profile、Capabilities 是 CAP System 的三层模型。三层机制简述：**Context** 是执行场景和要求；**Profile** 是可组合、可版本化的语言/运行时/库/资源约束；**Capability** 是具体代码需求。需求经函数/语言特性 inference、调用图/effect 传播、约束求解与合法实现选择，进入 ACE 受检 HIR/MIR 及适用 backend。细节与诊断以 Design46 为准，本战略只维护定位、命名、投资顺序和交付分层。

## 2. 命名比较与建议

| 候选 | 长处 | 容易造成的误解 | 建议用途 |
|---|---|---|---|
| **Context-Aware Programming** | 表达编译器理解代码的场景适用性；覆盖硬件及 realtime/deterministic 等软件约束 | 单独使用可能被理解为位置/环境感知或运行时自适应 | 对外英文概念，首次出现加上“compile-time capability checking”说明 |
| **面向场景编程** | 继承 Auto 最初设计理念；中文直观，不把场景缩为设备或语法方言 | 需说明场景是可验证的执行契约，而非业务场景模板 | **推荐对外中文概念名** |
| **Context-Oriented Programming** | 字面接近“面向场景” | 已有 COP 学术术语侧重按 context 动态调整行为/激活层，与本稿以编译期准入为核心的机制不同 | 不作为 Auto 对外唯一正式英文名，也不抢用 COP 缩写 |
| **Heterogeneous Programming / 异构编程架构** | 准确描述 CPU/GPU/MCU/accelerator 协同的结果 | 容易缩成 CPU+GPU offload；不充分表达确定性/实时/沙箱，也可能暗示透明调度 | 架构成果与硬件域子路线用语 |
| **Context-Aware Programming System（CAP System）** | 与面向场景编程理念同名，便于传播，也可用于编译器、IR/effect、包解析和诊断 | 首次出现展开为 Context-Aware Programming System；Context/Profile/Capabilities 是内部模型 | **统一正式系统名（用户确认）** |

命名结论（2026-10-22 用户确认）：统一系统名为 **CAP System（Context-Aware Programming System，面向场景编程系统）**；语言理念为 **面向场景编程（Context-Aware Programming，CAP）**；完整架构称 **Auto 跨生态编程架构**，在强调多设备执行时可说“跨生态异构编程架构”。英文定位建议：**Auto — a context-aware, cross-ecosystem programming language.**

“Context-Oriented Programming”既有含义见[Hirschfeld、Costanza、Nierstrasz 的原始论文](https://www.jot.fm/issues/issue_2008_03/article4/)；因此本稿对此前讨论中的英文建议作术语消歧，保留中文“面向场景编程”。CAP System 为本次用户确认的项目名称；此处不声称这三个英文词组具有唯一行业解释或品牌可注册性。

## 3. 产品与技术层次

| 层 | 方向与边界 |
|---|---|
| Auto Core + CAP | 同一套类型/语义/推导，按 context 验证需求；不为每个后端建立独立语言 |
| ACE | HIR保留结构及摘要，MIR表达适用执行语义，native/ABC/AV/A2X分别 lower；保留函数增量与共享 cache |
| AutoGPU / AutoShader | AutoGPU 是 GPU 场景族，AutoShader 是图形 stage 的 Auto 接口；compute/AI tile 按后续 profile 扩面 |
| AutoGraphics / AutoRender | wgpu 上的 host 图形组织、资源和组合能力；与 device 函数经类型化布局/绑定、dispatch 与生命周期连接 |
| Shader Components | 基础 AutoShader 成熟后的首个主打候选：可组合的效果与属性更新，面向 Web/desktop/mobile 的受支持子集；不把全部平台一次列为已支持 |
| 跨生态标准库与包 | core/math 通用基座、按方法/实现导出 capability、条件依赖/variant与可复现锁定；可达闭包裁剪，非法调用诊断 |

首个纵向作品优先选择可组合 shader 效果，而非直接承诺完整游戏引擎、CUDA 替代品或 AI 编译器。它仍不替代 v0.6 已约定的真实 UI/2D/3D 组合、游戏样本及 Forge 目标；是否纳入某版本在临近规划时按容量选择。

未来 GPU compute 可从图像/数组计算的有限 kernel 入手，再根据内存、同步、数值和性能证据选择 AI tile/NPU/DSP。WGSL/wgpu 是首个建议切片；SPIR-V/HLSL/MSL/CUDA/ROCm 等为候选后端，不是本稿支持矩阵。CAP 也不自动恢复此前暂缓的 MCU/板级项目，恢复时仍核既有条件。

## 4. 明确的实现时序

```text
现在：AutoShader小范围设计（不实现）
              │ 资源/布局/renderer接缝
829有效交接 → 830组合/输入 → 真实Auto入口与基础AutoGraphics
                                      │
可消费ACE语义/类型/来源接口 ────────────┤
                                      ↓
                          AutoShader最小切片 ↔ CAP首版
                                      ↓
                       Shader Components / stdlib与包能力化
                                      ↓
                  MCU / compute / realtime / deterministic / accelerator
```

逻辑顺序为 **ACE → 基础 AutoGraphics → CAP 实现**。图形独立研究可与 ACE 并行，但完整 CAP 的实现进入门要求两项基础均已建立。ACE 核心骨架的现有计划交付不能自动代替“新版架构完成”；基础 AutoGraphics 也不能只用 Rust driver 或独立 Iced 探针认领。进入时由实施 Plan 绑定实际冻结交付/可消费 API，详见 Design46 §10。

当前 v0.6 仍按工程重构、ACE、AutoGraphics/Forge 和 Linux 四条主线推进。现在登记 CAP 合同与元信息需求，避免后续 HIR/MIR、cache、库接口返工；**不把完整推导器、包求解或全部 GPU 后端加入已有 ACE、829/830 或本版发行门槛**。完整 JIT、自动任务放置、所有加速器完成不作为启动 CAP 的无限前置。

2026-10-22近期细化：现在先完成[AutoShader小范围草案](autoshader-minimal-language-and-renderer-interface.md)，830仅补当前资源/布局/局部renderer与交接事实，继续原组合/输入核心。基础图形可用手写WGSL建立，AutoShader不是其进入门。后续最小AutoShader与有限CAP围绕同一消费者逐步接通，不要求完整CAP所有profile/包系统先完成；所需ACE/图形实际支持面在实施Plan绑定，不因设计存在宣称已具备。完整CAP的两项进入门保持。

后续分期的价值与验收：

1. **CAP MVP**：对静态共享算法、dynamic/heap/FS 的正反例生成可追溯约束；有限 profile 在 ACE 真实源码路径工作。
2. **AutoShader 纵向切片（可与CAP首版逐步接通）**：同一 helper 在 CPU 与 shader 实际运行，stage/布局/同步反例受控拒绝，真实宿主与资源生命周期可复验。
3. **弹性库/包**：MCU 构建不携带无关 desktop/GPU runtime，误用时解释包/调用链；合法实现锁定、裁剪清单及 effect 增量失效可复现。
4. **计算域扩面**：compute、MCU、realtime、deterministic 各以实机/数值/资源/失败 oracle 验证；hard realtime 与跨设备逐位确定性保留独立证明责任。

上述为候选工作包，未分配 Plan ID；实施时读取 current Spec、相关已有 Plan 与交付收据，再冻结最小切片和版本范围。本次仅完成文档设计。

## 5. 技术决策原则与成功标准

- **先可解释，再扩大求解能力**：有限 capability/effect 摘要及 SCC 不动点起步；复杂证明和调度由真实需求驱动。
- **合法性先于最优选择**：先保证类型/数值/所有权/域/权限/预算，再按显式策略与测量选择实现；“最快”不是未经实测的承诺。
- **统一语义，保留设备边界**：host/device 的传输、dispatch 与同步显式；普通函数复用不意味着透明迁移所有程序。
- **弹性不隐藏错误**：配置排除与链接裁剪可减少依赖，不能删除非法调用或偷偷改变语义；未知 FFI/动态调用不按无 effect 放行。
- **复用 ACE/标准库/包基础**：不建立平行 parser、类型系统、store 或包管理器；所有摘要、变体和 profile 都参与版本/依赖/失效。

成功不按后端数量计分。核心证据是：共享函数经实际路径运行、受限能力及时报错且能解释来源、库/包依赖按约束可重复装配、错误缓存/权限变化被拒、真实设备的数值与资源证据明确。外部能力机制可参考[Slang 文档](https://docs.shader-slang.org/en/stable/external/slang/docs/user-guide/05-capabilities.html)，不据已有先例宣称 Auto 全部实现或自动优于其他语言。
