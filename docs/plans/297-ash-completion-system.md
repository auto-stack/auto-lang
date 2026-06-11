# Plan: ASH 外部命令参数自动补全系统

## Context

ASH shell 的补全系统存在严重断层：
- 命令补全使用硬编码 22 个命令数组，而 CommandRegistry 注册了 77+ 个命令
- Flag 补全是 STUB（`return Vec::new()`）
- 补全引擎无法访问 Shell 状态（注册表、工作目录、环境变量）
- 外部命令（git、cargo 等）无任何参数补全

本计划分两阶段：
1. **Phase A**（基础）：打通 CommandRegistry → 补全引擎，让内置命令的 flag/arg 能补全
2. **Phase B**（核心）：Atom 格式声明式补全定义 + 外部命令参数补全

## Phase A：打通 CommandRegistry → 补全引擎

### A1. ShellCompleter 变为有状态

**文件**: `crates/auto-shell/src/frontend/completions_reedline.rs`

```rust
pub struct ShellCompleter {
    signatures: Vec<CompletionSignature>,  // 注册表快照
}
```

- `ShellCompleter::new(signatures)` 接收 Signature 快照
- `complete()` 调用新的 `get_completions_with_context(line, &signatures)` 替代 `get_completions(line)`

### A2. CompletionSignature 类型（ash-core 内）

**新文件**: `crates/ash-core/src/completions/types.rs`

```rust
pub struct CompletionSignature {
    pub name: String,
    pub description: String,
    pub arguments: Vec<CompletionArgument>,
}

pub struct CompletionArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub is_flag: bool,
    pub short: Option<char>,
}
```

避免移动 `auto-shell/src/cmd.rs` 的 `Signature` 到 `ash-core`（保持依赖方向正确）。在 `auto-shell` 添加 `From<Signature> for CompletionSignature`。

### A3. 重写命令补全

**文件**: `crates/ash-core/src/completions/command.rs`

- 删除 `BUILTIN_COMMANDS` 硬编码数组
- `complete_command(input, signatures)` 从 signatures 动态生成补全
- 包含描述信息（`Completion::with_description`）

### A4. 新增 Flag 补全

**新文件**: `crates/ash-core/src/completions/flag.rs`

```rust
pub fn complete_flags(
    prefix: &str,
    command_name: &str,
    signatures: &[CompletionSignature],
    already_set: &[String],
) -> Vec<Completion>
```

- 查找 command 的 Signature
- 过滤 `is_flag` 的 Argument
- 匹配 `--name` 或 `-short` 前缀
- 排除已设置的 flag
- 返回 `Completion { kind: Flag }`，带描述

### A5. 上下文感知调度

**文件**: `crates/ash-core/src/completions/mod.rs`

新增 `get_completions_with_context(input, signatures)`：
- 与现有 `get_completions` 相同逻辑，但：
  - 命令名补全用 signatures 替代 BUILTIN_COMMANDS
  - `-` 开头走 `flag::complete_flags()`
  - 子命令位置走新的子命令检测
- 保留原 `get_completions()` 兼容（或统一后删除）

### A6. REPL 集成

**文件**: `crates/auto-shell/src/frontend/repl.rs`

```rust
let signatures = shell.registry().params();  // Vec<Signature>
let completion_sigs: Vec<CompletionSignature> = signatures.into_iter().map(Into::into).collect();
let completer = Box::new(ShellCompleter::new(completion_sigs));
```

### A7. 模块注册

- `crates/ash-core/src/completions/mod.rs` — 新增 `pub mod types; pub mod flag;`
- `crates/auto-shell/src/completions.rs` — re-export 新模块

### Phase A 验证

- `ls --<tab>` → 显示 `--all`, `--long`, `--human-readable` 等（带描述）
- `ls -<tab>` → 显示 `-a`, `-l`, `-h`, `-t`, `-r`, `-R`
- `gre<tab>` → 补全为 `grep`
- 空行 `<tab>` → 显示全部 77+ 命令（带描述）
- `cargo test -p ash-core` → 现有补全测试通过

---

## Phase B：Atom 格式外部命令补全

### B1. CompletionSpec 数据结构

**新文件**: `crates/ash-core/src/completions/spec.rs`

