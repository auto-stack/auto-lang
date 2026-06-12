# Plan 302: ASH 日常可用 Shell — 完整功能补充路线图

## Context

ASH (Auto Shell) 已具备 75 个内置命令、结构化管道、后台任务、Tab 补全、模块化 Prompt、历史记录+灰色提示（Plan 281）等功能。但对比 bash/zsh/fish，缺失多个日常必需的 shell 特性，导致用户 5 分钟内就会切回 bash。

本计划按优先级将所有缺失功能分为 4 个 Phase，每个 Phase 内的步骤有明确的依赖关系。

---

## 当前解析架构速览

```
用户输入
  │
  ▼
shell.rs::execute()
  ├─ reap_jobs()                     // 清理已完成的后台任务
  ├─ 检测 background &               // cmd &
  ├─ job control builtins            // jobs/fg/bg
  ├─ looks_like_auto_expr()          // AutoLang 表达式检测
  ├─ expand_variables()              // $VAR / ${VAR} 展开  ✅ 已实现
  ├─ parse_pipeline() → Vec<String>  // 按 | 分割            ⚠️ 简陋
  ├─ parse_redirect()                // I/O 重定向           ❌ 空壳
  └─ execute_single_command()
       └─ registry.get(cmd) or 外部命令
```

**关键文件：**
- `crates/ash-core/src/parser/pipeline.rs` — 管道解析（纯字符串分割）
- `crates/ash-core/src/parser/redirect.rs` — 重定向解析（空壳 stub）
- `crates/ash-core/src/parser/quote.rs` — 引号感知的参数解析（已完善）
- `crates/ash-core/src/parser/history.rs` — 历史展开（已完善，未接入）
- `crates/auto-shell/src/shell.rs` — Shell 执行引擎（1102 行）
- `crates/auto-shell/src/frontend/repl.rs` — REPL 循环

**核心问题：** 没有统一的 Shell Tokenizer/AST，各解析器独立工作在字符串上。

---

## Phase 1: 让 Shell 能干活（P0 — 阻塞日常使用）

### Step 1.1: I/O 重定向 (`>`, `>>`, `<`, `2>`, `2>&1`)

**文件：** `crates/ash-core/src/parser/redirect.rs`

当前是空壳：
```rust
// Line 14-16 — 直接返回 None
pub fn parse_redirect(input: &str) -> (String, Option<Redirect>) {
    // TODO: Phase 2
    (input.to_string(), None)
}
```

**实现方案：**

1. 扩展 `Redirect` 结构体：
```rust
pub struct Redirect {
    pub stdin: Option<String>,       // < file
    pub stdout: Option<String>,      // > file
    pub append_stdout: bool,         // true for >>
    pub stderr: Option<StderrRedirect>,  // 2> / 2>> / 2>&1
}

pub enum StderrRedirect {
    File(String),
    Append(String),
    ToStdout,  // 2>&1
}
```

2. 实现引号感知的重定向解析（不处理引号内的 `>` / `<`）

3. 在 `shell.rs::execute_single_command()` 中应用重定向：
   - 内置命令：暂不支持（结构化输出不经过文件）
   - 外部命令：通过 `Stdio::piped()` / `Stdio::file()` 实现
   - 混合管道：在管道的最后一个/第一个节点应用重定向

**依赖：** 无

### Step 1.2: `&&` / `||` 链式执行

**文件：** `crates/ash-core/src/parser/pipeline.rs`

当前只识别 `|`，把 `||` 误认为两个管道。

**实现方案：**

1. 引入操作符枚举：
```rust
pub enum ChainOp {
    Pipe,        // |
    And,         // &&
    Or,          // ||
}

pub struct ChainSegment {
    pub command: String,
    pub op: Option<ChainOp>,  // 连接到下一段的操作符
}
```

2. 修改 `parse_pipeline()` 为 `parse_chain()` — 逐字符解析，区分 `|` / `||` / `&` / `&&`

3. 在 `shell.rs` 中实现短路求值：
```rust
fn execute_chain(&mut self, segments: Vec<ChainSegment>) -> Result<Option<String>> {
    let mut result = None;
    let mut last_success = true;

    for seg in segments {
        match seg.op {
            Some(ChainOp::And) if !last_success => continue,   // && 短路
            Some(ChainOp::Or) if last_success => continue,     // || 短路
            _ => {
                result = self.execute_single_or_pipe(&seg.command)?;
                last_success = self.last_exit_code == 0;
            }
        }
    }
    Ok(result)
}
```

**注意：** 需要确保 `|` 管道优先级高于 `&&` / `||`（bash 行为）。
例如 `a | b && c` = `(a | b) && c`

**依赖：** 无

### Step 1.3: RC 文件加载（`~/.ashrc`）

**文件：** `crates/auto-shell/src/frontend/repl.rs`（新增初始化逻辑）

