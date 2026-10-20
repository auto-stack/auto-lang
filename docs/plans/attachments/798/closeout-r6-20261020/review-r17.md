# PLAN-798:r6 R17 验收/readiness记录

审查范围：final；outcome: needs_fix（readiness；已知work-owned门未闭，不重复冒领完整最终pass）。实施会话内由Codex重新核对源码/旧独立反例与实际日志，不是独立上下文或独立模型；未派代理。工作树`D:/autostack/.wt/lang-798/auto-lang`，分支`plan-798-dev`。修复代码`d1357870a0927dc0a859a97a992daf0a43b4a001`，文档后继`d72b2559653f9023ac6af653f3f9a4b215cf0f40`。基面cb2194c2461a203712bd601c7d314ab35544fec3；重基前后映射见rebase-range-diff.txt，不能把解决797接缝冲突称为全部patch完全等价。

## 原finding与新修复

R15原12控制实际重放见r15-independent-replay.log.txt：A08/A09当前grant拒绝/授权正例、可信unit信封与module冲突拒绝、空sidecar重推导保真以及manifest仅lease下GC后带span恢复均符合预期。旧原例关闭不覆盖漏接路径：R16发现driver仅lease第一个ABC单元、A08未lease、恢复未lease manifest。d1357870a补每个消费单元及manifest闭包；永久回归`av_cache_consumption_leases_cover_siblings_and_release`包含两函数A08/A09、真实manifest恢复、身份拒绝和每条lease释放检查。cache18/18、resources17/17、正式正确性6/6均通过，见lease-targeted.log.txt、lease-formal-correctness.log.txt。准备器stdout超帽/超时后代清理、协议错exit/多帧/错hash/迟到响应等正式测试在这6项中真实执行；不把测试数量当完整不变量证明。

加速父侧完整SHA-256保持每请求重新读取worker全文件，未缓存身份；空/abc/边界长度/大文件/同路径等长替换与旧算法匹配。797兼容：保留ABI和AV两个Options/解码/dispatch/各自stale身份，六种混合顺序以及改ABI/AV仅对应recipe变stale通过；ABI拒绝AV-only run。30个contract_v2、27个共享接缝、身份回归证据见final-contract.log.txt、rebased-seam-compatibility.log.txt及final-identity.log.txt（文件名以目录清单为准）。这些根代码未在后续生命周期修复中改变。

## AC→任务→证据

| AC | 任务与实际验证 | 当前结果 |
|---|---|---|
| 01 | T01/03；私有阶段凭证、wire v2/kind/producer及真实两函数vertical；K14见下 | 代码/对应正反控通过 |
| 02 | T02；有类型MIR/CFG/初始化、真实参数求值/来源；R12尾操作数及R13 prefix/call组合由resources17验证 | 通过 |
| 03 | T03；codec/数值trap/短路/命名实参/循环递归、原生独立结果对拍；核心未改部分复用有边界证据 | 通过；完整实验包重跑归merge |
| 04 | T03/07；grant、kind、资源/取消、准备器和worker失败退出实际反控 | 通过 |
| 05 | T04/05；恢复重验、依赖/身份/跨工程/Conflict/来源/manifest required-data及消费lease | 通过；R16漏接已修复 |
| 06 | T05/07；六场景同轮180样本、内存双线/分区、正式30样本与warm两道硬门 | 两次warm复验失败（1027.0/1465.5ms），不授pass |
| 07 | T07；正式corpus/负oracle/解码/故障/混合stale/已有sema与ABI兼容 | 正确性通过 |
| 08 | T06/08；实际796绑定、有限native/AJ边界与SD01..08；逐阶段非全增量事实、828职责 | 通过 |
| 09 | T09；同树独立方法审查、已披露会话限制；完整AC06未闭不得勾最终review | pending |
| 10 | T10；冻结整档、Spec/ledger、ff-only/归档/guard清理 | merge-owned pending |

## V与K14完整责任矩阵

V01私有授信/当前能力，V02 CFG/类型/初始化，V03数值trap/短路，V04参数/控制/递归，V05坏wire及可信生产类别，V06命中仍执行及A08/A09重验，V07精确单元键/依赖/跨工程，V08错身份/并发/GC，V09 4096块/33.5MiB产品/512修订及双线资源，V10 fuel/取消/帧/进程失败，V12真实runner负控，V13 796/native边界：对应永久族、当前35+6定向及R15独立控制通过。原先102条实验验收只用于未改组件；`unchanged-components.txt`证明core、独立锁、ac-core/artifact/semantics/Atom诊断依赖未变，不能外推修改后的driver。V11失败，依下节独立判定。V14按以下六条，不用旧全绿表代替：

