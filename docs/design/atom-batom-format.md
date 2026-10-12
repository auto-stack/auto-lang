# Atom / Batom 格式说明

> **文档性质**：格式参考手册（设计 + 详细设计 + 实现说明 + 用法）。
> **资料来源**：`docs/design/raw/atom*.md` 系列原始设计、`docs/specs/auto-atom/`、
> `docs/specs/auto-lang/frontend/design/atom-serialization.md`、计划 002–016/139/332
> （auto-lang 仓）与 Plan 291/292/295（已迁至 auto-shell 仓重编号为 013/012/014），
> 以及两边仓库的实际代码（`crates/auto-atom`、`crates/auto-lang/src/atom.rs`、
> `auto-shell/ash-core/src/pipeline/`）。
> **代码核对日**：2026-10-02。

---

## 0. 术语澄清：两个 "Atom"

本项目历史上存在**两套同名但不同层**的 Atom 概念，阅读任何文档前必须先区分：

| 概念 | 全称 | 领域 | 主场 | Batom 关系 |
|---|---|---|---|---|
| **语言级 Atom** | AuTo Object Markup | Auto 语言的数据标记格式（JSON 超集 + XML 节点），Auto 语法的静态子集 | auto-lang 仓 `crates/auto-atom` | 无直接关系 |
| **Shell Atom** | （管道数据载体） | ash/AutoShell 管线中带语义标签的 `Value` 包装 | auto-shell 仓 `ash-core/src/pipeline/` | **Batom = Binary Atom**，Shell Atom 的二进制编码 |

两者共享同一个底层值系统 `auto_val::Value`（47 个变体的动态值枚举），这是它们唯一的
血缘关系。Shell Atom 的 `value` 字段就是一个 `auto_val::Value`；语言级 Atom 的
`Node/Obj/Array` 同样构建在 `auto_val` 之上。

**Batom 只存在于 Shell 侧**——它是 ash 管线数据（Shell Atom）的二进制序列化格式，
Magic 为 `"BATM"`。语言级 Atom 的"二进制化"从未实施（MicroVM 静态池方案是另一条
嵌入式路线的纸面设计，见 §2.7）。

---

# Part A —— 语言级 Atom（AuTo Object Markup）

## A1. 设计定位与理念

来源：`docs/design/raw/atom.md`（概念起源文档）。

- **定义**：Atom 是 Auto 语言的一个子集，用来传递数据。如果把 Auto 语言看做
  "动态的数据"，Atom 就相当于 Auto 语言"凝聚"之后的产物——经过编译器处理后，
  所有动态内容（变量、函数、循环、导入）都被求值消除，剩下的静态数据就是 Atom。
- **定位**：JSON 和 XML 的结合体——JSON 的超集（所有合法 JSON 都是合法 Atom），
  同时引入 XML 式的"节点"概念描述嵌套树。
- **命名由来**：Atom = "AuTo Object Markup"。最初拟名 ASON（Auto Script Object
  Notation），因重名改为 Aton（AuTo Object Notation），又因易误读为 Atom 而
  干脆定名 Atom。
- **典型场景**：Automan 编译工具的配置语言、AutoGen 模板工具的模板产物、AutoUI
  的 UI 描述语言——凡"用 Auto 写、求值后取静态结果"的场合，输出形态都是 Atom，
  可再转换为 JSON/XML/YAML 或交给其他工具处理。

### 三支柱（`docs/design/raw/atom-serialize.md` §5）

| 概念 | 语法表现 | 物理用途 |
|---|---|---|
| **Object** | `{ k: v }` | 纯粹的数据载体（类似 JSON） |
| **Node** | `parent [id] (attr: val) { field: val; child_node; ... }` | 树状拓扑描述（替代 XML/HTML） |
| **Link** | `@ref(uuid)` / `link @uuid(protocol://address)` | 远程/跨进程句柄（跨进程锚点，反序列化时经 `LinkRegistry` 绑定物理通道） |

其中 Object 与 Node 是已实现的主体；Link 是设计蓝图（`@ref: "uuid_string"` 序列化
形态），静态解析器与 Shell 侧均未实现。

## A2. 文本语法规范（详细设计）

### A2.1 基本值类型

```js
// 字符串
"Hello, World!"
// 数字：整数 / 小数 / 科学计数法（设计文档另含 3.0f 浮点后缀记法，静态解析器未实现）
123
2.2
4.5e-10
// 布尔
true  false
// 空值（null 与 nil 等价，均解析为 Nil）
null
```

静态解析器（§A3.1）中整数落为 `i32`、小数/科学计数法落为 `f64`。

### A2.2 相对 JSON 的语法扩展

1. **注释**（配置文件场景关键特性）：

```js
// 单行注释
{
    a: 1,        // 行尾注释
    b: 2.2
}
/* 多行注释
   多行注释 */
```

