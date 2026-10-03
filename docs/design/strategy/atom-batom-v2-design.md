# Atom / Batom v2：独立数据模型、内存结构、Schema 与编码

> 状态：Draft RFC，2026-10-04。用户需求已记录；本文技术选择是待讨论的建议，尚未冻结格式或实现。
> 本次只建设新方向的设计。主电脑计划已到 739/740，本地不可见，暂不分配正式设计/Plan 编号。
> 不改旧 auto-atom/auto-val/Shell Batom、parser、VM 或消费仓；后续实施仍按设计/Plan/worktree 流程。
> 关联：[v0.6 提前开发](v0.6-offline-development-strategy.md)、[统一安装体系](unified-package-install-strategy.md)、[v0.6 roadmap](../../roadmap-v0.6.md)。

## 1. 用户需求与本稿建议

| 用户需求 | 建议 |
|---|---|
| 独立于 auto_val::Value，有自己的内存数据结构 | Atom 是独立数据层；auto-val 以后依赖/组合 Atom，禁止反向依赖 |
| 高性能访问 | 连续存储、文档内 ID、借用视图、Schema 字段槽和按需索引；区别构建态与冻结态 |
| 普通值语义 struct 自动序列化/反序列化，必要性待议 | 建议必要；显式 derive/声明启用，由 Schema 驱动字段映射；不以宿主内存 dump 代替编码 |
| 高效二进制，外置 Schema 或自描述待议 | 同一模型/基础编码，两种 profile：自描述 D 与 Schema 紧凑 S；先交付 D，S 经测量后加入 |
| 对应 Schema | 同时服务校验、访问器、结构体映射和版本演进；不要等编码器完成后补 |
| 必须有 Node 文本，最好有 Lisp 文本 | 两种可逆文本视图映射到同一模型；Lisp 是数据语法，不执行代码 |
| 其他需求 | 类型保真、引用、未知字段、确定性、错误与资源限制、演进、流式 IO、兼容策略和公平基准 |

“比 JSON/BSON 高效”是代表性工作负载上的实测交付目标，分别测大小、编解码、
字段访问、内存与分配。不能宣称在所有小消息、结构或硬件上同时胜出。

## 2. 边界：一种模型、多种表示

```text
Auto / Rust / C 的值语义类型 ──生成的 codec / view──┐
知识数据 / 包清单 / HIR 的领域模型 ──Schema────────┤
                                                   ▼
                                       独立 Atom 数据模型
                                         │     │     │
                                         │     │     └─ Batom/D、Batom/S
                                         │     └─────── Lisp 数据文本
                                         └───────────── Node 数据文本
                                                   │
                                      builder / frozen document / views
                                                   ▲
                                        auto-val 的未来适配层
```

Atom 不依赖 VM 的 Value、执行器、UI、Auto 编译器或动态对象机制；可供其他语言使用。
Atom 描述可持久化、可交换的数据。闭包、Future、函数指针、进程指针、文件/GPU 句柄
不属于可直接编码的普通 Atom 值，不能默默转换成 null。

HIR Schema 是 Atom 的使用者。类型检查、名称解析、支配/作用域等编译器语义由 HIR
层负责；通用 Atom Schema 不承担完整 Auto 类型系统。

旧源码事实：auto-atom 直接依赖 auto-val；其 Atom 只有 Node/Obj/Array/Empty。
Shell Atom 是 Value 加管道 AtomType，Shell Batom v1 对 Node 等未支持变体写 TAG_NIL。
现有 AST 的 ToNode/AtomWriter 也不是语义分析后的 HIR。上述现状是迁移输入，不是 v2 契约。

## 3. 逻辑数据模型

### 3.1 核心值种类

| 种类 | 语义 |
|---|---|
| Null | 显式空值；不等于字段缺失 |
| Unit | 无有效载荷的值；与 Null 区分，不直接复刻 VM 的所有 Nil/Void 规则 |
| Bool | true/false |
| Signed / Unsigned | i8/i16/i32/i64、u8/u16/u32/u64；位宽属于数据类型，溢出报错 |
| Float | f32/f64；二进制保留位模式，文本有特殊值的无损写法 |
| Text | 严格 UTF-8 文本，保留原字节，不自动 Unicode 归一化 |
| Bytes | 任意字节，不能与 Text 混用 |
| Symbol | 符号/标签，与字符串值区分；名称按文档/Schema 命名空间解释 |
| Array | 有序异构值序列 |
| TypedArray | 同类型数值等的连续序列；避免每元素重复类型 tag，不能混入动态值 |
| Object | 无 Schema 的字段集合，核心键为文本/符号名；拒绝重复键 |
| Record | 绑定 Schema 的字段集合，字段由稳定 FieldId 标识，有明确 presence |
| Node | 节点种类、统一命名字段、真正的位置参数与有序内容 |
| Variant | 明确的类型/分支标签与载荷；表达 enum、Option/Result 等领域数据 |
| LocalRef | 当前文档内的显式引用，面向 HIR 的引用/共享关系 |
| Link | 其他文档/实体的描述性引用；读取不触发下载或解引用 |
| Opaque extension | 可保留但未知语义的扩展值；不得当成已校验的普通值 |

