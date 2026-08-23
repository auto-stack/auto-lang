# Plan 428: 代码折叠 Phase B — core 渲染管线改造(逐 run 自绘)

> **状态**: ✅ P1-P4 落地并全量验证(2026-08-23 合并;恢复重建+死锁修复见 §7.5;实机人工验收 §7.4-5 待做)
> **改号说明**: 原 419-code-folding-phase-b.md → 428(2026-08-23;419 由 vm-lifecycle-three-tiers 占用)
> **原始**: 源自 414 §3"点击折叠为 Phase B,需按行隐藏渲染,fill_raw 整缓冲绘制做不到——待单独立项")
> **来源**: Plan 414 §3(fold Phase A 只交付视觉 chevron)/ Plan 413(fill_raw 架构与 26s/1MB shaping 教训)
> **关联**: 413(code_editor core)/ 414(auto-edit UX)/ 418(editor 后续批 §8.8-8.9 的测试基座)

---

## 0. 一句话结论

**折叠的本质矛盾是"整 buffer 一次 fill_raw"无法跳行**——Phase B 把正文绘制从单一 fill_raw 改造为可跳过折叠区间的管线,折叠状态/命中测试/两态 chevron 一并落地。

## 1. 现状盘点

- **已有(Phase A,414)**:`core/render.rs` `fold_opener_lines` 启发式(trim 以 `{` 结尾且非末行);gutter 画单态向下 chevron(`iced/gutter.rs`);无任何点击命中。
- **绘制架构(413)**:core render 产出 `list.text`(cosmic-text 排版 buffer),iced widget 一次 `renderer.fill_raw` 画整段正文;gutter 为 CPU 光栅缓存(revision 键控)。
- **性能边界**:`core/mod.rs` set_text 前强设假 viewport(800×1)避免全文档 shaping(1MB 26s 教训)——逐 run 改造必须维持此惰性。

## 2. 方案要点(设计抉择待 P0 定稿)

| 路线 | 思路 | 代价/风险 |
|---|---|---|
| A. 逐 run 绘制 | render.rs 逐 layout_run 产出 (text, rect),iced 端 fill_text 逐 run 画 | 灵活;run 数量多时 draw 调用与 CPU 成本需压测;失去 fill_raw 的批处理 |
| B. 投影 buffer | 折叠时构造"投影文本"(折叠区间替换为省略占位行)shaping 一次 | 状态同步复杂(光标/选区坐标映射 to 投影/from 原文);undo 交互 |
| C. 上游能力 | 考察 cosmic-text ViEditor/上级行隐藏支持 | 可能不存在;版本升级风险 |

- **折叠状态(core)**:folded 区间集合(行号区间)+ toggle API(native `code_editor_fold_toggle(key, line)`);区间来源 P1 先缩进/花括号启发式,后续可挂语法树。
- **渲染**:render.rs 产出可见行映射(folded 区间折叠到首行,画 `…` 折叠标记);选区/查找跨折叠区间的裁剪(clamp 到可见 run,search_matches 已有同型处理)。
- **gutter 交互**:chevron 两态(展开▾/收起▸);命中测试走 gutter 光栅已有 bounds 缓存模式。
- **交互正交性**:折叠不得进 undo 栈(视图态,非文本态);光标落入折叠区间时自动展开(或跳到区间边界,cosmic-edit 语义调研)。

## 3. Phases

- **P0 调研 spike** ✅(0.5-1 天):cosmic-text 行隐藏能力核实;路线 A 微基准(千行文档 60fps?)→ 定稿路线。(结论:路线 C 不存在、A 性能 66-300 倍余量,定稿 A,详见 §6)
- **P1 折叠状态+区间**(core):状态、toggle native、区间计算;单测。
- **P2 渲染管线改造**:可见行映射+折叠标记+正文绘制(按 P0 路线);26s/1MB ignore 测试不回归。
- **P3 gutter 两态+命中**:chevron 状态渲染、点击 toggle、hover 高亮。
- **P4 交互回归**:选区/查找/undo/IME 与折叠的组合;041 实机验收;MCP 矩阵增折叠项(native 通道)。

## 4. 验收

- 041:点 chevron 折/开 auto 语法块,正文/行号/滚动一致;折叠区间内搜索 match 跳转时临时展开或定位边界。
- `code_editor` 单测 +1MB perf ignore 测试通过;layout_tests/gutter 不回归。

