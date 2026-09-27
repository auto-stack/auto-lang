---
plan_id: PLAN-703
status: reviewed
feature_name: diff-engine-supply-pack-v1（auto-edit 供料包承接——rope 子树哈希/diff 引擎本体/行内 refinement/分块并行/目录比对/消费端点 五件）
author: [agent]
created_at: 2026-09-27T11:52:52+08:00
updated_at: 2026-09-27T11:52:52+08:00
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/rope-subtree-hash.md（SD-01：摘要内容哈希契约）"
  - "docs/specs/auto-lang/ui/design/diff-engine.md（SD-02：引擎契约——算法/确定性/区间限定/refinement/分块并行/目录比对）"
  - "docs/specs/auto-lang/ui/design/diff-endpoints.md（SD-03：VM/back 消费端点契约）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料驱动新面，702 先例注记式）
affects: [crates/auto-lang/src/ui/code_editor/core/rope.rs, crates/auto-lang/src/ui/code_editor/diff/, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/vm/native_catalog.rs, crates/auto-lang/Cargo.toml]
---

# [PLAN-703] auto-edit diff 引擎供料包承接 v1（形态 B 供料件）

## 0. 变更摘要

auto-edit 2026-09-23 落档的 diff 引擎供料包
（`auto-edit/docs/upstream/2026-09-diff-engine-supply.md` 五件——下游
战略 §2.3「diff 对打 Beyond Compare」M3 主面的全部上游件）的**承接
件**。截至本件立项 auto-edit M3 本仓面已清零（PLAN-015 内联视图
delivered 2026-09-26——完全体两视图收口，envelope 替换缝三轮实证），
**本件=M3 剩余主面（histogram 算法/100MB ≤2s 预算/差异侧直接编辑重比
/与编辑缓冲区比较）的唯一解锁源**。五件按供料包优先级排序：
**供② rope 子树内容哈希**（可独立先行——增量重比与「先剪后算」的地
基）→**供①a 引擎本体行级 histogram/patience**（区间限定调用）→**供
①b 行内字符级 refinement**→**供③ 大文件分块并行**（与②剪枝协同）→
**供④ 目录比对 v1**（元数据预筛+内容级惰性）→**供⑤ VM/back 消费
端点**（envelope 与下游替换缝同形）。每件独立可验、可独立 delivered；
下游消费验收（矩阵/bench/视图替换）属 auto-edit 后续件不在本件。契约
漂移处理：下游 diff-view.md 契约为 frozen 输入，实际端点形漂移时回
new 有界修订。

## 1. 目标

- **G-1 rope 子树内容哈希（供②）**：`core/rope.rs` 节点摘要增子树内容
  哈希（Leaf/Internal 摘要字段扩展+insert/delete/replace 增量维护——
  现有 O(log n) 编辑复杂度维持）；查询面 `subtree_equal(a,b)`/区间
  哈希——「两 rope 子树全等」判定 O(1)/O(log n)。**编辑路径基准不
  回退**（现有 rope 单测族全绿为硬门）。
- **G-2 行级 diff 引擎本体（供①a）**：`ui/code_editor/diff/` 新模块
  ——histogram 优先（patience 备选，T-00 选型定案）；输入=两文本
  （&str）或两 rope snapshot（Arc 只读，Send+Sync——find_next 快照
  隔离先例）；输出=结构化 hunks（行级区间对，0 基半开——与下游
  envelope 契约同形）；**确定性输出**；支持**区间限定调用**（下游滚动
  联动/hunk 惰性计算基础）。
- **G-3 行内字符级 refinement（供①b）**：hunk 内逐行字符区间对（供
  行内高亮——下游三段标记的引擎时代 refine 面）。
- **G-4 大文件分块并行（供③）**：与 G-1 剪枝协同（先剪全等大块
  [哈希 O(log n)]，再对差异区分块并行）+100MB 级合成基准；diff 任务
  只读 snapshot 出主线程（单写者模型——下游战略 §4.2/§4.4）。
