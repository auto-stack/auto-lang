---
status: proposed
describes: target-state
created_at: 2026-10-15
updated_at: 2026-10-16
implemented_by_plans: []
---

# Design 39：AC/ACC 增量前端、热重载与通用语言解析平台

> 用户需求（2026-10-15）：增量编译、热重载和插件系统从 AC/ACC 架构起点融入；未来将解析能力复用为 Auto 版 tree-sitter，以 Atom/Batom 配置语法，服务 IDE/LSP、第三方语言快速扫描和符号/签名元信息获取。进一步明确最终目标为亚秒级开发周转与尽量少的内存；大型仓库及依赖的完整AST/CST不得以全部常驻为前提，各阶段中间产物须纳入跨工程全局cache，采用磁盘持久化与有界内存工作集。
> 本稿记录已提出的产品约束和建议架构。具体算法、公开接口、产品名、仓库归属、基准硬件/负载、内存字节预算与排期尚未冻结；亚秒周转是产品目标，尚无达成证据。不声明已有通用生成器/分页增量树/原生热重载实现，不分配正式 Plan，不修改 PLAN-782 合同或实现。

关联：[Design 36](36-ac-data-resource-expansion.md)、[后端演进](strategy/auto-native-backend-evolution.md)、[Native ABI RFC](strategy/auto-native-abi-rfc.md)、[ACC 主体合同](strategy/auto-acc-bootstrap-contract.md)、[Atom Schema](strategy/atom-schema-dsl-design.md)、[AutoCache现状](../specs/auto-cache/project.md)、[全局cache历史设计](raw/auto-cache.md)、[旧 AIE](09-compiler.md)、[开发工具](14-developer-tools.md)、[方言拆分](32-autoui-repo-extraction.md)、[模块治理](37-v06-modularization-and-repo-boundaries.md)。

## 1. 结论与范围

两项需求直接影响 lexer/parser 的数据模型、入口、错误处理和生命周期，必须在真实 ACC lexer/parser 迁移前建立约束。可以继续补齐 enum/List 等语言能力，但不能把“批处理 parser 完成以后再加增量和通用化”作为默认路线。

建议共享源码与语法服务底座，分别建设 Auto 语义编译和多语言工具消费层：通用产品负责生成/运行解析器、容错具体语法树、增量更新和结构查询；AC/ACC 负责 Auto 名字解析、类型检查、资源分析、HIR 与 native。通用语法服务不依赖 Cranelift、Auto 所有权 checker 或完整编译器运行时。

增量解析、增量语义编译、运行时热重载是三个不同验收面。语法树复用不证明语义缓存正确；缓存命中不证明机器码可安全替换；插件安装也不等于插件可在线卸载。

AST/CST本身就是可持久化的编译中间产物，与语义查询、HIR和对象码进入同一个全局产物依赖图。编辑器的源码分块/rope和编译器的树分块/按需装入采用相同的有界工作集原则；仅把完整AST写到磁盘、命中后仍整库反序列化，不满足本设计。

各层具体数据、Schema/引用身份、存放与自动回收统一见 [Design40：编译中间产物与cache生命周期](40-compiler-artifacts-and-cache-lifecycle.md)。required-data强边与失效/provenance指纹分别处理，避免成品保留导致所有历史上游永不释放；用户交付物与runtime live代际也不能按普通cache淘汰。下一契约探针同时消费两份设计。

## 2. 本地事实与缺口

2026-10-16交付要求：首条正式集成完整编译管线，即使只是hello-world，也须消费统一store、阶段依赖/配置失效、fresh等价、重启/跨工程命中和有界生命周期。语法增量不证明语义/对象增量；787只验其语法合同，后续source→HIR→native接线从第一例落实这些要求。AutoDown尽早接入，门槛见后端战略§3.1；不把新增native/测试adapter工作偷偷塞入已executing的787。

以下为 2026-10-15 源码/文档只读观察，非性能或运行验收：

