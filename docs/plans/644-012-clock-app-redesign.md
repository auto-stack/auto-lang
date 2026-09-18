---
plan_id: PLAN-644
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 012-clock-app-redesign
author: [Antigravity]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 6
---

# [PLAN-644] 012-clock-app-redesign

## 0. 变更摘要

将 `examples/ui/012-stopwatch` 升级并重命名为 `examples/ui/012-clock`（Clock 现代时钟应用）。参考 iOS Clock、Material You 与 OneUI 时钟等现代主流移动端/桌面时钟的视觉规范与交互架构，对该 App 进行深度重构。
提供完整的五大功能模块：
1. **时钟（Clock，首页入口）**：呈现高精度传统手表 ⌚ SVG 指针表盘与数字时钟，动态显示当前日期、星期与本地时区；
2. **世界时钟（World Clock）**：卡片式展示全球主要城市时间、时差对比（与本地时差、昨日/今日/明日）、昼夜图标（☀️/🌙）；
3. **闹钟（Alarm）**：卡片式闹钟列表、开关状态切换、新建/修改步进器、多槽位持久化（`storage`）；
4. **秒表（Stopwatch）**：大字号走表与毫秒分段、醒目的胶囊按钮（开始/停止/计圈/复位）、计圈记录列表及最快/最慢圈速高亮标识；
5. **倒计时（Timer）**：常用预设快捷胶囊（1m/5m/10m/15m/25m/30m）、精细步进调节、到期通知横幅与友好消除。

同时，升级桌面小组件面板的 `view mini` 命名视图，添加传统指针手表表盘与数字时间组合，并在全应用深度适配 AutoUI Design Tokens 与系统深浅色主题，彻底消除 P642-D6（空横幅常驻"知道了"按钮）遗留债务。

---

## 1. 目标

- **G-1 应用重命名与工程元数据升级**：目录由 `examples/ui/012-stopwatch` 迁至 `examples/ui/012-clock`，更新 `pac.at`（name: "clock"，title: "Clock"，title_zh: "时钟"，icon: "clock"），同步更新主注册表 `app_registry.rs`、品牌色映射 `renderer.rs`、样式清单与示例说明文档。
- **G-2 UI/UX 现代视觉重塑与主题适配**：摒弃原有简陋粗粝的旧式排版，采用类似 iOS / OneUI 的现代圆角卡片、毛玻璃/双层背景感知、微交互态与标准设计变量（`bg-background`, `text-foreground`, `bg-card`, `border-border`, `bg-primary`, `bg-secondary` 等），深色深沉典雅、浅色清爽纯净，消除 P642-D6 遗留横幅按钮常驻泄漏。
- **G-3 传统手表表盘（⌚ Analog Clock Dial）**：以首屏时钟 Tab 作为入口，利用 AutoUI 原生支持的 SVG 矢量子树（`svg`, `circle`, `line`, `rect`），绘制精致的传统指针表盘（外圈轮廓、12 小时刻度与分钟细刻度、时/分/秒三针联动、中心轴帽与秒针红色/橙色点缀），指针根据系统真实时间角度平滑转动，并提供指针/数字双模切换或并存展示。
- **G-4 五大核心时钟业务功能完备化**：时钟、世界时钟、闹钟、秒表、倒计时五大功能 Tab 全面就绪，满足日常使用与演示需求。
- **G-5 桌面小组件（`view mini`）传统表盘升级**：在桌面 Dashboard 面板中，时钟小组件从单一数字文本升级为包含迷你传统指针表盘 + 本地时钟的精致卡片。
- **G-6 自动化测试覆盖与双轨保障**：更新 MCP 测试用例 `desktop_mcp.py`，确保自动化冒烟、Tab 切换、秒表计圈、倒计时提醒、世界时钟与闹钟持久化等 100% 自动验证通过。

---

## 2. 架构方案

