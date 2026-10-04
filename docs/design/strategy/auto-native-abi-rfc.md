---
status: proposed
describes: target-state
scope: auto-native-abi-and-hot-reload
created_at: 2026-10-04
updated_at: 2026-10-04
implemented_by_plans: []
supersedes: []
---

# Auto Native ABI RFC：独立版本域、原生模块与热重载

> Draft RFC。用户确认的产品范围与兼容政策见 §1；技术建议尚待评审和原型验证，
> 本文不表示 AC/ACC、ABI 或热重载已经实现。只整理设计，不启动代码计划或分配
> 正式设计/Plan 编号；依照离线开发策略，完整基线恢复后再登记实施计划。
> 编写基线：`v0.6-dev@47f16a173be050d529a73096bf451bd3046a68c9`。
> 本文沿用用户的 AC/ACC 称呼；roadmap 中 Auto 实现的编译器称 AAC，正式命名另行统一。

关联：[v0.6 roadmap](../../roadmap-v0.6.md)、
[离线开发策略](v0.6-offline-development-strategy.md)、
[Atom/Batom v2](atom-batom-v2-design.md)、
[内存与所有权](../04-memory-ownership.md)、
[旧 Native 战略](native-backend-strategy.md)。

## 1. 已确认需求、建议与待决事项

### 1.1 用户确认的范围（2026-10-04）

1. v0.6 ABI 服务 AC/ACC 原生编译。AVM、A2R 和其他转译器的迁移另行安排，
   不作为本文的设计/验收场景；不因旧 roadmap 的代表性探针安排扩大本 RFC 范围。
2. 支持单个可执行文件、独立 Auto 静态模块库、动态模块库，以及插件式热重载。
   动态链接和热重载是自主 AC 与 ABI 的核心动机，不能默默降为未来可选项。
3. ABI 版本独立于编译器版本：编译器 v0.6/v0.7 若都支持 ABI v0.1，可在该域内互操作；
   编译器 v0.8 若仅支持 ABI v0.2，与 v0.1 不直接兼容。这是版本政策示例，
   不是已发布的支持矩阵。
4. 首个实际平台为 Intel/AMD x86-64 + Windows 11；WSL Linux x86-64 可作为第二平台。
   NVIDIA 显卡是开发环境信息，不自动引入 CUDA/GPU ABI 需求。
5. 跨界类型按上一轮推荐细化；linker、x86-32 和运行时参与方式需要给出方案。

### 1.2 本稿建议（尚未裁定）

| 问题 | 建议 |
|---|---|
| Linker | 复用平台 linker；优先评估 Windows `lld-link` / `link.exe`、Linux `ld.lld` / 系统 linker。Auto 自己实现链接驱动与 ABI 检查 |
| 平台 | Windows x86-64 为首个验收平台；WSL Linux x86-64 为第二平台探针，是否成为 v0.6 发布门禁另定；x86-32 暂不纳入 |
| 公开调用 | 复用平台函数调用约定，Auto 自己定义公开类型表示、所有权、符号及运行时契约 |
| 运行时 | 每个进程由一个宿主常驻核心统一提供跨模块服务；可重载模块不携带另一套跨界运行时状态 |
| 热重载 | 新旧代际并存装载；通过常驻入口切换；首版在协作安全点暂停相关调用、迁移状态、提交切换和回收旧代际 |
| ABI 兼容 | 相同 ABI 域是必要条件；还必须匹配 target、公开接口和运行时服务需求，不以编译器版本号代替 ABI 域 |

### 1.3 当前必须保留的待决项

后端具体版本/接口封装与目标 conformance、SDK/sysroot 和 Linux 基线、公开类型/符号的 canonical 描述及指纹算法、
服务表实际字段、状态迁移 Schema 与首次支持范围、线程安全点/重载事务实现、
对外发布 ABI v0.1 的冻结门槛。不得把本文示意结构作为已经冻结的头文件。

## 2. 分层与边界

```text
Auto Source -> typed HIR -> native lowering -> object files
                                |                   |
                         ABI layout/signatures      v
                                             Auto link driver
                                                    |
                                             platform linker
                                                    |
                                          exe / static lib / DLL / so

resident host runtime <-> stable dispatch + module context <-> reloadable modules
                          |
                   generation + state lifecycle
```

ABI 包括数据布局、函数调用、符号/模块身份、运行时与生命周期规则。对象文件格式、
OS 动态装载与 Auto 的模块协调是不同职责，分别复用和实现。

