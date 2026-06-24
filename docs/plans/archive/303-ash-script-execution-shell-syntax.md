# Plan 303: ASH Script Execution + Shell Syntax (`>`)

**Status:** ✅ Completed

## Context

ASH (Auto Shell) 目前只能以交互式 REPL 模式运行。用户希望：

1. **`ash hello.at`** — 直接执行 `.at` 脚本文件（类似 `auto hello.at`），为未来 CLI 工具链打基础
2. **脚本内可调用 shell 命令** — Auto 脚本在 Shell 场景下支持 `>` 前缀，表示"这一行是 shell 命令"
3. **`source` 命令支持 `.at` 脚本** — 已有基础，但需增强以支持 `>` 语法

### 设计决策

**`>` 前缀在 Shell 层处理，不改 Auto 语言核心解析器。** 理由：
- `>` 是 Shell 场景特有的扩展，`auto hello.at` 不需要它
- 避免修改 Auto 核心解析器、VM codegen、所有 transpiler
- 在 `source_file()` / 新的 `execute_script()` 中预处理即可

### 目标效果

```auto
// deploy.at — 一个 ASH 脚本示例
let dirs = ["github", "gitcode", "gitee"]

for d in dirs {
    > echo "Deploying $d..."
    > cd $d && git pull && cargo build
}

let output = > git status --porcelain
if output.len > 0 {
    > git add -A
    > git commit -m "auto deploy"
}
```

执行：`ash deploy.at`

---

## Implementation

### Step 1: CLI 参数解析（`ash hello.at`）

**文件：** `crates/auto-shell/src/main.rs`

当前 `main.rs` 没有参数解析，直接启动 REPL。改为：

```rust
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Script execution mode: ash hello.at [args...]
        let script_path = &args[1];
        let mut shell = Shell::new();
        shell.execute_script_file(script_path)?;
        return Ok(());
    }

    // Default: interactive REPL
    let mut repl = Repl::new()?;
    repl.run()?;
    Ok(())
}
```

不引入 clap 依赖——`ash` 是一个 shell，参数解析保持极简。如果第一个参数是文件路径且存在，执行脚本；否则启动 REPL。

### Step 2: 增强 `source_file()` → `execute_script_file()`

**文件：** `crates/auto-shell/src/shell.rs`

当前 `source_file()` 逐行执行，每行通过 `self.execute()`。需要增加 `>` 行处理：

```rust
/// Execute a script file with `>` shell syntax support.
pub fn execute_script_file(&mut self, path: &std::path::Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .into_diagnostic()?;

    self.execute_script_content(&content)
}

/// Execute script content with `>` shell syntax support.
fn execute_script_content(&mut self, content: &str) -> Result<()> {
    // 1. Pre-process: separate into Auto blocks and Shell (> prefix) lines
    // 2. Auto blocks → send to AutovmReplSession::run()
    // 3. Shell lines → interpolate $var, then Shell::execute()

    let mut auto_block = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if !auto_block.is_empty() {
                auto_block.push('\n');
            }
            continue;
        }

        // Shell line: starts with >
        if trimmed.starts_with('>') {
            // Flush accumulated Auto block first
            self.flush_auto_block(&mut auto_block)?;

            // Strip > prefix
            let cmd = trimmed[1..].trim();

            // Interpolate Auto variables ($var → value)
            let cmd = self.interpolate_auto_vars(cmd);

            // Execute as shell command
            let _ = self.execute(&cmd);
            continue;
        }

        // Regular Auto line: accumulate
        auto_block.push_str(line);
        auto_block.push('\n');
    }

    // Flush remaining Auto block
    self.flush_auto_block(&mut auto_block)?;

    Ok(())
}

/// Flush accumulated Auto code block to the VM.
fn flush_auto_block(&mut self, block: &mut String) -> Result<()> {
    if block.trim().is_empty() {
        return Ok(());
    }
    let result = self.session.run(block);
    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
    block.clear();
    Ok(())
}
```

**关键：** Auto 代码按"块"发送给 VM，不是逐行。这样多行构造（`fn`、`for`、`if`）能正常工作。Shell 行（`>`）则是逐行执行。

### Step 3: Auto 变量插值

**文件：** `crates/auto-shell/src/shell.rs` + `crates/auto-lang/src/autovm_persistent.rs`

`>` 行中的 `$var` 需要引用 Auto VM 中的变量值，而非 Shell 环境变量。

**3a. 在 `AutovmReplSession` 中添加变量导出方法：**

