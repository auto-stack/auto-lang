---
status: proposed
describes: target-state
created_at: 2026-10-15
updated_at: 2026-10-16
implemented_by_plans: []
---

# Design 40：编译中间产物、统一引用与全局缓存生命周期

> 用户确认需求（2026-10-15）：统一设计编译各层数据、格式、引用、存放与cache；尽量使用Atom/Batom+Schema；热点驻内存、冷数据进磁盘、跨工程共享，以亚秒开发周转和低内存为目标；缓存必须自动治理，长期使用后用户无须搜寻各工程目录手动clean。
> 本文为目标态架构草案。产物目录/职责/不变量可用于下一计划取材；字段布局、Schema名/编号、GC阈值、引擎、算法与性能达标范围仍待专项探针冻结。未分配Plan，未修改实现/canonical Spec/在途782验收，未运行性能或清理测试。

关联：[Design39](39-ac-incremental-syntax-and-language-platform.md)、[AutoCache Spec](../specs/auto-cache/project.md)、[AIE Database](../specs/auto-lang/runtime/design/aie-database.md)、[阶段合同](../specs/auto-hir/stage-contract.md)、[AutoHIR](strategy/auto-hir-design.md)、[Atom/Batom](strategy/atom-batom-v2-design.md)、[Native ABI](strategy/auto-native-abi-rfc.md)。

## 1. 架构决定与非目标

2026-10-16时序：统一身份/阶段依赖/store从首次编译集成开始消费，不等完整语言后补。首个hello-world全链路也须验fresh、热点、重启、变更失效和有界回收；无实现层级保持future。AutoDown同时观察真实重算/命中与语义/诊断等价，不只看耗时或去重。最新分期见[Design43](43-ac-av-typed-mir-and-abc.md)：最小Typed MIR与ABC/AV基础进入v0.6，v0.7按需扩展性能优化/SSA/自主低层后端，不包含这些生命周期责任的延期；交付门见后端战略§3.1。

统一的是产物的语义模型、Schema/版本、身份/引用、依赖、物理访问与生命周期；不强制所有阶段共用一个万能Node结构，也不要求每次操作复制/保存全阶段完整状态。类型化内存视图、冻结二进制块、调试Atom文本是同一逻辑数据的不同消费形式。

全局store保存不可变产物和分块数据；元数据索引处理查询与依赖；共享会话服务处理overlay、调度、预算、lease和内存页。项目工作目录仅含必要manifest/明确构建输出，不再默认各自保存一套完整依赖中间产物。跨工程共享不能忽略feature/target/工具版本等语义上下文。

目标是后台自动保持健康，而非让用户定期管理cache。缓存淘汰不得改变程序语义或删除用户源码/交付物；压力超出不可回收工作集时先限制新工作，不能为了维持配额强删正在使用的代码或唯一用户数据。首次全量编译、广泛接口失效、无可用再生输入等有独立状态，不承诺任意负载都亚秒或无限容量。

非目标：重写通用关系数据库、对现有npm/Cargo目录做系统扫描删除、将所有运行时状态自动序列化、把新合同加入782、冻结公开Auto ABI、声称已有全阶段cache/自动健康治理交付。

## 2. 取材事实与现有差距

2026-10-15静态检查，观察主检出HEAD为6d1c95469d24b3dd3e33014a1f78b3b5ed6897b2；共享工作区有在途Plan簿记，本稿不修改那些内容。代码观察不是对旧模块的完整审查结论：

- `crates/auto-cache/src/{storage,registry,fingerprint,gc,lib,aie_bridge}.rs`已有blob、SQLite元数据、指纹与GC基础；`fingerprint::compute_content_hash`目前直接哈希原源码，不能推断已具备各阶段精细键。
- `gc.rs::GarbageCollector::run_gc`是元数据LRU删除；对外`lib.rs::AutoCache::run_gc`另有选择victim、先删metadata再删blob的路径，按记录大小统计。当前不同路径不能自动视为有lease/图保活/崩溃恢复/物理释放收据的统一生命周期；新实现先审计复用与退役，不追加第三套清理器。
- `storage.rs::BlobStore::put`使用临时文件+rename方向，现有固定临时名与并发完整性须专项探针，不继承为新跨进程发布已正确的证据。
- 旧AIE的FileId/FragId/各表主要是内存职责。新持久协议不能直接保存指针、DashMap或进程内arena编号，也不能将当前FragId的offset/generation直接当跨任意编辑稳定身份。
- ac-core现有S0–S6/Checked/发布和contained runner合同继续有效。本文A-*是产物分类，不替换现有阶段编号或扩大已有profile的支持。

