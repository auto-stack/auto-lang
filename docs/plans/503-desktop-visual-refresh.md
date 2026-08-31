---
plan_id: PLAN-503
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 桌面视觉体系刷新——stella-os 风格移植(dock/launcher/chrome/壁纸罩层)
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/desktop-shell.md: 视觉 token 与窗口 chrome 节(归位时定名)"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——桌面视觉双端同源"

affects: [crates/auto-lang/src/ui, crates/auto-lang/assets, crates/auto-man/assets/wm, examples/ui/028-launcher, docs/specs/auto-lang/ui]
current_step: 0
total_steps: 7
---

# [PLAN-503] 桌面视觉体系刷新——stella-os 风格移植(dock/launcher/chrome/壁纸罩层)

## 变更摘要

参照 stella-os web mock(D:\Down\stella-os\os-simulator\index.html,Caelestia 风:暖米白/
深蓝紫双主题 + 玫瑰粉 accent + 大圆角柔影 + 类毛玻璃)刷新 AutoOS 虚拟桌面视觉。
截图调研存档 `scratch/stella-shots/`(01–06)。

实机渲染修正的关键判断:**毛玻璃实际观感接近不透明**(window-bg 75~85% alpha 叠壁纸后
blur 几乎不可见),视觉主体 = 色温 + accent + 大圆角 + 柔影 + 间距。因此本计划**不做真
backdrop-blur**(iced 无此概念),统一按"高 alpha 底色 + 细边框 + 柔影"双端降格实现,
parity 风险低。scale 变换/弹性缓动/keyframes 不进本计划。

范围 = P0(token/accent + shell dock + 右键菜单 + 壁纸罩层 + launcher 重写)
+ P1(窗口 chrome 刷新)。后续 P2(token 体系化/blur parity 决策)/P3(动效)另行立项。

## 目标

1. **accent 玫瑰粉**: `theme.rs` ACCENT_PRESETS 校准/新增至 stella `#c4706a` 色系
   (light)/`#d4847e`(dark),双端(vue CSS 变量 + VM 语义色)同源生效;
2. **shell dock 刷新**: 图标格 48px + `rounded-xl` + 激活态 accent-light 底/竖条指示 +
   运行中圆点(替换 `"●"` text hack),hover 底色变化(无 scale);
3. **右键菜单/弹层 glass 化**: `bg-card/80` + 细边框 + `rounded-xl` + 柔影,菜单项
   hover `bg-primary/10`;
4. **壁纸罩层**: 壁纸层上叠 `bg-background/10`(dark `/35`)罩层,提升桌面可读性;
5. **028-launcher 重写**: 去 gray-800/900 硬编码,改语义 token;搜索框 + 分类胶囊
   (选中 = `bg-primary/15 text-primary`)+ 40px 品牌色 13% alpha 图标底块
   (`bg-[#xxx]/13` arbitrary alpha 已支持);
6. **窗口 chrome 刷新**: 标题栏 28→36px、窗体圆角 8→16px、阴影 (0,8)/24px/45% →
   (0,8)/32px/12% 柔化、按钮组(关闭已有;min/max 动词按 session 现状核实接入,
   无则视觉位预留并挂账 KNOWN-DEBT)、最大化去圆角;vue 轨 `VirtualWindow.vue`/
   `Taskbar.vue` 同步同款。

## 架构方案

分层改动,按门禁分级:

| 层 | 落点 | 改动性质 |
|---|---|---|
| token/accent | `ui/style/theme.rs`(ACCENT_PRESETS/语义色表) + vue index.css 变量段(`auto-man/src/vue.rs:1052-1094`) | Rust,小 |
| shell/dock/右键菜单/设置面板 | `crates/auto-lang/assets/*.at`(shell.at/desktop.at/notification_center.at/switcher.at) | **纯 .at 类串** |
| 壁纸罩层 | `iced/renderer.rs` `view_desktop_fn`(:11967 壁纸层上叠罩层)/ vue 轨 desktop host 同位 | Rust + vue 模板,小 |
| launcher | `examples/ui/028-launcher/src/front/app.at` | **纯 .at** |
| 窗口 chrome | `ui/iced/virtual_window.rs`(TITLEBAR_H/radius/shadow/按钮组) + `auto-man/assets/wm/VirtualWindow.vue`/`Taskbar.vue` | Rust + vue 模板 |

