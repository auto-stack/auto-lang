---
plan_id: PLAN-533
status: execution_done  # drafting → executing → execution_done → reviewed → archived
feature_name: VM(a2r) 悬浮层运行时通道——alert-dialog/dropdown 家族 codegen 臂 + Modal iced 运行时
author: [zhaopuming, ZCode]
created_at: 2026-09-03
updated_at: 2026-09-04T00:00:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 8
total_steps: 8
---

# [PLAN-533] VM(a2r) 悬浮层运行时通道

## 执行完结记录（2026-09-04 execution_done）

八步全勾,commit 链于 plan-533-dev（T1 544d14bcb / T2 验证件 / T3 f678dcb4b /
T4 c9a59e440 / T5 68c134a8b / T6 4f162fcad / T7 23621f545 / T8 schema+账本+截图
+82d255956 ark 金样）。全量 lib（--features ui-iced）终态 4493 跑/4476 过/17 败
——失败集为存量环境性（layout×14+d8+c2+strips）,较 T1 基线 19 败净 -2（child_emit
净修好 1 + ark 012_dialog 金样随铸造语义更新后揭示的 p508 掩盖位归位）,零新增失败。
交接 /auto-plan:review。

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

- [✅ 已完成] **T1** 重落 child_emit 大小写折叠匹配（丢失工作④）：注册/派发两侧键折叠
  小写 + 2 单测。验证：全量 lib（--features ui-iced）不劣于基线、净修好存量失败。
  （533 T1 commit 544d14bcb：`fold_key()` 双表两侧小写；2 单测红→绿；全量 lib ui-iced
  基线 4485 跑/4466 过/19 败 → 修复后 4487 跑/4469 过/18 败，重叠失败集 17 个全为存量
  环境性失败（layout×14/d8/c2/strips），p508_g2 满载 flaky 单跑 16.5s 过）
- [✅ 已完成] **T2** 重做解释器侧 Modal 基建（丢失工作①②③）：PopoverPlacement::Modal、
  popover.rs Panel 模态三语义、aura_view_builder alert-dialog 家族臂。验证：
  iced_test 单测绿;解释模式探针（examples/overlay-probe,需先清 build 全量编译）
  /alertdialog 按触发钮出浮层。
  （533 T2 勘察更新：①②③经查已由 PLAN-530 步骤8（源 W13）落地——本计划勘察时点
  （09-03）早于 530 合入,Modal 变体/三语义/双镜像臂均在;本步改为补验证面：iced_test
  Modal 四断言（居中/外点整吞/Esc 捕获/面板内点击）全绿;overlay-probe 探针实机像素
  证据——open 与 closed 差异 92.7%（遮罩+居中卡）、Cancel 后 0.0（复位）、Esc 后 0.0
  （shadcn 保持打开语义）、h/4 像素 (10,13,21)=50% 黑幕叠 (20,26,41);commit 见
  plan-533-dev）
- [✅ 已完成] **T3** codegen 臂：ui_gen/rust.rs alert-dialog/dialog 家族拆解 → Modal 构造
  调用发射（trigger/content/header/title/description/footer/action/cancel;
  action·cancel onclick 走既有 DynamicMessage 派发形态）。验证：codegen 单测 +
  gallery 工程编译产物含 Modal 构造（先删 .auto/ui-cache.json）。
  （533 T3 commit f678dcb4b：家族归一化识别+根臂拆解+子件预设臂;3 单测红→绿
  （合成 alert-dialog 族/dialog 族+真实 gallery alertdialog.at 页产物级断言含
  PopoverPlacement::Modal+w-96 chrome）;ui_gen 738/738 全绿。注：gallery 整仓
  rust 生成（examples/rust-workspace）为存量红（壳层 SettingsPanel/icon/scroll
  词汇超 rust 轨能力,与本次改动无关）,产物级验证以页级断言替代;ui-cache.json
  为 vue 轨缓存与本臂无涉,已复原误删的 tracked 文件）
