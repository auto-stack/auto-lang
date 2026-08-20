# Plan 407: 扫雷 render=rust 版完整支持

> **状态（2026-08-20 核查更新）**: 🟡 Phase 1–2 完成 + Phase 3 部分，均已合并 master（merge `f863be5e`）：Phase 1 render=rust 启动成功（acb6c759 + 2268a67b R1/R2/R4/store-on + 83dbb259 语句支持 + f7318259 LCG 溢出修复）、Phase 2 计时器（a130512a tick_msg + 871b4226 tick_interval_ms）、Phase 3 的 R6 右键 oncontextmenu（2fc6c562）与 R9 grid 居中（fb4870c2）已落地。**未做**：R7 动态窗口 resize（difficulty 切换→窗口尺寸，ui_gen/rust.rs 无 resize 逻辑，生成 main.rs 固定 370×506）、Phase 4（三后端对比验证 + 015/011 回归）。
> **优先级**: 中 — vue 版和 VM 版已可用，rust 版是第三后端
> **目标**: 038 扫雷 render=rust 版达到与 VM 版完全一致的功能（左键展开、右键插旗、计时、难度切换、动态窗口）

## 1. 背景

038 扫雷的 vue 版和 VM 版（render=vm）已完整可用。尝试 `auto run --render rust` 时，
a2r（Auto-to-Rust）codegen 生成的 Rust 代码有 **37 个编译错误**，完全无法编译。

根因是 a2r codegen 对 store 驱动的 widget、Obj 字面量、bool 类型、运算优先级等特性
的支持不完整。本计划系统修复这些问题，直到 rust 版与 VM 版功能一致。

## 2. 已发现的 a2r codegen bug

### 2.1 编译错误（阻塞性，37 个）

#### Bug R1: 运算优先级 — `.to_string()` 绑定错误
**现象**: `(.mine_count - .flags_placed).to_string()` 生成 `mine_count - flags_placed.to_string()`
**根因**: parser 或 a2r 把方法调用（`.to_string()`）的优先级高于减法（`-`），导致 `.to_string()` 绑定到 `flags_placed` 而非整个子表达式。
**影响**: 所有 `(expr).method()` 形式的表达式（store 里的 label 计算）
**位置**: parser 优先级 或 `ui_gen/rust.rs ast_expr_to_rust`

#### Bug R2: String move — view 引用 store String 字段
**现象**: `self.store.mines_label` 直接传入 `View::text_styled()` 导致 `cannot move out` 错误
**根因**: a2r 生成的 view 代码直接引用 `self.store.xxx`（String 类型），Rust 所有权规则要求 clone
**修复**: view 里引用 String 字段时自动加 `.clone()` 或 `&self.store.xxx`
**位置**: `ui_gen/rust.rs generate_view_tree` / `generate_view_method`

#### Bug R3: store 消息调用语法错误
**现象**: `self.store.on(MinesweeperStoreMsg::Init)` — `on` 方法不存在
**根因**: store handler 调用用了错误的 `on()` 语法，应该是直接方法调用（`self.store.init()` 或等价机制）
**影响**: 所有 `store.XXX()` 调用（Init/Reset/Reveal/Flag/SetDifficulty/Tick）
**位置**: `ui_gen/rust.rs generate_handler_body` / store 调用转换

#### Bug R4: Obj/HashMap 字段访问
**现象**: board 元素（Obj 字面量）在 Rust 里用 `cell["display"]` HashMap 索引，但类型和方法不匹配
**根因**: AutoLang Obj 字面量 `{ x: 0, y: 0, ... }` 在 a2r 里转译成 `serde_json::Value`（Object），字段访问用 `cell["display"].as_str()`。但生成的代码类型不匹配（`.as_str().unwrap_or_default().to_string()` 嵌套错误、`.as_str()` 返回 `&str` 再 `.to_string()` 再 `.as_str()` 链式调用出错）
**影响**: 所有 board 元素的字段读写（display/cell_class/number_class/x/y/mine/revealed/flagged/adjacent）
**位置**: `ui_gen/rust.rs ast_expr_to_rust` Dot/Index 分支

#### Bug R5: 整数运算与 to_string 混用
**现象**: `format!("{}{}", "💣 ", self.mine_count - self.flags_placed.to_string())` — `i32 - String` 类型错误
**根因**: Bug R1 的直接后果（`.to_string()` 绑定错误），但也暴露 a2r 对混合类型运算的处理缺陷
**位置**: 同 Bug R1

### 2.2 功能缺失（非阻塞性，但影响一致性）

#### Bug R6: 右键 oncontextmenu 支持
**现象**: render=rust 可能不支持 oncontextmenu → on_right_click
**根因**: `View::Button` 的 `on_right_click` 字段在 rust mode 的 `into_iced` 里有 mouse_area 包裹，但 a2r codegen 可能不生成 on_right_click 的绑定代码
**验证**: 编译通过后测试右键

#### Bug R7: 动态窗口 resize
**现象**: render=rust 没有 update 闭包里的 difficulty→resize 逻辑（那是 VM 专有的）
**根因**: VM 的 resize 逻辑在 `renderer.rs` 的 update 闭包里，rust mode 走不同的代码路径
**方案**: 在 rust mode 的 update 里也加 difficulty→resize，或通过 widget model 的 window_width/window_height

#### Bug R8: 计时器 Tick
**现象**: rust mode 的 Tick 触发机制可能与 VM 不同
**验证**: 编译通过后测试计时