2. **对象键可省略双引号**（类 JavaScript）：

```js
{ a: "1", b: "2" }
```

3. **换行可替代逗号**（逗号可省，换行本身是合法分隔）：

```js
[
    1
    2
    3
]

{
    a: "1"
    b: "2"
}
```

4. **根节点可以是数组、对象、或节点**（JSON 只允许这两种的扩展）。

5. **独立键值对**（设计文档允许顶层散置 pair；仅动态方向 AutoInterpreter 支持，
   静态解析器要求单根值——见 §A3.1 差异表）：

```
name: "Puming"
age: 41
```

### A2.3 节点语法（Atom 的核心特色）

XML 的节点：

```xml
<root id="123">
    <name>Puming</name>
    <age>41</age>
</root>
```

JSON 无法直接表达节点，只能退化成 `"node"/"children"` 约定。Atom 引入节点语法：

```
root(id: "123") {
    name("Puming")
    age(41)
}
```

规则：

- 形如 `name(args) { body }`——参数区 `(...)` 存放轻量元数据（属性/args），
  内容区 `{...}` 存放键值字段与嵌套子节点。
- **参数区**支持位置参数与命名参数：`db("my_db")`、`db(host: "localhost", port: 5432)`。
- **内容区**的成员可以是 `key: value` 字段，也可以是嵌套子节点，二者混排。
- 子节点空的 `{}` 在纯 Atom 语义中可省略（`name("Puming")` 即完整节点）；
  但在 Auto 语言中因与函数调用易混淆，节点的 `{}` **不可省略**。
- 深层嵌套继续加 `{}` 即可：`root(id:"123") { name("Puming") { surname("Zhao") {...} } age(41) }`。
- 设计文档另规定节点 id 的方括号记法 `Profile [user_001] (version: 1.0) {...}`
  （`atom-serialize.md` §1.1）；当前实现中 id 是 `Node` 的独立字段，`Display`
  以空格分隔输出在节点名之后，静态解析器不解析 `[id]` 记法。

### A2.4 Tag（枚举）的序列化策略

设计（`atom-serialize.md` §1.2）：tag 值序列化时**坚持生成名称标签**而非数字——
`button (color: RED)` 而非 `button (color: 0)`，编译器维护 `String <-> ID` 映射表；
反序列化读到 `RED` 时经编译器合成的 switch 分支还原为物理值。静态解析器侧的对应
行为是：未加引号且非 `true/false/null/nil` 的标识符解析为字符串（`Value::Str`），
名称标签因此天然可往返。

### A2.5 完整示例（`atom-serialize.md` §3）

Auto 源码：

```auto
tag Status { Active, Suspended }

type Profile {
    id    string
    name  string
    age   int
    state Status
}
```

序列化产出的 Atom 文本：

```auto
Profile [user_001] (version: 1.0) {
    name: "Gemini"
    age: 18
    state: Active  // Tag 自动转化为名称
    Bio {
        text: "Authentic & Adaptive"
    }
}
```

## A3. 数据模型（内存结构，实现核对）

### A3.1 `Atom` 枚举（`crates/auto-atom/src/atom.rs:47`）

```rust
pub enum Atom {
    Node(Node),   // 带名字、参数、属性与子节点的树节点
    Obj(Obj),     // 键值对对象（IndexMap 保序）
    Array(Array), // 有序值数组
    Empty,        // 空值（常量 EMPTY；Default）
}
```

`Atom::new(Value)` 只接受 `Value::Node/Array/Obj`，其余类型报
`AtomError::InvalidType`；`Atom::to_value()` 是反向转换（`Empty → Value::Nil`）。

### A3.2 `auto_val::Node`（`crates/auto-val/src/node.rs:100`）

```rust
pub struct Node {
    pub name: AutoStr,     // 节点类型名
    pub id: AutoStr,       // 可选唯一标识（子节点存储时作为键的一部分）
    pub num_args: usize,   // props 中 args 段的键数（args/props 统一存储，Plan 013）
    pub args: Args,        // 废弃字段，迁移期兼容保留
    props: Obj,            // 统一属性区（args 在前、body 字段在后，靠 num_args 分界）
    kids: Kids,            // 统一子节点存储
    pub text: AutoStr,     // 文本内容
}
```

子节点与字段的物理载体是 `NodeBody { map: IndexMap<ValueKey, NodeItem> }`，其中
`NodeItem = Prop(Pair) | Node(Node)`——即节点内容区本质是一张**保序映射**，
字段与子节点同池存放（Plan 012 从 BTreeMap+Vec 迁到 IndexMap，获得 O(1) 查找与
插入序保持）。

### A3.3 值与键

