---
plan_id: PLAN-505
status: archived                # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: desktop-debt-batch-1
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "schema/projection-protocol-v1.md: v1.4 → v1.5——字段表增量（__wm_wins.pager 旗标 / __wm_workspaces.more 溢出标签，宿主派生、指纹零扩展）+ 变更记录段"
  - "crates/auto-lang/assets/shell.at: 任务栏 top/bottom 双分支形态 → flex-col-reverse 数据级翻转单份 + __dock_border 宿主投影边线（440→308 行）"
  - "crates/auto-lang/src/ui/osconfig_daemon.rs: daemon 发现序两级 → 三级（PATH which 兑现，P501-1 扩展位落地）"
new_spec_components:
  - "实机验收通道——autoui_desktop MCP 注入工具（DesktopInject 队列：Bus 同臂消费/特权 App handler 直呼，五面 shell|settings|notification|launcher|desktop）+ AUTOUI_ACCEPTANCE=1 门控 + ServiceTick 排空；ADR 规程 docs/plans/reports/505-acceptance-channel.md + 统一驱动入口 autoui-verifier/scripts/acceptance_channel.py（drill/p487/p496/p501 四场景）——CUA/OS 注入受阻族的 in-process 统一解"
  - "事件泵排空拾取 drain_slot_events（drain-while-empty + MoveSizeStart/End 稳定分区前置）——NativeSlotEvents 批消息形态"
  - "on_dnd_finished 发起方锚定——拖出会话代号（DRAG_SESSION_GEN）+ DM::App dispatch 环比对 + dnd_finished_target 交付序"
  - "broker 显式停机 shutdown_broker（旗标+探测连接唤醒，五退出点接线）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——交互时序（事件泵批化）/shell 面五瑕疵清偿/实机验收通道（受阻族统一解）"

affects: [auto-lang/ui]
current_step: 9
total_steps: 9
---

# [PLAN-505] 桌面 DEBT 批处理一期——交互时序 + shell 面瑕疵 + 验收受阻族统一解

## 变更摘要

桌面特性线收官后的**债务集中清偿**（用户裁定的"桌面线完成后统一清"策略
兑现）。从 KNOWN-DEBT 及各归档计划登记中**裁剪**出四族可执行项，一次计划
批量清偿；环境依赖项（物理机复验）、决策项（真洞翻默认）、语言/VM 线缺陷
明确**排除在外**。

| 族 | 内容 | 债务号 | 价值 |
|---|---|---|---|
| **A 交互时序** | ① 事件泵 drain 修复（16ms 单发轮询 ≈62 事件/s → 每拍排空 + START/END 优先级出队，拖拽落位延迟秒级→帧级）② 快甩拖入首轮滞留 | 486 表格行（性能观感 🟡）/ P488-D3 | **最高**——直接影响 dock/拖放手感 |
| **B shell 面瑕疵** | ① shell.at 任务栏 top/bottom 双分支去重 ② pager 网格 ≤4 截断（计划定而未实现）③ window_thumbnail props 不透传 DOM ④ os-config daemon 发现序 PATH 级补全 ⑤ broker 停机旗标接生产调用点 | P487-3 / P497-1 / P497-2 / P501-1 / P480-R1 | 小而确定 |
| **C 实机验收受阻族** | "前台竞争/CUA 像素身份守卫阻断实机照"**一族统一解**：专用验收会话形态（守卫放行清单 / 无人值守窗口 / 驱动脚本统一入口），一次性解锁 P487-1、P496-1、P501-2 及 472/478/479 先例族的复验留痕 | P487-1 / P496-1 / P501-2（家族） | 方法论——终结"每计划一条受阻债"的滚雪球 |
| **D 增强小件** | ① on_dnd_finished 交付取完成时焦点 App ② 壁纸热切换（若证得"投影重注入+指纹天然支持"则顺路） | P488-D4 / 496 待澄清 | 低成本补全 |

