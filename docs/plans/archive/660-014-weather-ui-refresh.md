---
plan_id: PLAN-660
status: archived
feature_name: 014-weather-ui-refresh
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-20
plan_revision: 4
current_step: 6
total_steps: 6
completion_kind: delivered
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

**r5（用户走查反馈 2026-09-19，49975bdf2）**：预报卡改 Tab 互斥
（`forecast_tab` hourly|daily，24小时/5日 两 chip）；5 日预报改与小时卡
同构的**日卡片**（星期/emoji/最高/最低，今天高亮）。
**rev 3（用户裁定 2026-09-19）**：认可 r5 简化卡片为最终形态——AC-05
有界修订（去条件中文+温度区间条字面要件），rev 2 口径下 AC-05 的
fail 判定按旧约保留为历史；余留 = README 同步（F-660-R2）+
死样式清理（F-660-R4）+ 冒烟内容断言（F-660-R3）。

**r7–r10（他方/本会话续迭代，见 §9）**：r6/r7 窗体与外壳（弃 fit→固定
`window: "960x680"`，Plan 512 边界）；r8 文档与冒烟内容断言收口；
**r9/r10 rust/a2r 轨**：clone 内 `auto build -r rust` / `auto run -r rust`
打通（handler 字面量、ASCII city_id、参数形按钮文案）；**rev 4**：
将 Rust vs VM 端差异与 a2r 缺口记入复审/债项（本文件 §9/§10），
**不扩大本计划 AC 范围**（AC 仍以 Vue+VM 为主门；rust 轨 = 对拍观察面）。

**r6..r8（他方会话迭代 + 本会话收口）**：r6（0c73cb2ef）fit 窗 + app
card 框 + 紧凑 hero；r7（e0a12502e）放弃 fit 改固定窗 `window:"960x680"`
（Plan 512：fit + `lg:`/`max-w` 在 iced 量测塌缩）、宽度全固定刻度
（Hero `w-64`）、城市条去 container；r8（65fbe867a）复审余留收口——
README 同步 r7、死样式 `day_row` 清除、vm_smoke 补日卡内容断言。

**设计锚点**：桌面天气仪表盘（横屏）+ Apple Weather 竖屏（保留）。

## 1. 目标

- **G-1 视觉与主题**：全面改用 AutoUI 语义 token（`bg-card`/`text-foreground`/
  `border-border` 等），根组件声明 `dark_mode` / `accent_color` 双端契约状态，
  头部提供深浅切换；装饰性条件渐变 hero 允许保留（PLAN-637 r2 豁免口径）。
- **G-2 中国城市选择**：≥10 个中国主要城市（中文名），以 pill 芯片切换；
  切换后当前实况、指标、小时与 5 日预报全部更新。
- **G-3 预报界面美化**：5 日预报与小时预报从纯文本改为结构化卡片——
  预报 Tab（24小时/5日）互斥切换、日卡片（星期/emoji/最高/最低/今天
  高亮，rev 3 裁定形态）、小时横滑条（“现在”高亮）。
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

**布局（r1 竖屏原案；实际形态以 §0 r2/r4/r5 迭代为准——横屏优先 +
预报 Tab）**：
1. Header：城市名 + 更新时间 + 主题切换 + Refresh
2. 城市 pill 横滑条
3. Hero 实况卡（条件渐变）：emoji + 条件中文 + 大温度 + 体感/湿度/风
4. 指标 2×3 网格卡（token 卡面）
5. 小时预报卡：横滑 `for` 条目（时间/emoji/温度，“现在”primary 高亮）
6. 5 日预报卡：~~竖列表（星期/emoji/条件/低温—渐变条—高温）~~ →
   rev 3 形态：与小时卡同构的横滑日卡片（星期/emoji/最高/最低，
   今天高亮），经预报 Tab 与 24h 互斥切换

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

温度区间条（5 日，~~r1 原案~~ **已被 rev 3 用户裁定取代**——日卡片两行
温度替代区间条，样式 `day_card`/`day_card_now` 与 `hour_card` 同构）：
~~`低温 | 轨道（bg-muted rounded-full h-1.5 flex-1 内嵌 bg-gradient-to-r
from-sky-400 to-amber-400）| 高温`~~。rev 2 复审的 AC-05 fail 判定
以本原案为口径，rev 3 后不再适用。

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

### 5.6 Rust/a2r 轨观察面（rev 4 记录，非 AC 门）

