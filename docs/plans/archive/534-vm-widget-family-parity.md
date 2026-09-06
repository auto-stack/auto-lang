---
plan_id: PLAN-534
status: archived              # 终态：drafting → executing → execution_done → reviewed → archived
feature_name: VM overlay 家族余量补齐（sheet/drawer/hovercard,双轨）
author: [zhaopuming, ZCode]
created_at: 2026-09-03
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/plans.md: 修改（533 vm-overlay-runtime-channel 行的 overlay 家族面扩展——余量三员 sheet/drawer/hovercard 落地,Popover 原语获 Edge 贴边族）"
  - "docs/specs/auto-lang/ui/overview.md: 修改（VM/iced 组件覆盖面——overlay 三员从 unknown fallback 转正,模态 chrome 判定单点化 is_modal_chrome）"
new_spec_components:
  - "docs/specs/auto-lang/ui: 新增组件 sheet/drawer/hovercard（Edge 贴边族四向几何+scrim/外点/Esc 三路关、drawer 贴缘圆角+装饰把手、hovercard MouseArea 真 hover+__dlg_enter/leave 铸造、rust codegen 轨 Edge 发射+MouseArea 包锚,双轨同串）"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致——VM overlay 家族余量三员双轨落地+实机交互全链验证（开/外点/ESC/hover 进出）,gallery 三页+全站 68 页扫描"
  - "GOAL-003: Auto 作 Rust 脚本层——编译轨（ui_gen/rust.rs）overlay 发射面扩三员（side_panel_emission/drawer_handle_emission/generate_hover_card_popover）,与解释器轨 chrome/placement 同串一致"

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 12
total_steps: 12
---

# [PLAN-534] VM overlay/表单组件族补齐（widgets-gallery 双端 parity）

## 变更摘要

**2026-09-06 二次重定界（本文全面改写为执行就绪版）**：2026-09-03 首次重定界
时认定的余量四员 `dialog/sheet/drawer/hovercard` 中，**dialog 已由 PLAN-533 T5
落地**（`aura_view_builder.rs:1394`（tracked）/`:2931`（untracked）双镜像臂
`ModalDialogFamily::Dialog` + `ui_gen/rust.rs:3281` 路由 + `dialog-close` 子臂
`:1442/:2977`）。本计划范围收敛为 **sheet / drawer / hovercard 三员**，并补上
首版立项时遗漏的 **Rust 代码gen 轨（`ui_gen/rust.rs`）**——533 先例表明每个
overlay 成员需"解释器臂×2（D-GAP 双镜像）+ rust.rs 发射"双轨落地。

**历史**：立项时承接 PLAN-528 W12/W13（toggle_group/alert-dialog）已由
PLAN-530 落地；dialog 由 PLAN-533 T5 落地。**前置依赖全部解除**。

## 目标

1. **sheet**：`side` prop（left/right/top/bottom,缺省 right）→ 贴边全高/全宽
   面板 + scrim + 面板外点击关闭（shadcn Sheet 语义,同 dialog 族 dismiss 行为）;
   trigger/open 属性双形态;SheetTrigger/Content/Header/Title/Description/Footer/
   Close 子臂全套。
2. **drawer**：`direction` prop（四向,缺省 right）→ 同 sheet 通道;bottom 方向
   chrome 差异化（rounded-t + 装饰拖拽把手,无手势——差异文档化）。
3. **hovercard**：**真 hover 触发**（parser 铸造 enter/leave 事件 + anchor 包
   View::MouseArea）,Bottom 锚定非模态（无 scrim）;open-delay/close-delay
   v1 不消费（即时开合,文档化）。
4. **双轨一致**：解释器臂（aura_view_builder.rs）与 Rust 代码gen 臂
   （ui_gen/rust.rs）同 placement/chrome/dismiss 语义。
5. **gallery parity**：/sheet /drawer /hovercard 三页 VM 观感对齐 vue 基线
   （截图+交互）;全站 gallery 页扫描回归 + `cargo t iced` + `cargo tv` 绿。

## 架构方案

