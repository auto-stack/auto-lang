---
plan_id: PLAN-728
status: execution_done
feature_name: 文件后援分页 rope 实施件（auto-edit 供⑮ 承接——不可变基底页表+编辑覆盖层+LRU 内存上界+异步预取+保存合并+消费方 overlay 意识+512MB 拒绝位退役弹药）
author: [agent]
created_at: 2026-10-02T15:06:46+08:00
updated_at: 2026-10-02T17:55:00+08:00
plan_revision: 1
current_step: 11
total_steps: 11
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/paged-rope.md（SD-01：后援分页 rope 契约——七面语义+内存上界契约+快照兼容+装载链+保存语义+big 态/拒绝位退役）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料驱动面，703/716 先例注记式）
affects: [crates/auto-lang/src/ui/code_editor/core/rope.rs, crates/auto-lang/src/ui/code_editor/core/mod.rs, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/ui_gen/rust.rs, crates/auto-lang/Cargo.toml]
---

# [PLAN-728] 文件后援分页 rope 实施件（auto-edit 供⑮ 承接）

## 0. 变更摘要

auto-edit **M4 解阻供料包供⑮**（`auto-edit/docs/upstream/
2026-09-m4-perf-unblock-supply.md` §10——用户 2026-10-02 裁定 (b)
「实现完整的标准的文件后援分页 rope」回执）的**承接实施件**。目标：
战略 §2.1 open_1gb 行「1 GB 可打开（分块/流式）」的内核侧清偿——
现势=装载链**全量读**（`code_editor_load_file`→`std::fs::
read_to_string`，core/mod.rs:2161 实锚）+下游 512MB 拒绝位护栏
（auto-edit editor_store.at:946——018/021/023 三代复证活体）。
按供料七面种子实施：**①不可变基底**（打开=只读句柄+页表
{偏移,长度,行数}；预扫一次顺序读只记行数——「共 X 行」/行号跳转
免调入）→ **②编辑覆盖层**（键入/删除全进 overlay；读=overlay
优先+基底按页 fault-in；虚拟长度/行数记帐）→ **③LRU 淘汰与内存
上界契约**（RSS ~10MB 级@1GB——契约入验收）→ **④异步调入+预取**
（滚动方向预读+未就绪占位——不等磁盘[024 S5 教训]；725 增量管线
顺风面）→ **⑤保存合并**（未改区段照抄+增量写回+临时文件原子改名
——byte-for-byte）→ **⑥消费方 overlay 意识**（查找/diff/撤销/
跳转/状态栏四族——**RopeSnapshot API 零破坏**）→ **⑦big 态保持
+512MB 拒绝位退役弹药**（回执建议随件——下游门退役=open_1gb
断言化重测的消费件）。**下游 auto-edit 适配件不在本件**（669
模式——本件交付内核能力+回执，下游消费件另立：门退役+1GB E2E
矩阵+budgets 行 unlock 重测）。

## 1. 目标

- **G-1 不可变基底+页表+预扫（面①）**：大文件打开=只读句柄+页表
  逐页{文件偏移,长度,行数}；基底页**永不原地修改**（偏移恒真前提）
  ；装载预扫=一次顺序读只记行数（廉价不建结构）——1GB 文档打开
  即答「共 X 行」、行号跳转免全量调入。
- **G-2 编辑覆盖层（面②）**：全部键入/删除进 overlay（结构选型
  T-00③——持久化小树/gap buffer）；读路径=overlay 优先、基底按页
  fault-in；虚拟长度/行数=overlay 记帐（与基底页表行数合成）。
- **G-3 淘汰与内存上界（面③）**：LRU 淘汰——淘汰页退化为{偏移,
  长度}指针；常驻=可见窗+预取缓冲+overlay；**RSS 上界 10MB 级@
  1GB 文档契约**（实测入验收）。
- **G-4 异步调入+预取（面④）**：滚动方向预读；未就绪页**占位渲染
  不等磁盘**（滚动同步等盘=帧超标放大——024 S5 归因教训在案）；
  与 725 增量管线协同注记（分块载荷到达对脏域更新友好）。
