# Design 46：CAP System 跨生态编程架构（Context + Profile + Capabilities）

> **Draft 0.2（讨论重建）｜target-state｜2026-10-22**。未检出此前独立的 AutoGraphics/CAP Draft 0.2 文件；本文依据《3D引擎方案比较》讨论与当前仓库设计重建架构补编，不声称恢复原文或已有实现。
> 统一系统名：**Context-Aware Programming System（CAP System）**；对外理念名：**面向场景编程（Context-Aware Programming）**；架构成果：跨生态编程架构。Context、Profile、Capabilities 为 CAP 的三层技术模型。命名比较与阶段策略见[跨生态战略](strategy/cross-ecosystem-programming-strategy.md)。
> 实施顺序：**新版 ACE 编译器架构完成 → 基础 AutoGraphics 建立 → CAP 实现 → 多生态扩面**。当前只更新设计，不启动实现、不修改现行 Spec 或在途 Plan。

## 1. 设计原点与既有路线

Auto 的统一对象从操作系统平台扩展为计算生态：CPU host/system、MCU、GPU shader、GPU compute，以及未来 accelerator。统一语言、类型与共享函数，同时显式表达各执行域的内存、并行、调用、数值与资源差异。不会因为支持多个后端，就允许任意代码跨设备直接调用。

本设计把早期[面向场景草稿](raw/scenario.md)中的“场景”从方言/语法环境扩展到语义合法性、运行时假设和库依赖约束。方言决定可解析的领域语法，CAP 决定已解析代码的适用范围；两者关联但身份独立。一个工程内可有 host、shader 与 compute 多个 context，不能用一个全局 scenario 枚举覆盖全部函数。