| 入口 | 观察 | 对设计的影响 |
|---|---|---|
| experimental/ac-core/src/source/{lexer,parser,ast,adapter}.rs | 当前接口以完整源码为输入，lexer 返回 token Vec，parser 返回 Program/Diagnostic；独立源码 profile 选择支持面 | 可继续作为批处理入口与对照实现；未证明局部重扫、容错 CST 或编辑事务 |
| experimental/ac-core/src/atom_text.rs | Span 是 start/end 字节偏移；该 Atom reader 跳过注释、不保存版式 | 字节位置可沿用原则；不能直接称 lossless CST 或通用 grammar engine |
| crates/auto-lang/src/database/mod.rs、compile.rs；Design 09 | 有旧 AIE Database/FileId/FragId 与会话设计；细粒度增量仍有未落地部分 | 可审计复用算法/接口经验，不把旧能力直接记为 ac-core 已实现 |
| crates/auto-cache/src/{storage,registry,fingerprint,gc,aie_bridge}.rs；auto-cache Spec | 已有磁盘blob、SQLite索引、内容指纹/GC及集成桥；fingerprint当前compute_content_hash仍哈希原源码，AIE bridge接受调用方传入的接口hash | 以已有AutoCache为集成起点；不能把全阶段分页树/跨进程有界查询缓存说成已交付，不能只用接口hash缓存实现产物。Design09“全局CAS仅设计”的笼统表述已过时 |
| Design 14 | 旧 LSP 共享编译器 parser，文档提出失败后缓存上次有效 AST | 新语法服务须返回当前修订的容错树/诊断；旧 AST 仅可作为显式过期 fallback，不能冒充当前源码 |
| Native ABI RFC §1/§7/§8 | 已确认静态/动态模块及插件热重载范围，设计了宿主常驻服务、代际与安全回收 | 新前端需要模块/接口身份和重载边界；不在 lexer 内实现 DLL 装载 |
| 743 inventory；packages/aavm/lib/{token,lexer,parser}.at | token/扫描核心适配，Pratt 核心复用但 S-expr 输出重写 | 复用词法判据、优先级与测试；不能照搬单游标可变状态和 dump 产物作为新公共接口 |

当前 PLAN-782 复审/修复继续按原合同收口。此设计不向该计划追加入参快照、CST、查询引擎、热重载或生成器验收，也不以本稿认定其任何 finding 已闭合。

## 3. 推荐分层与依赖方向

```text
Atom/Batom Grammar + Language/Scanner/Query Pack
                    |
             Grammar compiler
                    |
            versioned parser artifact
                    |
Source snapshots / edit deltas / position maps
                    |
    syntax runtime: lex / parse / recover / reuse
                    |
      immutable CST + diagnostics + changed ranges
                    |
        +-----------+-----------------------+
        |                                   |
 Auto AST/semantic adapter             structure queries
        |                                   |
 resolve/typecheck/resource/HIR        editor/LSP/index outline
        |                                   |
 module/query dependency graph        foreign metadata candidates
        |
 object/link -> runtime module manager / reload transaction
```

图中是职责而非已冻结 crate/结构体。建议未来独立发布语法运行库/语法编译器；当前不创建新仓。LSP 是调用分析服务的协议 adapter，不是语法引擎内置网络服务。批处理 AC、常驻 IDE 和后续 Auto ACC 可用不同内部实现，但须通过同一语法/位置/修订契约及 conformance 语料。

所有阶段通过统一artifact/query store读写，不各自建立私有的全依赖内存数据库。磁盘产物、内存已解码对象与各工程的编辑overlay/配置是不同生命周期：共享不可变内容；工程会话只保存overlay、根引用和配置，语义配置不同则保留不同结果而不复制无关源码/语法数据。

Auto 语法与 IDE 使用同一权威语法定义/规则和适配层。手写 Pratt 可以先作为 Auto 插件或兼容实现；通用生成器不要求所有语言采用 Pratt，也不将上下文语义查询塞进通用 parser。若暂时保留双实现，明确唯一规范源、差分语料和退役条件，不能长期人工维护两份不同 Auto 语法。

## 4. lexer/parser 现在必须考虑的接口

### 4.1 源码、位置与身份