i128/u128、decimal、大整数、timestamp、UUID、tensor shape/stride 等先通过 Schema
和有命名空间的扩展表达；是否提升为核心 tag 根据真实场景和基准决定。
整数不绑定 usize/指针宽度。字符可由单标量 Text 和 Schema 约束表达。

这是候选核心集合，TypedArray、引用与扩展可分阶段实现，但初版格式要预留明确边界。
Map 非字符串键若确有需求，作为独立 typed map 扩展设计，不把 key 悄悄 stringify。

### 3.2 Object / Node：共享字段基础，保留语义区别

本节根据用户提出的 Obj/Node 冗余与 prime/args/body 歧义修订，取代初稿
“括号命名参数与正文 Field 是两个独立字段区域”的建议。以下仍是待确认设计。

Object 表达按 key 查值的字段集合；Node 表达具有种类、属性和有序内容的节点。
二者都能组成树，但只有“能组成树”不足以证明它们是同一种值：

```atom
{ "title": "笔记", "tags": ["Auto", "HIR"] }

document(title: "笔记") {
    paragraph(text: "第一段") {}
    paragraph(text: "第二段") {}
}
```

重复 `paragraph` 是两个按顺序出现的节点，不是重复的字段 key。
`document(title: "笔记") {}` 即使没有子节点，仍保留种类；不能自动变成普通 Object。
Object 内也可以显式放置 Node/Array 等值，但不把名为 `children` 的字段自动当作内容。

建议合并的是字段存储、查询、Schema 校验和 codec 基础设施，不是抹去这两类语义。
从 JSON 语法到内部表示可以经过 lowering；内部 enum 是否合并本身不决定 JSON 能否解析。
但若给所有 Object 附加节点种类、内容和构造规则，会让简单配置/字典承担不必要的概念负担。

```text
FieldSet = unique field name/FieldId -> Value

Object { fields: FieldSet }
Record { schema: TypeId, fields: FieldSet }
Node {
    kind: Symbol,
    fields: FieldSet,
    args: ordered Array<Value>,
    content: ordered Array<ContentItem>
}

ContentItem = Child(Node) | Text(Text)
```

`FieldSet` 是共享职责接口，不要求 Object/Record/Node 使用同一种索引策略。
Object 可用小表扫描/动态索引，Record 可用 Schema 槽位；Node 复用这些策略。
没有内容的 Object 不分配内容数组，也不为了共用代码变成完整 Node 实例。
ContentItem 的 Text 为知识文档的文字/节点交错留出位置；文本 literal 产生式尚待定案。
第一原型可以先实现 Child，但不能把混合内容默默降为一个汇总 text 字段。

字段有唯一名字；内容有顺序，可重复种类。需要某字段多个值时用 Array。
Child 的业务 ID 不充当内容容器唯一 key，避免相同 ID/空 ID 的子节点覆盖。
字段 lookup 不依赖它原来在 prime、括号或正文哪个地方；提供共同的借用访问接口。
取 Node 的 fields 是显式字段视图，不能称为 Node→Object 的无损转换并丢掉 kind/content。

#### 3.2.1 三处写法，一套属性语义

建议保留易写的表面语法，归一到唯一 fields：

| 表面形式 | 新契约建议 |
|---|---|
| 括号内 `style: value` | 写入 fields.style |
| 正文内 `style: value` | 同样写入 fields.style |
| prime | Schema 声明的 primary field 的语法糖，不另存一份 prime 值 |
| 无字段名的位置参数 | Schema 明确绑定到字段，或保留为真正有位置语义的 args；不可猜测 |
| 正文内子节点 | 追加到 content，不进入字段名空间 |

以 Button Schema 声明 `primary_field = text` 为例，以下领域简写与显式形式归一为相同值：

```atom
button "保存" (id: "save") { style: "primary" }
button(text: "保存", id: "save", style: "primary") {}
button { text: "保存"; id: "save"; style: "primary" }
```

所有写法的重复赋值统一报错，包括 prime 与显式 text 冲突、括号与正文重复 style；
即使值相同也不能自动掩盖重复声明。动态属性合并/覆盖如有业务需要，走显式领域机制，
不依赖“后写赢”或某个区域优先。诊断指出两个来源位置，迁移工具可辅助改写旧数据。

字段集合按 key/value 定义语义，插入顺序仅为稳定遍历/展示偏好，不能让字段搬位置改变值。
字段相对于子节点的位置、注释和区域来源由可选 CST/layout sidecar 保留；内容项的顺序
仍是核心数据。真正需要按顺序执行的条目，应建模为 Seq/Array/content，不伪装成字段覆盖。
Auto 源码里的属性表达式可能有副作用：语言前端仍必须保留和执行规定的求值顺序，
本设计的数据归一化不是授权编译器重排表达式。