- `auto_val::Value`：约 47 个变体的动态值（`value.rs:147`）——标量族
  （Byte/Int/Uint/USize/I8/U8/I64/Float/Double/Bool/Char/Nil/Null）、字符串族
  （Str/String/StrSlice/CStr）、容器族（Array/Block/Pair/Obj/Node）、范围
  （Range/RangeEq）、函数与 UI 族（Fn/ExtFn/Type/Lambda/Widget/Model/View/Meta/
  Method/Instance/Args/Ref/Error/Grid/VmRef/ValueRef/Closure）、代数族
  （Some/None/Ok/Err/Future）。
- `auto_val::ValueKey`：`Str(AutoStr) | Int(i32) | Bool(bool)` 三种键。
- `Atom` 的 `Display` 输出为紧凑调试形态，如 `"atom {a: 1; b: 2}"`（分号分隔、
  不转义）——**不保证可回读**，可回读的写方向见 §A4.3。

## A4. 具体实现

### A4.1 静态解析器：`crates/auto-atom`（约 2000 行，依赖仅 auto-val + thiserror）

`AtomParser::parse(&str) -> AtomResult<Atom>`（`parser.rs`）：手写递归下降、
逐字符扫描、带行/列定位的 `AtomError::ParseError`。支持的静态子集（模块注释原文）：

- 节点：`name(args) { children }`（参数区支持位置/命名参数；内容区支持
  `key: value` 字段与嵌套子节点混排）
- 对象 `{ key: value }`、数组 `[ value, value ]`
- 原始值：字符串、整数、浮点、布尔、null/nil
- 无引号对象键与节点名（标识符规则：字母或 `_` 开头；字母/数字/`_`/`-` 继续，
  **连字符是合法标识符字符**）
- `//` 与 `/* */` 注释
- 数组/对象/节点体内换行替代逗号

字符串转义表（与写方向严格镜像）：`\n` `\t` `\r` `\\` `\"`；未知转义 `\x`
解为字面 `x`。数字：无小数点/指数 → `Value::Int(i32)`，否则 `Value::Double(f64)`。

**静态解析器与设计文档的差异**（设计有而静态子集无，需走动态方向）：
`3.0f` 浮点后缀、顶层散置 pair、`[id]` 方括号记法、`@ref` Link、插值 `#{var}`
（被词法直接拒绝，保证"静态"语义）。

### A4.2 动态解析器：`AtomReader`（`crates/auto-lang/src/atom.rs`）

`AtomReader` 持有一个 `AutoInterpreter`，`parse(code)` 把任意 **Auto 代码**在
CONFIG 模式下完整求值（变量、循环、导入等全部生效），再把求值结果收敛为 `Atom`；
裸数组/对象直接透传。`read(path)` 从 `.at` 文件读取并解析。这是"Auto → 凝聚 →
Atom"方向（§A1）的运行时实现，AutoGen 模板求值产物即走此通路。

### A4.3 写方向：`AtomSource::to_at_source()`（`crates/auto-val/src/emit.rs`）

历史坑：`Display`/`to_astr()` 不转义引号/反斜杠/控制字符，含 `"`、`\`、`\n` 的
字符串序列化后**无法回读**。`to_at_source()` 是与解析器转义语法严格镜像的正确发射
器：标量变字面量（`"..."`、`42`、`3.14`、`true`），Array/Block → `[a, b, c]`，
Obj → `{ k : v; ... }`，嵌套 Node 缩进展开；VM-only 值（闭包/未来/Widget 等）
回落 Display（配置数据不应包含它们）。`to_at_source() → AtomParser::parse()`
构成完整 round-trip（`atom.rs` 内置 4 个回归测试守护，含特殊字符字符串、
字符串数组、`Uint` 精度等用例）。

### A4.4 AST 序列化三件套（`crates/auto-lang/src/ast.rs:95-137`）

AutoLang 自身的 AST → Atom 方向（规范：`docs/specs/auto-lang/frontend/design/
atom-serialization.md`）：

- `ToAtom`：返回 `AutoStr`（文本）——给天然是简单值的 AST 类型（Type → 字符串等）。
- `AtomWriter`：`write_atom(&self, f: &mut impl io::Write)` 流式直写 io，不产生
  中间字符串；`ToAtomStr` 对所有 `AtomWriter` 类型 blanket impl 出 `to_atom_str()`。
- `ToNode`：返回 `Node`（树结构）——给天然是节点的复杂语句（If/For/Fn/Store 等）。

输出格式为 **Lisp 风格 S 表达式**：`(if (branch cond body) (else else-body))`、
`(fn name=add params=(params ...) return=int body=(body ...))`；字面量 `int(42)`、
二元运算 `bina(op, left, right)`。节点产出型的 `to_atom()` 委托
`Value::Node(self.to_node())`。

### A4.5 构造 API 三层（设计：`docs/design/raw/atom-builder-api-design.md`；Plan 015/016 落地）

按场景从简到繁三层，合计减少构造代码 60–70%，全部向后兼容、零额外开销：

1. **链式方法**（简单静态构造）：`Node::new("config").with_prop("version","1.0")
   .with_child(Node::new("db"))`、`Array::from(0..10)`、`Obj::from_pairs([...])`、
   `Atom::node_full(name, props, children)` 等便利构造器。
2. **Builder 模式**（条件/运行时构造）：`auto_val::NodeBuilder`（`prop/props/child/
   children/child_if/child_option/prop_if/text/arg → build/build_atom`）与
   `auto_atom::AtomBuilder`（`node/array/obj/array_values/obj_pairs/empty → build`）。
3. **proc-macro DSL**（声明式；`crates/auto-lang-macros`，Plan 016）：`value!`、
   `atom!`、`node!` 三个过程宏。宏体按 AutoLang 语法书写，宏在编译期把 TokenStream
   转成字符串、生成 `AtomReader::parse` 调用并自动解包 root 节点——因此解析器支持
   的全部语法在宏里自动可用，且支持 `#{var}` 变量插值（`ToAutoValue` trait 支撑）。

