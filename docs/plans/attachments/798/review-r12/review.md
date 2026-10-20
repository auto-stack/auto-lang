# PLAN-798 R12：修复复审与 r4 readiness（2026-10-20）

**needs_fix**；review_scope=final（用户请求复审，实际止于旧例闭合、受影响类别/资源矩阵和 readiness，未授完整最终 pass）。当前聊天 Codex 独立于本轮实施，没有创建代理。

以主检出当前 **plan_revision=4**、§5.7/5.8、V-14/K14-01..06为合同。work 的“95测/正式30样本/无遗留”摘要不能覆盖未实现的 unit 执行、版本缓存和资源要求。四项均有具体证据；没有把825/826性能工作变成本计划门槛。

技能来源 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`，SHA-256 `0d68ba05276aff05a143c8f27010ee02b0a9bded14c27a0133d3a774c20f0cf3`；同时采用 verification-contract 的“旧例与完整不变量分离、失败路径及合法对照”规则。

## 基线

- 原树 `D:/autostack/.wt/lang-798/auto-lang`，branch `plan-798-dev`，commit `bf7e44f8ef18d9f6b9c03467986f5b91cadc58c3`；审前/结束 clean。
- diff base `13367d127584461449ee2f151c0c5987f0740044`，repair base `5def3af48689487a212b518df276e737cb62d8bd`；本轮5文件+197/-43：driver、ABC codec、两处测试及 contract-decision。
- 依赖最后修改提交：ac-core `e02c2dcc8094fc522569533304aea175f6aae867`、artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`、semantics-contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`。依赖、两份锁及根配置相对R11 diff=0；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean。
- 主检出计划、设计、Spec及配置哈希见 [baseline.json.txt](baseline.json.txt)。r4 SD冻结 [sd-frozen.md](sd-frozen.md)，SHA-256 `7ea8b7da73e31c319b35d07f17a2d97a856e14b801fd0ce2fc2149d8a5c91e15`；实现合同 [contract-decision-frozen.md](contract-decision-frozen.md)，SHA-256 `ca9747038ea95043e18f94a2aa77493e3c6243b7bb6e2102094c3d537341ae45`。
- debug/release探针身份见 [binary-identities.json.txt](binary-identities.json.txt)，源码/锁/输入/冻结负载生成器见 [source-identities.json.txt](source-identities.json.txt)。package历史名沿用review-r7，实际输入是本R12目录。没有用旧二进制冒充当前修复。

## R11具体旧例闭合

| 原finding | 当前实证 | 闭合范围 |
|---|---|---|
| R11-01 命名实参内131层漏过 | 命名/位置两态均前门resource；74/62层控制正常返回70/1；七组异实参仍合法 | 原具体输入闭合；合法低深度误拒仍有R12-01 |
| R11-02 单函数完整module变unit | MIR、手工DecodedAbc、codec后均完整module；真实Driver fresh返回42 | 类别往返原例闭合；完整r4类别/执行/旧版本矩阵尚有R12-02/03 |

见 [composed-sibling-probes.log.txt](composed-sibling-probes.log.txt)、[disjoint-probes.log.txt](disjoint-probes.log.txt)、[kind-probes.log.txt](kind-probes.log.txt)。没有重报已经修复的旧输入。

## F798-R12-01 [P2] 前门不考虑操作数位置/结合关系，真实71层合法路径被估成192层而误拒

AC02/04/06，T02/05/08，V02/09；最大构造路径不变量未完整闭合，包含本轮调用层重复计量的相邻回归。

`driver.rs:1020` 仍以 `cur_path + level_max + 1` 合成路径；:1119只保存最深子表达式，不保留它在外层串行运算中的位置。命名调用还在列表层额外+1，叠加当前单表达式计量；因此诊断中的“路径”既会重复计算调用层，也会把不在同一最大路径上的深度拼接。

复现与R11的131层反例只改变**操作数顺序**：70项左结合加法中，将60层 `idfn(x:...)` 调用放到最后一个操作数，外层仍是 `idfn(x: EXPR)`。

- 深调用位于**首操作数**：最长路径穿过69个Binary再进入深调用，真实Checked HIR=131层，应该拒绝。
- 深调用位于**末操作数**：最深左侧算术链与右侧深调用是两条分支，真实Checked HIR最长仅**71层**，应通过并返回70。

本轮独立遍历Checked HIR的Binary/Logical/Unary/Call.eval_args边测得上述层数。实际前门对两个输入都报 `token nesting 62 / expression construction path 192 exceeds ...128`。末操作数输入绕过错误前门后，经真实lower→encode→verify_abc→AV返回I32(70)，262步。3层调用/70项链及60层调用/单项低深控制都正常通过，说明不是类型/签名/运算错误。

证据 [tail-chain-probes.log.txt](tail-chain-probes.log.txt)，输入和遍历器见 [inputs/](inputs/) 与 [probes/src/main.rs](probes/src/main.rs) 的 `tail-chain-probes`。应依据真实结合关系/操作数位置计算最长构造路径、每个调用层只计一次；同时保留首操作数超深拒绝、末操作数合法、异实参/连续语句/命名与位置控制。仅修补 max/sum 切换不能证明整个路径不变量。

## F798-R12-02 [P2] wire已标明unit，公开loader仍授可执行凭证并允许带参unit执行

AC01/03/04/07/08，T01/03/07/08，K14-02/03/05；属于已批准r4类别准入义务未完成，不是因为合法kind位变化本身而拒绝。

`abc_verify.rs:100` 的 executable 仅根据“是否存在Call→Import”判断，没有结合 `is_unit` 或完成模块入口准入。直接将 unit bytes送公开 `verify_abc_bytes`，无需 driver闭包装配，便得到 `LoadedModule.executable=true`；`av.rs:90` 的 execute_entry 可进一步执行带参unit，unit路径跳过了完整module的零参i32入口要求。

| 公开入口控制 | 当前结果 |
|---|---|
| 合法完整module，零参/i32入口 | executable=true，返回7，2步（正确正控） |
| 相同载荷以真实unit类别编码，未做闭包装配 | is_unit=true，executable=true，execute返回7，2步 |
| 带一个i32参数的unit，原始bytes→loader→execute_entry([42]) | is_unit=true，executable=true，返回42，1步 |
| 相同带参载荷声明完整module | 正确拒绝 `mir.entry-sig` |

见 [unit-admission-probes.log.txt](unit-admission-probes.log.txt)。这是已经发生dispatch的成功执行，不是仅有“可验证”句柄。K14-02明确unit可验证/缓存但未闭合前不可作为完整入口执行；K14-03/05要求对应入口准入与非法dispatch=0。当前没有执行显式闭合并复查完整entry的步骤，不能仅把无import的unit称为已闭合而免掉合同。

应冻结并实现可验证unit与可执行完整闭包的转换/凭证，统一 raw bytes、手工DecodedAbc、driver和恢复入口；相应调整旧unit执行测试的oracle。现有 `core/src/av.rs` 仍有“带参unit无import即可执行”的测试/注释，与r4当前要求不一致，不能用它作新合同已通过的证据。合法module/合法kind声明的控制须继续保留。

## F798-R12-03 [P2] 新增类别语义未隔离旧产物，旧无kind单元被静默当作新module命中

AC01/03/04/05/07/08，T01/03/04/07/08，K14-04/05/06；R11-02的完整版本/cache收口仍开放。

本轮新增 `FLAG_IS_UNIT=2`，但 ABC_VERSION、ABC_SCHEMA、producer、MIR/ABC语义版本、A08/A09 schema和查询键均未变。旧格式没有该位，`flags & FLAG_IS_UNIT == 0` 不能区分旧unit与新完整module。contract-decision新段只记新增bit，未冻结旧歧义产物的失效/拒绝策略；§4头部布局文字也仍只列bit0来源。

独立按修复前wire重建旧单元：真实源码 `fn main() int { return 7 }` → split unit，移除位置sidecar，按旧writer只保留bit0，重新计算真实FNV trailer；其余bytes布局未变。这不是任意伪造新kind，模拟的是本仓前一提交曾合法持久化的无kind载荷。

- raw旧bytes仍version1，当前decoder将旧unit解成 `is_unit=false`，loader/AV成功返回7。
- 用真实ManagedStore、同owner/unit/body指纹和未变的生产信封/查询键种入该旧载荷，再走正常Driver：`cache_hit_abc=1`、`cache_miss_abc=0`、`abc_units_emitted=0`，无诊断，返回7。

证据 [legacy-cache-probes.log.txt](legacy-cache-probes.log.txt)；精确生成及真实store路径在probe的 `legacy-cache-probes`。没有覆盖现有store或替换用户产物：使用本次独立临时store并结束清理。

K14-06要求旧歧义缓存确定性miss重建或受控拒绝，不得原地当新语义读；当前不满足。应通过wire版本/可靠的格式代际或可信外层身份及cache键隔离，明确raw和受管恢复策略。**不能改成所有flags=0一律拒绝**：r4明确合法完整module的kind声明必须按完整规则判定；缺的是新旧语义的可区分身份，不是“翻位即非法”的oracle。

## F798-R12-04 [P2] r4明确要求的128MiB当前资源门未达，不能按历史归因或性能绿豁免

AC01/06/08，T01/05/08，V09..11及§5.8资源政策；属于现有合同要求的当前未达证据，不是新增内存上限。

本轮在**独立新release进程**，从空载开始以2ms采样库读取同一进程private-bytes峰值；计量包含输入生成、真实Driver前端/HIR/索引/分配/临时及AV结果，不扣前端。使用冻结 `load256::generate()`，没有缩小负载：

| 量 | 当前实测 |
|---|---:|
| 源码 | 273575 bytes（约267KiB） |
| 独立expected / 实际结果 | 2665 / 2665，无诊断 |
| 空载private-bytes | 761856 |
| 采样private-bytes峰值 | 211918848 |
| 相对空载峰值增量 | **211156992 bytes，约201.4MiB** |
| 冻结要求 | 134217728 bytes，128MiB |
| target_met | **false** |

证据 [memory-probe.log.txt](memory-probe.log.txt)、[probe-release-build.log.txt](probe-release-build.log.txt)，实际release二进制哈希见身份文件。此为一次当前反例测量，不声称它证明全部30样本性能或32MiB受管分区。冻结生成器文件哈希也已保留，输入可按同提交确定性重建。

历史汇总仍为 `worker_private_bytes_target_met=false`，而 `hard_gate_failures=[]`，见本目录冻结的 [historical-turnaround-summary.json.txt](historical-turnaround-summary.json.txt)；该历史材料只作归因对照，当前判断来自上面新测量。r4§5.8已明确历史约211MiB前端归因/债务不构成豁免，当前资源AC不可因此完成。

按合同由T01给出具体冻结输入、只读ac-core owner阻断和有界修订方案；能在授权实现内收敛则补当前证据，否则先回new解决政策/依赖可行性。不得直接改只读ac-core、扩大128数字、排除前端/子阶段、或只拿编辑P95绿关闭资源AC。登记此项不批准延后。

## 实际测试、readiness和路由

四条精确定向测试各1/1通过：`abc_kind_survives_roundtrip`，`av_resources_composed_path_inside_one_argument`，`av_resources_sibling_max_paths_not_concatenated`，`av_resources_sequential_shallow_statements_not_depth_summed`。分别使用 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test <abc_semantics_tier或resources_tier> --offline --locked <名称> -- --exact --test-threads=1`，输出以测试名保存在本目录。没有拿work95或旧132冒充当前测试总数。