## 5. 风险

- 路线 A 性能(大文档)——P0 必须先量化;失败则退路线 B。
- 光标/选区在投影坐标与原文坐标间的映射 bug 面(路线 B)。

## 6. P0 spike 结论(2026-08-23)

> 分支 `428-p0-spike`;基准源码副本见 `docs/plans/spike-428-bench/`(原始运行目录 `tmp/fold-bench/`,gitignored)。

### (a) cosmic-text 能力核实 —— 路线 C 不存在

**锁定版本**(`Cargo.lock`,仓库根):`cosmic-text 0.15.0` —— `auto-lang` 与 `iced_graphics 0.14.0`/`iced_tiny_skia 0.14.0` **同版**(0.14.2 仅被 gpui 拉入,与本编辑器无关)。即 widget 传入 `fill_raw` 的 `Buffer` 与 core 产出的是同一 crate 类型,无需转换。

**逐 run 隐藏能力:0.15.0 没有。** 对 registry 源码(`.cargo/registry/src/.../cosmic-text-0.15.0`)全量 grep `hide|visibility|Visible|collapsed|fold`(src/ 含 edit/vi/syntect)= **0 命中**。`BufferLine` 公开 API 全集:`new/reset_new/text/set_text/into_text/ending/set_ending/attrs_list/set_attrs_list/align/set_align/append/split_off/reset/reset_shaping/reset_layout/shape/shape_opt/layout/layout_opt/set_metadata/metadata` —— 无 `set_hide`/`is_hide`(docs.rs 0.15.0 交叉确认)。

**升级也拿不到**:最新版 cosmic-text 0.19.0 的 `BufferLine` 同样没有 hide/visibility API(docs.rs latest 核实)。路线 C 需要 fork cosmic-text 并 `[patch]` 连 iced 一起重定向(`Raw.buffer` 是 iced 自己依赖树里的 `cosmic_text::Buffer`,类型必须同源),长期维护成本高 → **否决**。

**关键源码事实**(`cosmic-text 0.15.0`):
- `LayoutRunIter`(`src/buffer.rs:99-173`)逐行累计 `line_height`,`LayoutRun.line_i` 暴露**原始行号** —— 这是路线 A 的天然跳过点(按谓词 skip 即可),但跳过不折叠纵向空间(后续 run 的 `line_y/line_top` 仍含隐藏行高度),需外挂 y 偏移前缀和。
- `shape_until_scroll`(`src/buffer.rs:412`)从 `scroll.line` 起对**所有行**累计高度,无排除点;`Edit::shape_as_needed` → `shape_until_scroll(prune)`(prune 会 reset 视口外 shaping)。折叠不会破坏惰性 shaping —— 被折叠行仍在视口内被 shape(浪费有界 ≤ 视口 ~50 行,见基准)。
- 语法着色走 cosmic-text `SyntaxEditor`(attrs 逐 span 色),`fill_raw` → iced_wgpu 经 **cryoglyph** `TextArea`(`default_color` + 逐 glyph 覆盖)渲染 —— 今天的高亮管线依赖整 buffer Raw 路径。
- `AttrsList::spans()/spans_iter()`(`src/attrs.rs:430`)可枚举颜色 span —— 路线 A 拆 span 的现成 API。
- cosmic-edit 不在 lockfile 中,无本地先例。
- **iced 0.14 的 `fill_text`(Paragraph)底层同样是 `cosmic_text::Buffer`**(`iced_graphics-0.14.0/src/text/paragraph.rs:19`)—— 同一 shaping 引擎,`Font::with_name("Consolas")` + `Wrapping::None` 下 monospace 前进宽度一致,选区四边形(cosmic-text 坐标)与逐 run 文本对齐风险小(仍需 P2 实证)。

### (b) 路线 A 微基准(Release,Win11/本机,Consolas 14px/行高 19,视口 800×950≈50 行,200 次取均,两轮一致)

