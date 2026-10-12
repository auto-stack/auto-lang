# AC 首个语言子集与目标代码

> 2026-10-04，设计建议，尚未实现或冻结。用户已确认 HIR 表达式使用 kind 表示表达式种类、
> type 表示结果类型，允许 type: i32 这样的基本类型 enum 简写。
> 首个平台、独立 ABI 版本域、静态/动态模块和热重载范围承接
> [Native ABI RFC](auto-native-abi-rfc.md)。本文不分配正式 Plan 编号、不修改旧前端。

关联：[HIR 语义](auto-hir-design.md)、[HIR Atom 文本](auto-hir-atom-text.md)、
[Atom Schema](atom-schema-dsl-design.md)、[离线开发策略](v0.6-offline-development-strategy.md)。

## 1. 表达能力、语言支持与编译能力分开验收

Atom 的 node/enum/引用/集合足以表达复杂的 HIR，包括类型、函数、模块、控制流与数据结构；
复杂度没有被 primary/secondary 两个槽限制，完整信息放在命名字段和结构内容中。
但 Atom 可以装下某种结构，不代表该结构的语言语义、verifier 或后端已经实现。

现有三组 HIR 样例只描述 i32/bool、局部绑定、赋值、结构化控制流和直接调用。
结构体、枚举值、集合、所有权、泛型、闭包和动态对象尚需具体 HIR 契约。
不能让 node tag 或任意 attributes 绕过这项工作。

每项支持至少区分：源码可解析、名称/类型已解析、Checked HIR 可验证、native 可生成、
运行时功能可执行。已有 parser 的语法覆盖远大于第一步 AC 的支持范围。
暂不支持的功能须在前端/HIR 能力检查中明确拒绝，不静默转去 AVM 或 A2R。

## 2. 建议采用三个逐步扩大的 profile

这些是范围建议，不是对现有实现的支持声明，也不以代码行数作为验收依据。

| 阶段 | 核心能力 | 足够支撑的代码 | 尚需补充的主要能力 |
|---|---|---|---|
| A：计算核心 | 固定宽度整数、bool、unit；函数/参数/返回、局部绑定与赋值；算术/比较/短路；if/else、while/loop、break/continue、直接/递归调用 | 数值算法、控制流程序、由少量纯函数组成的模块；基本调用顺序与溢出探针 | 字符串、堆集合、聚合值、生产 I/O |
| B：文本与数据工具 | 字符串与字符、List<T>、映射、值语义 record、携带数据的 enum、字段与索引读写、方法、模式匹配、Option/Result、模块导入、有限泛型实例化、文件与诊断服务 | 命令行工具、配置处理、Atom reader/validator、lexer、Pratt parser、部分 HIR pass | 编译器主体所需完整所有权/接口/互操作覆盖及端到端自举 |
| C：AAC 所需子集 | 按实际编译器清单增加嵌套泛型、接口与分派、生命周期/资源清理、Atom/HIR 构造、模块组织、后端和链接器互操作 | Auto 编写的前端、类型处理、HIR verifier/lowering 与编译驱动 | 完整 Auto 的动态/UI/并发及全部生态能力不由自举自动证明 |

B 中的有限泛型优先使用编译期实例化；若只是实现若干预置 List/Map，必须标成
“内建集合支持”，不能称已支持用户泛型。Map 的键/相等/hash、字符串的 UTF-8 与字符索引、
record 复制/移动、enum 布局和错误传播都需明确语义。
借用/资源释放不能等到 C 才第一次设计：B 引入堆数据前先确定一个可实现且安全的受限契约，
C 再扩充到编译器实际需要的范围。不要默认偷用 Rust String/Vec 的布局或引用规则。

A 的首批 HIR 探针仍可沿用 i32/bool 样例。进入源码编译时建议补 i64/u32/u64 与 unit，
浮点可单独增加；每种运算的 signedness、溢出、除零、移位及转换规则必须明确。
参数/局部可保留现有源码推导形式，但进入 Checked HIR 后不能残留 Unknown/Error。
顶层执行语句的入口降级、range-for 糖和默认参数可以逐项加入，不阻碍首个函数型入口。

## 3. 第一批可编译源码应该是什么样