- 源码由不可变 snapshot 标识，编辑 delta 明确 base revision、目标 revision 和替换范围；只接受匹配 base 的编辑，不按到达顺序猜测。取消/迟到任务不能向新修订发布诊断/符号。
- 内部 canonical span 建议采用 UTF-8 字节区间，携带 document/snapshot 身份；原文保留 LF/CRLF、注释和空白。LSP position encoding 经协商映射，支持 UTF-16 等协议位置；774 文本 len/char_at 的 Unicode 标量语义不因源码坐标改变。
- token 同时保留原始范围与必要的词法分类；解码后的字符串值和原始拼写分开。不能因保存 char 码点而丢失字节坐标，也不能把 UTF-8 byte 当语言字符串索引。
- Node/Token handle 的有效域包括语言/grammar 代际、snapshot 和所属树；旧 handle 不可盲用到新树。不承诺跨任意编辑永久不变的 NodeId。声明级 semantic ID 通过受控匹配保持，重名/移动/重命名时需明确失效规则。

### 4.2 状态、恢复与增量

- lexer 显式持有可快照/恢复的状态：注释/字符串/插值、缩进、上下文关键字等。编辑后从可信 checkpoint 重扫，直到位置及完整状态收敛；跨文件或语法包全局可变游标不得参与未记录输入。
- 源码读入口支持分块/rope adapter，避免每次编辑复制全文；scanner 的字符推进与 byte offset 同步，不能靠从头 char_at 扫描每个位置造成隐性二次复杂度。
- CST 保留足以重建源码的 token/trivia、字段名、范围、error/missing 节点与恢复诊断。可用原 snapshot 切片保存文本，不要求每节点复制原文。
- 同一套 parse 服务面向 IDE 返回当前修订 partial tree；批处理编译可调用 strict adapter，遇 error/missing 或不支持构造拒绝进入语义 Checked。语法容错不授予 HIR 凭证。
- 增量单位从文件级正确实现起步，可进到词法区域/子树/声明；早期允许明确的全文回退。必须记录复用/重解析范围，不能把每次全文parse包装为“增量已实现”。
- grammar/scanner 版本变动会使旧 parse state 失效；旧树继续属于旧 grammar lease，重新解析才产生新代树。

### 4.3 旧代码复用的裁定边界

token 表、词法判据、Pratt binding power 和语法案例可复用；单一可变 P 游标应封装在解析会话内，语法与名字/类型环境分离。parser 输出从 S-expr dump 改为结构化语法树/AST 适配是必要重设计。旧 VM workaround、隐式 char/int 混用、历史 lexer bug 不自动继承；每项按现行语言规范审定。

共享 Syntax API 不等于把所有旧 lexer/parser 函数逐行翻译成配置。复杂的上下文扫描可以是受控 scanner 扩展；分派、状态恢复和错误策略仍由公共语法服务约束。

## 5. 增量编译与模块/接口元信息

2026-10-16：[Design42](42-semantic-metadata-inference-and-comptime.md)补全语义查询侧的类型推导、泛型实例化、AVM编译期求值及双期反射。它们与parse一样是增量查询，尤其需要记录反射集合/不存在名字等负依赖。语义事实不归Codegen或全局可变Universe所有；完整meta不要求全依赖驻留，运行时保留采用显式投影/链接策略。787不因此扩为类型系统或CT执行计划。

建议查询层显式表示 parse、declarations、resolve、typecheck、HIR、codegen 的依赖与版本。缓存不能只以文件路径、mtime、语法树地址或裸 TypeId 为键。

| 身份/指纹 | 最小职责 |
|---|---|
| source revision/content | 当前源码与诊断位置；同文本不同位置上下文不可复用错误 span |
| language/grammar/scanner/query pack | 输入语法和结构查询含义；变动按消费者实际依赖失效 |
| semantic config | 编译 profile、包/导入解析、feature、comptime输入、内建/插件能力及分析版本 |
| interface descriptor | 公开签名、类型/资源/ABI承诺、导入需求；导出实现与接口身份分开 |
| implementation/body dependency | 内联、泛型实例、const/comptime展开、效果摘要等消费者实际依赖 |
| target/backend/runtime | 对象码缓存的 target/选项/后端版本/runtime合同；不得误复用跨target制品 |

“接口不变就不重编调用者”只能在调用者不依赖函数体等实现信息时成立。跨模块内联、泛型实例、comptime/const、资源效果摘要均需相应依赖边；旧 Design09 的简化三级hash不能直接升级成新的正确性承诺。

纯空白/注释修改可能只更新位置/展示，但要核查语言的换行/缩进、doc属性、嵌入片段等语义依赖，不能统一忽略。增量结果必须与相同配置下 clean build 的语义/诊断等价；测试另验证实际复用范围和性能，不能只看等价性冒称增量收益。

