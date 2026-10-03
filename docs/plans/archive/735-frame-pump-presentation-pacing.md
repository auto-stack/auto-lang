---
plan_id: PLAN-735
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: 帧泵呈现交付节奏清偿件（auto-edit PLAN-026 下游回执消费——掉泵/交付节奏上游第二段：呈现事实定谳+交付节奏根因+修复或通道语义裁定——帧两行转绿解锁）
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-03

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/frame-observability.md（SD-01：modify——§4/§5 泵交付语义更新：呈现事实通道+交付节奏契约）"
  - "docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md（SD-02：modify——§4b 泵治理语义升格：配对面口径→交付节奏契约+呈现真相通道）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（下游回执驱动面——731 先例注记式）

affects: [crates/auto-lang/src/ui/iced/renderer.rs, crates/auto-lang/src/ui/frame_bench.rs, crates/auto-lang/src/ui/frame_segments.rs, crates/auto-lang/src/ui/sched_diag.rs, docs/plans/evidence/]
current_step: 6
total_steps: 6
---

# [PLAN-735] 帧泵呈现交付节奏清偿件（掉泵第二段：呈现事实+交付节奏）

## 0. 变更摘要

auto-edit **PLAN-026**（731 消费重判件，2026-10-03 delivered@185f206）
下游回执的**上游第二段余题清偿件**。026 终判实测定谳了换代归因：
731 清偿了**帧工作面**（下游 S5 三子段 0.1-0.2ms——024 段外 ~108ms
消散，帧构建 ~2ms 对 16.7ms 预算余量 ~8×），但帧两行仍 FAIL——
红面=**帧泵呈现交付节奏**：286 个真建帧仅 37 个（13%）拿到自己的
present 通知，present→present 恒定节拍 ~105-120ms（scroll 8.1-9.5
fps=交付率实测），而**管道单程极快**（配对帧 begin→present 中位
5ms/最快 3ms）。731 T-04 的"掉泵治理"实为**泵率谱通道+配对面定谳**
（present=-1=测量配对面，墙钟对照实证"非呈现丢失"——上游例），
fall-through 本身未修；本件=真清偿：**①呈现事实定谳**（呈现节奏
vs 通知节奏分离——下游判定通道语义裁定的前提）→ **②交付节奏根因**
（RedrawRequested 发放/消费链 ~9/s 恒定节拍定谳——上游例同现象
0.41-2.14 pump/s 在案=本地试验床）→ **③修复或通道语义裁定**
（交付节奏修复为主臂；呈现真相通道为并臂——下游 frozen 口径的
读回语义适配）→ **④下游回执**（重判解锁预告）。本件 delivered=
auto-edit 帧两行转绿的最后上游前置。

## 1. 目标

- **G-1 上游例复现+域协调核验**：041-auto-edit 例 ladder 泵率谱
  复跑（731 回执 0.41-2.14 pump/s 带内复现=试验床成立）+
  `AUTO_SCHED_DIAG=1` 帧消息到达谱；712 r2 残余（T-16..T-19 桌面
  轨调查——无 renderer 域 WIP 在册）零冲突核验+711 泵计数器
  （queued_init/cpu_cont/parked_total/write_q）在位勘定。
- **G-2 呈现事实定谳（决策件——下游通道语义裁定的前提）**：在**不
  fork iced_winit** 约束下建立"真实呈现节奏"的测量通道（候选：iced
  0.14 重绘钩子/wgpu surface present 时点/OS 合成器时戳
  〔DwmFlush/DwmGetCompositionTimingInfo〕/731 墙钟对照法形式化
  ——T-00 勘定后择优），产出**呈现节奏 vs 通知节奏分离谱**：
  定谳下游 8-9.5fps 是"呈现真慢"还是"通知滞后"。
- **G-3 交付节奏根因定谳（研究件——边界化）**：RedrawRequested 发
  放节奏（订阅 receipt 计数）vs winit/iced 重绘调度 vs 泵消费滞后
  vs 消息队列竞争（MCP 驱动订阅同队列）四轴 trace——~9/s 恒定节
  拍的机制定谳（自 paced 回环假设 vs 合并语义 vs 唤醒策略，§4 列
  全候选）。产出=根因报告+修复方案选定（多臂允许）。
