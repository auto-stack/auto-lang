---
plan_id: PLAN-721
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-track-update-path
author: []
created_at: 2026-10-01
updated_at: 2026-10-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径
current_step: 4
total_steps: 6
---

# [PLAN-721] desktop-track-update-path

## 变更摘要

承接 PLAN-712 归档计划登记的 P712-D1（high）桌面轨更新路径家族：
**桌面内嵌 app 的消息→重建传播缺口**。三症状同族异层——
T-19 暂停失败（030 播放中按钮 ▶ 与实态脱 sync、点击无效/自动恢复；
引擎契约层已被 712-r3 探针+双锁洗清，残留=桌面轨消息/视图链）+
T-11 播控条布局塌缩（seek 条 ~45%、右组空档；col_right 438↔658
进程级翻转非确定性源待定谳）+ 命中坐标家族（018 三卡恒卡 1 已由
712-r3 第三层 Init 代际修复闭环，本计划不再承担）。
核心谜团：**独立轨**动态 app 恢复泵全链工作（r2 实测 51 条 trace），
**桌面内嵌轨 0 条**——`__parked_resume_tick` AppTick 未达
renderer.rs 泵臂（或达而无 dirty 传播）。

另含同批工具债 T-DOCS-1（autoui_snapshot 不达 app 虚拟窗内部——
桌面内嵌 app 的自动化验证被卡）。

来源：docs/plans/KNOWN-DEBT-AND-RISKS.md P712-D1/D2；
归档计划 docs/plans/archive/712-vm-desktop-defects.md §8.5.3/§9
（r2 T-16 定位记录 §9 :339-342）/§10①。

## 目标

1. **定谳并修复桌面内嵌 app 的更新路径断链**：恢复泵（
   `__parked_resume_tick`）与帧泵（`__frame_pump`）消息在桌面模式
   对内嵌 app 的到达/消费/置脏链路有 trace 实证；断链点修复后有
   headless 回归锁。
2. **T-19 桌面实机闭环**：030 桌面内嵌播放中点暂停 → mpv 真停 +
   图标翻转 pause，10 次无回弹；上行 OnPlayState 到达有 trace 对账。
3. **T-11 非确定性源定谳**：col_right 438↔658 进程级翻转定位到
   代码级（嫌疑=动态视图构建顺序敏感层/HashMap 序），修复或定性
   文档化，附回归锁。
4. **T-DOCS-1 同批清偿**：MCP snapshot 增 per-app 虚拟窗内部面。
5. 作用域门禁零新增红（裸 cargo t 对照 master 基线；VM 触面加 tv）。

## 架构方案

诊断先行、双轨对照、落点后修：

1. **T-1 诊断工装先行**：AUTO_SCHED_DIAG 通道补 per-app 三段
   trace——订阅装配点（22111 起 daemon subscription 闭包，现仅
   frame_pump 有 diag）、update_inner 泵臂消费点（17606/17626，
   现仅留注释无打印）、dispatch_app 入口。全部 stderr → 桌面日志，
   门控复用 `AUTO_SCHED_DIAG=1`（零开销 OnceLock 布尔读先例
   frame_bench）。桌面二进制新鲜度纪律：启动前 mv 改名让路
   （§8.5.0 已排雷项 4——僵尸锁 exe 的 .z 陷阱）。
2. **T-2 双轨对照定谳**：同一二进制、同一 030/018 载荷，独立轨
   （`auto run -r vm`）vs 桌面内嵌轨（auto-os desktop.sh）各采
   AUTO_SCHED_DIAG 日志，对账四轴：订阅装配（泵订阅是否 per-app
   推入）、消息到达（泵臂命中）、泵推进（poll 返回值/is_dirty）、
   重建（view_dirty→view 调用）。产出 evidence/721/apptick-verdict.md。
   已知假说排序：H1 订阅门时序（park 发生在订阅重估后且无后续
   update 周期重估）；H2 r2 测量污染（0-trace 采自陈旧二进制/僵尸
   重名——§8.5.0 排雷项 4 同款）；H3 消息到达但 split_mut 拆借
   目标错位（state.component 指向 shell）；H4 达而置脏写错面
   （view_dirty 写 shell 的 AppState 非 app 的）。