### A4.6 serde 通路（Plan 381 / 332 收官）

`auto-val` 提供 serde `Deserializer` 适配器（`serde` feature）：反序列化方向可
`#[derive(Deserialize)]` + `node.deserialize::<T>()` 零样板转 Rust 结构体；
Serialize 方向的定制宏（原 Plan 332 的 `#[derive(ToAtom)]`）收敛为单一目标设计，
语义沉淀在 `docs/design/07-data-structures.md`「Atom Serialization」节。

## A5. 用法

### A5.1 静态解析（依赖最轻，工具链首选）

```rust
use auto_atom::AtomParser;

let atom = AtomParser::parse(r#"config { version: "1.0" }"#).unwrap();
assert!(atom.is_node());

let atom = AtomParser::parse(r#"{ name: "Alice", age: 30 }"#).unwrap();  // 对象
let atom = AtomParser::parse("[42, -3.14, true, null]").unwrap();        // 数组
```

### A5.2 程序化构造 + 序列化 + 回读（round-trip）

```rust
use auto_atom::Atom;
use auto_val::{Node, AtomSource};

// 链式构造
let node = Node::new("role")
    .with_prop("name", "precise-coder")
    .with_prop("temperature", 0.3)     // 落为 Double
    .with_prop("max_turns", 40)        // 落为 Int
    .with_prop("skills", Value::Array(vec!["tdd".into(), "brainstorming".into()].into()));

// 写为可回读的 .at 源（转义正确）
let src = node.to_at_source();
// → role { name : "precise-coder"; temperature : 0.3; ... } 形态

// 回读
let parsed = auto_atom::AtomParser::parse(&src).unwrap();
```

### A5.3 宏 DSL

```rust
use auto_lang::{atom, node};

let atom = atom! {
    config {
        version: "1.0",
        debug: true,
    }
};
let arr  = atom![1, 2, 3, 4, 5];
let obj  = atom!{name: "Alice", age: 30};
let node = node! {
    db {
        host: "localhost",
        port: 5432,
    }
};
```

### A5.4 动态求值（Auto 代码 → 静态 Atom）

```rust
use auto_lang::atom::AtomReader;

let mut reader = AtomReader::new();
// 宏体/配置里可以写循环、变量——求值后被"凝聚"为静态节点
let atom = reader.parse(r#"
    for i in 0..3 {
        item(index: i) { label: "row" }
    }
"#).unwrap();
```

对应 AutoGen 模板场景：模板中的 `for s in students { student(name: s.name) {...} }`
求值后产出静态的 `student(name: "Zhang San", age: 10) { course(...) ... }` 树。

### A5.5 类型化访问（设计蓝图，`atom.md` §Auto 与 Atom 的转换）

解析出的 Atom 可按两种方式消费：

1. **带 Schema 的类型化访问**：配合 `type Student { name str; age int; courses []Course }`
   定义，Atom 直接映射为 Rust/Auto 结构体数组，当结构体成员直接访问（serde
   Deserializer 通路已支持反序列化方向）。
2. **动态访问**：类 JS 对象语法——`students.len`、`students[5].courses[3].score`，
   越界索引提前返回 null 而非报错。

## A6. 演进史（计划索引）