- **G-4 修复或通道实施（主臂）**：依 G-3 判定——(a) **交付节奏修
  复**（每真建帧获得通知/重绘请求路径修正——泵循环/订阅语义/
  request_redraw 发放面）；(b) **呈现真相通道**（frame_bench 增
  直读 present 时戳——G-2 通道产品化）；两臂可并存（修复=行为
  面，通道=测量面）。711 R-1 序障+731 §4b 配对面语义同步升格。
- **G-5 谱对照+回归门**：ladder 改前/改后泵率谱（pump/s 逼近帧率
  带——修复生效判据）+731 谱零回退（segsum/S5 子段带）+视觉
  golden 三形+分级门禁（`cargo t` 日常面+`cargo tv` 触面）。
- **G-6 下游回执+规范+账本**：downstream-handoff v2（呈现事实
  结论+交付节奏终态+重判解锁预告——669 模式）+SD-01/02 落档+
  P735-1（703/728/731 先例）。

### 非目标

- 下游判定口径任何变更（auto-edit 022 frozen——重判协议/读回通道
  语义裁定=下游重判件事，本件只供证据与通道；口径适配走下游件）。
- 多窗会话第二泵臂插桩（renderer.rs:19871 待办注记维持——单窗主
  路径外，711 T-06余 观测面）。
- 帧工作面再优化（731 已清偿——S5 子段带零回退门即封）。
- a2r/L2 形态帧档通道（跨形态交叉验证=下游重判件的面——本件
  回执可列建议不实施）。
- iced/winit/wgpu 依赖升级（钉版面勘定注记，升级另行）。

## 2. 架构方案

分层落点（2026-10-03 实勘，auto-lang master@cb2da5fba+下游
auto-edit main@252028a）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| 帧泵订阅 | renderer.rs:8349 `frame_pump_sub`——filter_map on Interaction/RedrawRequested→`__frame_pump` 消息；无 post-present 回执（"硬屏障需 fork iced_winit——计划约束禁止"注记） | 发放/消费节奏 trace+（T-02 后）修复面——fork 约束重估或绕行通道 | 711 D-1 原注记 |
| present 记账 | frame_bench.rs `note_frame_present` 于泵消费臂——**消费时刻=present 时戳**（R-1 序障代理） | 呈现真相通道（T-01/T-03b——直读 present 时点） | 716 组B 通道 |
| 探针语义 | frame_segments prev/cur 双槽——`present=-1`=配对面（731 §4b 定谳） | 语义升格：配对面→交付节奏契约（数据不变口径增注） | 731 SD-01 §4b |
| 交付节奏 | 下游实测 13% 到达率+~105-120ms 恒定节拍；上游例 pump/s 0.41-2.14 同现象 | 根因定谳+修复（T-02/T-03a） | 下游 026 回执+731 ladder |
| 观测工具 | sched_diag（AUTO_SCHED_DIAG=1 帧消息到达谱）+711 泵计数器+725 分段探针 | 复用+按需扩展（trace 轴增量） | 721/711 T-06 |

**关键设计约束（frozen）**：
① **呈现事实先行**（T-01 决策件——修复方案与通道语义都依赖"呈现
真慢 or 通知滞后"的定谳，不预设结论）。② **下游判定零触碰**
（auto-edit 022 frozen——本件零下游判定面改动；通道语义裁定权在
下游重判件，本件只供通道与证据）。③ **711 R-1 序障保持**（任何
新通道的时戳序：消费/呈现时刻 ≥ 帧构建时刻——序障语义重验入
回归门）。④ **712 r2 域边界**（dirty/epoch/poll_frame_pump 泵臂
——T-00 核验零冲突后才动交付面）。⑤ **门控纪律**（新增时戳
通道沿用 AUTO_FRAME_BENCH/AUTO_SCHED_DIAG 单门语义——门关零
开销）。

## 3. 技术栈