- **模块内部**：未暴露的布局、函数可由优化器自由改变。编译器可内联、标量替换，
  但不得改变公开 ABI；不能把跨重载边界的函数体内联到永久消费者中。
- **Auto 公开边界**：静态/动态独立编译模块共享同一 ABI 契约；静态形态经链接驱动校验，
  动态形态经宿主校验。可重载调用再增加常驻入口和代际规则。
- **外部边界**：C/系统库、第三方后端库按它们的目标平台接口调用。包装入口转换
  Auto 表示；不让 Rust enum、Vec、String 或未经承诺的 Rust ABI 成为公开契约。
- **数据描述**：Atom/Batom 可承载模块描述、布局元数据及可持久化状态；编码不是原生布局。
  普通 native 函数调用不序列化成 Batom。

HIR 应保留函数签名、参数的 view/mut/move 语义、外部 ABI 分类、公开类型身份及显式
布局要求。目标相关的 offset、寄存器和物理签名在 lowering 中确定。HIR/Atom 的内部
Rust 结构布局不成为 Auto 程序的 ABI；本方案不要求先冻结全部 HIR/编码格式。

## 3. ABI 独立版本域与兼容检查

### 3.1 版本政策

把 `0.1` / `0.2` 视为不同兼容域，不能按普通 semver 自动推断两者兼容。
一个编译器可以显式支持多个 ABI 域；消费与生成哪个域必须可查询、可选择并记录于制品。
若新编译器还提供 v0.1 发射模式，可继续生成 v0.1 制品，不能把 v0.2 制品伪装成 v0.1。

同域兼容承诺覆盖不同合规编译器版本及 AC/ACC 实现，不能在兼容键中强制要求
编译器二进制完全相同。编译器版本、后端版本仍记录为来源/诊断与构建缓存信息。
公开布局不得随 debug/release、worktree 路径或随机布局标志变化。

与旧 Native 战略的差异：旧文按同编译器版本定义域；本 RFC 按用户新裁定，
提出独立 Auto ABI 域。本文不改写旧历史决定，也不启动 fork-rustc。

### 3.2 模块兼容检查模型

建议分开记录：

| 身份 | 意义 |
|---|---|
| `abi_domain` | 例如 Auto Native ABI v0.1 |
| `target_profile` | CPU/OS/平台 ABI、指针宽度、字节序、调用/布局 profile；必要的系统库基线 |
| `module_identity` | 逻辑包/模块身份，与当前检出绝对路径无关 |
| `interface_descriptor` | 公共符号、签名、类型布局、所有权、错误规则及导入要求的 canonical 描述 |
| `runtime_requirements` | 服务表版本/大小及所需能力；运行时能否满足该需求 |
| `state_schema` | 持续状态的独立版本；ABI 相同不意味着状态可直接沿用 |
| `artifact_identity` | 内容身份与构建来源，用于选择不可变制品和缓存，不等同于接口身份 |

**允许互调 = ABI 域相同 + target/profile 兼容 + 导入的公开接口得到满足 + 运行时能力满足。**
公开接口中的布局相关配置必须进入描述；不盲目把所有私有 feature、优化级别和依赖 hash
放入兼容键。实现改变但接口不变应允许替换；修改公开字段/签名仍会破坏该模块接口，
即使语言级 ABI 域没变。新增未被消费者使用的导出，不应导致全部消费者必然失效。

指纹用于快速比较，原始 canonical 描述保留以解释冲突；hash 不是绝对唯一性证明。
类型身份需包含公开命名空间与结构/布局/所有权信息，不能只取类型名或本地 TypeId。

### 3.3 装载前后的校验

Auto 制品需要机器可读描述，静态 archive 中也保留成员/依赖信息。具体 section、
sidecar、Atom/Batom 编码和魔数另行验证；bootstrap 入口及它读取的固定头部必须独立定型，
否则会出现“先用未知 ABI 调函数，才检查 ABI”的循环。
制品还需明确 artifact kind，区分 Windows 静态 `.lib` 与 DLL import library，
不能仅凭相同扩展名决定模块包含的代码和运行时依赖。

先从文件描述检查 target/域/依赖，再调用 OS loader；装载后通过固定 bootstrap 入口
取得接口表并核对描述的一致性，成功后才执行 Auto 显式初始化。绑定描述与制品内容，
拒绝错配 sidecar；描述格式不支持时确定性拒绝，不猜测。

