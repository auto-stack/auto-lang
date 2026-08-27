# 042 — autodown VM render（plan 019 批次七 gallery 页）

VM（iced）轨道的 autodown 文档渲染 + 流式演示。plan 019 Phase 2 收口的
人工验收页：`markdown` / `autodown` widget 在 VM 下从 D-GAP-3 textarea
降级升级为**真渲染**——`content:` 文本经 autodown-core crate 的
`parse_blocks`（a2r 发射的单源 parser，auto-down 仓
`feat/plan-019-phase1-parser`）解析为统一块树，再分解为既有 iced View
变体（plan-450 批次三面板臂同款样式）。

## 运行

```bash
# CLI 以 autodown feature 构建（crate 路径见 Cargo.toml 注释）
cargo build --bin auto --features autodown
auto run -r vm   # 本目录
```

页面两块：

1. **静态文档**（final=true）：标题/段落+行内 marks/代码块（语言标签）/引用/
   有序+无序列表/表格/分隔线——全面板词汇一次成型。
2. **流式演示**（final=.streaming）：每次点击「下一块」追加一段文档；
   final=false 期间悬挂标记与半截链接按流式语义渲染（`stripDanglingTail`
   剥离悬挂 `- `/`> `，loading 链接保留 href 着色），追加完毕 final 翻
   true 收口。content 绑定状态 → 状态更新自然触发重解析与视图重建
   （流式路径 v1；逐块布局缓存为登记的 v1 性能债）。

## 可编辑文档（plan 019 批次九，Phase 3 编辑壳）

第四块演示 `autodown_editor` 标签的可编辑变体：点击聚焦文本叶子块、
键入/退格/Delete、Enter 块内软换行、Ctrl+Z/Y undo-redo、↑↓ 于首末
物理行跨块迁焦。行内 **粗**/*斜*/*码* marks 以样式段叠加渲染。
oninput 回环经 `autodown_editor_text("doc-ed")` native 读回全文绑定
状态（编辑器内部 last_external 差分防回显清场）。

v1 边界：markdown 输入规则未接线；块首退格不跨块合并；选区不跨块。
IME 手验清单待实机执行。

## natives 编程环（plan 019 批次八）

第三块演示 `.at` handler 内的可编程文档操作（`autodown_*` natives，JSON
传输循 read_dir 先例，全部由 autodown-core crate 支撑）：

- Init：`autodown_parse` → `autodown_insert_template`（顶层首位插模板块）
  → `autodown_serialize` 全环，渲染出模板标题；
- 「编程：首块插文本」按钮：`autodown_find_block` + `autodown_insert_text`
  （crate `applyOp` 轨）→ 重新序列化渲染。

natives 面：`autodown_parse(src, final)` / `autodown_serialize(json,
emit_ids)` / `autodown_text(json, id)` / `autodown_find_block(json, id)` /
`autodown_insert_text(json, id, offset, text)` /
`autodown_insert_template(json, md, parent, index)`（catalog 2950-2955）。
无 feature 的二进制：natives 可调用但返回构建错误（本页 Init 会报）。

## 行为锚定

- 渲染适配：`crates/auto-lang/src/ui/autodown_render.rs`（4 单测）+
  `aura_view_builder.rs` 臂级测试（widget 分派 + final 解析）。
- parser 语义：autodown-core crate 金标对拍（auto-down 仓
  `tests/parse_parity.rs`，18 组 fixtures × final/streaming 双模式，
  TS↔Rust 逐字节一致）。
- 无 feature 的二进制运行本页：widget 退化为 textarea（内容仍可见，
  D-GAP-3 既有降级路径）。
