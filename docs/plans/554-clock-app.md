---
plan_id: PLAN-554
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: clock-app
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 7
---

# [PLAN-554] Clock——012-stopwatch 原地升级为四 tab 时钟应用

## 变更摘要

AutoOS 缺 Windows Clock 对标物。现状 `012-stopwatch` 是半成品：`.Tick`
消息存在但**无人发送**（走表不走，仅 Start/Stop/Lap 改显示）；025-dashboard
已实证 `.Tick` 自驱动机制（`interval` 模型变量被 UI 运行时取走作周期）。
本计划把 012 **原地升级**为 Clock 四 tab 应用：秒表（修复走表）/ 计时器 /
世界时钟 / 闹钟。**目录名与 registry id `012-stopwatch` 保持稳定**（recent
槽、pac name 引用不破坏），pac title 改 "Clock"。

依赖：PLAN-552 `desktop:` 字段（软依赖，未合入时为无害未知键）。

## 目标

1. **秒表**：真实走表（Tick 驱动，精度 10ms 档显示 MM:SS.cc）+ 计圈
   （定长 5 槽，028 先例）+ Reset。
2. **计时器**：HH/MM/SS 设定 → 倒数 → 到零横幅提醒（in-app banner v1）
   + 暂停/继续/取消。
3. **世界时钟**：≥8 城当前时刻（`Time.now_sec` + 城市偏移表换算，含
   北京/伦敦/纽约/东京/悉尼/巴黎/莫斯科/洛杉矶），当前城高亮本地大表。
4. **闹钟**：HH:MM 设定 + 闹钟列表（storage 定长 5 槽持久化）+ 到点
   in-app banner（v1 无声）。
5. pac：`title: "Clock"`、`icon: "clock"`、`category: "tool"`、
   `desktop: "true"`、`window: "fit"` 保持。

## 架构方案

- **Tick 契约**（025-dashboard 实证形态）：model 声明 `interval int = <ms>`
  + `.Tick` 消息 handler → UI 运行时按 `interval` 周期派发。本应用
  `interval = 250`（世界时钟秒级、闹钟分钟级比对、秒表 10ms 显示精度分频：
  `sub` 计数器 25 进位——025 `.subTick` 分频同款）。
  **T1 探针**先钉死契约细节（变量名/类型/取走条件，读 renderer `__tick`
  事件名与 AppTickRecipe 段 + 025 SPEC），结论写回本 plan。
- **时间源**：`Time.now_ms()/now_sec()`（`vm/ffi/stdlib.rs:644-669` 实存）。
  世界时钟/闹钟比对用 now_sec 换算；秒表 elapsed 用 Tick 累计（暂停语义
  简单，不受系统调时影响）。
- **单组件约束**：全状态内聚 App（store 子组件 vm 生成损坏先例 013/038）；
  换算/补零逻辑写状态法 handler（模块级 fn 不进 vue SFC，024/028 先例）。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-010**：Clock 为 2026-09-05 盘点第一梯队缺口；012 现状
  （`examples/ui/012-stopwatch/src/front/app.at`）：`.Tick -> { .elapsed += 10 }`
  存在但无派发方——升级顺带修复而非重写。
- **Tick 机制**：`examples/ui/025-dashboard/src/front/app.at:9` 头注
  「`interval` 模型变量被 .Tick 机制取走作 setInterval 周期（250ms 基准）」；
  运行时侧 `renderer.rs` `TICK_EVENT("__tick")` + `AppTickRecipe`（tokio
  interval，5871-5936 段）。vue 侧对应 setInterval 生成（025 双端实测绿）。
- **时间原生**：`Time.now_ms/now_sec/now`（stdlib.rs 644-669）。
- **storage 定长槽惯例**：028 `launcher.recent_apps.0..4`（值只做 `!= ""`
  比较——vue 产物 null 链 TS18047 教训）。
- **通知面**：shell toast/通知中心（Plan 479）经 `__desktop_cmd` 上行——
  App 侧可用动词词表待 T1 核；无则 in-app banner（col 覆盖层）v1。
- 画廊：012 已归 "02-components"（vue.rs 分类链 007–012 臂）——升级后
  形态变化不改分类（画廊分类按前缀稳定）。

## 详细设计

### model（要点）

```
var tab str = "stopwatch"       // stopwatch | timer | world | alarm
// 秒表
var running str = "false"  var elapsed int = 0        // ms 累计
var sub int = 0                 // 250ms→10ms 分频基数（×25 显示步进）
var time_display str / ms_display str / lap1..lap5 str
// 计时器
var t_set_h/m/s int = 0  var t_left int = 0   // 剩余秒
var t_running str = "false"  var t_done str = "0"     // 到零横幅门控
// 世界时钟
var cities = ["Beijing","London",...]          // 平行列表（B12 规避）
var offsets = [8, 0, -5, 9, 11, 1, 3, -8]      // UTC 偏小时
var now str = ""  var rows = []                // handler 自建 {city,time}
var local_city int = 0
// 闹钟
var a_h str = "07"  var a_m str = "30"
var alarms = ["","","","",""]  var fired str = ""        // 当日已触发槽标记
var banner str = ""            // 通用覆盖横幅文案（空=隐藏）
// Tick
var interval int = 250
```

