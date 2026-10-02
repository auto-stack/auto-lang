---
plan_id: PLAN-725
status: reviewed                 # drafting → executing → execution_done → reviewed → archived
feature_name: 键入帧增量更新管线优化件（auto-edit M4 帧两行 FAIL 清偿本体——脏域/增量更新，学 Zed 三课「增量一切」）
author: [agent]
created_at: 2026-10-02T00:00:00+08:00
updated_at: 2026-10-02T00:00:00+08:00
plan_revision: 1
current_step: 8
total_steps: 8

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md（SD-01：帧管线增量更新契约——单帧单建/载荷增量/脏域重建/阶梯谱基准口径）
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN（auto-edit 侧目标不入本仓 goals 表——§4 授权注记）
affects: [crates/auto-lang/src/ui/iced/renderer.rs, crates/auto-lang/src/ui/aura_view_builder.rs, crates/auto-lang/src/ui/memo_deps.rs, crates/auto-lang/src/ui/code_editor/, crates/auto-lang/src/ui/dynamic.rs, crates/auto-lang/src/ui/component.rs, crates/auto-lang/src/trans/rust.rs（供⑫⑬ 并入臂，Q-1）, crates/auto-lang/src/tests/（plan725 探针族）, docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md（新）, docs/specs/auto-lang/ui/design/frame-observability.md, docs/specs/auto-lang/ui/overview.md, docs/design/autoui/editor-kernel.md（on_change 载荷契约节——design 档随行更新）]
---

# [PLAN-725] 键入帧增量更新管线优化件

## 0. 变更摘要

auto-edit M4「速度王座与发布」tag 的**当前唯一关键路径阻塞**——帧两行
armed FAIL（type_latency P50 110/P95 115ms vs 预算 ≤16.7ms ≈1 帧、
scroll_fps 8.01fps vs 阈值 54=面板 60Hz×0.9，谱
`frame-20261001-230957.jsonl`）——的**清偿本体**：编辑路径帧管线从
**每帧全量重建**改为**脏域/增量更新**。根因归因在案（PLAN-022 复工
实录：键入到上屏每帧全量重建管线 ~110ms/帧；debug/release 同量级
〔78/159ms、11.4fps〕=编译形态无关强证——瓶颈在管线结构非优化等级）；
测量通道已就绪（716 组B `ui::frame_bench`：AUTO_FRAME_BENCH 门控+
9918/9919 双轨——本件优化效果的测量面不用另建）。

起草期实勘（本仓 master@986e765ac）把 ~110ms 落到**五段成本链**上：
键入帧 = ①on_change 消息**全文 clone**（CodeEditor 闭包
`code_editor_text` O(doc)）→ ②VM 段（update_inner→解释器跑 handler）→
③builder 段**×2**（view() 脏重建全模板 walk + MCP 同步块**第二次**
全量 `view_with_debug_gated`）→ ④Element 段（全树 Element 新建+
`code_editor_set_text` 整串回灌+缓存 put-then-take 形态疑点）→
⑤iced 全窗 layout+draw。工作负载实锚：下游帧档 fixture 仅 **~120 行/
5KB**——110ms 是纯结构成本（非文本量），键入帧的语义变更面（tab 脏
点/编辑计数/状态栏）≪ 全树。本件=分段计量（T-00 阶梯谱基线）→
单帧单建（MCP 双建去重）→ 键入载荷增量形 → 视图重建脏域化（memo
域扩面——PLAN-045/046/047 机制的正宗消费场景）→ 上游阶梯谱基准
（键入→present 增量重建墙钟，改前/改后对照在档）；顺带清偿供料档
§8 余留小缺口（供⑫ print 拼接形+供⑬ frame i32 收窄——Q-1 并入
裁定）。下游重判（type_latency/scroll_fps 转绿）=本件 delivered 后
auto-edit 补跑件。

### 执行期落点注记（2026-10-02 work）

- 供⑬ 修复落点=**`crates/auto-lang/src/a2r_std.rs`**（frame fn i32 收窄）
  非 affects 表列的 trans/rust.rs——发射形无需改动，值域收窄在 std 层单点。
- 探针通道新增 **`crates/auto-lang/src/ui/frame_segments.rs`** + frame_bench
  门控开放（`ui/mod.rs` 注册）——T-00/T-05 常驻门控面（AUTO_FRAME_BENCH 族）。
- T-03 memo 扩面=**零代码交付**（勘定记录 §2③：定量门已由元素级根因修复
  达成；memo 消费面归下游 app opt-in——SD-01 §4 注记）；aura_view_builder/
  memo_deps/component 未触。
- 供⑫ 勘定修正：现势 master 发射形已合法（`println!("{}", format!(…))`），
  缺口不复发——处置=金样锁定（`test/a2r/04_strings/010_print_concat`）。

## 1. 目标

