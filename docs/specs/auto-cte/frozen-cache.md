# auto-cte 冻结缓存与资源合同

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


> Status: active (plan-817)；实现 `probe/src/{store,driver,observe}.rs` +
> 783 追加式有界读取 API。

## envelope 与信任

- `ac817/cte-executed/v1` envelope（producer=`plan817/cte-probe` 绑定**真实 AV 成功
  结果**；模型 compute/伪 vm 标记/单写 marking 均不能铸造执行）；自有层
  `ac817/a05-cte-exec/v1`（LayoutData 载体、783 统一 store、内容寻址 CAS，不复制
  796 store 数据库）。
- 全字段恢复准入+新鲜性：闭包 body/签名、meta 探针/事实（负依赖）、caps、engine
  三元版本、schema/producer/engine；同名异包、好 frame 错地址/owner/key、旧嵌入
  类型、foreign schema=**受控拒绝（非 miss）**；源码不变但 helper/字段集/权限变化
  仍失效（依赖 manifest 先验证完整新鲜）；hit `ct_execute=0` 且 consumer 逐次重
  准入/比较（hit 不缓存 consumer 裁决）；跨进程/跨工程命中不执行 AV（禁调反控）；
  trap/取消步骤零发布、失败保旧条目。
- 生命周期：lease 保活跨 GC、释放后按 783 水位收敛（oldest-first）、双工程 soft
  root 独立、用户哨兵逐轮 GC 字节不变。

## 预算与顺序（分配前预留）

- 受管三分区 hot 8 / 暂存 16 / AV 槽 8 = ≤32MiB；预算申请先于 clone/decode/编码；
  运行时采样含瞬态峰值；超标不填 target_met=true。
- **envelope 编码**：零分配计数 writer → 精确字节数 → 预留 Staging →
  `with_capacity(count)`+`to_writer`（容量==授权额，无增长链）。
- **恢复四段有序**：①零分配大小探测（经公共 `ManagedStore::db()` 只读 SQL
  `chunks.size`）在读取/重组上限处先行拒绝超界载体（加载器零调用）；②载入预留=
  loader 真实瞬态足迹（framed Vec+unframed 拷贝；frame 读取额与 payload 副本额
  **分账**，互不复用；783 `read_chunk_gated_bounded` 物理读取=metadata 预检+精确
  容量+`read_exact`，损坏/替换文件零内容分配拒）；③加载长度须与探测一致（内容寻址
  漂移=异质写窗口受控 infra 拒；probe SQL 错误传播，不吞成 miss）；④decode 额度=
  **零分配结构审计**。
- **decode 审计模型**（S+2T）：字符串内容字节总和 S（全部 owned 副本同时存活于
  解码结果，decoded≤encoded）+ 2×最大单 token 内容 T（serde_json 1.0.154
  `Deserializer::scratch` 为全文档共享单 Vec、逐 token clear 保容量、Vec 倍增封顶
  2×单 token；**转义字符串**与**超长数字字面量**同为 scratch token，非转义串走
  Borrowed 零拷贝）+ 数组元素×最宽槽位（Vec<(String,String,String)>=3×24B）×Vec
  倍增系数 + 固定项——每因子锚定 serde 实际代码路径与 Rust Vec 增长，非载荷倍数；
  稳态最坏（全部 owned+满 scratch）=授权值，realloc 瞬态亦在界内；非法/超深 JSON
  在审计处拒（from_slice 前置）。
- worker wire 读入有界（6MiB）且 base64 预检先于解码；预算事件日志
  （Reserve/Alloc/Release 有序）使"拒绝先于大分配"可观测。

## 资源硬门与周转

- worker private bytes 峰值 ≤384MiB、进程树同时存活采样归集 ≤512MiB（0/缺失采样
  =不可用即红）；规模：流式 ≥32MiB 真实冻结页 + 512 不同输入修订 + 真实水位 GC
  物理删除/收敛；spike/缩分区反控确定性真红。
- warm session 局部 CT helper body 编辑与 Meta 集合编辑的端到端
  （准备/序列化→IPC→新鲜性/执行或命中→冻结/consumer/HIR verify→父侧准入）单墙钟
  P90≤2000ms（正式 release、规定负载 115 函数 ≥256KiB）；raw 逐样本绑定当轮 hash/
  值/PID；阈值 0/坏样本/注入 spike/退化反控必红。


