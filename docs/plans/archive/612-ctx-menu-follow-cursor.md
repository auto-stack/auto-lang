---
plan_id: PLAN-612
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: ctx-menu-follow-cursor
author: ["zhaopuming"]
created_at: 2026-09-11
updated_at: 2026-09-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-612] ctx-menu-follow-cursor

> **来源（PLAN-611 终验移交，2026-09-11）**：用户终验 611 时确认外点关闭/
> 功能面全 OK，但两处右键菜单位置固定不合 Windows 惯例，点名改跟随鼠标。
> 用户指令直接修（611 同款免 worktree 模式）。

## 变更摘要

桌面两处右键菜单位置改为跟随鼠标落点：

1. **标题栏右键菜单**（chrome 自绘，PLAN-526 T37 原设计=固定右上对齐三键）
   ——右键落点即菜单左上角，窗内视口钳制。
2. **桌面空白菜单**（.at popover，widget 锚=全屏 mouse-area + bottom-start
   → 固定左下，PLAN-011 复审 F-R1 同源）——改用 **Plan 422 P3 坐标锚形态**
   （`popover (open:, x:, y:, ...)`，027-file-manager 先例），坐标由宿主
   `__mouse_moved` 臂持续写入桌面面 VM 状态（只写不置 dirty）。

## 目标

- AC-1：标题栏右键菜单出现在右键落点（跟手），窗缘钳制不越界。
- AC-2：桌面空白右键菜单出现在右键落点（跟手），不再固定左下。
- AC-3：外点/Esc 关闭、菜单项动作、611 全部既有判据零回归；复跑门
  （iced 档 + 全量失败集对账 + hash-lock）零新增。

## 需求分析与背景调查

- 坐标锚 DSL 自 Plan 422 P3 存在（`aura_view_builder convert_popover`
  `coord("x")/coord("y")` → `PopoverAnchor::Point`，headless 测试
  plan446_c1_popover + examples/ui/027-file-manager 双先例）——**本计划
  零 DSL/parser 增量**，纯消费既有形态。
- 标题菜单固定右上 = T37 设计语义（"标题条下方、右收边 8px 与三键对
  齐"）；空白菜单固定左下 = widget 锚（children[0]=全屏 mouse-area）+
  bottom-start 的几何必然，011 复审 F-R1 在案。
- 光标事实源：`WmState.last_cursor`（`__mouse_moved` 订阅持续回写，
  host 窗逻辑坐标）；桌面面组件 `state.desktop.desktop_app` 可达。

## 详细设计

- **A（chrome）**：`WmState.title_menu: Option<Wid>` →
  `Option<TitleMenuSpot{wid, x, y}>`；`TitleMenuOpen` 臂按
  `last_cursor - vwin.rect.origin` 现算并经纯函数
  `virtual_window::title_menu_spot` 钳制（面板 180×~168 + 8px 边距，
  y ≥ 标题条底——右键点在标题条上，菜单即其下方展开）；面板定位改
  `padding{left,top}` + Start/Start 对齐。
- **B（.at）**：desktop.at 空白菜单 popover 换坐标锚（mouse-area 移出
  为兄弟子件——widget 锚形态退役）；模型声明
  `__desktop_cursor_x/y`（float）；宿主 `__mouse_moved` 臂向桌面面组件
  持续写坐标（`write_state` 不置 view_dirty——`blank_menu` open 翻转
  才触发重建读取最新值，零逐帧重建成本）。
- **vue 轨**：popover 转换仅消费 `open`，vue 侧坐标定位缺位 → D-GAP
  登记（待澄清①），vue 桌面面暂维持原位（iced 桌面为主验证面）。

## 测试设计

- `title_menu_spot` 纯函数钳制单测（virtual_window.rs 新增 tests mod：
  常规/右缘/下缘/退化窗四例）。
- pack 全量编译冒烟（Plan 503 既有面——desktop.at 改动直过）。
- 复跑门：`cargo t iced` + 全量档失败集对账（基线=21 存量单）+
  hash-lock。

## 验收标准

- AC-1/AC-2：用户实机终验（跟手 + 钳制不越界）。
- AC-3：复跑门零新增 + 611 三判据不回归（用户复核）。

## 执行步骤