- **G-1 分段归因在档**：五段成本链（S1 消息载荷/S2 VM 段/S3 builder
  ×2/S4 Element/S5 layout+draw）在代表性编辑工作负载上的**键入→present
  墙钟分段谱**（改前基线）+ 优化优先级裁定记录——优化靶向有据，防
  「结构性猜测」收口（721「纸面推演禁止」纪律同款）。
- **G-2 单帧单建**：脏帧 AbstractView 只建**一次**——MCP 同步块复用
  主重建产物（现状：MCP 活跃时同一脏帧 `view_with_debug_gated` 走两
  遍+`read_all_state_materialized` 全量物化——下游 bench/MCP 驱动形
  态必开）。
- **G-3 键入载荷增量形**：code_editor on_change 消息不再无条件携带
  全文（O(doc) clone 退役）——lazy 物化（$event 实参消费点才读）或
  delta 形（Plan 673 统一 delta 队列既有契约）；`.Edit(str)` 形回读
  契约保真（.at 消费面零改动）。
- **G-4 视图重建脏域化（增量更新本体）**：键入帧的转换产物按脏域
  复用——编辑器 pane 的 content 绑定值未变时跳过重转换（memo 域
  〔PLAN-045/046/047：组件级/keyed-for 项级/依赖录制 version_fast〕
  扩面到编辑路径，具体形态 T-00 定参）。正确性纪律=PLAN-045「宁缺
  勿错」（memo 错误只允许变慢）+ 读侧一致性核查（P721-R1 陈旧读机理
  交互——增量缓存不得放大世界分裂）。
- **G-5 上游阶梯谱基准**：键入→present 增量重建墙钟**阶梯谱**（文档
  尺寸 5KB/100KB/1MB × VM 轨，716 通道消费），改前/改后双谱在档=
  本件效果的度量面（下游帧档测量面的上游对偶）。
- **G-6 供⑫⑬ 小缺口清偿**（Q-1 并入裁定）：供⑫ trans print 拼接形
  （println! 字面量约束）+ 供⑬ frame fn i32 收窄（E0308——与 SD-B
  §3b 值域口径一致）。
- **G-7 规范+账本+门禁**：SD-01..04 落档+specs.json P725-1+触面门禁
  （裸 `cargo t`+VM/编译器触面 `tv`+ui_gen 触面 `tu`）零新增红。

### 非目标

- **auto-edit 仓任何改动**（下游重判=下游补跑件；供料档回执=跨仓
  文档注记，Q-3 择路）。
- **编辑器内核逐键 O(n) 快照债**（editor-kernel「现状限制」在册：
  逐击键 before/after 全文快照 O(n) 基线、视口物化延后）——本件=
  帧管线（消息→重建→Element→layout/draw）；阶梯谱分段须可分离
  归因（防内核债污染管线判定），其清偿另批。
- **GPU 直渲/渲染后端换代**（战略 §4.2 第三课——iced+wgpu 已是 GPU
  路径，本件不碰渲染后端）。
- **tree-sitter 增量高亮**（716 组A 已交付增量面；高亮=文本域派生，
  与本件 memo=视图域正交——714 勘定册注记）。
- **front 窗口化消费/UX 件**（022 非目标沿用）；**tier/门禁改制**
  （fix-test-tiering 现行口径不动）。

## 2. 架构方案

分层落点（起草期实勘，本仓 master@986e765ac + auto-edit main@69e6cc9）：

