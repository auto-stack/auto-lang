# 标准库后端覆盖与 AV provider：一个执行器，多种实现来源

> 2026-11-16 · Draft 0.5。用户明确新增标准库能力至少支持AV/AC/A2R/A2PY，AV另需native/C/Rust三类实现来源。本稿为设计和计划约束，不是当前支持声明；本轮不写实现、不跑产品、不改canonical Specs。
> 衔接[平台I/O与内存底座](native-platform-io.md)、[Design56](../56-ace-m2-ffi-and-native-stdlib.md)、[Storage](../memory/storage-and-allocation.md)。当前最终三端为AV/AC/A2R；A2PY标准库义务保持，A2C完整对齐增强、原记录保留；AC阶段按§2.1。

## 1. 决定

维护一套AV解释器、一套Auto标准库公共API和语义合同、一套类型/错误/授权/资源规则；native、C、Rust是AV可选择的标准库provider，不是三个VM、三套语法或三条前端。provider选择与执行后端是正交轴。

| 轴 | 本轮要求 | 影响 |
|---|---|---|
| 执行/生成后端 | AV、AC、A2R、A2PY；原A2C继续 | 决定执行字节码或生成目标代码 |
| AV标准库provider | native、C、Rust | 只切换必要底层原语/系统资源的实现与依赖 |
| OS/arch | 当前Windows/Linux x64基线 | 实际目标ABI、编码、OS入口与依赖 |
| API/语义 | 同一模块/操作ID/签名/单位/错误/ownership | 不允许provider改变调用者可观察含义 |

默认仍是native：普通Auto算法加受检OS薄适配，不隐式依赖libc/Rust std；C/Rust是显式可选实现。纯数学、编码、集合、format等共享Auto源码，不为“三种provider”复制三套算法。只有原语/外部资源边界需要不同adapter，某provider缺所选必需操作则具名拒绝，不能静默回落其他provider。

## 1.1 普通标准库自有实现与高级外库（2026-11-15当前）

math、text/bytes、集合、parse/format、io/File/path及net/HTTP等普通标准能力，由共享Auto源码实现算法、协议和资源语义；native最低边界为Linux syscall / Windows公开系统DLL与OS原语。math不得默认包libm，io不得默认包C stdio/Rust std::io，net不得默认把reqwest/hyper/libcurl/Tokio网络栈充Auto实现。CPU受检原语与OS socket/事件机制允许；不是重写内核TCP/IP。

高级数据库、TLS/第三方图形/媒体等引擎暂可显式FFI，列真实依赖/功能边界，未来Auto替换另立实施增量。TLS库提供加密/握手不允许顺带接管普通HTTP/缓冲/URL等默认实现。参考Go/Rust的API组织与分层，不从“参考Rust”推导Rust各平台都不依libc。

AV/C、AV/Rust可继续显式提供兼容实现；A2R/A2PY/原A2C的运行时与低层adapter可以因目标而异。它们不代签AV/native或AC默认自有实现，也不要求纯算法复制四份来源文件。974负责Auto网络协议实现，983仍负责产品Web宿主接线，975负责自有JSON/编码与外部引擎分账。

## 2. 最低验收路径

每项新增公共标准库能力在其完成计划中有以下required路径；不是把四个后端再全乘三个provider。

| 路径 | 执行方式 | 最低证明 |
|---|---|---|
| AV/native | 唯一AV + 自有Auto/OS provider | 实际调用链与结果，无默认libc/Rust std委托 |
| AV/C | 同一AV + C provider | 实际C adapter/外库dispatch，不以改配置名或同native执行冒充 |
| AV/Rust | 同一AV + Rust provider | 实际Rust库经稳定C兼容shim/handle adapter调用，非直接私有Rust ABI |
| AC | 共享HIR/MIR→原生程序 | 实际执行，不借旧VM或先转译C/Rust业务代码 |
| A2R | 共享Checked HIR/A2X→Rust | 生成工程真实编译运行；生成Rust本身不是结果 |
| A2PY | 共享Checked HIR/A2X→Python | 新producer生成工程真实Python运行；旧AST emitter不代新路径 |
| A2C增强项 | 保已有能力/原用例/支持事实，完整对齐争取 | 用户最新决定：缺完整对齐不阻M2；不得伪造PASS或删历史 |