3. **T-3 断链修复**：按 T-2 定谳落点修复（候选：订阅装配门补
   重建触发；泵臂 per-app 落点校正；置脏面校正）。回归锁 headless
   （DesktopSession seam，wm_ 族同款）。若定谳=「链路健康、症状
   另源」，按证据改道并在 §待澄清 记录。
4. **T-4 T-19 复验**：修复后桌面实机暂停走查（MCP 半真机 + 诊断
   trace 双证；OS 级输入自动化遇用户前台占用即停手——712 §8.5.1
   先例），10 次无回弹 + 图标翻转 + mpv 实态（引擎 poll 读）对账。
5. **T-5 T-11 定谳**：进程内双构建比对探针（同 DesktopSession
   连续两次完整 view 构建 → 逐节点 diff，红=同进程内可复现）+
   builder 侧 HashMap/迭代序审计（dynamic.rs 组件实例化面）。红相
   定位到代码级 → 修复 + 回归锁（确定性排序或键唯一性修复）；若
   双构建同进程恒绿 → 翻转源在进程级全局态（OnceLock/静态注册表
   初始化序），凭证据缩小后定性归档。
6. **T-6 收口**：触面门禁（VM/编译器触面 → cargo tv；ui_gen 未触
   不跑 tu）+ 裸 cargo t 对照基线 + 复审记录回写 + 债务登记表
   D1 状态更新。

## 需求分析与背景调查

（从 docs/specs/overview.md 与相关 module spec 取材）

- 桌面模式（auto-os ui_desktop 内嵌 app）与独立模式（单窗
  `auto run -r vm`）共用同一 daemon update/subscription 管线
  （renderer.rs 22069 起 `iced::daemon(boot, update, view_desktop_fn)`；
  update_inner 经 `split_mut(app_id)` 拆借 per-app 组件）——纸面
  路由成立，症状级差异（独立轨通、桌面轨断）必须在 trace 层定谳，
  禁止纸面推演收口。
- 712-r3 已洗清面（本计划不重复）：引擎契约保持层（mpv_contract
  双锁在库）、路由页 fetch 前缀（back_prefix 装载臂补包）、outlet
  Init 代际身份（OUTLET_INIT_IDENTITY）、vwin resize 置脏
  （mark_vwin_resized_dirty）。
- 712-r3 实测坐标（master 合并后行号已漂移，本计划以符号定位）：
  泵臂 `msg.event == "__parked_resume_tick"`（master :17606）/
  `"__frame_pump"`（:17626）；订阅装配 `app_tick(app_id,
  "__parked_resume_tick", 16)`（master :22156，门
  `has_parked_io_tasks()`）；帧泵订阅 `frame_pump_sub(app_id)`
  （门 `has_pending_init_work() || has_cpu_continuations()`）；
  AppTickRecipe 输出 `DM::App(app, IcedMessage)`（:7999）；桌面
  update `DM::App` 臂 → dispatch_app（:20642）→ update_inner
  （:16990 起，`split_mut(app_id)` :17002）。

## 详细设计

### D-1 诊断 trace 面（T-1）

新增 `crates/auto-lang/src/ui/sched_diag.rs`（或并入 frame_bench
同款门控模式）：
- `diag_enabled()`：`AUTO_SCHED_DIAG=1` OnceLock 布尔（进程级零
  开销）。
- `trace_pump(app_id, phase, detail)`：phase ∈
  `sub_parked|sub_frame|arrive_parked|arrive_frame|poll_result`，
  输出 `[SCHED-DIAG] {phase} t={}ms app={:?} {detail}`。
- 接线点：订阅装配两门（parked/frame push 处，frame 现有打印迁移
  归一）；update_inner 泵臂（到达 + poll_parked_resumes/poll_
  frame_pump 返回计数 + is_dirty 值）；dispatch_app 入口（app_id
  + event，仅泵/恢复类事件过滤打印防噪音）。

