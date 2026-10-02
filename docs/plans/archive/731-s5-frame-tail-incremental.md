---
plan_id: PLAN-731
status: archived
feature_name: S5 帧尾段增量化件（帧两行清偿本体——layout/shaping/draw 段插桩归因+shaping 缓存与视口增量+layout 增量+组件重建消除+掉泵治理——下游 type_latency/scroll_fps 重判解锁）
author: [agent]
created_at: 2026-10-02T21:10:05+08:00
updated_at: 2026-10-02T21:10:05+08:00
plan_revision: 1
current_step: 8
total_steps: 9
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md（SD-01：modify——增 S5 段契约：子段插桩口径/shaping 缓存与视口增量/layout 增量/组件实例缓存/泵治理语义）"
  - "docs/specs/auto-lang/ui/design/frame-observability.md（SD-02：modify——S5 子段插桩扩展[layout/shaping/draw 三时间戳]——残差口径退役）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料/下游谱驱动面，703/728 先例注记式）
affects: [crates/auto-lang/src/ui/iced/, crates/auto-lang/src/ui/frame_bench.rs, crates/auto-lang/src/ui/code_editor/]
---

# [PLAN-731] S5 帧尾段增量化件（帧两行清偿本体）

## 0. 变更摘要

auto-edit **M4 发布门最后缺口**——帧两行 armed FAIL（type_latency
P95 95-112ms vs ≤16.7ms≈1 帧、scroll_fps 6.7-8.8fps vs ≥54——022
首判/024 重判谱）——的**清偿本体**。024 归因回执（auto-edit
`specs/auto-edit/tests/evidence-p024-frame-rejudge.md`）定界：725
增量管线后 S1-S4 段和仅 ~1.8ms 而**帧总 ~110ms——~108ms 为段外
=S5 域**（layout/shaping/draw，未插桩）；键入尾帧与 `ce_widget_new=1`
（编辑器组件重建）同现+**首帧 shaping 债**（editor-kernel「现状
限制」在册）；滚动段 begin→begin ~108ms 节奏+**present=-1 掉泵
直落**主导（无泵呈现 fall-through）+视口推进→**新暴露行整形**。
证据文件已点名修复方向：**shaping/layout 缓存+视口增量**。本件
五改：**①S5 子段插桩**（layout/shaping/draw 三时间戳——残差口径
退役，门控零开销纪律承袭）→ **②shaping 缓存+视口增量**（滚动只
整形新暴露行；首帧 shaping 债承接）→ **③layout 增量**（iced 全窗
layout→脏域/缓存——0.14 增界面 T-00 勘定）→ **④组件实例缓存**
（ce_widget_new 尾帧重建消除）→ **⑤掉泵治理**（present fall-
through 消除）+阶梯谱改前/改后对照（725 G-5 同款）+**下游重判
解锁回执**（auto-edit 帧两行重判收口件预告——v0.1-M4.2/M4 完全体
位点）。**下游重判件不在本件**（669 模式——本件出上游谱与回执）。

## 1. 目标

- **G-1 S5 子段插桩（观测先行）**：frame_bench 通道扩展——layout/
  shaping/draw 三子段时间戳（SD-B 增面；AUTO_FRAME_BENCH 门控零
  开销纪律承袭[门关 ns 级]）；**残差口径（total−segsum）退役**——
  S5 从"未插桩残差"升为分段口径；键入/滚动两形基线谱（S5 内部
  占比分解——改造面数据定界）。
- **G-2 shaping 缓存+视口增量**：视口推进只整形**新暴露行**（已
  整形行缓存命中——024 归因"视口推进→新暴露行整形"直对策）；
  首帧 shaping 债（editor-kernel「现状限制」在册）承接——warm/
  渐进策略（T-00 定形）；缓存键=行内容+字号/字体态（编辑后失效
  域窄化——与 725 脏域语义协同）。
- **G-3 layout 增量**：iced 0.14 全窗 layout→增量形（T-00 勘定其
  增界面：tree 结构缓存/只 layout 脏子树/布局结果 memo——按可
  行面裁剪；不可行面记录+替代[如 layout 剪枝/短路]）；**视觉零
  变化**（frozen ①——golden 对照）。
