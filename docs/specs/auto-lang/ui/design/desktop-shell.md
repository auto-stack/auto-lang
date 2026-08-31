# 桌面视觉体系(stella 风格)

## 范围

AutoOS 虚拟桌面的视觉 token 体系:accent 主题、shell dock、弹层 glass、壁纸罩层、窗口 chrome、launcher。双端(VM/iced 与 vue desktop host)同源实现。来源 Plan 503(stella-os web mock 移植,2026-08-31)。

## parity 降格条款(硬约束)

不引入 backdrop-blur/scale 变换/keyframes。stella 的玻璃感一律翻译为
「alpha 底色 + 细边框 + 柔影」三件套,双端同一组值。样式只用既有 style
utility(alpha `/N`、arbitrary `bg-[#hex]`、rounded/shadow/hover:)。

## 视觉 token 表

| 项 | 值 | 落点 |
|---|---|---|
| accent 玫瑰粉 | `#c4706a` = HSL(4,43%,59%),dark L+10→69%(≈`#d4847e`) | coral 预设三处同源:`ui/style/theme.rs`、`ui/code_editor/theme.rs`、`ui_gen/vue.rs` ACCENT_PALETTE_JS |
| dock 图标格 | 48px `rounded-xl` + `hover:bg-primary/10`;激活 = `bg-primary/15` 底 + 2px(`w-0.5`)accent 竖条;运行指示 = 4px(`h-1 w-1`)accent 圆点 | `assets/shell.at`(top/bottom 双分支)+ `auto-man/assets/wm/Taskbar.vue`(56px 底栏) |
| 弹层 glass 三件套 | `bg-card/80 border rounded-xl shadow-xl`,菜单项 `rounded-lg hover:bg-primary/10` | `assets/desktop.at`(右键菜单)、`notification_center.at`、`switcher.at` |
| 壁纸罩层 scrim | 图片壁纸上叠 background 语义色:light 10% / dark 35%,无 blur | `renderer.rs::desktop_wallpaper_scrim`(纯色壁纸分支不叠;vue 轨无壁纸层——P503-2) |
| 窗口 chrome | TITLEBAR_H=36、radius=16、柔影 (0,8)/32px(light 12%/dark 40%,focused 加深)、focused 描边 1px accent/60、macOS 三色圆点 12px(red=Close;yellow/green=视觉位,P503-1)、最大化(窗矩形 ≥98% 桌面几何判定)去圆角去影 | `ui/iced/virtual_window.rs`(native 槽位换算同源常量自动跟随)+ `auto-man/assets/wm/VirtualWindow.vue` |
| launcher 品牌色 | 每 app 6 位 hex → 图标底块 `bg-[<color>21]`(13% alpha 8 位 hex)+ 字形 `text-[<color>]`;宿主注入 `apps_colors` 平行列表(`renderer.rs::launcher_brand_color`:已知映射 + id 哈希粉彩兜底) | `examples/ui/028-launcher/src/front/app.at` |

## 引擎能力(本计划补齐)

- **style 串循环成员插值 `${member.field}`**:`for r in .rows` 循环体内
  `style: "… ${r.chip}"` 在 VM(`aura_view_builder::resolve_literal_interpolation_with`)
  与 vue(`ui_gen/vue.rs::interpolated_class_parts`,v-for 作用域求值)双端解析。
  原仅有 `${.state}` 前导点形态。测试:`plan503_tests`(插值双端 + launcher 行为)。
- **`bg-[#hex]/N` 组合**:arbitrary hex 带 alpha 修饰符此前整体静默丢弃
  (class 不以 `]` 结尾绕过预提取 + from_hex 不剥方括号),`parse_color_with_alpha`
  已补。生产标记建议 8 位 hex(`bg-[#7c9a6d21]`)——Tailwind v3 JIT 不生成
  非刻度 alpha 修饰符,8 位 hex 双端同源。
- **shell pack 编译冒烟**:`ui/shell.rs::pack_tests`(五 pack 真管线)——
  include_str 内嵌源不进 cargo check,此前语法回归只能实机 boot 才暴露。

## 已知限制

- 虚拟窗 min/max 无 WM 动词,三色圆点黄/绿为视觉位(KNOWN-DEBT P503-1)。
- vue 轨桌面无图片壁纸层,scrim 无锚点(KNOWN-DEBT P503-2,归 P2 token 体系化)。
- 亮色主题保证可用,默认 dark 不翻转(用户决策)。