Windows按当前已承诺平台格实跑；Linux先实现代码与保留fixture/原oracle，当期不运行Linux测试，逐操作移交[1063](../../plans/1063-ace-stdlib-linux-deferred-validation.md)为deferred-required/NOT_RUN，待1061原生Linux入口与环境完善后实跑。当前建设包的Linux完成条件改为实现与移交审查，不能升为verified；这不豁免Windows六路径/原A2C，不删Linux分母。某个OS底层指令/对象格式本身不需要跨后端同形，例如A2PY不生成ELF；1039/1040等基础设施不为新公共API签完成，消费它新增print/File等能力的计划才必须关闭上述路径。不能借“基础设施”名义把真实新增公共操作排除；948逐操作审计归属，985核完整分母。

纯Auto算法在三个AV配置下可复用身份、输入、算法和观察均相同的证据，且必须有适用性说明；系统/分配/资源等provider边界必须各有实际dispatch与失败/清理证据。不要求每个纯函数另写C/Rust实现，也不机械重复全部无变化用例。代表用例不能代替未覆盖公共操作的支持事实。

## 2.1 当前阶段门与最终对齐

### M2-A基础可用、M2-B三端对齐与AAC实证（2026-11-16当前优先）

用户最新裁定：**M2最终AV/AC/A2R公共能力一致；A2C完整对齐为增强项。** M2-A仍须完整AV/A2R语言/库及原required真实验证，AC至少有能承载AAC启动的真实多模块/类型与owned数据、泛型/Option/Result基本链、所需CTE/FFI、文本/分配/同步文件等基础。AC其余具名差额允许移M2-B：原行ID/输入/oracle/错误及资源义务、实际来源、B-owner/关闭条件必须保留，不能记PASS/NA或从分母删除。仅“原native不支持/旧profile外”不再是M2最终豁免。

[1067](../../plans/1067-ace-ac-native-async-and-task-parity.md)补原生异步/Future/Task；[1068](../../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)收全部三端语言/库/FFI/领域差额，并以AAC实际模块持续实证。985/A不等待两包，985/B及最终986/987必须收三端完整闭合；A2C现有代码/用例/支持记录保留、缺完整对齐不阻M2。A2PY既有标准库义务与AV三provider、Linux1063延期规则不变。

AAC在A出口后、M2-B期间推进，以真实Auto编译器模块检验AC语言/库；某任务所需能力未交只阻该任务/切片，缺口回语言/库真实owner补齐。AAC未使用的语言/领域/异步特性仍补独立三端用例。完整Gen1→Gen2→Gen3自举签章保持独立，不强加为M2通过前置；其所暴露的M2三端能力缺口必须在M2最终前关闭。此规则优先于旧“AC所有义务必须A期全齐”“M2-B与AAC完全无能力交接”“原A2C全量同等硬门”等表述。

## 3. 一个 AV 的组织方式

库调用统一解析到逻辑操作/共享Auto函数；会话装配将需要外部实现的操作绑定到版本化provider。拟概念字段为provider ID/来源类别/native或C或Rust、target/ABI、支持操作及语义版本、依赖与销毁入口；具体类型扩947既有模型，由967/1039各自任务核定，不建立第二registry/loader/值系统。

默认provider可按会话/模块显式配置，必要时操作级覆盖；不让普通业务每次print都写provider参数。选择在装配/实例化时验证并冻结，运行期不因库缺失、错误或恢复而擅自换实现。允许同一AV中不同模块明确使用不同provider，前提是资源来源与操作合同成立；不建立av-native/av-c/av-rust三套解释器二进制或语义分支。