| 面 | 现状（锚） | 本期形态 | 依据 |
|---|---|---|---|
| 消息载荷 | CodeEditor on_change 闭包每变更 `code_editor_text(&sk)` 全文 clone 进 IcedMessage.input_value（renderer.rs CodeEditor 臂 on_change 构造，~:27288）——handler 形参非 str 时全文白运 | lazy 物化（携带 storage_key，$event/str 形参消费点才读）或 delta 形 | PLAN-057 全文携带初衷=「.Edit(str) 首实参」；`on_with_input_for` 仅 $event 实参替换点消费 input_value（dynamic.rs:2402 起） |
| VM 段 | update_inner（renderer.rs:17345）→dispatch_app→`on_with_input_for`（dynamic.rs:2402）解释执行 handler；auto-edit SrcChanged 体=标量写+DiffBufLiveMark+console_lines | 分段计量定谳占比；解释器段若为主要成本→定向优化或另件裁定（Q-4） | 帧档 form=l0-vm（解释段=保守上界形态——预算判定面） |
| builder 段 | view() 脏帧全模板 walk 建 AbstractView（renderer.rs:23368-23374 `view_with_debug_gated`）；**MCP 活跃时同步块第二次全量建**（:23126，gate_dirty 门）+`read_all_state_materialized` 全量物化+vtree 转换 | 单帧单建（主重建产物共享给 MCP 同步/快照面）；脏域化=S3' 转换产物按脏域复用 | PLAN-062 Phase2 T11 先例（MCP 同步块曾每帧全量重建→已 view_dirty 门控——本次收口同向推进） |
| Element 段 | 脏帧全树 Element 新建（含 `code_editor_set_text` 整串回灌——外部值差分门 last_external 在，键入帧 t.src 不变=no-op）；Element 缓存 put-then-take（:23695-23696）——非脏帧快道命中率存疑（take 后缓存恒空？） | put-then-take 形态勘定+修复（若快道恒 miss）；Element 新建维持 iced 设计（重建频次由 S3' 脏域化压下来） | code_editor_set_text 外部值差分语义（core/mod.rs:2123 doc）；§6 测试锁 |
| layout+draw | iced 全窗重排/重绘每脏帧 | 计量定谳占比；占比小→注记不做（T-04 裁定） | 战略 §4.2 第三课边界 |
| memo 域 | PLAN-045 组件级（memo: true opt-in）/PLAN-046 keyed-for 项级/PLAN-047 依赖录制 version_fast——机制在库（ui/memo_deps.rs）未被编辑路径消费 | 编辑路径脏域化消费：候选=code_editor 元素转换缓存（键=storage_key+值指纹/读槽版本）/keyed-for 扩面/读槽版本快道——T-00 定参 | memo 正确性论证（PLAN-045 T-01）+P708-R1 教训（memo 失效纪律须验证） |
| 基准面 | 716 通道（frame_bench+9918/9919）下游 bench 消费在案；上游无编辑路径阶梯谱 | 上游基准件：基准 app+阶梯谱脚本（autoui-verifier 脚本基建复用）+双态谱落档 | 供②验收分工「实机帧序 live-fire 归下游」——本件=上游对偶面（infra 效果度量） |
| 供⑫⑬ | trans print 拼接形缺口+frame fn i32 收窄缺口（供料档 §8 在册，下游已各自回避适配） | 710 形态小修（单臂级）：print 臂拼接实参发射 `println!("{}", format!(…))`；trans frame 臂 `as i32` 收窄（或 fn 面 i32 返回——T-06 择一） | 供料档 §8 供⑫/供⑬ 验收形态建议原文 |

**关键设计约束（frozen）**：

① **默认形零行为差异**：门关（无 AUTO_FRAME_BENCH/MCP）路径零新增
开销；`.at` 消费面契约不破（`.Edit(str)` 回读、$event 替换、
code_editor_set_text 外部值差分语义逐一保真）。② **宁缺勿错**：
脏域化复用的失效纪律沿 PLAN-045 论证（读槽值指纹覆盖一切写点），
不可证形态降级原始路径——memo 错误只允许变慢。③ **判定不冒领**：
阶梯谱改前基线先行落档（T-00），改后对照同脚本同负载；下游两行
转绿=下游补跑判定（本件交付=通路+上游谱，AC-06 如实承载）。④
**跨仓零触碰**：本仓单仓实施；auto-edit 侧改动=零（供料档回执=
evidence+登记建议，Q-3）。⑤ **与 P721-R1 的交互**：读侧世界分裂
机理（同一 binding 相邻构建读值不稳定）未定谳前，增量缓存 correctness
须以读槽**重解析值指纹**（慢路径）为锚——不引入「跳过重解析」的
新快道（P708-R1 同教训）。

## 3. 技术栈

Rust（crates/auto-lang：ui/iced renderer+aura_view_builder+memo_deps+
code_editor+trans 臂）+ 既有剖析基建复用（P631 双段 profile
〔ui/style/mod.rs:40 内联模块〕/AUTO_SCHED_DIAG/frame_bench 通道）+
基准脚本（python，autoui-verifier scripts 惯例：MCP 驱动键入+9918/9919
读回）。门禁：`cargo check -p auto-lang` 快检+裸 `cargo t`+触面档
（tv/tu 按 diff 触面）。worktree：`D:/autostack/.wt/lang-725/auto-lang`
（branch plan-725-dev，Plan 529 布局）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-10-02 会话指令「/auto-plan:new 立项『键入帧
增量更新管线优化件』（auto-edit M4 帧两行 FAIL 清偿本体——M4 tag
关键路径）」——授权=**起草本件**（含「供料档 §8 小缺口批并入本件
或独立小件，由起草时定」的裁量委托→Q-1 建议并入）；执行/work 待
用户另行启动。范围=auto-lang 单仓（crates/docs/specs）；auto-edit
零改动（下游重判=下游补跑件）。无预算/自动续跑授权。用户注记：
本件为 M4 tag 当前唯一关键路径阻塞——**建议优先排期**。

**来源与版本**：