核心类型：

```rust
pub struct CompletionSpec {
    pub command: String,
    pub desc: Option<String>,
    pub subcommands: Vec<SubcommandSpec>,
    pub flags: Vec<FlagSpec>,
    pub args: Vec<ArgSpec>,
}

pub struct SubcommandSpec {
    pub name: String,
    pub desc: Option<String>,
    pub flags: Vec<FlagSpec>,
    pub args: Vec<ArgSpec>,
    pub subcommands: Vec<SubcommandSpec>,  // 嵌套（如 git remote add）
}

pub struct FlagSpec {
    pub short: Option<String>,   // "b"
    pub long: Option<String>,    // "branch"
    pub desc: Option<String>,    // "Create new branch"
    pub arg: Option<String>,     // 若存在，flag 带值
}

pub struct ArgSpec {
    pub position: usize,         // 位置参数序号（0 起始），特殊值 999 = 任意位置
    pub repeat: bool,            // 是否可重复（如 git add file1 file2 ...）
    pub name: Option<String>,
    pub desc: Option<String>,
    pub when: Option<WhenCondition>,
    pub source: Option<CompletionSource>,
}

pub enum WhenCondition {
    FlagsPresent(Vec<String>),
    FlagsAbsent(Vec<String>),
    Subcommand(String),
    PrevArg(String),
}

pub enum CompletionSource {
    Static(Vec<String>),
    Command { cmd: String, parse: ParseMode },
    Files { filter: Option<String> },
    Directories,
    Variables,
}

pub enum ParseMode {
    Line,           // 每行一个候选项（去掉前导空白和 * 标记）
    Field(usize),   // 按空白分割取第 N 个字段
    Json(String),   // JSON path 提取
}
```

### B2. CompletionProvider 补全引擎

**新文件**: `crates/ash-core/src/completions/provider.rs`

```rust
pub struct CompletionProvider {
    specs: HashMap<String, CompletionSpec>,
    cache: HashMap<String, (Instant, Vec<String>)>,  // TTL 缓存
}

impl CompletionProvider {
    pub fn register(&mut self, spec: CompletionSpec);
    pub fn has_spec(&self, command: &str) -> bool;

    /// 核心方法：解析上下文 → 匹配条件 → 获取候选项 → 过滤前缀
    pub fn resolve(
        &self,
        parts: &[&str],       // 已输入的 token
        cursor_part: usize,    // 光标所在 token 索引
        prefix: &str,          // 当前 token 的已输入部分
        already_flags: &[String],  // 已输入的 flags
        ctx: &CompletionContext,
    ) -> Vec<Completion>;
}

pub struct CompletionContext {
    pub current_dir: PathBuf,
    pub command_executor: Box<dyn Fn(&str, &Path) -> Result<String, String>>,
}
```

**resolve() 算法**：
1. 用 `parts[0]` 查找 `CompletionSpec`
2. 遍历 `parts[1..]` 匹配 subcommand 链，确定当前节点
3. 判断光标位置类型：flag 名 / flag 值 / 子命令 / 位置参数
4. 对位置参数：按 `position` 匹配，评估 `when` 条件
5. 解析 `source`：Static 直接用，Command 通过 `ctx.command_executor` 执行
6. 按 `prefix` 过滤候选，包装为 `Completion` 返回

### B3. ShellCompleter 集成 Provider

**文件**: `crates/auto-shell/src/frontend/completions_reedline.rs`

```rust
pub struct ShellCompleter {
    signatures: Vec<CompletionSignature>,
    provider: CompletionProvider,
    current_dir: PathBuf,
}
```

`complete()` 路由逻辑：
```
if parts[0] 在 provider.specs 中 → provider.resolve()
elif parts[0] 在 signatures 中 → get_completions_with_context()（内置命令）
else → file::complete_file()（文件补全）
```

### B4. REPL 同步补全状态

**文件**: `crates/auto-shell/src/frontend/repl.rs`

问题：reedline 拥有 completer 所有权，无法从 Repl 直接更新 completer 状态。

方案：使用 `Arc<Mutex<CompletionState>>` 共享状态：
```rust
pub struct CompletionState {
    pub current_dir: PathBuf,
}

// ShellCompleter 持有 Arc<Mutex<CompletionState>>
// Repl 持有同一个 Arc，每次命令后更新 current_dir
```