## 3. 全阶段产物目录

2026-10-16语义细化见[Design42](42-semantic-metadata-inference-and-comptime.md)：A05保存可物化的AV编译期结果/类型或生成项描述及依赖manifest；A06保存类型/签名/推导结果/泛型实例等不可变语义事实，不持久化求解器可变状态。runtime反射为A09/A10投影并受A12代际生命周期约束。CT源码不变仍须核依赖函数body、反射类型/候选集合、target/配置/VM版本及受控外部输入；只有完整输入不变才跳过执行，不另建私有meta/CT缓存。

以下A-*为设计条目标识，不是冻结的wire tag或现有文件名。每类需有独立领域Schema、受管入口、版本、key组成、再生recipe、引用策略与oracle。小型临时结果可只驻内存；只有收益/成本适宜的结果才持久化。

| 类别 | 主要数据 | 逻辑模型与物理表示建议 | 分块/读取与cache策略 |
|---|---|---|---|
| A-00 编译输入/配置 | 包锁定图、target、feature、profile、工具链/环境与插件输入；源码snapshot身份 | Atom Record/Variant+Schema；Batom配置manifest，原始源码为Bytes/独立原文blob | 配置小记录；源码分块/位置页；用户源码本体不归cache所有 |
| A-01 语言定义/生成器产物 | grammar、scanner接口/版本、parser表/自动机、树schema、query包 | Atom配置为源；已编译表为Schema约束的TypedArray/Record；native scanner为二进制payload+manifest | 按语言包/代际共享；不每工程重建；代码插件须pin其执行代际 |
| A-02 词法/位置/checkpoint | token kind、raw range、行索引、可恢复scanner状态、可选解码字面量 | Node/Record/TypedArray+Schema；只读可定位块 | 按词法区域/块；状态收敛后复用，原始拼写引用snapshot |
| A-03 容错CST | node kind/field ID、有序children/trivia、error/missing、相对宽度与来源 | 通用Node/Array/Record+Schema；读优化Batom或同模型冻结块、Atom/typed view | 不可变子树/页；按需映射；不能默认整文件/全部依赖CST常驻 |
| A-04 AST/声明摘要 | Auto AST投影、声明/导入/参数/签名、CST到AST映射 | 强类型Variant/Record+Schema；优先借用视图，必要时物化摘要 | 声明/函数体；不用“有CST就永久保存另一整棵AST”；摘要可脱离大树读取 |
| A-05 展开与条件结果 | 宏token/展开结构、comptime条件/依赖、生成来源 | Record/Variant+Schema、源码/结构payload | 按展开调用/实例；完整记录实际文件/环境/工具输入；不可控制副作用则禁用相应共享cache |
| A-06 解析/类型/接口元信息 | scope、符号/跨模块引用、类型/约束/impl/泛型与资源效果摘要、公开接口 | 类型化ID表、Record/Variant+Schema；原文签名候选与语义已确认信息分层 | 模块摘要/声明/查询；依赖接口无需先装载所有函数体；所需宏/泛型/const内容按需取 |
| A-07 公共HIR | Unchecked Bundle、签名/类型/定义/expr/place/body/control/effect/resource/source | AutoHIR Schema；Atom调试/交换、Batom类型化块+安全view | body/类型表/定义表分区；持久结果不能直接反序列化为本进程Checked凭证 |
| A-08 语义lowering/pass结果 | Typed MIR、规范化/清理计划、目标能力门、pass链与分析失效信息 | 各pass输入/输出Schema与版本；结构化清理/能力表 | 只缓存实际实现的pass；细粒度按body/模块；重写后按契约重验，不预先发明全部pass |
| A-09 后端/对象产物 | 新ABC模块及执行profile、backend输入/必要IR、COFF/ELF obj、调试/重定位/符号、runtime需求 | Auto描述用Schema；CLIF等私有IR若缓存使用后端自有格式/版本；obj保持真实原生Bytes | 模块/后端可验证单元；不强行转成Node；CPU/target/后端/选项/实际body依赖进key |
| A-10 链接/发布产物 | exe/lib/DLL、导入/导出、链接收据、接口/ABI/runtime/artifact manifest | 原生文件+Atom/Batom/Schema manifest；内容绑定 | 重用link输入、原子发布；用户请求的交付副本是独立输出，不因cache GC消失 |
| A-11 工具投影与诊断 | 分阶段诊断、span、outline/index、外语候选meta、查询结果 | 版本化Record/Variant；领域状态/来源明确 | 查询/快照级；诊断随revision映射，失败结果只在完整输入键与适用范围内复用 |
| A-12 热重载与状态 | generation/dispatch/state schema/迁移recipe、在途引用 | 描述可用Schema；仅显式声明的可持久状态可编码，线程/代码指针/lease是活运行态 | 旧代保持到安全点和lease释放；可选回滚代数/TTL有预算；runtime回收不是普通缓存LRU |

