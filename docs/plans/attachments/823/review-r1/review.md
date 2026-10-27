# PLAN-823 r8 独立复审 R1：needs_fix

- stage: review；review_scope: final（就绪核验发现合同内未完成项，停止完整最终验收，不发 pass）。
- reviewer_identity: 本聊天 Codex；本上下文未实施823，从主检出合同与指定树代码重建判断；未创建/联系代理，未声称其他模型身份。
- worktree: D:/autostack/.wt/lang-823/auto-lang；branch: plan-823-dev。
- reviewed_commit: ed69973940839c0c52042dad5fb4e40971215a6d。
- base_commit: c6c5ddb256089e6b628d7651cb9759664d15f22a；main观察点8a2347384e78b55f9533e44a4cd15cf34a5d1538。
- 入审/出审实施树 status --porcelain 均空；HEAD未变。D821/D822/D828/D852的已归档记录存在（822 r5、828 r3、852 r2）；完整依赖/窗口消费者并集认证尚pending，不能用归档状态替代。
- Plan原始输入及当前Spec SHA256见 input-hashes.json；规范增量冻结于 spec-delta-frozen.md。未修改canonical Spec或ledger。

## 实测与健康

在指定实施树执行（不运行整档）：

`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test av_closures_823_recipe --test av_closures_823_incremental --test av_closures_823_resources`

结果：3+5+3=11通过，0失败/ignore。

`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test av_closures_823_first_case --test av_closures_823_semantics --test av_closures_823_cache_compat`

结果：4+10+4=18通过，0失败/ignore。合计29，不冒计parallel壳或其他历史族。上述编译输出无警告。

`cargo fmt --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core -p auto-av-probe -- --check`

结果exit=1：abc.rs:2853、av.rs:3835及多处r8 env:None缩进，mir_verify.rs:884换行、closures_session/新增tests等格式差异。此前 --all 尝试被工作树缺少兄弟auto-ui的产品依赖阻断，不能当格式结果；限定两个实验包的检查真实运行并失败。

反例重放（cwd=实施树）：

`cargo run --quiet --manifest-path D:/autostack/auto-lang/docs/plans/attachments/823/review-r1/probe/Cargo.toml --target-dir experimental/ac-av-probe/target`

输入为本目录probe/src/main.rs，输出probe-output.txt；依赖指向冻结的真实core/probe，无实现修改。该程序打印反例实际值，不以自身exit=0表示产品符合合同。

## Findings（合同内阻塞，不是获准延期债务）

### F-823-R1-01 / P1：父env字段没有真正转移

影响AC-02/03/04、T-03/04、V-03/V-12；违反§5.1.4、§5.5的“嵌套env字段经专用受检capture转移、活字段位图及invoke读集参与验证/清理，已转出字段不可读”。

最小输入由first_case_module构造：body的ENV_GET field1后，CREATE_CLOSURE child以Move捕获该List；Drop child；再次ENV_GET父field1并按原程序push/read。期望该普通别名转移路径在适用门拒绝，或真实父字段责任转移后再次读被拒；实际verify=true、ABC准入成功、Success(29)、live=0。Clone同型正控也Success(29)。live=0不能证明没有两个owner。

已确认原因：av.rs:2139取env字段调用Value::clone；ownership.rs:1569仅define_slot(dst)，CreateClosure只杀局部src槽；父env无字段转出状态/读取约束。子env拿到的是共享句柄；销毁它不影响父字段，故重新读成功。原nested_and_frame_reuse仅Clone父字段，不能覆盖move责任转移。

关闭oracle：永久保留本Move反例与Clone正控；覆盖真实父字段转给子env、父退出后子仍正确、父读取/重复转出拒绝及每字段单次清理。定向重跑语义/受影响ownership门；不附整档。

### F-823-R1-02 / P1：正式recipe、父侧裁判和真正新worker未交付

影响AC-01/04/06/11、T-02/05/06/11、V-08/10/15；§5.2/5.3明确首例正式recipe和父侧expected为work，不允许execution pending。

全树实际owner为crates/auto-lang-core/src/test_cases；搜索该目录无823/closures注册，未有本recipe corpus。recipe测试中的judge_first_case是同一libtest内私有函数，调用run_closures_fixture也在同进程；文件首注与脚本autodown分支明确写formal parent-side corpus/registration留merge。restore_closures只是本进程ManagedStore::open+verify+execute，无新worker进程，cache测试注释将fresh handle冒称new-worker。

期望正式av-data-823经probe/run/contained worker，父侧独立expected、缺工具/错工具身份/漏case/错误输出确实使正式入口非绿，并保存/终止生产worker后由新PID恢复重验。实际仅函数级正控与judge Err断言通过。关闭时补生产薄接线、独立corpus、真实worker生命周期与父侧反控；不得搬到merge。脚本声称0 selected不能PASS，但目前未检查selected计数，也应随正式门补真实零选择负控。

### F-823-R1-03 / P1：资源与兼容性能必需oracle被后置

影响AC-05/06、T-04/05/06、V-07/09。§5.2要求父侧采样新worker private-bytes相对空载增量≤128MiB（含暂存/allocator），§5.3要求D798冻结scalar负载端到端P95<1000ms，明确不是merge延后项。

