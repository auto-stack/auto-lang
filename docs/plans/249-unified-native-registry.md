# Plan 249: Unify Dual Registry — Single-Registration Architecture + Lazy Registration

## Context

Auto VM 有两套独立的 native 函数注册系统：
- **BIGVM_NATIVES** (`native_registry.rs`): 编译期全局 `AutoVMNativeRegistry`，存 ID 和返回类型。`register_builtin_natives()` 中 ~505 次 `register_with_id` 调用。
- **NativeInterface** (`native.rs`): 运行时 shim 绑定，存函数指针和 name→ID 映射。`register_std_shims()` 中 ~222 次 `register` + ~140 次 `register_name` 调用。

同一个函数（如 `auto.list.len`）需要：定义常量(1) → 注册到 BIGVM_NATIVES(2) → 绑定 shim 函数(3) → 注册名称(4) → 在 codegen dispatch 中列出(5) → 在 engine CALL_SPEC dispatch 中列出(6) → 声明返回类型(7)。总计 7 处。

目标：**每个函数只定义一次**，通过宏自动生成所有注册点。然后进一步改为 lazy 注册——只在 codegen 首次遇到时才注册。

## 方案：Callback 宏 + 统一 Catalog + Lazy Registration

### 最终实现

两个 catalog 宏，各自驱动消费者：

**`for_each_native!`** (222 entries, 4-tuple): `(ID, CONST_NAME, shim_fn, "canonical.name")`
- `gen_native_constants!` → 生成 `pub const NATIVE_*: u16 = N;`
- `__register_shims` → 生成 `self.register($name, $fn);`
- `__register_names` → 生成 `self.register_name($canonical, $name);`

**`for_each_bigvm_native!`** (~491 entries, 3-tuple): `("name", id, ret_type_tag)`
- `register_bigvm!` → 生成 `register_with_id` 或 `register_with_id_and_type`（已不再用于 eager 注册）
- `NATIVE_ID_ENTRIES` → 运行时 name→fixed_id 映射，用于 lazy 注册

### Lazy Registration (Phase 6)

启动时不再全量注册 491 个 native 函数。`resolve_qualified()` 在查找失败时查 `NATIVE_ID_MAP`（从 `NATIVE_ID_ENTRIES` 构建的 HashMap），用固定 ID 按需注册。

**为什么需要固定 ID**：shim 绑定通过 `NATIVE_*` 常量绑定到固定 ID。lazy 注册必须使用与 shim 一致的固定 ID，否则 `CALL_NAT` 发出的 ID 与 shim 绑定的 ID 不匹配。

```
codegen 遇到 "auto.list.new"
  → resolve_qualified("auto.list.new")
    → registry.get("auto.list.new") → None
    → NATIVE_ID_MAP.get("auto.list.new") → Some(100)
    → registry.insert("auto.list.new", 100)
    → return Some(100)
  → emit CALL_NAT + 100
```

### Opaque Dispatch Table

8 个静态常量表 (`OPAQUE_DISPATCH_REGEX/URL/SEMVER/CHRONO/BASE64/HEX/SHA2/MIME`) + 2 个查找函数。

## 改动文件

| 文件 | 改动 |
|------|------|
| `vm/native_catalog.rs` | **新建**：`for_each_native!` + `for_each_bigvm_native!` + `OPAQUE_DISPATCH` 表 + `NATIVE_ID_ENTRIES` |
| `vm/native.rs` | 删除 ~233 常量，`register_std_shims` 改为 3 个宏调用 + 8 条手动别名 |
| `vm/native_registry.rs` | `resolve_qualified()` 添加 lazy 路径；`register_builtin_natives()` 缩减为只注册 vm_declarations |
| `vm/codegen.rs` | 2 处 opaque dispatch 替换为 `lookup_opaque_dispatch()`；4 处硬编码 ID 改为 `resolve_qualified()` |
| `vm/engine.rs` | 1 处 opaque dispatch 替换为 `lookup_opaque_dispatch_by_type()` |

## 成果量化

| 指标 | 改动前 | 改动后 |
|------|--------|--------|
| `native_registry.rs` 行数 | ~1180 | ~440 |
| 手动注册调用总数 | ~950 | ~8（仅别名） |
| 定义点数量（每个函数的注册次数） | 最多 7 处 | 最多 2 处（catalog 定义 + 宏消费） |
| Opaque dispatch 表副本 | 5 份 | 1 份 |
| 启动时注册的 native 函数 | 491 个 | ~30-50 个（仅 vm_declarations） |
| 未使用的 native 函数 | 仍注册 | 不注册 |

## 迁移策略：按类别分批

### Phase 1: 基础设施 ✅ DONE
- 创建 `native_catalog.rs`，定义宏框架
- 替换 native.rs 中 222 个 `pub const NATIVE_*` 常量为宏调用
- `cargo build --bin auto` + `cargo test -p auto-lang --lib` 通过

### Phase 2: Shim 绑定自动化 ✅ DONE
- `for_each_native!(bind_shims)` 替换 206 次 `self.register()` 调用
- `register_name()` 调用保留手动（~134 条别名映射）
- 320 VM tests passed

### Phase 3: BIGVM 注册自动化 ✅ DONE
- 扩展 `for_each_bigvm_native!` 到 ~491 个条目，三元组 `(name, id, ret_type_tag)`
- `register_bigvm!` 消费者宏，8 个类型分发 arm
- `native_registry.rs` 从 ~1180 行减少到 ~470 行
- 320 VM tests passed

### Phase 4: Opaque Dispatch 合并 ✅ DONE
- 提取 `OPAQUE_DISPATCH_*` 静态常量表
- `lookup_opaque_dispatch()` / `lookup_opaque_dispatch_by_type()` 函数
- 替换 codegen.rs 2 处 + engine.rs 1 处 dispatch match 块
- 320 VM tests passed

### Phase 5: Name 绑定自动化 ✅ DONE
- 扩展 `for_each_native!` 为四元组 `(ID, CONST_NAME, shim_fn, "canonical.name")`
- `__register_names` 宏自动生成 `register_name()` 调用
- 仅保留 8 条手动别名（1-to-N 关系）
- 320 VM tests passed

### Phase 6: Lazy Native Registration ✅ DONE
- 消除 codegen.rs 中 4 处硬编码 ID（190, 1292, 122, 1500）
- 新增 `NATIVE_ID_ENTRIES` 静态数组（491 条 name→fixed_id 对）
- `resolve_qualified()` 改为 `&mut self`，添加 lazy 注册路径（查 `NATIVE_ID_MAP`）
- `register_builtin_natives()` 从 ~50 行缩减为 ~10 行
- `register_builtin_natives()` 不再调用 `for_each_bigvm_native!` 宏
- 320 VM tests passed

## Verification

每个 Phase 完成后：
1. `cargo build --bin auto` — 编译通过
2. `cargo test -p auto-lang --lib -- vm::` — VM 测试通过（基线：320 pass）
3. 运行相关 cookbook 文件 — 运行时行为不变
4. `cargo test -p auto-lang -- trans` — a2r/a2c 转译器测试通过

## 待确认

1. `NATIVE_ID_ENTRIES` 与 `for_each_bigvm_native!` 的数据重复问题——后续可以用 build script 自动生成
2. 是否需要添加一致性测试来验证两个数据源的同步
