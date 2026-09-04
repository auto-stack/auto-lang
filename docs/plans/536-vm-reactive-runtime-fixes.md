---
plan_id: PLAN-536
status: executing        # drafting → executing → execution_done → reviewed → archived
feature_name: VM 运行时修复批——反应性三题 / 子件 prop 约束 / absolute 定位原语 / 家族浮层 open 绑定断链
author: [zhaopuming, ZCode]
created_at: 2026-09-04
updated_at: 2026-09-04T14:30:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 9
total_steps: 9
---

# [PLAN-536] VM 运行时修复批(反应性 + prop 约束 + absolute 原语 + 家族 open 绑定)

> 序号更正（2026-09-04）：本计划原立项为 PLAN-534,与先建的
> `534-vm-widget-family-parity.md`（09-03 23:56 立项）序号冲突,
> 按"后建者换号"改为 PLAN-536。

## 变更摘要

musk 侧 2026-09-03/04 实机实证（其 KD 059-FU1）的三个 VM 运行时问题,与浮层
通道（PLAN-533）无关、可并行立项：

1. **timer 驱动的状态写入不触发视图重渲染**——musk PollStream 500ms 轮询
   chats_get_session 全量回填,实跑 18 拍且完成启发式翻转（store 已拿到数据）,
   但画布永不刷新;用户视角"AI 回复了但界面永远不动"。变通=重选会话
   （handler 驱动可重渲染）。
2. **子件 Init handler 重入风暴**——单会话期 WorkspaceSelector/SettingsMenu/
   MentionInput/ChatsView 四个 Init 被调 1.6 万+次（重渲染循环反复跑 Init,
   副作用 per-render 重入;ChatsView.Init → ForgeStore.Init → LoadSessionList
   随之重复打后端）。
3. **Date.format 时区异常**——created_at=1788436450（19:54:10+08）渲染成
   08:34:14,偏差 11h19m56s,非整时区偏移,疑 native 换算缺陷。
4. **【09-04 增补】子件 prop 对象运行时约束（实证）**——子件 handler 作用域
   读写 prop 对象字段即崩（`RuntimeError("Invalid object ID: 0")`,musk
   ToolToggleKey 实录）;prop 传值为构建期快照,源端更新不达（store 键写入
   autoui_state 实证、子件读侧恒旧）。musk 已顶层内联绕行（4700cdc）。
5. **【09-04 增补】absolute 定位原语缺失**——视图元素 `absolute + right/top`
   类在解释器不消费,悬浮语义全退化为流内布局（musk 会话卡 × 挤压标题实录;
   用户裁定:悬浮元素不应影响任何兄弟布局）。方案=参考 popover 通道
   （详见执行步骤 T6）。
6. **【09-04 增补,源=musk PLAN-059 待澄清⑨】家族浮层 open 绑定在 fallback
   路径断链**——`auto run --render=vm` 走 "vm+vm merged → VM interpreter
   UI"（gallery/musk 同模式）,gallery /alertdialog 模态渲染成立;而 musk
   chats_view 的 alert-dialog 状态翻转正常（delete_confirm_open=true）但
   渲染树无 modal 节点,四形态探针全灭（slot 深嵌/视图根部/字面量文案/空
   trigger）,autoui_check 实证各视图子件均走 "unknown tag → Column
   fallback"。疑 fallback 分支的 bindings 解析断链（状态值运行时可见、
   转换期解析失败）。musk 已以内联确认行兜底（musk 059 T9）,根修后退役。

## 目标

1. timer/外部事件路径的状态写入触发视图失效（或提供显式失效原语）,musk
   PollStream 场景回复到达即显示。
2. Init 语义收敛:Init 只在挂载时执行一次（或有明确的 per-render 生命周期
   契约）,副作用不随重渲染重入。
3. Date.format 对 epoch 秒/毫秒产出与本机时区一致的 HH:mm:ss。
4. 子件 prop 生命周期语义定案(可写性/活性/快照边界)并文档化。
5. `absolute + right/top` 在解释器渲染为父容器内真悬浮层,不影响兄弟布局;
   musk 会话卡 × 实机验证无遮挡、位置正确。
6. musk 上下文（unknown-tag 子件 fallback 渲染）中 alert-dialog 家族随
   open 态真渲染;gallery /alertdialog /dropdownmenu 双页回归不劣化。