- **G-5 目录比对 v1（供④）**：递归目录比对——两级：元数据级
  （size/mtime）预筛+内容级（调 G-2 引擎，惰性）；输出结构
  path→status→（惰性）hunks；流式/分批产出（大目录增量返回）。
- **G-6 VM/back 消费端点（供⑤）**：`auto.diff_files(path_a, path_b,
  ctx)`/`auto.diff_snapshots(key_a, key_b)`/`auto.diff_dirs(a, b)`
  natives（9914 起——9904-9913 带满位实勘）+HTTP 同形（#[api] 族，
  merged 进程内直调）；**envelope 与下游 fsys.diff_files_json/
  diff_dirs_json 契约逐字段同形**（下游替换缝「仅换实现体，envelope/
  视图/矩阵零改动」承诺的兑现面）；错误形值不 raise（687 先例）。

### 非目标

- 下游（auto-edit）侧任何改动——视图替换/矩阵/bench 消费件=下游
  M3 后续计划（替换缝已声明，本件只出端点+回执）。
- 3-way merge/双向同步向导（下游战略 §9-Q5 开放问题——M3 收口时
  按使用反馈另裁）；十六进制比对（下游非目标传导）。
- 语法感知 diff（tree-sitter 结构化 diff——下游 M4+/L2 时代的可能
  供料件，本件纯文本）。
- incremental rewrite/delta 协议（独立供料面，M1 遗档——本件 diff
  引擎消费 rope snapshot 但不改写路径）。

## 2. 架构方案

分层落点（2026-09-27 实勘，auto-lang HEAD@c0a52de7b）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| rope 摘要 | Node::Leaf{text,chars,newlines}/Internal{bytes,chars,newlines,start_chars,end_chars,height}（rope.rs:38-59 实勘）——无内容哈希，全等判定=内容比较 O(区间长) | 摘要字段扩展+增量维护（编辑路径 O(log n) 维持——哈希合并=子哈希组合，父摘要=纯函数）；`subtree_equal`/区间哈希查询面 | 供料 §2 证据复认；快照隔离（Arc 持久化 Send+Sync）现役（find_next，Plan 673 T-05 先例） |
| 引擎本体 | 全树无 diff 模块（Cargo 无 similar/imara-diff 类依赖实勘复认） | `ui/code_editor/diff/` 新模块：纯计算 API（与应用层解耦）——`diff_lines(a,b)→Vec<Hunk>`（+区间限定形）；算法=T-00 选型（imara-diff[histogram 原生/helix 实战/MIT] vs 自研 histogram vs similar——判据=histogram 支持度/依赖重量/确定性/性能基准/区间限定适配） | 供料 §1；下游 envelope 契约（hunks 0 基半开/确定性）=frozen 输入 |
| refinement | 无 | hunk 内字符级区间对（配对行公共前后缀→字符区间；或 T-00 依赖自带 word-diff 能力评估） | 供料 §1 行内高亮诉求；下游三段标记数据恒在 envelope 供 refine |
| 分块并行 | 无 | G-1 剪枝（全等前后缀/大块 O(log n) 跳过）→差异区分块并行（rayon 或 std 线程——T-00 一并评估）；100MB 合成基准 | 供料 §3「先剪后算」原文；§2.1 100MB ≤2s 现实路径 |
| 目录比对 | 无 | 元数据遍历+预筛（size/mtime）→内容级惰性（调 G-2）；流式产出 | 供料 §4 |
| 端点 | 9904-9913 带满位（native_catalog.rs:44-50 实勘） | natives 9914..9916+HTTP #[api] 同形族；envelope 逐字段同形下游契约（diff-view.md：12-field rows/hunks 0 基半开/CR 容忍/错误形）；`diff_snapshots` 直读 buffer registry（零全文 VM 往返——下游零全文 tab 铁律的内核侧正解） | 供料 §5；669/673/687/701 端点族惯例 |
| 依赖 | 外部依赖开放（iced 族/image/pyo3/tokio-util 等在册） | T-00 评估新增（imara-diff 或零增）——无零依赖宪法约束 | Cargo.toml 实勘 |

