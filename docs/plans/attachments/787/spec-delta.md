# PLAN-787 拟议 Spec 增量（T-07 修复轮冻结稿，merge 时落 canonical）

对照计划 §"规范增量"表（SD-01..06）。本文件为 merge 输入；新模块名称为本仓
实验命名，不决定未来仓/产品品牌。**复审 R1+R2+R3+R4 修复轮**：按
F787-R1-01..10 + R2-01..11 + R3-01..06 + R4-01..06 修复后的实际实现重写
（替代历次审查冻结旧稿——旧稿未获通过）。

## SD-01 add docs/specs/auto-syntax/project.md

- **before**：无语法服务模块 Spec（783 的 A01/A02 为 Future、A03 为合成 ProbeSeam）。
- **after**：有限双 profile（auto-core / interface-mini-v1）语法服务：依赖方向
  snapshot→grammar-program/scanner→CST→结构查询；运行时不依赖 ac-core/
  Cranelift/auto-cache；Atom 配置为唯一规则源（编译期拒绝左递归/可空 repeat/
  未知引用/非前进 scanner）；实验边界（非完整 Auto 版 tree-sitter、非任意 CFG/
  regex、非动态插件、非 LSP）。
- **acceptance**：AC-01–06（V-01..V-13 全矩阵）。
- **证据**：verify 脚本 acceptance ALL PASS（R4 修复轮 16 族 raw passed 合计
  106——snapshot 4/grammar 13/reject 7/lossless 15/recovery 4/projection 19/
  local 3/replay 4/generation 3/depth 4/persisted 1/cache 7/admission 15/
  compat 3/perf 2/budget 2；过滤族有重叠，非独立测试去重总数；计数以实际
  运行日志求和为准，不据模板或历史轮数字认 pass）；matrix.md；四轮复审
  反例重放 7+18+8+3 全通过。

## SD-02 add docs/specs/auto-syntax/grammar-contract.md

- **before**：无规则程序合同。
- **after**：grammar-contract v1 DSL（token matcher 五类；步骤/repeat/field/
  same-line；Pratt 表+调用配置；newline/sep/sync 旋钮）；生成期拒绝定位到节点
  路径；GrammarId=名称+canonical+四版本 hash；同名不同内容隔离；codec 往返
  身份绑定；关键字/优先级配置反控（V-02）。
- **acceptance**：AC-02（V-02/V-03）。

## SD-03 add docs/specs/auto-syntax/snapshot-cst.md

- **after**：16KiB 内容寻址页+长度前缀目录（头部插入后缀页 hash 复用）；**页
  由快照目录 `Rc` 真实保活——活快照压力下不丢页，快照释放后页可回收**；
  UTF-8 边界拒绝+UTF-16 代理中点 floor+CRLF 行内策略；分页行索引（**跨页
  续行行起点回溯——列号从真实行起点计**）；树会话/lease/跨代拒绝；**CST =
  不可变共享内容块（CstBlock）+长度目录（bases）+根孩子目录**——节点 POD、
  块内边为局部下标、块跨树 Rc 共享；声明单元相对跨度+块内声明序号归属；
  无损重建断言；恢复=吃进到 sync∪first（≥1 token 前进或终止）；增量=at-rest
  checkpoint（词法粘连安全边界）+8-token 流式收敛（**scanner 硬限窗口、收敛
  即停——成功路径真实读取有界，重扫计量=实际扫描位置**）+**嫁接=旧块整体
  共享采纳（零节点复制；前缀身份映射）**；迟到/取消版本化发布；预算（显式
  栈/work_bytes 256MiB 默认）。
- **acceptance**：AC-01/03/04（V-01/V-04/V-06..V-08/V-12）。
- **修复轮证据**：反例 7/7；1MiB 中部 body 编辑内容块节点复用
  1147999/1148131（99.99%）、新节点 132、真实源页读取 ≤48KiB；R4 起源恢复
  与子树归属由 SD-05 的 admission/查询坐标 oracle 覆盖
  （syntax_admission_ subtree 族 + syntax_cache_ trivia 查询族）。

## SD-04 modify docs/specs/auto-ac/project.md

- **before**：783 沉淀的探针导航（A00–A12，A01/A02 Future、A03 合成）。
- **after**：补 experimental/ac-syntax 导航节（双 profile 语法实验、core 投影
  adapter 仅对照不替换编译入口、生产 parser 零切换）；链接 auto-syntax 三份
  新 Spec。
- **acceptance**：AC-06。

## SD-05 modify docs/specs/auto-ac/artifact-contract.md