- 证据基（auto-edit 侧，main@69e6cc9）：`docs/plans/archived/
  022-m4-716-consume.md`（帧档首判谱+归因：P50 110/P95 115ms+8.01fps
  armed FAIL；debug 对照 78/159ms+11.4fps；review 复现谱 95/106ms+
  9.1fps——判定带内一致）+`docs/upstream/2026-09-m4-perf-unblock-
  supply.md` §8（供⑫⑬ 余留；供⑧⑨⑪ 已清偿——2026-10-01 回执在册）
  +`tools/bench/results/frame-20261001-230957.jsonl`（首判谱：30 样本/
  25 有效对/first_present 4989ms 记账位）。
- 工作负载实锚（bench.py stage_frame）：fixture **120 行/~5KB**，
  `auto run -r vm`+AUTO_BENCH/AUTO_FRAME_BENCH 双门+MCP `autoui_type`
  单字符连发（type_latency）/换行连发 cursor-follow（scroll_fps）；
  SrcChanged handler 体=标量写+探针采样+DiffBufLiveMark——**不回写
  t.src**（编辑器文本只活在内核 buffer——content 绑定值帧间不变）。
- 归因链（PLAN-022 复工 summary+review 在录）：键入帧全量重建管线
  ~110ms/帧主耗；debug/release 同量级=编译形态无关；5KB 负载=结构
  成本非文本量。
- 测量通道（716 组B delivered@ec45f911c+Phase2 r3@9a71a5212）：
  `ui::frame_bench`（本仓 crates/auto-lang/src/ui/frame_bench.rs）——
  AUTO_FRAME_BENCH 门控零开销+9918/9919 .at 双轨+SD-B 契约册
  （docs/specs/auto-lang/ui/design/frame-observability.md）。
- 本仓脉络：P712-D1「桌面轨更新路径家族」（KNOWN-DEBT 在账，PLAN-721
  承接交付 update-path verdicts）——余题 P721-R1（读侧机理：OnTime
  触发的全新构建烤回 paused=false，同一 binding 相邻构建读值不稳定=
  读侧世界分裂/陈旧 memo 回放，机理开放）正对本根因的**读侧半边**；
  本件=写侧重建结构（全量→增量）+读侧一致性纪律锚定。memo/keyed 域：
  PLAN-045/046/047 机制在库（ui/memo_deps.rs：组件级 opt-in+keyed-for
  项级+动态依赖录制 version_fast）；714 勘定册正交性注记（高亮重算=
  文本域派生、memo=视图域——两缓存域无路径冲突）。
- editor 内核契约：docs/design/autoui/editor-kernel.md（rope 事实源+
  统一 delta 队列〔Plan 673〕+`code_editor_set_text` 外部值差分语义）；
  「现状限制」在册债=逐击键 before/after 全文快照 O(n)（100MB 单键
  25.3s/debug）+视口物化延后——**非本件范围**（阶梯谱分段可分离归因）。
- 起草期代码实锚（master@986e765ac）：update_inner 入口=renderer.rs
  :17345（frame_bench::note_frame_begin 插桩位）；view() 脏重建=
  :23300（view_dirty 门）+:23368（view_with_debug_gated）；MCP 同步块
  二次建=:23085-23154（gate_dirty 门+read_all_state_materialized+
  vtree 转换）；Element 缓存 put-then-take=:23695-23696；CodeEditor
  Element 臂 on_change 全文 clone=:27288；code_editor_set_text=
  core/mod.rs:2123；on_with_input_for=dynamic.rs:2402；P631 profile=
  ui/style/mod.rs:40。
- 历史关联：PLAN-062 Phase2 T11（MCP 同步块每帧全量重建→view_dirty
  门控化——本件 G-2 同向收口）/PLAN-673（编辑内核 delta 流——G-3
  消费面）/PLAN-716 组B（通道）/PLAN-721（verdicts）/P708-R1（memo
  失效纪律验证教训）/P702-D3（CPU-bound handler 占帧——VM 段占比
  定谳的先例语境）。

## 5. 详细设计

### T-00 帧分段剖析+勘定决策件（bounded 调查，产物=勘定记录）

1. **分段计量**：代表性负载（对齐下游帧档：~5KB/120 行基准 app+编辑
   器聚焦）上，把键入→present 差拆五段——S1 消息载荷（on_change
   clone）/S2 VM 段（update_inner→handler 解释）/S3 builder（主重建
   +MCP 同步第二次建分开计）/S4 Element（转换+code_editor_set_text）/
   S5 layout+draw。工具=现成剖析面（P631 双段 profile+AUTO_SCHED_DIAG
   +frame_bench）+临时插桩（worktree 内，收口前撤或门控常驻——
   AUTO_FRAME_BENCH 族纪律）。
2. **阶梯谱基线（改前）**：文档尺寸阶梯 5KB/100KB/1MB × 键入驱动——
   **先落档**（防优化后无对照；脚本=本件 T-05 交付物的前身，先跑
   基线态）。