C路径使用受检C ABI，CStr转换只发生在要求NUL的边界；长度buffer保持ab\0cd五字节。Rust路径复用967的公共元数据→自动C兼容shim，String/Vec/Result/opaque及panic、callback/future规则不另造。实现来源与实际ABI分开：同样C ABI不等于C标准库，Rust provider可通过C ABI提供服务。

## 4. 资源和状态不能因切换而分裂

资源身份绑定原provider/ABI版本/会话/代际与析构责任。C FILE*、Rust File对象和native fd/HANDLE不相互强转；方法/close按原provider分派，显式转换须有合同。即使模块默认provider之后改变，在途资源仍保留并调用原provider，卸载受现generation/lease保护。malloc分配的对象不能交Rust allocator释放，Rust返回资源也不能由Auto另一allocator随意释放。

统一错误类别保留原始来源码及部分进度；EOF/空值、EINTR/取消、NUL、编码、短读写、借用/移动/重复close/最后owner逐provider验。异步端口和调度继续归967/968，不为三个provider各建executor。恢复持久化只携身份，不恢复旧进程指针、原生句柄或grant，重新准入与授权后再调用。

## 5. 后端依赖与“无 libc”的适用边界

默认AV/native与独立AC的原生库路径仍遵此前无libc合同。AV解释器/启动宿主自身的实现语言与依赖另列，native provider不是“整个现AV进程已经脱离Rust”的证明；runtime自举不由本轮三来源要求代签。AV/C、AV/Rust允许它们已声明的库依赖；A2R/A2PY目标运行时依赖亦单列，不能要求Python解释器本身无C运行时，也不能用该事实豁免AV/native。所有路径的Auto公共语义必须相同；实现与运行时可以不同。

第三方数据库/TLS引擎等显式功能依赖继续逐路径列明；native AV必须能消费受检FFI，但这不证明外部引擎已由Auto重写或无C/Rust。默认基础库的无libc/无Rust std合同与显式第三方功能依赖分账，不把本次provider要求外推为重写所有生态库。

平台原语(native syscall/内存区域)→共享Auto库算法仍是默认路径硬前置；967提供显式C/Rust adapter；Python后端就绪由1042先行，不等待整个标准库或977的GD/REVERSE。完整公共新能力仍覆盖本次必需路径，允许已审切片先phase-fold交接；当前Windows Python/C/Rust腿pending不能勾完成，Linux腿仅按用户明确延期合同移交1063，不得套用到其他缺口。

## 6. 计划落点与依赖防环

| owner | 本次责任 |
|---|---|
| 1042（新增M2-A） | 最小Checked HIR/A2X→Python工程/库端口/错误资源映射与真实运行就绪；无972/973/977/985反向依赖；不是Python全语言或新增库操作全路径认证 |
| 977/PY | 消费1042已有producer/运行时继续完整Python语言、跨模块/全部原语料，不复制Python后端；GD/SCENE/REVERSE原任务保留 |
| 967 | 单AV provider装配与C/Rust adapter能力，补A2PY对其生成FFI适配的消费；F0仅原早期Rust切片，不被扩大 |
| 946、972–975、1041 | 每项新增公共库能力实现并记录六条主路径，保原A2C；算法共享，provider边界各有真实执行/独立oracle |
| 948 | 逐操作更新后端/provider分母和owner/缺口，不以研究或基础设施归档冒能力完成 |
| 978 | Python代码/runtime/provider/shim与解释器依赖、目标/ABI/版本入候选闭包和cache；B0最小范围保持 |
| 985 | 拒绝漏A2PY/AV任一provider、emit-only、旧emitter或静默fallback；原A2C/平台/分母保留 |

标准库完整能力计划对1042和所需967形成完成依赖；基础设施1039/1040不因此反等Python/完整FFI。1041的早期native切片只需945最小文本与已交底座，整包另依967/1042及必要PY切片，不依972整库；977/PY可消费已交库但1042不反依完整977/库。需要更大语言特性时T-01具名最小前置并复核DAG，不以万能“等977全完”掩盖阶段环。

