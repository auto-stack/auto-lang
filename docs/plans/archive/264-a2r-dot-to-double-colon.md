# Plan 264: a2r Dot to Double Colon — COMPLETE

## Context

Auto 语言统一用 `.` 表示模块路径、类型限定和字段访问（继承自 D 语言习惯）。Rust 区分 `::`（模块/类型路径）和 `.`（字段/方法访问）。a2r 转译器目前没有正确区分这两种情况，导致：

1. **类型标注中的 `module.Type` 未翻译**：`fn foo(x: forge.ForgeApiState)` 应输出 `fn foo(x: crate::forge::ForgeApiState)`，但实际输出 `fn foo(x: forge.ForgeApiState)`（12 个编译错误，全在 `server.rs`）
2. **表达式中的 `module.Type` 已部分处理**：`expr()` 中有 `lhs_is_type` 判断（line 938-999），用 `self.uses` 启发式识别模块名，成功时输出 `::`，否则输出 `.`
3. **当前靠 `fix_transpiled.py` 后处理**：用硬编码的 `QUALIFIED_MODULES` 列表 + 大小写启发式做正则替换

## 现状分析

### 信息流

```
Auto 源码 → Parser(共享 TypeStore) → AST → RustTrans → Rust 代码
```

### 已有的 meta 信息

| 来源 | 存储内容 | 能否判断 "X 是模块" |
|---|---|---|
| `TypeStore.type_decls` | 类型名→TypeDecl | 否，只记裸名 |
| `TypeStore.fn_decls` | 函数名→Fn | 否 |
| `RustTrans.uses` | 导入的模块名集合 | **部分** — `use forge` 会记录 `"forge"` |
| `RustTrans.local_modules` | 本地模块名集合 | **是** — 但只在 crate root 填充 |
| `RustTrans.glob_imported_modules` | 通配导入的模块 | **是** — `use crate::X::*` |
| `RustTrans.struct_fields` | 类型名→字段名列表 | 否，无模块信息 |
| `Type::User(TypeDecl)` | 只有 `name: Name` | **否** — 丢失了模块前缀 |

### 关键缺口

1. **`Type::User` 不保留模块路径**：Parser 遇到 `forge.ForgeApiState` 作为类型时，只存储 `"ForgeApiState"` 到 `TypeDecl.name`，`forge.` 前缀丢失
2. **`rust_type_name()` 只输出裸名**：line 578 `Type::User(usr) => usr.name.to_string()` — 无模块前缀
3. **表达式路径已有启发式但不够精确**：`lhs_is_type` 用 `self.uses` 匹配，但 `uses` 可能不完整
4. **没有跨模块的 "谁定义在哪个模块" 映射**

## 修复方案

### 核心思路

在多文件转译流程 (`transpile_rust_project`) 中，Phase 1 已经扫描了所有模块的文件名。我们利用这些信息构建一个 **`module_types` 映射**（模块名 → 该模块定义的类型名列表），然后在转译时用这个映射来判断 `.` 应翻译为 `::` 还是 `.`。

### 不修改 Type AST 的理由

`Type::User(TypeDecl)` 不带模块路径是 AST 层面的设计。修改它会影响整个编译器管道（Parser、Infer、VM Codegen 等），风险太大。转译器作为编译管道的末端消费者，应该在**转译器内部**解决路径问题。

### 实施步骤

#### Step 1: 构建 `module_types` 映射

**文件**: `trans/rust.rs` 的 `transpile_rust_project()` 函数 (line ~8340)

在 Phase 1（发现模块）和 Phase 2（解析模块）之间，新增阶段构建映射：

```rust
// Phase 1.8: Build module → type names mapping
// module_types["forge"] = {"ForgeApiState", "ForgeSession", "ForgeStreamResult", ...}
// module_types["types"] = {"Settings", "ToolRegistry", "ToolDefinition", ...}
let mut module_types: HashMap<String, HashSet<String>> = HashMap::new();
for module in &modules {
    let mod_name = if module.is_dir_module {
        module.source_path.parent().unwrap()
            .file_name().unwrap().to_string_lossy().to_string()
    } else {
        module.source_path.file_stem()
            .unwrap().to_string_lossy().to_string()
    };
    let source = std::fs::read_to_string(&module.source_path)?;
    for line in source.lines() {
        let trimmed = line.trim();
        for prefix in &["pub type ", "type ", "pub enum ", "enum "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = extract_first_identifier(rest);
                if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    module_types.entry(mod_name.clone())
                        .or_default()
                        .insert(name.to_string());
                }
                break;
            }
        }
    }
}
```