```rust
// crates/auto-lang/src/autovm_persistent.rs

/// Get a string representation of a local variable's value.
/// Used by ASH to interpolate $var in shell-command lines.
pub fn get_var_string(&self, name: &str) -> Option<String> {
    let codegen = self.codegen.as_ref()?;
    let local_idx = codegen.locals.get(name)?;

    // Read from VM stack at bp + local_idx
    let task = self.vm.tasks.get(self.main_task_id)?;
    let bp = task.bp as usize;
    let stack_pos = bp + local_idx;

    if stack_pos < task.stack.len() {
        let value = task.stack[stack_pos];
        Some(self.format_value(value))
    } else {
        None
    }
}

/// Get all local variable names (for bulk export).
pub fn get_all_vars(&self) -> HashMap<String, String> {
    let Some(codegen) = &self.codegen else { return HashMap::new(); };
    let Some(task) = self.vm.tasks.get(self.main_task_id) else { return HashMap::new(); };

    let bp = task.bp as usize;
    let mut vars = HashMap::new();
    for (name, idx) in &codegen.locals {
        let pos = bp + idx;
        if pos < task.stack.len() {
            vars.insert(name.clone(), self.format_value(task.stack[pos]));
        }
    }
    vars
}
```

**3b. 在 `Shell` 中实现插值方法：**

```rust
/// Replace $var in shell command text with Auto VM variable values.
/// Falls back to shell variables if not found in VM.
fn interpolate_auto_vars(&self, cmd: &str) -> String {
    let mut result = String::new();
    let mut chars = cmd.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // Read variable name
            let mut name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    name.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                result.push('$');
                continue;
            }

            // Priority: Auto VM vars > Shell vars > Env vars
            let value = self.session.get_var_string(&name)
                .or_else(|| self.vars.get_local(&name).cloned())
                .or_else(|| self.vars.get_env(&name))
                .unwrap_or_default();

            result.push_str(&value);
        } else {
            result.push(c);
        }
    }

    result
}
```

### Step 4: 更新 `source_file()` 复用新逻辑

**文件：** `crates/auto-shell/src/shell.rs`

现有的 `source_file()` 用于 `.ashrc`（纯 shell 脚本，不含 Auto 代码）。保持其行为不变，但将 `>` 行处理逻辑也加入（向后兼容）：

```rust
pub fn source_file(&mut self, path: &std::path::Path) -> Result<()> {
    let content = std::fs::read_to_string(path).into_diagnostic()?;
    // Use the new method — handles both shell lines (>) and regular lines
    self.execute_script_content(&content)
}
```

这样 `source` 命令也能加载含 `>` 语法的脚本。

### Step 5: 赋值捕获（可选增强）

`> ` 行的 stdout 可以赋值给 Auto 变量：

```auto
let files = > ls src/        // files 得到 shell 输出字符串
```

这需要在 `execute_script_content()` 中识别 `let/var name = > ...` 模式：

```rust
// 检测 let x = > cmd 模式
if let Some(rest) = trimmed.strip_prefix("let ") || stripped.strip_prefix("var ") {
    if let Some(eq_pos) = rest.find("= >") {
        let var_name = rest[..eq_pos].trim();
        let cmd = rest[eq_pos + 3..].trim();
        let cmd = self.interpolate_auto_vars(cmd);
        let output = self.execute(&cmd)?;
        // 把 output 设入 Auto VM
        self.session.run(&format!("let {} = \"{}\"", var_name, escape_for_auto(output)));
    }
}
```

---

## 文件变更总结

| 文件 | 改动 |
|------|------|
| `crates/auto-shell/src/main.rs` | 添加 CLI 参数检测，支持 `ash hello.at` |
| `crates/auto-shell/src/shell.rs` | 添加 `execute_script_file()`、`execute_script_content()`、`interpolate_auto_vars()`、`flush_auto_block()` |
| `crates/auto-lang/src/autovm_persistent.rs` | 添加 `get_var_string()`、`get_all_vars()` 方法 |
| `crates/ash-core/src/shell/mod.rs` | 可能需要 re-export（检查现有导出） |

## Verification

1. `cargo build -p auto-shell` — 编译通过
2. 创建测试脚本 `tmp/test_shell.at`：
   ```auto
   let name = "world"
   > echo "Hello $name"
   ```
3. `ash tmp/test_shell.at` — 输出 "Hello world"
4. 交互模式下 `source tmp/test_shell.at` — 同样工作
5. 更复杂的脚本（for 循环 + `>` 行）— 正确执行
6. `ash` 无参数 — 仍然进入交互 REPL