底座全部就绪（530 W9/W13 + 533 T5/T6/T7 交付）,本计划为模式复制 + 两处
原语扩展。**每成员三件套**：

| 成员 | 解释器臂（tracked+untracked 镜像） | rust.rs 轨 | 原语扩展 |
|---|---|---|---|
| sheet | `convert_side_panel(side)` 新函数,复用三助手 | `generate_modal_popover` 家族扩 side→EdgeX | Edge 贴边 placement×4 |
| drawer | 同 sheet（direction prop） | 同上 | 同上（bottom 圆角 chrome） |
| hovercard | `convert_hovercard`（MouseArea 包 anchor） | 同族扩 hover 发射 | parser 铸造 enter/leave |

**可复用资产（全部 533 已验证）**：

- `PopoverPlacement::Modal`（view.rs:827 枚举;popover.rs:383 viewport 相对
  几何 + snap 排除:394;renderer.rs:4232 `.modal()` 判定 → scrim Quad + 面板
  外整吞,on_dismiss=Some 时外点发布关闭）。
- 铸造自管开合机器:parser.rs:13044 区 mint pass（state `__dlg_open_N` +
  toggle/close 事件体）;dismiss 折算 `minted_dismiss_msg`
  （aura_view_builder.rs:6407,`__dlg_open_N` → `__dlg_close_N`）。
- `alert_dialog_split_children`（:6430,trigger/content 拆解）、
  `bare_trigger_button_view`（:6370,裸文本 trigger→Button）、
  dialog 子标签 styled/passthrough 臂先例（:1394-1440 区）、`dialog-close`
  → outline 按钮先例（:1442）。
- mouse-area hover 通道（`convert_mouse_area`:9481/onmouseenter+onmouseleave
  → View::MouseArea,Plan 484 起实机验证链路活）。

## 需求分析与背景调查

（2026-09-06 逐条对码核实）

- **VM 现状**：sheet/drawer/hovercard 在 aura_view_builder.rs **零匹配臂**,
  落 unknown fallback（:1500 注释:渲染为丢 style 的 Column,类串整体丢弃）——
  即三页在 VM 端完全退化。dialog 族已收（见变更摘要）。
- **placement 缺口**：PopoverPlacement 现有 9 变体（Bottom 族 3 + Top 族 3 +
  Left/Right/Modal）,锚相对;**无 viewport 贴边变体**——sheet/drawer 需要新增。
- **gallery 消费面**（决定子标签/props 支持范围）：
  - `pages/sheet.at`：`sheet (side: "right")`;sheet-trigger（内含 button）/
    content/header/title/description;Properties 表列 side/open。
  - `pages/drawer.at`：`drawer`（direction,缺省 right）;drawer-trigger
    **裸文本** "Open Drawer"（走 bare_trigger_button_view）/content/header/
    title/description;表列 open/direction/title。
  - `pages/hovercard.at`：hover-card-trigger（内含 avatar 组合）/content
    （内含自有 col 布局）;表列 open-delay/close-delay/open。
- **测试落点先例**：集成=`plan536_t1_reactive_probe_tests.rs:709`
  （`p536_t8_child_widget_root_alert_dialog_resolves_open`）+ 语料
  `test/ui/plan536_modal/`;几何=`ui/iced/layout_tests.rs` + popover.rs 内联测试。
- **边界**：533 延后清单中 tooltip/select/combobox/command 不在本计划范围;
  Plan 535（desktop-ux-followups）的 wrap_layout_onclick hover 基建是布局件
  级公共项,与 hovercard 的 overlay 触发通道不冲突、不互赖。

## 详细设计

（2026-09-06 全部落定,不再保留"倾向"项;行号为当前 master 基点）

### D1 贴边 placement（Edge 族）

- `view.rs` PopoverPlacement 增 4 单元变体：`EdgeLeft/EdgeRight/EdgeTop/
  EdgeBottom`（沿用现有单元变体风格,不引入带数据变体）。注释标注 PLAN-534
  + 语义：viewport 贴边（与锚无关）+ scrim + 外点 dismiss。