### 2.1 整体组件与文件布局
- **工程目录**：`examples/ui/012-clock/`
  - `pac.at`: 应用定义（name: "clock", title: "Clock", title_zh: "时钟", icon: "clock", render: "vue", window: "fit", desktop: "true"）
  - `README.md`: 应用说明与功能文档
  - `src/front/app.at`: 核心应用界面与交互逻辑（包含主视图 `view` 与桌面小组件命名视图 `view mini`）
  - `tests/desktop_mcp.py`: 基于 AutoUI MCP Server 的真实自动化回归测试脚本
  - `tests/t3_smoke.py`: 集成冒烟测试脚本

### 2.2 表盘几何与时间驱动架构
- 真实系统时间从 Auto 标准库 `Time.now_sec()` 获取 Unix 时间戳（秒级），结合应用内部 250ms 的 `.Tick` 周期进行细分推进（秒针转动平滑）。
- 角度换算逻辑：
  - 本地当前时分秒：
    - `hours = (now_sec / 3600 + offset) % 24`
    - `minutes = (now_sec / 60) % 60`
    - `seconds = now_sec % 60`
  - 传统指针表盘（以中心点 `(120, 120)`，半径 `R=100` 为基准）：
    - 秒针角度：`sec_deg = (seconds * 6) % 360`
    - 分针角度：`min_deg = (minutes * 6 + seconds / 10) % 360`
    - 时针角度：`hour_deg = ((hours % 12) * 30 + minutes / 2) % 360`
  - SVG 图元装配：
    - 刻度圈：`circle` + 12 个主刻度 `line`
    - 时针：粗短线，圆角 `stroke-linecap: round`
    - 分针：中长线，清晰指引
    - 秒针：高对比主题色（如橙色/红色），带延伸尾段与中心圆环
    - 旋转利用 SVG 标准属性 `transform: f"rotate(${deg} 120 120)"`

### 2.3 桌面小组件（`view mini`）升级
- 针对桌面 Dashboard（第四 overlay 槽 3 列网格卡片）：
  - 左侧或上方：迷你 70x70 传统手表 SVG 表盘，指针随秒针心跳跳动；
  - 右侧或下方：清晰的 `HH:MM:SS` 粗体数字时钟 + 日期（如 "09月18日 周五"）；
  - 极佳地结合了手表的优雅质感与数显的时效便利。

### 2.4 主题系统与 Design Tokens 规范
- 严格遵循 AutoUI 与 shadcn 语义 token：
  - 外壳与背景：`bg-background text-foreground`
  - 卡片底板：`bg-card text-card-foreground border border-border/50 rounded-2xl shadow-sm`
  - 导航胶囊：激活态 `bg-primary text-primary-foreground`，闲置态 `text-muted-foreground hover:bg-accent`
  - 操作按钮：主要行动 `bg-primary text-primary-foreground`，次要行动 `bg-secondary text-secondary-foreground`，危险/停止 `bg-destructive text-destructive-foreground`
  - 状态横幅：条件渲染，在 `banner != ""` 时才展示，消除空横幅留白与按钮游离问题。

---

## 3. 技术栈

- 语言与 DSL：Auto 语言 (`.at`) UI 语法（`widget`, `model`, `view`, `on`, `style`）
- 界面规范：AutoUI（编译至 Vue 3 + Tailwind CSS，并适配 Iced VM 渲染管道）
- 矢量绘制：SVG 规范（`svg`, `circle`, `line`, `rect`, `path`, `text`）
- 本地持久化：AutoUI `storage` 键值驱动（`storage.get`, `storage.set`）
- 系统接口：`Time.now_sec()` 标准时间 API
- 自动化测试：Python AutoUI MCP 驱动（`desktop_mcp.py`）

---

## 4. 需求分析与背景调查

- **背景梳理**：
  原 `012-stopwatch` 是 AutoUI 早期为了验证走表逻辑而创建的简单示例。在 os-003（Plan 554）期间，仓内为了验证桌面多 tab 功能，临时原地追加了计时器、世界时钟和闹钟，但当时仅做了骨架打通，存在如下硬伤：
  1. 目录名仍为 `012-stopwatch`，应用名和功能脱节；
  2. 样式陈旧、粗糙、无现代移动端时钟的精致感；
  3. 首屏直接落在秒表，而非符合大众心智的"时钟表盘"；
  4. 只有丑陋的文本数字时钟，缺少传统手表 ⌚ 指针表盘；
  5. 桌面小组件 `view mini` 只有一行单调居中文本；
  6. 存在已登记的已知债务 `P642-D6`（横幅消除按钮永久暴露）。