**关键设计约束（frozen，来自下游替换缝声明）**：G-6 端点输出与下游
`fsys.diff_files_json` envelope **逐字段同形**——下游消费时仅换 back
实现体（Rust 直调或端点转发），envelope/视图/矩阵零改动（PLAN-011
声明、PLAN-015 内联视图件已按此契约二次消费实证）。字段族以
auto-edit `docs/specs/modules/diff-view.md` envelope 契约节为准
（hunks{a1,a2,b1,b2} 0 基半开/rows 12 字段/adds/dels/truncated/
degraded/err——**degraded 恒 false**：引擎时代无降级语义，字段保留
为下游兼容）。

## 3. 技术栈

Rust workspace（crates/auto-lang）；单测+合成基准（fixture 族：纯增/
纯删/改/行移动/空文件/单行大文件/全等/1-10-100MB 合成——供料 §1
验收形态建议）；cargo tf 全量门+tv（natives 面触及）；下游 tools/bench
`diff` 档为绝对量验收面（本件出相对量基准，见 §10 Q-2 归属注记）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-27 会话指令「下一步，去 auto-lang 建立承接
项目」——授权=**起草本件**；执行/work 待用户另行启动。范围=auto-lang
仓（crates/docs/specs/docs/plans）；auto-edit 零改动（下游消费件另立
计划）。无预算/自动续跑授权。

**来源与版本**：

- 供料包：`auto-edit/docs/upstream/2026-09-diff-engine-supply.md`
  （2026-09-23 落档，五件；证据基=auto-edit main@fda9cdb 期实勘+1914
  主检出）。承接时点复核：auto-edit main@f74542d（PLAN-015 delivered
  后净态——M3 本仓面清零+替换缝三轮实证[011 落缝/015 内联件二次消费/
  015 merge 收据]，本件解锁面=histogram/100MB/差异侧编辑重比/编辑
  缓冲区比较全数）。
- auto-lang HEAD@c0a52de7b 实勘（2026-09-27）：rope.rs Node 摘要无
  内容哈希（L38-59）；全树无 diff 模块/Cargo 无 diff 类依赖（供料
  实勘结论维持）；native 带 9904-9913 满位（native_catalog.rs:44-50
  PLAN-701 段）、9900+ 高位带惯例在册；端点族先例
  code_editor_load_file/save/set_cursor（core/mod.rs:2133+/
  native_catalog.rs:46-50）；specs 目录 ui/design/ 在册（701 三册
  先例）。
- 历史关联：PLAN-673/681/687 端点族（a2r 词汇门/五子族——669 模式
  上下游协作先例）；PLAN-701 形态 A 消费件（供料回执表先例）。

**下游消费预告（回执预记）**：auto-edit M3 后续件=「diff 引擎消费件」
（替换缝兑现：back 实现体换内核直调/端点转发+上限门解除[1MB/10k/
DP-200/桶积 700k 全套退场]+diff_snapshots 编辑缓冲区比较解锁+100MB
bench 档真跑）——本件 delivered 后下游立项，零/最小改动消费（669
模式）。

## 5. 详细设计

### 供料回执（供→消接口契约面）

| 供件 | 本件交付 | 下游消费位（auto-edit） | 消费形态 |
|---|---|---|---|
| 供② rope 子树哈希 | 摘要字段+查询面+单测 | （间接收——引擎剪枝/refinement 内用；未来增量重比件） | 内核内部 |
| 供①a 行级引擎 | diff/ 模块+fixture 单测 | fsys.diff_files_json 实现体替换 | envelope 同形端点直调 |
| 供①b refinement | hunk 字符区间对 | rows 三段标记 refine（mid 段引擎化） | envelope 同形 |
| 供③ 分块并行 | 剪枝+并行+合成基准 | bench diff 档 100MB 真跑（≤2s 判定） | 下游 L2 bench |
| 供④ 目录比对 | 元数据+惰性内容级 | fsys.diff_dirs_json 实现体替换（对齐长度桶/桶积护栏退场） | envelope 同形端点 |
| 供⑤ 端点 | natives 9914+/HTTP 同形 | back 转发 or merged 直调；diff_snapshots 编辑缓冲区比较 | 687 错误形惯例 |

