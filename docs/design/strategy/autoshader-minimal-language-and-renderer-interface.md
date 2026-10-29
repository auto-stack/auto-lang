# AutoShader 最小设计：Auto、ACE、CAP 与 wgpu 接口

> **Draft 0.1｜target-state｜2026-10-22**。本轮仅设计，不启动编译器、renderer 或 CAP 实现；下文类型名/注解/产物字段都是概念接口，尚未冻结为语法、Schema、包或 API。
> 承接[Design45](../45-autographics-wgpu-engine-and-game-prototypes.md)、[CAP Design46](../46-context-profile-capability-cross-ecosystem.md)、[ACE 总路线](auto-native-backend-evolution.md)。近期执行合同仍为[PLAN-830](../../plans/830-autographics-surface-portal-composition-and-input.md)，本设计不把语言实现加入830。

## 1. 目标、现状与范围

AutoShader 是 Auto 在 GPU graphics stage 下的受限执行场景，复用 Auto 的函数、类型和诊断来源。第一条验证链是 **Auto 源码 → ACE 受检语义 → device lowering → WGSL → 现有 wgpu renderer → 实际画面**。不发明一门独立 Shader 语言，不要求先完成全部 CAP。

当前观察来自829候选工作树 `D:/autostack/.wt/lang-829/auto-lang/experimental/autographics-prototype`，不是已发布生产 API：`src/renderer/pipeline.rs` 的 `VERT`/`QuadPipeline::new_with` 使用手写 WGSL 与既有 Device/Queue；`target.rs` 的普通离屏目标尚需830增补采样用途；`error.rs` 有 `RendererError::BadShader` 分类。这些事实不证明 Auto 源码或通用 Shader 插件接口已经存在。来源指纹见[830 r2规划预检](../../plans/attachments/830/planning/preflight-r2.md)。

首轮只考虑现有 Windows/wgpu 版本线的 vertex/fragment、纹理着色和小型参数化效果。正式平台、预算、ABI及语法由后续实施 Plan 冻结。不加入 compute、storage写入/atomic/barrier、AI tile、第二GPU后端、完整材质编辑器、自动offload或跨设备逐位浮点一致性。

## 2. 最小语言面

| 类别 | 首轮候选 | 边界 |
|---|---|---|
| 值与类型 | f32、i32/u32、bool，有限vector/matrix与静态struct | bool可作为device逻辑值；host共享布局单独定义，不把Auto对象原样复制到GPU。当前ACE标量Spec不能据此外推已支持f32/vector |
| 计算 | 确定类型局部值、算术/构造/字段访问、if、直接函数调用及少量math intrinsic | 拒绝递归、动态换型、动态分派、heap/GC、host IO、任意指针/FFI；循环、泛型及更广库面按实例需求再选 |
| stage | vertex/fragment entry、位置/插值与颜色输出 | 显式stage接口；同一helper可按兼容事实在CPU/device实例化，但入口不是普通跨设备调用 |
| 资源 | uniform参数、只读采样texture与sampler | host负责创建/上传/绑定；device只得到受检资源引用。CPU指针、wgpu对象和VM Value不进入device函数 |
| 数值 | 明确精度/转换/溢出/NaN及误差合同 | 沿用Auto已确认语义；若目标表达不了，拒绝或在后续Plan批准受限profile，不能静默改语义 |

“静态函数”不自动意味着可上GPU。被调用函数、隐式操作、math实现、stage限制与设备要求都须纳入闭包。无确切支持事实的操作报unsupported，不能依赖最后生成WGSL碰运气。

### 三个语义样例（不是可执行源码）

1. **纹理着色**：vertex变换四边形并传UV；fragment采样输入纹理并乘tint。核对UV、布局、采样、颜色/alpha与已有合成结果。
2. **参数化背景**：fragment使用显式传入的time/size/color生成简单效果；host修改uniform即可更新。time由host输入，device不自行读取系统时钟；参数更新不应重建shader/pipeline。
3. **共享helper**：选择只有最小静态数学的Auto函数，分别在CPU和fragment调用，记录数值容差与边界值。CPU路径必须实际支持所用类型/intrinsic，不能用另写的Rust函数冒充同源Auto。

上述样例用于审查设计可表达性；本次未生成、编译或运行它们。

## 3. ACE 接口：复用语义，保留结构

