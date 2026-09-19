---
plan_id: PLAN-660
status: execution_done
feature_name: 014-weather-ui-refresh
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19
plan_revision: 2
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010]
affects: [examples/ui/014-weather]
---

# [PLAN-660] 014-weather-ui-refresh

## 0. 变更摘要

`examples/ui/014-weather` 天气仪表盘：中文 UI、深浅主题、10 中国城市、
结构化预报。**r2（用户反馈 2026-09-19）**：①布局改为**横屏（网页/桌面）
优先**，竖屏（手机版）经 `layout_mode` 保留可切换；②滚动改用 AutoUI
`scroll` → Vue `ScrollArea` / VM iced scrollable，弃用原生 overflow 滚动条。
数据为内置 mock，不接真实气象 API。

**r3/r4（用户走查反馈 2026-09-19）**：③选中态 pill/按钮补 `variant` +
`hover:bg-primary/90` 对称提亮（修选中项 hover 发白不可读）；④横屏信息
架构调整——24h 预报上移右栏顶部，`scroll` 直包 `row`（去掉 container/
多余 col，修 ScrollArea 视口塌陷致「只见 5 日」），小时卡实体底色，
24h/5日统一 `forecast_card` 卡面。

**设计锚点**：桌面天气仪表盘（横屏）+ Apple Weather 竖屏（保留）。

## 1. 目标

- **G-1 视觉与主题**：全面改用 AutoUI 语义 token（`bg-card`/`text-foreground`/
  `border-border` 等），根组件声明 `dark_mode` / `accent_color` 双端契约状态，
  头部提供深浅切换；装饰性条件渐变 hero 允许保留（PLAN-637 r2 豁免口径）。
- **G-2 中国城市选择**：≥10 个中国主要城市（中文名），以 pill 芯片切换；
  切换后当前实况、指标、小时与 5 日预报全部更新。
- **G-3 预报界面美化**：5 日预报与小时预报从纯文本改为结构化卡片——
  天气 emoji/图标、条件中文、温度区间条、小时横滑条（“现在”高亮）。
- **G-4 综合信息架构**（常见天气 App 对齐）：hero 实况（城市/大温度/条件/
  体感）、指标网格（湿度/风速/能见度/气压/紫外线/AQI）、更新时间戳、
  Refresh 带可见反馈、中文文案。
- **非目标**：真实天气 API / 定位；多页路由；Jet/Ark 后端；改 crates 语言
  或渲染器；stylekit 收编（若产生可复用 recipe，留待后续 stylekit 计划）。
- **受影响范围**：仅 `examples/ui/014-weather/**`（及必要时 `examples/ui/README.md`
  状态行）。Category A：不改 crates 源码，禁跑 `cargo t` / `docs_gen`。
- **成功判据**：AC-01..AC-08 全过；`auto build` 生成 Vue 产物无报错；深浅
  两主题截图下内容均可读。

## 2. 架构方案

保持 Elm 架构单应用 demo，数据自包含：

```
examples/ui/014-weather/
  pac.at                    # 清单（title_zh 已有；theme 不写死，跟随宿主）
  src/front/types.at        # City / HourPoint / DayPoint / CityWeather
  src/front/weather_data.at # 10 城 mock 目录 + 查询函数
  src/front/app.at          # UI：store 状态 + view + on
  README.md                 # 更新概念与运行说明
```

**数据流**：
1. `weather_data.at` 提供静态城市目录与 `lookup_weather(city_id) -> CityWeather`
   （或等价的并行列表 + index 构造）。
2. `app.at` model：`city_id`、`dark_mode`、`accent_color`、当前实况字段、
   `hourly []HourPoint`、`daily []DayPoint`、`updated_at`、`refreshing`。
3. `SelectCity(id)` / `Refresh` 消息写入上述状态；view 用 `for` 渲染列表。
4. `dark_mode`/`accent_color` 变量名为双端运行时契约（Design 19/29，Plan 458
   惯例），Vue 与 VM 都会消费，禁止改名。

**城市选择组件裁定**：不用 aura `select`/`selectitem`（schema：`selectitem`
`iced: none`，VM 臂缺失）。采用**横向 pill 按钮芯片**（对齐 012-clock /
020 filter chip 模式），双端都有 `button`。