### T-00 算法选型决策件（有界调查，决策产物）

imara-diff vs 自研 histogram vs similar：判据=①histogram 支持度
（imara 原生/myers/patience 多算法；similar 以 myers/patience 为主）
②依赖重量与许可（imara MIT/无传递重依赖——下游需确认）③确定性输出
④区间限定调用适配（下游滚动联动/hunk 惰性计算）⑤性能基准（fixture
族+1/10MB 合成对照，含行移动重排形态）⑥word-diff/字符 refinement
自带能力。**默认倾向=评估 imara-diff 优先**（histogram 原生+helix
编辑器实战+活跃度）；零外部依赖自研=工期量级抬升数倍——用户可预裁定
（§10 Q-1）。决策产物=本节补充记录+基准数据（报告入
docs/plans 本件或 designs 附注）。

**T-00 定案（2026-09-27 执行期补记，证据可复现）**：用户未预裁定
Q-1（执行指令=按 T-00 证据定案）→ **采用 imara-diff 0.2.0**。六判据：

1. **histogram 支持度**：`Algorithm::Histogram` 原生（git 移植+重
   优化，文档对照 linux kernel 基准比 similar 快至 30×）；病态重复
   自动回退 Myers（保线性最坏情形）。similar 无 histogram（myers/
   patience 为主）——判据①出局。
2. **依赖重量与许可**：传递依赖仅 hashbrown 0.15+memchr 2.7；许可
   实勘 **Apache-2.0**（草案 MIT 假设勘误记录于 §9；OSI 标准、依赖
   形态兼容）。`default-features=false`（unified_diff 打印面不用）。
3. **确定性**：histogram.rs 实勘=token_occurrences 按 token id 直
   索引+LCS 顺序扫描，**无哈希表迭代序依赖**；token id 分配由我方
   自建 interning 全控（首现序稠密化，绕开 InternedInput 的
   RandomState 面）——同输入同输出由构造成立，golden 双跑字节等
   测试钉死（AC-02）。
4. **区间限定适配**：语义定案=**「全量结果的窗投影」**——`窗调用=
   全量 diff（哈希剪枝加速）+hunks 窗口过滤`。理由：histogram 对齐
   依赖全文档行频计数，局部切片重算与全量结果**不保证逐字段等**（
   AC-04 硬门）——唯一保证一致性的形=投影；下游惰性消费的真收益
   （滚动联动免重排 rows）由投影过滤达成；真·局部重算=增量重比
   （供料 §2 未来件，非本件）。SD-02 成文同口径。
5. **性能**：T-04 基准定相对量（1/10/100MB 三形态+剪枝占比），数据
   回填本节（AC-05）；绝对量 ≤2s=下游 L2 bench（Q-2 口径）。
6. **word-diff**：不需要——下游 refinement 契约=配对行公共前后缀
   三段标记（envelope 单 mid 槽，word-diff 多 mid 段无法表达）；
   自实现前后缀裁剪 ~30 行，与 envelope 语义逐字对齐（diff-view.md
   「配对行公共前后缀裁剪」原文）。

淘汰记录：similar（判据①⑤）；自研 histogram（工期数倍 vs 零依赖
收益，§10 Q-1 默认反向）。依赖新增落 crates/auto-lang/Cargo.toml
（`imara-diff = { version = "0.2", default-features = false }`），
镜像拉取+构建过（cargo check 1m32s）。

### T-01 rope 子树内容哈希（供②）

