# Plan 402: AutoUI 示例 — 扫雷游戏(038-minesweeper)

> **状态(2026-08-08)**: ⚪ 设计完成,待实现
> **分支**: `plan402/038-minesweeper`(待创建)
> **动机**: 在 AutoUI 示例库中新增一个扫雷游戏示例(对标 012-stopwatch 的精简单文件范式),
> 用于集中展示 AutoLang 当前已具备但尚未被现有示例覆盖的能力组合:
> 二维数组(`List<List<Cell>>`)、DOM 事件修饰符(`oncontextmenu.prevent`)、
> 定时器约定(`var interval` + `.Tick`)、迭代算法(`loop` + 显式栈)。

---

## 1. 目标

新增一个**纯前端、单文件、无后端、无测试**的扫雷游戏示例,定位与 `012-stopwatch` / `011-calculator`
同层(最简层级),目录编号 `038-minesweeper`。

### 已确认的决策

| 维度 | 决策 |
|------|------|
| 示例定位 | 纯前端精简游戏(对标 012-stopwatch);**不**带后端、**不**带 tests、**不**用 escape-hatch |
| 游戏架构 | 方案 A —— 游戏逻辑全部用 AutoLang 写在 `widget App` 的 `on`/`fn` 块内,单文件 |
| 游戏特性 | 左键揭开 + 连锁空白展开;右键插旗/取消;难度选择(初级/中级/高级);计时器 + 剩余雷数 |
| 首次点击 | **保证安全** —— 首次点击在点击时才布雷,保证首点格及其 8 邻均非雷 |
| 连锁展开 | **显式栈 + `loop` 迭代**(非递归),规避任何潜在调用深度限制 |
| 示例编号 | `038-minesweeper`;`pac.at` 中 `name: "minesweeper"` |

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