**实现方案：**

1. 在 `Repl::new()` 中检测 `~/.ashrc`，存在则逐行执行：
```rust
fn load_rc_file(shell: &mut Shell) {
    let rc_path = dirs::home_dir().map(|p| p.join(".ashrc"));
    if let Some(path) = rc_path {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') { continue; }
                    let _ = shell.execute(line);
                }
            }
        }
    }
}
```

2. 同时支持项目级 `.ashrc`（在 `cd` 时检测并加载，类似 `.envrc`）

3. 在 `Shell` 结构体中记录已加载的 RC 文件，防止重复加载

**依赖：** Step 1.4（alias 命令，因为 RC 文件主要用途之一就是定义 alias）

### Step 1.4: Alias 系统

**文件：**
- `crates/auto-shell/src/shell.rs` — 添加 `aliases: HashMap<String, String>`
- `crates/auto-shell/src/cmd/` — 添加 `alias` / `unalias` 命令
- `crates/ash-core/src/parser/` — 添加 alias 展开逻辑

**实现方案：**

1. 在 `Shell` 结构体添加：
```rust
aliases: HashMap<String, String>,
```

2. 实现 `alias` / `unalias` 命令：
   - `alias ll='ls -la'` — 定义别名
   - `alias` — 列出所有别名
   - `unalias ll` — 删除别名

3. 在执行管线中添加 alias 展开（在变量展开之前）：
```rust
fn expand_aliases(&self, input: &str) -> String {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    if let Some(cmd) = parts.first() {
        if let Some(expansion) = self.aliases.get(*cmd) {
            if let Some(rest) = parts.get(1) {
                return format!("{} {}", expansion, rest);
            }
            return expansion.clone();
        }
    }
    input.to_string()
}
```

4. 防递归：检测循环别名（A→B→A），设置最大展开深度（如 10 层）

**依赖：** 无

---

## Phase 2: 让 Shell 舒服（P1 — 严重影响体验）

### Step 2.1: 通配符自动展开（`*`, `?`, `**/`）

**文件：** `crates/ash-core/src/parser/quote.rs`（或新增 `glob.rs`）

**实现方案：**

1. 在 `ash-core/Cargo.toml` 添加 `glob` crate 依赖

2. 在 `parse_args()` 之后添加 glob 展开阶段：
```rust
fn expand_globs(args: Vec<String>, cwd: &Path) -> Vec<String> {
    args.into_iter().flat_map(|arg| {
        if arg.contains('*') || arg.contains('?') {
            // 相对路径转绝对路径
            let pattern = if arg.starts_with('/') {
                arg.clone()
            } else {
                format!("{}/{}", cwd.display(), arg)
            };
            match glob::glob(&pattern) {
                Ok(paths) => {
                    let expanded: Vec<_> = paths
                        .filter_map(|p| p.ok())
                        .map(|p| p.display().to_string())
                        .collect();
                    if expanded.is_empty() { vec![arg] } else { expanded }
                }
                Err(_) => vec![arg],
            }
        } else {
            vec![arg]
        }
    }).collect()
}
```

3. 关键规则（和 bash 一致）：
   - **引号内的通配符不展开**：`echo "*.rs"` → 字面量 `*.rs`
   - **无匹配时保留原样**：`ls *.xyz`（无文件） → 传 `*.xyz` 给 ls
   - **排序**：展开结果按字母排序

4. 在 `shell.rs` 的执行管线中调用（在变量展开之后、命令执行之前）

**依赖：** 无

### Step 2.2: Tilde 展开（`~`, `~/path`, `~user`）

**文件：** `crates/auto-shell/src/shell.rs`

当前只在 `cd` 内硬编码处理 `~`。

**实现方案：**

1. 添加通用 tilde 展开（在 alias 展开之后、变量展开之前）：
```rust
fn expand_tilde(&self, input: &str) -> String {
    // 逐词处理：只有独立的 ~ 或 ~/ 开头的词才展开
    // 不展开引号内的 ~
    let mut result = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut word_start = true;
    // ... 字符扫描，遇到 word_start + '~' 则替换为 home_dir
}
```

2. 规则：
   - `~` → `/home/user`
   - `~/docs` → `/home/user/docs`
   - `~other` → 读取 `/etc/passwd`（Unix）或跳过（Windows）
   - `"~"` → 不展开

**依赖：** 无

### Step 2.3: 多行输入（`\` 续行 + 引号续行）

**文件：** `crates/auto-shell/src/frontend/repl.rs`

**实现方案：**