- `popover.rs` `Panel::layout`（:329 区）几何,对齐 Modal 的 viewport 相对
  先例（:383）：
  - EdgeLeft/Right：`y=viewport.y`、`h=viewport.height`（全高）,宽=内容宽,
    x 贴 viewport 左/右缘;
  - EdgeTop/Bottom：`x=viewport.x`、`w=viewport.width`（全宽）,高=内容高,
    y 贴上/下缘。
  - **layout 上限**：Edge 族尺寸上限取 viewport（同 snap_within_viewport
    分支的 Limits 写法）,防内容超视口。
- snap 排除（:394）：`self.placement != PopoverPlacement::Modal` 改
  `!matches!(self.placement, Modal | EdgeLeft | EdgeRight | EdgeTop | EdgeBottom)`
  （贴边/居中即终位,不做翻转钳制）。
- `renderer.rs:4232` modal 判定（决定 scrim+整吞）改同一 `matches!`——
  **Edge 族挂 scrim + 外点整吞**;on_dismiss=Some 时外点发布关闭 =
  shadcn Sheet 语义（区别于 alert 族不关、同 dialog 族三路关）。

### D2 sheet/drawer 转换（同函数,side 参数化）

- 新增 `convert_side_panel(props, children, side: PopoverPlacement, chrome: &str)`
  （untracked）+ `convert_side_panel_tracked_ctx`（镜像,对齐
  `convert_alert_dialog_tracked_ctx`:6583 的 path 追踪结构）。不复用
  `ModalDialogFamily` 枚举（side 参数化后加变体会爆炸,独立函数更清晰）。
- 内部复用三助手：`alert_dialog_split_children`（扩标签族,见 D3）、
  `bare_trigger_button_view`（裸文本 trigger）、`minted_dismiss_msg`
  （sheet/drawer 为可关闭族,同 dialog:ESC/外点折算 `__dlg_close_N`;
  无铸造 open 绑定时返回 None=自管形态不接管,同现有语义）。
- 臂接线（两处,D-GAP 双镜像）：
  - `"sheet"|"Sheet"`：side prop → Edge 映射,缺省 `"right"` → EdgeRight;
  - `"drawer"|"Drawer"`：direction prop 同映射,缺省 `"right"`。
  - prop 解析：`AuraPropValue::Expr(Expr::Str)` 字面优先,非字面经
    `resolve_expr_to_value` 取 str;非法值落缺省并 debug 日志。
- chrome（与解释器臂、rust.rs 轨同串,保双轨视觉一致）：
  - sheet（四向同款,首版不做分向 border 差异）：
    `"bg-background border shadow-lg p-6 gap-4"`（宽度由内容/默认 w-96 量级
    决定,全高由 placement 几何保证,不进 chrome）;
  - drawer：同 sheet;direction=bottom（或 top）时追加
    `"rounded-t-lg"`（bottom）并**首子前插装饰把手**（居中 `w-8 h-1
    rounded-full bg-muted` bar 视图,纯视觉,无拖拽手势）。
- `View::Popover { anchor, content, placement: EdgeX, open, on_dismiss }`——
  与 dialog 族完全同构,open 属性直驱/铸造 toggle 双形态免费获得。

### D3 子标签族清单（split 扩展 + 兜底臂）

- `alert_dialog_split_children`（:6430）trigger/content 判定追加：
  `sheet-trigger|sheet_trigger|sheettrigger`、`drawer-trigger|drawer_trigger|
  drawertrigger`、`hover-card-trigger|hover_card_trigger|hovercard-trigger|
  hovercardtrigger` 及对应 `*-content`（对齐现有 kebab/snake/连写三形态）。
- 兜底 passthrough 臂（组外裸子件,镜像 :1407 区 dialog-trigger/content
  透传先例）：上述 trigger/content 全族。
- styled 子臂（镜像 :1413-1440 dialog 同名臂,仅换标签名）：
  `sheet/drawer-header`（flex flex-col gap-2）、`-title`（text-lg
  font-semibold）、`-description`（text-sm text-muted-foreground）、
  `-footer`（flex justify-end gap-2）;
  `sheet-close/drawer-close` 镜像 `dialog-close`（:1442 outline 按钮,onclick
  取铸造 close）。
