# Plan 203: Native Registry Namespace 统一

## Status: ✅ Phase 1-5 Complete (Phase 5f deferred)

Verified 2026-04-24:
- ✅ Phase 1: `QualifiedName` struct in `vm/qualified_name.rs` with path() and dot-separated module support
- ✅ Phase 2: Use Resolver — `import_scope` HashMap + `handle_use_stmt()` populates from `use mod: items`
- ✅ Phase 3: Codegen uses `resolve_qualified()` everywhere (checks qualified_registry then short-name fallback)
- ✅ Phase 4: `resolve_qualified()` in native_registry, `to_canonical()` fallback for short names

Phase 5 完成 (2026-04-27):
- ✅ Phase 5a: `TYPE_CANONICAL_MAP` + `to_canonical()` 修正（12 个显式映射）
- ✅ Phase 5b: 删除 ~137 个短名别名，Http/Response/Option/Result 重命名为 canonical
- ✅ Phase 5c: codegen 中 10 处硬编码短名替换为 canonical
- ✅ Phase 5d: 删除 compile.rs ~25 行动态短名注册（用 import_scope 替代）
- ✅ Phase 5e: 重构 `try_mono_dispatch()`，HashMap/String 类型映射，消灭 ~55 个 monomorphic 别名
- 🔧 Phase 5f: 裸名 + #[rust_fn] 注解批量重命名（Future Work，风险高）

> 日期：2026-04-21
> 状态：设计阶段
> 依赖：Plan 200（VM 缺失特性）

## 问题

当前 AutoVM 的 native 函数查找完全依赖**字符串匹配**。同一个函数因 `use` 导入方式不同，有多个调用名，但 registry 只注册了其中一个。

```auto
use auto.fs: read_text       → 调用名 "read_text"
use auto.fs; fs.read_text()   → 调用名 "fs.read_text"
auto.fs.read_text(...)        → 调用名 "auto.fs.read_text"
File.read_text(...)           → 调用名 "File.read_text"    (现有风格)
```

四个名字指向同一个 native shim，但 registry 只注册了 `"File.read_text"`。

**现状统计：**
- `BIGVM_NATIVES` 是 `HashMap<String, u16>`
- codegen 中有 **7 处**通过字符串拼凑后调用 `.get_id(name)` 查找
- `infer_type_from_var` 用硬编码 heuristic 推断类型名
- `use` 语句在 VM codegen 中**完全被忽略**，不参与 native 查找
- 无 `QualifiedName` 类型、无 canonical ID 概念

## 设计原则

参考 Rust 的 `DefId` 模式：

1. **Resolver 阶段**把所有名称（短名、模块前缀、全限定）解析为 canonical ID
2. **Registry 只注册一次**，key 是 canonical ID
3. **Codegen 只看 canonical ID**，不再拼字符串查表
4. Import 风格在 resolver 阶段被抹掉

## 架构

### 核心概念

```
QualifiedName = ModulePath + ItemName
    例: auto.fs.read_text, auto.list.push, auto.str.len

ModulePath = ["auto", "fs"] | ["auto", "str"] | []

CanonicalId = u16  (复用现有 native ID)
```

### 三层结构

```
┌─────────────────────────────────────────────────┐
│  Layer 1: QualifiedName Registry                │
│  "auto.fs.read_text"      → NativeId(1000)     │
│  "auto.file.read_text"    → NativeId(1000)     │  ← 别名
│  "auto.list.push"         → NativeId(101)      │
│  "auto.str.len"           → NativeId(170)      │
└─────────────────────────────────────────────────┘
          ↑ 注册一次，全限定名
┌─────────────────────────────────────────────────┐
│  Layer 2: Use Resolver                          │
│  use auto.fs: read_text                         │
│    → scope["read_text"] = QualifiedName("auto.fs.read_text")
│  use auto.fs                                    │
│    → scope["fs"] = ModulePath("auto.fs")        │
│  (无 use)                                       │
│    → 直接查 QualifiedName                       │
└─────────────────────────────────────────────────┘
          ↑ 编译期执行，产出 QualifiedName
┌─────────────────────────────────────────────────┐
│  Layer 3: Codegen                               │
│  收到 canonical NativeId，直接 emit CALL_NAT     │
│  不再做字符串拼接或 registry 查找                  │
└─────────────────────────────────────────────────┘
```

### 调用流程对比

**Before（当前）：**
```
list.push(1)
  → infer_type_from_var("list") → "List"     // heuristic 猜
  → format!("{}.{}", "List", "push")          // 拼字符串
  → BIGVM_NATIVES.get_id("List.push")         // 字符串查表
  → Some(101)                                  // 拿到 ID
  → CALL_NAT 101
```

