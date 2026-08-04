# Plan 381：`Value` 支持 serde `Deserialize` —— 配置子集的 Deserializer 适配器

> **Status**: ✅ 全部完成并归档（2026-08-04）。Phase A-C（ValueDeserializer + Node::deserialize + doctest）+ **Phase D（lenient deserialize_with 辅助：lenient_bool / string_or_list / nonempty_string / lenient_f64）** 已合并 master。811 行 de.rs，165+25 测试通过。auto-ai 的 loader/role_config 迁移现已具备条件（leniency 辅助已就位）。
> **更新（2026-08-04）**:Phase A-C 落地后已迁:auto-os-config `registry.rs`、auto-musk `app_config.rs`(标量)。Phase D 补齐 leniency 辅助,auto-ai 的 loader/role_config 迁移不再被阻塞。
> **类型**: 完整计划(设计 + 实施)
> **日期**: 2026-08-04
> **影响**: `crates/auto-val`(新增 `serde` feature + `ValueDeserializer`)
> **来源**: auto-os-config Plan 003 §8 —— 把模块注册表从 TOML 迁到 auto-atom 时,手写了 `opt_string`/`required_string` 抽字段;同样的模式在 `auto-ai` 的 `role_config.rs`/`loader.rs`/`app_config.rs` 重复 4+ 处。
> **关联**: Plan 332(`#[derive(FromAtom)]`,未实施)—— 本计划是它的**底座**;332 未来可基于 serde 路径构建,不冲突。
> **风险**: 低 —— 纯新增 optional feature,不改现有 API,桥接只读。

---

## 1. 背景与动机

### 1.1 痛点:每个 `.at` 结构都要手写抽字段函数

auto-lang 有完整的 `AtomParser`(解析)和 `AtomSource::to_at_source`(序列化),但把一个 **Rust struct** 从解析出的 `Value`/`Node` 里取出来,目前是逐结构手写:

```rust
// auto-ai/crates/auto-ai-agent/rust-ref/src/config/role_config.rs
cfg.name = opt_string(&node, "name");
cfg.description = opt_string(&node, "description");
cfg.temperature = opt_float(&node, "temperature");
cfg.max_turns = opt_uint(&node, "max_turns");
// … 每个字段一行,每个结构一份函数
```

这套 `opt_string`/`opt_uint`/`opt_float`/`opt_string_list` 模式在 `role_config.rs`、`loader.rs`、`app_config.rs`、`registry.rs` 重复出现。**这是 serde `#[derive(Deserialize)]` 本该消灭的样板**,但因为 `Value` 不实现 `Deserialize`,无法直接用。

### 1.2 目标

给 `auto-val::Value` 加一个 **serde `Deserializer` 适配器**(feature-gated),让任何 `#[derive(Deserialize)]` 的 struct 直接从 `Value`(或 `Node` body)反序列化:

```rust
#[derive(Deserialize)]
struct RoleDecl {
    name: String,
    #[serde(rename = "model_tier")]
    tier: String,
    temperature: Option<f64>,
    skills: Vec<String>,
}
let node = AtomParser::parse(src)?;
let role: RoleDecl = node.deserialize()?;   // ← 一行
```

### 1.3 为什么是 serde 适配器(而非 Plan 332 的自定义 trait)

Plan 332 设计了 `#[derive(FromAtom)]`(自定义 `FromAtom` trait + proc-macro,未实施)。本计划走 **serde `Deserializer` 适配器** 路线:

| 维度 | serde 适配器(本计划) | Plan 332(FromAtom derive) |
|---|---|---|
| 新 trait | ❌ 无 | ✅ `FromAtom`/`ToAtom` |
| 新 crate | ❌ 无(proc-macro 不需要) | ✅ 扩展 `auto-lang-macros` |
| 生态 | ✅ Rust 序列化主流 | 自定义体系 |
| 已有 `#[derive(Deserialize)]` 的 struct | 直接可用 | 要改派生 |
| 定制能力(如 `#[atom(node="role")]`) | serde 的 `rename`/`skip` 够用 | 更针对 .at |