### handler（要点）

- `.Tick`：分频→各 tab 更新（秒表 elapsed+=250 且显示步进 cc；timer
  t_left 递减到 0 触发 banner；world 每 4 tick 重算行；alarm 每分钟比对
  now 换算 HH:MM ∈ alarms 且未 fired → banner + fired 标记）。
- `.StartStop/.Lap/.Reset`（秒表，012 现逻辑保留 + 接 Tick）；
- `.TStart/.TPause/.TCancel`（计时器）；
- `.SetLocal(i)`（世界时钟高亮切换）；
- `.AAdd/.ADel(i)`（闹钟槽 CRUD + storage `clock.alarms.0..4` 持久化）；
- `.DismissBanner`（横幅关闭，fired 保持防重复触发）。

### view（要点）

顶部 tab 四胶囊（选中 `bg-primary/15`）；各 tab 内容卡片；banner =
覆盖层 col（`bg-background/95` + 文案 + Dismiss 按钮）。

## 测试设计

`tests/desktop_mcp.py`（012 目录，011/013 惯例，双端）：

1. 秒表 Start→等 ~1s→显示前进（MM:SS.cc 变化断言）；Lap 记圈；Reset 清零。
2. 计时器设 00:00:01→Start→到零 banner 出现 + Dismiss 可关。
3. 世界时钟 ≥8 行渲染；北京/伦敦时差断言（now_sec 基准算期望值，容差
   ±60s 换算）。
4. 闹钟设当前下一分钟→等触发 banner；storage 重开恢复列表。

（等待类断言用 mcp 轮询超时 10s，038 timer 先例。）

## 验收标准

1. 双端四 tab 全功能可用；秒表真实走表（修复原 012 不走表缺陷）。
2. `desktop_mcp.py` 双轨全绿。
3. 闹钟 storage 持久化跨重启恢复。
4. pac title "Clock"/icon "clock" 生效（boot 窗标题与图标格）。
5. registry id `012-stopwatch` 不变（recent 槽/dock pinned 无破坏）。
6. `examples/ui/README.md` 012 行更新（Clock 四 tab）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 Tick 契约探针**
  读 `crates/auto-lang/src/ui/iced/renderer.rs` `TICK_EVENT`/`AppTickRecipe`
  段与 vue 侧 setInterval 生成（ui_gen）+ 025 SPEC「Tick 机制」节；确认
  `interval` 变量取走条件与 vue/vm 双端行为；顺带核对 `__desktop_cmd`
  动词表有无 notify。结论写回本节。
  验证：探针笔记（scratch/p554/）
- [ ] **T2 骨架 tab 化**
  `examples/ui/012-stopwatch/src/front/app.at`：model 增 tab/新状态族；view
  改四 tab 胶囊 + 占位内容；原秒表面板迁入 stopwatch tab。
  验证：`cd examples/ui/012-stopwatch && auto build`
- [ ] **T3 秒表接 Tick**
  `.Tick` 分频驱动 elapsed + 显示；Lap 扩 5 槽。
  验证：`auto run` 手测走表/暂停/计圈/复位
- [ ] **T4 计时器**
  设定三 input + Start/Pause/Cancel + 到零 banner。
  验证：`auto run` 倒数 1s 冒烟
- [ ] **T5 世界时钟**
  城市表 + `Time.now_sec` 换算 handler + 行渲染 + 本地城高亮。
  验证：`auto run` 与系统时钟肉眼对拍（北京/纽约）
- [ ] **T6 闹钟 + storage**
  槽 CRUD + storage 持久化 + Tick 分钟比对触发 banner。
  验证：`auto run` 设下一分钟闹钟等到触发
- [ ] **T7 测试与回写**
  `tests/desktop_mcp.py` 四断言组（双端）；pac.at 改 title/icon/category/
  desktop；README 012 行更新。
  验证：`python tests/desktop_mcp.py` 双轨绿 + `cargo check -p auto-lang`
  （若 T1 触及 ui_gen 则跑）

## 复审记录

## 待澄清事项

1. 闹钟到点走 shell toast/通知中心（`__desktop_cmd` 上行）还是 in-app
   banner——T1 动词表核对后定；v1 默认 banner（保守面）。
2. 世界时区夏令时：v1 固定偏移表（8 城多数无 DST 或影响 ±1h），SPEC
   登记限制；DST 规则表远期。
3. 秒表精度 10ms 档基于 250ms Tick 分频外推（elapsed+=250 实际是墙钟
   步长）——显示 cc 两位够用；如需真精度改 `Time.now_ms` 差值法（T1
   后裁定，倾向差值法：Start 记锚点，Stop 累计）。
