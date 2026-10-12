# 文件后援分页 rope（后援形 rope 契约）

> PLAN-728 SD-01（供⑮ 承接，2026-10-02）。auto-edit 下游战略 §2.1
> open_1gb 行「1 GB 可打开（分块/流式）」的内核侧清偿——装载链全量读
> （`code_editor_load_file`→`read_to_string`）与下游 512MB 拒绝位
> （editor_store.at 三代复证）的上游解阻件。落点
> `crates/auto-lang/src/ui/code_editor/core/rope.rs`（`Node::Chunk` 叶
> 变体）+ `rope/file_backing.rs`（PageStore）+ `core/mod.rs`（装载/
> 编辑/保存臂）。

## 摘要契约

大文件打开=**只读句柄+预扫页表**；文档真源仍是那只 rope——基底页
永不驻留全文，编辑进既有 rope 路径（overlay=持久化 rope 复用，
frozen ③）。七面语义：

| 面 | 契约 | 实现锚 |
|---|---|---|
| ①不可变基底+页表+预扫 | 打开=一次顺序读逐页记 `{offset,len,chars,newlines,start/end_chars,hash}`；页边界 UTF-8 字符边界（std 校验器 `error_len()==None` 定位尾部残缺字符，EOF 页不回退直接报 InvalidData——与 `read_to_string` 同语义）；`line_count`/`len_bytes`/`content_hash`/`range_hash` **零驻留应答**（预扫摘要+树合成） | `PageStore::prescan`、`Node::Chunk` |
| ②编辑覆盖层 | 全部 insert/delete/replace 走既有 `replace_bytes`（split/concat）；`Chunk` split=页文本一次 fault-in+两侧子区间描述符；插入文本=普通 Leaf；虚拟长度/行数=树摘要（页表行数+编辑差自动合成） | `split` Chunk 臂 |
| ③LRU+内存上界 | 页缓存预算默认 **6MB**（`PageConfig.cache_budget`，下限=2 页）；淘汰页退化 `{offset,len}` 指针；**常驻上界契约=树节点+页表+页缓存 ≤12MB 结构计量 @1GB**（`structural_resident_estimate`+`structural_resident_bytes` 断言面；RSS 实测=进程峰值轮询另补） | `PageCache`、AC-03 |
| ④异步调入+预取 | 后台线程预取（±16 页半径，通道生命周期=store 生命周期）；查询路径同步 fault-in（单页 64KB NVMe ~100μs，帧预算内）——**不等磁盘=frozen ④ 由预取保温达成**（滚动/编辑窗口命中内存），非阻塞大装载；页面态可观测 `PageState::{Resident,Cold}` | `spawn_prefetcher`、`prefetch_around` |
| ⑤保存合并 | `write_backed`：未改区段基底句柄 pread **绕过 LRU** 磁盘到磁盘照抄+编辑 span 直写；temp（同目录 `<name>.<pid>.p728tmp`）+`rename` 原子改名；**byte-for-byte**（BOM/EOL 解释归下游层，本件字节忠实）；外部修改检测=任意基底 store len/mtime 对预扫基线不符→保存拒绝（**报错不静默**），成功后基线刷新 | `Rope::write_backed` |
| ⑥消费方 overlay 意识 | `Rope`/`RopeSnapshot` 公共 API **零破坏**（frozen ①——签名钉探针在案）：查找（逐页 fault 扫描）/diff（`prune_spans`+`range_hash` 整页 Merkle 快路免驻留）/撤销（快照冻结免费）/跳转+状态栏（O(1) 摘要）不改调用面即正确 | `api_signature_pin` 探针 |
| ⑦装载链+big 态 | `code_editor_load_file` 阈值 **50MB**（下游 big 态 013 域对齐；阈值下 `read_to_string`→`from_str` 逐字节原样 frozen ②）；>阈值走 `open_file_backed`（预扫+页表+头窗口）；装载路径零语法耦合；natives/a2r 臂零改动（装载形内化） | `PAGED_LOAD_THRESHOLD` |

