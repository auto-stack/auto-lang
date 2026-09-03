---
plan_id: PLAN-533
status: drafting         # drafting → executing → execution_done → reviewed → archived
feature_name: VM(a2r) 悬浮层运行时通道——alert-dialog/dropdown 家族 codegen 臂 + Modal iced 运行时
author: [zhaopuming, ZCode]
created_at: 2026-09-03
updated_at: 2026-09-03T23:30:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 8
---

# [PLAN-533] VM(a2r) 悬浮层运行时通道

## 变更摘要

musk PLAN-059 定案的本仓遗留大项：**编译 VM 轨没有任何悬浮层家族实现**——
`ui_gen/rust.rs`（6426 行）分发表无 alert-dialog/dialog/popover 任何浮层臂，
alert-dialog 编译为普通 button（无点击语义/开合机制），dropdown/dialog/tooltip/
toast 等约 100 个悬浮语义元素全部退化为流内容器。本计划给 a2r 编译轨补**浮层
运行时通道**（四件套：codegen 臂 + Modal iced 运行时 + 开合/open 绑定 + ESC/外点
事件回流），并重做此前在 auto-musk-dev 分支（已删,未合回）上完成的三件丢失工作。
完成后 musk 侧恢复执行其 PLAN-059（T4 验证 → T5-T8 家族铺开 → T9 三场景回归）。

## 目标

1. alert-dialog/dialog 家族在编译 VM 轨以**居中模态 + 全视口暗幕**悬浮呈现，
   ESC/遮罩点击/取消按钮均可关闭且状态复位，action onclick 正确派发。
2. dropdown_menu 以**锚定弹层**呈现（trigger 下方、越界翻转、外点关闭）。
3. widgets-gallery overlay 家族页面（/alertdialog /dialog /dropdownmenu）双轨
   （vue/vm）对拍通过，AutoUI snapshot 包含 overlay 层。
4. schema `aura.at` 已实现家族 iced 标注 none→native 随实现回填。

## 现状勘察（2026-09-03 实证,与 musk PLAN-059 联合勘察）

- **codegen 侧**：`crates/auto-lang/src/ui_gen/rust.rs`（6426 行）`tag_to_view_fn`
  （:3613）按 tag 映射视图构造 fn——alert-dialog/dialog/popover **零臂**（grep 0 命中，
  2026-09-03 复核）；`"modal"/"tooltip"` 两臂为死映射（运行时无 View::Modal/对应
  实现）。工程 `.auto/ui-cache.json` 缓存编译产物——codegen 改动后必须删该文件
  强制重编；Windows 下运行中的 auto.exe 锁文件，cargo build 前须 taskkill。
- **解释器侧**：`ui/iced/popover.rs`（529 行）自绘锚定浮层已有
  （placement/at_point/gap/open/on_dismiss/Esc/外点关闭全备），renderer 已接
  `AbstractView::Popover`，aura_view_builder 已有 popover-trigger/content 拆解臂
  ——但**无 Modal 形态**（`PopoverPlacement` 无 Modal 变体）。
- **丢失工作（auto-musk-dev 分支已删未合回,需重做）**：
  ① `view.rs` `PopoverPlacement::Modal` 变体；
  ② popover.rs Panel 模态三语义（layout 根=全视口命中区+content 居中；
     update 内容外点击/ESC=dismiss+捕获；draw 先画全视口暗幕 Quad）；
  ③ aura_view_builder alert-dialog 家族拆解臂（trigger/content→popover-*
     委托,placement_override=Modal,oncancel 别名折算 ondismiss）；
  ④ child_emit.rs 注册/派发两侧键**大小写折叠匹配**修复（musk PLAN-059 T2,
     当期全量 lib 4284 过/173 败 vs 基线 4280/175,零新增失败净修好 3）+ 2 单测。
- **schema 矩阵**（`schema/aura.at`）：overlay 类组件 36 个,35 个 `iced: none/
  unknown`;mouse-area（484,iced: full）已有——musk 实测 hover 命中区事件链路活。

## 技术栈

- 主战场：`crates/auto-lang/src/ui_gen/rust.rs`（codegen）、`ui/iced/popover.rs`
  + `ui/view.rs` + `ui/aura_view_builder.rs`（解释器侧对齐）、`schema/aura.at`（回填）。
- 验证：widgets-gallery（vue/vm 双轨）、AutoUI MCP snapshot/截图、iced_test、
  musk 实机三场景（删除确认/工程目录 dropdown/设置 dialog）。

## 需求分析与背景调查

- 上游依据：musk `docs/plans/059-vm-overlay-infrastructure.md`（T4 根因定案 +
  codegen 侧确认节,含 2026-09-03 用户实机复测）；PLAN-058 待澄清⑩（跨 widget
  派发）与 055 子件缺陷族（子件 model 全实例共享根态——musk 2026-09-03 实机
  ToolBlock 点一张全展开二次实锤,已由 musk 侧内联规避,但子件模型缺陷本身在本仓）。