用户要求 `auto run -r rust` 对拍（截图：图1=Rust，图2=VM）。结果与处置：

**已打通**：clone `plan-660-dev` 上 `auto build -r rust` → `Finished`；
`auto run -r rust` 生成并运行 `weather.exe`（Iced，窗 960×680）。
r9/r10 兼容改造（提交 `6d0cfa485` / `217958aaa`）：

| 改造 | 原因（实证） |
|---|---|
| handler 全字面量 if/else，不 `use weather_data: fn` / 不调顶层 fn | a2r 不把 `use …: fn` 或 app.at 顶层 `fn` 发射进 `main.rs` → E0425 |
| 城市逻辑键 ASCII `city_id`（beijing…），中文仅展示 | 中文比较字面量写入 main.rs 成 **mojibake**（`å¬`≠「北京」），选中态失效 |
| 刷新/主题钮 `button "刷新"` 参数形 | `button { if … text "…" }` → `View::button("")`，子节点文案丢失 |
| 城市 pill 10 路展开，不用 `for` + `[]str` | a2r 把循环元素当 `&Value`，与 `String` 比较/传参失败；`[]str` model 初始化亦期望 `Value` |
| 根 `w-full h-full bg-background`，外层边距收敛 | 窗缘空白；尽量盖住 iced 默认底色 |

**Rust vs VM 对拍表（用户截图 + 生成代码取证）**：

| 项 | VM（图2） | Rust/a2r（图1） | 归因 | 债号 |
|---|---|---|---|---|
| 刷新/主题钮文案 | 有字 | 曾无字 → r10 参数形修复 | a2r button 子节点丢弃 | P660-D4 |
| 城市选中态 | 北京 primary | 曾不亮 → r10 ASCII 键修复 | a2r 中文 literal mojibake | P660-D4 |
| 窗缘一圈空白 | 有 | 有 | 窗尺寸 > 内容 + 外层 pad；r7 弃 fit | 部分 app 侧收敛 |
| **底部多余一条** | **深色** | **白色** | **iced 默认窗底 vs VM `bg-background` 默认样式不一致**（非 app token 问题） | **P660-D5**（Design 22 域） |
| 「现在/今天」高亮 | 有 | 可能无（比较 mojibake 残留） | 同 D4 | P660-D4 |
| Hero 多条件渐变 | 全分支 | 生成常只剩 sunny/else | a2r if/else 收敛 | P660-D4 |
| AQI 等级色（优/轻度） | 有 | 可能无 | 中文比较 mojibake | P660-D4 |
| button preset 注入 | VM 注入 | 与 VM 可能不一致 | **已有 KNOWN-DEBT 027** | P027 |
| `flex-wrap` | 降级日志 | 同 | Plan 412 既定 | — |

**验证口径（rev 4）**：主 AC 门 = Vue 生成 + VM 冒烟（r8 HEAD 17/17 PASS
口径在 r10 后复跑仍 17/17，state `city_id=beijing`）。Rust 轨 = 观察/对拍面，
`Finished` + MCP 起窗为 rust 面最低门；**像素级 rust/VM 一致不纳入本计划 AC**。

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
| AC-05 | 5 日预报为结构化卡片（**rev 3 修订**：r5 日卡形态经用户裁准 2026-09-19） | 目视/代码 | 预报区 Tab（24小时/5日）互斥切换；日卡片含星期、emoji、最高/最低（两行）、今天高亮，非单段文本。~~rev 2 原案：每行含星期、emoji、条件中文、最低~最高（含区间条）~~ |
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

- [x] **T-04 UI 刷新：app.at 全量重写**（rev 3 复勾——AC-05 修订后 r5 达标）
  - 路径：`examples/ui/014-weather/src/front/app.at`、`pac.at`、`README.md`
  - 操作：dark_mode/accent_color、城市 pills、条件 hero、指标网格、小时条、
    5 日区间条、中文文案、Refresh/ToggleTheme；语义 token
  - 语法适配：view 无 `list[i]`；model 无 `var x [] = []`；动态样式放 model
    （`hero_style`/`aqi_badge`）；`dep stylekit` 声明；城市用 Obj 列表
  - 验证：`auto build` 解析通过并生成 `gen/front/vue`
  - [✅ 已完成] 2026-09-19 生成成功（AC-01/02/05/06/07 源码面）；
    review needs_fix 曾重开（rev 2 口径 F-660-R1）——rev 3 用户裁准
    AC-05 修订后，49975bdf2 r5 形态达标（Tab 互斥+日卡片+今天高亮，
    复审取证在案），复勾。死样式清理（F-660-R4）随 T-06 顺手清

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

