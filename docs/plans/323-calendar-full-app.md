# 016-calendar → 完整月历应用（Plan 323）

> **For Claude:** 在专用 worktree `plan-323/calendar-full-app` 里实现。改 stdlib/VM 后跑 `cargo build -p auto` 再测；改 `.at` 前先调 `/auto-lang-creator`。每阶段 `examples/ui/016-calendar` 在 **vm 和 rust 两模式**下目视一致（Plan 319 已统一渲染，两者必须同形）。回归 `cargo test -p auto-lang --lib`。

## Goal

把 016-calendar 从"40 个写死的 grid-item + 切月写死字符串"的静态壳子，改造成**数据驱动、可导航、可看事件/节假日**的完整月历应用，并顺带补齐 DateTime stdlib 的日期算术能力。月视图为主。

## 已确认的决策（来自调研讨论）

1. **范围**：完整月历——动态网格 + 上/下月/今天导航 + 选中日 + 今天高亮 + 事件指示点 + 节假日标记 + 事件侧栏。**不含**周/日多视图、拖拽、农历。
2. **后端**：只做**节假日数据 + 事件持久化**；日期网格纯前端算。复用 015-notes 的 `#[api]` + 内存 DB 模式。
3. **日期算术缺口（已定：前端算 + 纯 Auto）**：日期网格**前端算**。把 99% 写成**纯 Auto**（闰年表、Zeller 算星期、整数算术），自动 transpile 到所有后端、**零 transpiler 改动**；唯一碰系统调用的「今天日期」单独抽成 `DateTime.today_str()`，做三处 per-target 映射（`#[vm]` + a2r chrono + a2vue JS 垫片）。详见 Phase 1。
4. **样式复用**：本轮靠 **DayCell widget + for 循环**消除重复（40 cell → 1 个 DayCell）；通用样式预设/别名机制**另立计划**，不塞进本计划。

## 架构

### 前端 widget 拆分（仿 015-notes 多 widget 模式）

```
src/front/
  app.at            # Calendar（根 widget）：持有 state，编排子 widget
  month_header.at   # MonthHeader(year, month)：标题 + ◀ 今天 ▶ 导航
  weekday_row.at    # WeekdayRow：Su..Sa 表头行（7 列，须与网格列对齐）
  day_cell.at       # DayCell(cell)：单个日期格（日期号 + 今天/选中/事件点/节假日高亮）
  event_sidebar.at  # EventSidebar(day, events)：选中日的事件列表 + 新建事件
  calendar_util.at  # 纯函数：build_month_grid()、月份/今天判定等
```

### 数据模型

**前端 Calendar state（app.at `model`）：**
```auto
model {
    var year int            // 当前显示年份
    var month int           // 当前显示月份 1-12
    var selected_date str   // "2026-04-15"，选中的日期
    var today str           // "2026-06-17"，启动时算一次
    var days = []           // List<DayCellData>，42 格（6 周）
    var events = []         // 本月事件（后端拉取）
    var holidays = []       // 本月节假日（后端拉取）
}
```

**单格数据（calendar_util.at，前端/后端共享类型放 back/api.at）：**
```auto
pub type DayCellData = {
    day: int            // 1-31；0 表示前后月填充格
    date: str           // "2026-04-15"，用于事件/节假日查找
    is_today: bool
    is_selected: bool
    is_other_month: bool
    event_count: int
    is_holiday: bool
    holiday_name: str
}
```

### 数据流

1. `.Init` → 算 `today`；`month/year` 取今天所在月；拉 `events`/`holidays`；调 `build_month_grid()` 填 `days`。
2. `.PrevMonth`/`.NextMonth`/`.Today` → 用 `add_months` 改 `year/month` → 重拉本月 `events`/`holidays` → 重建 `days`。
3. `.SelectDay(date)` → 改 `selected_date` → 重建 `days`（刷新 selected 标记）→ EventSidebar 显示该日事件。
4. view 里 `grid { for cell in .days { DayCell(cell: cell) } cols: 7 }`。

## 任务（按阶段，B→C 顺序）

> **注意**：Phase 1（stdlib）是 Phase 2 的硬前置。Phase 2 先把根 widget 跑通（单 widget），Phase 3 再拆子 widget，避免一次性改太多难调试。