### 5.1 全阶段全局cache与按需工作集

用户确认跨工程磁盘cache和有界内存工作集为架构要求。建议沿用AutoCache的元数据索引+不可变内容寻址blob方向，把历史“仅构建产物cache”扩展为分阶段、可分页查询的artifact store；具体数据库/分块编码先探针，不另造一套互不相通的parser cache。

| 层 | 可持久化/共享内容 | 内存策略 |
|---|---|---|
| 原始源码/位置 | 不可变内容块、行/位置索引；未保存编辑为session overlay | 活动窗口/必要位置页；不先读完所有依赖源码 |
| token/CST | grammar/scanner身份、token/子树块、checkpoint、error/missing、子块索引 | 按区域/子树页读取、复用；当前操作pin必要页 |
| AST/声明摘要 | 语言专用投影、符号/导入/签名、到CST的来源映射 | 类型视图尽量借用；索引/声明级按需解码，避免常驻第二棵完整AST |
| 语义与HIR | 解析后的声明/类型/约束、函数体分析、所需宏/const/泛型输入、版本化HIR与来源 | 按查询/声明/函数体缓存；背景分析可取消且受总预算限制 |
| lowering/对象/链接输入 | pass配置、目标降级、obj/库/manifest | 需要时加载或交linker；依赖接口与函数体需求分开 |
| runtime代际 | artifact/state/接口描述与实际装载映像分离 | live代码/状态按lease与代际管理；不能按普通cache淘汰仍在执行的代码 |

三层访问路径：请求先查有界内存cache，再查磁盘索引/blob，真正缺失或过期才重算。索引本身也要分区/按需查询，不能把全仓依赖图/符号表全量装入内存后才宣称AST已分页。后台预热只在预算内按热点做，不默认深度分析整条依赖树。

内存预算以字节和实际保留量计，分别统计活动页、解码对象、索引、字符串驻留、构建任务暂存、pinned快照/代际；节点数/条目数LRU不是总内存上限。预算控制需有任务并发限制、背压/取消与旧snapshot释放策略；不能靠不断加内存掩盖常驻工作集增长。

### 5.2 CST/AST物理格式与Atom/Batom

语法配置是Atom/Batom文档，不意味着每个热路径节点都必须是动态Obj或完整反序列化的struct树。逻辑Node/字段/数组接口与物理紧凑分页存储分开；语法树采用schema数字ID、块内索引/可重定位引用、源码片段引用，提供Atom借用视图与语言强类型访问器。HIR可采用专用结构，但同样以有界查询和可持久化块管理。

Batom作为候选块载体，必须探测随机定位、独立块解码、完整性与引用校验；“支持Batom序列化”不等于已经支持分页/零拷贝。每个请求仅解码需要的块，不能先解压/反序列化整文件或整库。分块粒度要兼顾局部性、压缩比、跨块开销和大声明；大函数也不能成为不可淘汰的无界单块。

目标优先验证可直接读取的冻结二进制块：mmap或有界buffer读入后，通过生成的typed view访问offset/数组，无须再分配完整Node/Obj/struct树。建议比较Batom读优化profile与本地专用块表示；不是将Rust/Auto宿主struct内存dump当持久协议。映射/首次触页、必要校验、压缩解码、跨块定位均单独计时计内存，避免以“零拷贝”掩盖首次访问成本。

不可变子树保存相对宽度/位置映射等可复用数据，文件开头插入文本不应仅因所有绝对offset移动就改写所有下游块；实际布局由探针决定。重扫依然服从词法状态：跨区域未闭合字符串等可扩大失效范围，必须正确回退而非用错误复用满足延迟。进程指针、锁、session ID和内存局部TypeId不直接入持久格式，跨块引用有明确作用域/重定位。

### 5.3 跨工程、跨会话共享

cache identity以输入内容、阶段算法/schema/grammar版本和该阶段真正依赖的上下文为准，项目目录不是普遍隔离键，包名/版本相同也不是充分共享条件。共享层与实例层分离：