**After（目标）：**
```
list.push(1)
  → resolver 查 var_types["list"] 得知类型
  → resolver 查该类型的 method "push" 的 QualifiedName
  → QualifiedName("auto.list.push") → NativeId(101)
  → codegen 直接 emit CALL_NAT 101
```

### 实例方法的处理

实例方法（如 `list.push(1)`）需要先推断 receiver 类型，再查找方法：

```
Type::List(_) 的 method "push" → QualifiedName("auto.list.push")
Type::Str(_) 的 method "len"   → QualifiedName("auto.str.len")
Type::Map(_,_) 的 method "get" → QualifiedName("auto.hash_map.get")
```

这需要一个 **Type → MethodTable** 映射：
```rust
type MethodTable = HashMap<String, QualifiedName>;  // method_name → canonical name

let type_methods: HashMap<&str, MethodTable> = hashmap! {
    "List" => { "new" → "auto.list.new", "push" → "auto.list.push", ... },
    "str"  => { "len" → "auto.str.len", "upper" → "auto.str.upper", ... },
    ...
};
```

## 实施步骤

### Phase 1: QualifiedName 类型 + Registry 改造

**目标：** 建立 canonical 注册机制，不破坏现有代码。

1. 新增 `QualifiedName` 类型（`ast/qualified_name.rs`）
2. 扩展 `AutoVMNativeRegistry`：
   - 新增 `register_qualified(path: &str, id: u16)` — 全限定名注册
   - 新增 `resolve_qualified(path: &str) -> Option<u16>` — 全限定名查找
   - 保留现有 `get_id(short_name)` 不变（向后兼容）
3. 在 `register_builtin_natives()` 中用全限定名重新注册所有 native：
   - `"auto.file.read_text" → 1000`
   - `"auto.list.push" → 101`
   - 等等
4. 现有短名别名作为 fallback 保留，不删除

### Phase 2: Use Resolver

**目标：** `use` 语句参与名称解析。

1. 在 codegen 中处理 `Stmt::Use`：
   - `use auto.fs: read_text` → 在 scope 中记录 `read_text → QualifiedName("auto.fs.read_text")`
   - `use auto.fs` → 在 scope 中记录 `fs → ModulePath("auto.fs")`
2. 新增 `resolve_call_name(name: &str) -> Option<u16>`：
   - 先查 scope 映射得到 QualifiedName
   - 再查 registry 得到 NativeId
   - 最后 fallback 到现有字符串查找（向后兼容）

### Phase 3: Codegen 消除字符串查找

**目标：** codegen 不再拼字符串查 registry。

1. 函数调用处（`Expr::Call`）：
   - bare name 调用（`read_text(...)`）→ 先查 scope，再查 registry
   - dot 调用（`fs.read_text(...)`）→ 先解析 module prefix，再查 registry
   - instance 调用（`list.push(1)`）→ 通过 type method table 查找
2. 替换所有 7 处 `BIGVM_NATIVES.lock().unwrap().get_id(...)` 调用
3. 移除 `infer_type_from_var` 中的硬编码 heuristic

### Phase 4: 迁移 + 清理

1. 移除 registry 中的短名别名（如果 Phase 2/3 覆盖完全）
2. 更新所有测试用例
3. 更新 `#[vm]` 函数声明中的名称约定

## 命名规范

| 类别 | 全限定名 | 现有短名 |
|------|---------|---------|
| 文件 I/O | `auto.fs.read_text` | `File.read_text` |
| 列表 | `auto.list.new` | `List.new` |
| 字符串 | `auto.str.len` | `str.len` |
| 哈希表 | `auto.hash_map.insert` | `HashMap.insert` |
| JSON | `auto.json.parse` | `Json.parse` |

## 风险

1. **向后兼容**：Phase 1-2 必须保留短名 fallback，否则所有现有测试崩溃
2. **实例方法推断**：`var_types` 推断可能不准确，需要渐进增强
3. **模块加载**：`use auto.fs` 需要 VM 知道 `auto.fs` 是什么模块，当前没有模块系统
4. **工作量**：7 处 codegen 改造 + resolver 集成 + 100+ native 重新注册

---

## Phase 5: 消灭短名别名（Short-Name Alias Elimination）

> **Status: Planned**
> **Date: 2026-04-27**

Phase 1-4 建立了全限定名机制，但保留了 ~198 个短名别名作为 fallback。本 Phase 目标是逐步消灭这些别名，使 registry 只注册 canonical 全限定名，所有短名通过解析机制还原。

### 设计原则

1. **Registry 只注册一次** — 每个 native 函数只有一个 canonical 全限定名（如 `auto.collections.List.push`）
2. **Use 语句通过 import_scope 还原** — `use auto.list: push` 让 `push` 映射到 canonical 名，不在 registry 注册别名
3. **Canonical 命名保持类型名大小写** — `List.push` 不是 `list.push`，`TaskHandle.send` 不是 `taskhandle.send`
4. **渐进式消灭** — 6 个类别独立处理，不一次性重构