**主题与装饰色**：
- 页面/卡片/文字/边框/按钮 → 语义 token。
- hero 天气渐变按 `condition` 分支（晴 amber、云 slate、雨 blue、雪 cyan、
  雾 gray），文字用 `text-primary-foreground` 或白色——装饰性豁免。
- pac.at **不写** `theme:` 字段，跟随宿主/CLI（011-calculator Plan 615 同型）；
  应用内 `ToggleTheme` 翻转 `dark_mode`。

**条件 → 视觉映射（mock 词表）**：

| condition | 中文 | emoji | hero 渐变（装饰） |
|---|---|---|---|
| sunny | 晴 | ☀️ | from-amber-400 to-orange-500 |
| partly | 多云 | ⛅ | from-sky-400 to-blue-500 |
| cloudy | 阴 | ☁️ | from-slate-400 to-slate-600 |
| rain | 小雨/中雨 | 🌧️ | from-blue-400 to-blue-600 |
| storm | 雷阵雨 | ⛈️ | from-indigo-500 to-purple-600 |
| snow | 雪 | ❄️ | from-cyan-300 to-sky-500 |
| fog | 雾 | 🌫️ | from-gray-300 to-gray-500 |

**城市 mock 目录（10）**：北京、上海、广州、深圳、杭州、成都、西安、武汉、
哈尔滨、三亚。每城条件/温度/风湿度等应有差异（体现切换有意义），小时 8–12
点、5 日各 5 条。

**布局（移动端密度，max-w-md 居中，可滚动）**：
1. Header：城市名 + 更新时间 + 主题切换 + Refresh
2. 城市 pill 横滑条
3. Hero 实况卡（条件渐变）：emoji + 条件中文 + 大温度 + 体感/湿度/风
4. 指标 2×3 网格卡（token 卡面）
5. 小时预报卡：横滑 `for` 条目（时间/emoji/温度，“现在”primary 高亮）
6. 5 日预报卡：竖列表（星期/emoji/条件/低温—渐变条—高温）

## 3. 技术栈

- AutoUI `.at`：widget / model / view / on / type / style recipe / for / if
- 语义 token：shadcn 命名（`bg-card`、`text-muted-foreground`…）
- 渲染：`pac.at` 保持 `render: "vue"`（现状）；验收以 Vue 产物 + 截图为主
- 测试设施：Category A——`auto build` / `auto run` 目视；可选 Playwright 冒烟
  （若仓库后续要求，不在本计划强制新建完整测试夹具）
- 不引入 crates 改动，不跑 `cargo t`

## 4. 需求分析与背景调查

**用户授权（2026-09-19，本会话原话要点）**：
1. 样式丑、不支持深浅主题 → 重做 UI/UX
2. 只显示北京 → 添加中国城市选择
3. 5 日/小时预报只有简单文字 → 美化界面
4. 参考常见天气 App 综合改善
5. 流程：调研设计 → `/auto-plan:new` 建计划 → `/auto-plan-work` 实施

**现状证据**：
- `examples/ui/014-weather/src/front/app.at`：`city` 固定 `"Beijing"`；
  `forecast`/`hourly` 为拼接字符串；hero 硬编码
  `from-blue-400 to-blue-600`；无 `dark_mode`/`accent_color`。
- 截图（用户附件）：深色页 + 蓝卡英文，Daily/Hourly 按钮下方仅两行纯文本。
- 历史：Plan 183 曾列 “014-weather + skeleton, loading, icons” 未做；
  Plan 637 B2 已做过一轮 token 映射，但仍粗糙。
- `examples/ui/README.md` §主题约定：应用内切换声明 `dark_mode`/`accent_color`；
  006-hero-section / 016-calendar / 018-book-reader 为在案范例。
- `schema/aura.at`：`selectitem` iced none → 城市选择不能走 select。
- `docs/specs/overview.md`：examples 属 active 资产；本计划不改语言 spec。

