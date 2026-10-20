# PLAN-798 R9 修复复审（2026-10-20）

outcome: **needs_fix**；review_scope: final（本次执行旧例闭合与受影响验收，未授完整最终 pass）。

主审为当前聊天中的 Codex；本轮没有参与实施修复、没有创建代理。依据主检出的计划 r2 和原实施树的实际源码/提交重新判断，不以 work 的“全部闭合”摘要作证明。前几轮独立材料已有丢失登记，本目录保留实际源码、锁、输出与身份；`*-final.log.txt` 是本次重放，不混用目录内较早的准备日志。

## 基线与闭合范围

- worktree: `D:/autostack/.wt/lang-798/auto-lang`；branch: `plan-798-dev`；reviewed_commit: `e00340842fdca15417c3195041ca909b1fcb8bb1`；审前及审后实施树 clean。
- diff base: `13367d127584461449ee2f151c0c5987f0740044`；repair base: `f9d9821577eefd0ccfb74530b7d1077923ea9c33`。本轮修复是 8 文件 +335/-51，没有修改依赖、两份 Cargo.lock 或根测试配置。
- ac-core 最后修改提交 `e02c2dcc8094fc522569533304aea175f6aae867`；artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`；semantics contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`。内容 tree 另见 [dependency-identities.json.txt](dependency-identities.json.txt)。组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。
- Spec、配置、锁、主检出计划/设计输入哈希见 [final-baseline.json.txt](final-baseline.json.txt)。主检出三个现行 Spec 与实施树一致。规范增量冻结 [sd-frozen.md](sd-frozen.md)，SHA-256 `4b6582965b882a0b46e6ac1b20fdfb14726d72a523628332ee59d50b5c529b3e`；实际合同冻结 [contract-decision-frozen.md](contract-decision-frozen.md)，SHA-256 `47e65a126ae2bd19b27ceb403185acdd2f02b29f33496f6c65eecd3420386f39`。
- 明确执行离线、锁定的 release worker 构建，结果及二进制哈希见 [worker-build-fresh.log.txt](worker-build-fresh.log.txt)、[binary-identities.json.txt](binary-identities.json.txt)。这绑定本轮测试产物，不表示 `native_worker_binary()` 默认仅检查文件存在的路径自身已有源码身份保证。

| R8 finding | 当前实证 | 结论 |
|---|---|---|
| F798-R8-01 兄弟实参误计深度 | 原 16/128/129/200 个浅层未加括号实参全部返回 1；129/400/600 项长链仍前门拒绝 | 原样输入闭合；闭括号分支仍全局相加，R9-01 开放 |
| F798-R8-02 NegI32 缺来源 | 共享 `MirInstr::can_trap` 覆盖 Bin 与 NegI32；原最小程序诚实恢复为 `av.trap.int-overflow@func#0:45..46`，空 sidecar 在执行前拒绝 | 本轮完整受影响分类及正负控通过；关闭，T04 恢复完成 |
| F798-R8-03 首次准备无界/旧文字 | 现有独立 300s 轮询墙；r4(c) 旧降格文字确已显式撤销；原在途 linker 约 1010ms 即 Budget | 旧无截止及文字子项闭合；新准备器的输出/杀树失败，R9-02 开放 |

## F798-R9-01 [P2] 同层括号表达式仍累计成深度，64 条浅语句被误拒

AC02/04/06，T02/05，V02/09；属于 R8-01 完整不变量未闭合及修复相邻回归，不是要求扩大支持面。

`probe/src/driver.rs:1084` 在每个 `)`/`]` 上执行 `construction += level_max + ...`，最后 :1126 再加最外层。该累加器没有在语句、函数或兄弟表达式结束时取最大/复位。因此它计算所有已闭层的总量，不是注释声称的最大构造路径。

最小生成规则：`fn main() int {` 内生成 N 行 `let vI = (1 + 1)`，最后 `return v0`。这些是独立语句，括号 nesting=2，每个表达式只有一次加法。N=16/63 经过真实 Driver 返回 2；N=64/65/100 被门前 `av.resource.frontend-nesting` 拒绝，报告构造路径分别 129/131/201。相同输入由已有 scalar 前端得到 Checked HIR，再经真实 lower→encode→verify_abc→AV 均返回 I32(2)，分别 257/261/401 步。证据 [sibling-probes-final.log.txt](sibling-probes-final.log.txt)，生成及两条执行路径完整保存在 [probes/src/main.rs](probes/src/main.rs) 的 `sibling-probes` 模式。

这是低深度、低输入、低表项、低 fuel 的合法程序；128 是深度预算，不是整个文件最多 63 个带括号算术表达式。修复须以实际表达式构造路径保留层级关系，对同层兄弟、连续语句、不同函数取最大，同时保留长串链及真正嵌套拒绝。增加“串行/嵌套”和“兄弟/连续语句/多函数”组合控制；不能只复写原宽实参案例或提高阈值。