- **G-4 组件实例缓存**：`ce_widget_new` 尾帧重建消除（编辑器组件
  实例跨帧缓存——046 memo/keyed 域协同注记：实例缓存与 memo 失效
  正交性 T-00 勘定）；重建计数→0 或受控谱。
- **G-5 掉泵治理**：present=-1 fall-through 消除（无泵呈现直落→
  泵呈现兜底/调度修正——711 R-1 序障合法代理面与 712 r2 帧泵域
  边界协调[§10 Q-2]）；泵率谱（present/帧占比）。
- **G-6 阶梯谱对照**：键入形（文档 5KB/100KB/1MB）+滚动形——
  改前/改后 S5 分段谱+帧总谱（725 G-5 同款口径）；**下游判定
  预期校准**（P95 ≤16.7ms/≥54fps 的上游证据谱——判定本身归
  下游重判件）。
- **G-7 下游重判解锁回执**：回执节成文（改后谱+重判件预告——
  auto-edit 帧两行重判收口件[v0.1-M4.2 位点]；判定口径 frozen
  下游 022 原样——本件不重定义）。
- **G-8 规范+账本**：SD-01 S5 段契约+SD-02 插桩扩展+P731-1。

### 非目标

- **下游重判件**（auto-edit bench 帧档复跑+budgets 转绿+对比表
  刷新——本件 delivered 后消费件另立；判定口径不换[frozen ③]）。
- 725 已落段重构（S1-S4——单帧单建/载荷增量/脏域重建**零回退**
  [frozen ②]）；GPU/wgpu 渲染换轨与 RQHost（渲染拓扑域——Q2
  裁定面）；供⑭/供⑮ 域。
- 文本编辑语义/键入载荷面变更（725 已定形）；视觉输出任何变化
  （frozen ①——布局/着色/滚动行为 golden 对照全绿为门）。
- 712 r2 域（VM 装载调度——帧泵**观测与呈现兜底**面协同不并案；
  §10 Q-2 协调位）；多窗 runner 第二泵臂（716 SD-B 待办注记位
  ——T-00 勘定涉及时一并注记不扩案）。

## 2. 架构方案

分层落点（2026-10-02 实勘，auto-lang master@93cd69a55）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| S5 观测 | 残差口径（total−segsum——725 SD-01:58 注记「未插桩」）；三子段无时间戳 | **子段插桩**（layout/shaping/draw——frame_bench 扩展+SD-B 增面） | 024 归因回执；716 SD-B 通道 |
| shaping | 每帧全量整形+视口推进新暴露行整形（024 实录）；首帧 shaping 债在册 | **行级缓存+视口增量**（缓存键/失效域——725 脏域协同） | 证据文件点名方向 |
| layout | iced 0.14 全窗 layout/帧 | 增量形（增界面勘定——脏子树/缓存/短路） | 725 五段链⑤定义 |
| 组件 | ce_widget_new 尾帧重建（024 实录） | 实例缓存（046 memo 域协同——正交性勘定） | 同上 |
| 泵 | present=-1 fall-through 直落（024 滚动实录） | 呈现兜底/调度修正（711 代理面/712 边界协调） | 716 SD-B 域注记 |
| 判定 | 下游 022/024 谱 FAIL 维持 | 上游改后谱=重判依据（判定归下游——口径 frozen） | 669 模式 |

**关键设计约束（frozen）**：
① **视觉零变化**——布局/着色/滚动行为 golden 对照全绿为门（任何
视觉差=回归非优化）。② **S1-S4 零回退**——725 已落段不重构；键入
脏帧 S1-S4 段和 ~1.8ms 带不劣化入验收。③ **判定口径不换**——下游
022 帧内口径/面板×0.9 原样；本件只压时长不重定义测量。④ **分段
计量先行**——T-00 插桩归因定改造优先序（数据驱动，不盲改）。⑤
712 r2/721 余题域协调（帧泵同域——改动面避让或协调[§10 Q-2]）。

## 3. 技术栈

