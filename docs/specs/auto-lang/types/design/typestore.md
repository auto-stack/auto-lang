# TypeStore：统一类型存储

## 范围

`types.rs:TypeStore` 的数据结构、共享协议、泛型模板与别名机制、
`infer/registry.rs` 的废弃迁移状态。

## 原则

单一数据源（ADR-02，plan-084）：所有类型/函数/spec/enum 声明集中于一个 `TypeStore`，
parser、codegen、推断上下文共享同一实例。声明体用 `Rc<T>` 共享不可变引用，
跨组件经 `Arc<RwLock<TypeStore>>` 访问（infer/context.rs:73）。

## 细节

### 数据结构（types.rs:133）

```rust
pub struct TypeStore {
    type_decls: HashMap<AutoStr, Rc<TypeDecl>>,
    fn_decls: HashMap<Name, Fn>,
    spec_decls: HashMap<AutoStr, SpecDecl>,
    rust_types: HashSet<String>,            // Plan 190: use.rust 导入
    rust_type_paths: HashMap<String, String>,
    generic_templates: HashMap<String, GenericTemplate>,
    type_aliases: HashMap<AutoStr, AutoStr>, // Plan 090
    enum_decls: HashMap<AutoStr, Rc<EnumDecl>>,
}
```

### 协议与不变量

- `register_type_decl` 对带泛型参数的类型自动登记 `GenericTemplate`（但 `param_types`
  留空、标 TODO——模板替换尚未真正实现，`create_generic_instance` 只做包装不替换）。
- `register_ext_methods`：ext 块方法并入目标类型的 TypeDecl；目标未注册时创建
  placeholder TypeDecl，方法同时登记进 fn_decls 供 import_items 查找。
- 模块导入两式（Plan 085）：`merge` 全量合并（同名覆盖，enum 用 or_insert 不覆盖）、
  `import_items` 选择性导入。
- 别名：`resolve_type_alias` 递归解析直到真实类型；`find_type_for_name` 先解别名再查声明。
- `is_type` 统一判定 type/enum/spec 三类命名空间。
- enum 变体值缺省取索引（`get_enum_variant_value`），支持按变体名全局反查（Plan 127）。

### 废弃注册表的迁移状态

| 旧位置 | 状态 |
|---|---|
| `type_registry.rs`（REPL 持久化） | 仍存在，内部委托/并存于 TypeStore |
| `infer/registry.rs:TypeRegistry` | 头注释标 DEPRECATED，但仍被 `type_registry.rs`、`parser.rs`、`vm/codegen.rs`、`autovm_persistent.rs` 引用 |
| `Database.type_info_store` | 只剩方法名级别的残缺数据，由 TypeStore 取代 |

即"合并"在数据结构上完成，调用方迁移未收尾（design/02 Open Question 仍成立）。

## 显式非目标

- TypeStore 不做类型推导（推导在 `infer/`，经 `type_env` 读写）。
- 泛型模板替换（substitute）未实现——`create_generic_instance` 返回的
  `GenericInstance` 不做字段类型替换。
- 不做跨 crate 序列化；REPL 持久化仍走 `type_registry.rs`。

> 来源: docs/design/02-type-system.md（§TypeStore Unification）、crates/auto-lang/src/types.rs、infer/registry.rs、infer/context.rs
