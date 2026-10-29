---
status: accepted-direction
describes: target-state-and-gap-matrix
created_at: 2026-10-20
updated_at: 2026-10-28
implemented_by_plans: []
planned_by: [PLAN-798, PLAN-828, PLAN-817, PLAN-818, PLAN-820, PLAN-821, PLAN-822, PLAN-823, PLAN-824, PLAN-825, PLAN-826, PLAN-827, PLAN-862, PLAN-863, PLAN-864, PLAN-865, PLAN-866, PLAN-868, PLAN-870, PLAN-871]
---

# AV语言能力矩阵：源码、内核与替换差距

整体定位与后续规划读取步骤见[ACE总入口 §0](auto-native-backend-evolution.md)（原称AC/AV/AJ总入口）；本表负责AV能力缺口与迁移证据，不代替整体路线或current Spec。

## 1. 结论与证据口径

2026-10-24版本裁定：下表缺口中属于现有语言/AutoVM/A2X全部示例及对应消费者的能力，进入v0.6 Rust版ACE的M2–M4成熟度路线，不是因“本波未承诺”就整体留给远期或AACE。原820–827等Plan仍各按既有支持面验收，后续按缺口另立Plan；Auto实现的AACE明确延后至v0.7、Rust版成熟度门之后，见[版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md)。

798、828、817/818及820–827全部实际交付可认领**ACE首版核心骨架基本成型**，仍是有限profile的组合，不等于完整Auto语言或旧AVM替换。828提供函数增量/安装基础，827提供真实scalar源码验收；817/818仍主要限i32/bool，820–824的typed fixture不自动获得源码/CTE/native消费者，825/826提高执行/编码效率而不增加语言覆盖率。

本矩阵衔接[Design43](../43-ac-av-typed-mir-and-abc.md)、[Design44](../44-av-data-layout-place-and-ownership.md)，实施顺序与替换门见[分阶段路线](av-source-programs-and-avm-replacement-roadmap.md)。这是2026-10-20目标与缺口快照，不回写canonical Spec或把未来Plan当现状。本轮只写文档，未运行任何程序。

**2026-10-21排期/事实更新**：798已归档，主检出auto-av/auto-mir Spec已有有限交付；完整函数增量828仍在实施。D828之后优先817/818、827及820→821→822→823→824功能线，825→826按实测择机插入；独立consumer可并行，公共核心/注册单写交接，见[共同合同](../../plans/reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)。本矩阵仍是逐层目标/缺口表，原各Plan任务、支持面和预算不因排期而扩张；实际状态/版本与交付按当前Plan/Spec和收据重核，不把下表目标变成PASS。

状态不是百分比：“限”=合同承诺指定源码profile的有限闭环；“T”=拟经typed MIR/ABC fixture验证内核，**没有源码闭环**；“—”=本波未承诺；“无完整标准库”不等于宿主测试shim可当产品API。以下“完成后”均是**条件预期**，当前实现通过须逐层拿证据，不能把表里的字改成PASS代替运行。

能力完成需要分别记录：
1. source语法/绑定/推导与合法性；
2. 资源/effect完整的Checked HIR/MIR；
3. ABC codec/准入及新进程重验；
4. AV实际执行与独立结果/副作用/cleanup；
5. stdlib/provider实际装配和权限；
6. SDK/CLI/会话/安装消费路径。

第4层的fixture不能代替第1层，第5层声明/签名不能代替executed，第6层“能启动”不能代替真正经AV运行。函数unit工件kind与语言Unit值/无返回函数是两个不同维度。

**2026-10-28当前续接**：798/828及820–827有限合同基础已进入后继阶段，868已交付共享lowering。下面T表示typed层能力，现状须查current Spec，不再表示822–824仍未实施；它仍不等于source支持。862（r3，executing，入口前端部分）/863（输出）/864（集合与拥有型List）/865（闭包源码）/866（在线AV）和870/871（native扩面/覆盖退役）见[续接基线](ace-implementation-handoff.md)。新增计划认领不把任何源码缺口改为PASS。

## 2. 本波交付后的逐能力矩阵

