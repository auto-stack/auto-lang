# PLAN-798 R10 修复复审（2026-10-20）

**needs_fix**；review_scope=final（旧例闭合及受影响验收；未授完整最终 pass）。

当前聊天 Codex 延续独立复审，未参与此次实施、未创建代理。依据主检出计划 r2、原实施树源码和重新执行的反例判断；work 的 8/10 及“全部闭合”摘要不作为验收证据。

技能来源 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md`，SHA-256 `0d68ba05276aff05a143c8f27010ee02b0a9bded14c27a0133d3a774c20f0cf3`；并应用同级 new 的 `references/verification-contract.md`，遵守本仓 2026-10-14 门禁时序。

## 基线

- 原树 `D:/autostack/.wt/lang-798/auto-lang`，分支 `plan-798-dev`；提交 `f24bd46a6f84286c30c90b156d9b4e7d72ad1358`；审前/审后 clean。
- base `13367d127584461449ee2f151c0c5987f0740044`；repair base `e00340842fdca15417c3195041ca909b1fcb8bb1`；本次实际 4 文件 +221/-61，只有 driver/native_reference 与两处测试。
- ac-core 最后修改提交 `e02c2dcc8094fc522569533304aea175f6aae867`、artifact-probe `37db1a07eef2182773e7b952684da16ece40658b`、semantics-contract `243a79f49dce3c36905f02e980fd36f4603dbf8d`。依赖、两份锁、根配置相对 R9 diff=0；组内 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` clean。
- Spec、输入、内容 tree 等见 [baseline.json.txt](baseline.json.txt)，结束复核见 [final-baseline.json.txt](final-baseline.json.txt)。主检出三个现行 Spec 与实施树哈希一致。
- 规范增量冻结 [sd-frozen.md](sd-frozen.md)，SHA-256 `4b6582965b882a0b46e6ac1b20fdfb14726d72a523628332ee59d50b5c529b3e`；[contract-decision-frozen.md](contract-decision-frozen.md) SHA-256 `47e65a126ae2bd19b27ceb403185acdd2f02b29f33496f6c65eecd3420386f39`。
- 显式离线锁定构建当前 release worker：[worker-build.log.txt](worker-build.log.txt)。实际二进制、反例源码/锁/输入身份分别见 [binary-identities.json.txt](binary-identities.json.txt)、[source-identities.json.txt](source-identities.json.txt)。probe package 名沿用 review-r7，当前源码位于本 R10 目录，不混用旧 binary。

## 上轮闭合范围

| finding | 本轮独立结果 | 结论 |
|---|---|---|
| R9-01 连续浅语句误计深度 | 原 16/63/64/65/100 条 `let vI=(1+1)` 全部 Driver=2；原宽参 16/128/129/200 全部=1；长链 129/400/600 仍门前 resource | 旧例闭合；不同 sibling 的深度仍拼接，R10-01 开放 |
| R9-02 准备器双流/失败/子树 | 双流采用 ac-core contained executor；帽内 stdout 正常、双流超帽失败、超时后代消失；非零退出/缺产物拒绝 | 本轮受影响完整监督路径通过，关闭 R9-02 |
| 更早 NegI32 / native 剩余截止等 | NegI32 诚实恢复带真实 span，空映射拒绝；迟滞 linker 在约 1054ms 被剩余 1s 预算拒绝；真实 native 双 return 与落空对照均 exit1/AV1 | 未发现回退，不重报旧例 |

## F798-R10-01 [P2] 从不同实参取得的两个最大值仍相加，伪造不存在的深路径

关联 AC02/04/06，T02/05/08，V02/09，SD05；原 R9-01 最大构造路径不变量未完整闭合，属于修复后的相邻误拒。没有要求新语法或扩大预算。

`probe/src/driver.rs:1089` 用 `cur_path + level_max + 1` 计算闭层路径。:1094 把子层路径保留到父层 `cur_path`；:1125 的逗号/命名冒号只清 `level_run`，留下 `cur_path` 和 `level_max`。当这两个最大值来自不同实参时，仍被相加成一条路径。

独立输入：

```text
fn first(a int, b int) int { return a }
fn main() int {
    return first(a: <60对括号包住字面量1>, b: <70个1用+连接>)
}
```

这是两个兄弟表达式：第一个有 60 层括号、没有串行运算；第二个有 69 个加法、没有该嵌套。源码 nesting=62，两条构造路径各自低于 128；不存在“60 层里再接 70 项链”的路径。预期 Driver 返回 1。