| Plan | 主题 | 状态 |
|---|---|---|
| 002–006 | ToAtom AST 测试 / ToNode 重构 / ToAtom→AutoStr / AtomWriter（S 表达式）/ AtomWriter 修正 | ✅ |
| 011 | auto-atom 从原型到生产库（错误处理/查询 API/JSON 支持） | ✅（crate 现 active，独立成 `crates/auto-atom`） |
| 012 | Node 存储 BTreeMap+Vec → IndexMap（O(1) + 保序） | ✅ |
| 013/014 | args/props 统一、body/nodes/kids 统一为 `Kids` | 013 ✅ / 014 ⏳（索引标记） |
| 015 | Atom Builder API（链式 + Builder，~735 LOC/77 测试） | ✅ |
| 016 | 宏 DSL（`value!`/`atom!`/`node!` proc-macro + 插值） | ✅ |
| 139 | 类 Serde 的 Auto↔Atom 自动序列化（编译期合成 to_atom/from_atom） | 设计沉淀 |
| 332 | `#[derive(ToAtom)]`/`#[derive(FromAtom)]` 标注驱动 .at 序列化 | Serialize 方向收官（2026-08-27，S1+S2 完成） |
| 381 | auto-val serde Deserializer 适配器（反序列化零样板） | ✅ |

## A7. 未实现的设计分支（存档备忘）

- **MicroVM Atom**（`docs/design/raw/microvm-atom.md`）：无 OS、无 malloc 的 MCU
  场景下，Node/Obj/Array 统一为"静态池 + 空闲链表"，String Interning 池 +
  `uint16_t` 索引代替指针（`NodeIdx`/`StrIdx`）。纸面设计，未落地。
- **ASTL 扩展**（`docs/design/raw/extending_atom.md`）：把 Atom 格式的 AST 当语言
  处理、加语法糖提升人读性的方向（v0.2 构想）。
- **Link `@ref` 远程句柄**：三支柱之一，序列化蓝图见 `atom-serialize.md` §4。

---

# Part B —— Shell Atom 管线与 Batom 二进制格式

## B1. 设计背景

来源：auto-shell 仓 `docs/plans/old/013-autoshell-warp-design.md`（原 auto-lang
Plan 291，2026-06-10 设计、2026-08-01 归档；Plan 292 落地 Atom 管线、Plan 295 落地
分层架构）。愿景是把 AutoShell 升级为"结构化数据管线（对标 NuShell）+ AI 集成
（对标 Warp 2.0）+ Block UX"的三位一体 Shell，分五个 Phase：

- **P0 Atom 管线**：用 `AtomPipeline(Atom|AtomStream|Text|Empty)` 取代
  `PipelineData(Value|Text)` 作为 shell 一等数据载体，全部命令输出 Atom。✅
- **P1 Batom 二进制**：Atom 的高性能二进制编码，跨命令/跨进程管道传输。✅
- **P2 命令扩展**：18 → 74+ 命令。✅
- **P3 AI 集成 / P4 Block UX**：⏸（后续计划部分覆盖）。

执行策略修正：原计划集成 NuShell `embed-nu`，因该库停留 v0.3.0（NuShell 0.69.x）
与现代版本不兼容，改为**零依赖自实现**（手写 JSON/YAML/TOML/XML/CSV 解析器）。

## B2. Shell Atom 语义层设计（`ash-core/src/pipeline/atom.rs`）

### B2.1 `Atom`：值 + 语义类型标签

```rust
pub struct Atom {
    pub value: Value,          // auto_val::Value —— 实际数据
    pub atom_type: AtomType,   // 语义标签：这份数据"是什么"
}
```

关键思想：下游命令不靠运行时探查 Value 字段猜测数据形状，而是读语义标签决策
（表格渲染、进度条、过滤等）。`AtomType` 现有 **18 种**（设计文档记 21 种，落地
收敛为 18）：

| 分组 | 标签 |
|---|---|
| 文件系统 | `FileEntry`（单条目）、`FileList`（如 `ls` 输出） |
| 进程 | `ProcessEntry`、`ProcessList`（如 `ps` 输出） |
| 系统信息 | `DiskEntry`、`CpuInfo`、`MemoryInfo`、`SystemInfo` |
| 搜索/数据 | `MatchList`、`CountResult` |
| 通用结构化 | `Table`（同构对象数组）、`Record`（单对象） |
| 文本/标量 | `Text`、`Path` |
| 构建/运行 | `BuildResult`、`RunResult` |
| 元信息 | `HelpInfo`、`Nothing`（无类型/未知） |

便利构造器：`Atom::text(...)`、`Atom::file_list(...)`、`Atom::process_list(...)`、
`Atom::path(...)`、`Atom::system_info(...)`、`Atom::nothing(...)`、`Atom::empty()`；
判定方法：`is_structured()`（非 Text/Nothing 即结构化）、`is_empty()`（Nil/Null/
Void）、`as_text()/into_text()`（Str 取原文，其余委托显示格式化）。

另有类型推断引擎 `infer_atom_type`：从 Value 结构反推语义标签（bridge 层
`PipelineData ↔ AtomPipeline` 用）。

