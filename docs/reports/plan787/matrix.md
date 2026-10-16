# PLAN-787 验收矩阵与性能报告（T-07；复审 R1+R2 修复轮）

- 工作树 `D:/autostack/.wt/lang-787/auto-lang`，分支 `plan-787-dev`
- 基面 master `bf773ca9b`；本轮修复绑定提交见 §4
- 入口：`pwsh scripts/verify-ac-syntax-787.ps1 -Scope acceptance`
  （**acceptance 即含 work-owned 全矩阵含 perf_/syntax_budget_**——F787-R1-09
  修复：不再把 work-owned 族后置 closeout；closeout 另跑完整工作区 + 783 触面）

## 0. 复审 R2 修复轮（2026-10-16，F787-R2-01..11 全闭合）

R2 独立反例 **18/18 通过**（修复前 7 pass/11 fail；R1 七例保持通过）。
R2 修复要点：可达性错误语义（has_error_nodes 只看当前根可达——共享块旧
错误残留不污染修复树）；put_cst **发布仲裁**（manifest 携带 revision，迟到
旧任务入库不发布，不替换当前持久化根）；流式页键含当前块 hash（第二份
文档共享 store 不冲突）；流式 required 闭包补**源页+grammar** 边（GC 后
源可重建/grammar 可装载）；admission 下界先判后算/页环拒绝/流身份强校验；
A02 checkpoint **按末 token 拼写重放**还原真实 mid（嵌套深度/转义挂起）
+ 恢复对照 fresh 尾部全等；TreeSession 树 id 全局唯一（跨会话 handle 拒绝）；
投影固定 2048 深度上限改节点访问预算（合法深链/嵌套/块投影通过）；目录页
4096 条（framed≈250KiB≤512KiB）+ 视图全 chunk 读取峰值计量 + 暂存计入
受管峰值（4.8MiB≤8MiB）+ OS 采样覆盖查询/重建（增量 15.6MiB≤64MiB）；
canonical 含边 field、诊断全字段比较、replay 链上轮增量工件、子进程受截止
（复用探针 run_deadlined）、跨进程查询精确 span/name/hash 断言+负控。

## 1. V 矩阵（acceptance 实测 2026-10-16 R2 修复轮）

| ID | AC | filter | 实测 | 状态 |
|---|---|---|---|---|
| V-01 | 坐标/快照/编辑/代际 | snapshot_ / generation_ | 4 + 3（含活快照页保活/释放回收、跨页续行行列） | PASS |
| V-02 | Atom 双 profile 配置驱动 | grammar_ | 13 | PASS |
| V-03 | 生成期拒绝 | grammar_reject_ | 7 | PASS |
| V-04 | 无损/恢复 | lossless_ / recovery_ | 15 + 4 | PASS |
| V-05 | core 投影/strict | core_projection_ | 19（valid 41/41 全等 + invalid 85/85 平行 + **深链/深嵌套/深块投影无固定上限**） | PASS |
| V-06 | 增量局部 | incremental_local_ | 3（1MiB：真实读取 ≤48KiB+报告 214B、重解析 ≤1 声明、**内容块节点复用 1147999/1148131=99.99%**、fresh 全等；开头插入零后缀复制） | PASS |
| V-07 | 增量状态/回退可见 | incremental_replay_ | 4（core+mini 各 200 轮确定种子；未闭合回退=fresh；逆序屏障） | PASS |
| V-08 | 代际/迟到/取消 | generation_（+handle 单测） | 3 | PASS |
| V-09 | 持久化/重启小查询 | syntax_cache_ | 5（跨 store 实例、checkpoint 完整往返+**恢复对照 fresh 尾**、双工程共享隔离、真子进程查询精确断言+负控） | PASS |
| V-10 | 数据拒绝 | syntax_admission_ | 6（坏块/缺页/坏身份/截断页/**有效 frame 坏 manifest ref**/**根替换+lease+GC 生命周期**均受控） | PASS |
| V-11 | 64MiB 工作集 | syntax_budget_ | 2（见 §2 大负载读数；含巨声明受控 ResourceExhausted） | PASS |
| V-12 | 深度全路径 | depth_ / depth_persisted_ | 4 + 1（**真 ≥4096 同时未闭合嵌套**；持久化编解码往返全路径迭代） | PASS |
| V-13 | 兼容/证据 | compatibility_ / perf_ | 3 + 2（含 core 旧 parser 对照） | PASS |

复审反例重放：R1 `counterexamples` **7/7** + R2 `counterexamples` **18/18**
通过（R2 修复前 7/11 fail）。

