# Storage 与内存分配资源

> **状态：设计方向记录 r1 / target-state｜2026-11-09**。
> 本文属于 [Design04：内存与所有权](../04-memory-ownership.md) 的内存分配专题，记录用户本次认可的方向。具体接口、语法、ABI 表示、实现与验证门，等 M2 实现标准库时继续讨论后冻结。
> 本轮仅整理设计文档，不启动实现、不新建实施计划、不修改现有计划任务/验收门、不将 proposed 设计写成 current Spec。下文示意类型不代表 ACE 已支持相应泛型或 Storage 实例化。

## 1. 结论与设计目标

**保留 Storage 作为 Auto 面向用户的主要存储模型；prelude 提供默认策略；Storage 实例保存实际存储与资源来源；按需要在其下抽取可共享的分配接口。**

不因为借鉴 Zig 就要求每个分配函数都新增显式 allocator 参数，也不把独立 Allocator 类型/关键字或动态接口作为已定方案。需要保留的是内存来源明确、实例可区分、生命周期可验证和失败可处理这些能力。

类型级 Storage 已能表达 Heap/Fixed 的布局与容量差异；仅靠类型选择和 prelude 导入，仍不足以处理多个资源实例、共享 arena、跨边界释放及完整标准库的分配。扩展时优先沿现有 `List<T, S>`/`Storage<T>` 体系，避免两套重叠用户模型。

标准库核心不强依赖 libc。Heap 的含义是动态存储策略，不承诺其实现一定调用 C `malloc/realloc`；平台内存、Auto 自有分配器、显式 libc adapter 可以是不同来源。使用 C ABI 不等于依赖 C 标准库。

## 2. 已有设计与当前证据

本轮只读取主检出 HEAD `b4f1af0be` 及当时磁盘文档，不以其他会话 WIP 或旧文档中的“已支持”概述代替新版交付证据。

| 来源 | 已观察到的内容 | 不能据此推定的能力 |
|---|---|---|
| [数据结构设计](../07-data-structures.md) | `List<T, S>`、Heap/Inline 存储及自定义 Storage 的设计 | 全泛型/全后端 Storage 已支持或已经无堆 |
| [storage.at](../../../stdlib/auto/storage.at) | `Storage<T>` 有 `data()`、`capacity()`、`try_grow()`；Heap 有 ptr/cap | 完整资源生命周期、失败原子性、任意 allocator 已具备 |
| [list.at](../../../stdlib/auto/list.at) | `List<T, S>` 保存 `store S`；文件同时列出泛型/约束限制 | 已完成 ACE 公共 List/Storage 全链 |
| [inline.at](../../../stdlib/auto/inline.at) | 内联固定容量存储声明与目标实现 | 固定存储一定在栈上，或所有操作不会失败 |
| [prelude.at](../../../stdlib/auto/prelude.at)、[default_storage.at](../../../stdlib/auto/default_storage.at) | 存在 prelude 和默认策略愿景/示例 | 环境默认注入已全面交付；[类型计划记录](../../specs/auto-lang/types/plans.md)仍把055注入列为未落地 |
| [VM list_storage.rs](../../../crates/auto-lang-core/src/vm/list_storage.rs) | `InlineInt64Storage` 当前内部使用 `Vec<Value>` | 该 VM 实现可作为“零堆分配”运行证据 |

这些差额用于 M2 重核，不在本次顺手修复。早期文档的“Inline 全在栈上、无分配失败”应按本稿细分：内联描述布局，固定描述容量；运行时表示和失败行为分别取证。

## 3. 四项职责分开

| 层次 | 回答的问题 | 拟采用的组织 |
|---|---|---|
| 默认政策 | 不写 S 时选什么 | prelude/目标构建配置给出默认 Storage，允许显式覆盖 |
| Storage 类型 | 元素放哪里，如何增长 | Fixed/Heap/Arena/Pool 等策略；决定布局、容量与可用操作 |
| Storage 实例 | 具体使用哪块内存、哪个资源 | 自带内联缓冲区，或持有特定资源实例及必要的所有权/借用 |
| 底层分配服务 | 如何申请、调整和回收字节 | 需要时抽取尺寸/对齐/失败/回收合同；名称和接口待 M2 冻结 |

容器拥有多少已初始化元素、元素何时析构，由容器与语言所有权规则决定；原始存储由 Storage/资源负责。分配与释放的来源合同不替代 view/mut/move 或自动析构规则。

示意关系：

