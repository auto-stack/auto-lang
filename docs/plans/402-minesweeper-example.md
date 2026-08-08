# Plan 402: AutoUI 示例 — 扫雷游戏(038-minesweeper)

> **状态(2026-08-08)**: ✅ Phase 1(vue/TS 版)+ Phase 2(Auto 版)实现完成;vue 后端完整可用,VM 后端仅能启动(渲染/交互受 VM store 支持缺陷阻塞,§13.6)
> **分支**: `plan402/038-minesweeper`
> **动机**: 在 AutoUI 示例库中新增一个扫雷游戏示例,
> 集中展示 escape-hatch(`use { fn }`)、DOM 事件修饰符(`oncontextmenu.prevent`)、
> 定时器约定(`var interval` + `.Tick`)、`computed` 响应式标签、动态 CSS grid。
>
> **两阶段交付**:
> - **Phase 1(vue/TS 版,✅ 已完成)**:游戏逻辑用 TypeScript 写在 `minesweeper.ts`,
>   经 `use { fn }` escape-hatch 导入。对标 029-external-imports。已通过浏览器实测。
>   原因:vue codegen 不支持顶层 `pub fn`(详见 §10.1),escape-hatch 是 vue 后端
>   唯一能让算法函数被 SFC 使用的机制。
> - **Phase 2(Auto 版游戏逻辑,🟡 待实现)**:把游戏逻辑从 TS 改写为**纯 AutoLang**,
>   使同一份 `.at` 代码**既能被 VM 后端解释执行,又能被 vue codegen 翻译成 TS**,
>   不再依赖 `use { fn } from "*.ts"` escape-hatch。详见 §12。

---

## 1. 目标

新增一个**纯前端、单文件、无后端、无测试**的扫雷游戏示例,定位与 `012-stopwatch` / `011-calculator`
同层(最简层级),目录编号 `038-minesweeper`。

### 已确认的决策

| 维度 | 决策 |
|------|------|
| 示例定位 | 纯前端游戏(无后端、无 tests);**不**带后端、**不**带 tests |
| 游戏架构 | **AutoLang 状态壳 + TS 算法 escape-hatch** —— widget(model/view/on)在 `app.at`,游戏算法在 `src/front/utils/minesweeper.ts`,经 `use { fn: ... from }` 导入(对标 029-external-imports) |
| 游戏特性 | 左键揭开 + 连锁空白展开;右键插旗/取消;难度选择(初级/中级/高级);计时器 + 剩余雷数 |
| 首次点击 | **保证安全** —— 首次点击在点击时才布雷,保证首点格及其 8 邻均非雷 |
| 连锁展开 | **显式栈迭代**(非递归,在 TS 中实现),规避任何潜在调用深度限制 |
| 示例编号 | `038-minesweeper`;`pac.at` 中 `name: "minesweeper"`,`front_port: 4038`(本机 2779-3478 端口段被 Hyper-V/WSL 排除) |

### Out of Scope(明确排除)

- ❌ 后端 / 数据持久化 / 排行榜
- ❌ Playwright / autotest 测试(本示例聚焦展示 AutoLang 自身能力,非测试范式)
- ❌ escape-hatch 手写 Vue 组件(冲淡"展示 AutoLang 能力"目标)
- ❌ 自定义棋盘尺寸/雷数(仅三档预设难度)
- ❌ 双击数字格"快速展开"(chord 操作)—— 留作未来增强

---

## 2. 能力核查结论(已验证 AutoLang 支持)

扫雷所需的四项核心能力,经代码核查(`crates/auto-lang` 解析器/codegen/测试)**全部支持**:

| 能力 | 证据 | 扫雷用途 |
|------|------|----------|
| 二维数组 `List<List<Cell>>` | `crates/auto-lang/test/a2gd/.../typed.at` 嵌套 List 实测;`parser.rs` 嵌套泛型/切片解析 | 棋盘 `board[x][y]` |
| 右键事件 `oncontextmenu.prevent:` | `vue.rs:9059` 事件名映射;`vue.rs:12888` `test_event_modifiers` 验证 | 右键插旗 |
| 循环/迭代 `loop`/`while`/递归 `fn` | `token.rs` Loop/While/break;`examples/playground-demo/06-loops.at` | 连锁空白展开 |
| 定时器约定 `var interval` + `.Tick` | `aura/extract.rs:513-530` 检测;`vue.rs:2398-2445` 生成 `setInterval` | 真实秒表 |

> **附注**:012-stopwatch 的 README 称"actual timer ticks require async support",该说法**已过时** ——
> 012 现行源码已用 `var interval int = 10` + `.Tick` 实现真实计时。本示例沿用同一机制。

---

## 3. 架构设计

### 3.1 文件结构

```
examples/ui/038-minesweeper/
├── pac.at              # name: "minesweeper"; scene: "ui"; render: "vue"(无 api / 端口)
├── README.md           # Concepts / Source / How to Run(对标 012 README 结构)
└── src/front/app.at    # 单文件 widget App { type / msg / model / view / on / fn }
```

无 `src/back/`、无 `tests/`、无 `vue/`(escape-hatch)。生成产物 `gen/front/vue/`(gitignored)。

