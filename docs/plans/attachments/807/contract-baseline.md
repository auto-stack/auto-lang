# PLAN-807 T-01 合同基线冻结（contract-baseline）

2026-10-18。冻结时刻的源基线：master `2046a025b`（= 本计划 worktree base）。
本文件是 T-01 的权威交付：API/支持面/资源合同/旧 oracle/命令与快照端口选择的
冻结记录。后续任务（T-02..T-07）不得偏离本冻结；需要偏离 = 回 needs_replan。

## 1. 有界 probe（分页最小 API 跨包证明）——PASS

（T-02 复核注：§3/§2 的两处实施勘定已并入下文——`prefetch_byte_range/
file_len` 升 pub、`prune_spans/PruneSpans` T-02..T-03 过渡 pub；观测面由
静态计数改为每店 `worker_exited_handle`。）

（R1 复审勘定并入：§3 纯快照形签名更正为 `DiffSnapshotsPair` 枚举实参；
§6 a2r 行更正——`diff_files_window` 旧/新均为**仅 on 导出**（off=符号
缺席，E0425），非 panic 臂；其余三函数 off panic 文案逐字冻结不变。）

- 方法：worktree base 源的逐字副本放入独立 crate `p807-probe`（临时目录，
  不入受管树），仅做最小可见性调整：
  ① `PageConfig` struct `pub(crate)`→`pub`；
  ② `Rope::open_file_backed` `pub(crate)`→`pub`；
  ③ `Rope::content_hash_of` `pub(crate)`→`pub`（UI core/mod.rs:742 跨包消费点）；
  ④ `rope::file_backing` 模块保持 crate 私有。
- 命令/结果：`RUSTC_WRAPPER= cargo test`（独立 crate，依赖=仅 std）→
  **29 单测 + 1 跨包集成测全绿，0.33s**。集成测（独立测试 crate）消费
  `Rope::open_file_backed(&path, cfg)` + `PageConfig{page_bytes,cache_budget,prefetch_radius}`
  + 打开即答面（len/line_count/content_hash 零物化）+ 跨页编辑 + 快照冻结 +
  `page_state` 观测 + drop 后句柄行为。
- 结论：分页最小 API（`open_file_backed`/`PageConfig`/`PageStore` 公有面/
  `PageState`）可跨包；无需暴露 `Node`/`PageDesc`/`prescan`/`prune_spans`
  （后两者留 crate 私有）。**未动正确性语义**（源逐字副本，测试原样通过）。

## 2. 预取所有权闭环缺陷——已观测确认（T-01 义务）

- 观测：独立测试 crate 持 `(rope, store)` → `drop(rope)`、`drop(snapshot)` 后
  `Arc::strong_count(&store) ≥ 2`（本句柄 1 + worker 强 clone ≥1）。
  `spawn_prefetcher`（file_backing.rs:418）worker 闭包持 `Arc::clone(store)`；
  store 的 `prefetch_tx: Mutex<Option<Sender>>` 持 sender。闭环：
  store 永活（worker 强引用）→ sender 永活 → `rx.recv()` 永不 Err → 线程永不退出。
- 迁入修复设计（T-02 落地，§5.5 有界寿命修复）：
  ① worker 改持 `Weak<PageStore>`，逐批 `upgrade()`，失败即退出；
  ② store 析构（最后强引用释放）→ sender drop → `recv()` Err → 线程退出
    （不 join、不阻塞 drop 路径）；
  ③ 存活 snapshot 经 `Node::Chunk{store: Arc<PageStore>}` 继续持强引用——
    snapshot 存活期间 store/线程存活是正确语义，不算泄漏；
  ④ 观测面（T-02 实施勘定，优于 T-01 草案的进程级静态计数——免并行测试
    互扰）：`PageStore::worker_exited_handle() -> Arc<AtomicBool>` 每店克隆
    句柄（store 释放后仍可读；Release/Acquire 配对），worker 全部退出路径
    （recv Err / upgrade 失败 / spawn 失败臂）置位；crate 内永久回归测试
    `prefetch_worker_exits_after_last_external_release` 断言"snapshot 存活
    期间不退出且可读、最后释放后有界轮询内退出"；auto-lang 层另以
    "最后释放后 Windows 同名重建成功"（delete-pending 语义：句柄全关前
    同名 create 失败）作 OS 级旁证（T-06）。