- musk 侧约束：全程不动 musk backend/web 源码;musk 侧内联确认行（PLAN-058 形态）
  在本计划落地前为最优可用形态,落地后由 musk PLAN-059 T9 切换标准组件。

## 执行步骤

- [ ] **T1** 重落 child_emit 大小写折叠匹配（丢失工作④）：注册/派发两侧键折叠
  小写 + 2 单测。验证：全量 lib（--features ui-iced）不劣于基线、净修好存量失败。
- [ ] **T2** 重做解释器侧 Modal 基建（丢失工作①②③）：PopoverPlacement::Modal、
  popover.rs Panel 模态三语义、aura_view_builder alert-dialog 家族臂。验证：
  iced_test 单测绿;解释模式探针（examples/overlay-probe,需先清 build 全量编译）
  /alertdialog 按触发钮出浮层。
- [ ] **T3** codegen 臂：ui_gen/rust.rs alert-dialog/dialog 家族拆解 → Modal 构造
  调用发射（trigger/content/header/title/description/footer/action/cancel;
  action·cancel onclick 走既有 DynamicMessage 派发形态）。验证：codegen 单测 +
  gallery 工程编译产物含 Modal 构造（先删 .auto/ui-cache.json）。
- [ ] **T4** 生成侧浮层运行时：Modal iced 实现（全视口暗幕 Quad + 居中卡片,
  宽 min(480px,90vw)）,确认生成代码可引用的运行时 crate 面（复用/下沉
  ui/iced/popover.rs Panel）。验证：gallery /alertdialog 实机出浮层。
- [ ] **T5** 触发器开合 + open 态绑定：__popover_toggle 自管开合 + state_ref
  v-model 对齐 vue 轨语义。验证：连续开合状态复位;MCP snapshot 断言
  open 前后差异。
- [ ] **T6** ESC/外点 dismiss 事件回流：onDismiss/onCancel 折算 update:open(false)。
  验证：ESC/遮罩/取消三路关闭 + 状态复位。
- [ ] **T7** dropdown_menu anchored 臂（Phase 1 视范围裁定,可与 T4 并行）：
  placement bottom-start + 越界翻转 + 外点关闭（复用 popover Panel）。
  验证：gallery /dropdownmenu 按 Open 出锚定弹层。
- [ ] **T8** 收尾：schema aura.at 已实现家族 iced none→native 回填;账本回写
  （overlay 缺口族、跨 widget 派发修复、丢失工作重做归档）;gallery 双轨对拍
  截图入 attachments;通知 musk 侧恢复 PLAN-059（T4 验证起）。

## 测试设计

- **单测**（本仓）：overlay 挂载/卸载、anchored 定位与翻转、modal backdrop
  dismiss、update:open 折算、child_emit 大小写折叠——iced_test。
- **gallery 门禁**：/alertdialog /dialog /dropdownmenu VM 端按 trigger → MCP
  snapshot 断言弹层节点存在且不在文档流父链下 + 截图目验浮空;与 vue-ref 对拍。
- **既有门禁**：全量 lib（--features ui-iced）不劣于基线;auto build --gen-only;
  vm-safe-lint;musk 侧四门禁（build strict/vitest/对拍/探针）在 T8 联测时复跑。

## 验收标准

1. gallery /alertdialog /dialog：VM 实机居中模态+遮罩悬浮,ESC/遮罩/取消可关,
   action onclick 派发正确,截图双份。
2. gallery /dropdownmenu：锚定弹层、外点关闭、越界翻转正确。
3. AutoUI snapshot 包含 open 态 overlay 层。
4. schema overlay 家族 iced 标注回填,schema 校验通过。
5. musk 三场景（删除确认/工程目录 dropdown/设置 dialog）联测通过
   ——该条与 musk PLAN-059 T9 共同签收。

## 待澄清事项

1. **Phase 裁剪**：tooltip/hover_card/select/combobox/drawer/sheet/command/
   context_menu/menubar/nav_menu 是否 Phase 2 另批（musk 三场景仅需
   alert_dialog + dropdown_menu,Phase 1 建议只做这两族+dialog）。
2. **modal 库选型**：iced 0.14 原生 Stack+overlay 自研（倾向,combo_box 内部即此
   模式,依赖面小）vs 引入 iced_aw Modal——待复审裁定。
3. **AutoUI MCP overlay 可见性**：snapshot 需定义 open 态 overlay 层的呈现口径
   （musk 侧验收自动化依赖）。
4. **丢失工作口径**：auto-musk-dev 分支三件（①②③）+T2 大小写折叠按"重做"
   处理（原分支未合回已删）,还是能从 musk 侧留存的 PLAN-059 检查点记录
   （本文件现状勘察节）直接复原——建议直接按本文档重做,不找回升序。