- 原文/未展开语法块在语言语法条件相同时可共享；路径与当前revision的展示绑定留在session或来源映射。若路径影响宏/语义，则纳入相应阶段输入。
- 同包不同feature/cfg/target/依赖解析/宏环境可能有不同语义实例，但无须重复存储相同原文/语法块。不同版本可共享相同不可变块，语义身份不能仅因内容局部相同而合并。
- AC/ACC不同实现仅在阶段格式与行为兼容经conformance验证后复用结果；不能只看到同schema字符串就授信。缓存HIR重新进入真实类型/结构/资源验证门才获得本进程Checked凭证，不能反序列化一个trusted标志绕过verify；该成本纳入周转测量。

磁盘cache共享不会自动共享各进程解码后的AST/类型对象。建议探测用户级共享编译/分析服务：IDE/LSP/CLI为轻客户端，多个工程是相互隔离的session，公用不可变产物、内存页cache与调度。服务内部也不允许全依赖常驻；同一artifact/context任务采用single-flight并发合并，避免多个项目同时重复分析。

无daemon模式可先用同一磁盘store+不可变只读映射/有界本地cache，必须单独测量解码复制和多进程总内存，不能将“同一磁盘文件”宣传为“只占一份物理内存”。OS文件页cache可回收但不是零开销，评测同时报告私有内存、共享映射/实际驻留与系统总压力。

### 5.4 缓存正确性、事务和回收

输入query key与输出blob内容身份分开；key包含真实实现输入与实际依赖边，接口不变不能复用已变函数体的obj。持久化依赖图需能发现变化而不先重新parse所有文件；已知snapshot/文件变更日志优先，mtime只作提示，初次扫描/恢复确认按实际工作量记录。

blob不可变、完整写入后才原子发布manifest/索引；跨进程同key生产有协调、失败/崩溃恢复和坏块检测。SQLite等存索引不强制每个AST节点一行；热点树批量读取分块blob，避免遍历产生N次随机数据库查询。

“cache数据库”是key→artifact/块目录、依赖图、事务与预算管理的逻辑服务，不必等于自研存储引擎。先比较已有SQLite索引+分片blob与索引+不可变segment/pack文件（按offset/length定位块、只读映射），分别测小文件open/目录查找、随机查询、顺序扫描、并发发布和GC。避免靠文本目录遍历查找cache，也不先假定自研数据库比成熟索引更快。pack合并/compaction不得搬走仍被reader引用的映射或产生无界临时空间；验证后才确定布局与引擎。

GC以session根、产物依赖和短期lease判活；正在读取/发布的块不可删除，崩溃lease有恢复规则。缺失/坏块/旧schema受控失效并重算，不静默返回部分成功。淘汰已解码对象不改变语义；不对live runtime代际套用普通cache删除。后台写盘/GC/预热受前台I/O和延迟预算约束。

### 5.5 亚秒开发周转的验收口径

产品目标：在明确参考设备、仓库/依赖规模与改动类型下，常规开发edit→受影响分析/代码生成→可安全生效的热重载达到亚秒；建议验收以端到端P95 < 1s为主，同时报告P50/P99和峰值内存。具体负载、样本数、字节预算与初版达标范围由下一探针冻结，不能只测parser耗时或缓存查询耗时代表目标已达成。

| 路径 | 必须独立报告 |
|---|---|
| 热工作集局部修改 | edit→parse/semantic/codegen→reload生效；实际重算/读取块数、正常与失败行为 |
| 内存冷、磁盘cache热 | 重启服务/切换工程后的首次操作；验证不全库反序列化、不因进程重启重编依赖 |
| 磁盘cache冷/公共接口广泛变化 | 完整构建/大范围失效的实际时间和内存；单列长任务，不用热路径数字冒称全部亚秒 |
| 多工程共享依赖 | 相同与不同配置、多版本/多源码副本分别测；总私有内存、共享页、重复任务与cache命中 |
| 受限内存/磁盘压力 | 淘汰、重载入、GC/损坏恢复的尾延迟及正确性；避免无界pin/换页抖动 |

一秒内完成仅在约定负载实测后声明；尚无性能结果。大范围语义变化、首次全量构建和安全点等待不能靠降低检查/丢弃受影响项伪装为命中；需要显式进度/取消和归因，并在最终产品支持矩阵说明。磁盘I/O与重计算有取舍，追求有界热点常驻而非每次操作先逐出全部数据。

### 5.6 从首例进入系统与各阶段用例（2026-10-16）