---

### Phase 1 — 日期算术：纯 Auto + today_str 的 per-target 映射（前置）

**核心策略（已定 Option A）**：日期网格**前端算**，但 99% 写成**纯 Auto**（纯算术 + 查表）→ a2r/a2vue **零改动**自动 transpile。唯一碰系统调用的「今天日期」单独抽成 `DateTime.today_str()`，做三处 per-target 映射。

**关键避坑**：现有 `DateTime` 对象的字段访问器 `.year()/.month()/.day()`、`.weekday()` 全是 `#[vm]`，**不 transpile**。所以前端拿到 today 字符串后必须**立刻 split 出 year/month/day 整数**，之后所有算术只在整数上做，绝不碰 `#[vm]` 的 DateTime 对象字段。新增的日期函数全部是**接收整数的纯 Auto 自由函数**。

**Files:**
- Modify: [stdlib/auto/datetime.at](stdlib/auto/datetime.at) — 只加 1 个 `#[vm] static fn today_str() str`
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs` — `today_str` 的 `#[rust_fn]` 实现
- Modify: `crates/auto-lang/src/trans/rust.rs` — a2r 给 `DateTime.today_str()` 加 stdlib 映射 → `a2r_std::datetime::today_str()`（chrono `Local`）
- Modify: `crates/auto-lang/src/trans/ts_expr.rs` + `ts_runtime.rs` — a2vue/a2ts 把 `DateTime.today_str()` 特判 → runtime 垫片（仿 `print`/`range`）
- Create (Phase 2 落地，类型先定): `examples/ui/016-calendar/src/front/calendar_util.at` — 纯 Auto 日期函数

**1a. `#[vm] static fn today_str() str`（唯一 per-target 依赖）**
```auto
// datetime.at —— 只加这一个 #[vm]
#[vm]
static fn today_str() str   // 返回本地时区 "2026-06-17"
```
- **VM**：stdlib.rs `#[rust_fn]`，`chrono::Local::now().format("%Y-%m-%d")`。
- **a2r**：rust.rs stdlib 映射（见 `emit_a2r_stdlib` ~1029、映射表 ~7499）加 `DateTime.today_str` → `a2r_std::datetime::today_str()`；helper 用 chrono `Local`。
- **a2vue/a2ts**：ts_expr.rs 特判 `DateTime.today_str()` 调用 → emit `today_str()`（仿 ts_expr.rs:30 的 `print` 特判）；ts_runtime.rs 垫片加：
  ```ts
  export function today_str(): string {
      const d = new Date();
      const m = String(d.getMonth() + 1).padStart(2, "0");
      const day = String(d.getDate()).padStart(2, "0");
      return `${d.getFullYear()}-${m}-${day}`;   // 本地时区，非 UTC
  }
  ```

**1b. 纯 Auto 日期算术（calendar_util.at，Phase 2 落地；Phase 1 先验证「纯 Auto 能 transpile 到 a2r」）**
```auto
// calendar_util.at —— 全部纯 Auto，零 #[vm]，零 transpiler 改动，接收整数
pub fn is_leap(year int) bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
pub fn days_in_month(year int, month int) int {
    let dim = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    if month == 2 && is_leap(year) { return 29 }
    return dim[month - 1]
}
pub fn weekday_of(year int, month int, day int) int {
    // Sakamoto 算法：纯整数算术，返回 0=Sunday..6=Saturday（日历 Su 开头，无需换算）
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]
    var y = year
    if month < 3 { y = y - 1 }
    return (y + y / 4 - y / 100 + y / 400 + t[month - 1] + day) % 7
}
pub fn add_months_ym(year int, month int, n int) (int, int) {
    // 返回新 (year, month)；不夹紧日（调用方用 days_in_month 夹紧）
    let total = year * 12 + (month - 1) + n
    return (total / 12, total % 12 + 1)
}
pub fn month_name(month int) str { /* 查 12 名表 */ }
```

> **为何不复刻 `DateTime.weekday()` 的 0=Mon 语义**：`weekday_of` 是全新纯 Auto 函数，直接返回 **0=Sunday**（日历 Su 开头），省掉 `(weekday+1)%7` 换算。