## 双轨形态与窗口视图

- **阈值下（<50MB）**：纯内存 rope 原样（小文件键入带不劣化——
  谱 `typing-small-5m` 对照在档）。
- **阈值上**：后援形 rope + cosmic Buffer **头窗口**（默认 2MB，
  `PAGED_WINDOW_BYTES`，行首对齐截断）。窗口内交互打字经提交漏斗
  映射全文档区间（`push_delta_from_texts` 窗口基址平移+`edit_paged`
  直达臂——结构化编辑**零全量物化**）；行号跳转/查找落点超出窗口
  时**重物化窗口**（`rewindow_to_line`，目标行上溯 50 行起）。
- **S2 视口物化仍属下游消费件**（673 延后裁定在案）：全文档滚动
  窗口跟踪、窗口外打字体验、fold 面折叠发现（>50MB 档折叠面返回
  空图）为 auto-edit 适配件范围，见回执节。

## 维护不变式

1. **基底不可变**（frozen ③）：`Node::Chunk` 节点构造后永不修改；
   split 产新描述符；基底文件句柄常开（偏移恒真=照抄/摘要前提）。
2. **页边界=字符边界**：预扫切页回退至多 3 字节；任何页切片独立
   合法 UTF-8。
3. **摘要内容决定**：整页摘要=预扫平铺摘要；树链合成与内存 rope
   同源（703 契约不变）——双轨等价测试（同操作序列 vs 纯内存 rope）
   为主判据。
4. **比较行走字节基**（703 隐患修正）：对齐行走切割对齐到对侧节点
   边界，在多字节文本上可落任意字节位——leaf 比较用字节切片
   （`LeafView::bytes`），语义仍字节精确（纯 ASCII 语料下 703 从未
   踩中；分块树形态分歧后确定性触发——修正溯及内存 rope 同受益）。
5. **锁序**：doc →（弃）→ editor →（弃）→ paged；paged 持有期间
   不取 doc（`push_delta_from_texts` 平移量先取后弃锁再编辑）。

## 帧域协同注记（S5/725）

fault-in 不接 725 脏域管线（帧域正交）：缓冲区窗口与 rope 分层，
预取载荷到达不触脏帧；窗口重物化（跳转/编辑相交）走既有
`set_buffer_window`（有限视口→cosmic 惰性整形，全量 shape 不发生
——673 26s/1MB 教训的护栏在位）。**find 全文档扫描的页内换行重扫**
（每 `line()` O(页) 换行计数）为已知债务位：800ms@50MB debug 量级
（谱 answer 线），优化方向=页级换行索引/行游标缓存，下游 1GB E2E
消费件裁定优先级。

## 基准口径（谱：docs/reports/p728-bench.jsonl）

阶梯 50MB/512MB/1GB×{load 装载墙钟, answer 打开即答+远跳+滚动窗,
edit 远端×50, save 合并写}+typing 双轨带（5MB 内存 rope vs 1GB 后援
形逐键分布）。`#[ignore]` 显式跑：
`AUTO_LANG_P728_BENCH=50m,512m,1g cargo test --release -p auto-lang
--lib plan728_bench -- --ignored --nocapture`。上界断言（AC-03）：
1GB 档树节点+页表+页缓存 ≤12MB（结构计量）；RSS 实测由
`scripts/measure_test_mem.py` 进程峰值轮询补记（重量注册按 Plan 564
三步走另议——bench 为显式档不入日常池）。

## 512MB 拒绝位退役回执（Q-3，下游裁定）

本件证「1GB 可装载可编辑可保存」后，auto-edit 侧门处置两案：
**(a) 移除**——内核上界契约+下游 budgets open_1gb 断言化重测兜底；
**(b) 抬升**——抬至内存上界反推值（如 4GB，页表/缓存线性外推仍
10MB 级）。随件附带：`code_editor_text` 全量读出在 >50MB 档仍是
O(n) 物化（VM 事件面 Plan 413 消费）——下游适配件应改走
`doc_snapshot`/分段读。回执预告位见供料档 §10。