#### 3.2.2 primary、identity 与 binding 必须分开

primary 是“该种类最常用的一个字段”的简写位置，不是一个运行时 fallback 查询算法。
Schema 可声明 Button/Text 的 primary 为 text，Image 为 src，Project 为 name；也可不声明。
不能因某实例同时出现 id/name/text 而临时切换 primary，也不能按字符串/数字类型猜字段。

必须分别描述：节点 kind、primary field、业务 identity field、语言变量 binding、
内部 AtomId 与 LocalRef 锚点。某种 Schema 可以明确让 primary 与 identity 指向同一字段，
但不能让所有 Node 的 `id()` 自动读取 primary，导致改按钮文案就改身份。
裸标识符是符号值还是语言变量绑定由语言语法定义；Atom 数据核心不自动定义变量。

Node Schema 至少描述 `primary_field: FieldId?`、可选的位置参数绑定表、identity 字段策略、
字段类型/presence、内容类型/基数和顺序约束。primary/参数绑定属于 descriptor 契约，
需随 Schema 演进受控，不在解析器内维护独立的组件硬编码表。

未知 Schema 时，基础数据格式使用显式字段名，保留真正的 args，不能猜 prime 是 id/name/text。
领域简写在 Schema/组件注册表已明确时先归一；canonical Node/Lisp/Batom 写完整的语义值，
无需让接收者重新猜简写。可先读原始 SyntaxNode 等待绑定，但不能把尚未绑定的树宣称为
已经校验/完整解释的 AtomDoc。描述符可由调用者提供或明确嵌入，不自动访问网络。

#### 3.2.3 本地旧实现的输入证据

本地基线不包含主电脑未 push 的计划 739/740，以下只能说明已看到的代码：

- `auto-val/src/node.rs` 的统一 API 已把括号/正文字段放入 props，以 num_args 划分来源；
  legacy args、id 等路径仍存在。位置参数的统一 API 使用空 key，不能作为新协议的槽位设计。
- 通用 `main_arg()` 首先读取 id；legacy args 为空时查 `_arg0`、name，否则读第一个 legacy arg。
  `id()` 又依赖 main_arg。这不是统一的 id/name/text primary 声明。
- `auto-lang/src/parser.rs::get_primary_prop()` 按 UI tag 硬编码 id/name/src/text，并默认 text；
  普通 parse_node 路径还区分标识符 binding 与其他值映射到 content。

因此新设计需解决“多处各自推断”的问题；不是立即修改这些兼容路径。
待完整 v0.5 恢复，逐条做语法/AST/runtime/UI/数据读写的迁移矩阵和显式 adapter。

### 3.3 缺失、空、默认值和引用

- 缺字段不是一个普通 Value；Object/Record 查询返回 presence 与值。
- Optional、Nullable 和 default 分开：字段不存在、存在但 Null、使用默认值不能合并。
- 树结构是基础；共享/环用显式 LocalRef，编码器不递归展开引用，也不重复复制大子树。
- 内存 AtomId 是文档内句柄，序列化必须重映射为 frame 内对象/锚点编号；不带进程地址。
- LocalRef 允许的目标与是否允许环由 Schema/消费方验证；引用可存在不代表树遍历能无限跟随。
- Link 记录目标身份与可选选择器/Schema 身份；包依赖、知识引用、所有权仍由领域层区分。

## 4. 内存设计：独立、高性能、可借用

### 4.1 建议 API 层次

以下是职责草图，非可直接编译或 ABI 已冻结的 Rust 定义：

```rust
struct AtomBuilder;                 // 可变构建，拥有各 arena/buffer
struct AtomDoc;                     // 紧凑冻结文档，拥有存储
struct AtomId(u32);                 // 当前文档内定位，不是稳定业务 ID
struct AtomRef<'a> { /* doc, id */ } // 借用视图
struct AtomHandle { /* owner, id */ } // 跨作用域保有文档所有权
struct AtomCell;                    // 内联标量 / 容器描述 / 索引
```

候选存储是连续 AtomCell 数组、容器 item/field 数组、节点头部数组、UTF-8/bytes
buffer 与 Symbol 表；以索引/范围访问，减少逐字段 Box、String 和引用计数对象。
第一版用安全 enum/结构布局验证；紧凑 16B cell 是候选测量目标，不是未实现的承诺。
不先引入 NaN boxing、裸指针 tagging 或依赖宿主布局的 union 协议。

### 4.2 访问路径

| 访问 | 建议机制 |
|---|---|
| Array / TypedArray 下标 | 连续元素/offset，O(1)，边界检查 |
| Record 已知字段 | 编译后的 Schema 将 FieldId 映射到槽位，presence 单独保存 |
| Object/Node 字段 | 小对象连续扫描；大对象按需 ID→位置索引，阈值实测 |
| 子节点/正文遍历 | 有序连续 item，借用迭代，不 clone Value |
| 字符串/字节 | 对冻结 buffer 借用，UTF-8 校验后返回视图 |
| 反复路径查询 | 预编译 selector/访问器；不是每次拆字符串路径 |