**任务：**
1. datetime.at 加 `today_str` 声明；stdlib.rs 加 `#[rust_fn]` 实现。
2. a2r：rust.rs stdlib 映射 + `emit_a2r_stdlib` 里 `a2r_std::datetime::today_str`。
3. a2vue/a2ts：ts_expr.rs 特判 + ts_runtime.rs 垫片。
4. 加最小 a2r 测试验证纯 Auto `is_leap`/`days_in_month` 能 transpile 并跑（这是「纯 Auto 免改 transpiler」假设的回归保护）。

**验证：**
- `cargo build -p auto` 干净。
- VM：`DateTime.today_str()` 返回今天。
- a2r 测试：纯 Auto `days_in_month(2024,2)==29`、`days_in_month(2026,2)==28` transpile+跑过。
- a2vue：生成代码里 `today_str()` 来自 runtime 垫片（目视确认）。
- `weekday_of(2026,6,17)==3`（2026-06-17 周三，Su=0）。

---

### Phase 2 前置 — VM handler 走真 Codegen（解锁 handler 计算）

> **状态**：Plan 323 Phase 2 实现中发现 `.days = build_month_grid(...)` 网格始终为空。深挖根因（MCP snapshot + 源码）确认是 **VM widget handler 不具备计算能力**，与 `.at` 代码无关。本前置计划已确认并批准，作为 Phase 2 的硬前置。**在 plan-323 worktree 里实现。**

**根因三层：**
1. imported-module 的函数（`build_month_grid` 等）**根本没进 VM** —— `lib.rs:1708-1744 run_file_dynamic_ui` 对每个 `use module` 只抽 `WidgetDecl`，函数定义被丢弃，`exports_by_name` 里没这些符号。
2. handler 由 bespoke 迷你编译器编译（`vm_bridge.rs:893 compile_handler_stmts`），只发 Expr/Store/If；for/数组/对象/`CALL` 全是 `Err("unsupported")`（`:1004`），被 `new():157` 的 `if let Ok` **静默吞掉**。
3. Init 走第二条路 —— `call_handler_ast`（`:468`）的 AST 解释器 `exec_stmt`/`eval_expr`，对数组/循环/调用同样无能为力（`_ => Ok(())` / `_ => Nil`）。015-notes 能用只因 `try_api_call`（`:670`）写死 5 个 HTTP shim。

**关键发现**：真 Codegen（`compile.rs:1479 compile_module_to_bytecode`）**已完整实现** for-in / 数组字面量 / 对象字面量 / `.push()` / `CALL`。迷你编译器里重造这些是纯重复，且 `CALL` 要绝对 flash 地址（`engine.rs:3981`），迷你编译器无重定位表 = 要再手搓 Linker。

**架构决策（Hybrid，用户已确认）**：handler 体 + imported 函数都走真 Codegen → 一个 linked `CompiledPackage` → 真 VM `call_fn_by_name`。保留 `VmBridge` 作为 state 容器 + dispatch 门面。唯一真正新代码：(a) 状态绑定 shim（`.field`/state 标识符 → 对 state 堆对象的 load/store）；(b) `Linker` 跨模块合并 `strings`/`object_keys`/`object_types`（`build_month_grid` 要造 `Value::Obj` cells）。HTTP shim 抽成 native，015-notes 不回归。

**风险锚点（已读源码确认）：**
- `Linker::link`（`loader.rs:246-328`）**只**合并 code + 重定位 + `global_symbols`；`strings`/`object_keys`/`object_types` 在 `Module` 上被丢弃 —— **HIGH**，对象字面量跨模块索引错乱，必须补合并。
- 正确 flash 路径已存在：`VMLoader::bootstrap` → `VirtualFlash::from_vec_with_metadata`（`loader.rs:120-139`），携带 object_keys/object_types；`VmBridge::new` 现在用 `VirtualFlash::new(0)` 手搓、丢了 metadata。
- state 是 `GenericInstanceData` 堆对象 id `4000000`，**按位置** `get_field(idx)`/`set_field(idx)`（`vm_bridge.rs:231,299-350`）—— shim 复用现有 `GET_GENERIC_FIELD`/`SET_GENERIC_FIELD`，不动堆布局。

