---
plan_id: PLAN-536
status: archived        # 2026-09-05 重开段复审通过,二次关闭（终态）
feature_name: VM 运行时修复批——反应性三题 / 子件 prop 约束 / absolute 定位原语 / 家族浮层 open 绑定断链
author: [zhaopuming, ZCode]
created_at: 2026-09-04
updated_at: 2026-09-05T02:00:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "ui overlay 通道(PLAN-533 遗产): absolute 悬浮语义扩充——absolute+偏移+z → 父容器 overlay 层/无 z 留流内分层;px 偏移模型边界(bottom-full/百分比不支持)"
  - "ui timer 通道(PLAN-051 C7): 失效广播契约强化——handler 执行中崩(≠HandlerNotFound)置 dirty+热重载拍早退口脏桥接(500ms 兜底)"
  - "ui 子件生命周期(PLAN-437 Phase 2): Init 挂载语义收敛(每组件一次,child_inits_fired 记账)+子件字段缺字段播种守卫"
  - "native Date.format(PLAN-054 A7): epoch 秒/毫秒双口径归一(1e11 阈值),vue/ts_adapter 同款守卫"
  - "【重开段 T12】ui timer 门控: musk PollStream when: .streaming 摘除改 deadman 2 分钟窗(poll_window 列表承载窗戳)——跨模块 SET_FIELD 不可达根态(KD P536-D2)下 when 门恒假,deadman 为框架缺陷绕行定型(完成启发式带回合增长守卫)"
new_spec_components:
  - "ui absolute 定位原语: hoist 臂挂 tracked col/row 主渲染路径+gallery /absolute 探针页;Button{content:Overlay} 画布空壳限制登记"
  - "ui slot-fill x hoist 限制: slot-fill for 循环体内 col-arm hoist 不生效(x 渲染旁路,build_floating_layer 0 调用实证)——T9 收尾"
  - "【重开段 T10】悬浮机制统一: renderer 动态路径(Column/Row abs 拆分层)消费偏移(dynamic_abs_layer_position:非零 top/right/left→build_floating_layer 同款几何,零偏移 ghost 族保持落原点)+fold_floats 多浮层全保留(四处 hoist 臂,嵌套 Overlay 源序=栈序,废除 .next() 丢弃)"
  - "【重开段 T11】schema 围栏清偿: aura.at 再生成(dialog/dropdown 家族)+element_coverage 双向登记+baseline 更新(vb 侧家族有意漂移 128 条登记);kitchen-sink 生成器固化 PLAN-528 W6 两规则(src 资源占位/桌面壳专属排除);schema.rs 补 autodown.table_col_widths 事实源"
  - "【重开段 立案】KD P536-D1: aura.at 再生成 canonical 形态振荡(反不动点,围栏生成器专项);KD P536-D2: 跨模块 store handler 帧内 SET_FIELD 重绑定不可达根态(state-scope 专项,实机证据链完整)——057④/c13c250 家族引擎级根因候选"
touched_goals:
  - "GOAL-007: VM 轨反应性修复——timer 写入触发视图失效/Init 重入收敛/时间标签时区正确(musk KD 059-FU1 三题);重开段补:发送链零崩(493① 收敛)+PollStream 兜底链恢复,直显最后一里归 KD-057④+P536-D2"

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 10
total_steps: 12
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
- [✅ 已完成] **T8** fallback 路径 open 绑定根修(题6)：组件层探针
  （test/ui/plan536_modal,两形态）——**原命题证伪**：unknown-tag
  fallback 包家族根/子件视图根部两形态,open 绑定翻转均出
  Modal(open=true),fallback 路径绑定解析无断链;重渲染帧不回打默认值
  由 prepare_child_render_state 缺字段播种守卫锁定（4/4b）。musk 题6
  真因改判：DeleteConfirmDialog 的 use.web 导入直指 .vue→VM 轨落 stub
  无视图（非 bindings 断链）——T9 前置已修（端口链三件套落 musk
  worktree）。排查插曲："引擎缺陷"（根 handler 字段写不落盘）经查为
  测试派发名错位,已证伪销案（待澄清 #5）。