### Canonical 命名规范

当前问题：canonical 名把所有段都小写了（`auto.list.push`），导致 `TaskHandle` → `taskhandle`、`TaskSystem` → `tasksystem`。

正确规范：

```
auto.<module_path>.<TypeName>.<method>    # 实例/静态方法
auto.<module_path>.<function>             # 模块级函数
```

| 类型 | 当前 canonical | 正确 canonical |
|------|---------------|----------------|
| List 实例方法 | `auto.list.push` | `auto.collections.List.push` |
| str 函数 | `auto.str.len` | `auto.str.len`（str 本身小写，正确） |
| TaskHandle 方法 | `auto.task.handle_send` | `auto.task.TaskHandle.send` 或 `auto.task.handle_send` |
| 裸函数 | `auto.time.sleep_ms` | `auto.time.sleep_ms`（正确） |

> **注意**：引入模块层级（`auto.collections`）需要先定义 stdlib 模块地图。在模块地图确定之前，可以先用 `auto.List.push`（省略中间模块层）作为过渡。

### 6 类短名及消灭策略

#### 类别 A：codegen 硬编码短名（5 处）

**成因**：代码里直接写了短名字符串，没有走解析机制。

| 硬编码 | 位置 | 改为 |
|--------|------|------|
| `"Iterator.next"` | codegen.rs:1732 | canonical 全限定名 |
| `"Task.spawn"` | codegen.rs:4790 | canonical 全限定名 |
| `"Task.send"` | codegen.rs:4804 | canonical 全限定名 |
| `"TaskHandle.send"` | codegen.rs:4902 | canonical 全限定名 |
| `"str.len"` | codegen.rs:5007 | canonical 全限定名 |

**消灭方式**：直接替换字符串。
**风险**：极低。只要替换后的 canonical 名已在 registry 注册即可。
**依赖**：无。可立即执行。

#### 类别 B：`to_canonical()` 转换错误（~25 个）

**成因**：`to_canonical()` 做了 `prefix.to_lowercase()`，导致类型名丢失大小写。例如：
- `TaskHandle.send` → `auto.taskhandle.send`（应为 `auto.task.TaskHandle.send`）
- `TaskSystem.start` → `auto.tasksystem.start`（应为 `auto.task.TaskSystem.start`）
- `Response.status_code` → `auto.response.status_code`（应为 `auto.http.Response.status_code`）
- `Result.Ok.map_err` → 多段式路径 `to_canonical()` 无法处理

**涉及**：TaskHandle（6）、TaskSystem（2）、Response（3）、Result 多段式（2）、其他大小写错误（~12）

**消灭方式**：
1. 定义 Type → CanonicalPrefix 映射表（如 `TaskHandle` → `auto.task.TaskHandle`）
2. 修正 `to_canonical()` 使用映射表而非 `to_lowercase()`
3. 修正 registry 中对应的 canonical 注册名

**风险**：中等。需确保所有类型都有映射条目。
**依赖**：需要先确定 canonical 命名规范。

#### 类别 C：Monomorphic 类型后缀别名（~68 个）

**成因**：Plan 194 引入的类型特化后缀。`List.push_int` 和 `List.push` 是不同的调用入口（虽然可能共享 native ID），codegen 通过拼接 `"{type}.{method}_{suffix}"` 查表。

**涉及**：List（35）、HashMap（17）、HashSet（16）

**消灭方式**：
1. codegen 的 monomorphic dispatch 已经知道 `List<int>` 的 push 对应哪个 native ID
2. 改为在 codegen 时直接用已知 ID emit `CALL_NAT`，不再拼接后缀查表
3. 移除 registry 中所有 `*._int`、`*._str` 等后缀别名

**风险**：中高。需要重构 monomorphic dispatch 的 codegen 路径。
**依赖**：独立，但工作量最大。

#### 类别 D：`use` 语句动态注册的短名

**成因**：`compile.rs` 在处理 `use auto.list: push` 时，除了往 `import_scope` 写映射，还额外调用 `register_with_id("push", id)` 往 registry 注入短名。这导致 registry 被污染，且组合爆炸。

**涉及**：所有 `use mod: item` 和 `use mod: *` 语句产生的动态短名。

**消灭方式**：
1. 删除 `compile.rs` 中 `handle_use_stmt` 里的 `register_with_id()` 调用
2. 完全依赖 `import_scope` HashMap（codegen 中已有 fallback 路径）
3. 扩展 `import_scope` 支持模块级 `use`（`use auto.list` → `list` 映射到 `auto.list` 前缀）

**风险**：低。`import_scope` 路径已存在且工作正常。
**依赖**：需确认 import_scope 覆盖所有 use 场景（包括 `use mod` 无 item 列表的情况）。

