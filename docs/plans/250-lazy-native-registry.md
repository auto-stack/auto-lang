# Plan 250: AOT-Style Lazy Native Registration

## Context

当前 `register_builtin_natives()` 在 VM 启动时全量注册 ~491 个 native 函数到 `BIGVM_NATIVES`，即使大部分函数在当前程序中根本用不到。

类比 C 语言的链接器：每个 symbol 在不同编译 session 中可能分配到不同的地址，但在同一个编译 session 内，symbol→address 映射是固定的。

**目标**：将注册改为"编译期按需分配"——codegen 在首次遇到某个 native 调用时才注册，未使用的函数永远不注册。

## 实际实现（策略 A 变体：固定 ID + Lazy 注册）

### 核心改动

1. **消除 codegen.rs 中 4 处硬编码 ID** — 全部改为 `resolve_qualified()` 名称查找
2. **创建 `NATIVE_ID_ENTRIES` 静态数组** — 491 个 `(name, fixed_id)` 对
3. **`resolve_qualified()` 添加 lazy 路径** — 查找失败时查 `NATIVE_ID_MAP`，用固定 ID 注册
4. **缩减 `register_builtin_natives()`** — 只注册 `register_vm_declarations()` 扫描到的 stdlib 函数

### 为什么需要固定 ID（而不是完全动态分配）

shim 绑定（`register_std_shims()`）通过 `NATIVE_*` 常量绑定到固定 ID。如果 lazy 注册分配了不同的 ID，`CALL_NAT` 发出的 ID 与 shim 绑定的 ID 不匹配。因此 lazy 注册必须使用与 shim 一致的固定 ID。

`NATIVE_ID_MAP` 的作用不是分配新 ID，而是**延迟注册**——只在 codegen 首次需要时才将 name→fixed_id 写入 `BIGVM_NATIVES` registry。

### 改动文件

| 文件 | 改动 |
|------|------|
| `vm/codegen.rs` | 4 处硬编码 ID（190, 1292, 122, 1500）改为 `resolve_qualified()` |
| `vm/native_catalog.rs` | 新增 `NATIVE_ID_ENTRIES: &[(&str, u16)]` 数组（491 条目） |
| `vm/native_registry.rs` | `resolve_qualified()` 改为 `&mut self`，添加 lazy 注册路径；`register_builtin_natives()` 从 ~50 行缩减为 ~10 行 |
| `vm/native.rs` | `register_shim_by_name()` 的 `let natives` → `let mut natives` |

### Lazy 注册流程

```
codegen 遇到 "auto.list.new"
  → resolve_qualified("auto.list.new")
    → registry.get("auto.list.new") → None（未注册）
    → to_canonical → 已是 canonical
    → NATIVE_ID_MAP.get("auto.list.new") → Some(100)（在白名单中）
    → registry.insert("auto.list.new", 100)
    → return Some(100)
  → emit CALL_NAT + 100
```

别名路径（如 `"Option.or"`）：
```
register_shim_by_name("Option.or")
  → resolve_qualified("Option.or")
    → registry.get("Option.or") → None
    → to_canonical("Option.or") → "auto.option.or"
    → registry.get("auto.option.or") → None
    → NATIVE_ID_MAP.get("Option.or") → None
    → NATIVE_ID_MAP.get("auto.option.or") → Some(1550)
    → registry.insert("Option.or", 1550)
    → return Some(1550)
```

## 验证

| 指标 | 结果 |
|------|------|
| `cargo build --bin auto` | 0 errors |
| `cargo test -p auto-lang --lib -- vm::` | 320 passed（与基线一致） |
| `cargo test -p auto-lang --lib -- vm::native_registry` | 8 passed |

## 性能影响

| 指标 | 改造前 | 改造后 |
|------|--------|--------|
| VM 启动时间 | 注册 491 个条目 | 只注册 vm_declarations（~30-50 个） |
| 首次 native 调用 | O(1) HashMap | O(1) HashMap + O(1) NATIVE_ID_MAP lookup |
| 后续 native 调用 | O(1) HashMap | O(1) HashMap（相同） |
| 未使用的 native 函数 | 仍注册 | 不注册（节省内存和时间） |

## 待确认

1. `NATIVE_ID_ENTRIES` 与 `for_each_bigvm_native!` 的数据重复问题——后续可以考虑用 build script 自动生成
2. 是否需要添加一致性测试来验证 `NATIVE_ID_ENTRIES` 和 `for_each_bigvm_native!` 的数据同步