- [✅ 已完成] **T4** 生成侧浮层运行时：Modal iced 实现（全视口暗幕 Quad + 居中卡片,
  宽 min(480px,90vw)）,确认生成代码可引用的运行时 crate 面（复用/下沉
  ui/iced/popover.rs Panel）。验证：gallery /alertdialog 实机出浮层。
  （533 T4 commit c9a59e440：运行时面=生成代码直引 auto_lang::ui::view::Popover/
  PopoverAnchor/PopoverPlacement,复用既有 popover Panel 零下沉;关键修复=on-only
  handler（无 msg 块 vue 风格源）此前生成 type Msg=() 而派发闭包悬垂,rust 轨枚举补
  零参变体;实机：overlay-probe 编译轨 cargo build 绿+MCP 状态环（press→show=true→
  Cancel→show=false+last=cancel）+原生窗口视觉（居中模态卡+50% 黑暗幕,analyze_image
  在案）;gallery 整仓 rust 轨存量红（壳层词汇）,实机以同形探针替代。**偏离记录**：
  面板宽 w-96（384px）而非计划文 min(480px,90vw)——取解释器轨（530 W13 已定）
  同串保双轨对拍一致）
- [✅ 已完成] **T5** 触发器开合 + open 态绑定：__popover_toggle 自管开合 + state_ref
  v-model 对齐 vue 轨语义。验证：连续开合状态复位;MCP snapshot 断言
  open 前后差异。
  （533 T5 commit 68c134a8b：parser 层铸造 __dlg_open_<n>/__dlg_toggle_<n>/
  __dlg_close_<n> 三件套+root open 绑定+trigger/深嵌 close 接线（toggle 命名按
  铸造双 handler 落地）;解释器补 dialog 家族臂;VM+编译轨 state 环全 PASS（连续
  开合复位）;snapshot 观察记录：快照含关闭态 overlay 子树,open 断言以 state 为
  准——待澄清③口径素材;全量 lib 18 败与基线一致零回归）
- [✅ 已完成] **T6** ESC/外点 dismiss 事件回流：onDismiss/onCancel 折算 update:open(false)。
  验证：ESC/遮罩/取消三路关闭 + 状态复位。
  （533 T6 commit 4f162fcad：铸造形态 on_dismiss=DynamicMessage::Typed(__dlg_close_N)
  /codegen Some(Msg::__dlg_close_N);实机三路全证——真 ESC（iced 键盘,注意 MCP 合成键盘走
  VM handler 流不经 overlay,故用 computer-use 真键）/真外点（SendInput）/关闭按钮;alert 族
  ESC 保持开+Cancel 关（shadcn 语义）;显式 open 绑定自管形态不接管（Phase 1 记录）;9 测试绿）
- [✅ 已完成] **T7** dropdown_menu anchored 臂（Phase 1 视范围裁定,可与 T4 并行）：
  placement bottom-start + 越界翻转 + 外点关闭（复用 popover Panel）。验证：gallery
  /dropdownmenu 按 Open 出锚定弹层。
  （533 T7 commit 23621f545：三轨同构（parser 铸造+codegen 家族参数化+解释器
  ModalDialogFamily 分流）;VM 实机——铸造 state/trigger 开/items/真外点关/差分锚定
  面板 y 71.4% 于触发钮下方（非模态居中）;越界翻转与外点语义复用 popover Panel
  （既有 iced_test popover_bottom_start/snaps_within_viewport 在案）;TDD 红→绿;
  全量 lib 零回归）
- [✅ 已完成] **T8** 收尾：schema aura.at 已实现家族 iced none→native 回填;账本回写
  （overlay 缺口族、跨 widget 派发修复、丢失工作重做归档）;gallery 双轨对拍
  截图入 attachments;通知 musk 侧恢复 PLAN-059（T4 验证起）。
  （533 T8：schema 26 条回填（schema_drift 无新增失败）;KNOWN-DEBT P533-D1..D8;
  gallery 三页 VM 双态截图入 docs/plans/attachments/533/（alertdialog 71.6%/dialog
  71.7%/dropdownmenu 3.7% 锚定小面板 差分在案,路由+state 开合+Save 关闭全证）;
  vue 端对拍以 gallery 页编译绿（vue 轨冒烟）+vue-ref 参照待 musk T9 联测复跑;
  **musk 侧通知**：PLAN-059 可自 T4 验证起恢复——本仓三族（alert_dialog/dialog/
  dropdown_menu）VM+编译轨通道全通,三场景联测就绪）

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