**子任务（按序）：**
1. **`Linker` 跨模块合并表**（`loader.rs:246`，HIGH 风险，先发带单测）：新增 `link_full() -> CompiledPackage`，按拼接顺序对每模块 `strings`/`object_keys`/`object_types` 做索引重定位，回填该模块 code 里的 `LOAD_STR`/`CREATE_OBJ` 立即数。先读 `opcode.rs`+`engine.rs` 确认立即数宽度/位置。Gate：`cargo test -p auto-lang --lib`。
2. **imported-module 函数加载**（`vm_bridge.rs:135` 扩 `new()` 签名 + `lib.rs:1708-1744`）：抽完 WidgetDecl 后用真编译器把 app+imports 编成多模块 program（复用 `CompileSession::resolve_uses`/`take_compiled_modules`），产出 `Vec<Module>` 传进 `VmBridge::new`。Gate 单测：`exports_by_name` 含 `build_month_grid`。
3. **handler 用真 Codegen 编译 + 状态绑定 shim**（新文件 `crates/auto-lang/src/ui/handler_codegen.rs`）：`rewrite_state_refs`（用 `state_field_names` 分类 state 字段）+ 合成 `Stmt::Fn`（prologue `GET_GENERIC_FIELD` load、epilogue `SET_GENERIC_FIELD` 写回、`RET`）+ 调真 `Codegen`。`vm_bridge.rs:140-162` 换用它；删 `compile_handler_stmts`/`compile_stmt`/`compile_expr`/`CompileContext`（`:893-1175`）。Gate：counter 自增回归 + `build_month_grid` 产 42 cells。
4. **unified flash + dispatch**（`vm_bridge.rs`）：`new()` 用 `link_full()` 合成 `CompiledPackage` 走 `VMLoader::bootstrap`（带 metadata）替换手搓 flash；`call_handler` 改 `vm.call_fn_by_name`；删 `handler_closures`/`handler_addrs`；`new()` 失败 `log::warn!`。
5. **统一 Init 到 bytecode**（`dynamic.rs:584`、`lib.rs:1752-1756`、`vm_bridge.rs:468-808`）：`fire_init(&mut self)` → `bridge.call_handler("Init",&[])`；删 `call_handler_ast` + AST 解释器死代码。
6. **HTTP shim 抽成 native**（新文件 `crates/auto-lang/src/ui/native_api.rs`，`#[cfg(feature="ui-interpreter")]`）：从 `try_api_call` 抽 5 个 ureq native + `register_ui_api_natives()`；handler 编译前注册使 Codegen 发 `CALL_NAT`。Gate：015-notes 端到端。
7. **还原 016-calendar 动态设计**（`app.at`/`calendar_util.at`）：删诊断态，恢复 `DateTime.now()` Today、真实 `add_months_year/month`+`month_name` handler、Obj-cell view 分支。今天/选中高亮仍受 `extract_style` loop binding 限制，保持静态 per-branch class（Phase 2 follow-up）。

**验证（headless）：** 杀僵尸 `taskkill //F //IM auto.exe`；`auto r -r vm`；MCP snapshot 断言 `.days` 42 cell、`month_label` 正确、PrevMonth/NextMonth 后变化；回归 001/002/013/015/016 vm 模式。

---

### Phase 2 — 前端动态网格 + 真导航（单 Calendar widget）

**Files:**
- Rewrite: [examples/ui/016-calendar/src/front/app.at](examples/ui/016-calendar/src/front/app.at)
- Create: `examples/ui/016-calendar/src/front/calendar_util.at`

**calendar_util.at 核心：**
```auto
use datetime

/// 构建一个月的 42 格网格（含前后月填充）。Su 开头（weekday: 0=Mon..6=Sun，需换算成 Su 开头）。
pub fn build_month_grid(year int, month int, today str, selected str, events []Event, holidays []Holiday) []DayCellData {
    var cells List<DayCellData> = List<DayCellData>.new()
    let offset = weekday_of(year, month, 1)   // 纯 Auto，0=Sunday，直接当 1 号前的填充格数
    let dim = days_in_month(year, month)
    // 前面填充：上月末尾几天
    // ... 逐格构造 DayCellData（is_today/is_selected/is_other_month/event_count/is_holiday）
    return cells.to_array()
}
```

