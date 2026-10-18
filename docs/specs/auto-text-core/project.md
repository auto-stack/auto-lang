# auto-text-core

> **Status**: active
> 路径：`crates/auto-text-core`  | 技术栈：Rust（依赖仅 imara-diff + std；dev-deps serde_json）
>
> PLAN-807 交付：文本基础整体抽取——持久化 rope、文件后援分页、快照与
> diff 引擎自 `auto-lang/src/ui/code_editor` 迁入本 crate 单一来源；UI 只
> 留文档注册表与 editor 交互，VM/a2r/Rust 生成路径消费同份实现。本 spec
> 为该冻结增量的 canonical 沉淀。

## 模块与依赖

`crates/auto-text-core` 是文本数据与算法的唯一实现：`src/rope.rs`（AVL
持久化 rope + 内容摘要 + 文件后援 `Node::Chunk`，自 `ui/code_editor/core/
rope.rs` 逐字迁移）、`src/rope/file_backing.rs`（PageStore 预扫页表/LRU
页缓存/预取 worker/合并保存）、`src/diff/{mod,envelope,dirs}.rs`（histogram
引擎 + patience 锚点 + envelope JSON + 目录五态比较）。normal 依赖闭包仅
imara-diff（+std）——无 auto-lang/auto-man/auto-ui/auto-vm/解释器栈反向边
（module-boundaries 域 `lang-core`；闭包由
`test/fixtures/modularity/text-core-consumer/` 独立 manifest + `verify-deps.sh`
钉住）。

auto-lang 侧兼容出口（类型/函数身份不变，同一实现）：`ui::code_editor::
core::rope` 与 `ui::code_editor::diff::{mod,dirs}` 为薄 re-export 门面
（TypeId 同型钉扎于 plan807 回归）；`diff::envelope` 门面保留唯一 UI 状态
片——key→snapshot 查找（`code_editor_with`）。792 交付闭包
`DELIVERY_CRATES` 已纳入（13 成员）。

## 公开面（PLAN-807 T-01 冻结）

- `rope::{Rope, RopeSnapshot, PageStore, PageState, PageConfig}`；`Rope`
  含 `open_file_backed`/`content_hash_of`/`write_backed`/
  `structural_resident_estimate`（跨 crate 消费点升 pub：`PageConfig`/
  `open_file_backed`/`content_hash_of`/`prefetch_byte_range`/`file_len`）。
- `PageStore` 观测面：`page_state/cache_bytes/structural_resident_bytes/
  prefetched_pages/prefetch_around/worker_exited_handle/file_len`。
- `diff::{DiffOpts,Hunk,DiffOut,Refinement,diff_lines,diff_lines_parallel,
  diff_lines_windowed,diff_snapshots,refine_inline}` + `diff::dirs::*` +
  `diff::envelope::*`（路径形四入口 + 纯快照形
  `diff_snapshots_envelope_snapshots(DiffSnapshotsPair::{Both|Missing})`——
  快照或具名 missing 结果，err 形单源）。
- 内部结构（`Node`/`PageDesc`/`PruneSpans`/`prune_spans`/interning/
  grouping/查询引擎）保持 crate 私有。

## 关键合同

- **分页不变式**（PLAN-728 全量保留，详见
  [auto-lang/ui/design/paged-rope.md](../auto-lang/ui/design/paged-rope.md)）：64KB 页/6MB 缓存/±16 预取；UTF-8
  预扫与字符边界；BOM/EOL 字节忠实；快照 COW；`write_backed` streaming +
  基底 len/mtime 变更拒绝；1GB 结构计量 ≤12MB。
- **预取所有权寿命（PLAN-807 闭环清偿）**：worker 持 `Weak<PageStore>`；
  最后外部强引用（rope/snapshot/store 句柄）释放 → store 析构 → sender
  drop → `recv()` Err → worker 受控退出并置位 `worker_exited`；存活
  snapshot 经 chunk 节点强引用继续保活（正确共享寿命，非泄漏）。
  观测面 `worker_exited_handle()`（store 释放后仍可读）+ crate 内永久回归
  + Windows delete-pending 同名重建硬观测。
- **diff 语义**：单管线确定性（serial≡parallel）；窗口调用=全量结果投影
  （非重 diff 切片）；snapshot diff 走 prune 走剪枝（与文本 diff 计数一致，
  hunk 边界合法差异）；envelope JSON 字段序冻结（默认形无 rows_total；
  错误=值非 raise）。
- **VM 消费**（见 [auto-lang/vm/overview.md](../auto-lang/vm/overview.md)
  的 PLAN-807 节）：四 diff shim（9915/9916/9917/9921）纯面直调本 crate +
  `EditorSnapshotProvider` 端口供快照；a2r_std 同构。

## 边界与已知限制

- 本 crate 不含语法高亮/cosmic 布局/编辑器交互（留 auto-lang UI）；50MB
  装载阈值（`PAGED_LOAD_THRESHOLD`）与 2MB 布局窗口
  （`PAGED_WINDOW_BYTES`）归 UI core/mod.rs。
- `find` 全文档扫描的页内换行重扫债务位沿 728 记录不变；显式全文
  `to_string` API 复杂度不另作承诺。
- 不搬 Git 仓（R08/R09 负责）；text-core 仍在语言仓 workspace 内。

> 来源: docs/plans/807-text-snapshot-diff-core.md（SD-01）；代码核对
> crates/auto-text-core/src/**、crates/auto-lang/src/ui/code_editor/**
> （门面/adapter）、vm/{host_capabilities,native}.rs、a2r_std.rs、
> test/fixtures/modularity/text-core-consumer/**（复审 R1..R3 收据：
> docs/plans/attachments/807/review-r{1,2,3}/）。
