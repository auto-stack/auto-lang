# Plan 294: AshPrompt — 模块化 Prompt 引擎

## 设计目标

参考 Starship 的架构模式，为 AutoShell 构建一个轻量的模块化 Prompt 引擎：

1. **模块化**：每个 Prompt 元素（目录、git、时间、命令耗时等）是独立模块，可插拔
2. **并行计算**：用 rayon 并行计算所有模块（Starship 的核心性能秘诀）
3. **TOML 配置驱动**：用户通过 `~/.config/ash-prompt.toml` 控制模块开关、样式、格式
4. **惰性缓存**：git 信息等用 `OnceLock` 只计算一次，多模块共享
5. **与 AutoShell 深度集成**：从 Shell 状态获取命令耗时、上次退出码、VM 状态等

## 与 Starship 的关系

**参考架构，不引入依赖。** 原因详见 Plan 293 讨论中的分析：

| Starship | AshPrompt |
|----------|-----------|
| 30+ 依赖（clap, gix, pest, process_control...） | 只新增 rayon + toml（~2 个依赖） |
| `Context` 耦合 CLI 参数、环境变量、全局线程池 | 简洁的 `AshContext`，从 Shell 状态构建 |
| 100+ 模块（覆盖所有语言工具链） | 初始 ~10 个核心模块，按需扩展 |
| 通过 `starship prompt` 子命令输出 | 直接实现 `reedline::Prompt` trait，无进程开销 |
| 格式字符串用 pest PEG parser | 简化版 `$module` 变量替换，手写 parser |
| 数据来源：外部命令 + 文件系统 | 数据来源：Shell 状态 + AutoVM + 文件系统 |

---

## 架构设计

### 整体结构

```
reedline::Prompt trait (接口层)
    ↑ implements
AshPrompt (模块化 Prompt 引擎)
    ├── AshContext (上下文：cwd, git, shell state, config)
    ├── PromptModule trait (模块接口)
    ├── PromptSegment (渲染单元：styled text)
    ├── AshConfig (TOML 配置加载)
    └── modules/ (具体模块实现)
        ├── directory.rs    ← 当前目录（~缩写、截断）
        ├── git_branch.rs   ← Git 分支名
        ├── git_status.rs   ← Git 状态（±? 等符号）
        ├── cmd_duration.rs ← 上一条命令耗时
        ├── status.rs       ← 上一条命令退出码
        ├── time.rs         ← 当前时间
        ├── username.rs     ← 用户名
        ├── hostname.rs     ← 主机名
        ├── shell.rs        ← Shell 标识
        └── character.rs    ← 提示符字符（❯）
```

### 核心类型

```rust
/// Prompt 模块 trait
///
/// 参考 Starship 的 `fn module<'a>(context: &'a Context) -> Option<Module<'a>>` 模式，
/// 但更简洁：不需要 Starship 那样复杂的 StringFormatter/Segment 系统。
pub trait PromptModule: Send + Sync {
    /// 模块名称（用于配置文件中的 key）
    fn name(&self) -> &str;

    /// 渲染模块内容，返回 None 表示不显示
    ///
    /// 这个函数应该是无副作用的纯计算。
    /// 磁盘 I/O（git、文件系统扫描）由 AshContext 惰性提供。
    fn render(&self, ctx: &AshContext) -> Option<PromptSegment>;
}

/// 渲染结果：一段带样式的文本
///
/// 对应 Starship 的 `Module`（一组 `Segment`），
/// 但 AshPrompt 中每个模块只产出一个 PromptSegment，
/// 多个 PromptSegment 拼接成最终 Prompt 字符串。
#[derive(Debug, Clone)]
pub struct PromptSegment {
    /// 模块产出的内容文本
    pub content: String,
    /// 样式（前景色、背景色、粗体等）
    pub style: SegmentStyle,
}

/// 样式定义（简化版，不搞 Starship 那样的 prev_fg/prev_bg 链式引用）
#[derive(Debug, Clone, Default)]
pub struct SegmentStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl PromptSegment {
    /// 转为 ANSI 彩色字符串
    pub fn to_ansi_string(&self) -> String {
        let mut style = nu_ansi_term::Style::new();
        if let Some(fg) = self.style.fg {
            style = style.fg(fg.into());
        }
        if let Some(bg) = self.style.bg {
            style = style.bg(bg.into());
        }
        if self.style.bold { style = style.bold(); }
        if self.style.italic { style = style.italic(); }
        if self.style.underline { style = style.underline(); }
        style.paint(&self.content).to_string()
    }
}
```

### AshContext：Prompt 上下文

```rust
/// Prompt 渲染上下文
///
/// 每次渲染 Prompt 时构建一次，所有模块共享。
/// 重计算用 OnceLock 缓存，避免多个模块重复扫描。
pub struct AshContext {
    /// 当前工作目录
    pub cwd: PathBuf,
    /// 用户 HOME 目录
    pub home: PathBuf,
    /// 上一条命令耗时（毫秒），None 表示首次启动
    pub cmd_duration_ms: Option<u64>,
    /// 上一条命令退出码，None 表示首次启动
    pub last_status: Option<i32>,
    /// Prompt 配置
    pub config: AshConfig,

