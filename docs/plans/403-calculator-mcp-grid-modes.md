# Plan 403: 011-calculator 扩展 — MCP 操纵 + Grid 布局 + 多模式 UI

> **状态（2026-08-08）**: 🟡 规划中。需求已明确，调研完成，待实施。
> **动机**: 011 是纯前端整数加减乘除玩具（325 行单文件、col/row 嵌套 + 22 个硬编码样式、无括号/小数、README 与代码脱节）。本计划把它扩展为可被 MCP 完整操纵、grid 布局、并支持多模式（Scientific/Programmer）的示例。
> **与 Plan 401 的关系**: 401 是"018-027 玩具→完整 App 升级"。011 的扩展性质不同——涉及 MCP 基建（新工具）+ grid 重构 + 多模式 UI 工程，是独立主题，故单独立项。401 §待办已加指引"→ 见 Plan 403"。

---

## 调研结论（已完成）

### 011 现状
- 纯前端单文件 `app.at`（325 行），无后端、无测试。
- 仅整数加减乘除（`var val int`），**无小数**（`.Dot` 是空 handler）、**无括号**、**无运算符优先级**（`2+3*4` = `(2+3)*4` = 20）。
- bug：`%` 键误接到 `.Dot`（`app.at:32`）。
- 布局：`col > row*5`，每按钮 inline `style:` tailwind 字面量（~22 个）。最后一行只有 3 按钮（0/.//=）。
- 4 个操作符 handler（Add/Sub/Mul/Div）是复制粘贴（~90 行重复）。
- README 与代码脱节（README 写 double/Percent/calc-btn-* 类，代码是 int/Negate/inline style）。

### MCP 操纵能力（关键发现：已大部分就绪）
**AutoUI MCP 已完整支持按键操纵**（`crates/auto-lang/src/ui/mcp_server.rs`，12 个 `autoui_*` 工具，:9247）：
- `autoui_action(element_id, action:"press")` — 点按钮（需先 `autoui_find` 解析 label→id）。
- `autoui_keyboard(key)` — 发控制键（Enter/Escape/Backspace…），**但不支持数字/运算符字符**（enum 限控制键，`mcp_server.rs:730`）。
- `autoui_state(fields:["display"])` — 读显示值。
- 已有 Python 测试范式：`examples/ui/013-todo/tests/desktop_mcp.py` + `autotest/` DSL（given/when/then → MCP 调用）。

**缺口（需新增基建）**：
1. 无"批量按键序列"工具——求值 `2+3` 要 8 次 round-trip（find+press × 4）。需新增 `autoui_press_sequence`。
2. `autoui_keyboard` 不支持数字/运算符——若想让 MCP 像真人敲键盘那样输入 `2`/`+`/`3`，需扩展该工具或走 `autoui_action press`（按钮点击）。

### Grid 布局（关键发现：双路径已生产就绪）
- `grid` 在 vue（CSS grid via Tailwind `grid`/`grid-cols-N`/`gap-N`）和 iced（列分行分解 `build_grid`，`renderer.rs:814`）都已就绪。
- 016-calendar 已用 `grid { grid-item... cols:N gap:M style:"..." }`（`app.at:32-53`）。
- 比 col/row 嵌套简洁：一个容器替代 5 个 row + 每按钮 flex-1。
- **限制**：无 col-span（若 `=` 要跨两列需该行降级回 row）；用 prop 名 `cols`（不是 schema 写的 `columns`）。

---

## 需求与方案

### 需求 1：完整按键 MCP 操纵 + 表达式求值接口

**1a. 验证现有按键操纵能力（本轮必做，零基建）**
- 011 已有 `bind { "7" -> .Digit7, "Enter" -> .Equals, ... }`（`app.at:71-89`）。
- 用 `autoui_action press`（点按钮）+ `autoui_state`（读 display）跑通 `2+3=5`。
- 写 `tests/desktop_mcp.py`（对齐 013/015 范式）+ acceptance 契约。