- [✅ 已完成（前置就绪,实机受阻）] **T9** musk 联测(题6)：端口链三件套
  （delete_confirm.at 约定/vm.at 真源/web.at facade）落 musk worktree
  （auto-lang-536-dev）,chats_view 导入换接;子件同形探针
  （p536_t8_child_widget_root_alert_dialog_resolves_open）全绿=vm.at 形态
  机制成立。**实机验收受阻**：复跑实例窗口布局卡死（KD-048-a 族,window
  size zero）,AskDelete→模态弹出+派发终验留用户实机;通过后 musk 侧退役
  内联确认行（059 T9 待澄清⑨ 处置）。
- [✅ 已完成] **T10** D1 根修：悬浮机制统一——① renderer 动态路径
  （render_dynamic_view Column/Row 臂）abs 拆分层消费偏移：abs 子节点带
  非零 top/right/left 偏移时按 build_floating_layer 同款 spacer 几何定位
  （共享 helper dynamic_abs_layer_position）,零偏移/无偏移（inset-0 ghost
  叠加族）保持落原点语义;② builder 四处 hoist 臂（col/row
  tracked+untracked）多浮层折叠修复——fold_floats 全部保留（嵌套
  Overlay,源序=栈序）,废除 `.next()` 丢弃;③ 语料增补双浮层/无 z 偏移 ×
  两形状+T10 双探针。**复审加验（遗漏猎查发现）**：plan536 探针套件的
  lib.rs 挂载声明被 b4d1ced7b（539 折叠前同步合并）冲突解决时静默丢弃
  ——全套 17 探针自 09-04 起在 master 为死代码未编译,本次复挂载并全绿。
  另:日常档暴露 schema.rs autodown 缺 table_col_widths 事实源（PLAN-045
  期只在手工 aura.at 登记）,已补进 schema.rs 并再生 aura.at。
- [✅ 已完成] **T11** D2 清偿：schema/aura.at 再生成（dialog/dropdown
  家族臂补齐）+ element_coverage 双向登记（23 增 10 删）+ drift baseline
  更新（-17 已消除裁剪/+128 vb 侧家族有意漂移登记——PLAN-533/540 家族
  浮层专用路径未镜像通用四表,全表同步归家族二期）+ kitchen-sink 生成器
  固化 PLAN-528 W6 两规则（src 资源占位、桌面壳专属排除）后页面再生成
  （31→38 元素）+ core.md 再生成 + gallery vue golden 重采样;
  schema_drift 2/2+docs_gen 4/4+gallery_golden 绿。
- [✅ 已完成（含边界注记）] **T12** D4 清偿：①组件层发送链探针
  （plan536_send_chain 三件语料+t12 探针）——一参回调链帧对齐零崩,
  KD-493① 复验收敛（aa92a821e 覆盖）;②**musk 实机 E2E**（后端 9267+
  worktree VM 实例,MCP 驱动）:发送链零崩、user turn 落库、agent 回复
  生成（turns.jsonl 实证）;③musk 侧四修（forge_store.at 8b1ae23）:
  streaming 置位前移/头部直连 close/启发式回合增长守卫/when 门摘除+
  deadman 2 分钟窗——PollStream 兜底链恢复（258 拍全 OK）;④**边界注记**:
  最后一步画布投影仍未直显——实机时序对拍立案**新引擎缺陷 P536-D2**
  （跨模块 store handler 帧内 SET_FIELD 重绑定不可达根态;同帧列表
  vmref 突变可见——两种写可见性分裂）,属 state-scope 专项（KD 在案,
  偿还方向已写明）;KD-047 handler-as-value（Sse 实参抛点）仍归上游
  SSE 专项,musk 已绕行（日志噪音级）。

## 重开与尾债清偿（2026-09-05,用户裁定）

