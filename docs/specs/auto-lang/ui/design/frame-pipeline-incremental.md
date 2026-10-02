# 帧管线增量更新（frame pipeline incremental）

> SD-01（PLAN-725 落账真源——auto-edit M4 帧两行 FAIL〔type_latency/
> scroll_fps〕清偿本体：编辑路径帧管线从每帧全量重建改脏域/增量更新；
> PLAN-731 补全尾段——S5〔layout/shaping/draw〕从残差口径升分段口径+
> 尾段增量化）。
> before=全量重建为隐性现状（无契约）；after=本文契约。基=auto-lang
> plan-725-dev；731 增量=plan-731-dev（勘定与双态谱：
> `docs/plans/evidence/725/`、`docs/plans/evidence/731/`）。

## 1. 五段成本链模型（分段口径）

键入帧 = ①消息载荷（on_change 全文物化）→ ②VM 段（update_inner→
handler 解释）→ ③builder 段（模板→AbstractView）→ ④Element 段
（AbstractView→iced Element）→ ⑤layout+draw（iced 内部段）。
分段探针 `ui::frame_segments`（`[P725-FRAME]` 行，§5）。

## 2. 单帧单建

- **契约**：脏帧 `view_with_debug_gated` **恰调用一次**（探针 `builds=1`
  断言面）。MCP 同步块（view() 内，PLAN-062 T11 gate_dirty 门控族）复用
  主重建产物（提升点建：view/id_map/probe 存 `frame_build`；同步块消费
  克隆、主重建段消费原件）。
- **顺序语义**（PLAN-633 保持）：状态采集（`read_all_state_materialized`）
  在视图构建**之后**——dark_mode 同步块（view_dirty 写点）上移至单建
  提升点之前，主题翻转帧当帧即脏。
- 非脏同步帧（gate_ws/首同步）无提升产物，同步块就地自建（原语义）。

## 3. 键入载荷增量契约（on_change 静态消费判定）

- 编辑器（code_editor/autodown_editor）on_change 消息的 `input_value`
  按 `ui::dynamic::input_payload_consumed` 静态判定发射：三消费点
  （`$event` 实参/input_state_map 绑定/空 payload 首实参）均不命中 →
  `input_value: None`（O(doc) 整串携带退役）；命中任一 → 逐字面旧发射。
- 保守语义：绑定表缺席=携带（宁缺勿错）。`.at` 消费面零改动（契约节
  见 docs/design/autoui/editor-kernel.md §8）。

## 4. 脏域复用与失效纪律

- **元素级 O(1) 高度上报**：hosted 编辑器 `content_height()` 无折叠快道
  （folds 空 → rope 摘要 line_count×line_height）；有折叠走
  `fresh_fold_map()` O(n) 慢路径。根因链：iced `scrollable::new`→
  `enclose()`→`size_hint()`→`size()` 构造期急切求内容高度——逐行物化
  形态每次 Element 重建执行（1MB 实测 95ms/帧）。
- **Element 缓存勘定**：put-then-take 死写已移除（iced 所有权模型下
  快道结构性不可能命中）；非脏帧路径=cached_converted_view 克隆+全树
  Element 重建（成本由分段探针孤儿帧行观测）。
- **memo 域**：PLAN-045/046/047 机制在库（组件级 opt-in/keyed-for 项级/
  依赖录制 version_fast）；编辑路径脏域化的消费面=下游 app 侧 opt-in
  （`memo: true`/keyed-for 声明），本仓不扩面。失效纪律沿 PLAN-045
  论证（读槽值指纹覆盖一切写点；宁缺勿错）。

## 4b. S5 尾段契约（PLAN-731——尾段增量化）

- **fold 域 revision 缓存**（T-01 主力）：fold 区域发现
  （`regions_from_texts` 全文档括号扫描）**每帧执行退役**——
  `CodeEditorCore::cached_fold_regions` 键=`revision`（任何文本变更
  bump：edit/undo/redo/load），命中=小 Vec 克隆（区域集=可折叠 opener
  行，O(regions)≪O(lines)）；失效域=编辑行窄化载体（未编辑帧零重扫，
  与 §4 脏域纪律协同）。重扫闭包在调用方锁内取行（render=buffer 行，
  fresh_fold_map=rope 行；锁序 cache→editor/doc，无反向路径）。
  **内存上界**：O(regions)；分页装载下缓存域=已装载行集（728 协同
  注记）。
- **整形缓存与视口增量**：行级整形缓存主体=cosmic-text `BufferLine`
  内建（`shape_until_scroll` prune=false 保留已整形行——键=行内容+
  字体/字号态）；视口推进只整形新暴露行（hosted `sync_external_scroll`
  粗定位+窗口臂）；首帧行渐进=虚拟化窗口现状即渐进形态（不碰装载
  预算行）。
- **layout 增量形勘定（不可行面记录）**：iced 0.14 无脏子树/布局 memo
  公开界面，Element 无 Clone、`view()` 契约=消息后全树重建（Tree 状态
  经 cache 复用）。实测 s5_layout release 0.02-0.05ms/帧——非瓶颈；
  替代案=尾段缓存主力（fold 缓存）+受控重建谱（fall-through 帧
  s3b+s4 release ~0.7ms，泵消息驱动）。