- T1 session.rs `TitleMenuSpot{wid,x,y}` + `title_menu` 字段换型 [✅ 已完成]
- T2 renderer.rs `TitleMenuOpen` 臂按 last_cursor-rect 现算锚点（`title_menu_spot`
  钳制）[✅ 已完成]
- T3 renderer.rs 两处 vwin 装配点换传 `title_menu_pos: Option<(f32,f32)>` [✅ 已完成]
- T4 virtual_window.rs `virtual_window_element`/`title_menu_panel` 签名与
  padding-left/top 跟手定位；`title_menu_spot` 纯函数 + 4 例钳制单测 [✅ 已完成]
  证据：`cargo t iced` p012 四测 PASS。
- T5 renderer.rs `__mouse_moved` 臂向桌面面组件持续写
  `__desktop_cursor_x/y`（`write_state` 不置 view_dirty）[✅ 已完成]
- T6 auto-os shell/desktop.at：空白菜单 popover 换坐标锚
  （`x: .__desktop_cursor_x, y: .__desktop_cursor_y`，mouse-area 移出为
  兄弟子件；模型声明两变量）[✅ 已完成]
- T7 pack 同步 + 金样 + 复跑门 [✅ 已完成]
  证据：shell-pack-sync desktop.at pin `c207f048e5→5c87aa9579` 后四件全等；
  iced 档 187/188（p012 四测全过，唯一红=lucide `film` 存量）；a2vue 金样
  `AUTO_LANG_UPDATE_GOLDEN=1` 再生后转绿；全量档 4749/4771——失败集 21=21
  与定型基线全等 + `osconfig_daemon::ensure_ready_override` 1 枚环境 flake
  （单测复跑 3/3 过，同 ffi_dual_019 在案类，非本计划引入）。
- T8（终验加值）右键标题栏先聚焦置顶：非最前窗的菜单层被前窗遮挡
  （用户终验发现）——`TitleMenuOpen` 臂补 `state.wm_focus(wid)`（立即
  z_order 刷新+焦点+MRU；右键无 in-flight 按钮态，用即时 focus 而非软
  聚焦；左键仍走 GlobalPress 软聚焦+release 偿还，双击首击即聚焦）
  [✅ 已完成]
  证据：iced 档 187/188 零新增。

## 复审记录

### 复审（2026-09-11，报障用户本人实机终验 + 工件复跑门）

stage: review | plan_id: PLAN-612 | plan_revision: 1 | outcome: **pass** |
reviewed_commit: auto-lang master（612 实现+置顶加值提交）| 
spec_inputs: 无规范增量

- **AC-1 ✓ pass（用户实机）**：标题栏右键菜单出现在右键落点（跟手），
  窗缘钳制不越界。
- **AC-2 ✓ pass（用户实机）**：桌面空白右键菜单出现在右键落点，不再
  固定左下。
- **AC-3 ✓ pass**：复跑门零新增（iced 187/188+p012 四测+全量 21=21 基线
  全等+hash-lock 四件全等+金样再生绿）；用户终验确认外点/Esc 关闭、菜单
  项动作、611 三判据（缩略稳定/切换稳定/标题菜单外点关）全部无回归。
- **终验加值（已并入 T8）**：右键标题栏先聚焦置顶，非最前窗菜单不再被
  前窗遮挡。

findings: 无。
evidence: 执行步骤 T1-T8 收据 + 本轮用户对话实录（终验 OK）。
next: 归档（archive/）。

### merge 回执（2026-09-11，五检查点）

prepared ✓（执行态基线=折叠定型 master）| landed ✓（交付直接在 master：
feat 提交+置顶加值提交，无分支折叠）| ledger_refreshed ✓（autos-desktop-
program 桌面域指针无本计划行，归档即登记）| archived ✓（git mv
docs/plans/archive/）| cleaned ✓（无专用 worktree；tmp 探针脚本与截图留
auto-lang tmp/ 作过程证据）。

## 待澄清事项

1. vue 轨 popover 坐标定位缺位（D-GAP 登记）：vue 桌面面空白菜单暂维
   持 widget 锚原位行为；radix-vue Popover 坐标定位需 virtual element
   方案，宜专项。
2. 空白菜单右下缘钳制依赖坐标锚的 Panel snap（T36 引入的视口钳制语义
   对 Point 锚是否同样生效）——实机验证；若不生效改 .at 侧钳制或宿主
   钳制补充。