旧AVM需求证据来自[VM现状](../../specs/auto-lang/vm/overview.md)、[前端模块解析](../../specs/auto-lang/frontend/design/module-resolution.md)、[stdlib装配](../../specs/stdlib/design/assembly-manifest.md)、[源码域](../../specs/auto-hir/stage-contract.md)以及[23个现存程序目录](../../plans/reports/av-source-program-catalog-2026-10-20.md)。**有文件/expected不等于本轮已证明旧AVM可运行**；G0/827负责从实际入口、feature/平台和强oracle建立当前分母。

| ID | 能力 | 本波后源码层 | 本波后MIR/ABC/AV | stdlib/产品层 | 现有合同owner | 仍缺什么 | 后续阶段/工作包 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| C-01 | i32/bool常量、局部存取、参数/返回 | 限 | 限 | — | 798 | 入口/结果需区分OS退出、guest返回及显示值 | G1 / 827 | P0 |
| C-02 | i32算术/比较/除余/溢出 | 限：checked profile | 限 | — | 798 | 旧wrapping与新trap按SEM-01分profile决策，不能无条件等价 | G0/G2 | P0 |
| C-03 | i64/u64/f32/f64精确数据位型 | 无新增源码承诺 | T：const/copy/call/return | — | 820 | 字面量/推导/转换/源码返回仍需接通；NaN移动保真≠算术 | G2 / R2-N | P0 |
| C-04 | uint/byte及其他现存数值类型 | — | — | — | 无 | 逐类型从旧清单确认，不由u64支持推导全数值族 | G0/G2 / R2-N | P1 |
| C-05 | 宽整数/浮点算术、位运算、Math | — | 仅i32子集 | — | 820只数据；825/826不扩算术 | AP-20和边界值需要新的typed操作、源推导和stdlib装配 | G2/G3 / R2-N | P0 |
| C-06 | Unit值、无返回main、隐式尾表达式/顶层脚本 | ✅ 862有限entry-unit源域 | AV-only结果/入口/恢复受检；非任意Unit值位 | — | 862已归档；旧798不自动扩面 | 非所有后端入口/Unit参数局部与方法；NA-4沿已批准边界 | G2 / R2-E | P0 |
| C-07 | let/var、遮蔽、标量赋值 | 限 | 限 | — | 798 | 完整源码语境与动态脚本模式仍需盘点 | G1/G2 | P0 |
| C-08 | if/loop/break/continue/短路/递归 | 限 | 限 | — | 798 | 128构造预算/显式帧/fuel仍按profile，不等任意源码规模 | G1；G2增量扩面 | P0 |
| C-09 | for-in、range、迭代器/yield | 部分 864（仅 `for x in <List>`：len 前读一次、Copy 每轮物化/owned 投影上下文、迭代期结构修改编译拒；range/数组/迭代器/yield 不在本批） | List访问不等通用迭代 | — | 822仅集合；generator另案 | AP-08/AP-13需迭代降级及挂起状态/清理 | G2/G5 / R2-L,R5-A | P0/P1 |
| C-10 | 直接函数、命名实参、求值顺序 | 限 | 限 | — | 798 | 默认/变参/高阶引用不由此自动支持 | G1/G2 | P0 |
| C-11 | 默认/变参、函数值和高阶调用 | — | 823仅已声明闭包签名 | — | 823部分基础 | 完整调用语法、绑定、签名/返回和方法HOF接口 | G2 / R2-E,R2-L | P0 |
| C-12 | 名义record、嵌套字段原位读写 | 限：现有record域 | 限：Copy/owned分阶段 | — | 820/821 | AP-02/AP-03仅域内；匿名对象/类/继承不是同一能力 | G2 / R2-D | P0 |
| C-13 | char、UTF-8文本、Str/str | 限：现有text-record域 | 限：774拥有/借用规则 | — | 821 | 常规旧源码str字段与新Str/str资源合同需SEM-03 | G0/G2 / R2-D | P0 |
| C-14 | tuple/解构/默认构造/空record | — | 仅目标设计，无明确本批全支持 | — | 820未承诺完整tuple | 按实际旧用例逐项建合同，源码空record保持现有拒绝 | G2 / R2-D | P1 |
| C-15 | List<T>布局、Index/增删/清理 | ✅ 864（auto.source.collections.draft：new/push/insert/get/set/remove/len/clear 源码面；L01–L07 真源绿；List<Str>/List<拥有型type>及L04/L05原源 owned view/remove 转移已交付） | T：822具名typed合同 | — | 822执行；864源码 | native集合仍待D871后新包；HOF库/嵌套List/record容器字段不由此支持 | G2 / R2-L | P0 |
| C-16 | Option/Result/有限enum载荷 | ✅ 864（?T/Result<T,E>/enum 声明+构造+推断；V01–V07 真源绿；完整error propagation另验） | T：822具名typed合同 | — | 822执行；864源码 | AV原源已交付；native变体和完整error propagation另验 | G2 / R2-L | P0 |
| C-17 | is/match、guard与载荷解构 | ✅ 864（is 语句穷尽分发+载荷绑定，guard/投影降为受检分支链；穷尽/重复/未知臂具名诊断；V01–V07 真源绿；guard 表达式与通配超集仍后续切片） | T：受检分支投影 | — | 822执行；864源码 | 非完整guard表达式/通配/复杂解构；native投影随后继变体包验证 | G2 / R2-L | P0 |
| C-18 | owned闭包、捕获、间接调用 | —（865已认领） | T：823受限环境/调用 | — | 823执行；865源码 | lambda/捕获/逃逸源码待865；map/filter/reduce另属库与方法能力包 | G2 / R2-L | P0 |
| C-19 | 共享可变捕获、循环对象图、跨挂起借用 | — | 明确拒绝或尚无owner | — | 821–824首批排除 | 若旧消费者依赖则不能永久排除；须专门共享/生命周期设计 | G0/G5 / R5-O | P1 |
| C-20 | 匿名对象、动态字段、Map/Set | — | Dyn不是Map/对象实现 | — | 824不交付Map/Set | AP-19/JSON对象读写、键相等/迭代顺序/缺字段语义 | G2/G3 / R3-C | P0 |
| C-21 | 模块全局变量与初始化顺序 | — | 未交付global Place | — | Design44仅模型 | 多模块初始化/一次性/环、状态寿命与隔离 | G3 / R3-M | P0 |
| C-22 | use/pub/re-export、pac/包/模块解析 | — | 798有限函数闭包非完整模块系统 | — | 796仅语义身份 | AP-07依赖闭包/名称冲突/缓存/stdlib选层，复用resolver与783 | G3 / R3-M | P0 |
| C-23 | 用户泛型、spec/trait、有界派发 | — | 内建List实例化不等用户泛型 | — | 796有事实；822非完整泛型 | AP-17真实推导/实例化、约束/单态化和失败诊断 | G3 / R3-T | P0 |
| C-24 | 方法/ext/接收者/动态分派 | ✅ 881旧record/text-record static/view方法窄面 | 共享whole/unit lowering→AV/mn、模式/owner/方法body增量及恢复已交付 | — | 771/782语义；881 producer已归档，871核整体覆盖 | 非mut实例/ext/trait/动态派发/任意重载，非collection profile方法；非方法文本/资源gap独立保留 | G2/G3 / R3-T | P0 |
| C-25 | 字符串API、格式化、数字转换 | 限：774/782列明操作 | 限：该profile操作 | 无完整标准库 | 821非所有Str方法 | f-string/Unicode/转换/输出，AP-04/14/17共同前置 | G2/G3 / R2-E | P0 |
| C-26 | ?、Err传播、try/catch、panic与资源清理 | — | trap与Result数据已分态 | — | 821/822仅基础 | AP-06/AP-18；显式错误通道/cleanup，不把trap自动当Err | G0/G2/G6 / R2-X,R6-F | P0 |
| C-27 | Dyn/Any源码、动态类型检查 | — | T：递归白名单 | — | 824 | 显式Dyn不是Unknown兜底，更不是完整动态脚本/反射 | G2/G6 / R2-X,R6-F | P1 |
| C-28 | 完整源码所有权/借用与返回来源 | 限：774/782支持面 | 限：根move/loan/guard | — | 821–824 | 部分move/共享/挂起另案，不能以enum载体宣称完整borrow checker | G2/G5 | P0 |
| C-29 | print/stdin/stderr、普通CLI结果 | 862入口已交付；863输出待实施 | 测试trace不是print | — | 862已归档/863 r2 drafting | typed host-import/console与精确流断言由863；stdin/完整CLI及其他宿主另扩，不因print完成认全宿主 | G2/G4 / R2-E,R4-P | P0 |
| C-30 | FS/argv/env/time/process/权限 | — | 未交付host服务 | — | 测试shim不等标准库 | AP-10/11实际副作用、sandbox、取消/子进程清理与grants | G3/G4 / R3-H,R4-P | P0 |
| C-31 | JSON/regex/hash/编码及常用库 | — | 824 portable≠Json API | — | 无本批实现承诺 | AP-09/19/20，738 actual manifest + typed provider executed证据 | G3/G4 / R3-C,R3-H | P0 |
| C-32 | Future/await、generator挂起恢复 | — | 明确非目标 | — | 823同步环境不含async | AP-13/14：帧/环境跨挂起、取消与资源返回 | G5 / R5-A | P1 |
| C-33 | Task/Msg/actor/channel/调度 | — | 明确非目标 | — | 无 | AP-12：状态隔离、消息有序/背压、stop/join和取消 | G5 / R5-A | P1 |
| C-34 | HTTP/TCP/SSE/service请求生命周期 | — | 无实际接线 | — | 797 ABI不等HTTP host | AP-15及真实服务项目，受控socket/流/异步终值/停止 | G5 / R5-S | P1 |
| C-35 | AutoUI/store/handler/computed/回调 | — | 无实际接线 | — | 814拆包不实现新AV语义 | AP-16：新AV驱动真实交互、Provider/渲染契约和退出 | G6 / R6-U | P1 |
| C-36 | C/Rust/Python调用及反向回调 | — | 无通用adapter | — | 797仅有限原生ABI；824拒外部资源 | AP-17/18/21及保留消费者，typed外对象/线程/GIL/资源归属 | G6 / R6-F | P1/P2 |
| C-37 | REPL/持久会话、debug/source map/ABT | — | 798有有限source映射 | — | 不等现有交互产品 | 符号/变量读写/断点/重启/会话隔离与旧bytecode工具迁移 | G6 / R6-I | P1 |
| C-38 | 嵌入API、AutoGen模板、Man动态求值 | — | 无产品owner接线 | — | SDK/工具拆包非AV迁移 | 逐调用点审计并真实运行，保留错误/退出/资源语义 | G4/G6 / R4-P,R6-I | P1 |
| C-39 | CTE/meta反射/类型生成 | —（817另有窄consumer） | 817仅i32/bool执行结果 | — | 817/796已有限交付；824不扩其schema | 复合CTE/泛型实例化/生成声明/runtime反射分别专项，不等AACE才做 | v0.6语言/语义扩面；AACE另在v0.7 | P1/P2 |
| C-40 | native/JIT/OSR/deopt | — | 818仅有限标量native；AJ未交付 | — | 818；825/826不是JIT | 新数据native扩面另立；JIT/OSR不是AVM替换硬前置 | 可选性能分支 / RP | P2/P3 |
| C-41 | 模块/数据持久化、跨进程与增量 | 限：实验profile | 限：783重验/lease/版本 | — | 798/820–824；825/826优化 | 整项目imports/stdlib/provider/配置与结果依赖闭包、真实编辑 | G1→G8逐阶段 | P0 |
| C-42 | SDK/CLI/安装包、默认选择与可回滚迁移 | — | 内核通过不等产品 | — | 814–816不自动迁VM | 实际入口/平台/feature分SKU、先opt-in再默认、无静默回退 | G4/G7/G8 | P0/P1 |
| C-43 | AACE及AAC/AAV主体/真自举 | — | 本波不足以推定 | — | 743/Design43；2026-10-24分版 | compiler-source与可编译语言分清；先Rust版全示例成熟度，旧AAVM展示非证据 | v0.7 / RB；不提前实施 | 后续版目标 |

