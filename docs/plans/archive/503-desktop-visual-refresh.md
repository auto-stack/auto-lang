---
plan_id: PLAN-503
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: 桌面视觉体系刷新——stella-os 风格移植(dock/launcher/chrome/壁纸罩层)
author: [zcode]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/architecture.md: 主题/accent ADR 区——coral 预设校准至 stella 玫瑰粉 + 桌面视觉降格条款(无 blur/scale,keyframes 不进)"
  - "docs/specs/auto-lang/ui/overview.md: 桌面线条目增补 plan-503 视觉体系段(链接 design/desktop-shell.md)"
new_spec_components:
  - "docs/specs/auto-lang/ui/design/desktop-shell.md: 桌面视觉 token——accent 玫瑰粉/dock 图标格/弹层 glass 三件套/壁纸 scrim/窗口 chrome(TITLEBAR 36/radius 16/三色圆点)/launcher 品牌色图标底块(bg-[#hex21] 8 位 hex 双端形式)"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致——桌面视觉双端同源(stella 风格移植,降格 parity)"

affects: [crates/auto-lang/src/ui, crates/auto-lang/assets, crates/auto-man/assets/wm, examples/ui/028-launcher, docs/specs/auto-lang/ui]
current_step: 7
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

- [✅ 已完成] M1 accent 与 token 校准(theme.rs + vue index.css 变量;双端截图验证)
  - coral 预设校准至 stella 玫瑰粉 #c4706a = HSL(4,43%,59%)(dark +10 → 69% ≈ #d4847e);不新增预设。
  三处同源副本同步:`ui/style/theme.rs:86`、`ui/code_editor/theme.rs:93`、
  `ui_gen/vue.rs` ACCENT_PALETTE_JS。暗色 background 色温按计划倾向不动。
  `cargo check -p auto-lang`(default+ui-iced)绿;实机截图验证归 M6。
- [✅ 已完成] M2 shell dock 刷新(shell.at + Taskbar.vue)
  - 图标格 h-12 w-12 rounded-xl + hover:bg-primary/10;激活窗竖条(w-0.5 h-5
  bg-primary rounded-full div)+ 图标格 bg-primary/15;运行指示 4px 圆点
  (bg-primary,替换 "●" text hack);任务栏 h-12→h-14(48px 格留 4px 呼吸)、
  gap-2;launcher 钮 h-10 w-10 rounded-xl。top/bottom 两份分支同步。
  Taskbar.vue 同款(56px 底栏/rounded-xl/激活 bg-primary/15 + 竖条 span/去描边)。
  新增守卫测试 `shell_packs_compile`(五 pack 真管线编译)绿。
- [✅ 已完成] M3 右键菜单/弹层 glass 化 + 壁纸罩层(desktop.at 等 + renderer.rs 罩层)
  - desktop.at 右键菜单 bg-card/80 border rounded-xl shadow-xl + 菜单项
  rounded-lg hover:bg-primary/10;notification_center/switcher 去全部 gray 硬编码
  → 语义 token + glass 三件套;switcher 选中行 bg-blue-900 → bg-primary/15 + text-primary。
  renderer.rs 新增 `desktop_wallpaper_scrim`(图片壁纸上叠 bg-background 语义色
  scrim:light 10%/dark 35%,无 blur 降格 parity);纯色分支不叠(底色无对比噪音,
  且不透明桌面面会盖住罩层)。vue 轨 desktop host 无图片壁纸层(纯色
  bg-background),scrim 无锚点不叠——与 VM 轨纯色分支同判定(记 P2 壁纸 parity)。
- [✅ 已完成] M4 launcher 重写(028 app.at,品牌色映射)
  - 去 gray-800/900 全部硬编码:面板 bg-card/80 border rounded-2xl shadow-2xl、
  scrim bg-background/60、搜索框 bg-background/50、选中 bg-primary/15 text-primary。
  新增分类胶囊行(cats 12 枚,PickCat 过滤三分支:recent/rest/scored 全过分类门)。
  品牌色图标底块:每 app 6 位 hex → `bg-[<color>21]`(13% alpha)+ `text-[<color>]` 字形;
  标记用 8 位 hex 而非 `/13` 修饰符(Tailwind v3 JIT 对非刻度 alpha 修饰符不生成
  CSS,双端同源取 8 位 hex);块半径 rounded-xl(VM 无 rounded-[10px] arbitrary 档)。
  宿主注入第 7 平行列表 apps_colors(`launcher_brand_color`:已知映射 + id 哈希
  粉彩兜底)。**引擎能力补齐(TDD)**:style 串循环成员插值 `${r.field}` 双端皆缺
  ——VM `resolve_literal_interpolation_with` + vue `interpolated_class_parts` 各扩
  循环成员形态;parser 补 `bg-[#hex]/N` 组合(此前静默丢弃)。plan503_tests 6 项全绿
  (插值双端/launcher 编译+品牌块渲染+PickCat 过滤)。