3. **勘定四项**：①Element 缓存 put-then-take 快道命中率（take 后缓存
   恒空→非脏帧快道恒 miss？还是 iced 批内多消息时命中？——实测定谳）；
   ②G-3 载荷形态择路（lazy 物化 vs delta 形——消费面/契约面/改动面
   对比）；③G-4 脏域化形态择路（元素级转换缓存 keyed on 值指纹/
   keyed-for 扩面/读槽版本快道——正交性+失效纪律评估）；④S2/S5 占比
   定谳（VM 段>30%？layout+draw 占比？→T-04/Q-4 裁定输入）。
4. **读侧一致性核查**：增量缓存正确性锚定=读槽重解析值指纹（慢路径）
   ——P721-R1 世界分裂机理下不引入「跳过重解析」快道（frozen⑤）。
   剖析中若现形读侧机理证据（相邻构建同 binding 读值漂移），登记
   evidence+KNOWN-DEBT 关联回写建议（R1 定谳入口移交）。
5. **定量门定参**：AC-04 的改后阈值（5KB 档 P95 目标+大文件档增长
   界）按基线谱+五段占比冻结——起草期方向性目标（5KB 档 P95
   ≤16.7ms@60Hz 上游基准 app VM 轨）若基线显示不可达分量（如 S2
   解释段独占 >15ms），如实上调并记 Q-4。

### T-01 单帧单建（G-2）

MCP 同步块（renderer.rs:23085-23154）复用主重建（:23368）产物：
脏帧 AbstractView+id_map 一次构建，Element 转换与 MCP 快照面
（styled_vtree/探针/state 物化）共享同源——`view_with_debug_gated`
每脏帧恰一次（探针计数断言）。注意：主重建在 view() 后半段、MCP
同步块在前——需要构建产物前置或同步块后移（顺序语义=PLAN-633
「状态采集移到视图构建之后」保持）。`read_all_state_materialized`
全量物化随帧一次化（现状同步块内已是每脏帧一次——去重后维持）。

### T-02 键入载荷增量形（G-3，形态 T-00② 择路）

on_change 闭包不再无条件 `code_editor_text(&sk)`：lazy 形=消息携带
storage_key（读点延迟到 $event/str 形参消费——on_with_input_for 的
input_value 消费点改造）或 delta 形=变更通知走 Plan 673 delta 队列
（`code_editor_delta` 既有契约）。契约保真面：`.Edit(str)` 回读=
按需物化后逐字节等价；$event 替换语义不变；textarea/autodown_editor
同族闭包（同款全文携带先例 PLAN-057）按同一形态评估（改或不改都
成文——scope 注记）。editor-kernel.md 事件节随行更新（design 档）。

### T-03 视图重建脏域化（G-4，形态 T-00③ 择路）

编辑路径转换产物按脏域复用。候选形态（T-00 定参，可能组合）：

- **元素级**：convert_code_editor 的转换产物缓存——键=(storage_key,
  值指纹〔len+cheap hash 或读槽版本〕, lang, 配置位)；值未变→复用
  AbstractView 子树（深拷贝或 COW——View 树克隆成本 vs 重转换成本
  T-00 实测定参）。
- **memo 域扩面**：keyed-for 项级（编辑器 pane 在 tabs for 循环内——
  PLAN-046 for_item 站点）+组件级 opt-in 面（下游 app 侧 `memo: true`
  属下游件，本件交付机制与基准 app 验证）。
- **读槽版本快道**：PLAN-047 依赖录制 version_fast 消费——**仅当**
  读侧一致性核查（T-00④）通过后启用（frozen⑤）。

失效纪律=PLAN-045 论证沿用（读槽值指纹覆盖一切写点；不可证形态降级）；
观测面=SiteCounts 计数器族（memo_deps.rs 既有）+新增编辑路径站点。

### T-04 渲染段处置（按 T-00①/④ 定谳择做或注记）

- Element 缓存 put-then-take：若快道恒 miss→修复回填形态（draw 后
  回填或 iced 运行时钩子——形态勘定后定）；若仅批内命中→按基线谱
  判定值不值（注记不做亦可，如实记录）。
- layout+draw 占比：若 S5 <10%（预期）→注记不做；若显著→登记
  （GPU 直渲/后端面=战略第三课，另档）。
- 本任务允许「勘定后零代码交付」（判定记录+证据在档）——不硬做。

### T-05 上游基准件收口（G-5）

基准 app（形态 T-00 定：现役 examples/ui 编辑器例复用 or 新 minimal
fixture app——对齐下游帧档负载语义）+阶梯谱脚本（MCP autoui_type 驱动
+9918/9919 读回+P50/P95/分布，autoui-verifier scripts 惯例）——改后
终态谱落 docs/plans/evidence/725/（+基线谱对照）。脚本可重复（复审
复跑面）、产物 JSONL 入仓（下游谱同形惯例）。

### T-06 供⑫⑬ 小缺口批（G-6，Q-1 并入裁定）