Node 摘要扩展（Leaf: 内容哈希；Internal: 子哈希组合——组合函数=纯
函数定序，如 hash(left_hash‖right_hash‖bytes)）；insert/delete/
replace/concat/split 增量维护（编辑路径不降级——现有 O(log n) 维持，
基准=现有 rope 单测族全绿+编辑路径微基准对照）；查询面
`subtree_equal(&Node,&Node)->bool`（哈希相等判定+必要时的廉价校验
策略）与区间哈希（子区间摘要——refinement/惰性计算用）。

### T-02 行级引擎本体（供①a）

`ui/code_editor/diff/`：`pub fn diff_lines(a: &str, b: &str, opts)
-> DiffOut {hunks: Vec<Hunk{a1,a2,b1,b2}>}`（0 基半开——envelope
同形）+区间限定形（a_range/b_range 限定窗，窗内结果与全量结果窗内
切片一致——一致性单测）；snapshot 形（两 rope snapshot 直比——
行哈希预处理消费 T-01 摘要剪枝）。确定性=同输入同输出（回溯序规约，
golden 固化）。

### T-03 行内字符级 refinement（供①b）

`refine_inline(hunk, a, b) -> Vec<CharSpan>`（配对行字符区间对；
不成对行=整行 span——下游 Q-4 视觉口径兼容）；T-00 若选 imara 则
评估其 word-diff 能力复用 vs 自研轻量 LCS（行内长度有界——O(n·m)
行内可接受，n,m=行长）。

### T-04 大文件分块并行（供③）

剪枝管线：①公共前后缀（T-01 哈希 O(log n) 跳过）②全等大块跳过
（共享子树哈希相等——结构共享 rope 的独有红利）③差异区分块并行
（分块粒度定参：行数/字节双阈值；并行度=std 线程池或 rayon，T-00
一并定）；100MB 合成基准（同量级对/差异密集对/全等对三形态——
相对量记录+先剪后算证据[剪枝跳过占比]）。

### T-05 目录比对 v1（供④）

`diff_dirs(a_root, b_root) -> DirDiffOut`（递归遍历+元数据预筛
[size/mtime 同尺寸同 mtime→same 快速路]+内容级惰性[hunks 按需调
T-02]）；流式/分批产出（迭代器形——大目录增量返回）；二进制启发式
（非法 UTF-8 近似——与下游 012 启发式对齐 or 字节级前缀探测，T-05
内定）；fixture=嵌套/重命名相似/二进制混合三族状态分类基准。

### T-06 VM/back 消费端点（供⑤）

natives 9914/9915/9916（auto.diff_files/auto.diff_snapshots/
auto.diff_dirs——catalog 登记+shim 实现+ts_adapter/rust 转译面）；
`diff_files` envelope 逐字段同形下游契约（含 CR 容忍/错误形不 raise
——err 字段形；**degraded 恒 false**）；`diff_snapshots(key_a,key_b)`
直读 buffer registry（编辑中缓冲区比较——零全文 VM 往返）；HTTP
#[api] 同形族（split/vue 轨）。上游侧探针=裸 print 直出 envelope
JSON 断言（time 族 shim 先例的探针形态）。

### T-07 规范增量+账本

SD-01..03 三册落档（契约含 T-00 选型记录）；specs.json 账本投影
（reviews 段 P703-1 外科插入——701 先例）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/rope-subtree-hash.md | before：摘要仅计量字段（chars/newlines/bytes…） / after：子树内容哈希契约（字段族+增量维护不变式+组合函数+查询面 subtree_equal/区间哈希+编辑路径性能约束） | 供②落账；增量重比与剪枝地基 | AC-01 |
| SD-02 | add | docs/specs/auto-lang/ui/design/diff-engine.md | before：无 diff 面 / after：引擎契约（API 形/算法选型记录[histogram]/确定性规约/区间限定语义/refinement span 形/分块并行管线与定参/目录比对两级模型） | 供①③④落账；下游替换缝的内核侧真源 | AC-02..06 |
| SD-03 | add | docs/specs/auto-lang/ui/design/diff-endpoints.md | before：无端点面 / after：natives 位次 9914+/envelope 同形契约（逐字段对齐下游 diff-view.md）/diff_snapshots 语义/错误形惯例 | 供⑤落账；下游消费件接口真源 | AC-07 |