**排除项**（登记但不入本期）：P494-2/-3 真洞物理机复验（环境依赖，等物理机
窗口）；真洞默认值翻转（决策项，待使用反馈）；P488-D1/-D2、P495-1/-2
（语言/VM 线缺陷与 tv 档卫生，归语言线专项）；P485-1（已有 TestLock 缓解，
维持观察）；P480-R2（口径文档项，随 500 复审顺带核对）；457 长线。

## 目标

- **G1 A 族**：拖拽松手到 dock 落位延迟从秒级降到帧级（drain-while-empty +
  优先级出队）；快甩拖入不再滞留首轮。
- **G2 B 族**：五项瑕疵逐项清偿，各自原有验证口径复跑绿。
- **G3 C 族**：形成可复用的实机验收通道（一份操作文档 + 一个放行机制），
  并用它把 P487-1/P496-1/P501-2 三条受阻债的实机照补齐留痕。
- **G4 D 族**：P488-D4 落地；壁纸热切换定案（做或明确不做+理由）。
- **G5 记账**：所有清偿项回写 KNOWN-DEBT 已清偿标记；排除项维持登记不丢失。

## 架构方案

无新架构——四族均为既有面上的修复/补全/流程件。A 族改
`ui/session.rs:1220` 段事件泵（drain + 分级出队）；C 族是**流程资产**（验收
通道文档 + 守卫放行配置），落在 `docs/` 与测试驱动脚本目录，复用
`.agents/skills/autoui-verifier/scripts/` 既有自动化入口。

## 技术栈

既有栈。零新依赖。

## 需求分析与背景调查

（KNOWN-DEBT 全量对账 2026-08-31：P47x–P50x 登记项逐条过筛）

- **A 族证据**：486 表格行（session.rs:1220 段 16ms 单发 try_recv；系统级
  LOCATIONCHANGE 噪声下 MOVESIZESTART/END 排队，实机松手→落位延迟数秒）——
  修复方向已在债务行写明（drain-while-empty + 可选分级）；P488-D3 合成拖拽
  实证的毫秒级快甩滞留同源。
- **B 族证据**：P487-3（shell.at v1 起双分支重复）、P497-1（计划文本定了
  ≤4 截断、实现漏了）、P497-2（a2vue 金样 SFC props 不透传）、P501-1
  （发现序 PATH 级留扩展位）、P480-R1（enable_broker 无生产调用点）。
- **C 族证据**：P487-1/P496-1/P501-2 三条同因（"OS 注入受阻变体/前台竞争/
  CUA 像素身份守卫"），472/478/479 各计划均有同族先例——每案单独绕的边际
  成本已高于统一解。
- **排程**：500（execution_done）待复审、503（视觉刷新）drafting 待领——
  本期与其零交叠（A 族碰 session 事件泵段、B 族碰 shell 资产与 registry
  小点，均不在 503 视觉面/500 协议面）。504 在途不涉。

## 详细设计

### 1. A 族：事件泵与拖入时序

- `native_dock_event_subscription`（session.rs:1220 段）改为**每拍
  drain-while-empty**；通道分级：MoveSizeStart/End 走高优先级通道（或同通道
  出队时优先级拾取），LOCATIONCHANGE 噪声降级；
- 指针采样节流保持 30Hz 不变（只改排空策略，不改采样率）；
- P488-D3：快甩（START→END 毫秒间隔）时 DragWatch 终态直接按 END 时指针
  判定（跳过中间 Over 态的滞留）。

### 2. B 族五小件

各 0.5 天内：双分支合并为参数化单分支（P487-3）；pager 行渲染加 ≤4+"+N"
（P497-1）；a2vue 生成器 props 透传补（P497-2）；发现序第三级 PATH 探测
（P501-1）；`enable_broker` 接 desktop 关停路径一处调用（P480-R1）。

### 3. C 族：实机验收通道（流程资产）

- 产出 ADR 短文：专用验收会话操作规程（什么状态可注入、守卫放行怎么配、
  失败回退）；
