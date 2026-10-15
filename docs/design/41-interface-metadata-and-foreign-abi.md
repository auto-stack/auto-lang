---
status: proposed
describes: target-state
created_at: 2026-10-15
updated_at: 2026-10-15
implemented_by_plans: []
---

# Design 41：统一接口元数据、Foreign Interface IR 与跨 ABI 架构

> 需求：AC/ACC 的编译/ABI架构须包含C生态，并为未来经FFI+ABI消费Rust生态预留通用接口；与增量、插件、热重载及全局cache一体设计。
> 本稿整合2026-10-15用户提供的 [A2C讨论](https://chatgpt.com/share/6ad0577b-5c00-83ea-9684-e64956b98cb1)。来源核对/采纳与修正见 [审计记录](../reports/20261015-c-foreign-abi-discussion-audit.md)。技术建议仍为draft；不声明新语法/布局/loader已实现，不分配Plan，不扩大782或旧后端验收。

关联：[Native ABI RFC](strategy/auto-native-abi-rfc.md)、[Rust互操作现状](28-rust-interop-architecture.md)、[C生态战略](strategy/c-ecosystem-strategy.md)、[后端演进](strategy/auto-native-backend-evolution.md)、[Design39](39-ac-incremental-syntax-and-language-platform.md)、[Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[AutoBindgen Spec](../specs/auto-bindgen/project.md)。

## 1. 统一原则：共享接口描述，分别执行 ABI 契约

| 层 | 职责 | 与其它层的界限 |
|---|---|---|
| Interface Metadata | 类型/函数/接口/来源/版本/所有权/效果/能力的共同描述基础 | 不直接决定寄存器/布局，也不把候选声明视作已验证接口 |
| AutoABI | Auto模块公开类型、调用、对象/接口与兼容域 | 独立于编译器版本；同域还需target/接口/runtime匹配；不等于C ABI别名 |
| Foreign ABI Adapter | 目标C/system、Rust shim、未来COM等边界的布局/调用lowering和语义转换 | ABI Adapter是FFI一部分，不能只实现参数搬运就宣称FFI完成 |
| FFI | 导入/导出、绑定生成、外部类型、错误、所有权、回调/线程、链接/装载 | 不强制普通native调用经过Value/JSON/Batom；自动声明不等于自动安全 |
| Module Protocol/Runtime | 发现/预检/握手、dispatch/lease、状态/回调/代际与热重载 | 独立于grammar插件版本和具体C调用约定；ABI相容不代表可立即卸载 |

普通标量/指针/已证明兼容布局尽量采用目标平台惯例，兼容时直接机器调用；需要时生成薄wrapper，复杂对象用明确句柄/接口和转换。复杂Auto对象不因追求独立而强行改成C struct，简单调用也不为统一而先装箱成动态Value。

同一Auto业务函数可有AutoABI入口、C导出wrapper、Rust FFI入口或VM adapter；每个入口的ABI/模式/错误契约明确，业务实现无需复制。可重载跨模块调用仍按dispatch/lease规则，即使物理签名相同也不能绕过生命周期。

## 2. 架构与四份契约

```text
Auto declarations      C target headers / SDK metadata     Rust source+rustc shim
         |                         |                               |
         +------- Interface Metadata / Foreign Interface IR --------+
                                   |
               validate semantic identity / target / ownership
                                   |
             +---------------------+--------------------------+
             |                     |                          |
       Native ABI lowering    A2C/A2R adapter              VM adapter
             |                     |                          |
      AutoABI / C wrapper    C/Rust calls+wrapper       typed stub/dynamic call
             +---------------------+--------------------------+
                                   |
             package / linker / loader / module-runtime lifecycle
```

拆分Auto ABI Specification、Interface Metadata Schema、FFI Contract和Module Protocol四份契约，共享基础类型/来源模型，独立版本化。Batom编码/Schema版本、AutoABI域、应用接口版本、foreign library版本、runtime服务版本各自独立。图不是要求现在另造四套大型实现，也不是把语义HIR与foreign元信息强行合并为同一种IR。

## 3. 统一元数据的最小职责

以下是后续Schema需要覆盖的类别，字段名/编号未冻结；由可信frontend/adapter生成且再校验，不通过任意字符串约定下游猜测。

| 类别 | 必需信息 |
|---|---|
| 身份与来源 | provider生态、包/模块与具体解析版本、source/binary fingerprint、target/SDK/toolchain配置、来源工具/Schema、候选/已确认/可调用状态 |
| 逻辑类型 | 固定宽度标量、目标C别名如long/size_t/char、指针/nullable/const、数组/长度、record/union/enum、函数指针、opaque对象、已具体化泛型实例 |
| 物理ABI | 每个边界的calling convention/variadic、宽度/对齐/offset/packing/enum表示、直接/间接参数与返回、target ABI profile；不能只记abi=c一个词 |
| 函数与接口 | 导入/导出方向、命名空间/符号/版本、签名、接口ID及required capabilities、目标实际提供者；声明不等于动态库一定有同符号 |
| 生命周期 | view/mut/move/clone映射、谁分配/释放及deleter、buffer长度单位/编码、可保存/逃逸/alias限制、callback/context注册注销 |
| 运行语义 | status/errno/错误对象、不跨界unwind的政策、重入与线程亲和/并发约束、同步/异步终止、库/代际pin与释放条件 |
| 生成与依赖 | header/macro/SDK/环境闭包、编译器/shim生成器/flags/实例集合、布局/调用oracle及支持拒绝面、adapter manifest和cache key |

Header通常无法独立证明所有权、回调存活、线程规则和错误语义。允许原始FFI接口带unknown/unsafe状态，必须由领域元数据（GIR等）、明确annotation或审核的安全wrapper补齐，才能宣称安全API。不得为“自动绑定”凭函数名推断free配对或返回值所有权。

Host metadata与MCU/freestanding共用逻辑模型，能力profile不同：纯标量FFI不强制堆、GC或常驻动态loader；需要分配/句柄/重载的host profile显式声明服务。C语言源码生态、C ABI互操作与A2C部署三件事分别验证。

## 4. C导入：provider、header与链接分离

建议统一Import语义节点，区分provider（auto/c/rust/python等）、source resolver、symbol selector/alias和配置身份。provider不等于机器ABI：use.rs最终可调用C ABI shim，但仍是Rust生态导入；use.py也不因此变成C静态接口。

讨论中的候选写法保留为目标态示意，**不是本稿批准的新语言语法**：

```auto
use.c stdio: printf
use.c sqlite3: sqlite3_open_v2
use.c <stdio.h>: printf
use.c "my_library.h": my_function
```

Package resolver从pac.at/标准SDK描述得到headers、library、target、宏/搜索路径等；header resolver保留angle/quote搜索语义、基目录和实际预处理参数，不按文件名猜系统/项目头。header、link library/import library和runtime binary是不同实体，不能从`sqlite3`字符串可靠猜出全部；static/dynamic/显式运行期加载由配置/装载API表达，不把所有use.c变成动态反射。

自动Symbol Binding目标是已有可解析元信息时用户无需重复写`#[c] fn ...`。手工声明仍供无header或特殊shim使用，需显式来源与风险状态，不能自称由真实header验证。普通Auto安全包包装raw C binding；两层长期共存。标准stdio映射是stdio.h（printf不属于stdlib.h），但目标CRT实际类型/可用符号仍由SDK决定。

头文件摄入必须包含实际target、sysroot、宏、include搜索与依赖闭包；优先评估libclang/Clang frontend等有语义依据的工具。通用CST扫描只给候选，不能替代预处理、target layout和C ABI分类。结构体bitfield/packing、long/size_t/char signedness、union、varargs、函数式宏、static inline逐项冻结支持面：常量可求值，非导出定义可生成C shim，不支持则明确拒绝，不对不存在的动态符号盲做dlsym。

printf等varargs还需默认参数提升、format与字符串NUL/编码处理，不可把Auto str/Str直接当char*，也不以一个printf示例代表完整C FFI。声明与header匹配不自动证明库版本/导出和运行契约匹配。

## 5. Rust与其它生态

AC/ACC消费Rust库的默认候选路径：真实Rust类型/签名与实例需求→按目标toolchain/config编译受限shim→显式C兼容边界/句柄→统一Foreign IR。Rust的extern Rust与repr Rust无对外稳定保证，`extern C`也不会让String/Vec/trait对象内部布局变成可跨界承诺。编译器探针可证明特定构建布局，不是永久ABI契约。

泛型/trait调用先在Rust侧具体化或提供明确受管接口；不承诺对任意Rust crate零适配全覆盖。borrow/mut/move、Send/Sync、panic/abort、future/stream、callback/deleter和库保活均要显式转换；catch_unwind不能恢复abort/任意内存故障。必要时使用opaque handle并由原库析构，不能跨allocator误释放。

Rust wrapper/shim的实际源码、rustc/target、crate源内容/依赖图/features/cfg、生成器、实例集合、接口和native依赖进入cache键。共享缓存与同版本号本身不授予布局/生命周期正确性。旧fork-rustc/Versioned Rust ABI研究保留历史，不作为新C/AutoABI主线前置，也不据“同版本”无条件直调Rust私有ABI。

C++仅通过现成C API或有限且可验证的C++→C shim选用；COM/GObject/Objective-C等有独立对象模型与adapter，不能把其全部称为普通C ABI。完整C++模板/STL/异常/Apple SDK并未承诺支持；GIR/Vulkan registry/WinMD作为元信息源候选，实际适用范围逐项计划验证。

## 6. 多后端、版本与插件生命周期

Native按静态签名生成直接调用或wrapper；VM消费同一接口契约但用已验证stub/动态调用适配；A2C输出匹配目标C的include/调用/导出，A2R按其消费面生成Rust包装。支持矩阵分别记录，不能因旧VM/A2R/A2C通过就宣称AC/ACC支持，也不能借新IR自动抹平它们已有的callback等拒绝边界。

A2C编译出的C函数不是自动符合完整AutoABI；如需宣称该域兼容，必须具有相应module/runtime/wrapper并通过同域conformance。Native调用C库无需先把Auto转译为C；保留A2C为MCU/特殊工具链/现有C工程部署和bootstrap路径。

AutoABI同域且target/接口/runtime条件满足时直接互调；不同域默认拒绝，只能选择已实现、版本明确且有类型/所有权/状态转换证据的adapter，不能承诺自动跨大版本。C桥也不能神奇地转换不兼容Auto对象。复用平台C调用惯例不等于跨平台二进制通用。

最小C ABI bootstrap作为发现/握手候选，与现有RFC的文件预检和固定头部兼容；不得先调用未知ABI函数后再检查版本。发现得到的AutoABI函数指针不能作为C指针盲调；面向外语消费者应返回实际C wrapper或受支持接口。OS loader提前执行的入口不是安全沙箱。

外部宿主→C入口→Auto逻辑是独立验收方向。外部宿主保存的callback/trampoline及其context、foreign destructor、后台任务和borrow都可能pin旧代；没有注销/排空证据时拒绝卸载。普通DLL装载成功不证明热重载；失败候选保留旧代，群组切换/状态迁移按Native ABI RFC。

## 7. 编译产物与cache接线

作为Design40 A00/01/06/08/09/10/12的具体消费者，新增领域Schema：Import request、Foreign interface/type/layout、Binding/Adapter manifest、Foreign package/target配置与native dependency description。仍使用统一ArtifactId/LocalRef/ExternalRef与边分类，不建立独立FFI cache目录常驻全部库元信息。

Query分层：输入/工具chain解析→header/SDK semantic提取→接口归一与人工约束合并→布局/调用adapter生成→shim编译→link/load核对。header语法内容可共享；布局/adapter按target配置区分；Rust shim按实际构建环境区分。同header包可供Native/A2C/VM读取，但执行生成产物不是因此通用。

完整key还须覆盖传递header、搜索次序、条件宏、生成头、SDK/CRT、布局选项、实际二进制/库版本绑定；新增头或搜索路径改变可能改变解析结果，不能只哈希一次已选中的header文件。Input manifest/source map/query耗时与失败有界缓存，避免每次use重跑大型SDK分析。

IR/hash来源记录不强pin所有header AST/中间shim；实际装载的库/reader/回调需要的artifact/code是强数据与lease，不能让普通GC删掉deleter仍依赖的映像。stale adapter/旧toolchain产物按Design40软保留与预算回收，独立安装SDK/用户交付库不属于可随意删除cache。

## 8. 建议的先行验证（未编号）

| Case | 成果/入口 | 关键正反oracle |
|---|---|---|
| F-01 | 统一metadata与Auto/C target layout/调用探针 | scalar/float/record/指针/回调双向真实native；异质参数防交换，target/abi/packing错位拒绝 |
| F-02 | header→Foreign IR→Native与A2C消费 | 原始selector+package resolver等价；引号/尖括号同名头、宏/target变化不误cache；未知所有权仍raw；宏/inline不冒充外部符号 |
| F-03 | 外部C宿主加载Auto插件 | 小型标准宿主候选SQLite extension；真实入口/回调/错误/原方释放；宿主不认识AutoABI仍可消费C wrapper |
| F-04 | Rust crate→C兼容shim→Auto调用 | 配置/私有布局改变失效、泛型实例明确、String/Vec不穿私有布局、panic边界与原库deleter保活；错误签名不能盲调 |
| F-05 | Auto模块与版本/发现 | Auto同域互调、不兼容拒绝；C bootstrap只发现受声明入口；跨域adapter必须有显式能力和转换控制 |
| F-06 | reload+GC+多工程cache | 外来callback/handle/deleter pin旧代；失败不切换；释放后可回收；复用metadata但不复用错误target的shim |

首个平台仍为既定Windows x64；Linux/MCU样例分别按profile加入，不把原C战略Linux旗舰先行变成本轮AC前置。SQLite extension等仅为候选验证项目，不批准所有列举库/新backend；既有PLAN-595/597/610资产可作参考但需新域实际证据。

## 9. 外部一手依据与未决项

- [Rust ABI](https://doc.rust-lang.org/reference/items/external-blocks.html)：Rust ABI无稳定保证，C ABI匹配具体target主流C编译器；同为C不代表跨target一致。
- [Rust type layout](https://doc.rust-lang.org/reference/type-layout.html)：repr C与repr Rust的保证不同，外层表示不能自动约束内部字段的表示。
- [C include syntax](https://gcc.gnu.org/onlinedocs/cpp/Include-Syntax.html)：angle/quote对应不同搜索规则，不能仅凭名字推断。
- [Libclang](https://clang.llvm.org/docs/LibClang.html)：可提供C接口语义/位置等摄入基础，工具可用不等于本仓已集成或完整C安全信息已获取。

待冻结：接口Schema/identity、raw/safe标注与诊断、import/provider/pac语法、clang/SDK来源及支持子集、varargs/bitfield/inline边界、Rust shim生成与句柄规则、C bootstrap、AutoABI跨域adapter、host/freestanding服务裁剪、双向callback/线程/重载和cache预算。现行Spec维持已实现事实；设计与讨论中的v1/v2、函数名等都是例子，不是发布新ABI版本。
