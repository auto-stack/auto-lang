# PLAN-787 验收矩阵与性能报告（T-07；复审 R1..R7 修复轮）

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

## 0b. 复审 R3 修复轮（2026-10-16，F787-R3-01..06 全闭合）

R3 独立反例 **8/8 通过**（修复前 1/7；R2 18 例与 R1 7 例保持通过）。要点：
投影改**迭代引擎**（显式工作栈，`projection_engine.rs`——平链 2200/真嵌套
511 层 if/800 层调用在**正常线程栈**投影成功，预算超限受控 TooDeep；valid
41/41 逐字节全等保持）；发布身份**会话域**（内存 `published_revision`——
迟到判定只在同一编辑会话内；同文本跨修订幂等复用（revision 移出内容工件）、
跨会话/重启 revision=0 可发布）；发布原子序**先注册后定点删除**（探针
`remove_root` 缝隙；注册失败旧当前根保留——trigger 注入反例）；MidToken
携带**原始起始行**（跨行未闭合 token 恢复后行号与 fresh 全等，A02 codec
同步）；越源 span 读取 admission（decl origin/span 上界 vs total_len）；
计量合口径（同驻留合计：parse 树+页 payload 同点采样；目录页 JSON/finish
瞬态计入；manifest **实读**字节分列；单查询增量读取 ≤512KiB 实测；目录页
2048 条）+ 查询 bin **按范围取源**（仅物化 span 覆盖页，子树 origin 平移
outline——无整源驻留/无整源 Snapshot）。

## 0c. 复审 R4 修复轮（2026-10-16，F787-R4-01..06 全闭合）

R4 独立反例 **3/3 通过**（修复前 0/3；R1 7 例、R2 18 例、R3 8 例全部保持
通过——键形保持历史稳定，四代探针植入件均真实命中）。要点：A03 持久化
**规范形**（`canonicalize`：页内容由树逻辑内容——根孩子文档序+先序 DFS+
相对跨度/字段/标记——决定，与内存块分配历史无关；同 snapshot 的
fresh/增量树页字节相同 → 同键 CAS 幂等复用，修复前页键含块序号且内容为
内存块原样编码、两种布局同键不同内容 → Conflict；不可达节点不入页，与
has_error_nodes 可达性语义一致）；**子树读取 admission 在真实入口验证整棵
可达子树**（页内声明归属一致/绝对跨度 ≤源长/非 trivia 孩子 ⊆父 span、
trivia 孩子右界 ⊆父 span/边字段在 grammar 字段名表内/kind 在声明面内；
decl_range 保持根级单点检查）；**范围查询坐标合同**（源覆盖窗口=声明单元
坐标域 [origin, span_hi)——origin 含前导 trivia 可位于语义 span 之前的源页；
origin 平移 checked_sub 受控退出；文本切片两端 token 边界，页边界切多字节
字符时先切再转码）；**计量口径补全**（受管=实际容量：Vec capacity×元素
大小+String/内层堆容量+同驻留合计——目录页落盘/页 put 期间 parse 产物、
payload framed 副本、finish 时 move 前目录与 JSON/edges；put 路径零克隆；
OS=各阶段期间采样器峰值：build/warm 查询/冷恢复/源重建分列+生成器进程
分列+缺失项显式记录——parse 内部工作栈仅 OS 可见、非 Windows sysinfo=0
整体缺口）；**真实内存削减**（封块 shrink_to_fit+token 流收紧：单 64KiB
块 parse 瞬态容量口径 5.88MiB→3.0MiB；诚实计量下 builder 受管峰值
8827→8027KiB 仍 ≤8MiB 合同，不放宽预算）；spec-delta 重写（会话域仲裁/
DECL_INDEX_PAGE=2048/规范形/admission/预算证据边界）；rustfmt 全量净。

## 0d. 复审 R6 修复轮（2026-10-16，F787-R6-01..03 全闭合）