- [✅ 已完成] M5 窗口 chrome 刷新(virtual_window.rs + VirtualWindow.vue;按钮组与动词核实)
  - TITLEBAR_H 28→36(native 槽位换算同源常量自动跟随);窗体 radius 8→16
  (WIN_RADIUS);柔影 (0,8)/32px——light 12%/dark 40%,focused 加深(+8/+12);
  focused 描边 2px accent → 1px accent/60;最大化(窗矩形 ≥98% 桌面几何判定,
  无 maximized 标志的近似)去圆角去影。按钮组裁定=macOS 三色圆点 12px
  (待澄清①按倾向落):red #ff5f57=Close(动词既有);yellow/green=视觉位预留
  (待澄清②核实:WmCommand 无虚拟窗 min/max 动词,仅 native 槽位有)→
  P503-1 KNOWN-DEBT 挂账。VirtualWindow.vue 同款(h-9/rounded-2xl/shadow-xl→
  focused shadow-2xl/border-primary/60/三色圆点组)。
  测试:desktop_surface_z_slot 装配测试(过新 chrome 真管线)绿。
- [✅ 已完成] M6 双端验证(autoui-verifier + golden 基线)
  - **VM 轨实机**(`ui_desktop --fullscreen --apps-dir examples/ui`,
  `AUTO_UI_ACCENT=coral`,截图+视觉分析目检):玫瑰粉 accent 全局生效(无蓝紫残留)、
  三色圆点+1px accent/60 焦点描边+16px 圆角+柔影、dock 48px rounded-xl 图标格、
  右键菜单 glass 三件套(半透明+细边+大圆角)、launcher(搜索卡/胶囊行/品牌色
  块/首行玫瑰底/scrim 压暗)逐项过;**图片壁纸分支**:shell.desktop.wallpaper
  指向库内 png 启动,壁纸+罩层(压暗、图标可读)生效。
  - **vue 轨实机**(`auto run --desktop` + AUTO_DESKTOP_APPS,vite :3001 截图目检):
  Taskbar 56px 无描边 rounded-xl/聚焦窗玫瑰底/布局钮无边框、VirtualWindow
  三色圆点+大圆角+柔影+玫瑰焦点、launcher 同款(胶囊 'all' 玫瑰选中确认)。
  - **golden**:gallery_golden 更新(3 行:app.at=本计划 palette;kitchen-sink=
  **master 存量红**——基线 a5ad3fe01 早于 Plan 497 kitchen-sink 再生成
  f6c5387f1,gallery_golden 不在 cargo t 日常档漏检;dumpA/B 对照实验证实与本
  计划无关)。"459-dual-app/p051 golden"核查:仓内无此二名的 golden 测试面
  (p051 grep 零命中),gallery 即唯一 golden 门禁。
- [✅ 已完成] M7 复审与归档准备
  - 收尾局部验证:auto-man cargo check 绿;plan503(6)+shell_packs(1)+
  desktop_surface 装配(1)全绿;worktree 两提交(M1-M5 实现 + golden 基线)。
  复审本体(/auto-plan:review)与归档(/auto-plan:merge)按范式移交。

## 复审记录

**复审人/时间**: zcode(auto-plan:review),2026-08-31。复审基 = worktree `plan-503-dev`
@71e01fd3d(3 提交),diff 对 a406173c2 逐文件核对;验证全部在 worktree 内重跑。

### 验收标准逐项

| # | 标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | accent 玫瑰粉双端同源(CLI + 设置面板) | PASS | 三处同源副本(theme.rs:86/code_editor:93/ui_gen palette JS)一致 #c4706a=HSL(4,43,59);CLI 入口实机 vm 全局玫瑰+vue 实机确认;"设置面板"= app 级 accent_color 面(015/gallery SetAccent)走同源 palette;desktop settings.at 无 accent 面(计划 affects 未含,非遗漏) |
| 2 | dock 图标格/竖条/圆点双端一致 | PASS | shell.at+Taskbar.vue diff 核对;vm/vue 实机截图目检(48px rounded-xl 格/激活竖条+bg-primary/15/4px 运行圆点) |
| 3 | 右键菜单/弹层 glass + 壁纸罩层 | PASS | 实机:菜单半透明+细边+大圆角;图片壁纸 scrim 压暗生效;纯色分支不叠(设计判定,见实施附注) |
| 4 | launcher 去硬编码+胶囊+品牌色底块 | PASS | 单测 launcher_compiles_and_renders_brand_chips/pickcat_filters;vm+vue 实机(品牌色块/胶囊 'all' 玫瑰选中/首行玫瑰底);gray-800/900 grep 零残留 |
| 5 | 窗口 chrome 双端一致+最大化去圆角 | PASS | vm 实机(36px 栏/16px 圆/柔影/三色圆点/1px accent/60);vue 实机同款;最大化=VM 几何判定(≥98% 桌面),vue store 无最大化态(随 P503-1 注记) |
| 6 | parity 条款(无 blur/scale/keyframes) | PASS | diff 全文 grep:仅注释提及"无 blur"与 iced 阴影 blur_radius 24→32(非 backdrop-blur);零 scale-/keyframes 引入 |
| 7 | 门禁绿 | PASS | **worktree cargo tf 3330/3330 全绿**(金样同步后);cargo tv 2 红/cargo tt 21 红——与 master 同参运行**失败清单逐一相同**(a2r `as u32` 族+cookbook 族=master 存量,非本计划回归);gallery golden+a2vue desktop 金样两基线更新(理由见提交) |

