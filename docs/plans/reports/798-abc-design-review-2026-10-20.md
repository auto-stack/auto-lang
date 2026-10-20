# PLAN 798 新 ABC 架构评审与设计建议

日期：2026-10-20。评审对象是AC/AAC系列的AV，而非旧AVM。本文回答类型安全、执行效率、字节码体积，以及Place/所有权扩展是否合理；参照WebAssembly及V8的对应层次。

**结论：保留类型化槽位、显式CFG、独立验证和profile隔离。798适合作为正确性探针，但不宜将当前“固定宽度编码＋解码回MIR对象＋MIR解释循环”直接冻结成长期高效ABC。** 优先复核unit/module信息丢失的问题；扩面前确定类型等价、Place和资源状态合同；再以测量决定紧凑wire和装载后执行表示。

范围是设计建议，不是798最终验收，不修改其状态、任务、验收或既有设计。本轮不修改实现、测试、Plan合同或canonical Spec，不创建实施树。代码观察基面为`plan-798-dev`提交`f24bd46a6f84286c30c90b156d9b4e7d72ad1358`，八份核心文件的内容身份见[输入收据](../attachments/av-abc-review-20261020/baseline.json.txt)。以下字节数按编码逐字段计算，复杂度按源码推导；没有运行性能基准或反例探针，涉及运行表现者分别标注待复核/待测。

## 1. 主要建议及优先级

“优先”表示建议处理顺序，不自动追加为798的阻塞验收。只有确认违反其已批准合同的正确性问题才应进入798 needs_fix；新增生产能力须另立或修订后续Plan。

| ID | 优先级与性质 | 观察或风险 | 设计建议 | 建议落点 |
|---|---|---|---|---|
| ABC-A01 | 优先复核，正确性 | unit/module由函数数量推断，wire不保留is_unit | 明确工件kind及entry准入，证明encode/decode保持语义；先补单函数自调用反例 | 798 owner先核验；若证实则属现有正确性修复 |
| ABC-A02 | 扩面前，类型合同 | Const/Bin按标量shape，Copy/Call按表索引判断等价 | 统一primitive/结构签名/名义类型的等价与规范化规则 | 820设计冻结 |
| ABC-A03 | 架构优先，执行表示 | LoadedModule保留CheckedMir，执行遍历Vec/enum对象图 | MIR、portable ABC、私有LoadedProgram分层；装载时生成已验证紧凑执行计划 | 820前接缝设计，优化另行实施 |
| ABC-A04 | 近期，确定的复杂度问题 | 每条指令计算block_base，重复扫描此前块 | 预计算block起点或持有单一PC，诊断定位移出热循环 | 小范围效率修复，需单独授权 |
| ABC-A05 | 近期，调用开销 | 每次Call创建参数Vec、帧槽Vec，并遍历所有槽初始化 | 帧arena、预计算参数映射、受控调用窗口，保留初始化与所有权证明 | 820/821或有界效率计划 |
| ABC-A06 | 近期测量，wire密度 | BIN固定14B，槽/常量/块引用均u32；所有整数常量查池 | 先比较ULEB/窄宽操作数，再考察直接typed opcode、短常量与调用参数窗口 | 新wire版本探针，不能暗改v1 |
| ABC-A07 | 格式扩面前，模块组织 | 单一顺序wire、无函数body目录，名称/slot kind/inline sidecar增加体积 | 分清必需语义、可选诊断、运行元数据；函数长度/目录支持按需装载 | 后续ABC格式设计 |
| ABC-A08 | 所有权实施前，语义完整性 | 现有Copy＋初始化gen-only分析不足以表达move/borrow/drop | Place＋访问模式＋资源状态/清理边；必需生命周期信息不可随debug剥离 | 820/821，822/823顺延 |
| ABC-A09 | 扩面前，资源和权限 | CFG位图规模依赖块数×槽数；能力声明不等于宿主授权 | 约束验证工作量，路径状态稀疏化；requires与host grants分离 | 821以后及宿主接入前 |
| ABC-A10 | 扩面前，可维护性 | 编解码、typing、use/def、trap、fuel分类分别维护 | 单一指令合同描述及覆盖审计；未来kill/borrow/cleanup也纳入 | 820/821合同 |
| ABC-A11 | 性能决策前，证据 | 798亚秒指标主要是端到端周转，不是解释器吞吐 | 分层比较Wasm格式、Ignition解释、Wasm基线/优化机器码；统一语义和计量 | 独立对标决策包 |