## 2. 性能读数（perf_，32 样本/场景 + 3 预热，debug 构建，单线程）

- 机器：开发机（win32 10.0.26200 x64）。
- 文档 264KiB（8000 声明 mini）：
  - fresh parse：P50 199.0ms / P95 207.4ms / P99 214.7ms
  - 局部 edit 增量路径：P50 227.3ms / P95 239.0ms / P99 250.0ms；
    **同轮计时的 fresh_after_edit：P50 199.9ms**（F787-R1-09：报告所声称的
    fresh 对照在计时区内实测）；纯增量重扫 ≤214B（真实读路径计量）。
    debug 构建下增量不快于 fresh（嫁接目录开销 vs 均为 debug 慢的解析）——
    本轮不据此宣称增量时延优势；收益证据以重扫字节/重解析声明数/内容块
    复用率为准（V-06）。
  - 回退（未闭合字符串，**断言确实回退**，F787-R1-09）：P50 0.0ms / P95 0.1ms。
  - 重启小查询（put+open+子树重建，含 store 打开与 SQLite 事务）：P50 63.0ms / P95 94.2ms / P99 126.2ms。
- core 对照（F787-R1-09 新增）：361KiB core 文档，CST fresh parse
  P50 441.7ms vs 旧手写 parser P50 78.7ms（debug 构建，对照证据——新路径
  不替换生产入口，时延优势不是本计划交付主张）。
- V-11 大负载：64MiB / 2,567,818 声明，**生成器独立进程**（ac787-gen，自身
  private bytes 峰值 956KiB 分列）：
  - 流式构建 204.7s（页 put 主导，probe 每次提交全目录扫描为放大项——783
    边界，不在本计划修）；200 次随机子树查询 17.2s；
  - **builder 受管工作集峰值 4089KiB ≤ 8MiB**（声明目录在线分页落盘，
    F787-R1-08：不累积百万级目录）；
  - hot 页缓存 8092KiB ≤ 8MiB；manifest 目录 1064KiB；**受管合计 ≤16MiB**（索引计入）；
  - 单子树 payload ≤512KiB、单页解码 ≤512KiB；
  - **OS private bytes（Windows 实测，采样覆盖构建/查询/源重建全阶段）：
    runtime 增量 15,972KiB ≤ 64MiB 目标**；非 Windows 宿主 sysinfo 返回 0 →
    测试显式记录证据缺口，不作断言；
  - 受管峰值含暂存（parse 树/页 payload/目录页 JSON，R2-09 口径）：
    4811KiB ≤ 8MiB；视图单次 chunk 读取峰值（数据/目录/源页统一）≤512KiB；
  - 磁盘 316MiB（源页+树页 CAS）；源重建逐页比对与生成文件全等（64MiB 零整源驻留）。
- **范围声明**：以上为 mini/core 两个受限 profile 的探针负载读数，不得推断
  任意语言或全仓亚秒（计划 §5.6）；debug 构建数字供相对比较。

## 3. 兼容与依赖审计

- `compatibility_dependency_audit_no_production_surface_touched`：git diff
  `bf773ca9b..HEAD -- crates/ experimental/ac-core/src/` 为空（零生产面改动；
  shim-metadata 预存警告归属不变）。
- 783 探针触面（加法扩展：ChunkKind::SourcePage=10、A02 key_inputs 补
  grammar-id/page-hash）：`--lib` 14/14、`admission_` 7/7、`dep_audit_` 5/5
  （codec 面期望随 A01/A02/A03 真实生产者翻转更新——对应测试更新授权内）、
  `identity_` 4/4、`concurrency_` 3/3、review_r1/r3/r4/r5 21/21。
- ac-core 公开 API 原样工作；生产 parser 零切换（投影仅对照用）。

## 4. 提交绑定

- 修复轮提交见 git log（R1：F787-R1-01..10；R2：F787-R2-01..11 全闭合）。
- 冻结件：feasibility.md（T-01）；本文件（T-07 修复轮）；spec-delta 见
  attachments/787/spec-delta.md（按修复后实现更新）。

## 5. 修复轮边界声明（不冒充完成）

- OS private bytes：Windows 宿主实测并断言；非 Windows sysinfo=0 → 测试内
  显式打印证据缺口行（不以自定义页数冒充 OS 证据）。
- put 提交成本随块数超线性增长（probe staging 预算锁的全目录扫描）——
  783 既定边界，源页按 256KiB 聚合已缓解（4096→256 次 put）；大负载构建
  204.7s 主要消耗于此，非语法运行时本身。
- debug 构建下增量编辑时延不优于 fresh（见 §2）——时延优势不在本轮主张面。