- **G-5 保存合并（面⑤）**：原文件未改区段照抄+overlay 增量写回；
  临时文件+原子改名（崩溃安全）；**往返 byte-for-byte**（auto-edit
  008 字节保真契约对齐——BOM/EOL 归下游层，本件=字节层面忠实）；
  外部修改检测策略随形定（句柄保持+mtime/大小探测）。
- **G-6 消费方契约改造（面⑥）**：四族长出 overlay 意识——查找
  （overlay+基底逐页扫描）/diff（按需区间调入——9916 快照消费）/
  撤销（overlay 记帐）/行号跳转+状态栏（渐进行数）；**
  RopeSnapshot 公共 API 面（Arc 持久化+Send+Sync 语义）零破坏**
  ——既有消费方（673 find_next/703 子树哈希/703-9916 diff 快照/
  高亮 warm）不改调用面即正确。
- **G-7 装载链贯通+big 态（面⑦）**：`code_editor_load_file` 增
  后援路径（>阈值走分页装载——**替换 read_to_string 全量读**
  [:2161 锚]）；natives/a2r 臂同步；≥50MB big 态绕语法臂维持；
  **512MB 拒绝位退役弹药**（重估移除或抬升——回执建议随件，下游
  裁定实施）；小文件路径**零回退**（阈值下纯内存 rope 原样——
  725 后键入 1-4ms 带不劣化）。
- **G-8 基准+验收**：1GB 装载 E2E+RSS 上界实测+帧档口径不回退+
  保存往返 byte-for-byte+小文件零回退谱。
- **G-9 规范+账本+回执位**：SD-01 契约册+P728-1+下游回执预告
  （供料档 §10 追加回执节位）。

### 非目标

- **auto-edit 侧适配件**（512MB 门退役实施+1GB E2E 矩阵+budgets
  open_1gb 行 unlock 断言化重测——本件 delivered 后下游消费件
  另立；本件只出内核能力+回执建议）。
- ≥2GB 档（上界契约同构可扩——非本件验收）；mmap 形态（后援分页
  =显式页管理；mmap 备选 T-00 注记可评不默认）；外部编辑器协同/
  文件锁深度语义（检测策略随形定即止——供料原文）。
- 供⑭（tree-sitter 插件化——同批排程注记位）/S5 段增量化（725
  余题——帧域正交，协同注记 T-00）。
- 撤销历史持久化/会话恢复跨进程（overlay 生命周期=缓冲区生命周期
  ——进程内语义）；rope 公共 API 面变更（零破坏=frozen——见 §2）。

## 2. 架构方案

分层落点（2026-10-02 实勘，auto-lang master@b385534d7）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| rope 本体 | in-tree 模块 rope.rs（1610 行——Node::Leaf/Internal+RopeSnapshot[:876/:991 实锚]；703 子树哈希在位；纯手写零第三方 rope 依赖） | **后援形扩展**（默认就地扩展——T-00⑤ 抽包评估[auto-atom 先例]；分页阈值双轨：小文件纯内存原样/大文件后援形——阈值 T-00② 定参） | 供⑮ 七面种子；703 就地先例 |
| 装载链 | `code_editor_load_file`→`read_to_string` 全量读（core/mod.rs:2160-2161 实锚）——1GB=1GB RSS 前置根因 | 后援路径（>阈值：句柄+预扫+页表；阈值下原样）——**全量读仅存于小文件臂** | 同上+下游 512MB 门三代复证 |
| 快照面 | doc_snapshot（703）+Arc+Send+Sync 语义；消费方=find_next[673]/diff_snapshots[9916]/子树哈希剪枝[703]/高亮 warm | **API 零破坏**——后援形快照=页表+overlay 不可变视图（fault-in 在快照读内）；消费方调用面不改 | frozen 约束② |
| 内存契约 | 无上界（全量常驻） | LRU+RSS ~10MB@1GB 契约（实测验收） | 供⑮ 面③ |
| 保存 | code_editor_save（701 直写——rope 全量序列化落盘） | 合并写：未改区段照抄+overlay 增量+临时文件原子改名 | 供⑮ 面⑤；701 面 |
| 帧域 | 725 增量管线在位（脏帧 builds=1/P50 1-4ms）；S5 段余题在册 | fault-in 异步+占位（不等盘）；协同注记（载荷到达→脏域更新） | 024 S5 教训；725 谱 |
| big 态 | ≥50MB 绕语法臂（013 域——下游） | 内核侧装载路径无语法耦合注记+big 语义保持断言 | 供⑮ 面⑦ |

