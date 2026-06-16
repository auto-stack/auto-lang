# Plan 317: Ash 24-bit Truecolor 支持

> **Status**: Draft
> **关系**: 增强 ash 的色彩能力。当前渲染管线已能发 24-bit 序列，但**没有任何色彩源真正使用 24-bit，也没有终端能力检测**。本计划补齐「检测 + 使用 + 优雅降级」三件套，达到 Fish 的「monospaced rainbow」水平。
> **参考**: Fish 的 `update_fish_color_support()`（[D:\github\fish-shell\src\env_dispatch.rs:372-446](file:///d:/github/fish-shell/src/env_dispatch.rs)）—— 终端色彩能力检测算法。

---

## 1. 现状（已核实）

| 能力 | 状态 |
|---|---|
| 渲染管线发 24-bit 序列 | ✅ `buffer_to_ansi.rs:119/143` 已把 `Color::Rgb(r,g,b)` → `\x1b[38;2;r;g;b`，且有测试（`:223-230` 断言 `38;2;255;0;128`） |
| 色彩源使用 24-bit | ❌ 高亮器（`highlight.rs:76-82`）只用 16 色命名色（Cyan/Green/...）；prompt、ls 表格同理 |
| 终端能力检测 | ❌ 全仓无 `COLORTERM`/`truecolor`/terminfo 检查 |
| 非真彩终端优雅降级 | ❌ 若现在发 Rgb 到非真彩终端，nu-ansi-term 照发 `38;2;...`，终端显示乱码/原样转义 |

**结论**：管道通、但不使用、不检测、不降级 → 实质上不支持 24-bit。

---

## 2. 参考：Fish 的做法

Fish 的 `update_fish_color_support()`（env_dispatch.rs:372）按优先级判定 `supports_24bit` / `supports_256color`：

1. **用户显式覆盖**：`$fish_term24bit` / `$fish_term256` 设了就用其布尔值。
2. **screen 特例**：`$STY` 存在（在 `screen` 内）→ 禁用 24-bit（screen 需 `truecolor on`）。
3. **`$COLORTERM`**：等于 `truecolor` 或 `24bit` → 启用 24-bit。
4. **默认推断**：除非 `$TERM == xterm-16color`，且非 Apple_Terminal → 启用 24-bit。

256 色同理（`$fish_term256` 覆盖，否则非 `xterm-16color` 即支持）。

发色时：想要的 RGB 色，若终端不支持 24-bit，**降采样到最近 256 色**（再不行到 16 色）再发。这样在任何终端都不乱码。

---

## 3. 设计

### 3.1 ColorDepth 检测（新模块 `frontend/term/color.rs`）

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum ColorDepth { True24, Index256, Index16 }

/// 检测当前终端的色彩深度（镜像 Fish 的 update_fish_color_support）。
pub fn detect_color_depth() -> ColorDepth { ... }
```

判定逻辑（按优先级）：
1. `$ASH_TERM24BIT` / `$ASH_TERM256`（ash 自己的覆盖，对应 Fish 的 `$fish_term24bit`）。
2. `$STY` 存在 → 降到 256（screen 特例）。
3. `$COLORTERM == truecolor|24bit` → True24。
4. 默认：`$TERM != xterm-16color` 且非已知不支持真彩的终端 → True24；否则按 256/16 推断。

> 检测结果**进程内缓存一次**（启动时算一次；环境变量运行时变化罕见，可加 `detect_color_depth()` 重算钩子作增强）。

### 3.2 色彩解析器 + 优雅降级（`color.rs`）

```rust
/// 把「期望色」按当前 ColorDepth 解析成实际可发的索引/RGB。
/// - True24：原样 Rgb。
/// - Index256：Rgb → 最近 xterm 256 色板索引。
/// - Index16：Rgb → 最近 16 色命名色。
pub fn resolve_for_term(rgb: (u8,u8,u8)) -> ResolvedColor { ... }
```

- **256 降采样**：标准 xterm 256 色板（16 命名 + 216 立方 + 24 灰阶），对 RGB 做最近邻（欧氏距离，可选加权）。
- **16 降采样**：映射到 16 命名色（黑/红/绿/黄/蓝/品红/青/白 各明暗）。

`ResolvedColor` 在 `buffer_to_ansi` 发色时用：
- True24 → `AnsiColor::Rgb`（现状）。
- Index256 → `AnsiColor::Fixed(n)`。
- Index16 → `AnsiColor::<命名色>`。

### 3.3 接入 `buffer_to_ansi`

`cell_style_to_ansi(fg, bg, modifier)` 当前直接把 ratatui `Color` 转 nu-ansi-term。改为：遇到 `Color::Rgb(r,g,b)` 时，先经 `resolve_for_term` 按检测到的深度降级，再发。这样**所有走 ratatui buffer 的 24-bit 色自动获得降级**（ls 表格、未来任何 RGB 渲染）。

### 3.4 高亮器 / 主题用上 24-bit

- **语法高亮**：`AshHighlighter` 当前硬编码 16 色命名色。改为读「主题」（默认提供一个 24-bit 主题，如 One Dark / Catppuccin 风格的 hex 值），经 `resolve_for_term` 降级后输出。
- **主题配置**：`~/.config/ash.toml` 或 `.at` 里 `theme = "dark"` / 自定义 hex（如 `[syntax] string = "#98c379"`）。
- 默认主题：在真彩终端显示更柔和准确的 24-bit 色；非真彩自动降到 256/16，仍可读。

### 3.5 reedline 高亮路径

注意：reedline 的语法高亮走 `Highlighter` trait 返回 `StyledText`（nu-ansi-term Style）。这条路径**不经 ratatui buffer**，需要单独确保高亮器返回的色按 `ColorDepth` 降级（在 `AshHighlighter` 内部处理，或包一层）。

---

## 4. 实现阶段

### Phase 1：检测 + 降级基础设施（无功能变化，铺路）

1. `frontend/term/color.rs`：`ColorDepth` + `detect_color_depth()`（镜像 Fish 算法）+ `resolve_for_term(rgb)`（256/16 降采样）+ 单测。
2. `buffer_to_ansi`：`Color::Rgb` 经 `resolve_for_term` 降级后再发。
3. 检测结果缓存（进程内 once）。
4. **验证**：在真彩终端 Rgb 原样发；模拟非真彩时降到 Fixed/命名色（单测 + 一个 `force ColorDepth` 的测试钩子）。

### Phase 2：高亮器用 24-bit 主题

1. 默认语法主题（24-bit hex：keyword/string/comment/...）。
2. `AshHighlighter` 读主题，色值经 `resolve_for_term` 降级后输出（兼容 reedline 路径）。
3. `theme` 配置项（`ash.toml`/`.at`）：选内置主题或自定义 hex。
4. **验证**：真彩终端看到 24-bit 高亮；非真彩降到 256/16 仍可读；`echo` 一段彩色测试。

### Phase 3：彩虹 / ls / prompt 体验

1. ls 文件类型色可用 24-bit（按扩展名更细腻的着色）。
2. prompt 主题色 24-bit。
3. 一个 `ash` 内置 demo（如 `color rainbow`）打印 24-bit 渐变，等价 Fish 的「monospaced rainbow」——**直观验证 24-bit 生效**。
4. **验证**：rainbow 在真彩终端是平滑渐变；非真彩是 256 色阶梯。

---

## 5. 边界与风险

| 风险 | 应对 |
|---|---|
| 检测误判（终端不支持却被判 True24）→ 乱码 | 保守默认 + `$ASH_TERM24BIT` 覆盖；参考 Fish 的 xterm-16color/screen/Apple_Terminal 黑名单 |
| 256 降采样失真 | 用 xterm 标准 256 色板 + 加权欧氏距离（人眼对绿更敏感）；可选 perceptual 距离 |
| reedline 高亮路径不经 buffer | Phase 2 在 AshHighlighter 内部降级，不依赖 buffer_to_ansi |
| 性能（每字符查色板） | 降采样查表 O(1)；检测结果缓存 |
| Windows Terminal 默认 COLORTERM | Windows Terminal 设 `COLORTERM=truecolor` → 自动 True24 ✓ |

---

## 6. 验证

- `cargo test`：`color.rs` 单测（检测各 env 组合、256/16 降采样最近邻正确性、buffer_to_ansi 降级）。
- 手动：`COLORTERM=truecolor` 下跑高亮 + `color rainbow` → 24-bit 平滑渐变；`COLORTERM=` 空 + `TERM=xterm-16color` 下 → 16 色仍可读、无乱码。
- 回归：现有 16 色行为在「检测为 16 色的终端」下不变。

---

## 7. 关键文件（预估改动）

| 文件 | 改动 |
|---|---|
| `auto-shell/src/frontend/term/color.rs` | **新**：ColorDepth + detect + resolve_for_term + 降采样 |
| `auto-shell/src/frontend/renderer/buffer_to_ansi.rs` | `Color::Rgb` 经 resolve_for_term 降级 |
| `auto-shell/src/frontend/term/highlight.rs` | 读 24-bit 主题 + 降级输出 |
| `auto-shell/src/prompt/*` | prompt 色 24-bit（Phase 3） |
| `auto-shell/src/config.rs` / `auto_config.rs` | `theme` 配置项 |
| `auto-shell/src/shell.rs` | `color rainbow` demo builtin（Phase 3） |