下列是沿用 Auto 写法的目标源码示意，尚未由 AC 编译运行；int 的 native 映射见下一节。

```auto
fn sum_to(n int) int {
    var i = 0
    var total = 0
    while i < n {
        total = total + i
        i = i + 1
    }
    return total
}

fn main() int {
    if sum_to(10) == 45 {
        return 0
    } else {
        return 1
    }
}
```

它已经覆盖函数调用、参数绑定、推导局部、可变写入、循环、比较、分支和返回，
足以证明通路不只会输出一个常量。启动包装器调用 Auto main，并将结果交给进程退出服务；
Auto main 不是 Windows PE 的直接加载入口。启动/退出可以由最小宿主运行时负责。

首批执行验收还应覆盖递归、嵌套调用、短路副作用、命名参数求值顺序、循环嵌套、
超范围字面量与未初始化读取的拒绝。使用可观察的返回/退出值检验，print 不是前提。
之后加入最小诊断服务；生产字符串格式化不是测试专用 trace intrinsic。

## 4. 旧前端接入的已知类型问题

本地可见代码中：parser.rs::parse_ident_or_generic_type 将 int/i32 都返回 Type::Int，
trans/rust.rs::rust_type_name 将 Type::Int 输出为 i64。这说明不能从这个 AST 变体
直接恢复“源码明确要求 32 位整数”的语义。

HIR 的 i32 是精确宽度的内建类型；源语言 int、i32 别名、整数字面量默认类型和算术策略
需按语言现行规范与完整 v0.5 基线核定。本文不把已有 int 全局改为 i32。
如果 AC 要区分源码 int/i32，应让语义前端保留或建立准确的声明类型身份，
并明示历史别名的兼容政策；不能靠输出时改一个类型字符串来解决。

当前几天可先从新的 Atom HIR 输入验证 native，完整基线恢复后再接现有 AST。
“HIR → exe”与“Auto 源码 → exe”分别验收，前者完成不称后者已经完成。

## 5. AC 的第一步目标是原生 AOT

第一目标平台是 Windows 11 x86-64，优先评估 x86_64-pc-windows-msvc profile。
WSL Linux x86-64 为第二平台。建议的第一条通路：

```text
阶段 A0：Atom text → binder → Checked HIR
                                    ↓
阶段 A1：Auto source → AST → resolve/typecheck → Checked HIR
                                    ↓
                           native lowering
                                    ↓
                    backend IR（Cranelift 为候选）
                                    ↓
                          x64 COFF .obj
                                    ↓
                 Auto link driver + platform linker
                                    ↓
                       PE .exe → 原生执行
```

首个 .obj 可以由宿主测试驱动调用并验证；随后做带 main 与最小启动包装器的 .exe。
这不是经 Rust/C 源码转译再编译的 AC 路线。反汇编是调试和审阅产物；无需把汇编文本
作为必须经过的主编译格式，也不要求第一步就有 JIT 或完整优化器。

最小运行时先负责启动/退出和明确的 trap/诊断路径；B 再扩充内存、字符串、集合、文件服务。
没有堆对象的程序不必带入整个 auto-val/AVM/AutoUI。
普通 native 运算和调用不经过 Batom 序列化，也不把每个标量装成 Atom；
Atom 是 HIR 的结构承载与交换形式，native lowering 产生目标布局和机器操作。

静态 .lib、DLL、独立 ABI 版本域和插件热重载仍是 v0.6 的核心目标，见 Native ABI RFC。
“先生成 exe”只是首个开发里程碑，不是把这些要求推迟到 v0.7。
公开 ABI 与模块调用探针应尽早并行于语言子集设计，以免 B/C 建立在错误的跨模块假设上。

## 6. 后端与工具链路线（2026-10-04 用户确认）

完整分期和比较见 [AC/ACC 后端演进](auto-native-backend-evolution.md)。
AC → ACC 改变编译器主体实现语言；自主机器码后端是另一条演进线。
v0.6 的 Rust AC 与 Auto ACC（roadmap 中称 AAC）均使用 Cranelift 主路径，
先完成约定语言子集/主体的原生自举；后端版本和平台能力仍须探针验证。
v0.7 推进自主低层 IR/SSA、优化体系与自主后端实验，不承诺该版完全替换 Cranelift。