    // --- 惰性缓存字段 ---
    /// Git 仓库信息（惰性，多个 git 模块共享）
    git_info: OnceLock<Option<GitInfo>>,
}

impl AshContext {
    /// 从 Shell 状态构建
    pub fn from_shell(shell: &Shell) -> Self { ... }

    /// 获取 git 信息（惰性计算，最多执行一次）
    pub fn git_info(&self) -> Option<&GitInfo> {
        self.git_info.get_or_init(|| {
            // 尝试发现 git 仓库
            // 如果不需要 git 模块，这永远不会执行
            discover_git_info(&self.cwd)
        }).as_ref()
    }
}

/// Git 仓库信息（如果当前目录在 git 仓库中）
pub struct GitInfo {
    pub branch: String,
    pub status: GitStatus,  // staged, unstaged, untracked 数量
    pub root: PathBuf,
}

pub struct GitStatus {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicted: usize,
    pub ahead: usize,
    pub behind: usize,
}
```

### Prompt 拼接引擎

```rust
/// AshPrompt 主结构，实现 reedline::Prompt
pub struct AshPrompt {
    /// 注册的模块列表（有序）
    modules: Vec<Box<dyn PromptModule>>,
    /// 右侧 Prompt 模块
    right_modules: Vec<Box<dyn PromptModule>>,
    /// 提示符字符模块
    character: Box<dyn PromptModule>,
}

impl AshPrompt {
    /// 创建默认 Prompt（内置模块集）
    pub fn new(config: AshConfig) -> Self {
        let mut p = Self {
            modules: Vec::new(),
            right_modules: Vec::new(),
            character: Box::new(CharacterModule::new(&config)),
        };

        // 左侧 Prompt 默认模块顺序（参考 Starship PROMPT_ORDER）
        p.add_module(DirectoryModule::new(&config));
        p.add_module(GitBranchModule::new(&config));
        p.add_module(GitStatusModule::new(&config));
        p.add_module(CmdDurationModule::new(&config));

        // 右侧 Prompt
        p.add_right_module(TimeModule::new(&config));

        p
    }

    /// 渲染左侧 Prompt（并行计算所有模块）
    fn render_left(&self, ctx: &AshContext) -> String {
        use rayon::prelude::*;

        let segments: Vec<PromptSegment> = self.modules
            .par_iter()
            .filter_map(|m| m.render(ctx))
            .collect();

        segments.iter()
            .map(|s| s.to_ansi_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// 实现 reedline::Prompt trait
impl reedline::Prompt for AshPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        // 每次渲染时构建新的 context
        // （开销极小，主要是指针和 OnceLock）
        let ctx = AshContext::from_current();
        Cow::Owned(self.render_left(&ctx))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        let ctx = AshContext::from_current();
        Cow::Owned(self.render_right(&ctx))
    }

    fn render_prompt_indicator(&self, mode: PromptEditMode) -> Cow<'_, str> {
        let ctx = AshContext::from_current();
        Cow::Owned(
            self.character.render(&ctx)
                .map(|s| s.to_ansi_string())
                .unwrap_or_else(|| "⟩ ".to_string())
        )
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("..> ")
    }

    fn render_prompt_history_search_indicator(
        &self, search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Owned(format!("({}): ", search.term))
    }
}
```

### 配置系统

```toml
# ~/.config/ash-prompt.toml

# Prompt 格式（$module_name 表示模块插入点）
format = "$directory$git_branch$git_status$cmd_duration$character"
right_format = "$time"

# 如果 false，Prompt 前不加空行
add_newline = false

# 命令执行超时阈值（超过此时间才显示 cmd_duration 模块）
cmd_duration_threshold = 2000  # ms

# 目录模块配置
[directory]
style = "cyan bold"
truncation_length = 3
home_symbol = "~"

# Git 分支模块配置
[git_branch]
style = "green bold"
symbol = " "

# Git 状态模块配置
[git_status]
style = "red bold"

