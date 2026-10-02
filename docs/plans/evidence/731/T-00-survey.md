# PLAN-731 T-00 勘定报告：S5 插桩+归因+四定形（2026-10-02）

## 1. 插桩落点（AC-01 面）

| 探针 | 落点 | 口径 |
|---|---|---|
| s5_layout | `iced/frame_probe.rs` FrameProbe 根包装（dynamic_view_impl 尾，PointerPressArea 外层；快道早退点同包） | layout() 括号=全树 layout（含非编辑器文本件布局期整形）；纯滚动帧无 view 重建 → 0（如实） |
| s5_shaping | `code_editor/core/render.rs` shape_as_needed 括号 | cosmic-text 窗口整形+新暴露行（**不含** sync_external_scroll 的 shape_until_scroll——该臂在 ce_draw_block 臂钻取覆盖） |
| s5_draw | FrameProbe draw() 括号 | 全树 draw（编辑器 render/几何/gutter 光栅+fill_text 入队）；present 侧 wgpu flush 不在内 → 谱面以 gpu_residual（配对帧 total−segsum−s5sum）单列 |
| 臂钻取 | `[P725-ARMS]`（arm_acc/arm_count） | ce_draw_block（编辑器 draw 关键段全链）/ce_gutter（光栅）/ce_fold_scan（fold 域发现）/ce_text_runs（fill_text 段数）/code_editor（render_dynamic_view 编辑器臂） |

门控纪律承袭：AUTO_FRAME_BENCH 未设=不取 Instant（`.then(Instant::now)` 形）、零写入零分配（frame_segments gate_off 单测绿）。
`[P725-FRAME]` 行新增 `s5_layout_us/s5_shaping_us/s5_draw_us` 三字段；**残差口径退役**（模块头注记）。

## 2. 基线谱（改前；debug 与 release 双态，041-auto-edit VM 轨）

### debug（ladder-baseline.jsonl）