| 指标 | 1000 行 | 10000 行 |
|---|---|---|
| `set_text`(惰性视口,一次性) | 1.5-1.6 ms | 2.5-2.7 ms |
| `shape_until_scroll` @ 顶部(已 shape 后) | 0.015 ms | 0.10-0.15 ms |
| `shape_until_scroll` @ 行 n/2(全新视口窗) | 0.57-0.77 ms | 0.60-0.90 ms |
| `shape_until_scroll` @ 下移一行(滚动帧) | 0.014-0.019 ms | 0.075-0.13 ms |
| 逐 run 提取 (text, rect) ×50 run | 0.002-0.003 ms | 0.002-0.003 ms |
| 每屏 layout run 数(=fill_text 调用数) | 50 | 50 |
| 折叠 60% 后可见调用数 | 35 | 35 |
| **路线 A 每帧估计**(提取 + 1-5µs/run) | **0.05-0.25 ms** | **0.05-0.25 ms** |

60fps 预算 16.6 ms,余量 **66-300 倍**。考虑语法着色 span 拆分(每行 3-8 span → 每屏 150-400 次调用):0.25-2.0 ms,仍有 8-60 倍余量。滚动帧(增量 shape + 提取)< 0.2 ms。run 数天然以视口封顶(50 = 视口高/行高,软换行只增 layout 行不增屏内 run 总量),与文档总行数无关。

### (c) 路线裁决:**A(逐 run 自绘),B 为保留退路,C 否决**

- **C 否决**:上游无能力(0.15.0 与最新 0.19.0 均无),fork+patch iced 成本高。
- **A 选定**:性能以 66-300 倍余量通过;iced fill_text 与 fill_raw 同为 cosmic-text 引擎,字体/精度风险可控;**折叠状态只在 render 处按行号谓词跳过 + y 偏移前缀和,全部几何(选区/查找/caret/滚动)保持原文坐标**,映射面最小;26s/1MB 惰性 shaping 不受影响(shape_as_needed 语义不变)。
- **B 退路**(仅当 P2 实测出现对齐/字体回退漂移,如 CJK 回退差异):构造投影 buffer(折叠区间替换为占位行)走现有单次 `fill_raw` —— 基准同表证明投影视口 shape 最差 ~0.6-0.9 ms,性能同样可行;代价是投影↔原文双向行号映射与编辑同步(原计划 §2 已列)。
- 已识别实现要点:A 需按 `AttrsList::spans()` 拆色(否则丢语法高亮);fill_text 用 `Font::with_name("Consolas")`(与 `mono_family()` 对齐)+ `Wrapping::None` + 精确 bounds;gutter 行号/折叠 chevron 用同一投影 y(前缀和);命中测试 `mouse_y + shift → buffer.hit()`。

### (d) 对 P1-P4 的影响(定稿后相位不变,P2 内容收敛)

- **P1(不变)**:折叠区间集合 + toggle native + **fold-map(y 偏移前缀和 + 原/投行号互查)**;单测。
- **P2(按 A 收敛)**:render.rs 逐 run(按 span 拆色)产出 (text, rect, color),iced 端以 fill_text 逐项绘制正文,**移除正文单次 fill_raw 路径**(preedit/gutter/装饰 quad 不变);新增对齐校验(选区 quad 与 Paragraph 实测宽度一致性单测);26s/1MB ignore 测试不回归。
- **P3(不变)**:gutter 两态 chevron + 命中(命中走 fold-map 的 y 偏移)。
- **P4(不变+补)**:组合回归外,明确两条策略——光标进入折叠区间自动展开;折叠区内搜索命中跳转时自动展开(或定边界)。

---

## 7. P1-P4 实施记录(2026-08-23,分支 `428-fold-b`,自 master 8b5426fa)

> ⚠️ **状态:代码全部落地、核心单测全绿,但收尾验证被环境故障中断**
> (会话后期 Bash 工具全体 `spawn bash.exe ENOENT`,无法跑测试/git——
> 本文档与全部代码改动已在磁盘上,见 §7.4 待办清单)。

### 7.1 落地内容(按相位)

**P1 折叠状态+区间+fold-map**(全部单测绿):
- 新 `core/fold.rs`:`FoldRegion`(花括号配对启发式,`} else {` 净深度≤0
  跳过、未闭合不折叠、支持嵌套发现)+ `FoldMap`(merged 隐藏区间、
  `hidden_above`/`project_y`/`unfold_y` 反投影命中、`hidden_range_
  containing` 供自动展开)。10 项单测(含 unfold_y 往返、嵌套合并)。