- 落一个放行配置/开关 + 用该通道补拍三条债的实机照（P487-1 齿轮开面板 +
  dock 热切换/Esc、P496-1 壁纸/图标交互、P501-2 齿轮→os-config 全链）；
- 驱动脚本归 `.agents/skills/autoui-verifier/scripts/`（复用入口不另起）。

### 4. D 族

- P488-D4：DoDragDrop 完成回注时取**发起时锚定 AppId**（不查完成时焦点，
  避开 VM 无焦点查询的缺口）——语义更稳；
- 壁纸热切换：探针验证投影字段重注入是否天然热刷新（指纹门控）；是→
  顺路做并实机照；否→一行"不做+理由"回写 496 债务行。

## 测试设计

1. **T1 A 族**：事件泵单测（注入 100 条噪声 + 2 条 START/END → 一拍排空、
   优先级序断言）；快甩用例（START/END 同拍到达 → 终态正确）。
2. **T2 B 族**：各小件既有验证口径复跑（shell 装载测/pager 用例/a2vue 金样/
   daemon 发现序单测/broker 单测）。
3. **T3 C 族**：通道规程演练一次全绿 + 三债实机照归档。
4. **T4 实机**：拖拽落位延迟体感复核（A 族）；其余随 C 族通道。

## 验收标准

1. A 族：注入式单测绿 + 实机拖拽落位延迟体感达标（秒级→即时）。
2. B 族五项各回写已清偿；T2 全绿。
3. C 族：规程文档 + 放行机制入库，三债实机照留痕回写。
4. D 族：P488-D4 单测+清偿；壁纸热切换定案成文。
5. `cargo t ui`、`cargo t session` 不回归；零警告；KNOWN-DEBT 记账零丢失
   （排除项仍在登记）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **A-事件泵**：`crates/auto-lang/src/ui/session.rs:1220` 段 drain-while-empty
   + START/END 优先出队 + T1 单测。
   验证：`cargo t session && cargo t native_dock`。
   [✅ 已完成] worktree 8c1227e0e：`drain_slot_events` 纯函数（drain-while-empty
   + sort_by_key 稳定分区 START/End 前置）；unfold 改一拍成批上行
   （`NativeSlotHwnd` 单发变体 → `NativeSlotEvents` 批，Disconnected 尾批
   照发 + 流终止自愈保持）；T1 `slot_pump_drains_noise_batch_and_prioritizes_gesture_markers`
   （100 噪声+2 边界一拍排空+优先序断言）绿——native_dock 29/29、
   session 80/80（ui-iced 档；注：`cargo t` 默认档不编 ui 模块，需
   `--features ui-iced`）。
2. **A-快甩**：`ui/native_dock/mod.rs` DragWatch 终态快路（END 即判）+ T1。
   验证：`cargo t native_dock`。
   [✅ 已完成] worktree 8c1227e0e：`end()` 原生即按 END 时指针判定（无
   Over 依赖），滞留根因在泵排队——批内优先分区使 END 先于噪声消费；
   T1 `dragwatch_fast_flick_same_batch_end_judged_immediately`
   （START/END 同批 + 零中间采样 → DockCandidate 即判、会话复位、残余
   噪声弃置）绿。
3. **B1 任务栏去重**：`crates/auto-lang/assets/shell.at` 双分支合并 + 装载测。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] worktree 029a5f7ea：双分支 ~150 行重复收敛为参数化单份——
   位置由容器 `flex-col-reverse` 数据级翻转（iced col_reverse Plan 412
   已有臂），任务栏边线 border-b/border-t 条件化；shell.at 440→308 行。
   装载测 desktop_mcp 3/3 绿（真 shell.at 装载 + pager hover 面不回归）。