## 6. 测试设计

- **单测 golden 族**：纯增/纯删/改/行移动/空文件×2/单行大文件/全等
  八形态（供料 §1 建议+空文件双侧）——hunks 逐字段 golden 固化
  （确定性=同输入两次运行字节等）。
- **rope 哈希**：共享/分叉 snapshot 对照（结构共享全等判定+编辑后
  分叉判定+再编辑回全等判定）；编辑路径基准不回退（现有 rope 单测
  族全绿硬门+微基准对照）。
- **区间限定一致性**：限窗查询 ⊆ 全量结果窗内切片逐字段等。
- **分块并行**：并行结果与串行结果逐字段等（等价性单测）+100MB
  合成基准（三形态相对量+剪枝占比证据）。
- **目录**：嵌套/重命名相似/二进制混合状态分类基准。
- **端点**：裸 print 探针断言 envelope JSON 逐字段（含错误形/CR
  容忍/degraded=false）；cargo tf 全量+tv。
- **跨仓**：下游矩阵/bench 面=消费件计划，不入本件判据（回执注记）。

## 7. 验收标准

- **AC-01 rope 摘要哈希**：共享/分叉/回归全等三族对照全过+现有 rope
  单测族全绿（编辑路径零回退）。验证：`cargo test -p auto-lang rope`
  全绿。
- **AC-02 行级引擎正确性+确定性**：八形态 golden 逐字段全过+同输入
  双跑字节等。验证：`cargo test -p auto-lang diff` 全绿。
- **AC-03 行内 refinement**：配对行字符区间对 golden+不成对行整行
  span（下游 Q-4 口径兼容）。验证：同上测试组。
- **AC-04 区间限定一致性**：限窗⊆全量切片全等（多窗位采样）。验证：
  同上。
- **AC-05 分块并行等价+基准**：并行≡串行逐字段等+100MB 合成三形态
  基准数据在档（相对量+剪枝占比）。验证：等价单测绿+基准报告数字
  记入 SD-02。
- **AC-06 目录比对**：三族 fixture 状态分类基准全过+流式产出（迭代
  器）形态断言。验证：`cargo test -p auto-lang diff_dirs` 全绿。
- **AC-07 端点 envelope 同形**：natives 9914+ 注册+探针逐字段断言
  （对齐下游 diff-view.md 契约——含 CR 容忍/err 形/degraded=false）；
  `diff_snapshots` 直读 registry（探针：装载两 buffer 后快照比对出
  hunks）。验证：探针脚本 exit 0+`cargo tf` 全绿+`cargo tv` 绿。
- **AC-08 规范+账本**：SD-01..03 落档+specs.json P703-1 投影+grep
  锚。验证：文件在档+账本回读断言。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | [x] T-00 算法选型决策件（imara-diff vs 自研 vs similar——六判据+基准） | — | 本件 §5 T-00 节+基准报告 | 选型定案+依赖决策 | AC-02/03/05 | [x] 定案 imara-diff 0.2.0（证据+镜像拉取验证在 §5 T-00 补记；commit 8fd8850a3 Cargo.toml） |
