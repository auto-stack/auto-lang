# PLAN-774 第五轮独立最终复审

2026-10-14，review_scope=`final`，outcome=`needs_fix`。本轮剩一项 P2：累计记账已修好，但 SUMMARY 的 `peak` 解析仍丢高位。保留历轮通过结论，不把解析缺口写成分配器仍超预算。

## 基线与独立性

- 同一树 `D:/autostack/.wt/lang-774/auto-lang`，branch=`plan-774-dev`；reviewed_commit=`8e4fb77289ad95eacfa2538f33dd460da2d707cf`，repair_code_commit=`c781dde710d9cf3c2f510fe7fe1cfd6046fd41a7`；base=`5e21561d2c303a02319ff7a29e4607933e09a38f`，previous_reviewed_commit=`c5e631bd43d1855ba76e144dfac31be3da487bd1`。入/出场同 HEAD、实现树 clean、diff-check clean。
- 主检出 HEAD=`5207f0e83de515d145e42c2423a5fcddaef7deb3`；Plan revision3，入场 execution_done6/8；[审前 Plan](plan-before-review.md)。保留主检出其他既有 WIP。
- 当前 Codex /root 审阅上下文未参与实施/修复；沿用独立审阅上下文，未另建 agent、模型或 checkout，不冒称另起新会话。
- 当前 Spec/下游/源码/依赖/锁文件/门禁/技能绑定见 [input-hashes.tsv](input-hashes.tsv)。ac-core 源码、三道旧门禁、根定义和 crates 相对 base 无本计划 diff；当前 770/771 的身份与码点 API 同步状态保留。
- 冻结规范增量：[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，SHA256=`83a721c3986952ab55009cde220bd8799bca4b747efa127c0d82db0f68c8f71a`。未发布 canonical 或 live ledger。

## 实际验证

| 本轮入口 | 结果 | 持久证据 |
|---|---|---|
| 774 PS gate：locked check、完整 probe、fmt、警告正反控制、永久状态探针、隔离/hash | 8步通过；63 probe tests，状态探针1/1 | [输出](774-gate.log)、[当前 HEAD 收据](774-gate-receipt.txt)、[check](probe-check.txt)、[test](probe-test.txt)、[fmt](probe-fmt.txt)、[state](probe-state-probe.txt) |
| 完整 ac-core locked test | 145 passed，既有1 ignored | [输出](core-full.log) |
| 741 -SkipMainGates | 13步通过 | [输出](741-gate.log) |
| 745 | 13步、74例通过 | [输出](745-gate.log) |
| 772（脚本历史名767） | 14步、45例通过 | [输出](767-gate.log) |
| r1/r2/r3/r4 独立控制重放 | 7/7、3/3、4/4、1/1通过 | [r1](r1-replay.log)、[r2](r2-replay.log)、[r3](r3-replay.log)、[r4](r4-replay.log) |
| 新 SUMMARY 数值协议控制 | 1通过、1失败，exit101；再生重放相同结果 | [首次输出](summary-controls.log)、[重放输出](summary-replay.log)、[断言](summary-controls.rs)、[入口](replay.ps1)、[锁文件](replay-Cargo.lock) |

ac-core 与旧 gates 串行，测试在原树执行。774 收据 head 与当前提交一致；按收据 `raw-outputs` 的实际目录 `raw-20261014-020259` 复制全文，没有按收据结束时刻猜目录。无 crates/root/语法参考变更，未跑根 cargo t/tv/docs_gen。健康检查通过；原型无未处理 warning，日志 trace 均属既有 oracle，没有新生产 debug 输出。

本轮门禁开始后重新生成的774 PE指纹见 [native-hashes.tsv](native-hashes.tsv)（临时产物索引，未复制exe）；生产shim artifact hash在门禁收据内。原函数体受控std适配器的本轮快照见 [永久探针](permanent-state-probe.frozen.rs) 与 [r4重放](r4-state-probe.frozen.rs)。输入表中的主Plan/债务hash是簿记前观察；审前Plan已单独冻结，实现树输入在收尾逐项重新验hash均未变。

## 历轮闭合与遗漏扫描

| 历史发现 | 本轮判断 |
|---|---|
| R1-01/05、R2-03 | 完整外部账本、零预算/负值、成功释放与 invalid 分离、首表/扩表失败回滚保留；r1资源控制通过 |
| R1-02/03、R2-01 | 来源/访问模式矩阵、借入参数 EndView、mut/static 合法入口、真实借用返回、LIFO/退出/临时/move/clone 等保留；来源控制全部通过 |
| R1-04、R2-02 | 默认 view/显式模式、静态与跨行字面量、下游身份/拼写、嵌套 record 与公平 R2 控制保留；没有再缩减合法面 |
| R1-06、R2-04 | 60s/locked、contained executor、警告控制、失败收据和当前 HEAD/raw 绑定保留；本轮门禁通过 |
| R3-01/02、D1 | i64 setter非截断、真扩表恢复与空位复用独立、SD现行规则/统计整理保留；r3控制4/4通过 |
| R4-01 | **累计保存/比较/释放恢复已清偿**：live_bytes/peak存储均u64；原r4状态控制翻绿，永久状态探针覆盖余量/再申请。**观测解析链仍未闭合**，见R5-01 |

未发现新增合同外必需工作。完整借检查器、生产 ABI、List、通用性能/大内存保证、语言实现和 ACC 自举均仍为明确非目标。问题来自现行 D-06 已要求同步的数值观测链，不是增加支持范围。

## P774-R5-01 — P2：SUMMARY 的 peak 仍被窄化为 u32

位置：`experimental/ac-core/probes/data-resources/src/lib.rs:650,668`；`decisions.md:386`、`verification.md:177`，以及 probe-cases/interface 的全精度声明。关联 D-06、V-08、AC-01/03/04/05；延续 R4-01 的观测同步要求。

协议顺序是 `(attempts, allocs, frees, live, live_bytes, peak)`。修复把返回类型改成 `(u32,u32,u32,u64,u64,u32)`，扩宽了第4项 **live对象数**，却保留最后 **peak字节数** 的 u32，并在返回处执行 `nums[5] as u32`。虽先按u64读取字符串，转换仍静默丢高位。

独立协议输入（live_bytes=peak，符合峰值关系）：

```
SUMMARY attempts=3 allocs=3 frees=0 live=3 live_bytes=4294967298 peak=4294967298
```

原已提交 `summary_of` 的实际返回：`live_bytes=4294967298`，`peak=2`。u32上界正控通过；上界+3的合法宽值控制失败。本例是**协议输入对真实已提交解析器的控制**，没有生成高内存 PE，也不宣称真实4GiB申请；原r4小物理内存/逻辑账本状态控制另行通过。

当前永久状态探针直接检查 `Counters`，63 native probes 的峰值都在u32范围，因而现有全绿不能证明“计数→输出→解析”的高位传递。decisions/verification还重复了错误元组，interface/delta称观测全精度，存在可沉淀文档与实际源码不一致。

修复要求：按字段位置统一计数类型；沿现行模型应为 `(u32,u32,u32,u32,u64,u64)`，最后项不得窄化。同步 decisions/probe-cases/interface/verification/SD；加入永久边界 oracle，覆盖合法峰值u32上界与跨界，并同时检查live对象数、live_bytes、peak各字段。至少重放本包解析控制；若声称端到端全精度，再用小内存受控状态验证原formatter输出交原parser后的值，避免新增大内存验收负担。不得把expected改成截断值。

## AC 与规范增量

| AC | 本轮判断 |
|---|---|
| AC-01 | partial：D01..07溯源与下游同步保留，D06观测数值类型与全精度声明不一致 |
| AC-02 | pass：五类真实ABI、异质顺序、字段/嵌套/out-return保留通过 |
| AC-03 | partial：资源回收、拒绝、故障/trap与累计状态通过；峰值计数解析oracle仍丢精度 |
| AC-04 | partial：Unicode/索引、预算执行/恢复通过；跨累计边界的观测链不完整 |
| AC-05 | partial：隔离与旧gates通过，无feature/公开ABI/自举冒称；SD引用§3.11的观测声明需按实际修正后重冻 |
| AC-06 | fail：存在单一P2阻断，T07不能勾选 |
| AC-07 | pending：merge-owned未执行 |

SD add=`auto-ac/data-resource-contract`，modify=`auto-ac/project`、`auto-hir/stage-contract`、`auto-acc/project`，无retire；当前组件与前文已核对，touched_goals=[]有书面解释，不推进native自举Goal。冻结版正确包含§3.11和63+1用例；其“观测u64全精度”结论仍依赖本次未修好的解析链，不能直接作为已通过合同沉淀。

## 收敛与交接

唯一当前阻断是P774-R5-01。此前发现的累计饱和/超预算放行已修好；本轮缺口是字段位置错位及永久观测测试缺失。原样例翻绿和运行时不变量闭合均保留，不重构来源模型/ABI，不重新缩减支持面。

`needs_fix → executing1/8`；T01保留，T02..06按依赖重开，T07/08未勾；revision仍3，本轮未改变语义合同。原树修复解析类型及报告/永久oracle，提交后重放本包与旧控制，再独立复审。

pending_closeout：最终pass、Spec/导航/ledger/index实际沉淀、合入授权、归档/任务数、批回归到期、wt-guard与同树清理。本轮仅审阅/证据/主Plan簿记，没有修实现、合入、归档或更新canonical/live ledger。