iced 订阅/事件面（filter_map 配方+AppTickRecipe 族）+winit/wgpu
重绘调度勘定+frame_bench/frame_segments/sched_diag 观测面扩展+
ladder.py 泵率谱复跑。Rust 局部改动（ui/iced+ui 观测面）——分级
门禁 `cargo t`+`cargo tv`；不碰 transpiler/aavm/book。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-10-03 会话指令「……你可以直接去
auto-lang 建立计划吗？」——授权=**起草本件**（auto-edit
PLAN-026 下游回执的承接——669 模式上游件；执行/work 待用户另行
启动）。范围=auto-lang 单仓（ui/iced+观测面+evidence+specs）；
auto-edit 零改动（其回执为输入证据）。无预算/自动续跑授权。

**来源与版本**：

- 交接链：auto-edit **PLAN-026** 归档件
  （auto-edit 仓 `docs/plans/archived/026-m4-full-closeout.md`，
  delivered@252028a——v0.1-M4.2 重判收口 tag 在册）+回执证据
  （`specs/auto-edit/tests/evidence-p026-frame-rejudge.md`——
  归因换代全文+upstream 回执要点四条）+四跑探针 JSONL
  （`tools/bench/results/frame-20261003-*.jsonl`）+上游复验谱
  （`tests/evidence-p026-upstream-reverify.jsonl`）。
- 上游谱系：731 T-04 泵率谱（`docs/plans/evidence/731/
  T-05-comparison.md` §2——pump/s 0.41-2.14+orphan 17-37/18-43
  帧+"泵消费率≪实际呈现率=测量配对面"归因）+725 分段探针+711
  D-1 泵订阅+R-1 序障。
- 现状实勘（master@cb2da5fba）：`frame_pump_sub`
  （renderer.rs:8349——RedrawRequested→消息，无 post-present 回
  执）；`note_frame_present` 消费臂（frame_bench.rs——716 组B）；
  frame-observability.md §4 边界（712 r2 delivered+archived
  451dc1401+ff32d7004，残余 T-16..T-19 无 renderer WIP；多窗第
  二泵臂待办 renderer.rs:19871）+§5 prev/cur 双槽；
  frame-pipeline-incremental.md §4b 泵治理语义（配对面口径）。
- 下游实测摘要（026，2026-10-03，v0.4.2-2640-gcb2da5fba）：建帧
  286/自 present 37（13%）/配对帧 begin→present 中位 5ms 最快
  3ms/present→present ~105-120ms 恒定/scroll 25-30 distinct/3s
  =8.1-9.5fps/type 窗 segsum ~2ms+s5 三子段 0.1-0.2ms；024 对照
  （同日志法）：突发重建形 b2b 3-4ms+paired 17-24%。

**机制候选（T-02 trace 轴——不预设）**：
(a) winit/iced 重绘合并语义（连续 request_redraw 折叠——通知只
盖批量末帧）；(b) 泵回环自 paced（通知→异步回环每圈成本 ~110ms
→吞吐 1/latency≈9/s——下游节拍恒定性旁证）；(c) 事件循环唤醒
策略（空闲等待+消息队列排空时机）；(d) 消息队列竞争（MCP 动作
订阅同 update 队列）；(e) wgpu/合成器 present pacing（vsync 不
符——105ms 非 16.7ms 整数倍，弱候选）。

## 5. 详细设计

### T-00 复现勘定+域协调（G-1）

041 例 ladder 泵率谱复跑（731 带内核验——试验床成立断言）+
`AUTO_SCHED_DIAG=1` 到达谱采样+711 泵计数器在位勘定（四计数器
grep 锚+采样形态）；712 r2 残余 T-16..T-19 状态核验（零冲突
断言——有冲突则协调序注记）；iced/winit/wgpu 钉版勘定注记
（Cargo.lock 版本+升级约束面——非目标边界确认）。

### T-01 呈现事实定谳（G-2——决策件）

不 fork 约束下呈现节奏测量通道择优实施（候选清单见 §2——T-00
勘定后定）：候选 A=iced 0.14 现有钩子（window redraw/with
surface 全量面勘定）；候选 B=wgpu/渲染适配层 present 时点直读
（renderer.rs 自绘路径——716 根包装探针同域）；候选 C=OS 合成
器时戳（DwmFlush/DwmGetCompositionTimingInfo——Win32 面，进程
外真值）；候选 D=731 墙钟对照法形式化（begin 行数×墙钟 vs 通
知数——非逐帧但可判节拍）。产出=**呈现节奏 vs 通知节奏分离谱**
（三相位：type/scroll/typenl——定谳"呈现真慢 vs 通知滞后"）+
通道成本两态（门关零开销验证）。决策输出：下游 8-9.5fps 的定性
（呈现事实/通知伪影/混合）——T-03 双臂权重随此定。