第一步 AC 用 Rust 编写，可以直接调用 Cranelift：
ObjectModule 提供 AOT 对象文件发射，其当前官方源码包含 COFF 路径；Windows 展开信息
也有明确支持路径。但这不证明我们的具体函数、布局和 DLL/热重载协议已经可用。

正式选型前用同一组探针检查 Windows 调用约定、对象链接、跨模块参数/返回、
栈与 unwind、静态库/DLL 导入导出，以及源码诊断需求；LLVM 为备选。
不用先手写 x86 指令选择和寄存器分配，也不因使用后端库就否认 AC 的自主前端/HIR/ABI。

链接器复用 lld-link 或 link.exe，Auto 自己管理输入模块、符号/接口/ABI 检查、
SDK/sysroot/运行时需求和制品元数据；不要在第一个闭环中另写一个完整 linker。
本稿没有检查这台机器已安装哪个 SDK/linker，也未生成对象文件。

AAC 自举目标应定义为：Auto 编写的编译器主体由 AC 编译成原生制品，能够重新编译
该主体及代表性输入，进行分阶段一致性验证。仍可调用后端库与系统链接器；
若要求将后端也改用 Auto 实现，那是另一个远大于本次自举的目标。
仅重新编译一个 lexer 不构成 AAC 自举完成；具体主体边界以能力清单与验收合同确定。
ACC 继续通过明确的后端桥接接口调用 Cranelift；薄 Rust C ABI 库为优先评估方案，
初期独立后端进程亦可研究。桥接协议与生产模块 ABI 分开，不暴露 Rust 私有布局。
v0.6 同步建立 pass 前后契约、效果/来源信息、校验与优化前后对拍，做必要的规范化/基础优化；
v0.7 的自主 SSA 可以先接 Cranelift，再增加自己的代码生成。具体 pass 清单按后续计划确定。

## 7. 几天内可推进的独立工作与后续顺序

1. 先为 A profile 补结构 Schema、typed HIR/verifier 的有效与非法样例。
2. 建立独立 backend probe，从新 HIR 输入发射 .obj，宿主调用，再链接 .exe。
3. 在 ABI 线尽早探测公开函数/模块边界，反馈 HIR 的签名/所有权表示。
4. 完整 v0.5 恢复后接 AST adapter；按 A 源码验收，明确兼容性差异。
5. 对 Auto 编译器候选主体建立真实能力清单，再按 B/C 顺序扩大；不能只按关键词表估算。

auto/lib/lexer.at 可见 str/字符操作、Token record、TokenKind enum、List<Token>、
模块导入；parser.at 又使用方法、嵌套 List、状态与递归。它们可作为 B/C 调研输入，
但既有自举文件的覆盖注释不是当前 AC 完整能力清单，更不是直接搬来就能 native 编译的证明。
编译器主体还需类型处理、HIR、Atom、后端接口、文件/进程服务和构建驱动。

正式实现遵循架构设计 → 独立 Plan → worktree 流程，避免覆盖不可见的主机器代码。
[Plan 741](../../plans/741-ac-hir-native-core.md) 已建立，编号经用户指定；
只负责有限 HIR/native 首个闭环，不因本战略加入 pass 框架、自举或自主后端任务。
其他能力分别制定后续计划，执行状态以计划及复审证据为准。

## 8. 资料与核查边界

- [Cranelift ObjectModule](https://docs.rs/cranelift-object/latest/cranelift_object/)：AOT 对象发射。
- [Cranelift object 后端源码](https://docs.rs/cranelift-object/latest/src/cranelift_object/backend.rs.html)：COFF 分支与 Windows unwind 文档。
- [Microsoft x64 调用约定](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170)：目标物理调用规则。
- [LLD Windows 支持](https://lld.llvm.org/windows_support.html)：COFF、DLL 与 Windows 工具链路径。
- [LLVM 对象文件生成教程](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl08.html)：备选后端的原生对象生成通路。

本稿依据本地源码和官方资料制定建议；未运行 AC、HIR parser/verifier 或原生链接/执行。