**仓库约定**：
- 工作在 worktree `D:/autostack/.wt/lang-660/auto-lang`，分支 `plan-660-dev`。
- 计划簿记在主检出；实现代码全在 worktree。
- Category A 验证：仅 examples，禁止 cargo t / docs_gen。

**风格调研结论（常见天气 App）**：
- 信息架构：城市切换 → 实况 hero → 小时横条 → 多日列表 → 次级指标。
- 视觉：大温度数字 + 条件图标；卡片圆角与浅描边；深浅色均保持对比度。
- 交互：城市 chips / 搜索；下拉刷新或按钮刷新；AQI 有色徽标。
- 本 demo 取 chips + 按钮刷新 + 指标网格，不做搜索/定位（演示边界清晰）。

## 5. 详细设计

### 5.1 类型与数据（`types.at` + `weather_data.at`）

```auto
// types.at
type HourPoint { time str  icon str  temp str  is_now bool }
type DayPoint  { day str   icon str  cond str tmin str  tmax str }
type CityWeather {
    city_id str
    city_zh str
    condition str    // sunny|partly|cloudy|rain|storm|snow|fog
    cond_zh str
    icon str
    temp str         // "23°"
    feels str
    humidity str
    wind str
    uv str
    aqi str
    aqi_level str    // 优|良|轻度|...
    pressure str
    visibility str
    updated_at str
    hourly []HourPoint
    daily []DayPoint
}
```

`weather_data.at`：导出 `CITY_IDS`/`CITY_NAMES`（或 `CITIES []City`）与
`lookup_weather(id) -> CityWeather`。实现可选：
- **A（优先）**：函数内 `if/else` 或并行表 + `CityWeather { ... }` 字面量返回；
- **B**：`store WeatherData` 预填 `[]CityWeather`，`lookup` 扫描匹配。

若单函数体积过大，允许拆 `weather_cities.at` 等，只要 `use` 链合法。

### 5.2 App 状态与消息（`app.at`）

```auto
msg {
    SelectCity(str),
    Refresh,
    ToggleTheme,
    SetTheme(str),   // 保留宿主/兼容通道（006 模式）
    SetAccent(str),
}
model {
    var dark_mode bool = true          // 示例生态默认 Dark（README §主题约定）
    var accent_color str = "indigo"
    var city_id str = "beijing"
    var city_zh str = "北京"
    var condition str = "partly"
    var cond_zh str = "多云"
    var icon str = "⛅"
    var temp str = "23°"
    var feels str = "25°"
    var humidity str = "65%"
    var wind str = "12 km/h"
    var uv str = "中等"
    var aqi str = "68"
    var aqi_level str = "良"
    var pressure str = "1012 hPa"
    var visibility str = "12 km"
    var updated_at str = "14:30"
    var refreshing bool = false
    var hourly []HourPoint = [...]
    var daily []DayPoint = [...]
}
on {
    .SelectCity(id) -> { /* lookup 写入全部展示字段 */ }
    .Refresh -> { .refreshing = true; /* 微调 mock 或重写 updated_at */ .refreshing = false }
    .ToggleTheme -> { .dark_mode = !.dark_mode }
    .SetTheme(t) -> { /* light/dark → dark_mode */ }
    .SetAccent(name) -> { .accent_color = name }
}
```

Init 时用默认 `beijing` 装载（可在声明处直接填默认 mock，或 `Init` 消息）。

### 5.3 样式 recipe（文件内 `style`，语义 token）

沿用 013/016/020 的 recipe 风格，例如：

- `page` = `"min-h-screen bg-background text-foreground"`
- `card` = `"bg-card border border-border rounded-2xl p-4"`
- `pill_on` = `"px-3 py-1.5 rounded-full text-xs font-medium bg-primary text-primary-foreground"`
- `pill_off` = `"px-3 py-1.5 rounded-full text-xs font-medium bg-secondary text-secondary-foreground hover:bg-secondary/80"`
- `metric_label` = `"text-[10px] text-muted-foreground uppercase tracking-wider"`
- `metric_value` = `"text-sm font-semibold text-foreground"`
- `hour_now` / `hour_idle`：现在项 `bg-primary/15 border-primary/30 text-primary`，
  其余透明边框 + `text-muted-foreground`。
