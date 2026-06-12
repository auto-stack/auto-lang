# Plan 275: AURA 键盘绑定 (Key Bindings)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 AURA widget 添加键盘绑定语法，让按键触发与按钮点击相同的 handler，用于 calculator 等场景。

**Architecture:** 新增 `bind` 块语法，在 parser 解析后存入 AST，extract 阶段写入 `AuraWidget.key_bindings`，运行时通过 iced subscription 监听键盘事件并路由到已有 handler。路由基础设施已完全就绪（`IcedMessage → on_with_input → call_handler`），只需补上 key → handler 的映射。

**Tech Stack:** Rust, iced 0.14 subscriptions, AutoLang parser

---

## Context

用户希望 calculator 的数字键和操作符可以同时用键盘输入。当前 AURA 只支持 widget 级的 `onclick`/`onchange` 事件，没有全局键盘监听。iced 已有 `debug_keyboard_sub()` 处理 F12，subscription 批处理机制完善，消息路由通用化 — 扩展键盘绑定是自然的。

## 状态：已实现 ✅

所有步骤已完成并通过验证。

---

## 语法设计

新增 `bind` 块（与 `on` 块平级）：

```auto
widget App {
    msg Msg { Digit0, Digit1, Add, Equals, Clear }
    model { ... }
    view { ... }

    bind {
        "0" -> .Digit0
        "1" -> .Digit1
        "+" -> .Add
        "-" -> .Sub
        "Enter" -> .Equals
        "Escape" -> .Clear
        "Ctrl+s" -> .Save       // Ctrl + 小写 s
        "Ctrl+S" -> .SaveAll    // Ctrl + Shift + s = Ctrl + 大写 S
    }

    on {
        .Digit0 -> { ... }
        .Add -> { ... }
        .Equals -> { ... }
    }
}
```

**选择 `bind` 块而非其他方案的理由：**
- `on` 块内 `key` 前缀会打破 `.Name -> { body }` 的一致性模式
- button 内 `key: "1"` 不适合全局快捷键（Escape、Ctrl+S）
- `bind` 是纯映射（key → handler），不含代码逻辑，职责清晰
- 与 `on` 块对称 — `on` 映射 UI 事件，`bind` 映射键盘事件

---

## Key 字符串匹配规则

bind 块中的 key 字符串**严格区分大小写**，使用 OS 返回的原始字符：

### 无修饰符

| bind 写法 | 实际按键 | 说明 |
|-----------|---------|------|
| `"s"` | s | 小写 s |
| `"S"` | Shift+s | 大写 S |
| `"+"` | 小键盘+ 或 Shift+= | 加号（平台 fallback） |
| `"$"` | Shift+4 | 美元符号 |
| `"Enter"` | Enter | 命名键 |
| `" "` | Space | 空格 |

**规则：字符就是输入结果，`s` 和 `S` 是不同的绑定。**

### 有修饰符（Ctrl / Alt）

| bind 写法 | 实际按键 | 说明 |
|-----------|---------|------|
| `"Ctrl+s"` | Ctrl+s | Ctrl + 小写 s |
| `"Ctrl+S"` | Ctrl+Shift+s | Ctrl + 大写 S |
| `"Ctrl+1"` | Ctrl+1 | Ctrl + 数字 1 |
| `"Alt+q"` | Alt+q | Alt + 小写 q |

**规则：修饰符前缀 + 原始字符。Shift 不作为前缀出现（效果已体现在字符本身）。**

### 支持的修饰符前缀

- `Ctrl+` — Control 键
- `Alt+` — Alt 键
- 可组合：`"Ctrl+Alt+s"`

### 支持的命名键

| bind 写法 | 按键 |
|-----------|------|
| `"Enter"` | Enter |
| `"Escape"` | Escape |
| `"Backspace"` | Backspace |
| `"Tab"` | Tab |
| `" "` | Space |
| `"ArrowUp"` / `"ArrowDown"` / `"ArrowLeft"` / `"ArrowRight"` | 方向键 |
| `"Delete"` | Delete |
| `"Home"` / `"End"` | Home / End |

### 平台兼容性

在 Windows 上，iced 对 Shift+双字符键返回的是基础字符 + Shift modifier（如 Shift+= 返回 `Character("=")` with `Modifiers(SHIFT)`，而不是 `Character("+")`）。renderer.rs 中有 fallback 映射表处理此差异：

```rust
// Windows fallback: Shift+= 的 "+" 映射
("=", "+"), ("8", "*"), ("-", "_"), ("/", "?"),
```

这使得 `bind { "+" -> .Add }` 在所有平台上都能工作。

---

## 实施步骤

### Step 1: AST 数据结构 ✅

**文件:** `crates/auto-lang/src/ast/ui.rs`

新增 `BindBlock` 和 `KeyBinding` 结构体，`WidgetDecl` 添加 `bind: Option<BindBlock>` 字段。