| K14 | 授信/执行/恢复/runner覆盖结论 |
|---|---|
| 01 | 完整单函数self-call module=42往返执行；多函数正例仍有效 |
| 02 | unit可验证不可直接执行；合法raw kind正控不能被一概拒绝 |
| 03 | complete module入口零参i32/import/profile规则；不外推未来能力 |
| 04 | 未知flags/version拒绝；可信unit信封/载荷冲突在A08/A09和无源恢复拒绝 |
| 05 | 当前grant在A08/A09/闭包/新进程恢复重新核验；正式runner合法caps正反控 |
| 06 | v1不按v2执行、确定性stale miss重建；来源适用性/位置漂移及manifest闭包保活 |

## 同轮计量与不通过保留

正式30样本`lease-formal.log.txt`：edit P50/P95/P99=799.7/867.8/927.6ms、sig P95=385.8ms，exit0。

六场景`lease-turnaround-{raw.jsonl,summary.json,result.json}.txt`：180条全部正确，两线met=true；A=211615744B/201.8125MiB，整体峰217808896B，B真实残差6193152B/5.90625MiB；hot/staging/slots未超预算。但body-edit P95=1026.9766ms，exit5。该轮失败不能用正式门或旧bench通过替换；也不凭其他编译在运行就宣称代码无关。

一次受控复验使用相同release二进制、负载、30样本和1000ms门，基线/worker SHA及完整raw/summary另存`bench-controlled-*`；实际exit5，body-edit P95=1465.4601ms，sig40.3117ms，180条均正确，A200.91015625MiB/B6.37890625MiB双线通过。此前9/11/10%且无compiler进程的三次采样仅说明起始窗口，非全程独占证明。

## SD与健康/遗漏

已准备11份MIR/AV合同/导航文件，修改AC产物/HIR阶段/TestCase架构及派生INDEX；SD01..08逐项对照当前代码。实际标量/私有凭证、版本2与类别、当前授权/来源/required-data/lease/预算、有限native引用及完整函数增量尚缺的四段明确；没有把798冒领为828在线安装。临时r1 Schema/AVEN/A09旧表头与前门旧run扫描描述已校正。`touched_goals=[]`仅表示experimental/测试接缝，不改产品目标入口。规范仍在工作树，未发布到账本。

两包定向fmt与diff-check通过。`cargo fmt --all`会包含只读auto-down既有生成代码格式差异，失败日志保留，不改依赖。根全特性既有warning及shim-metadata旧warning单独归因；本计划改动未新增compiler warning。未发现额外范围的隐性延期；全链路增量作为用户认可的828新交付，不是798已承诺项的逃逸。

## F798-R17-01 [P2] warm body-edit硬门未闭

原AC-06/V-11/T-05，非新增要求。两次同代码、同负载、各180条正确样本，body-edit P95=1026.9766/1465.4601ms，均exit5；正式门通过不能替代它。新增逐单元lease发生在原sig前端早退之后，可能增加命中/持久路径开销；目前没有同窗逐阶段或A/B因果证据，既不能归结为纯环境，也不能用首个unit lease代替全部消费保护。下一work先计量当前grant/缓存读取/lease/sidecar/manifest等真实调用，再做有界优化，保留R15/R16安全/来源/GC回归并按同一合同重测。该798已有门不得转移给828或降低阈值。

状态：executing8/11；T-05开放，T-07通过；T-09/10未勾。未运行merge-owned整档、未合入/发布Spec/更新账本/归档/删树。所有失败与通过原始文件保留；不继续无新证据的盲目重试。

## 下一步

AC06完全通过后才能execution_done→最终review pass；随后merge在新冻结提交统一跑裸cargo t（含docs_gen）、cargo tv、实验workspace完整/native及实际796/788/797触面门；当前这些均pending，未提前运行。tf仅到期主检出单实例，现有批量收据2026-10-20T09:20Z且798不被5整除，当前未到期。Spec/ledger/index沉淀、归档/实际合入/guard与清理未执行，绝不声明交付。
