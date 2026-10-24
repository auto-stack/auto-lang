# auto-ac MIR-native 单元缓存/链接/周转（PLAN-818）

> Status: active（实现=`experimental/ac-mir-native/src/{cache,link_driver,driver,
> incremental}.rs` + 783/787/788 受管 store 复用）。与
> [mir-native-scalar.md](mir-native-scalar.md)（语言面）配套。

## 函数单元与对象键

- 每函数一枚可重定位 COFF（保留 external symbol）；逻辑 symbol 由
  owner/声明/签名/私有 ABI 配置生成，不含绝对地址，不以 body hash 改掉
  callee 符号；entry wrapper/必要 runtime/链接输入 manifest 独立。
- key 轴=函数 MIR 语义内容/owner、读取类型/常量/callee 签名与外部符号计划、
  MIR 语义/codec 版本、Cranelift/backend 版本、target/CPU/flags/opt_level、
  私有 ABI/runtime/debug 策略；codegen 实际读到的 module 级信息完整入 key。
- callee body-only 改动恰重编该函数；caller 在未内联/只依赖签名时复用；
  callee签名变化使相关 caller 失效；函数重排不串引用。cache hit 证明
  `codegen_emit_units=0`（禁调反控），link/run 每次真实执行、不缓存 PASS。

## store 生命周期（783 继承 + 818 新事实）

- **单元 COFF 缓存项不带 project**（783 put 带 project 自动 soft-root 会使
  体量 churn 永不可收敛）；manifest 按 787 R3-03 顺序发布摘旧 root；大对象
  按 ≤256KiB 页+manifest 落盘/恢复，publish 服从暂存上限。
- bad-address/owner/target/profile/callee-signature 与自洽异址/内层 manifest
  错配在实际消费门拒绝或 miss；同键异内容=受控 Conflict 且旧提交保留；
  hash 保留换对象=Conflict、活身份存活。无上游活缓存（仅 manifest+对象字节）
  仍可合法 link/run，不把 CheckedMir 从磁盘 trusted 标志恢复。
- 事务：stage→backup→commit 合同；link 失败/超时/发布 IO 失败保留旧产物
  byte-identical+回收 owned 暂存+准确恢复收据；发布目标被目录占据=用户财产，
  不备份不删除。

## D828 增量消费（history/回退）

- warm-up 一次全模块编译（828 前端+全单元层）→每 body 编辑=828 单 body 事务
  prepare→mn 门重验证该单元→恰该单元 COFF 重发（fp/代际仅它动）→commit→
  真实 link/run。未改单元 fp/代际不动、whole-module 阶段恒 0。
- **history 复用**：内容命中（改回历史 body）=cv→fp 映射从 A09' store 复用
  历史单元——零全模块工作、零重发、fp 回退、代际不跳；改回旧值即成正控。
- **语法回退**：828 语法增量 NoConvergence（合法编辑、引擎局限）=记录性
  **全请求回退**（`warm_edit_fallback` 标注，永不标记 incremental）：从
  **session 当前源**（含全部已提交编辑前缀，exactly-once 保留）重建，绝不
  从原始 request source 重建丢前缀；回退后整请求诊断按先诊断后结构准入
  传播（诊断拒绝按设计不达 link/run）。回退**不是增量样本**。

## 资源与周转硬门

- 受管数据 ≤48MiB 分区（MIR/符号 16、lowering/索引/codec 24、I/O 8，账本
  入 trace）、worker 私有峰值 ≤512MiB、含 link/PE 子进程请求树 ≤768MiB
  （5ms 采样器，瞬态 spike 反控）；单 COFF≤16MiB、对象合计≤64MiB、link
  输入≤128MiB、IPC 元数据 4MiB/stdout+stderr 各 1MiB。
- 水位 GC 真物理删除至低水位；lease 活则 manifest 存活、释放后全量回收；
  root 哨兵不动；≥32MiB 真不同 COFF 冷集+512 轮真实编辑/churn 门实测。
- **时间门**（§5.5）：warm session 单函数 body 编辑与合法签名重建的
  「请求准备→当前正确 PE 结果经父侧准入/比较」P95<1000ms，含真实 link/run，
  不扣编译/IPC/排队——计时语义见
  [auto-test-case/architecture.md](../auto-test-case/architecture.md) 的
  rounds 协议（父侧观测连续窗口，含独立期望比较）。