原生机器码、第三方编译器文件与原始源码不必拆成Atom节点；用Schema封装描述、输入/输出和引用即可。“统一Atom/Batom+Schema”不等于改变PE/COFF等已有格式。格式转换不应引入文本roundtrip。

[Design41](41-interface-metadata-and-foreign-abi.md)为foreign生态具体化本目录：import/package/target输入归A00，SDK/header生成定义归A01/05/06，接口与layout/ownership元信息归A06，ABI adapter归A08，Rust/C shim编译与库manifest归A09/10，callback/deleter与加载代际归A12。头文件传递依赖/搜索与宏/SDK配置、Rust crate内容/features/依赖/toolchain/实例集合分别进入适用query key；候选CST不能成为可执行FFI授信。来源fingerprint不永久pin全部SDK AST，实际执行/读取引用的code/data按required-data和lease保活。该映射不声明新产物已支持。

[Design43](43-ac-av-typed-mir-and-abc.md)明确：MIR、ABC及AV版本/语义profile进入各适用query key；可移植ABC与target相关机器码cache分域。A05通过ArtifactId引用ABC及冻结结果，不重复存活VM堆；装载后的表/指令和portable常量可按需物化，不将格式可序列化视为可直接信任执行。各层均是目标目录，不代表当前783已实现所有producer。

### 3.1 分层执行与增量发布的产物扩展

按[Design43 §8–10](43-ac-av-typed-mir-and-abc.md)的AC/AV/AJ统一架构，A08保存共同MIR及适用优化结果，A09可同时有ABC、AJ快速/优化和发布原生版本，A10保存发布快照/完整闭包/链接清单，A12管理当前安装的代码版本与代际lease。未来运行画像/推测假设作为有版本、来源和预算的输入/描述纳入相关A00/A08/A09，不把进程指针/活反馈对象存成可移植payload。逻辑产物不为每层重复拷贝语义图。

查询键除输入与实际依赖外，按消费者区分target/CPU、优化档、插桩/调试、热重载/调用策略、ABI/runtime服务版本、画像及推测条件等。输入源码一致不保证dev/release机器码可互用；原生miss时仍可命中语义/CTE/MIR。跨单元内联保存callee body/layout等依赖，查询粒度与优化/装载单元不同，不强制MIR基本块逐个落盘。

持久机器码须有符号/重定位、数据、ABI/runtime要求、验证/生成recipe等；已装载JIT内存地址不是可发布文件。发布固定快照/profile并遍历入口/导出/初始化/反射/实例等完整闭包，校验cache后补编缺失，包含未运行冷代码。匹配的优化版本可复用，不匹配重编适用层；纯native禁回退profile不得采用需要解释器才能纠正假设的版本。

cache回收不改变构建结果：有输入可再生，无输入明确缺失；发布交付物应独立复制或进入受管持久发布store，部署不依赖开发cache，不能让普通LRU删除发布物。增量发布仍可用完整平台链接，不承诺现有linker全增量。验证覆盖warm→release、首次/第二次发布、body/接口/profile/target失效、cache损坏/回收和清除开发cache后运行；与同快照/profile的无cache构建比较语义/闭包。发布/冷构建耗时独立于编辑P95<1s报告。