用户要求亚秒周转从开始落实，每个阶段用例都考虑该总体指标。这是v0.6架构/测试要求，不等v0.7优化器完成。阶段首次有可执行入口就定义并产出测量合同；代表负载形成性能回归，不能最后再加计时。

端到端从系统接收编辑/构建请求开始，包括防抖/排队/锁等待、失效判定、cache查找/完整性校验、读取/解码、重算、写盘/发布、后台竞争及必要安全点等待，直到该revision必要诊断/产物发布且新代码可安全生效。早期如尚无reload，分别报告请求→正确诊断、请求→可运行产物、请求→真实运行结果，不冒称热重载完成。失败/超时/取消也记录延迟与状态，不丢掉长尾样本美化P95。

parse、声明/meta查询、推导、泛型实例化、CT编译/AVM执行、HIR/verify、lower/codegen、link、runtime描述/发布及reload各阶段用例均须明确：输入/规模/变更、cache状态、语义oracle、预期复用/重算、真实调用计数、耗时/等待/I/O及峰值内存、预算和结果状态。尚不存在的阶段标not-applicable，缺测记gap，不能填0ms充数。

每次请求使用关联trace/嵌套span，分列wall/CPU/等待和子查询；类型推导与CT相互调用时避免重复归因。按真实并行依赖分析关键路径，不直接相加各阶段P95，不以“每阶段<1s”冒充端到端亚秒。阶段预算须为调度/I/O/发布留余量并绑定设备/负载，此处不凭空指定固定毫秒数。

至少分列no-op、局部body编辑、公开签名/类型变化、CT依赖变化、重启磁盘热、跨工程及GC/内存压力；持续编辑报告取消/过期任务、队列积压和最新revision完成延迟，不能只测孤立单请求。目标路径端到端P95<1s并报P50/P99；首次全量/广泛失效单列并解释，不混进热样本，也不把目标缩成空请求命中。

普通功能用例保持语义判定并输出基础观测；性能阈值门禁使用预先冻结的代表负载、样本/预热、参考机/可比环境、绝对预算与回退规则，不对每个微小单测在竞争机器上武断断言毫秒数。探针开销及测试工具构建准备时间另列，用户路径实际包含的步骤不准排除。阶段功能通过但完整周转缺测/超标时，性能门明确pending/未达，不冒称已经交付亚秒。

## 6. 热重载与三类插件

| 类别 | 扩展内容 | 生命周期与校验 |
|---|---|---|
| 语言包 | grammar、scanner、树schema、queries/语言元信息抽取规则 | 版本/内容身份、宿主API兼容、旧tree/session pin；新请求使用新代；不混合旧parse state |
| 编译器扩展/方言 | 解析扩展、语义/诊断/HIR适配或受控pass | 顺序/输入输出profile/依赖/能力明确；不得伪造Checked；非确定性输入进入依赖或禁缓存 |
| 原生运行时插件 | 业务代码、模块状态、回调/任务/析构等 | 采用Native ABI RFC的公开接口、宿主服务、lease、状态迁移、事务发布及安全回收 |

语言包可先静态注册或进程重启切换，再实施在线代际替换；不要求所有编译器插件第一版都可原地热卸载。用户要求的原生模块热重载仍是v0.6核心范围，不因分期被降为可选。数据grammar无需执行任意代码；scanner/semantic插件若有可执行代码，边界另定，普通同进程ABI不是恶意插件沙箱。

parser 只负责给出模块/声明/导入/属性及可重载边界所需的结构/来源；resolver/typecheck建立模块身份和接口描述；backend保留reload boundary、不把可替换实现内联进永久消费者；loader/runtime处理代际和状态。热重载不要求lexer理解DLL布局。

必须从一开始分开 module_identity、interface_descriptor、artifact_identity、state_schema、runtime_requirements。内存/对象/回调/外来deleter依赖旧代代码时旧代仍须pin；重载失败不发布半套入口，新代准备失败保持旧代服务，旧代有在途调用不得卸载。

## 7. Atom/Batom grammar 产品边界

用户确认以Atom/Batom承载语法配置。两者是同一grammar模型的文本/二进制表示，不是两份独立语义。grammar schema、parser artifact、公共syntax-tree schema、结构query规则和metadata schema分别版本化；Atom Schema描述数据合法性，不会自行产生任意源语言的parser。

