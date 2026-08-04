# Plan 293: AshMenu — 自适应补全菜单

> **日期**: 2026-06-10
> **最后更新**: 2026-06-11
> **状态**: ✅ 已完成
> **提交**: `7a6be76e` (与 Plan 294 合并提交)

## 目标

替换 reedline 默认的 ColumnarMenu，实现自适应布局的智能补全菜单。

## 已实现功能

### 自适应布局引擎
- **CompactGrid** — 无描述项的多列网格布局
- **DescriptiveList** — 有描述项的双列布局（名称 + 描述）
- 根据数据内容和终端宽度自动选择最佳布局

### 类型感知补全系统
8 种 `CompletionKind`：
| 类型 | 用途 | 示例 |
|------|------|------|
| `Command` | 内置命令 | `ls`, `cd`, `grep` |
| `External` | 外部命令 | `git`, `cargo` |
| `File` | 文件路径 | `src/main.rs` |
| `Directory` | 目录路径 | `src/` |
| `Variable` | 环境变量 | `$PATH` |
| `Flag` | 命令标志 | `--verbose` |
| `Subcommand` | 子命令 | `cargo build` |
| `AiSuggested` | AI 建议补全 | （未来扩展） |

### 交互特性
- 方向键 / Page Up/Down / Home/End 导航
- 半屏分页（最多显示终端高度 50%）
- ANSI 类型着色（基于 `nu-ansi-term`）

## 文件结构

```
crates/auto-shell/src/menu/
├── mod.rs          (14 行)  — 模块导出
├── ash_menu.rs     (625 行) — 菜单核心实现
├── layout.rs       (128 行) — 自适应布局引擎
├── render.rs       (203 行) — ANSI 渲染
└── style.rs        (47 行)  — 样式定义
```

## 测试

单元测试覆盖布局引擎和补全过滤逻辑。