> **关键换算**：`weekday_of()`（纯 Auto，Phase 1）直接返回 **0=Sunday**，与日历表头 Su 开头一致，`offset = weekday_of(year,month,1)` 直接用、无需换算。**Phase 2 第一步先写一个调试输出确认 offset 对**（这是日历最常见的对不齐 bug）。

**app.at 改造要点：**
- `model` 改成上面的动态字段；删掉 d1..d7 和写死的日期。
- `msg Msg { Init, PrevMonth, NextMonth, Today, SelectDay(str) }`
- `on`：
  - `.Init` → `today = DateTime.today_str()`；纯 Auto split today 出 year/month；`days = build_month_grid(...)`。
  - `.PrevMonth` → `let (y, m) = add_months_ym(.year, .month, -1); .year = y; .month = m;` → 重建 days（Phase 4 再接事件/节假日，先空）。
  - `.NextMonth`/`.Today` 同理。
  - `.SelectDay(date)` → `.selected_date = date` → 重建 days。
- `view`：
```auto
grid {
    for cell in .days {
        if cell.day == 0 {
            text "" { style: "..." }   // 填充格
        } else {
            button cell.day {
                onclick: .SelectDay(cell.date)
                style: "...今天/选中/节假日条件样式..."
            }
        }
    }
    cols: 7
    gap: 0
    style: "w-full mt-4"
}
```

> **已验证并修复（Plan 323 Phase 2）**：`for` 作为 `grid` 子节点**原本不**产出多 cell——`ForLoop` handler 返回单个 `View::Column` 包住所有迭代，`convert_grid` 把它当 1 个 cell，导致动态日历塌成一根竖条。
> 已在 [aura_view_builder.rs](crates/auto-lang/src/ui/aura_view_builder.rs) 修复：
> - 新增 `for_loop_iterations` helper（非 tracked 路径），`convert_grid` 用 `flat_map` 把每个 `for` 子节点展开成逐迭代的 cell；
> - `convert_grid_tracked_ctx`（VM 每帧渲染走的 tracked 路径，F12 关也走它）同步展开：每个 cell 按顺序分配 `cell_idx` 路径，使 build-time 路径 `[..cell_idx]` 与 `render_dynamic_view` Grid arm 的访问顺序一致。
> 回归测试：`convert_grid_flattens_for_loop_into_cells` + `convert_grid_tracked_flattens_for_loop_into_cells`（7 元素数组 → 7 cell，tracked 版断言 cell 路径为 `[0..7]`）。

**验证：**
- vm + rust 两模式：默认显示当前月（2026-06），日期数对、1 号对在正确的星期列、今天高亮。
- 点 ◀ ▶ 真的切月，月份标题和网格都变。
- 点某日 → 选中高亮。
- **回归**：`cargo test -p auto-lang --lib`（Plan 319 的 grid 测试不能挂）。

---

### Phase 3 — 拆子 widget（MonthHeader / WeekdayRow / DayCell）

**Files:**
- Create: `month_header.at`, `weekday_row.at`, `day_cell.at`
- Modify: `app.at`（用 `use` 引入并组合）

**MonthHeader(year, month)：**
```auto
widget MonthHeader(year int, month int) {
    view {
        row {
            button "◀" { onclick: ???, ... }
            text f"${month_name(.month)} ${.year}" { style: "..." }
            button "▶" { onclick: ???, ... }
            style: "w-full items-center justify-between"
        }
    }
}
```

> **待解问题（实现时定）**：子 widget 如何向父 widget 发事件（◀/▶ 点击要让父 Calendar 切月）？015-notes 里子→父目前靠共享 state 同步，没有显式事件冒泡。**两种方案**：
> (a) 导航按钮留在**根 Calendar** 里（不放进子 widget），只把"纯展示"的标题/表头/单元格拆成子 widget——最简单，推荐。
> (b) 给 MonthHeader 传一个回调 props（需调研 props 是否支持函数/事件类型）。
> **先按 (a)**：MonthHeader 只显示标题 + 今天按钮；◀/▶ 按钮留在 app.at，onclick 直连 `.PrevMonth`/`.NextMonth`。这样不依赖事件冒泡能力。

