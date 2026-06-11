# Plan 293: AshMenu — 自适应补全菜单

## 设计目标

用一个自建的 `AshMenu` 替换 reedline 的 `ColumnarMenu`，参考 Fish Shell 的 Pager 设计，实现：

1. **自适应布局**：无描述时紧凑多列网格，有描述时展开为名称+描述列表
2. **半屏 Pager**：首次弹出占终端 50% 高度，信息密度高
3. **内置搜索**：激活后可输入字符实时过滤补全列表
4. **分类着色**：按补全类型（命令/文件/变量/flag）分色显示
5. **Fuzzy 匹配**：用 `nucleo-matcher` 做模糊匹配并高亮命中字符
6. **可扩展**：为后续 AI 补全（Plan 291 Phase 3）预留分类标记

## 设计参考

### Fish Shell Pager 的核心设计

Fish 的 Pager（~700 行 Rust）之所以是「金标准」，因为遵循两条铁律：
- **可发现性法则**：每个补全都有描述
- **响应性法则**：永远不冻结 UI

Fish Pager 的布局策略：
- 所有补全项只有名称、无描述 → 紧凑多列网格
- 有描述 → 名称左对齐 + 描述右侧，自适应列宽
- 占半屏高度，用 PageUp/PageDown 翻页
- Ctrl+S 进入搜索模式，实时过滤

### AshMenu 的布局模式

```
输入: "ls sr"
┌──────────────────────────────────────────┐
│ src/   src-tauri/   src-web/   srg/      │  ← 紧凑网格（无描述）
│ sro/   srt/         srv/                  │
└──────────────────────────────────────────┘

输入: "cargo "
┌──────────────────────────────────────────┐
│ 📦 build       编译当前项目                │  ← 名称+描述（有描述）
│ 📦 check       快速语法检查                │
│ 📦 test        运行所有测试                │
│ 📦 run         运行二进制文件               │
│ 📄 build.rs    匹配文件                    │
└──────────────────────────────────────────┘
```

布局选择不是用户切换的，而是由数据驱动的：
- 所有项都无描述 → `CompactGrid`
- 任一项有描述 → `DescriptiveList`

---

## 架构设计

### 整体结构

```
reedline Menu trait (接口层)
    ↑ implements
AshMenu (自适应菜单)
    ├── AshSuggestion (增强的补全项: name + description + kind + icon)
    ├── LayoutEngine (布局引擎: 自动选择 CompactGrid / DescriptiveList)
    ├── FuzzyMatcher (模糊匹配: nucleo-matcher + 高亮)
    └── SearchState (内置搜索: 实时过滤)
```

### 核心类型定义

```rust
// === 补全项增强 ===

/// 补全来源分类
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionKind {
    Command,      // 内置命令 (ls, cd, grep)
    External,     // 外部命令 (git, cargo)
    File,         // 文件路径
    Directory,    // 目录路径
    Variable,     // 环境变量 ($PATH)
    Flag,         // 标志参数 (--verbose)
    Subcommand,   // 子命令 (cargo build)
    AiSuggested,  // AI 推荐
}

/// 增强的补全建议（内部表示）
#[derive(Debug, Clone)]
pub struct AshSuggestion {
    /// 显示名称（用于渲染和匹配）
    pub display: String,
    /// 实际替换文本
    pub replacement: String,
    /// 可选描述
    pub description: Option<String>,
    /// 补全类型（决定颜色和图标）
    pub kind: CompletionKind,
    /// fuzzy 匹配高亮索引（由 FuzzyMatcher 填充）
    pub match_indices: Vec<usize>,
}

/// AshMenu 配置
pub struct AshMenuConfig {
    /// 菜单名称
    pub name: String,
    /// 最大显示行数（0 = 自动，取终端高度的 50%）
    pub max_visible_lines: u16,
    /// 是否启用内置搜索
    pub search_enabled: bool,
    /// 是否启用 fuzzy 匹配
    pub fuzzy_match: bool,
    /// 紧凑网格模式的最小列宽
    pub min_column_width: usize,
    /// 列间距
    pub column_padding: usize,
}

impl Default for AshMenuConfig {
    fn default() -> Self {
        Self {
            name: "ash_menu".to_string(),
            max_visible_lines: 0,        // 自动
            search_enabled: true,
            fuzzy_match: true,
            min_column_width: 12,
            column_padding: 2,
        }
    }
}
```