**parity 条款(硬约束)**: 不引入 backdrop-blur 类;stella 的玻璃感一律翻译为
"alpha 底色 + 边框 + 阴影"三件套,双端同一组值;新增样式只用既有 style utility
(alpha `/N`、arbitrary `bg-[#hex]`、rounded/shadow/hover: 均已支持)。

stella token 对照表(设计输入):

```
accent        #c4706a / #d4847e(玫瑰粉)   → ACCENT_PRESETS
bg-glass      白45% / 暗30,30,46@55%      → bg-card/80 等价
选中态        accent-light 底 + accent-dark 字 → bg-primary/15 + text-primary
radius        18px(面板/窗口) 12px(卡片)   → rounded-2xl / rounded-xl
shadow        0 8px 32px 10%黑(dark 40%)   → shadow-xl / chrome 常量
hover         bg 5%黑 / 8%白               → hover:bg-primary/10 或 hover:bg-foreground/5
品牌色图标底   color + 13% alpha           → bg-[#xxx]/13
```

## 需求分析与背景调查

- 调研产物: stella-os 设计 token 清单与组件范式(2026-08-31 会话分析,
  截图 `scratch/stella-shots/01-06`);
- 现状(已核实): 窗口 chrome 硬编码于 `virtual_window.rs:30-34`(TITLEBAR_H=28/
  radius=8/shadow 45% 黑);dock=shell.at 朴素 row + `"●"` 文本指示;launcher 硬编码
  gray-800/900;壁纸无罩层(`renderer.rs:11967` 层序);样式系统无 blur、无 scale,
  有 alpha 修饰符/arbitrary 值/渐变/hover: 前缀;
- 主题机制已有: thread-local DARK_MODE + ACCENT_PRESETS 五预设(theme.rs:83-98),
  `--theme/--accent` CLI 与设置面板双入口;
- 双端 chrome 分叉: VM 轨 Rust 宿主绘制、vue 轨 `assets/wm/*.vue` 手写模板——
  两端各改一处,金样对拍纪律。

## 详细设计

### M1 accent 与 token 校准
- `theme.rs:83-98`: ACCENT_PRESETS 的 coral 校准至 #c4706a/#d4847e(或新增 rose 预设,
  实施时核对 coral 现值后定);暗色 background 色温对照 stella #1a1a2e 微调评估
  (现值 9,14,26 已同族,倾向不动);
- vue 轨 index.css `--primary` 变量段同步(auto-man/src/vue.rs:1052-1094);
- 验证: `--accent coral` 双端截图。

### M2 shell dock 刷新
- `assets/shell.at`: dock 图标格 `h-12 w-12 rounded-xl`,激活态 `bg-primary/15` +
  左侧竖条指示(2px `bg-primary`),运行指示圆点改 4px `bg-primary rounded-full`
  (替换 `"●"` text);任务栏高度/间距对齐 stella 68px 侧栏或 48px 底栏(dock 位置
  storage 配置已有,`shell.dock.position`);
- vue 轨 `Taskbar.vue` 同步。

### M3 右键菜单/弹层 + 壁纸罩层
- `assets/desktop.at`: 右键菜单 `bg-card/80 border rounded-xl shadow-xl`,菜单项
  `rounded-lg hover:bg-primary/10`;notification_center/switcher 同族梳理;
- 壁纸罩层: `renderer.rs:11967` 壁纸层上叠 `bg-background/10`(dark `/35`)矩形层;
  vue 轨 desktop host 同位叠加。