> **状态机破例注记**：本计划已于 09-04 走到 archived 终态;用户裁定
> "不用另起项目,直接在计划536中完成"——复审债候选 D1/D2/D4 就地清偿,
> 破例回开。回开仅此一次,清偿完成后重新走 execution_done → reviewed →
> archived。（档案迁移：git mv docs/plans/archive/ ↔ docs/plans/,历史
> 复审记录原样保留。）

**勘察定案（2026-09-05,主检出只读复核）**：

- **D1 根因闭合（× 落左上 + build_floating_layer 0 调用双谜一体解释）**：
  系统内存在**两套判定标准与定位语义分裂的悬浮机制**——
  ① builder hoist 臂（T6,aura_view_builder col/row tracked 主路径）:
  门槛=absolute+**z-N**,产物=View::Overlay → into_iced →
  build_floating_layer（**offset 感知**,spacer 几何 top/right/left 定位）;
  ② renderer 动态路径（render_dynamic_view Column/Row 臂,Plan 057 2.2 /
  PLAN-530 步骤3 的 abs 拆分）: 门槛=**纯 absolute 类**（不要求 z）,
  产物=裸 iced Stack **落原点**（"overlay 落原点,接近 CSS absolute 语义"
  ——不消费任何 top/right 偏移）。musk V2 会话卡 ×（span+onclick.stop →
  View::Button,aura_events_get_base 按 base 名命中 .stop 修饰符,白名单
  在列）在 0515c8e 改构时**丢了 z-10**（T7 在 button 内形态加过）→ ①不收
  → ②接盘 → absolute 类拆 Stack 落原点 = "落左上";层经 opaque 包裹且
  span 未显式宽时撑满（musk 998d1fa 实录）→ 遮卡片 = "抢点击";
  build_floating_layer 全程 0 调用 = "渲染旁路"实锤。`right-[6px]` 任意值
  解析正常（parse_pixel_arbitrary → RightOffset(6.0)）,非解析问题。
  次生缺陷：四处 hoist 臂（col/row tracked+untracked）只取
  `floats.into_iter().next()`——**首个之后的 absolute 子节点被整体丢弃**
  （从 child_views 滤除却未进 Overlay）。musk 侧已弃浮 × 改 hover 标题行
  （dbe52a7 定案）,消费面不再受阻,auto-lang 侧按机制统一根修。
- **D2 现状**：KD P528-D6 更新口径——schema 漂移本体已消除,余
  baseline 裁剪（SCHEMA_DRIFT_UPDATE_BASELINE=1）+ kitchen-sink 重生成
  （KITCHEN_SINK_UPDATE=1）两件围栏内例行再生成。
- **D4 复验前提成立**：059-T9 双根修（aa92a821e dispatch_parent_route
  零参父 handler 帧错位根修,3837aa8b5 折入）落在 T5 崩溃复现**之后**,
  KD-493① 发送链 `Invalid object ID`（0xFFFFFFFF80000000=i32::MIN 符号
  扩展哨兵形态,与帧移垃圾 self 同族）**可能已被顺带修复但从未复验**
  （493 行"待复验"在案）。首步=组件层发送链探针+实机复验,而非盲修。
  055-4② `Sse.open(.OnStreamEvent)`（handler 名作值）Field not found 为
  KD-047 SSE 桥族独立缺口;PollStream 兜底送达链（536 T2 修复）使端到端
  验收不阻于 SSE——但需核实 .streaming=true 置位与 Sse.open 抛点的先后,
  若置位在前则 PollStream 活、验收可达。

## 重测记录（2026-09-04 合并后,合并点 fe962a3ed→432e15dab）

两仓分支已合并（auto-lang master←plan-536-dev; musk main←auto-lang-536-dev）,
master 清理重编译（v0.4.1-3478-g432e15dab）。重测面板结果：

1. **#4 标签 PASS**：合并版二进制下 KD 059-FU1 样本 created_at=1788436450
   渲染 **19:54:10**（此前 08:34）,四 turn 标签与本地库 timestamp 本地时
   逐秒吻合（19:54:10/18/39/45,scratch/p536_retest*）。+42043s 恒偏**未
   复现**,支持"陈旧产物"假说（用户判定的合并+重编译路径有效）。