## 2. 当前 ABC 的准确定位

实际路径为：

```text
Checked HIR → Typed MIR → 手工版本化ABC编码
                           ↓
                     解码成MirModule
                           ↓
                 MIR verifier → CheckedMir
                           ↓
          LoadedModule { checked: CheckedMir, executable }
                           ↓
            AV遍历MirBlock / MirInstr / Vec<Value>
```

这不是Rust enum的原始内存转储：wire有显式字段、字节序和版本。但在逻辑形状及装载后内存上，它仍非常接近MIR。其优点是少维护一套语义，便于验证首个闭环；代价是源代码元信息、分析结构和执行结构较紧耦合。

证据：`core/src/abc.rs:215`的`encode_abc`，`abc_verify.rs:40`的私有CheckedMir字段，`av.rs:263`以后直接match MirInstr。最小profile的六类顺序指令和三个terminator，不能据此认为已经建立完整生产ISA。

建议维持一套执行语义，而允许三种表示：

```text
Typed MIR       编译器共享语义、Place、资源义务、变换和诊断
Portable ABC    版本化持久/传输、类型/函数引用、紧凑编码、装载前无信任
LoadedProgram   私有执行计划、预解码操作、块/参数/布局表、受控句柄
```

LoadedProgram不是第二个语义IR，也不必建立通用优化框架。可以先是简单的平坦操作数组和几张紧凑表；必须由验证/装载服务构造，保持不可变，并检查预算。后续若压缩wire，解释循环可继续执行预解码表示，不必每次执行都解析LEB。

## 3. 类型安全评审

### 3.1 应保留的基础

- decoder拒绝未知版本/profile、保留位、opcode/子码、非法bool、截断与配额超限；不把坏输入作为成功模块。
- `verify_abc`重验公开DecodedAbc的版本/profile并调用真实verifier；LoadedModule内部字段私有。缓存中的标记不能自行创建Checked。
- verifier检查全部块的结构和类型，不仅检查入口可达块；确定初始化分析对可达CFG取前驱状态交集，考虑零次循环。
- bool与i32在AV中是不同语义类型；入口实参也检查实际Value种类。未知宿主import不进入任意调用。
- 显式帧循环避免guest递归消耗Rust递归栈；受控trap、resource、cancel与infra分开。

上述机制适合继续扩面。它们目前证明的是有限标量profile的准入边界，不是未来聚合内存、句柄、借用和宿主FFI的完整安全性。

### 3.2 ABC-A01 unit 与 module 的语义不能由函数数推断

目前有三条可以直接对照的源码规则：

```text
MirModule：显式含 is_unit: bool
decode_abc：is_unit = (n_funcs == 1)                 abc.rs:683
verify：unit中的Call(Func(...))被拒绝              mir_verify.rs:617–623
```

encoder没有编码is_unit；其header flags仅表达source sidecar。单函数unit与单函数完整module因而不能无歧义区分。

静态可构造反例：一个完整module只有零参/i32的main，分支为false；未走分支包含对main自身的直接Call，另一分支返回42。编码前`is_unit=false`时直接调用和entry均符合现有规则；编码再解码后变成unit，该Call即使未执行也会在全块类型检查时被`mir.unit-callee`拒绝。`assemble_closure`会将`fn:name`重定位为Func，driver又确实对完整闭包执行encode/decode，因此不仅是无关的内部结构差异。

**这是从源码得到的语义往返反例，尚未运行探针。** 应先按实际public路径复核：单函数自调用、单函数非零参数unit、单函数完整entry、多函数module四类，以及不可达调用。不能用给module强添dummy函数绕过。

建议以显式wire kind或可信envelope的“预期kind＋内容一致性验证”区分unit/module。对象自称kind仍不授信；executable closure必须完成链接、entry、imports和类型一致性检查。修复需绑定编码/解码及真实driver，不仅补一个struct字段。

### 3.3 ABC-A02 统一类型等价和名义身份

`Const`允许重复primitive表项，按shape检查；`Copy`比较`dst_ty != src_ty`，Call实参和结果也按索引比较；`EqI32`又能处理两个bool。它们目前不会自动造成类型混淆，但相同primitive在wire表重复登记时，不同指令的接受规则不一致；未来record/alias/generic加入后会更难判断是否应结构等价。

建议在820明确：

