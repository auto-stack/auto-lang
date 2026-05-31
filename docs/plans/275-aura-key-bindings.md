# Plan 275: AURA 键盘绑定 (Key Bindings)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 AURA widget 添加键盘绑定语法，让按键触发与按钮点击相同的 handler，用于 calculator 等场景。

**Architecture:** 新增 `bind` 块语法，在 parser 解析后存入 AST，extract 阶段写入 `AuraWidget.key_bindings`，运行时通过 iced subscription 监听键盘事件并路由到已有 handler。路由基础设施已完全就绪（`IcedMessage → on_with_input → call_handler`），只需补上 key → handler 的映射。

**Tech Stack:** Rust, iced 0.14 subscriptions, AutoLang parser

---

## Context

用户希望 calculator 的数字键和操作符可以同时用键盘输入。当前 AURA 只支持 widget 级的 `onclick`/`onchange` 事件，没有全局键盘监听。iced 已有 `debug_keyboard_sub()` 处理 F12，subscription 批处理机制完善，消息路由通用化 — 扩展键盘绑定是自然的。

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
        "9" -> .Digit9
        "+" -> .Add
        "-" -> .Sub
        "*" -> .Mul
        "/" -> .Div
        "Enter" -> .Equals
        "Escape" -> .Clear
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

---

## 实施步骤

### Step 1: AST 数据结构

**文件:** `crates/auto-lang/src/ast/ui.rs`

新增（在 `OnBlock` 附近）：

```rust
#[derive(Debug, Clone)]
pub struct BindBlock {
    pub bindings: Vec<KeyBinding>,
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: String,       // "1", "+", "Enter", "Ctrl+S"
    pub handler: String,   // ".Digit1"
}
```

在 `WidgetDecl` 添加 `pub bind: Option<BindBlock>` 字段。

### Step 2: Parser — `parse_bind_block()`

**文件:** `crates/auto-lang/src/parser.rs`

新增 `parse_bind_block()` 方法（~40行），参考 `parse_on_block()` 但更简单 — 无参数、无 body，只有 `"key" -> .Handler`。

在 `parse_widget_decl()` (行 9771) 的 keyword match 中加入 `"bind"` 分支。

**解析规则：**
- key 是双引号字符串（`TokenKind::String`）
- 箭头用 `->`（复用 `parse_on_block` 的 arrow 解析）
- handler 是 `.Name`（复用 dot-name 解析）

### Step 3: AuraWidget 存储

**文件:** `crates/auto-lang/src/aura/types.rs`

在 `AuraWidget` 添加字段：

```rust
pub key_bindings: HashMap<String, String>,  // key_str -> handler_pattern
```

**文件:** `crates/auto-lang/src/aura/atom.rs` — 构造处加 `key_bindings: HashMap::new()`

### Step 4: Extract — 提取绑定

**文件:** `crates/auto-lang/src/aura/extract.rs`

在 `extract_widget_from_decl()` 中，从 `decl.bind` 提取 `key_bindings` HashMap。

### Step 5: DynamicComponent 传递

**文件:** `crates/auto-lang/src/ui/dynamic.rs`

在 `DynamicComponent` 添加 `key_bindings: HashMap<String, String>` 字段，从 `AuraWidget` 克隆。添加 `pub fn key_bindings(&self)` 访问器。

### Step 6: Iced 键盘订阅（核心运行时）

**文件:** `crates/auto-lang/src/ui/iced/renderer.rs`

替换 `debug_keyboard_sub()` 为统一的 `keyboard_subscription(key_bindings: &HashMap<String, String>)`：

1. F12 → `DEBUG_TOGGLE_EVENT`（保持不变）
2. Named keys（Enter、Escape 等）→ 查表映射
3. Character keys（"1"、"+" 等）→ 查表映射
4. Ctrl 组合键 → `format!("Ctrl+{}", c)` 查表

在 `.subscription()` 闭包中（行 2057）：
- 读取 `state.component.key_bindings()`
- 调用 `keyboard_subscription(&key_bindings)`
- 移除原来的 `subs.push(debug_keyboard_sub())`

**关键：输入焦点冲突** — 当 text input 获得焦点时，按键应输入文本而非触发绑定。iced 的 `listen_with` 回调接收 `status` 参数，`Status::Captured` 表示已被 widget 消费，此时应跳过绑定。

**Named key 映射表：**

| `.at` 字符串 | iced Named 枚举 |
|---|---|
| `"Enter"` | `Named::Enter` |
| `"Escape"` | `Named::Escape` |
| `"Backspace"` | `Named::Backspace` |
| `"Tab"` | `Named::Tab` |
| `" "` | `Named::Space` |
| `"ArrowUp"` | `Named::ArrowUp` |
| `"Delete"` | `Named::Delete` |

Character keys 直接用字符匹配。

### Step 7: 更新 Calculator 示例

**文件:** `examples/ui/011-calculator/src/front/app.at`

添加 `bind` 块，绑定数字键和运算符。

### Step 8: 编译修复（机械性）

所有 match/构造 `AuraWidget` 的文件加 `key_bindings: HashMap::new()`：
- `crates/auto-lang/src/a2ui/import.rs`
- `crates/auto-lang/src/ui_gen/` 下的 generator 文件
- 所有 AuraWidget 相关测试

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

## 验证

```bash
# 1. 编译
cargo build -p auto --features ui-iced

# 2. 运行 calculator
auto examples/ui/011-calculator/src/front/app.at

# 3. 验证键盘绑定
# - 按键盘 "1" → display 显示 "1"
# - 按键盘 "+" → expr 显示 "1 +"
# - 按键盘 "2" → display 显示 "2"
# - 按 Enter → display 显示 "3"，expr 显示 "1 + 2 ="
# - 按 Escape → 清除所有
```
