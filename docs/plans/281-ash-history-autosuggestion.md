# Plan 302: ASH Fish-style History Autosuggestion

## Context

ASH 当前没有基于历史的快速提示功能。用户必须完整输入命令或使用 Tab 补全。Fish Shell、PowerShell PSReadLine、zsh-autosuggestions 都提供了"灰色幽灵文字"——根据历史记录自动提示完整命令，按一个键即可接受。

经过研究发现，ASH 使用的 **reedline 0.44.0** 已经内置了两个 Hinter 实现：
- **`DefaultHinter`** — 基于历史的前缀匹配提示
- **`CwdAwareHinter`** — 同上，但额外考虑当前工作目录（更精准）

我们只需把它们接入 ASH 的 REPL 即可，无需从头实现匹配算法。

## Fish Shell Autosuggestion 核心设计参考

| 方面 | Fish 的做法 | ASH 的选择 |
|------|------------|-----------|
| **匹配算法** | 前缀精确匹配（非模糊），大小写敏感优先 | 使用 reedline 内置 `CwdAwareHinter` |
| **提示来源** | 历史 > Tab补全 > 大小写不敏感历史 | reedline 内置策略（历史优先） |
| **接受全条** | → / Ctrl+F / End / Ctrl+E | **Ctrl+F**（主）+ **→**（辅）+ End |
| **接受一词** | Alt+→ / Alt+F / Ctrl+→ | Ctrl+→（未来可加） |
| **样式** | ANSI 256色 555（暗灰） | `nu_ansi_term::Color::DarkGray` |
| **历史格式** | YAML（cmd + when + paths） | reedline `FileBackedHistory`（已有） |
| **去重** | LRU + reverse-iteration dedup | reedline 内置 |
| **最少触发字符** | 无硬性限制 | 设为 1（输入第一个字符即开始提示） |

### 快捷键选择理由

- **Ctrl+F**：左手小指 Ctrl + 右手食指 F，双手不离开主键盘区，语义直觉（Forward）
- **→**：Fish/PowerShell 用户习惯的默认键，作为兼容备选
- **End**：语义"跳到行尾"，自然接受全部建议
- **不用 Tab**：Tab 用于正常的命令/文件补全，避免冲突

## Implementation

### 修改文件

**1. `crates/auto-shell/src/frontend/repl.rs`**（核心改动，约 15 行）

```rust
// 新增 import
use reedline::{CwdAwareHinter, DefaultHinter};
use nu_ansi_term::Color;

// 在 Repl::new() 中，Reedline::create() 链式调用加入 hinter
let hinter = Box::new(
    CwdAwareHinter::default()
        .with_style(Color::DarkGray.bold())  // 暗灰色提示
        .with_min_chars(1)                    // 输入 1 字符即触发
);

let line_editor = Reedline::create()
    .with_history(history)
    .with_completer(completer)
    .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
    .with_quick_completions(true)
    .with_partial_completions(true)
    .with_edit_mode(edit_mode)
    .with_hinter(hinter);  // ← 新增这一行
```

**2. 按键绑定**（同文件 `repl.rs`，在 keybindings 设置区域）

```rust
// Ctrl+F → 接受整条 autosuggestion
keybindings.add_binding(
    KeyModifiers::CONTROL,
    KeyCode::Char('f'),
    ReedlineEvent::Edit(vec![EditCommand::Complete]),
);

// → (Right Arrow) → 接受整条 autosuggestion（Fish 兼容）
// 注：reedline 默认已绑定 → 为 forward-char，在行尾自动接受 hint
// 如果需要显式绑定，可添加：
// keybindings.add_binding(
//     KeyModifiers::NONE,
//     KeyCode::Right,
//     ReedlineEvent::Edit(vec![EditCommand::Complete]),
// );
```

### 不需要改动的部分

- **历史存储**：`FileBackedHistory` 已在 `~/.auto-shell-history` 持久化，hinter 直接读取
- **历史匹配算法**：`CwdAwareHinter` 内置实现，通过 `&dyn History` trait 访问历史
- **渲染**：reedline 自动在光标后渲染灰色 hint 文字，无需手动处理 ANSI 转义
- **逐词接受**：reedline 的 `next_hint_token()` 已支持，`Ctrl+→` 可后续添加

### 未来增强（不在本次范围）

- 自定义 `AshHinter` 实现：按频率/最近使用排序、路径验证
- `Ctrl+→` 逐词接受
- 配置文件控制（颜色、最少触发字符数、开关）
- 大小写不敏感 fallback（类似 Fish 的 icase_history_result）

## Verification

1. `cargo build -p auto-shell` — 编译通过
2. 运行 `ash`，输入几条命令后退出
3. 重新启动 `ash`，输入之前命令的前几个字符，应看到灰色提示
4. 按 **Ctrl+F** 或 **→** 应接受完整建议
5. 按 **Tab** 应仍然触发正常的补全菜单（不冲突）
6. 继续打字应让灰色提示自动更新或消失