**结论**:本计划是**底座**(快速、通用、低风险);Plan 332 标注为"未来可基于 serde 路径构建"(宏生成 serde 调用而非新 trait)。两者不冲突,本计划先行。

---

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **机制** | 手写 `ValueDeserializer` 实现 `serde::de::Deserializer` | 不能 `#[derive(Deserialize)]`——阻碍见下 |
| **feature gate** | `serde` feature(optional,默认关) | 不强加 serde 依赖给不需要的消费者(VM 核心) |
| **覆盖范围** | 仅静态配置子集:Str/Int/Uint/I64/Double/Bool/Array/Obj/Nil/Null | VM 变体(Fn/Closure/Widget/Grid/VmRef…)在反序列化路径返回 error |
| **AutoStr 阻碍** | 适配器内部 `value.as_str().to_string()` 取 owned `String`,不碰 ecow | `AutoStr = ecow::EcoString`,ecow 无 serde feature;改 ecow 是跨库改动,不值 |
| **Obj key 阻碍** | map 反序列化时用 `ValueKey::name()` 取 `&str`,非 `Str` key 报错 | `Obj` 的 key 是 `ValueKey`(Str/Int/Bool),serde map 要 `String` key |
| **Node 便捷入口** | 加 `Node::deserialize::<T: Deserialize>()` 方法 | 等价于"取 node body 作为 Obj 再反序列化",符合配置读取习惯 |
| **错误类型** | `serde::de::value::Error`(标准) | 不引入新 error 类型 |

### 2.1 为什么不能直接 `#[derive(Deserialize)] enum Value`

三个硬阻碍:
1. `Value::Str(AutoStr)`,`AutoStr = ecow::EcoString`,而 `EcoString` 不实现 `Deserialize`(ecow 无 serde feature)→ 派生编译不过。
2. `Value` 有 ~50 个变体,大量 VM 运行时类型(Fn/Closure/Widget…)持有函数指针/VM 引用,根本无法从外部数据反序列化。
3. `Obj` 的 key 是 `ValueKey` 枚举,不是 `String`,serde map 反序列化不兼容。

因此**手写 Deserializer 适配器**是唯一干净路径——它在 `deserialize_*` 方法里手动从 `Value` 取值,绕开上述所有阻碍。

---

## 3. 实现设计

### 3.1 文件:`crates/auto-val/src/de.rs`(新)

```rust
//! serde Deserializer for the static-config subset of `Value`.
//!
//! Behind the `serde` feature. Lets any `#[derive(Deserialize)]` struct read
//! directly from a parsed `Value`/`Node`, replacing hand-written opt_string/
//! opt_uint field extraction.

use crate::{Value, Obj, Array, ValueKey};
use serde::de::{self, Deserialize, Deserializer as _, IntoDeserializer, Visitor, MapAccess, SeqAccess};
use serde::de::value::Error as DeError;

/// A wrapper that makes `&Value` usable as a `serde::de::Deserializer`.
pub struct ValueDeserializer<'a>(pub &'a Value);