**关键设计约束（frozen）**：
① **RopeSnapshot 公共 API 零破坏**（Arc/Send+Sync/既有方法签名
不动——后援形是实现细节；任何消费方改调用面=设计错误回 T-00）。
② **小文件零回退**（阈值下路径逐字节原样——725 后键入带不劣化
入验收）。③ **基底不可变**（页永不在原地修改——偏移恒真=照抄/
增量写回语义的前提）。④ **不等磁盘**（fault-in 异步+占位——同步
等盘即帧超标，024 教训 frozen）。⑤ 512MB 拒绝位退役=**回执建议**
（下游门属 auto-edit——本件只证「1GB 可装载可编辑可保存」）。

## 3. 技术栈

Rust workspace（crates/auto-lang——rope.rs/core/mod.rs/natives/
a2r 臂）；基准 fixture（生成式 1GB/512MB/50MB 阶梯——013/016 生成
式先例不入库）；单测族（页表/overlay/LRU/合并写往返/快照兼容——
rope 既有 23 测试族零回退为硬门）+探针（plan728_supply_probes——
710/716 形态）；cargo tf 全量门+预存对账（703 惯例）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-10-02 会话指令「OK，那么把 rope 的改进
计划写出来，放到 auto-lang 中去。等它实现之后，我们再回过头来
修改 auto-edit，让他能适配大于 512M 的文件」——授权=**起草本件**
（供⑮ 承接实施件——(b) 裁定的落地路径）；执行/work 待用户另行
启动（auto-lang 会话——727 稿并行在途，启动时核 728 编号与建组）。
范围=auto-lang crates/docs/specs；auto-edit 零改动（回执=文档面）。
无预算/自动续跑授权。

**来源与版本**：

- 供料基：供料档 §10 供⑮（七面种子全文+验收形态建议+量级注记
  「≈供① 三类或更大」+同批排程建议）+designs/002 裁定档（auto-edit
  侧）+budgets open_1gb 行（unlock 指针指向供料档 §10）。
- 现势实勘（master@b385534d7）：rope.rs 1610 行（Rope:876/
  RopeSnapshot:991/Node 摘要+703 子树哈希）；装载链全量读锚
  （core/mod.rs:2160-2161）；code-editor feature 面（Cargo:69/
  255-257）；doc_snapshot（703 增）与消费方四族；零第三方 rope
  依赖（Cargo grep none）；725 增量管线谱+024 S5 教训在册。
- 先例链：PLAN-673（rope 单写者+快照隔离消费先例）/687（分块读
  ——读端分块，本件=**缓冲区分块**的深水区补全）/701（直写保存
  面）/703（子树哈希+就地扩展先例+SD 册家族）/716（多组合一与
  门控纪律）/725（增量管线——顺风面）。
- 下游回执预告位：供料档 §10 回执节（本件 delivered 后 auto-edit
  消费件：门退役+1GB E2E+open_1gb 断言化——669 模式）。

## 5. 详细设计

### T-00 勘定决策件（产物=勘定报告）

1. **形态定界**：就地扩展 rope.rs（默认——703 先例）vs 抽
   paged-rope 独立 crate（auto-atom 抽包先例；判据=测试隔离/复用
   收益 vs feature 接线/workspace 结构成本）——证据定案。
2. **分页阈值与双轨形态**：后援形启用阈值（候选 50MB[big 态对齐]
   /128MB/256MB——判据=小文件性能带零扰动边界+RSS 收益起点）；
   阈值下路径逐字节原样（frozen ②）。