- 供⑫：trans/rust.rs print 臂拼接实参发射 `println!("{}", format!(…))`
  （或拆 print! 链——T-06 择一）；测试=拼接形 trans 单测（红→绿）。
- 供⑬：trans frame 臂发射 `as i32` 收窄或 Rust 面 frame fn 改 i32
  返回（与 SD-B §3b 值域 saturating 口径一致——T-06 择一）；测试=
  发射形断言（i32 承接编译面）。
- 触面门=`cargo tt`（trans 臂）+`cargo tu`（若 ui_gen 面触）；下游
  复验指引（探针标记行重埋 regen exit 0）随供料档回执建议登记。

### T-07 回归门+规范+账本（G-7）

- 门禁：裸 `cargo t`（对照 master 基线零新增红——预存红册对照）+
  `cargo tv`（VM/编译器触面）+`cargo tu`（ui_gen 触面时）。快照面
  （ui_snapshots）基线对照——视图重建路径改动**不得**改渲染输出
  （快照绿=行为不变强证）。
- SD-01..04 落档+`python scripts/spec-index.py`+specs.json P725-1
  投影（外科插入先例）。KNOWN-DEBT 关联回写：P721-R1（若 T-00④ 有
  读侧证据）/P712-D1（家族注记）。
- 范围断言：auto-edit 仓零触碰；本仓 diff 限 affects 面。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/frame-pipeline-incremental.md | before：无帧管线增量契约（全量重建为隐性现状） / after：帧管线增量更新契约——五段成本链模型+单帧单建语义+键入载荷增量契约（on_change lazy/delta 形）+脏域复用失效纪律（沿 PLAN-045 论证）+阶梯谱基准口径（负载/档位/读回/判定式） | 增量管线行为契约收口 | AC-01/02/03/04/05 |
| SD-02 | modify | docs/specs/auto-lang/ui/design/frame-observability.md | before：通道契约+下游 live-fire 分工 / after：补上游阶梯谱基准节（基准 app 形态+档位+双态谱纪律——通道的上游消费面） | 通道消费面扩展 | AC-04 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | before：现状节止于 rope/delta 换代（673） / after：帧管线增量更新注记（单帧单建/载荷增量/脏域化+阶梯谱——M4 帧两行清偿路径） | 模块总览进度 | AC-06 |
| SD-04 | modify（跨仓注记，Q-3 择路） | auto-edit docs/upstream/2026-09-m4-perf-unblock-supply.md | before：§8 供⑫⑬ 在册未清偿+管线件无回执位 / after：供⑫⑬ 清偿回执+帧管线件回执节（阶梯谱对照+下游重判指引） | 供料回执闭环 | AC-05/06 |

（editor-kernel.md on_change 载荷契约节=design 档随行更新，不入
SD 表——L2 设计文档惯例。）

## 6. 测试设计

- **单帧单建**：门控探针（AUTO_FRAME_BENCH 族 env）断言脏帧
  `view_with_debug_gated` 调用数=1（MCP 活跃态）；MCP 快照内容
  等价性（styled_vtree/state 物化——去重前后逐字段对拍）。
- **载荷增量**：大 fixture（1MB）键入消息 clone 计数断言（O(doc)
  退役——lazy 物化点计数）；`.Edit(str)` 回读对拍（lazy/delta 形
  vs 旧全文形逐字节等价）；$event 替换回归（terminal/textinput 族
  既有测试绿）。
- **脏域化**：PLAN-045/046/047 既有 memo 测试族回归+新增编辑路径
  缓存正确性（值变→失效重转/值同→复用/不可证形态→降级）+
  SiteCounts 计数断言；**渲染输出不变**强证=UI 快照基线对照
  （ui_snapshots 全绿）。
- **阶梯谱**：脚本双态可重复（改前 commit 基线谱+改后终态谱——同
  负载同脚本）；谱 JSONL 入仓；分段归因表（S1..S5 占比改前/改后）。
- **供⑫⑬**：trans 单测红→绿（拼接形发射/i32 收窄形）+`cargo tt`
  /`tu` 触面档。
- **回归门**：裸 `cargo t` 对照基线零新增红+`cargo tv`+（触面时）
  `cargo tu`；零未处理警告+无遗留 debug 打印（复审健康检查）。

## 7. 验收标准

- **AC-01 分段归因在档**：五段成本链键入→present 分段谱（改前基线）
  +T-00 勘定记录（四项勘定+定量门定参）落 docs/plans/evidence/725/。
  验证：勘定记录+基线谱 JSONL。
- **AC-02 单帧单建**：MCP 活跃脏帧 build 计数=1（探针断言绿）+
  MCP 快照内容等价（对拍绿）+快照/UI 回归绿。验证：探针测试+回归门。