// ---- entry convenience ----
impl Value {
    /// Deserialize a `T` from this value (requires `serde` feature).
    pub fn deserialize_into<'de, T: Deserialize<'de>>(&'de self) -> Result<T, DeError> {
        T::deserialize(ValueDeserializer(self))
    }
}
```

### 3.2 标量映射

| `Value` 变体 | serde 调用 |
|---|---|
| `Str(s)` / `String(_)` / `StrSlice(_)` | `visitor.visit_str(s.as_str())` |
| `Int(i)` / `I8` / `U8` / `Byte` / `Uint` | `visit_i64` / `visit_u64` |
| `I64(i)` | `visit_i64(i)` |
| `Float(f)` / `Double(f)` | `visit_f64(f)` |
| `Bool(b)` | `visit_bool(b)` |
| `Nil` / `Null` / `Void` | `visit_unit()` |
| `Array(a)` / `Block(a)` | `visit_seq(SeqIter(a))` |
| `Obj(o)` | `visit_map(MapIter(o))` |
| 其他(Fn/Closure/Widget…) | `Err(custom("cannot deserialize VM value"))` |

### 3.3 `deserialize_any` 的分派

核心方法。按 `Value` 变体调用对应 `visitor.visit_*`。这是所有 `deserialize_str`/`deserialize_i64`/… 的统一入口(serde 允许 `deserialize_any` 兜底,我们用它简化:所有标量方法都走 `deserialize_any`)。

### 3.4 `MapIter`(Obj → MapAccess)

```rust
struct MapIter<'a> {
    iter: indexmap::map::Iter<'a, ValueKey, Value>,
    current_value: Option<&'a Value>,
}
impl<'a> MapAccess<'a, DeError> for MapIter<'a> {
    fn next_key_seed<K: DeserializeSeed>(&mut self, seed: K) -> Result<Option<K::Value>, DeError> {
        match self.iter.next() {
            Some((k, v)) => {
                let key_str = k.name().ok_or_else(|| de::Error::custom("non-string map key"))?;
                self.current_value = Some(v);
                seed.deserialize(StrDeserializer(key_str)).map(Some)
            }
            None => Ok(None),
        }
    }
    fn next_value_seed<V: DeserializeSeed>(&mut self, seed: V) -> Result<V::Value, DeError> {
        let v = self.current_value.take().expect("next_value_seed before next_key");
        seed.deserialize(ValueDeserializer(v))
    }
}
```
注意:`Obj::iter()` 返回 `indexmap::map::Iter`(保插入序),serde map 不要求排序,正好。

### 3.5 `SeqIter`(Array → SeqAccess)

```rust
struct SeqIter<'a> { iter: std::slice::Iter<'a, Value> }  // Array.values: &[Value]
impl<'a> SeqAccess<'a, DeError> for SeqIter<'a> {
    fn next_element_seed<T: DeserializeSeed>(&mut self, seed: T) -> Result<Option<T::Value>, DeError> {
        match self.iter.next() {
            Some(v) => seed.deserialize(ValueDeserializer(v)).map(Some),
            None => Ok(None),
        }
    }
}
```
检查 `Array` 的字段:`pub struct Array { pub values: Vec<Value> }`,所以 `arr.values.iter()`。

### 3.6 `Node` 便捷入口

`Node` 的 body 是 props + kids。配置读取习惯是"把 node 当成它的 body 对象反序列化"。实现:构造一个临时 `Value::Obj`(合并 props),或直接让 `Node::deserialize` 走 props 的 map。

```rust
impl Node {
    /// Deserialize a `T` from this node's body (props + kids treated as a map).
    pub fn deserialize<'de, T: Deserialize<'de>>(&'de self) -> Result<T, DeError> {
        // 把 props 收集成 Obj,作为 map 喂给 serde。
        // kids(命名子块)如果也要参与,递归为 Value::Node —— 但配置反序列化
        // 通常只取标量 props,嵌套块走单独的 struct + 手动 resolve。
        // v1: 只反序列化 props(覆盖 role_config 的 opt_* 用例)。
        todo!()
    }
}
```
**v1 范围**:`Node::deserialize` 只处理 props(标量字段)。嵌套子块(tier_routing、provider 块)的反序列化留给 v2(需要 struct 能声明"这个字段从名为 X 的子节点取")。这覆盖了 `role_config.rs` 的全部用例(角色字段全是标量/数组)。

### 3.7 feature gate + Cargo.toml

`crates/auto-val/Cargo.toml`:
```toml
[features]
default = []
serde = ["dep:serde", "serde/indexmap"]   # indexmap: Obj 用了 indexmap