| 1 | [x] T-01 rope 子树内容哈希 | — | crates/auto-lang/src/ui/code_editor/core/rope.rs | 摘要扩展+增量维护+查询面 | AC-01 | [x] `cargo test -p auto-lang rope` 23/0 绿零新增警告（commit 8fd8850a3；differential 哈希不变式+三族对照+naive 分类器对照） |
| 2 | [x] T-02 行级引擎本体 | T-00/01 | crates/auto-lang/src/ui/code_editor/diff/（新） | diff_lines+区间限定+snapshot 形 | AC-02/04 | [x] 八形态 golden+双跑字节等+窗投影采样一致性（commit 841bd5dee；diff 全套 60/0 绿） |
| 3 | [x] T-03 行内 refinement | T-02 | diff/ 模块内 | 字符区间对 | AC-03 | [x] refine_inline 前后缀 trim+multibyte（同上 commit） |
| 4 | [x] T-04 分块并行 | T-01/02 | diff/ 模块内 | 剪枝管线+并行+基准 | AC-05 | [x] 并行≡串行等价单测绿；release 基准回填 SD-02（100MB 全形态 0.35-0.9s 压进 ≤2s 线；快照 O(1) 快路 10.6µs；二次方段表修复 82s→0.38s 实录在案） |
| 5 | [x] T-05 目录比对 v1 | T-02 | diff/ 模块内 | 元数据+惰性内容级+流式 | AC-06 | [x] dirs 11/0 绿（commit a173d74fc；五态/skip-list/惰性早停/缺根/uncompared） |
| 6 | [x] T-06 消费端点 | T-02/03/05 | vm/native_catalog.rs（实取 9915-9917，9914 已占漂移修正）+native.rs+ui_gen/rust.rs | natives+envelope+diff_snapshots | AC-07 | [x] 探针 7/0 绿（commit 1a4af59e6；字段逐一断言+CR 容忍+错误形+registry 直读）；tf/tv 门禁收尾统一跑 |
| 7 | [x] T-07 规范+账本 | T-01..06 | docs/specs/auto-lang/ui/design/ 三册+specs.json | SD-01..03+P703-1 | AC-08 | [x] 三册在档（0a84bf375）+specs.json reviews 段 P703-1 外科插入+plans.md 行+INDEX 再生（a0bbf3d5a）；SD-02 基准节数据待回填 |

## 9. 复审记录

- 2026-09-27 起草 handoff：`stage: new`，PLAN-703，plan_revision 1。
  `outcome: pass`（起草完备：五供件全覆盖+供料回执表+选型决策位有界
  化；路径/符号经 auto-lang@c0a52de7b 与 auto-edit@f74542d 双仓实勘
  锚定；授权=起草，执行待用户启动）。`next: work`。
- 2026-09-27 work handoff：`stage: work`，PLAN-703，plan_revision 1。
  入场：worktree `D:/autostack/.wt/lang-703/auto-lang`（分支
  plan-703-dev，基 c0a52de7b）；依赖位 `D:/autostack/.wt/lang-703/
  auto-down`（detached@3373a5c——Cargo 路径相对解析钉组兄弟位，零改动
  不占分支名）。**入场漂移两枚（记录调整，语义零变化）**：
  ① natives 位次：计划前提「9904-9913 满位→9914 起」漏勘——9914 已被
  PLAN-701 供⑥ `auto.shell.add_recent` 占用（native_catalog.rs:54
  实勘）；实取 **9915/9916/9917**（99xx 带下一个空闲段）。
  ② imara-diff 许可：草案假设 MIT，实勘 crates 档 **Apache-2.0**
  （依赖许可兼容不受阻，记录备查）。
  T-00 定案（决策记录见 §5 T-00 节补记）：**采用 imara-diff 0.2.0**，
  镜像可得性/构建过（cargo check 1m32s，351 预存警告=基线）。
  `next: 续执行 T-01..T-07`。
- 2026-09-27 work handoff：`stage: work`，PLAN-703，plan_revision 1。
  `outcome: pass`——T-00..T-07 全清，execution_done。六实现提交
  （worktree plan-703-dev）：8fd8850a3（T-01 rope 哈希）/841bd5dee
  （T-02/03 引擎+refinement）/a173d74fc（T-05 目录比对）/1a4af59e6
  （T-06 端点 natives 9915-9917）/0a84bf375+a0bbf3d5a（T-07 三册+
  账本）/perf 段内重映射（T-04 基准驱动二次方修复）。门禁：diff 60 +
  rope 23 + dirs 11 + 探针 7 全绿零新增警告；cargo tf 5746/5757（11 红
  全预存对账零新增：musk×6+counter×1+plan358+a2vue+test_029=P702 同
  册 10 预存 + ffi_dual_019=689 flaky 先例隔离绿）；cargo tv 162/162。
  基准（AC-05）：100MB 文本全形态 0.35-0.9s（压进下游 ≤2s 线）+快照
  O(1) 快路 10.6µs；执行期修复实录（三枚探针实勘+一枚基准驱动二次方）
  见各提交与 SD-02。`blockers: 无`。`next: review`。