| 档 | 相位 | s5 P50 | s5 均值分解（layout/shaping/draw） | ce_fold_scan 均值 |
|---|---|---|---|---|
| 5kb | type | 0.50ms | 0.14/0.08/0.30 | 0.045ms |
| 100kb | type | 1.21ms | 0.13/0.10/0.99 | **0.72ms** |
| 1mb | type | 7.06ms | 0.14/0.26/**6.71** | **6.29ms（90%）** |
| 1mb | scroll | 7.01ms（P95 38.66） | 0.18/0.29/8.61 | 6.91ms |
| 1mb | typenl（024 换行协议形） | 6.22ms（P95 6.56） | 0.13/0.21/5.52 | 5.52ms |

### release（ladder-baseline-release.jsonl）

| 档 | 相位 | s5 P50 | 分解 | ce_fold_scan |
|---|---|---|---|---|
| 100kb | type/scroll/typenl | 0.24ms | 0.03/0.03/0.19 | 0.10ms |
| 1mb | type | 1.16ms | 0.03/0.15/0.97 | **0.78ms（67%）** |
| 1mb | scroll | 1.25ms（P95 2.35） | 0.03/0.18/1.20 | 0.84ms |

## 3. 归因定谳

1. **ce_fold_scan=S5 绝对主导**：`render()` 每帧 `fold::regions_from_texts`（O(total lines) 全文档逐行文本收集+括号扫描），随文档尺寸线性；typing/scroll/typenl 全相位每帧执行（与是否编辑无关）。1MB 档 debug 6.3ms/帧（90% S5）、release 0.78ms/帧（67%）。**直对策=按 revision 缓存**（编辑才失效——725 脏域协同）。
2. **s5_shaping 已受控**（release 1MB 0.10-0.18ms/帧）：cosmic-text 窗口整形+725/629 已落虚拟化（hosted sync_external_scroll 粗定位+shape_until_scroll 窗口臂）。滚动尾帧（debug 38ms）=首大步进 sync 臂突发（shape_until_scroll 从 scroll.line 到窗底新行）——量级受步幅约束，release ~2ms。**「行级整形缓存」的主体已由 cosmic-text BufferLine 内建承载**（shape_until_scroll prune=false 保留已整形行）——731 增量=首帧行渐进（现状即渐进——虚拟化窗口）+失效域窄化（revision 键 fold 缓存即其载体）。
3. **s5_layout 非瓶颈**（release 0.02-0.03ms/帧）：iced 0.14 全窗 layout 在本 app 树（041 例）上极廉。
4. **ce_widget_new 非成本**：code_editor 臂（含 CodeEditor::new+set_text diff）debug 0.17ms/rebuild。024 归因「编辑器组件重建→全量重整形」的重建计数关联=**同帧伴随**（重建帧必过 draw→fold_scan+整形），非因果成本——插桩定谳。
5. **尾帧（022/024 P95 95-112ms）上游不复现**：debug+release、type/scroll/typenl（024 换行协议）全相位上游 041 例均无 >40ms 帧（release 最大 P95 2.35ms）。尾帧为**下游 app 特有形态**（auto-edit 编辑器页结构/驱动差异——候选：页面组件树规模、autodown 块编辑器多 CodeEditor 嵌布、装载后首帧行）。**本件插桩（s5 三子段+臂钻取）随 master 下沉后，下游重判件（v0.1-M4.2 位点）直接分段归因收口**——669 模式（上游谱+回执，判定归下游）。
6. **present 配对率低**（paired 2-4/20 帧段）：帧泵（listen_raw RedrawRequested→异步消息）固有竞争——非「掉泵呈现丢失」（024 实录 distinct present/3s 与实际帧率一致），是**测量配对面**：多数帧分段以孤儿行（present=-1）落账。731 谱面以 s5 子段直读帧成本（不依赖配对），泵率以 distinct present/秒单列。

## 4. 四定形（T-00②③④⑤）

- **② iced 0.14 layout 增界面勘定**：`UserInterface::build` 消费 Element（无 Clone）、`view()` 契约=每次消息后全树重建 widget 结构（Tree 状态经 cache 复用）；无脏子树/memo 公开界面。**实测成本地板**：非脏 fall-through 帧（clone converted+全树 Element 重建）debug ~5ms、release ~0.7ms；layout 本体 0.03ms。**裁定（Q-1 替代案路线）**：layout 增量化在本 app 类非瓶颈，全面增量不可行面记录在案；替代案=T-01 fold 缓存主力+受控重建谱（成效以阶梯谱为准）。
- **③ shaping 缓存定形**：键=行内容+字体/字号态（cosmic-text BufferLine 内建——prune=false 保留）；731 落地=**fold 域 revision 缓存**（编辑行窄化失效：revision 变更才重扫）+首帧行渐进（虚拟化窗口现状即渐进形态，不碰装载预算行——Q-3 默认渐进裁定生效）。内存契约：fold 缓存 O(lines) Vec 上界随文档（728 分页协同注记：分页装载下缓存域=已装载行集）。
- **④ 组件缓存正交性+掉泵机理**：CodeEditor widget 为 registry 态+每帧轻构（TEXTAREA_CONTENTS 模式），「实例缓存」在 iced 即时模式语义下=无操作面；重建计数受控谱（0.17ms/rebuild debug）即 AC-04 受控臂。046 memo 面：本件不触 memo_deps/ui_epoch——零扰动（正交性成立）。掉泵=测量配对面+泵消息重建成本（fall-through ~0.7ms release）——治理=成本清偿（T-01）+泵率谱（ladder distinct present/秒）+归因注记，AC-05 受控臂。
- **⑤ 视觉 golden 基线**：三形（type/scroll/resize）改前基线在档（`examples/ui/041-auto-edit/src/front/tests/screenshots/731-*.png`，golden.py 收敛门后自洽 0.00% 三绿）。收敛门=快照哈希连续两次相等（键入逐字符异步排干——settle 2.5s 仍会截到中途态的实录修正）。

## 5. 改造优先序（数据驱动）

1. **T-01 fold 域 revision 缓存**（#1 成本，全相位收益）。
2. T-04 泵率谱通道（ladder 增 distinct present/秒指标）+fall-through 计数归因注记。
3. T-02/T-03 = 勘定记录+受控谱+golden 门（无可实施增量面——数据不支持盲改）。

## 6. 工件清单

- `docs/plans/evidence/731/ladder.py`（谱脚本：S5 字段+type/scroll/typenl 三相位+臂聚合）
- `docs/plans/evidence/731/golden.py`（视觉 golden 三形：baseline/diff+收敛门）
- `docs/plans/evidence/731/ladder-baseline.jsonl` / `ladder-baseline-release.jsonl`（改前双态谱）
- `crates/.../ui/frame_segments.rs`（S5 三子段）、`ui/iced/frame_probe.rs`（根探针）、
  `ui/iced/renderer.rs`（根包装接入）、`ui/code_editor/core/render.rs`（shaping 打点+fold 臂）、
  `ui/code_editor/iced/widget.rs`（ce 臂钻取）
