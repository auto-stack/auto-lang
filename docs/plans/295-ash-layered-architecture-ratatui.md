# Plan 295: AutoShell 分层架构 + ratatui 集成

> **日期**: 2026-06-10
> **状态**: 设计阶段
> **影响范围**: Plan 291/292/293/294 全部需要调整
> **前置条件**: 无（这是其他计划的前置）

---

## 一、问题背景

我们近期规划了 4 个 AutoShell 相关计划：

| Plan | 内容 | 状态 |
|------|------|------|
| 291 | Warp-style 全栈升级（结构化管线 + AI + Block UX） | 设计已批准 |
| 292 | Atom Pipeline（Atom 作为管线一等公民） | 设计已批准 |
| 293 | AshMenu（自适应补全菜单，参考 Fish Pager） | 设计已完成 |
| 294 | AshPrompt（模块化 Prompt 引擎，参考 Starship） | 设计已完成 |

**两个新发现打破了原有假设**：

1. **ratatui 可以不接管终端**：ratatui 的 `Buffer` + `Widget::render()` 可以在内存中渲染，然后转成 ANSI 字符串。这意味着 AshMenu/AshPrompt 的渲染可以用 ratatui widget，不需要 ratatui 接管终端。
2. **AutoShell 需要分层架构**：未来要支持 TUI + GUI 双前端（借助 AutoUI），但当前代码把逻辑和终端渲染混在一起。

**结论**：在实施 291-294 之前，需要先建立分层架构和引入 ratatui，否则后续改动会越来越大。

---

## 二、当前架构问题

### AutoShell 当前耦合分析

```
crates/auto-shell/src/
│
├── 纯逻辑层（零终端依赖）        ← 应该是后端
│   ├── parser/                   解析管线、引号、重定向
│   ├── cmd/pipeline_data.rs      PipelineData(Value|Text)
│   ├── cmd/registry.rs           CommandRegistry
│   ├── cmd/parser.rs             ParsedArgs
│   ├── cmd/data.rs               sort/uniq/head/tail/wc/grep
│   ├── cmd/external.rs           std::process::Command
│   ├── data/types.rs             AshFileEntry, AshProcessEntry...
│   ├── data/convert.rs           metadata → Value
│   ├── data/value.rs             ShellValue
│   ├── completions/command.rs    命令名补全
│   ├── completions/file.rs       文件路径补全
│   ├── completions/auto.rs       变量补全
│   └── bookmarks.rs              书签管理
│
├── 混合层（逻辑 + 终端耦合）     ← 需要拆分
│   ├── shell.rs                  Shell 状态 + println! 输出
│   ├── cmd.rs                    Command trait (接收 &mut Shell)
│   ├── cmd/commands/*.rs         18 个命令（逻辑 + 格式化）
│   ├── completions.rs            补全调度（纯逻辑，但被 reedline 桥接污染）
│   └── data/table.rs             表格渲染（嵌入 nu-ansi-term Style）
│
└── 终端层（重度终端依赖）         ← 应该是前端
    ├── repl.rs                   Repl 拥有 Reedline + ShellPrompt
    ├── completions/reedline.rs   reedline::Completer 适配
    └── term/                     高亮、Prompt（目前是 stub）
```

**5 个关键耦合点**：

1. **`Command` trait 接收 `&mut Shell`** → 命令无法脱离 Shell 独立执行
2. **`Shell::cd()` 直接 `println!`** → 输出应该是返回值，不是副作用
3. **`data::Table` 嵌入 `nu_ansi_term::Style`** → 数据和渲染混在一起
4. **`Repl` 直接拥有 `Reedline`** → REPL 循环和 Shell 引擎不分离
5. **`completions/reedline.rs` 编译期依赖 reedline** → 补全逻辑无法在非终端环境使用

---

## 三、目标架构