- **不新增** hovercard 的 header/title 等子臂（gallery 未用,内容自带布局,
  content 整体走 passthrough 即可）。

### D4 hovercard（真 hover 触发）

- **parser 铸造扩展**（parser.rs:13044 mint pass）：hovercard 根（及
  hover-card-trigger）参与现有 `__dlg_open_N` state 铸造;trigger **无显式
  onclick** 时不再补 toggle,改补两条：
  - `onmouseenter: .__dlg_enter_N`（体：`.__dlg_open_N = true`）
  - `onmouseleave: .__dlg_leave_N`（体：`.__dlg_open_N = false`）
  用户显式绑定优先不覆盖（同现有 toggle 规则）。
- **解释器臂**（双镜像）：`convert_hovercard` ——split_children 取
  trigger/content;anchor 转 View 后包一层 `View::MouseArea`
  （对齐 convert_mouse_area:9481 的事件解析,enter/exit 接铸造消息;若
  trigger 已有自有 onclick 保留并存）;content 装 col 挂 chrome
  `"w-80 bg-popover border rounded-lg shadow-md p-4"`;placement=
  `Bottom`（锚相对,hover 卡悬于锚下,非模态无 scrim）;`on_dismiss=None`
  （无外点整吞;关闭只靠 leave,Esc 不接）。
- **降级路径（写明切换点）**：若 MouseArea×Popover 组合实测异常（如
  leave 事件被 overlay 拦截）,回退 click-toggle——parser 侧改为补
  `__dlg_toggle_N`（一行切换）,VM 臂去 MouseArea 包裹;差异记录 README。
- open-delay/close-delay：v1 不消费（VM 轨无定时器原语）,即时开合,
  README/KNOWN-DEBT 文档化。

### D5 Rust 代码gen 轨（ui_gen/rust.rs）

- 路由（:3281 `"alertdialog" | "dialog" | "dropdownmenu"`）追加
  `"sheet" | "drawer" | "hovercard"`。
- `generate_modal_popover`（:3356）扩两形态:
  - sheet/drawer：side/direction prop → EdgeX 发射（placement 枚举新变体
    的 Rust 构造串）,chrome 串与 D2 完全一致;on_dismiss 同铸造折算
    （若 rust.rs 侧尚未接 T6 回流,按 533 现状对齐——执行时以 rust.rs
    现有 dialog 发射为准镜像）;
  - hovercard：anchor 外包 mouse-area 发射（对齐 rust.rs 现有 mouse-area/
    MouseArea 发射先例,执行时定位）,placement=Bottom。
- 双轨一致性由单测断言 chrome/placement 同串保障（见测试设计）。

## 测试设计

（作用域:Category B 局部 VM/ui 改动;日常 `cargo t iced` 局部 + `cargo tv`
VM 语料档;**不触 aavm 路径,零 taa 触发**;fold 前裸 `cargo tf` 全量）

- **几何单测**（`ui/iced/layout_tests.rs` 或 popover.rs 内联,对齐 Modal
  断言先例）：Edge 四臂——全高/全宽/贴缘坐标断言;snap 排除不翻转断言。
- **转换单测**（aura_view_builder 测试区/探针模块,对齐 alert_dialog 断言）：
  - convert_side_panel：trigger/content 拆解、side 缺省 right、direction
    映射、chrome 串、on_dismiss 铸造折算、open 属性直驱;
  - convert_hovercard：MouseArea 包裹存在性、enter/exit 消息接线、
    placement=Bottom、非模态（无 scrim 判定）;
  - rust.rs 轨：sheet/drawer/hovercard 发射含同款 placement/chrome 串
    （双轨一致断言）;
  - parser：hovercard trigger 补 enter/leave 铸造、显式绑定不被覆盖。