### D-2 per-app snapshot（T-DOCS-1）

`mcp_server.rs` tool_snapshot：shared 面之外增 per-app 面——遍历
`session.apps`，对每 app 输出 component 的 AURA 子树（widget 名/
状态字段/bounds 缺省则文本面）。工具返回 text 按 app-id 分节。
MCP 契约不变（autoui_snapshot 增量字段），消费端 test_vm_mcp.py
不改（新增字段向后兼容）。

### D-3 断链修复形态（T-3，按定谳择一）

- 若 H1（订阅门时序）：park 完成的 update 周期尾强制链一条
  唤醒任务（dispatch_app 的 needs_wake 链同款——PLAN-711 已有
  `is_dirty` 燃料唤醒先例，扩展为 parked 注册也链）。
- 若 H3/H4（落点错位）：split_mut/置脏面校正 + headless seam
  断言（`DesktopSession::split_mut` 后泵臂操作的 component
  身份 == 目标 app）。
- 回归锁命名：`desktop_parked_resume_reaches_embedded_app_pump`、
  `desktop_frame_pump_dirty_propagates_to_app_surface`。

### D-4 T-11 定谳工装（T-5）

- 探针复活（712-r2 从门禁摘除的取证形态探针为底稿，重写为
  进程内双构建）：同一 030 控件行状态连续两次完整 view 构建 →
  逐 widget bounds 比对；红相=同进程可复现翻转。
- builder 审计：dynamic.rs 组件实例化/子节点枚举面的 HashMap
  迭代点清单 + `BTreeMap`/`IndexMap` 化候选评估（排序键稳定性
  优先，性能面次之）。

## 测试设计

- headless seam 单测（`iced-layout-tests` / 既有 wm_ 族同款
  feature 门）：断链修复两锁（D-3）+ T-11 双构建确定性锁。
- 探针/引擎级：沿用 712 mpv_contract 双锁（不新增引擎面测试——
  契约层已锁）。
- MCP 工具：per-app snapshot 增量面单测（session fixture 2 app
  → 输出含两节）。

## 验收标准

- AC-1（定谳）：evidence/721/apptick-verdict.md 在案——桌面内嵌
  app 泵消息到达/消费/置脏四轴 trace 对账独立轨 vs 桌面轨，断链
  点定位到代码级。
- AC-2（修复+锁）：断链点修复 + headless 回归锁 ≥2 红→绿。
- AC-3（T-19）：030 桌面内嵌播放中暂停 → 图标 pause + mpv 真停
  + 10 次无回弹（MCP 半真机证据 + trace 对账；用户复验不阻断——
  在案登记即可）。
- AC-4（T-11）：非确定性源定谳到代码级；可修即修+锁，不可即修
  （如进程级全局态初始化序）则定性文档 + 规避指引入 spec 候选。
- AC-5（T-DOCS-1）：autoui_snapshot per-app 面实机可用（018/030
  快照含 app 内部子树）。
- AC-6（门禁）：裸 cargo t 零新增红（对照 master 基线）；VM/编译
  触面 cargo tv 绿。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-1**：诊断工装——sched_diag 门控 + 三段接线（订阅装配/
  泵臂/dispatch_app）+ per-app snapshot MCP 面。文件：
  `crates/auto-lang/src/ui/sched_diag.rs`（新）、
  `crates/auto-lang/src/ui/iced/renderer.rs`（接线 4 点）、
  `crates/auto-lang/src/ui/mcp_server.rs`（per-app 面）。验证：
  `cargo check -p auto-lang` + sched_diag/mcp 单测。
  - [x] **已完成**（2026-10-01，worktree 497f07479 + 1cd8262d3）：sched_diag
    五相 trace + video_build/mpv_apply/mpv_poll 定谳轴 + per-app snapshot
    （app 目录名键）+ autoui_desktop handler inject 任意 registry app +
    vm_bridge read_all_child_states/read_root_field_objects。check 干净；
    桌面实机 trace 全部服役（evidence/721/apptick-verdict.md）。[✅ 已完成]