```text
List<T, S> 持有 Storage 实例
    ├─ Fixed：直接拥有固定缓冲区
    ├─ Heap：使用确定的动态内存资源
    └─ Arena/Pool：使用某个具体区域或池实例
                      ↓
              按需复用底层分配服务
```

同一 `ArenaStorage<T>` 类型可以对应请求 A 和请求 B 的两个 arena。类型说明策略，实例决定实际区域；两个容器也可以在规则允许时共享同一个资源。现有 `store S` 结构可以承载有状态对象，不需要先发明另一套类型系统。

## 4. Prelude 默认策略

桌面/服务器默认 Heap，受限环境默认 Fixed 或明确的区域策略，属于合理的便捷政策；**政策不应仅由 OS 硬编码**。Linux 应用也可能要求无堆，MCU/RTOS 也可能具备通用堆或内存池。

默认策略由确定的目标/构建配置选定，不能由运行时环境随意改变，也不能因调用点导入了不同 prelude 就重新解释已编译对象。显式 Storage 是高级控制入口，默认机制不能禁止它。

省略 S 的 `List<T>` 必须在类型解析阶段落实成确定类型，具体 Storage、容量等布局参数进入类型身份及有关编译/缓存依赖。修改默认政策可能改变布局或失败条件，不能视作无影响的导入调整。

公共算法宜按 Storage、切片/视图或迭代协议表达需求，避免把某个应用的默认 Heap/Fixed 固定为所有调用者的唯一表示。可使用更少操作的实现不必被迫满足“总可增长”。具体泛型接口留待 M2。

## 5. 实例、生命周期与失败合同

以下是后续实现需要回答的语义问题，不是本次冻结的函数签名或诊断码。

| 事项 | 设计方向 |
|---|---|
| 资源注入 | 默认实例便捷构造；高级调用者能提供某个具体 Storage/资源实例。构造语法、由值持有还是借用待定 |
| 来源与释放 | 确保内存由正确资源/原方回收，不能从当前默认资源反推旧对象来源；运行中切换默认策略不能改变旧对象的释放方 |
| 生命周期 | 容器及存活 view 不能超出 backing buffer/arena 的有效期；reset/整体释放不能留下仍可使用的引用 |
| 移动与增长 | Fixed 随所属对象移动、Heap 扩容、容器 swap/move 等动作，必须遵守借用与地址有效性规则；不能留下指向旧内联地址的内部指针 |
| 元素与原始内存 | 明确已初始化区间、元素析构与缓冲区回收的分工；arena 回收字节不自动等于元素资源析构完成 |
| 失败 | 容量不足、OOM、预算、取消和非法访问分开；失败前检查/准备与成功提交的边界明确，不丢原数据或重复消费输入 |
| 尺寸与对齐 | 申请、增长、拷贝中的字节数/元素数、溢出与对齐必须受检；不能照搬某个 libc 的偶然行为 |
| clone/转换 | 明确目标 Storage/资源、成本及失败行为；不能把 Fixed→Heap 或跨 arena 深复制伪装成无代价转换 |
| 并发与 FFI | 资源实例的线程约束及外部 deleter 明确；外部库分配的对象不能直接交 Auto 默认资源释放 |

Fixed 是“固定容量/内联布局”的设计族，不等于静态全局存储或栈存储：它随所属对象存在。Fixed 本身不扩容，也不表示 push 等操作总能成功；容量耗尽如何表达必须独立规定，不允许静默丢数据或切到 Heap。

是否为库作者公开独立 allocator spec、是否提供类型擦除资源句柄、哪些接口静态分派，均待实际消费者证明需要后再定。公共 ABI 的服务适配不应强迫全部内部调用采用动态分派。

## 6. 无堆、区域分配与整体程序保证

示意 `List<Str, Fixed<16>>` 只能说明外层容器的策略；拥有字符串的 backing 仍可能分配。HashMap 节点、格式化/JSON 临时值、闭包环境和错误构造也必须纳入真实可达分配闭包。

| 模式 | 含义 |
|---|---|
| 无 libc 堆依赖 | 不使用 C 的 malloc/free；仍可有 Auto 分配器和 OS 内存来源 |
| 无通用堆、允许区域分配 | 可以从预留 SRAM/固定 buffer/arena 申请空间；仍有运行期分配与容量耗尽 |
| 严格无运行期分配 | 使用预先构造的对象、固定容量容器及调用者缓冲区；所需操作闭包无分配 |

三者不能互相冒充。只换 List 的默认 Storage 不构成全程序无堆证明。M2 先通过明确的接口、支持表和代表性真实执行核对；完整自动需求推导继续遵守 [Design46](../46-context-profile-capability-cross-ecosystem.md) 的进入门，不为本专题提前建设完整 CAP。

