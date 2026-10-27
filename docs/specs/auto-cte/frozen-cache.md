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