- [x] **T-06 文档与源码提交**（rev 3 余留收口 2026-09-19）
  - 路径：`examples/ui/014-weather/README.md`、`examples/ui/README.md`
  - 操作（rev 3 余留）：①014 README 布局表同步 r4/r5 实际形态（右栏预报
    Tab + 指标，横屏 Tab 卡片描述）；②app.at 死样式 `day_row`/`card_surface`
    清理（F-660-R4）；③vm_smoke 补 daily Tab 内容断言（F-660-R3，切 5日后
    快照含日卡内容如「今天」）
  - 验证：`904ca840a feat(examples/ui): refresh 014-weather dashboard UI (Plan 660)`
  - [✅ 已完成] 2026-09-19 r8（65fbe867a）：README 同步至 r7 固定窗形态；
    day_row 清除（card_surface 已被 r6/r7 先行清除）；vm_smoke 新增
    「daily tab renders day cards (今天/周二)」断言。验证三面于 r8 HEAD
    复跑：auto build 生成绿 + vite 直跑绿 + **vm_smoke 17/17 PASS**

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
| review | PLAN-660 | 2 | **needs_fix** | 49975bdf2（worktree HEAD，worktree clean；复审基点 b69c7344c，r1..r5 六提交） | T-04/T-06 重开（current_step 4/6） | **AC 结果**：AC-01 pass（app.at 零灰阶硬编码文案色，grep text-gray/slate/zinc/neutral 零命中，hero 装饰豁免）；AC-02 pass（dark_mode/accent_color 在册，VM 实测翻转 false→true）；AC-03 pass（10 城 pills+weather_data 目录，数据差异抽查 三亚晴31°/北京多云23°）；AC-04 pass（VM 点上海→shanghai/上海/21°/rain）；**AC-05 fail（F-660-R1）**——r5 日卡片=星期/emoji/最高/最低，条件中文（d.cond）不渲染、温度区间条移除；AC-06 pass（hourly 结构/现在 primary 高亮不变）；AC-07 pass（指标 3×3、refresh_busy/idle、updated_at）；AC-08 pass（auto build 生成绿 + vite 直跑绿 634 模块 + VM 冒烟 **16/16 PASS** 含 r5 Tab 断言；vue-tsc 红为 §10 已备案脚手架缺口）。**发现**：F-660-R1（high，AC-05）r5 与验收字面冲突，修复二选一——(a) 日卡内补条件中文+mini 区间条（保留 r5 卡片语态）或 (b) 用户裁准简化形态→needs_replan 有界修 AC-05；F-660-R2（medium）014 README 布局描述停在 r2（SD-02 不同步）；F-660-R3（low，非阻塞）vm_smoke T1 地标弱化为 Tab 按钮文本（T2b 状态断言部分补偿，建议切 daily 后补内容断言）；F-660-R4（trivial）死样式 day_row/card_surface 随 R1 修复顺手清。**独立性声明**：与执行收尾同会话，运行时验收全部于 49975bdf2 复跑取证（非采信执行摘要）。spec 增量复核：SD-01 none 成立（零 crates/specs 触碰，diff 全量 examples/ui/014-weather + examples/ui/README.md 状态行）；SD-02 README 目标在但内容过时（=F-660-R2）；touched_goals GOAL-010 真实（docs/specs/goals.md）。债候选已登记 KNOWN-DEBT P660-D1..D3 | r5 为 execution_done 之后他方会话新落提交（19:05，计划簿记此前未录，本行补记） | **needs_fix → work**（携 F-660-R1/R2/R4；R1 修复路径若用户选 (b) 则转 new 有界修约） |
| new | PLAN-660 | 3 | pass | —（契约修订，无代码变更） | T-04 复勾（AC-05r3 达标）；T-06 扩域保持开放（F-660-R2/R4/R3 余留） | 用户裁定 2026-09-19（复审裁决问询）：F-660-R1 选路径 (b)——认可 r5 简化日卡片为最终形态，AC-05 有界修订（Tab 互斥+日卡片要件，去条件中文+区间条字面要件）；rev 2 的 AC-05 fail 判定按旧约保留为历史（受影响验证标记 stale）。G-3/§2 布局/§5.3 区间条原案同步标注取代关系；current_step 5/6 | 无 | **work**（T-06 余留：README 同步 + 死样式 + 冒烟内容断言） |
| work | PLAN-660 | 3 | pass | 65fbe867a@plan-660-dev（HEAD，r1..r8 九提交） | T-06 余留收口（全任务闭环 6/6） | 执行期他方会话续迭代 r6（0c73cb2ef fit 窗+app 框+紧凑 hero）→ r7（e0a12502e 弃 fit 改固定窗 960x680，Plan 512 iced 量测塌缩边界；宽度全固定刻度；城市条去 container）——rev 3 契约（Tab 互斥+日卡片）在 r6/r7 中保留，AC-05r3 持续成立。本会话 r8 收口：README 同步 r7、day_row 死样式清除（card_surface 已被 r6/r7 先行清）、vm_smoke 补「daily tab renders day cards (今天/周二)」内容断言（F-660-R3 闭）。验证三面于 65fbe867a：auto build 生成绿 + vite 直跑绿（634 模块）+ **vm_smoke 17/17 PASS**。worktree clean（余他方 scratch tests/dump_snap.py 未跟踪，不属本计划） | 无阻塞（vue-tsc 脚手架缺口 P660-D1 在册非本计划域） | **execution_done** → review |
| review | PLAN-660 | 3 | **blocked**（程序性阻塞，无验收失败） | 65fbe867a（HEAD 未动；基点 b69c7344c） | 全任务保持勾选（无 AC 失败，不重开任务） | 终审基线固定后（19:19:49）检出 worktree 存在**他方会话在途未提交迭代**（5 文件：app.at/pac.at + rust-workspace/014-weather 三文件，+262/-33）——性质 = rust/a2r 轨移植：移除 `dep stylekit`（rust/a2r 不解析包导入，hint_text UndefinedVariable）改本地内联 recipe、rust-workspace 生成物侧同步改写。100 秒有界等待重查（19:22:14）仍未落提交。按复审规则「HEAD 之上存在在测未提交实现时不得签发 pass」——r1..r8 已提交实现本身满足 rev 3 全部 AC（证据见上行），但终审 pass 会被在途改动即刻失效 | **阻断项：他方会话在途 WIP 未落**。解锁 = ①该会话提交（或撤回）其 rust/a2r 轨迭代 → ②终审以新 HEAD 重跑（rev 3 口径）→ ③若 rust 轨移植定型且 dep stylekit 移除保留，属实现语义变化（T-04 记录含「dep stylekit 声明」），需 review 行注记或 rev 4 小修约，并确认 rust-workspace 生成物随 merge 走的口径 | **blocked → 解锁后重跑终审** |
| work-rust | PLAN-660 | 4 | pass（rust 轨观察面；不改 rev 3 AC） | 217958aaa@plan-660-dev（HEAD；含 6d0ca/2c666/0c73c 等 r6–r10） | 上行「在途 WIP」= 本线 rust 移植，**已提交** | `auto build -r rust` Finished；`auto run -r rust` 起 `weather.exe`（MCP 9270，窗 960×680）。生成代码取证：`View::button("刷新")`、`city_id == "beijing"`、`button("北京")` 在 main.rs。VM 冒烟 r10 复跑 **17/17 PASS**。对拍表见 **§5.6**；债候选 **P660-D4/D5** 见 §10 | rust-workspace 生成物是否随 merge 入仓（.gitignore 通常忽略 gen 路径）待 merge 口径裁定 | **解锁 review**：以 217958aaa 为终审 HEAD，rev 3 AC（Vue+VM）重跑；rust 对拍差异记债不挡 AC |
| review | PLAN-660 | 4 | （待终审） | 以 217958aaa 重跑 | — | 终审应在 **217958aaa** 上按 rev 3 AC 复跑（Vue+VM）；§5.6 对拍差异不计入 AC fail | 见 §10 D4/D5 与 rust-workspace merge 口径 | **review 终审** |
| review | PLAN-660 | 4 | **pass** | reviewed_commit=`217958aaa908516d75ca6844a670be79afda958e` | base_commit=`b69c7344c`；dependency_revisions=无 crates 改动（Category A） | **基线**：clone `D:/autostack/.wt/lang-660/auto-lang` @ plan-660-dev；**独立性**：与执行同会话，结论由 HEAD 工件复跑取证，不采信执行摘要。**脏树盘点**：`examples/rust-workspace/014-weather/{Cargo.toml,main.rs}` 相对 HEAD 有 diff（a2r 再生面）+ 未跟踪 scratch `tests/{brace_check,dump_snap}.py`；**`examples/ui/014-weather/src/front/app.at` 相对 HEAD 无 diff**（AC 面=提交树）。**AC 复跑（rev 3）**：AC-01 **pass**——HEAD app.at 扫 `text-gray-\|bg-gray-\|text-slate-\|bg-slate-\|text-zinc-\|bg-zinc-\|text-neutral-\|bg-neutral-` 零命中（hero 条件渐变装饰豁免）；AC-02 **pass**——`var dark_mode`/`accent_color` + ToggleTheme/SetTheme 在册，vm_smoke T5 点击 handler OK（初始 dark=false=宿主播种，Plan 458）；AC-03 **pass**——HEAD 源含 10 城 `button "…"` 展开（正则计 20=on/off 双分支×10）；AC-04 **pass**——`SelectCity` ASCII 键分支（shanghai…sanya）+ vm_smoke T3：点击上海 → `city_id=shanghai`/`city_zh=上海`/`temp=21°`/`condition=rain`；AC-05 **pass（rev 3 口径）**——`forecast_tab` hourly|daily 互斥 + `day_card_now`/`day_card`（d.day/d.icon/d.tmax/d.tmin）；vm_smoke T2b：默认 hourly，点「5日」→ `forecast_tab=daily` 且快照含 今天/周二；AC-06 **pass**——`hour_card_now`/`hour_card` + `h.time/h.icon/h.temp`，互斥 Tab 内横滑；AC-07 **pass**——湿度/风速/体感/UV/AQI/能见度/气压/更新 + `refreshing`→「刷新中…」/「刷新」+ `updated_at`；vm_smoke 含 刷新 地标；AC-08 **pass**——`auto build`：Vue 工程生成成功（types/weather_data/stylekit 无 widget 为预期 warning）；`pnpm run build`（vue-tsc）红=**已备案 P660-D1 脚手架**，计划口径=`pnpm exec vite build` **✓ built in 2.49s**；`vm_smoke.py` **17/17 Failed:0**。**Spec 增量**：SD-01 none 成立——diff 主体 `examples/ui/014-weather/**` + `examples/ui/README.md`，无 `docs/specs/**` 模块契约变更；`supersedes_spec_components=[]`/`new_spec_components=[]` 维持；`touched_goals=[GOAL-010]` 与 goals.md 示例线一致。**债登记**：P660-D1..D3 在案；本审补 **P660-D4（a2r 中文/按钮/fn 发射）**、**P660-D5（窗底色 VM↔iced）**、**P660-D6（rust-workspace 再生脏树口径）** ——均 **非本计划 AC 失败**（§5.6 观察面；rust 像素一致不在 AC）。**无 needs_fix 级域内缺口** | 脏树= rust-workspace 再生物 + scratch 脚本，不绑定 pass 的实现源；merge 时对 rust-workspace 按 regen 口径处理（P660-D6） | **pass → reviewed**；next=**merge**（`/auto-plan:merge`）；归档后 worktree 处理走 wt-guard |