Rust workspace（ui/iced 渲染层+frame_bench+code_editor 协同面）；
阶梯谱基准（725 G-5 脚本基建复用——autoui-verifier 形态）；视觉
golden 对照（视图快照族——015/022 矩阵口径的上游对应）；cargo tf
全量门+预存对账（703/728 惯例）；VM/a2r 双轨注记（iced 层共享——
重判在 release 全链[下游谱形态]）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-10-02 会话指令「OK，那么给 S5 立项」
（承接上轮「要让 M4 走到名副其实的完全体，就要把 S5 排进
auto-lang 队列」之议——用户确认立项）——授权=**起草本件**；执行
/work 待用户另行启动（auto-lang 会话——729/730 HTTP 域并行在途，
启动时核 731 编号与建组；帧泵域与 712 r2 余题协调[§10 Q-2]）。
范围=auto-lang crates/docs/specs；auto-edit 零改动（回执=文档面）。
无预算/自动续跑授权。

**来源与版本**：

- 归因基：auto-edit `specs/auto-edit/tests/evidence-p024-frame-
  rejudge.md`（S5 定界全文——~108ms 段外/layout/shaping/draw 未
  插桩/ce_widget_new 同现/首帧 shaping 债[editor-kernel「现状
  限制」在册]/滚动 begin→begin ~108ms+present=-1 掉泵直落/视口
  推进新暴露行整形/修复方向点名「shaping/layout 缓存+视口增量」）。
- 谱系：022 帧档首判（P50 110/P95 115ms、8.0fps——armed FAIL
  基线）+024 重判（P95 95-112ms、6.7-8.8fps——725 后 FAIL 维持
  谱+S1-S4 段和 ~1.8ms 对照）。
- 契约基：725 SD-01 frame-pipeline-incremental（五段链定义[S5=
  ⑤layout+draw iced 内部段]+残差口径注记 :58）+716 SD-B frame-
  observability（通道/门控/0=未捕获值语义——本件扩展面）。
- 并行域注记：712 r2 归档余题（P712-D1 家族）+721 余题 R1（读侧
  机理）——帧泵/更新路径同域协调；046 memo/keyed（组件缓存正交
  性）；729/730 HTTP 域无路径冲突。
- 下游回执预告位：M4.1 tag 后余题清单头位（S5=发布门前置——
  本件 delivered 后下游重判收口件即 v0.1-M4.2/M4 完全体位点）。

## 5. 详细设计

### T-00 S5 插桩+归因（决策件）

1. **子段插桩**：layout/shaping/draw 三时间戳（frame_bench 扩展
   ——门控纪律承袭；SD-B 增面）；键入/滚动两形基线谱——S5 内部
   三子段占比分解（数据定改造优先序）。
2. **iced 0.14 增界面勘定**：layout 增量可行面（tree 缓存/脏子树/
   布局 memo/短路——源码实勘定界；不可行面记录+替代方案）。
3. **shaping 缓存定形**：缓存键（行内容+字体/字号态）/失效域
   （725 脏域协同——编辑行窄化）/首帧债策略（warm/渐进——editor-
   kernel 现状限制承接）。
4. **组件缓存正交性勘定**（046 memo 域——实例缓存与 memo 失效面
   交互）+**掉泵机理定位**（present fall-through 根因——泵调度/
   呈现条件实勘；711 代理面边界）。
5. **视觉 golden 基线**：改前快照族（键入/滚动/resize 三形——
   改后对照门）。

### T-01 shaping 缓存+视口增量（G-2）

行级整形缓存（键/失效域按 T-00③）；视口推进只整形新暴露行（命中
谱断言）；首帧债策略落地；缓存内存契约注记（行缓存上界——大文件
协同[728 分页：只缓存可见域+预取窗——上界同族]）。

### T-02 layout 增量（G-3）

依 T-00② 可行面实施（脏子树/缓存/短路择一或组合）；**视觉 golden
对照全绿为门**（frozen ①）；不可行面记录+替代案成效谱。

### T-03 组件实例缓存（G-4）

ce_widget_new 尾帧重建消除（编辑器组件实例跨帧缓存——失效条件=
结构性[resize/lang 态]非每帧）；重建计数断言（→0 或受控）；046
正交性验证（memo 面零扰动）。

### T-04 掉泵治理（G-5）

present fall-through 消除（依 T-00④ 根因——泵呈现兜底/调度修正）；
泵率谱（present/帧占比——滚动形达标带校准）；712 r2 域边界协调
注记（§10 Q-2）。

