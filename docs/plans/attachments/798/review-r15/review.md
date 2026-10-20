# PLAN-798 R15 独立最终验收（2026-10-20）

**needs_fix，r5，executing 3/11。** R14 同轮 bench/内存证据问题关闭，正式 30 样本门已有可复验 PASS；本轮继续完整信任、恢复、资源生命周期和失败退出核查，发现 5 项 P2 合同内缺口及 1 项 P3 健康问题。不能据性能门和既有测试总数授最终 pass。

当前聊天 Codex，独立于实施、延续同一 R15 审查上下文；没有代理或另建审查 checkout。使用 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`（SHA-256 `0d68ba05276aff05a143c8f27010ee02b0a9bded14c27a0133d3a774c20f0cf3`）及 verification-contract。只读实施；本轮新增的是可重放审查附件。未修改实施、canonical Spec 或 live ledger，未合入/归档/清树。

## 冻结基线与证据边界

- 主检出计划 r5，审前主 HEAD `7902f37b120f0a9deca4db327dea8d9f62024d40`，计划审前 SHA-256 `8D32504D8358373CD5371DF640B34893423C4B9CB24B8D5C992F97F9ECBC3255`，见 [baseline.json.txt](baseline.json.txt)。计划簿记修改不升 semantic revision。
- 唯一实施树 `D:/autostack/.wt/lang-798/auto-lang`，branch `plan-798-dev`，commit `79b31f5dcb99b2b95e1c79a5ae58299c0139f8f6`；base `13367d127584461449ee2f151c0c5987f0740044`。末次直接只读复核 HEAD/分支/根一致、status 无输出、diff-check exit 0。受限调用曾报 worktree 索引访问错误，改用已授权的只读复核确认，没有以那个错误当 clean。
- 相比 R14 `c8c598f0aa67f79d8b083346b5c552a655353d1d` 仅 5 个报告/证据文件变化；实现、依赖和测试配置未变。ac-core 最后变更 `e02c2dcc8094fc522569533304aea175f6aae867`，artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`，sema-contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`；组内 auto-down 只读 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 全部实现、锁、配置指纹见 [source-identities.json.txt](source-identities.json.txt)，附件指纹见 [evidence-identities.json.txt](evidence-identities.json.txt)。当前三份 Spec 与本轮审前输入一致，见 [spec-stability.json.txt](spec-stability.json.txt)；[SD 冻结](sd-frozen.md)、[实现合同冻结](contract-decision-frozen.md)、[r5 资源政策](memory-policy-frozen.md)均保留。
- R13 前门具体反例及 R14 重放的关闭状态保留：当前代码未改，其输入/依赖/配置也未改，不重复造一轮资源/性能压力以推翻或外推旧证据。新的入口不变量用下面独立反控验证。

## R14 证据收口：通过

[同轮审计](work-evidence-audit.json.txt)核对 raw、BenchSummary、正式门日志三件 SHA-256 均匹配。180 行、六场景各 30 行、body edit 30 个不同输入 hash；按当前 percentile 算法重算 edit P95 **414.7482ms**、sig P95 **20.6905ms**。A 线 **211058688B / 201.3MiB**，B 线残差 **4354048B / 4.2MiB**，两线 met=true、hard_gate_failures=[]；受管 staging/hot/slots 峰值分别 16175923/8352112/4432B，均在分区预算内。见 [同轮 summary](work-bench-summary.json.txt)。**F798-R14-01 CLOSED**。

保存的 [正式门日志](work-formal-30.log.txt)确认 30 样本 PASS，edit P95 **948.9ms**、sig **599.6ms**；另一轮 961.7ms 仅作 work 历史说明，本轮不把未保存的另一轮单独认作独立 raw。代码/依赖/配置不变，接受已核对的同基线正式门证据，未因当前新缺口重复跑门取绿。V-11 的这一性能证据通过，不代表其他 AC 已完成。

## F798-R15-01 [P2] A08/A09 缓存命中没有重新检查当前能力授权

AC-04/05，V-01/06/08/14，K14-05；T-04/08，影响 T-07 的真实 driver 消费。`probe/src/driver.rs:350`（MIR restore）及 `:703`（ABC verify）只校验能力 allowlist/声明，没有把载荷所需能力与本次 `caps` 比较；闭包装配和 `execute` 也不接收当前 grants。`worker_restore.rs` 的专门恢复入口有该检查，不能替普通 Driver 入口授证。

独立输入 `fn main() int { return 7 }`；在本进程独占 scratch store 内用当前真实生产信封/当前 owner、unit、body_fp 和查询键，保存语义合法、明确声明 `hir.test.trace.v1` 且含 `Trace(MarkA)` 的单元。没有复用旧版本或错误 frame。对 A09 与 A08 分别请求 `caps=[]`，均 **diag=None，Success(7)，trace=[MarkA]，av_execute=1**。A09 hit=1/emitted=0；A08 restored=1/emitted=1。当前 grant 的正控也成功，普通无 trace 单元正控成功；所以缺的是实时授权门，不是未知能力或无效 opcode 门。

修复：在所有缓存/恢复到闭包的消费入口统一检查实际所需能力与当前授权；缺授权在 dispatch 前拒绝。回放 A08/A09、fresh/hit/restart/正式 runner 的适用路径，保留已授权及普通单元正控。不能仅依赖 key 含 caps 指纹或 source 前端检查：本例 source 不使用 trace，缓存载荷仍被实际执行。

## F798-R15-02 [P2] 单元生产信封与完整 module 载荷类别冲突未拒绝

AC-01/03/04/05/07，V-14，K14-04/05；T-01/03/04/07/08。`EnvelopeIdentity::{mir_bound,abc_bound}` 表示 MirUnit/AbcUnit，但 driver 的两条命中路径及 `worker_restore.rs:163` 没有核对载荷 `is_unit`；`cache::assemble_closure:843`直接取每个输入的 `funcs[0]`，抹掉输入类别并产生完整模块。

在上面同样的合法零参/i32 单函数输入上，仅把载荷改为 `is_unit=false`，按当前 wire v2 编码后放入**单元**生产信封及正确单元键。A08/A09 都接受并执行 7；两条路径 `av_execute=1`，没有类别冲突诊断。对照的真实 unit 也正常通过闭包装配。此前 raw 无可信 expected-kind 的合法翻位正控仍应通过，本 finding **不要求 raw 翻位即拒绝**；本次有可信生产上下文规定单元，违反 §5.8 的 producer/载荷一致性。

修复：消费单元信封时核对 unit 类别及单函数形状，闭包入口按完整 module 规则验证；将同一检查用于 driver A08/A09、manifest 新 worker 恢复。补该上下文冲突反控，继续保留 raw 合法完整 module、unit 不可直接执行及 self-call module=42 的正控。

## F798-R15-03 [P2] Driver 接受空 sidecar 后发布无来源的 trap

AC-02/04/05，V-05/06/08，K14-06；T-04/08。`driver.rs:768-770` 把任何可解码 sidecar 都设为 bound=true，空表/不覆盖 trap 的表不会触发 source 重推导；后续 verifier 允许缺失调试表。`worker_restore.rs` 已做覆盖检查，普通命中入口没有复用。

输入 `fn main() int { return 1 / 0 }`，真实 spanless 单元、正确当前 snapshot sidecar 键/信封，sidecar 内容为正常编码的空表。A08/A09 均 **diag=None，Trap(DivByZero, span=None)，av_execute=1**，没有 rebind-lower。相同单元不提供该空 sidecar 时 Driver 会从当前 source 重推导，得到 **func#0:23..28**，证明来源可恢复。

修复：在 Driver 使用 sidecar 前验证适用性、索引/范围和所有 trap-capable 指令的覆盖；有当前 source 则按冻结策略重建，无法恢复则受控 unavailable/拒绝。统一 Bin/NegI32 和位置漂移路径，不把仅可解码当作绑定成功。

## F798-R15-04 [P2] Manifest 没有 required 边，生产消费也未接实际 lease

AC-05，V-08/09，T-04/05/08。`driver.rs:951-958` 发布 manifest 使用 `edges=vec![]`，忽略写入结果；全部生产 driver/worker_restore 没有 acquire_lease。当前 `av_cache_gc_and_lease` 测试手工 lease 单个单元且项目根仍保留，因此不能证明 manifest→执行所需单元的闭包保活。

独立用真实 Driver 生成除零样例，正常按 manifest 恢复先得到带真实 span 的 trap；数据库查询该 manifest 的 required 边为 **0**。对 **manifest 本身 acquire_lease**，移除本探针项目的 soft roots，调用 store 既有 `high=0,low=0` 测试模式以回收全部不可达候选：GC 扫 4/live 1/删除 3，manifest 仍存活但单元已删；再次恢复报 `unit d_main ... artifact ... not present`。没有伪称这是已加载 AV 帧执行中途崩溃；它明确证明持有 manifest 的 lease 无法保住重建闭包所需的产物。默认 64/48MiB 水位的规模承诺没有改变，0 水位只隔离 reachability 的判别。

修复：发布真实 required-data 边，按当前请求适用性保护 source sidecar，并在 driver/无源恢复实际消费期间持有/释放必要 lease；manifest 发布失败应可观察。补只有 manifest root/lease 的 GC 正控、并发恢复及失败释放控制，不能以所有历史单元永远各自 soft-root 代替依赖闭包和有界生命周期。

## F798-R15-05 [P2] 正式 runner 的构建准备器仍可能背压超时并遗留后代

AC-04/07，V-10/12，T-07/08。`crates/auto-lang/src/test_cases/ac_av.rs:130-180` 的 `locate_worker` 使用裸 Command：stdout piped 但从未排空；stderr overflow 只在主进程退出后读取；超时仅 child.kill 后返回，没有 Job/kill-tree/reap。正常 `call_worker` 和 `native_reference::bounded_worker_build` 已有受限执行器，不能覆盖这个不同调用入口。

[prep-helper.rs](prep-helper.rs)从当前源**提取原 locate_worker 函数体**，只替换编译期工具/根目录为本审查 scratch、将 BUILD_WALL 加速为 300ms（READER_GRACE=100ms），其进程/管道控制流不改。伪 cargo 是同一 helper 的子模式，所需后代 pid 由它自己落盘。不是在真实生产预算下运行完整 Cargo 构建，结果范围是该原控制流的失败退出。

[实际输出](formal-prep-probes.log.txt)：2MiB stdout 模式不是 cap 错误，而在约 **305ms** 返回 BUILD_WALL；启动 sleep 后代的模式约 **304ms** 返回同一错误，随后在**调用进程仍活着时**用 Windows GetExitCodeProcess 查得后代 **259 / STILL_ACTIVE / alive=true**。探针随后只终止自己创建并记录 pid 的后代，未遗留该进程。该独立 consumer 漏接属于原有生命周期合同缺口，不重开 R9 已关闭的 native 准备器原例。

修复：让正式 runner 的准备也使用包含 suspend→Job→resume、双流 cap、单截止和 kill-tree/reap 的设施；补 stdout/stderr 超帽、超时后代及退出后继承管道的成功/失败控制，继续绑定当前 worker 身份。

## F798-R15-06 [P3] 计划新增 core 测试仍有未处理 warning

AC-09 健康检查，T-03/08。`core/src/abc.rs:1063` 的 `use crate::mir::*` 由 `use super::*` 已覆盖；[实验验收日志](experimental-acceptance.log.txt)实际产生 `unused import`。root 的 566 条及 shim-metadata 既有告警单独保留，不归本计划新增；但不能把这个本计划命名空间的告警也称作零新增。删除冗余导入后做相应定向 check/test/fmt；不要求为了该行重跑整档。

## 逐 AC / V / K14 结论

本轮已有 [实验包验收](experimental-acceptance.log.txt)实际 **101 个**测试通过：core21、ABC15、binding4、cache16、case5、MIR8、native-admission11、native-trace1、resources17、vertical3。不是 probe101 加 core21。正式正确性 [4/4](formal-correctness.log.txt)，现有共享 probe 接缝 [16/16](probe-compatibility.log.txt)；work 摘要18/18不作为本轮实际运行计数。以上证据不能覆盖新反控失败。

| AC | 当前结论 |
|---|---|
| AC-01 | partial：vertical/有限版本合同、原单函数42正确；可信单元类别冲突 F02 未闭 |
| AC-02 | partial：HIR lowering/CFG/数值/前门既有范围通过；Driver 来源保真 F03 失败 |
| AC-03 | partial：codec、AV 数值/顺序/native 对拍通过；跨生产上下文类别 F02 失败 |
| AC-04 | fail：缺当前授权 F01，类别 F02、来源 F03及准备器生命周期 F05未闭；fuel/帧/取消原范围通过 |
| AC-05 | fail：普通 fresh/hit/重启/键失效正例通过，但 F01..04 证明恢复/授权/来源/required/lease不变量未闭 |
| AC-06 | partial：同轮六场景30样本/双线内存/受管分区/正式P95通过；资源回收闭包 F04 未闭 |
| AC-07 | partial：真实runner正确性/反控/现有兼容通过；K14 trusted producer 类别和准备失败 F02/F05未闭 |
| AC-08 | partial：796绑定、AJ/native/797非目标边界、SD路径有效；coverage 全绿声明须按 F01..06更正 |
| AC-09 | fail：本报告完成审查但验收不通过，不能勾review完成；实验命名空间 warning F06未处理 |
| AC-10 | merge-owned pending：未触发合入、规范沉淀或归档 |

| V | 结果 / 范围 |
|---|---|
| V-01 | partial：私有凭证/未知版本正反控通过；当前grant消费 F01失败 |
| V-02 | pass：MIR变异、owner、初始化/CFG及前门已关闭原例在当前基线通过 |
| V-03 | pass：数值trap/潜伏分支/短路与独立native比较 |
| V-04 | pass：实参顺序/循环/递归定向族 |
| V-05 | partial：wire坏输入通过；生产上下文 F02/来源恢复 F03未闭 |
| V-06 | partial：命中禁调/仍执行正例通过；命中再授信 F01/F03失败 |
| V-07 | pass：单body/callee签名/owner/跨工程/版本键现有控制 |
| V-08 | fail：F01/F03/F04，不能用直接lease单元的测试代替生产闭包 |
| V-09 | partial：4096CFG、前门、33.5MiB/512churn、内存正反控通过；实际依赖保活 F04失败 |
| V-10 | partial：AV fuel/取消/帧/trace限制通过；准备子进程失败清理F05未闭 |
| V-11 | pass（冻结证据复用）：本轮 raw/summary/正式30样本身份均核对；新修复代码后须评估受影响重跑 |
| V-12 | partial：正式4项及共享16项通过；准备入口 F05失败 |
| V-13 | pass：实际796引用准入/依赖闭包及有限native模型通过；不声称797尚未落地实现兼容或热重载 |
| V-14 | fail：细分见下 |

| K14 | 当前结论 |
|---|---|
| 01 | 原完整单函数 self-call module=42、raw往返及正式fresh/hit通过，多函数语义对照通过 |
| 02 | unit凭证可验证不可直接执行、raw合法module翻位正控通过 |
| 03 | 当前 verifier/执行门的entry范围/零参i32/imports拒绝规则与已有控制一致；不外推任意未来imports能力 |
| 04 | fail F02：未知flags/version原控制通过，**可信单元producer冲突**未检；raw合法翻位不判错 |
| 05 | fail F01/F02：手工/raw以及专门restart授权正控有效，不能覆盖Driver A08/A09 当前grant和producer消费 |
| 06 | partial：旧v1 raw拒绝、当前键旧载荷miss重建通过；位置漂移/sidecar适用性 F03 未闭 |

## SD、健康、遗漏和下一步

SD-01..07 目标均为真实 repo-relative Spec 路径；当前三个 modify 源版本/哈希未漂移，新增 auto-mir/auto-av 尚未发布。SD-01（有限MIR）/SD-05（HIR消费前门）现有边界有效，SD-02/03（类别、grant、恢复、生命周期）受 F01..04阻断，SD-04不能把 A08/A09生产者登记为已具完整required/lease，SD-06正式recipe能力要包含准备失败边界F05；SD-07仍仅有限合同/native准入，未暗授AJ/热替换/新native后端。`touched_goals=[]` 的 experimental范围说明保留。

冻结实现合同头部仍写 AVEN v1、§8 A09 v1，而实际 envelope_version=2/A09 v2；当前§7代际段及代码为事实。T-08更新提案时须消除这些冲突、使用当前 r5 双线政策，不能直接把旧表抄入canonical规范。此为知识增量定稿的具体校正项，未在review修改语义合同。未来825/826优化不作为798新增门槛。

两个实验 package fmt、完整diff-check通过；F06是实际新增告警。扫描未发现需要新增范围的TODO式延期；发现的是上述现有合同漏接，已登记债务候选但不批准延期或降低AC。

T-01/03/04/05/07/08及其review/merge完成态重开；保留 T-02/06/11，**3/11**。全部六项由原树 work 同批修复，核对入口/状态闭合矩阵，补对应永久反控和受影响定向验证后提交，再送审。R14-01及此前已经关闭的具体旧例不复活；完整不变量的新反例不能因旧例关闭而豁免。

pending_closeout_gates：最终 code/AC/Spec review pass **之后**，merge在新冻结提交跑根裸 `cargo t`、编译器 `cargo tv`，独立实验workspace/真实native及实际796/788/已合入797触面兼容；其他档按最终diff。`tf`只在主检出按到期规则单实例执行。Spec/ledger/index、归档合入、实际计数、wt-guard和清理均 pending。当前未运行根整档 t/tv/tt/tb/tu/taa/tf/docs_gen，不授 delivered。

重放新的缓存探针：

```powershell
# cwd D:/autostack/.wt/lang-798/auto-lang
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r15/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target
experimental/ac-av-probe/target/debug/plan798-review-r15.exe
```

[输入及生成器](probes/src/main.rs)、[结果](cache-path-probes.log.txt)。最终共12条具名控制输出（含manifest生命周期），探针exit0表示实验运行完成，**不是**产品验收pass；每项错误行为和期望已在上文列清。首轮审查探针的SQL列名写错曾引起**探针自身**panic，见保留initial日志；修正为store真实size列后重放，最终日志无panic，未把该探针错误归实施finding。一次过低GC水位只得到backoff的中间日志也保留，结论取最终确定性reachability运行。