### 分层设计

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend Layer                        │
│                                                         │
│  ┌──────────────────┐    ┌───────────────────────────┐  │
│  │   TUI Frontend    │    │   GUI Frontend (未来)      │  │
│  │                   │    │                           │  │
│  │  reedline         │    │  AutoUI renderer          │  │
│  │  ratatui widgets  │    │  GPU 加速                 │  │
│  │  crossterm        │    │  桌面窗口                  │  │
│  │                   │    │                           │  │
│  │  - AshMenu        │    │  - 富文本补全面板          │  │
│  │  - AshPrompt      │    │  - AI Agent 面板           │  │
│  │  - Table 渲染     │    │  - Block 编辑器            │  │
│  │  - Highlighter    │    │                           │  │
│  └────────┬──────────┘    └────────────┬──────────────┘  │
│           │                            │                 │
│  ┌────────┴────────────────────────────┴──────────────┐  │
│  │              Ash Renderer (ratatui-core)            │  │
│  │                                                     │  │
│  │  Buffer → Widget 渲染 → ANSI String (TUI)           │  │
│  │  Buffer → Widget 渲染 → Cell Grid    (GUI)          │  │
│  └─────────────────────────┬───────────────────────────┘  │
└────────────────────────────┼──────────────────────────────┘
                             │
┌────────────────────────────┼──────────────────────────────┐
│                    Backend Layer (零终端依赖)              │
│                            │                              │
│  ┌─────────────────────────┴───────────────────────────┐  │
│  │              Ash Engine                              │  │
│  │                                                     │  │
│  │  ShellState (cwd, vars, env, bookmarks)              │  │
│  │  CommandRegistry + PipelineExecutor                  │  │
│  │  CompletionEngine (命令/文件/变量补全)                │  │
│  │  AutoVM session                                      │  │
│  │                                                     │  │
│  │  所有输出都是 PipelineData / Atom                    │  │
│  │  零 println! / 零 ANSI / 零 reedline / 零 crossterm  │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                           │
│  数据层: Atom / BAtom / Value / AshFileEntry / ...        │
└───────────────────────────────────────────────────────────┘
```

### Crate 拆分方案

```
crates/
├── ash-core/              ← 新建（后端，零终端依赖）
│   ├── shell_state.rs     Shell 状态（cwd, vars, bookmarks）
│   ├── command.rs         Command trait（接收 &mut ShellContext）
│   ├── pipeline.rs        PipelineData, PipelineExecutor
│   ├── completions.rs     Completion 引擎（纯逻辑）
│   ├── parser/            管线解析、引号、重定向
│   ├── data/              类型定义（无 Style/ANSI）
│   ├── commands/          内置命令实现
│   └── Cargo.toml         依赖: auto-lang, auto-val, miette, chrono, sysinfo
│
├── ash-tui/               ← 新建（TUI 前端）
│   ├── repl.rs            Repl 主循环（reedline 驱动）
│   ├── menu/              AshMenu（ratatui widget → Buffer → ANSI）
│   ├── prompt/            AshPrompt（模块化 Prompt 引擎）
│   ├── renderer/          ratatui 渲染桥接（Buffer → ANSI string）
│   ├── table.rs           表格渲染（ratatui Table widget）
│   ├── highlight.rs       语法高亮
│   ├── completions.rs     reedline::Completer 适配器
│   └── Cargo.toml         依赖: ash-core, reedline, ratatui-core, ratatui-widgets, nu-ansi-term
│
├── ash-gui/               ← 未来（GUI 前端）
│   └── Cargo.toml         依赖: ash-core, auto-ui renderer
│
└── auto-shell/            ← 现有，改为薄壳（bin only）
    ├── main.rs            CLI 入口，选择 TUI 或 GUI 前端
    └── Cargo.toml         依赖: ash-core, ash-tui (默认), ash-gui (可选)