### 布局引擎

```rust
/// 布局策略
enum LayoutMode {
    /// 紧凑网格：所有项无描述时使用
    /// 类似 Fish Pager 的默认模式
    CompactGrid {
        columns: u16,
        col_width: usize,
        rows: u16,
    },
    /// 描述列表：任一项有描述时使用
    /// 名称左对齐，描述右对齐
    DescriptiveList {
        name_width: usize,
        rows: u16,
    },
}

/// 根据补全数据自动选择布局
fn choose_layout(suggestions: &[AshSuggestion], terminal_width: u16) -> LayoutMode {
    let has_any_desc = suggestions.iter().any(|s| s.description.is_some());
    if has_any_desc {
        let name_width = suggestions.iter()
            .map(|s| unicode_width::UnicodeWidthStr::width(s.display.as_str()))
            .max()
            .unwrap_or(20)
            .min(40); // 名称最大 40 字符宽
        DescriptiveList { name_width, rows: ... }
    } else {
        let max_item_width = suggestions.iter()
            .map(|s| unicode_width::UnicodeWidthStr::width(s.display.as_str()))
            .max()
            .unwrap_or(12);
        let columns = (terminal_width as usize) / (max_item_width + column_padding).max(1);
        CompactGrid { columns, col_width: max_item_width, rows: ... }
    }
}
```

### 颜色方案

```rust
/// 按类型着色
fn kind_color(kind: &CompletionKind) -> nu_ansi_term::Style {
    match kind {
        CompletionKind::Command    => Color::Cyan.bold(),           // 青色粗体
        CompletionKind::External   => Color::Green.normal(),       // 绿色
        CompletionKind::File       => Color::White.normal(),       // 白色
        CompletionKind::Directory  => Color::Blue.bold(),          // 蓝色粗体
        CompletionKind::Variable   => Color::Magenta.normal(),     // 紫色
        CompletionKind::Flag       => Color::Yellow.normal(),      // 黄色
        CompletionKind::Subcommand => Color::Cyan.normal(),        // 青色
        CompletionKind::AiSuggested => Color::LightGreen.italic(), // 浅绿斜体
    }
}
```

### 搜索模式

参考 Fish Pager 的 Ctrl+S 搜索：

```rust
/// 搜索状态
struct SearchState {
    /// 是否处于搜索模式
    active: bool,
    /// 搜索查询字符串
    query: String,
    /// 过滤后的索引（原始 values 中的下标）
    filtered_indices: Vec<usize>,
}

// 搜索模式下，按键处理：
// - 普通字符：追加到 query，重新过滤
// - Backspace：删除 query 最后字符，重新过滤
// - Enter/Esc：退出搜索，保持当前过滤结果
// - 方向键：在过滤结果中导航
```

### 文件结构

```
crates/auto-shell/src/
├── menu/                          ← 新增目录
│   ├── mod.rs                     ← 模块导出
│   ├── ash_menu.rs                ← AshMenu 主结构（impl reedline::Menu）
│   ├── suggestion.rs              ← AshSuggestion, CompletionKind
│   ├── layout.rs                  ← LayoutMode, 布局计算引擎
│   ├── render.rs                  ← 渲染：compact_grid(), descriptive_list()
│   ├── search.rs                  ← SearchState, 搜索过滤逻辑
│   └── style.rs                   ← 颜色方案, kind_color(), MenuTextStyle
├── completions/
│   ├── mod.rs                     ← 扩展 Completion 结构体
│   ├── command.rs                 ← 返回带 kind 的补全
│   ├── file.rs                    ← 返回带 kind 的补全
│   ├── auto.rs                    ← 返回带 kind 的补全
│   └── reedline.rs                ← ShellCompleter 适配 AshMenu
└── repl.rs                        ← 替换 ColumnarMenu → AshMenu
```