**WeekdayRow：** 7 个 `text`（Su Mo Tu We Th Fr Sa），`style` 让每格等宽，**与下方 grid 列严格对齐**（业界常见坑）。

**DayCell(cell: DayCellData)：**
```auto
widget DayCell(cell DayCellData) {
    view {
        if .cell.day == 0 {
            text "" { style: "h-10" }
        } else {
            button .cell.day {
                onclick: ???    // 同上，选中事件——见下
                style: 条件样式（today=蓝底白字 / selected=蓝边 / holiday=红字 / 普通=灰字）
            }
            if .cell.event_count > 0 {
                text "•" { style: "蓝点" }   // 事件指示
            }
        }
    }
}
```

> **事件冒泡问题（同 Phase 3 的 MonthHeader）**：DayCell 点击要让父 Calendar 执行 `.SelectDay(cell.date)`。同样**优先方案 (a)**：把 DayCell 做成**纯展示**，点击逻辑由父 widget 的 `for` 循环里直接绑 `onclick: .SelectDay(cell.date)`——即 DayCell 不处理点击，父在循环里包一层可点击容器。或者：DayCell 接收一个 `on_select` 事件 props（需验证 AutoUI 是否支持事件类型 props，**实现时 spike 一次**）。

**验证：**
- 拆分后 vm+rust 渲染与 Phase 2 完全一致（纯重构，零行为变化）。
- DayCell 的今天/选中/节假日条件样式正确。
- `cargo test -p auto-lang --lib`。

---

### Phase 4 — 后端：节假日 + 事件持久化

**Files:**
- Create: `examples/ui/016-calendar/src/back/api.at`、`back/db.at`
- Modify: `app.at`（`use back.api: ...`，`.Init`/切月时拉数据）

**back/api.at（类型 + `#[api]` 端点）：**
```auto
pub type Event = {
    id: int
    date: str         // "2026-04-15"
    title: str
    color: str        // 可选，事件点颜色
}
pub type Holiday = {
    date: str         // "2026-04-04"（清明）
    name: str         // "清明节"
}

#[api(method = "GET", path = "/api/holidays")]
pub fn list_holidays(year int, month int) []Holiday { use db; return db.holidays_in(year, month) }

#[api(method = "GET", path = "/api/events")]
pub fn list_events(year int, month int) []Event { use db; return db.events_in(year, month) }

#[api(method = "POST", path = "/api/events")]
pub fn create_event(date str, title str) Event { use db; return db.create_event(date, title) }

#[api(method = "DELETE", path = "/api/events/:id")]
pub fn delete_event(id int) bool { use db; return db.delete_event(id) }
```

**back/db.at：**
- 内置节假日表（先放 2026 中国法定节假日样例：元旦/春节/清明/劳动/端午/中秋/国庆；**数据准确性非重点，能演示即可**，注释说明可换 API）。
- 内存事件存储 + JSON 持久化（仿 015-notes 的 db.at）。
- `holidays_in(y,m)`/`events_in(y,m)` 按 date 前缀过滤。

**前端接线：**
- `use back.api: list_holidays, list_events, create_event, delete_event`
- `.Init` 和每次切月：`.holidays = list_holidays(.year,.month)`、`.events = list_events(.year,.month)` → 重建 `days`（让 event_count/is_holiday 生效）。

**验证：**
- vm+rust：节假日格变红/标名；有事件的格出现指示点。
- 后端跑起来（`auto run` 或对应启动方式），事件能新建/删除并持久化（重开还在）。
- 注意：前端调后端是同步函数调用语义（015-notes 模式），确认本例的渲染管线对 `.Init` 里调后端函数能正确返回数据。

---

### Phase 5 — 事件侧栏 + 交互闭环

**Files:**
- Create: `event_sidebar.at`
- Modify: `app.at`（布局：左日历 + 右侧栏）

