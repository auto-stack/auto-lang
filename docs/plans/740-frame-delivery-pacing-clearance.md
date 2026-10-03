---
plan_id: PLAN-740
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 泵回环交付节奏清偿件（auto-edit PLAN-027 回执余题②消费——scroll 54fps 解锁前置：交付节奏定责+最小面修复+下游复判解锁）
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md（SD-01：add §4d——交付节奏契约：驱动跟随性+唤醒节奏修复语义）"
  - "docs/specs/auto-lang/ui/design/frame-observability.md（SD-02：modify §1/§4——交付节奏三轴观测通道：action 消费节奏/request_redraw 发出/RedrawRequested 到达打点位）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（下游回执驱动面——731/735 先例注记式）

affects: [crates/auto-lang/src/ui/iced/renderer.rs, crates/auto-lang/src/ui/dynamic.rs, crates/auto-lang/src/ui/sched_diag.rs, docs/plans/evidence/]
current_step: 0
total_steps: 5
---

# [PLAN-740] 泵回环交付节奏清偿件（scroll 54fps 解锁前置：定责+最小面修复）

## 0. 变更摘要

auto-edit **PLAN-027**（735 消费终裁件，2026-10-03 delivered@552c789
archived@612efce——P027-1 账本回执）登记的**上游域余题② 清偿件**：
下游 scroll_fps 判据（≥54fps）物理不可达的定责已从「驱动器上限」
修正为「**帧泵回环/事件循环交付节奏上限 ~10/s**」——027 实测证据链
（16ms 驱动 34Hz 下 draw_end 间隔 91-95ms 恒定；突发内 0-1ms 连建；
帧工作 2-3ms 全绿；type_latency 行通道 P95 3-4ms 首绿）证明帧能力
完全够 54fps，**卡点=唤醒→重建→绘制循环的持续推进节奏**（735 §4
机制候选 (b)「泵回环自 paced ~110ms/圈」的下游实证形态）。

本件三步走：**①三轴打点+驱动率阶梯勘定**（action 入队→消费节奏/
request_redraw 发出→RedrawRequested 到达→draw_end 对齐打点——定位
~93ms 定 pacing 的责任轴，机制候选不预设 735 §4 前科）；**②定责+
最小面修复**（已实勘的无争议下界=renderer.rs poll_mcp_actions 单条
try_recv——drain-to-empty；其余按 T-01 定责，候选=16ms tokio tick
真实触发节奏〔current-thread runtime×winit 唤醒耦合/0.14.1 陈旧唤醒
守卫〕与重绘调度面）；**③双谱+回归门+下游回执**（交付率==有效驱动
率至 ≥54Hz 档；731/735 谱零回退；下游复判通道=auto-edit 027 修订
协议零工具改动——本件不含下游复判，另件预告）。

## 1. 目标

- **G-1 三轴勘定+定责（决策件前置）**：AUTO_SCHED_DIAG 扩轴（action
  消费节奏轴/request_redraw 发出轴/RedrawRequested 到达轴——
  draw_end 轴 735 已在位）+041 例驱动率阶梯探针（有效驱动
  ~10/16/34/≥54Hz 档——batch-drive 机制 T-00 内建勘定，MCP 调用时延
  p50 ~62.5ms 是单发驱动的物理上限，阶梯需绕开）。产出=定责报告
  （~93ms pacing 的责任轴+机制级定责）。
- **G-2 最小面修复（G-2 主面）**：按 T-01 定责落修复——(b)
  drain-to-empty 无争议必做；其余候选（tick 触发节奏/重绘调度/
  即时绘制臂）按定责证据最小面实施。**护栏 frozen**：不 fork
  iced_winit（若定责唯一路径在其内部且不可绕→`needs_replan` 钉版
  升级评估 Q-1）；dirty/epoch 调度域（712 r2）零触碰；门关零开销
  纪律延续。
- **G-3 交付节奏契约兑现（041 例谱面）**：改后阶梯谱**交付率跟随
  有效驱动率（±10%）至 ≥54Hz 档**；draw_end 间隔随驱动节拍收缩
  （非 91-95ms 恒定）。
