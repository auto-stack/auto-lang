# diff 引擎契约（算法/确定性/管线/目录比对）

> PLAN-703 T-02/T-03/T-04（供①a/①b/③，2026-09-27）。落点
> `crates/auto-lang/src/ui/code_editor/diff/`。下游替换缝的内核侧真源
> ——M3「diff 对打 Beyond Compare」主面的计算层（下游视图件全部只是
> 本引擎的编排面）。

## 算法选型（T-00 定案记录，2026-09-27）

**采用 imara-diff 0.2.0**（`default-features=false`；许可 Apache-2.0；
传递依赖仅 hashbrown+memchr）。六判据：

1. **histogram 原生**（git 移植+重优化；病态重复自动回退 Myers 保线
   性最坏情形）。similar 无 histogram——出局。
2. **依赖重量**：两件传递依赖；`unified_diff` 特性关（自建 envelope
   rows）。
3. **确定性**：histogram 实现纯数组扫描（token 直索引+LCS 顺序扫描，
   无哈希表迭代序依赖）；token id 由我方首现序 interning 全控（绕开
   InternedInput 的 RandomState）——golden 双跑字节等钉死。
4. **区间限定适配**：语义定案=**「全量结果的窗投影」**（见下）。
5. **性能**：见基准节（相对量；绝对量 ≤2s 判定=下游 auto-edit L2
   bench——计划 §10 Q-2 口径，上游不冒领）。
6. **word-diff 不需要**：下游 refinement 契约=配对行公共前后缀三段
   （envelope 单 mid 槽，word-diff 多 mid 段无法表达）——自实现 ~30
   行 `refine_inline`。

淘汰记录：similar（判据①⑤）；零依赖自研 histogram（工期数倍，
§10 Q-1 默认反向——用户未预裁定，按证据定案）。

## 管线（唯一管线纪律）

每个入口走同一条管线——串行/并行仅线程数不同（AC-05 等价=线程数不
变性，非算法切换）：

1. **行切分**：universal-newlines（`\r\n` 剥尾 `\r`；行尾孤立 `\r`
   同剥；尾空元素吸收）——下游 envelope 契约同语义。
2. **首现序 interning**：稠密确定性 token id（before 先编号、after
   续编；零哈希器状态→跨运行稳定）。
3. **patience 锚点分块**（中段 ≥512 行时）：两侧各恰出现一次的行值
   =强制 keep，切分中段为独立段——内容决定（同输入必同切分），并行
   调度不改变切分。**锚集经双坐标严格单调过滤后入场**（PLAN-704
   D-2）：a 位由 distinct first-occurrence 天然严格递增，b 位经
   patience LIS（O(n log n)）过滤——换位/移动族的块间序冲突锚弃用
   （如 620 行换位形三块锚集择一 300 锚），段构建的单调游走假设由
   此成立（缺陷史：未过滤时 cursor 倒退产生退化段，编辑脚本本身
   错误——620/0 双向纯增实证）。
4. **histogram per 段**：imara `Diff::compute_with(Histogram, …)`（自
   strip 公共前后缀）；段流按位拼接（段间 keep 隐式）。
5. **hunk 归组**（下游 ⑤ 同构）：非 keep 间距 ≤2·ctx 双坐标同满→归
   组；ctx 扩张钳位文件界；0 基半开 `[a1,a2)`。

## API 面

| 面 | 签名 | 语义 |
|---|---|---|
| `diff_lines` | `(&str, &str, DiffOpts) → DiffOut` | 文本行级 diff（串行） |
| `diff_lines_parallel` | 同上 | 同输出，段级有界 worker 池（std::thread::scope+split_at_mut 无锁分片） |
| `diff_lines_windowed` | `(&str, &str, opts, a_range, b_range) → DiffOut` | **窗投影**：全量 diff 过滤+裁剪到窗矩形；hunk 归属=任一侧与窗相交；adds/dels=坐标落窗内的变更数 |
| `diff_snapshots` | `(&RopeSnapshot, &RopeSnapshot, opts) → DiffOut` | T-01 prune 快路（subtree_equal O(1)）+**divergence 包络**（首尾 diverged 外包线）+包络内单次全局行 diff——与同内容文本路径对齐全等 |
| `refine_inline` | `(&str, &str) → Refinement{pre,post}` | 配对行公共前后缀字符区间（`a_mid/b_mid(char_len)`）；不成对行整行 mid=Q-4 形（envelope 构建侧决策） |

`DiffOut { hunks: Vec<Hunk{a1,a2,b1,b2}>, adds, dels }`——0 基半开。

### 窗投影语义（区间限定调用的定案）