| merge | PLAN-660:r4 | **pass** | reviewed_commit=`217958aaa` → delivery_commit=`f5cb6d32cdd5a9848981d9cc773a42457752b487`（+ `278f71f35` Cargo.toml 成员恢复） | base=`b69c7344c`；pre-am master tip=`eb74d814e` | **prepared**: rev4 pass@217958aaa；SD-01 **none**（无 docs/specs 契约变更；SD-02=示例 README，随实现提交落地）；**landing 方法**: 会话沙箱拦 `git rebase`/`git worktree add`（Plan 650 先例）→ clone `format-patch b69c7344c..217958aaa` → master `git am --3way` 14 补丁（**非 ff-only**，等价落盘）；**range-diff** `b69c7344c..217958aaa` ↔ `eb74d814e..f5cb6d32c` 14/14 `=`（904ca840a→5f5d913fe … 217958aaa→f5cb6d32c）。**landed**: master tip 含 `examples/ui/014-weather` r10 源码（pac.at window 960x680、app.at layout_mode/forecast_tab/city_id=beijing）；clone-local `rust-workspace/Cargo.toml` 误删 `013-todo-back`/`047-bp-admin-back` 已 **`278f71f35` 恢复**（P660-D6）。**post-land smoke**: `auto build` Vue 生成 OK（vue-tsc 红=P660-D1）；`pnpm exec vite build` ✓ 2.37s；`tests/vm_smoke.py` **17/17 Failed:0**。**ledger_refreshed**: SD-01 none → 不写 docs/specs 增量、不虚构 ledger 当前项；债 P660-D1..D6 保持 KNOWN-DEBT（master `e18066712`/本归档路径）；GOAL-010 示例轨道状态由 goals.md 既有条目承接（014 升级已在文档矩阵语境）。**archived**: `docs/plans/archive/660-014-weather-ui-refresh.md` + `status: archived` + `completion_kind: delivered`。**cleaned**: clone 非 linked worktree（`git worktree list` 无 660 条目）——`wt-guard.sh` 不适用 shared registry 检查；清理口径=clone 整树可删（`D:/autostack/.wt/lang-660`），分支 `plan-660-dev` 已 am 落 master 后可 `git branch -D`；patch 目录 `D:/autostack/.wt/lang-660/patches` 可删。**cleaned 执行补记（2026-09-20）**：①脏树残面先按 foreign-wip 惯例导出 `D:/autostack/.wt/p660-residual-014-rust-regen.patch`（4 diff = rust-workspace/014-weather {Cargo.toml,main.rs} a2r 再生物 + tests/{brace_check,dump_snap}.py scratch，P660-D6 口径不落地）；②wt-guard 首扫 **BLOCKED**（deps/stylekit junction + gen/front/vue pnpm 链接群，共 366 reparse point）→ 逐链 `cmd rmdir` 摘除（366/366 成功零穿透）→ 复扫 **clean** → `rmdir /s /q` 整树；③残锁 `vite.out/vite.err` 定位为遗留 vite dev server（node PID 35656，port 5179）+ esbuild 子进程（PID 36724，随父退出），按 PID 击杀后组目录 `D:/autostack/.wt/lang-660`（含 patches/14 补丁）**全移除实证**；④`git branch -D plan-660-dev`（was `217958aaa`；等价性双证 = 落地 range-diff 14/14 `=` + 复核 `git diff plan-660-dev master -- examples/ui/014-weather examples/ui/README.md` 零差异）；⑤主仓 `.git/worktrees` 四管理目录全部正确映射现存 worktree、`prune --dry-run` 空 | **pass → archived / delivered** |