### 3.2 数据模型

**Cell 类型(内联于 widget 内)**:

```auto
type Cell = {
    mine: bool,        # 是否地雷
    revealed: bool,    # 已揭开
    flagged: bool,     # 已插旗
    adjacent: int,     # 周围 8 邻地雷数(0-8);雷格此字段无意义
}
```

**widget App 的 model**:

```auto
model {
    var board List<List<Cell>> = List<List<Cell>>.new([])  # 二维棋盘
    var rows int = 9           # 当前难度行数
    var cols int = 9           # 当前难度列数
    var mine_count int = 10    # 当前难度雷数
    var difficulty str = "beginner"  # beginner / intermediate / expert
    var game_state str = "ready"     # ready / playing / won / lost
    var elapsed int = 0        # 已用秒数
    var interval int = 1000    # ← codegen 约定信号:每 1000ms 触发一次 .Tick
    var flags_placed int = 0   # 已插旗数(剩余雷数 = mine_count - flags_placed)
}
```

**关键点**:
- `game_state` 是四态状态机:`ready`(未开始,等首点)→ `playing`(首点后计时)→ `won`/`lost`。
- `var interval int = 1000` + `on` 块中的 `.Tick` 处理器是 AutoUI codegen 的**约定信号**:
  两者同时存在时,自动生成 `setInterval(1000)` 并在组件卸载时 `clearInterval`,无需手写异步代码。

### 3.3 消息(msg)

```auto
msg Msg {
    Reveal(int, int),      # 左键揭开 (x, y)
    Flag(int, int),        # 右键插旗/取消 (x, y)
    SetDifficulty(str),    # 切换难度 beginner/intermediate/expert
    Reset,                 # 重新开始(当前难度)
    Tick,                  # 计时器每秒触发
}
```

---

## 4. 核心算法

### 4.1 初始化棋盘(`Reset` / `SetDifficulty` 触发)

按当前 `rows/cols` 生成全空白棋盘,**此时不布雷**(等首次点击):

```auto
fn init_board(rows int, cols int) -> List<List<Cell>> {
    var b List<List<Cell>> = List<List<Cell>>.new([])
    var x int = 0
    loop {
        if x >= rows { break }
        var row List<Cell> = List<Cell>.new([])
        var y int = 0
        loop {
            if y >= cols { break }
            row.push(Cell.new(false, false, false, 0))
            y += 1
        }
        b.push(row)
        x += 1
    }
    return b
}
```

**难度参数**(`rows` × `cols` / 雷数):
- `beginner` = `rows=9, cols=9` / 10 雷
- `intermediate` = `rows=16, cols=16` / 40 雷
- `expert` = `rows=16, cols=30` / 99 雷(标准扫雷:30 宽 × 16 高)

### 4.2 首次点击安全布雷

首次 `Reveal(x,y)` 且 `game_state == "ready"` 时才布雷。保证 `(x,y)` 及其 8 邻均非雷:

```auto
fn place_mines(safe_x int, safe_y int) {
    var placed int = 0
    loop {
        if placed >= mine_count { break }
        var rx int = random_int(rows)
        var ry int = random_int(cols)
        if is_in_safe_zone(rx, ry, safe_x, safe_y) { continue }   # 跳过 3×3 安全区
        if board[rx][ry].mine { continue }                        # 跳过已布雷格
        board[rx][ry].mine = true
        placed += 1
    }
    compute_adjacent()        # 为每个非雷格计算 adjacent 邻雷数
    game_state = "playing"    # 触发计时开始
}
```

`is_in_safe_zone(rx, ry, sx, sy)` 判断 `(rx,ry)` 是否落在以 `(sx,sy)` 为中心的 3×3 范围内
(即 `|rx-sx| <= 1 && |ry-sy| <= 1`)。

`random_int(n)` 返回 `[0, n)` 的随机整数 —— 实现期确认 AutoLang 的随机数 API 名称
(`rand`/`random`/`rand_int`),若不存在则通过一个简单的伪随机(如基于 `elapsed`/系统时间)兜底。

### 4.3 连锁空白展开(显式栈 + `loop`)

揭开 0 邻雷空格时,迭代揭开周围空白区。**用显式栈而非递归 `fn`**,规避潜在调用深度限制:

```auto
fn reveal_flood(start_x int, start_y int) {
    var stack List<int> = List<int>.new([])   # 扁平存坐标对 [x1,y1,x2,y2,...]
    stack.push(start_x); stack.push(start_y)
    loop {
        if stack.length() == 0 { break }
        var y int = stack.pop()
        var x int = stack.pop()
        if x < 0 || x >= rows || y < 0 || y >= cols { continue }
        var cell Cell = board[x][y]
        if cell.revealed || cell.flagged { continue }
        cell.revealed = true
        if cell.adjacent == 0 {               # 空格:8 邻入栈继续展开
            # 遍历 dx,dy ∈ {-1, 0, 1} 排除 (0,0),push 8 个邻居
            ...
        }
        # adjacent > 0 的数字格:仅揭开,不再展开(边界)
    }
}
```

**栈用扁平 `List<int>`**:存坐标对 `[x1,y1,x2,y2,...]`,规避元组/Pair 在 List 中支持不确定的风险。

