# auto-ac AAC 增量查询、持久缓存与编译发布驱动（PLAN-1052）

> **Status**: experimental（PLAN-1052 冻结；实现 = `experimental/ace/aac/store/` +
> `experimental/ace/aac/driver/`，经 packages-aggregates-ct 车道 Gen0 编译、AV
> 执行；conformance = `experimental/ace/tests/incremental/`）。
> 现行规则以本文为准；既有 Rust 事实页/查询会话（SQ-A 934、实例 935、CT 936）
> 与 978 工程身份不因本文改写。

## 定位与边界

- 本层是 AAC **策略主体**：Auto 拥有模块编译、正负语义依赖、cache/恢复/事务发布
  与对象链接驱动的**决策**；Rust 保留 Gen0 前端编译、AV/机器执行、消费侧安全
  复验与显式 runtime/工具（1051 服务与 host linker）。
- 服务不得接源码代做 parse/bind/infer/实例或 HIR lowering；序列化 Checked 标志
  不具权威——恢复准入必须重验身份/校验/覆盖/诊断，缓存不带活工作状态。
- 有限面：本包声明的支持 profile（1044 冻结）；完整 CL/代际入口归 1053/1054，
  本包不因子集完成宣称完整 AAC。

## 六轴 BUILD 身份键

| 轴 | 内容 | 不入键 |
|---|---|---|
| schema | 构建/装配 schema 版本 | — |
| target | 目标平台/backend | — |
| stdlib | stdlib 闭包内容版本 | — |
| grammar | 语法/grammar 内容版本 | — |
| pass | pass/阶段链版本 | — |
| bridge | 桥/ABI/服务身份 | — |

- 键 = 域分离长度前缀 hash（`identity_hex2` 两两嵌套，
  `ac1052/axis2/v1` → `ac1052/pair/v1` → `ac1052/build/v1`）；任一轴变化即换键，
  键不含文件系统路径或绝对 span。
- 页键（`ac1052/page/v1`）：owner logical + 签名轴 + body 轴 + 调用方 capability
  摘要；body-only 编辑只换 body 轴，签名读者保持 warm。
- ref 键（`ac1052/ref/<domain>/v1`）：根记录把页身份绑定到其 required 产物集；
  同 payload 不同 domain 绝不共享键。

## 查询 host 与正负依赖

- 完整查询键：query kind + logical + 签名轴 + body 轴 + capability 集；capability
  排序去重，调用顺序不入键；body-only 编辑保持签名轴 → 只读签名调用方零重算。
- **正依赖** = 求解实际读到的 callee 签名版本；**负依赖** = 求解期"该名字不存在"
  的候选集；另持 fn 候选集指纹。任一正依赖版本变化或负候选翻转 ⇒ 该 owner 必须
  重算（V-04 添加缺失名 N 后曾记录 negative read 的 A 重算并成功）。
- 邻域判定：签名变/负翻转 ⇒ caller-recompute(2)；仅 body 变 ⇒ callee-recompute(1)；
  全不变 ⇒ caller-reuse(0)。生产计数（solver_runs/body_hits/body_misses/
  rejections）为真实生产者数字，禁止以总 cache hit 冒语义正确。

## store 与恢复准入

- 恢复准入六条件全通过才可用：信封（producer/schema/algo/profile）→ 页校验和 →
  页键与当前源轴匹配 → 引用完整 → 节点覆盖 → 无诊断；任一失败为**受控
  miss/拒绝**（unknown-envelope/corrupt-page/stale-page/dangling-ref/
  missing-coverage/with-diagnostics），**不得**"查不到就推导"。
- 发布三态：prepare（候选校验，不安装）→ commit（单次切换 current；失败保旧）→
  rollback（native 更新失败保持旧可运行闭包字节不变）；修复后真产新根。
- 错误分类独立：language / resource / os / tool / admission 各自成类，资源拒绝
  与语言诊断互不冒充。

## 包闭包与生成 FFI 装配

- 锁闭包身份 =（lock 内容摘要, 包集摘要, provider, target, capability 集）；
  **cwd/工作区根不入身份**——同锁闭包换 cwd 身份不变（V-05）。
- 声明闭包 vs 物化集逐项核对：物化但未声明的依赖拒绝（不静默吞入）；声明缺失
  拒绝；provider/target ABI 变化使受影响 artifact 失效；缺外部库报 tool/link
  错误并保旧闭包。
- 消费面：auto-man `auto-package-lock {model,codec,verify}` 锁合同形状、978 SDK
  与 967 binding/shim 身份；不另建联网包管理器。

## 与既有面接缝

- 934 事实页/查询会话、935 实例、936 CT、978 工程身份保持其 owner 与合同；本文
  只定义 AAC 主体在其上的**策略所有权**（身份键/读集/提交/恢复准入/闭包绑定）。
- SD-02：`docs/specs/auto-semantics/source-query-session.md` 在 review 验证后补
  本包已交事实与本文件指针，不把目标态提前写成 current。