## 7. 逐能力验收记录

每个操作/特征子行包含：源/独立expected、后端、AV provider、target/ABI、实际实现与依赖、真实命令/子终态、错误/资源/恢复结果和来源身份。复用现TestCase/worker/固定组，不造第三套测试矩阵系统。a2py/ace-a2x/stdlib等现职责组按真实改面选并集；缺Python工具/库/runner保持pending，不自动NA/PASS。

控制至少包括：C或Rust配置却执行native、缺provider静默fallback、错provider资源关闭、漏Python行、仅生成未运行、旧Python emitter替新producer、所有后端相等但共同不符独立oracle。纯函数无外部dispatch时明确分类，用资源边界真实case证明三provider可用，不伪造调度计数。

本稿只确定架构/覆盖/依赖，不声称已有Python新后端或三个完整provider。具体public spelling/descriptor wire与Python类型表示由对应T-01有界先证，改变语义/平台/兼容门须正式修约。

## 8. 文件后缀选实现来源（2026-11-13用户确定）

保持平铺：公共`io.at`加`io.at.at`/`io.c.at`/`io.rs.at`/`io.py.at`四份候选实现，共五个顶层文件。所有文件都是Auto源码；最后`.at`是源码扩展名，前一个at/c/rs/py是实现来源。装配时公共文件与选中的一份实现组成一个逻辑模块，后端另行指定；AV/C与A2C可消费同一受检Auto C适配源码，AV/Rust与A2R可消费同一受检Rust适配源码，调用桩/代码发射归后端。不承诺现旧侧文件无需迁移即可复用。

既有loader按目标选`.vm.at/.rs.at/.c.at`，新来源语义须在既有装配owner扩展，识别最长保留后缀且不将`io.at.at`误列独立公共模块；948盘点、1043扩既有source装配、967消费并接真实FFI，旧归档计划不重开。平台不再扩一套后缀和优先级：少量差异在provider内用静态条件，大段差异拆普通平台模块后条件导入，沿966的统一绑定/CTE接线。

## 9. 平台静态匹配与 #in 编译期命名空间（2026-11-13用户确定）

用户确定采用`#in`，它给紧随的结构或块建立编译期命名空间上下文，内部`is`/`if`/`for`等控制结构继承编译期属性，不再重复写`#is/#if/#for`。多平台示例：

```auto
#in sys is current {
    Windows -> use auto.platform.windows.io: open, read, write, close
    Linux -> use auto.platform.linux.io: open, read, write, close
    Mac -> use auto.platform.macos.io: open, read, write, close
}
```

块式同样表达该规则：

```auto
#in sys {
    is current {
        Windows -> { /* 选择Windows声明/导入 */ }
        Linux -> { /* 选择Linux声明/导入 */ }
        Mac -> { /* 选择macOS声明/导入 */ }
    }
}
```

`current`解析到本次编译/装配目标的`sys.current`，Windows/Linux/Mac按sys公开常量定义查找，组成同一OS判别域；不是三个布尔开关。示例常量拼写遵用户方案，底层target的windows/linux/macos值映射在1043/T-01核定，不隐式大小写转换。原完整限定名`#is sys.current { sys.Windows -> ... }`继续有效，独立布尔形式为`#if sys.current == sys.Windows`。宿主OS另命名；Windows宿主生成Linux代码必须选Linux。macOS只是可扩展示例，当前产品验收仍是Windows/Linux，未自动新增Mac运行支持。

`#in`作用于名字查找与编译期阶段，不创建运行时对象，不把sys成员永久use到外层。名称遮蔽/歧义/hygiene复用唯一binder，禁止无声覆盖已有绑定，限定名可消歧；选中声明/use仍按原模块的编译期Emit规则落位，不被临时namespace层吞掉。编译期的是控制选择、展开和声明/导入处理；声明出的普通函数不会仅因位于该区域就立即执行，函数自身阶段规则及CT权限/预算继续沿原合同。namespace没有赋予任意编译期OS副作用或读取宿主环境的权限。