#### Bug R9: grid 布局一致性
**现象**: VM 修了 build_grid 的 Fill wrapper / align_x 问题，rust mode 走 `into_iced`（rust mode 专用），需要确认同样修复生效
**验证**: 编译通过后对比布局

## 3. 任务分解

### Phase 1: 编译通过（修复 37 个编译错误）⚪
**目标**: `auto run --render rust` 能编译并显示初始窗口

- [ ] **R1: 运算优先级** — 修复 parser 或 a2r 对 `(expr).method()` 的处理
  - 确认根因在 parser 还是 codegen
  - 如果在 parser：修复方法调用 vs 算术运算的优先级
  - 如果在 codegen：在 ast_expr_to_rust 的 Bina 分支正确处理子表达式的方法调用
  - 临时 workaround：store .at 里拆成变量

- [ ] **R2: String clone** — view 引用 store String 字段时自动 clone
  - 在 generate_view_tree 里，引用 `self.store.xxx`（String）的地方加 `.clone()` 或传 `&`
  - 影响：mines_label, timer_label, cell_class, number_class 等

- [ ] **R3: store 消息调用** — 修复 `store.XXX()` 的 Rust 转译
  - 确认 store handler 在 rust mode 如何调用（直接方法？消息枚举？）
  - 修复 generate_handler_body 里的 store 调用语法

- [ ] **R4: Obj 字段访问** — 修复 board 元素的 HashMap/serde_json 字段访问
  - 统一 Obj 字面量在 Rust 里的表示（serde_json::Value::Object）
  - 修复字段读取：`cell["display"].as_str().unwrap_or_default().to_string()`
  - 修复字段写入：`cell["mine"] = json!(true)` 等

- [ ] **R5: 验证编译通过** — `cargo build` 无错误

### Phase 2: 基础功能（左键展开 + 计时）⚪
**目标**: 能开始游戏、点击展开、计时器走动

- [ ] 左键 Reveal 正确触发布雷 + flood-fill
- [ ] 计时器 Tick 正常更新 elapsed
- [ ] 信息栏（💣数/⏱时间）正确显示
- [ ] 难度切换重建棋盘

### Phase 3: 完整功能（右键 + 布局 + 窗口）⚪
**目标**: rust 版与 VM 版功能完全一致

- [ ] **R6: 右键插旗** — 确认 oncontextmenu → on_right_click 在 rust mode 生效
- [ ] **R7: 动态窗口 resize** — rust mode 也支持 difficulty→窗口大小
- [ ] **R8: 计时器** — 确认 rust mode 的 Tick 机制
- [ ] **R9: grid 布局** — 确认紧凑网格 + 居中 + gap 一致

### Phase 4: 验证与回归 ⚪
- [ ] 三后端功能对比（vue / VM / rust）
- [ ] 015-notes 回归（确保 a2r 修改不破坏其他示例）
- [ ] 011-calculator 回归（已有 rust mode 示例）

## 4. a2r codegen 架构要点

```
AutoLang .at
    ↓ (parser)
AuraWidget AST
    ↓ (ui_gen/rust.rs generate_rust)
Rust main.rs (struct + view + update + handlers)
    ↓ (cargo build)
原生 iced 二进制
```

关键函数（`crates/auto-lang/src/ui_gen/rust.rs`）：
- `generate_rust` (431): 入口，生成整个 main.rs
- `generate_view_method` (1436): 生成 view() 方法
- `generate_view_tree` (1899): 生成 view 树（button/grid/col/row/text 等）
- `ast_expr_to_rust` (4005): 表达式 → Rust 代码
- `generate_handler_body` (3508): handler body → Rust 代码

store handler 转译涉及：
- `trans/rust.rs`: AutoLang 逻辑 → Rust 代码（算术/循环/条件/赋值）
- `ui_gen/rust.rs`: widget 结构 + store 集成

## 5. 风险

- **a2r codegen 修改面广**: 5447 行的 rust.rs + trans/rust.rs，修改可能引入回归
- **store 驱动是新模式**: 已有 rust mode 示例（011-calculator 等）可能不使用 store，store 支持是全新的
- **缓解**: 每个 bug 修复后立即编译验证；Phase 4 做三后端对比 + 回归测试

## 6. 相关文件

| 文件 | 作用 |
|---|---|
| `crates/auto-lang/src/ui_gen/rust.rs` | a2r widget codegen（主要修改） |
| `crates/auto-lang/src/trans/rust.rs` | a2r 逻辑转译（handler body） |
| `crates/auto-man/src/rust_ui.rs` | Rust 项目生成器（Cargo.toml + main.rs 组装） |
| `crates/auto-lang/src/aura/extract.rs` | .at → AuraWidget AST（parser 层面可能需修优先级） |
| `examples/ui/038-minesweeper/src/front/app.at` | 扫雷 widget（view 壳） |
| `examples/ui/038-minesweeper/src/front/minesweeper_store.at` | 扫雷 store（游戏逻辑） |

## 7. Plan 406 关联

Plan 406（VM 类型系统审计）修复的 VM bug 中，部分（bool 编码、to_int 等）也可能影响 a2r
codegen。a2r 生成的是 Rust 原生代码，不经过 VM nanbox 类型系统，所以 nanbox 相关 bug 不影响
rust 版。但 bool/string 的类型处理在 a2r 里可能有自己的 bug（独立于 VM）。