### T-05 阶梯谱对照（G-6）

键入形（5KB/100KB/1MB）+滚动形——改前/改后：S5 三子段谱+S1-S4
段和（零回退门）+帧总谱；**下游判定预期校准**（P95 ≤16.7ms/≥54fps
上游证据——判定归下游）。

### T-06 回归门（G-6/725 frozen）

cargo tf 全量（预存对账零新增）+725 谱零回退（S1-S4 段和带）+
视觉 golden 三形全绿+716 SD-B 探针族回归。

### T-07 下游回执+规范+账本（G-7/8）

回执节（改后谱+重判件预告——供料档或 024 回执对应位）；SD-01 S5
段契约+SD-02 插桩扩展落档；P731-1（703/728 先例）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md | before：五段链契约（S1-S4 增量语义已落[S5=残差口径未插桩注记 :58]） / after：增 S5 段契约——子段插桩口径/shaping 缓存与视口增量语义（键/失效域/内存上界）/layout 增量形（可行面记录）/组件实例缓存失效条件/泵治理语义+改后阶梯谱 | 帧管线增量契约的尾段补全 | AC-02..06 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/frame-observability.md | before：双时间戳通道（begin/present——S5=残差） / after：S5 子段插桩扩展（layout/shaping/draw 三时间戳——门控零开销纪律同款；残差口径退役注记） | 观测面与管线面同步 | AC-01 |

## 6. 测试设计

- **插桩**：三子段时间戳到达序+门开/关两态零开销实证（716 纪律
  同款复验）。
- **shaping 缓存**：命中/失效单测（编辑后窄化失效——未编辑行缓存
  保持）+视口推进增量断言（新暴露行整形数=新暴露行数）+大文件
  协同注记（728 分页装载下缓存域）。
- **layout/视觉**：golden 三形（键入/滚动/resize）改前改后对照
  全绿——**任何像素/结构差=红**；布局不变式单测。
- **组件缓存**：ce_widget_new 计数断言（键入 N 帧重建=0）+046
  memo 面回归。
- **泵率**：present/帧占比谱（滚动形≥目标带）+fall-through 计数
  →0。
- **阶梯谱**：改前/改后双谱（725 脚本复用）+S1-S4 段和带（~1.8ms
  不劣化）。
- **回归门**：cargo tf 预存对账零新增+716/725 探针族回归。

## 7. 验收标准

- **AC-01 S5 插桩**：三子段时间戳在档（两形基线谱）+门控零开销
  复验绿+残差口径退役注记。验证：探针+谱。
- **AC-02 shaping/视口增量**：缓存命中谱+新暴露行增量断言+失效
  域窄化单测绿。验证：单测+谱。
- **AC-03 layout 增量**：可行面实施（或不可行面记录+替代案成效）
  +**视觉 golden 三形全绿**。验证：golden 对照。
- **AC-04 组件缓存**：尾帧重建计数→0（或受控谱）+046 回归绿。
  验证：计数断言。
- **AC-05 掉泵治理**：fall-through →0（或受控+归因注记）+泵率谱
  达标带。验证：泵率谱。
- **AC-06 阶梯谱+零回退**：改后 S5 谱在档（三子段压降）+S1-S4
  段和带不劣化+帧总谱（下游判定预期校准注记）。验证：双谱对照。
- **AC-07 回归门**：tf 预存对账零新增+探针族回归绿。验证：tf
  输出对账。
- **AC-08 规范+账本+回执**：SD-01/02 落档+P731-1 回读 True+下游
  回执节在档。验证：文件在档+账本断言。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 插桩+归因 | — | frame_bench+iced 源勘+勘定报告 | 子段谱+四定形+golden 基线 | 全 | [x] 报告在档（evidence/731/T-00-survey.md：定谳 ce_fold_scan=S5 主导[1MB debug 6.3ms/帧=90%]、整形/layout 已受控、尾帧上游不复现[下游特有]、四定形+双态基线谱+golden 三形自洽 0.00%；commit 7454e4c13） |