R2-*等为路线工作包标识，尚非Plan编号/实施合同。不能借本矩阵向798/820–826追加完整语言、标准库、UI或FFI验收。后续必须以真实程序和stage gate拆适中Plan，缺失owner要显式显示，不能埋成“顺便支持”。

## 3. 替换前必须裁定的语义差异

| 决策ID | 已发现的证据/冲突 | 必须留下的决策与验收 |
|---|---|---|
| SEM-01 | 旧engine整数ADD/SUB/MUL有wrapping实现；[Draft算术规范](../../conformance/01-arithmetic.md)也记wrapping；798明确checked i32溢出trap | 分source/execution profile固定数值语义；保留798原profile，若需legacy兼容用具名新profile/转换与边界用例；不得把两端差异全部说成bug或静默改为同一语义 |
| SEM-02 | 旧VM null-family明确保留List负索引；822负下标先拒 | 正负下标/越界/空容器的保留语义、profile或显式源码迁移策略；负索引不能当共同输入直接对拍 |
| SEM-03 | AP-08的Note.title为str；774/821拥有字段仅Str、借str字段拒绝 | 源类型含义/安全拥有化/借用与兼容profile如何保持可观察行为；未裁定不改821 strict域。不能把改写成fixture的程序冒充原源码通过 |
| SEM-04 | 823初版move/显式clone的owned闭包；旧调用面含隐式捕获/可变共享和alias | Copy捕获、拥有捕获、共享修改和逃逸按用例决定，不能无条件深clone改变共享效果 |
| SEM-05 | 798入口零参/i32、显式return；旧源码有顶层脚本/Unit main/尾值和显示格式 | entry选择、脚本执行顺序、Unit/guest结果/OS exit/print分别冻结；不硬编码按文件名挑结果 |
| SEM-06 | 新profile trap/Result/resource/infra分态；旧.as有try/catch/Err传播及外对象错误 | 错误ABI、cleanup、捕获范围及进程退出分别对照；resource/infra永不充语言负例 |
| SEM-07 | 普通对象、record复制/alias、None/null/缺字段与Dyn并不一一对应 | 对象身份/字段更新/clone/空态/JSON类型边界用强oracle裁定，不能按相同位型兼容 |
| SEM-08 | 源规范和现状Spec有历史差异，AP-18甚至是已知保守误报的回归见证 | 对每项记录“保留语义/旧bug修正/显式破坏变更/未知”，附权威来源和确认记录；未知阻塞所属迁移门，不能改expected补绿 |