---

## 实现任务

### Task 1：扩展 Completion 数据结构
**文件**: `crates/auto-shell/src/completions.rs`

- [ ] 在 `Completion` struct 中添加 `description: Option<String>` 字段
- [ ] 添加 `kind: CompletionKind` 字段（默认 `Command`）
- [ ] 更新所有 `complete_command()` / `complete_file()` / `complete_auto()` 调用点，填充 kind
  - command.rs → `CompletionKind::Command`
  - file.rs → 目录 `CompletionKind::Directory`，文件 `CompletionKind::File`
  - auto.rs → `CompletionKind::Variable`
- [ ] 运行 `cargo test -p auto-shell` 确保现有测试通过

### Task 2：创建 AshMenu 框架
**文件**: `crates/auto-shell/src/menu/` (新建目录)

- [ ] 创建 `menu/mod.rs`，导出子模块
- [ ] 创建 `menu/suggestion.rs`，定义 `AshSuggestion`、`CompletionKind`
- [ ] 创建 `menu/ash_menu.rs`，实现 `AshMenu` struct 并 `impl reedline::Menu`
  - 基础字段：`active`, `values: Vec<Suggestion>`, `selected: usize`, `settings: MenuSettings`
  - 先用最简单的 ColumnarMenu 逻辑作为起点（逐方法替换）
  - 实现 `update_values()`, `replace_in_buffer()`, `menu_string()`, `menu_required_lines()`
- [ ] 创建 `menu/style.rs`，定义颜色方案 `kind_color()`
- [ ] 确保 `cargo build -p auto-shell` 编译通过

### Task 3：自适应布局引擎
**文件**: `crates/auto-shell/src/menu/layout.rs`, `menu/render.rs`

- [ ] 创建 `layout.rs`，实现 `LayoutMode` enum 和 `choose_layout()` 函数
  - CompactGrid：计算列数、行数、列宽
  - DescriptiveList：计算名称宽度、行数
- [ ] 创建 `render.rs`，实现两个渲染函数：
  - `render_compact_grid()`: 多列网格，每项一个 cell，类型着色
  - `render_descriptive_list()`: 名称 + 描述两栏，类型着色
- [ ] 在 `menu_string()` 中集成布局选择和渲染
- [ ] 手动测试：输入 `ls `（文件补全→紧凑网格），验证渲染正确

### Task 4：类型着色和选中高亮
**文件**: `crates/auto-shell/src/menu/style.rs`, `menu/render.rs`

- [ ] 完善颜色方案：按 `CompletionKind` 分色
- [ ] 选中项高亮：反转背景色（和 reedline 的 `selected_text_style` 一致）
- [ ] 文件/目录后缀标识：目录带 `/`，可执行文件带 `*`
- [ ] 手动测试验证视觉效果

### Task 5：键盘导航和翻页
**文件**: `crates/auto-shell/src/menu/ash_menu.rs`

- [ ] 方向键导航：Up/Down 移动选中，Left/Right 在网格模式中水平移动
- [ ] Tab/Shift-Tab：下一个/上一个补全项
- [ ] PageUp/PageDown：翻页（当补全项超过可见行数时）
- [ ] Home/End：跳到第一项/最后一项
- [ ] 半屏高度：`menu_required_lines()` 返回 `min(items_rows, terminal_height / 2)`
- [ ] 翻页滚动：当 `selected` 超出可见区域时，调整 `skip_rows`

### Task 6：内置搜索过滤
**文件**: `crates/auto-shell/src/menu/search.rs`

- [ ] 创建 `SearchState` struct
- [ ] 搜索模式激活：Ctrl+S（或 `/`）进入搜索
- [ ] 实时过滤：每次输入字符后，对 `values` 做 prefix/fuzzy 过滤
- [ ] 搜索模式下导航：在过滤后的子集中移动选中项
- [ ] 退出搜索：Esc 清除搜索并恢复完整列表，Enter 接受当前选中
- [ ] 搜索提示：搜索模式激活时，在 menu 顶部显示搜索框 `/query`