复用现`is/#is`匹配和`#`整结构规则，`#in`只是显式阶段及命名空间上下文，不新增匹配器/解释器。单行`-> use`与块式`-> { use ... }`都进统一语句解析/绑定；未选分支只保必要语法检查，不符号绑定/类型检查/链接。平台选择表重复等值OS臂须诊断，一般is的有序guard规则保留；无匹配造成必需IO操作缺实现时由装配/绑定具名拒绝，不全局改变一般#is空展开规则。target/namespace输入、选中源/provider/ABI进入原查询读集和恢复身份；未选文件body变化不导致本目标语义重编，候选存在性与目录变化仍按真实解析依赖处理。

旧`in sys #is current`及“in仅作候选、不必实施”的建议已由用户此次`#in`决定取代。裸`in`的通用运行时namespace scope仍非本轮必做；本轮要求实现显式`#in`及上述最小静态选择，不据此扩大任意对象with/动态namespace功能。

## 10. 加载机制交付范围（2026-11-13设计快照）

新增[PLAN-1043](../../plans/archive/1043-ace-stdlib-context-assembly.md)：平铺provider后缀解析、公共声明与所选实现的逻辑模块装配、最小`#in`阶段/namespace与条件导入、缓存/恢复/错误来源、跨后端实际消费。复用944/947/960模块/选择事实与936/958 source-CT前门；不重开已归档计划、不另造loader/binder/CTE。最小目标判别值可在现标量CT支持域表示，具体编码在T-01核定；缺所需合法源能力先具名修约，不借旧VM求值或手填HIR。

1043是本套新加载机制的下一实施计划。它的AV/AC及既有A2R/A2C装配部分可在936/958/944/947/960正式来源核实后推进；全包的A2PY实跑需1042正式交付，因此1042是最终依赖，不是第一步调研/装配工作的阻塞。1043不依完整966/967/972/973；最小#in与平台选择由1043先交，966消费1043继续完整CTE/宏/metadata与原完整target条件验收，原任务范围保持，避免“库等966整包、966又等库”的环。

1039（OS I/O/内存原语）可与1043并行，1040继1039补Linux ELF/syscall。两者可继续显式目标接线，不能冒新#in/后缀装配已交；1043也不能冒无libc I/O已实现。1042提供Python最小producer，967提供真实C/Rust FFI；946/1041/972–975及完整SDK/认证最终消费这些必要来源，缺任一required路径仍未完成。纯选择fixture只证明文件/作用域/阶段和实际后端消费，不代AV/C或AV/Rust真实外库dispatch证明。

| owner | 对新加载机制的责任 |
|---|---|
| 1043（新增） | 首交平铺source装配与最小#in、单行use/模块可见、增量/恢复/诊断、AV/AC/A2R/A2PY及原A2C消费；不以选择标签冒真实外库能力 |
| 966 | 消费1043，不重写namespace/静态选择器；T-09/T-10扩完整CTE目标条件/宏环境并复核最小#in继承与依赖行为 |
| 967、946、1041、972–975 | 整包消费1043正式装配/语义来源，能力级真实provider调用/资源验收仍各归原owner；F0等早期切片不新增等待 |
| 948 | 分母追加本加载机制的来源/公共符号/后端/provider与后继，旧947/944归档不冒新合同已交 |
| 978、985 | 候选SDK保所选源及context/namespace/ABI闭包，仓外恢复实跑；完整A认证拒绝漏加载来源/后端或旧后缀旁路 |

当前1039–1043、946/948及967均已归档，完整库按972→973→974/975推进，973另需968；§6/§10的起草次序保设计来源，不作实时状态。Linux先代码后测试以§2和1063为当前完成/移交规则。本轮仅修设计/计划，未实施或运行产品，不修改canonical Specs与归档历史。