### B2.2 `AtomPipeline`：管线载体（`atom_pipeline.rs:21`）

```rust
pub enum AtomPipeline {
    Atom(Atom),                    // 单个类型化值
    Stream(AtomStream),            // 惰性流（内存游标迭代）
    ExternalStream(ExternalStream),// 外部子进程流式输出（I/O 背书，不可序列化）
    Text(String),                  // 纯文本（外部命令/旧兼容）
    Code { spans, language },      // 语法高亮代码 span（结构化 RGB+粗斜体）
    Empty,                         // 无数据
}
```

## B3. Batom 二进制格式（详细设计）

> 实现唯一权威：`ash-core/src/pipeline/batom.rs` 模块头注释 + 代码。以下规范
> 与该实现逐字段核对。

### B3.1 总体布局

```text
┌──────────┬───────────┬──────────────────────────────────┐
│ Magic    │ Header    │ Payload                          │
│ 4 bytes  │ 16 bytes  │ Variable                         │
│ "BATM"   │           │                                  │
└──────────┴───────────┴──────────────────────────────────┘
```

### B3.2 Header（16 字节，小端）

| 偏移 | 类型 | 字段 | 说明 |
|---|---|---|---|
| +0 | u8 | `version` | 当前为 1 |
| +1 | u8 | `flags` | bit 0 = stream（流式编码标记）；其余保留 |
| +2 | u8 | `atom_type_tag` | AtomType 的 u8 编码（单 Atom 时有效；stream 时为 `Nothing`=0x00） |
| +3 | u8 | `_reserved` | 保留 |
| +4 | u32 | `string_table_offset` | 字符串表起始的绝对字节偏移 |
| +8 | u32 | `string_table_entries` | 去重后的字符串条数 |
| +12 | u32 | `payload_size` | 值树区字节数 |

### B3.3 Payload

1. **值树**（紧跟 Header，从偏移 20 开始）：递归标签化编码（§B3.4）。
2. **字符串表**（位于 blob 末尾，`string_table_offset` 指向）：每条
   `[u32 len][u8[len] bytes]`，UTF-8，**编码时全局去重**——字段名与重复值只存
   一次，值树中以 u16 索引引用。

### B3.4 值标签（单字节 tag）

| Tag | 值 | 负载格式 |
|---|---|---|
| `TAG_NIL` | 0x00 | 无 |
| `TAG_VOID` | 0x01 | 无 |
| `TAG_NULL` | 0x02 | 无 |
| `TAG_BOOL` | 0x03 | 1 字节 0/1 |
| `TAG_INT` | 0x04 | i32 LE（4B） |
| `TAG_UINT` | 0x05 | u32 LE（4B） |
| `TAG_I64` | 0x06 | i64 LE（8B） |
| `TAG_FLOAT` | 0x07 | f64 LE（8B） |
| `TAG_ARRAY` | 0x09 | u32 元素数 + 递归元素（`Value::Array`/`Block` 同用） |
| `TAG_OBJ` | 0x0A | u32 条目数 + 每条 (键, 值)；键按 ValueKey 变体编码：Str 键→字符串编码，Int 键→TAG_INT+4B，Bool 键→TAG_BOOL+1B |
| `TAG_RANGE` | 0x0B | 2 × i32 LE |
| `TAG_RANGE_EQ` | 0x0C | 2 × i32 LE |
| `TAG_SOME` | 0x0D | 递归内值 |
| `TAG_NONE` | 0x0E | 无 |
| `TAG_OK` | 0x0F | 递归内值 |
| `TAG_ERR` | 0x10 | 字符串编码 |
| `TAG_PAIR` | 0x11 | u8 键类型（0=Str/1=Int/2=Bool）+ 键 + 递归值 |
| `TAG_GRID` | 0x12 | u32 表头数 + 每条 (键, 值) + u32 行数 + 每行 header_count 个单元格 |
| `TAG_BYTE` | 0x13 | 1 字节（`Value::Byte`/`U8` 同用） |
| `TAG_USIZE` | 0x14 | u64 LE（8B） |
| `TAG_CHAR` | 0x15 | u8 UTF-8 长度 + UTF-8 字节 |
| `TAG_ERROR` | 0x16 | 字符串编码 |
| `TAG_STRING_INLINE` | 0x20 | u16 长度 + 字节（收集期之后才出现的字符串兜底） |
| `TAG_STRING_REF` | 0x21 | u16 字符串表索引 |

字符串族（`Str/String/StrSlice/CStr`）统一走字符串表引用。

**未覆盖变体的兜底**：`Fn/ExtFn/Type/Lambda/Node/Widget/Model/View/Meta/Method/
Instance/Args/Ref/VmRef/ValueRef/Closure/Future`（注释明言"不属于 shell 管线"）
以及实现上顺带落入兜底分支的 `Double/I8` 等，一律编码为 `TAG_NIL`（静默降级）。