1. primitive按语义kind规范化；是否接受重复项及重映射策略唯一。
2. 函数签名按已解析类型及参数mode比较；不因本地TypeIdx数字相同就跨模块等价。
3. record/enum保留名义owner、声明/实例与revision；相同布局不等于同类型。
4. `EqBool`等可在装载时专门化，不继续让命名为I32的操作隐式容纳越来越多类型。

新增反例应同时包含“等价primitive不同索引”及“相同宽度但不同名义record”，不能用一种修复误放开另一种。

### 3.4 类型验证、权限与运行期检查分别负责

分配失败、下标、对象存活代际、store归属、容器epoch、实际动态分支和host句柄都需要受控检查；不能因静态类型验证通过就删掉这些检查。相反，静态已证明的i32/Bool分支可以在私有LoadedProgram中消除重复类型分派，不必让每次算术都重新猜Value种类。

当前capability是有限测试trace白名单，尚无任意host服务。扩host前应明确模块`requires`与装载方`grants`的交集，不能将字节码自带能力字符串视为授权。FNV trailer只检查意外损坏，不是真实性或类型安全凭据；外层内容身份和重新验证仍需保留。

## 4. 执行效率评审

### 4.1 ABC-A04 诊断索引在热路径产生额外线性扫描

`av.rs:270`对每条普通指令计算：

```text
flat = func.block_base(block_idx) + ip
```

而`mir.rs:217`的block_base遍历此前全部块并累加指令数。执行第b个块中的一条指令，额外工作为O(b)；固定后部热循环会反复付这笔费用，即使没有trap、不需要查询span。对于B个单指令块依次执行，前缀遍历项合计B(B−1)/2，而guest操作数只有O(B)。这不是声称某个固定倍数的实际减速，而是源码层可确认的复杂度。

verifier已经预计算block bases，执行器应采用同样原则：装载时一次建立`block_start[]`，或改成平坦PC；span lookup只在错误/调试需要时发生。保留source mapping，不以删除诊断获得速度。建议用相同guest步数、不同块序号/块数的热循环测斜率。

### 4.2 ABC-A05 每次调用有可避免的分配和全槽扫描

`av.rs:378`收集`arg_vals: Vec<Value>`，随后为callee再建一个`Vec<Value>`，遍历其全部槽；参数之外的槽通过typed_zero初始化。大量小函数或递归会让allocator、复制和清零占据明显比重。现有槽预算先于分配检查值得保留。

建议先测量：空/小callee调用、宽参数调用、很多临时槽但少活值、递归。候选实现是连续帧arena、frame base/size、装载时预计算参数落位、复用已释放窗口；参数可先写入保留的新帧再切换，不必每次创建独立参数Vec。

这不意味着绕过初始化证明，也不要求立即使用MaybeUninit/裸指针。首个owned实现可以用安全Uninit状态；退出、arena扩容和借用指向必须维持frame实例身份。未来含资源的返回不能先pop帧再盲目复制Value。

### 4.3 ABC-A03 先区分文件编码与执行表示

固定u32编码便于decode，但当前执行器根本不直接解释这些wire字节；它执行解码后的MIR enum。因此“把wire改成变长整数会让每条指令更难解码”不是当前热循环的直接成本，只影响装载阶段。

建议优先建立小型LoadedProgram：扁平操作/块起点、预解析callee和参数映射、类型化数值操作、来源side table。保留一个语义实现和对应验证映射，不复制复杂优化器。算术仍须保留溢出/除零等语言检查；去掉重复类型match不等于去掉语义检查。

`intern_type`、`intern_const`使用Vec线性查重（mir.rs:309/317），也值得在大量常量/实例化后测量；首版可保持小表简单性，超过实测阈值再加有预算的索引。不要在优化执行器时忽略lowering、验证和链接阶段。

## 5. 字节码体积评审

### 5.1 当前编码的逐字段成本

下表来自`abc.rs:331–404`，不含类型/常量表、函数/块头和source sidecar：

| 指令 | 当前字节数 | 成因 |
|---|---:|---|
| CONST dst,const | 9 | 1字节opcode＋两个u32 |
| COPY dst,src | 9 | 同上 |
| BIN sub,dst,lhs,rhs | 14 | opcode/sub各1B＋三个u32 |
| UN sub,dst,operand | 10 | opcode/sub各1B＋两个u32 |
| CALL n个参数 | 11＋4n | opcode/kind/callee/dst/argc＋参数槽列表 |
| BRANCH cond,then,else | 13 | opcode＋三个u32 |
| JUMP / RETURN | 5 | opcode＋一个u32 |
| 槽描述 | 5/槽 | kind u8＋type u32 |
| 来源记录 | 16/条 | instr/func/start/end各u32 |