### Task 7：Fuzzy 匹配（可选增强）
**依赖**: `nucleo-matcher` crate

- [ ] 添加 `nucleo-matcher` 依赖到 `Cargo.toml`
- [ ] 创建 `menu/fuzzy.rs`，封装 fuzzy match 逻辑
- [ ] 匹配高亮：在渲染时，对 fuzzy 命中的字符加下划线
- [ ] 匹配排序：fuzzy score 高的排在前面
- [ ] 配置项：`fuzzy_match: bool` 开关（默认开启）

### Task 8：替换 REPL 中的 ColumnarMenu
**文件**: `crates/auto-shell/src/repl.rs`, `completions/reedline.rs`

- [ ] `repl.rs`: 将 `ColumnarMenu::default()...` 替换为 `AshMenu::new(AshMenuConfig::default())`
- [ ] `completions/reedline.rs`: `ShellCompleter` 返回的 `Suggestion` 填充 `description` 和 `style` 字段
- [ ] 从 `repl.rs` 移除 `ColumnarMenu` 的 import，改为导入 `AshMenu`
- [ ] Tab 键绑定保持不变（reedline 的 `UntilFound` + `Menu` + `MenuNext`）
- [ ] 运行 `cargo test -p auto-shell` 全量测试
- [ ] 手动端到端测试：
  - `ls <Tab>` → 文件补全，紧凑网格
  - `cd <Tab>` → 目录补全，紧凑网格
  - `l<Tab>` → 命令补全，紧凑网格
  - `echo $P<Tab>` → 变量补全，紧凑网格
  - 搜索模式 Ctrl+S → 输入过滤

---

## 风险和注意事项

1. **reedline Menu trait 的 `menu_string()` 返回纯 String**
   - 渲染完全由我们控制，只要输出正确的 ANSI 转义序列即可
   - 参考 reedline 自带的 ColumnarMenu/IdeMenu 实现模式

2. **同步阻塞**
   - 当前 reedline 的 `Completer::complete()` 是同步的
   - AshMenu 本身不解决异步问题，但通过减少不必要的补全计算和搜索过滤来保持响应速度
   - 异步补全是后续 Plan 291 Phase 3 的内容

3. **跨平台 ANSI 兼容性**
   - Windows Terminal 和主流终端都支持 ANSI escape sequences
   - 使用 `nu-ansi-term`（已有依赖）处理 ANSI 输出，自动适配平台

4. **不修改 reedline crate**
   - AshMenu 在 auto-shell crate 内实现，只实现 reedline 的 `Menu` trait
   - 不 fork reedline，保持依赖关系简单

---

## 与其他 Plan 的关系

| Plan | 关系 |
|------|------|
| Plan 291 (Warp-style) | AshMenu 是 Phase 4 Block UX 的前置，但可独立实施 |
| Plan 292 (Atom Pipeline) | 补全数据未来可从 Atom pipeline 获取，当前先用 Completion struct |
| Plan 153 (AI Agent) | `CompletionKind::AiSuggested` 为 AI 补全预留分类 |

---

## 关于 Prompt 计划

Prompt（参考 Starship）建议 **单独做一个计划**，原因：

1. **关注点不同**：AshMenu 解决「补全弹出菜单的交互体验」，Prompt 解决「提示符的美化和信息密度」。两者代码不重叠。
2. **优先级不同**：补全菜单直接影响工作效率（每次 Tab 都会用到），Prompt 是美化性质（看得舒服但不影响功能）。AshMenu 优先级更高。
3. **技术栈不同**：AshMenu 实现 `reedline::Menu` trait，Prompt 实现 `reedline::Prompt` trait + 自建模块系统。实现者可以不同。
4. **复杂度不同**：AshMenu ~800 行，Prompt 模块系统 ~2000 行（需要并行计算、TOML 配置、100+ 模块定义）。

建议命名：**Plan 294: AshPrompt — 模块化 Prompt 引擎**