**已知的往返损益**：`TAG_OBJ` 解码时非字符串键（Int/Bool）被 `to_string()` 字符串化
后 `obj.set(key_str, ...)`——即 Obj 的 Int 键 `5` 往返后变 Str 键 `"5"`。

### B3.5 AtomType 标签（Header 与流式逐项使用）

| Tag | AtomType | | Tag | AtomType |
|---|---|---|---|---|
| 0x00 | Nothing | | 0x09 | MatchList |
| 0x01 | FileEntry | | 0x0A | CountResult |
| 0x02 | FileList | | 0x0B | Table |
| 0x03 | ProcessEntry | | 0x0C | Record |
| 0x04 | ProcessList | | 0x0D | Text |
| 0x05 | DiskEntry | | 0x0E | Path |
| 0x06 | CpuInfo | | 0x0F | BuildResult |
| 0x07 | MemoryInfo | | 0x10 | RunResult |
| 0x08 | SystemInfo | | 0x11 | HelpInfo |

### B3.6 流式编码（AtomStream）

```text
Magic(4) + Header(16) + u32 atom 数 count + count × [u8 atom_type_tag + 值树] + 字符串表
```

- `flags` bit 0 = 1 标记流；`atom_type_tag` 固定 `Nothing`（流无单一类型）。
- 字符串表对**全部** atom 全局去重（先遍历收集再编码）。
- 逐项前缀类型标签的设计支持增量解码。

### B3.7 设计决策与原始设计的差异

代码明载的决策：**零 serde 依赖**（ash-core 保持轻依赖）、**字符串去重**、
**小端序**（贴合 CPU）、**单趟编码**（先收集字符串再一趟写出）、**聚焦管线类型**
（只编码 shell 管线会出现的 Value 变体）。

对照 Plan 013 原始设计，落地时**刻意简化**掉的部分：schema-inline 类型定义区
（类 FlatBuffers 的 `type_count`/类型表）、可选 LZ4 压缩（flags 位控制）、
`AUTOSHELL_PIPE_FORMAT=batom` 跨进程管道协商——均未实现；Header 字段也从设计稿的
`type_count: u16` 布局改为实现中的 `atom_type_tag: u8 + reserved`。性能目标
（设计稿：1000 条文件信息 45KB→18KB、解析 5x、跨进程 10x）未做正式对标，实测
基准见 §B4.3。

## B4. 具体实现

### B4.1 结构

- **`BatomEncoder`**：两阶段。Phase 1 递归收集字符串（`StringTable`：
  `Vec<String>` + `HashMap<String, u16>` 去重索引）；Phase 2 预留 Magic+Header
  空间后单趟编码值树，随后写字符串表，最后回填 Header（offset/entries/
  payload_size）。可复用（`buf.clear()` + 重建表）。
- **`BatomDecoder<'a>`**：零拷贝游标（`data: &[u8], pos`），先按 Header 解析
  末尾字符串表（`from_utf8_lossy` 容错），再从偏移 20 递归解码值树；流式按
  count 逐项解码。
- **便捷函数**：`encode_atom/decode_atom`、`encode_pipeline/decode_pipeline`、
  `encode_stream/decode_stream`；管线级 `AtomPipeline::to_batom()/from_batom()`。
- **`BatomError`**（thiserror，7 种）：`InvalidMagic`、`UnsupportedVersion`、
  `UnexpectedEof(offset)`、`InvalidTag`、`InvalidAtomType`、
  `StringIndexOutOfRange{index,max}`、`BadStringTableOffset{offset,len}`、
  `PayloadTooLarge`。
- **管线级编码规则**（`encode_pipeline`）：`Atom` → 直编码；`Stream` → 流式；
  `Text` → `Atom::text` 编码；`Code` → spans 拍平为纯文本（颜色元数据不保留）；
  `Empty` → 空 Atom；**`ExternalStream` 不可编码**（持有 I/O 资源，调用方须先
  `.into_text()` 收敛）。

### B4.2 测试覆盖（同文件内 25+ 单测）

原语往返（含 i32/i64/u32/f64 极值、空串、中文、emoji）、复合结构往返（数组/
对象/Range/Some/None/Ok/Err）、嵌套"对象数组"（模拟 ls 输出）、**全部 18 种
AtomType 往返**、管线与流往返、错误路径（坏 Magic、截断数据）、二进制格式断言
（`bytes[0..4] == b"BATM"`、`bytes[4] == 1`）、**去重效果断言**（100 条含重复
字符串的条目编码后 < 2000 字节）。基准测试另在 `ash-core/benches/batom_bench.rs`
（criterion）。