4. **B2 pager 截断**：shell.at pager 行 ≤4+"+N" + 用例。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] worktree 029a5f7ea：机制裁定为债项两条路径中的"协议字段"
   （协议升 v1.5，零语言改动——take(N) 语言原语按"无新架构"约束不做，
   见待澄清④）——`__wm_wins` 条目增 `pager` 旗标（每分区 z_order 前 4）、
   `__wm_workspaces` 条目增 `more` 溢出标签 "+N"；派生在宿主投影环
   （指纹零扩展：派生面为既有 win 段纯函数）。新用例
   `shell_projection_pager_truncation_v15`（6 窗→4 旗标+more="+2"/空分区
   空串）绿；desktop_mcp 3/3、shell 族 8/8、投影族 32/32 绿。
   顺带修复 504 遗留：`tests/osconfig_integration.rs` LaunchSpec 缺
   `name`/`fit` 字段编译红（tf 档不含 tests/ 目标漏网，本步补 `name: None,
   fit: false`）。
5. **B3-B5 小件**：a2vue props 透传（`ui_gen/vue.rs` 生成段）；daemon PATH 级
   （`ui/osconfig_daemon.rs` 发现序）；`enable_broker` 生产调用点（session
   关停路径）。
   验证：`cargo t vue && cargo t osconfig_daemon && cargo t session`。
   [✅ 已完成] worktree 18f115b9d：B3——`generate_shadcn_attrs` catch-all
   补 props 排序遍历透传（原只透 class，wid/win/fallback_icon 静默丢弃），
   金样 window_thumbnail/virtual_window 更新，a2vue 14/14 绿零连带；
   B4——`which_in` 纯逻辑（unix 可执行位）+ `which_daemon` 生产包装接
   `ensure_ready` 注入缝，Offline 文案列全三级，22/22 绿（含 2 新用例）；
   B5——`shutdown_broker`（旗标置位+探测连接唤醒，幂等）接五个桌面退出点，
   新用例 `shutdown_broker_sets_flag_and_is_idempotent` 绿。组合档
   vue+osconfig+session+broker+stage3 376/376 绿。
6. **C-验收通道**：ADR 短文（`docs/design/autoui/` 或 plans/reports）+ 放行
   开关 + autoui-verifier 脚本入口。
   验证：通道演练一轮绿。
   [✅ 已完成] worktree 9d16ed37a（重写为 bfc93768c）：机制定稿为**内进程
   MCP 注入**（不再与 OS 注入缠斗）——`DesktopInject` 进程级队列（Bus 记录
   并入 shell `__desktop_cmd` 同臂消费 / 特权 App handler 直呼，五面
   shell|settings|notification|launcher|desktop）+ ServiceTick 排空 +
   `autoui_desktop` 工具（`AUTOUI_ACCEPTANCE=1` 门控，生产缺省拒绝）；
   ADR `docs/plans/reports/505-acceptance-channel.md`（规程/注入面/回退）+
   统一入口 `.agents/skills/autoui-verifier/scripts/acceptance_channel.py`
   （drill/p487/p496/p501 四场景）。单测锚
   `desktop_injects_flow_through_real_arms` 绿 + **drill 演练一轮绿**（真
   宿主 boot → 齿轮 handler 注入 → 设置面板实开实拍
   `assets/505/drill/drill-01-settings-panel.png`）。
7. **C-补拍三债**：P487-1/P496-1/P501-2 实机照归档回写。
   验证：三债行已清偿注记。
   [✅ 已完成] worktree e866bd447：三场景实拍全过并归档
   `docs/plans/reports/assets/505/`——P487-1（开面板/热切换置顶 border-b/
   Esc 自隐三帧）、P496-1（壁纸 #1e3a5f 实变 + 图标激活 calculator）、
   P501-2（系统分区徽标 + 外部仓 os-config App 实拉起整窗）。补拍实拍
   发现并修正 B1 遗留：taskbar 注册件 if 条件样式在 live 装配路径丢失
   （视图树单测两路皆绿、仅实机可见）→ 边线类改宿主投影拼接
   （`__dock_border` + apply_dock_edges_now 热同步）+ 回归锚
   `shell_root_col_position_classes_and_taskbar_present`。KNOWN-DEBT 三债
   行清偿注记随 S9 统一回写。