recipe_managed_peaks_recorded注释却把process-level private-bytes列为merge-stage；实际只有wire+heap_estimate算式及exec peak，不是进程采样，也未验hot/staging各自预算。当前attachments仅模块合同、binding和截取测试摘要，无本计划新worker峰值/旧scalar采样原始记录；compatibility脚本只选820/821/822及profile隔离，没有817/818实际consumer或core-i32原recipe重放。

关闭oracle：真实父侧瞬态采样、既定合计/分区cap内与超界/故障正反控，以及冻结旧scalar和实际817/818入口按原输入/口径重放。新2函数typed churn的P95不能代替旧源负载；不要求修改预算或跑全局档。

### F-823-R1-04 / P1：关键状态反控的覆盖声明不成立

影响AC-02/03/04/05、T-03/04、V-04/11/12。

sem_cancel_after_commit_releases_env在执行前设置CancelFlag(true)。独立重放实际只有3个alloc（4、4、0字节），没有捕获clone的第4个alloc，更未有第5个body push；取消从EnvPlan prepare委托的clone visitor返回。正常正控5 alloc、Success(29)。因此它证明提交前clone取消清理，不能证明提交后env发布/调用中取消。

此外call_typing_negatives只测arity/结果类型/非closure；未有承诺的同宽错签名、错generation、跨store及旧frame token反例；CallClosure运行分支仅contains_key(env)，未核cref.gen/sig或store域。当前静态封闭入口能否使伪值不可达需工作侧给明确路径/受检前门与有效正反控，不能据元数据字段存在断言完成。strip测试仅无spans，不是同一输入strip前后比较。

关闭oracle：先证明执行到已发布env/目标callee的状态，再触发取消并验清理；按合同实际入口固定generation/store/frame/模式及strip子例、危险目标计数0和合法对照。未先建立可达性的内部伪句柄不作为新增产品缺陷。

### F-823-R1-05 / P1：V-13子case1被未经授权复用上游结项

影响AC-09、T-09及依赖的T-11、V-13。§5专项明确本consumer重放273575B/115函数/至少30不同source修订，并保留全链路生产/IO与P95双门，四子case无skip。canonical cache-and-conformance亦要求后继重放。

增量测试文件首注及T-09行以“828自己交付、822已重放、823消费同机制”替代本计划重放；仅main/body两个手造MIR模块。512 churn在7个常量值循环，没有source前端或273575B基线；binding又写30修订全谱缺口仍在。未提供代码/依赖/验证配置/输入均未变的可复用证据包；本计划已改公共MIR/wire/verifier/link路径。

关闭oracle：在当前冻结代码重放源基线，或提供可审计同输入/相关代码依赖配置不变的证据；不能用typed子case2或上游历史绿替代。只跑对应source增量case，不跑整档。

### F-823-R1-06 / P2：格式健康门实际失败

影响T-06/T-07健康检查；见上方命令。修复本diff引入的格式差异，限定core/probe fmt check退出0。不要在review修代码，也不因格式修复触发全档。

## 验收证据映射与边界

| AC | 当前结果 | 证据 / 缺口 |
|---|---|---|
| AC-01 | partial | 首例29及手算judge通过；F02正式入口缺失 |
| AC-02/03 | fail | F01父env转移反例；F04必需状态oracle不足；已有borrow/clone正例通过 |
| AC-04 | partial | arity/type/准入反例通过；F01/F02/F04未闭合 |
| AC-05 | partial | 故障/fuel/wall定向3绿；F03真实峰值、F04取消状态不足 |
| AC-06 | partial | store handle重开/identity 4绿；F02真worker/F03旧consumer与性能未闭合 |
| AC-07 | fail | 本轮needs_fix；健康F06失败 |
| AC-08 | pending merge | 本轮不执行 |
| AC-09 | partial | typed身份/漂移/退化/512churn共5绿；F05源基线缺失 |
| AC-10 | partial | 同源结构可见；未重新认证所有V14壳/窗口并集，保留历史模块子集进度，不发最终pass |
| AC-11 | partial | typed K行有定向结果；正式runner/新worker/必需状态受F01..05影响，不能以binding翻ok完成 |

V01/02已有定向支持；V03/04/06/11/12受F01/F04影响；V05模块保源证据保留但全路径闭合待核；V07/09受F03；V08/10/15受F02；V13受F05；V14同源模块证据不扩大为profile交付。

规范增量SD01..07的路径/意图可作修复输入；当前只是摘要，未有与r8及实际实现一致、含目标Spec版本和完整规则的定稿（例如父字段转移、新worker与父侧recipe不能按已交付写入）。本轮冻结其原文供后续比对；metadata路径保留、touched_goals=[]理由有效，SD最终验收pending，不发布。

## 路由

outcome: needs_fix；Plan保持executing。重开T02/T03/T04/T05/T06/T09；T11原未勾选且受依赖影响，继续未完成；保留T01/T10=2/11。历史完成描述保留作已完成子集证据，以本轮关闭表为当前状态。T07/T08均pending。不是取消现有实现，而是撤回整任务完成认定。

next: /auto-plan:work在同一树修复上述合同内问题和永久oracle，提交后按受影响范围重审。review不修实现、不merge。merge-owned裸cargo t+tv+新workspace完整合同档仍pending；本轮没有运行任何整档。不存在获准延期、预算放宽或范围减少。