## 3. 新 crate 冻结：`auto-text-core`

- 位置 `crates/auto-text-core/`；workspace 成员；edition 2021。
- 依赖：`imara-diff`（workspace 版，diff 引擎既有唯一三方依赖）；
  dev-deps：`serde_json`（envelope 窗口测试解析对照）。**禁**新增其他依赖；
  **禁**依赖 auto-lang/auto-man/auto-ui/auto-vm/任何解释器栈
  （闭包由 `test/fixtures/modularity/text-core-consumer` 钉住）。
- 布局：
  ```
  src/lib.rs                 — crate 文档 + 顶层 re-export
  src/rope.rs                — 迁自 ui/code_editor/core/rope.rs（逐字 + §1 可见性）
  src/rope/file_backing.rs   — 迁自 ui/code_editor/core/rope/file_backing.rs
                               （+ §2 闭环修复 + 测试 import 路径修正）
  src/diff/mod.rs            — 迁自 ui/code_editor/diff/mod.rs（import 修正）
  src/diff/envelope.rs       — 迁纯算法部分；key 查找剥离（见 §5）
  src/diff/dirs.rs           — 逐字迁移
  ```
- 公开面（frozen；`PruneSpans`/`prune_spans`/`PageDesc`/`prescan`/内部
  query 引擎函数全部留 crate 私有；**过渡例外**：T-02 提交时 diff 引擎尚在
  auto-lang 侧消费 `prune_spans`/`PruneSpans`，两符号在 T-02..T-03 窗口
  暂为 `pub` 并带 transitional 注记，T-03 迁入 diff 后回 `pub(crate)`——
  终态与冻结一致）：
  - `rope::{Rope, RopeSnapshot, PageStore, PageState, PageConfig}`
  - `Rope`：`new/from_str/snapshot/len_bytes/len_chars/line_count/
    byte_to_point/point_to_byte/line_start_byte/line_end_byte/slice_bytes/
    line/to_string/lines/content_hash/range_hash/subtree_equal/insert_bytes/
    delete_bytes/replace_bytes/open_file_backed/is_char_boundary/
    content_hash_of/write_backed/structural_resident_estimate`
  - `RopeSnapshot`：既有全部 pub 查询面（len/line/point/slice/to_string/
    lines/content_hash/range_hash/subtree_equal）
  - `PageStore`：既有 pub 面（`page_state/cache_bytes/
    structural_resident_bytes/prefetched_pages/prefetch_around`）+
    T-02 实施勘定两处升 pub：`prefetch_byte_range`（UI core/mod.rs:846/1008
    跨包消费点——分页窗口再物化与编辑时预取）、`file_len`（观测面家族，
    与 Debug 输出配对；迁移后无 crate 内调用方）
  - `diff::{DiffOpts,Hunk,DiffOut,Refinement,diff_lines,diff_lines_parallel,
    diff_lines_windowed,diff_snapshots,refine_inline}`
  - `diff::dirs::{DirCounts,DirDiffError,DirDiffIter,DirDiffOptions,
    DirEntryDiff,DirStatus}`
  - `diff::envelope::{diff_files_envelope,diff_files_envelope_window,
    diff_files_envelope_from_paths,diff_files_envelope_from_paths_window,
    diff_dirs_envelope}` + **新增纯快照形**
    `diff_snapshots_envelope_snapshots(pair: DiffSnapshotsPair) -> String`
    （R1 勘定：入参为枚举 `DiffSnapshotsPair::{Both(RopeSnapshot,
    RopeSnapshot) | Missing(String)}`——T-01 草案的双 `&RopeSnapshot` 形
    实施期按计划 §5.3"接收快照或具名missing结果"收敛为枚举；成功/缺键
    路径与旧 key 形逐字节一致，`Missing` 由调用方（UI adapter/VM 端口/
    a2r 端口）经既有判错优先解析）。