字段名、节点名、enum Symbol 优先 interning；普通用户文本不强制驻留，防大字符串
去重表反而浪费时间/内存。运行时 SymbolId 是文档内 ID，序列化/跨文档比较按名称
或 Schema 的稳定身份，不比较两个文档的整数 ID。

初版不承诺在所有操作上零分配。由 String 名创建查询、按需索引、COW 编辑和编码
都可能分配；基准要区分预编译访问器与首次访问。

### 4.3 构建、共享、修改与回收

Builder 支持修改/预留容量；freeze 明确承担压紧、引用重映射和校验成本。
冻结 AtomDoc 不可变，适合跨线程共享；借用 AtomRef 不逐节点持有 Arc。
跨作用域保留 AtomHandle 才持有 owner，不能让引用比 buffer 活得更久。

第一版采用 builder→freeze；COW、局部 patch、增量回收另阶段设计，防止在原型期
同时实现通用 GC 和持续变化的 arena。若支持可变删除/复用，必须有 generation/
生命周期守卫；不能复用 AtomId 后让旧引用指向另一个值。

未来 auto-val 可以保留优化过的运行时标量，并让复合数据变体组合 AtomHandle/
Atom 存储。是否共享、COW 或重新物化必须保持 Value 的复制/写入语义；“借用 Atom”
不是直接替换全部 Value，也不是任意不同布局的两个 struct 可以相互 reinterpret。

### 4.4 内存形式与二进制形式

先以语义模型一致为目标，不要求两种表示逐字节一致。
内存高效访问偏向固定槽/offset；传输紧凑偏向 varint、packed 数组和省略重复名称。
Batom 字符串/bytes 可在验证后借用；可变长编码不是 Rust/C struct 的天然内存布局。

若 HIR/知识索引实测有强烈 mmap/随机访问需求，再评估带 offset table 的 read-optimized
profile 或本地缓存格式。外部数据必须校验后访问，不能直接把不可信 bytes cast 成 AtomDoc。
可搬移 buffer 内保存相对 offset；schema LayoutId、AtomId 均不能变成跨平台 ABI 承诺。

## 5. 普通 struct 的自动编解码：建议保留，显式启用

### 5.1 必要性与范围

HIR 数据、包清单、系统配置与知识记录都需要类型化视图/实体。如果没有自动映射，
各工程会手写同类转换；字段变更容易导致 text、binary、Schema 三处漂移。

建议以声明/derive 生成三类能力：

1. 类型→Schema 描述，包含稳定 TypeId/FieldId/VariantId。
2. owned 类型↔Atom 的 encode/decode，按字段/边界检查转换。
3. 可借用 typed view/accessor，以及可直接写 binary 的编码路径，避免必须先构造整棵 DOM。

不为所有类型默认生成。涉及 private、secret、skip、rename、default、required、
未知字段、字段/分支编号等均需声明；拥有指针/资源/闭包的类型默认拒绝，允许显式 adapter。

### 5.2 Struct 数据映射不等于内存 dump

以一个 `Position { x: f32, y: f32 }` 为例，Schema 定义两个字段的意义、编号和类型；
Rust、C、Auto 各自生成安全转换/访问器。宿主字段 offset、padding、对齐、大小端
只用于当前目标的本地 adapter，不进入通用 schema 身份或持久化内容。

固定布局同类型数组可以有快路径，但必须证实宽度、位模式、对齐、endianness 和
生命周期条件；不能把任意 C struct memcpy 成“跨平台零拷贝”。字符串、指针、动态
数组和所有权尤其不能这样处理。

Schema 应产生“类型正确的访问器”，不承诺从任何 Batom buffer 直接获得合法 `&T`。

### 5.3 Rust/Auto/C 的分工

Rust 可先用可选 derive 层；C 使用 schema 生成 codec/view；Auto 的编译器自动合成
等完整基线恢复后接入。Schema 与映射契约先独立设计，不在这几天修改旧编译器。
Serde 可作为可选互操作 adapter，Atom 模型与主要 codec 不强制依赖它。

## 6. Schema：数据契约、访问契约与演进

### 6.1 三层边界

| 层 | 内容 |
|---|---|
| 核心类型 | 标量、Array、Record、Node、Variant、引用等的通用规则 |
| 领域 Schema | 包、知识、HIR 等的字段、基数、类型/分支、引用目标、数据约束 |
| 消费方语义 | HIR 类型检查/控制流、依赖求解、权限、知识推理等，由各领域执行 |

Schema 自身以 Atom 表达，定义版本固定的元 Schema；先提供 owned 描述和编译后的
Schema/访问表。Bootstrap 需要少量固定元结构，不要求用尚不存在的 AutoC 编译它。

### 6.2 必需描述