```

---

## 四、对现有 Plan 的影响

### Plan 291 (Warp-style 全栈升级)

**影响**：架构层面保持不变（管线、AI、Block UX 三位一体），但实施路径需要调整。

| 原计划 | 调整 |
|--------|------|
| Phase 0: Atom 管线 → 在 auto-shell 中实施 | → 移到 `ash-core` 中实施 |
| Phase 1: NuShell crate 集成 | → 在 `ash-core` 中集成 |
| Phase 2: AI 智能集成 | → AI 引擎在 `ash-core`，AI UI 在 `ash-tui` |
| Phase 3: Block UX | → 用 ratatui 的全屏模式实现，在 `ash-tui` 中 |
| Phase 4: 桌面 GUI | → 对应 `ash-gui` crate |

**Phase 0 (Atom 管线) 是前置条件，保持优先级不变，但实施位置从 auto-shell 移到 ash-core。**

### Plan 292 (Atom Pipeline)

**影响**：设计不变，但目标 crate 从 `auto-shell/src/pipeline/` 变为 `ash-core/src/pipeline/`。

所有 Atom 相关的类型（`Atom`, `AtomType`, `AtomPipeline`）放在 `ash-core` 中，零终端依赖。

### Plan 293 (AshMenu)

**影响最大**。渲染方式从"手写 ANSI"变为"ratatui widget → Buffer → ANSI"。

| 原计划 | 调整 |
|--------|------|
| 手写 `render_compact_grid()` / `render_descriptive_list()` | 用 ratatui 的 `Table`/`List` widget 渲染到 `Buffer` |
| 手写列宽计算、对齐逻辑 | ratatui 的 `Layout` + `Constraint` 自动处理 |
| 手写 ANSI 拼接 | `Buffer` → ANSI string 转换函数 |
| 手写选中高亮 | ratatui 的 `Style` + `Modifier::REVERSED` |
| 手写搜索框 | ratatui 的 `Paragraph` widget |
| 位于 `auto-shell/src/menu/` | 位于 `ash-tui/src/menu/` |

核心新增：需要一个 `buffer_to_ansi()` 函数，把 ratatui `Buffer`（`Vec<Cell>`）转换为 ANSI 彩色字符串，供 reedline 的 `menu_string()` 使用。

### Plan 294 (AshPrompt)

**影响中等**。Prompt 渲染可以继续用 `nu-ansi-term`（不需要 ratatui widget），但位置从 auto-shell 移到 ash-tui。

| 原计划 | 调整 |
|--------|------|
| 位于 `auto-shell/src/prompt/` | 位于 `ash-tui/src/prompt/` |
| 手写 `PromptSegment::to_ansi_string()` | 保持不变（nu-ansi-term 足够） |
| 依赖 rayon + toml | 保持不变 |

Prompt 模块不需要 ratatui widget（它只是一段 ANSI 文本），所以 Plan 294 的技术方案基本不变，只是位置移动。

---

## 五、实施路线

### 执行顺序

```
Phase A: 分层基础 (本计划, Plan 295)
    │
    ├── A1: 拆分 ash-core crate（从 auto-shell 提取纯逻辑）
    ├── A2: 引入 ratatui 依赖
    ├── A3: 实现 Buffer → ANSI 转换桥接
    └── A4: 创建 ash-tui crate（TUI 前端）
    
Phase B: Atom 管线 (Plan 292, 在 ash-core 中实施)
    │
    └── B1-B12: 原计划的 12 个 Task
    
Phase C: AshMenu (Plan 293, 在 ash-tui 中实施)
    │
    └── C1-C8: 原计划的 8 个 Task（用 ratatui widget 渲染）

Phase D: AshPrompt (Plan 294, 在 ash-tui 中实施)
    │
    └── D1-D8: 原计划的 8 个 Task
    
Phase E: Warp-style 扩展 (Plan 291, 分阶段实施)
    │
    ├── E1: Phase 1 - NuShell crate 集成 (在 ash-core)
    ├── E2: Phase 2 - AI 智能集成 (ash-core + ash-tui)
    └── E3: Phase 3 - Block UX (ash-tui 全屏模式)