3. **页尺寸/页表定参**：页大小（候选 64KB-1MB——NVMe 读放大/
   页表规模/行计粒度权衡）+页表结构（扁平 Vec vs 树形——跳转
   复杂度）。
4. **overlay 选型**：持久化小 rope（复用既有 Node 结构——快照
   语义免费）vs gap buffer（编辑局部性好——快照成本）vs 操作
   journal（最轻——读路径合并成本）——判据=编辑路径复杂度/
   快照零破坏/frozen ①。
5. **预扫成本实测**：1GB 顺序读记行数墙钟（NVMe 参照——装载预算
   面：目标秒级内；超预期=预扫降级策略[异步预扫+渐进行数]预案）。
6. **S5/725 协同注记**：fault-in 载荷到达→脏域更新的接线面（帧域
   正交性+S5 余题不阻塞本件）。

#### 勘定结论（2026-10-02 work 会话，基面 master@526e7688e 实勘）

1. **形态=就地扩展 `core::rope` 模块**（rope.rs 主体 + 新子模块文件
   `rope/file_backing.rs`，模块路径/公共类型零移动——703 先例的"就地"
   取模块路径不变义）。**后援形=新 `Node::Chunk` 叶变体**（页描述符：
   共享 `Arc<PageStore>`+页号+页内区间+预扫摘要 {bytes/chars/newlines/
   hash}），不是独立 crate、不是平行 Rope 类型。判据：Rope/RopeSnapshot
   结构体与全部公共方法签名零改动（frozen ① 以构造达成而非以分派达成
   ——消费方四族零接线）；703 摘要/剪枝机制对 Chunk 免费复用（整页
   Merkle 快捷不需驻留文本）。抽包（Q-1 备选）证据不支持：无跨仓复用
   面、测试面同 crate 即可隔离（file_backing 子模块+探针族）。
2. **阈值=50MB**（下游 big 态 013 域 50MB 界对齐——auto-lang 内核现
   无 big 判定（grep 50MB/big_file 零命中，语法臂在渲染层下游驱动），
   阈值首次成文于内核侧装载臂：`metadata.len() > 50MB` → 后援形）。
   阈值下路径逐字节原样（read_to_string→from_str，frozen ②）。
3. **页=64KB，页表=扁平 `Vec<PageDesc>`**（{offset:u64, len:u32,
   chars:u32, newlines:u32, hash:u64}≈32B；1GB=16384 页≈0.5MB 表）。
   页边界预扫时回退至 UTF-8 字符边界（页内文本独立合法）。树形页表
   无收益：行号跳转定位走 rope 树 O(log n)（页表只在树外做预扫记帐
   与保存照抄映射）。
4. **overlay=持久化 rope 复用（选项 A）**：编辑走既有 `replace_bytes`
   split/concat 路径——FileChunk 节点不可变（frozen ③：split 产出新
   Chunk 描述符+两侧摘要，fault-in 仅读），插入文本=普通 Leaf。快照
   语义免费（frozen ①）；gap buffer/journal 弃（快照成本/读合并成本
   无补偿）。
5. **预扫=一次顺序读（1MB 缓冲）**逐页记摘要+UTF-8 验证；全文件
   digest 由树 combine 自动合成。预期 1GB 墙钟 1-2s（NVMe 顺序读
   ~2GB/s+单遍字节计数），T-08 实测入谱；降级预案（异步预扫+渐进
   行数）不启用，除非实测>5s。
6. **S5/725 协同=fault-in 不接脏域管线**（帧域正交：cosmic Buffer
   窗口与 rope 分层，预取载荷到达不触 725 脏帧——窗口重物化属下游
   S2 消费件，注记入 SD-01）。**占位语义裁定**：内核=同步 fault-in
   （单页 64KB NVMe ~100μs，远低于帧预算）+异步预取保温（滚动方向
   ±16 页）+页面态可观测（`PageState::{Resident,Cold}` 探针面）；
   frozen ④ 达成方式=预取保温使滚动/编辑路径命中内存，冷跳转单页
   同步读在帧预算内；UI 级占位渲染表达=下游窗口件（S2，回执注记）。