## F798-R9-02 [P2] 首次准备器未完整监督双流和子树，超帽仍成功、超时遗留后代

AC03/06/08，T05/08，V03/09；延续 R8-03 的资源收容/失败传播义务，属于替换启动器的覆盖缺口。

`probe/src/native_reference.rs:184..246` 新建了另一个 `Command.spawn` 监督器。stdout 设置为 piped 却没有读取；stderr 超过 1MiB 时只是 break，超限状态没有传给父线程；:219 的 `child.kill()` 只杀直接进程；:233 还把 reader 超时/缺日志用空表吞掉。声明中的“输出帽”和“预备墙杀树”并未实现为同一有界失败语义。

本轮用该函数公开的 `AV798_NATIVE_PREP_CARGO` 和 `AV798_NATIVE_PREP_WALL_MS=1500` seam，调用真实 `bounded_worker_build`。辅助程序复制自身到预期 worker 路径，仅用于隔离测试；没有替换真实 release worker，没有运行伪 worker 请求。

| 准备命令控制 | 应有结果 | 实际 |
|---|---|---|
| 正常少量 stderr，产物存在，exit0 | 成功 | 17ms，`Ok(())` |
| 试图写 2MiB stderr，exit0，产物存在 | 超过 1MiB 必须失败 | 14ms，`Ok(())`；读者关闭管道后写入错误被 helper 忽略，父侧没有超帽拒绝 |
| 仅写 128KiB stdout，exit0，产物存在 | 低于 1MiB，应成功排空 | 1506ms，误报 preparation timeout：未读 stdout 造成管道背压 |
| 父启动持续写 heartbeat 的后代后阻塞 | 期限后整棵树回收 | API 1504ms 返回 killed；随后 400ms 内 heartbeat 从 15 增到 19，后代仍运行 |

结果见 [prep-probes-final.log.txt](prep-probes-final.log.txt)。后代存活在 probe 内于 API 返回后立即采样，避免外层 shell 等待继承管道掩盖时序；随后仅终止本次 marker 记录的辅助子树，`review-owned-helper-cleanup.success=true`。较早 `prep-probes.log.txt` 的 shell 末端采样曾等到辅助进程自然结束，不是有效的即时清理证据；本次没有用它冒称杀树成功。

源程序 [prep-helper.rs](prep-helper.rs)、驱动 [probes/src/main.rs](probes/src/main.rs) 可直接重放。应复用已验证的 contained executor/有界准备设施，或实现等价的暂停创建与 Job 收容、双流排空/帽、单一截止、所有失败路径整树终止和 reader 收尾。输出超帽必须显式失败，不能截断日志后以 exit0+文件存在授成功。保留正常、工具失败/缺件、双流帽内/超帽、在途迟滞和后代持管控制。默认已有 worker 仍只查 `is_file` 的身份局限也应在同一准备合同中处理或明确要求调用前绑定；本轮显式构建仅保证此次证据使用正确二进制。

## 本轮实际验证

所有 cargo 命令从原实施树执行，使用 `--offline --locked`。无根整档 t/tv/tt/tb/tu/taa/tf 或 docs_gen。

| 实际目标 | 结果 | 证据 |
|---|---|---|
| auto-av-core `--lib` | 20/20 | core.log.txt |
| auto-av-probe `cache_tier`/`native_admission_tier`/`resources_tier` | 15+10+11=36/36 | affected-targeted.log.txt |
| auto-av-probe `abc_semantics_tier`/`binding_tier`/`case_binding`/`mir_tier`/`native_trace_parity`/`vertical` | 13+4+5+8+1+3=34/34 | other-targeted.log.txt |
| 正式 `tests::ac_av_probe_tests::`，显式 skip `ac_av_probe_formal_turnaround_gates` | 4/4 | formal-controls.log.txt |
| 既有 `tests::probe_adapter_tests::` | 16/16（实际 filter 口径，非 work 的 18） | compatibility.log.txt |
| auto-test-case `--test contract_v2` | 20/20 | contract.log.txt |
| 已保存前轮信任/缓存/前门/COFF/解码反例 | 行结果均符合已修预期 | old-probes-final.log.txt、edge-probes-final.log.txt |
| 真实 native 双 return/落空对照 | native exit1、AV=1 | native-branch-control.log.txt |
| fake前耗59000ms + 实际12000ms迟滞 linker | 1010ms 即 Budget | native-budget.log.txt |
| 两个 package 格式、完整 diff-check | exit0/0 | fmt-scoped.log.txt、diff-check.log.txt |

