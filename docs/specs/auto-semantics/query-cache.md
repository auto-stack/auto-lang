# auto-semantics 查询缓存与周转（783 store 消费）

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


> **Status**: experimental（PLAN-796 冻结；现行规则以 D21–D24 为准）。
> 登记实现 = `experimental/ace/crates/semantics-store/src/{store,budget,ipc,
> turnaround}.rs`；持久化经 783 ManagedStore（层 `S796`、载体
  `SnapshotRecord`），零改 783/787 实现。

## 查询键（键覆盖一切实际读取依赖）

公共轴：query/package/module/algo/schema（页 v2）/producer/target-sem/
caps；逐查询轴：

| 查询 | 专有轴 |
|---|---|
| InferBody | fn-sig-fp + fn-body-fp + fn-cand-fp + type-cand-fp + called-sigs-fp（读签名变化=换键）+ **src-fp**（位置漂移=换键） |
| FnSig | fn-sig-fp + **hdr-sem**（签名级头语义版本轴） |
| RecordFields | field-set-fp + ref-versions-fp（直接依赖有效版本）+ unit-topology-fp（SCC 拓扑） |
| ListCandidates | **index-content-fp**（fn 候选+record 候选+decl 签名版本全量字节——签名编辑/record 增删=换键，body-only 幂等） |

组页键另含 members+group-content-fp（成员 canonical+引用有效版本）。
**同键不容两容**：payload/头语义变更必须换键（schema/codec 版本轴或逐
查询语义轴），783 Conflict 不可绕过、不靠清库规避。

## 读回准入与新鲜性

- 见 [semantic-contract.md](semantic-contract.md) 页种类覆盖映射——
  嵌入 ref 全量当前（包/模块/存在性/内容版本），字段级 ModuleIndex
  准入，签名级 FnSig 头（body-only 不误失效）。
- 依赖有效版本 = 成员 canonical + 外部单元有效版本传递闭包（一次 SCC
  分区 + 依赖序，`SCC_BUILDS` 计数控制只分区一次）；Hit 复用即当前项目
  soft root（单页/required 链/组页三态跨 GC 保活）。

## 预算（资源状态，非语言错）

- SessionMeter 贯穿 decode（payload 2×）/solve（保守前置预留：语句/
  表达式/名称字节，全程持有）/publish（clone/encode 前页预算）；worker
  每步上报，父侧计数。超预算 = typed `ResourceExhausted`，先于 clone/
  encode，不以 codec 尾错兜底。
- 页预算从**实际将编码的 manifest** 逐字段估算（facts 正/负字段含目标
  函数名 origin）；名称按多重性预留（solve×4/publish×8）；
  `MAX_PAGE_BODY`=512KiB。32MiB 流式页/4096 深链/512 churn 换轮 GC
  收敛为永久控制。

## IPC 与周转口径

- 语义 worker（`semaworker/1`）经 788 受控执行：Job containment（悬停
  spawn→assign→resume，子孙收割）、4MiB 请求/响应元数据上限、工具身份
  四轴（protocol/recipe/exe-sha256/worker-version）入报告与 stale 判定。
- **周转硬门**（冻结参考负载 gen796-1mi/v1，1MiB 模块局部 body 编辑）：
  P90 ≤ 2000ms，计时窗口 = **请求准备（steps/nonce/base64/JSON/hash）
  → worker 调用 → 父侧准入至当前结果可用**的单一完整墙钟；连续分段
  `prep/worker/admit`（µs 计数器）铺满窗口可对账（早/迟阶段反控）；
  冷 warm-up 会话单列 total，不并入编辑指标；30 原始样本 + 阶段 trace
  保留。raw 报告逐样本数值准入（存在/非 null/数值/有限/正值，违规即
  FAIL 不默认 0ms）。

## 失效与恢复

- 首例即真实 store 写入→进程退出→恢复→必要依赖验证→取结果；重算禁调
  反控（hit 不调 compute）。坏 frame 地址/owner/schema/越界/循环组=
  受控拒绝/重算；不缓存 PASS 或瞬时取消/资源失败。
- 旧版本页迁移：语义版本轴隔离新旧键（保留旧页不可达不清库；当前页
  无冲突发布；store 重开可读——永久迁移控制）。