- `CodeEditorCore` 增 `folds: Mutex<BTreeSet<usize>>`(折叠 opener 集合,
  视图态不进 undo)+ `fold_map: Mutex<Arc<FoldMap>>`(渲染快照)。
  API:`fold_toggle`/`fold_is_folded`/`unfold_line`/`fold_hidden_count`/
  `auto_unfold_at_cursor`。**状态操作即时计算区间**(`fresh_fold_map`)——
  native 通道可在无渲染时 toggle(headless 测试/MCP);渲染 map 只管几何。
- native:`code_editor_fold_toggle(key, line_1based) -> Bool`(2932)、
  `code_editor_fold_hidden_count(key) -> Int`(2933)——shim + catalog +
  bigvm 返回类型表 + codegen intrinsics 双表 + 无特性回退,全套。

**P2 渲染管线改造(route A)**:
- `render.rs`:可见 run 走查按 `is_hidden` 跳行、`project_y` 前缀和投影;
  正文按行 `AttrsList::spans_iter()` 拆色产出 `TextRun{text,x,y,size,
  line_height,color}`(同色相邻合并、空白片跳过;P0 基准 150-400 片/屏
  预算内);折叠 opener 行尾追加 `⋯` 标记(软换行多 run 时挂最后 run)。
  选区/搜索/当前行高亮同步投影+隐藏跳过;caret 落隐藏行不绘制;
  滚动条 thumb 比例改按有效行数(total - hidden)。
- `draw.rs`:`TextSection`(weak buffer)删除,`EditorDrawList.text_runs`
  上位 + `fold_hidden`(gutter 光栅缓存键混入,折叠切换无文本 revision
  也失效缓存)。`editor_buffer_weak` 及其 Arc::make_mut 弱句柄协议整体
  退役——不再需要。
- `iced/widget.rs`:正文逐片 `fill_text`(mono_font 与 core `mono_family`
  同族、`Wrapping::None`、`Shaping::Advanced`、`LineHeight::Absolute`),
  选区四边形与文本同源同栈(P0 §6(a) 的对齐前提)。

**P3 gutter 两态+命中**:
- `GutterSection.folds: Vec<GutterFold{y, folded}>`;光栅两态三角
  (展开 ▾ / 折叠 ▸,右向几何=底边满高、尖端收敛)。
- `LayoutInfo` 增 `fold_bands: Vec<(line_i, proj_top, orig_top)>`(命中
  反投影)+ `fold_column: Option<Rect>`;`handle_mouse_press` 折叠列
  命中→`fold_toggle`;正文点击/拖拽选区 y 经 `unfold_y` 反投影后再交
  cosmic hit(点击映射到绘制所见行)。

**P4 交互**:
- 光标进入折叠体自动展开:`handle_input` KeyPressed 臂尾 `auto_unfold_
  at_cursor`(键盘移入隐藏行即揭示);`find_next` 命中折叠区行同样揭示
  (编辑器锁→fold 锁序与 render 一致)。
- 041 接线:`auto-edit.at` 增 `view.fold`(视图菜单"折叠切换");
  `app.at` 增 `fold_hidden` 状态 + `.ActFold`(toggle 激活编辑器第 2 行
  fn 块 + 读回隐藏数)+ 状态栏 `Fold N` 读数;矩阵 `desktop_mcp.py`
  增 **T3b**(T4 ActNew 清空文本之前执行:toggle→hidden==2→toggle→0)。

### 7.2 已验证(shell 故障前)

- `fold` 全族 15/15(fold.rs 10 项 + 集成 3 项:渲染投影/列点击 toggle/
  光标自动展开 + Phase A chevron 适配 1 项)。
- code_editor 套件 33/33(旧契约测试已适配新 `text_runs`/`GutterFold`)。
- 编译:lib(`ui-iced,code-editor`)通过。

### 7.3 设计要点/取舍

- 折叠是视图态:不 bump revision、不进 undo、不触发 text_changed;
  gutter 缓存键 `revision ^ (fold_hidden << 52)`。
- 滚动仍按原文行推进(滚过大型折叠区要按原文行数滚)——已知取舍,
  后续可在 wheel 路径按 fold-map 跳跃隐藏段(挂账)。
- 折叠列命中按"投影带包含 y"判定;命中非 opener 行落到正文点击语义。
- 嵌套折叠:隐藏区间 merge;`unfold_line` 展开所有与命中行相交的
  folded opener(自动展开语义)。