8. **D 族**：P488-D4（发起时锚定 AppId）+ 壁纸热切换探针定案。
   验证：`cargo t native_dnd && cargo t ui`。
   [✅ 已完成] worktree d85576abb：P488-D4——native_dnd 增拖出会话代号
   （每次 DoDragDrop 完成自增），DM::App dispatch 环前后比对锚定发起方
   （DoDragDrop 在发起方 handler 内联阻塞至完成），DndFinished 交付序 =
   发起锚定（取走）> 完成时焦点 > primary（v1 回退保持）；锚定判定纯
   函数化，用例 `dnd_finished_delivery_anchors_at_initiator`（偏差场景：
   焦点漂移仍交付发起方）绿。D-2 壁纸热切换探针**定案 = 天然支持**
   （投影重注入 + 指纹门控热刷新；p496-01 实机照即证——SaveWallpaper
   后同会话背景实变 #1e3a5f）。dnd+ui 档 836/836 绿。
9. **记账收尾**：全部清偿项回写 KNOWN-DEBT；排除项核对仍在；健康检查；
   状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] worktree b0053671b：KNOWN-DEBT 十二项清偿回写（486 性能行/
   P480-R1/P487-1/P487-3/P488-D3 同族治理注记/P488-D4/P496-1/P497-1/
   P497-2/P501-1/P501-2 + P496 壁纸热切换定案段）；排除项（P494-2/-3、
   P488-D1/-D2、P495-1/-2、P485-1、P480-R2）核对仍在登记。健康检查：
   默认档 `cargo check -p auto-lang` 零错；变更文件 rustfmt 偏差与基点
   对齐（osconfig_daemon 新增 1 处已归零，余为存量）；收官范围化组合
   （native_dock/session/desktop_mcp/shell/a2vue/osconfig/dnd/injects）
   159/159 绿。

## 复审记录

（2026-08-31，/auto-plan:review；worktree plan-505-dev @ 8c1227e0e..复审卫生修正）

**验收逐条核对**（verify, don't trust——全部在 worktree 内复跑）：

1. **A 族 ◐→pass（机制全证，实机复跑环境受阻已登记）**：注入式单测绿
   （`slot_pump_drains_noise_batch_and_prioritizes_gesture_markers` 等 3 用例
   + native_dock 29/29、session 80/80 复跑）。实机拖拽体感：t5_smoke
   SendInput 管线本会话被阻断（`caption 拖拽链 = false`，P504-3/P496-1
   同族环境限制——原生窗拖入 dock 本质要求对外部窗的真实 OS 输入，内进程
   通道不覆盖）；时延改善由单测数学背书：100 噪声+2 边界一拍排空 = 原
   ~1.6s+ 排队 → ≤16ms 泵送 + ≤400ms 执行节拍。留痕 P505-2（ADR §4
   边界注记）。
2. **B 族五项 pass**：P487-3（shell.at 440→308 行，desktop_mcp 3/3 复跑）/
   P497-1（v1.5 pager 面，`shell_projection_pager_truncation_v15` 复跑绿）/
   P497-2（a2vue 14/14 复跑绿，双金样含 props）/P501-1（osconfig 22/22
   复跑，含 2 新用例）/P480-R1（`shutdown_broker_sets_flag_and_is_idempotent`
   复跑绿）。KNOWN-DEBT 各行清偿注记在案。
3. **C 族 pass**：规程（`505-acceptance-channel.md`）+ 放行机制
   （AUTOUI_ACCEPTANCE 门控，生产缺省拒绝）+ 统一入口
   （`acceptance_channel.py`）入库；三债实机照七帧 + drill 一帧全部
   git 归档 `assets/505/`（复审补 drill 入库——原 ADR 引 tmp/ 未归档）；
   三债行清偿注记回写。补拍顺带实锤 B1 实机缺陷并修正（见 P505-1）。
4. **D 族 pass**：P488-D4 单测 `dnd_finished_delivery_anchors_at_initiator`
   复跑绿；壁纸热切换定案成文（P496 段尾注 = 天然支持，p496-01 实拍证）。