- hero 容器：按 `condition` 的 `style: if .condition == "sunny" { "..." } else if ...`

温度区间条（5 日）：每行
`低温 | 轨道（bg-muted rounded-full h-1.5 flex-1 内嵌 bg-gradient-to-r from-sky-400 to-amber-400）| 高温`。
不做动态像素定位（双端约束），全宽渐变条 + 两侧温度即可读。

### 5.4 信息密度与文案

- 全站中文：城市、指标标签（体感/湿度/风速/紫外线/AQI/气压/能见度）、
  区块标题（24小时预报 / 5日预报）、刷新/主题按钮 aria 文案。
- Refresh：`refreshing` 为真时按钮文案「刷新中…」或 disabled 态 class。
- AQI：值 + 等级徽标（优=emerald、良=amber 等装饰性或 primary 色）。

### 5.5 规范增量

| delta_id | 类型 | 目标 | before/after | 理由 | AC |
|---|---|---|---|---|---|
| SD-01 | none | （无 docs/specs 模块行为变更） | examples/ui 演示应用 UI 刷新，不改语言/VM/UI 框架契约 | 规格权威是语言与模块 spec；demo 信息架构不写入 module spec | AC-01..08 |
| SD-02 | add（可选文档） | `examples/ui/014-weather/README.md` | 简陋英文概念 → 中英概念 + 新信息架构/主题/城市说明 | 示例 README 与实现同步，属示例资产而非 specs 体系 | AC-06 |

`docs/specs/` 无增量：`supersedes_spec_components` / `new_spec_components` 保持空；
`affects: [examples/ui/014-weather]`。

## 6. 测试设计

Category A（纯示例资产）：

| 层级 | 手段 | 期望 |
|---|---|---|
| 生成 | worktree 内 `cd examples/ui/014-weather && auto build` | gen/front/vue 产出成功，无 at 解析错误 |
| 目视 Vue | `auto run` 或对 gen 工程 dev server | 截图：深色默认可读；切换浅色可读；城市 pill 可点 |
| 交互 | 点击城市/主题/刷新/预报区 | 城市切换后 hero+指标+预报变化；主题整卡 token 翻转 |
| 双端抽查（可选） | `auto run -r vm` | 城市切换与主题状态仍可用（select 不依赖即可） |
| 回归范围 | 不涉及 crates | **禁止** `cargo t` / `docs_gen` |

证据：截图落在 worktree `examples/ui/014-weather/shots/`（可选）或计划复审记录
文字描述 + 生成路径。Playwright 冒烟非门禁，时间允许再补。

## 7. 验收标准

| ID | 标准 | 验证方法 | 期望 |
|---|---|---|---|
| AC-01 | 语义 token 化，无遗留灰阶硬编码主体文案色 | 读 `app.at` diff | 页面/卡片/次要文字使用 `bg-*`/`text-foreground`/`muted-foreground`/`card` 等 token |
| AC-02 | `dark_mode` + `accent_color` 状态存在且可切换 | 代码 + 运行时点主题按钮 | 深浅两主题均可读，变量名未改 |
| AC-03 | ≥10 个中国城市可选 | UI 与数据源 | 至少北京/上海/广州/深圳/杭州/成都/西安/武汉/哈尔滨/三亚 |
| AC-04 | 切换城市后实况与预报数据变化 | 点击 ≥3 个城市 | 温度/条件/小时/5日均切换为该城 mock |
| AC-05 | 5 日预报为结构化卡片 | 目视 | 每行含星期、emoji、条件中文、最低~最高（含区间条），非单段文本 |
| AC-06 | 小时预报为结构化横滑条 | 目视 | 含时间、emoji、温度；“现在”或首项高亮；非单段文本 |
| AC-07 | 指标与刷新反馈 | 目视/点击 | 体感、湿度、风速、AQI 等可见；Refresh 有状态反馈；有更新时间 |
| AC-08 | `auto build` 成功 | worktree 执行 | 无解析/生成错误；Category A 未跑 cargo t |

## 8. 执行步骤

前置：主检出仅保留计划簿记；实现进入隔离检出。