## 源 CT 冻结层（PLAN-936，SD-03）

- 新执行身份 `ac936/ct-exec/v1`/producer `plan936/source-cte`/层 `S936`：无
  source 级指纹——执行键=callee logical+签名/体版本+**传递闭包版本**+canonical
  实参（或替换后表达式 canonical 哈希）+engine/lane；整模块 hash、调用点
  span、provenance revision 不入键。UseBinding（owner/site/期望）随请求走，
  不入缓存页。
- **checked-child 传递版本**：内层 CT 值翻转（或模板体编辑）改变替换后实例体
  的结构版本→外层键变（源码 hash-only 缓存必误 hit）；迟到/预算耗尽=资源态
  零发布；sink 写路径准入拒绝自家 codec 会拒的页（外来 schema/producer）。
- 恢复：跨上下文同键 hit 零 AV（R 窗计量 probes/bytes/decode 真实读计数）；
  缺页=真 miss 真执行（不假装恢复）；首坏 CT 受控拒后修复编辑真执行真发布。
- 共享预算：单根累计 10s、单 AV min(2s,remaining) 不重置、查询环按 (owner,span)
  节点身份具名拒、普通递归非环（AV fuel）；envelope 页界 512KiB、单串界 64KiB
  （完整闭包 canonical）。原 817 层/预算/生命周期规则不变。


## 有界成本诊断（PLAN-1016）

`ace-comptime::conformance` 的显式 ignored 测试
`cte_resources_tests::resources_short_cost_probe` 复用原 `gc_deck` 生成器：
默认保留300个helper、Closure lane、真实AV/冻结/发布与逐值闭式oracle，
仅缩至2 calls × 2 batches。每批还在 forbid_av 下验证同输入hit且ct_execute=0。
小规模GC调用同一collector，诊断水位设置为当前DB/locks/logs底板之上，
验证物理删除；不声称覆盖正式64MiB水位收敛、lease和sentinel全合同。
原32MiB/512修订与两轮64MiB测试规模、断言及正式性能门保持。

运行前显式设置 `CTE_COST_PROBE_ROOT` 到本计划的 plan-artifacts 目录；
可选 `CTE_COST_HELPERS`/`CTE_COST_CALLS`/`CTE_COST_BATCHES` 调整单一维度。
先用 `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-comptime
--test conformance --no-run` 完成构建，构建时间单列；再仅执行上述完整测试名
（`--exact --ignored --nocapture`），诊断外层进程上限60秒，超界先查阶段日志/减工作量。
默认探针约1秒；300 helpers × 50 calls × 4 batches 的本机修后样本约16秒，
这些是观测值，不是跨机器承诺或正式门替代。

观测 `av_frontend_{parse,resolve,typecheck,emit,bind,verify}`、
`av_lower_verify`、`av_vm_run`、`freeze_publish` 与单独GC时间；父级span包含子级，
不得把嵌套span相加充总时间。绑定span包含源码位置映射。共享执行尾的
`run_entry_on_av_observed` 是原准入/执行路径上的被动阶段回调；原入口仍委托同一实现。
源编译/Closure每调用冷编译依然存在，不把文件系统优化描述成编译复用。

存储分离诊断：`ace-artifacts` lib ignored
`store::cost_probe::short_store_scan_cost`，同样要求显式scratch根。
固定128/512/2048个blob与锁文件、37字节临时残留，分别测space与残留扫描，
并断言字节/文件数。`BlobStore`普通文件类型及managed目录/残留类型读取使用
DirEntry枚举元数据，避免Windows逐文件重复打开；原blob符号链接分支、managed目录
不跟随链接、临时残留配额、命名空间锁覆盖及物理计量规则保持。
文件枚举随存量增长仍有成本，未引入跨进程缓存计数或改写预算协议。


## 日常测试选择（PLAN-1016 r4）

`cargo t ace-comptime`排除正式性能/压力；原32MiB/512修订、两轮64MiB GC、周转campaign以及
`cte_snapshot_tests::performance::cte_snapshot_resident_lane_baseline_30_revisions`
仅通过显式`cargo t ace-perf`运行。原采样数/水位/门限不变。
日常保留`cte_snapshot_tests::cte_snapshot_resident_lane_small_revisions`，
以8函数/3次真实修订检查值、generation与未改函数全阶段零工作，不承担正式P90验收。