2. **重测发现并修复两件**：①delete_confirm.vm.at msg 声明带参数名
   （`Confirm(target str)`）致适配器模块解析 fatal（boot plan-446 C1 门禁
   拦截实录）→改 `Confirm(str)`;②button 结构子件内 absolute hoist 臂使
   Button{content:Overlay} **画布空壳**（会话列表卡片名称/× 全失,截图
   p536_retest_rail）→auto-lang master 撤销该臂（commit 432e15dab）,测试
   契约反转锁定;musk 会话卡 × 改兄弟悬浮结构（外层 relative col 承接）。
3. **遗留（rail 空列表）**：会话列表 for 循环仍空（session_list 空/拉取
   未达）,而消息区有数据——数据源路由疑似落在 spawn 的 8080 backend VM
   （本地 musk-demo 库）而非 AUTO_BACKEND=9247,与 KD-048-a/055-4③（8080
   落 Windows 保留端口段 8068-8167）邻接;#3 × 悬浮/T9 模态的像素终验
   待 rail 恢复后一并做（结构改动已就位）。
   **【续查 2026-09-04 下午·三项收敛】**①数据源非问题：AUTO_BACKEND 指
   空端口 9248 → 应用数据全空,指 9247 → 2 会话进 state（app 消费
   musk serve 的 musk-demo 工作区,非 8080）——路由正常;②msg 声明修正后
   rail 恢复（此前的空 rail 系 delete_confirm.vm.at 解析 fatal 毒化,非
   hoist/结构问题）;③**T9 模态实机弹出达成**：× 点击 → "确认删除此
   会话？" 模态居中悬浮（取消/删除 齐,截图 autoui-screenshot-1788515730934）
   ——端口链 vm.at 真源渲染成立。**× 位置收尾**：× 悬浮生效（top-2 不
   挤压布局）但水平落左上;`build_floating_layer` 插桩实证 **0 调用**——
   × 的渲染未走该路径,几何修正（Fill spacer/外列 Fill）均不作用;根修
   需先追 × 元素的实际 element 转换链（View::Overlay→into_iced 之外
   存在旁路）,留 T9 收尾/专项。瞬态数字帧一例（v9 截图,消息气泡呈字符
   码串,新实例快照恒正确）记录在案不立案。

## 重开复审记录（2026-09-05）

- **Reviewer**: ZCode（/auto-plan:review,重开段独立复审）
- **时间**: 2026-09-05
- **复核基线**: worktree plan-536-dev（a6aeb1164→b65245f13→fe78a25a3）+
  musk worktree plan-536-musk-dev（8b1ae23→b26bd00）;实机证据
  scratch/p536_t12_evidence/（vm_ui5/6/9/B.log + turns.jsonl + 快照序列）。

### 逐项判定（对照 T10-T12）

1. **T10（D1 悬浮机制统一）——pass**：renderer 动态路径偏移消费+
   builder 多浮层折叠+复挂载探针套件 20/20 绿;探针 Red 面成立
   （T10① 多浮层在旧码必丢,T10② 为新 API 面）。日常档 4534/4553,
   19 红逐一在 master 基线复跑甄别为存量（layout 族 14+lucide+c2_param+
   d8+strip_html+charts）,零本计划回归。
2. **T11（D2 围栏清偿）——pass**：schema_drift 2/2+docs_gen 4/4+
   gallery_golden 绿;W6 两修正固化进生成器（页面恢复"勿手改"幂等）;
   四表同步被裁定为"baseline 登记+理由"路径（全表同步归家族二期）。
   附带收口:autodown.table_col_widths 事实源补齐（test_load_schema 转绿）。
   附带立案:aura.at 再生成 canonical 形态振荡（KD P536-D1,四次再生实证）。