- **before**：A01/A02 Future、A03 ProbeSeam（合成造数）。
- **after**：A01/A02/A03 = Codec 且**真实生产者**（ac-syntax-artifact-adapter）：
  A01 语法 canonical 二进制（装载身份重验；可按（grammar-id, revision）从
  store 装载——跨进程恢复不编译 .at）；A02 token 流+**完整 scanner checkpoint**
  ——终态按末 token 拼写**重放**还原（嵌套注释真实深度/字符串转义挂起/
  mode），恢复续扫与 fresh 词法尾部全等；键含 grammar 身份（跨 grammar 真读
  拒绝）；**A02 --required--> A01+源页**。源字节以**内容寻址 SourcePage**
  持久化（快照 16KiB 页聚合为 ≤256KiB store 源页；跨快照/工程共享），manifest
  记源页表可流式重建原文。A03 页=**持久化规范形**（规范形 v1：页内容由
  树逻辑内容——根孩子文档序+先序 DFS+相对跨度/字段/标记——决定，
  与内存块分配历史无关；同一 snapshot 的 fresh/增量树页字节相同，同键 CAS
  幂等复用；键形保持历史稳定，未来布局算法演进时必须为 A03 键引入布局
  版本分量隔离新旧页；不可达节点不入页）+目录分页（**2048 条/页**，framed ≤512KiB）
  +按需页视图（重启小查询禁调 parser；视图 lease）+**子树读取 admission 在
  真实入口验证整棵可达子树**（页内声明归属一致/绝对跨度≤源长/非 trivia 孩子
  ⊆父 span、trivia 孩子右界⊆父 span/边字段在 grammar 字段名表内/kind 在声明
  面内；decl_range 保持根级单点检查）；**零宽坐标合同（F787-R5-01）**：空产生式
  （epsilon，如空 args/params）由生产者放在**已消费 token 终点**上（零宽但
  start 为真实坐标），与 MISSING 占位一致走父子约束；伪造 start=0 占位形
  零宽 child 在父外仍拒绝——合法 epsilon 与坏跨度可区分。**范围查询坐标合同
  （F787-R5-02）**：源覆盖窗口=outline name 字段绝对范围（语义 span 内，
  不含前导 trivia），按真实 token 范围取源；窗口驻留超限 → 受控
  ResourceExhausted（AC787_QUERY_WINDOW_LIMIT 可测阈值两侧），绝不用空 name
  冒充精确恢复成功；**限额在物化之前判定（F787-R6-02）**——窗口已越限即
  受控退出，查询进程驻留上界 = limit + 一个 store 源页（≤256KiB 超前），
  不先整窗物化再检查。required 闭包
  （当前根 GC 存活覆盖可重开全数据）：put_cst 链 = manifest
  --required--> 数据页+目录页+A02（→A01+源页）；流式链 = manifest
  --required--> 数据页+目录页+**源页+A01**（流式无 A02 工件——直接闭包，
  不以 A03→A02 链解释流式保活）。**发布仲裁（会话域）**：修订比较的适用域=
  单一编辑会话——`published_revision` 仅存 store 会话内存、**不进内容工件**
  （同文本跨修订幂等复用；跨会话/重启的 revision=0 是新文档发布，不是迟到
  任务）；原子序=先注册新根、成功后定点删除被替换旧根行——注册失败（探针
  trigger 注入等）旧当前根完好保留。新 ChunkKind ×5（加法：SyntaxGrammar/
  TokenStream/CstPage/CstManifest/SourcePage；旧 kind 字节与拒绝面不变；GC
  算法零改动；**五 kind 的公开产物读入口（按 key/按 id）全接入**——kind
  解码以 codec 帧层为单一事实源，store 元数据层委派之，无重复表漂移
  （F787-R7-01：修复前 store 本地表漏 SourcePage=10，合法工件读入口报
  bad kind）。
- **acceptance**：AC-05（V-09..V-11）。

## SD-06 modify docs/specs/auto-ac/artifact-probe.md

- **before**：探针边界（冻结预算/协议）以合成 A03 为准。
- **after**：记录 787 加法扩展（五 ChunkKind 含 SourcePage、三层 coverage 翻转
  （dep_audit 期望同步）、A02 key_inputs 补 grammar-id/page-hash、Future 拒绝面
  换 A05 起）；GC 边界、staging 8MiB、64/48MiB 水位等冻结值不变；787 的
  syntax_budget_ 在同预算合同下运行（**work-owned，在修复提交上实测**，
  F787-R6-01），**证据口径（F787-R4-04 + R5-03 + R6-01）**：builder
  受管 ≤8MiB = 实际容量（Vec capacity×元素大小+String/内层堆容量）+ 同驻留
  合计（parse 产物在目录页落盘/页 put 期间仍存活；payload 克隆+探针 framed
  副本**各计一份**；finish 时 move 前目录与 JSON/edges 同驻留）。返回树驻留项逐项计入：
  `blocks`/`root_edges`/`decls`/`bases`/`block_decl` Vec 外层 + 各 `CstBlock`
  的 nodes/edges/fields capacity + `tree.diags` 及其 message String capacity
  + tokens/lex_diags；push_chunk 期间 snap/store 源码页 payload（活到返回）
  与 put_source_pages **三份**峰值（聚合 buf 保留+payload 克隆+探针 framed
  副本，F787-R6-01：修复前两份口径在 256KiB 页少计 262192B）一并进入同驻留
  高水位。
  视图受管 = hot 页缓存 ≤8MiB +
  manifest 目录计入 ≤16MiB 合同；OS private bytes = 各阶段（build/warm 查询/
  冷恢复查询/源重建）**期间采样器**的最大观测增量（生成器独立进程分列），非返回点
  当前值；单查询增量读取（目录页+数据页，manifest 一次性入口成本分列）≤512KiB。
  **明确排除项**：parse Machine 内部工作栈/回溯暂存在 parse_snapshot 返回前
  释放——不在受管口径内、仅 OS 采样器可见；非 Windows 宿主 sysinfo=0 时 OS
  证据整体缺口（显式记录，不冒充通过）。原预算合同（staging 8MiB、64/48MiB
  水位）不变——本次仅补齐漏计项，不改写合同线。
- **acceptance**：AC-05/06。