1. 在 `Repl::run()` 的 `Signal::Success` 分支添加多行检测：
```rust
let mut buffer = line;
loop {
    let trimmed = buffer.trim_end();
    if trimmed.ends_with('\\') {
        // 行尾反斜杠 — 续行
        buffer.truncate(buffer.trim_end().len() - 1);
        buffer.push(' ');
        let continuation = self.line_editor.read_line(&continuation_prompt)?;
        match continuation {
            Ok(Signal::Success(next_line)) => buffer.push_str(&next_line),
            _ => break,
        }
    } else if has_unclosed_quote(&buffer) {
        // 未闭合引号 — 续行
        buffer.push('\n');
        let continuation = self.line_editor.read_line(&continuation_prompt)?;
        match continuation {
            Ok(Signal::Success(next_line)) => {
                buffer.push_str(&next_line);
            }
            _ => break,
        }
    } else {
        break;
    }
}
```

2. 续行 prompt 使用 `..>` （与当前代码中的 `continuation_prompt` 一致）

3. `has_unclosed_quote()` 辅助函数：统计奇数个引号即为未闭合

**依赖：** 无

### Step 2.4: 命令替换（`$(cmd)` 和 `` `cmd` ``）

**文件：** `crates/auto-shell/src/shell.rs`

**实现方案：**

1. 在变量展开函数中增加 `$()` 处理（和 `${VAR}` 同级）：
```rust
fn expand_command_substitution(&mut self, input: &str) -> Result<String> {
    // 查找 $( ... ) 对，支持嵌套
    // 提取内部命令，调用 self.execute() 获取输出
    // 用输出替换 $( ... )
}
```

2. Backtick 语法：将 `` `cmd` `` 转换为 `$(cmd)` 后走同一逻辑

3. 规则：
   - `echo "dir: $(pwd)"` → `echo "dir: /home/user"`
   - 嵌套支持：`echo $(basename $(pwd))`
   - 尾部换行去除（和 bash 一致）

**依赖：** Step 1.2（`&&`/`||`，因为替换中的命令可能用到）

---

## Phase 3: 让 Shell 精致（P2 — 锦上添花）

### Step 3.1: 语法高亮

**文件：** `crates/auto-shell/src/frontend/` 新增 highlighter

reedline 提供 `Highlighter` trait：
```rust
pub trait Highlighter: Send {
    fn highlight(&self, line: &str, cursor: usize) -> StyledString;
}
```

**实现方案：**

1. 实现 `AshHighlighter`，按 token 类型着色：

| Token 类型 | 颜色 |
|-----------|------|
| 命令名（内置） | Cyan + Bold |
| 命令名（外部） | Green |
| 字符串（引号内） | Yellow |
| 标志（`-f`, `--flag`） | Blue |
| 管道/操作符 | Magenta |
| 变量（`$VAR`） | Red |
| 普通参数 | Default |

2. 需要判断首词是否为内置命令 — 使用 `CommandRegistry` 的快照

3. 挂载：`Reedline::create().with_highlighter(highlighter)`

**依赖：** 无

### Step 3.2: Vi 编辑模式

**文件：** `crates/auto-shell/src/frontend/repl.rs`

reedline 内置支持，只需添加切换机制：

1. 将 `Emacs::new(keybindings)` 改为根据配置选择模式：
```rust
let edit_mode: Box<dyn EditMode> = match config.edit_mode {
    EditMode::Emacs => Box::new(Emacs::new(keybindings)),
    EditMode::Vi => Box::new(Vi::new(keybindings)),
};
```

2. 在 `~/.ashrc` 中支持 `set editing-mode vi`

3. 添加 `bind` 内置命令用于运行时切换

**依赖：** Step 1.3（RC 文件）

### Step 3.3: `source` 命令

**文件：** `crates/auto-shell/src/cmd/` 新增 `source.rs`

**实现方案：**

1. 注册 `source` 命令，接受文件路径参数
2. 读取文件，逐行执行（同 RC 文件逻辑）
3. 支持 `. file` 语法（bash 兼容）

**依赖：** Step 1.3（RC 文件加载的逐行执行逻辑可复用）

### Step 3.4: 逐词接受 Hint（Ctrl+→）

**文件：** `crates/auto-shell/src/frontend/repl.rs`

reedline 的 `Hinter` trait 已支持 `next_hint_token()`，只需绑定按键：

```rust
keybindings.add_binding(
    KeyModifiers::CONTROL,
    KeyCode::Right,
    ReedlineEvent::Edit(vec![EditCommand::MoveWordRight]),
);
```

需要确认 reedline 的 `MoveWordRight` 是否会自动触发 `next_hint_token()`。如果不会，需要自定义事件。

**依赖：** Plan 281（已完成的 autosuggestion）

### Step 3.5: `pushd` / `popd` / `dirs`

**文件：** `crates/auto-shell/src/cmd/` 新增 `dirstack.rs`

**实现方案：**

1. 在 `Shell` 中添加 `dir_stack: Vec<PathBuf>`
2. 实现 `pushd`（cd + 压栈）、`popd`（弹栈 + cd）、`dirs`（显示栈）
3. 已有 `BookmarkManager` 可复用部分逻辑