### T-02 交付节奏根因（G-3——研究件）

四轴 trace（§4 候选 a-d——e 弱候选记录即可）：RedrawRequested
发放计数（订阅 receipt 面插桩——事件级）vs 消费时戳 vs 建帧计
数 vs 队列深度（泵计数器四件）。产出=根因报告（机制级定谳——
主导轴+次要轴）+修复方案选定（(a) 交付节奏修复臂/(b) 通道臂/
双臂——含成本与风险）；**边界护栏**：不改 dirty/epoch 调度语义
（712 r2 域）——若根因落该域，回 `needs_replan` 协调。

### T-03 修复/通道实施（G-4——依 T-01/T-02 判定）

臂 (a)：交付节奏修复——per-frame 通知保证（订阅/回环/重绘请求
面修——T-02 选定的最小面）；臂 (b)：呈现真相通道产品化
（frame_bench 增 `present_truth_us` 类字段——T-01 通道的谱面
落位）；R-1 序障重验（新时戳序断言）+门关零开销验证+731 §4b
语义同步升格（配对面口径→交付节奏契约）。

### T-04 谱对照+回归门（G-5）

ladder 改前/改后双谱（pump/s 逼近帧率带=修复生效判据；通道臂
=呈现/通知分离谱复现）+731 谱零回退（S1-S4 段和+S5 子段带）
+视觉 golden 三形+`cargo t` 日常面+`cargo tv` 触面+716 探针族
回归。

### T-05 下游回执+规范+账本（G-6）

downstream-handoff v2（呈现事实结论+交付节奏终态+通道语义建议
+重判解锁预告——669 模式第二段闭环）+SD-01/02 落档+P735-1
（015 先例——merge 期项）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/frame-observability.md | before：§4 边界注记（712 r2 域/多窗第二臂待办）+§5 present=-1 配对面口径 / after：§4 增交付节奏契约行+呈现真相通道边界；§5 配对面口径升格（呈现/通知分离语义+新时戳字段） | 呈现事实与通道语义的真源 | AC-02/03/04 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md | before：§4b 泵治理语义=配对面定谳（731 态） / after：§4b 升格=交付节奏契约（T-02 根因结论+修复语义+通知保证面） | 泵域契约随根因落档 | AC-03/04 |

## 6. 测试设计

- **泵率谱对照**：ladder 三相位双谱（改前/改后）——pump_per_sec
  目标带=≥帧驱动节奏×0.9（修复臂）/呈现-通知分离比注记（通道臂
  ）；orphan 计数对照（026 下游带 87% 为改前基）。
- **呈现真相通道验证**：通道时戳 vs OS 合成器真值抽检（候选 C
  在位时）或墙钟对照交叉（候选 D）——通道语义断言。
- **零回退门**：731 阶梯谱三档×三相位 segsum/S5 子段带+视觉
  golden 三形 0.00%。
- **序障+门控**：R-1 序（新时戳 ≥ 构建时戳）断言×N；门关零
  开销（716 微基准法——ns 级占帧预算比）。
- **分级门禁**：`cargo t` 日常面全绿+`cargo tv` 触面档+716 探针
  族（帧通道五面）回归。
- **范围断言**：auto-edit 零改动（其仓 porcelain 断言随回执）+
  dirty/epoch 调度语义零触碰（712 r2 域 grep 锚）。

## 7. 验收标准

- **AC-01 试验床+域协调**：上游例泵率谱复现带内（731 带对照）+
  712 r2 零冲突断言+钉版注记在档。验证：ladder JSONL+核验记录。
- **AC-02 呈现事实定谳**：呈现/通知分离谱在档（三相位）——
  下游 8-9.5fps 定性结论（呈现事实/通知伪影/混合）+通道成本
  两态验证。验证：分离谱 JSONL+定谳报告。