```

### 各 Phase 的依赖关系

```
Phase A (分层基础)
    │
    ├──→ Phase B (Atom 管线) ──→ Phase E1 (NuShell 集成)
    │
    ├──→ Phase C (AshMenu)
    │
    ├──→ Phase D (AshPrompt)
    │
    └──→ Phase E2 (AI) ──→ Phase E3 (Block UX)
```

**Phase C 和 Phase D 可以并行**（AshMenu 和 AshPrompt 无依赖）。
**Phase B 和 Phase C/D 也可以并行**（Atom 管线和 TUI 前端是正交的）。

---

## 六、Phase A 详细任务

### Task A1: 创建 ash-core crate

**操作**：
1. 创建 `crates/ash-core/` 目录结构
2. 从 `auto-shell/src/` 提取以下纯逻辑模块到 `ash-core/src/`：

| 源文件 | 目标位置 | 备注 |
|--------|---------|------|
| `parser/*.rs` | `ash-core/src/parser/` | 原样迁移 |
| `cmd/pipeline_data.rs` | `ash-core/src/pipeline.rs` | 重命名 |
| `cmd/value_helpers.rs` | `ash-core/src/value_helpers.rs` | 原样迁移 |
| `cmd/registry.rs` | `ash-core/src/command/registry.rs` | 原样迁移 |
| `cmd/parser.rs` | `ash-core/src/command/parser.rs` | 原样迁移 |
| `cmd/data.rs` | `ash-core/src/commands/data.rs` | 纯数据操作命令 |
| `cmd/external.rs` | `ash-core/src/commands/external.rs` | 外部进程执行 |
| `data/types.rs` | `ash-core/src/data/types.rs` | 类型定义 |
| `data/convert.rs` | `ash-core/src/data/convert.rs` | 数据转换 |
| `data/value.rs` | `ash-core/src/data/value.rs` | ShellValue |
| `completions.rs` | `ash-core/src/completions.rs` | 补全调度 |
| `completions/command.rs` | `ash-core/src/completions/command.rs` | 命令补全 |
| `completions/file.rs` | `ash-core/src/completions/file.rs` | 文件补全 |
| `completions/auto.rs` | `ash-core/src/completions/auto.rs` | 变量补全 |
| `bookmarks.rs` | `ash-core/src/bookmarks.rs` | 书签管理 |
| `shell/vars.rs` | `ash-core/src/shell_state/vars.rs` | 变量存储 |

3. `ash-core/Cargo.toml` 依赖：
   - `auto-lang`, `auto-val` (已有)
   - `miette`, `thiserror` (已有)
   - `chrono`, `sysinfo`, `indexmap`, `dirs` (已有)
   - **不包含**：`reedline`, `crossterm`, `nu-ansi-term`

4. 处理 `data/table.rs`：拆分为两部分
   - `ash-core/src/data/table.rs`：表格数据结构（列名、行数据），**移除 `nu_ansi_term::Style`**
   - `ash-tui/src/renderer/table.rs`：表格渲染（用 ratatui Table widget）

5. 处理 `shell.rs`：拆分为
   - `ash-core/src/shell_state.rs`：`ShellState` struct（cwd, vars, session, bookmarks, registry）
   - `ash-core/src/engine.rs`：`ShellEngine`（执行命令，返回 `PipelineData`，不 println）

6. 处理 `Command` trait：
   - `&mut Shell` → `&mut dyn ShellContext`（trait，暴露 cwd、vars、env 等）
   - `ShellState` 实现 `ShellContext`

7. 运行 `cargo test -p ash-core` 确保所有迁移的测试通过

### Task A2: 引入 ratatui 依赖

**操作**：
1. `ash-tui/Cargo.toml` 添加：
   ```toml
   ratatui-core = "0.1"       # Buffer, Widget trait, Layout, Style, Cell (v0.1.1 on crates.io)
   ratatui-widgets = "0.3"    # Table, List, Paragraph, Block, Clear (v0.3.1 on crates.io)
   ```
   注意：**不引入 `ratatui` 主 crate**（它包含 crossterm backend），只引入 core + widgets。

2. 验证编译通过

### Task A3: 实现 Buffer → ANSI 转换桥接

**文件**: `ash-tui/src/renderer/mod.rs`, `ash-tui/src/renderer/buffer_to_ansi.rs`

这是关键技术桥接：把 ratatui 的 `Buffer`（`Vec<Cell>`）转换为 ANSI 彩色字符串。

```rust
/// 将 ratatui Buffer 转换为 ANSI 彩色字符串
/// 用于 reedline 的 menu_string() 和其他需要纯文本输出的场景
pub fn buffer_to_ansi(buf: &ratatui_core::buffer::Buffer) -> String {
    let mut output = String::new();
    let width = buf.area.width as usize;
    
    for row in 0..buf.area.height as usize {
        for col in 0..width {
            let cell = &buf[(col as u16, row as u16)];
            // 将 ratatui Cell 的 fg/bg/modifier 转换为 ANSI escape sequence
            output.push_str(&cell_to_ansi(cell));
        }
        if row < buf.area.height as usize - 1 {
            output.push('\n');
        }
    }
    output
}

fn cell_to_ansi(cell: &ratatui_core::buffer::Cell) -> String {
    use ratatui_core::style::{Color, Modifier};
    
    let mut style = nu_ansi_term::Style::new();
    
    // fg color
    match cell.fg {
        Color::Rgb(r, g, b) => style = style.fg(nu_ansi_term::Color::Rgb(r, g, b)),
        Color::Indexed(n) => style = style.fg(nu_ansi_term::Color::Fixed(n)),
        _ => {}
    }
    
    // bg color
    match cell.bg {
        Color::Rgb(r, g, b) => style = style.bg(nu_ansi_term::Color::Rgb(r, g, b)),
        Color::Indexed(n) => style = style.bg(nu_ansi_term::Color::Fixed(n)),
        _ => {}
    }
    
    // modifiers
    if cell.modifier.intersects(Modifier::BOLD) { style = style.bold(); }
    if cell.modifier.intersects(Modifier::ITALIC) { style = style.italic(); }
    if cell.modifier.intersects(Modifier::UNDERLINED) { style = style.underline(); }
    if cell.modifier.intersects(Modifier::REVERSED) { style = style.reverse(); }
    
    style.paint(cell.symbol()).to_string()
}
```

这个函数让 AshMenu 可以：
1. 用 ratatui widget 渲染到 `Buffer`
2. 调用 `buffer_to_ansi()` 转为字符串
3. 在 reedline 的 `menu_string()` 中返回这个字符串

### Task A4: 创建 ash-tui crate

**操作**：
1. 创建 `crates/ash-tui/` 目录结构
2. 从 `auto-shell/src/` 提取终端相关模块：
   - `repl.rs` → `ash-tui/src/repl.rs`
   - `completions/reedline.rs` → `ash-tui/src/completions.rs`
   - `term/highlight.rs` → `ash-tui/src/highlight.rs`
3. `ash-tui/Cargo.toml` 依赖：
   ```toml
   ash-core = { path = "../ash-core" }
   reedline = "0.44.0"
   ratatui-core = "0.1"
   ratatui-widgets = "0.3"
   nu-ansi-term = "0.49"
   rayon = "1.12"
   toml = "0.8"
   ```

4. `auto-shell` crate 变为薄壳：
   ```toml
   # crates/auto-shell/Cargo.toml
   [dependencies]
   ash-core = { path = "../ash-core" }
   ash-tui = { path = "../ash-tui" }
   ```
   `main.rs` 只做：初始化 `ShellEngine`，创建 `Repl`，调用 `repl.run()`。

5. 运行 `cargo build -p auto-shell && cargo test -p ash-core && cargo test -p ash-tui`

### Task A5: 清理 auto-shell 中的死依赖

**操作**：
1. 从 `auto-shell/Cargo.toml` 移除已迁走的依赖
2. 移除 4 个实际未使用的依赖：`crossterm`, `uucore`, `regex`, `unicode-segmentation`
3. `auto-shell` 最终只依赖 `ash-core` + `ash-tui`
4. 确保全量测试通过

### Task A6: 更新 workspace Cargo.toml

**文件**: 根目录 `Cargo.toml`

添加 `ash-core` 和 `ash-tui` 到 workspace members：
```toml
[workspace]
members = [
    "crates/auto-lang",
    "crates/auto-val",
    "crates/ash-core",     # 新增
    "crates/ash-tui",      # 新增
    "crates/auto-shell",   # 保留（薄壳 bin）
]
```

---

## 七、更新后的 Plan 依赖图

```
Plan 295 (本计划: 分层 + ratatui) ← 前置，最先实施
    │
    ├──→ Plan 292 (Atom 管线)     ← 在 ash-core 中实施
    │       │
    │       └──→ Plan 291 Phase 1 (NuShell 集成)
    │
    ├──→ Plan 293 (AshMenu)       ← 在 ash-tui 中实施，用 ratatui widget
    │
    ├──→ Plan 294 (AshPrompt)     ← 在 ash-tui 中实施
    │
    └──→ Plan 291 Phase 2+ (AI + Block UX) ← 依赖以上所有
```

**实施优先级**：
1. **Plan 295**（本计划）→ 分层基础，1-2 周
2. **Plan 293**（AshMenu）+ **Plan 294**（AshPrompt）→ 可并行，1-2 周
3. **Plan 292**（Atom 管线）→ 2-3 周
4. **Plan 291 Phase 1**（NuShell 集成）→ 3-4 周

---

## 八、风险和注意事项

1. **ratatui 版本兼容**：ratatui 刚完成 workspace 拆分（0.30+），`ratatui-core` 和 `ratatui-widgets` 是新 crate，API 可能还有变化。建议锁定具体版本。

2. **reedline + ratatui 共存**：两者都依赖 crossterm（间接），版本需要一致。当前 reedline 0.44 用 crossterm 0.29，ratatui-core 0.7 用 crossterm 0.29，版本兼容。

3. **Buffer → ANSI 转换性能**：每次 menu_string() 都要遍历整个 Buffer 做 ANSI 转换。对于一个 60x10 的补全菜单（600 cells），这应该在 <1ms 内完成，不影响体验。

4. **迁移期间的测试覆盖**：从 auto-shell 拆分到 ash-core + ash-tui 时，每个模块迁移后立即运行测试，确保无回归。

5. **不要过度设计**：ash-gui crate 现在只创建空目录和 Cargo.toml，不实现任何功能。GUI 前端是未来的事。

---

## 九、关于 Plan 293/294 是否需要单独更新

| Plan | 需要更新？ | 原因 |
|------|-----------|------|
| Plan 293 (AshMenu) | **需要小幅更新** | 渲染方式从"手写 ANSI"改为"ratatui widget → Buffer → ANSI"，文件位置从 auto-shell 移到 ash-tui。但核心设计（自适应布局、搜索、fuzzy）不变。 |
| Plan 294 (AshPrompt) | **基本不需要** | Prompt 渲染用 nu-ansi-term 足够，不需要 ratatui widget。只是文件位置从 auto-shell 移到 ash-tui。 |
| Plan 292 (Atom) | **不需要** | 纯数据层，不涉及渲染，只是目标 crate 变了。 |
| Plan 291 (Warp) | **不需要** | 愿景不变，只是 Phase 间有了更清晰的依赖关系。 |

**建议**：Plan 293 在实施时按新架构调整即可，不需要现在重写。Plan 295 已经包含了所有架构变更的指引。