- 测试：rope/file_backing/dirs 的 `#[cfg(test)]` 单测族随源逐字迁移（= §6
  旧 oracle 载体）；`diff_bench`（100MB 谱）与 `plan704_*` 回归同迁。

## 4. 旧路径薄重导出（auto-lang 侧，类型身份不变）

| 旧路径（frozen） | 新形态 |
|---|---|
| `crate::ui::code_editor::core::rope::Rope/RopeSnapshot/PageStore/PageState/PageConfig` | `pub use auto_text_core::rope::*`（同类型，身份保持） |
| `crate::ui::code_editor::core::rope::file_backing::*` | 旧 `pub(crate)` 项已全部经 rope.rs 重导出；`file_backing` 模块名不再从 UI 暴露（消费方=零，已核实：仅 rope.rs 自身与 core/mod.rs 经 `rope::` 前缀消费） |
| `crate::ui::code_editor::diff::{DiffOpts,Hunk,DiffOut,Refinement,diff_*,refine_inline}` | `pub use auto_text_core::diff::*` |
| `crate::ui::code_editor::diff::dirs::*` | `pub use auto_text_core::diff::dirs::*` |
| `crate::ui::code_editor::diff::envelope::diff_files_envelope(_window)(_from_paths)(_window)/diff_dirs_envelope` | `pub use auto_text_core::diff::envelope::*` |
| `crate::ui::code_editor::diff::envelope::diff_snapshots_envelope(key_a,key_b)` | **UI adapter 保留同名同签**：`code_editor_with(key,…doc_snapshot())` 查两键 → 纯形；缺席文案/判错优先不变（§5）。ui_gen 发射串不改（T-05）。 |

消费方迁移（T-04/T-05）：
- `vm/native.rs` 四 shim（9915/16/17/21）：on 臂改调 `auto_text_core` 纯面 +
  快照端口（§5）；off 臂 raise 文案/cfg 结构不动。
- `a2r_std.rs::diff`：on 臂改调 `auto_text_core`；`diff_snapshots` 经快照端口；
  off 臂 panic 文案不动。
- `ui_gen/rust.rs` 发射串：**零改动**（旧门面路径续存）。
- `ui/code_editor/core/mod.rs`：`rope::` 前缀不变（重导出承托）。

## 5. 快照端口冻结：`EditorSnapshotProvider`（终名，T-01 定名）

- 位置 `vm/host_capabilities.rs`（779/802 装配模式第 7 槽；**独立 trait**，
  不给 `CodeEditorHostProvider` 加必实现方法——802 外部实现零破坏）。
- 值域：`fn snapshot(&self, key: &str) -> Option<auto_text_core::RopeSnapshot>`
  ——同源 owned Arc 快照（`RopeSnapshot` 是文本 core 类型，非 UI 类型；
  满足 802 值域纪律"无 UI 类型跨界"）。`Send+Sync`。
- 装配：进程默认四件套（`set_process_default_editor_snapshot_provider[_if_absent]/
  process_default_/clear_`）+ `NativeInterface::set_/editor_snapshot_provider` +
  `AutoVM::set_/editor_snapshot_provider` + `new_inner` 构造期快照（与其他七槽
  逐条同构）。
- UI adapter（T-04）：`ui::code_editor` 侧新增安装点，委托
  `code_editor_with(key, |core| core.doc_snapshot())`；单门控 `code-editor`
  （与被委托模块一致）；`if_absent` 安装。a2r/ui_gen 发射不改。
- 缺席语义映射（frozen）：
  - **缺 key**（provider 在位、key 无编辑器）：`err envelope "编辑器不存在: <key>"`，
    判错优先 = 旧序（先 key_a 后 key_b；`a 缺→a，否则 b`）。
  - **缺 provider**：定义为无法找到相应 editor → 同 err envelope，missing=key_a
    （与旧"两键皆缺"路径同形）。
  - feature off（`#[cfg(not(feature="code-editor"))]`）：VM shim 不取参即 raise
    `"<fn>: the \`code-editor\` feature is disabled"`；a2r panic 同文案。
  - 快照传输不经 VM 全文字符串（`add_string(json)` 仅 envelope 结果）。