| 既有材料 | 本设计如何承接 |
|---|---|
| [Design45 AutoGraphics](45-autographics-wgpu-engine-and-game-prototypes.md)、[图形战略](strategy/autographics-forge-v06.md) | 保留 wgpu、共享设备、Surface/Portal、UI/游戏与迁移交接边界；新增 AutoShader/AutoGPU 长期语言机制 |
| [ACE 总入口](strategy/auto-native-backend-evolution.md)、[Design43](43-ac-av-typed-mir-and-abc.md) | 复用共享语义、Typed HIR/MIR、ABC/AV、CTE、函数增量与缓存；CAP 是约束层，不另建编译器 |
| [HIR 设计](strategy/auto-hir-design.md)、[当前 HIR Spec](../specs/auto-hir/project.md)、[当前 MIR Spec](../specs/auto-mir/project.md) | 现有 core/scalar/text/record 等有界 IR profile 是格式/执行支持域；不是已交付的场景 profile 或完整 CAP |
| [Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[41](41-interface-metadata-and-foreign-abi.md)、[42](42-semantic-metadata-inference-and-comptime.md) | 复用产物/依赖/来源、FFI 元信息及语义推导/CTE；不复制 store、类型系统或解释器 |
| [C 生态战略](strategy/c-ecosystem-strategy.md)、[标准库 Spec](../specs/stdlib/project.md) | MCU/a2c 仍按实际支持与重启条件推进；stdlib capability 化是目标态，不把旧后端当成已完成的新 lowering |

## 2. Context / Profile / Capability 三层

| 层 | 定义与职责 | 例子 |
|---|---|---|
| **Context（场景）** | 某段代码的预期执行环境及要求；包含执行域、阶段、宿主/部署条件、实时性/确定性等维度 | `desktop host`、`system`、`mcu interrupt`、`gpu fragment`、`gpu compute`、`compile-time` |
| **Profile（约束配置）** | 对 Context 的可版本化、可组合具体契约：允许/必需/禁止的能力、资源上限、语义和需验证的保证 | `mcu-static@1`、`shader-portable@1`、`system@1 + realtime@1` |
| **Capability（能力原子）** | 编译器可识别的最细需求或可用机制；可带类型、作用域、stage、数值精度、设备限额等参数 | `lang.dynamic_type`、`mem.heap`、`io.fs`、`gpu.texture_sample(fragment)`、`gpu.atomic(u32,workgroup)` |

Context 表达“在哪种条件下运行”，Profile 固化“准入合同”，Capability 表达“这段代码具体需要什么”。target triple、backend 名称与设备 feature 是另外的事实，不能只用 `backend = a2r` 推定代码属于 desktop，也不能因为 CPU 具有某指令就认为当前场景允许它。

Profile 建议结构为 `id/version + applies_to + allowed + required + forbidden + limits + obligations`。`required` 是环境必须提供的基础设施；`forbidden`/`no_heap` 是否定约束，不是一个代码可以通过“请求 no_heap 能力”绕过的开关。`deterministic`、`realtime` 是由多项约束与证据组成的契约，不是一个布尔能力标签。

组合遵循约束求交：允许集合取交集，禁止与义务取并集，资源上限取较严格值。profile 依赖/互斥和目标模型须一起检查；矛盾组合报错，不能由最后一份配置覆盖。用户显式放宽应产生不同身份/版本，且不能越过实际 target/backend 支持边界。`desktop + realtime` 可以收紧 heap/阻塞策略，`fragment + compute-only barrier` 则应拒绝。

Capability 的注册身份、版本、蕴含关系和参数规则必须明确。未知原子、未知 FFI effect 与未知设备限制按未证明处理；不把 Unknown 当成 Dynamic 或“全部允许”。编译期 capability 准入也不等于运行时授权：文件路径权限、插件隔离和设备句柄仍由宿主/加载器落实。

## 3. 场景 profile 示例（目标态，支持面待 Plan 冻结）

| Profile | 可用/常见能力 | 默认限制与额外义务 |
|---|---|---|
| `desktop/app` | 静态核心；按运行时启用 dynamic、heap、GC；经宿主提供 FS/net/threads/GPU dispatch | OS/目标/宿主授权分别检查；desktop 不意味着所有能力默认授予 |
| `system` | 静态类型、显式内存/所有权、native/FFI；按目标允许 heap、threads、FS | 禁止动态换型及隐式 GC；不是一律无 heap 或无 OS；FFI 的分配/阻塞/effect 必须申明 |
| `mcu-static` | 静态存储、有限栈、MMIO、interrupt、DMA，按芯片提供 atomics/f32 | 默认无 heap/GC/动态类型/OS IO/GPU 执行；板级包确定中断安全与地址访问。软件浮点可作为明确实现，成本受预算约束 |
| `gpu-shader-portable` | 静态标量/vector/matrix、buffer、texture；vertex/fragment 分 stage | 禁止主机 IO、任意分配/GC及默认递归；derivative/隐式 LOD 等按 stage 检查；地址空间、布局与 uniformity 另验 |
| `gpu-compute-portable` | compute entry、workgroup/storage、显式纹理访问、受支持 atomics/barrier | workgroup 尺寸/共享内存/数值能力受设备及 backend 限制；不自动支持 fragment-only 操作或 AI tile 指令 |
| `realtime`（叠加） | 静态/预分配数据、可界定循环/调用、经证明的中断或调度协议 | 禁止无界分配/GC pause/阻塞/无界递归；需最坏执行时间、调度、驱动/硬件证据。普通编译通过不证明 hard realtime |
| `deterministic`（叠加） | 显式输入/时钟、固定 seed、固定顺序、规定整数/浮点规则 | 拒绝隐式 wall clock/entropy、未定序并发与非确定 reduction；声明同设备/同 backend 或跨设备的保证范围；跨 GPU 浮点逐位一致另验 |
| `compile-time` | Checked 片段经 AV 执行、可追踪的输入/冻结结果 | IO 仅显式授权且纳入依赖；指令/内存/递归预算与确定性约束复用 CTE 合同；宿主 app 权限不继承至 CTE |
| `sandbox`（叠加） | 宿主显式授予的导入与受控资源 | 限权/预算/取消，加载时重验；native/FFI 未建立隔离时不能凭注解声称安全 |

`auto.math` 的静态纯数值函数可跨 CPU/MCU/GPU 复用，但 f32、sqrt 的实现、精度、trap/NaN、舍入和成本仍须匹配。数学函数名字相同不能证明设备可用或数值语义等价。

## 4. 函数级与语言特性级 inference

原则：**可推导的需求由编译器推导；边界、承诺与入口用显式声明约束。** 推导结果首先是需求表达式，不是给每个函数永久贴一个封闭 profile 名单。对指定 profile/target 询问能否满足需求，得到适用场景；未来新 profile 不要求重写所有纯函数注解。

以下 Auto、属性、配置和 CLI 均为**语义示意，不是已实现语法**，不替代现行语法参考：

```auto
fn distance(a: vec3<f32>, b: vec3<f32>) -> f32 {
    length(a - b)
}
// 推导 math.f32、vector<f32,3>、所选 length/sqrt 的需求
// 不需分别写 host/mcu/gpu 三组声明

#[profile(system)]
fn change_type() {
    var a = 1
    a = "string"  // 需要 lang.dynamic_type；system 拒绝
}

#[context(gpu.fragment)]
fn shade(input: ShaderInput) -> vec4<f32> { ... }

#[capability_limit(math.f32, vector)]
fn public_math(...) { ... } // 显式上界，函数体超出时拒绝
```

`var a = 1` 的可变性与动态类型分开；若始终是同一静态类型，不能仅因 `var` 就要求 dynamic。跨类型重赋值若属于语言允许的动态语义，推导 `lang.dynamic_type`；若静态声明禁止换型，先报类型错误。`Any/Dyn`、装箱、反射、GC 分别记账：动态值不必然要求 GC，使用确定类型的静态反射不必然要求 runtime reflection。

| 来源 | 推导内容 |
|---|---|
| 表达式、类型及隐式 lowering | 动态换型、字符串/容器的实际分配、捕获逃逸、clone/drop 的调用 effect；不能只扫描函数显式调用 |
| 控制流与调用图 | 普通调用传递需求；递归 SCC、间接调用、清理路径也在闭包内；阻塞/同步/原子含作用域与顺序 |
| 泛型/接口/高阶函数 | 对类型参数及回调 effect 保留符号约束，实例化后求具体需求；如 `map(f)` 继承 `f` 与容器实现的需求 |
| 闭包/动态分派 | 闭包环境分配/捕获及调用 effect 分开；动态分派的接口定义 effect 上界，未知目标不能按纯函数放行 |
| FFI/预编译库/插件 | 依赖可信且可验证的接口摘要；未标注 effect 按 unknown 处理，在受限场景拒绝或要求可核验 adapter |
| 入口/导出接口 | 显式限定执行域/阶段和对外需求上界；修改公开需求可能破坏消费者兼容，需按包 API 变更处理 |

对同一函数可生成多种受检实例，按类型实参、有效 profile、target、实现选择区分；共享语义与可复用产物按兼容性复用。编译器不能为了通过 system 检查，静默把用户的动态换型程序改写为另一种语义。

## 5. 能力推导与约束求解

首版采用有限、可解释的单调约束分析：类型/effect 摘要 + capability 蕴含闭包 + 调用图 SCC 不动点；不先做任意程序定理证明或全局最优搜索。

对选定实现与实例的函数 `f`，以 `R_local` 表示本体含隐式操作的直接需求，`I` 为已选择实现：

```text
R(f,I) = closure(R_local(f,I) ∪ ⋃ R(callee instance,I))
E(p,t,b,h) = Allowed(p) ∩ Provided(target, backend, runtime, host)
Legal(f,p,t,b,h,I) ⇔
    R(f,I) ⊆ E(p,t,b,h)
    ∧ R(f,I) ∩ Forbidden(p) = ∅
    ∧ Required(p) ⊆ Provided(target, backend, runtime, host)
    ∧ Stage/AddressSpace/Type/ABI/Uniformity predicates
    ∧ Resource bounds and semantic obligations satisfied
```

其中 `∩/⊆` 对带参数原子表示兼容性/蕴含检查，不是机械字符串集合运算。未能证明预算或保证应报“缺少证据/不支持”，不等价于已证明违规，更不能算通过。阶段化实现需区分 language-semantic pass、backend-supported pass 与资源/部署保证。

对运行时 `if`，两个可能分支需求取并；不能用“这次没执行”绕开 FS 或 heap 检查。经受检的编译期分支或显式 capability 分支，可对具体配置消除不可达需求，但名称/类型合法性依语言规则检查；不可由普通 DCE 先删掉越权代码来放行。编译期执行本身的 effect 归 compile-time context，生成代码的 effect 归目标 context。

可替换实现用 `R_cpu OR R_gpu` 等条件需求表示；在当前配置选择一个满足契约的候选后再计算具体闭包。不同普通调用的全部必需需求取 AND，不能因为存在另一条合法路径就忽略不合法路径。递归 SCC 在有限原子/参数抽象域上迭代至稳定；分析预算用尽保守拒绝并给原因，不悄悄允许全部能力。

输出应包含 `inferred requirements + predicate obligations + provenance + selected implementations + supported configurations`，并可解释每一条约束来自哪个表达式、callee、隐式操作、包边或 profile。

## 6. 最佳实现选择

“最佳”定义为**满足同一语义合同与用户策略后的确定性优选**，不默认表示最快。普通函数重载先按类型/签名规则解析；capability 不能让一个类型上错误的 overload 成功。只有同一逻辑操作显式注册的 backend/设备实现候选，才进入后续选择。

1. 按参数/结果类型、精度/溢出/错误、所有权、执行域和 ABI 合同筛选。
2. 排除 capability、stage、设备限额、realtime/deterministic 义务不满足者；检查所需 bridge 是否存在。
3. 在用户策略内排序：显式 pin 优先，其次经声明且可验证的特化优先，再按固定的成本模型选择；目标吞吐、延迟、内存、功耗可分别指定，缺测量不编造成本。
4. 同优先级不可判定时诊断歧义/要求显式 pin；结果写入构建解析记录，不能依机器偶然安装的软件静默改变。

例如同一批量向量操作可提供 scalar CPU、CPU SIMD、GPU compute 实现。小批量可能更适合 CPU；GPU 的传输/调度成本必须算入策略。MCU 中只能选择满足数值/内存/预算的静态实现；没有合法候选应报错。不同语义的近似数学不可自动替代精确数学，fallback 不得扩大权限、破坏确定性或悄悄迁移执行域。

host→device 是显式 dispatch 与资源传输/同步边界，不是普通 Auto 函数栈调用；设备函数不能直接读 host 指针。后续运行时设备选择可在构建时已验证的 variants 中显式分派，选择策略、失败路径、权限与资源都须受检。MVP 先做构建期选择，不加入透明自动 GPU offload。

## 7. 代码、标准库与包系统

### 7.1 三种声明各司其职

代码表达“实际需要什么”（inference/显式边界）；库接口声明“使用本 API 需要什么”（可传递摘要）；包提供“有哪些实现、按哪些条件选依赖”（manifest/resolution）。Profile/target/host 提供能力的事实不能由包自我声明任意扩权。

公开接口建议导出精确/参数化需求及稳定上界。显式 `requires` 不是可信免检：源码函数体需求须落在声明范围内；私有实现可重新推导。预编译包的摘要与内容身份/编译器语义版本绑定，恢复/装载时重验兼容性；签名或 hash 只能绑定来源，不能自行证明第三方 effect 声明为真。

### 7.2 弹性标准库

| 库层（建议名） | 组织原则 |
|---|---|
| `auto.core` | 最小静态数据/控制/基础协议；不隐式依赖 OS、heap、GC；具体类型仍受目标表示与操作约束 |
| `auto.math` | 通用静态数学接口 + 按精度/目标约束的实现；vector 不要求物理 SIMD；非所有 API 对所有设备无条件可用 |
| `auto.alloc` / 容器扩展 | 需要 heap/allocator 的 API 与静态容量/调用方 buffer 版本分开；按方法细分，不能只在包根标一个 heap |
| `auto.fs/net/thread` | 显式宿主服务与 effect；不进入 MCU/device 的可达代码闭包 |
| `auto.mcu` | 板级 MMIO/interrupt/DMA adapter，地址/中断安全及静态资源合同 |
| `auto.gpu` / AutoShader | device 数学、纹理、buffer、stage/workgroup；host 侧的设备创建/dispatch API 单独命名与归域 |
| AutoGraphics/AutoRender/Shader Components | host 场景/合成与 device shading 分包/分入口；共享数据布局、类型化资源绑定与反射元信息连接两端 |

“自动裁剪”分两步：先按配置与受检条件选择合法 API/实现及依赖闭包，再对选中代码做通常的可达性/链接裁剪。整包不可达的可选 GPU 实现可从 MCU 构建消失；一旦 MCU 根入口调用 GPU-only API，就必须报错，不能删除调用、返回默认值或静默 CPU fallback。可选功能关闭后，其公共 API 不可作为仍存在的接口导出。

导出符号、回调、插件注册、反射根、静态初始化和资源文件也参与可达根；不能仅从 `main` 扫描后误删。生成裁剪清单，列保留/排除的模块、实现、native 库和资源及原因；保持用户可观察语义与 side effect。标准库 capability 化应复用[现有装配合同](../specs/stdlib/design/assembly-manifest.md)，迁移时明确旧 API 和依赖身份映射。

Rust 的 [`no_std`](https://doc.rust-lang.org/reference/names/preludes.html#the-no_std-attribute) 是 crate 级不自动链接 `std` 的机制，仍可选择 [`alloc`](https://doc.rust-lang.org/stable/alloc/)。CAP 的目标是在多维约束下做更细粒度的推导与选择；不能把 `no_std` 等同“不支持 heap”，也不能据此声称 Auto 已实现或全面优于 Rust。

### 7.3 包解析、锁定与跨域产物

包元信息候选字段：`capability_schema`、导出接口需求、`variants` 条件/入口/依赖、目标/ABI/数值约束、资源预算。它们是设计字段，不在本稿修改现行包格式。

解析器先结合 context/profile/target/backend/host contract 解出具体 variants，再验证传递依赖与函数摘要；发现源代码需求超过摘要时构建失败并回报包链。无解时提供最小冲突链；optional 不等于自动授予能力。host 工具/build script、目标 runtime、device 库分别构建图，避免 MCU 构建意外链接 GPU/桌面运行时，也避免 device 库获得构建机权限。

锁文件或构建 manifest 记录包版本/内容、feature/variant 选择、profile 版本/指纹、capability schema、target/backend/toolchain、设备最低要求、数值策略与生成资源。device 实测特性仅在显式部署策略内决定变体，保证可重放解析。公开 API 增加 capability 需求视为兼容性收紧；按包版本/迁移策略处理。

## 8. ACE HIR/MIR 与 backend lowering

```text
source / dialect AST
    → 名称/类型/泛型/effect 推导 ↔ 受限 CTE
    → CAP 需求摘要、profile 合成、候选解析、诊断
    → Checked HIR + versioned capability/predicate evidence
        ├─ 结构化 A2X lowering → Rust/C/... → target 工具链
        ├─ device 结构化 lowering → AutoShader IR → WGSL → wgpu
        └─ Typed MIR（适用的受检执行语义）
             ├─ native adapter → Cranelift → CPU
             ├─ ABC → verifier/loader → AV（CTE/host 有限 profile）
             └─ 后续 device/accelerator lowering（独立扩展与验收）
```

| 阶段 | 必须承担的 CAP 职责 |
|---|---|
| 语义工作态 | 类型/effect/泛型及 CTE 协同求解；保留未决约束，具体可执行片段进入受检状态后才执行；不让 Unknown 流入 backend |
| Checked HIR | 保存解析身份、结构化控制流、类型/泛型实例、执行域、effect 摘要、来源与所选实现；校验实际需求与声明一致，并绑定配置/依赖身份 |
| Typed MIR | 显式内存/调用/cleanup/effect、跨域边界；verifier 验证 lowering 未引入被禁能力。GPU 地址空间/barrier/uniformity 等需专门模型，不能假设当前 CPU 标量 MIR 已支持 |
| A2X | 从 HIR 保留结构，结合 Rust/C 等运行时/FFI 能力检查；宿主语言支持某操作不代表源 profile 允许它 |
| AutoShader | Auto 的 device 场景族及 stage 检查；首选窄的 Auto→受检 device 表示→WGSL→wgpu 纵向切片，后续按实测决定 Naga/其他翻译桥 |
| native/ABC/AV | 校验所需 runtime 导入、能力版本与执行 profile；loader 拒绝未授予或版本不匹配的导入；CTE 与 app 分别授权 |
| 其他 GPU/AI backends | SPIR-V/HLSL/MSL、CUDA/ROCm/tile/NPU 为候选，不因本设计存在就声称已支持；各自冻结语言子集、资源/精度和 lowering 证据 |

CPU/MCU/GPU 共享语义事实，不强迫所有 backend 经过同一物理 IR，不为 shader 新建另一套 Auto 语法/类型系统。AutoGPU 是 GPU 场景族，AutoShader 是图形 stage 接口，compute/AI kernel 是同一体系的其他 profile；不是三门独立语言。AutoRender 是 host 渲染组织层的候选名称，不能把它和 GPU device 函数混为一层。

降级、优化和桥接后重新校验 effect/数值/控制流及资源义务；例如隐式装箱、清理、软件浮点辅助函数或 barrier 重排都不能逃过验证。backend 不支持应在该 backend 的能力门报具体原因，不要伪造前端 profile 错误。

### 8.1 增量、缓存与热重载

需求摘要以函数/实例为首版单位，复用 ACE 查询图与来源。callee body 引入新 capability 时，即使签名未改，依赖其 effect 摘要的 caller/根合法性也须失效；仅依赖签名且摘要未变的消费者不重做无关工作。类型布局、泛型/CTE body、实现选择、profile/target/backend/host contract 改变按真实依赖失效。

缓存键至少包含源码/语义事实、能力 schema/规则版本、profile 指纹、实例与所选 variant、target/backend/runtime/ABI和数值策略；设备具体 ID 仅在其事实影响结果时加入。恢复的摘要/IR凭证按版本/上下文重验，不能依赖缓存存在就跳过准入。函数级来源路径独立映射，不把绝对机器路径作为公共语义身份。

热重载不得给存活程序静默增加权限或换 execution domain。候选需求、授权/导入、代际资源与布局兼容都验证后才安装；失败保留旧版。GPU pipeline/bind group/资源布局的代际与帧同步另设 adapter 合同；沿用 Design40/43 生命周期，不建立第二个全局 cache。CAP 接入也要证明未改函数复用与受影响根精确失效，保持既有结构门及周转性能口径。

## 9. 诊断：从非法操作解释到场景入口

错误必须区分：`type mismatch`、`capability forbidden/missing`、`profile conflict`、`stage/address-space violation`、`backend unsupported`、`resource limit`、`guarantee unproven`、`ambiguous implementation`、`package resolution conflict`。以下错误码为拟议命名，不是已发布接口：

```text
error[CAP-capability-forbidden]: system@1 禁止 lang.dynamic_type
  at control.at:8:5  a = "string"
  origin: a 初始为 i32，此赋值需要动态换型
  required by: main → update → change_type
  policy: entry main 使用 system@1，forbidden: lang.dynamic_type
  help: 保持静态类型，或显式使用合适的 host 边界
```

```text
error[CAP-package-conflict]: mcu-static@1 无法满足 gpu.compute.dispatch
  root: board_app → image_filters::blur → gpu_only::dispatch
  target/backend: 所选 MCU / a2c，无可用 GPU dispatch bridge
  candidates: gpu variant 不适用；未声明满足相同语义的 static CPU variant
  help: 选择/提供显式静态实现，或将调度移至支持的 host
```

GPU derivative 出现在 compute stage、非 uniform 路径执行 barrier、workgroup 超限应分别定位到操作和 stage/限额来源。realtime 无法证明 FFI 的有界阻塞，应报未知义务与外部库路径，不能虚构 WCET。诊断建议不能自动扩大 profile 权限，也不为“通过检查”悄悄去掉用户要求。

编译器/LSP 候选能力：查看函数推导摘要、查询某配置为何可用/不可用、查看包裁剪与实现选择原因、导出机器可读冲突链。离线分析可输出“待特化/未证明”，正式产物必须把所有准入义务解决或明确拒绝。

## 10. 分阶段路线与验收门

**近期设计与实现分期澄清（2026-10-22）**

[AutoShader小范围草案](strategy/autoshader-minimal-language-and-renderer-interface.md)现在只做设计；[830 r2](../plans/830-autographics-surface-portal-composition-and-input.md)只补当前资源/布局/renderer与交接边界，不实施Shader语言或CAP。基础AutoGraphics可先用手写WGSL建立，不以AutoShader完成为前置。

未来AutoShader最小纵向实现需可消费ACE语义/类型/来源接口、所需math与实际图形宿主，不要求先完成完整CAP全部profile/包求解。可先显式有限受检profile/需求摘要，随CAP首版复用同一事实逐步接通；不建立长期平行能力系统。下表C1/C2是互相供证的最小切片，不要求C1覆盖所有场景后才允许C2；完整CAP进入门A/B、各后端实际准入和后继独立Plan保持。



下面是依赖阶段，**不是已经分配的 Plan 或固定版本承诺**。不改变 v0.6 四主线；后续从[远期候选](../roadmap-future.md)选择容量。

| 阶段 | 进入条件 | 最小成果与退出证据 |
|---|---|---|
| A：新版 ACE 完成 | 按 ACE 已有路线实施 | 用户所称“新版 ACE 完成”须在后续实施 Plan 绑定实际交付范围与冻结收据：共享语义/HIR/MIR及适用 backend、真实源码入口、函数增量/cache/受检安装、必要 CTE/元信息可消费。核心骨架成型不自动等于全架构完成；CAP 不要求等所有未来 AJ/自主后端/NPU 项目 |
| B：基础 AutoGraphics 建立 | 图形原型可独立与 ACE 并行；生产接线守原迁移门 | 按 Design45/图形战略完成约定的 wgpu/资源/组合、真实 Auto 源码→图形宿主及必要 CPU/GPU 参数边界；实际 UI/输入/生命周期有证据。单独 Rust driver 或 Iced 探针不算基础已完成；小型 AutoShader 原型可研究但不先造完整 CAP |
| C1：CAP 静态 MVP | **A、B 两门均完成并确认交接** | 先冻结少量原子、host/system/MCU/GPU 的有限 profile、函数/effect 推导和调用图闭包；对显式 dynamic/heap/FS 与静态共享数学做正反例，解释完整需求链；在 ACE 主线上受检集成 |
| C2：图形纵向验证 | 所需最小显式受检profile/摘要及AutoShader/宿主边界；可随C1逐步接通，不等完整CAP | 同一个静态 helper 在受支持 CPU 与 WGSL shader 路径编译运行；非法 host IO/stage/barrier 拒绝；类型化 buffer/uniform 布局、资源释放与错误映射闭环。Shader Components 作为首个主打候选，支持组合/属性更新；具体平台由 Plan 冻结 |
| C3：stdlib/包/选择 | C1/C2 摘要和边界稳定 | core/math 与 host/device 分层、manifest 摘要/条件依赖/锁定、可解释实现选择和裁剪；MCU 正例不拉入 host/GPU runtime，误调 GPU 反例报错；增量变更摘要导致精确失效 |
| D：跨生态扩面 | 每项前置及实际 backend 分别成熟 | 按目标需求恢复 a2c/MCU、GPU compute，再选择 realtime/deterministic、AI tile/NPU/DSP；每个 profile 有独立真实运行、数值/资源/失败 oracle。GPU compute 不因 shader 通过而自动通过，hard realtime/跨设备确定性单独验收 |

C1–C3 的实施时可按最小消费者切片调整粒度，但不绕过 A/B 进入门，不把完整 CAP 加入当前 ACE 或829/830的验收。近期仅登记需求/元信息扩展空间，避免未来破坏 HIR/MIR 与缓存身份，不提前实施求解器。

### 10.1 必需验证矩阵

后续 Plan 至少冻结以下分母与 oracle，不以静态标签或编译成功替代运行证明：

- **静态/动态与传递闭包**：同型 var、跨型赋值、隐式分配、cleanup、递归 SCC、回调/动态分派/泛型实例；正例允许、违规最短链可解释。
- **CPU/MCU/device**：共享函数实际运行；数值容差与陷阱规则预先规定；stage、地址空间、barrier uniformity、资源限额反例拒绝。
- **包/裁剪/实现**：可选无关分支不进入链接/资源闭包、必需非法调用拒绝；FFI 未知摘要、冲突 profile、未知 capability、无候选/歧义都失败可解释；解析锁可复现。
- **增量/恢复/重载**：修改 callee effect、profile、target、backend、包 variant 后正确失效；无关函数身份复用；坏缓存拒绝；候选增加需求被拒时旧版继续有效。
- **保证范围**：realtime 使用冻结硬件/驱动/调度及 WCET 方法；deterministic 记录数值/并行规则与重放输入。缺证据即未证明，不能通过缩小测试数据宣称普遍保证。

## 11. 待冻结决策与参考

待实施 Plan 确定：注解/manifest/CLI 正式语法、首批 capability registry 与 effect 参数域、公开摘要兼容策略、profile 组合与诊断 schema、设备特性探测、CAP pass API及缓存扩展、首个平台/语言子集/资源预算、A/B 实际完成收据。首版只用有限集合与参数约束，复杂 SAT/SMT 求解、运行时多变体调度和自动跨设备划分按证据再选。

参考（外部事实与 Auto 提案分开）：[Slang capability 用户文档](https://docs.shader-slang.org/en/stable/external/slang/docs/user-guide/05-capabilities.html)展示能力推导与平台约束的已有实践；Auto 采用共享语义/effect 与场景约束组合，是本稿建议，不表示复制其 overload 或平台实现。其余外部对比与命名依据见[战略 §2](strategy/cross-ecosystem-programming-strategy.md#2-命名比较与建议)。讨论来源为[《3D引擎方案比较》](https://chatgpt.com/c/6ac4a7c2-1250-83ea-9cdb-bb66f1062eed)。