以上是新增producer和查询合同的目标，不说明当前783或AJ已经提供这些能力；首个MIR/AV探针只验证其有限支持面，完整发布器和分层AJ另立计划。

## 4. 统一身份与引用

### 4.1 分清查找键、内容身份和业务身份

| 概念 | 建议职责 | 禁止混用 |
|---|---|---|
| SchemaIdentity | family/revision/语义与布局fingerprint；物理profile单列 | 格式版本不等于编译器版本或公开ABI版本 |
| QueryKey | 阶段+算法版本+规范化参数+实际输入/依赖指纹+适用配置 | 不能只用包名/版本、绝对目录、mtime或interface hash替代全部实现输入 |
| ArtifactId | 已发布manifest的内容身份，绑定Schema、payload描述和强引用 | 不是工程本地自增号，也不是本地内存地址 |
| BlobId / ChunkId | 具体不可变字节块的内容身份；编码/压缩profile可产生不同BlobId | 逻辑相同不表示物理bytes相同；不以压缩字节变化冒充语义变化 |
| Module/Def identity | 模块与声明的语义身份/配置实例；能定位当前版本 | 不凭名称或局部结构相同合并不同声明 |
| LocalRef | artifact/owner作用域内，带类别的table index或相对offset | local TypeId/FieldId/ExprId必须有owner与界限，不直接跨模块比较 |
| ExternalRef | artifact/chunk + 类别/目标table/index + 预期Schema；必要时经manifest import槽压缩 | 不在每个节点重复长hash；不能解引用任意路径/网络位置 |
| SourceRef | 原始内容snapshot + 逻辑document绑定 + byte范围/映射 | 展示路径与共享内容分离，路径确实影响语义时仍纳入相应query |
| Session/Lease/Generation | 用户会话、临时reader/任务、runtime版本的运行态身份 | 不写入跨工程内容key；保活/过期不只比较PID或墙钟 |

建议索引中QueryKey映射到ArtifactId；ArtifactManifest列出Schema、生产者版本、数据块/引用、再生recipe和来源。canonical内容不含last_used/访问次数/本机目录/租约；这些放可变索引。hash不能代替类型/边界/语义校验，也不以校验和代表发布者可信。

持久ID使用分域内容身份，内存热点可以重映射为短整数；二者经受控resolver转换。持久引用不存进程指针。包含循环/递归的语义图使用“模块/组manifest+组内ID”或import槽构造，避免逐节点hash互相递归无法落定。结构相同的CST子树共享不改变HIR中“每个表达式是独立求值位置”的语义。

### 4.2 数据引用、失效依赖与来源必须分开

| 边 | 是否保活 | 用途 |
|---|---|---|
| required-data | 被保留根确实需要可读取该目标时，作为强数据闭包保活 | manifest到payload/跨块必要引用；删目标会使当前产物不可消费 |
| recomputable | 不作永久强边；目标可缺失，有recipe时按需再生 | 读取很少的派生投影、可重算分析；缺失不是silent-success |
| invalidation fingerprint | 保存影响结果的身份/hash及依赖关系，不要求其完整bytes常驻/永久在盘 | 判断是否过期；接口、body、macro、配置变化各自传播 |
| provenance | 默认不保活大输入/日志；有显式debug/复现保留策略时另设根 | 谁/何版本生成、输入身份/原来源、故障解释 |

已存在obj可以记录其HIR/AST输入hash而不强保留整条上游历史。源码位置sidecar等只在消费者需要时保留或重建。反之，当前CST manifest需要的真实子块是强数据边，不能只保根却删子块。GC和cache miss resolver必须消费同一份显式边分类。

旧query索引/recipe/反向依赖本身也有保留策略；不能删除payload却永久留下无界元数据。逻辑依赖的环按可达性处理，不能仅靠引用计数回收。

## 5. 格式、存放与读写

### 5.1 Atom/Batom+Schema的分工

每类产物先冻结语义Schema，再选物理表示。Atom文本便于调试、审核和交换；Batom常规profile用于编码；读优化profile或同模型本地冻结块用于直接view/mmap。Schema生成语言类型、字段访问器与校验器，不要求普通owned struct与磁盘布局相同。具体Schema文档/命名/字段编号由后续计划建立，本稿中的英文概念名仅为职责草图。

