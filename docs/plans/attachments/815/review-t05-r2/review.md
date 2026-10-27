# PLAN-815 T-05 R2 定向独立复审

**needs_fix；Plan executing 10/17，T-05 仍未勾选。** 原 R1 七项：F02/05/06/07 CLOSED，F01/03/04 原例改善但完整不变量 PARTIAL/OPEN；另登记 R2-01 VM 再启崩溃、R2-02 卫生、R2-03 未完成的真实交互正控。当前六项待闭（P1 两项、P2 三项、P3 一项）；R2-03 是 R1 已指出的验收缺口本轮编号化，不是新加能力要求。

## 基线、身份与范围

- review_scope=phase，仅 T-05 提前定向；不是 T-09/M815-B 全计划终审。reviewer=Codex `/root`，延续 R1 独立审查，有早期合同/账本参与限制，未参与本轮实施；未创建 agent、另树或虚构模型/会话身份。
- 主检出最新 Plan r3，T-05 未勾、10/17；工作补记 5e5eddceb 已读，R1 提及的八项 work 记录现已补齐，不能继续当成缺失。本轮不把该补记当作偏差获批。
- 同一唯一树 `D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`，clean HEAD **54aadefadc803af91f6095f8e594b938848d84bd**；修复 **5c4dc277373e2cf388be8e3ea5acbefe891ed5d0**，repair_base **31a25f5cf0e40c42617140768cd2488df7b55257**，原 T-05 base 13b41e9d47cfe62768f68f73b658c3c92b9d3d48。
- M814-B 93a25c924d1beafab130588bad4d1041dc86e48c 已发布/祖先事实保持，无新增外部门阻塞。修复实际改 12 文件，包括 runtime 和 tool-execution；本轮据此审新增路径，没有只回放旧输入。
- [baseline](baseline.json.txt)/[终核](final-baseline.json.txt) 绑定变更及旧相关源码、依赖/锁/测试配置、计划/合同/current Specs；终核零漂移。真实重建 bin SHA256 **2817bb62e34cd48a5d37d0416db9927a068f0cce36e3b18db5a4fcb087154260**。
- [冻结 Spec delta](spec-delta-reviewed.md) 核验 SD-01..06，并补登记新增 runtime 公共退出 API 的实际规范触面 SD-07 候选。完整 B 草稿/最终审定仍 T-08/09。未写 canonical/live ledger。

## 原 finding 关闭表

| R1 ID | 本轮结论 | 独立证据与边界 |
|---|---|---|
| F01 | PARTIAL/OPEN，剩余 P2 | 原 cancel-tree 的 owned parent/leaf 均死、TCP 关闭；共享 Job/进程组接线成立。但 join_readers 超时静默 detach，自有 reader 可跨最终报告继续存活，见下文 |
| F02 | CLOSED（已验输入/控制面） | 同一真实安装等待点，2 字节 InputAck 成功；1MiB 合法输入遇不读者时 cancel、终态、Describe 仍可达，树退出；有限队列与专职 StoppableWrite 代替控制循环阻塞。reader 生命周期另归 F01，不混为关闭 |
| F03 | PARTIAL/OPEN，P1 | 已呈现真实 Iced 的 cancel/EOF 正控通过；初始化期间取消/EOF 仍丢失，见下文 |
| F04 | PARTIAL/OPEN，P2 | 两个原 closed-control case 在 EOF 后均 exit1；非零码不会被 EOF0 覆盖。但 worker 致命写失败不结束仍在读输入的宿主，见下文 |
| F05 | CLOSED（原 Rust 终态分类） | 真实 Rust dispatch/cargo 等待点受控负控确认已到达，cancel 最终 outcome=cancelled，原 Failed 消失；同一 Rust dispatch 未取消而 cargo 受控 exit7 的对照最终 Failed（rust-failure.json.txt）；不冒称真实 Rust UI 编译/窗口正控 |
| F06 | CLOSED | base_revision=5c4dc2773，新依赖已锚定，配置未放松；777 strict pass，0 violations/3 grandfathered，正常 runtime/execution 方向，无新增禁向边 |
| F07 | CLOSED（独立消费能力增量） | 新 lib+真实 bin 的外部 consumer **5/5**，A describe/validate/version拒绝保持，新增未服务 render 真进程负控；闭包原断言原生重放通过。夹具新格式红独立归 R2-02 |