| 1 | T-01 shaping 缓存 | T-00 | iced 渲染层/cosmic-text 协同面 | 视口增量+缓存 | AC-02 | [x] 命中谱+单测绿（fold 域 revision 缓存[core::cached_fold_regions]：ce_fold_scan 6.3-7.8ms→~1µs；1MB s5 P50 debug 6.60→0.81ms[-88%]/release 1.16→0.36ms[-69%]；滚动 P95 13.88→1.49ms[首轮 38.66 波动注记在档]；单测 p731×3 绿+code_editor 129 绿+fold 31 绿+golden 三形 0.00%；commit 0664025e7） |
| 2 | T-02 layout 增量 | T-00 | iced layout 面 | 脏域/缓存形 | AC-03 | [x] 勘定记录+替代案成效（iced 0.14 无增界面+实测 s5_layout release 0.02-0.05ms/帧非瓶颈——Q-1 替代案路线；golden 三形 0.00%；T-05-comparison.md §3） |
| 3 | T-03 组件缓存 | T-00 | code_editor 组件面 | 重建消除 | AC-04 | [x] 受控谱（ce_widget_new 实测 0.17ms/rebuild 非成本——024 因果勘定修正；memo 92/92 绿=046 正交零扰动；T-05-comparison.md §4） |
| 4 | T-04 掉泵治理 | T-00 | 帧泵呈现面 | fall-through 消除 | AC-05 | [x] 泵率谱+归因注记（pump_per_sec/orphan 计数入 ladder+JSONL；present=-1=测量配对面定谳[墙钟对照实证非呈现丢失]；T-05-comparison.md §2） |
| 5 | T-05 阶梯谱 | T-01..04 | 725 谱脚本复用 | 改前/改后双谱 | AC-06 | [x] R1 修复复验（F-1：T-00 态二进制全档基线重跑归档——三档×三相位+泵率齐备；F-2：segsum 文件实算重写[1MB +3.8%/5KB -4.2% 同带+100KB -36% 改善向=s3a 波动，无系统性回退]；数字全部出自 JSONL 在档） |
| 6 | T-06 回归门 | T-01..04 | tf 全量+golden+探针族 | 回归门 | AC-03/07 | [x] 对账零新增（裸 cargo t 全档 5034 run/5019 绿/15 红：13 精确对上批量回执 known_reds_seen[musk p053×4+p054×2/plan606_gallery/projector/desktop_bus/desktop_surface/schema×2+kitchen_sink]+2 环境负载红[lock_serializes 隔离绿=并行 flake；ash_leak_probe=autoterm_core.dll 跨仓工件缺席=新 worktree 环境红]——零 731 归因新红；frame_bench/frame_segments/code_editor 129/fold 31/memo 92 探针族绿；golden 0.00%×3；计划文本 tf 归批量档[fix-test-tiering 裁定，dep_parity_018 超时亦负载 flake——隔离 0.785s 绿]） |
| 7 | T-07 回执+规范+账本 | T-05/06 | SD-01/02+P731-1+回执节 | 落账与回执 | AC-08 | [x] R1 修复复验（F-2 定量门措辞如实化+F-5 SD-02 face 面覆盖注记+F-3 arm_count_n 实计+F-4 占位符填实；回执数字同步在档版） |

## 9. 复审记录

- 2026-10-02 起草 handoff：`stage: new`，PLAN-731，plan_revision 1。
  `outcome: pass`（起草完备：024 归因回执逐点承接[S5 定界/修复
  方向点名/两形机理]——五改各有所据；观测先行[frozen ④——子段
  插桩归因定序]防盲改；视觉零变化+S1-S4 零回退+判定口径不换三
  frozen 防破坏性；712 r2/046 域协调显式注记；下游重判边界清晰
  [669 模式——v0.1-M4.2/M4 完全体位点预告]；路径/符号经
  auto-lang@93cd69a55 与 auto-edit 024 回执/022-024 谱三源锚定；
  授权=起草[用户「给 S5 立项」指令在录]，执行待用户启动——
  auto-lang 会话，729/730 并行+帧泵域协调[§10 Q-2]）。`next:
  work`。