- **用户授权与范围**：
  用户指示：调研分析设计，用 `/auto-plan-new` 建立计划，再用 `/auto-plan-work` 实施。工作范围为 `auto-lang` 单仓内 `examples/ui/012-stopwatch` 的迁移与重构，以及引用该路径的注册表/渲染器/测试同步。

---

## 5. 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | 理由 | 关联验收项 |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-lang/ui/overview.md` | 策展示例 `012-stopwatch` 升级并重命名为 `012-clock` | 示例功能名实相符，反映完整的 Clock 应用能力 | AC-01 |
| SD-02 | modify | `docs/specs/auto-lang/ui/plans.md` | 记录 012-clock 重构与传统手表表盘支持 | 记录 Clock 应用架构与设计升级 | AC-02, AC-04 |

### 5.1 数据模型（`model`）定义
- 导航状态：`tab str = "clock"`（默认为时钟首页，支持 `"clock" | "world" | "alarm" | "stopwatch" | "timer"`）
- 表盘与时钟模型：
  - `now_sec int`: 当前时间戳
  - `h_deg str`, `m_deg str`, `s_deg str`: 指针旋转角度字符串
  - `local_time_str str`: 本地时间 `HH:MM:SS`
  - `local_date_str str`: 本地日期 `YYYY年M月D日 星期W`
  - `dial_style str = "analog"`: 表盘模式（`"analog"` / `"digital"` / `"both"`）
- 世界时钟模型：
  - 8 大主要城市及固定时区偏移（北京 +8、东京 +9、悉尼 +11、伦敦 +0、巴黎 +1、莫斯科 +3、纽约 -5、洛杉矶 -8）
  - 各城市卡片：城市名、本地时差（如 "今天 -8小时"）、昼夜指示（☀️ 白天 / 🌙 夜间）、格式化时间
- 闹钟模型：
  - 5 槽定长持久化（`clock.alarms.0..4`），支持开闭状态切换与时间设定、当日已响标记防重响
- 秒表模型：
  - `sw_on str = "false"`, `elapsed int`, `time_display str`, `ms_display str`
  - 计圈列表：`laps []str`（记录 Lap 序号、分段圈速与累计时间，并标识极值）
- 倒计时模型：
  - 设定值 `t_set_h`, `t_set_m`, `t_set_s`；剩余毫秒 `t_left`；运行开关 `t_on`
  - 快捷预设：1m, 5m, 10m, 15m, 25m, 30m 一键填装
- 横幅提示：
  - `banner str`: 闹钟响铃或倒计时结束时弹出，支持点击"知道了"优雅清除，空时不占位。

### 5.2 界面视觉结构
- **顶部顶栏**：
  - 左侧标题 "时钟 Clock" + 优雅小徽章；
  - 胶囊 Tab 切换栏（时钟、世界时钟、闹钟、秒表、倒计时），带有顺滑的激活态高亮。
- **时钟（Clock）主屏**：
  - 居中展示大号 SVG 传统手表表盘（直观感受机械表转动之美）；
  - 表盘下方为清晰的大号数显时钟、当前日期与星期；
  - 底部提供表盘视图切换按钮（指针 / 数字 / 双显）。
- **世界时钟（World Clock）**：
  - 网格或双列卡片，每张卡片具有优雅微阴影与圆角，呈现城市、昼夜、时差与大字号时间。
- **闹钟（Alarm）**：
  - 列表卡片展示已有闹钟，带醒目的开关态按钮与删除钮；
  - 底部带快速新增面板（现代步进器与预设按钮）。
- **秒表（Stopwatch）**：
  - 居中经典超大数字码表；
  - 底部双大圆钮（开始/停止/计圈/复位）；
  - 下方滚动式计圈记录卡片。
- **倒计时（Timer）**：
  - 环形倒计时进度轮廓 + 居中大倒计数字；
  - 预设时间胶囊栏；
  - 操控按钮（开始/暂停/取消）。