### B4.3 性能（Plan 013 收尾记录）

- 单个 int 编码：~82 ns
- 1000 条文件条目编码：~400 µs
- 去重收益：100 条重复字段条目 < 2 KB（无去重时显著更大）

## B5. 用法

### B5.1 Rust 编码/解码

```rust
use ash_core::pipeline::{batom, Atom, AtomType, AtomPipeline};
use auto_val::Value;

// 单个 Atom
let atom = Atom::new(Value::Int(42), AtomType::CountResult);
let bytes: Vec<u8> = batom::encode_atom(&atom).unwrap();      // Magic "BATM" 开头
let back: Atom = batom::decode_atom(&bytes).unwrap();
assert_eq!(back.atom_type, AtomType::CountResult);

// 管线级（推荐入口）
let pipe = AtomPipeline::text("hello world");
let blob = pipe.to_batom().unwrap();
let restored = AtomPipeline::from_batom(&blob).unwrap();
assert_eq!(restored.as_text(), "hello world");
```

### B5.2 构造结构化数据后走管线

```rust
// 模拟 ls 输出：FileList = 对象数组
let mut entry = auto_val::Obj::new();
entry.set("name", Value::str("test.txt"));
entry.set("size", Value::Int(1024));
let atom = Atom::new(Value::Obj(Box::new(entry)), AtomType::FileEntry);
let bytes = batom::encode_atom(&atom).unwrap();
```

命令实现侧的模式：命令产出 `AtomPipeline::file_list(Value::Array(...))` →
下游命令读 `atom_type()` 决策（表格渲染/过滤）→ 需要跨进程/持久化时 `to_batom()`。

### B5.3 设计蓝图（未实现，存档）

- **跨进程管道**（Plan 013）：`export AUTOSHELL_PIPE_FORMAT=batom` 协商后，
  `ls | batom-encode | ./external-tool | batom-decode | where size > 1024`。
- **`!T` 跨界通讯协议**（`docs/design/raw/result-type.md` §6）：错误类型跨
  `Task + Msg` 消息总线时执行"物化拦截"，序列化引擎按 `type_hash` 把真实错误
  深度反序列化进 BAtom 字节流，接收端重建本地 `!T`——Actor 模型内存隔离的
  设想载体。两处均停留在设计层。

---

## 附录：资料索引

### 设计与规范文档（auto-lang 仓）

| 文件 | 内容 |
|---|---|
| `docs/design/raw/atom.md` | Atom 概念起源、命名、语法总纲、Auto↔Atom 转换 |
| `docs/design/raw/atom-serialize.md` | Atom-Text 物理规范（Node-Centric）、Tag 策略、Link 机制、三支柱 |
| `docs/design/raw/atom-builder-api-design.md` | 构造 API 三层设计（链式/Builder/宏 DSL） |
| `docs/design/raw/extending_atom.md` | ASTL 方向扩展构想（未实现） |
| `docs/design/raw/microvm-atom.md` | MCU 嵌入式 Atom 静态池设计（未实现） |
| `docs/design/raw/result-type.md` §6 | BAtom 协议跨界通讯蓝图（未实现） |
| `docs/design/07-data-structures.md` | Atom Serialization 节（332 语义沉淀） |
| `docs/specs/auto-atom/project.md` | auto-atom crate 模块规格 |
| `docs/specs/auto-lang/frontend/design/atom-serialization.md` | AST 序列化体系规范（trait 三件套） |
| `docs/design/11-shell-tools.md`、`docs/design/ash-design-summary.md` | Plan 291–295 状态总述（Batom=P1） |
| `docs/plan-indices/01-ast-core.md`、`docs/plan-reports/01-ast-core.md` | Plan 001–016 AST/Atom 演进汇总 |

### 计划（auto-lang 仓 `docs/plans/archive/`）

002/004/005/006（ToAtom/AtomWriter）、011（auto-atom 重构）、012（IndexMap）、
015（Builder API）、016（宏 DSL）、139（序列化系统）、332（derive 宏）、
292（autoshell-atom-pipeline）、295（ash 分层架构，含 Batom 基准）。

### auto-shell 仓（Batom 主场）

| 位置 | 内容 |
|---|---|
| `docs/plans/old/013-autoshell-warp-design.md` | 原 Plan 291：Atom 管线 + Batom 二进制完整设计（含未实现的压缩/schema/跨进程） |
| `ash-core/src/pipeline/atom.rs` | Shell Atom + 18 种 AtomType |
| `ash-core/src/pipeline/atom_pipeline.rs` | AtomPipeline 载体 + `to_batom/from_batom` |
| `ash-core/src/pipeline/batom.rs` | **Batom 格式唯一权威实现**（编码器/解码器/25+ 测试） |
| `ash-core/benches/batom_bench.rs` | criterion 基准 |