独立小module还有34B基本头（无capabilities）和8B checksum。一个只含`main()->i32`返回42、1个槽、1个常量、1个块、无sidecar的module，按当前布局为102B：42头尾＋10类型＋5常量＋27函数/槽元数据＋4块头＋9 CONST＋5 RETURN。这是格式算术，非运行encoder的实测收据。

因此当前格式“紧凑”的含义主要是显式二进制而非文本；尚未达到Wasm或Ignition那样对常用小操作数进行专门压缩的程度。不能由上表推断整个应用相对Wasm大多少倍。

### 5.2 ABC-A06 建议先比较两种窄操作数方案

| 候选 | 优点 | 必须验证的代价 |
|---|---|---|
| ULEB/SLEB等变长索引/整数 | 小索引通常1B；适合portable文件 | 截断/溢出/最大长度、最短编码政策、decode速度 |
| 默认8位＋Wide/ExtraWide或按函数宽度 | 顺序decode简单，常见槽号紧凑 | 前缀成本、混合字段宽度、重定位/边界检查 |

保留BIN＋sub时，三个索引均<128的二元指令可从14B变为5B；若进一步采用专用`AddI32` opcode，可为4B。COPY可从9B变为3B，RETURN可从5B变为2B。这些只比较该指令，尚未计宽指令、表及元数据，也不是整体收益承诺。