块头包含可识别格式/版本、Schema、范围与完整性信息；独立块校验包含长度算术、offset/引用类别、解压预算、对齐/字节序。允许必要的分块验证但不假定首次访问“免费”；HIR重新走真实语义门后才能取得本进程Checked，后端obj命中也要核验target/runtime/来源绑定再消费。

热块优先直接view，冷块可独立压缩。builder→freeze写入新块，旧块不可原地修改；未改块共享。大文件/函数可继续分块，索引与字符串表也分页；编辑开头导致offset移动时尽量只改位置映射/相关路径，不重写所有下游块。

### 5.2 存储职责

建议沿用已有AutoCache的索引+CAS基础，先比较“独立blob”与“不可变segment/pack+块目录”。索引存键、manifest、图边、lease、预算/访问统计、发布/回收journal；大型payload不按AST节点逐行SQL随机访问。表/引擎不是新的公开ABI，选型基于实测。

逻辑分类：index/catalog、immutable blocks/segments、journal/staging、session overlays、runtime generations、quarantine。具体路径由store分配/验证，GC只能操作其受管namespace；日志、数据库WAL、orphan块、临时文件、压缩迁移副本和quarantine都计费，不让它们变成“统计外垃圾”。

跨工程service共享只读块和有界解码对象，相同QueryKey single-flight；独立客户端仅持session配置/编辑overlay/短期视图，不各自常驻整条依赖。无daemon路径可共享磁盘/只读映射，但须评估重复解码和总驻留，不等同已实现内存去重。工作树内不创建junction/symlink，也不为cache命中绕过仓库安全规则。

### 5.3 发布与崩溃恢复

建议流程：注册写入事务/预留空间→进程唯一staging→完整写入/校验→发布不可变块→事务性发布manifest与query索引→完成收据。重名内容必须验证一致，不覆盖正在读取的blob；不同生产者对同key给出不同内容须报告不确定性/坏key，而非last-writer-wins。

数据库与文件系统不假设一次原子提交。崩溃点由journal/reconciliation定位：无索引完整块是可回收orphan，索引缺块受控miss/重算，半成品不作为命中；恢复不用先加载/parse全部工程。增量扫描与读时校验受I/O预算。检查索引与数据失败不自动清空健康缓存。

## 6. 根、保留策略与所有权

“正在用”“值得留”“用户数据”必须分开。项目关闭不等于用户数据可删，项目曾经用过也不能令所有历史cache永久保留。

| 类别 | 保留/回收政策 |
|---|---|
| 用户源码、未保存编辑、交付制品、显式备份 | 不属于可随意淘汰cache；有自己所有者/保存政策。cache不能是未保存编辑的唯一可靠副本 |
| live reader/编译/发布事务、runtime在途代际 | 硬pin最小required-data闭包；操作结束释放；不可按LRU强删 |
| 正在编辑/运行工程的最近热点 | 软根/较高优先级，超预算可淘汰可重建数据，不影响源码与运行安全 |
| 近期项目/工具链/常用依赖产物 | 有限保留窗口与再生代价权重；不是永久强root |
| 历史失败/诊断/日志、旧grammar/旧版本产物 | 有界条数/字节/期限与版本兼容政策；冷且无消费者优先清理 |
| orphan/staging/过期journal/quarantine | 确认非live事务后优先清理；隔离区也有预算与期限 |
| 显式offline/reproducible pin | 与普通cache分账，容量可见且设预算；不能无声宣称用户pin可无限增加而仍满足硬配额 |

pin防运行破坏，保留窗口防短期反复重编；二者不可混用。项目当前snapshot可登记软根，closed/missing project记录在宽限后过期；从全局登记表处理，不扫描用户所有磁盘去猜工程位置。包源码、安装的工具链/SDK如果是独立受管资产，按安装/包存储政策治理，不能作为普通可重建cache误删。

lease记录连接/任务身份、进程启动身份与必要心跳；PID复用、时钟跳变、机器休眠后不能仅凭TTL删活映像。expiry触发liveness核对，不能证明安全时延迟物理回收并限制新工作。崩溃session与合法长期任务有不同回收证据。

## 7. 自动健康治理

### 7.1 预算与触发