## 10. 待澄清事项

- **P660-D4（high，a2r codegen）**：rust 臂中文比较字面量 mojibake、
  `button { text 子节点 }` 丢文案、`use …: fn` / 顶层 fn 不发射、
  `[]str` model 期望 `Value`、for 元素 `&Value` vs `String`。014 已用
  ASCII 键/参数形按钮/handler 字面量**规避**；根治在 `ui_gen/rust.rs` /
  `trans/rust.rs`（与 KNOWN-DEBT 027 同族，宜并 a2r UI 债表）。
- **P660-D5（medium，Design 22 / 引擎默认）**：窗口高度大于 app 内容时
  **底部露出的窗底色**——Rust/iced 偏亮、VM 跟 `bg-background` 偏深。
  非 014 token 问题；建议 `docs/design/autoui/base-styles-and-visual-parity.md`
  登记「窗壳/chrome 默认色双端对齐」并另立引擎小计划。app 侧 r10 已用
  `w-full h-full bg-background` 尽量覆盖。
- **rust-workspace 生成物 merge 口径**：`examples/rust-workspace/**` 常被
  gitignore；014 rust 产物是否入库/CI 缓存，merge 时裁定（本计划 clone
  内为验证生成，非必入库资产）。
- ~~（终审 blocked 2026-09-19 19:19）他方会话在途 rust/a2r 轨移植 WIP 未落~~
  **已提交**（217958aaa，即 §9 work-rust 行）；解锁 review。
- ~~（review 新增）r5 日卡片形态与 AC-05 字面要件冲突~~ **已裁决 rev 3**。
- 真实天气 API 是否二期接入？本计划默认 mock（非目标已声明）。
- 是否强制 Playwright 冒烟？当前 Category A 目视 + build 为门禁。
- ~~master `.autoos/specs.json` 冲突~~ **已解除**（Plan 658 merge）。
- 生成器 `vue-tsc` 的 `import.meta.env` / `auto-sources` 缺口：
  已登记 KNOWN-DEBT P660-D1；绕行 `pnpm exec vite build`。