## 6. 四 native / a2r / 发射冻结矩阵（V-04 oracle）

| 入口 | 参数 pop 顺序（左→右推、右先 pop） | 钳位/域 | 冻结输出 |
|---|---|---|---|
| 9915 `auto.diff_files(pa,pb,ctx)` | ctx→pb→pa | ctx `.max(0) as usize`；envelope 内 ctx==0→3 | envelope JSON（§7 字段序）；错误=值（`err` 字段，无 raise） |
| 9916 `auto.diff_snapshots(ka,kb)` | kb→ka | — | hunk 净形 rows:[]；缺 key/provider→err envelope |
| 9917 `auto.diff_dirs(pa,pb)` | pb→pa | cap 5000→truncated | 五态+counts+truncated+err |
| 9921 `auto.diff_files_window(pa,pb,ctx,off,lim)` | lim→off→ctx→pb→pa | 三 int 各 `.max(0) as usize` | 窗口形（rows_total+truncated 激活） |
| a2r `diff_files/diff_dirs/diff_snapshots`（off 有 panic 臂）+ `diff_files_window`（**仅 on 导出**） | 同语义（i64 ctx/off/lim） | 同上 | 前三者 off 臂 panic 文案逐字；`diff_files_window` feature off = **符号缺席**（旧 base 同形——off 根本无该符号，调用处 E0425 编译红；R1 勘定：T-01 草案误称"四入口均有 off panic"，按保持既有支持面原意记为 on-only，不默默新增 fallback 扩大启用面） |
| ui_gen 发射 | — | `({a2}).max(0) as usize` 等 | 发射串零改动（§4） |

- envelope 字段序（frozen，PLAN-716 golden 钉）：
  - 默认形 `{"hunks":[…],"rows":[…],"adds":N,"dels":N,"truncated":false,"degraded":false,"err":""}`
    （**不含** `rows_total`）；
  - 窗口形在 `dels` 后插 `"rows_total":N`，`truncated` 激活
    （=offset>0 或 offset+返回行数<total）；
  - 行 12 字段序 `lo,ro,ln,rn,lk,rk,lpre,lmid,lpost,rpre,rmid,rpost`；
    `lo/ro` 1 基（缺侧 0）；`lk/rk∈{ctx,del,add,""}`；
  - 错误文案：`文件不存在: <path>` / `读取失败: <io>` / `目录不存在: <root>` /
    `编辑器不存在: <key>`（均经 `json_str` 转义）。

## 7. 分页/资源/保存合同（frozen，V-02/V-03 oracle）

- 常量：默认页 64KB、cache 预算 6MB、预取半径 ±16 页、`MIN_PAGE_BYTES=256`；
  消费面 50MB 装载阈值（`PAGED_LOAD_THRESHOLD`，留 UI core/mod.rs）与
  2MB 布局窗口（`PAGED_WINDOW_BYTES`，同留 UI）。
- 不变式：prescan UTF-8 逐页校验（坏 UTF-8 = `InvalidData`，含文件尾断字；
  非尾页字符跨界回退 ≤3B）；BOM/EOL 字节忠实；页边界不拆字符；快照 COW；
  LRU 预算约束；`write_backed` streaming（未改区段磁盘→磁盘）、
  `<dest>.<pid>.p728tmp` 同卷临时 + 原子 rename、基底 len/mtime 变更拒绝
  （`external modification` 文案）、自身保存后 baseline 重戳；
  1GB 结构计量 ≤12MB（树节点+页表+缓存，`structural_resident_estimate` +
  `structural_resident_bytes` 之和；非 RSS 上界）。