- **AC-03 载荷增量**：键入消息 O(doc) clone 退役（1MB 档 clone 计数
  断言）+`.Edit(str)`/$event 契约保真（对拍绿）+editor-kernel.md
  契约节更新。验证：单测+对拍+文档在档。
- **AC-04 脏域化+阶梯谱**：改后阶梯谱（5KB/100KB/1MB×VM 轨）对照
  基线在档；**5KB 档键入→present P95 ≤ T-00 冻结阈**（方向性
  ≤16.7ms@60Hz——基线谱显示不可达分量时如实上调并记 Q-4 裁定）；
  大文件档键入成本谱不随文档尺寸线性放大（分段可分离内核 O(n) 债
  归因——管线段增量形成立）；渲染输出零变化（快照绿）。
  验证：双态谱+分段表+快照门。
- **AC-05 供⑫⑬ 清偿**：两臂 trans 单测红→绿+触面档（tt/tu）绿+
  清偿回执登记（Q-3 择路）。验证：单测+门禁+回执位。
- **AC-06 下游重判解锁通路**：SD-01..04 落档+下游重判指引（帧档
  复跑口径+unlock 注记建议——budgets 两行 validity 联动）随供料档
  回执/登记建议在档；**真判定=下游补跑**（不冒领——本件交付=
  通路+上游谱）。验证：SD 落档+回执/登记建议文。
- **AC-07 规范+账本+范围**：specs.json P725-1 回读 True+裸 `cargo t`
  对照基线零新增红+（触面时）tv/tu 绿+auto-edit 仓 porcelain clean+
  本仓 diff 限 affects 面。验证：账本断言+双仓 porcelain+门禁收据。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） | 状态 |
|---|---|---|---|---|---|---|---|
| 0 | T-00 分段剖析+勘定 | — | 本件 §5 T-00 节（renderer.rs 五段锚+P631/sched_diag/frame_bench 复用） | 分段谱基线+四项勘定+定量门定参 | AC-01/04 | 勘定记录+基线谱 JSONL 在档 | [x] |
| 1 | T-01 单帧单建 | T-00 | renderer.rs:23085-23154（MCP 同步块）+:23368（主重建） | 脏帧 build=1+快照等价 | AC-02 | 探针断言+对拍+回归绿 | [x] |
| 2 | T-02 载荷增量 | T-00 | renderer.rs CodeEditor on_change 臂（~:27288）+dynamic.rs:2402 消费点+editor-kernel.md | O(doc) clone 退役+契约保真 | AC-03 | clone 计数断言+对拍绿 | [x] |
| 3 | T-03 脏域化 | T-00/T-01 | aura_view_builder.rs convert_code_editor（:11663）+memo_deps.rs+component.rs | 编辑路径转换产物脏域复用 | AC-04 | memo 族回归+计数断言+快照绿 | [x] |
| 4 | T-04 渲染段处置 | T-00 | renderer.rs:23695（缓存）+S5 计量 | 定谳择做或注记（允许零代码交付） | AC-04 | 勘定记录+（若做）探针 | [x] |
| 5 | T-05 阶梯谱收口 | T-01..T-04 | 基准 app+谱脚本+evidence/725/ | 改后谱+对照+分段表 | AC-04 | 双态谱 JSONL 在档可复跑 | [x] |
| 6 | T-06 供⑫⑬ | — | trans/rust.rs print 臂+frame 臂 | 两臂清偿+回执登记 | AC-05 | trans 单测红→绿+cargo tt/tu | [x] |
| 7 | T-07 门禁+规范+账本 | T-01..T-06 | SD-01..04+specs.json+KNOWN-DEBT | 收口落账 | AC-06/07 | 账本断言+双仓 porcelain+门禁收据 | [x] |

## 9. 复审记录

- 2026-10-02 work handoff：`stage: work`，PLAN-725，plan_revision 1。
  `outcome: pass`（全部 8 任务执行完；证据：勘定记录+双态谱
  evidence/725/T-00-survey.md——①put-then-take 死缓存定谳+死写移除；
  ②载荷静态消费判定形择路（三消费点静态可知）；③根因=iced scrollable
  构造期 size_hint→content_height→fresh_fold_map 逐行物化（1MB 95ms/帧
  实测）→无折叠快道 O(1)（等价单测锁）；④S2 非瓶颈/S5 残差口径注记。
  双态谱：5KB segsum P50 11.68→5.30ms、100KB 24.04→5.88、1MB 126.76→8.07
  （尺寸缩放清零，1MB/5KB=1.52；builds=1 单帧单建探针断言实锤；快照
  .snap.new 与主检出残片逐字节一致=渲染输出零变化强证）。门禁：裸
  cargo t/tt 零新增红（musk×6/renderer×2/漂移×3/docs_gen/gallery/e4/
  projector 主检出同形复现——预存红对照在案；plan502/plan707 隔离绿=
  负载 flake）；tv 162/162 绿；tu 未触发（ui_gen 零触面）；警告数
  385=385 零新增。规范：SD-01 新档+SD-02 §5+SD-03 注记+SD-04
  downstream-handoff.md（Q-3 fallback 形）+editor-kernel §8 契约节；
  specs.json P725-1 投影+spec-index 零 diff。供⑫⑬清偿回执建议文在档
  （downstream-handoff.md——auto-edit 仓零触碰）。`code_commit`:
  plan-725-dev worktree（本提交）。`next: review`。