### B5. 内置补全定义

**新文件**: `crates/auto-shell/src/completions/definitions/`

```
definitions/
    mod.rs      — register_all(provider) 入口
    git.rs      — git CompletionSpec
    cargo.rs    — cargo CompletionSpec
```

每个文件用 Rust 代码构建 `CompletionSpec`（后续可改为加载 `.atom` 文件）。

**git 补全定义涵盖**：
- 子命令：add, branch, checkout, clone, commit, diff, fetch, log, merge, pull, push, rebase, remote, reset, stash, status, switch, tag
- checkout args: branch 列表（`source: { command: "git branch --list" }`），`-b` 时无补全
- push args: remote → branch（`source: { command: "git remote" }` + `source: { command: "git branch --list" }`）
- add args: 修改文件（`source: { command: "git status --porcelain", parse: field(1) }`）

### B6. CompletionKind 传递到菜单

当前 `completion_to_suggestion()` 把 `kind` 丢弃了。修复：
- `CompletionKind` 编码到 `Suggestion.extra[0]`（已有 `extra` 字段传 "fuzzy"）
- `AshMenu` 从 `extra[0]` 读取 kind，正确着色

### Phase B 验证

- `git <tab>` → 显示子命令列表（add, branch, checkout...）带描述
- `git checkout <tab>` → 在 git 仓库中显示分支列表
- `git checkout -b <tab>` → 不补全（新分支名由用户输入）
- `git push <tab>` → 显示 remote 列表（origin, upstream...）
- `git push origin <tab>` → 显示分支列表
- `cargo <tab>` → 显示子命令（build, run, test, check...）
- `ls --<tab>` → 仍然显示内置命令 flags（Phase A 不受影响）
- 缓存：连续 `<tab>` 不重复执行 `git branch`（5 秒 TTL 内）

---

## 实现顺序

| 步骤 | 内容 | 涉及文件 |
|---|---|---|
| 1 | `types.rs` — CompletionSignature/Argument | `ash-core/src/completions/types.rs` (新) |
| 2 | `flag.rs` — flag 补全 | `ash-core/src/completions/flag.rs` (新) |
| 3 | 重写 `command.rs` 用 signatures | `ash-core/src/completions/command.rs` |
| 4 | `mod.rs` — 新增 `get_completions_with_context` | `ash-core/src/completions/mod.rs` |
| 5 | ShellCompleter 有状态化 | `auto-shell/src/frontend/completions_reedline.rs` |
| 6 | REPL 传递 signatures | `auto-shell/src/frontend/repl.rs` |
| 7 | 模块注册 + re-export | 两个 crate 的 mod.rs/completions.rs |
| 8 | `spec.rs` — CompletionSpec 等 | `ash-core/src/completions/spec.rs` (新) |
| 9 | `provider.rs` — 补全引擎 | `ash-core/src/completions/provider.rs` (新) |
| 10 | ShellCompleter 集成 Provider | `completions_reedline.rs` |
| 11 | 共享状态 `Arc<Mutex>` | `repl.rs` + `completions_reedline.rs` |
| 12 | git/cargo 补全定义 | `auto-shell/src/completions/definitions/` (新) |
| 13 | CompletionKind 传递到菜单 | `completions_reedline.rs` + `menu/ash_menu.rs` |

步骤 1-7 = Phase A，步骤 8-13 = Phase B。

## 关键设计决策

1. **Signature 不移动到 ash-core**：保持 `ash-core` 无 auto-shell 依赖。用 `CompletionSignature` 做桥接。
2. **补全定义先用 Rust 代码**：后续可加 `.atom` 文件加载。先验证引擎正确性。
3. **`ctx.command_executor` 是注入的**：ash-core 不直接执行外部命令，由 auto-shell 注入闭包。
4. **TTL 缓存**：动态 source（如 `git branch`）结果缓存 5 秒，避免连续 tab 重执行。
5. **`when` 条件简化**：Phase B 只支持 `FlagsPresent` / `FlagsAbsent` / `Subcommand` / `PrevArg` 四种。组合条件（and/or）留后续。
