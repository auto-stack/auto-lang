# Plan 295: Ash 分层架构

> **日期**: 2026-06-10
> **最后更新**: 2026-06-11
> **状态**: ✅ 已完成
> **提交**: `5600fae1` (Phase A1), `989616aa` (Phase A2-A5), `d25322ec` (Phase A6)

## 目标

将 AutoShell 代码分离为纯逻辑层和终端依赖层，最终拆分为独立 crate。

## 已实现功能

### Phase A1: Core 层迁移
- 创建 `core/` 模块，**零终端依赖**
- 迁移纯逻辑模块：`parser/`、`data/`、`bookmarks.rs`、`shell/vars.rs`、`completions/`、`cmd/`
- 通过 `pub use` 保持向后兼容

### Phase A2-A5: Frontend 层 + Buffer→ANSI 桥接
- 新增 `ratatui-core 0.1` 和 `ratatui-widgets 0.3` 依赖
- 创建 `frontend/` 模块（终端依赖代码）
- 实现 **Buffer→ANSI 桥接**：将 ratatui `Buffer` 转换为 ANSI 字符串
  - 支持完整颜色/修饰符渲染
  - 使 ratatui 组件能在 reedline 上下文中工作
- 迁移终端依赖模块：`frontend/repl.rs`、`frontend/term/`、`frontend/completions_reedline.rs`

### Phase A6: ash-core Crate 创建
- 创建新 `ash-core` crate（纯逻辑，零终端依赖）
- 将 `core/` 所有模块从 `auto-shell` 移至 `ash-core`
- 更新 workspace `Cargo.toml`
- `auto-shell` 通过 `pub use ash_core as core` 保持兼容

## 最终架构

```
ash-core (crate)                    ← 纯逻辑，零终端依赖
  ├── parser/      — 纯解析逻辑
  ├── data/        — ShellValue, 类型, 转换
  ├── completions/ — 补全类型 + 逻辑
  ├── cmd/         — 命令执行辅助
  ├── bookmarks/   — 书签管理
  ├── shell/       — 变量管理
  └── pipeline/    — Atom 数据流系统（Plan 291/292）

auto-shell (crate)                  ← 终端 + Shell 集成
  ├── frontend/    — 终端 UI（未来将成为 ash-tui）
  ├── menu/        — AshMenu 补全 UI（Plan 293）
  ├── prompt/      — AshPrompt 模块（Plan 294）
  ├── cmd/         — 内置命令（74 个）
  └── shell.rs     — 主 Shell（混合层）
```

## 测试

- **ash-core**: 99+ 测试（含 Batom 基准测试）
- **auto-shell**: 473 测试（含 74 个命令的单元测试）
- **编译验证**: `cargo build -p auto` 通过