### M4 launcher 重写
- `examples/ui/028-launcher/src/front/app.at`: 面板 `bg-card/80 rounded-2xl shadow-2xl
  border`,搜索框 `bg-background/50`,分类胶囊 `rounded-full` 选中 `bg-primary/15
  text-primary`,应用项图标 `h-10 w-10 rounded-[10px] bg-[<app.color>]/13`;
- 品牌色数据: pac/注册表 app 条目增 color 字段(若无则 M4 内定义静态映射表)。

### M5 窗口 chrome 刷新
- `virtual_window.rs`: TITLEBAR_H 28→36;窗体 radius 8→16;shadow → (0,8)/32px/12%
  (dark 40%);focused 描边从 2px accent 改为 1px accent/60 + 阴影加深;最大化时
  radius 0;
- 按钮组: 三色圆点风(12px,#ff5f57/#febc2e/#28c840)或符号按钮组——开工时定
  (待澄清 1);min/max 动词核实 session 现有 API,无则预留视觉位 + KNOWN-DEBT 挂账;
- `VirtualWindow.vue` 同步同款(h-9 titlebar、rounded-2xl、按钮组、focused 态)。

### M6 双端验证
- vm 桌面(`auto run -r vm` desktop 模式)+ vue desktop-host(examples/desktop-host)
  双端截图目检;调用 autoui-verifier 技能;gallery/桌面 golden 基线更新。

### M7 复审与归档准备

## 测试设计

- 门禁分级: M1/M3(罩层)/M5 含 Rust 改动 → Category B(`cargo check -p auto-lang` +
  `cargo t ui` 或 `cargo t iced`);M2/M4 纯 .at → 目检 + golden;token 色值改动需核对
  schema_drift 不涉(style 系统非 schema 围栏范围,实施时确认);
- 双端一致性: 桌面双端截图对照(vm/vue),accent/dock/chrome/launcher 逐项;
- 回归: 桌面相关 golden(459-dual-app、p051 等)基线更新。

## 验收标准

1. accent 玫瑰粉双端同源生效(CLI `--accent` 与设置面板两入口);
2. dock 图标格/激活竖条/运行圆点新样式双端目检一致;
3. 右键菜单与弹层 glass 化(alpha 三件套),壁纸罩层生效;
4. launcher 去硬编码暗色,语义 token + 分类胶囊 + 品牌色图标底;
5. 窗口 chrome: 36px 标题栏/16px 圆角/柔影/按钮组,双端一致;最大化去圆角;
6. parity 条款遵守: 无 backdrop-blur/scale/keyframes 引入;
7. 门禁绿(cargo t ui/iced + 桌面 golden)。

## 执行步骤

- [ ] M1 accent 与 token 校准(theme.rs + vue index.css 变量;双端截图验证)
- [ ] M2 shell dock 刷新(shell.at + Taskbar.vue)
- [ ] M3 右键菜单/弹层 glass 化 + 壁纸罩层(desktop.at 等 + renderer.rs 罩层)
- [ ] M4 launcher 重写(028 app.at,品牌色映射)
- [ ] M5 窗口 chrome 刷新(virtual_window.rs + VirtualWindow.vue;按钮组与动词核实)
- [ ] M6 双端验证(autoui-verifier + golden 基线)
- [ ] M7 复审与归档准备

## 复审记录

（复审时填写）

## 待澄清事项

1. **按钮组风格**: macOS 三色圆点 vs 传统符号按钮(─ ▢ ×)——M5 开工时定,倾向三色
   圆点(stella 视觉签名),但需评估暗色主题下可读性;
2. **min/max 窗口动词**: session 现有 API 面需 M5 开工核实(close 必有;min/max 若无,
   视觉位预留 + KNOWN-DEBT 挂账);
3. **亮色主题默认值**: stella 亮色出彩,但 theme.rs 默认 dark=true——本计划不动默认值,
   仅保证亮色美观可用;默认翻转与否留用户决策;
4. **widget 面板**(stella 顶部 dashboard 卡片组): 我们 shell 无此概念,本计划不做,
   归后续计划评估。