# 命令耗时模块配置
[cmd_duration]
style = "yellow"
min_time = 2000  # 超过 2 秒才显示

# 时间模块配置
[time]
style = "yellow"
format = "[$time]($style)"
time_format = "%H:%M"

# 提示符字符
[character]
success = "❯"
error = "❯"
style_success = "green bold"
style_error = "red bold"

# 模块开关（false = 禁用）
[hostname]
disabled = true

[username]
disabled = true
```

```rust
/// 配置加载
#[derive(Debug, Clone)]
pub struct AshConfig {
    pub format: String,
    pub right_format: String,
    pub add_newline: bool,
    pub cmd_duration_threshold: u64,
    pub module_configs: HashMap<String, toml::Value>,
}

impl AshConfig {
    /// 从文件加载，文件不存在则返回默认配置
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ash-prompt.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// 获取某模块的配置
    pub fn module_config(&self, name: &str) -> Option<&toml::Value> {
        self.module_configs.get(name)
    }

    /// 判断某模块是否被禁用
    pub fn is_module_disabled(&self, name: &str) -> bool {
        self.module_configs.get(name)
            .and_then(|v| v.get("disabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}
```

### 具体模块示例

#### Directory 模块

```rust
pub struct DirectoryModule {
    style: SegmentStyle,
    truncation_length: usize,
    home_symbol: String,
}

impl PromptModule for DirectoryModule {
    fn name(&self) -> &str { "directory" }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let cwd = &ctx.cwd;
        let home = &ctx.home;

        let mut dir_str = cwd.to_string_lossy().to_string();

        // 1. 缩写 HOME 为 ~
        if dir_str.starts_with(home.to_str().unwrap_or("")) {
            dir_str = dir_str.replacen(home.to_str().unwrap(), "~", 1);
        }

        // 2. 截断到 N 个路径组件
        let components: Vec<&str> = dir_str.split('/').filter(|s| !s.is_empty()).collect();
        if components.len() > self.truncation_length {
            let start = components.len() - self.truncation_length;
            dir_str = components[start..].join("/");
        }

        Some(PromptSegment {
            content: format!("{} ", dir_str),
            style: self.style.clone(),
        })
    }
}
```

#### Git Branch 模块

```rust
pub struct GitBranchModule {
    style: SegmentStyle,
    symbol: String,
}

impl PromptModule for GitBranchModule {
    fn name(&self) -> &str { "git_branch" }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let git_info = ctx.git_info()?;
        Some(PromptSegment {
            content: format!("{}{} ", self.symbol, git_info.branch),
            style: self.style.clone(),
        })
    }
}
```

#### Git Status 模块

```rust
pub struct GitStatusModule {
    style: SegmentStyle,
}

impl PromptModule for GitStatusModule {
    fn name(&self) -> &str { "git_status" }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let git_info = ctx.git_info()?;
        let s = &git_info.status;

        let mut parts = Vec::new();
        if s.staged > 0     { parts.push(format!("+{}", s.staged)); }
        if s.unstaged > 0   { parts.push(format!("!{}", s.unstaged)); }
        if s.untracked > 0  { parts.push(format!("?{}", s.untracked)); }
        if s.conflicted > 0 { parts.push(format!("~{}", s.conflicted)); }
        if s.ahead > 0      { parts.push(format!("⇡{}", s.ahead)); }
        if s.behind > 0     { parts.push(format!("⇣{}", s.behind)); }

        if parts.is_empty() { return None; }

        Some(PromptSegment {
            content: format!("[{}] ", parts.join(" ")),
            style: self.style.clone(),
        })
    }
}
```

#### Command Duration 模块

```rust
pub struct CmdDurationModule {
    style: SegmentStyle,
    min_time: u64,
}

impl PromptModule for CmdDurationModule {
    fn name(&self) -> &str { "cmd_duration" }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let ms = ctx.cmd_duration_ms?;
        if ms < self.min_time { return None; }

        let content = if ms < 1000 {
            format!("{}ms ", ms)
        } else if ms < 60_000 {
            format!("{:.1}s ", ms as f64 / 1000.0)
        } else {
            let mins = ms / 60_000;
            let secs = (ms % 60_000) / 1000;
            format!("{}m{}s ", mins, secs)
        };

        Some(PromptSegment { content, style: self.style.clone() })
    }
}
```

#### Character 模块（提示符符号）

```rust
pub struct CharacterModule {
    success_char: String,
    error_char: String,
    success_style: SegmentStyle,
    error_style: SegmentStyle,
}

impl PromptModule for CharacterModule {
    fn name(&self) -> &str { "character" }

