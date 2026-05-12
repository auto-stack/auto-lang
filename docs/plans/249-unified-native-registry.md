# Plan: Unify Dual Registry — Single-Registration Architecture

## Context

Auto VM 有两套独立的 native 函数注册系统：
- **BIGVM_NATIVES** (`native_registry.rs`): 编译期全局 `AutoVMNativeRegistry`，存 ID 和返回类型。`register_builtin_natives()` 中 ~505 次 `register_with_id` 调用。
- **NativeInterface** (`native.rs`): 运行时 shim 绑定，存函数指针和 name→ID 映射。`register_std_shims()` 中 ~222 次 `register` + ~140 次 `register_name` 调用。

同一个函数（如 `auto.list.len`）需要：定义常量(1) → 注册到 BIGVM_NATIVES(2) → 绑定 shim 函数(3) → 注册名称(4) → 在 codegen dispatch 中列出(5) → 在 engine CALL_SPEC dispatch 中列出(6) → 声明返回类型(7)。总计 7 处。

此外，opaque 方法的 dispatch 表在 `codegen.rs` 中重复 4 次、`engine.rs` 中重复 1 次，共 5 份相同数据。

目标：**每个函数只定义一次**，通过宏自动生成所有注册点。

## 方案：Callback 宏 + 统一 Catalog

### 新文件：`native_catalog.rs`

定义一个 `for_each_native!` 回调宏，包含所有 ~360 个 native 函数的完整元数据：

```rust
macro_rules! for_each_native {
    ($mac:ident) => {
        $mac! {
            // (id, canonical_name, shim_fn, ret_type, aliases)
            (100, "auto.print", shim_print, Unit, &["auto.lang.print"]),
            (101, "auto.println", shim_println, Unit, &[]),
            // ... ~360 entries
            (2200, "auto.list.len", shim_list_len, Int, &[]),
            // ...
        }
    };
}
```

### 三个消费者宏

**1. `gen_constants!`** — 替换 ~233 个 `NATIVE_*` 常量：
```rust
macro_rules! gen_constants {
    (($id:expr, $name:expr, $fn:expr, $ret:expr, $aliases:expr) $(, $rest:tt)*) => {
        pub const NATIVE_FUNC: u16 = $id;
        gen_constants!($($rest),*);
    };
    () => {};
}
for_each_native!(gen_constants);
```

**2. `gen_registry!`** — 替换 `register_builtin_natives()` 中 ~505 次调用：
```rust
macro_rules! gen_registry {
    (($id:expr, $name:expr, $fn:expr, $ret:expr, $aliases:expr) $(, $rest:tt)*) => {
        registry.register_with_id_and_type($name, $id, $ret);
        $(&registry.register_with_id($aliases, $id);)*
        gen_registry!($($rest),*);
    };
    () => {};
}
```

**3. `gen_shims!`** — 替换 `register_std_shims()` 中 ~222+140 次调用：
```rust
macro_rules! gen_shims {
    (($id:expr, $name:expr, $fn:expr, $ret:expr, $aliases:expr) $(, $rest:tt)*) => {
        iface.register($id, $fn);
        iface.register_name($name, $id);
        $(&iface.register_name($alias, $id);)*
        gen_shims!($($rest),*);
    };
    () => {};
}
```

### Opaque Dispatch Table 合并

将 `codegen.rs` 中 4 处 + `engine.rs` 中 1 处的 opaque 方法 dispatch match 块合并为一个静态查找表：

```rust
// native_catalog.rs
pub static OPAQUE_DISPATCH: &[(&str, &str)] = &[
    ("auto.url_opaque", &[
        ("scheme", "auto.url_opaque.scheme"),
        ("host", "auto.url_opaque.host_str"),
        ("path", "auto.url_opaque.path"),
        // ...
    ]),
    ("auto.http.response", &[
        ("status_code", "auto.http.response_status_code"),
        // ...
    ]),
];
```

codegen.rs 和 engine.rs 中各保留一个 `lookup_opaque_dispatch(type_name, method)` 函数调用此表。

## 改动文件