- **集成语料**：新增 `test/ui/plan534_side_panels/src/front/app.at`
  （sheet 四向 + drawer 两向 + hovercard 三块,结构对齐 plan536_modal）;
  `plan536_t1_reactive_probe_tests.rs` 补断言（side→placement 解析、
  open 解析、hover enter/leave 状态翻转）。
- **双端验证**（autoui-verifier MCP 驱动）：
  - /sheet /drawer /hovercard 三页 VM 截图 vs vue 基线;
  - 交互:trigger 点击→贴边面板+scrim 出现;外点→关;ESC→关;
    hovercard 鼠标进入 anchor→卡出现,移出→消失;
  - 全站 gallery 页扫描回归（pages/ 现 68 个 .at 文件,执行时以实际
    计数为准）零新增异常。

## 验收标准

- [x] `/sheet` VM 页（复审 G1 ✅）：四向 side 各自贴边正确（left/right 全高、top/bottom
      全宽）、scrim 遮罩、trigger 点击开/外点关/ESC 关、open 属性直驱形态可用。
- [x] `/drawer` VM 页（复审 G2 ✅）：direction 映射正确;bottom 方向圆角+装饰把手渲染。
- [x] `/hovercard` VM 页（复审 G3 ✅,avatar 项=D4 债）：hover 进出即时开合（或降级 click 并已记录）;
      avatar 锚原位渲染;无 scrim 非模态。
- [x] 双轨一致（复审 G4 ✅）：rust.rs 发射与解释器臂同 placement/chrome（单测断言在案）。
- [x] 三页 VM 截图（复审 G5 ✅,vue 运行对照=D3 债）与 vue 基线观感对齐（截图落账）;全站扫描零新增异常。
- [x] `cargo t iced`（复审 G6 ✅,tf 3462/3463 唯一红 charts 存量） + `cargo tv` 全绿;无未说明 workaround;differences
      （delay 不消费/无拖拽手势）已在 KNOWN-DEBT 与 README 文档化。

## 执行步骤

（原子任务:精确文件路径 + 确切操作 + 验证命令;每步完成后追加
[✅ 已完成] 一行证据。行号为 2026-09-06 master 基点,执行时以 grep 重新定位。）

1. [✅ 已完成] view.rs PopoverPlacement 增 EdgeLeft/EdgeRight/EdgeTop/EdgeBottom 四单元变体+PLAN-534 语义注释；`cargo check -p auto-lang` 零错（worktree 需组内 auto-down 兄弟 worktree 供 path 依赖解析，已建 detached@master） **[PLAN-534 T1]**
2. [✅ 已完成] popover.rs Panel::layout 增 Edge 四臂几何（viewport 相对,L/R 全高 T/B 全宽）+ Limits 恒取 viewport 上限 + snap 排除;判定单点化为 `PopoverPlacement::is_modal_chrome()`（view.rs,即"同一 matches!"的共享实现）,renderer.rs 两处 .modal() 站点（:4232/:18283）同口径;`cargo t popover` 4/4 绿 **[PLAN-534 T2]**
3. [✅ 已完成] layout_tests.rs 补 Edge 四臂断言（iced 层直构 Popover、modal=false 无 scrim：贴缘坐标用文本 bounds,全高/全宽用面板矩形内/外点击捕获差异,锚在左上反证 snap 不翻转）;执行中发现并修复 Edge 节点尺寸未取 panel 矩形的缺口（node_size=panel_bounds.size(),content 锚面板原点）;`layout_tests` 32/32 绿 **[PLAN-534 T3]**
4. [✅ 已完成] alert_dialog_split_children 增 sheet/drawer/hover-card 的 trigger/content 全形态标签（kebab/snake/连写+hovercard- 混合形,对齐现有清单风格）;`cargo check` 零错 **[PLAN-534 T4]**
5. [✅ 已完成] convert_side_panel untracked 臂落地（side 参数化+chrome
   `w-96 …h-full`/`…w-full`+drawer 竖向圆角/把手+minted_dismiss+bare
   trigger）;sheet/drawer 臂接线（side/direction prop 解析缺省 right,非法
   值 eprintln+落缺省）;styled/passthrough 子臂双区全量。执行中补 D2 隐含
   前置:parser `modal_dialog_tag_role` 白名单扩 sheet/drawer（gallery 页
   无显式 open,不扩则铸造 toggle 不存在、页开不了）;`cargo t iced`
   166/168（2 红均非新增:lucide manifest 缺口=master 存量,clipboard=OS
   剪贴板环境 flake 复跑即绿） **[PLAN-534 T5]**