- 资源证据：`plan728_bench.rs` 谱（#[ignore] 显式档）迁至新包内 1GB 定向
  资源测形态（T-06：work 树机器单实例预约运行 1g 档；RSS 由
  `scripts/measure_test_mem.py` 峰值轮询口径补记）；旧谱
  `docs/reports/p728-bench.jsonl` 历史行不重写。
- §2 闭环修复为迁入必做项（非可选项）。

## 8. 文件目录 diff 支持面（frozen，V-05 oracle）

五态（existence→kind conflict→binary heuristic→size diff→byte-equal/
uncompared）；skip-list（`.` 前缀 + target/node_modules/gen/dist/build/
`__pycache__`）；≤2MB 内容域；>2MB 同大小 = `same + note:"uncompared"`；
cap 5000 → `truncated:true`（counts 仍同域）；counts 只累计已迭代域；
lazy 迭代可早停；`DirEntryDiff::hunks` 惰性；缺失根 = `RootMissing` →
`目录不存在` err envelope；walk 元数据竞态跳过（非失败）。
单测族（existence/modified/binary/uncompared/skiplist/lazy/missing/hunks）
随源迁移 = oracle。

## 9. 命令表（frozen）

- work（每任务定向，禁整档 t/tv/tt/tb/tu/taa）：
  - `cargo check -p auto-text-core -p auto-lang`
  - `cargo test -p auto-text-core`
  - `cargo test -p auto-lang --lib plan807`（新上层回归族）
  - `cargo test --manifest-path test/fixtures/modularity/text-core-consumer/Cargo.toml`
  - 相关旧滤串复跑：`--lib plan728`、`--lib plan703`、`--lib plan710`、
    `--lib plan716`（探针族零漂移）
  - 777：`python scripts/modularity-audit.py --mode strict --no-cargo-check`
    （T-06 需先在干净受管 HEAD `--refresh-baseline` 重采——根/成员 manifest
    变更使旧基线过期；forbidden 不放宽）
  - 792：受影响 profile 走既有 driver
    `python test/fixtures/modularity/generated-ui-consumer/driver.py --scratch <临时>`
    （T-06 记录实跑命令与 manifest crates 数变化 12→13）
- merge（冻结提交统一）：裸 `cargo t` + `cargo tv`（VM 触面）+ `cargo tu`
  （Rust 生成接线触面）。
- 独立 consumer（T-06 新建 `test/fixtures/modularity/text-core-consumer/`）：
  独立 [workspace]，仅依赖 `auto-text-core`（path），fresh 构建即 V-06；
  附 `verify-deps.sh`（cargo tree 断言闭包无 auto-lang/auto-man/auto-ui/auto-vm）。

## 10. 777/792 登记冻结

- `.autoos/module-boundaries.toml` `[domains.members]` 增
  `"auto-text-core" = "lang-core"`（文本基础，抽取方向 lang-monolith→lang-core，
  合法边；适用既有 `lang-core-no-monolith-no-man-no-tools` 禁向族）。
- 792 `DELIVERY_CRATES`（rust_ui.rs:3988）增 `"auto-text-core"`
  （12→13；auto-lang 必需 path 依赖——不登记则交付根 fresh 解析红）。
- 共享写面串行纪律（批次合同 §2）：根 Cargo.toml members、tests.rs 注册、
  host_capabilities 追加节、777 基线——只增自己的片段，后入者取并集。

## 11. 归属与不变更声明

- 本计划不搬 Git 仓（R08/R09 负责）；不改 802 editor 22 native；不迁
  editor22 项/fold/highlight/render；不动 `PAGED_LOAD_THRESHOLD`/
  `PAGED_WINDOW_BYTES`（留 UI）；不新增分页视口功能；不重写 diff 算法；
  不重定性能阈值。
- 旧算法删除面（T-03 验收）：`ui/code_editor/core/rope.rs` 与
  `ui/code_editor/core/rope/file_backing.rs`、`ui/code_editor/diff/{mod,envelope,dirs}.rs`
  的**算法体**不再存在于 auto-lang（薄重导出/adapter 除外）；生产路径无第二
  份可执行副本。