- 2026-10-02 执行 handoff：`stage: work`，PLAN-731，plan_revision 1。
  `outcome: pass`（八任务全清偿：T-00 插桩归因[frame_probe 根包装
  三子段+ce 臂钻取+双态基线谱+golden 三形自洽门——收敛门修正截图
  抢跑]→定谳 **ce_fold_scan=S5 绝对主导**[1MB debug 6.3ms/帧=90%，
  每帧全文档括号扫描]与**022/024 尾帧上游不复现**[debug+release×
  type/scroll/typenl 全相位——下游 app 特有，插桩随件下沉供其重判
  归因]；T-01 fold 域 revision 缓存[命中谱 6.29ms→~1µs；1MB s5
  P50 debug -88.5%/release -69%；滚动尾帧 P95 38.66→1.49ms]；T-02
  layout 增量=不可行面勘定+受控谱[实测 0.02-0.05ms/帧非瓶颈——Q-1
  替代案路线数据成立]；T-03 组件缓存=受控谱[ce_widget_new 0.17ms/
  rebuild 非成本——024「重建→重整形」因果勘定修正；memo 92/92 绿=
  046 正交]；T-04 泵率谱通道+配对面归因[present=-1=测量面非丢失，
  墙钟对照实证]；T-05 四谱对照+golden 0.00%×3（S1-S4 零回退主张 R1 F-2 修正——见 R1）；
  T-06 裸 cargo t 全档对账[5034 run/5019 绿/15 红=13 预存 known_reds_seen
  +2 环境负载红——lock 隔离绿/ash dll 缺席]——零 731 归因；T-07 SD-01 §4b/§5b+SD-02 §5 worktree 落档+下游回执
  downstream-handoff.md）。三 frozen 全程无违例。`code_commit`:
  plan-731-dev 7454e4c13/0664025e7/fd5041658/fb071e2d1（worktree
  D:/autostack/.wt/lang-731/auto-lang，基面 master@7d50989f7，组内
  auto-down 兄弟 detached@895f8d0）。`task_ids`: T-00..T-07 全。
  `evidence`: docs/plans/evidence/731/{T-00-survey.md,
  T-05-comparison.md,downstream-handoff.md,ladder*.jsonl,golden.py,
  ladder.py}+golden 基线 PNG。`blockers`: 无。`next`: review。
  注记：计划文本「cargo tf 全量」按 fix-test-tiering（2026-09-30
  裁定）调整=裸 cargo t 复审门+tf 归批量档（/auto-plan:regress 主
  检出单实例）；P731-1 ledger/specs.json 发布归 merge 件
  （auto-plan-merge 技能面）。

- 2026-10-02 R1 复审（同会话声明：与执行同 session——结论自工件重建：
  diff 直读/探针族复跑/证据 JSONL 实算，不采信执行摘要）。`stage: review`，
  PLAN-731，plan_revision 1。`outcome: needs_fix`。`reviewed_commit`:
  fb071e2d1（plan-731-dev @ worktree lang-731）；`base_commit`: 7d50989f7；
  `dependency`: auto-down detached@895f8d0f。`spec_inputs`: SD-01/SD-02
  worktree 版（fb071e2d1）。`acceptance_results`: AC-01 pass（三子段
  在档+门控单测绿+残差退役注记）/AC-02 pass（命中谱 6.29ms→~1µs+失效
  窄化单测×3+等价单测——视口增量=命中谱断言形，计划 §5 T-01 原文口径）
  /AC-03 pass（不可行面记录+替代案成效+golden 三形 0.00%）/AC-04 pass
  （受控谱+memo 92/92）/AC-05 pass（受控+归因注记+泵率谱通道；达标带
  校准归下游——计划未定义数值带，注记在案）/AC-07 pass（裸 cargo t
  5034/5019 绿+15 红全对账[13 预存 known_reds_seen+2 环境/负载]，零 731
  归因新红；tf 归批量档=fix-test-tiering 裁定）/AC-06 partial/AC-08
  partial——受 F-1/F-2/F-3 拖累。`findings`:
  **F-1**（AC-06/证据包；高）ladder-baseline.jsonl（debug）被 1mb-only
  复跑覆写——5kb/100kb 基线行丢失；T-00-survey/T-05-comparison 引用
  7.06/38.66 等出自首次运行控制台，与在档文件（6.24/7.51）不一致——
  修复=基提交（7d50989f7）重建 debug 二进制全档重跑基线（终版脚本含
  typenl/泵率）覆写归档+文档数字对齐在档文件；
  **F-2**（AC-06/SD-01 §4b；高）「S1-S4 segsum 零回退 ±2%」无证据
  支撑——在档实算 1MB 段和均值 base 7.71 vs after 8.88（跨跑波动
  ~±8-15%）——修复=实算重写（S1-S4 打点/实现路径零改动=diff 事实
  [frame_segments S1-S4 打点与 renderer 主路径仅加包装]+跨跑带如实
  陈述），SD-01 §4b 定量门措辞同步；
  **F-3**（AC-01 臂钻取语义；中）arm_count("ce_text_runs") 实计编辑器
  draw 调用数非 fill_text 段数——修复=按 list.text_runs.len() 实计或
  更名；
  **F-4**（簿记；低）§9 work 记录占位符 `<T-07 提交>` 未填 fb071e2d1；
  **F-5**（注记；低）dynamic_view_impl 的 view_named(face) 早退路径
  （dashboard face 面）无 S5 探针覆盖——SD-02 边界注记补记（bench/
  ladder 主路径不受影响）。`evidence`: 复跑探针族 p731×3/frame×51/
  code_editor×129/fold×31/memo×92 全绿；diff 直读（renderer 两包装点+
  中心编辑臂 bump+失效路径审计 set_text:724/edit:771/paged:833/
  load:933/handle_input 1540-1807/undo 剪贴板 2245-2296 全 bump）；
  定量核对 release 1mb type 1.16→0.36 ✓/scroll P95 2.35→0.94 ✓
  /debug after 0.81/1.49 ✓（after 文件一致）——**debug baseline 文件
  与文档引用不一致（F-1）**。`next`: work（R1 修复→R2 复审）。