6. [✅ 已完成] tracked 分发区 sheet/drawer 臂 + convert_side_panel_tracked_ctx（path/id_map/probe 结构对齐 convert_alert_dialog_tracked_ctx）;passthrough/styled 子臂 tracked 侧同批落地;`cargo t iced` 同上 166/168 **[PLAN-534 T6]**
7. [✅ 已完成] parser.rs hover_card_role 助手（root/trigger 判定,不入
   modal_dialog_tag_role——hover 走 enter/leave 非 toggle）+ mint pass
   hovercard 分支:同一 `__dlg_open_N` state+open 绑定铸造,trigger 补
   onmouseenter→`__dlg_enter_N`/onmouseleave→`__dlg_leave_N`（事件落
   trigger 自身节点,转换臂从 trigger events 读进 MouseArea——与 toggle
   包裹落内层规则相反,注释在案）;TDD 先红后绿;显式绑定不覆盖负例在案;
   `cargo t parser` 213/213 **[PLAN-534 T7]**
8. [✅ 已完成] convert_hovercard untracked+tracked 双镜像臂（anchor 包
   View::MouseArea 接 enter/leave;placement=Bottom;chrome "w-80
   bg-popover border rounded-lg shadow-md p-4";on_dismiss=None;trigger
   onclick 经子件转换并存）+ 双区 hovercard 臂接线;新增
   plan534_side_panel_tests 三探针（sheet EdgeRight+铸造 dismiss /
   drawer EdgeBottom+把手首子 / hovercard MouseArea+Bottom+非模态）
   全绿;`cargo t iced` 167/168（唯一红=lucide master 存量）;MouseArea×
   overlay 组合留待 T11 实机验证定降级与否 **[PLAN-534 T8]**
9. [✅ 已完成] ui_gen/rust.rs:角色表+可关闭族扩 sheet/drawer;side/direction
   → Edge placement 发射表 side_panel_emission（chrome 与解释器臂同串）+
   drawer 竖向把手发射 drawer_handle_emission;hover_card_role +
   generate_hover_card_popover（MouseArea 包锚+Bottom+on_dismiss None+enter
   direct-msg 接线）+ 路由前置;双轨同串单测
   test_side_panels_codegen_matches_interpreter_chrome 一次通过（含铸造
   __dlg_close_1/__dlg_enter_3 折算断言）;`cargo t ui_gen::rust` 42/42 +
   `cargo t parser` 213/213 **[PLAN-534 T9]**
10. [✅ 已完成] 新增语料 test/ui/plan534_side_panels/（sheet 四向 open
   直驱 + drawer bottom 显式/right 铸造裸 trigger + hovercard 铸造,
   pac.at 同步）;plan536_t1_reactive_probe_tests.rs 追加
   plan534_side_panel_probe_tests 三组断言（side→Edge placement 全表+
   初渲染全闭合+计数 7 / open 直驱 Flip*+铸造 __dlg_toggle_1 双面板开 /
   __dlg_enter_2·__dlg_leave_2 状态翻转+Bottom 面板开合同步）全绿;
   `cargo tv` 3603/3604（唯一红=charts gallery vue codegen,master 同样红
   属存量,与 534 无关） **[PLAN-534 T10]**