所有probe从原树offline/locked构建；debug运行旧例和类别/store反控，release独立进程运行资源反例。两package fmt、完整diff-check均exit0。既有依赖告警及probe附件自身两处unused变量保留在日志，没有本次新增实施告警。未跑整档cargo t/tv/tt/tb/tu/taa/tf/docs_gen；readiness已失败，没有重复完整V11和整套最终验收。K14未实测部分继续pending，不外推全矩阵pass。

重放：

```powershell
# cwd = D:/autostack/.wt/lang-798/auto-lang
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r12/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target
$env:P798_INPUT_ROOT='D:/autostack/auto-lang/docs/plans/attachments/798/review-r12/inputs'
experimental/ac-av-probe/target/debug/plan798-review-r7.exe tail-chain-probes
experimental/ac-av-probe/target/debug/plan798-review-r7.exe unit-admission-probes
experimental/ac-av-probe/target/debug/plan798-review-r7.exe legacy-cache-probes
Remove-Item Env:/P798_INPUT_ROOT
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r12/probes/Cargo.toml --release --offline --locked --target-dir experimental/ac-av-probe/target
experimental/ac-av-probe/target/release/plan798-review-r7.exe memory-probe
```

AC01类别/版本政策未闭；AC02/04因前门与unit准入fail；AC03有限数值和原42例有进展，但K14执行合同fail；AC05旧缓存策略fail；AC06当前201.4MiB及前门fail，完整V11pending；AC07 K14正式路径矩阵pending；AC08合同/coverage/资源收口未闭；AC09fail，AC10merge pending。SD01保留既有有限实现边界；SD02/03的kind/执行/版本策略未闭；SD05的最大深度仍不符，SD06完整K14接线证据pending；其余保留原边界。规范路径和experimental touched_goals=[]说明有效，未发布canonical Spec或改live ledger。

保留仅T06完成，T01/02/03/04/05/07/08待work，T09/10未完成，**executing 1/10**。当前frontmatter8/10与唯一任务表不一致，按实际任务/证据归一为1。原树work完成四项及r4完整readiness后提交再审；语义合同未由本次review修改，plan_revision保持4。

pending_closeout_gates：最终review pass之后，merge在冻结提交执行根裸cargo t、编译器cargo tv、独立workspace全套/真实native及实际796/788/已合入797兼容，其余按diff；tf仅主检出到期单实例。work-owned V11/V14/128MiB要求不可改称merge-owned来授当前pass。Spec/ledger、归档、guard清理仍pending。本轮未修实现、合入、归档或移除任何worktree。
