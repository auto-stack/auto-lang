# PLAN-798 R11：修复定向复审与 r3 readiness（2026-10-20）

**outcome=needs_fix**。review_scope=final（用户请求最终复审；实际止于修复反例与当前 r3 readiness，未授完整最终 pass）。主审为本聊天 Codex，未参与实施、未创建代理；按实际代码与重新运行的探针判断。

当前主检出已是 **plan_revision=3**，§5.7、V-14、唯一任务表和§10明确写着类别/版本合同尚未实现。`5def3af48` 的 work 收据针对 r2 前门修复，不能覆盖 r3。遵循 review 技能的 readiness 规则，已知 work-owned 要求未完成时不再耗一轮完整最终验收；本次仍按用户明确请求验证旧修复及相邻路径，并将 r3 静态线索落为实际反例。

技能 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` SHA-256 `0d68ba05276aff05a143c8f27010ee02b0a9bded14c27a0133d3a774c20f0cf3`；应用 `auto-plan-new/references/verification-contract.md` 和本仓门禁时序。

## 基线与旧例闭合

- 原树 `D:/autostack/.wt/lang-798/auto-lang`；branch `plan-798-dev`；commit `5def3af48689487a212b518df276e737cb62d8bd`；审前/结束均 clean。
- diff base `13367d127584461449ee2f151c0c5987f0740044`，repair base `f24bd46a6f84286c30c90b156d9b4e7d72ad1358`；修复仅 driver.rs 和 resources_tier.rs，+115/-16。
- 依赖最后修改提交：ac-core `e02c2dcc8094fc522569533304aea175f6aae867`、artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`、semantics-contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`。依赖、两份锁及根配置相对 R10 diff=0；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean。
- 计划/设计、canonical Spec、锁、依赖 tree 身份见 [baseline.json.txt](baseline.json.txt)。主检出三个现行 Spec 与实施树一致；当前设计和 r3 计划版本分别取自主检出，没有把实施树旧计划当合同。
- r3 规范增量冻结 [sd-frozen.md](sd-frozen.md)，SHA-256 `8d4c3675098bf5aa06979c2e409cece526d35b21ee148e15e8e81038fb75f1b5`；现有实现合同 [contract-decision-frozen.md](contract-decision-frozen.md) 仍为 SHA-256 `47e65a126ae2bd19b27ceb403185acdd2f02b29f33496f6c65eecd3420386f39`，尚无 r3 kind/版本收口。
- 实际探针二进制、源码/锁/输入身份见 [binary-identities.json.txt](binary-identities.json.txt)、[source-identities.json.txt](source-identities.json.txt)。probe package 名历史沿用 review-r7，当前编译输入是本 R11 目录。

R10 的 **7 组异路径兄弟组合全部通过**，值与直接 HIR→MIR→ABC→AV一致；60括号/70项链两序分别 1/70。原连续浅语句 16/63/64/65/100 全部 Driver=2。同路径“60 层调用内包 70 项链”仍门前 resource，3 层对照返回70。见 [disjoint-probes.log.txt](disjoint-probes.log.txt)、[sibling-probes.log.txt](sibling-probes.log.txt)。这关闭 R10 的具体误拒例，但完整构造路径预算因下一项尚未闭合。

## F798-R11-01 [P2] 同一个命名实参内的两段路径被取 max，131 层实际构造仍获准执行

AC02/04/06，T02/05/08，V02/09；属于修复 R10-01 引入的**漏准入回归**，原 128 层预算未变。

`driver.rs:1018..1024` 的 `close_candidate` 在 `had_comma` 下使用 `max(cur_path, own)+1`。:1151 将命名冒号也当作兄弟列表标记；:1160 的 sibling 折叠只计算 bare chain，丢失该**同一实参内部**嵌套子表达式与外层串行链的关联。修复避免拼接不同 sibling，却也拆开了一条真实路径。

独立程序生成：定义 `idfn(x int) int { return x }`；main 返回外层 `idfn(x: EXPR)`。EXPR 是 **60 层 `idfn(x: ... )` 包住字面量1，放在70项左结合加法链的第一个操作数**，即 `nested60(1) + 1 + ... + 1`。两部分属于同一个命名实参，最深路径贯穿69个 Binary、60个内部 Call、外部 Call及叶子。

本轮从真实 Checked HIR 的 ExprKind::Binary/Logical/Unary/Call.eval_args 边做独立迭代遍历，最长构造路径 **131 层**；不是仅由 token 估算的猜测。正常前门却 `front_door_ok=true`，Driver 实际返回70，无 resource 诊断。把**外层**调用改成位置实参，Checked HIR 仍是131层，前门则正确报 `av.resource.frontend-nesting`（估计132）。因此这不是要求新的规模保证或把合法浅输入改为拒绝：同一个既有构造预算按实参写法出现绕过。

| 控制 | Checked HIR 最长路径 | 前门/Driver实际 | 应有结果 |
|---|---:|---|---|
| 内部3层调用 + 70项链，外层命名实参 | 74 | 通过，70 | 通过 |
| 内部60层调用 + 单项，外层命名实参 | 62 | 通过，1 | 通过 |
| 内部60层调用 + 70项链，外层命名实参 | **131** | **通过，70** | 前门 resource |
| 同上，外层位置实参 | 131 | resource | resource |

精确输入在 [inputs/](inputs/)，驱动与 HIR 测量保存在 [probes/src/main.rs](probes/src/main.rs) 的 `composed-sibling-probes`，结果 [composed-sibling-probes.log.txt](composed-sibling-probes.log.txt)。本例没有栈溢出/宿主 panic；不声称已经造成它们。缺陷是冻结前门资源准入不成立。

应区分“当前 sibling 的完整组合路径”与“已完成 siblings 的最大路径”，先在每个表达式内部组合，再跨 siblings 取 max；命名冒号本身不应抹掉实参内部的组合关系。修复同时保留 R10 异路径合法控制、此次同路径拒绝、参数顺序、命名/位置等价和真低深度控制。现有新测试只覆盖异路径分离与把链整体包进括号的组合，漏了“深调用作为左结合链首操作数”的路径；多轮诊断应覆盖两种 AST 方向。

## F798-R11-02 [P2] r3/V-14 仍未实现：单函数完整 module 经 codec 变 unit，合法返回42的程序被拒绝

AC01/03/04/05/07/08，T01/03/04/07/08，V-14，SD02/03。性质是 **r3 已明确重开的 work-owned 正确性收口缺口**，不是这两文件修复新引入，也不把已声明未完成的工作说成“修复回退”。

按 §5.7 的具体原公开入口实际执行：

```text
fn main() int { if false { return main() } else { return 42 } }
```

| 阶段 | 本轮实际结果 |
|---|---|
| compile_scalar_source → lower_checked | 一函数完整 MIR，`is_unit=false`，verify_mir成功 |
| 正确完整 MIR 包入公开 DecodedAbc → verify_abc → AV | admitted=true，Success I32(42)，6步 |
| 同一 MIR encode_abc → decode_abc | `is_unit` 变成 **true**，仍只有一个函数 |
| 解码结果 → verify_abc | `mir.unit-callee: a unit must address callees through imports` |
| 真实 Driver fresh 请求 | `stage=abc-verify`，`abc verify: mir.unit-callee`，无结果 |

根因仍是 `core/src/abc.rs:683` 的 `is_unit: n_funcs == 1`；wire没有可靠保留工件类别。见 [kind-probes.log.txt](kind-probes.log.txt)、[single-module-self-call.at](inputs/single-module-self-call.at)。本轮把计划原先的静态线索变成实际跨公开入口的正控/反例，没有改 expected 或加 dummy 函数。

T01先冻结 unit/module 的 entry、imports、执行条件；T03修复类别往返和公开授信入口；若 wire/持久语义变化，T04按 r3 冻结版本、producer/cache键及旧歧义产物策略。完成 V-14 的 fresh/hit/新 worker restart、非法 kind/entry/import/旧版 dispatch=0 控制。上述后续路径本轮未完整执行，不能从当前 fresh失败或手工正控外推通过。825/826 的运行表示/紧凑格式性能工作仍不是本计划新增门槛。

## 实际验证与范围

在原实施树使用 offline/locked 编译本目录 probe，并执行上述 disjoint/composed/kind/sibling 模式。三条既有精确定向测试分别通过1项：

- `av_resources_sibling_max_paths_not_concatenated`
- `av_resources_sequential_shallow_statements_not_depth_summed`
- `av_resources_bool_and_named_arg_chains_front_door`

实际命令为 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test resources_tier --offline --locked <精确名称> -- --exact --test-threads=1`；逐条日志按测试名称保存在本目录。没有拿上轮132或work93的总数冒充本轮执行。两package fmt与全计划diff-check均exit0；新增实施告警未见，已知依赖告警/审查附件自身 unused变量原样保留于构建输出。