11. [✅ 已完成] 实机验证全绿（MCP 驱动,证据 scratch/p534_evidence/ 28 件）:
    /sheet 触发器开（铸造 toggle 落内层按钮,源回显可见 onclick:
    .__dlg_toggle_1）+scrim 遮罩截图→OS 级 ESC 关（state true→false）;
    /drawer 裸文本 trigger 同链;hovercard 实机 hover（ctypes SetCursorPos,
    MCP 无 hover 原语）进→开/出→关（__dlg_open_2 翻转+截图）——MouseArea×
    Popover 组合实测正常,**不降级**。全站扫描 68 页（=pages/ 数）:三员+
    dialog 族 9 页全绿 minted=True;9 异常全为"连续 ~55 页导航后主线程栈
    溢出"连接拒绝尾部,master 二进制同现→存量归因。偏差两项登记:vue 基线
    运行面本机不存在（对照降级为 shadcn 语义+vue-ref 源语义,对齐）;gallery
    avatar 家族零高渲染缺口（先于 534,hover 实验移语料工程完成）。
    MCP keyboard 不进原始事件流→ESC 须 OS 键盘,均记录 summary.md
    **[PLAN-534 T11]**
12. [✅ 已完成] KNOWN-DEBT 登记 P534-D1~D6（delay 不消费/无拖拽手势/vue
    基线未运行对照/avatar 家族缺口/gallery 导航栈溢出存量/未降级裁定+ESC
    自动化口径）;gallery README 已知边界注记三员 VM 差异。fold 前裸
    `cargo tf` 全量:3462/3463,唯一红=charts gallery vue codegen（master
    复现同红,存量归因）。执行期新触发并已清偿的门禁:schema_drift 双围栏
    （三员入 aura.at 别名面+sheet-close 块+element_coverage 登记+baseline
    +98/-18 人工复核）+docs_gen P5（sheet-close 入 sheet 根 sub_widgets）
    ——全绿 **[PLAN-534 T12]**

## 复审记录

**复审人**：ZCode（/auto-plan:review 独立复审,2026-09-06）
**复审基点**：worktree `.wt/lang-534/auto-lang` 分支 `plan-534-dev`（13 commits,
merge-base 2fc544511;真实 diff 15 文件 +1852/-63,全部在计划声明面内;master
同期前进部分与本计划无关,已用 merge-base 剥离）。

### 验收逐条复验（verify, don't trust——全部复审自查重跑）

| # | 判据 | 结论 | 证据 |
|---|---|---|---|
| G1 | /sheet 四向贴边+scrim+三路关+open 直驱 | ✅ | 复审重跑 layout_tests 32/32（Edge 四臂贴缘坐标/全高全宽/snap 不翻转,iced 层直构命中探测）;corpus 探针 6/6（四向 Edge 映射+初渲染全闭合+计数 7;Flip* 直驱翻转）;实机 sheet 页 trigger 开（state=true,源回显 `onclick: .__dlg_toggle_1`）+OS 级 Esc 关（true→false）;**外点关本次复审补证**:OS 级点击 scrim 上→dismiss→false、面板内点击保持 open（scratch/p534_evidence/drawer_bottom_open_review.png 同工程;首轮 FAIL 系窗口未前台,焦点归位后 PASS） |
| G2 | /drawer direction 映射+bottom 圆角/把手渲染 | ✅ | 复审实机补证:scratch 临时工程 bottom drawer（铸造形态）trigger 开→截图（全宽贴底+rounded-t+顶部居中把手+scrim,state=true;`drawer_bottom_open_review.png`）;headless 把手首子断言+codegen `rounded-t-lg` chrome 串断言在案 |
| G3 | /hovercard hover 进出+avatar 原位+非模态 | ✅（avatar 项 partial→债 P534-D4） | hover 进/出实机（语料工程,state `__dlg_open_2` true/false 翻转+截图,卡片悬于锚正下方无 scrim）;on_dismiss=None 探针断言;**avatar 原位渲染 partial**:gallery 页 avatar 触发区零高——先于 534 的 convert_avatar 子件弃置缺口（P534-D4 已登记）,hovercard 特性本体不受影响（锚机制文本触发器全验证+gallery mint 接线树内可见） |
| G4 | 双轨一致（placement/chrome 同串单测） | ✅ | `test_side_panels_codegen_matches_interpreter_chrome` 复审重跑过（sheet EdgeRight+横条 chrome+`__dlg_close_1` 折算/drawer EdgeBottom+rounded-t+把手/hovercard MouseArea+Bottom+`__dlg_enter_3` 接线+on_dismiss None,全串断言） |
| G5 | 三页截图对齐 vue 基线+全站扫描零新增异常 | ✅（vue 运行对照 partial→债 P534-D3） | VM 截图 10 张落账 scratch/p534_evidence/;全站扫描 68 页三员+dialog 族全绿 minted=True,9 异常全为存量栈溢出尾部（master 二进制同现,归因非 534）;**vue 运行对照 partial**:本机 gallery 无 vue 运行面（无 node_modules/gen）,对照降级 shadcn 语义+vue-ref 源语义（对齐）,真 vue 对照登记 P534-D3 留补 |
| G6 | `cargo t iced`+`cargo tv` 全绿;差异文档化 | ✅ | 复审重跑 tf 3462/3463+tv 3603/3604——唯一红=charts gallery vue codegen,**master 同红**（Plan 575 review 记录同口径,存量）;iced 167/168 同因;KNOWN-DEBT P534-D1..D6+gallery README 已知边界注记在案 |

