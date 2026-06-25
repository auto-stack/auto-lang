# Plan 332: `#[derive(ToAtom)]` / `#[derive(FromAtom)]` — 标注驱动的 .at 序列化

> **类型**:完整计划(设计 + 实施)
> **状态**:设计草案,待评审
> **日期**:2026-06-25
> **前身**:[004-agent-roles-profession-upgrade.md](../../../auto-ai/plans/004-agent-roles-profession-upgrade.md)(Plan 004 引入了手写 `serialize_at_role`,并补齐了 `auto-val/emit.rs` 发射器,本计划在其上抽象出派生宏)
> **关联**:331(autoui-vue-widgets)、auto-ai Plan 004
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行。

---

# 第一部分:设计

## 1. 背景与动机

### 1.1 现状:每个 .at 结构都要手写一对读写函数

auto-lang 的 `.at`(Atom)格式有完整的**解析器**(`AtomParser::parse`),Plan 004 又补齐了
**escape-correct 发射器**(`auto-val/emit.rs` 的 `AtomSource::to_at_source()`)。但把一个
**Rust struct** 与 `.at` 之间互转,目前仍是逐结构手写:

- `ProfessionConfig` ↔ `parse_at_profession` / `serialize_at_role`(手写,~120 行)
- `AgentMode` ↔ `parse_mode_at`(手写)
- `WorkflowStep` ↔ `parse_at_workflow`(手写)

每个结构都要:(a) 手写一个读 prop 的解析函数,(b) 手写一个 set_prop 的序列化函数,
(c) 维护字段名/类型/Option 语义的一致性。**这是典型的可被 derive 消除的样板**。

### 1.2 目标

提供 `#[derive(ToAtom)]` 和 `#[derive(FromAtom)]`,让一个普通 Rust struct
**零样板**地与 `.at` 互转,语义对标 Serde 的 `#[derive(Serialize)]` / `#[derive(Deserialize)]`:

```rust
#[derive(ToAtom, FromAtom)]
#[atom(node = "role")]          // 根节点名(序列化为 `role { ... }`)
struct RoleConfig {
    name: String,
    #[atom(rename = "model_tier")]   // 字段名 → .at 键名
    tier: ModelTier,                  // 自定义类型(需 ToAtomValue/FromAtomValue)
    temperature: f64,
    #[atom(skip)]                     // 跳过该字段
    internal_flag: bool,
    skills: Vec<String>,              // → [a, b, c]
    token_budget: Option<u64>,        // None → 省略该 prop
    description: Option<String>,
}

// 生成:
impl ToAtom for RoleConfig { ... }   // fn to_atom_node(&self) -> Node
impl FromAtom for RoleConfig { ... } // fn from_atom_node(&Node) -> Result<Self>
```

### 1.3 为什么必须是 Rust proc-macro(不能靠 Auto 的编译期能力)

Plan 004 §0 已调研确认:**Auto 的 comptime/VM/反射只能作用于 Auto 代码,不能反射 Rust struct**。
`ProfessionConfig` 住在 Rust 源码里,Auto 的所有执行入口只吃 `.at`/Auto 源码。仓库里唯一的
Rust→Auto 路径(`trans/r2a.rs`)是用 `syn` 解析 Rust 的——那就是 Rust 生态的解析器,
与 Auto 的能力无关。**因此对标 Serde 的派生,只能用 Rust proc-macro + `syn`。**

本计划放 `auto-lang-macros` crate(已是 proc-macro crate,含 `syn`/`quote`/`proc-macro2`,
且已依赖 `auto-val`)。