**EventSidebar(selected_date, events)：**
- 显示 `selected_date` 当天的事件列表。
- 一个 input + "添加" 按钮 → `.AddEvent(title)` → `create_event(.selected_date, title)` → 刷新 events。
- 每条事件有删除按钮 → `.DeleteEvent(id)` → `delete_event(id)` → 刷新。
- 若选中日无事件，显示空态提示。

**app.at 布局：**
```auto
row {
    col { MonthHeader; WeekdayRow; grid { for ... DayCell } }
    col { EventSidebar(...) }
    style: "..."
}
```

**验证：**
- 点格 → 侧栏切到该日事件。
- 新建/删除事件 → 网格指示点同步增减、侧栏列表更新。
- vm+rust 一致。

---

### Phase 6 — 打磨与边界

- **表头列对齐**：WeekdayRow 的 7 格与 grid 的 7 列等宽对齐（用同样的 `w-full` + 等分）。
- **今天按钮**：MonthHeader 加 "Today" 按钮 → `.Today`。
- **边界**：跨年切月（12 月→1 月年+1）、2/29 闰年、前后月填充格不可选中（`is_other_month` 样式置灰 + 点击切到那个月，可选）。
- **README 更新**：说明动态数据流、前后端结构、节假日数据来源。
- **回归全量**：`cargo test -p auto-lang --lib`；vm+rust 目视一致；015-notes 等其它示例不受 stdlib 改动影响。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| `for` 作为 `grid` 子节点不产出多 cell | **已修复**：`convert_grid` + `convert_grid_tracked_ctx` 增加 `for` 展开（`for_loop_iterations` helper / 内联展开），cell 按 `cell_idx` 分配路径；2 个回归测试通过 |
| 子 widget 无法向父发事件（点击） | 优先方案 (a)：纯展示子 widget + 父在 for 循环里直接绑 onclick；不依赖事件冒泡 |
| 日期算术 transpile 缺口（DateTime 仅 VM） | 纯 Auto 写（闰年表/Zeller-Sakamoto），自动 transpile；仅 `today_str` 做 a2r(chrono)+a2vue(JS 垫片) 映射 |
| a2vue `today_str()` 时区 | JS 用 `getFullYear/getMonth/getDate`（本地），勿用 `toISOString`（UTC）；a2r 用 chrono `Local` |
| 误用 `#[vm]` DateTime 字段（.year()/.weekday()） | 新函数全接收整数；today 字符串拿到后立刻 split 成 int，算术只在 int 上 |
| 前端 `.Init` 调后端函数的数据返回时序 | 015-notes 已验证此模式可用；本例照搬，先确认再扩 |
| stdlib 改动影响其它示例 | 回归全量 lib 测试 + 跑几个依赖 datetime 的示例 |
| 节假日数据准确性 | 非重点，内置 2026 样例 + 注释"可换 API"；不纠结准确性 |
| 样式重复仍存在 | 本轮接受（DayCell 已大幅减少）；通用样式预设另立计划 |

## Out of Scope（本计划不做，记为 follow-up）

- 周/日多视图、事件拖拽改期、重复事件、农历、`dayMaxEvents` 溢出 "+more"。
- 通用 Tailwind 样式预设/别名/`@apply` 语言能力（另立计划）。
- 节假日真实 API 接入（本轮内置表）。
- `VNodeKind::Grid`（Plan 319 follow-up，DevTools 树把 grid 显示为 Column）。

## Verification（总）

1. **Phase 1**：纯 Auto `days_in_month`/`weekday_of` a2r 测试过（自动 transpile，免改 transpiler）；`today_str()` VM/a2r/a2vue 三路返回正确今天；`cargo build -p auto` 干净。
2. **Phase 2-3**：vm+rust 两模式目视一致；切月/选中/今天高亮正确；offset 对齐；`cargo test -p auto-lang --lib` 无回归（含 Plan 319 grid 测试）。
3. **Phase 4-5**：后端起得来；节假日/事件 CRUD 持久化；前后端数据流通；vm+rust 一致。
4. **Phase 6**：边界（跨年/闰年/填充格）、表头对齐、README；全量回归绿。
5. 全程专用 worktree；master 稳定；全绿后 merge 回 master。