histogram 对齐依赖**全文档行频计数**——切片重算与全量结果的窗内切片
**不保证逐字段等**。AC-04 硬门（限窗⊆全量切片全等，多窗位采样）唯一
保证成立的形=投影：底层全量（哈希剪枝+锚点并行加速）+hunks 窗过滤裁
剪。下游滚动联动的惰性收益由投影过滤达成（免全量 rows 重排）；真·局
部重算=增量重比（供料 §2 未来件），不在本件。

### 快照路径设计边界

prune 包络（非 per-region diffing）：区域局部直比曾实测与全文管线失
协（位置对齐剪枝在编辑点后整体移位场景下逐子树级联、区域直方图对齐
与全文档 pass 分叉——7/6 vs 2/1 实证）。包络收口保证 snapshot≡text
对齐（验证锚断言 change counts 全等）；散布多编辑点的包络内共享回收
=未来精化（perf-only，正确性不受损）。

## 基准（AC-05 相对量；绝对量归下游 L2）

复现：`cargo test --release -p auto-lang diff_bench -- --ignored
--nocapture`（2026-09-27 实测，release 档，全 suite 4.35s）。

| 形态 | 规模 | 串行 | 并行 | 产出 |
|---|---|---|---|---|
| 全等 | 100MB / 2.94M 行 | **354 ms** | 364 ms | 0 hunk（trim 快路） |
| 1% 散布改 | 100MB | **380 ms** | 384 ms | 17,773 hunks |
| 10% 散布改 | 100MB | **637 ms** | 886 ms | 177,730 变更行 |
| 100% 全换 | 10MB | 83 ms | 76 ms | 单 replace hunk |
| 10% 散布改 | 10MB | 29 ms | 28 ms | 1,778 hunks |
| 快照全等 | 99MB | — | — | **10.6 µs**（subtree_equal O(1)） |
| 快照单点编辑 | 99MB | — | — | **909 ms**（包络=编辑邻域，1 hunk） |

读数：

- **文本路径 100MB 全形态 0.35–0.9s**——上游已压进下游 §2.1「≤2s」预
  算线内（绝对量判定仍归下游 L2 bench，Q-2 口径）。
- **T-01 结构共享红利实证**：同内容对比走快照面 10.6µs vs 文本面
  354ms——3.3 万倍；单点编辑 909ms（包络=编辑邻域）。
- **并行收益形状依赖**：锚点稠密文件切出大量细段（17k+），worker 池
  分发开销吞掉并行收益（10% 形态并行反慢 40%）——`diff_lines` 缺省
  串行是安全缺省；并行收益存在于少而大的段形态（历史测量：修二次方
  前 10MB/10% 并行 235ms vs 串行 843ms）。段粒度自适应并行=后续精化。

### 执行期修复的二次方（基准驱动，值得记录）

初版 per-segment 直传全局 token 界——imara 按界分配 occurrence 表，全
局 id 稠密导致**后段单行段仍分配 2.9M 项表**（O(segments×tokens) 二次
方；1%/10% 100MB 形态实测 82-88s、总 suite 240s）。修复=段内局部重映
射（id 压缩到段内首现秩，O(段) 总量 O(n)，id 确定性保持）——上述表格
即修复后数字。教训入档：**分块调用第三方引擎时，token 界必须随块压缩**。

## rows 切片流位契约（PLAN-704 D-1）

`GroupedHunk.fc/lc`=changes 向量下标（分组器输出契约）；`build_rows`
按 keep+change 流下标切片——两套下标经 **per-change 流位映射
（`chg_stream`）单源换算**（建流时记录每个 change 的流位，切片
`lo/hi` 自映射推导）。切片语义=011 下游流位契约：逐 hunk 切片无
重复、变更行必在位、切片域不越 hunk 窗（rows 投影规范锚=下游过渡
参考实现，多 hunk/纯增/纯删/删多增少四族零漂移）。缺陷史：fc/lc
被直接当流下标消费——变更行整块丢失（纯增形 adds=3 而零 add 行）、
前导 ctx 重复（多 hunk 形 41 vs 21）、大文件 O(H²) 放大。

## 验证锚

`cargo test -p auto-lang diff`：八形态 golden（纯增/纯删/改/行移动/
空×2/单行大文件/全等，ctx=1 紧化逐字段）+双跑字节等+多 hunk 归组语
义断言+窗投影采样一致性+并行≡串行+snapshot≡文本计数+multibyte
refinement+dirs 11 项（见 diff-endpoints.md 联动）。
**PLAN-704 回归组**（`diff::` 模块 30 项内）：plan704_d2（换位形
锚集单调单元[三块择一 300 锚]+换位编辑脚本 310/310 双向对称+kept
记账不变量 `len−dels==len−adds`）+plan704_rows（多 hunk rows=下游
参考 21 行对照[含配对行三段标记样本]/删多增少 7 行逐行序列/纯增纯
删变更行存在性）+plan703_supply_probes 7 项（registry 直读面零
回归）。