[dependencies]
serde = { workspace = true, optional = true }
# 其余不变
```
`src/lib.rs`:
```rust
#[cfg(feature = "serde")]
mod de;
#[cfg(feature = "serde")]
pub use de::ValueDeserializer;
```

---

## 4. 实施路线(分 Phase)

### Phase A — `ValueDeserializer` 标量 + Obj + Array
- 新增 `crates/auto-val/src/de.rs`,`ValueDeserializer` 实现完整 `serde::de::Deserializer` trait。
- 标量方法(deserialize_str/i64/u64/f64/bool/unit)+ `deserialize_any` + `deserialize_seq` + `deserialize_map`。
- `MapIter`、`SeqIter`、`StrDeserializer`(key 用)。
- `Value::deserialize_into<T>()` 便捷方法。
- Cargo.toml + lib.rs feature gate。
- **验证**:单测——`#[derive(Deserialize)] struct { a: String, b: i32, c: Option<f64>, d: Vec<String> }`,从手构的 `Value::Obj` 反序列化,字段值正确;缺失字段 + `Option` → None;类型不匹配 → error。

### Phase B — `Node::deserialize<T>()` 便捷入口
- 把 node 的 props 收集成 map 视图喂给 serde(v1 只 props,不含 kids)。
- **验证**:从真实 `.at` 文件 `AtomParser::parse` → `node.deserialize::<RoleDecl>()`,字段值与 `role_config.rs` 手写解析一致。

### Phase C — 集成验证 + 文档
- 在 `auto-val` 顶层加一个 doctest 展示完整用法。
- 更新 `crates/auto-val/README`(若有)或模块文档,说明 `serde` feature。
- 标注 Plan 332 为"未来基于本计划的 serde 路径"。

---

## 5. 验证清单

- [ ] `ValueDeserializer` 实现 `serde::de::Deserializer` 全部必需方法(编译通过)。
- [ ] 标量:String/i32/u32/i64/f64/bool/unit 全覆盖,round-trip 正确。
- [ ] `Option<T>`:缺失 → None,存在 → Some。
- [ ] `Vec<T>`:`Value::Array` → Vec,元素递归反序列化。
- [ ] 嵌套 struct:`Value::Obj` 里的 `Value::Obj` 字段递归。
- [ ] enum(serde `#[serde(rename_all)]`):bare ident `info` ↔ `"info"` 一致(因为 parser 已把 bare ident 变 `Value::Str`)。
- [ ] 错误:类型不匹配、缺 required 字段、VM 变体,都有清晰 error message。
- [ ] feature 关闭时:`auto-val` 不依赖 serde(现有消费者无影响)。
- [ ] 从真实 `.at`(role/daemon config)反序列化,与手写解析结果逐字段一致。

---

## 6. 不在范围(v1)
- ⏸ `Node` kids(命名子块)的反序列化(v1 只 props)—— Phase D 的 `lenient_*` 辅助不解决这个问题;musk 的 harness 子节点仍手写。
- ⏸ `Serialize` 方向(`Value` ← struct)—— Plan 332 的 `ToAtom` 或单独的 `ValueSerializer`。
- ⏸ 改 ecow 加 serde feature(用适配器绕开,不碰第三方库)。
- ⏸ Plan 332 的 `#[derive(FromAtom)]` 宏(标注为未来基于本计划)。

---

## 7. 衍生迁移(部分已完成)

- ✅ `auto-os-config/backend/src/registry.rs`:**已迁**(commit c0ec7f8)—— `parse_module_node` + `opt_string`/`required_string` → `#[derive(Deserialize)]` + `node.deserialize()`。22 单测通过。
- ✅ `auto-musk/.../app_config.rs`:**已迁标量**(commit e56649e)—— 5 个标量字段走 serde;`harness` 子节点保留手写(v1 不支持 kids)。4 测试通过。
- ⏸ `auto-ai` 的 `role_config.rs`/`loader.rs`/`workflow.rs`:**未迁**—— leniency 分歧(见 Phase D)。这是 Phase D 的目标消费者。

---

## 8. Phase D:容错 `deserialize_with` 辅助(为 auto-ai 迁移铺路)

