# PLAN-816 T-13 R9 — needs_fix

2026-10-21，revision 2，review_scope=phase / M816-A。按用户指定绑定
`89b0b33ef71fd6c5a3f40fd8a7fc726063efcefa`。本聊天 Codex /root 独立于实施会话，
从冻结源码、失败出口和实际结果重建结论；模型/会话 ID 未暴露，未创建 agent。

- 原工作树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，入场/收尾 clean。
- 修复基线 `2d21cff8c3eb813fbce389d5449e0bb0311f1355`；3 文件 +167/-32。
- 阶段基线 `b65d0e963bbf39f97de5c1d6531997f6f21c760b`；M815-A
  `e609caf1af511b96832f511ab9d1fa6c0c11f563` 为祖先，tool/protocol/model 三包源零漂移。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- [102 项输入指纹](inputs.json)；[冻结 SD 候选](spec-delta-frozen.md)，SHA256
  `cbc62a16540f5476868fb8eb9eb68b5cbd6d3999d81920d4875f4f49fdd64e59`。
  Plan hash在本轮簿记前采集。没有修实施、合入、归档或清树。

| Finding / 子项 | R9 结论 |
|---|---|
| F-816-R3-02 / 凭证创建/复制静默降级 | 子项 CLOSED：fallible API 和具名错误确已接入，两消费方均不启动 writer |
| F-816-R3-02 / client 失败拒绝与清理 | 子项 CLOSED：两个真实失败入口均具名，线程4→4 |
| F-816-R3-02 / run 失败拒绝与清理 | **仍 OPEN / P2**：两个入口具名返回，但 reader 线程4→6；永久负控漏查真实消费者 |
| F-816-R9-01 / 正常完成与新防御检查竞态 | **新增修复回归 / P2**：正常完成误报凭证缺失、返回时 writer 仍活，R8 同例通过 |
| R8 前已闭取消栅栏/终止有界化/Linux健康 | 对应原控制保持通过；不把上述新完成状态与旧取消反例混为一谈 |
| F-R7-01、F-R6-01、F-R5-01 | 保持 CLOSED |

## F-816-R3-02：run 的拒绝出口没有清理已启动 readers

影响 T-02/T-12、AC-02 当期/AC-07。`take_stdin -> Result`、`try_clone -> Result`
消除了旧的 None/NULL 静默降级，具名拒绝方向正确；R8 旧测试无法编译本身不证明
新的失败出口已完成资源清理。

`auto-tool-execution/src/lib.rs:1160–1165` 先启动 stdout/stderr CaptureWorker，
随后 `:1171–1177` 创建失败、`:1179–1185` 复制失败均用 `?` 直接离开 run。
这两条新出口没有调用 finish_early。局部 CaptureWorker 没有 Drop 停止/join，
JoinHandle drop 只是丢掉观察柄，线程持有的 stop/owned_closed 也没有被设置。
OwnedProcess 的 Drop 只处理所拥有进程，不能替代自有 reader 收尾。
交接记录声称这两处“经 finish_early 清理”，与代码不符。

独立 [真实入口故障回放](replay-admission.ps1) 把两消费包的 lib **逐字节原样复制**
到 TEMP，仅在 win_stdin 的 from_stdin / try_clone 边界分别注入 PermissionDenied。
生产调用方、reader 创建、错误映射和 Drop 没有替换。
[插入 diff](admission-instrumentation.diff)、[源码身份及装配证明](instrumentation-proof.json)。
这是分配函数边界故障注入，未声称实际耗尽系统句柄。

输入沿用既有持管道反例：真实 te-child orphan，DirectChildOnly，有限寿命3秒的
非owned后代继承 stdio。故障点等 heartbeat 出现后才返回，使真实失败出口稳定覆盖
“后代已经持管道”的状态。保留无故障的真实 stdin-echo 正控，必须成功且字节完整。
[原始结果](admission-output.txt)：

| 消费方 / 失败点 | 返回值 | 自有线程 | 非owned heartbeat | 结论 |
|---|---|---|---|---|
| run / create | Io，stage=create stdin cancel credential，PermissionDenied，13ms | **4→6** | 继续推进 | fail |
| run / clone | Io，stage=clone stdin cancel credential，PermissionDenied，20ms | **4→6** | 继续推进 | fail |
| client / create | 相同具名 create 错误，47ms | 4→4 | 继续推进 | pass |
| client / clone | 相同具名 clone 错误，21ms | 4→4 | 继续推进 | pass |

每个失败例等有限持有者退出后线程才回4，随后才进入下一例；回放结束没有留下线程。
没有把清理非owned后代当作修复要求。回放失败计数为2，exit1；所有四例的具名分类
都已通过，不能把本项误读为 API 仍在静默忽略 DuplicateHandle 错误。

### 本轮永久负控质量

新增 `credential_creation_failure_is_named_rejection`（lib.rs:1757 起）只构造
`StdinTakeError::Credential`、检查文本/枚举，没有调用 from_stdin、run 或 client。
新增 `credential_clone_failure_maps_named_error`（:1772 起）把人工 Err 送入
map_credential_clone，没有经过真实复制失败后的消费与清理路径。
两测试作为局部格式控制有用，但不构成“两消费方拒绝并清理”的行为负控。
本轮两测试绿且两个真实 run 出口红，直接证明该覆盖断层，无需假设异常发生概率。

修复应对两条错误出口停止并收集所有已启动 readers，或者调整启动顺序避免尚未
取得停止能力就创建线程；保留正确的具名分类。永久控制需从两实际消费入口到达
创建/复制失败分支，断言拒绝、未启动 writer、既有资源真正收尾，并保留同路径合法输入。