## 现状勘察（证据源）

- musk `docs/plans/KNOWN-DEBT-AND-RISKS.md` **059-FU1 行**（实证链+变通+日志
  计数:PollStream 18 拍/Init 5498→16293 次/时间偏差样本）。
- musk `src/front/forge_store.at` timer PollStream（every_ms:500,when:.streaming）
  ——when 门与失效路径的交互待查。
- 复现最小面猜测：任意 .at 工程 timer handler 写 state + 视图绑定该 state。

## 执行步骤

- [✅ 已完成] **T1** 复现探针：组件层 5 探针全绿（plan051_timer 语料 + 新增
  test/ui/plan536_reactive 双臂语料：直绑 vs 子件 prop）——timer→handler→
  dirty→重建视图全链绿,含 store 跨模块绑定与子件 prop 每帧重解析;实机
  canvas 探针（scratch/p536_t1_live_probe.py,autoui_screenshot 像素差）：
  根 widget timer（0.050%）与 store timer+子件 prop 双臂（0.017%）画布均
  重绘。**定案**：框架派发路径无通路差异;musk 题1 断点=handler 执行中崩
  （≠HandlerNotFound）后 dirty 不置位（与题4 "Invalid object ID" 同根,
  副作用已落/画布永冻）;次要缺口=hot-reload 拍早退口无脏桥接（异步回填
  无尾随派发时画布不醒）。
- [✅ 已完成] **T2** timer 写入失效根修（题 1）：①dynamic.rs 两处派发
  Err 臂（`on` + `on_with_input_for`,含 legacy fallback）——执行中崩
  （≠HandlerNotFound）置 dirty（HandlerNotFound=无副作用不置,防伪失效）;
  ②renderer.rs 热重载拍早退口补 is_dirty→view_dirty 桥（500ms 兜底唤醒,
  干净拍零开销）。TDD:p536_t2 RED（Boom 语料复现 Invalid object ID+dirty
  未置）转 GREEN;p536 全 6 探针绿;UI 切片 286/288（2 红=d8_toggle_dark_mode
  +plan055_strip_html,干净 master 同红,存量基线注记）。musk 实机复验在 T5。
- [✅ 已完成] **T3** Init 重入收敛（题 2）：子件 Init 收敛挂载语义——
  VmBridge 新增 child_inits_fired 名集合（跨帧存活记账）,aura_view_builder
  fire_child_init_if_any 按名一次（首渲染派发,重建帧跳过）;props 仍每帧
  重播种,派生值响应归 watch/computed（对齐 vue onMounted）。TDD:语料
  ChatBubble.Init 写根态计数,RED 实证 1+6 帧重放 7 次,转 GREEN 恒 1;
  UI 切片 344/346（含 chart/init 族,2 红=master 存量）。musk 计数复验在 T5。
- [✅ 已完成] **T4** Date.format 时区（题 3）：format_date_ms 双口径归一
  （|值|<1e11 视为 epoch 秒 ×1000;先于 Local 换算,只动单位不动时区）。
  TDD:RED 实证秒口径 1788436450 → "00:47:16"（1970 垃圾,即 musk 所见
  "非整时区偏移"假象的缺陷类）,转 GREEN "19:54:10";单测四件（双口径
  一致/无 1970 垃圾/毫秒路径不误伤/跨日闰日边界）;date 族 42/42 全绿,
  musk_vm_track 既有 Date.format 端到端（createdAt×1000 形态）兼容。
- [✅ 已完成（含边界注记）] **T5** musk 联动回归：以 worktree 修复版二进制
  起 musk VM 实机（split 模式,后端 9247,日志 tee scratch/p536_t5_musk_vm.log）。
  **题2 达成**：全会话期子件 Init 计数=每实例每子件 1 次
  （App/ChatsView/MentionInput/SettingsMenu/WorkspaceSelector 各 3 次=2 实例
  +1 重载,对比 KD 059-FU1 的 5498→16293 万级）,LoadSessionList 不随帧重入。
  **题1 框架层达成**：PollStream 订阅实机活跃（UI_EVENT 2081 拍,when 门=派发
  前过滤实证）;组件层+实机语料像素双证见 T1。**边界注记**：musk UI 端到端
  "发送→免重选直显"被**范围外存量债**拦阻——MentionInput.send 实机复现
  `Invalid object ID: 18446744071562067969`（KD-493① 同族）+
  `ChatsView.SendInput` 内 Sse 桥族 `Field 'OnStreamEvent' not found` 崩
  （KD-055-4② 同族）,user turn 未达后端。该两族 KD 明归上游 SSE 桥专项,
  非 536 范围;T2 修复在崩溃场景仍保证已落盘状态的重渲染（画布不再冻结）。
  **题3 注记**：native 双口径已修（单测证）;musk 气泡标签仍错系 musk 侧
  msgTimeLabel 的 `createdAt*1000` i32 回绕（VM int 宽度债,归 PLAN-057）
  ——T4 已使 Date.format 接受裸秒,musk 侧去 ×1000 的一行式消费修复归 T7
  musk worktree 触点。KD 059-FU1 核销回写待 T7 后由 review/merge 执行。
