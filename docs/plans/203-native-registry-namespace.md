# Plan 203: Native Registry Namespace 统一

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

## 参考

- Rust `DefId` 机制：name resolution 阶段把所有路径解析为唯一 ID
- Python `sys.modules`：import 时绑定到运行时对象
- Plan 200：暴露了 `fs` 模块别名问题