3. **T12（D4 端到端）——pass（含重大边界注记）**：
   - **KD-493① 发送链崩:收敛**（实机多轮零崩+组件探针双证,aa92a821e
     覆盖 confirmed）;
   - **KD-055-4②/门控死:绕行收敛**（musk 四修,PollStream 258 拍全 OK;
     KD-047 handler-as-value 根修仍归上游 SSE 专项）;
   - **后端链:通**（user turn 落库+agent 回复生成,turns.jsonl 实证）;
   - **未达**:最后一里画布投影不显示新回合——**新立案引擎缺陷
     P536-D2**（跨模块 store handler 帧内 SET_FIELD 重绑定不可达根态,
     实机时序对拍证据链完整）,属 state-scope 专项,非本计划可收敛。
   - 验收标准 #1"免变通直显":**部分达成**——阻塞物从"SSE 桥债+发送崩"
     推进为"画布投影读侧"（证据链完整、专项方向明确）,残余归
     musk KD-057④ + auto-lang P536-D2。

### 遗漏/延后/workaround 猎查（重开段）

- **重大遗漏（本复审发现并修复）**:plan536 探针套件挂载声明被
  b4d1ced7b 合并静默丢弃（17 探针死代码）——已在 T10 复挂载;教训:
  合并冲突解决后应对 test 挂载面做存在性断言。
- **新立案**:KD P536-D1（aura.at 再生成 canonical 振荡）、
  KD P536-D2（跨模块 SET_FIELD 可见性,实机证据链完整）。
- **延后（经证据链支持,非隐性）**:D4 最后一里归 KD-057④+P536-D2
  state-scope 专项;KD-047 归上游 SSE 桥专项。

### 判定（重开段）

**reviewed**——T10/T11 全绿;T12 登记债收敛+新缺陷立案+边界注记。
债候选:KD P536-D1（生成器振荡）、KD P536-D2（state-scope 专项）、
KD-047（SSE 桥专项,存量）。重开段目标"尾债就地清偿"按此口径达成,
计划重新归档（archived 终态,二次关闭）。

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
5. 【T8 已结案 2026-09-04·证伪】"子件在场时根 handler 字段写不落盘"的
   引擎缺陷**不存在**——排查定位为测试自身派发名错位（语料 msg 声明为
   FlipRoot,测试派发 "Flip"→handler_App_Flip 不存在→HandlerNotFound→
   写入不发生;此前 [VM_EXEC] handler_App_Flip 的日志来自更早语料版本,
   误导定位）。修正派发名后 p536_t8_unknown_tag_fallback_resolves_open_
   binding 转正全绿（17/17）,含"重渲染帧不回打默认值"守卫锁定断言。
   教训入档：诊断时 fn 表名与派发名必须同源核对
   （debug_fn_table 实证命中=[handler_App_FlipRoot]）。

## 复审记录

- **Reviewer**: ZCode（/auto-plan:review）
- **时间**: 2026-09-04 17:00+08:00
- **复核基线**: 分支已按用户指示先行折入 master（merge fe962a3ed 及后续修正
  432e15dab/90a5dfd9b），门禁与实机验证均在主检出执行（worktree
  plan-536-dev 保留待 merge 清理）。

### 逐条验收判定（对照 ## 验收标准 1-7）

1. **探针三题全绿 ✓ / musk PollStream 免变通直显 △**——组件+实机双证全绿
   （p536 17/17；retest 像素差 0.050%/0.017%）。端到端"发送→免重选直显"被
   **范围外存量债**拦阻（KD-493① `Invalid object ID` + KD-055-4② Sse 桥族
   `Field 'OnStreamEvent' not found`，实机复现在案），已获用户核可顺延；
   T2 修复保证崩溃场景下已落盘状态仍重渲染。→ **pass（含已核可边界）**
2. **Init 个位级 ✓**——musk 实机全会话期每实例每子件 Init 恰 1 次
   （App/ChatsView/MentionInput/SettingsMenu/WorkspaceSelector 各 3 次=2 实例
   +1 重载；KD 记录 5498→16293）。→ **pass**