- **AC-03 交付节奏根因**：根因报告（四轴 trace——主导轴定谳）
  +修复方案选定记录（含边界护栏核验）。验证：根因报告+trace
  JSONL。
- **AC-04 修复/通道生效**：臂 (a)=泵率谱达带（≥帧驱动×0.9）；
  臂 (b)=呈现真相通道逐帧真值（语义断言+序障重验）；731 谱零
  回退+golden 绿。验证：双谱+回归门输出。
- **AC-05 下游回执+规范+账本**：downstream-handoff v2 在档（重
  判解锁预告——下游件预告位）+SD-01/02 落档+P735-1 回读 True
  （merge 期）+范围断言（auto-edit 零改动+712 r2 域零触碰）。
  验证：回执档+SD diff+账本断言。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 复现勘定+域协调 | — | 本件 §5 T-00 节 | 试验床+协调断言 | AC-01 | [x] 复现谱在档 |
| 1 | T-01 呈现事实定谳 | T-00 | 呈现测量通道（候选 A-D 择优） | 分离谱+定性 | AC-02 | [x] 分离谱在档 |
| 2 | T-02 交付节奏根因 | T-01 | renderer.rs:8349 泵域+订阅面 | 根因报告+方案选定 | AC-03 | [x] 根因定谳 |
| 3 | T-03 修复/通道实施 | T-02 | frame_bench/frame_segments/renderer.rs | 修复或通道+序障 | AC-04 | [x] 双谱达标 |
| 4 | T-04 谱对照+回归门 | T-03 | ladder+golden+分级门禁 | 零回退+生效 | AC-04 | [x] 门禁全绿 |
| 5 | T-05 回执+规范+账本 | T-01..04 | downstream-handoff v2+SD-01/02 | 第二段闭环 | AC-05 | [x] 回执在档 |

## 9. 复审记录

- 2026-10-03 起草 handoff：`stage: new`，PLAN-735，plan_revision 1。
  `outcome: pass`（起草完备：下游 026 回执四要点逐项承接〔呈现
  事实/交付节奏/通道语义/瞬态观察位〕；呈现事实先行+下游判定零
  触碰+R-1 序障保持+712 r2 域护栏四约束防冒进；机制候选五轴列全
  不预设；双臂设计允许根因驱动择优；路径/符号经 master@cb2da5fba
  实勘锚定〔frame_pump_sub:8349/note_frame_present/frame_segments
  双槽/§4b 语义〕；试验床=上游例本地同现象在案〔731 pump/s 谱〕
  ——零下游依赖可独立推进；授权=起草〔用户本会话指令〕，执行待
  启动——Q-1..Q-3 见 §10）。`next: work`。