- **G-4 零回退门**：731 s5 三线带+735 配对率 100%+type total P95 带
  +golden 三形 0.00%+`cargo t` 零新红+`cargo tv` 触面+716 探针族
  绿。
- **G-5 下游回执+规范+账本**：downstream-handoff-v3（复判解锁预告
  ——通道=auto-edit 027 修订协议零工具改动；判定阈值语义 frozen
  不变）+SD-01/02 落档+P740-1（015 先例 merge 期项）。

### 非目标

- **下游复判件**（auto-edit scroll_fps 54fps 正式判定——027 协议
  复跑，本件 delivered 后另件消费；本件只承诺上游交付能力谱面）。
- 判定阈值/语义任何变更（下游 54fps/面板×0.9 frozen——本件只修
  交付能力）。
- 帧工作面优化（731 已清偿——s5 带 0.1-0.2ms 零回退维持即可）。
- 711 泵 D-2 门控语义重构（稳态滚动走输入驱动帧，不依赖泵订阅
  ——027 实勘；除非 T-01 定责直指，不动）。
- 判定通道面（735 已定谳呈现面真值通道——027 协议即终态）。

## 2. 架构方案

分层落点（2026-10-03 实勘，auto-lang master@fe5a270b5[735@4195f7af9
+737@59ff4e66f+739 祖先链——起草时 master 实勘推进位〔736 T-03 在途会话；
代码锚复核=de3a64353→fe5a270b5 仅行号微漂 9402→9407〕]）：

| 面 | 现状（实勘锚） | 本期形态 | 依据 |
|---|---|---|---|
| 交付节奏 | ~93ms/圈恒定（027 下游实测；驱动率无关） | **跟随有效驱动率至 ≥54Hz** | 027 回执 |
| action 消费 | `poll_mcp_actions` 单条 try_recv/16ms tick（renderer.rs:9207/:9402/:23170——AppTickKind::Poll + MissedTickBehavior::Skip :8568） | **drain-to-empty**（每拍循环取空——无争议下界）；tick 真实触发节奏打点 | 027 实勘 |
| 重绘调度 | request_redraw→RedrawRequested→draw 链（735：呈现随输入 +6-10ms；0.14.1 陈旧唤醒守卫注记） | 三轴打点定责；若该轴 10Hz→即时绘制/ControlFlow 面最小修 | T-01 |
| 观测通道 | AUTO_SCHED_DIAG：redraw_deliver（735 轴①——送达观测）+frame_msg enqueue+frame_pump enter | **扩轴**：action drain 时刻/request_redraw 发出时刻（门控零开销纪律） | SD-02 |
| 下游复判 | auto-edit 027 协议（行通道+16ms 驱动） | 零改动复判（另件） | 027 §协议修订记录 |

**关键设计约束（frozen）**：① **机制候选不预设**（735 §4 前科——
T-00 勘定先于修复；已实勘的 (b) 单条 try_recv 除外=无争议下界）。
② **不 fork iced_winit**（735 约束；唯一路径落内部→needs_replan，
Q-1 钉版升级评估）。③ **712 r2 域零触碰**（dirty/epoch 调度语义
grep 锚断言——735 同款护栏）。④ **门关零开销**（AUTO_SCHED_DIAG
门控扩轴——716 门控纪律）。⑤ **双态如实**（若修复后仍有残余 pacing
面——归因回执如实，不冒领）。

## 3. 技术栈

auto-lang ui/iced 层（renderer.rs 订阅装配+poll 通道+dynamic.rs 泵
臂+sched_diag.rs 扩轴——Rust 工具面）+ladder740.py 驱动率阶梯谱
（docs/plans/evidence/740/）+golden 三形回归+`cargo t`/`cargo tv`
门禁。零 .at 源改动；auto-edit 零改动（范围断言随回执）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-10-03 会话指令「OK，请起草这个计划」——
授权=**起草本件**（消费 P027-1 余题②；scroll 54fps 解锁前议题位
=026 归档预告→027 回执第三轮延续）。执行/work 待用户启动；无预算
/自动续跑授权。范围=auto-lang 单仓（ui/iced 层+evidence+SD）；
auto-edit 零改动（复判另件）。

**来源与版本**：

