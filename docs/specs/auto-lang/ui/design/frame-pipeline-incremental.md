# 帧管线增量更新（frame pipeline incremental）

> SD-01（PLAN-725 落账真源——auto-edit M4 帧两行 FAIL〔type_latency/
> scroll_fps〕清偿本体：编辑路径帧管线从每帧全量重建改脏域/增量更新）。
> before=全量重建为隐性现状（无契约）；after=本文契约。基=auto-lang
> plan-725-dev。勘定与双态谱：`docs/plans/evidence/725/`。

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

## 5. 阶梯谱基准口径（上游对偶面）

- **负载**：`examples/ui/041-auto-edit`（全 chrome 编辑器例）+ MCP fixture
  通道装载（AUTOUI_TEST_FIXTURES 写 auto_open_path+触发 App.Tick）；档位
  5KB/100KB/1MB（deterministic 行模板）；驱动 `autoui_type` 单字符 30 键
  60ms 节拍（下游 stage_frame 协议同源）。
- **读回**：stderr `[P725-FRAME]` 分段行；判据 **segsum（S1..S4 之和）
  P50/P95**——present 配对（泵序障代理）受 bounds 回路竞争不稳定，
  total 口径弱化注记在册；S5=残差口径（total−segsum，未插桩）。
- **定量门**：5KB 档 segsum P95 ≤16.7ms（@60Hz）；大文件档键入成本谱
  不随文档尺寸线性放大（1MB/5KB P50 比 ≤1.6）。
- **脚本**：`docs/plans/evidence/725/ladder.py`（幂等 fixture 再生成；
  `--label baseline|after` 双态复跑）。双态谱：`ladder-baseline.jsonl` /
  `ladder-after.jsonl`。
- **门控**：`ui::frame_segments` 与 frame_bench 同门（AUTO_FRAME_BENCH
  单一门）；门关零分支零写入。prev/cur 双槽配对（泵序 begin→present）；
  present=-1 孤儿行=无泵呈现帧成本面。`[P725-ARMS]`/`[P725-ARM]`=Element
  臂钻取行（勘定遗留面，同门）。

## 6. 边界注记

- 下游帧档两行（type_latency/scroll_fps）真判定=下游补跑件（本契约
  交付通路+上游谱；下游重判指引随供料档回执建议——见 plan evidence）。
- S2（VM 解释段）定谳非瓶颈（0.07-0.33ms/帧标量写 handler）；解释器
  定向优化不立项。
- 编辑器内核逐键 O(n) 快照债/首帧 shaping 残差（editor-kernel「现状
  限制」在册）与管线段可分离归因。