OS loader 可能先执行 DLL 入口/依赖初始化或 ELF constructors，因此文件预检与
bootstrap 都不是同进程恶意代码沙箱。合规 Auto 模块的这些入口仅做必要平台工作，
业务初始化/迁移须显式执行。

## 4. 首个平台、x86-32 与后端

| Profile | 推荐定位 | 制品与平台约定 |
|---|---|---|
| `x86_64-pc-windows-msvc`，Windows 11 | 首个原型与发布验收平台 | PE/COFF、Windows x64 函数 ABI、Windows SDK/import libs、PDB 路径探针 |
| `x86_64-unknown-linux-gnu`，WSL2 Linux | 第二平台验证建议，发行版/glibc 基线待定 | ELF、System V AMD64 psABI、所选 sysroot、动态 loader；须在 Linux 实际执行 |
| x86-32 | v0.6 暂不支持的建议 | 若有实际设备/遗留库需求再立项，独立 target/profile 与验收 |

同为 x86-64 不意味着 Windows/Linux 制品可互加载；WSL 有效验证 Linux 进程，
不能替代 Windows 图形/GPU 或另一个 Linux 发行环境的验收。
Linux musl、Windows GNU target 不自动继承上述 profile 的兼容承诺。

x86-32 会增加指针/usize 宽度、调用约定、布局、链接/装载和运行时测试面；
“已有后端支持 32 位”不等于 Auto ABI 已支持。当前需求无 32 位消费证据，暂不扩大范围。
编译器拒绝未支持 target，不依靠截断指针运行。

推荐基础 CPU profile 不要求开发机特有的 AVX2/AVX-512；额外 ISA 要写入制品要求并检查。
GPU 发射、CUDA 互操作、设备内存等不属于本文首批 CPU ABI。
2026-10-04 用户确认 v0.6 AC/ACC 主路径使用 Cranelift；Windows/模块必需能力仍须探针，
若不成立再有证据地评估 LLVM 备选。v0.7 自主后端可并存，不能仅更换后端就改变公开 ABI。
完整路线见 [AC/ACC 后端演进](auto-native-backend-evolution.md)。
支持平台 C 约定不意味着复杂 C 聚合类型分类自动完成。