**依赖：** 无

---

## Phase 4: 架构优化（长期）

### Step 4.1: 统一 Shell Lexer

当前各解析器独立工作在字符串上，没有统一词法分析。长期应引入统一的 Shell Lexer：

```rust
pub enum ShellToken {
    Word(String),
    Pipe,              // |
    LogicalAnd,        // &&
    LogicalOr,         // ||
    RedirectIn,        // <
    RedirectOut,       // >
    RedirectAppend,    // >>
    StderrRedirect,    // 2>
    Background,        // &
    Semicolon,         // ;
    Newline,           // \n
}
```

**好处：**
- `&&`/`||`、重定向、管道的解析统一在 lexer 层完成
- 消除各解析器间的重复引号处理
- 为语法高亮提供 token 流
- 为未来添加 `()` 子 shell、`{}` 命令组等高级特性打基础

**依赖：** Phase 1 + Phase 2 全部完成后，作为重构进行

### Step 4.2: 配置系统完善

当前只有 `~/.config/ash-prompt.toml` 控制提示符。应扩展为完整的 `ash.toml`：

```toml
[shell]
history_size = 10000
autosuggestion = true
autosuggestion_min_chars = 1
edit_mode = "emacs"  # or "vi"
color_autosuggestion = "dark_gray"

[aliases]
ll = "ls -la"
la = "ls -a"
gs = "git status"

[completion]
case_sensitive = false

[plugins]
# 未来插件系统
```

**依赖：** Step 1.3, Step 1.4

---

## 执行管线总览（所有 Phase 完成后）

```
用户输入
  │
  ▼
┌─────────────────────────────────────┐
│  1. History Expansion               │  !! / !n / !string
│     (history.rs — 已有，需接入)       │
├─────────────────────────────────────┤
│  2. Alias Expansion                 │  ll → ls -la
│     (shell.rs — Phase 1.4)          │
├─────────────────────────────────────┤
│  3. Tilde Expansion                 │  ~ → /home/user
│     (shell.rs — Phase 2.2)          │
├─────────────────────────────────────┤
│  4. Variable Expansion              │  $VAR / ${VAR}  ✅ 已有
│     (shell.rs)                       │
├─────────────────────────────────────┤
│  5. Command Substitution            │  $(cmd) / `cmd`
│     (shell.rs — Phase 2.4)          │
├─────────────────────────────────────┤
│  6. Glob Expansion                  │  *.rs → file1.rs file2.rs
│     (quote.rs — Phase 2.1)          │
├─────────────────────────────────────┤
│  7. Redirect Parsing                │  > file / >> file / < file
│     (redirect.rs — Phase 1.1)       │
├─────────────────────────────────────┤
│  8. Chain/Pipeline Parsing          │  | / && / ||
│     (pipeline.rs — Phase 1.2)       │
├─────────────────────────────────────┤
│  9. Command Dispatch                │
│     ├─ 内置命令 (registry)           │
│     ├─ 外部命令 (exec)              │
│     └─ AutoLang 表达式 (vm)         │
└─────────────────────────────────────┘
```

---

## Phase 里程碑总结

| Phase | 包含步骤 | 预估工作量 | 里程碑 |
|-------|---------|-----------|--------|
| **Phase 1** | Step 1.1–1.4 | 3–5 天 | ASH 可替代 bash 执行日常命令 |
| **Phase 2** | Step 2.1–2.4 | 2–3 天 | ASH 使用体验流畅 |
| **Phase 3** | Step 3.1–3.5 | 3–4 天 | ASH 具备现代化 shell 体感 |
| **Phase 4** | Step 4.1–4.2 | 3–5 天 | 架构可维护、可扩展 |

## Verification（每个 Step 完成后的验证方式）

| Step | 验证命令 | 期望行为 |
|------|---------|---------|
| 1.1 | `echo hello > /tmp/test.txt && cat /tmp/test.txt` | 文件包含 `hello` |
| 1.2 | `true && echo yes` / `false && echo no` | 输出 `yes` / 无输出 |
| 1.3 | 写 `~/.ashrc` 含 `echo "loaded"` 后启动 ash | 显示 `loaded` |
| 1.4 | `alias ll='ls -la'` 然后 `ll` | 等同 `ls -la` |
| 2.1 | `echo *.rs` | 展开为文件列表 |
| 2.2 | `echo ~/Documents` | 展开为绝对路径 |
| 2.3 | `echo "hello \` + Enter + `world"` | 输出 `hello world` |
| 2.4 | `echo "dir: $(pwd)"` | 输出当前目录 |
| 3.1 | 输入 `ls -la` | 命令名/参数/标志有不同颜色 |
| 3.3 | `source ~/.ashrc` | 重新加载配置 |