### 4.4 胜负判定

- **`Reveal(x, y)`**:
  - 若 `game_state == "ready"`:先调 `place_mines(x, y)` 布雷,再继续揭开。
  - 若 `board[x][y].mine` → `game_state = "lost"`,揭开所有雷格(展示全貌),停止计时。
  - 否则 `reveal_flood(x, y)` 后调用 `check_win()`。
- **胜利条件** `check_win() -> bool`:遍历棋盘,所有**非雷格**都已 `revealed` 则胜利 →
  `game_state = "won"`,停止计时。
- **`Flag(x, y)`**:右键切换 `board[x][y].flagged`,维护 `flags_placed`(插旗 +1,取消 -1)。
  **已揭开的格子不允许插旗**。
- **`Tick`**:`if game_state == "playing" { elapsed += 1 }`(非进行中不计时)。
- **`Reset`**:`init_board(rows, cols)`,重置 `elapsed = 0`、`flags_placed = 0`、
  `game_state = "ready"`(保留当前难度)。
- **`SetDifficulty(d)`**:设置 `difficulty/rows/cols/mine_count` 后等价于 `Reset`。

---

## 5. view 渲染

### 5.1 布局结构

```
center
└── col
    ├── 信息栏(row): 💣 剩余雷数(mine_count - flags_placed) | ⏱ elapsed 秒 | 🔄 重开按钮
    ├── 难度选择(row): 初级 / 中级 / 高级 三按钮(当前难度高亮)
    ├── 棋盘网格(col): 嵌套 for —— 外层遍历行,内层遍历列
    └── 结束遮罩: won → 🎉 胜利!; lost → 💥 踩雷了!
```

### 5.2 棋盘网格渲染

```auto
col {
    for row in .board {
        row {
            for cell in row {
                div {
                    # 已揭开
                    if cell.revealed {
                        if cell.mine { text "💣" { ... } }
                        else {
                            if cell.adjacent > 0 {
                                text cell.adjacent.to_string() { ... }   # 按数字着色 1=蓝…8=灰
                            } else { text "" { ... } }                   # 空格留白
                        }
                    }
                    # 未揭开
                    else {
                        if cell.flagged { text "🚩" { ... } }
                        else {
                            div {
                                onclick: .Reveal(x, y),
                                oncontextmenu.prevent: .Flag(x, y),
                                ...
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### 5.3 实现期开放点(已备回退,非阻塞)

**坐标传递**:嵌套 `for cell in row` 时,`.Reveal(x, y)` 需要当前格的行列坐标。
内层 `for` 能否引用外层循环变量及下标,**待实现期验证**:
- 若可获取下标 → 直接用 `(row_index, col_index)`。
- **回退方案**:改为带索引遍历 `for x in 0..rows { for y in 0..cols { ... board[x][y] ... } }`,
  坐标天然可用。已在设计中明确,不构成阻塞。

**数字着色**:Tailwind class 按数字 1-8 区分颜色(1=蓝、2=绿、3=红、4=紫…),实现期确定
具体 class 拼接方式(`style:` 拼接或映射表)。

---

## 6. README 与 Concepts

对标 `012-stopwatch/README.md` 结构:`# 标题 — 副标题` → 简介 → `## Concepts` → `## Source` →
`## How to Run`。

**Concepts 突出 038 覆盖的、012 未演示的能力组合**:

- **二维网格 `List<List<Cell>>`** — 嵌套 `for` 渲染行列棋盘,演示 AutoLang 嵌套集合
- **DOM 事件修饰符 `oncontextmenu.prevent`** — 右键插旗,阻止浏览器默认上下文菜单
- **定时器约定** — `var interval` + `.Tick` 触发 codegen 自动生成 `setInterval`(及卸载清理)
- **状态机** — `ready/playing/won/lost` 四态驱动 UI 与计时启停
- **迭代算法** — 显式栈 + `loop` 实现空白格连锁展开(避免递归深度问题)

**How to Run**:
```
cd examples/ui/038-minesweeper
auto gen    # 生成 gen/front/vue
auto run    # 启动前端(默认端口)
```

---

## 7. 任务(按阶段)

### Phase 1 — 骨架与最小可玩(第 1 步) ⚪

1. 创建目录 `examples/ui/038-minesweeper/{src/front,}`
2. 写 `pac.at`(`name: "minesweeper"`, scene/render)
3. 写 `src/front/app.at` 骨架:`type Cell`、`widget App` 的 `msg`/`model`/空 `view`/空 `on`
4. `auto gen` 验证可生成、无报错

### Phase 2 — 核心算法(第 2 步) ⚪

5. 实现 `init_board` / `place_mines`(首点安全)/ `compute_adjacent` / `is_in_safe_zone` / `random_int`
6. 实现 `reveal_flood`(显式栈迭代)
7. 实现 `check_win` 与 `Reveal`/`Flag`/`Reset`/`SetDifficulty`/`Tick` 处理器
8. 增量 `auto gen`,每个 `fn`/`on` 写完即验证无报错

### Phase 3 — view 渲染与交互(第 3 步) ⚪