建议grammar模型表达 terminal/regex子集、seq/choice/repeat/optional、词法/语法优先级与结合性、命名节点/字段、trivia、上下文规则、冲突、scanner扩展点及错误恢复。语法编译器必须校验坏引用/循环或不终止模式/歧义、报告unsupported，记录依赖，不以顺序碰巧匹配掩盖未裁定语义。

算法先进行有界对比探针：确定性LR/有界GLR、PEG等与Auto手写Pratt adapter的适配；评估左递归、歧义、恢复、incremental复用和scanner状态。本文不冻结选型，也不承诺任意语言或线性最坏性能。初期选两种差异明确的语言/profile作验收，再扩面；不承诺兼容tree-sitter grammar.js、查询语法或二进制ABI。

产品需提供独立库API及可批处理CLI。LSP宿主可复用语法诊断、outline、结构高亮/folding、候选symbols；完整definition/rename/type-aware completion依赖语言语义服务或既有官方LSP。

## 8. 第三方语言 symbol/signature 的信任分层

建议元信息同时记录language、source/build config、grammar/query版本、snapshot、范围、来源工具与状态，而不只输出一个signature字符串。

1. **syntax候选**：从CST提取声明/名字/参数文本/导入与范围，可不完整、未解析、条件编译不确定；用于快速导航/索引或下一步分析候选。
2. **semantic元信息**：由该语言的resolver/typechecker、官方编译器/LSP或经验证adapter补齐作用域、类型、重载、泛型/宏/条件配置和真实导出。缺失时标记unknown，不猜测。
3. **interop可调用接口**：将semantic结果映射为经目标ABI/布局/所有权/链接验证的桥接descriptor；只有这一层可供编译器建立可执行FFI绑定。

例如C/C++预处理与宏、Rust cfg/宏/泛型、动态语言运行时定义，不能仅靠grammar保证真实导出。源码声明也不证明目标二进制确实导出同符号。加载器仍须核验库/target/公开接口和运行时能力，解析器不成为新的类型/ABI授信捷径。

C header/SDK、Rust shim与Auto声明汇入统一接口描述及target语义的具体边界见 [Design41](41-interface-metadata-and-foreign-abi.md)。通用parser负责候选结构/索引，foreign语义frontend和adapter负责预处理、布局/调用与生命周期；Import provider不等于物理ABI。相关元信息/绑定/生成库与Design40共用缓存，不能每次import重复分析完整SDK。

## 9. 后续计划顺序建议（未分配编号）

| 工作包 | 可独立验收成果 | 前置与时序 |
|---|---|---|
| A. 前端与全局cache共享契约及全路径探针 | snapshot/edit/分页CST/AST/位置/恢复/grammar代际、阶段key与依赖、AutoCache集成/跨session共享、模块身份；两语法例+Auto、多项目/超内存工作集与端到端延迟探针 | 建议作为下一项架构准备计划；可先规划，涉及782实现接口待其fresh收口基线；数据库/树分页与语法引擎一起评审 |
| B. enum/is + TokenKind | Auto语言枚举与native分派；语法与语言无关token ID分层，不把通用树kind硬绑Auto enum | 可与A共同规划；无需等待完整parser-generator交付，不向782追加任务 |
| C. 集合/模块和首个ACC lexer | List<Token>、use、拥有文本适配与真实扫描对拍 | 依赖语言能力；迁移前消费A合同，旧核心按§4.3适配 |
| D. 通用grammar compiler/runtime MVP | Atom/Batom grammar→parser产物→容错CST/增量edit、结构查询及两个语言profile | 依赖A算法探针；独立产品工作包，不塞进enum计划 |
| E. Auto parser/AST及增量语义查询 | 共享语法规则、真实AST、声明/签名、查询依赖；clean vs incremental一致 | 根据A/D实测决定先adapter还是生成器接入，不阻塞所有自举语言扩面 |
| F. ABI/热重载并行线 | 模块descriptor/接口兼容、静态/DLL探针→常驻宿主/lease→状态迁移/发布/回收 | 尽早与A对齐身份，按Native ABI RFC独立实施，不等ACC迁移完再设计 |
| G. LSP与外语元信息 | 语法outline/query→semantic adapter→interop descriptor逐层验收 | 不把完整多语言语义/FFI包装成语法生成器天然交付 |
| H. 全阶段cache与共享会话实现 | 持久artifact/query store、字节预算页cache、跨session协调/single-flight、崩溃/GC恢复与依赖元信息 | 消费A及已有AutoCache；与D/E/F按接口分步接入，非全局常驻AST数据库 |