> **状态**:待实施
> **动机**:Phase A-C 落地后,auto-os-config/musk 的干净站点已迁。但 auto-ai 的 `loader.rs`/`role_config.rs`/`workflow.rs` 仍是手写 `opt_*`,因为它们有**6 份行为各异的 `opt_*` 副本**。直接用默认 serde 会改变这些既有配置的接受行为(回归风险)。Phase D 提供一组 `#[serde(deserialize_with)]` 辅助函数,**精确复刻**现有 leniency,让 auto-ai 能无行为变化地迁移。

### 8.1 要加的辅助(基于实测的现有语义)

每个辅助是 `crates/auto-val/src/de.rs` 里的 `pub fn`,签名符合 serde 的 `deserialize_with` 约定(`fn<'de, D>(deserializer: D) -> Result<T, D::Error>`)。

| 辅助 | 复刻自 | 行为 |
|---|---|---|
| `lenient_bool` | `loader.rs` `opt_bool` | `Bool` 直接用;`0`/`1`(Int/Uint)→ false/true;字符串 `"true"/"yes"/"1"/"on"` ↔ `"false"/"no"/"0"/"off"`;其余 → error(不是 None,因为是字段级 deserialize_with) |
| `string_or_list` | `role_config.rs` `opt_string_list` | `Array<Str>` → Vec;单个 `Str` → 1 元素 Vec;Nil/缺失 → 空 Vec(default) |
| `nonempty_string` | `registry.rs` `opt_string` | `Str` → Some(非空时)/ None(空串时);Nil → None。用于"空串等价于未设"的字段 |
| `lenient_f64` | `role_config.rs` `opt_float` | `Float`/`Double` → f64;`Int`/`Uint` → as f64(整数当浮点) |

> `tier` 解析(`parse_tier`,tier 名 ↔ `ModelTier` 枚举,未知→Mid default)属于 auto-ai 的领域逻辑,不放 auto-val。迁移时在 auto-ai 侧用 `#[serde(deserialize_with = "parse_tier_de")]` 本地辅助。

### 8.2 设计要点
- 放 `de.rs` 里(feature-gated,`#[cfg(feature = "serde")]`)。
- 签名是 serde 标准的 `fn<'de, D: Deserializer<'de>>(D) -> Result<T, D::Error>`——它们消费 `Deserializer`,内部用 `deserialize_any` 拿到 `Value` 再做 lenient 判断。
- 但 `ValueDeserializer` 的 `deserialize_any` 已经按 `Value` 变体分派了;`deserialize_with` 辅助需要在**拿到原始 Value**后做自定义判断。实现方式:辅助内部调 `deserializer.deserialize_any(ValueVisitor)`,其中 `ValueVisitor` 把任意 `Value` 收集成一个临时枚举,再 lenient 转换。或者更简单:辅助直接调 `Value::deserialize_any` 收 `serde_json::Value` 风格的中间态——但这绕了。

  **更干净的实现**:加一个 `ValueCollector` visitor,`deserialize_any` 把 `Value` 原样回收(`visit_str`/`visit_i64`/...重建 `Value`),辅助拿到 `Value` 后 match 做判断。这复用现有分派,不绕路。

### 8.3 验证
- 每个辅助加单测:覆盖它声称接受的每种输入形状 + 拒绝的形状。
- 关键回归测试:用 auto-ai 真实的 `.at` 片段(`auth_required : yes`、`models : "a,b,c"`、`tier : mid`),确认辅助的输出与现有 `opt_*` 逐字段一致。
- 默认 feature 配置不拉新依赖、不破坏现有 160 测试。

### 8.4 不在 Phase D 范围
- ⏸ `Node` kids 反序列化(独立问题,musk harness 仍手写)。
- ⏸ `opt_models` 的 3 形状(Obj 数组/Str 数组/逗号串)——太特化,留在 auto-ai 本地辅助。
- ⏸ 实际迁移 auto-ai 站点(Phase D 只铺路;迁移是 auto-ai 侧的独立工作)。
