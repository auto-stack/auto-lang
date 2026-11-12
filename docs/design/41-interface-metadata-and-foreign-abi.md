---
status: proposed
describes: target-state
created_at: 2026-10-15
updated_at: 2026-11-05
implemented_by_plans: []
---

# Design 41：统一接口元数据、Foreign Interface IR 与跨 ABI 架构

> **2026-11-05 CA-E接续**：[941最终组合验收](../plans/941-ace-m1-expanded-and-architecture-acceptance.md)已正式规划，设计见[Design51 §16](51-shared-a2x-and-hir-to-rust.md#16-ca-earch最终组合验收2026-11-05target-state)。937已交付不代938/939；总门分别验证typed/source方向、布局/调用/资源/独立认证与恢复依赖，缺项不签M1。下方CA-D初订及“CA-E未立项”为历史，不改797 candidate/descriptor3。

> **2026-11-05 当前正式规划**：CA-D已拆[937共享事实/标量底座](../plans/937-shared-foreign-interface-and-scalar-abi-bridge.md)→[938受检源码foreign调用](../plans/938-checked-source-foreign-scalar-calls.md)→[939资源/句柄/同步回调](../plans/939-foreign-resource-handle-and-callback-bridge.md)，均drafting r1，仅规划；当前合同见§12与[基线/排期](../plans/reports/ace-ca-d-baseline-and-schedule-2026-11-05.md)。937可独立于929/934主体交付，公共H/正式采样分窗；后两包接稳定source/query事实。下方旧“CA-D未取号/921在途”等为当时快照，797 candidate/descriptor3/原限额与M1/M2硬门不变。

> **2026-11-02 M2全场景替换约束**：[M2合同r2](../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)要求AutoUI/AutoMan及全部现有应用所用host/FFI在M2真实接通，并完成产品切换/旧实现disable；不能归类未来M3/M4而免验。优先明确typed import/有限已验shim/预编译库，不为未使用的新通用生态扩容；未来M3/M4暂不规划。M1有限797/C桥保持，不扩大在途ABI合同或假称桥接已实现。

> **2026-11-02范围增补**：用户已将共享HIR→C第二目标加入M1，见[独立批准件](../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)、[Design51 §13](51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)及本文§10。新增有限C/AutoABI桥为M1必需；完整通用FFI仍是独立能力线。本轮只设计/规划，797候选支持面/旧批准哈希/913–917合同与current实现均未改变。

> **2026-10-28续接基线**：797已交付Windows x64的autoabi.win-x64.v0.1-candidate及Auto/C有限双向互调，现状见[auto-abi](../specs/auto-abi/project.md)。这不是全类型/跨平台ABI定稿，也不含完整C importer、Rust shim/Python生态、动态loader和回调热卸载；必要FFI可与语言扩面交叉推进。 统一进度快照与未完工作见[ACE续接基线](strategy/ace-implementation-handoff.md)，后续按current Spec/Plan及merge收据重核；本注不扩大原Plan验收。

> 需求：AC/ACC 的编译/ABI架构须包含C生态，并为未来经FFI+ABI消费Rust生态预留通用接口；与增量、插件、热重载及全局cache一体设计。
> 本稿整合2026-10-15用户提供的 [A2C讨论](https://chatgpt.com/share/6ad0577b-5c00-83ea-9684-e64956b98cb1)。来源核对/采纳与修正见 [审计记录](../reports/20261015-c-foreign-abi-discussion-audit.md)。技术建议仍为draft；不声明新语法/布局/loader已实现，不分配Plan，不扩大782或旧后端验收。

关联：[Native ABI RFC](strategy/auto-native-abi-rfc.md)、[Rust互操作现状](28-rust-interop-architecture.md)、[C生态战略](strategy/c-ecosystem-strategy.md)、[后端演进](strategy/auto-native-backend-evolution.md)、[Design39](39-ac-incremental-syntax-and-language-platform.md)、[Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[AutoBindgen Spec](../specs/auto-bindgen/project.md)。

## 1. 统一原则：共享接口描述，分别执行 ABI 契约

2026-10-16时序：ABI合同/C双向互调探针尽早实施，不等完整AC；公开模块/互操作承诺前冻结有限AutoABI profile，随后扩大C导入/Rust C-shim。三个合同域各自版本化/验证，首批范围及门槛见Native ABI RFC§1.5。新ABI/FFI用例从开始纳入AutoDown，元数据/shim/toolchain/target输入纳入全局cache；这里仍是设计要求，不是发布声明。

| 层 | 职责 | 与其它层的界限 |
|---|---|---|
| Interface Metadata | 类型/函数/接口/来源/版本/所有权/效果/能力的共同描述基础 | 不直接决定寄存器/布局，也不把候选声明视作已验证接口 |
| AutoABI | Auto模块公开类型、调用、对象/接口与兼容域 | 独立于编译器版本；同域还需target/接口/runtime匹配；不等于C ABI别名 |
| Foreign ABI Adapter | 目标C/system、Rust shim、未来COM等边界的布局/调用lowering和语义转换 | ABI Adapter是FFI一部分，不能只实现参数搬运就宣称FFI完成 |
| FFI | 导入/导出、绑定生成、外部类型、错误、所有权、回调/线程、链接/装载 | 不强制普通native调用经过Value/JSON/Batom；自动声明不等于自动安全 |
| Module Protocol/Runtime | 发现/预检/握手、dispatch/lease、状态/回调/代际与热重载 | 独立于grammar插件版本和具体C调用约定；ABI相容不代表可立即卸载 |

普通标量/指针/已证明兼容布局尽量采用目标平台惯例，兼容时直接机器调用；需要时生成薄wrapper，复杂对象用明确句柄/接口和转换。复杂Auto对象不因追求独立而强行改成C struct，简单调用也不为统一而先装箱成动态Value。

同一Auto业务函数可有AutoABI入口、C导出wrapper、Rust FFI入口或VM adapter；每个入口的ABI/模式/错误契约明确，业务实现无需复制。可重载跨模块调用仍按dispatch/lease规则，即使物理签名相同也不能绕过生命周期。

2026-10-16分层执行补充见[Design43](43-ac-av-typed-mir-and-abc.md)：解释/快速JIT/优化JIT和发布原生版本共享逻辑接口，物理帧/调用可经adapter适配；内部调用合同、AutoABI及平台C ABI分别存在。推测/内联版本须记录被依赖body/layout和模块代际，不能仅因公开签名没变就继续调用旧实现。外部callback/deleter及在途代码仍按lease保活；新版本安装先复核快照，模块组切换保持已批准的一致性事务。

这不扩大797的Windows x64原生互调面；完整解释↔JIT互调、代码替换、OSR/deopt和loader另立Plan。发布闭包必须包含承诺的FFI导入、runtime服务和反射保留，机器码cache匹配不替代ABI/权限/部署依赖验证。

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

内部语义事实与推导/CT反射的owner及持久模型见[Design42](42-semantic-metadata-inference-and-comptime.md)。本章Interface Metadata/Foreign IR是其受控消费/外语导入投影，不能取代内部语义服务；target layout与逻辑类型身份分开。runtime保留/调用adapter须和模块代际、ABI/权限绑定，不直接保存TypeInfoStore宿主函数指针或把同名类型视作同版本布局。

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

## 10. A2C第二目标的有限ABI桥（2026-11-02，target-state）

用户批准A2C加入M1后，首期C路线中的CA-D负责有限桥接，不能再仅作为可选后继；详见[C路线§6–§9](strategy/shared-a2x-and-hir-to-c-roadmap.md#6-有限autoabi桥与通用ffi边界)。它与CA-A/B/C的语义/资源代码生成、CA-E的Q4与M1总验收分别关闭，不把A2C源转译成功等同ABI/FFI通过。

### 10.1 三层独立合同

| 层 | 本次必须成立 | 不得推断 |
|---|---|---|
| A2X/C语义生成 | 受检HIR/资源事实→C内部表示/typed helper→真认证，独立oracle | 普通C函数天然满足AutoABI；C内部record布局等于公开对象布局 |
| 有限C/AutoABI桥 | 独立wrapper/descriptor/header、target/layout/调用/ownership/runtime准入，实际双向三腿 | 历史声明IR probe已证明任意Auto源码FFI；repr(C)或同版本工具足够证明互调 |
| 通用FFI平台 | 仍按Interface Metadata/Foreign IR/adapter/provider/模块运行时分层推进 | 加C目标便自动拥有任意header导入、Rust/Python生态或安全热卸载 |

M1桥沿autoabi.win-x64.v0.1-candidate与descriptor/3：pure scalar和fallible context/status/out、UTF-8/数组buffer view、record const/mut/out、opaque handle原方destroy、同步per-call callback+ctx。沿797原bool/char合法性/长度单位/错误provider/代际和失败out逐字节不变；不扩大f32公开ABI、按值聚合、varargs/bitfield/union、跨线程/保存callback、unwind或loader能力。A2C源域宽度和797声明类型域不同：typed i64/u64/f64互调证据不冒C前端完整宽源码运算。

### 10.2 实际调用和来源

CA-D必须分别证明Auto源→生成C实现→公开wrapper→独立C caller、受检有限foreign声明→生成目标call→独立C callee，以及同域AutoABI peer互调。仅wrapper外壳调用C而Auto业务从未经过受检foreign边界，不算Auto→C闭环；typed import/手工声明与Auto use.c源语法支持分别列证。声明/头/schema只为候选来源，准入必须校验实际符号/物理形态/工具target/依赖和权限。

公共Interface Metadata/Foreign IR只补本桥真实需要的受控投影：逻辑签名/名义身份、param mode/ownership/borrow/effect、ABI physical lowering、provider与真实deps。不能复制一套与HIR推导并列的类型系统或独立FFI缓存DB；sourcemap/元数据/库依赖接统一产物与lease。受限源profile尚无外部语法时，可从受检声明桥接，但不得报成任意源FFI支持。

独立C工具实际编译header/provider/caller，用sizeof/alignof/offsetof和异质非零实参/ctx/多out/handle观察；相同生成器双方自洽不算独立oracle。错域/target/packing/signature/runtime/权限在调用前拒，native调用0，已知不兼容不能盲调用制造UB。C/Rust生成文件编译成功也不替调用准入；资源由原方释放、callback/deleter按真实最后owner保活。

### 10.3 当前基线与交接

797归档9/9的历史merge1799f86ca不是当前主线祖先；当前canonical Spec及abi代码经过896迁移，集成以现行内容/SHA与真实三腿复验为准，不能复制历史PASS。旧profile维持candidate，本增补不发布完整AutoABI版本。C与Rust provider同读语义事实、各自生成适配，普通native调用C仍无需先转C源码。

CA-D私有wrapper/参考可先设计；a2x-core/store/worker接缝取CA-A正式接口、资源方式取CA-B稳定提交，ABI descriptor/ctool/worker、ac_protocol/测试/SDK注册全文件单写并核所有consumer。913主来源恢复/lease，914场景/布局，915资源，916/917旧A2R；只在实际共用符号交接，不新增整波依赖。既有797原限额/负依赖/key/退役语义不放宽。M2所有现有场景用到的foreign callback/线程/GIL/shim等必须接管；未使用的新通用importer/任意库/热卸载扩展暂不排未来计划，不能借旧M3分类免除当前产品所需。

## 11. A2C、统一ABI与FFI的近期结构（2026-11-03，target-state）

本轮只规划：921 CA-A实际在途，CA-B按Design51 §14正式立约；797仍是Windows x64 candidate/descriptor3，未发布新ABI版本，也未实现任意header importer/Rust shim/动态卸载。922实际盘点说明产品入口/宿主接线仍缺，不能把A2C能编译当全M2或通用FFI完成。本节将既有四份合同落实到可交接边界，不现在新增四套框架/数据库或复制类型系统。

### 11.1 三种表示和五个责任层

| 责任层 / owner | 权威输入→输出 | AC/AV/A2X的消费与限制 |
|---|---|---|
| 内部语义/资源（core verifier + semantics-contract） | Checked HIR/函数凭证、796名义声明/版本、模式/借用/能力/CT依赖→只读接口事实 | 所有后端同源，不由ABI或C helper重新推导类型/ownership |
| Interface Metadata / Foreign IR（中立投影） | Auto受检事实或foreign候选声明+来源/人工约束→逻辑接口描述；候选与已验证事实分开 | 不含宿主函数指针，不假定源已支持use.c；不把target布局回写Auto逻辑类型 |
| ABI physical plan（各域adapter） | 逻辑签名+domain/profile/target/layout/错误/ownership→物理lane、wrapper/调用转换及布局校验 | native内部ABI、AutoABI公开边界、平台C ABI三域分开；物理相同仍不自动兼容 |
| FFI binding/provider（工具与边界消费） | 已验证interface+符号/库/工具/权限/runtime依赖→受控绑定/adapter | AC直接机器调用、AV typed host stub、A2R/A2C发薄目标wrapper；不用Value/JSON统一普通标量调用 |
| module/runtime lifecycle（host owner） | 实际image/provider/版本、handle/callback/deleter/在途pin→可用调用与最后释放 | ABI相容不等于可卸载；静态链接/同步per-call与未来动态/跨线程分别授信 |

三种表示分别是Auto内部值、A2C私有生成表示、公开AutoABI/C wire对象。CA-B C struct/Str runtime只属于第二种；CA-D显式转换/布局证明后才跨公开边界。不能为省wrapper直接把内部record/字符串或Rust String/Vec私有布局暴露，也不能要求native调用C先转译成C源码。

### 11.2 最小共享接口事实与调用准入

逻辑事实至少有DeclRef/owner/名义content版本、source/profile/凭证来源、函数参数/结果/receiver mode、borrow-return来源约束/alias排他要求、error/effect/capabilities；foreign侧补provider/package与解析版本、真实符号、raw/safe状态及人工ownership/长度/编码/callback约束。ABI投影另绑domain/version/target/calling convention、record size/align/offset/packing、参数/多out物理形态、bool/Unicode char合法域、buffer长度单位、runtime服务版本与实际库身份。并非所有字段来自一个已存在Schema，具体有限投影须CA-D feasibility冻结，不能以缺字段默认安全。

数据流：候选metadata→名义/shape/owner/版本/来源验证→ABI/target/工具/provider/权限/runtime匹配→process-local可调用binding→真实call。持久数据即使hash正确仍需重验，不序列化SafeCallable/Checked/PASS/函数地址。错域/target/packing/签名/env/provider/代际/权限在调用前拒，被调计数0；合法非零ctx/handle及混合异质参数不能被全拒。已知不兼容不实际盲调制造UB，负控验证真实拒绝点。

所有权模型从共同事实投影：view借到声明的调用/owner期限，mut排他且writeback规则明确；owned handle由原provider destroy，跨allocator free禁用；fallible out只在success commit，失败保原out字节；borrow return不能指向已死临时。同步callback显式ctx、不可保存；调用中callback与deleter的实际最后owner保活其provider，退出路径顺序可核。跨线程/保存callback/GIL/动态重载只在已有应用确需时由M2-HOST对应包明确实现，不由本有限桥默认放行。

### 11.3 首个有限落地包仍是CA-D

CA-D须带真实消费者再固化共享接口：797 native ABI validate/emit与A2C wrapper/import至少两个实际消费，不先交一套无人用的统一trait。可将abi中纯模型投影复用现semantics-contract或中立窄模块，具体crate边界由有界探针确定；A2X不得为了借descriptor直接依赖Cranelift/AV/旧AST栈，物理ABI实现留各adapter。796/797现有canonical格式/候选身份可兼容则保，确需变更显式版本和所有读者迁移，不能偷改descriptor3。

有限端到端验收分三腿：Auto source经新A2C生成实现→公开wrapper→独立C caller；受检finite foreign声明→真实目标调用→独立C callee；同域AutoABI peer互调/797当前原三腿回归。只有typed import则明确typed，不冒任意use.c语法；只外wrapper调用C且Auto语义链未消费foreign事实不算Auto→C。真实header/sizeof/alignof/offsetof、异质参数/ctx/多out、句柄provider/原方destroy与callback为独立oracle，不能相同生成器双方自洽充认证。

CA-D所需A/B接口包括：TargetDescriptor/provider/完整fileset身份与source anchor；CheckedFunction与资源/排他/borrow事实出口；C owned/view表示的受控转换及清理调用点；decl→symbol/source map、target配置/工具/头/库依赖closure。CA-B先把这些边界写清，不实现foreign语法/public wrapper，不把自己的internal helper当公开ABI。CA-D可先私有header/reference/797投影研究，资源桥集成等CA-B稳定版本；无需等CA-C整个集合包。当前不分CA-D号，接口和独立oracle齐后正式立项，完整桥仍为M1必需，不能延期到M2之后。

### 11.4 增量、cache与资源约束

逻辑metadata按名义/签名/mode/实际body读取/CT/meta/负依赖失效；物理plan再含target/ABI/layout/runtime/effect/权限，binding再含实际header/宏/搜索候选集/SDK/CRT/库/工具flags与provider代际。callee只改不被读取的body不重编普通调用者；布局/头搜索解析变化/新增更高优先级头不能因调用点源码相同盲命中。接口提取/adapter生成/工具编译的真实P逐项计，未改函数/绑定生产IO0与resident身份；restore未授信元数据有界重验R，原CALL/ABI失配不得被cache PASS绕过。失败后保旧adapter/artifact集合且修复后下一restore复验；活callback/deleter/handle引用的产物有lease，最后owner才可GC，独立安装SDK不是任意缓存清理对象。

沿统一ManagedStore/Design40 namespaces，不复制FFI DB/runner。CA-D保797原build60s/run5s/total70s、IPC4MiB/每流1MiB、热点8MiB/暂存索引8MiB、单产物64MiB/请求128MiB及原更严边界；新域预算必须先正式冻结，不能拿CA-B更宽认证门覆盖797。日常emit/metadata/证书核查不隐式编译，显式认证工具单列；保P90≤2000ms和原正确性/结构/资源条件。

### 11.5 与M2产品替换的顺序

922已明确M2-EXEC/HOST/SESSION/TEST、UI/Man/Web迁移和TS provider等尚需正式包。M2-HOST可复用有限typed import与已验证手工shim接现有Rust库，原源码/库/ABI变化才低频构建；不等通用header importer/Rust自动shim平台全部完成才迁已有应用，也不能因其属host而免验。通用FFI扩展按真实缺口分包，无消费者的新生态不抢M2关键路径；CA-D/Q4不替全部产品/所有A2X目标和旧实现disable。

## 12. CA-D正式拆包与可独立交付边界（2026-11-05，target-state）

本轮用户授权正式立项、仅规划。当前797的descriptor/layout/validate/emit已有真实三腿，但emit的业务体仍是有限BodyKind声明IR；native私有ABI的bool为I32、Str为ptr+I32长度，891宽typed ABI又用整数位载体，均不能直接别名为公开AutoABI。A2C的a2x_result/record/Str也为私有表示。CA-D正式拆[937底座](../plans/937-shared-foreign-interface-and-scalar-abi-bridge.md)→[938源码调用](../plans/938-checked-source-foreign-scalar-calls.md)→[939资源桥](../plans/939-foreign-resource-handle-and-callback-bridge.md)三个中等包，以实际生产者/消费者关闭，不交无人消费的统一trait，不把typed/probe支持冒普通源FFI。

| 包 | 可审查交付 | 真正前置与独立性 |
|---|---|---|
| CA-D1 共享接口事实与标量桥底座 | 无后端依赖的逻辑接口/物理计划及797兼容投影；797真实native ABI发射/准入与新A2C标量导出wrapper两实际消费者；Auto源码经既有C provider生成业务体后供独立C caller调用；current797三腿重验 | 797、921已交付，可在不改core/source、共享lowering、a2x-core/store/session及929恢复协议的边界独立实施。native消费者在此仍是797声明IR，不冒mn源码公共导出；不含源foreign call |
| CA-D2 受检有限foreign标量调用与源码接线 | 有来源及人工契约的typed foreign declaration注入源环境，普通源码调用经受检事实→共享MIR/native直接机器调用，或HIR/A2C薄调用；禁止在外层wrapper偷调用C冒源码闭环；连真实source恢复/首编辑 | D1；公共S窗口取929稳定交付/释放接口，若934已接管query事实则消费其权威端口。无模板/CT读依赖的本域不强等935/936整包，但其新事实已存在时不能绕开 |
| CA-D3 资源、句柄与同步回调桥 | 在D2真实调用面关闭UTF-8/bytes/i64元素view、record const/mut/out、多out原子提交、opaque原方destroy及同步per-call callback+ctx；native imports与A2C imports/exports真实消费，共同资源事实、lease/失败清理/恢复 | D1+D2，925资源/924闭包底座已交付；私有header/oracle可并行，资源事实/HIR/两后端窗口按实际SHA释放 |

D1完成不是完整CA-D/M1完成；D2也不核销D3。D3不得用797旧probe资源PASS代新源码桥。所有既有797公开形态都由D1..3的显式支持矩阵覆盖：typed宽标量与i64数组view可经受检typed adapter入口，不冒当前只覆盖i32/bool的普通Auto源运算；任意use.c/header importer、f32公开ABI、按值聚合、跨线程/保存callback、unwind/动态loader不扩。

中立模型拟置于ACE内backend-free窄crate（具体路径由D1有界可行性冻结），仅依赖必要semantics-contract/编码；abi的Cranelift发射/工具调度留原处。提取descriptor/3兼容类型可re-export，但原canonical字节、strict reader与未知版本拒绝须回归；确需改wire先needs_replan并迁全部读者，不能默改。A2C不依赖ace-abi/Cranelift，provider继续无process/store副作用；binding、认证和lease在host组合根。

D2从显式typed declaration manifest注入受限source环境，不在统一CST外再造parser，不把ExternalFn兄弟签名或console HostCall复用成任意foreign。逻辑contract必须先验证来源/名义类型、模式、effects/error、capability与物理plan，地址/Checked/SafeCallable/PASS不持久化；unknown ownership只能raw候选，不能进安全source调用。AV/A2R foreign支持在新矩阵单列：本轮必须关闭native+A2C的有限方向及四端普通业务oracle，不以这两腿推断四端FFI等价；若既定M1登记另有AV/A2R required foreign行，该行继续阻塞，不能临时转NA。

各包首例即文件化TestCase+独立C头/调用者/被调用者及手算oracle，实际调用有callee计数；错误domain/target/packing/签名/权限/provider在调用前拒且计数0，不以UB作负控。元数据提取/adapter/目标函数/tool生产分别计P，非读者P及其payload I/O均0；R只深验既有页，不隐藏求解或编译。D1typed/checked输入恢复与D2/3 source恢复分别验收，不相互冒认；后两包含restart后无预热首编辑、首错误→修复→再恢复。

全部包沿Design40 ManagedStore/lease和797原更严限额、六场景×30及P90≤2000ms，source域继承自身原样本/预算；daily emit/admit不隐式spawn，显式ABI认证需实际compile/link/run。资源包有界收敛及真实最后owner pin，不宣称进程重启/重跑为HMR。929、934–936的source/query/实例/CT页各有owner，公共H与正式采样单写/单实例，写面顺序不新增无关语义依赖。完整CA-D交付后最终CA-E/ARCH组合签章仍需另立，不预占号；M2当前所有应用host接管/旧实现disable依然是v0.6硬门。