内存预算M与磁盘高/低水位Dhigh/Dlow、磁盘安全余量R、staging/compaction临时预算、并发任务内存分别配置并纳入全局总量。建议提供自动默认值和用户可选总上限，不要求用户按工程手动clean；具体字节/比例/采样与退让算法由探针冻结。

触发包含：写入前空间预留、超过高水位、卷剩余空间触底、低频后台检查、session退出/崩溃恢复、工具链/grammar格式淘汰。GC不等到磁盘已满才启动；每次写入准入前检查真实可用空间，并为索引/journal/必要恢复保留余量。软保留可在压力下提前缩短。

### 7.2 从安全判活到价值淘汰

1. 建立回收epoch和读/写屏障，收集live lease、发布事务及显式pin所需的强数据闭包。新reader须原子取得lease或看到tombstone后重查/重算，不能在判死与delete之间偷入。
2. 优先处理确认失活的staging/orphan/过期失败记录/不兼容旧格式，再处理可重建、低价值冷产物。索引清理与大块删除协调，不能只删记录留文件。
3. 候选排序综合近期/频率、预计重算/下载成本、共享消费者数、实际可释放字节及阶段价值；LRU可作基线，但不能仅靠last_used决定全部。活动集保留有滞回，避免写入和淘汰互相追赶。
4. 软根到期或预算不足时解除保留，再计算required-data闭包；删除一个工程的manifest不删除其他工程正在共享的块。可重建父产物可先失效后释放其专属数据，不能留下“命中但必需子块已删”的状态。
5. 按有界批次回收到Dlow和足够R；每批重新核对压力与前台工作。自适应权重/统计也要有界，不为“智能”维护另一个无限增长数据库。

策略应可解释：每次淘汰可回答原因、访问/共享/代价类别与实际空间收益；默认不保留无限详细事件日志。无需机器学习才能称自动治理，先证明有界、安全、稳定收益。

### 7.3 删除、压缩整理与物理空间

回收采用可恢复状态：candidate→tombstoned→等待reader/映射退出→物理删除或segment重写→索引收敛/完成收据。顺序可由实现细化，但必须保证读者安全、崩溃后可续做、复活不会读到半删除产物。

segment内块失活不等于磁盘空间已释放。稀疏segment按有效比例与临时空间预算选择compaction；新segment和offset目录原子切换，旧映射lease释放后再删旧segment。Windows映射/占用阻止删除、外部硬链接、文件系统行为分别计入“逻辑失效/等待回收/实际释放”，不得按metadata file_size求和宣称已释放空间。数据库页、WAL/journal/checkpoint及日志轮转也需预算与维护，维护不能每次阻塞前台长时间VACUUM。

每轮收据至少列：扫描/判活/淘汰数、逻辑失效bytes、实际释放bytes、待reader释放bytes、临时峰值、失败项与retry、用时/前台影响。删除失败有具体错误与受管路径，幂等重试，不能吞错或循环增长隔离区。

### 7.4 压力与无法回收的边界

若live硬pin已占预算或没有足够空间完成安全compaction，先取消/暂停低优先级预热、限制并发、拒绝新cache写入或走有界不持久化路径；后者若也无资源则明确返回资源状态。保留旧运行代际和用户数据，不默默无限扩大quota。

“用户不用管cache”表示正常使用下自动达标，例外自动退让并给出明确可操作状态；不是保证磁盘损坏、用户无限pin、所有容量被外部占满时仍无限工作。产品要有聚合健康状态/占用与自动回收结果，clean命令是维护接口而非日常必需步骤。

## 8. 性能与正确性验收矩阵

所有数字目标须带参考设备、数据规模、样本和版本；亚秒开发循环沿Design39测端到端P95 < 1s目标，不能只测hash查找。不为通过性能减少验证或漏受影响依赖。