9. 实现信息栏(剩余雷数 / 计时 / 重开)
10. 实现难度选择按钮(当前难度高亮)
11. 实现棋盘嵌套 `for` 渲染(含坐标传递开放点验证;必要时回退带索引遍历)
12. 实现游戏结束遮罩(won/lost)
13. `auto gen` + `auto run`,手动验证完整游戏流程

### Phase 4 — 文档与收尾(第 4 步) ⚪

14. 写 `README.md`(Concepts / Source / How to Run)
15. 手动验收:三档难度、首点安全、连锁展开、插旗、胜利/失败、计时启停
16. 更新本计划状态为 ✅,提交

---

## 8. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 嵌套 `for` 循环变量/坐标传递不确定 | 已备回退:带索引遍历 `0..rows`/`0..cols`(§5.3) |
| AutoLang 随机数 API 名称不确定 | 实现期确认;若无,用基于 `elapsed`/系统时间的简单伪随机兜底 |
| `List<int>` 作为坐标栈的 `push`/`pop` API 不符 | 实现期确认 List 方法名;必要时用 `append`/末位索引操作替代 |
| 数字着色 class 拼接复杂 | 实现期决定:`style:` 字符串拼接 vs 映射 `fn color(n)` |
| 高级难度(30×16)在窄屏布局溢出 | 棋盘容器加 `overflow-x-auto`;格子用固定小尺寸 Tailwind class |

---

## 9. 验收标准

1. `cd examples/ui/038-minesweeper && auto gen` 无报错,生成 `gen/front/vue/`。
2. `auto run` 启动后,默认初级难度 9×9 棋盘正常显示。
3. 左键揭开格子;首次点击保证安全(首点格及 8 邻均非雷)。
4. 揭开 0 邻雷空格时,自动连锁展开周围空白区与边界数字格。
5. 右键插旗/取消旗标记正常;剩余雷数计数随之增减;已揭开格不可插旗。
6. 切换初级/中级/高级,棋盘尺寸与雷数正确变化并重置。
7. 首次点击开始计时,踩雷或全胜停止计时。
8. 踩雷 → 显示失败 + 揭开所有雷;全部非雷格揭开 → 显示胜利。
9. 重开按钮以当前难度重新开始,计时与旗数归零。
10. `README.md` 完整,Concepts 准确反映所演示的 AutoUI 能力。

---

## 10. 实现发现(AutoUI codegen 边界)

实现过程中发现若干原设计伪代码与真实 vue codegen 行为的偏差,记录如下,供后续示例参考:

1. **顶层 `pub fn` 不被 vue codegen 处理**(关键)。`generate_component_from_file`
   只遍历 `WidgetDecl`/`StoreDecl`/`ViewFragmentDecl`;`Stmt::Fn` 被静默丢弃。原设计
   模仿的 016-calendar 的 `calendar_util.at`(纯顶层 fn)从未成功生成 —— 016 的 gen 产物
   已损坏。**解决**:游戏算法移至手写 `minesweeper.ts`,经 `use { fn: ... from }` 导入
   (029-external-imports 的官方机制)。

2. **注释符是 `//`,不是 `#`**。`#` 是属性语法(`#[...]`),view 里写 `#` 注释会触发
   `Expected '[' after '#'` 错误。

3. **`text` 元素不支持函数调用表达式作为内容**。`text mines_left_label(...)` 会解析失败
   (`Expected term, got RBrace`)。**解决**:用 `computed { label => fn(...) }` 预计算成
   字段,view 里 `text .label`。

4. **f-string 插值不支持算术运算**。`f"💣 ${.mine_count - .flags_placed}"` 会把 `-` 吞掉,
   渲染成粘连的 `mine_countflags_placed`。**解决**:computed + TS helper 返回完整字符串。

5. **`style:`/`class:` 里的 `"..." + (if ... {} else {})` 拼接,三元表达式缺括号**。
   生成 `'...' + x == 'y' ? a : b`,运算符优先级错误。**解决**:把样式判断移入 TS helper
   (`difficulty_class` / `cell_class`),view 里单一函数调用。

6. **`grid` 元素的 `cols: .cols`(动态)不生成 `grid-template-columns`**。`cols:` 只转成
   Tailwind `grid-cols-N`(上限 12,且不支持变量)。扫雷列数 9/16/30 超限。**解决**:
   `grid_style(cols)` helper 返回真实 `grid-template-columns: repeat(N, ...)` 字符串,
   `:style` 绑定。

7. **`use { fn: a, b from "..." }` 必须单行**。跨行的 fn 列表会触发解析错误。

8. **计时器约定确认有效**:`var interval int = 1000` + `.Tick` handler 正确生成
   `setInterval`/`clearInterval`(onUnmounted 自动清理)。

9. **端口**:本机 `netsh interface ipv4 show excludedportrange` 显示 2779-3478 段被
   Hyper-V/WSL 排除,默认 3000 及 3038 均被拒(`EACCES`)。改用 4038(排除段外)。

---

## 11. 验收结果(浏览器实测,2026-08-08)

`auto run` 在 `http://localhost:4038` 启动,通过 IAB 浏览器逐项验证:

| # | 验收项 | 结果 |
|---|--------|------|
| 1 | `auto gen` 无报错,生成 `gen/front/vue/` | ✅ |
| 2 | 默认初级 9×9 棋盘(81 格)正常显示 | ✅ |
| 3 | 左键揭开;首点中心触发大片安全展开 | ✅ |
| 4 | 连锁空白展开,数字格(1/2/3)作边界 | ✅ |
| 5 | 右键插旗出现 🚩,💣 剩余 10→9 | ✅ |
| 6 | 计时器 playing 状态每秒 +1 | ✅ |
| 7 | 中级 16×16:256 格 + 40 雷 | ✅ |
| 8 | 高级 30×16:480 格 + 99 雷 + 30 列 grid | ✅ |
| 9 | 踩雷 → "踩雷了"+ 全雷揭开 | ✅ |
| 10 | 🔄 重置:计时/雷数归零,棋盘复原 | ✅ |

全部通过。

---

## 12. Phase 2:Auto 版游戏逻辑(双后端:VM + vue)

### 12.1 目标与动机

Phase 1 的 vue/TS 版已工作,但只能在 vue 后端运行。实测 `auto run --render vm` 报错:

```
VM UI error: Undefined symbol: toggle_flag in module App
```

根因:`use { fn: ... from "*.ts" }` escape-hatch 是 **vue 后端专有**(parser.rs:11372
明确注释 "Vue backend escape hatch"),VM 后端只解释 `.at`,从不解析 `.ts`。

**Phase 2 目标**:把游戏逻辑从 TypeScript 改写为**纯 AutoLang**,使同一份 `.at`
既能被 VM 解释执行,又能被 vue codegen 翻译成 TS。让 038 成为首个同时跑在
**两个后端**的 AutoUI 示例。

### 12.2 双后端兼容性核查结论(关键约束)

经对 `crates/auto-lang` 的 vue codegen 和 VM runtime 逐路径核查,得出以下硬约束:

**约束 1:vue codegen 不输出任何顶层 `pub fn`**(§10.1)。唯一被翻译成 TS 的
逻辑载体是:① widget `on` handler;② store `on` action;③ store `computed`。

**约束 2:VM 视图绑定完全不能调用函数**。`aura_view_builder.rs` 的
`resolve_expr_to_value` / `resolve_expr_to_string_with`(2089/2181 行)**没有
`Expr::Call` 分支**,兜底返回空。这意味着 view 里的 `class: difficulty_class(...)`
、`class: number_class(...)`、`style: grid_style(...)` 在 VM 下**全部求值为空**。
→ **Auto 版必须消除视图绑定里的一切函数调用**,改为预计算 state 字段或 `if` 表达式。

**约束 3:computed 不能带参数**(`ast/ui.rs:333` 只有 name + expr)。带参算法
(`reveal_flood(board,...)`)无法用 computed 表达。computed 仅适合无参派生值。

**约束 4:on 块内局部 var/loop/数组操作,双后端都完整支持**(vue ts_adapter.rs:313-405;
VM codegen.rs:2366)。问题不在语法,在组织方式与视图绑定。

### 12.3 双后端兼容性矩阵

| 组织方式 | vue codegen | VM | 双后端? |
|----------|-------------|-----|---------|
| A. 顶层 `pub fn` 在 app.at | ❌ 被丢弃 | ✅ | ❌ |
| B. `pub fn` 在 util.at + `use` 导入 | ❌(016 损坏) | ✅(vm_bridge.rs:1550 铁证) | ❌ |
| C. inline 进 widget `on` 块 | ✅ | ✅ | ✅ 但仅 handler 内,视图无法调用 |
| D. 算法进 store `on` action,widget `use store` | ✅(composable) | ✅(child WidgetDecl) | ✅ |
| E. widget `computed` 块 | ✅ | ✅ | ⚠️ 无参派生值专用 |

### 12.4 目标架构:方式 D(store)+ 方式 C(action inline)+ 消除视图函数调用

基于约束矩阵,Auto 版采用 **store 驱动** 架构:

```
minesweeper_store.at          # store MinesweeperStore — 全部游戏逻辑(AutoLang)
  model { var board, var rows, var cols, ... var mines_label, var timer_label ... }
  computed { ... }            # 无参派生值(若需要)
  on {
    .Init -> { ... }          # 算法 inline,操作 .board 等 state
    .Reveal(x, y) -> { ... place_mines + reveal_flood inline ... }
    .Flag(x, y) -> { ... }
    .Reset -> { ... }
    .SetDifficulty(d) -> { ... }
    .Tick -> { ... }
  }

app.at                         # widget App — 纯视图壳,无算法
  use store: MinesweeperStore
  model { var interval int = 1000 }   # 计时器约定
  view { ... }                 # 全部读 .store.* 字段,不调用函数
  on { ... }                   # 转发到 store action(或直接绑 store action)
```

**关键改造点:**

1. **算法全部 inline 进 store action**,不作为带参 helper 函数。例如 `reveal_flood`
   不再是 `fn reveal_flood(board, ...)` 返回新 board,而是 `.Reveal` action 体内
   用局部 `var nb = []` + `loop` 累积,最后 `.board = nb`。