### 遗漏/延后/workaround 扫描

- **遗漏**: 无——计划 6 目标均有对应 diff 落点;新增能力(循环成员插值/`bg-[#hex]/N`)有 TDD 测试钉死(plan503_tests 6 项)。
- **延后(经批准形态)**: min/max 动词视觉位预留(P503-1)、vue 壁纸层(P503-2)——均为计划原文预设路径 + KNOWN-DEBT 挂账,非静默缩水。
- **Workaround/偏差(记录不隐藏)**:
  1. `bg-[#xxx]/13` → 8 位 hex `bg-[#xxx21]`:Tailwind v3 JIT 不生成非刻度 alpha 修饰符,8 位 hex 双端同源;VM parser 缺口已补(超出字面但属 parity 条款兑现)。
  2. `rounded-[10px]` → `rounded-xl`:VM 无 arbitrary 半径档(静默丢弃),12px 双端等价。
  3. 计划 M1 字面"index.css --primary 变量段同步"未执行:index.css 默认 --primary=indigo 是**默认 accent** 的正确值,coral 校准经 palette JS/CLI bootstrap(theme.rs 单源)生效——计划措辞与实现偏差,功能无缺口。
  4. 任务栏 48px 底栏与 48px 图标格不可同储 → 栏高 56px(已在实施附注记录)。
- **master 存量红上报(非本计划门禁)**: tt 21 红(a2r 转换器 `as u32` 强转发射回归)+tv 2 红(cookbook async/devtools)+kitchen-sink golden 失配(本分支已顺带修复)——建议用户另立排查。

### 结论

**PASS**——7/7 验收标准过,无阻断债;路由 `reviewed`,可走 /auto-plan:merge。

## 待澄清事项

1. **按钮组风格**: macOS 三色圆点 vs 传统符号按钮(─ ▢ ×)——M5 开工时定,倾向三色
   圆点(stella 视觉签名),但需评估暗色主题下可读性;
   **→ 已裁定(M5)**: 三色圆点 12px(#ff5f57/#febc2e/#28c840),实机暗色目检可读良好;
2. **min/max 窗口动词**: session 现有 API 面需 M5 开工核实(close 必有;min/max 若无,
   视觉位预留 + KNOWN-DEBT 挂账);
   **→ 已核实(M5)**: WmCommand 无虚拟窗 min/max(仅 native 槽位 NativeSlotMin/Close);
   yellow/green 落视觉位,P503-1 KNOWN-DEBT 挂账;
3. **亮色主题默认值**: stella 亮色出彩,但 theme.rs 默认 dark=true——本计划不动默认值,
   仅保证亮色美观可用;默认翻转与否留用户决策;
4. **widget 面板**(stella 顶部 dashboard 卡片组): 我们 shell 无此概念,本计划不做,
   归后续计划评估。

## 实施附注(执行期决策记录)

- **品牌色类串形态**: 计划原文 `bg-[#xxx]/13`——实施改用 8 位 hex `bg-[#xxx21]`:
  Tailwind v3 JIT 对 arbitrary 色 + 非刻度 `/13` 修饰符不生成 CSS(vue 轨会静默丢),
  8 位 hex 双端同源;VM parser 侧 `/N` 组合缺口已顺带补齐(防呆,测试钉死)。
- **图标底块半径**: `rounded-[10px]` VM 侧无 arbitrary 半径档(静默丢弃)→ 双端等价
  `rounded-xl`(12px)。
- **任务栏高度**: 图标格 48px(h-12)与"48px 底栏"不可同储,取栏高 56px(h-14)
  留 4px 呼吸(贴近 stella 68px 侧栏比例)。
- **最大化判定**: VWinState 无 maximized 标志,以"窗矩形 ≥98% 桌面几何"近似
  (去圆角去影);vue 轨 store 无对应态,v1 不做(差异随 P503-1 min/max 动词一并清)。
- **引擎能力补齐(M4 前置)**: style 串循环成员插值 `${r.field}` 双端皆缺,
  VM/vue 各扩一臂 + plan503_tests 钉死;新守卫 shell_packs_compile(五 pack
  真管线编译)+ launcher 编译/PickCat 行为冒烟。