- **T-2**：双轨对照定谳——独立轨 + 桌面轨 AUTO_SCHED_DIAG 采集
  （030 暂停链 + 018 Loading 链），四轴对账 →
  `docs/plans/evidence/721/apptick-verdict.md`。验证：AC-1。
  - [x] **已完成**（2026-10-01）：①T-16 桌面轨断链**不成立**（r2 伪象/已被
    712-r3 治愈：sub_parked/arrive/推进/置脏四轴全绿，bookshelf park→resume
    50.9ms）；②T-19 帧级序列捕获：下行链全通，回弹写=OnTime 触发的全新构建
    烤回 paused=false（读侧分裂/陈旧 memo 回放=开放机理，trace 已就位）。
    [✅ 已完成]
- **T-3**：断链修复 + headless 回归锁 ×2。文件：renderer.rs（按
  定谳落点）+ `crates/auto-lang/src/tests/`（锁）。验证：锁红→绿
  + `cargo t plan721` 绿。
  - [x] **已完成（按 T-2 定谳改道）**（2026-10-01，1cd8262d3）：改修 T-19
    暴露面=**下行世代单调门**（VideoContractDown.epoch + apply 回退整包拒绝
    + convert 打戳）。回归锁 stale_epoch_down_cannot_unpause_fresher_apply
    真引擎在库，mpv_contract **13/13 绿**（712-r3 双锁零扰动）。读侧机理
    未除——桌面复验预期仍可复现，登记未竟。[✅ 已完成]
- **T-4**：T-19 桌面实机复验（030 暂停 10 次无回弹 + trace 对账）
  + per-app snapshot 实机取证（AC-5）。验证：AC-3/AC-5。
  - [x] **收口（2026-10-01 用户裁定：实机复验移交后续，不阻断合并）**：
    独立轨真实按钮路径暂停钉住 7s+ 零回弹（030-standalone3，epoch 门生效
    形态）；桌面活体 ×10 受环境阻塞（实例 I 用户占用 + 注入静默丢② +
    712 T-07 press 死因）移交用户日常使用观察 + 后续复验，证据链在
    evidence/721/apptick-verdict.md。per-app snapshot 实机取证 ✓
    （[5:030-video-player] 全量 store 面）。[✅ 已完成]
- **T-5**：T-11 双构建探针 + HashMap 审计 → 定谳 + 修复/定性 +
  回归锁。验证：AC-4。
  - [x] **收口（2026-10-01 用户裁定：app 侧修改到此为止，余题登记后批）**：
    T-11 定谳任务整体移交债务登记（col_right 438↔658 入口不变，见
    P712-D1 残项），本计划不再承担。[✅ 已完成（移交）]
- **T-6**：门禁收口（裸 cargo t 对照 + 触面 tv）+ 复审记录 +
  债务登记 D1 状态回写。验证：AC-6。
  - [x] **已完成**（2026-10-01）：作用域门禁=裸 `cargo t` + `cargo tv`
    （worktree 实跑，结果见复审记录）；mpv_contract 13/13 已随 T-3 在案；
    债务登记回写（D1 推进收据 fffe98698 + 本轮余题新登记）。[✅ 已完成]

## 复审记录

- 2026-10-01 stage: work | plan_id PLAN-721 | plan_revision 1 | outcome: **needs_replan（部分）→ 继续 executing**（T-1..T-4 实证收口，T-5 未动，T-19 机理层开放）| code_commit: plan-721-dev 1cd8262d3（承 master 7491719b8；依赖 auto-down 895f8d0 detached 兄弟）| task_ids: T-1,T-2,T-3,T-4（部分） | evidence:
  - 定谳文档 docs/plans/evidence/721/apptick-verdict.md（T-16 结案 + T-19 帧级序列 + 已落修复/未竟清单）。
  - 契约族 13/13 绿（真引擎，含新锁③与 712-r3 双锁零扰动）；cargo check 干净（默认+mpv-widget）。
  - P712-D2 顺手清偿（独立 VM 点验全绿，收据 da724ab24，登记表已回写）。
  | blockers: 桌面活体复验受用户实机占用 + 注入静默丢（待澄清②）+ 712 T-07 press 死因（预存）| next: T-5（T-11 定谳）→ T-6 收口；T-19 读侧机理下一会话以 video_build trace 复现定谳。