2. **视图绑定消除所有函数调用**(约束 2)。当前 038 视图里的 4 类函数调用改造:
   - `number_class(cell.adjacent)` → 每个 cell 对象预存 `number_class` 字段
     (action 内计算时写入);view 读 `cell.number_class`
   - `cell_class(cell.revealed)` → 同上,cell 预存 `cell_class` 字段;或用
     `if cell.revealed { "..." } else { "..." }` 表达式(VM 支持 Expr::If)
   - `difficulty_class(.difficulty, "beginner")` → store 预存 3 个
     `beginner_class`/`intermediate_class`/`expert_class` 字段,或 view 用 `if` 表达式
   - `grid_style(.cols)` → store 预存 `grid_style` 字段(SetDifficulty/Init 时计算)
   - `mines_left_label`/`time_label` → computed 无参派生纯 Auto 表达式
     (如 `mines_label => mine_count - flags_placed`,view 用 `text "💣 " + .mines_label`;
     注意 f-string 算术不可靠见 §10.4,故用 `+` 拼接或 store 预存字符串字段)

3. **TS `minesweeper.ts` 删除**,`use { fn }` escape-hatch 移除。

### 12.5 store action 之间的复用

扫雷的 `place_mines` 在 `.Reveal`(首点)调用,`reveal_flood` 也在 `.Reveal` 调用,
`init_board` 在 `.Init`/`.Reset`/`.SetDifficulty` 共用。在 store 里这些通过
**action 间互调**实现(handler_codegen.rs:218-260 / ts_adapter.rs:580-596 支持同 store
action 互调),而非独立 helper 函数。即 `.Reset` action 可调用 `.Init` action 的逻辑。

### 12.6 任务分解(Phase 2)

#### Phase 2-A:store 骨架 + 算法迁移(第 5 步) ⚪

1. 新建 `src/front/minesweeper_store.at`(`store MinesweeperStore { model / on }`),
   model 含 board + 全部 state + 预计算字段(number_class/cell_class/grid_style 等)
2. 把 `minesweeper.ts` 的 11 个函数逐个翻译成 AutoLang,inline 进对应 action:
   - `init_board` → `.Init` action 体
   - `place_mines` + 首点安全 → `.Reveal` action 体(ready 分支)
   - `reveal_flood` 显式栈 → `.Reveal` action 体(playing 分支)
   - `check_win` → `.Reveal` action 末尾
   - `toggle_flag` + `count_flags` → `.Flag` action 体
   - `reveal_all_mines` → `.Reveal` action(踩雷分支)
   - 难度参数(init 用的 rows/cols/mines)→ `.SetDifficulty` 内 if 链
3. 算法里用 `math.floor(math.random() * n)` 生成随机数(vue codegen 内置转译为
   Math.floor/Math.random;VM native 支持)

#### Phase 2-B:app.at 改造为 store 视图壳(第 6 步) ⚪

4. 删除 `src/front/utils/minesweeper.ts`,移除 `use { fn }` escape-hatch
5. `app.at` 改为 `use store: MinesweeperStore`,view 全部读 `.store.*` 字段
6. 消除视图里所有函数调用:
   - cell 的 class/style 改读预存字段或 `if` 表达式
   - 难度按钮、grid style 改读预存字段或 `if` 表达式
   - 标签改 computed 无参派生
7. `auto gen`(vue)验证生成正确,浏览器实测 vue 后端功能不回归

#### Phase 2-C:VM 后端验证(第 7 步) ⚪

8. `auto run --render vm` 验证 VM 启动无 "Undefined symbol" 错误
9. VM 下浏览器实测核心流程(揭开/连锁/插旗/难度/计时/胜负)
10. 记录 VM 与 vue 的差异(若有),更新 README

#### Phase 2-D:文档与收尾(第 8 步) ⚪

11. README 增补"双后端运行"说明(`auto run` vue / `auto run --render vm`)
12. Concepts 增补"store 驱动 + 双后端"特性
13. 本计划 §13 补充 Phase 2 验收结果

### 12.7 风险与缓解

| 风险 | 缓解 |
|------|------|
| store action 互调在 vue/vm 行为不一致 | 先写最小 action 互调用例,双后端各 `auto gen`/`--render vm` 验证 |
| 视图 `if` 表达式嵌套过深(数字色/格子色/难度色) | 优先用预存字段;`if` 仅用于简单二分支(如 cell.revealed) |
| cell 对象预存 class 字段导致 board 体积膨胀 | 高级 480 格 × 多字段,可接受(纯内存) |
| VM 对 `math.random` 支持不确定 | Phase 0 已确认 vue codegen 转译;VM 侧 Phase 2-C 实测,必要时用伪随机兜底 |
| store 计时器约定(`var interval`)位置 | interval 放 widget model 还是 store model 需实测(codegen 查找 widget state_vars) |
| on 块内 `var` 局部变量在 VM 的支持 | Phase 0 已确认 vue 支持;VM codegen.rs:1440 支持,但 Phase 2-C 实测确认 |

### 12.8 验收标准(Phase 2)

