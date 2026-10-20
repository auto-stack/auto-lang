# PLAN-798 R13：r5修复定向复审与readiness（2026-10-20）

**needs_fix**；review_scope=final（用户请求复审，实际止于修复反例、受影响路径及readiness，未授完整最终pass）。本聊天Codex独立于实施、未创建代理。

以主检出 **plan_revision=5** 为准。当前§10明确T01/05/07/08的完整K14矩阵与30样本正式验收仍pending；本轮不拿工作摘要的“修复完毕/100测”替代这些义务。r5用户批准的方案C双线内存政策按§5.9执行，历史r4单线128MiB失败记录保留。

技能 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`，SHA-256 `0d68ba05276aff05a143c8f27010ee02b0a9bded14c27a0133d3a774c20f0cf3`；采用 verification-contract 的组合/合法对照/证据边界规则；没有因r5修订重置既有复审历史。

## 基线

- 原树 `D:/autostack/.wt/lang-798/auto-lang`；branch `plan-798-dev`；commit `06114f4316468030f1b160bb028046b9e5beab71`，审前/结束clean。
- base `13367d127584461449ee2f151c0c5987f0740044`，repair base `bf7e44f8ef18d9f6b9c03467986f5b91cadc58c3`。中间修复 `00bd25ddf`，新阶段T11 `06114f431`；累计13文件+898/-235。
- 依赖最后修改提交：ac-core `e02c2dcc8094fc522569533304aea175f6aae867`、artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`、semantics-contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`。依赖、两份锁及根配置相对R12 diff=0；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean。
- 计划/设计/Spec/锁/依赖tree见 [baseline.json.txt](baseline.json.txt)。SD冻结 [sd-frozen.md](sd-frozen.md) SHA-256 `7ea8b7da73e31c319b35d07f17a2d97a856e14b801fd0ce2fc2149d8a5c91e15`；r5资源政策另冻结 [memory-policy-frozen.md](memory-policy-frozen.md) SHA-256 `ba1ac371a5fb6d507ac1596fc1beede77e4b64930cf8c730a05c50d0978f4a9d`；实现合同 [contract-decision-frozen.md](contract-decision-frozen.md) SHA-256 `fa8ca27b6d48b1f1452469dbfb41ac93f43683b883a7c519470825bbcb223243`。
- 两个实际二进制及源码/锁/精确输入身份见 [binary-identities.json.txt](binary-identities.json.txt)、[source-identities.json.txt](source-identities.json.txt)。当前正控使用公开ABC_VERSION=2；旧缓存负控仍明确写version1并重算FNV，不把v2合法kind翻位当旧版本错误。package历史名review-r7不表示使用旧源。

## R12四项的本轮闭合

| R12 finding | 本轮独立证据 | 结论 |
|---|---|---|
| 01 合法尾操作数71层误拒 | 深调用位于尾部：真实HIR71，Driver=70；移到首部：HIR131，前门resource；原命名/位置131层、异实参及低深控制均符合预期 | 原误拒闭合；新前缀组合遗漏见R13-01 |
| 02 unit直接执行 | raw零参/带参unit均 `executable=false`，执行入口返回unit-kind Infra；完整module正常返回7，带参module正确entry-sig拒绝 | 当前受影响旧矩阵闭合；不外推完整K14全部入口 |
| 03 无kind旧缓存静默命中 | raw v1拒绝 `abc.decode.version`；真实store种入v1载荷，Driver返回7且 hit0/miss1/emitted1；永久测试双次重放通过 | 旧例与本轮版本隔离控制闭合 |
| 04 单线128MiB政策未达 | 用户批准r5方案C替代单线；当前release六场景×1测得A≈201.6MiB、B≈3.6MiB，两线硬门及越界/缺归因反控通过 | 按获准r5政策关闭该阻塞；未声称旧总峰值已低于128MiB |

结果见 tail-chain / composed-sibling / disjoint / kind / unit-admission / legacy-cache 的各 `.log.txt`。既有程序/expected保留，只做新wire版本所需适配。没有重报R12具体反例。

## F798-R13-01 [P2] 调用前的一元前缀被当作callee名字的一部分弹掉，真实131/132层路径仍漏过128层预算

关联AC02/04/06、T02/05/08、V02/09、SD05；本轮LevelChain重写引入的组合路径漏算。128层合同不变，没有新增规模要求。

根因：`driver.rs:1136` 的 operand 在读取普通标识符时用 `mem::take(&mut self.unary)` 消费前缀深度；:1326..1328把callee标识符按这个规则压入vals并标记ident_top；随后 :1249..1252 看到 `(` 将callee值弹掉，却未恢复已消费的前缀。闭层 :1267 再调用operand(path)时已经没有这些前缀，因而 `-f(...)`/`!f(...)` 的一元节点从路径计量中消失。普通分组 `-(...)` 没有该callee弹出路径，故会正确保留前缀。

最小生成程序：`idfn(x int) int { return x }`；main返回 **20个一元负号 + 110层命名idfn调用 + 字面量1**。从真实Checked HIR的Unary和Call.eval_args边迭代测得最长构造路径**131层**，括号nesting≤112，仍在前门token括号预算内。预期门前 `av.resource.frontend-nesting`；实际 `front_door_ok=true`，Driver执行返回1。

| 区别性控制 | Checked HIR最长路径 | 实际Driver |
|---|---:|---|
| 5个负号 + 110层调用 | 116 | 合法返回-1 |
| 20个负号 + 3层调用 | 24 | 合法返回1 |
| **20个负号 + 110层调用，不加额外分组** | **131** | **放行，返回1** |
| 同一131层语义，仅将整个调用包一对括号 | 131 | 正确门前resource（估计132） |
| 布尔对照：20个`!` + 110层bool调用，不分组 | 132 | **放行，main返回1** |
| 相同布尔表达式加一对分组括号 | 132 | 正确门前resource |

整数独立直接管线正好执行242步，带/不带额外分组的HIR深度相同；没有把前门resource误当语言trap。布尔HIR常量展开产生额外一层，所以实测132而非猜测131。未声称本例已经造成宿主栈溢出。

证据 [prefix-call-probes.log.txt](prefix-call-probes.log.txt)，精确输入在 [inputs/](inputs/)，生成、独立HIR遍历与Driver路径保存在 [probes/src/main.rs](probes/src/main.rs) 的 `prefix-call-probes`。

应使一元前缀绑定到**完整调用表达式**，callee名字撤回不能吞掉它；普通局部标识符和调用需分别处理。永久覆盖 `-`/`!`、callee与普通operand、带/不带分组、低深通过/超深拒绝，且保留R12的左右结合与异实参控制。新precedence模型确已改善之前的最大路径问题；此次遗漏说明诊断与oracle还需覆盖prefix×call的关联，而不是再次调高预算或逐例放行。

## r5双线内存的独立证据

按主检出§5.9已记录的用户方案C执行：A前端隔离峰≤224MiB；B=`max(worker peak, child peak)-A`≤128MiB；缺归因整峰计B。该政策是本次明确批准的合同修订，不以r4旧上限重新判当前数据。

本轮显式offline/locked构建当前release worker，再执行 `bench-turnaround --samples 1 --gate-ms 1000 --out <本目录raw>`：

| 项目 | 当前实测 |
|---|---:|
| 冻结负载 | 与生成器相同的273575B输入 |
| A：frontend_peak_bytes | 211423232B，201.6289MiB ≤224 |
| overall private peak | 215154688B |
| B：overall−A | 3731456B，3.5586MiB ≤128 |
| 两个target_met | true / true |
| 六场景原始行 | 6（各1样本） |
| hard_gate_failures / exit | [] / 0 |

见 [memory-smoke-summary.json.txt](memory-smoke-summary.json.txt)、[memory-smoke-raw.jsonl](memory-smoke-raw.jsonl)、[worker-build.log.txt](worker-build.log.txt)。`av_turnaround_two_line_memory_gate` 的A越/B越/双过/缺归因保守四态通过，原始输出 [memory-gate-controls.log.txt](memory-gate-controls.log.txt)。这足以核实此次政策接线和代表性当前数据，**不代替合同要求的六场景≥30样本及正式AutoDown完整V11**；后者在当前共享计划本来就标pending。

## 测试及证据边界

| 实际目标 | 本轮结果 | 输出 |
|---|---|---|
| core `--lib` | 21/21 | core.log.txt |
| probe `--test abc_semantics_tier` | 15/15 | abc-cache-tests.log.txt |
| probe `--test cache_tier` | 16/16 | abc-cache-tests.log.txt |
| 精确 `av_turnaround_two_line_memory_gate` | 1/1（四态） | memory-gate-controls.log.txt |
| 精确 `av_cache_kindless_v1_wire_isolated` | 另行重放1/1；已计入cache16，不重复加总 | legacy-cache-test.log.txt |
| 两package fmt / 完整计划diff-check | exit0 / 0 | fmt.log.txt、diff-check.log.txt |

合计53项不同定向测试通过。所有cargo操作从原实施树offline/locked执行；没有根整档t/tv/tt/tb/tu/taa/tf/docs_gen，未重跑完整30样本来掩盖readiness或当前新finding。新增实施告警未见；已知依赖及审查附件自身两处unused变量原样在构建日志中。

probe模式：tail-chain-probes、composed-sibling-probes、disjoint-probes、kind-probes、unit-admission-probes、legacy-cache-probes、prefix-call-probes均在本轮实际运行。源码包hash和输入hash保存，不用work100总数冒充本轮。

重放核心反例：

```powershell
# cwd = D:/autostack/.wt/lang-798/auto-lang
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r13/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target
$env:P798_INPUT_ROOT='D:/autostack/auto-lang/docs/plans/attachments/798/review-r13/inputs'
experimental/ac-av-probe/target/debug/plan798-review-r7.exe prefix-call-probes
Remove-Item Env:/P798_INPUT_ROOT
```

## AC / SD / 路由

AC02/04/06因R13-01 fail；AC03/05在本轮类别/旧缓存控制有实证进展，完整K14尚按T01/07/08 pending；AC01政策与K14冻结/完整readiness仍未齐，AC07完整正式路径/30样本pending；AC08汇总和前门SD05未闭；AC09 fail，AC10 merge pending。r5 T11双线政策本轮定向通过；保留T03/04/06/11完成，重开T02，T01/05/07/08原已pending，T09/10未完，**executing 4/11**。

SD02/03的unit/v2与缓存修复在本轮范围认可；SD03最终沉淀须按§5.9写A/B硬门并清理被取代的单线表述。SD05受前缀漏算阻塞；其它delta保留既有审核边界。规范路径及experimental touched_goals=[]说明有效。没有修改canonical Spec/ledger或在review中改变语义合同，plan_revision仍5。

原树work修复prefix×callee状态并补永久区别性oracle，完成T01/05/07/08已列明的K14完整矩阵与正式30样本readiness后提交再审；不能把这些work-owned项移到merge来授当前pass。若实施只申请修复局部复核，当前记录只说明上表闭合范围，不授整个计划最终pass。

pending_closeout_gates：最终独立review pass之后，merge在冻结提交统一根裸cargo t、编译器cargo tv、独立workspace全套/真实native及实际796/788/已合入797兼容，其余按最终diff；tf只在主检出到期单实例。Spec/ledger、归档和guard清理pending。未修实施、合入、归档或清理树。