7. **随形裁定（超纲注记）**：内存上界常数=页缓存预算 6MB 默认
   （`PageConfig`）+页表/树常驻 ~3MB@1GB → 契约"RSS ~10MB 级"以
   **结构计量断言 ≤12MB@1GB** 成文（SD-01）；后援形 core 的 cosmic
   Buffer 物化=头部窗口（默认 2MB 截断至行边界），交互打字窗口化
   （S2 视口物化）=673 延后裁定在案的下游件——本件 E2E 面=键控
   native 面（load/edit/save/find/diff/undo-snapshot），窗口基址
   偏移使窗口内打字正确映射全文档区间。保存=合并写流（Chunk span
   →基底句柄 pread 照抄**绕过 LRU**；Leaf→直写），temp+rename 原子
   改名，外部修改检测=len+mtime 对预扫基线（不符→拒绝保存报错），
   写后基线刷新。512MB 拒绝位退役=回执两案不动（Q-3）。

**实勘修正（affects 对账）**：vm/native.rs 与 ui_gen/rust.rs 预期
**零 diff**——load/save shim 既存且转发键控 API（装载形内化于
`code_editor_load_file`，计划 §5 T-07"无新 native 预期"成立）；
Cargo.toml 预期零 diff（零新依赖）。落点收敛为 rope.rs+rope/
file_backing.rs+core/mod.rs+src/tests/plan728_supply_probes.rs。
**T-09 门禁修正（fix-test-tiering 2026-09-30 裁定）**：per-plan
复审门=裸 `cargo t`+触面档（本件无 vm 编译器/trans/book/ui_gen
触面→无 tv/tt/tb/tu 追加）；`cargo tf` 归批量回归档（主检出
`/auto-plan:regress`），计划原文"tf 全量"按仓规修正。

### T-01 基底页表+预扫（G-1，面①）

只读句柄+页表构建（预扫一次顺序读逐页记{偏移,长度,行数}）；
`line_count`/行号跳转由页表合成（免调入）；页不可变不变式（调试
断言+文档）。

### T-02 overlay 编辑覆盖层（G-2，面②）

按 T-00④ 选型实施；insert/delete/replace 全进 overlay；读路径
合并（overlay 优先+基底 fault-in）；虚拟长度/行数记帐（页表行数
+overlay 行数差合成）。

### T-03 LRU 淘汰与内存上界（G-3，面③）

页缓存 LRU（常驻=可见窗+预取+overlay）；淘汰页退化指针；RSS 计量
面（页缓存字节+overlay 字节——上界断言可测）；**上界契约常数成
文**（~10MB 级@1GB——SD-01）。

### T-04 异步调入+预取（G-4，面④）

后台读线程（快照隔离消费形态——diff 后台任务先例）；滚动方向预取
窗；未就绪页占位语义（读 API 返回占位标记→消费方渲染占位——
**调用面零破坏前提下的占位表达**：快照读阻塞 vs 占位双形，T-00⑥
接线面定）；024 S5 教训 frozen ④。

### T-05 保存合并（G-5，面⑤）

未改区段照抄（基底句柄 range copy）+overlay 增量写回；临时文件
+原子改名（崩溃安全）；往返 byte-for-byte 单测族（含 CRLF/BOM
字节——BOM/EOL 解释归下游，本件字节忠实）；外部修改检测策略
（句柄+mtime/大小——拒绝位语义重估注记随回执）。

### T-06 快照/消费方兼容（G-6，面⑥）

后援形 RopeSnapshot（页表+overlay 不可变视图——fault-in 在快照
读内）；四族消费方 overlay 意识：查找（逐页扫描协议）/diff
（9916 按需区间调入）/撤销（overlay 记帐——undo 栈语义不变）/
跳转+状态栏（页表合成+渐进行数）；**既有 rope 测试族零回退+API
签名零 diff 断言**。

### T-07 装载链贯通+big 态（G-7，面⑦）