    fn render(&self, ctx: &AshContext) -> Option<PromptSegment> {
        let is_error = ctx.last_status.map(|s| s != 0).unwrap_or(false);
        Some(PromptSegment {
            content: if is_error {
                format!("{} ", self.error_char)
            } else {
                format!("{} ", self.success_char)
            },
            style: if is_error {
                self.error_style.clone()
            } else {
                self.success_style.clone()
            },
        })
    }
}
```

### 文件结构

```
crates/auto-shell/src/
├── prompt/                        ← 新增目录
│   ├── mod.rs                     ← 模块导出 + AshPrompt struct
│   ├── context.rs                 ← AshContext, GitInfo, GitStatus
│   ├── module.rs                  ← PromptModule trait, PromptSegment, SegmentStyle
│   ├── config.rs                  ← AshConfig, TOML 加载
│   ├── engine.rs                  ← Prompt 拼接引擎（rayon 并行）
│   └── modules/                   ← 具体模块实现
│       ├── mod.rs                 ← 模块注册
│       ├── directory.rs           ← 当前目录
│       ├── git_branch.rs          ← Git 分支
│       ├── git_status.rs          ← Git 状态
│       ├── cmd_duration.rs        ← 命令耗时
│       ├── status.rs              ← 退出码
│       ├── time.rs                ← 时间
│       ├── username.rs            ← 用户名
│       ├── hostname.rs            ← 主机名
│       └── character.rs           ← 提示符字符 (❯)
├── term/
│   └── prompt.rs                  ← 保留为空壳或删除，功能迁到 prompt/
└── repl.rs                        ← 替换 ShellPrompt → AshPrompt
```

---

## 实现任务

### Task 1：添加依赖
**文件**: `crates/auto-shell/Cargo.toml`

- [ ] 添加 `rayon = "1.12"` 依赖（并行计算模块）
- [ ] 添加 `toml = "0.8"` 依赖（配置文件解析）
- [ ] 运行 `cargo build -p auto-shell` 确认编译通过

### Task 2：核心类型定义
**文件**: `crates/auto-shell/src/prompt/module.rs`

- [ ] 定义 `PromptModule` trait（`name()` + `render()`）
- [ ] 定义 `PromptSegment` struct（`content` + `style`）
- [ ] 定义 `SegmentStyle` struct（fg, bg, bold, italic, underline）
- [ ] 实现 `PromptSegment::to_ansi_string()` 转换为 ANSI 彩色字符串
- [ ] 编写单元测试

### Task 3：AshContext 上下文
**文件**: `crates/auto-shell/src/prompt/context.rs`

- [ ] 定义 `AshContext` struct（cwd, home, cmd_duration_ms, last_status, config）
- [ ] 定义 `GitInfo` / `GitStatus` struct
- [ ] 实现 `AshContext::from_current()` 从环境构建
- [ ] 实现 `git_info()` 的 OnceLock 惰性缓存
- [ ] 实现 `discover_git_info()` —— 用 `std::process::Command` 调 `git` 命令（简单版，不引 gix）
  - `git rev-parse --abbrev-ref HEAD` → 分支名
  - `git status --porcelain` → 文件状态统计
  - `git rev-list --left-right --count HEAD...@{upstream}` → ahead/behind
- [ ] 编写测试（mock cwd 和 git 输出）

### Task 4：配置系统
**文件**: `crates/auto-shell/src/prompt/config.rs`

- [ ] 定义 `AshConfig` struct（format, right_format, add_newline, module_configs）
- [ ] 实现 `AshConfig::load()` —— 从 `~/.config/ash-prompt.toml` 读取
- [ ] 实现 `AshConfig::default()` —— 默认配置
- [ ] 实现 `is_module_disabled()` / `module_config()` 方法
- [ ] 编写测试（临时文件 + TOML 解析）

### Task 5：Prompt 引擎
**文件**: `crates/auto-shell/src/prompt/engine.rs`, `prompt/mod.rs`

- [ ] 定义 `AshPrompt` struct（modules, right_modules, character）
- [ ] 实现 `AshPrompt::new(config)` —— 注册默认模块集
- [ ] 实现 `render_left()` —— rayon `par_iter` 并行计算所有模块
- [ ] 实现 `render_right()` —— 右侧 Prompt
- [ ] 实现 `reedline::Prompt` trait（所有 render 方法）
- [ ] 编写测试（mock context，验证输出格式）

### Task 6：核心模块实现（第一批）
**文件**: `crates/auto-shell/src/prompt/modules/`

- [ ] `directory.rs` —— 目录显示（~ 缩写、截断、Windows 路径处理）
- [ ] `character.rs` —— 提示符字符（成功/失败不同颜色）
- [ ] `cmd_duration.rs` —— 命令耗时（超过阈值才显示）
- [ ] `git_branch.rs` —— Git 分支名
- [ ] `git_status.rs` —— Git 状态符号
- [ ] `time.rs` —— 当前时间（右侧 Prompt）
- [ ] 每个模块编写单元测试

### Task 7：集成到 REPL
**文件**: `crates/auto-shell/src/repl.rs`, `shell.rs`

- [ ] `Shell` struct 记录上次命令执行耗时和退出码
- [ ] `Shell` struct 持有 `AshConfig`（启动时加载一次）
- [ ] `Repl` 中替换 `ShellPrompt` 为 `AshPrompt`
- [ ] `repl.rs` 移除 `ShellPrompt` 定义和 `term/prompt.rs` 引用
- [ ] 运行 `cargo test -p auto-shell` 全量测试
- [ ] 手动端到端测试：
  - 普通目录：`~/projects/my-app ❯`
  - Git 目录：`~/projects/my-app main ❯`
  - 有改动：`~/projects/my-app main [+2 !1 ?3] ❯`
  - 命令耗时：`~/projects/my-app 5.2s ❯`
  - 错误退出码：`~/projects/my-app ❯`（红色字符）

### Task 8：扩展模块（第二批，可选）
**文件**: `crates/auto-shell/src/prompt/modules/`

- [ ] `username.rs` —— 用户名（SSH 或 root 时显示）
- [ ] `hostname.rs` —— 主机名（SSH 时显示）
- [ ] `status.rs` —— 上一条命令退出码（非零时显示）
- [ ] `shell.rs` —— Shell 标识
- [ ] `memory_usage.rs` —— 内存使用率（可选）

---

## Git 信息获取策略

| 方案 | 优点 | 缺点 | 推荐 |
|------|------|------|------|
| **调用 `git` 命令** | 零依赖，信息准确 | 需要 git 在 PATH 中 | ✅ 初期采用 |
| **用 `gix` crate** | 纯 Rust，无需 git 二进制 | 重依赖（~200 个子 crate） | 后期可选 |
| **手写 `.git` 解析** | 轻量 | 只能获取基本分支信息 | 折中方案 |

初期用 `std::process::Command("git", ...)` 获取信息，加上超时控制（200ms）避免卡顿。
如果后期 git 模块成为性能瓶颈，再引入 `gix`。

---

## 新增依赖总览

| Crate | 版本 | 用途 | 大小影响 |
|-------|------|------|----------|
| `rayon` | 1.12 | 并行模块计算 | 中（但 auto-shell 间接已有） |
| `toml` | 0.8 | 配置文件解析 | 小 |

**总计新增 2 个直接依赖**，对比 Starship 的 30+ 个。

---

## 与其他 Plan 的关系

| Plan | 关系 |
|------|------|
| Plan 293 (AshMenu) | 并行开发，无依赖。AshMenu 管 Tab 补全 UI，AshPrompt 管提示符渲染 |
| Plan 291 (Warp-style) | AshPrompt 是 Plan 291 Phase 1 的子集实现 |
| Plan 292 (Atom Pipeline) | 未来 Prompt 模块可从 Atom 获取数据（如 git status 的结构化表示） |
| Plan 153 (AI Agent) | 未来可添加 AI 推荐模块（显示上下文窗口使用量、当前模型等） |

---

## 风险和注意事项

1. **rayon 全局线程池**：rayon 的全局线程池只能初始化一次。如果 auto-lang 的 VM 也用 rayon，不会有冲突。如果有，可改用 `rayon::ThreadPool::new()` 自建池。

2. **Windows 路径**：当前 `ShellPrompt` 已经处理了 UNC 前缀和路径分隔符，`DirectoryModule` 需要继承这个逻辑。

3. **配置文件不存在**：`AshConfig::load()` 必须优雅降级到默认配置，不能 panic。

4. **reedline::Prompt trait 的生命周期**：`render_prompt_left()` 返回 `Cow<'_, str>`，我们的 `AshPrompt` 需要 `self` 活得足够长。由于 `AshPrompt` 被 `Repl` 持有，这不是问题。

5. **性能**：每次 Prompt 渲染都构建 `AshContext` 并并行计算模块。实测 Starship 这个过程 <5ms，对 AutoShell 来说完全可以接受。如果后续发现瓶颈，可以将 `AshContext` 缓存在 `AshPrompt` 上，只在状态变化时重建。