- 交接链：PLAN-735 归档件（呈现事实定谳+配对修复+draw_end 真相
  通道——delivered@4195f7af9；§3.3 驱动节拍建议=027 已消费）+
  auto-edit PLAN-027 归档件（P027-1 回执：**余题②=泵回环吞吐上限
  ~10/s 定谳**——91-95ms 恒定/突发 0-1ms 连建/帧工作 2-3ms 全绿/
  34Hz 驱动无效）+737（MCP spawn_blocking——current-thread runtime
  冻结面修复，与本件候选 (a) 同域背景）+739（输入回写边界——§4c
  在档）。
- 027 实测证据（下游，auto-edit tools/bench/results/frame-
  20261003-1140xx+evidence-p027-frame-channel.jsonl）：MCP 调用地板
  p50 14.1-15.8ms→16ms 节拍有效驱动 ~30ms 周期 ~34Hz；draw_end
  间隔 91-95ms 恒定（两跑一致）；突发 ~93ms 一次、内含 ~6 行
  0-1ms 连建+1 脏帧；scroll 行通道 9.6-10.0fps vs ≥54 FAIL。
- 041 例对照（735 谱，docs/plans/evidence/735/ladder-p735-after
  .jsonl）：60ms 驱动下 pump/s 10.07-10.56==驱动节奏读数（MCP 调用
  p50 62.5ms+60ms sleep≈95-107ms 有效周期）——**041 在 ~10Hz 驱动
  下 1:1 跟随**；34Hz 驱动下的跟随性未测（本件 T-00 阶梯补测）。
- 代码实勘锚（master@fe5a270b5 复核在位）：
  - `renderer.rs:9207 poll_mcp_actions`——`rx.try_recv()` **单条**
    （drain 面）；
  - `renderer.rs:9407 mcp_action_subscription`——
    `AppTickKind::Poll(poll_mcp_actions, 16)`；`:23170` 装配
    （primary app）；
  - `renderer.rs:8544-8597 AppTickRecipe::stream`——
    `tokio::time::interval_at(16ms)`+`MissedTickBehavior::Skip`
    （**tick 实际触发节奏=current-thread runtime 驱动面**——737
    spawn_blocking 同域）；
  - `renderer.rs:23027` 泵 D-2 门控（仅 Init/CPU 工作窗口——稳态
    滚动不依赖）；`:8619 frame_pump_sub`；`:8660 redraw_diag_sub`
    （送达观测，恒 None 不产消息）；
  - `dynamic.rs:1554 poll_frame_pump`——CPU_PUMP_ROUND_BUDGET 8ms
    轮次；
  - iced_winit **0.14.1 钉版**（735 注记：0.14.0→0.14.1 加
    `current > Instant::now()` 陈旧唤醒守卫——跨版 ControlFlow 行为
    差异在案）。
- 机制候选清单（T-00 不预设——证据定责）：
  (a) 16ms tokio tick 实际触发节奏 ~90ms（current-thread runtime×
  winit 唤醒耦合/陈旧唤醒守卫副作用）；(b) 单条 try_recv 供给耦合
  （无争议下界——已实勘）；(c) request_redraw→RedrawRequested 的
  winit 合并/延迟（0.14.1 ControlFlow 面）；(d) 驱动期多订阅
  （timer/parked/media/mcp_poll 同享 runtime）竞争；(e) vsync/
  compositor（弱候选——93 非 16.7 整数倍，735 已列）。
- 规范口径：frame-pipeline-incremental.md §4b（735 升格=交付节奏
  契约——本件 SD-01 在其后续写 §4d）+frame-observability.md §1/§4
  （通道面+边界——本件 SD-02 扩轴）。

## 5. 详细设计

### T-00 三轴打点+驱动率阶梯勘定（决策件前置）

AUTO_SCHED_DIAG 扩轴（门控零开销）：**action drain 轴**（每次
poll_mcp_actions 实际取到动作的时刻+本次 tick 距上次的间隔——16ms
tick 真实触发节奏的直接证据）+**request_redraw 发出轴**（update 轮
request_redraw 调用时刻）+**RedrawRequested 到达轴**（735
redraw_deliver 观测已备——激活对读）。**驱动率阶梯探针**（041 例，
ladder740.py）：绕开 MCP 单发时延上限的 batch-drive 机制勘定（候选
=N 字符单 call[若语义=一次 bulk 编辑则改内部通道直推 test-only
action 注入]）——阶梯 ~10/16/34/≥54Hz 有效驱动各档，记录交付率
（distinct draw_end/s）与间隔分布。**下游对照数字**（027 在案）随
报告引用。产出=定责报告（evidence/740/T-00-T-01-report.md）。