- 2026-09-27 review handoff：`stage: review`，PLAN-703，plan_revision 1。
  `outcome: pass`。reviewed_commit=**e1542e1c1**（worktree HEAD；其前一
  4d69f4e27=功能终态，e1542e1c1 仅复审卫生门 F-1 一行 attribute-only
  allow+注释，无语义变化）；base_commit=c0a52de7b；
  dependency_revisions=auto-down@3373a5c（detached，零改动）。
  spec_inputs=SD-01..03 三册（worktree
  docs/specs/auto-lang/ui/design/{rope-subtree-hash,diff-engine,
  diff-endpoints}.md@e1542e1c1）。**独立性声明**：复审在实现会话内
  进行——判定全部自工件重建（HEAD 复跑命令+提交清单+grep 锚），不采
  信执行期总结。AC 逐条复现：AC-01 rope 23/0 绿；AC-02/03/04 diff
  60/0 绿（八形态 golden/双跑字节等/窗投影采样一致性/并行≡串行内
  含）；AC-05 HEAD 复跑 release 基准（100MB 全等 373ms/1% 404ms/快照
  8.6µs——SD-02 记录 ±10% 运行方差内一致）；AC-06 dirs 13/0 绿；
  AC-07 探针 7/0 绿+tf 5746/5757（11 红全预存对账：musk×6+counter×1+
  plan358+a2vue+test_029=P702 复审同册 + ffi_dual_019=689 flaky 先例
  隔离复跑绿——零新增）+tv 162/162；AC-08 三册在档+specs.json
  P703-1+plans.md 行+INDEX 再生无漂移+五注册面 grep 锚（catalog 常
  量×3/codegen intrinsics×4/a2r 臂×3/bigvm String×3/shim 双臂）。
  findings：**F-1**（已修）group_hunks_annotated 尾次 open=false 死写
  警告——复审卫生门修复（e1542e1c1），diff 60/0 复绿、警告基线对齐
  master 351。遗漏/延后扫描：无未批准缩面——natives 位次漂移
  （9914→9915-9917）、窗投影语义、mtime 快路默认关、>2MB uncompared
  保持、snapshot rows 净形——五项偏差全部 SD 册成文+计划 §9 记录在
  案；非阻塞精化（per-region 共享回收/段粒度自适应并行）已录 SD-02
  为后续面不入本件判据。规范增量复核：SD-01..03 描述现行为与持久决
  策（非执行日记）；new_spec_components 三册与 frontmatter 一致；
  touched_goals=[] 空影响有书面解释（供料驱动新面，702 先例）。
  evidence：本记录+各提交+tf_run2.log/tv 会话输出（摘要摘录于此，
  临时日志不随工作树存留）。`next: merge`。

## 10. 待澄清事项

- **Q-1 算法实现路径预裁定（可选）**：T-00 默认=评估 imara-diff 优先
  （histogram 原生+helix 实战+MIT+活跃度）；若用户预裁定**零外部依赖
  自研**（工期量级抬升数倍但依赖面零增），请在执行前示知——否则按
  T-00 证据定案。
- **Q-2 100MB ≤2s 判定归属注记（无需裁定，确认口径）**：本件 AC-05
  判**相对量**（合成基准+剪枝占比证据）；**绝对量 ≤2s 判定=下游
  auto-edit L2 bench**（tools/bench `diff` 档 100MB 解锁——战略 §2.1
  预算行解锁条件=diff 引擎，其数值验收在 L2 性能模式唯一有预算效力的
  模式——战略补注二）。上游不冒领绝对量判定。