R6 独立反例 **4/4 通过**（修复前 2/4：8MiB 名称窗口/put 峰值下界断言红；
R1 7、R2 18、R3 8、R4 3、R5 4 例全部保持通过——六代全绿）。要点：
**上游租约缺陷按归属修复**（F787-R6-03/P1：783 `acquire_lease` 的
`ls-{ms}-{pid}` 在已释放行残留时同毫秒顺序申请撞 leases 主键，128 连发
91 次 UNIQUE 冲突——经独立 fix worktree `fix-783-lease-id` 以 `LEASE_SEQ`
进程内序列修复【staging 事务 ID 既有的 STAGING_SEQ 同类机制；liveness
语义不变：OS 锁文件裁定】+ 永久回归
`reachability_sequential_same_millisecond_leases_do_not_collide`，落
master `7c0caa18c` 后回绑本分支重验；787 侧未私修、未 sleep 遮红）；
**源页 put 三驻留计量**（F787-R6-01：聚合 buf + payload 克隆 + 探针 framed
副本三份同驻留，put_peak = capacity + 2×len + 64——修复前两份口径在
256KiB 页少计 262192B）；**查询窗口有界物化**（F787-R6-02：限额在
**append 之前**判定——窗口已越限即受控 exit 4，驻留上界 = limit + 一个
store 源页 ≤256KiB，不再先整窗物化再检查；8MiB 名称/默认 2MiB limit 实测
物化恰为 2MiB+256KiB 边界内）；**work-owned 64MiB 必验在修复提交上完成**
（R4 轮曾误列为 merge-owned pending——本计划 §6/验收脚本明确
syntax_budget_ 为 work-owned，已更正；见 §2 新口径读数）；兼容审计基面
随 master 回绑重锚 7c0caa18c（断言语义不变：787 自身改动相对基线零
生产面）。merge-owned 门禁清单更正：final pass 后 = 完整独立语法
workspace + 受影响 783 套件 + SD/索引/ledger 沉淀 + 合流归档/guard 清理
（本计划无生产 Rust/文档生成器触面，不含根整档/docs_gen）。

## 0e. 复审 R7 修复轮（2026-10-16，F787-R7-01 闭合）

R7 独立反例 **1/1 通过**（修复前 0/1；R1–R6 共 44 例保持全绿）。要点：
**kind 表统一**——store 侧 `ChunkKind::from_code` 曾本地复制枚举表并漏
`SourcePage=10`，`get_artifact`/`get_artifact_by_id` 在读元数据时即
`Internal("bad kind")`（合法落盘工件在公开产物读入口不可读；CstView 的
raw chunk 路径不受影响）；修复为**委派 codec 帧层的唯一表**
（`from_u8` 提为 pub(crate)，单一事实源——两表不再漂移；未知字节仍
None 拒绝，边界不变）。永久正控
`syntax_cache_source_page_artifact_reads_by_key_and_id`：真实
`put_source_pages` → 新 store 实例重开 → 按 page-hash key 与按 artifact
id 均可读（kind=SourcePage、payload 全等；附带 put_peak 三份口径下界
自检）。probe 拒绝边界（--lib/admission_/dep_audit_/identity_）全绿。

## 1. V 矩阵（acceptance 实测 2026-10-16 R7 修复轮）

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
| V-09 | 持久化/重启小查询 | syntax_cache_ | 10（**SourcePage 公开产物读入口按 key/按 id 重开可读（R7-01）**、跨 store 实例、checkpoint 完整往返+**恢复对照 fresh 尾**、双工程共享隔离、真子进程查询精确断言+负控、**同 snapshot fresh/增量入库幂等共存（正反序/重复编辑/重开）**、**跨页前导 trivia 查询（空白/CRLF/Unicode 注释/多页/非首声明）**、**name 窗口限额受控 exit4**、**窗口限额驻留上界（真实 512KiB 名+64KiB limit，自报物化 ≤limit+256KiB）**） | PASS |
| V-10 | 数据拒绝 | syntax_admission_ | 18（坏块/缺页/坏身份/截断页/坏 manifest ref/根替换+lease+GC 生命周期/**同文本跨修订复用**/**重启会话发布**/**注册失败保旧根**/**越源 span 拒绝**/**子树 admission：越源 child/越父 span/异归属/坏字段拒绝+跨页非零 origin 正控**/**空 args 零宽生产者正控**等均受控） | PASS |
| V-11 | 64MiB 工作集 | syntax_budget_ | 2（**work-owned，R6 修复提交上实测**；见 §2 大负载读数；含巨声明受控 ResourceExhausted） | PASS |
| V-12 | 深度全路径 | depth_ / depth_persisted_ | 4 + 1（**真 ≥4096 同时未闭合嵌套**；持久化编解码往返全路径迭代） | PASS |
| V-13 | 兼容/证据 | compatibility_ / perf_ | 3 + 2（含 core 旧 parser 对照；**审计基面随 master 回绑重锚 7c0caa18c**） | PASS |