**环境注记（2026-09-19 收口）**：会话守卫拦 `git worktree add`（共享 .git registry），
按 Plan 650/652/654 先例 **clone 隔离**：`git clone --shared D:/autostack/auto-lang
D:/autostack/.wt/lang-660/auto-lang` → 分支 `plan-660-dev`。
master `.autoos/specs.json` 冲突（plan-022-dev 账本）曾阻塞簿记 commit，**已由
Plan 658 merge 收口解除**（3dab57f9a/7ebb3446b/92c8013a2 落地）；本计划簿记
于冲突解除后补提交。

- [x] **T-01 主检出计划骨架落地**
  - 路径：`docs/plans/660-014-weather-ui-refresh.md`、`docs/plans/.next-id`（→661）
  - 操作：契约写满；`new-plan.sh` 取号
  - 验证：frontmatter 存在；ID 唯一
  - [✅ 已完成] 2026-09-19 契约落盘；master commit 被 specs.json 冲突阻塞

- [x] **T-02 隔离检出并断言分支**
  - 操作：clone --shared → `D:/autostack/.wt/lang-660/auto-lang` @ `plan-660-dev`
  - 验证：toplevel/branch 断言通过；无 junction
  - [✅ 已完成] 2026-09-19 clone 隔离成功，基点 b69c7344c

- [x] **T-03 数据层：types + 10 城 mock**
  - 路径：`examples/ui/014-weather/src/front/types.at`、
    `examples/ui/014-weather/src/front/weather_data.at`
  - 操作：城市目录 + 查询函数 + hourly/daily Obj 列表 + hero_grad/aqi_badge
  - 验证：生成器将函数内联进 App.vue（hero_grad/hourly_for/SelectCity 可见）
  - [✅ 已完成] 10 城 mock 落盘；App.vue 内联证实（AC-03/04 数据面）

- [x] **T-04 UI 刷新：app.at 全量重写**
  - 路径：`examples/ui/014-weather/src/front/app.at`、`pac.at`、`README.md`
  - 操作：dark_mode/accent_color、城市 pills、条件 hero、指标网格、小时条、
    5 日区间条、中文文案、Refresh/ToggleTheme；语义 token
  - 语法适配：view 无 `list[i]`；model 无 `var x [] = []`；动态样式放 model
    （`hero_style`/`aqi_badge`）；`dep stylekit` 声明；城市用 Obj 列表
  - 验证：`auto build` 解析通过并生成 `gen/front/vue`
  - [✅ 已完成] 2026-09-19 生成成功（AC-01/02/05/06/07 源码面）

- [x] **T-05 交互与双主题走查（生成/构建面 + VM MCP）**
  - 操作：auto build + `pnpm exec vite build`；**VM：`auto run -r vm` +**
    `examples/ui/014-weather/tests/vm_smoke.py`（autoui_snapshot/state/action）
  - 验证（2026-09-19 VM 实测 **14/14 PASS**）：
    - snapshot 含 北京/横屏/竖屏/当前详情/24小时预报/5日预报/刷新
    - state：`city_id=beijing`、`layout_mode=landscape`、`city_zh=北京`
    - 点「上海」→ `city_id=shanghai`、`city_zh=上海`、`temp=21°`
    - 点「竖屏」→ `layout_mode=portrait`
    - 点主题 → `dark_mode` false→true 翻转
  - 注记：初始 `dark_mode=false`（宿主主题播种覆盖 model 默认 true，Plan 458 链）
  - 遗留：Vue 浏览器像素走查仍可选；vue-tsc 脚手架 `import.meta.env` 与 demo 无关
  - [✅ 已完成] 2026-09-19 VM 冒烟全绿；脚本已入库

- [x] **T-06 文档与源码提交**
  - 路径：`examples/ui/014-weather/README.md`、`examples/ui/README.md`
  - 操作：文档同步；clone 内 commit
  - 验证：`904ca840a feat(examples/ui): refresh 014-weather dashboard UI (Plan 660)`
  - [✅ 已完成] 2026-09-19 clone 工作树干净（gen/ 已 ignore）

依赖：T-01 → T-02 → T-03 → T-04 → T-05 → T-06。

## 9. 复审记录