### T-01 定责+修复方案选定（决策件）

从阶梯谱定谳责任轴：(a) 若 action 消费节奏 ~90ms→runtime×winit
唤醒面（候选 1=drain 后仍需修 tick 触发；可能落 tokio driver
block_on 喂入节奏或 0.14.1 守卫）；(b) 若消费 34Hz 而
request_redraw 10Hz→update 轮节流面；(c) 若 request_redraw 34Hz 而
RedrawRequested 10Hz→winit 合并/ControlFlow 面（即时绘制臂/Poll
面——不 fork 约束下的最小绕行）；(d) 若全轴 34Hz 而 draw_end
10Hz→present 面新域（回 needs_replan）。方案选定含边界护栏核验
（712 r2 域零触碰断言预检）。

### T-02 最小面修复实施（依 T-01）

必做（无争议下界）：`poll_mcp_actions` drain-to-empty（每拍循环
try_recv 至 Empty——防单条耦合；畸形/Disconnected 臂原样保留）。
其余按 T-01 定责落最小面（每处修复附门关零开销自证）。716 探针族
+帧通道五面回归随改随跑（fast 面 `cargo t iced`）。

### T-03 双谱+零回退门（G-3/G-4）

ladder740.py 改前/改后双谱（三档 size×三相位+阶梯驱动档——交付率
跟随性断言：**交付率==有效驱动率 ±10% 至 ≥54Hz 档**；draw_end 间隔
分布随节拍收缩）+731 s5 三线带（0.36/0.42/0.34 域）+735 配对率
100%+type total P95 带+golden 三形 0.00%+`cargo t` 日常面（5038 基
线零新红——16 红预存在档）+`cargo tv` 触面+716 探针族 19 绿。

### T-04 下游回执+规范+账本（G-5）

downstream-handoff-v3（evidence/740/——**复判解锁预告**：auto-edit
027 协议复跑即得判定；驱动余量注记〔下游 MCP 调用地板 ~15ms→54Hz
需 drive 睡眠 ≤3.5ms——复判件参数预告〕）+SD-01/02 落档
（worktree specs 提交——merge 期正式沉淀）+P740-1=merge 期项。
范围断言：auto-edit 零改动+712 r2 域零触碰。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add（§4d 新节） | docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md | before：§4b 交付节奏契约（735 升格——配对面口径→通知保证面）+§4c 输入回写边界（739） / after：**+§4d 交付节奏持续率契约**——呈现交付率跟随有效驱动率至面板率（跟随性判据=交付率==驱动率 ±10%；唤醒→重建→绘制循环持续推进语义+修复面〔drain-to-empty+定责修复〕落契约） | 交付节奏契约的行为真源 | AC-02/03 |
| SD-02 | modify（§1+§4） | docs/specs/auto-lang/ui/design/frame-observability.md | before：§1 通道面（735 现状——redraw_deliver 送达观测）+§4 边界注记 / after：§1+交付节奏三轴观测通道（action drain 节奏/request_redraw 发出/到达对读——AUTO_SCHED_DIAG 门控扩轴语义+零开销自证）；§4+交付节奏观测边界注记 | 三轴打点的通道契约 | AC-01 |

## 6. 测试设计

- **阶梯跟随性谱**（主验收面）：ladder740.py 阶梯驱动 ~10/16/34/
  ≥54Hz——改后交付率==有效驱动率（±10%）+draw_end 间隔分布随节拍
  收缩断言+改前对照谱在档。
- **三轴打点谱**：AUTO_SCHED_DIAG 扩轴对读（action drain 间隔分布/
  request_redraw→到达时延/到达→draw_end 时延）——定责证据链。
- **零回退门**：731 s5 三线+735 配对 100%+type total P95 带+golden
  三形 0.00%（基面源 baseline 对照）+`cargo t` 5038 基线零新红+
  `cargo tv`+716 探针 19 绿。