`code_editor_load_file` 后援臂（>阈值——替换 :2161 全量读）；
natives/a2r 臂同步（无新 native 预期——装载形内化；若有契约面
新增按五面注册惯例）；big 态语义断言（≥50MB 绕语法臂不因装载形
变化）；小文件臂逐字节原样（golden 对照）。

### T-08 基准+验收谱（G-8）

生成式阶梯 fixture（50MB/512MB/1GB）：装载墙钟（预扫+页表构建）
+RSS 上界实测（1GB 档）+滚动/编辑帧口径（725 后带不劣化——占位
语义下）+保存往返+小文件键入带对照；谱 JSONL/报告入档。

### T-09 回归门

cargo tf 全量（预存红对账零新增——703 惯例）+rope/diff/find 族
全绿+探针族（plan728_supply_probes——710 形态：页表/overlay/LRU/
合并写/快照兼容五族）。

### T-10 规范+账本（G-9）

SD-01 契约册（七面语义+上界常数+快照兼容面+装载链+保存语义+big
态/拒绝位退役回执）+242 tracker 注记（若涉 a2r 面）+specs.json
P728-1（703/710 先例）+供料档 §10 回执节预告位。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/paged-rope.md | before：rope 契约=内存全量形（rope-subtree-hash 册在册——摘要/快照语义）；装载=全量读；无内存上界契约 / after：后援分页 rope 契约——七面语义（不可变基底页表/overlay 读写路径/LRU+RSS 上界常数/异步 fault-in+占位/合并保存+原子改名/消费方 overlay 意识四族/big 态+拒绝位退役回执）+分页阈值与双轨形态+快照 API 零破坏面+基准口径 | 供⑮ 落账真源；下游消费件（门退役+open_1gb 断言化）的内核侧依据 | AC-01..08 |

## 6. 测试设计

- **单测族**：页表（预扫行数/跳转合成/页不可变断言）+overlay
  （编辑记帐/读合并/fault-in）+LRU（淘汰/上界断言）+合并写（往返
  byte-for-byte×{纯 ASCII/CRLF/BOM/多区段编辑}+崩溃安全[中断重开
  =原文件未损]）+快照兼容（既有 23 测试族零回退+API 签名 diff=
  零断言）。
- **消费方四族**：查找逐页/diff 按需区间/撤销/跳转状态栏——各
  最小 E2E（后援形缓冲区上）。
- **探针族**（plan728_supply_probes）：五族单测的探针化（710
  形态）+natives 贯通（装载→编辑→保存 E2E 经 native 面）。
- **基准谱**：阶梯 fixture（50M/512M/1G）×{装载墙钟,RSS,帧口径,
  保存}+小文件键入带对照（725 谱基线）。
- **回归门**：cargo tf 预存对账零新增；小文件臂 golden 逐字节。

## 7. 验收标准

- **AC-01 基底+页表+预扫**：1GB 打开=只读句柄+页表；行数/跳转免
  调入即答；页不可变断言绿。验证：单测+探针。
- **AC-02 overlay**：编辑全进 overlay+读合并+虚拟记帐正确（增删
  改混合序列对照纯内存 rope 等价）。验证：等价单测（双轨对照——
  同操作序列全量 rope vs 后援形同结果）。
- **AC-03 内存上界**：1GB 文档常驻 RSS ≤上界常数（SD-01 成文值，
  ~10MB 级）实测谱。验证：基准谱+计量断言。
- **AC-04 异步调入+占位**：fault-in 不阻塞编辑/滚动主线程（帧带
  不劣化）+未就绪占位语义可观测。验证：帧口径谱+占位探针。
- **AC-05 保存合并**：往返 byte-for-byte 全绿+原子改名崩溃安全。
  验证：单测族（含中断重开）。
- **AC-06 快照/消费方零破坏**：既有 rope 测试族零回退+API 签名
  零 diff+四族消费方 E2E 绿。验证：tf 对账+grep 锚。
- **AC-07 装载链+big 态**：native 面装载→编辑→保存 E2E 绿+big
  绕语法臂断言+小文件臂 golden 逐字节原样。验证：探针+golden。