---

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **宏 crate 选址** | `auto-lang-macros`(扩展现有 proc-macro crate) | 已有 syn/quote/proc-macro2 依赖 + 已依赖 auto-val,零新增基建 |
| **底层发射** | 复用 Plan 004 的 `auto-val::emit`(`Node::to_at_source`) | 已 escape-correct + 有 round-trip 测试,不重造 |
| **底层解析** | 生成调用现有 `Node::get_prop_of` 的代码 | 复用 auto-val 的 Node API,不引入 parser 到宏 |
| **trait 形态** | `ToAtom`/`FromAtom`(新 trait,放 auto-val),生成 `to_atom_node`/`from_atom_node` | 与 `AtomSource`(值→源码)正交:这是 struct↔Node 的结构化层 |
| **字段标注** | `#[atom(node=, rename=, skip, default)]`(对标 serde rename/skip) | 最小标注集覆盖现有手写需求 |
| **Option 语义** | `Option<T>`:序列化 None→省略 prop;反序列化缺失→None | 与现有 ProfessionConfig 的"全是 Option、None 即未设"一致 |
| **Vec 语义** | `Vec<T>` → `Value::Array`;空 Vec 默认省略(可 `#[atom(skip_serializing_if_empty)]` 等价) | 匹配现有 serialize_at_role 行为 |
| **自定义类型** | 提供 `ToAtomValue`/`FromAtomValue` trait(标量↔Value),为 ModelTier 等实现 | 比 Serde 的整 trait 切分更简单,够用 |
| **根节点名** | `#[atom(node = "role")]` 必填(无默认) | .at 是 `name { ... }` 结构,根名必须有 |
| **错误处理** | `FromAtom` 返回 `Result<Self, AtomDeError>` | 缺必填字段/类型不符要可恢复 |

---

## 3. 标注规格

### 3.1 容器标注(`#[atom(...)]` on struct)

| 标注 | 值 | 说明 |
|---|---|---|
| `node = "name"` | 字符串字面量 | **必填**。序列化的根节点名,如 `role`、`mode`、`profession` |
| `legacy_node = "oldname"` | 字符串字面量 | 可选。反序列化时也接受旧节点名(如 Plan 004 的 `profession`→`role` 兼容) |

### 3.2 字段标注(`#[atom(...)]` on field)

| 标注 | 说明 | 默认 |
|---|---|---|
| `rename = "key"` | .at 中的 prop 键名 | 用 Rust 字段名 |
| `skip` | 该字段不参与序列化/反序列化 | — |
| `default` | 反序列化缺失时用 `Default::default()` | 缺失→Error(非 Option 字段) |
| `node` | bool(默认 false);true 表示该字段是子节点而非 prop(嵌套 `child { }`) | false(prop) |

### 3.3 类型映射

| Rust 类型 | .at Value |
|---|---|
| `String`, `&str` | `Str` |
| `bool` | `Bool` |
| `i32/u32/i64/u64/f64/f32` | `Int`/`Uint`/`Double` |
| `Option<T>` | None→省略;Some(v)→T 的值 |
| `Vec<T>` | `Array`(空→默认省略) |
| 任意 `T: ToAtomValue` | 该类型的 `to_value()` |

---

## 4. 架构分层

```
┌─────────────────────────────────────────────────────┐
│  应用 struct(如 auto-ai 的 RoleConfig)            │
│  #[derive(ToAtom, FromAtom)] #[atom(node="role")]   │
└───────────────┬─────────────────────────────────────┘
                │ 宏展开生成
┌───────────────▼─────────────────────────────────────┐
│  auto-lang-macros(本计划)                          │
│  #[proc_macro_derive(ToAtom)] → 生成 impl           │
│  #[proc_macro_derive(FromAtom)] → 生成 impl         │
│  生成代码调用:auto-val 的 Node API + trait         │
└───────────────┬─────────────────────────────────────┘
                │ 依赖
┌───────────────▼─────────────────────────────────────┐
│  auto-val(Plan 004 已有 emit.rs)                   │
│  trait ToAtom { fn to_atom_node(&self)->Node }      │  ← 本计划新增
│  trait FromAtom { fn from_atom_node(&Node)->Result }│  ← 本计划新增
│  trait ToAtomValue { fn to_value(&self)->Value }    │  ← 本计划新增(标量)
│  trait FromAtomValue { fn from_value(&Value)->Self }│  ← 本计划新增(标量)
│  Node::to_at_source()(emit.rs,已有)                │
└─────────────────────────────────────────────────────┘
```