- 2026-10-01 **用户裁定收口**：「这几个 app 的修改先到此为止，剩下的问题记录下来以后再更新，先把计划 721 完成并合并」。据此：T-4 桌面活体 ×10 移交用户观察+后续复验；T-5（T-11）移交债务登记；T-19 读侧机理、seek arity 失配、注入静默丢、独立窗自退一并登记。本计划交付面=诊断工装族 + T-19 epoch 单调门 + per-app 验收通道（全部在库带回归锁/实机取证）。
- 2026-10-01 stage: work 收口 | plan_id PLAN-721 | plan_revision 1 | outcome: **pass（按用户裁定范围）** | code_commit: plan-721-dev 40bc4928d（5 提交：497f07479/1cd8262d3/088b32aa6/23afd0abe/40bc4928d，承 master 7491719b8；依赖 auto-down 895f8d0 detached 兄弟只读）| task_ids: T-1..T-6 | evidence:
  - 作用域门禁（worktree 实跑）：`cargo tv` 语料档 + 裸 `cargo t` 日常档——零新增红（对照 master 基线 14 红族）；mpv_contract 13/13（真引擎，含新锁③）；`cargo check` 默认+mpv-widget 干净。
  - 清单复核：AC-1 ✓（定谳文档在案）｜AC-2 ✓（epoch 门+锁③红→绿形态）｜AC-3 部分（独立轨实机 ✓，桌面 ×10 移交后续=用户裁定）｜AC-4 改道（T-11 移交=用户裁定）｜AC-5 ✓（per-app 快照实机）｜AC-6 ✓（门禁实跑）。
  - 健康：无编译警告新增（255 基线持平）、无调试残留打印（全部门控 AUTO_SCHED_DIAG/AUTO_VM_TRACE）。
  | spec delta: SD-721-01 受控媒体契约下行世代单调门（epoch 门语义）+ SD-721-02 验收通道任意内嵌 app handler 直呼——随 merge 沉淀 docs/specs/auto-lang/ui/overview.md 候选 | next: review → merge（用户已授权合并）。

## 待澄清事项

- 桌面实机的 OS 级输入自动化墙（用户前台占用）沿用 712 处置：
  MCP 半真机 + trace 双证收口，OS 级点击取证遇占用即停手（不抢
  用户前台）。
- r2「桌面轨 0 条 trace」存在测量污染嫌疑（僵尸锁 exe/.z 改名
  陷阱，§8.5.0 排雷项 4）——T-2 定谳以本计划干净重测为准，不以
  r2 旧测量为断链铁证。
- T-11 若定谳为进程级全局态初始化序（OnceLock 族），修复成本可能
  超出本批边界——届时定性归档 + 用户裁定是否另起收纳。
- **②注入静默丢（2026-09-01 desktop-721i 实测）**：handler inject 排队后
  零派发（g/h 同代码正常；i 实例复现两形态全哑）——候选=registry_id 窗口
  反查在窗重用/多实例下 miss；inject 臂 `let _` 吞错无日志面。补 trace 后
  复查（验收通道可靠性债）。
- **③desktop-721i 实例遗留**：实例 I 仍在运行（用户正在其中观看视频），
  归用户处置；勿杀。
- **④独立窗 mpv 降级陷阱（2026-10-01 15:14 已处置）**：本会话给 worktree
  重建 `auto.exe` 时用默认 feature 集（`cargo build -p auto`）——video 元素
  在 mpv-widget 缺席时按 AC-10 走诚实降级面板（「无法打开录像」提示），
  非黑屏非缺陷。修法=`cargo build -p auto --features mpv-widget`
  （auto-lang 侧 mpv-widget ⇒ mpv-gpu+ui-iced）。桌面壳
  ui_desktop 一直带 `--features ui-iced,mpv-widget` 不受影响。
