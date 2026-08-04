# Plan 294: AshPrompt — 模块化 Prompt 引擎

> **日期**: 2026-06-10
> **最后更新**: 2026-06-11
> **状态**: ✅ 已完成
> **提交**: `7a6be76e` (与 Plan 293 合并提交)

## 目标

实现 Starship 风格的模块化 Prompt 引擎，每个 prompt 元素是独立的可配置模块。

## 已实现功能

### 7 个内置 Prompt 模块
| 模块 | 功能 | 位置 |
|------|------|------|
| `DirectoryModule` | 当前目录路径 | 左 prompt |
| `CharacterModule` | 提示符字符 ❯ | 左 prompt |
| `GitBranchModule` | Git 分支名 | 左 prompt |
| `GitStatusModule` | Git 工作区状态（staged/unstaged/untracked） | 左 prompt |
| `CmdDurationModule` | 命令执行耗时 | 左 prompt |
| `TimeModule` | 当前时间 | 右 prompt |
| `StatusModule` | 上一条命令退出状态 | 左 prompt |

### 架构特性
- **并行渲染** — 所有模块通过 rayon 并发渲染
- **惰性 I/O** — Git 信息通过 `OnceLock` 延迟发现
- **TOML 配置** — 从 `~/.config/ash-prompt.toml` 加载配置
- **SegmentStyle** — ANSI 样式（fg/bg 颜色、bold、italic、underline）
- **PromptSegment** — 样式化文本片段，拼接成最终 prompt

### 接口
```rust
pub trait PromptModule: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, ctx: &AshContext) -> Option<PromptSegment>;
    fn position(&self) -> PromptPosition; // Left | Right
}
```

实现了 `reedline::Prompt` trait，直接替换原有 `ShellPrompt`。

## 文件结构

```
crates/auto-shell/src/prompt/
├── mod.rs                  — 模块导出
├── engine.rs    (187 行)   — Prompt 引擎核心
├── context.rs   (221 行)   — AshContext 共享上下文
├── config.rs    (188 行)   — TOML 配置加载
├── module.rs    (115 行)   — PromptModule trait 定义
└── modules/
    ├── character.rs      (112 行)
    ├── cmd_duration.rs   (110 行)
    ├── directory.rs      (103 行)
    ├── git_branch.rs      (59 行)
    ├── git_status.rs      (82 行)
    ├── status.rs          (85 行)
    └── time.rs            (59 行)
```

## 测试

模块渲染测试覆盖基本输出格式。