---

# 第二部分:实施

## 5. 实施阶段

### Phase A — auto-val:新增 4 个 trait + 标量实现

**交付**:trait 定义 + 基础类型实现 + 测试。

1. `auto-val/src/atom_de.rs`(新文件):
   - `pub trait ToAtom { fn to_atom_node(&self) -> Node; }`
   - `pub trait FromAtom: Sized { fn from_atom_node(node: &Node) -> Result<Self, AtomDeError>; }`
   - `pub trait ToAtomValue { fn to_value(&self) -> Value; }`(标量,复用 `ToAutoValue` 或独立)
   - `pub trait FromAtomValue: Sized { fn from_value(v: &Value) -> Result<Self, AtomDeError>; }`
   - `#[derive(Debug, thiserror::Error)] pub enum AtomDeError { MissingField, WrongType, ... }`
2. 为标量实现 `ToAtomValue`/`FromAtomValue`:`String`/`bool`/`i32`/`u32`/`i64`/`u64`/`f64`/`f32`。
   (`Vec<T> where T: ToAtomValue` 也实现 → Value::Array。)
3. `lib.rs` 导出。
4. 单测:标量往返。

**验证**:`cargo test -p auto-val atom_de`。

### Phase B — auto-lang-macros:`#[derive(ToAtom)]`

**交付**:序列化派生宏。

1. `crates/auto-lang-macros/src/derive_atom.rs`(新文件):
   - `pub fn derive_to_atom(input: TokenStream) -> TokenStream`
   - 解析 `#[atom(node = "name")]`、字段 `#[atom(rename=, skip, default)]`
   - 生成:`impl ToAtom for #Struct { fn to_atom_node(&self) -> Node { let mut n = Node::new(#node_name); if !field.is_skip() { n.set_prop(#key, field.to_value()); } ... n } }`
   - Option 字段:`if let Some(v) = &self.field { n.set_prop(...) }`
   - Vec 字段:`if !self.field.is_empty() { n.set_prop(...) }`
2. `lib.rs`:`#[proc_macro_derive(ToAtom, attributes(atom))] pub fn derive_to_atom(...)`
3. 单测(在 auto-val 或独立 test crate):
   ```rust
   #[derive(ToAtom)]
   #[atom(node = "role")]
   struct R { name: String, temp: f64, budget: Option<u64> }
   // → to_atom_node() 产出含 name/temp 的 Node,budget=None 时省略
   ```

**验证**:编译 + 断言生成的 Node 的 props。

### Phase C — auto-lang-macros:`#[derive(FromAtom)]`

**交付**:反序列化派生宏。

1. `derive_atom.rs`:`pub fn derive_from_atom(input) -> TokenStream`
   - 生成:`impl FromAtom for #Struct { fn from_atom_node(n: &Node) -> Result<Self, AtomDeError> { Ok(Self { field: FromAtomValue::from_value(&n.get_prop_of(#key))?, ... }) } }`
   - Option 字段:缺失→None(不报错)
   - `#[atom(default)]`:缺失→`Default::default()`
   - 必填字段缺失→`Err(AtomDeError::MissingField)`
   - `legacy_node`:接受旧节点名
2. `lib.rs`:`#[proc_macro_derive(FromAtom, attributes(atom))]`
3. round-trip 测试:`#[derive(ToAtom, FromAtom)]` → `to_atom_node` → `from_atom_node` → 相等。

**验证**:round-trip 测试 + 缺失字段报错测试。

### Phase D — auto-ai:ProfessionConfig 迁移到 derive(验证 + 清理)

**交付**:用 derive 替换 Plan 004 的手写 `serialize_at_role` + `parse_at_profession` 的样板部分。