- [✅ 已完成] **T6** absolute 定位原语(题5)：①②**根修=hoist 臂挂上 tracked
  主渲染路径**——勘察实证 Plan 409 的 col hoist 只活在未追踪 convert_column,
  真实视图(build_with_debug_gated)从不过臂(="absolute 类 VM 不消费"实录根因);
  补 convert_column_tracked_ctx + convert_row_tracked_ctx 双臂+未追踪 row 臂;
  extract_absolute_position 扩 button/mouse_area 载体。**语义定界**：
  absolute+偏移+z-N → 父容器 overlay 层(renderer stack+opaque 先例已在位,
  锚=父 bounds,不占兄弟流内格);absolute 无 z → 分层技巧(p051-min-ta
  textarea 叠加族)保留流内(负面测试锁定);③renderer Overlay 臂无需改
  (3948 popover 先例原样);④mouse_area 载体进 hoist 白名单。验证：语料
  test/ui/plan536_absolute 三断言 RED→GREEN(tailwind 刻度口径 top-2=8px);
  gallery 增 /absolute 探针页三 demo(route+nav 注册),worktree 二进制实机
  渲染证(floating badge/×/Card title 全出,截图 scratch/p536_t6_absolute_probe.png);
  UI 切片 364/366(2 红=master 存量)。
- [✅ 已完成（含边界注记）] **T7** musk 联测：musk worktree
  `.wt/lang-536/auto-musk`(auto-lang-536-dev,58df8a9)——①会话卡 × 类串
  对齐 VM absolute 语义（top-1/2/-translate → top-2+z-10）;②msgTimeLabel+
  relayFormatTimestamp 裸秒直传（×1000 i32 回绕根除;旧二进制一程实机证
  09:45:23==后端本地时,KD "08:34 vs 19:54" 消除）;③settings z-100→z-50。
  auto-lang 侧：convert_button 结构子件 hoist 臂（musk 会话卡 × 的直接
  路径,builder 单测绿）+ vue/ts_adapter Date.format 1e11 双口径守卫。
  **边界注记**：× 像素终验受 KD-048-a 窗口布局卡死阻,留用户实机目验;
  VM 偏移模型为 px（bottom-full/top-1/2 百分比不支持）→ settings 浮位
  暂不合,记 spec 边界;新二进制标签 +42043s 恒偏异常见待澄清 #4;
  KD 059-FU1 核销回写归 merge（musk 仓 bookkeeping）。
- [✅ 已完成（命题证伪+新缺陷立案）] **T8** fallback 路径 open 绑定根修
  (题6)：组件层探针（test/ui/plan536_modal,三变体隔离）——**原命题证伪**：
  unknown-tag fallback 包家族根 v1 全绿（open 绑定解析无断链,翻转即
  Modal(open=true)）;但加子件后根写失效暴露**新引擎缺陷**（待澄清 #5）。
  musk 题6 真因改判：DeleteConfirmDialog 的 use.web 导入直指 .vue→VM 轨
  落 stub 无视图（非 bindings 断链）——T9 前置已修（端口链三件套落 musk
  worktree,commit 见 T9 行）。受阻测试 #[ignore] 在案（引擎缺陷修复后
  转正）。子件字段逐帧默认值重播种打回 handler 写入的机制缺陷亦已修
  （prepare_child_render_state 4/4b 缺字段守卫）。