**1b. 新增 `autoui_press_sequence` MCP 工具（基建，本轮做）**
- 工具签名：`autoui_press_sequence { keys: ["2","+","3","="], delay_ms?: 50 }`。
- 实现：逐个 key 解析为按钮（按 label 匹配 `button "2"`）→ `autoui_action press`，最后可选读 state。
- 验证：`autoui_press_sequence { keys: ["2","+","3","="] }` → 返回 `display: "5"`。
- **简化**：当前计算器无 `(` `)`，故先支持 `1+2*3` 这类（无括号）。括号表达式求值依赖需求 1c（或后置）。

**1c. 运算符优先级 / 表达式求值（可选，本轮可做简化版）**
- 当前计算器是链式左到右（无优先级）。要做 `2+3*4=14`（优先级）或 `2*(3+4)=14`（括号），需改计算逻辑。
- **简化方案**：本轮保持链式（`1+2*3` = 按键顺序求值），MCP 表达式求值接口先支持无括号的按键序列。优先级/括号作为"计算引擎升级"后置（见需求 3 或独立任务）。

### 需求 2：Grid 布局重构

把 `col > row*5` + 22 个 inline style 改成 `grid { grid-item }`：
```auto
grid {
    grid-item { button "C" { onclick: .Clear, style: "bg-gray-600 text-white rounded-lg p-4 text-lg" } }
    grid-item { button "+/-" { onclick: .Negate, ... } }
    ... (每个按钮一个 grid-item)
    cols: 4
    gap: 1
    style: "w-full p-2 bg-gray-800 rounded-b-2xl"
}
```
- 去掉每按钮的 `flex-1`（grid track 管宽度）+ 5 个 `row { ...; style: "w-full" }` 包裹。
- 修复 `%` 键误接（接到 .Dot 的 bug）——给 `%` 一个真实 handler（取模或百分号）或移除。
- `=` 跨两列：若要，该行降级回 `row`（grid 无 col-span）；或保持 4 列均分。

### 需求 3：多模式 UI（Scientific / Programmer）

**较高要求，本轮可后置或先做参考版**：
- **方案 A（先做 shadcn-vue 参考版）**：在 `examples/ui/011-calculator/reference/` 放一个纯 shadcn-vue 的 Scientific/Programmer 计算器工程（手写 Vue），作为 Auto 实现的对标设计稿。
- **方案 B（直接用 Auto）**：用 `if .mode == "basic"` / `"scientific"` / `"programmer"` 切换三套 grid 布局。模式切换按钮在顶部。Programmer 模式需十六进制/位运算（计算逻辑复杂）。
- **本轮建议**：先做基础模式 grid 重构（需求 2），Scientific/Programmer 作为后续 Phase（依赖计算引擎升级）。

---

## 计算引擎升级（支撑需求 1c / 3，可后置）

当前是链式整数引擎（无优先级/小数/括号）。要做真正的表达式求值需：
- **小数支持**：`var val int` → 浮点（或字符串表达式）。
- **表达式求值**：把 `display` 当表达式字符串，`=` 时用 shunting-yard 或递归下降解析求值（支持优先级 + 括号）。
- 这是个独立子任务，可作为 Plan 403 的后续 Phase 或独立计划。

---

## 实施顺序（增量）

1. **需求 2：grid 重构**（示例源码层，零基建）→ 011 改 grid 布局 + 修 `%` bug + 去重复 handler。
2. **需求 1a：MCP 操纵验证**（零基建）→ `tests/desktop_mcp.py` 跑通按键→读值。
3. **需求 1b：`autoui_press_sequence`**（MCP 基建）→ 新工具 + 验证 `2+3=→5`。
4. **需求 1c / 引擎升级**（可选）→ 优先级/小数/括号。
5. **需求 3：多模式**（后置）→ shadcn-vue 参考版或 Auto 三模式。

---

## 验证

- grid 重构后：`auto run`（vue :3011）+ `auto run -r vm`（iced）双路径渲染正常，按键布局对齐。
- MCP：`cd examples/ui/011-calculator && auto run -r vm` → MCP :9247 → `autoui_press_sequence { keys:["2","+","3","="] }` → `display: "5"`。
- 回归：016-calendar（grid 已用）不受影响；其他示例 MCP 不受影响。

## 不做（明确后置）
- 运算符优先级 / 括号表达式（除非本轮做 1c）。
- Programmer 模式的十六进制/位运算（需求 3 后置）。
- vue 模式的 MCP（当前 MCP 仅 iced 嵌入；vue 走 playwright）。