### 遗漏/延后/workaround 猎捕

- diff 全文 TODO/FIXME/HACK/临时 扫描:零残留;
- 12 步与 diff 逐条对得上,无丢件;执行期两处隐含前置（parser 角色表扩容、
  Edge 节点全高矩形）已补齐并有测试;
- 延后两项均非计划内任务缩水,系执行期发现的**预先存在**约束:avatar 家族
  缺口（D4）、vue 运行面缺失（D3）——均已在 KNOWN-DEBT+待澄清④⑤登记;
  delay 不消费/无拖拽为计划文既定接受项（待澄清①②,execution 前已写明）;
- workaround:无。降级路径（hovercard click-toggle）经验证**不需要**。

### 债务候选

KNOWN-DEBT P534-D1..D6 已登记（D3 vue 对照/D4 avatar 为本轮 partial 项;
D5 gallery 导航栈溢出=master 存量,建议独立归因计划）。

### 路由

**PASS → `status: reviewed`**。六判据全过（G3/G5 各含一项 pre-existing
partial,按 053 先例以 D 项甄别放行,债已入账）。可进 `/auto-plan:merge`;
merge 时建议向用户口头复述 D3/D4 两项 partial 的处置意向（补做 or 接受）。

## 待澄清事项

（2026-09-06 改写时清算:原①Esc 语义已由 533 T6 落地定案（dialog 族三路
关/alert 族不关）;原②P3 拆分已随本改写失去意义（范围仅剩三员,单计划
体量可控）;原③toggle_group v-model 属 530 范围,moot。剩余:）

1. hovercard **open-delay/close-delay v1 不消费**（即时开合）——本计划
   按"接受并文档化"执行;若用户要求真延迟,需 VM 轨定时器原语,另立。
2. drawer **拖拽 snap 手势**（vaul 语义）不做,v1 静态贴边+装饰把手;
   后续手势需求另立计划。
3. ✅ 全站扫描页数口径已实证:pages/ 68 个 .at 文件,扫描访问 68 页
   （scratch/p534_evidence/13_fullsite_scan.md）。
4. **vue 基线运行对照未做**（执行期发现）:本机 gallery 无 vue 运行面
   （无 node_modules/gen,需 auto gen+npm install 超执行面）,观感对照降级
   为 shadcn 语义+vue-ref 源语义（对齐,截图在案）;真 vue 对照登记
   KNOWN-DEBT P534-D3,留复审裁定是否补。
5. **hovercard 实机 hover 实验移至语料工程**（执行期发现）:gallery 页
   触发器内 avatar 家族 VM 渲染零高（先于 534 的存量缺口,KNOWN-DEBT
   P534-D4）,hit area 不可悬停;语料工程（文本触发器）实机 hover 进/出
   全通,降级裁定=不降级。另:gallery 连续导航 ~55 页后主线程栈溢出为
   master 存量（双二进制复现,KNOWN-DEBT P534-D5）。