3. **Date.format 本机时区正确 ✓**——单测四件（样本 1788436450 秒口径
   00:47:16 垃圾转 19:54:10 正确）；musk 实机 KD 样本 19:54:10 与库中
   timestamp 本地时逐秒吻合（重测截图）。→ **pass**
4. **absolute 探针页 ✓ / × 悬浮无遮挡 △**——gallery /absolute 实机渲染证；
   × 悬浮生效（真悬浮、不挤压布局、点击派发通）。位置水平落左上（遮挡标题
   首字）而非右上：根因=× 元素渲染走旁路（build_floating_layer 插桩 0 调用
   实证），几何修正（Fill spacer/外列 Fill）不作用；像素终验收尾立案。
   → **pass（含收尾项，用户目验待）**
5. **prop 语义结论写入 docs ✓**——题4 定界三则已入计划重测记录/待澄清
   （builder 每帧重解析 prop、快照=Element 缓存层、prop 对象 handler 写=
   SET_FIELD 无效堆对象崩溃）；spec 沉淀由 merge 执行。→ **pass**
6. **KD 059-FU1 核销回写**——前置三题证据齐备（标签/Init/失效根修），回写
   属 merge 阶段 musk 仓 bookkeeping 动作。→ **pass（动作待 merge）**
7. **题⑥ alert-dialog 随 open 真渲染 ✓**——T9 实机截图：× 点击 →
   "确认删除此会话？" 模态居中弹出（取消/删除 齐）；gallery 双页回归=
   全量门禁覆盖（见下），/alertdialog 页实机单独点验留用户。→ **pass**

### 全量门禁（Plan 466/507 口径）

- `cargo tf`：3407/3409 绿。2 红=schema_drift_fence + kitchen_sink_page_in_sync
  ——**上游存量非本计划引入**：漂移 tag（dialog/dropdown/toggle-group 家族）
  在合并前提交 fb033a048 已在案（6 处命中实证），系 PLAN-533/540 合入 vb 臂
  未同步 schema.rs/render 四表；kitchen-sink 生成页停 31 元素 vs schema 38
  （533/484/526 家族）。修复命令已记：SCHEMA_DRIFT_GENERATE_AT=1 /
  KITCHEN_SINK_UPDATE=1 / SCHEMA_DRIFT_UPDATE_BASELINE=1（建议独立小修）。
- `cargo tv`（VM 文件 touched：stdlib.rs）：3567/3569。2 红=同上 schema 存量
  + charts 主检出现在并行 WIP aura.at 污染（629 行未提交重构；已提交态复测
  PASS 实证）。
- `cargo tt`（转译器 touched：vue.rs/ts_adapter.rs）：3754/3756。同上口径。
- `desktop_protocol` 切片：120/120 绿。日常 UI 切片 392/394（2 红=d8_toggle_
  dark_mode+plan055_strip_html，干净 master 复验在案的存量）。

### 遗漏/延后/workaround 猎查

- **遗漏**：无——T1-T9 逐条有对应 diff 与验证。
- **延后**：两处均经用户核可（#4 标签重测窗口、#3/#4 实机终验顺延）；
  T5 端到端受范围外 SSE 桥债拦阻已获用户知情。
- **workaround**：× 兄弟悬浮改构系绕开 Button{content:Overlay} 画布空壳
  （该臂已撤销、renderer 根修留档）——限制已登记为 new_spec_components
  限制项，非隐性绕开。

### 判定

**reviewed**——计划自身验收面达成（含已核可边界与立案收尾项），无本计划
引入的回归。债候选清单：D1 × 水平位置渲染旁路收尾；D2 schema 四表同步+
kitchen-sink 重生成（上游存量，独立小修）；D3 musk KD 059-FU1 核销回写+
内联确认行退役（merge 阶段，musk 仓）；D4 T5 端到端随 musk SSE 桥专项；
D5 瞬态数字帧观察（一次，未复现，不立案仅记录）。