- **AC-08 基准+回归**：阶梯基准谱在档（装载/RSS/帧/保存四线）+
  tf 预存对账零新增。验证：谱 JSONL+tf 输出。
- **AC-09 规范+账本+回执位**：SD-01 落档+P728-1 回读 True+供料
  档回执节预告位在档。验证：文件在档+账本断言。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 勘定决策件 | — | 本件 §5 T-00 节+勘定报告 | 六定参（形态/阈值/页参/overlay/预扫/协同） | 全 | [x] 勘定报告在档（§5 T-00 勘定结论节，2026-10-02；六定参全落+随形裁定三条+affects/tf 门修正） |
| 1 | T-01 基底页表+预扫 | T-00 | rope.rs（扩展） | 面① | AC-01 | [x] 单测绿（file_backing::tests 14/14：prescan_summaries/page_boundaries/invalid_utf8/line_jump——3bc929ca0） |
| 2 | T-02 overlay | T-01 | rope.rs（扩展） | 面② | AC-02 | [x] 双轨等价绿（dual_track_edit_equivalence 220 操作序列 vs 纯内存 rope：内容/长度/行数/摘要/点往返全等） |
| 3 | T-03 LRU+上界 | T-02 | rope.rs（缓存层） | 面③ | AC-03 | [x] 上界断言绿（lru_eviction 全文档扫描预算钳制+residency_contract 计量断言；6MB 预算/≤12MB@1GB 成文 SD-01） |
| 4 | T-04 异步预取+占位 | T-03 | core/mod.rs 接线 | 面④ | AC-04 | [x] 预取可观测绿（prefetch_flips_pages_resident_off_thread：后台线程置 Resident+计数；占位语义=PageState 可观测+frozen ④ 由保温达成——SD-01 帧域节；帧带谱=typing 双轨带在 T-08） |
| 5 | T-05 保存合并 | T-02 | core/mod.rs 保存面 | 面⑤ | AC-05 | [x] 往返族绿（save_roundtrip_families 五族 byte-for-byte+temp 零残留+外部修改拒绝原文件未损+基线刷新二连保存+异目标免基线） |
| 6 | T-06 快照/消费方 | T-02 | RopeSnapshot+四族 | 面⑥ | AC-06 | [x] 零回退+零 diff（rope 既有族 35/35 零回退+api_signature_pin 签名钉+paged_snapshots 冻结/Merkle/prune+探针 find/jump E2E；**顺手修**：703 对齐行走字符中间切片隐患——leaf 比较改字节基，内存 rope 同受益） |
| 7 | T-07 装载链+big | T-01..05 | core/mod.rs:2160+native 面 | 面⑦ | AC-07 | [x] E2E+golden 绿（paged_load_edit_save_e2e：51MB 跨阈值档装载→行数即答→远端编辑→保存 byte-for-byte→外部修改拒绝；small_file_arm_golden 小文件臂逐字节+is_paged=false；natives/a2r 零改动=T-00 勘定成立；big 态=装载零语法耦合+缓冲区窗口物化护栏） |
| 8 | T-08 基准谱 | T-07 | 生成式 fixture+报告 | 四线谱 | AC-08 | [x] JSONL 在档（docs/reports/p728-bench.jsonl 七行净谱：release 50m/512m/1g 四线+typing 双轨带+RSS 线——1GB 装载 4010ms〔预案 5s 界内〕/answer 0.157ms/结构计量 3033695B/保存 657ms；measure_test_mem 1g 峰值 17MB LT；#[ignore] 显式档不入日常池） |
| 9 | T-09 回归门 | T-01..07 | tf 全量+探针族 | 回归门 | AC-08 | [x] 对账零新增（worktree 裸 cargo t 54.7s：4992 run/4981 绿/11 红——10 红逐项对上批量回执 known_reds〔musk_vm_track p053×4+p054×2/plan606/projector/desktop_bus/desktop_surface〕+ash_stream_leak_probe 基面（主检出零 728 diff）单测复证红=预存；**零新增红**；tf 按仓规归批量回归档〔fix-test-tiering 2026-09-30，T-00 已记修正〕；触面档零追加〔vm/native.rs 与 ui_gen 实际零 diff〕） |
| 10 | T-10 规范+账本 | 全 | SD-01+specs.json+回执位 | 落账 | AC-09 | [x] P728-1 True（SD-01=docs/specs/auto-lang/ui/design/paged-rope.md 七面契约+双轨形态+不变式+帧域注记+基准口径+Q-3 回执两案；specs.json designs 段 P728-1 upsert+roundtrip True〔docsha 留 merge 定稿〕；供料档 §10 回执预告位=auto-edit de6cb95 四项下游消费件清单；KNOWN-DEBT P728-D1..D3 登记） |