实际前门却报 `av.resource.frontend-nesting`，声称 `token nesting 62 / expression construction path 131`。相同输入由现有 scalar 前端得到 Checked HIR，再经真实 lower→encode→verify_abc→AV 返回 I32(1)，143 步。调换两个实参的表达式，直接管线正确返回 70/143 步，前门仍误拒 131。另有 60/69 与 40/90 组合分别误报 130/131；都远低于输入、表项、验证步数、执行预算。

区别性控制同时执行：

| 组合 | 实际 Driver | 预期 |
|---|---|---|
| 第一个实参 3 层括号，第二个 70 项链 | 1 | 成功 |
| 第一个 60 层括号，第二个字面量 | 1 | 成功 |
| 第一个字面量，第二个 70 项链 | 1 | 成功 |
| 第一个 60 层括号，第二个 70 项链 | resource 131 | 成功=1 |
| 上项调换表达式 | resource 131 | 成功=70 |
| **同一条路径**：3 层 idfn 调用内放 70 项链 | 70 | 成功 |
| **同一条路径**：60 层 idfn 调用内放 70 项链 | resource 130 | 预算拒绝 |

证据：[disjoint-probes.log.txt](disjoint-probes.log.txt)。精确程序保存在 [inputs/](inputs/)；生成和独立管线源码在 [probes/src/main.rs](probes/src/main.rs) 的 `disjoint-probes`。这证明本项是 sibling 关联丢失，不是要求放开同路径真实超深。

修正应在 sibling 结束时先计算**这个表达式自身的完整路径**，再对兄弟取最大；不能把来自不同 sibling 的“最大子层”与“最大串行链”组合。父表达式的串行运算上下文需保留。为同路径组合与异路径组合、参数顺序、连续语句及跨函数保留区别性断言；不要通过只降低 `cand` 或提高阈值使真实超深放行。

本轮 work 确实调整了状态模型并补了组合测试，但 `resources_tier.rs:226..247` 的新组合仅把嵌套与长链放在同一路径；没有检验两轴分别落在不同 sibling 的情况。多轮失败诊断仍需补这个覆盖边界，原最大路径义务不变。

## R9-02 的独立关闭证据

复用 R9 的原 `prep-helper.rs` 四模式，实际调用新 `bounded_worker_build`，短墙 1500ms；helper 只写到独立临时目录的假产物路径，没有替换真实 release worker。

| 控制 | 本轮实际结果 |
|---|---|
| 正常少量 stderr、exit0、产物存在 | 110ms，Ok |
| 2MiB stderr、exit0、产物存在 | 52ms，Err stream cap |
| 128KiB stdout，低于帽 | 50ms，Ok，无背压误超时 |
| 父阻塞且后代持续写 heartbeat | 1513ms，Err tree killed；API 返回后 400ms heartbeat 15→15；后续 OS 查询 PID5448 不存在 |
| 额外：2MiB stdout | 109ms，Err stream cap |
| 额外：命令 exit7，即使产物存在 | 49ms，Err worker build failed |
| 额外：exit0 但缺产物 | 63ms，Err produced no binary |

见 [prep-probes.log.txt](prep-probes.log.txt)、[descendant-os-check.log.txt](descendant-os-check.log.txt)、[prep-extra-controls.log.txt](prep-extra-controls.log.txt)。本轮 probe 的测试后清理只在 heartbeat 继续增长、证实本次后代还存活时才调用，避免对已经退出的 PID 再执行清理；本轮没有触发它。

已有 worker 新增 10s 有界功能探测，真实 worker 的 native 正控通过。此探测仅校验 helper 子命令可用，不宣称它等价于源码/依赖内容哈希；本轮证据通过显式构建和实际二进制哈希绑定当前代码。

## 本轮命令与测试结果

全部构建/测试从原实施树离线、锁定执行。没有整档 cargo t/tv/tt/tb/tu/taa/tf 或 docs_gen。总计 **132 项定向测试通过**；独立反例另列，不把反例发现记作测试绿。