```text
Auto source / AST
  → ACE resolve/type/infer/instantiate ↔ 受限CTE（如样例实际需要）
  → Checked HIR + stage/resource/effect facts + source mapping
  → device专属检查与结构化lowering
  → WGSL + interface description + requirements + diagnostics mapping
  → renderer adapter
```

ACE拥有语言类型、符号/函数身份、求值顺序、泛型与来源事实；device检查/lowering消费这些事实，不能在WGSL生成器里复制parser/typechecker。普通Auto函数只在需求合法时产生device实例。入口注解与resource声明的正式写法待后续Plan，当前不改核心语法参考。

首轮优先从结构化 Checked HIR lower，以保留函数、分支与stage结构。可有窄的device投影，但是否需要独立AutoShader IR由whole-path探针决定；不在本稿先固定完整IR。现行Typed MIR/ABC/AV仅有有界执行面，不能把GPU地址空间/插值/纹理语义塞成未经验证的通用call。未来共享MIR片段须先扩展合同和验证器，再逐项验收；不要求本轮GPU链经过CPU MIR或ABC。

编译输出中保留函数/入口与资源声明的源码映射。WGSL语义/验证错误、pipeline不兼容和设备限制错误分别归类；后端报错有映射时指回Auto位置，无映射时明确指出生成入口及产物身份，不伪造源码行。

## 4. CAP 接口：先有限契约，后统一求解

AutoShader消费 `gpu.vertex` / `gpu.fragment` 场景及静态数学、纹理访问等需求；CAP最终负责这些需求的推导、传播、profile合成、实现选择和包解析。stage规则、资源布局与数值义务不能简化为一个 `supports_gpu=true`。

首个device实现允许先使用**显式、有限、可验证的shader profile/需求摘要**，随最小纵向切片接入CAP首版。它只能放行已覆盖操作；未知effect或间接目标拒绝。注册身份/规则版本须可追踪，未来迁移到统一CAP时复用同一份受检事实与反例，不形成永久的第二套能力注册/求解系统。

完整CAP继续遵守“新版ACE完成、基础AutoGraphics建立后进入实现”的路线。AutoShader的窄检查和最小CAP可逐步接通，无需先实现MCU、包系统求解、所有profile、JIT或加速器。现在只写设计，不提前启动这套有限检查器。

## 5. 编译产物与 renderer 的三份接口合同

下表定义职责和必要信息，**不冻结结构体字段名或二进制格式**：

| 合同 | producer → consumer | 必要事实与责任 |
|---|---|---|
| 受检device单元 | ACE语义 → device lowering | 具体类型/函数实例、stage IO、可达调用/effect、resource声明、数值策略与来源；未经检查的AST/Unknown不能进入 |
| Shader产物包 | device lowering → renderer adapter | WGSL内容/身份、entry及stage、接口/绑定布局描述、最低设备要求、profile/语义版本、来源映射与依赖身份；不携带Device/Queue、宿主指针或编译器活对象 |
| 参数与资源绑定 | host/Auto SDK → renderer | 静态字段值与布局版本、texture/sampler句柄及代际/owner、目标格式；renderer检查匹配后创建bind group/pipeline并编码draw；源码编译器不负责创建GPU资源 |

**host/device数据合同**：资源布局必须说明字段类型、offset/size/alignment、矩阵约定、binding编号/可见stage、纹理/采样器种类和访问方式。首版可从同一受检描述生成host打包器与WGSL声明，反射是构建期接口元信息，不依赖runtime reflection。不能直接按Rust/Auto内存布局复制，不能默认为紧密打包；格式版本或布局不匹配应在使用前拒绝。

**renderer所有权**：继续借用现有宿主持有的同一Device/Queue，维护GPU对象与资源寿命。生成WGSL和手写WGSL可通过局部adapter走同一渲染路径，Surface/Portal只关心内容输出和合成合同，不依赖shader来源或Auto AST。830只需保持这种分层并记录接缝，不必实现通用用户shader加载/注入API。

**颜色与目标合同**：颜色空间、alpha、纹理格式、采样usage、pass顺序与设备代际由当前renderer/830实证定义，未来Shader产物必须匹配；shader换来源不能改变Portal命中/焦点、更新次数或合成顺序。Surface依赖图用于内容调度；shader内计算/调用图用于device分析，分别建模。