- namespace、schema family、revision、精确内容 fingerprint；revision 本身不证明兼容。
- 稳定 TypeId/FieldId/VariantId；名称为可读标识/alias，与内存槽位、生成字段顺序分离。
- scalar/record/node/array/variant/ref/link，optional、nullable、required、default。
- Node 的统一 fields、primary/位置参数绑定、identity、内容顺序与基数、重复 child 和引用目标规则。
- bounds、enum、长度/数量、引用闭合；约束语言先保持有限、声明式，不执行任意代码。
- 废弃/保留编号、版本迁移声明、unknown 策略、编码选项与域扩展命名空间。

不自动按 Rust/Auto 字段顺序分配公开 FieldId，否则重排源码会破坏协议。
本地内存 offset 和 runtime hash 索引不写入持久 Schema 的跨平台部分。

### 6.3 演进规则建议

- 已发布编号不复用；删除字段/variant 后保留编号。
- 增加 optional 字段可作为兼容候选；加 required、改变类型/默认语义必须显式评审。
- 重命名但保持编号与类型可保留二进制身份；文本旧名通过有界 alias 适配。
- 编码文档的 Schema fingerprint 标识精确发送方描述；不同版本需明确兼容/迁移，
  不能因 family 相同而套用新内存布局。
- 通用 reader 保留可跳过的未知字段/扩展 bytes；typed decode 为投影时，要声明它
  是否携带 extras。重新编码能否保留未知字段取决于此，不能空口承诺。
- Schema 获取由调用方提供或解析明确的本地/可信 registry；解析器不隐式联网。

### 6.4 Schema 文本示意

下例仅表示元模型意图，字段名和元 Schema 语法尚未冻结：

```atom
schema(namespace: "auto.demo", family: "Person", revision: 1u32) {
    record(name: "Person", type_id: 1u32) {
        field(name: "name", field_id: 1u32, type: :Text, required: true) {}
        field(name: "age", field_id: 2u32, type: :U32, required: true) {}
        field(name: "nickname", field_id: 3u32, type: :Text, optional: true) {}
    }
}
```

Schema 不只服务二进制压缩。即使采用自描述编码，也需要它生成 typed view、
校验记录、明确默认值与支持演进。

## 7. Batom：自描述 D 与 Schema 紧凑 S

### 7.1 取舍与建议