重放入口：

```powershell
# cwd = D:/autostack/.wt/lang-798/auto-lang
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r11/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target
$env:P798_INPUT_ROOT='D:/autostack/auto-lang/docs/plans/attachments/798/review-r11/inputs'
experimental/ac-av-probe/target/debug/plan798-review-r7.exe composed-sibling-probes
experimental/ac-av-probe/target/debug/plan798-review-r7.exe kind-probes
Remove-Item Env:/P798_INPUT_ROOT
```

由于 r3 明确仍有 work-owned 缺口，本轮没有运行完整 V11、根整档或独立 workspace 整套，也没有重跑未变的 native 准备设施。R10 关于 R9-02 的关闭证据可在 native_reference/依赖/配置内容未变的范围保留；不把它扩成 r3 完整验收 pass。

## AC / SD / 状态与后续

AC01/03 的 V-14 fail，AC04 同时受类别和前门影响；AC02/06 因 R11-01 fail；AC05/07 的 r3版本缓存/多入口验收 pending，不能靠旧r2绿授pass；AC08 r3类别/版本及前门规范增量未闭；AC09 fail，AC10 merge pending。未受影响的 T06语义ref证据保留。SD02/03必须补实际kind/版本事实；SD05资源前门尚不成立；其余delta保留既有审核边界。Spec路径有效、experimental 的 touched_goals=[]解释保留；未发布canonical Spec/ledger。

保持r3已经重开的T01/03/04及待验收T07/08，重开此次有反例的T02/05；仅T06完成，**executing 1/10**。按原树work修复两项并完成r3 readiness与区别性oracle后再送审。语义合同未由本次review修改，plan_revision保持3；历史r2/r3收据均保留。

pending_closeout_gates：最终代码/验收/规范review通过后merge在冻结提交统一根裸cargo t、编译器cargo tv、独立workspace完整套件/真实native及实际796/788/已合入797兼容；其它档按最终diff；tf只到期在主检出单实例。work-owned V11/V14不可后置冒称当前pass。Spec/ledger、归档、guard清理仍pending。未修实施、合入、归档或清理树。
