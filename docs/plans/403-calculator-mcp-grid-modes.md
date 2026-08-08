# Plan 403: 011-calculator 扩展 — MCP 操纵 + Grid 布局 + 多模式 UI

> **状态（2026-08-09）**: ✅ 代码全部完成。需求 1a/1b/1c/2/3 代码完成 + grid 按钮等宽对齐修复 + VM List 基建修复。**VM 运行时 MCP 表达式验证受阻于 VM 浮点运算缺陷**（详见下方"已知限制"）。vue 路径完整可用。
> **分支**: `plan403/011-calculator`（worktree `D:/autostack/auto-lang/.worktree/plan-403`）。
> **动机**: 011 是纯前端整数加减乘除玩具（325 行单文件、col/row 嵌套 + 22 个硬编码样式、无括号/小数、README 与代码脱节）。本计划把它扩展为可被 MCP 完整操纵、grid 布局、并支持多模式（Scientific/Programmer）的示例。
> **与 Plan 401 的关系**: 401 是"018-027 玩具→完整 App 升级"。011 的扩展性质不同——涉及 MCP 基建（新工具）+ grid 重构 + 多模式 UI 工程，是独立主题，故单独立项。401 §待办已加指引"→ 见 Plan 403"。

---

## 完成总结（2026-08-09）

### ✅ 已完成的需求
| 需求 | 状态 | 提交 | 说明 |
|------|------|------|------|
| 2: Grid 布局重构 | ✅ | `0bc72d9c` | col/row → grid，统一 Digit/Operator handler，修 `%` bug |
| 2: Grid 按钮等宽对齐修复 | ✅ | `78501aa8` | iced build_grid Fill 列包装 + vue w-full，按钮等宽对齐、间距可配置(~4px) |
| 1a: MCP 操纵验证 | ✅ | `eafdee25` | autoui_find/state 正常；带参 press 发现丢参数 |
| 1b: press_sequence + 带参 press 修复 | ✅ | `95bc6141` | autoui_press_sequence 工具 + extract_dyn_msg 参数编码 |
| 1c: 表达式引擎（优先级/括号/小数/幂） | ✅ | `4e72f4bd` | shunting-yard 双栈引擎，List<str> 数字栈 + apply_top/prec/fmt_num |
| 3: 多模式 UI | ✅ | `4e72f4bd` | Basic/Scientific/Programmer 三模式 + 模式切换 |

### ✅ VM List 基建修复（Plan 403 副产品，修复真实 bug）
调试表达式引擎运行时验证时，发现并修复了 VM 列表类型的多个缺陷：
1. **`shim_list_push` 丢字符串**：`ListData<Value>` 分支把 `is_string` 的元素存成 `Value::Int(字符串索引)` 而非 `Value::Str`（native.rs）。→ 改为解析真实字节存 `Value::Str`。
2. **`ListData<String>` 无 shim 支持**：`CREATE_LIST_STR` 创建 `ListData<String>`，但 `shim_list_push/pop/get/len` 只 downcast `ListData<i32>` 和 `ListData<Value>`，`List<str>` 的所有操作静默失败。→ 四个 shim 全部补 `ListData<String>` 分支。
3. **CALLSPEC `get`/`last` 不完整**：`type_name=="List"` 的 inline 分发，`get` 只处理 `ListData<i32>`，`last` 不存在。→ 补 `ListData<String>` + `ListData<Value>` 分支 + 新增 `last`。
4. **`str.to_float()` 缺失**：str 类型方法只有 `to_int`/`to_uint`，无 `to_float`。→ 补 `to_float`/`parse_float` handler。
5. **f64 `to_string()` 走 int 路径**：f64 值的 type_name 是 `<unknown_nv:...>`，`to_string` 把浮点位当 i32 解码（输出天文数字）。→ `to_string` handler 增加 `is_f64` 检查 + `fmt_f64` 辅助函数。

### ⚠️ 已知限制（VM 浮点运算缺陷）
- **VM 浮点运算损坏**：`var v float = 3.0 + 4.0` 在 VM 中结果损坏（`v` 被存为 nanboxed i32 而非 f64）。这是 VM 的 codegen/存储层问题，浮点值的栈编码可能被错误地 nanboxed 成 i32。
- **影响**：表达式引擎 `apply_top` 依赖 `nums.get(...).to_float()` 做浮点算术，受此缺陷影响，VM 模式下 `=` 求值无法正确计算（显示 Error）。vue 模式不受影响（vue codegen 不经 VM）。
- **验证**：整数运算在 VM 中正常（`3+4 → 7`），仅浮点损坏。
- **后续**：VM 浮点支持需独立计划修复（涉及 codegen 的 float 类型处理 + 栈编码）。这是 Plan 403 范围外的 VM 基建工作。

### 验证结果
- ✅ vue 路径：`auto run` → playwright 截图确认按钮等宽对齐、间距一致(~4px)、三模式切换正常。
- ✅ iced/VM 路径：窗口正常启动渲染；013-todo / 016-calendar 回归通过（List 基建修复未破坏现有功能）。
- ⚠️ VM 表达式求值：受 VM 浮点缺陷限制，`=` 显示 Error（整数算术本身正常）。
- ✅ MCP `autoui_press_sequence` 工具：按键序列操纵正常（`2+3=` 能触发完整 handler 流程）。

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
- **关键发现（2026-08-08 实测）**：当前 `autoui_action press` 触发带参 onclick（如 `.Digit(7)`）时**丢失参数**——handler 被调用（日志 `handler: .App.Digit`）但 `n` 拿不到值，display 不变（`mcp_server.rs:execute_action_vnode` 只取消息名，不传 onclick 的字面参数）。故 `press_sequence` 实现时需修复此点：从按钮的 onclick AST 提取字面参数（`Digit(7)` 的 `7`）一并传入 `call_handler`。这是需求 1b 的核心难点。

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

1. **需求 2：grid 重构** ✅ 已完成（`0bc72d9c`）→ grid 布局 + 修 `%` bug + 统一 Digit/Operator handler。vue/iced 双路径验证通过。
2. **需求 1a：MCP 操纵验证** ⚠️ 部分完成 → `autoui_find`/`autoui_state` 正常；但带参 press 丢参数（见 1b 发现），需先修才能跑通 `2+3=5`。
3. **需求 1b：`autoui_press_sequence`** ✅ 已完成（`95bc6141`）→ 新工具 + **修复带参 press 传参** + 验证 `2+3=5` / `5*3=15` / `9-4=5` / `1+2+3=6`（链式）。
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