### Step 2: Parser — `parse_bind_block()` ✅

**文件:** `crates/auto-lang/src/parser.rs`

新增 `parse_bind_block()` 方法，解析 `"key" -> .Handler` 语法。

### Step 3: AuraWidget 存储 ✅

**文件:** `crates/auto-lang/src/aura/types.rs`

`AuraWidget` 添加 `key_bindings: HashMap<String, String>` 字段。

### Step 4: Extract — 提取绑定 ✅

**文件:** `crates/auto-lang/src/aura/extract.rs`

从 `BindBlock` 提取 `key_bindings` HashMap。

### Step 5: DynamicComponent 传递 ✅

**文件:** `crates/auto-lang/src/ui/dynamic.rs`

`DynamicComponent` 添加 `key_bindings` 字段和访问器。

### Step 6: Iced 键盘订阅 ✅

**文件:** `crates/auto-lang/src/ui/iced/renderer.rs`

替换 `debug_keyboard_sub()` 为 `keyboard_subscription()`，使用 `OnceLock<Mutex<HashMap>>` 全局存储 + `listen_with` fn pointer。

### Step 7: 更新 Calculator 示例 ✅

**文件:** `examples/ui/011-calculator/src/front/app.at`

添加 `bind` 块，绑定数字键、运算符、Enter、Escape、Backspace。

### Step 8: 编译修复（机械性）✅

所有构造 `AuraWidget` 的文件加 `key_bindings: HashMap::new()`。

### Bug Fix: Handler 编译器跳过注释 ✅

**文件:** `crates/auto-lang/src/ui/vm_bridge.rs`

`compile_stmt()` 中 `Stmt::EmptyLine` 和 `Stmt::Comment` 导致 handler 编译失败（静默跳过）。
添加 match 分支跳过这两种语句。

---

## 关键文件清单

| 文件 | 改动类型 |
|------|----------|
| `crates/auto-lang/src/ast/ui.rs` | 新增 `BindBlock`, `KeyBinding`; `WidgetDecl` 加 `bind` 字段 |
| `crates/auto-lang/src/parser.rs` | 新增 `parse_bind_block()`; `parse_widget_decl()` 加 `bind` 分支 |
| `crates/auto-lang/src/aura/types.rs` | `AuraWidget` 加 `key_bindings` 字段 |
| `crates/auto-lang/src/aura/extract.rs` | 从 `BindBlock` 提取绑定到 `key_bindings` |
| `crates/auto-lang/src/ui/dynamic.rs` | `DynamicComponent` 加 `key_bindings` 字段和访问器 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | `keyboard_subscription()` 替换 `debug_keyboard_sub()` |
| `crates/auto-lang/src/ui/vm_bridge.rs` | handler 编译器跳过 EmptyLine/Comment |
| `examples/ui/011-calculator/src/front/app.at` | 添加 `bind` 块 |

## 数据流

```
bind { "1" -> .Digit1 }
  ↓ parser.rs: parse_bind_block()
BindBlock { bindings: [KeyBinding { key: "1", handler: ".Digit1" }] }
  ↓ extract.rs
AuraWidget.key_bindings: { "1" → ".Digit1" }
  ↓ dynamic.rs
DynamicComponent.key_bindings: { "1" → ".Digit1" }
  ↓ renderer.rs: keyboard_subscription()
iced Key::Character("1") → 查表 → "Digit1" → IcedMessage { event: "Digit1" }
  ↓ update()
component.on_with_input("Digit1", None) → call_handler("Digit1") → 执行 bytecode
```

---

## 未来扩展方向

### 多键 → 单事件

暂不加 `|` 语法。当前用重复行实现：

```auto
bind {
    "Enter" -> .Equals
    "=" -> .Equals       // 两种方式触发同一个事件
}
```

### 作用域限定

当前 `bind` 作用域是 widget 级。每个 widget 有自己的 `key_bindings` HashMap。
未来路由系统实现时，根据焦点链决定哪个 widget 的绑定生效。

```auto
widget ListView {
    bind {
        "j" -> .Next
        "k" -> .Prev
    }
}

widget DetailView {
    bind {
        "Ctrl+s" -> .Save
        "Escape" -> .Back
    }
}
```

## 验证

```bash
# 1. 编译
cargo build -p auto --features ui-iced

# 2. 运行 calculator
auto examples/ui/011-calculator/src/front/app.at

# 3. 验证键盘绑定
# - 按键盘 "1" → display 显示 "1"
# - 按键盘 "+" (小键盘或 Shift+=) → expr 显示 "1 +"
# - 按键盘 "2" → display 显示 "2"
# - 按 Enter → display 显示 "3"，expr 显示 "1 + 2 ="
# - 按 Escape → 清除所有
# - 按 Backspace → 清除所有
```
