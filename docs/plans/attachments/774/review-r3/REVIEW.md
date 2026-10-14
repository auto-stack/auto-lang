# PLAN-774 r3 第三轮独立最终复审

2026-10-13，`review_scope: final`，`outcome: needs_fix`。本轮完成完整范围门禁、前两轮反例重放、源码/合同/SD 对账和新增 native 边界控制。旧缺陷已有实质修复，但预算宽度有可复现错误，V05-15 的恢复后扩表 oracle 仍需校准。

## 基线

- 唯一树：`D:/autostack/.wt/lang-774/auto-lang`，branch `plan-774-dev`。
- reviewed_commit：`b3bab4278c9f065a5609a57d02d1f2552f640eab`；修复代码提交 `da6a693c9439da2ae1e13a5e106af06789155818`；基面 `5e21561d2c303a02319ff7a29e4607933e09a38f`；上一轮 `773f9a09aeb874845dc0a0cf44d89a07e3b3a87f`。入场/出场 HEAD 一致、实现树 clean、diff-check clean。
- Plan：主检出 revision 3、入场 execution_done 6/8；主 HEAD `98e8059fe78a64390c11e5f761a132dc8708fb48`。审前原文见 [plan-before-review.md](plan-before-review.md)。其他计划/设计 WIP 保留。
- 审阅者：当前 Codex /root，沿用独立审阅上下文，未参与实施或修复；无另建模型/agent/checkout，不冒称另起新聊天。
- 依赖：同树 auto-ac-prototype path dependency、Cranelift 0.126.2；原型与旧门禁源码、根定义无 774 diff；主检出相关 Specs 与该基面无语义漂移。770/771 的实际身份/拼写同步已核对主检出，不读其工作树旧计划副本。
- Spec/代码/锁文件/合同/技能 hash：[input-hashes.tsv](input-hashes.tsv)。冻结 SD：[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，SHA256 `ff5ec7d62c8538bdade8eb8e99c3302aaf710da157d6e5cfbf4934cc80f73346`（与上一轮相同）。

## 实际重跑结果

| 入口 | 结果 | 持久证据 |
|---|---|---|
| 774 PS gate（含 locked check/full probe/fmt/警告控制/scope/hash） | 7/7 步，**61 passed**，0 failed/ignored | [gate 输出](774-gate.log)、[提交后收据](774-gate-receipt.txt)、[check](probe-check.txt)、[test](probe-test.txt)、[fmt](probe-fmt.txt) |
| 完整 ac-core locked test | **145 passed**，既有 1 ignored | [日志](core-full.log) |
| 741 -SkipMainGates | 13/13 步 | [日志](741-gate.log) |
| 745 | 13/13 步，74 cases | [日志](745-gate.log) |
| 772（脚本旧名 767） | 14/14 步，45 cases | [日志](767-gate.log) |
| 第一轮反例 | 7/7 passed，exit0 | [日志](r1-replay.log) |
| 第二轮来源控制 | 3/3 passed，原 EndView panic 已清偿 | [日志](r2-model-replay.log) |
| 第三轮独立 native 控制 | **2 passed / 2 failed**，exit101 | [输入](counterexamples.rs)、[输出](counterexamples.log)、[重放](replay.ps1) |

执行均在同一树；旧三门禁串行。未跑根 cargo t/tv/docs_gen，因无 crates/root 定义改动，遵守计划明确门禁。编译器零警告/格式/scope 检查通过。PE 的生成与运行复用 contained executor；61 测包含 AV 前驱、Deadline 文件前驱、真实 Win x64 ABI，不冒称 Auto native feature 交付。

本轮 gate 收据 HEAD 正确为 `b3bab4278...`，实际 raw 目录已解析并复制，而非推导时间戳；之前 work 收据只作为历史证据。工具/shim 完整 hash 在收据；本轮生成 PE 的 hash 见 [native-artifact-hashes.tsv](native-artifact-hashes.tsv)。反例输入和输出独立持久保存，不依赖临时目录保留。

重放：PowerShell 执行 `& docs/plans/attachments/774/review-r3/replay.ps1`（可传 `-Tree`）。使用外部 ignored manifest + 冻结 lock，编译原已提交 harness/shim；未改实现。当前应 exit101，两预算断言失败；扩表恢复与 u32 预算正控制通过。

## 当前发现

### P774-R3-01 — P2：i64 预算静默截断为 u32，合法小请求错误拒绝

类型：原宽度/预算不变量缺口（延续 R1-01）；AC-03/04；T-02/03/04/06；D-01/06；V-05/08。

冻结接口 `p774_set_budget(i64)`，decisions §3.8 和 interface §7 都规定 `v≥0` 为真实字节预算，负值清除；没有声明“大于 u32::MAX 就按模 2^32 取值”。`shim/src/lib.rs:1016–1020` 却执行 `Some(bytes as u32)`。

真实 PE 的三个相邻控制：

| 设置值 | 请求 | 按现合同应有结果 | 实际 |
|---|---|---|---|
| `0xFFFF_FFFF` | 2 bytes | 成功 | 成功，SUMMARY `1 1 1 0 0 2` |
| `0x1_0000_0000`（4 GiB） | 2 bytes | 成功 | 被截成零预算，拒绝；PE 断言 exit2，SUMMARY全0 |
| `0x1_0000_0001` | 2 bytes | 成功 | 被截成预算1，拒绝；PE 断言 exit2，SUMMARY全0 |

此处没有分配 4 GiB，只分配 2 字节；不是新性能/任意规模承诺，而是声明 i64 通道输入的预算值不能静默变成另一个数。源码的长度/源语言 i32 限制不能替代 setter 参数域的规则。

修复：预算与 aggregate live-byte 比较使用一致且不截断的表示/checked arithmetic；若确需 bounded budget，则明确入口、单位、范围、失败机制及两侧 oracle，先拒绝越界、不能改变成别的预算。保留 0/负值语义及释放后额度归还。修复后重放这两失败控制和现有预算/转换/记账合法控制。

### P774-R3-02 — P2：V05-15 重试未实际触发恢复后的扩表

类型：必需 oracle 缺口（延续 R2-03）；AC-03；T-03/04/06；D-06；V05-15。

`tests/v05_fault.rs:369+` 先成功插入16 owner，第17项扩表失败；随后 **owners.remove(0) + free**，有效 entry 数变为15；再试只插回16，`ledger_insert` 的 `len_n==cap` 不成立。因此绿色 V05-15 证明失败后旧表仍能复用，**没有执行其注释/probe-cases/verification 声称的“清旋钮后扩表成功”**。原有 owner“内容完好”也只以 free 检查，未读其内容。

本轮新独立控制保留16个 owner，再清旋钮重试第17项，读回全部17个 buffer 的字节并逆序释放，**通过**：SUMMARY `18 17 17 0 0 17`。所以修复实现本身的这一恢复路径有独立通过证据，不能把本项误称为实测资源泄漏；缺的是 work-owned 用例与冻结 oracle 实际一致。

修复：让 V05-15 在保持原16 owner 存活时重试，或保留当前复用控制并追加真正跨容量的恢复控制；永久测试/冻结表应区分两种路径和 peak=17/16，验证原内容、状态及资源平衡。不得只重命名原绿色测试为扩表成功。

### P774-R3-D1 — P3：规范增量的显式引用与数量需要校准（非独立阻断项）

类型：知识增量整理建议；SD-01/03。

`proposed-spec-delta.md` 的 hash 与第二轮一致，显式增补仍只标 §3.8、探针数量仍39。提案也引用整个 decisions §3 和 interface-contract，因此能通过当前有效 §3.9 解析恢复规则；**相同hash/没有逐字重复新节本身不构成失效证明**。有效合同的跨行/静态字面量返回和来源模型已恢复，这项不重新打开已通过的 R2-02。

为避免 merge 从报告历史附录搬入已作废语句，建议显式标出 §3.9 的优先级及61用例；来源模型实际 illegal 数组为16条，报告写15。最终 canonical 应是 current-state，不搬过程史。该整理不替代上述两个实际失败/必需oracle finding。

处理：随预算修复刷新冻结SD时校准当前规则引用/数量；保留默认view、字面量返回、合法跨行和元数据责任，沉淀时过滤已作废拒绝规则。此项列为非阻断建议，不要求额外语言实现或提前发布 canonical/live ledger。

## 历轮闭合与 AC 覆盖

| 范围 | 当前结论 |
|---|---|
| R1-01 64/65、零预算、全额归还 | 旧样例7控制通过；新 i64 budget 边界发现 R3-01，不全关预算不变量 |
| R1-02 / R2-01 参数来源、EndView、incoming mut、storage/call | 原反例与新增来源矩阵/真实PE通过；合法借入与静态字面量返回已恢复；模型保持探针边界 |
| R1-03 LIFO/临时/branch/break/continue/move-in/return/clone | 全套 native 探针复跑通过；FREE顺序能区分FIFO；内容独立/带owner申请失败有断言 |
| R1-04 / R2-02 身份、源拼写、literal支持、nested layout、R2理由 | 主检出770/771已同步，当前decisions §3.9恢复；平面/嵌套ABI与正反通过；SD引用整理建议R3-D1不重新阻断 |
| R1-05 释放口径/双free | 不读freed header；成功free/invalid分离，负控exit1和计数均通过 |
| R2-03 首次/扩表失败 | 首表失败/回滚/重试通过；扩表失败旧owner复用通过；独立真跨容量恢复通过，但永久V05-15待R3-02 |
| R1-06 / R2-04 警告、60s/locked、timeout前驱、收据/基线 | 本轮新提交后门禁与raw/hash正确；主计划状态历史保持；规范增量引用当前有效合同、边界正确 |

| AC | Task | 判定与证据 |
|---|---|---|
| AC-01 | T01/02/05 | partial：七决定基本完整、溯源与下游同步已核对；预算域仍需校准 |
| AC-02 | T03 | pass：五类真实Win x64 ABI、异质顺序/字段、嵌套调用与rec2/out-return，61探针覆盖 |
| AC-03 | T03/04 | partial：资源/非法状态/fault/trap基本通过，V05-15 mandatory oracle未与声称一致；宽度错误存在 |
| AC-04 | T02/03/05 | fail：Unicode/索引/转换/0/1/4096/既有预算正反全过，新预算输入边界实测失败 |
| AC-05 | T06 | pass（本基线）：零生产/registry/schema/old source diff、旧gate全绿；SD按有效合同引用、不宣称feature；未来预算修复后需重新冻结对应delta |
| AC-06 | T07 | fail：完整独立验收已执行，存在合同内finding，不勾最终pass |
| AC-07 | T08 | pending：未合入/归档/清理，不预勾 |

范围不扩为完整所有权/GC、公开ABI、List/enum/use/生产I/O/ACC自举。raw/f-string 等770非目标未新增为本轮代码验收。D-02源literal规则属合同核对，774不实现语言parser；D-01精确i32与char名义语义不被宿主Rust/裸机器表示冒充。

## 收敛与路由

本轮不重复打开已清偿样例。下一修复围绕两个阻断finding及受影响矩阵：预算 i64输入×数值保存×live加法×拒绝分类；ledger失败×原容量未改变×真跨容量恢复。SD引用/数量整理是随修复的非阻断建议。之前7/3控制覆盖特定旧问题，不能覆盖预算通道截断或验证某分支实际执行；本次输入/断言已保存，供同树直接重放。

`needs_fix → executing`：T01保持完成，T02..06重开，T07/08未勾，current_step=1/8；plan_revision仍3（本轮只记录，不改变语义合同）。next：同树work修复、提交、重放失败及合法控制，更新冻结提案后再独立review。无需再次大幅重构来源模型或改旧编译管线。

pending_closeout：通过后的Spec/导航/ledger/index、合入授权、归档与任务总数、批回归到期、wt-guard及同树清理。本轮不实施修复、不合入/归档、不发布canonical/live ledger。