1. `ProfessionConfig` 加 `#[derive(ToAtom, FromAtom)] #[atom(node="role", legacy_node="profession")]`
2. 为 `ModelTier` 实现 `ToAtomValue`/`FromAtomValue`(序列化为 `"max"` 等)。
3. 特殊处理(`inherit` 合并、`system_prompt_append` 累加、`tools`/`tools_append` 替换/追加)
   保留在 `load_profession`/`merge_over` 手写层(这些是**业务语义**,不是纯序列化)。
   derive 只负责**纯字段往返**;继承/合并逻辑仍手写调用 derive 生成的 `from_atom_node`。
4. 删除 `serialize_at_role`(改为 `cfg.to_atom_node().to_at_source()`)和
   `parse_at_profession` 的逐字段读取(改为 `ProfessionConfig::from_atom_node(&node)`)。
5. 确保所有现有测试通过(这是回归保险)。

**验证**:`cargo test -p auto-ai-agent`(92 个测试不回归)+ 新 round-trip。

### Phase E — 文档 + 示例

1. `docs/` 下加 `atom-derive-guide.md`:标注清单 + 完整示例。
2. 在 auto-ai 的 Plan 004 README 里更新引用(指向本计划的 derive)。

---

## 6. 验证检查点

| 阶段 | 仓库 | 验证 | 预期 |
|---|---|---|---|
| A | auto-lang | `cargo test -p auto-val atom_de` | 标量往返通过 |
| B | auto-lang | `cargo build -p auto-lang-macros` + test crate | ToAtom 生成正确 Node |
| C | auto-lang | round-trip test | ToAtom→FromAtom 等值 |
| D | auto-ai | `cargo test -p auto-ai-agent` | 92 测试不回归 |
| E | — | docs 检查 | 示例可编译 |

---

## 7. 范围与非目标

### 在范围内
- `ToAtom`/`FromAtom` 派生 + `ToAtomValue`/`FromAtomValue` 标量 trait
- 标注:`node`/`legacy_node`/`rename`/`skip`/`default`
- Option/Vec/标量/自定义类型(实现 trait)的映射
- 迁移 `ProfessionConfig` 作为首个 dogfood

### 非目标(v1)
- **Enum 派生**(.at 的 enum 表示待定,留 v2)
- **`#[atom(flatten)]`**(扁平化嵌套,Serde 有但 .at 暂不需要)
- **向前兼容的 schema 演进**(版本号等)
- 迁移 `AgentMode`/`WorkflowStep`(作为后续,验证 derive 成熟后再做)
- 流式/增量解析

---

## 8. 风险与缓解

| 风险 | 缓解 |
|---|---|
| proc-macro 错误拖垮整个 workspace 编译 | Phase B/C 先在独立 test crate 验证;Phase D 迁移前确保 round-trip 测试覆盖 ProfessionConfig 全字段 |
| `ModelTier` 等自定义类型需手动实现 trait | 提供 `ToAtomValue`/`FromAtomValue`(标量级,简单);未来可加 `#[derive(ToAtomValue)]` 但 v1 手写够用 |
| 与现有手写 `load_profession` 的 inherit/merge 语义冲突 | 明确分层:derive 只做纯字段往返;inherit/append/replace 业务逻辑保留在手写层,调用 derive 生成的底层 |
| emit.rs 的转义在 derive 路径下未被覆盖 | Plan 004 已有 round-trip 测试;Phase D 迁移后 auto-ai 的测试会间接覆盖 derive+emit 全链路 |

---

## 9. 后续(本计划之外)

- `#[derive(ToAtomValue)]`:让 `ModelTier` 等自定义标量也零样板(目前手写 trait 实现)。
- 迁移 `AgentMode`、`WorkflowStep` 到 derive(等 ProfessionConfig 迁移稳定后)。
- .at 的 enum 表示 + `#[derive(ToAtom)]` for enum。
