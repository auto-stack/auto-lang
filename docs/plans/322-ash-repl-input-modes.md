# Plan 322: Ash REPL 输入模式系统

> **Status**: ✅ Phase 1-2 implemented (2026-06-17). P1: conservative auto-expression detection + Shell fallback. P2: syntax-based multiline continuation (replaces Plan 302's backslash/quote-only with full {}/()/[]/"" continuation via needs_continuation). P3 deferred: ModeState infrastructure ready in repl_mode.rs, but F-key keybinding needs reedline upgrade (0.44 lacks KeyCode::Function).
> **关系**: 重构 Shell 当前的 `looks_like_auto_expr` 启发式，建立 **自动检测 + 手动锁定 + 自动回退** 三层输入模式系统。支持 Shell / AutoScript / AI 三模式 + 语法自动多行。

---

## 1. 背景

### 1.1 现状

Ash REPL 目前用 `looks_like_auto_expr()` 启发式判断输入是 Shell 命令还是 Auto 表达式：

```rust
fn looks_like_auto_expr(&self, input: &str) -> bool {
    trimmed.starts_with("fn ")
        || trimmed.starts_with("let ")
        || first_char == 'f'          // ← 误捕 find/fmt/file/fd
        || first_char == '['          // ← 可能是 test
        || first_char == '{'          // ← 可能是 brace expansion
        || first_char.is_ascii_digit()
        || self.is_function_call(trimmed)
}
```

**问题**：
- `first_char == 'f'` 误捕所有 `f` 开头的外部命令。
- 无多行支持（每行一次 `session.run()`）。
- 无模式概念——用户无法说「接下来都是 Auto 代码」。
- 未来要加 AI 自然语言输入，无处安放。

### 1.2 目标

建立三模式输入系统：

| 模式 | Prompt | 含义 | 检测方式 |
|---|---|---|---|
| **Shell** | `>` | Shell 命令（ls/git/cargo/...） | 默认 |
| **AutoScript** | `#` | Auto 语言代码（let/fn/表达式） | 自动检测 / F2 锁定 |
| **AI** | `?` | 自然语言 → AI 翻译成命令 | 仅 F3 手动 |

加多行续行（语法自动检测）。

---

## 2. 设计

### 2.1 三层机制

```
层 1: 自动检测（默认，零学习成本）
  输入 → 精确启发式 → Shell / Auto → 显示对应 prompt
  多行: 语法检测（未闭合 { ( [ " → 续行）

层 2: 手动锁定（F1/F2/F3，熟练用户）
  F1 → 锁定 Shell（所有输入当 Shell）
  F2 → 锁定 AutoScript（所有输入当 Auto）
  F3 → AI 模式（每行自然语言）
  Esc → 解除锁定，回到自动检测

层 3: 自动回退（安全网）
  误判 → 送 Auto parser → 执行失败 → 自动重试为 Shell
```

### 2.2 Prompt 符号

```
> ls -la              ← Shell 自动检测
# let x = 5           ← Auto 自动检测（let 关键字）
· a + b               ← 多行续行（未闭合 {）
? how to list files   ← AI 模式

▌> ls                 ← Shell 锁定（F1，加粗/变色标记）
▌# fn add(a) { a+1 }  ← Auto 锁定（F2）
```

- 自动检测：prompt 正常色。
- 锁定：prompt 变色（如蓝色加粗）+ 可选 `▌` 前缀标记。
- 多行续行：`·` 或 `…`。
- AI：`?`。

### 2.3 改进的 Auto 表达式检测

替换当前 `looks_like_auto_expr`，更保守、更精确：

```rust
fn is_auto_expression(&self, input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() { return false; }

    // 1. Auto 关键字开头（最强信号）
    for kw in ["fn ", "let ", "mut ", "const ", "use ", "type ", "enum "] {
        if trimmed.starts_with(kw) { return true; }
    }

    // 2. 纯表达式（强信号）
    let first = trimmed.chars().next().unwrap();
    if first == '"' || first.is_ascii_digit()
        || first == '[' || first == '('
    {
        return true;
    }

    // 3. 已知 Auto 函数调用 name(...)
    if self.is_function_call(trimmed) { return true; }

    // 4. 算术表达式（弱信号，但常见）
    // 匹配: 1 + 2, 3.14 * x, -5 + 3
    if is_arithmetic_expression(trimmed) { return true; }

    // 移除的旧规则:
    // - first_char == 'f'（误捕 find/fmt/file/fd）
    // - first_char == '{'（误捕 brace expansion）
    // - first_char == '-'（误捕 flag -la）

    false  // 默认 Shell
}
```

### 2.4 自动回退

```rust
fn execute_with_fallback(&mut self, input: &str) -> Result<Option<String>> {
    // 1. 如果锁定模式，直接用锁定的
    if let Some(mode) = self.locked_mode {
        return self.execute_in_mode(input, mode);
    }

    // 2. 自动检测
    if self.is_auto_expression(input) {
        match self.execute_auto(input) {
            Ok(result) => return Ok(result),
            Err(auto_err) => {
                // 3. Auto 失败 → 回退 Shell
                // 但要避免无限循环：只回退一次
                match self.execute_shell(input) {
                    Ok(result) => return Ok(result),
                    Err(shell_err) => {
                        // 两者都失败 → 报 Auto 错误（通常更有用）
                        return Err(auto_err);
                    }
                }
            }
        }
    }

    // 4. 默认 Shell
    self.execute_shell(input)
}
```

### 2.5 多行续行

**自动检测**（不需要 F4 手动切换）：

```rust
fn needs_continuation(input: &str) -> bool {
    // 1. 行尾反斜杠
    if input.trim_end().ends_with('\\') { return true; }

    // 2. 未闭合的括号/引号（跨行）
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_bracket = 0i32;
    let mut in_string = None; // Some(') or Some(")

    for c in input.chars() {
        match c {
            '(' if in_string.is_none() => depth_paren += 1,
            ')' if in_string.is_none() => depth_paren -= 1,
            '{' if in_string.is_none() => depth_brace += 1,
            '}' if in_string.is_none() => depth_brace -= 1,
            '[' if in_string.is_none() => depth_bracket += 1,
            ']' if in_string.is_none() => depth_bracket -= 1,
            '\'' | '"' if in_string.is_none() => in_string = Some(c),
            c if Some(c) == in_string => in_string = None,
            _ => {}
        }
    }

    depth_paren > 0 || depth_brace > 0 || depth_bracket > 0 || in_string.is_some()
}
```

**REPL 行为**：
```
> fn add(a, b) int {
·     a + b
· }
<执行>
```

- 第一行未闭合 `{` → prompt 变 `·` → 继续读取。
- 所有行拼接后一次性送 `session.run()`（Auto）或 `execute_shell_content()`（Shell）。

### 2.6 AI 模式

```
? list all rust files modified today
→ AI: find . -name "*.rs" -newermt today
  [Enter] 执行  [e] 编辑  [Esc] 取消
```

- F3 进入 AI 模式 → prompt 变 `?`。
- 输入自然语言 → 调用 LLM → 返回命令建议。
- 用户确认/编辑/取消 → 执行后回到之前模式。
- AI 模式是「一次性」（每次输入后回到之前的 Shell/Auto 模式），除非锁定。

> AI 模式的 LLM 集成本身是独立计划（Plan 291 Phase 3），本计划只做模式切换的框架。

### 2.7 按键绑定

| 键 | 作用 | reedline 事件 |
|---|---|---|
| **F1** | 锁定/解锁 Shell | `KeyBinding(F1) → ToggleMode(Shell)` |
| **F2** | 锁定/解锁 AutoScript | `KeyBinding(F2) → ToggleMode(Auto)` |
| **F3** | AI 模式（一次性） | `KeyBinding(F3) → EnterMode(AI)` |
| **Esc** | 解除锁定 | `KeyBinding(Esc) → UnlockMode` |
| **Ctrl+1/2/3** | F1/F2/F3 别名（笔记本友好） | 同上 |

### 2.8 模式状态

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Shell,
    AutoScript,
    AI,
}