## 仍开放的六项与关闭 oracle

### F-815-T05-R1-01 — P2：reader 超时后仍带活线程返回

**原生命周期缺口继续开放；AC-03/V-03、AC-04，T-05。** owned 子树终止原例已闭，本项剩余范围是自有 reader 真停，严重度由原整树泄漏 P1 调整为 P2。

- 真实入口：`python docs/plans/attachments/815/review-t05-r2/held-reader.py`。真实 tool 做真实 Vue 生成、进入安装；仅安装器替为受控真实 OS 子进程。它将自己 stdout 管道句柄 DuplicateHandle 到本次审查私有、但不属于 tool Job 的 holder，再正常退出（duplicated=true/error0）。不操纵其他会话、不将 holder 当产品 owned 子进程、不制造假 UI 成功。
- 期望：只终止 owned 树；即使非 owned 持管道，也应使自有 reader 真停并具名报告收尾失败，不能到期就当 join 完成。当前 tool-execution canonical 明确 kill≠真停、非 owned holder 应有 PipesHeld，V-03 同样要求 reader 退出。
- 观察：约 **10.45s** 后得到 RunReport（错误是后续 backend Cargo.toml missing，没有 reader/管道收尾错误）；Describe 仍正常。此时线程数 **7**，释放这个外部 holder 后降为 **6**；释放是唯一外部状态变化，与下述 read_until/静默 detach 代码对应，说明 final 之前 reader 没真停。基线线程数5只是背景参照，不把所有新增线程都归为 reader。线程名称接口本机返回空，未冒称按名称定位成功。
- 确认原因：`proc_host.rs::join_readers` 先 drain JoinHandle，等待至 TEARDOWN_GRACE 后直接 return；超时没有错误/停止/实际 join；read_until 本身不可停。原新测试以 readers Vec 为空断言“已 join”，会把 drain 后 detach 误验为真停。
- 关闭：根自然退出/取消/EOF、正常 EOF 与非 owned 持管道/读错等实际资源边界，均证明 reader/writer 真停后才最终报告或具名失败；holder 存活不能被越权 kill。补真实线程终态 oracle，不能只检查句柄 Vec 已清空。
- 证据：held-reader.json.txt/log.txt、held-reader.py、raw-process-final/held-reader-scratch/。首轮无法取得线程名称与一次旧目录准备失败均保留，不计产品测试失败；最终 fresh-root/线程计数复验才是结论依据。

### F-815-T05-R1-03 — P1：VM 初始化清零吞掉本次取消/EOF

**原完整取消不变量继续开放；AC-03/V-03，T-05。**

- `python .../vm-initialization.py`：收到真实 `vm interpreter` progress 后立即 cancel 或关闭 stdin；不是 malformed request，也不是缺环境。随后真实 MCP `first state sync in view()` 确认窗口已呈现。
- cancel-init **2/2**：已收到 Cancelled，但等待2s仍无最终报告且窗口继续运行；随后 EOF 再次触发退出才收口。eof-init **2/2**：输入已关，3s 后工具/UI 仍存活；没有最终报告。正控 eof-presented 则 exit0 退出。
- 确认原因：host on_terminate 在初始化时把 HOST_EXIT_REQUESTED 置 true，renderer.rs:18941 的 run_session 启动无条件 store(false) 将同一操作已经生效的请求清除。观测秒数仅是有界探针窗口，不声称新增秒级 SLA；源码丢标志与后续 EOF 第二次请求可解除形成区分性证据。
- 关闭：进入前、组件构建/主线程 handoff/会话启动中、已呈现后的 cancel/EOF 都不丢请求，并验证旧会话退出请求不误杀下一会话；不能只把清零移到另一处竞争窗口。保留已呈现成功对照。
- 证据 vm-initialization.json.txt/log.txt/py、raw-process-final/vm-initialization-scratch/。初版脚本 caller.P 未绑定 imported helper 的 module.P，原始记录保留；最终脚本已修正到真正 fresh root 后再现全部四个失败。

### F-815-T05-R1-04 — P2：只记录 fatal 码，工作者失败不能关闭宿主

**原内部致命退出不变量继续开放；AC-03/V-03，T-05。** canonical tool-protocol 明确控制流写失败为内部退出1，不是下一次任意输入完成后再决定的普通业务错误。