#### 类别 E：裸函数名（~13 个）

**成因**：`sleep`、`parse_sse`、`str_new` 等顶层 stdlib 函数，用户代码直接写 `sleep()` 而不通过 `use` 导入。

**涉及**：`sleep`、`parse_sse`、`str_new`、`str_append`、`int.str`、`str.bytes`、`uint.to_hex`、`alloc_array`、`realloc_array`、`free_array`、`str.split_once`、`str.match_count`、`str.replace_first`

**消灭方式**：
1. 用户必须通过 `use auto.time: sleep` 导入后才能使用（走 import_scope）
2. 或者 codegen 对裸名尝试 `to_canonical()` 推导（但裸名无点分隔符，`to_canonical()` 无法处理）
3. 最终方案：裸名全部要求 `use` 导入，不支持无 `use` 直接调用

**风险**：中高。**破坏性变更**——所有现有代码中直接写 `sleep()` 的地方都需要加 `use`。
**依赖**：依赖类别 D 先完成（import_scope 路径必须健壮）。

#### 类别 F：ID 冲突别名（~15 个）

**成因**：同一个函数注册了多个不同 ID。例如 `str.len` → ID 170，`auto.str.len` → ID 1500。可能是不同 shim 实现，也可能是历史重复。

**涉及**：`str.len`(170 vs 1500)、`String.len`(171 vs 1500)、`str.upper`(175 vs 1511)、`String.is_empty`(185 vs 1501) 等

**消灭方式**：
1. 逐个审查 shim 代码，确认两个 ID 的实际行为是否相同
2. 如果相同：合并为一个 ID，移除重复注册
3. 如果不同：保留两个 canonical 名，明确区分

**风险**：低。逐个审查即可。
**依赖**：无。可随时执行。

### 执行顺序

```
Phase 5a: 类别 A — 硬编码短名替换（5 处，极低风险）
Phase 5b: 类别 B — 修正 to_canonical() 和 canonical 命名规范（~25 个）
Phase 5c: 类别 D — 移除 use 动态注册，依赖 import_scope
Phase 5d: 类别 F — 审查并合并 ID 冲突别名（~15 个）
Phase 5e: 类别 C — 重构 monomorphic dispatch（~68 个，工作量最大）
Phase 5f: 类别 E — 裸函数名要求 use 导入（破坏性变更，最后执行）
```

Phase 5a-5d 可并行推进，互不依赖。Phase 5e 工作量最大但不阻塞其他。Phase 5f 是破坏性变更，建议放到下一个 major version。

### 类别 A 实施细节

**修改文件**：`crates/auto-lang/src/vm/codegen.rs`

```rust
// Before (5 places):
BIGVM_NATIVES.lock().unwrap().resolve_qualified("Iterator.next")
BIGVM_NATIVES.lock().unwrap().resolve_qualified("Task.spawn")
// ... etc

// After:
BIGVM_NATIVES.lock().unwrap().resolve_qualified("auto.iterator.next")  // 或 canonical 名
BIGVM_NATIVES.lock().unwrap().resolve_qualified("auto.task.spawn")
// ... etc
```

**验证**：`cargo test -p auto-lang` 全部通过。

### 类别 B 实施细节

**修改文件**：
- `crates/auto-lang/src/vm/native_registry.rs` — 修正 `to_canonical()` 和 canonical 注册名
- `crates/auto-lang/src/vm/native_registry.rs` — 添加 Type → CanonicalPrefix 映射表

```rust
// 修正 to_canonical()：
// Before:
fn to_canonical(name: &str) -> Option<String> {
    let (prefix, rest) = name.split_once('.')?;
    let lower = prefix.to_lowercase();
    Some(format!("auto.{}.{}", lower, rest))
}

// After: 使用映射表
fn to_canonical(name: &str) -> Option<String> {
    let (prefix, rest) = name.split_once('.')?;
    let canonical_prefix = TYPE_CANONICAL_MAP.get(prefix)?;
    Some(format!("{}.{}", canonical_prefix, rest))
}
```

### 类别 D 实施细节

**修改文件**：`crates/auto-lang/src/compile.rs`

```rust
// Before (compile.rs:261-267):
if let Some(native_id) = registry.resolve_qualified(&full_path) {
    registry.register_with_id(item, native_id);  // ← 删除这行
}

// After: 只依赖 import_scope，不注册别名
if registry.resolve_qualified(&full_path).is_some() {
    // import_scope 已经在 codegen 中记录了映射，无需额外注册
}
```

**需确认**：`use auto.list`（无 item 列表）场景下，`list.push()` 的解析路径。

---

## 参考

- Rust `DefId` 机制：name resolution 阶段把所有路径解析为唯一 ID
- Python `sys.modules`：import 时绑定到运行时对象
- Plan 200：暴露了 `fs` 模块别名问题