复审反例重放：R1 **7/7** + R2 **18/18** + R3 **8/8** + R4 **3/3** +
R5 **4/4** + R6 **4/4** + R7 **1/1** 全部通过（R7 修复前 0/1；四代探针植入件在键形
稳定下真实命中，无空过——R4 修复中曾短暂引入 A03 键布局分量致 R2 两例
miss，已回退保持键形历史稳定）。
acceptance 16 族 raw passed 合计 **112**（snapshot 4/grammar 13/reject 7/
lossless 15/recovery 4/projection 19/local 3/replay 4/generation 3/depth 4/
persisted 1/cache 10/admission 18/compat 3/perf 2/budget 2；过滤族有重叠，
非独立测试去重总数）。

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
- V-11 大负载（**R6 修复轮实测**——F787-R6-01 要求 work-owned 证据在修复
  提交上完成）：64MiB / 2,567,818 声明，**生成器独立进程**
  （ac787-gen，自身 private bytes 峰值 948KiB 分列）：
  - 流式构建 361.4s（页 put 主导，probe 每次提交全目录扫描为放大项——783
    边界，不在本计划修）；200 次随机子树查询 0.7s；
  - **builder 受管工作集峰值 8027KiB ≤ 8MiB**（F787-R4-04/R5-03/R6-01 诚实
    口径：实际容量（Vec capacity×元素+String 堆容量+R5 起返回树/源码页
    计入）+同驻留合计——parse 产物在目录页落盘/页 put 期间存活、**源页 put
    三份驻留（聚合 buf+payload 克隆+探针 framed 副本）全计**、finish 时
    move 前目录与 JSON/edges 同驻留；put 路径零克隆；封块 shrink_to_fit+
    token 收紧使单 64KiB 块 parse 瞬态容量口径 5.88→3.0MiB，诚实计量下
    峰值 8827→8027KiB 仍满足合同，不放宽预算）；
  - hot 页缓存 8092KiB ≤ 8MiB；manifest 目录 1162KiB；**受管合计 ≤16MiB**（索引计入）；
  - 单子树 payload ≤512KiB、单页解码 ≤512KiB；
  - **OS private bytes（Windows 实测，各阶段期间采样器峰值——R4-04/R6 口径）：
    build 15,352KiB / warm 查询 14,628KiB / 冷恢复查询 17,176KiB / 源重建
    15,232KiB / 终态 15,192KiB，runtime 增量峰值 17,176KiB ≤ 64MiB 目标**；
    非 Windows 宿主 sysinfo 返回 0 → 测试显式记录证据缺口，不作断言；
    **缺失项显式记录**（parse Machine 内部工作栈/回溯暂存仅 OS 采样器
    可见，不在受管口径内）；
  - 视图单次 chunk 读取峰值（数据/目录/源页统一）≤512KiB；**单次恢复查询
    增量读取（manifest 实读分列后）≤512KiB**；
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

- 修复轮提交见 git log（R1：F787-R1-01..10；R2：F787-R2-01..11；R3：
  F787-R3-01..06；R4：F787-R4-01..06；R5：F787-R5-01..03；R6：
  F787-R6-01..03；R7：F787-R7-01 全闭合）。
- 冻结件：feasibility.md（T-01）；本文件（T-07 修复轮）；spec-delta 见
  attachments/787/spec-delta.md（按修复后实现更新）。

## 5. 修复轮边界声明（不冒充完成）

- OS private bytes：Windows 宿主实测并断言；非 Windows sysinfo=0 → 测试内
  显式打印证据缺口行（不以自定义页数冒充 OS 证据）。
- put 提交成本随块数超线性增长（probe staging 预算锁的全目录扫描）——
  783 既定边界，源页按 256KiB 聚合已缓解（4096→256 次 put）；大负载构建
  204.7s 主要消耗于此，非语法运行时本身。
- debug 构建下增量编辑时延不优于 fresh（见 §2）——时延优势不在本轮主张面。
