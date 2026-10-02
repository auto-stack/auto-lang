---
plan_id: PLAN-735
status: drafting               # drafting → executing → execution_done → reviewed → archived
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
current_step: 0
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
| 0 | T-00 复现勘定+域协调 | — | 本件 §5 T-00 节 | 试验床+协调断言 | AC-01 | [ ] 复现谱在档 |
| 1 | T-01 呈现事实定谳 | T-00 | 呈现测量通道（候选 A-D 择优） | 分离谱+定性 | AC-02 | [ ] 分离谱在档 |
| 2 | T-02 交付节奏根因 | T-01 | renderer.rs:8349 泵域+订阅面 | 根因报告+方案选定 | AC-03 | [ ] 根因定谳 |
| 3 | T-03 修复/通道实施 | T-02 | frame_bench/frame_segments/renderer.rs | 修复或通道+序障 | AC-04 | [ ] 双谱达标 |
| 4 | T-04 谱对照+回归门 | T-03 | ladder+golden+分级门禁 | 零回退+生效 | AC-04 | [ ] 门禁全绿 |
| 5 | T-05 回执+规范+账本 | T-01..04 | downstream-handoff v2+SD-01/02 | 第二段闭环 | AC-05 | [ ] 回执在档 |

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

## 10. 待澄清事项

- **Q-1 fork iced_winit 约束重估（条件活——T-00 勘定后）**：711
  D-1 注记"无 post-present 回执，硬屏障需 fork iced_winit——计划
  约束禁止"。本件 T-01 候选 A-C 均为绕行通道（不 fork）；若勘定
  证明绕行面全部不可行且呈现真相是必要件，fork 约束的解除=用户
  裁定件（本件不预设）。
- **Q-2 修复深度（T-02 判定后示知）**：臂 (a) 交付节奏修复的
  行为面改动（订阅语义/回环结构）若触及 iced 公开 API 之外
  （winit 事件循环参数面等），成本与升级约束=T-02 报告内容，
  执行启动前用户过目。
- **Q-3 下游协议裁定联动（非本件域——注记）**：若 T-01 定谳为
  "通知伪影"（呈现其实跟帧走），下游 022 frozen 口径的读回语义
  适配=下游重判件事——本件回执给结论与通道，裁定权在下游件
  （跨仓协调注记，非阻塞）。