| 目标 | 实际结果 | 原始输出 |
|---|---|---|
| auto-av-core `--lib` | 20/20 | core.log.txt |
| auto-av-probe：native_admission_tier / resources_tier | 11+12=23/23 | affected-targeted.log.txt |
| probe：abc_semantics_tier / binding_tier / cache_tier / case_binding / mir_tier / native_trace_parity / vertical | 13+4+15+5+8+1+3=49/49 | other-targeted.log.txt |
| 正式 ac_av_probe_tests，显式 skip formal_turnaround_gates | 4/4 | formal-controls.log.txt |
| 既有 probe_adapter_tests | 16/16（实际 filter 口径） | compatibility.log.txt |
| auto-test-case contract_v2 | 20/20 | contract.log.txt |
| 旧信任/缓存/解码/COFF/前门反例 | 已修具体旧例均符合预期 | old-probes.log.txt、edge-probes.log.txt、sibling-probes.log.txt |
| native 剩余截止、双 return/落空 | Budget≈1054ms；两程序 native=AV=1 | native-budget.log.txt、native-branch-control.log.txt |
| 两 package fmt、完整 diff-check | exit0/0 | fmt.log.txt、diff-check.log.txt |

根/依赖的已有构建告警与 R9 相同；本次四文件未新增实施告警。审查 helper 中两处 unused variable 属附件自身，不冒称实施告警。

关键重放命令（cwd 为原实施树）：

```powershell
cargo build --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r10/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target
$env:P798_INPUT_ROOT='D:/autostack/auto-lang/docs/plans/attachments/798/review-r10/inputs'
experimental/ac-av-probe/target/debug/plan798-review-r7.exe disjoint-probes
Remove-Item Env:/P798_INPUT_ROOT
```

测试分别使用根或 `--manifest-path experimental/ac-av-probe/Cargo.toml` 的 `cargo test -p <package> --lib/--test <上表目标> --offline --locked`；probe 目标附 `-- --test-threads=1`。正式协议 filter 为 `tests::ac_av_probe_tests:: -- --test-threads=1 --skip ac_av_probe_formal_turnaround_gates --nocapture`，兼容 filter 为 `tests::probe_adapter_tests:: -- --test-threads=1`。准备 helper 运行方式沿用 R9 报告，将源码/二进制路径改为本目录及 `review-r10-prep-helper.exe`；额外三控制使用 `prep-extra-helper.rs`/`review-r10-prep-extra-helper.exe`。

## AC / V / SD / 路由

AC01 最小全路径通过；AC02/04 因 R10-01 fail；AC03 有限数值、来源、真实 native 与准备资源控制通过；AC05 当前缓存/恢复正负控通过；AC06 因 R10-01 fail，完整新基线 V11 独立验收 pending；AC07 当前正式协议与兼容通过；AC08 有限语义/native/代际模型通过，前门规范增量收口未闭；AC09 fail；AC10 merge pending。

V01/02/04/05 常规信任/CFG/codec/控制流测试通过，前门例外=R10-01；V03 数值/trap/来源/native及准备控制通过；V06..08 当前缓存/身份/GC/lease控制通过；V09 fail；V10 执行预算/取消定向通过；V11 当前提交完整独立 30 样本正式门及六场景 release 门未跑：已发现合同内失败，不再消耗完整最终验收周期，也不复用 work 的“30样本PASS”授最终 pass；V12/13 定向通过。既有前端 180–211MiB 峰值/128MiB 目标 miss 的来源登记保留，本轮未修复或豁免。

SD01/02 有限实现、SD03/04 缓存/来源、SD06 正式协议、SD07 有限模型与准备监督的本轮受影响部分认可；SD05 的“最大构造路径且兄弟独立”仍因 R10-01 与实际行为不符。现行/新增 Spec 路径有效，`touched_goals=[]` 的 experimental 范围说明保留。没有发布 canonical Spec 或改 live ledger。

任务保持 T01/03/04/06 完成；T02/05/08 因 R10-01 待修，T07 的新基线完整验收 pending，T09/10 未完成，**executing 4/10**。原树 work 修正 sibling 完整路径的关联、补上述异路径 oracle 与多轮诊断，提交后再审。未修改实施代码、合入、归档或清理树。

pending_closeout_gates：最终 pass 后 merge 在冻结提交执行根裸 cargo t、编译器触面 cargo tv、独立 workspace 完整套件/真实 native 和实际 796/788/已合入 797 兼容；其余档按最终 diff，tf 只在主检出到期单实例。work-owned 完整新基线 V11 仍需在最终验收中通过，不能改称 merge-owned 以授当前 pass。Spec/ledger、归档和 guard 清理保持 pending。