**错误与生命周期**：首次创建失败返回受控错误，不留下可使用的半成品。未来Shader替换/重载若纳入实施范围，应先验证新产物/绑定和pipeline，成功再切换、按在途GPU工作释放旧资源；失败保留原可用pipeline。该未来重载语义不新增为830验收；当前复用现有错误/资源合同，并记录缺项。

## 6. 参数更新、缓存与设备准入

uniform值变化通常只更新对应buffer；源码/入口/布局/target格式等改变才影响相关产物或pipeline。编译产物缓存复用ACE store，依赖包含源码/语义版本、实例、profile/规则、设备要求、backend及数值策略；GPU pipeline缓存由renderer/device owner管理，并包含布局、格式、采样/渲染状态和device代际。两类cache对象与生命周期不同，不复制ACE全局store。

实际adapter/device必须满足产物最低要求，准入失败带需求/实际值。830仅记录自己用到的features/limits及资源事实，不实现CAP registry、全设备数据库或自动fallback。后续生成器亦不得为“跑起来”暗加第二设备、改变alpha或让device调用host服务。

## 7. 对830的有限影响

830继续以真实UI/2D/3D组合与输入为目标，沿用全部S/V/AC分母、Windows单窗profile和G829/迁移门。补充项只进入现有任务及交接：

- T-02：从whole-path实验记录纹理格式/usage/颜色/alpha/采样、最小参数布局、实际设备要求与失败分类。
- T-04：合成调度不依赖手写WGSL的文本组织或Auto语言对象；实现仍可采用当前内置WGSL，记录局部shader创建/绑定接缝，不为未来语言预建通用框架。
- T-08：在现有handoff中列producer/consumer、输入输出/绑定、错误/资源寿命与AutoShader尚缺的接口；V-10/AC-08核对文档是否与当前实现一致。

本轮不要求830接入Auto源码、生成WGSL、支持自定义材质/热重载、修改ACE或运行CAP求解器；不额外添加游戏、平台、GPU探针族或强制shader替换测试。

## 8. 后续切片与可验证结果

| 阶段 | 前置/范围 | 后续实施必须提供的证据 |
|---|---|---|
| 现在：小范围设计 | 本草案与830 r2边界 | 概念样例、三接口职责与未冻结决策可审查；不是代码通过记录 |
| 830：组合/输入 | 有效G829来源与单独启动授权 | 原合同的嵌套/输入/生命周期及局部接缝交接；手写WGSL仍可使用 |
| 真实Auto/基础图形 | 迁移后公开SDK/宿主接口交付 | Auto控制场景与事件、资源/输入链实际工作；基础成立不依赖AutoShader完成 |
| AutoShader最小纵向切片 | 可消费ACE受检类型/函数与来源接口、所需f32/vector/math支持、实际wgpu宿主与布局合同；按后续Plan绑定交付范围 | 同源Auto产生WGSL并实际渲染；三样例、数值/布局/stage反例；与有限CAP事实逐步接通，不等完整CAP扩面 |
| Shader Components / CAP扩面 | 纵向切片通过；完整CAP按Design46进入门 | 多效果组合、真实参数更新及包/能力传播；compute/MCU/AI后续分别选范围 |

实现Plan需要固定以下正反例及实际入口，不以字符串快照或能编译替代：三样例真实画面/状态；同一helper的CPU/device数值比较；Auto动态换型/host IO/递归被device检查拒绝；错误stage/缺binding/错布局/资源代际被对应准入门拒绝；错误WGSL或pipeline失败受控；uniform更新不重复创建pipeline。替换/重载、设备丢失和多平台若未纳入首例，应明确列为未支持，不能宣称完整覆盖。

## 9. 待冻结项与参考

后续Plan决定：正式入口/资源语法、精确类型与intrinsic清单、设备profile身份、最小接口描述及版本、host打包方式、适用布局/颜色规则、WGSL验证库与错误映射、是否需要device IR、CPU对拍入口、数值容差及性能预算。本草案不替代实现计划、不分配Plan编号。

技术参考：[WGSL官方规范](https://www.w3.org/TR/2026/CRD-WGSL-20261021/)的stage、resource与host-shareable布局规则供后续核对；实际实现仍以冻结wgpu版本接受的语言面及设备验证为准。Auto侧接口和分期均是本文提案，不表示现行编译器已支持。