| ID | 代表输入/操作 | 必须区分的oracle |
|---|---|---|
| V-01 | 同工程/跨工程，内容相同但路径不同；target/feature/依赖变体 | 相同适用key复用；语义不兼容不误命中；原文/CST可共享；不同来源span仍正确 |
| V-02 | parse/semantic/pass/obj各阶段单独改配置/算法/依赖 | 对应层及真正消费者失效，未受影响层命中；不能只更改签名key复用旧body对象码 |
| V-03 | artifact LocalRef/ExternalRef错类别/owner/越界/循环、schema/profile错误、坏块 | 真实读取/验证入口受控拒绝；合法兄弟通过；持久trusted标志不能构造Checked |
| V-04 | 输入总体超过内存预算，查询小范围、跨块大声明、重启磁盘热 | 不全库装入/反序列化；测读块/解码/私有/共享/峰值，结果与fresh一致 |
| V-05 | 新obj保留、上游HIR/CST变冷；多个产物共享同块 | provenance/失效边不永久pin历史，真正required-data保活；删一工程不破坏另一工程 |
| V-06 | 多reader/producer同key与不同结果；逐发布点杀进程 | single-flight/一致性检查；无半提交命中；orphan可恢复；不同结果不覆盖成成功 |
| V-07 | GC判死后新reader、映射仍live、runtime在途、PID复用/休眠 | reader和新lease安全，旧代不提前卸载；崩溃lease可回收，活进程不因TTL误删 |
| V-08 | 连续多工程/多版本/工具链升级/失败构建 churn | 达水位后自动收敛，不随历史次数无界增长；数据/索引/WAL/临时/日志全计入；用户不手动clean |
| V-09 | pack稀疏、compaction各阶段崩溃、低盘、删除受阻 | 有界临时峰值和读者安全；收据区分失效与真释放；恢复幂等、无半套目录/数据 |
| V-10 | 交付exe/源码/unsaved overlay/显式pin与cache副本并存 | 自动回收不删用户唯一数据/交付副本；无权限逃逸出store；不遍历任意target目录删除 |
| V-11 | 热集/磁盘热/全冷/大范围变化、多工程并发 | 分别报告端到端P50/P95/P99、I/O/重算/内存与共享收益；GC/预热不挤占前台导致失控尾延迟 |
| V-12 | 缺源/缺工具链、离线包/recipe不可再生、损坏索引/块 | 不伪称所有cache可无条件再生；按资产/可再生策略保留或明确失效；不自动清空全部健康store |

长期健康压力测试用受控造数与参考设备，测试数据全部在专用受管store；不对用户真实缓存运行破坏性演练。校验日志与测试脚本后续计划物化，目前矩阵是目标不是已执行证据。

## 9. 实施分解建议（未编号）

1. **产物Schema/身份与分页格式合同探针**：与Design39共享前端探针合并取材，冻结A00..12边界、各阶段真实输入key、引用类别与两种可读块方案，跑V01..04/基础V05；复用已有AutoCache盘点，暂不实现全部语言能力。
2. **统一store/预算/事务与多session基础**：受管manifest/块/索引API、single-flight、journal恢复、byte预算，接一条真实source→CST→摘要→HIR→obj链；证据不是仅人工造blob。
3. **自动健康治理与迁移收口**：root/lease/soft-retention、图回收、物理删除与compaction、WAL/临时/日志、disk pressure；V05..10/12，原有两条GC入口确定迁移/退役路线，不能永久双写。
4. **语法产品/语义查询/热重载分线接入**：逐项消费统一产物合同与生命周期，V11端到端；性能与预算达标后才宣传亚秒/低内存。现有工具链/缓存格式兼容窗口有界，无需删除用户工程重新开始。

每项按auto-plan:new独立定界/取号、唯一worktree实施、独立review、冻结提交merge门禁；当前只做设计，不创建实施树。内存预算/Schema/GC协议是架构合同，不能在实现中靠扩大budget或永久pin逃避验收。

## 10. 未冻结问题与交接

待探针：每层Schema/字段编号与演进、Batom读优化布局、块/大声明切分、hash域与canonical算法、循环组构造、原生payload/receipt绑定、共享服务/独立模式、预算自动值/软根期限/权重、lease与跨进程崩溃证明、segment压缩与GC调度、现有cache迁移及实际性能矩阵。数据库引擎不先预设重写。

下一正式计划应把本稿与Design39作为统一前置，而非“先做AST，未来再想cache”。用户已确认的方向是各层产物可描述、可引用、可按需共享且自动受控回收；本文具体建议仍需契约审阅和实施授权。现行Spec只记录已落地事实，设计提交不等于这些能力已经实现。
