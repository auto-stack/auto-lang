# PLAN-820 R10 — needs_fix (only T-05 reopened)

2026-10-25；final / affected repair + outstanding acceptance/spec reconciliation。reviewer：本聊天Codex，独立于实施上下文，未启动代理。唯一实施树D:/autostack/.wt/lang-820/auto-lang，plan-820-dev，受审HEAD403eb57d0f1b20cb5600d620edc0e1ac465876cd，repair base8e0e0ef337991c1e1726847b8c6c70aaccf26a6c，原base37e51253689631ffa61a9af58041c5f6646a7569。入审/结束HEAD不变；最终实施树恢复clean。

入审主检出b3b0691c3286da1aac4dc6d92aa72f4af18ed014；Plan入审SHA256 CE6C1D33CF33A42D1C1DCE7EFE4DAB4397860108B62BEAA53D3939C3FDD115F4。Spec输入及验证配置/锁文件hash分别存于[spec-inputs](spec-inputs.json.txt)、[verification-inputs](verification-inputs.json.txt)。auto-down兄弟c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。D798实际交付7e192b264f4093855bbfe410a0c8147555c6275f、D828交付944403b50均通过受审HEAD祖先检查（exit0），不只依赖archived状态。

## R9独立CLOSED；本轮补完的数据域证据

- 根仓真实guest六整数矩阵（MIN/-1/0/1/42/MAX）定向1/1、0 ignored，1.52s，[结果](int-matrix-result.txt)；生产judge_tests3/3，[结果](judge-unit-result.txt)。另原样重放R9外部真实guest -1/0/1/42全部Pass，[源码](real-int-replay.rs)/[结果](real-int-result.txt)。零值回归已关闭，T-06完成标记保留。
- 原R8七形状组合按真实parent再判：core无值Fail/夹带typed Error，data bool0/1 Pass，bool2/00和匹配exit42的weird/bad均Error，[源码](judge-replay.rs)/[结果](result.txt)。旧finding不重开。
- V-12精确族cap两侧：模块types+consts+imports+funcs=65536可经MIR/ABC准入；65537两门均Resource(TableEntries)。每函数slots+blocks+instr=65536准入，65537两门均Resource(TableEntries)。源码空record保持source.unsupported拒绝。[可复现输入](v12-boundary-replay.rs)/[结果](v12-boundary-result.txt)。
- V-03扩展17组精确位型：MIN_I64、MAX_U64、bool0/1、f32/f64 ±0/±∞/正负NaN payload及冻结f32 7f80ad00，均经真实Const/Copy/Call/Return、store信封发布和restore_data后按位相等。[输入](expanded-bits-replay.rs)/[结果](expanded-bits-result.txt)。不是只做to_bits自身恒等断言。
- 布局大表资源控制：250×1024字段模块（估计16,402,973 B）MIR/ABC均准入；300×1024字段估计19,683,373 B在16MiB decoded门拒绝。[结果](layout-memory-result.txt)。最初审查探针错误地期望300表应准入而assert失败，原[输入](layout-memory-initial-replay.rs)/[输出](layout-memory-initial-result.txt)保留；修正后按真实冻结预算将它归为有效资源负控，**不是产品缺陷**。
- 对计划自原base以来28个Rust改动文件逐一rustfmt --edition2021 --check --config skip_children=true，全部exit0；完整diff-check通过。旧root warnings不伪称新增；未发现新增未处理编译告警或杂散调试输出。

## 本轮用户明确授权的性能例外 EX820-R10-PERF

本聊天询问收据所记旧标量周转P95=1064.5ms（门槛1000ms）“环境问题不重试”是暂不复测还是本轮豁免；**用户原话选择：已豁免本轮该项验收**。因此将Plan验收合同revision6→7，仅追加这一轮的具名例外，不重试该项、不把红数值改记测试PASS、不放宽canonical长期1000ms门，也不豁免正确性/资源/增量结构及merge-owned冻结门。此例外不覆盖下面编译与依赖闭包缺陷。

## F820-R10-01 P1 — 已交付817消费者编译被Value扩展破坏，锁文件亦未同步

**既有规则**：§5.1.6允许在公共定义扩展时做最小版本化消费者适配；§5.3、T-05、AC-06/V-09明确要求已合入817旧scalar路径重放，新data不能隐式扩大CTE支持面。817就在受审基面中（不是当前新出现的WIP或待合入后继）。