- 2026-10-03 work handoff：`stage: work`，PLAN-735，plan_revision 1。
  `outcome: pass`。`code_commit`：worktree plan-735-dev
  5da3b4727（T-01/T-02 观测插桩）+7b9b2f8f5（T-03 配对修复+真相
  通道）+67a91eaad（T-04/T-05 双谱证据+SD-01/02 落档）；基面
  master@c989483ce；worktree=D:/autostack/.wt/lang-735/auto-lang
  （组内 auto-down detached@895f8d0 零改动）；依赖修订=iced 0.14.0
  （iced_winit **0.14.1** worktree 解析——谱系钉版注记入 T-00 报告，
  下游同版）/winit 0.30.13/wgpu 27.0.1/patches iced_widget 在位。
  `task_ids`：T-00..T-05 全完。`evidence`：
  - **AC-01**：ladder-p735-base.jsonl 泵率谱 0.42-3.16 带内（731 带
    0.41-2.47 对照）；711 四计数器采样全零在档；712 r2 零冲突
    （五在途分支对泵域五文件零提交实勘）；钉版注记在档。
  - **AC-02**：分离谱+定性=呈现随驱动更新 +6-10ms（呈现真快），
    ~95-110ms 节拍=驱动器调用周期传递（62.5ms p50+长尾实测）；
    731「呈现随键入」归因修正。T-01-T-02 报告 §2。
  - **AC-03**：四轴根因=发布点错位伪影（ready 唤醒链通知+
    __bounds_collected 旋转孤儿化，subF=0/enq=0 实证稳态订阅门关）；
    修复方案选定=双臂（配对修复+真相通道），无 fork/无订阅门变更/
    无 712 r2 域触碰。同报告 §3。
  - **AC-04**：行配对 9-33%→**100%**（改后两跑 260/260）+pump/s 达
    驱动节奏带（type/typenl ~10/s=驱动读数）+R-1 序 404 行 0 违例+
    draw_end 真相通道（行列+frame_bench 读侧）+S5 带零回退+golden
    三形 0.00%（基面源 baseline 对照）+cargo t 5022/5038（16 红全
    预存零新红——14 批量回执登记+ash 预存未登记+plan502 负载
    flake scoped 绿）+tv 162 绿+frame 51 绿+plan716 探针 19 绿。
    T-03-T-04 双谱报告。
  - **AC-05**：downstream-handoff-v2 在档（重判解锁预告+通道语义
    建议+VM 9920 下游立项建议——本件未接线零触碰）；SD-01/02 落档
    （worktree specs 提交——merge 期正式沉淀）；P735-1=merge 期项
    （/auto-plan:merge 执行）；auto-edit 仓零改动（ porcelain 断言
    ——本会话未触该仓）。范围断言：712 r2 域（dirty/epoch/
    poll_frame_pump 泵臂）零触碰——diff 仅 frame_segments 发布点+
    frame_bench 新原子+frame_probe draw 末点+renderer 观测订阅/trace
    行（全门控）。
  `blockers`：无。`next: review`（/auto-plan:review——独立复审门，
  P735-1 specs.json upsert 归 merge 期）。

- 2026-10-03 review：`stage: review`，PLAN-735，plan_revision 1。
  `outcome: pass`。`reviewed_commit`：67a91eaad（worktree
  plan-735-dev tip；三提交 5da3b4727/7b9b2f8f5/67a91eaad 全在
  base c989483ce 之上，树 clean）。`base_commit`：c989483ce。
  `dependency_revisions`：iced 0.14.0/iced_winit 0.14.1（worktree
  解析——下游 auto-edit 同版；731 带谱 0.14.0 差异已钉版注记）/
  winit 0.30.13/wgpu 27.0.1/patches iced_widget。
  `spec_inputs`：SD-01=frame-observability.md @SHA256
  66f43aa84e120ba8…；SD-02=frame-pipeline-incremental.md @SHA256
  b21fa6c94e10adb1…（frozen@67a91eaad——与 plan §5 规范增量表逐项
  对读一致，零超新增需求）。
  `acceptance_results`：AC-01..05 全 pass——AC-01 带内复现+协调
  （在途五分支泵域零提交复审再勘）；AC-02 分离谱+定性（呈现随
  驱动 +6-10ms/节拍=驱动周期传递）；AC-03 四轴根因+双臂选定；
  AC-04 行配对 25%→100%（JSONL 复算 55/220→184/184+76/76=260/260）
  +R-1 零违例复算+golden 三形 0.00%（735-{type,scroll,resize}.png
  在档）+`cargo t` 5022/5038 零新红（16 红全预存——14 批量回执
  登记+ash 主检出同红+plan502 scoped 绿）+`cargo tv` 162 绿；
  AC-05 回执 v2+SD-01/02 在档+P735-1=merge 期+auto-edit porcelain
  零改动+712 r2 域零触碰（renderer diff 三 hunk 复勘——无
  dirty/epoch/poll_frame_pump 触碰）。复审复跑：`cargo t frame
  plan716_supply` 55/55 绿（同码同配置复用 t/tv 全档证据——
  码/deps/配置自运行未变，复用理由=diff 仅 docs）。`findings`：
  F-NB1 单跑 5kb/typenl 日志尾截断切片（复跑 43/43=100% 非缺陷）；
  F-NB2 segsum type 相位跨跑负载漂移 +12-34%（逐段均匀+S5±10%+
  golden 0.00%+diff 事实承载零回退）；F-NB3 fmt 漂移=遗留 examples
  （p023_probe 等五文件，base 同文——非本件引入）。独立性注记：
  同会话复审——结论自工件重建（diff hunk/JSONL 复算/复跑）非执行
  摘要采信。`evidence`：docs/plans/evidence/735/（双谱 JSONL+两报
  告+回执 v2+ladder735.py/golden735.py/探针脚本）。`next: merge`。