## 7. AutoABI、平台层与标准库

不同 Storage 可能产生不同大小、布局和生命周期。独立模块不能仅因为都拼写 `List<T>` 就默认布局相同。跨界应采用明确的具体合同、受检视图或拥有资源表示；如需资源服务/原方 destroy，来源与保活规则进入接口合同。具体 AutoABI 表示留给既有 ABI owner，不在此定义通用 List wire 或公开泛型 ABI。

公共算法尽可能 Auto 单源，容器通过 Storage 获得内存；底层平台服务只在需要时提供页/区域或系统接口。Fixed/用户提供 buffer 不应被迫引入 OS 或 libc。外部 C/Rust 库的依赖和析构按原 provider 合同保留，不能因核心库无 libc 就假定外部闭包同样无 libc。

本稿衔接 [Design54](../54-stdlib-semantics-and-ace-integration.md) 的统一操作/资源合同、[Design56](../56-ace-m2-ffi-and-native-stdlib.md) 的原生标准库分层及 [Design41](../41-interface-metadata-and-foreign-abi.md) 的跨界规则。它们和现有 M2 计划中“allocator/显式分配”的措辞在此表示需说明分配资源的职责，**不由该词单独推导出已批准的 Zig 式参数列表或独立语言特性**。

## 8. Zig 与 D 的参考边界

- **Zig**：参考 allocator 实例可传递、固定 buffer/arena 可替代通用堆、失败需处理的设计。Auto 可以把这些能力封装在 Storage 中，不要求照搬每个 API 的显式 allocator 参数。[官方 Memory 文档](https://ziglang.org/documentation/0.17.1/#Memory)
- **D allocator 库**：有 GCAllocator/Mallocator 及静态组合，支持可选择的分配来源；官方明确说明其库机制不会自动接管 new、数组字面量或拼接等内建分配。不能据库层切换宣称全语言分配已切换。[官方 allocator 文档](https://dlang.org/phobos/std_experimental_allocator.html)
- **D @nogc / BetterC**：@nogc 约束 GC 分配，不等于无分配；BetterC 移除 D runtime 依赖、仍以 C runtime 为基础，不等于只用静态存储。Auto 的无堆/无 libc/运行时配置应分别定义。[No-GC](https://dlang.org/spec/function.html#nogc-functions)、[BetterC](https://dlang.org/spec/betterc.html)

以上为比较输入，不把其他语言的保证视为 Auto 已实现能力，也不为 Auto 引入 GC。

## 9. M2 再讨论的事项

本轮只保留方向，不分配新 Plan、不新增本专题的施工任务或最终验收门。进入 M2 标准库的具体设计时，由现有 owner 复核相关前置，并在必要时修订正式合同。

| 待定问题 | 讨论时需要的具体输入 |
|---|---|
| Storage spec 的完整职责 | List/Str/HashMap 等真实操作；固定/增长、析构与失败的分工 |
| 默认注入与容量 | prelude/profile 的解析范围、默认 Fixed 容量来源、显式覆盖和跨包身份 |
| 实例构造与共享 | 两个独立请求 arena、两个容器共享资源、返回拥有值及资源 reset 的生命周期 |
| 底层分配接口 | 是否有跨类型共享字节分配需求；尺寸/对齐/回收、静态分派或 ABI 服务 |
| 无堆配置 | 真实程序的嵌套拥有值、临时分配及后台资源闭包；不同“无分配”目标分别定义 |
| 平台与实现复用 | Windows/Linux/受限环境需要哪些内存来源；保留或替换哪些现有 runtime |
| ABI 与验证 | 既有资源合同适用范围、move/clone/失败/最后消费者、来源一致与恢复边界 |

建议用同一算法分别消费 Heap、Fixed、两个独立 arena 的小型真实切片来讨论这些选择；这是未来讨论的取材建议，不是本次已授权的探针或新门禁。若届时需要改变现有 Plan 的语义/支持承诺，应由对应 owner 正式修订，不能用本文替换其已有验收。

## 10. 本次归档边界

本文记录的方向为：**默认策略便捷，具体 Storage 可控，资源实例明确，底层职责可复用，所有权与失败规则可验证。**

本次仅新增本专题并从内存/数据结构/标准库设计建立导航；代码、current Specs、Plan 状态与编号、实施 worktree 和产品测试均未变更。后续实现完成后，实际能力再按原流程沉淀到模块 Specs。