- **组件实例缓存勘定**：编辑器态本就 registry 跨帧（TEXTAREA_CONTENTS
  模式——widget 每帧轻构 ~0.17ms debug 含 set_text diff）；「实例缓存」
  在 iced 即时模式语义下无可实施面，重建计数受控谱在档（code_editor
  臂）。046 memo 面正交（零触碰 memo_deps/ui_epoch）。
- **泵治理语义**：`present=-1` 孤儿行=**测量配对面**（帧泵消息消费
  滞后/合并——`__frame_pump` 异步回环），非呈现丢失；相位墙钟与
  begin 行数对照为证。泵率谱（distinct present/秒+孤儿计数）入阶梯
  谱脚本（§5b）；呈现节奏真值归下游 live-fire 档。
- **改后阶梯谱（731 定量门）**：1MB 档 s5 P50 debug ≤1.0ms / release
  ≤0.5ms；滚动尾帧（s5 P95）debug ≤2.0ms；S1-S4 segsum 零回退带
  （±5%）；视觉 golden 三形（键入/滚动/resize）改前/改后 0.00% 全等。

## 5. 阶梯谱基准口径（上游对偶面）

- **负载**：`examples/ui/041-auto-edit`（全 chrome 编辑器例）+ MCP fixture
  通道装载（AUTOUI_TEST_FIXTURES 写 auto_open_path+触发 App.Tick）；档位
  5KB/100KB/1MB（deterministic 行模板）；驱动 `autoui_type` 单字符 30 键
  60ms 节拍（下游 stage_frame 协议同源）。
- **读回**：stderr `[P725-FRAME]` 分段行；判据 **segsum（S1..S4 之和）
  P50/P95**——present 配对（泵序障代理）受 bounds 回路竞争不稳定，
  total 口径弱化注记在册；S5 分段口径见 §5b（731 起三子段直读，
  残差口径退役）。
- **定量门**：5KB 档 segsum P95 ≤16.7ms（@60Hz）；大文件档键入成本谱
  不随文档尺寸线性放大（1MB/5KB P50 比 ≤1.6）。
- **脚本**：`docs/plans/evidence/725/ladder.py`（幂等 fixture 再生成；
  `--label baseline|after` 双态复跑）。双态谱：`ladder-baseline.jsonl` /
  `ladder-after.jsonl`。
- **门控**：`ui::frame_segments` 与 frame_bench 同门（AUTO_FRAME_BENCH
  单一门）；门关零分支零写入。prev/cur 双槽配对（泵序 begin→present）；
  present=-1 孤儿行=测量配对面（§4b 泵治理语义）。`[P725-ARMS]`/
  `[P725-ARM]`=Element 臂钻取行（勘定遗留面，同门）。

## 5b. S5 分段谱与三形口径（PLAN-731）

- **分段行扩展**：`[P725-FRAME]` 行增 `s5_layout_us/s5_shaping_us/
  s5_draw_us`（§4b 契约的观测面；SD-B §5 同步）。s5_layout=根包装探针
  layout() 括号（全树 layout 含非编辑器文本件布局期整形；纯滚动帧无
  view 重建=0）；s5_shaping=编辑器 cosmic-text 窗口整形括号（不含
  sync 臂——ce_draw_block 臂钻取覆盖）；s5_draw=根包装探针 draw() 括号
  （含编辑器 render/光栅与全树绘制入队）。present 侧 wgpu flush 以
  gpu_residual（配对帧 total−segsum−s5sum）单列。
- **相位**：type（60ms 节拍 30 键）/scroll（`__mcp_scroll` 绝对偏移
  25 步×900px×200ms）/typenl（022/024 下游换行协议形——"z\n" 30 键）。
- **臂钻取聚合**：`[P725-ARMS]` ce_draw_block/ce_gutter/ce_fold_scan/
  ce_text_runs 按相位归账（滚动突发跨帧累积，flush 行归其突发末帧相位）。
- **泵率谱**：pump_per_sec（distinct present/相位墙钟秒）+
  orphan_fallthrough_frames（§4b 配对面语义）。
- **脚本与双态谱**：`docs/plans/evidence/731/ladder.py` +
  `ladder-{baseline,after}[-release].jsonl`（同负载同脚本可复跑；
  `--auto-bin` 切 debug/release）。视觉 golden 三形：
  `docs/plans/evidence/731/golden.py`（autoui_screenshot baseline/diff
  通道+快照哈希收敛门——键入逐字符异步排干的截图抢跑修正）。

## 6. 边界注记

- 下游帧档两行（type_latency/scroll_fps）真判定=下游补跑件（本契约
  交付通路+上游谱；下游重判指引随供料档回执建议——见 plan evidence）。
- S2（VM 解释段）定谳非瓶颈（0.07-0.33ms/帧标量写 handler）；解释器
  定向优化不立项。
- 编辑器内核逐键 O(n) 快照债/首帧 shaping 残差（editor-kernel「现状
  限制」在册）与管线段可分离归因。