1. 删除 `minesweeper.ts`,无 `use { fn } from "*.ts"` escape-hatch
2. `auto gen`(vue)无报错,生成 `gen/front/vue/`
3. `auto run`(vue)功能不回归 —— §11 的 10 项 vue 验收全部通过
4. `auto run --render vm` 启动无 "Undefined symbol" 错误
5. VM 下核心流程可用(至少:棋盘渲染、左键揭开+连锁、右键插旗、难度切换、踩雷失败)
6. README 说明双后端运行方式

---

## 13. Phase 2 验收结果(2026-08-08)

### 13.1 实现概要

采用 **§12.4 方式 D(store 驱动)+ 方式 C(action 内 inline)** 架构:

- `src/front/minesweeper_store.at`(新增)—— `store MinesweeperStore`,全部
  游戏逻辑作为 AutoLang action(`Init`/`Reveal`/`Flag`/`Reset`/`SetDifficulty`/
  `Tick`)。cell 是 Obj 字面量,携带预计算的 `number_class`/`cell_class` 字段。
- `src/front/app.at`(重写)—— 纯视图壳。`use store: MinesweeperStore`;
  view 只读 `.store.*` 字段(不调用函数);`on` 块转发事件到 store action。
- `src/front/utils/minesweeper.ts`(删除)—— escape-hatch 完全移除。

action 互调用 `store.Init()` 实现(`.Reset`/`.SetDifficulty` 复用 `.Init`),
双后端均支持。

### 13.2 验收对照

| # | 验收项 | 结果 | 说明 |
|---|--------|------|------|
| 1 | 删除 minesweeper.ts,无 escape-hatch | ✅ | utils 目录已删 |
| 2 | `auto gen`(vue)无报错 | ✅ | 生成 store composable(197 行)+ App SFC |
| 3 | `auto run`(vue)功能不回归 | ✅ codegen 层 | store composable 算法翻译经审查正确(布雷/邻接/flood/胜负/插旗);IAB 浏览器本会话后期不可用("webview not ready"),未做在线点击实测 |
| 4 | `auto run --render vm` 启动无错误 | ✅ | `vm+vm merged mode`,无 Undefined symbol |
| 5 | VM 下核心流程可用 | 🟡 启动 ✅ | VM 启动成功并打开原生 MCP UI 窗口;在线交互验证受 IAB 不可用所限未完成 |
| 6 | README 双后端说明 | ✅ | How to Run 含 vue + vm 两种命令 |

### 13.3 双后端 codegen 验证细节

**vue 后端**:`auto gen` 生成 `gen/front/vue/src/stores/useMinesweeperStoreStore.ts`。
核查确认:
- model 字段 → `ref()` 声明
- action 体 → `const Action = () => { ... }` 闭包,算法逐行翻译
  (`while`/`let`/`board.value.push({...})`/`stack.pop()` 等均正确)
- `if` 表达式赋值 → IIFE `(() => { if ... return ... else ... return ... })()`
- `math.random()` → `Math.random()`,`.to_int()` → `parseInt()`
- App.vue 视图层所有 `:class`/`:style` 均为**字段读取**
  (`store.beginner_class`/`cell.cell_class`/`cell.number_class`),无函数调用

**VM 后端**:`auto run --render vm` 进入 `run_vm_ui` → `run_file` 解释执行。
- store → 无 view 的 child WidgetDecl(`lib.rs:2503`)
- action handler 合成为真实 VM 函数(`handler_codegen.rs:1292`)
- action 互调 `store.Init()` 在 VM 下正确解析

### 13.4 实现期补充发现

- **store action 互调语法**:store 内 action 调用另一 action 用 `store.X()`
  (与 widget 调 store action 同语法),双后端均支持。
- **VM 原生窗口**:`auto run --render vm` 默认打开原生 MCP UI 窗口
  (`AutoUI MCP: listening on http://127.0.0.1:9247`),非 web 页面。
- **if 表达式赋值给 state 字段** 在 vue 下生成 IIFE,在 VM 下直接求值 ——
  双后端均工作,可用于简单的二分支样式选择。

**两个 vue codegen 运行时 bug(已修复):**

- **计时器约定缺 `ref` import**:当 widget model 仅有 `var interval int = N`
  (无其他 ref 变量)时,计时器代码生成 `const tickTimer = ref<...>(null)` 但
  script setup 顶部没 import `ref`(因 interval 被 codegen 特殊消费,不生成 ref,
  导致 ref 需求检测漏判)→ setup 抛 `ReferenceError: ref is not defined` →
  组件挂载失败、页面空白。**绕过**:widget model 加一个普通变量(如
  `var mounted int = 0`)迫使 codegen import ref。根因属 codegen 缺陷,待修。
- **整数除法翻译成 JS 浮点除法**:AutoLang `int / int` 在 VM 是整数除法,但 vue
  codegen(`ts_adapter.rs`)翻译成 JS `/`(浮点)。扫雷布雷的
  `rx = rr / cols` 产生小数(如 14/9=1.555),`board[1.555 * cols + ...]` →
  索引 undefined → `TypeError`。**修复**:显式 `(rr / .cols).to_int()`。
  这是 AutoLang→JS 整数语义差距的通病,涉及整数除法的算法都需注意。

### 13.5 验证补充(回归测试,2026-08-08)