Wasm使用有位宽上限的LEB整数编码，规范允许部分非最短编码；ABC若为了CAS身份选择仅接受最短编码，应作为自己的明确规则，而不是错误声称Wasm本身一律canonical。[Wasm值编码](https://webassembly.github.io/spec/core/binary/values.html)

V8 Ignition采用累加器减少显式操作数，并有Wide/ExtraWide、短寄存器和特化调用编码。其当前`Add`还携带动态反馈信息，不应只拿一个opcode的长度与ABC做不等价对比。[Ignition设计说明](https://v8.dev/blog/ignition-interpreter)、[V8指令定义](https://github.com/v8/v8/blob/main/src/interpreter/bytecodes.h)、[操作数宽度](https://github.com/v8/v8/blob/main/src/interpreter/bytecode-operands.h)

建议先在三操作数模型上试窄编码，不立即把AV改成累加器机；后者会影响寄存器分配、指令数及MIR映射，需要真实分布证明收益。

### 5.3 常量、临时槽和参数列表也占体积

- 当前i32常量池条目5B，每次CONST还要9B。反复使用的大常量适合池；0/1、布尔、少用的小整数可比较短立即数编码，不能一概都去池。
- 每个临时槽有5B元数据，运行时还有一个Value。已有lowering以表达式temp为主，后续可比较同类型、互不重叠生命周期的物理槽复用；含borrow/owned后必须以已证明的liveness及清理边界复用，不能简单按最后一次“读值”判断死亡。
- CALL为每个参数写u32 slot。可比较连续参数窗口、常见小arity形式；保留非连续形式兜底。物理重排不能重排源码求值，额外move成本也要计入。
- 签名参数数目用u32但Call argc为u8且decode限制255。这是现有有限profile边界，不应默认为永久语言上限；未来版本应统一表述和宽度策略。

### 5.4 ABC-A07 语义元数据与诊断元数据分开

当前driver已经对持久单元使用spanless code并另存sidecar（driver.rs:475/552等），这是正确方向；不能将“尚需完善section布局”说成“完全没做分离”。最终assembled closure仍可能带inline source map，且执行热路径仍计算flat index。

若每条指令都有来源记录，16B/条sidecar可能比14B的BIN本身更大。建议发布/调试分别计量：代码、必需类型/布局/生命周期表、符号名称、来源、cache envelope；来源可delta编码、按需加载，但不能随意删除trap定位。函数名在imports/exports或调试需要时保留，不能把所有name都当可剥离。

后续wire可用显式section kind/version/length及函数body长度/目录，支持按函数索引/验证/装载。Wasm分开function声明与code body，利于流式和并行编译；AV已有函数单元cache，也应保留而非再强制整模块物化。[Wasm模块编码](https://webassembly.github.io/spec/core/binary/modules.html)

未知可选debug section可按明确政策跳过；未知必需类型/资源section和未知opcode仍须拒绝，不能为了“向前兼容”把未知指令当NOP。

## 6. Place 与所有权扩展评审

### 6.1 现有槽位 CFG 是合理起点

AV不必为了对标Wasm就改成结构化栈机。类型化槽位更直接映射共享MIR、复合Place、调用参数和native后端，也便于诊断原变量。问题在于当前opcode集合只有值操作，尚无位置/权限/转移责任。

在[Design44](../../design/44-av-data-layout-place-and-ownership.md)基础上建议冻结如下逻辑模型，再决定具体wire编码：

```text
ValueOperand：Copy(place) | Move(place) | Constant
Place：base + typed projections + access/owner context
Projection：Field / Index / guarded Variant / supported Deref
Call signature：参数类型＋view/mut/move mode＋返回责任＋effect/capability
Cleanup：正常边及失败边上的已初始化资源义务
```

这些逻辑概念不要求每一个都变成一条运行opcode；静态可消除的权限操作可仅保留验证信息，真正需要的初始化、drop、句柄和边界检查仍由执行计划承担。

### 6.2 ABC-A08 不能把 Copy 改名 Move 就完成所有权

当前`Value`可Copy，Copy直接复制源槽，目标覆盖也直接赋值；Call收集参数副本，Return取值后pop整帧。它们对i32/bool合适，扩owned资源后每处都必须确定：源是否失效、目标旧值是否清理、临时是否拥有、返回责任何时转出。

verifier的`defs/in/out`模型当前只做初始化gen：`new_out = new_in ∪ defs`。Move需要kill，借用需要权限/存活状态，部分构造需要精确回滚；给`use_slots/def_slot`加几个分支不足以完成这些证明。

建议821首批继续保守：含资源record整根move/clone，字段可replace，部分move拒绝；reservation防止LHS稳定后RHS破坏同根；borrow不能逃逸或跨挂起；失败replace保留旧值；所有受控退出均承担清理。不要为了少写verifier提前引入动态万能Value或通用共享引用。

### 6.3 ABC-A09 路径和状态数量必须有边界

当前初始化分析用三组块×槽bitset，内存守卫已在分配前计算，这是优点。其主要量级为`3 × B × ceil(S/64) × 8`，另有CFG和Vec开销；自由CFG还可能需要多轮传播。

扩到字段路径后不能机械将S替换成所有类型字段的笛卡尔积。建议只为实际使用的projection建立MovePath/trie，按根保守处理重叠，稀疏/稠密表示按实测选择；编解码、验证迭代、构造、drop及freeze都计工作量和峰值。

Wasm指令验证算法能以单遍处理核心指令流，与其结构化控制流和类型规则有关；不能据此认为AV任意CFG的所有权分析也应天然单遍，或Wasm所有GC类型检查都恒为线性。[Wasm验证算法](https://webassembly.github.io/spec/core/appendix/algorithm.html)

clone大容器、路径遍历等单条操作可能有随数据量增长的成本。未来若指令融合/拆分或装载后专门化，必须明确fuel按哪一层计费；保留语义work量、取消点和失败清理，不以减少dispatch数绕过CTE预算。

### 6.4 ABC-A10 将指令语义分类集中管理

目前opcode编号、encode/decode、verifier typing、def/use、can_trap、runtime match分别维护。随着Place/资源指令加入，容易出现“执行支持但验证漏状态”“会失败却无source/drop边”等跨表遗漏。

建议形成一份可审计的InstructionSpec：操作数种类、输入/输出类型、copy/move/borrow/kill、effect、may_trap/resource、清理/来源要求、fuel成本、profile准入。可以先用普通表及完整性测试，不必先造复杂代码生成DSL。MIR与ABC允许不同物理编码，但不能各维护相反语义。

## 7. 与 WebAssembly 和 V8 的正确对标方式

### 7.1 三个不同层次

| 对标对象 | 值得AV借鉴 | 不能直接照搬或混比 |
|---|---|---|
| Wasm规范及二进制 | 显式类型、严格decode/validation、紧凑索引、分section/函数、可移植性 | Wasm是格式/语义标准，不是单个解释器；基础Wasm没有Auto的move/borrow合同 |
| V8 Ignition JS解释器 | 累加器/窄操作数、特化指令、虚拟寄存器布局、解释/JIT衔接 | JS动态语义/反馈/GC成本不同；AV静态数据不必复制所有反馈和对象机制 |
| V8的Wasm编译与优化执行 | 快速基线编译、分层执行、代码/元数据管理、热代码优化 | native执行结果不能当成ABC解释dispatch的同层比较 |

Ignition的“寄存器”是虚拟寄存器/解释帧槽，并非CPU寄存器；这一点与AV槽位模型相近。其高效还来自专门生成的handler和整体runtime工程，不只是opcode长度。[Ignition说明](https://v8.dev/docs/ignition)

Sparkplug从字节码快速生成代码并保持解释器兼容帧，提示AV现在应定义逻辑帧映射、类型/owner元数据和generation接缝；不要求798立即实现JIT/OSR，也不要求AV和native共用相同物理布局。[Sparkplug](https://v8.dev/blog/sparkplug)

V8旧Wasm pipeline文档中某些“无deopt/OSR”的描述有历史边界：2025年官方已说明Wasm间接调用推测内联和deoptimization。此处只采纳快速基线＋优化层的原则，不把旧页所有限制当当前事实。[V8 Wasm优化更新](https://v8.dev/blog/wasm-speculative-optimizations)

### 7.2 数值语义必须对齐

AV当前i32加减乘溢出trap，MIN/-1及MIN%-1也按已批准合同trap；Wasm i32加法按模2^32，signed remainder的MIN/-1结果为0。JS Number又是另一套数值语义。因此不能用不同检查量的程序宣称某个编码/引擎更快。[Wasm数值语义](https://webassembly.github.io/spec/core/exec/numerics.html)

吞吐对比可先选已证明不溢出、无外部IO的i32程序；错误路径另测。若Wasm需要显式实现AV checked arithmetic、fuel或资源检查，应计入其成本并解释；不能关掉AV安全合同后称为可交付性能。

### 7.3 ABC-A11 建议的测量矩阵

| 负载 | 区分的问题 | 主要指标 |
|---|---|---|
| 同guest步数、热块位置从前到后变化 | block_base与CFG元数据开销 | ns/guest操作、CPU、块数斜率 |
| 有界整数循环、不同运算链 | dispatch/typed Value match | 吞吐、动态指令数、检查成本 |
| 小函数高频调用、宽参数、递归 | 参数复制、帧分配/初始化 | allocations/call、bytes/frame、calls/s |
| 小整数/重复常量/多临时槽 | wire压缩和寄存器分配 | 各section原始/压缩字节、槽数 |
| 多函数、长CFG、多类型/字段 | 装载/验证/内存放大 | decode/verify/load各耗时、loaded RSS/受管峰值 |
| owned replace/失败构造/容器clone | 生命周期和预算代价 | 结果/事件正确性、分配/释放/取消延迟 |
| fresh/warm/restart/局部编辑 | 实际开发周转 | 端到端P50/P95、阶段span、缓存命中与重算 |

对比组分列AV解释器、V8 Ignition固定解释层、V8 Wasm基线层、V8 Wasm优化层；可另列现有Cranelift native。每组固定版本/实际flags/CPU/预热/样本及返回值校验，证明执行层没有中途变化；具体flags以被测d8/Node版本实际支持为准。本轮未运行这些组，不能给出相对Wasm/V8的实测速比。

体积同时报告code-only、必需metadata、debug、envelope、压缩包和装载后内存。LEB、Wide、专用opcode、常量池、slot复用各做消融；先定位实际占比，再决定ISA变化。798的P95<1s编辑指标继续保留，但不能替代解释器吞吐和体积对标。

## 8. 建议的决策顺序

1. **798当前正确性复核**：先验证ABC-A01的公开路径反例，绑定当前提交；若成立由原owner按现有review/work流程修复，本文不改其状态。
2. **820前冻结共享合同**：unit/module、类型等价、MIR/ABC/LoadedProgram边界、Place/资源模式、必须保留的元数据、旧profile拒绝策略。这里只冻结接缝，不把完整所有权/JIT塞进798。
3. **小范围效率验证**：优先消除block_base重复前缀和确认Call分配占比；再比较窄wire、frame arena和预解码执行计划。用结果决定是否另立有界优化Plan。
4. **821–824按既有依赖扩面**：每个新增操作先有typing/use/def/kill/effect/cleanup合同及区分反例；新增profile不扩大817/818标量支持。
5. **实测后再决定累加器、superinstruction、shared/GC、JIT和稳定公开ABC**。这些并非当前正确性探针通过的隐含要求，也不应因对标V8一次全部引入。

用户下一步可以据此选择要采纳的设计变更；本轮交付仅是上述评审建议及输入证据，没有修改现行设计/计划和实现。