- 2026-10-02 R2 复审（同会话工件重建，承 R1）。`stage: review`，PLAN-731，
  plan_revision 1。`outcome: pass`。`reviewed_commit`: 8e95ac426（R1 修复
  后 plan-731-dev HEAD；R1 基线=fb071e2d1）。`findings_resolved`:
  F-1 ✓（基线重建点勘定修正=**T-00 态 7454e4c13**（非裸基——s5 字段
  依赖插桩），组内 worktree 二进制全档重跑归档——ladder-baseline.jsonl
  3 档×3 相位 9 摘要+泵率齐备[1mb 6.60/8.67/6.62+滚动 P95 13.88]；首轮
  7.06/38.66 降级为跨跑波动注记——survey/comparison/回执数字全面对齐
  在档）；F-2 ✓（segsum 文件实算：5KB -4.2%/1MB +3.8% 同带、100KB
  -36% 改善向=s3a MCP 同步块跨跑波动主源[s1/s2 恒定 0/0.15ms 分段分解
  实证]——「无系统性回退」以 diff 零改动事实+跨跑带如实陈述，SD-01
  §4b 定量门措辞同步）；F-3 ✓（arm_count_n 按 list.text_runs.len()
  实计——基线侧旧口径[draw 计数]注记：ce_text_runs 未参与任何主张）；
  F-4 ✓（占位符填实）；F-5 ✓（SD-02 face 面覆盖边界注记）。
  `evidence`: R2 背书校验 5/5 PASS（9 摘要+pump 齐备断言+6.60/13.88/
  0.81/1.49/0.36 文件直读全中）；修复后 code_editor 129+frame 51 绿；
  R1 代码面=门控探针（arm_count_n）+文档——AUTO_FRAME_BENCH 未设零
  行为差（golden/full-face 证据不受累）；worktree clean@8e95ac426。
  `acceptance_results`: AC-01..AC-08 全 pass（AC-06/AC-08 自 partial
  升格；AC-08 的 P731-1 回读 True 保持 merge 件域——review 纪律不动
  live ledger）。`next`: merge（auto-plan-merge：SD-01/SD-02 发布+
  P731-1+归档+worktree 组清理）。

## 10. 待澄清事项

- **Q-1 iced layout 增量路线（T-00② 内定，非阻塞）**：0.14 增界面
  实勘定（脏子树/缓存/短路——可行面择优）；若全面增量不可行，
  **替代案**（剪枝/短路+shaping 缓存为主力）亦为合法交付——以
  阶梯谱成效为准，不预设路线。
