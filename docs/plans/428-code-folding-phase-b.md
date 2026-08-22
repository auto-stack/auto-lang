# Plan 428: 代码折叠 Phase B — core 渲染管线改造(逐 run 自绘)

> **状态**: 🚧 P0 调研已完成(2026-08-23,路线定稿 A,见 §6;P1-P4 待实施)
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