| 文件 | 改动 |
|------|------|
| `vm/native_catalog.rs` | **新建**：`for_each_native!` 宏 + `OPAQUE_DISPATCH` 表 |
| `vm/native.rs` | 删除 ~233 常量，删除 `register_std_shims` 中 ~362 次调用，改为 `gen_constants!` + `gen_shims!` |
| `vm/native_registry.rs` | 删除 `register_builtin_natives` 中 ~505 次调用，改为 `gen_registry!` |
| `vm/codegen.rs` | 4 处 opaque dispatch match 块替换为 `OPAQUE_DISPATCH` 查找 |
| `vm/engine.rs` | 1 处 opaque dispatch match 块替换为 `OPAQUE_DISPATCH` 查找 |
| `vm/mod.rs` | 添加 `mod native_catalog` |

## 其他模块的重复注册问题

| 模块 | 重复情况 | 严重程度 |
|------|---------|---------|
| Opaque method dispatch | codegen.rs x4 + engine.rs x1 = 5 份 | 高 — 本次合并 |
| Type canonical map | `native_registry.rs` TYPE_CANONICAL_MAP | 低 — 仅 5 条映射，保持现状 |
| Return type tracking | `native_registry.rs` NativeRetType + `codegen.rs` infer_call_spec_return_type | 中 — catalog 宏会统一 ret_type 字段 |
| Codegen stdlib module list | `codegen.rs` 两处 `matches!(obj_name, "env"\|"fs"\|...)` | 低 — 仅 2 处，可提取为常量 |

## 迁移策略：按类别分批

### Phase 1: 基础设施 ✅ DONE
- ✅ 创建 `native_catalog.rs`，定义宏框架（186 条目）
- ✅ 在 `vm.rs` 中添加 `pub mod native_catalog;`
- ✅ 定义 `gen_native_constants!` 消费者宏
- ✅ 定义 `bind_shims!` 消费者宏（备用）
- ✅ 替换 native.rs 中 222 个 `pub const NATIVE_*` 常量为宏调用
- ✅ 保留 11 个手动常量（通过 register_shim_by_name 注册的函数）
- ✅ 在 lib.rs 增加 `#![recursion_limit = "512"]`
- ✅ `cargo build --bin auto` 编译通过
- ✅ `cargo test -p auto-lang --lib` 单元测试通过（3276 passed, 与基线一致）
- ✅ 冒烟测试通过（List/Map/String/Math/URL）

**实际 entry 格式**：`(ID, CONST_NAME, shim_fn)` — 三元组，不含 canonical_name 和 ret_type。
后续 Phase 会扩展 entry 格式以支持 BIGVM 注册和 name 绑定。

**未纳入 catalog 的 11 个手动常量**：
- NATIVE_STR_CONTAINS(1504), NATIVE_STR_STARTS_WITH(1505), NATIVE_STR_ENDS_WITH(1506), NATIVE_STR_TO_INT(1516)
- NATIVE_MATH_ABS(1700), NATIVE_MATH_MIN(1701), NATIVE_MATH_MAX(1702), NATIVE_MATH_SQRT(1750)
- NATIVE_MATH_MIN_F(1714), NATIVE_MATH_MAX_F(1715), NATIVE_MATH_CLAMP(1725)

### Phase 2: Shim 绑定自动化（待做）
- 将 `bind_shims!` 接入 `register_std_shims()` 中，替换 ~222 次 `self.register()` 调用
- `register_name()` 调用暂保留手动（别名映射复杂度高）

### Phase 3: BIGVM 注册自动化（待做）
- 扩展 catalog entry 格式，加入 `canonical_name` 和 `ret_type`
- 将 `register_in_bigvm!` 接入 `register_builtin_natives()`
- 替换 ~505 次 `register_with_id()` / `register_with_id_and_type()` 调用

### Phase 4: Opaque Dispatch 合并（待做）
- 提取 `OPAQUE_DISPATCH` 静态表
- 替换 codegen.rs 中 4 处 + engine.rs 中 1 处 match 块

### Phase 5: 清理（待做）
- 将 alias `register_name()` 调用迁入 catalog
- 删除不再需要的手动注册代码

## Verification

每个 Phase 完成后：
1. `cargo build --bin auto` — 编译通过
2. `cargo test -p auto-lang --lib` — 单元测试通过
3. 运行相关 cookbook 文件 — 运行时行为不变
4. `cargo test -p auto-lang -- trans` — a2r/a2c 转译器测试通过