- **Q-2 712 r2/721 余题域协调（执行启动时核）**：T-04 掉泵治理
  与 712 r2 帧泵域（dirty/epoch/装载调度）同域——启动时核其余题
  状态（已清偿则直改；在途则改动面避让或协调序）；冲突不可避时
  T-04 可后置（T-01..03 主力不受阻——阶梯谱分段成效可分验）。
- **Q-3 首帧 shaping 债策略（T-00③ 内定）**：warm（装载期预算
  整形）vs 渐进（前 N 帧分摊）——按装载墙钟带[728 谱 4.0s 内不
  显著劣化]与首帧行带权衡；默认渐进（不碰装载预算行）。

## 11. 合并收据（PLAN-731:r1，2026-10-03）

- `stage: merge` | PLAN-731 | plan_revision 1 | `outcome: pass`（分阶段见下）。
- **prepared**：reviewed 基线=R2 pass@8e95ac426（reviewed_commit）；canonical
  Spec 差=SD-01（frame-pipeline-incremental.md §4b/§5b）+SD-02
  （frame-observability.md §5）——均在 dev 分支 fb071e2d1/8e95ac426 内；
  deposit 提交=51ef8c562（docs/projection-only 后裔→delivery 候选）；
  依赖=auto-down detached@895f8d0f（只读消费）。
- **landed**：master 基面 7d50989f7 后零代码推进（仅 6 簿记提交——含并行
  会话 PLAN-733 簿记）；worktree rebase master——range-diff 1-5 提交全等
  （`=`），第 6 提交 `!`=specs.json 冲突调和（**事故与修复在案**：冲突
  误取 --theirs 侧旧账本致 33 行 729/733 款项暂失——即时发现并按
  upsert 语义重放[checkout master 版+P731-1 追加]amend 复原，对 master
  diff 复核=仅 +P731-1 一项）；hash 映射：7454e4c13→7adb938be/
  0664025e7→b9627d3cf/fd5041658→9c33cb3ee/fb071e2d1→5f349d603/
  8e95ac426→fd64ecab0/51ef8c562→30bda6831；主检出 `git merge --ff-only
  plan-731-dev` 成功——master tip=30bda6831=delivery commit，无合并
  提交；FF-OK 断言过。
- **ledger_refreshed**：workspace=D:/autostack/auto-lang（.autoos/specs.json
  为**跟踪文件**——729 起入库，随合并落地）；款项=P731-1（designs 段，
  canonical 唯源=SD-01/SD-02 两文件+evidence/731/）；回读核对：designs
  145 项含 P731-1，master diff 仅 +P731-1（729/733 款项无损）；plans 索引
  =docs/specs/auto-lang/ui/plans.md 731 行（✅ reviewed→archived，
  archive/ 路径）；spec-index.py 再生=INDEX.md 无 diff（无新文件/状态
  变更——预期）。
- **archived**：docs/plans/archive/731-s5-frame-tail-incremental.md
  （git mv+status: archived；completion_kind: delivered）。
- **cleaned**：wt-guard 双仓 clean（auto-lang+auto-down——reparse point 扫描
  双绿）；auto-down worktree 经属主仓移除（auto-lang 侧 remove 报 not a
  working tree 属预期——跨仓属主）、auto-lang worktree 移除、分支
  plan-731-dev 删除（指向 30bda6831=已合并）、组目录 .wt/lang-731 rmdir
  完成；主检出冒烟 cargo check -p auto-lang Finished 无错（落地后
  master known-good）。
- **部署观察**：主检出 target/release/auto.exe 与 gen/front 产物**陈旧**
  （predates 30bda6831——fold 缓存未入主检出 release 面）；消费面=下游
  auto-edit bench（其组树自建二进制，不受主检出陈旧影响）+本机桌面实机
  会话。陈旧项登记：release 二进制（构建日期 2026-10-02 前）、web bundle
  gen/front/vue/dist（同）。批量回归到期判定：回执
  docs/plans/.last-batch-regression.json=last_covered_plan_id 726@
  2026-10-02T05:25Z；731%5=1≠0，且 727-731 中无可被 5 整除者已落地
  （730 worktree 在途未落），回执龄 <48h——**未到期**；下次到期判定点=
  PLAN-735 落地或 48h+任意合并。