Protobuf 的 wire record 使用字段号、wire type 与载荷；字段名字/完整声明来自对应
Schema。wire type 允许跳过未知字段，不能单靠字段号还原全部领域含义。
见 [官方编码说明](https://protobuf.dev/programming-guides/encoding/)。

Cap'n Proto 使用分段数据与相对指针支持跟随内存中的结构；这是直接访问的一条
技术路线，不代表任意压缩字节都能按宿主 struct 布局读取。
见 [官方编码说明](https://capnproto.org/encoding.html)。

建议 Atom 使用同一逻辑模型/基础 codec，提供以下两个 profile：

| | Batom/D（自描述） | Batom/S（Schema 紧凑） |
|---|---|---|
| 必须有外部 Schema 才能读结构 | 否 | 是；可内嵌 descriptor，或由调用方提供精确 Schema |
| 字段/节点名称 | 在字典/数据中携带 | 优先 FieldId/TypeId，可省名称 |
| 通用查看、知识与应用数据流通 | 默认选择 | 配有 Schema/registry 才使用 |
| 固定记录批量、HIR 缓存/RPC | 可用，开销实测 | 重点优化候选 |
| 未知数据 | 可看通用结构/原始扩展 | 没有 Schema 时只能保留结构编号/原始载荷，不猜字段含义 |
| 类型约束与语义校验 | 可选 Schema 进一步校验 | Schema 必需，但领域语义仍由调用方检查 |

D 也可以附 Schema 身份/描述来验证；S 可以携带 Schema 后成为可独立流通的文件。
独立知识包等不能只附一个接收方无法获取的 hash 就宣称自包含。
两种 profile 不做两套数据模型/不同意义的 Node；S 优化不改变文档语义。

首个实现优先 D，建立通用语料和基准后做 S。若收益不足，不强行增加第二种编码的维护成本。

### 7.2 候选 envelope

为兼容明确拒绝旧 Shell v1，建议沿用 magic `BATM`，major 使用 2，不能伪装为 v1。
以下字节布局是候选，需要 golden vectors 和资源/兼容验证后才能冻结：

```text
magic[4] = "BATM"
major:u8 = 2
minor:u8 = 0
profile:u8 = D | S
flags:u8
metadata_len:ULEB128
payload_len:ULEB128
metadata[metadata_len]      // 自带长度的可扩展描述
payload[payload_len]       // 一份有明确 root 的 Atom 数据
```

metadata 可以包含 Symbol 字典、Schema identity/descriptor、锚点表和扩展说明。
可选项默认不写；critical 扩展不认识则拒绝，noncritical 扩展可保留/跳过。
压缩属于显式扩展或外层容器，不默认压缩所有小消息，也不能把压缩数据伪装成可借用视图。

major 不兼容拒绝；minor 的新增能力必须有可识别边界，不能简单忽略所有未知 flags。
ULEB128 最大位宽和长度受限，长度算术检查溢出；单 frame API 拒绝未消费尾部字节。
多 frame 流通过单独 reader API 显式消费，不借尾部容忍掩盖错误。

### 7.3 D 的核心编码候选

第一轮采用 `kind + payload_length + payload` 的可跳过 value envelope。
kind 和 length 使用有界 varint；Null/Unit/Bool 可有固定零/小载荷，但优化 tag 分配
在基准后确定，不在草案中锁死未经测试的字节编号。

- 整数 payload：signed ZigZag+varint、unsigned varint；保留逻辑位宽。固定宽编码
  为可评估替代，不能因为小整数压缩好就默认所有大数/随机访问场景也更快。
- 浮点：固定 little-endian IEEE f32/f64；保留有符号零与 NaN 位模式。
- Text/Bytes：显式字节长度；Text 校验 UTF-8，Bytes 不进行有损字符解码。
- Symbol：文档字典索引或 inline；名称/字段名去重。普通字符串去重为可选优化。
- Array：数量和有序值；TypedArray 只声明一次 element type/数量，检查载荷精确大小。
- Object：有序 name/value 项；Record：type/schema identity 与有序 field ID/name/value。
- Node：kind、统一 fields、真正的 args 和有序 content；不编码字段原来所在的语法区域。
- Variant/Ref/Link：明确 tag/目标描述；未知 kind 以 Opaque 保存，不能默默改成 Null。

无索引的 TLV 适合顺序扫描，但随机字段读取不自动 O(1)；reader 可按需建立 offset
索引，成本计入首次访问。需要更强直接访问时再评估 index/read-optimized 扩展。

### 7.4 S 的优化边界

Record 用稳定 FieldId 代替名称；Schema 决定字段类型和可生成访问路径。
字段仍保留可跳过的边界，不能仅因“Schema 已知”取消所有长度/演进信息。
packed homogeneous array、共享 descriptor 与批量记录有机会减少重复类型信息。
是否省每字段 kind/长度需要兼容/跳过与实际收益证据，不能把 S 草率变成宿主布局 dump。

若缺 Schema，S reader 可以检查 framing/资源边界并保留原字节，但不能承诺恢复
完整 typed Atom/Node 文本。Schema 自身与身份必须严格验证后生成字段槽访问器。

### 7.5 编解码的真实成本

已知大小的 frozen doc 可先计算长度再写，或用有界缓冲；不能为未知长度输出宣称
“既完全单遍流式又无任何缓冲”。Dictionary 构建和 sizing 都计入 encode 基准。
真正分块/无限流、随机写回等由后续明确的 framing/patch 协议解决，不混入核心承诺。

## 8. 文本：Node 必需，Lisp 可逆视图

### 8.1 Node 文本核心

建议保持熟悉的 Node + Object + Array；核心只解析数据，独立于 Auto 执行器。
支持注释、字符串转义、严格 UTF-8、位置参数、命名字段、有序内容和根标量/容器。
canonical writer 保留明确 `{}`，避免空节点与求值调用混淆。

```atom
person(name: "Ada", age: 36u32) {
    role: :researcher
    topic(subject: "compilers") {}
    active: true
}
```

裸整数字面量默认 i64，裸浮点默认 f64；显式后缀保存 i32/u32/f32 等位宽。
`:researcher` 是 Symbol，不是 Text。名称不合词法规则时使用有转义的 symbol 写法，
具体词法在格式原型中定案。`field` 名、Node kind、Symbol 值不能因首字母大小写隐式换义。

Bytes、Variant、LocalRef/Link、TypedArray、带身份的 Record 与特殊浮点数需要保留
命名空间的明确 literal/构造表示；下一阶段先冻结这些产生式，再宣称全模型文本往返。
literal 是受限的数据构造，不调用用户函数。NaN payload 的无损文本要写明确位模式，
不能打印成普通 `nan` 后仍声称逐位保真。

括号与正文中的命名字段归一到同一个 fields；上例的 role/active 不是另一类属性。
canonical writer 可把短字段置于括号、长字段置于正文，但排版不能改变属性语义。
Schema 可以定义 primary/位置参数简写，但核心无 Schema 解析必须无歧义。领域简写先展开成同一
模型，展开结果与来源位置可查；不让每个 Schema 插入任意可执行 parser。

普通 JSON 数据的语法兼容是目标，需用 corpus 验证。JSON 标准仅建议成员名唯一，
并未在语法上禁止重复 key；v2 严格 reader 的拒绝策略需明确披露，不能宣称接受所有合法 JSON。
兼容 adapter 如需处理重复 key，必须由调用者显式选择策略，不在核心默默覆盖；
Atom→纯 JSON 对 Node、Symbol、Bytes、引用等需要显式 tagged adapter，不假装无损直转。

### 8.2 Lisp 数据文本

提供 `parse_sexpr/write_sexpr`，不是只有 debug printer；canonical 形式显式区分
Node、Array、Object、Record、Field/Child 等。以下是上面 Node 的等价视图候选：

```lisp
(:node :person
  (:args)
  (:fields
    (:field :name "Ada")
    (:field :age 36u32)
    (:field :role :researcher)
    (:field :active true))
  (:content
    (:child
      (:node :topic
        (:args)
        (:fields (:field :subject "compilers"))
        (:content)))))
```

Array/Object/Record 等各有明确构造 tag；保留数据项类型与 fields/args/content 的区别，不依靠猜测
`(name value)` 是函数调用、字段还是子节点。更短的领域 S-expression 可由 Schema
另定义，但 canonical 形式不能丢类型/顺序。

不处理 eval、宏求值、变量捕获或插值执行。已有 AST `to_atom()` 的历史 Lisp 写法
作为 adapter 输入，不能在未核对时宣称与新版 canonical sexpr 全兼容。

### 8.3 语义往返与编辑往返

普通 AtomDoc 保证约定域内的数据语义/类型/顺序往返，不默认保留注释、空白与排版。
编辑器需要的逐字节往返用独立 CST/SourceMap/trivia 层；源位置作为 sidecar，
不强迫所有知识包/网络消息携带文本编辑元数据。

浮点位模式、整数位宽、字段 presence、unknown bytes 和引用目标是数据保真部分，
不能被“语义差不多”掩盖。canonical mode 与 raw lossless mode 的不同规则明确命名。

## 9. 其他必须提前设计的需求

### 9.1 确定性、内容地址与版本

同一确定性输入得到相同 canonical bytes。定义 Symbol/锚点重新编号、Object 字段
顺序、Schema 字段编号、浮点与扩展的规则；Array、真正的 args 和 Node content 有序，不排序。
根据 §3.2 的字段集合语义，建议 canonical 模式按明确的名称字节序/稳定 FieldId 排列字段，
避免同一字段搬到括号/正文、调整字段插入顺序就改变 canonical hash。
普通 writer 可保持展示顺序；真正需要 key 顺序参与语义的 OrderedMap 应另作明确类型，
不能一边视 Object 为字典、一边隐式赋予它按字段顺序执行的含义。具体排序规则尚待冻结。

定义 wire version、Schema family/revision/fingerprint、领域语义版本三个维度。
内容 hash 不包括无关进程地址/缓存/访问索引；签名、校验和、压缩与 Merkle 分块是
外层可选机制，不能把 checksum 当成来源可信证明。

### 9.2 引用、来源与扩展

LocalRef 解决文档内关系；Link 解决跨文档关系；领域 Schema 携带业务身份/出处。
来源位置、权限、生命周期和知识推断状态可以通过 sidecar/Schema 表达，不给所有
Atom 值硬塞 AutoOS 业务字段。扩展有命名空间、明确 critical/optional 策略。

### 9.3 安全读取与诊断

限制 frame/字符串/数组大小、嵌套深度、引用/锚点数量、解压后预算和索引构建量；
检查 offset、长度乘加、重复键、非法 UTF-8、未知 critical tag、dangling ref。
不对不可信深层数据无界递归，不因未知长度预分配巨大容量。

错误包含格式版本、byte offset、逻辑 path/field ID；文本增加行列/源码 span。
禁止有损替换未知值、无声溢出/截断、自动执行数据或自动拉取 Link/Schema。

### 9.4 查询、选择与局部更新

第一版提供借用 iterator、字段/下标访问与可编译 path。复杂查询语言、patch/diff、
跨文档全文/图查询暂不纳入核心；Schema/引用 ID 的设计不应封死这些能力。
冻结数据和工作副本的区分应支撑后续知识版本、HIR 缓存和安装事务快照。

### 9.5 可移植与模块依赖

核心以稳定宽度、显式字节序和可选 alloc/IO 边界设计。MCU no_std 完整支持在
v0.7 考虑，不因此把桌面随机访问改成链表。裸 Unicode 名称规则跨语言一致。

候选模块分工：

```text
atom-core       独立模型、builder、frozen doc、view；不依赖 auto-val
atom-text       Node/Lisp parser + writer
batom-codec     D/S reader/writer、framing、校验
atom-schema     描述、校验、访问表；依赖 atom-core
atom-derive     可选类型映射/代码生成
auto-val adapter 未来消费上述能力，保持 runtime 语义
```

这只是职责分层，不强制一开始创建六个 crate；第一批可用少量独立 crate 的模块实现。

## 10. 验证矩阵与性能目标

### 10.1 正确性与格式验收

- Node/Object/Record/Array/TypedArray/标量、位宽、浮点位模式、缺失/Null/Unit、引用与扩展。
- Node→模型→Lisp→模型→Batom→模型；compare 定义覆盖顺序、presence 与类型。
- 括号/正文/Schema primary 的同义写法归一相等；跨区域重复字段拒绝并报告两个位置。
- 无 Schema 不猜 primary；改 text 不改 identity；多个同类/同 ID 子节点保持数量和内容顺序。
- 普通 Object/Node 共享字段访问但保留 kind/content；空 Node 不降为 Object；JSON 重复 key 策略明确。
- struct derive/adapter 的 owned/borrowed 读取、schema mismatch、未知字段保留与投影限制。
- 旧→新格式显式 adapter 与拒绝路径；只验证实际可映射域，损失必须报告。
- 版本/Schema 演进：新增/删除/重命名/编号保留、未知 variant、default/required 变化。
- golden byte vectors、跨语言 reader、一份协议测试 corpus；格式冻结前不能只有 Rust 自对拍。
- fuzz/损坏输入：截断、超长 varint、整数溢出、深层、越界、恶意长度、引用环/悬空。
- 生命周期：buffer 释放/搬移、clone/share、builder freeze/reindex 与跨文档引用拒绝。

### 10.2 公平基准

| 工作负载 | 主要问题 |
|---|---|
| 小配置/包清单 | header 与 dictionary 是否反而增大，小对象 lookup 税 |
| 多模块 HIR/AST 结构 | 名称去重、类型/引用、顺序、选择性读取与最大内存 |
| 知识文档/关系快照 | 大文本、metadata、Link、unknown 保留与更新消费 |
| 大批普通 Record | Schema 编译首次成本、codec/typed view、重复字段 |
| 数值数组/科学数据 | packed 数组、宽度/字节序、借用读取；与等价数据比较 |

记录 payload 和 Schema/字典总大小、encode/decode 时间、首次及稳态字段访问、
峰值内存、分配数、冻结/索引成本；尺寸由 KB 到 MB，较大档受资源预算控制。
比较相同语言/优化等级/硬件/安全校验和等价语义，区分 DOM↔DOM 与 typed↔typed；
不能只拿 Batom 借用 view 比 JSON 全 DOM 解析，或漏掉 Schema/字典生成成本。

JSON/BSON 使用公共等价子集；额外 Node/Bytes/引用类型采用公布的 tagged 映射，
单独报告转换成本。可补 Protobuf/其他格式作为 Schema 模式对照，结论限定工作负载。
达不到预定主场景的目标时先分析/优化或收窄适用范围，不发布全面优越宣称。

## 11. 近期开发分期建议

1. 讨论并确认模型边界：Obj/Node 共享基础、统一字段与 Schema 简写、Record、Null/Unit/presence、引用。
2. 独立内存原型 + 借用访问 + 第一版 Schema 描述，采用新样例，不续做旧公共类型。
3. 冻结 Node/Lisp 核心产生式和扩展 literal；实现 Batom/D 最小集与黄金语料。
4. owned struct codec/typed view 与 HIR、安装清单、知识记录三个真实消费者样本。
5. 测量后调整布局/字典/整数策略，再决定 Batom/S 与 read-optimized profile。
6. 完整 v0.5 恢复后再接 auto-val、旧 AST/VM/CLI；不在这几天实现旧接口迁移。

每期有独立验收，不将格式名称或 Draft 示例当成代码已实现；正式编号与执行计划在
解决主电脑编号冲突、方案确认后建立。本次未执行代码或 cargo 测试。

## 12. 待讨论决策

1. 是否接受“自描述默认 D，Schema 紧凑 S 后续实测加入”的路线？
2. Record/Node/Variant 是否全部进入第一阶段模型，引用与 TypedArray 分期到哪步？
3. struct 自动 codec 是否作为 v0.6 必交付，Auto 编译器合成延后到完整基线接入？
4. 是否接受 Obj/Node 保留语义区别、共享 FieldSet，prime 由 Schema 声明、同名字段跨区域唯一？
   内容混排、真正 args、Symbol 写法、identity、canonical 字段排序细节如何冻结？
5. 主要消费场景优先 HIR、系统包清单还是知识数据；哪些基准决定性能交付门槛？
6. Lisp 是否第一阶段就实现 parser，还是先 writer 后在格式冻结前补 reader？

## 13. 资料与使用边界

- 本地实际代码：`crates/auto-atom/{Cargo.toml,src/atom.rs,src/parser.rs}`、
  `crates/auto-val/src/{node.rs,value.rs}`；Shell 的 `ash-core/src/pipeline/{atom.rs,batom.rs}`。
- 历史意图：[Atom](../raw/atom.md)、[扩展 Atom](../raw/extending_atom.md)、
  [Atom serialize](../raw/atom-serialize.md)、[MicroVM](../raw/microvm-atom.md)。
  它们不是全已实现的现行协议，不直接继承其中 ABI/自动合成/性能宣称。
- [Protobuf 编码](https://protobuf.dev/programming-guides/encoding/)、
  [Cap'n Proto 编码](https://capnproto.org/encoding.html) 的官方内容已只读核对。
  [FlatBuffers Schema](https://flatbuffers.dev/schema/) 留作后续对照；本次访问得到主要为导航的页面，
  未用其未核实正文支持具体结构布局或性能结论。
- [JSON 标准 RFC 8259](https://www.rfc-editor.org/rfc/rfc8259)：Object 的字段集合、Array 顺序与重复成员名的互操作边界。