- [✅ 已完成（前置就绪,实机受阻）] **T9** musk 联测(题6)：端口链三件套
  （delete_confirm.at 约定/vm.at 真源/web.at facade）落 musk worktree
  （auto-lang-536-dev）,chats_view 导入换接;子件同形探针
  （p536_t8_child_widget_root_alert_dialog_resolves_open）全绿=vm.at 形态
  机制成立。**实机验收受阻**：复跑实例窗口布局卡死（KD-048-a 族,window
  size zero）,AskDelete→模态弹出+派发终验留用户实机;通过后 musk 侧退役
  内联确认行（059 T9 待澄清⑨ 处置）。

## 测试设计

- 本仓：三题各配 iced/vm 单测或探针断言;全量 lib（--features ui-iced）不劣于基线。
- musk 侧联测：沿 KD 059-FU1 的实证配方（发送消息→6 秒回复→免重选直显）。

## 验收标准

1. 探针三题全绿;musk 实机 PollStream 场景免变通直显。
2. musk 单会话期 Init 调用数个位级;LoadSessionList 无 per-render 重入。
3. Date.format 样本（1788436450 等）本机时区正确。
4. absolute 探针页通过;musk 会话卡 × 悬浮无遮挡(用户目验)。
5. 子件 prop 约束有明确语义结论并写入 docs(spec 或 KD 出口)。
6. musk KD 059-FU1 核销回写。
7. 题⑥：musk 上下文 alert-dialog 随 open 真渲染;gallery 双页回归不劣化
   （T8-T9）。

## 待澄清事项

1. 【T1 已定案】题 1 与 `when` 门的关系：门是**派发前过滤**（fire_timer
   内对根态求值,假丢弃本拍）,订阅层消息无条件到达 update——when=false
   时仍见 [UI_EVENT] 与门语义不矛盾（p536_t1_when_gate_is_pre_dispatch_filter
   探针锁定）。
2. 题 2 是否与 PLAN-526 T23 的 interaction(Idle)/hover 收口同根（渲染循环
   结构性重跑）。
3. 【题⑥ 增补 2026-09-04】T8 与 T6 的实施顺序：两者同动 aura_view_builder
   （overlay 层/绑定解析）——建议同批或紧随;若 T6 落地后本题表现形态变化
   （浮层挂载点改变等）,以 T8 的组件层探针重新定案再动根修。源实录与全部
   复现证据:musk docs/plans/059-vm-overlay-infrastructure.md 待澄清⑨;
   兜底现状:musk auto-musk-dev-1 已返场内联确认行（web 轨不受影响）。
4. 【T7 实测增补 2026-09-04】新二进制（T6 hoist 臂入列后）musk 气泡时间
   标签现 **+42043s 恒偏**（三 turn 同偏,20:06:06/26/07:06 vs 后端
   09:45:23/43/46:23;旧二进制同 musk 源=09:45 正确）——偏移非时区整倍,
   机制未明;T6 增量仅视图树重排（hoist/载体白名单）,不触 state/求值路径,
   与该值异常的因果链待查。旧二进制（T2-T4）+裸秒直传已证标签正确
   （run A）,T4 native 单测全绿。复现证据:scratch/p536_t7_snapshot*.txt、
   p536_t5_musk_vm.log。
5. 【T8 发现 2026-09-04·引擎缺陷立案】**存在带 model 的子件时,根 widget
   handler 对自身模型字段的 SET 写不落盘**。复现：test/ui/plan536_modal
   三变体隔离——①fallback 包家族根独占=Flip 落盘 ✓;②+ChatsLike 子件
  （model 含字段）=App.Flip 执行 Ok（handler_App_Flip 无 Err）但 root_open
   恒 false;③换 col 包裹（无 fallback）同败=子件在场即触发,fallback/
   reseed 守卫均非因。子件自身 handler 写入（ChatsLike.Flip）正常。疑似
   codegen 字段槽解析在根+子件模型合并编译时错位（SET_FIELD index 化）。
   修复需引擎级排查（vm/codegen 字段解析序）,超出本计划范围——立案转
   专项;受阻测试 p536_t8_unknown_tag_fallback_resolves_open_binding
   #[ignore] 在案,修复后转正。**连带影响评估**：musk 现网 App 层 handler
   少、模型字段写入多在子件域,故未成大面积现场;但 012-stopwatch 等根
   计时器写法（根 handler 写根字段）在带子件工程中疑似同雷,建议 review
   阶段评估普适度。