5. **门禁 pass**：`cargo tf` 3329/3329（含 1M churn 档）；ui-iced 档全量
   4227/4227（tf 盲区补跑，500 复审口径）；默认档 `cargo check` 零错；
   零新增警告（触及文件警告逐条对位——唯一落新增行的 duplicated #[test]
   为基点 18483-18484 存量双标记位移）；变更文件 rustfmt 与基点 hunk 数
   对齐；diff 零 TODO/FIXME/HACK。

**遗漏/延后/Workaround 扫描**：

- P488-D3 按"同族治理"注记清偿（native dock 拖拽臂实证）——OLE 拖入臂
  （DragEnter/WM_NULL 上膛）机制未改动，已在债务行明示余留观察（非静默
  延后；计划详细设计本就把 A-快甩定在 DragWatch 臂）。
- **P505-1（复审新登记）**：注册件 if 条件样式 live 装配丢失根因未修，
  B1 以投影拼接规避——workaround 已显式登记。
- **P505-2（复审新登记）**：实机拖拽体感复跑环境受阻留痕。
- 排除项（P494-2/-3、P488-D1/-D2、P495-1/-2、P485-1、P480-R2、457）
  核对仍在登记，零丢失。

**与计划文本的偏差（已裁定/记录）**：B2 机制取协议字段路径（待澄清④
成文）；边线类由 border 条件化改宿主投影拼接（P505-1 规避，S7 补拍
实拍发现）；504 遗留 tests/osconfig_integration 编译红顺带修复（tf 档
不含 tests/ 目标的门禁盲区，另行提示 merge 关注重复）。

**裁定：reviewed**——五条验收全 pass（1 条机制全证+环境受阻留痕），
无未批准的静默延后；债项 P505-1/P505-2 已登记。

> **簿记事故注记（merge 期补录）**：执行/复审的全部簿记（状态翻转/
> [✅]/复审记录/元数据）曾在主检出以未提交形态存在，被并行会话的清理
> 动作清回 drafting 基点；merge 会话自上下文逐字重建（内容与 worktree
> 提交链证据一致），本次 merge 提交将簿记一并入库固化。

## 待澄清事项

- C 族放行机制形态（守卫白名单 env / 专用测试窗标记 / 时段窗口）在 T6 时
  按实机环境定稿——原则：不改生产守卫默认行为。
- 壁纸热切换若探针结论为"需额外管道"则明确不做（本期不扩范围）。
- P488-D4 的"发起时锚定"若与 dnd_finished 既有消费者语义冲突，回退为债务
  维持（记录冲突证据）。
- A 族优先级出队若通道结构不便分级，允许退化为"纯 drain 不分级"（已消
  90% 延迟），分级转增强——以实测数据定。
- **④（S4 执行期裁定）B2 机制取债项两路径中的"协议 v1.5 字段"**（per-win
  `pager` 旗标 + `ws.more` 标签，宿主派生）而非 take(N) 语言原语：计划
  "无新架构——既有面上的修复/补全"约束 + 语言原语需 parser/视图求值/快照/
  ts_adapter 四面同步超出 0.5 天小件档；take(N) 若将来有通用诉求另立语言
  增强计划。

## spec-sync 回写记录

（2026-09-01，/auto-plan:merge 本仓扩展程序）

- `.autoos/specs.json` upsert：P505-1（reports）/P505-2（goals）/P505-3
  （architecture）/P505-4（designs）/P505-5（tests）/P505-6（reviews），
  `file` 指本归档路径，`related: [PLAN-505]`。
- module 回写：`docs/specs/auto-lang/ui/overview.md` 增「505 桌面 DEBT
  批处理一期」段、`ui/plans.md` 增 505 行（vm 面无改动不入）。
- 全局件：`docs/specs/goals.md` GOAL-009 引用补 505。
- 索引再生：`python scripts/spec-index.py` → `docs/specs/INDEX.md`。