## F-816-R9-01：正常完成清空槽后被误判为凭证缺失

影响 T-02/T-12、AC-02 当期/AC-07，**本次防御代码引入的新回归**。

writer 正常退出顺序是：drop stdin → `lib.rs:959–960` 清空共享槽 → `:966`
发送完成通知 → 线程返回。槽为空和线程结束之间存在合法窗口。
新的 `stop_and_join :1014–1026` 用 `!is_finished()` 加 retained=None 直接报
“credential was not created”。线程在收尾中也可能满足这两个条件，不是初始化失败。

确定性 [完成顺序回放](replay-completion-race.ps1) 仅在生产 worker **正常清空槽之后、
发送完成通知之前**插入透明调度暂停，再追加测试；原写入、清槽、停止、防御和 join
代码均未替换。[两行插入 diff](completion-instrumentation.diff)。真实 te-child
stdin-echo，写入正常完成；100ms 后释放暂停，远小于生产3秒宽限。

- 当前89b0：[失败输出](completion-race-output.txt)，
  `Err(Io{stage:"stdin cancel credential missing at stop"})`，
  `writer_alive_at_return=true`，`returned_before_release=true`，exit101。
- 以上一R8冻结提交的完整原 lib 和平台模块重复**同一测试/插入点/断言**：
  [通过输出](completion-baseline-output.txt)，
  `Ok(WriterOutcome{written:20,end:Done})`，writer不存活，未提前返回，exit0。
- 两例都先释放暂停、join writer并收尸，再进行断言；没有另建Git树或改动实施树。

因此新检查不能用“线程还没结束”区分“从未取得凭证”和“正常已清槽待返回”。
修复应按实际所有权/完成状态协调，避免对正常收尾给出初始化失败诊断；缺少凭证的
不变量应在启动前建立。保留这个正常完成组合与原R6/R7有效凭证取消栅栏作控制。
不要求放宽原真停合同或引入任意更短的停止宽限。

## 既有通过项与本轮范围

| 检查 | 本轮结果 |
|---|---|
| 包定向测试 | [44/44](package-tests.txt)：client15，execution11+18 |
| 真实工具独立 consumer | [16/16](fixture-tests.txt)，实际二进制先构建，offline/locked消费 |
| R1五反例 / R2四控制 | [全过](r1-replay-output.txt) / [全过](r2-replay-output.txt)，含双向完整流量/干净与坏EOF/held readers |
| R5两writer入口+三stop终态注入 | [全过](r5-replay-output.txt) |
| 原有效凭证取消顺序 | [R6栅栏通过](r6-race-output.txt)、[R7栅栏通过](r7-after-check-output.txt) |
| 终止失败有界化 | [原R8边界注入通过](termination-failure-output.txt)，0ms具名返回并真实清理 |
| Linux两包+consumer | [exit0](platform-output.txt)，净新增告警0；仅三条已识别旧告警 |
| Windows fmt/fixture rustfmt/clippy --no-deps/rustdoc/CLI-Man check | [全部exit0](gates.json)，新两包零告警，旧scope告警不计新增 |
| 闭包/777 strict | [PASS](deps.txt) / [pass，78边、0违规、2grandfathered](modularity.json) |
| 新入口负控 | 四失败入口均具名；run两例清理失败，client两例通过；合法写入控制通过 |
| 新正常完成组合 | 当前失败/R8基线通过 |

共60个不同现有Windows测试通过，不以重复回放或故障控制凑数。R6/R7探针仅把
Option装配改为Result装配，逆转该一处文本后与R8记录hash完全一致，断言和栅栏未改，
见 instrumentation-proof.json。R8旧None/NULL直接赋值探针因构造状态消除不再适用，
本轮用分配边界注入替代其语义，不能把编译失败记作运行验收通过。

测试依赖/锁/配置/Specs未变；本轮重跑受影响检查及旧共享回放。仅3个源码文件、
主Plan/候选与上述两处probe装配变动。未跑整档t/tv/tt/tb/tu/taa/tf；Linux仅编译。

| AC / task | 当前结论 |
|---|---|
| AC-02当期 / T-02 | partial：普通执行/取消保持；初始化失败reader收尾和新正常完成组合未过 |
| AC-07 / T-12 / V-05 | partial：真实工具/窄能力/拒绝分类/闭包通过，全路径资源收尾未闭 |
| AC-08 / T-13 | fail：上述两项P2未闭 |
| AC-09 / T-14 | pending / merge-owned |
| B原AC-01..04其余面、AC-05/06 | 原阶段门pending，不新增B面finding |

SD-05-A进程消费边界仍对齐；SD-01-A的全出口真停断言不能发布。SD-02/03/04与完整
profile等留原B门，元数据路径及GOAL-003保持；未改canonical Specs/live ledger。

连续失败诊断已收敛：显式错误类型不等于消费方完成清理；局部错误格式测试不等于
失败出口负控；完成状态不能由“槽为空+尚未join”拼接推断。保留本轮明确通过的
fallible API、client拒绝、原取消时序、终止有界化、Linux健康，不重新扩展要求。

next：原树work一次修齐 run 两失败出口及正常完成误判，补真实消费者/状态组合永久
控制，提交后受影响复审。T02/T12重开，T13未过，T01保持，executing1/14。
阶段pass后T14才执行冻结裸cargo t、实际触面/新包fixture/闭包与合入、SD/受管ledger/
来源读回；当前无VM/trans/UI生成/AAVM触面，最终并集复核；tf仅主检出到期批量。