pub struct ModeState {
    /// 当前锁定模式（None = 自动检测）。
    pub locked: Option<InputMode>,
    /// 上次自动检测到的模式（用于 AI 模式后恢复）。
    pub last_auto: InputMode,
    /// 多行缓冲（None = 单行模式）。
    pub multiline_buffer: Option<String>,
}
```

---

## 3. 实现阶段

### Phase 1: 改进表达式检测 + 自动回退

1. 重写 `looks_like_auto_expr` → `is_auto_expression`（保守、精确）。
2. 添加自动回退：Auto 失败 → 重试 Shell。
3. 添加 `is_arithmetic_expression` 辅助。
4. 单测：各种输入的分类正确性。

### Phase 2: 多行续行

1. `needs_continuation(text)` 语法检测。
2. REPL 读取循环：未闭合 → prompt 变 `·` → 继续读 → 拼接。
3. 多行 Shell（`\` 续行）+ 多行 Auto（未闭合 `{`）。
4. 集成到 Repl::run()。

### Phase 3: 模式锁定 + Prompt 符号

1. `ModeState` + `InputMode` 枚举。
2. F1/F2/Esc 键绑定（reedline Keybindings）。
3. Prompt 符号根据模式变化（`>` / `#` / `·` / `▌>`）。
4. 锁定状态传递到 `execute_with_fallback`。

### Phase 4: AI 模式框架

1. F3 键绑定 → 进入 AI 模式。
2. Prompt 变 `?`。
3. 输入 → 占位（调用 LLM 是 Plan 291 Phase 3 的工作）。
4. AI 返回后回到之前模式。

---

## 4. 关键文件

| 文件 | 改动 |
|---|---|
| `auto-shell/src/shell.rs` | `is_auto_expression` 替换 `looks_like_auto_expr`；`execute_with_fallback` 自动回退 |
| `auto-shell/src/frontend/repl.rs` | F1/F2/F3 键绑定；多行读取循环；prompt 模式符号 |
| `auto-shell/src/frontend/repl.rs::AshPrompt` | 根据 `ModeState` 渲染 prompt 符号 |
| `auto-shell/src/repl_mode.rs`（新） | `InputMode` + `ModeState` + `needs_continuation` |

---

## 5. 边界与风险

| 风险 | 应对 |
|---|---|
| Auto parser 太宽容（什么都能 parse） | 自动回退基于**执行失败**而非 parse 失败 |
| 回退导致双重错误信息 | 先报 Auto 错误（通常更有上下文），Shell 错误作为 hint |
| F 键在笔记本上需要 Fn | 同时支持 Ctrl+1/2/3 别名 |
| 多行检测误判（嵌套引号等） | 保守：宁可多续行，不可截断 |
| 已有 keybinding 冲突 | F1 当前未绑定；Ctrl+E 已用于编辑器，避开 |

---

## 6. 验证

- **单测**: `is_auto_expression` 对各种输入的分类；`needs_continuation` 的括号/引号检测。
- **集成**: `ash -c 'let x = 5'` → Auto 执行；`ash -c 'find .' → Shell 执行（不再误判）。
- **手动**: 交互式 `fn add(a) { a+1 }` 多行输入；F2 锁定后连续写 Auto 代码。