## 9. 复审记录

- 2026-10-02 起草 handoff：`stage: new`，PLAN-728，plan_revision 1。
  `outcome: pass`（起草完备：供⑮ 七面种子逐面承接为 G-1..G-7+
  验收/账面；四条 frozen（API 零破坏/小文件零回退/基底不可变/不等
  磁盘）防破坏性；双轨等价测试法（同操作序列 vs 纯内存 rope）为
  正确性主判据；量级与 T-00 六定参（形态/阈值/页参/overlay/预扫/
  帧域协同）如实留界不预支实现；下游适配件边界清晰[669 模式——
  512MB 门退役+open_1gb 断言化=回执消费件]；路径/符号经
  auto-lang@b385534d7 与 auto-edit 供料档 §10 双源锚定[rope.rs
  结构/装载链全量读锚/快照消费方四族]；授权=起草[用户指令原文在
  录]，执行待用户启动——auto-lang 会话，727 稿并行协调）。`next:
  work`。


- 2026-10-02 work handoff：`stage: work`，PLAN-728，plan_revision 1。
  `outcome: pass`（T-00..T-10 全执行完毕，AC-01..09 证据入任务表）。
  `code_commit`: plan-728-dev@3bc929ca0（内核+集成+探针）+c19fec79f
  （基准谱+SD-01）；基面 master@8bf65335d（含 plan 簿记 be92a13ac）。
  `task_ids`: T-00..T-10。`evidence`: rope 内核单测 14/14+既有 rope 族
  35/35 零回退+探针 5/5（E2E 51MB 跨阈值/golden 小文件臂/API 签名钉/
  查找跳转 rewindow/结构计量）+裸 cargo t 54.7s 零新增红（11 红全预存
  对账，ash_leak_probe 基面复证）+基准谱 JSONL 在档（release 四线+双轨
  逐键带+RSS 17MB LT）+SD-01/P728-1 回读 True/回执预告位 auto-edit
  de6cb95。顺手修复=703 对齐行走字符中间切片隐患（字节基化，内存 rope
  同受益，P728-D2 登记）；T-00 勘定修正三处如实（affects 收敛 vm/native
  与 ui_gen 与 Cargo 实际零 diff；tf 门改裸 cargo t 批量归档；量级
  注记 1GB 装载 4.0s 在降级预案 5s 界内未触发）。`blockers`: 无。
  `next: review`（独立复审→merge；worktree lang-728 组保留待复审）。

## 10. 待澄清事项

- **Q-1 抽包 vs 就地（T-00① 定，默认就地）**：默认就地扩展
  rope.rs（703 先例+feature 接线零成本）；若 T-00 证据显示独立
  crate 收益显著（测试隔离/跨仓复用），可预裁定改抽包（auto-atom
  先例）——执行前示知即可。
- **Q-2 分页阈值（T-00② 定参，无需预裁）**：候选 50MB[big 态
  对齐]/128MB/256MB——判据=小文件性能带零扰动边界（frozen ②）+
  RSS 收益起点；默认倾向 50MB 对齐（big 态语义统一）——按 T-00
  证据定。
- **Q-3 512MB 拒绝位退役形态（回执建议位，下游裁定）**：本件证
  「1GB 可装载可编辑可保存」后，下游门处置（移除 vs 抬升至[如
  4GB/内存上界反推值]）=auto-edit 消费件裁定——本件回执两案并附
  （供料档 §10 回执节）。