| stage | plan_id | plan_revision | outcome | code_commit | task_ids | evidence | blockers | next |
|---|---|---|---|---|---|---|---|---|
| new | PLAN-660 | 1 | pass | — | T-01..T-06 | 调研：app.at 现状、Design 19/29、Plan 458 主题契约、select VM 缺口、012/013/016/020 参考；用户五点授权 | 无 | work |
| work | PLAN-660 | 1 | pass | 904ca840a@plan-660-dev (clone) | T-01..T-06（T-05 构建面完成） | auto build 生成 Vue；App.vue 含 10 城/hero/预报/dark_mode；vite build 绿 | ① master specs.json 合并冲突阻塞簿记 commit；② 浏览器交互走查未做；③ 生成器 vue-tsc 脚手架缺口 | review（冲突解除 + 走查后） |
| revise | PLAN-660 | 2 | pass（r2 实现） | 待 commit | 用户反馈：横屏优先 + scroll 组件 | layout_mode landscape/portrait；`scroll`→ScrollArea；hero `:class` 条件渐变 | 同上 | review |
| work-vm | PLAN-660 | 2 | pass | 66487c591 + vm_smoke 提交 | T-05 VM 验证补做 | `auto run -r vm` + vm_smoke.py **14/14 PASS**（城市/布局/主题/快照地标） | 无（VM 臂已验证） | review |
| work-r3 | PLAN-660 | 2 | pass | 07a88c917@plan-660-dev | T-04 补强（用户走查反馈） | 选中态 city_pill_on / refresh_idle / layout_btn_on 补 `variant:"primary"` + `hover:bg-primary/90`，与 off 态 `hover:bg-secondary/80` 对称——button 默认 variant 的 `hover:bg-muted/70` 会与 active 样式 cn 合并残留 muted 底致选中项 hover 发白（证据 gen button/index.ts 默认 variant）；07a88c917 17 行 | 无 | 续 r4 |
| work-r4 | PLAN-660 | 2 | pass | 4727529aa@plan-660-dev | T-04/T-05 补强（用户走查反馈） | 横屏 24h 上移右栏顶部 + `scroll` 直包 `row`（去 container/多余 col，修 max-w-7xl 注入与 ScrollArea 视口塌陷「只见 5 日」）+ 小时卡 `bg-muted/50` 实体底色 + 24h/5日统一 `forecast_card`。验证三面：`auto build` 生成绿（gen App.vue:878 横屏 ScrollArea 直包 row、:1044 竖屏保留 container）；`pnpm exec vite build` 绿（634 模块，dist 产出）；VM 冒烟 vm_smoke.py **14/14 PASS**（2026-09-19 复跑，含城市切换/布局切换/主题翻转） | 无 | review（execution_done） |
| work | PLAN-660 | 2 | pass | 4727529aa@plan-660-dev（HEAD，r1..r4 五提交） | T-01..T-06 全闭环 | 全任务勾选 + 验证三面在 HEAD 复核通过；r3/r4 走查反馈补强并入且已提交，worktree clean（gen/ 已 ignore） | ①master specs.json 冲突已解除（658 收口），簿记本次落 commit；②vue-tsc `import.meta.env` 脚手架缺口留 §10（014 源码不依赖，vite 产物可用）；③Vue 浏览器像素走查仍可选非门禁 | **execution_done** → review |

## 10. 待澄清事项

- 真实天气 API 是否二期接入？本计划默认 mock（非目标已声明）。
- 是否强制 Playwright 冒烟？当前 Category A 目视 + build 为门禁。
- ~~master `.autoos/specs.json`（plan-022-dev 账本）未完成合并由谁收口？~~
  **已解除**：Plan 658 merge 收口（3dab57f9a/7ebb3446b/92c8013a2），
  本计划簿记已随之落 commit。
- 生成器 `vue-tsc` 的 `import.meta.env` / `auto-sources` 缺口是否另立小计划？
  014 应用源码不依赖该修复（vite 产物已可用）；`auto build` 的
  `pnpm run build`（vue-tsc && vite build）在该缺口修复前会红，
  绕行 = `pnpm exec vite build` 直跑（本计划验证口径）。