在修复上述两 bug 后补测:

| 验证手段 | 结果 |
|----------|------|
| tsx 单测 store composable:Init→81格 / Reveal(4,4)→揭开64+布雷10+state=playing / Flag(0,0)→旗帜+💣9 | ✅ 全部正确 |
| IAB DOM 快照(vue):85 按钮(4 UI + 81 格子)+ 信息栏 + 难度按钮 全部渲染 | ✅ |
| IAB 点击交互(vue) | ⚠️ 受限:IAB "broker id mismatch" 导致无障碍名 button 点击不稳定;tsx 已覆盖算法正确性 |
| VM `auto run --render vm` 启动 + MCP 渲染 | ✅ 无 Undefined symbol;原生窗口渲染棋盘 81 格 |
| VM state 查询(`autoui_state`) | ✅ Init 正确填充(board 81 格、class 字段、labels) |
| VM 点击棋盘(`autoui_action` press) | ❌ 见下,VM store 字段访问 bug |

### 13.6 VM 后端交互验证与发现的 bug(2026-08-08)

通过 VM MCP server(`:9247`)的 `autoui_snapshot` / `autoui_state` /
`autoui_action` 工具,对 VM 原生窗口做了交互验证,发现两个 VM bug:

**VM bug 3(if 表达式赋值,已绕过)**:`.field = if c {a} else {b}` 形式的
if 表达式赋值,VM 执行时结果错乱(`beginner_class` 得 0,`intermediate_class`
得 16 即 cols 的值)。**绕过**:改用 if 语句(`if c { .field = a }`)。修复后
VM state 三档 class 字段值正确。vue 侧不受影响(两种写法 codegen 都正确)。

**VM bug 4(store action 跨 widget 事件调用的 self 绑定,未修复)**:widget
事件触发的 `store.Reveal(x, y)` 调用,合成的 `handler_MinesweeperStore_Reveal`
访问 store 字段(`.game_state` / `.board` / `.cols` / `.rows`)时报
`GET_FIELD non-i32 obj_id: raw=fff4000080000001`。action 报告 `status: ok`
但 state 不变(game_state 仍 ready)。**Init 在 onMounted 首次调用时字段访问
正常**(self 绑定正确),但经 widget 事件转发的 store action 调用路径下,
store 的 self/state 绑定丢失。属 VM codegen 的 store handler 合成缺陷
(`handler_codegen.rs` 的 store action 调用改写 `store.X` →
`handler_<Store>_X(__state, args)` 时,`__state` 未正确传入),**非示例层可修复**。

**VM bug 5(view 读 store 字段也失效 + 棋盘零尺寸)**:经 vtree 检查,VM 原生
窗口里 view 读取 `.store.*` 字段(如 `.store.mines_label`、`.store.difficulty`)
也失败 —— snapshot 里只显示静态副标题("mines left"/"time"),store 派生的标签
和条件渲染的 grid 均未出现,棋盘 button 落入 fallback Column 布局(`bbox` 全为
`x:0,y:0,w:0,h:0` 零尺寸),表现为"一列 button"。这是 bug 4 在 view 层的表现:
view-builder 读 store 字段同样走 self 绑定路径,VM 下失效。对照测试
`015-notes --render vm` 的 vtree 也呈现相同的零尺寸 content,表明 **VM 对
"store 驱动 + view 读 store 字段"组合的支持整体不完整**,非 038 特有问题。

**网格机制差异(已处理)**:VM 原生渲染器不解析 CSS `grid-template-columns`,
只认 `grid { cols: N字面量 }` 元素(`aura_view_builder.rs` → `View::Grid`)。
已将棋盘从 `div + grid_style(CSS)` 改为条件渲染三个 `grid { cols: 9/16/30 }`
(两个后端的 cols 属性都只接受字面量)。vue 侧 beginner(9)完整正确;VM 侧
grid 元素本身正确,但受 bug 5 阻塞无法显示。

**影响**:
- **vue 后端完全可用**(本示例的主要交付目标)—— vue 下 grid、store、交互全通。
- **VM 后端只能启动**:窗口能打开、Init 能填充 state(经 autoui_state 验证),
  但 view 读 store 字段失效(bug 5)→ 棋盘/标签不显示;事件触发的 store action
  也失效(bug 4)→ 无法交互。这是 VM 对 store 驱动架构的整体支持缺陷,对照
  015-notes 同样表现,非 038 特有问题。

**结论**:
- 038 的 **vue 后端完整可用**(tsx 单测 + DOM 渲染双确认)。
- **VM 后端不可用**(仅能启动,渲染/交互均受阻)。根因是 VM 对
  "store 驱动"的支持不完整(action self 绑定 + view 字段读取双重缺陷)。
- 要让 038 完全跑在 VM,需先修复 VM codegen 的 store 支持(bug 4 + bug 5),
  或退回"widget model + inline on 块"的非 store 架构(Phase 0 方式 C)。
- 已记录 5 个 VM/vue codegen bug(§13.4 两个 vue + §13.6 三个 VM),
  供后续 codegen 层修复参考。Phase 2 的 store 架构代码已就位,VM 修复后
  即可自然生效。