`cargo fmt --all` 会展开只读路径依赖并报告 auto-down 等既有格式问题；该输出原样保留 `fmt.log.txt`，随后使用 `-p auto-av-core -p auto-av-probe` 检查本计划两个 package，exit0。未把依赖格式当成本计划新 finding。根构建含已有依赖/生成代码告警，review helper 自身有两处 unused variable；修复 8 文件未新增实施告警。

可重放命令：

```powershell
# 在 D:/autostack/.wt/lang-798/auto-lang
cargo build --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --release --bin av_probe_worker --offline --locked
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core --lib --offline --locked
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test resources_tier --test cache_tier --test native_admission_tier --offline --locked -- --test-threads=1
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test abc_semantics_tier --test binding_tier --test case_binding --test mir_tier --test native_trace_parity --test vertical --offline --locked -- --test-threads=1
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r9/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target
experimental/ac-av-probe/target/debug/plan798-review-r7.exe sibling-probes
rustc D:/autostack/auto-lang/docs/plans/attachments/798/review-r9/prep-helper.rs -o experimental/ac-av-probe/target/debug/review-r9-prep-helper.exe
$env:AV798_NATIVE_PREP_CARGO=(Resolve-Path experimental/ac-av-probe/target/debug/review-r9-prep-helper.exe).Path
$env:AV798_NATIVE_PREP_WALL_MS='1500'
# 分别 normal / stderr-overflow / stdout-small / descendant；结束后清除这三个环境变量
$env:P798_PREP_MODE='descendant'
experimental/ac-av-probe/target/debug/plan798-review-r7.exe prep-probe
```

源码包 Cargo.toml 的 package 名沿用 review-r7，路径已绑定 R9 和此原实施树；不据这个名称冒称使用旧源码。锁、源文件哈希另保存于 `probe-identities.json.txt`。

## AC / V / SD 与后续

AC01 最小全路径通过；AC02/04 因 R9-01 fail；AC03 的值/数值 trap/来源/真实 native 常规对拍通过，但准备完整收容义务 partial；AC05 当前 cache/sidecar 正负控通过；AC06 因 R9-01/02 fail，V11 新提交完整独立复验 pending；AC07 正式协议及旧兼容通过，不能据此宣称全部最终验收完成；AC08 有限语义/native 准入模型通过，准备合同/SD 收口受 R9-02 阻塞；AC09 fail，AC10 merge pending。

V01/02/04/05 常规信任/CFG/codec/执行矩阵定向通过，前门例外 R9-01；V03 来源分类旧缺口关闭、真实值/trap 控制通过，准备例外 R9-02；V06..08 当前缓存/恢复/坏身份/GC/lease 控制通过；V09 fail；V10 指令/帧/取消定向通过；V11 本轮没有重跑完整 30 样本正式和六场景 release 门，已存在合同内失败时不耗另一个完整最终验收周期；V12/13 定向通过。work 的旧性能摘要不自动覆盖本次最终验收。既有前端 180–211MiB 峰值/128MiB 目标 miss 仍按已登记来源保留，没有本轮修复或新豁免主张。

SD01/02 有限 MIR/ABC/AV 与 profile 保留；SD03/04 来源分类与恢复修复认可；SD05 的最大构造深度主张因 R9-01 仍不符；SD06 协议控制通过；SD07 中冷准备输出帽/杀树主张因 R9-02 不符。现行 Spec 路径与新增路径有效，`touched_goals=[]` 的 experimental 范围解释仍成立。没有发布 canonical Spec 或改 live ledger。

多轮失败后的诊断必须从原例闭合升级到覆盖边界，不能只再改计数变量或补一条 sleep：

| 剩余不变量 | 前轮为何漏过 | 需补的设计/永久 oracle |
|---|---|---|
| 最大表达式路径，不累加兄弟 | 新正例仅未加括号的宽实参；没有连续已闭层 | 串行/嵌套与同层/多语句/多函数组合，明确表达式路径状态 |
| 准备到收尾全程有界 | 测试只断言父函数及时返回，没有排空双流、超帽失败与后代死亡断言 | 复用监督设施，帽内/超帽/持管后代/失败退出矩阵并观察实际回收 |

任务状态：保留 T01/03/06，关闭 R8-02 后恢复 T04；T02/05/07/08 重开/保持待完成，T09/10 未完成，**executing 4/10**。原树 work 修复 R9-01/02、补上述诊断及矩阵并提交后再审；本轮没有修改实施代码。

pending_closeout_gates：最终 review pass 后，merge 在冻结提交执行根裸 `cargo t` 和编译器触面 `cargo tv`，独立 workspace 完整套件/真实 native，以及实际 796/788/已合入 797 兼容；其余档按最终 diff。`tf` 仅主检出到期单实例。work-owned 新基线 V11 不可移交 merge 来冒充当前 pass。Spec/ledger、归档、wt-guard 清理均未执行。