- `python .../worker-fatal-open-input.py`，同样关闭真实控制 stdout 接收端，保持协议 stdin 开放。Describe 正控无需 EOF 即 exit1；generate_ui 合法未服务 jet 请求的工作者已记录 fatal control write，却2s不退出；只有客户端补 EOF 才 exit1。
- 确认原因：host.rs:162 的 respond_recorded 只写 AtomicI32，没唤醒/停止阻塞读输入的 serve_loop；Launched/输入 writer 错误亦主要仅记码。最终码修正不等于 fatal 生命周期已收口。
- 关闭：最终响应/progress/Launched/input writer 的致命输出失败都触发宿主关闭、停止自有工作并真正退出；客户端持续开放/不再发送 stdin 也有效。正常 EOF=0、坏帧=2、内部错误=1 及竞争优先级分别验证。不以新超时常量代替错误传播。
- 证据 worker-fatal-open-input.json.txt/log.txt/py；原 replay 的 EOF 后 exit1 改善记录保持。

### F-815-T05-R2-01 — P1：同一真实工具第二次 VM 请求崩溃

**新发现的既有串行会话生命周期缺口；AC-03/V-03，T-05。** 单飞限制只拒并发；已完成 operation 后槽位恢复/工具继续服务是当前宿主行为与工作交接承诺，未批准“一进程只能运行一次 VM”。

- `python .../vm-restart.py`：同一个真实工具，第一次 VM 确认真正呈现，cancel 后 RunOk.outcome=cancelled；第一次最终报告后再发第二个独立合法 VM 请求，pac 与请求均 render=vm。第二次请求已获接收/运行进度，没有 busy/准入错误。
- 期望：第二次会话可按已声明能力正常运行与收口，不丢结果、不崩整个协议宿主。实际第二次无最终响应，工具 **exit101**；stderr `Create event loop: RecreationAttempt`，并有第二次 MCP bind9247 失败迹象。首个窗口、请求、取消都来自本探针自己的进程。
- 已确认原因：host.rs:229 在每次 MainJob 都重进 one-shot Iced event-loop 创建；同进程旧 runtime/MCP 状态的复用/拆除未建立。没有将旁边的端口日志当作 panic 的唯一根因，直接 panic 是 iced_winit 的 RecreationAttempt。
- 关闭：同一服务连续两个 VM 操作（取消后与自然结束后），再串接普通 A 请求，均有匹配结果/正确退出；初始化失败再运行也保持；同时对账 MCP/后端资源与退出标志。不能只 catch panic 后把已接收的第二次合法能力降成任意失败，或强迫客户端重启宿主绕过现有合同。具体隔离/复用方案由 work 实证选择。
- 证据 vm-restart.json.txt/log.txt/py、raw-process-final/vm-restart-scratch/。保留初次 pac 与 request 不同但实际到达 VM 的原记录；最终同源输入复验再次 exit101。

### F-815-T05-R2-02 — P3：新增 warning 与新夹具格式红

**修复卫生回归；AC-05 当期范围，T-05。**

- `host.rs:93` InFlight.launched 在移除 Lookup 读取后不再使用，真实 build 新增 `field launched is never read`；ui-tools lib 诊断由 R1 的22条变23条，不以旧依赖大量 warning 掩盖这一条。见 new-warning.txt/build.log.txt。
- `test/fixtures/modularity/ui-tool-consumer/src/lib.rs:278` 新 multiline write 未按 rustfmt 折行；按所有12个净改动中 Rust 文件逐个 `rustfmt --check --edition 2021 --config skip_children=true`，唯一 fixture exit1；其他修改文件通过。见 fmt.log.txt。git diff --check 通过不能抵消格式红。
- 关闭：本轮所有净改动格式通过、该新增诊断消失，旧 warning 单独归因；无需为格式修复跑整档或重跑所有未变功能证据。

### F-815-T05-R2-03 — P2：真实 Launched/交互退出成功正控仍未交付

**R1 readiness 缺口继续存在，本轮编号化；AC-03/V-03，T-05。** 主 Plan T-05 明列 V-03 完整范围；contract-baseline 规定 T-05 真实 build/run/交互/取消全路径。不是要求本轮提前完成 T-06 的所有 profile/媒体矩阵。