- **桌面小组件（`view mini`）**：
  - 紧凑并排：左侧迷你传统手表表盘（⌚），右侧数字时间 + 本地时区/日期。

---

## 6. 测试设计

### 6.1 自动化测试
1. **示例与注册表门禁**：
   - 运行 `cargo test -p auto-lang scan_examples_ui_curation_set`，断言 `012-clock` 成功被桌面策展注册表扫描识别。
2. **代码生成与编译检查**：
   - 在 `examples/ui/012-clock` 目录下执行 `auto build --gen-only` 或 `cargo check -p auto-lang`，断言 0 警告 0 报错。
3. **真实 MCP 驱动交互测试**：
   - 运行 `python examples/ui/012-clock/tests/desktop_mcp.py`：
     - 测试 1：Tab 切换到首页时钟，断言 SVG 表盘指针与时间文本正确渲染；
     - 测试 2：秒表启动、走表时间推进、计圈记录累加、复位清零；
     - 测试 3：倒计时设定与触发，断言到期横幅弹出与消除；
     - 测试 4：世界时钟多城市时间渲染与时差校验；
     - 测试 5：闹钟新增、开闭切换与跨重启持久化恢复。

### 6.2 手动与视觉走查
- 双端（Vue 与 VM）启动走查：
  - `auto run`（Vue 模式）
  - `auto run -r vm`（VM 模式）
  - 检查深浅色主题切换时的表盘反色表现、文字对比度与卡片视觉层次。

---

## 7. 验收标准

- **AC-01 命名与工程规范**：目录为 `examples/ui/012-clock`，`pac.at` name 为 `clock`，相关注册表及配置引用全集无缝更新，`scan_examples_ui_curation_set` 绿。
- **AC-02 传统手表表盘渲染**：时钟首页具备高精度 SVG 传统指针表盘，时针、分针、秒针随真实时间转动，刻度与表盘布局优雅美观。
- **AC-03 五大功能模块完整可用**：时钟、世界时钟、闹钟、秒表、倒计时五大 Tab 均可交互且视觉协调一致。
- **AC-04 桌面小组件升级**：`view mini` 成功渲染传统手表表盘 + 数字时钟，在桌面 Dashboard 中视觉效果优秀。
- **AC-05 债务与缺陷修复**：修复 P642-D6 横幅按钮泄漏问题，空状态下无多余残影按钮。
- **AC-06 测试门禁全绿**：`desktop_mcp.py` 全部断言通过，无 regression。

---

## 8. 执行步骤

- [ ] **T-01 计划入库与独立 Worktree 建立**：在 master 提交本计划与 `.next-id`，基于 Plan 529 布局创建 `D:/autostack/.wt/lang-644/auto-lang` 工作树。
- [ ] **T-02 目录迁移与元数据同步**：在 worktree 内执行 `git mv examples/ui/012-stopwatch examples/ui/012-clock`，更新 `pac.at`、`crates/auto-lang/src/ui/app_registry.rs`、`crates/auto-lang/src/ui/iced/renderer.rs`、`scripts/style_palette_manifest.json` 与 `examples/ui/README.md`。
- [ ] **T-03 传统手表表盘与时钟首页实现**：在 `012-clock/src/front/app.at` 实现 SVG 矢量手表表盘（外圈、刻度、时/分/秒针平滑换算）与日期数字显式。
- [ ] **T-04 现代 UI/UX 与五大功能重构**：实现现代胶囊 Tab、优化世界时钟卡片、闹钟列表与持久化、秒表计圈极值高亮、倒计时快捷预设与到期提示，全面落地 Design Tokens。
- [ ] **T-05 桌面小组件 `view mini` 升级**：在 `app.at` 中重塑 `view mini`，集成传统指针手表表盘与紧凑数字时间。
- [ ] **T-06 自动化测试适配与门禁验证**：升级并运行 `tests/desktop_mcp.py`，运行注册表单元测试与语法检查，完成双端验证。

---

## 9. 复审记录

- stage: new | plan_id: PLAN-644 | plan_revision: 1 | outcome: pass | next: work

---

## 10. 待澄清事项

- （无。需求明确，范围清晰。）