本轮未批准这些语言语义变更。827 T-01/02先取证并制作具体决策材料，后续Plan在依赖实施前冻结；材料超出既有授权的语义/兼容改变须另取得用户决定。非冲突公共子集可先经G1运行，不等待所有语义争议才能验证gcd。Draft conformance不是覆盖当前已批准profile的更高权威；也不能只因为某次旧运行不同就改新合同。

## 4. 可审计覆盖率与缺口登记

未来登记单位为(capability-ID, source-profile, execution-profile, program-ID, entry/SKU, platform/features, toolchain/input/stdlib/provider hashes)。一个证据同时触达多能力可关联，但不能冒领未走路径；一项能力的全部必需子例通过才可标该支持项达标。

分别报告：源码闭环能力数/固定能力分母、typed-fixture-only数、完整程序成功数/固定程序分母、入口迁移数/固定入口分母、库实际执行符号数/保留调用符号分母。不得把这些分母混成“支持语言95%”；当前本轮**没有新的运行通过项**。43行是本轮首版能力分类，最终替换分母由G0的全量调用/语料/stdlib扫描形成，不能用23个种子程序代替全产品清单。

旧测试绿也不能自证AV正确。每个程序使用独立手审/规范expected；只有旧AVM、a2r/native都适用且环境齐全时才加差分。非确定并发只对合同允许的偏序/集合归一化，不对任意输出排序；OS资源错误与语言负例分开；没有oracle的用例明确gap，不读worker的expected生成actual。

已有实现的迁移缺口与新语言愿景分开：对旧AVM从未交付的JIT、OSR、MicroVM等不制造退役硬前置；对仍保留的现有.as/FFI/UI/REPL消费者，优先级较低不等于允许永久删除。哪些旧展示/废弃接口可不保留，须有显式范围决定与来源，不能由新AV不支持推导为“已退役”。

## 5. 更新责任

[827](../../plans/archive/827-av-source-program-baseline.md)先建立程序/能力/入口清单、实际source runner与强oracle，G0/G1完成后将实证沉淀到拟议auto-av/language-support.md，后继Plan逐项更新。主检出Spec不在本次改写；Design43/44保持语义/数据合同来源，本表只管理交付缺口与迁移证据。

对真实程序的原则：保留源码和依赖/资源字节，允许受控sandbox/输入/受支持入口参数；切源目录、删imports、换成等价MIR、把复杂分支剪掉都只能形成新变体ID，不能把原AP标为通过。