- 2026-10-03 merge：`stage: merge`，PLAN-735:r1。`outcome: pass`。
  收据五检查点（证据实名）：
  - `prepared`：reviewed 基线=67a91eaad（worktree 三提交
    5da3b4727/7b9b2f8f5/67a91eaad，基 c989483ce）；canonical Spec
    diff=SD-01/02（worktree 提交在案，frozen SHA256 66f43aa8…/
    b21fa6c9…）；投影目标=P735-1（designs）+P735-2（reviews）；
    delivery commit=4195f7af9。
  - `landed`：master@4195f7af9（ff-only 无合并提交）。rebase 两次
    （master 并行前移——730/734/736 落库）range-diff 全等证明：
    5da3b4727=87faaadc3、7b9b2f8f5=4ff3670c8、67a91eaad=4195f7af9
    （逐对 `=`）；rebased 态 scoped 复跑 55/55 绿。主检出冒烟
    （落地后含并行面）：cargo t frame plan716_supply 55/55 绿。
  - `ledger_refreshed`：.autoos/specs.json P735-1（designs——帧观测
    分离+交付节奏契约）+P735-2（reviews pass 收据）回读 verified
    （file/status/related 全对，760 items）；ui/plans.md 735 行+
    ui/overview.md 注记+INDEX 再生（spec-index.py 26 projects）；
    投影提交=d2c6209fa。
  - `archived`：docs/plans/archive/735-frame-pump-presentation-
    pacing.md，status: archived（本提交）。
  - `cleaned`：（清理后回填——wt-guard 双组树 clean+worktree/branch/
    组目录注销）。
  部署观察：零常驻产物消费本仓构建（auto run 按需源构建；下游
  auto-edit bench 档自建 auto-lang）——零重建项，732 惯例维持。
  批量回归到期判定：735%5=0 且 735>732（receipt last_covered）→
  **到期**——cleaned 后由 /auto-plan:regress 主检出单实例执行。

## 10. 待澄清事项

- **Q-1 fork iced_winit 约束重估（已决——T-00/T-01 勘定闭环）**：
  711 D-1 注记"无 post-present 回执，硬屏障需 fork iced_winit"。
  本件 T-01 候选 B（根包装 draw 末点绝对时戳）在**不 fork** 约束下
  成立——iced_winit RedrawRequested 臂内 draw 同步先行于 compositor
  present（0.14.0/0.14.1 双版源勘，lib.rs :924-940/:950-981），逐帧
  呈现真值无需 post-present 回执；fork 解除议题**关闭**（无必要）。
- **Q-2 修复深度（已决——T-02 判定闭环）**：根因=测量通道发布点
  错位，非 iced/winit 行为面缺陷——修复零触及 iced 公开 API 之外
  （frame_segments/frame_bench 纯应用层观测面）；winit 事件循环
  参数面零改动。执行深度未超预案，无需用户过目项。
- **Q-3 下游协议裁定联动（非本件域——注记维持）**：T-01 定谳=
  「呈现事实+通道伪影」混合（呈现跟驱动节奏；伪影=配对错位已修），
  下游 022 frozen 口径的读回语义适配=下游重判件事——本件回执给
  结论与通道（downstream-handoff-v2 §3 建议），裁定权在下游件
  （跨仓协调注记，非阻塞）。

## 11. spec-sync 回写记录

- 2026-10-03 merge（d2c6209fa）：SD-01→docs/specs/auto-lang/ui/design/
  frame-observability.md（§1 通道三时戳/§4 交付节奏契约+真相通道边界/
  §5 配对面口径升格）；SD-02→docs/specs/auto-lang/ui/design/
  frame-pipeline-incremental.md（§4b 泵治理语义升格=交付节奏契约）。
  账本 P735-1（designs）/P735-2（reviews）；ui/plans.md 735 行；
  ui/overview.md PLAN-735 注记；INDEX 再生。