- **门控零开销**：扩轴 ns 级门关自证（716 微基准法）。
- **范围断言**：auto-edit 零改动（porcelain）+712 r2 域
  （dirty/epoch/poll_frame_pump 泵臂语义）零触碰 grep 锚。

## 7. 验收标准

- **AC-01 勘定+定责**：三轴打点谱+驱动率阶梯谱（改前）在档——
  定责报告给出 ~93ms pacing 的责任轴与机制级证据（候选清单逐项
  判定）。验证：报告+JSONL（evidence/740/）。
- **AC-02 修复生效**：改后阶梯谱交付率跟随有效驱动率（±10%）至
  ≥54Hz 档+draw_end 间隔随节拍收缩+修复面最小性注记（每处 diff
  对应定责证据）。验证：ladder740 双谱对照表。
- **AC-03 零回退**：731 s5 带+735 配对 100%+type total P95 带+
  golden 0.00%+`cargo t` 零新红+`cargo tv`+716 探针绿。验证：双谱
  JSONL+门禁输出。
- **AC-04 下游回执+规范+账本**：handoff-v3 在档（复判解锁预告+
  驱动余量参数）+SD-01/02 落档+范围断言（auto-edit 零改动+712 r2
  域零触碰）+P740-1=merge 期。验证：回执档+SD diff+断言记录。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 三轴打点+阶梯勘定 | — | sched_diag.rs 扩轴+evidence/740/ladder740.py | 定责材料 | AC-01 | [ ] 打点谱+改前阶梯谱在档 |
| 1 | T-01 定责+方案选定 | T-00 | evidence/740/T-00-T-01-report.md | 责任轴定谳 | AC-01 | [ ] 报告在档（候选逐项判定） |
| 2 | T-02 最小面修复 | T-01 | renderer.rs:9207 drain-to-empty+定责面 | 交付节奏解锁 | AC-02 | [ ] 门关零开销自证+fast 面绿 |
| 3 | T-03 双谱+零回退门 | T-02 | evidence/740/ 双谱 | 契约兑现 | AC-02/03 | [ ] 跟随性断言+零回退带 |
| 4 | T-04 回执+规范+账本 | T-01..03 | handoff-v3+SD-01/02+P740-1（merge 期） | 下游解锁预告 | AC-04 | [ ] 范围断言绿 |

## 9. 复审记录

- 2026-10-03 起草 handoff：`stage: new`，PLAN-740，plan_revision 1。
  `outcome: pass`（起草完备：P027-1 余题②逐项承接〔91-95ms 恒定/
  突发连建/帧工作全绿/34Hz 驱动无效四证据在案〕；**机制候选清单五
  项不预设**〔735 §4 前科——T-00 三轴打点定责先于修复，唯 (b) 单条
  try_recv=已实勘无争议下界先行〕；**不 fork iced_winit 约束延续+
  唯一路径落内部→needs_replan 升级评估 Q-1**；712 r2 域零触碰护栏
  延续；下游复判=另件预告〔027 协议零工具改动+驱动余量参数〕不在
  本件范围；驱动率阶梯的 batch-drive 机制=T-00 内建勘定项〔MCP 单
  发时延 ~62.5ms 是 041 例驱动上限的物理事实〕；路径/符号经
  auto-lang@de3a64353 实勘锚定〔renderer.rs:9207/9407/8544-8597/
  23027/8619/8660+dynamic.rs:1554〕；授权=起草〔用户指令〕，执行
  待用户启动）。`next: work`。

## 10. 待澄清事项

- **Q-1 iced_winit 钉版升级（条件件——T-01 定责若唯一路径落 0.14.1
  内部且无绕行面）**：默认倾向=不升级（0.14.1 行为差异面 735 已注
  记；升级=跨版回归风险）；届时以定责证据请示用户裁定。
- **Q-2 下游复判件参数预告确认（无需裁定——handoff-v3 随附）**：
  54Hz 驱动余量=下游 MCP 调用地板 ~15ms → 复判 drive 睡眠需 ≤3.5ms
  （027 协议参数预告——复判件另案时确认）。