辅助函数 `extract_first_identifier`：取第一个标识符（在 `<`、` `、`{` 之前）。

#### Step 2: 将 `module_types` 传递给每个 RustTrans 实例

**文件**: `trans/rust.rs`

在 `RustTrans` struct 中新增字段：

```rust
pub struct RustTrans {
    // ... 现有字段 ...
    
    /// Maps module name → set of type names defined in that module.
    /// Used to determine if `module.Type` should be `module::Type` in Rust.
    module_types: HashMap<String, HashSet<String>>,
}
```

在 `transpile_rust_project()` Phase 3 中传递：

```rust
let mut transpiler = RustTrans::new(AutoStr::from(&module.output_name));
transpiler.module_types = module_types.clone();
```

#### Step 3: 修改 `rust_type_name()` — 为 User type 添加模块前缀

**文件**: `trans/rust.rs` line 578

```rust
Type::User(usr) => {
    let name = usr.name.to_string();
    // Find which module defines this type, prefix with crate::module::
    self.qualify_type_name(&name)
}
```

新增 helper：

```rust
fn qualify_type_name(&self, name: &str) -> String {
    for (mod_name, types) in &self.module_types {
        if types.contains(name) {
            return format!("crate::{}::{}", mod_name, name);
        }
    }
    // Fallback: check if this type is in current file (no prefix needed)
    // or is a well-known std type
    name.to_string()
}
```

**注意**：需要排除当前模块自身定义的类型（避免 `crate::server::ServerResponse` 在 server.rs 内部出现）。可通过记录当前转译的模块名来过滤。

#### Step 4: 修改表达式中的 `lhs_is_type` 判断

**文件**: `trans/rust.rs` line 940-955

当前逻辑用 `self.uses` 判断，增强为也检查 `self.module_types`：

```rust
let is_type_name = if let Expr::Ident(lhs_name) = lhs.as_ref() {
    let name = lhs_name.as_str();
    // 原有检查：大写开头、Rust 原生类型
    name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
        || matches!(name, "u8" | "u16" | ...)
    // 新增：在 module_types 中作为模块名出现
        || self.module_types.contains_key(name)
    // 原有：在 uses 中出现
        || self.uses.iter().any(...)
} else { false };
```

#### Step 5: 处理 `use` 语句中的 `.` → `::` + `crate::` 前缀

**文件**: `trans/rust.rs` `use_stmt()` 方法 (line ~6092)

当前已有 `local_modules` / `glob_imported_modules` 检查。增强：
- `use forge` → `use crate::forge::*;`（已有）
- `use forge.wiki` → `use crate::forge::wiki;`（需要新处理点号路径）

#### Step 6: 排除自身模块的类型前缀

在 `qualify_type_name` 中：

```rust
fn qualify_type_name(&self, name: &str) -> String {
    for (mod_name, types) in &self.module_types {
        if types.contains(name) {
            // 如果是当前模块定义的类型，不加前缀
            if mod_name == &self.current_module_name {
                return name.to_string();
            }
            return format!("crate::{}::{}", mod_name, name);
        }
    }
    name.to_string()
}
```

需要在 `RustTrans` 中添加 `current_module_name: String` 字段。

## 关键文件

| 文件 | 操作 |
|---|---|
| `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs` | 修改 — Step 1-6 |
| `d:/autostack/auto-coder/coder/rust/src/server.rs` | 验证 — 转译后应无 `.` 编译错误 |
| `d:/autostack/auto-coder/coder/scripts/fix_transpiled.py` | 参考 — Pass 2 逻辑将被替代 |

## 验证方式

```bash
# 1. 转译 auto-coder 项目
cd d:/autostack/auto-coder/coder && auto.exe build --backend=rust --dir=.

# 2. 检查编译错误数量（目标: 12 个 `.` 相关错误 → 0）
cd rust && cargo check 2>&1 | grep "^error" | wc -l

# 3. 对比检查 server.rs 中类型路径
grep -n "forge\.\|types\.\|relay\." src/server.rs | grep -v "forge::" | head
```