- 当前主计划原 work 注记仍把 Launched/URL/交互 quit 正控交给 T-06；本轮修复记录仅提供受控 proc_host 回显、安装期 InputAck/取消和“真实 backend 诚实失败”等证据。读实际 tool_process/consumer 的断言，没有成功运行真实后端→Launched/URL→Input(q)实际退出链的验收。
- helper URL 嗅探、给不读者写入管道、控制 ACK 成功、纯 VM 窗口呈现都不能替代真实已启动可交互 UI 运行链；fake 只能用负控，不能把受控替代服务当真实 Vue 成功发布。
- 关闭：工作阶段提交已进入实际运行态的真实工程/后端，来源可复验；真实 tool 报 Launched/准确URL，日志/control分离，Input 触发实际交互退出/正确最终结果，且随后仍能服务。补字段/结果/进程与 owned 资源 oracle。明确此 T-05 必要正控；全 profile/媒体/发行矩阵仍按 T-06/07，未缩后续任务。

## 当期 AC 与验证账

| AC | R2 当期结论 |
|---|---|
| AC-01/02 | partial；本轮生成/旧门面/请求缝正控保持，全工具/字段组合仍归 T-06/07，T-04 原闭合项未回滚 |
| AC-03 | fail；原七项部分关闭不等于完整生命周期通过，以上六项中的运行/验收缺口阻断 T-05 |
| AC-04 | partial；777/外部consumer/闭包转绿，新 DELIVERY26 的完整792/fresh/locked/build/run后续证据未冒称已验 |
| AC-05 | fail（当期卫生）；新增告警/夹具格式红，完整终审仍T-09 |
| AC-06 | pending；最终pass后merge-owned裸t+必要触面档/实际合入与受管发布等 |
| AC-07..12 | A/L 已发布历史保持，不自动覆盖新B；没有重开未受影响历史 |

- 独立 scoped nextest：**74项，首次73过/1败（其中1项 LEAK）**。root_exit_terminates_pipe_holding_descendants 首次“2s没有首heartbeat”，隔离2项 **2/2通过（1项LEAK仍在）**。源码/fixture/输入相同归因见 execution-failure-attribution.json.txt/unmodified-input-comparison.json.txt：该失败是就绪观察不成立，未证明本次 Send 改坏树；未在旧二进制实跑，不能冒称基线二进制也红。LEAK 是 DirectChildOnly 负控刻意保留非owned持管道5s的 nextest 现象，不包装成无泄漏全绿。
- 上述 scoped含19 protocol、20 tool_process、5proc_host及30共享执行设施当前选择面；另 consumer **5/5**、真 Vue generation 单条 **1/1**、Man公开兼容/媒体/请求缝 **6/6**。不同测试共86项最终断言通过（有首次失败及LEAK如上），不以总数抵消独立反例。
- 777 strict pass **0 violations/3 grandfathered**；配置本身未放松，新锚对应5c4dc2773，方向允许。闭包 shell脚本首次exit127（bash子环境），保留日志；原生 cargo tree --offline 用同一禁列/必需项重放，零禁列/零缺项，见 consumer-closure-native.json.txt/consumer-tree.txt。环境入口失败不计产品红。
- 原 R1 replay.py all 全部实际启动，结果保留在 replay-results/；不能把回放脚本exit0当断言全覆盖，其逐字段状态已独立检查。
- 所有新反例使用真正 fresh 私有根；准备脚本修正记录、无效采样/旧目录失败原件以 initial-/pre-fresh-root- 命名保留。最终依据对应未带这些前缀的结果；每个脚本对所起 tool/helper/私有UI做 finally 清理，非owned holder也仅为本探针拥有的进程。
- 没有跑整档 t/tv/tt/tb/tu/taa/tf/docs_gen；未改实施、canonical或live ledger、合入、归档、清树。

## 再送审

原唯一树一次修齐当前六项，先提交完整生命周期实现、永久反例与真实正控/来源/规范草稿，再送新 SHA。复审回放原例与已支持相邻状态，不仅验证一个窗口已呈现后的 cancel。T-05仍未勾，Plan executing10/17；T-06..08/T-09与最终pass后的merge冻结门保留，未授权减能力/把本次失败转非阻断债务。