Windows 参数/栈/返回及展开元数据要求见
[Microsoft x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170)。
Linux 规则以 [x86-64 psABI 项目](https://gitlab.com/x86-psABIs/x86-64-ABI) 的规范为依据，
实施时固定所采用规范版本并生成 conformance 语料。

## 5. Linker：复用底座，自主驱动

### 5.1 推荐分工

“C/Rust 的 linker”通常指这些工具链使用的目标文件链接器。linker 处理符号、section
和重定位，不要求程序源码用 C/Rust 写。Auto 可以定义自己的符号和调用协议，
只要对象格式、重定位与平台装载要求符合所选工具链。

| 组件 | v0.6 推荐职责 |
|---|---|
| Native emitter | 生成目标文件、导出/导入、调试/展开信息及 ABI 描述 |
| Auto link driver | 校验模块 ABI、解析依赖和 sysroot、确定入口/运行时、生成 response file/导出清单、调用 linker，记录可复现输入 |
| 平台 linker | 生成 exe、DLL/so、导入库并完成实际符号解析/重定位；static archive 使用相应 librarian/archiver |
| OS loader | 映射 PE/ELF，处理平台依赖和装载操作 |
| Auto module runtime | 协商服务、绑定接口、管理代际和状态、切换入口与安全回收 |

优先评估 `lld-link` / `ld.lld`，允许 `link.exe` 或系统 linker 作为显式替代。
使用 LLD 不要求 emitter 必须使用 LLVM；也不表示“装一个 linker 就有 SDK/sysroot”。
Windows SDK、必要 CRT/编译辅助库及 Linux 启动文件/系统库的定位属于 link driver。
本次没有安装或验证这些工具，不能视为本机可用性收据。

LLD 官方说明支持 ELF、PE/COFF 和 Windows PDB；Windows 端可与相应工具链配合。
具体选项/兼容性需固定版本验收。[LLD](https://lld.llvm.org/)、
[LLD Windows support](https://lld.llvm.org/windows_support.html)

### 5.2 自写 linker 的可能收益与代价

| 可能收益 | 获得收益必须实际完成的工作 |
|---|---|
| 函数/模块粒度增量链接 | 缓存与依赖图、地址/重定位更新、失效和并发模型 |
| 内存链接、JIT 或特殊热补丁 | executable memory、符号/重定位、平台展开注册及安全切换 |
| 自定义制品布局/加载方式 | 格式、装载器、调试器与系统生态适配 |
| 控制编译/链接延迟 | 测量瓶颈，处理代码尺寸、调试信息、优化与工具链兼容取舍 |

这些是可研究收益，不是自写后自然成立。代价包含 COFF/ELF、archive/import libs、
重定位、TLS、展开/调试、SDK 库兼容及长期维护。复用 linker 仍允许模块增量构建，
仅重链被修改模块，再由运行时换代；不需要每次重链整个宿主。

建议 v0.6 先保留 linker adapter 接口，用实测决定以后是否自研。热重载首先依赖
可替换模块边界和生命周期，常驻入口/状态协调也不是传统 linker 自动提供的能力。

## 6. 首批公开类型和物理签名建议

本节只限制公开稳定边界的物理表示，不限制编译器内部或模块内部的全部语言能力。
更复杂的源码类型可降低为受支持表示/句柄；无法正确降低时显式诊断。

| 类型/语义 | ABI v0.1 候选规则 |
|---|---|
| `i8/u8/i16/u16/i32/u32/i64/u64` | 宽度固定；标量按目标约定传递 |
| `f32/f64` | 明确 IEEE 表示与对应目标浮点参数/返回规则 |
| bool | Auto 公共表示暂定 u8 的 0/1；非法值拒绝；C bool/Windows BOOL 按外部声明转换 |
| `usize/isize`、指针 | 当前 profile 为 64 位；只能在同进程/target 使用；公开接口优先明确宽度 |
| 简单 struct / 固定数组 | 显式 ABI 类型：声明序、自然对齐、padding/尾填充可计算，包含类型递归满足公开规则；首版统一间接传参/显式 out 返回 |
| UTF-8 文本/字节/typed slice 借用 | `{ptr, len}` 描述，通过指向描述的参数传递；规定元素、长度、对齐和有效期，文本不要求 NUL |
| 资源/拥有型对象 | 宿主管理的不透明 handle，定义创建、借用、转移、销毁；不导出容器内部结构 |
| 回调 | 明确函数签名与 context；可重载回调使用宿主常驻 trampoline/注册 token，取消后排空调用 |
| enum/Option/Result | 按具体 API 转为显式 tag+payload 或 status+out；首版不把 niche/私有优化布局暴露出去 |

源码 `int/uint` 的精确宽度仍由语言类型规范决定，HIR 必须提供解析后的实际类型；
ABI 不从 C 的 `int/long` 或宿主 Rust 类型猜测它们。handle 候选为固定宽度的 opaque token，
包含可校验的注册身份/代际；实际 bit allocation 和复用规则需避免 stale token 被误认。

struct 布局候选算法：字段 offset 向上对齐到字段 alignment；struct alignment 为最大
字段 alignment；最终 size 向上对齐到 struct alignment。首版公开 struct 不含隐式
packing、bitfield、未定义大小、未经确认的特殊对齐类型；这些支持需明确增补规则。
padding 不承载数据，不要求可按字节比较，更不自动成为可序列化表示。

### 6.1 参数、返回和借用

- 简单 scalar/pointer 返回沿用平台约定；聚合值首版使用显式 out pointer，不借助
  未定义的跨平台隐式 sret 规则。此为 Auto wrapper 物理签名，不改变外部 C 函数原型。
- `view` 的 scalar 可以物理复制，但仍保留源码只读语义；struct/slice view 通过借用
  指针。`mut` 必须满足独占/别名纪律，不能因为 ABI 接收裸指针就忽略语言检查。
- 不含资源的值类型 move 可由生成 wrapper 接管存储；资源 move 通过定义的 handle
  转移协议。成功/失败是否消费参数须逐接口声明，不能只凭 `.move` 猜测失败路径。
- 借用默认只在当前调用有效，callee 不得保存。需要跨调用保存时显式复制或转换为
  宿主管理的所有权/lease；重载前排空可指向模块存储的借用。
- `len=0` 候选允许 null，非零长度要求有效、对齐的缓冲区；边界检查元素数乘法溢出。
  返回借用默认限于宿主管理对象/lease，禁止把临时栈或可卸载全局存储作为长期返回值。

### 6.2 外部调用与复杂源码类型

C FFI 使用真实目标 C 类型与聚合传参规则；Auto struct 不能无条件 reinterpret 成 C struct。
`c_long`、`size_t`、BOOL、UTF-16/NUL 字符串等由目标 adapter 定义并验证。首批 FFI 可
限定标量/指针和已验证 struct；C varargs/复杂聚合按明确子任务增加，不以错误原型绕行。

外部库分配的内存/对象由它规定的释放入口释放，转换成 Auto owned 对象时明确复制或
托管 foreign deleter。foreign deleter 依赖的库同样必须保持装载。

泛型公开入口使用已具体化的签名/实例；泛型模板的分发、编译缓存不自动成为二进制契约。
闭包/spec 对象/动态值可先用 handle+typed wrapper 暴露；代码或 vtable 依赖某代模块时
需 pin 该代或迁移，不能仅靠共享 allocator 卸载代码。完整 Future/任务对象布局不在本节冻结，
如自举确需跨界则在对应子集中补充，不能静默假设全语言已覆盖。

## 7. 运行时：宿主常驻核心与模块上下文

### 7.1 推荐的最小运行时参与方式

无需把每个算术/纯函数调用变成 runtime 调用，也不默认引入 tracing GC。
运行时只承担跨模块必须统一的服务和重载生命周期。

| 常驻服务 | 最小职责 |
|---|---|
| allocator | 明确 alloc/free 的尺寸、对齐、失败规则；跨 Auto 模块拥有型存储使用同一服务域 |
| handle registry | 对象/资源身份、类型、owner、代际校验；拒绝 stale handle，管理 move/release |
| module manager / dispatch | 兼容检查、导入绑定、稳定入口、调用 lease、代际引用、重载事务 |
| diagnostics / errors | 错误码、错误对象所有权、日志及明确的边界失败行为 |
| lifecycle registration | 线程、任务、回调、析构/vtable、资源 cleanup 的代际归属与排空 |

标准库文件/网络/时间等功能可按需作为普通库或服务层提供，不要求第一天放进核心表。
运行时的语言实现可先 Rust 引导，经固定布局和平台 ABI 的 wrapper 提供服务，
以后迁到 Auto；实现语言不改变公开运行时协议，也不依赖 Rust ABI 永久稳定。

### 7.2 服务表与部署

建议 bootstrap 接收常驻服务表和 module context，返回经验证的模块描述/入口表。
以下是职责示意，字段名、版本宽度和函数物理签名尚未冻结：

```text
HostServices { header_version, table_size, capabilities, alloc, free,
               handles, diagnostics, lifecycle, ... }
ModuleDescriptor { abi_domain, target_profile, module_identity,
                   interface_descriptor, runtime_requirements,
                   imports, exports, state_schema, lifecycle_hooks }
```

扩展表使用版本/长度/能力协商；追加字段不能让旧模块越界读取。核心字段改变语义则升级
对应服务协议或 ABI 域，不能只增 table_size。bootstrap 自身的兼容头部单独明确。
需要业务状态的公开入口显式接收 module context；纯函数可采用不带 context 的物理签名，
两者都进入 canonical 接口描述。生成的 wrapper/trampoline 负责取得当前 context，
不能隐式依赖另一个 DLL 的 TLS 或未记录的全局变量。

宿主可将核心静态链接进 exe，也可使用一份不可热卸载的 runtime DLL/so。
纯静态程序同样允许只带一份核心；模块内部没有跨界用途的私有存储可独立管理。
可重载 Auto 模块不得各自静态复制 handle/allocator/dispatch 的跨界状态。
运行时/宿主的原地热替换不属于首版模块重载范围；需要升级核心时重启宿主。

分配服务统一并不能解决对象析构代码寿命：注册在宿主里的 module destructor
会 pin 对应代际，只有销毁/迁移/改绑后才能解除。
Windows 跨 DLL 使用不同 CRT/heap 的对象/释放函数可能出错；因此本稿建议跨界使用
明确宿主服务或原创建方的释放函数，而非依赖恰好相同的 CRT。
[Microsoft CRT boundary guidance](https://learn.microsoft.com/en-us/cpp/c-runtime-library/potential-errors-passing-crt-objects-across-dll-boundaries?view=msvc-170)

### 7.3 错误与展开

可失败公开入口建议返回明确 status，成功值走 out 参数，错误详情走宿主管理 handle。
首版跨模块不传播 Auto panic、Rust panic、C++ exception、longjmp。
可恢复错误由 wrapper 转换；无法安全恢复的内部故障明确终止/报告，不能伪造正常返回。
Rust 引导实现不能假定 catch_unwind 能捕获 abort、访问违规或任意外部异常。

“不跨界展开”不意味着 Windows 不需要展开/栈信息；后端仍须满足平台规则及调试需求。
错误返回的 out 参数初始化/销毁规则、错误 handle 是否消费、callback 错误传播，
必须在最终头文件中逐项写清楚。

## 8. 动态模块与热重载契约

### 8.1 常规动态库与可重载模块

普通动态库可以使用平台导出/导入并在进程生命周期固定绑定。
可重载模块的跨模块调用必须经过常驻 dispatch 或宿主 trampoline，不依靠 OS
自动把已经缓存的函数指针/IAT 引用切到新 DLL。
对外 C 导出若也要保持调用地址，公开地址由常驻宿主提供；直接得到旧 DLL 函数地址的
foreign caller 必须注销/重新绑定，无法证明退出时拒绝卸载。

模块内部调用可直接调用和内联。跨可重载边界不允许把被调用实现直接静态内嵌到
永久消费者；优化必须知道 reload boundary。某 static lib 链入 reloadable DLL 后随该代
整体替换；static lib 本身不能单独靠装载操作实现热替换。

### 8.2 代际、调用 lease 与发布

每次重建生成不可变且代际唯一的路径，如 `module.<artifact-id>.dll` / `.so`，
新旧模块可同时存在，不覆盖正在使用的映像。可重载依赖通过 host imports 解析；
避免新模块的 OS 依赖意外指向旧代模块。必要的 native 依赖图要进入装载/回收记录。

进入模块时同时取得正确的代际和调用 lease，返回后释放；取入口与取得 lease 必须有
一致的同步协议，不能“先读指针、再加引用计数”，否则可与卸载竞争。
多入口/多模块相关变更通过一次发布 generation table 完成，不能逐槽切换制造混合代际。
编译器生成边界入口，运行时实现协调；具体 lock/epoch 算法通过并发原型后确定。
首版的相关模块组以暂停后整体发布为基础；消费者也必须遵守代际一致的调用协议。
只原子替换一个指针但让调用者分次取得不同代际的入口，并不能保证一次业务操作一致。

### 8.3 首版重载事务（协作安全点）

1. **Build / Preflight**：生成新制品，检查描述、接口、runtime 与依赖要求；旧模块继续服务。
2. **Load / Prepare**：装载新代并显式准备，尚不公布入口；失败释放准备资源。
3. **Quiesce**：关闭相关新调用，取消并排空异步回调/任务，等待在途调用和借用归零，
   协作线程到达安全点。超时取消重载并恢复旧代服务，不强杀仍执行旧代码的线程。
4. **State / Validate**：保留旧状态，创建或迁移候选状态并验证。首版可暂停相关模块，
   不承诺任意程序零停顿；独立状态 Schema 按 §9 处理。
5. **Commit**：以一次事务发布入口表与状态 context；从此新调用使用新代。
   对外通知/不可逆副作用在提交后处理；提交前准备/迁移必须可取消、可清理。
6. **Retire / Reclaim**：在旧代码仍装载时调用显式 cleanup，解除其所有注册/资源引用。
   确认旧代无可执行依赖后调用 OS unload，记录实际回收结果。

准备/迁移/检查失败时，销毁候选状态，入口和旧状态保持可用。提交后不是任意撤销历史；
若要回滚已运行新代码后的业务状态，需另有显式快照/迁移协议。
清理失败则旧代进入可诊断的隔离/pinned 状态并限制后续重载，不能以无限积累 DLL
作为成功回收，也不能为清理资源强行跳过生命周期约束。
Quiesce 必须区分新的顶层调用与已经取得旧代 lease 的嵌套调用/收尾 hooks，
允许在途调用按明确协议完成；不能关掉所有入口后等待一个还要回调模块的旧调用。
协调锁不得在等待线程、执行用户 hook 或任意外部回调时持有；须用负面并发语料核查死锁。
迁移不能透明撤销已经发生的文件/网络/设备副作用，模块须明确其准备与提交边界。

### 8.4 可卸载条件

| 仍依赖旧代的实体 | 必须如何处理 |
|---|---|
| 在途函数、栈返回地址 | 等待返回/安全点，调用 lease 归零 |
| 外部回调/捕获闭包 | 注销并排空；或经常驻 trampoline 改绑 context/实现 |
| 对象析构、vtable、type operations | 在旧代仍装载时销毁/迁移；无法迁移则 pin 并报告 |
| 线程/任务、TLS destructor | 协作退出或明确解绑；首版禁止不可追踪的后台执行/代际 TLS 资源 |
| 模块全局状态、资源 cleanup | 明确 module context 与显式 hooks；不把存活资源留给未知 unload 顺序 |
| 普通 DLL/so 的直接依赖引用 | 记录并解除/重载相关组；不能只看 Auto 调用计数 |

Windows DllMain 持有 loader lock，不在其中做业务初始化、迁移或等待线程。
FreeLibrary 的平台引用计数不能代替上述 Auto 生命周期检查。
[DLL best practices](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices)、
[FreeLibrary](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary)

Linux 初版建议固定依赖并使用局部、立即解析的装载模式；不依靠全局符号覆盖实现换代。
dlclose 成功不保证映像真的从地址空间移除；回收验收区分 Auto 引用解除、应用资源释放
和 OS 映像驻留。[Linux dlopen/dlclose](https://man7.org/linux/man-pages/man3/dlopen.3.html)

## 9. 状态保留和迁移

ABI 域、公开接口版本、状态 Schema 是三个不同维度：ABI 相同的模块可以有不同状态结构；
ABI 不同的模块也可能导出可转换的数据，但仍不能直接二进制互调。

首版建议区分：

| 状态种类 | 重载处理 |
|---|---|
| 宿主拥有且约定表示不变的值/资源 handle | 经校验保留，module context 重新绑定 |
| 模块内部私有持久状态 | 旧代导出逻辑快照，新代迁移到新结构，准备成功后提交 |
| 临时缓存/可重建状态 | 显式声明丢弃并重建，不能顺手丢掉用户持久状态 |
| 指针、闭包、GPU/文件等实时资源 | 不直接序列化；使用宿主资源 token、专用 adapter 或拒绝该次迁移 |

候选 hooks：prepare、quiesce、export_state、import_state、activate、retire、cleanup。
最终是否合并 hooks、参数/失败规则通过原型决定；这些名字不是已提供的 API。
迁移导出不可修改旧状态，导入使用独立候选存储；无正确迁移路径时明确拒绝重载。
普通 native 类型的 ABI layout hash 不能代替状态 Schema，不能盲目 memcpy 新旧对象。

Atom/Batom 适合逻辑快照，但热重载架构不等待其字节格式冻结：先用显式 typed state
和版本化 adapter 验证事务，再接入已确定 codec。若以后宣称通用持久化快照/跨进程
迁移，则必须验证真正的编码和资源重建；目前只规定同进程重载所需的状态契约。

## 10. 与 AC/ACC、自举及构建缓存的关系

AC 用 Rust 实现也可以直接产出 Auto ABI 制品；ACC/AAC 用 Auto 实现后遵守相同规范。
源语言语义、公开 ABI 的 lowering、模块描述生成和链接驱动必须有共同 conformance
语料，不能靠“实现看起来一样”承诺跨编译器兼容。

后端选型时检查 COFF/ELF、平台调用约定、调试/展开、PIC/外部符号、入口表发射能力。
外部编译器后端库通过它提供的受支持接口/桥接消费，不把其私有布局纳入 Auto ABI。
Rust AC 直接调用 Cranelift；Auto ACC 自举后继续经明确桥接调用，
优先评估薄 Rust 库的受限 C ABI，亦可先研究独立后端进程。
编译器内部桥接协议与生产 Auto 模块 ABI 各自版本化，明确内存所有权、错误、能力与释放。
优化器须保持 effect、trap/释放及 reload boundary；自主后端并存时共用 ABI conformance，
后端版本进入构建缓存身份，不自动进入公开 ABI 兼容域。

构建缓存身份需包括源码/IR、编译器/后端、目标及代码生成配置；这与 ABI 兼容域分开。
两个实现不同的制品可以 ABI 兼容，两个内容相同但 target 不同的描述也不能互加载。
符号/接口身份不包含绝对 worktree 路径；路径重映射、制品复用是后续构建策略工作。

## 11. 分阶段验证与冻结门槛

当前只交付 RFC。正式实施应拆成中等规模 Plan，沿用设计审批、专用 worktree、
按改动范围验证和独立复审；不占用未知主电脑 Plan 号段，不在本次写入 current-state specs。
下表是覆盖顺序建议，不表示动态库或热重载退出 v0.6 范围。

| 阶段 | 必须得到的证据 |
|---|---|
| A：物理 ABI 子集 | 选定 Windows 工具链；标量/float/struct 间接参数/返回、嵌套调用；对象文件真实链接执行；C 双向互调与非法声明诊断 |
| B：模块与运行时 | 独立静态库及 DLL 构建消费；bootstrap/服务表/公开接口校验；allocator、handle、borrow、error 的生命周期；缺失依赖/能力确定性拒绝 |
| C：热重载闭环 | 宿主不重启，入口调用行为改变；状态保留与改变结构后的显式迁移；在途调用和 callback 排空；失败候选不影响旧代；旧代实际可回收 |
| D：编译器一致性 | AC/ACC 互为 producer/consumer（静态、动态）；两个宣称 ABI v0.1 的合规编译器版本组合；debug/release 公共布局一致；v0.2 不匹配显式拒绝 |
| E：第二平台 | 在固定 WSL Linux 环境重复代表性 conformance/module/reload 语料；不把仅交叉发射视为执行验收 |

热重载验收必须包含：

- 在并发取入口/进入模块时反复切换，验证 lease 无卸载竞争；相关组不出现混合代际。
- 在途旧调用、后台任务、外部回调、带 destructor 对象分别阻止过早卸载。
- 状态 Schema 不变和改变两种成功路径，以及迁移失败、缺失迁移、超时、加载失败路径。
- 连续多代重载的 host handle/资源/内存/映像记录；如可追踪实体阻止回收，应显示原因。
- 不兼容 ABI/target/接口/服务表、错配描述、旧 handle、借用失效的负面语料。
- 避免一次输出变化的 demo 代替回收、状态和失败恢复验收；具体循环次数/负载上限由 Plan 固定。

ABI v0.1 的对外冻结至少需要：规范化头文件/描述、公开类型 golden layouts、签名/符号
向量、编译器与运行时共同 conformance、模块与重载收据、支持/不支持矩阵和版本变更规则。
只有 AC 单版本原型时可称“ABI v0.1 草案原型”，尚不能宣称跨编译器版本兼容已验证。
同进程 native 模块中的未知 raw pointer 逃逸不能靠计数器自动发现；可重载 profile
必须约束/检查这些逃逸，未受管理的 FFI/后台执行需登记或明确拒绝卸载。

## 12. 本轮建议的评审重点

1. 是否采用平台 linker + Auto link driver，保留将来自研替换接口？
2. 是否按独立 ABI 域定义版本，公开接口和运行时能力另外核对？
3. Windows x86-64 首验收、WSL Linux 第二平台与 x86-32 暂不支持的范围是否合适？
4. 首批公开类型是否接受 scalar + 间接 struct/slice + owned handle + callback wrapper？
5. 是否采用常驻宿主服务与显式 module context，以及协作安全点的首版重载事务？
6. 状态迁移是否接受显式 Schema/adapter，暂不承诺任意对象自动迁移或任意线程零停顿？

以上技术选择批准后，再与 Atom/HIR 负责线对齐必要语义，开展后端/运行时探针。
现在不等待全格式冻结，也不要求 ABI 全部细节先完成才允许 HIR 原型推进。

## 13. 来源与证据边界

- 用户 2026-10-04 在本会话确认的范围与独立版本域政策是 §1 的依据。
- v0.6 roadmap / 离线策略是本轮工程基线；Atom/Batom 草案只作为未冻结的设计输入。
- 旧 Native 战略、所有权与错误设计提供历史意图，不自动视为已实现或已适用于新 ABI。
- 先前“聊天”模式 ABI 深聊的完整正文尚未定位并对账，本稿不声称已完整继承其所有结论；
  后续取回正文后应审计冲突/遗漏，保留本轮用户确认项与候选方案的区分。
- 外部依据均为文中链接的官方 linker 文档、Microsoft 平台文档和 Linux man-pages；
  查询日期 2026-10-04。本文只做设计与文档核查，没有运行 native 构建、装载、
  热重载、WSL 或 ABI conformance 测试，不是平台兼容性收据。