- 2026-10-02 起草 handoff：`stage: new`，PLAN-725，plan_revision 1。
  `outcome: pass`（起草完备：根因归因+测量通道+工作负载三面下游在
  案〔PLAN-022 归档件+frame JSONL 实读〕；五段成本链起草期代码实锚
  〔master@986e765ac 行级坐标〕；memo 域机制在库且正交性勘定在册
  〔714〕；供⑧⑨⑪ 已清偿复核+供⑫⑬ 余留定界；定量门防冒领设计
  〔基线先行+T-00 冻结+不可达如实上调〕；下游重判=通路交付非判定
  冒领；跨仓零触碰纪律+frozen 五条）。`next: work`（执行待用户
  启动；建议优先排期——M4 tag 唯一关键路径）。

- 2026-10-02 review：`stage: review`，PLAN-725，plan_revision 1。
  `outcome: pass`（复审与实施同会话——独立性限制已声明；结论自工件重建：
  双态谱 JSONL 直接重算〔builds 分布 {1:56}/数值逐项吻合〕、新鲜 cargo t、
  证据包 SHA256 短哈在录、master 侧红集对照）。`reviewed_commit`:
  e86b91062（worktree plan-725-dev）；`base_commit`: 96c876cea；依赖
  auto-down@895f8d0（组内兄弟，零改动）。AC 结果：AC-01 pass（勘定+
  基线谱 cb99c2ab1ca9/be729058e80a）；AC-02 pass+P725-R1（builds=1 工件
  重建 56/56；自动化断言未立→债务）；AC-03 pass+P725-R2（谓词契约单测
  绿；clone 计数自动化受 MCP 驱动实测约束——s1 探针在库）；AC-04 pass
  （5KB P95 10.59≤16.7；1MB/5KB=1.52；渲染不变=提取层快照字节一致+
  功能证据——P725-R3 前提勘正：ui_snapshots 为 .at→AuraWidget 提取层
  非渲染层）；AC-05 pass（010_print_concat 金样+parity 扩展绿；tt 零
  新增红；回执建议文 d297096d067a）；AC-06 pass（SD-01..04 落档，SD-04
  Q-3 fallback 形）；AC-07 pass（P725-1 回读 TRUE@designs；tv 162/162；
  tu 未触面；auto-edit porcelain clean+本件 diff 零跨仓路径；diff 限
  affects——两处落点偏差已注记）。规范增量核验：SD-01 4c361a79923c/
  SD-02 1d1d657bffbc/SD-03 ab2b2a098809/editor-kernel §8 02680218c990
  ——描述现势行为与持久决策 ✓；touched_goals=[] 有据（auto-edit 侧目标
  不入本仓 goals 表）。findings：P725-R1(P2)/P725-R2(P3)/P725-R3(P3)
  ——前两者已入 KNOWN-DEBT（复审提交）。`next: merge`。

## 10. 待澄清事项

- **Q-1 供⑫⑬ 并入 vs 独立小件（起草建议=并入，无需裁定）**：两臂
  均 710 形态单臂级小修，并入本件 T-06（同 worktree 同门禁）；若
  用户偏好独立 L0 快通道（更快落 master 供下游复验），执行前示知——
  T-06 拆出为 fix-* 轻量路径即可。
- **Q-2 AC-04 定量门终值（T-00 冻结，方向性默认在案）**：5KB 档
  P95 ≤16.7ms@60Hz 为方向性目标；T-00 基线谱若显示不可达分量（如
  VM 解释段独占超标），如实上调+Q-4 裁定——冻结值与理由入勘定记录。
- **Q-3 供料档回执路径（执行期定）**：auto-edit 仓并行会话在途时可
  能不可直书——fallback=本仓 evidence+登记建议文随附（022 T-06 Q-3
  先例同款）。
- **Q-4 VM 段（解释器）优化归属（T-00 定谳后用户裁定）**：若 S2
  占比显著（>30% 帧预算），解释器段定向优化（handler 派发路径/
  Tick 合并语义等）另立件还是并入本件扩围——执行期以勘定记录+
  分段谱提请裁定。
- **Q-5 基准 app 形态（T-00 定参，确认口径）**：默认=现役
  examples/ui 编辑器例复用（最小 shim 对齐帧档负载语义）；若用户
  偏好独立 fixture app（强可控弱代表性），执行前示知。