### 7.4 待办(环境恢复后,按序)

1. `cargo test -p auto-lang --features ui-iced,code-editor --lib code_editor`
   —— native e2e 扩展(fold natives 经 `run_with_capture` 全链路)尚未跑过。
2. `cargo test -p auto-lang --features ui-iced,code-editor --lib large_file_
   renders -- --ignored` —— 26s/1MB 惰性不回归(此前一次长跑被环境故障杀)。
3. 全量 feature 套件(基线 3581/0)+ 无特性套件(native_catalog ID 冲突
   校验测试会核对 2932/2933)。
4. 构建 bin + 041 矩阵(含新 T3b;跑前 taskkill auto.exe)。
5. 实机人工验收:点 chevron 折/开、折叠后点击正文行落点正确、
   Ctrl+F 命中折叠区自动展开。
6. 提交(计划号 428 入头)+ 合并 master + 清理 worktree/分支;
   本文档在主检出为未跟踪态,提交时随分支纳入。

### 7.5 环境恢复后的补验与修复记录(2026-08-23,恢复会话)

**背景**:§7.4 中断后,并行清理会话(0311aec5)删除了 worktree 与分支,仅以
`git diff` 备份了**已跟踪文件**的补丁(`scratch/worktree-cleanup-2026-08-23/
428-fold-b-uncommitted.patch`,11 文件 815+);新建的 `core/fold.rs` 是未跟踪
文件,未入任何备份(tar 只有目录骨架)。恢复会话从 master 重建 worktree、
改 041 路径后应用补丁,**按调用点契约重建 fold.rs**(mod.rs/render.rs 的
region_at/build/is_hidden/project_y/unfold_y/hidden_range_containing/
regions_from_texts 引用 + 集成测试语义完整约束了公开面),重建后全部测试
通过——重建与原版语义一致(签名偏差仅两处:line_height 字段 pub、
unfold_y 返回 Option,均由编译器从调用点反推)。

**恢复会话修复的三个问题**(均为原会话最终代码未经全量重跑掩盖的):

1. **find_next 同线程重入死锁**:P4 在 find_next 内插入的 `unfold_line` 在
   持有 editor guard 时经 fresh_fold_map 二次获取 editor 锁(非重入 Mutex)
   → `core_search_highlights_and_finds` 死锁挂起。修复:set_cursor/
   set_selection 后 `drop(editor)` 再 unfold,滚动调整前重新取锁
   (core/mod.rs)。原会话"15/15"只是 fold 过滤子集,全量套件没重跑过。
2. **native e2e 断言期待值错误**:VM 的 `print` 对 Bool 渲染 1/0
   (shim_print 无 true/false 字面量),断言改按 1/0 校验(native.rs)。
   输出 `1
2
0
0` 语义本就正确(折→隐藏2→展开→0)。
3. (重建本身)fold.rs 单测 10 项按原语义等价重写。

**§7.4 清单补验结果**:

| 项 | 结果 |
|---|---|
| code_editor 套件 | **36/36**(2 ignored 为 1MB perf) |
| 1MB 惰性 ignore | **通过,4.30s** |
| 全量 feature 套件 | lib **3594/0** + desktop_behavior 8/0 |
| 无特性套件 | lib **3125/0**(含 2932/2933 ID 冲突校验) |
| 041 矩阵(含 T3b) | **T3b 4/4 全过**(fold engaged 2 hidden → released 0) |
| 实机人工验收 | 待做(§7.4-5;交互路径已有集成测试覆盖) |

**既有红归因**(与 428 无关,stash 干净 master 实证同样失败):
- `ui_snapshots` 3 项:015-notes 源码在 Plan 370(cf2fbc27)改写后快照未再生
  (2876→2920 字节)+ insta assertion_line 元数据 + worktree 绝对路径差异
  (仓库历史上有专门的 .snap.new 残留清理提交,勿在 worktree 再生快照)。
- `vue_capabilities::cap_widget_map_model_init` 1 项:master 同挂。
- 矩阵 `paste restores text`:本机剪贴板被间歇独占(环境性)——同刻干净
  master 的 bin+041 同挂同一条;T6 其余编辑动作全绿,handler 逻辑无关。