各包大小/拆分由正式auto-plan:new取材后确定。上述是架构分解，不是已批准的实施计划、预算或时间表。需要新架构合同，但不先重写全部旧编译器，不要求先完成整个通用产品才继续自举。

## 10. 关键验收反例

- 连续插删、跨CRLF/Unicode/注释/插值/缩进边界：incremental与fresh parse的树/诊断等价；checkpoint恢复完整；保留原文，坐标往返正确。
- 未闭合字符串/括号/声明：当前revision仍可查询其余结构；strict编译拒error/missing；旧AST/过时任务不得覆盖当前结果。
- grammar/scanner/query换版、插件卸载/重载：相关缓存失效；旧树可在lease内读取，释放前旧代码不卸载；坏schema/未知版本/跨revision handle受控拒绝。
- 私有函数体、公开签名、类型布局、inline/const/comptime、插件配置各自变化：只失效正确依赖，结果与clean build一致；未变化查询真实复用。
- runtime重载期间旧调用/任务/回调/析构仍存活、新代初始化/迁移失败：验证原子切换、旧代保留或明确拒绝，不能仅“DLL加载成功”。
- 同一外语声明受不同宏/cfg/target影响：候选meta不得直接获可执行绑定；错误/过期/条件不明结果有显式来源状态。
- 语法歧义、复杂regex、scanner不前进、恶意/超预算输入：明确生成期拒绝/运行期限/恢复策略，不能静默截断成功。性能目标须探针后单列规模、设备和预算，不先许诺“任意语言实时”。
- 仓库/依赖或单文件总量超过内存预算（含1GiB源码压力例），只查询小范围：记录块读取/解码、索引与临时分配，证明不是全量加载后LRU；结果仍与对应fresh基线等价。1GiB是压力负载候选，非语言尺寸上限或现已支持声明。
- 重启/切换工程、两个session同时请求同依赖、不同feature版本分叉：验证磁盘热命中、相同key只生产一次、共享原文/语法而隔离语义、私有内存不按依赖副本线性复制；不能只测磁盘去重率。
- 编辑开头使后续绝对offset改变、跨块词法状态/大声明、查询已淘汰节点：验证位置正确和必要块复用，无悬空指针/旧revision授信；handle允许重新取页而非永久pin整棵树。
- 在blob准备/索引发布/并发读取/GC时杀生产者、移除或破坏缓存块：无半提交成功、读者安全，受控重算且语义/诊断不变；可证明预算内恢复，不全库重建。

## 11. 外部一手参考（2026-10-15 核对）

- [Tree-sitter定位](https://tree-sitter.github.io/tree-sitter/)：parser generator + incremental parsing library，输出具体语法树并在语法错误下保持可用结果。
- [Grammar DSL](https://tree-sitter.github.io/tree-sitter/creating-parsers/2-the-grammar-dsl.html)：terminal、优先级、冲突、字段等是grammar模型内容，不能只提供正则列表。
- [Advanced parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)：edit/旧tree/结构复用、嵌入语言区间与并发tree生命周期。
- [External scanners](https://tree-sitter.github.io/tree-sitter/creating-parsers/4-external-scanners.html)：缩进/heredoc等需自定义扫描；完整scanner状态序列化/恢复是编辑正确性的组成部分。
- [Queries](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html)：CST结构匹配提供结构消费接口，不能代替语言语义分析。

这些是设计参考，未在本仓做性能比较、生成器选型或兼容性验收；本稿不搬入第三方产品依赖。

## 12. 待冻结决定

grammar算法与支持子集、分页CST布局/树复用、位置API与边界整数宽度、grammar/scanner/query版本政策、声明ID匹配、分阶段cache/query schema与AutoCache兼容方式、共享服务/映射模式、可持久语义表示、插件边界、首批语言与外语meta来源、产品名/仓库归属、参考设备/负载与内存字节预算，均由A及对应专项探针产出后评审。亚秒开发周转、全局磁盘cache与有界内存工作集是已确认目标；不能把具体预算未定当作重新回到整库AST常驻的理由。
