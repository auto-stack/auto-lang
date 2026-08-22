# Plan 419: 代码折叠 Phase B — core 渲染管线改造(逐 run 自绘)

> **状态**: 📋 已立项待实施(2026-08-22,源自 414 §3"点击折叠为 Phase B,需按行隐藏渲染,fill_raw 整缓冲绘制做不到——待单独立项")
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

- **P0 调研 spike**(0.5-1 天):cosmic-text 行隐藏能力核实;路线 A 微基准(千行文档 60fps?)→ 定稿路线。
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