**真实定向命令**（只选择两例，不跑全CTE套件/性能战役）：

```text
cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml -p auto-cte-probe --test cte_snapshot_tests -- cte_snapshot_production_observation_incremental cte_snapshot_cold_first_call_bool_true_and_false
```

[完整构建结果](cte-consumer-build.log.txt)：auto-cte-probe编译失败，**两测试均未执行**。`experimental/ac-cte-probe/probe/src/av_driver.rs:231`（snapshot lane）与`:469`（旧closure执行路径）match只覆盖auto_av_core::Value::I32/Bool，820新增I64/U64/F32/F64/Record使两处均E0004 non-exhaustive patterns。该依赖路径和错误已确定，不是计时波动、机器负载或误构造输入。

另锁闭包：首次命令Cargo自动在CTE Cargo.lock里为auto-av-probe补一条 `windows-sys 0.59.0`依赖边（820此前新增的真实direct dependency）。[唯一自动diff](cte-generated-lock.diff.txt)。审查未提交/保留这项实现改动：初始clean为证据，已将这唯一自动生成的一行恢复HEAD，并复核tree clean；构建错误本身产生在解析该边后的图上，与这一行是否入库无关。

对原冻结lock独立执行 `cargo metadata --manifest-path experimental/ac-cte-probe/Cargo.toml --locked --format-version 1`，exit101，[结果](cte-locked-result.txt)。因此当前提交也不能在不隐式更新lock的冻结输入上复现消费者构建。

**最小关闭oracle**：原树对snapshot/closure两结果消费点补最小profile边界适配：原I32/Bool语义不变，意外data新值明确受控拒绝，不为CTE扩宽标量/record支持，不以panic/unreachable掩盖边界；同步提交CTE lock的实际依赖边。先证明metadata --locked通过，再在committed冻结lock上运行上述两定向正控及相关不支持profile/结果类型反控（机制需要的两个lane都覆盖），保留原817/CTE合同。仅改锁和边界消费不需重跑已闭的全部data矩阵/完整性能战役。

## AC / Spec对账与下一步

| AC | 本轮结果/证据边界 |
|---|---|
| 01 | 数据首例/native/typed父侧及整数边界PASS；复用R1/R6/R7已闭证据，R10补精确位型store/call |
| 02/03 | 数据语义、Copy/Place/求值序/类型等价PASS；复用已绑定semantics13与完整opcode回归，R10补V12两侧 |
| 04 | 数据准入/非法类型/profile/来源及资源分态PASS；V12与先前坏内层dispatch0证据保留 |
| 05 | 数据资源边界/错误事务/峰值PASS；复用已关闭parent采样/故障/取消与R3共享预算，R10补大表边界 |
| 06 | **FAIL**：817真实consumer编译与lock闭包；其余数据/父侧反控已闭；旧标量性能仅EX820-R10-PERF本轮豁免 |
| 07 | 未授最终pass，待本条关闭 |
| 08 | merge pending |
| 09 | 已核数据consumer增量/授权/旧snapshot与反控证据PASS，未将缺失的CTE编译当作incremental成功 |

Spec逐目标/当前hash已核：[审读候选](spec-delta-candidate.md)，尚非final-pass冻结delta。SD-01/02需明确i32/bool旧运算保留，只有新增宽标量仅数据面；SD-03/04/05/06/07追加新profile/current能力而不覆盖已落817/818事实；补SD-08 auto-mir/project导航/现状（现文件仍有历史“仅i32/bool/828未来”描述），metadata加入该modify目标。touched_goals空的有限实验说明保留。旧草案测试总数更新为稳定ID/审查证据引用，长期Spec不沉淀执行日记和本轮一次性豁免。canonical/ledger未写。

当前基面不含后续818实现；不声称在未来合并状态测试过818。merge在新集成基面上须按实际消费面重新验证，不能拿本HEAD的pass覆盖未来差异。该事项不替代本轮已经存在而实际坏掉的817。

回executing6/9，**仅T-05重开**；R9与所有旧findings独立关闭保持，T-06保持完成。下一轮只需关闭具名consumer/lock缺口、核受影响边界与剩余Spec收尾；不要再次全面重放已闭data/性能矩阵。未改实施源码/最终锁状态、未合入。merge-owned裸cargo t+tv+独立workspace完整有限合同档、将来集成基面新增消费者门仍pending。
