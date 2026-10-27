# PLAN-823 r8 / R2 修复复审：needs_fix

2026-10-27；stage=review；review_scope=final（受影响面复审；已有未闭合项，不执行全计划最终回归）。reviewer_identity=本聊天Codex，继续R1独立审查上下文，未实施修复、未创建代理，不冒称其他模型或会话。

## 版本与证据

- 主检出入审HEAD：95de4f14279ceb5d5ab8b7ac6875fac49851ede2。
- 唯一实施树：D:/autostack/.wt/lang-823/auto-lang，plan-823-dev。
- reviewed_commit：0d5d58f6b93bbf3f7e687b103f4b19b7ae8fe6ef。
- repair_base：ed69973940839c0c52042dad5fb4e40971215a6d；Plan diff base仍c6c5ddb256089e6b628d7651cb9759664d15f22a。
- 入审/结束实施树git status --porcelain均空，HEAD未漂移；未修实现。
- 合同以主检出r8为准；95de4f142包含R1簿记，但未包含六项修复关闭表/新证据链接，current_step仍2；此次依据实际代码/反例重新认定，不把提交说明视为证明。
- Spec输入及Plan原文hash：input-hashes.json；SD冻结：spec-delta-frozen.md。依赖仍沿R1的已合入D821/822/828/852；本次修复未变相关依赖来源，完整共享窗口并集认证仍不发pass。未写canonical Spec/ledger。

## 检查范围与结果

在实施树执行：

1. `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test av_closures_823_semantics`：19通过，0失败/ignore；但产生unused_mut警告（semantics.rs:1207）。范围=F01环境转移与F04取消/签名/strip，以及同机制borrow/clone正控。
2. `cargo fmt --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core -p auto-av-probe -- --check`：exit0。F06格式原问题关闭。
3. `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test av_closures_823_incremental <精确滤串> -- --nocapture`，串行选三个新入口：recipe_new_process_worker_restore、worker_memory_hard_gate_parent_sampled、incremental_baseline_campaign_replay：各1通过，0失败/ignore。源码campaign耗时12.83s，当前构建的真实worker运行273575B/115函数/30修订，all_correct/structural_ok/gate_pass均过。新进程restore确有子进程且Success I32(29)。内存测试只证明所写绝对peak断言，不扩大为原完整资源合同。
4. 同一test target精确选择incremental_body_edit_identity、incremental_env_contract_drift_refused、incremental_forced_degradation_gate_nonzero：各1通过。连同当前source campaign，AC09/T09的四子case可关闭；不重新跑512churn或无关性能矩阵。
5. 总计25个现有定向测试通过，不含自建反例程序。不跑整档t/tv/tt/tb/tu/taa、全workspace或acceptance聚合。
6. 自建输入仅在主检出attachments下：probe/src/main.rs、worker-replay.ps1；真实core/probe来自上述冻结树。`cargo run --quiet --manifest-path D:/autostack/auto-lang/docs/plans/attachments/823/review-r2/probe/Cargo.toml --target-dir experimental/ac-av-probe/target`，输出probe-output.txt。程序以catch_unwind记录panic，不以exit0表示产品通过。

## R1关闭表

| Finding | 旧例 / 修复进展 | 不变量与当前结论 | 最小关闭范围 |
|---|---|---|---|
| F-823-R1-01 P1 | 直接ENV_GET→Move capture已拒绝；新增MoveFromEnv正例通过 | OPEN：加一次普通Move即可绕过别名规则，父env仍可读 | 见下面反例；别名传播/转移失败出口及合法Clone对照 |
| F-823-R1-02 P1 | 新进程restore单测通过；正式recipe常量/dispatch/worker分支已添加 | OPEN：worker白名单仍拒绝closures；身份映射和父侧judge调用均漏接；独立823正式corpus仍未加入 | 真实formal入口正控+错误结果/漏例/缺工具/错身份等反控，父裁判真正调用；新进程恢复证据保留 |
| F-823-R1-03 P1 | 新增父侧ChildPeakSampler测试通过，绝对peak≤128MiB是有用的更严格该样本上界 | PARTIAL/OPEN：idle进程结果被丢弃，未记录样本峰值；hot/staging仍无预算断言，超界/故障灵敏度未补；旧core/817/818消费者重放及D798冻结兼容负载仍无相应新入口/证据。F05的大source增量campaign不等于所有V09兼容族 | 按原V07/09补缺的定向oracle，不要求放宽预算/全局重跑；不能仅因绝对界不同就否定已通过的样本上界 |
| F-823-R1-04 P1 | 新同宽错签名测试通过；增加成功输入strip比较 | OPEN：新“提交后”测试实际仍停在捕获clone之前，状态证明匹配到了普通4字节源分配；frame/store/generation子例仍以i32→Closure类型错误替代，strip仅正常成功未覆盖合同trap/cancel清理 | 先证明真实env发布/进入目标callee后取消；将实际可达路径/封闭入口论证与状态反控对应；保留当前已通过子项 |
| F-823-R1-05 P1 | 本代码真实重放source campaign通过，typed identity/drift/degradation三个控制亦通过 | CLOSED：此前未重放问题关闭；AC09/T09可记完成 | 复用当前证据，后续仅在输入/相关代码/配置漂移时重验 |
| F-823-R1-06 P2 | 限定两个实验包fmt check exit0 | CLOSED：原格式问题关闭；新编译警告另立R2-02 | 不重复格式历史失败 |

### F-823-R1-01 补充反例：通过局部Move洗掉env别名

规则仍为§5.1.4/§5.5；影响AC02/03/04、T03/04、V03/V12。输入由原first_case_module派生，增加同型child和两个局部槽：

`EnvGet list(field1) → Move(alias2,list) → CreateClosure child(capture Move alias2) → Drop child → EnvGet list(field1) → 原push/read`

预期：该普通别名路径不得作为父字段ownership转移，适用verify门拒绝。实际verify=Ok，ABC准入成功，Success I32(29)/steps214/live0。直接Move控制已Err(mir.capture.env-alias)，Clone控制Success29；因此是原例关闭、共享不变量未关闭。

已确认原因：mir_verify.rs:1407的alias_slots仅标记ENV_GET原dst，没有沿普通Move或其他值传递保留来源；临时槽alias2因此未标记。live0不能证明没有共享owner。不得只禁这一个临时槽形状；按支持面检查来源传播、可重定义/分支等代表兄弟路径，保留Clone正常语义。此次未引入新规模要求。

### F-823-R1-02 补充重放：正式worker入口仍不可达

输入/脚本worker-replay.ps1，直接运行当前编译worker的case-serve（协议层目标可达、无假worker）。collections同型已交付正控：exit0、success、av_core_version=av.collections/1。closures：exit2，`unknown recipe ac-av/closures-run-v1`。见worker-result.txt（输出摘录）。

已确认三处漏接：

- av_probe_worker.rs:584白名单条件没有包含closures_recipe，新分支749因此不可达。
- worker core_version_for:509未映射closures，若只修白名单会回落av.core-i32/1；父侧ac_av.rs:99却将closures身份设为av.collections/1。须同真实av.closures/1合同对齐后完成正控。
- ac_av.rs:1365的判决分派在collections后直接回退judge_expectation，未调1675的judge_closures_fixture；后者只有单测调用。现有单测不能证明正式错误输出必红。crates/auto-lang/test/cases/av-data/823不存在，脚本autodown仍仅调用probe端私有judge测试。

关闭时沿真实dispatch→case-serve→身份校验→父侧judge→总体非绿串通，不把helper有定义或mock通过冒充生产调用链。

### F-823-R1-04 补充重放：30 steps仍是提交前

按新增测试相同split_units→FunctionCode→ProgramSnapshot路径park30。实际allocs只有id0:4、id1:4、id2:0，捕获clone id3尚不存在。raise cancel并resume后Cancelled，allocs仍只有3个、live0；与R1仍是同一未达状态。

semantics.rs:1214仅断言`any(bytes==4)`，它匹配源Str而非捕获clone；step预算含deep-size记账，不能将30当30条guest指令。原新增测试19绿不证明已进入body。关闭需使用区分性状态证据（env发布/目标callee进入及本次clone事件身份），然后触发取消，且旧提交前取消仍通过。

## 新修复回归

### F-823-R2-01 / P1：未信任EnvGet的dst越界导致verifier panic

影响AC04/05、T03/04；原规则=未信任typed/MIR/ABC输入由门受控拒绝，不能panic。

输入：first_case_module.funcs[1].blocks[0].instrs[0]=EnvGet{dst:u32::MAX,field:0}。同一输入的dst=1是合法正控。实际catch_unwind=Err，panic于mir_verify.rs:1413:64，`len is 13 but index is 4294967295`。为修复新增的check_env_capture_discipline在slot范围typing之前直接索引alias_slots[dst]，坏输入因此逃出诊断门。因果由实际栈与代码调用顺序确认；非工具/环境错误。

关闭oracle：上述typed入口受控Err、0 dispatch、合法dst正控仍过；检查该预扫描的其他slot/env索引及ABC反序列化后同门路径。仅重放坏输入/相关typing与环境语义控制，不附全档。

### F-823-R2-02 / P2：新增unused_mut警告未处理

semantics.rs:1207 `Paused(mut paused)`产生unused_mut；本次真实cargo test输出已确认。违反T06/T07零未处理编译警告健康门；去除无用mut后定向编译无此警告即可，F06格式已关闭不重开。

## 验收、规范与下一步

| AC | 当前判断 |
|---|---|
| AC01/06 | partial：真进程恢复进展保留；正式recipe/旧consumer/资源证据未闭合 |
| AC02/03 | fail：env别名转移不变量仍失败；取消与必要状态oracle不足 |
| AC04/05 | fail/partial：新未信任输入panic，原资源门仍缺适用项 |
| AC07 | fail：needs_fix，另有新warning |
| AC08 | pending merge |
| AC09 | pass：source campaign与typed三控制当前提交通过 |
| AC10 | partial：保留模块同源历史证据，完整profile/共享窗口消费者并集不发pass |
| AC11 | partial：原typed K行保留；formal pipeline与状态反控受未闭合项影响 |

SD01..07沿r8原文冻结，目标metadata路径保持，touched_goals=[]及理由保留；尚无含当前MoveFromEnv/真实身份/调用状态与目标Spec基面的最终规范草案，不将未交付规则发布为现状。R1未完成的Spec最终审查与完整依赖并集继续pending。

outcome=needs_fix，Plan executing，当前完成T01/T10/T09=3/11；其他重开work任务不因修复提交标题而回勾。merge-owned裸cargo t+tv+新workspace完整合同档仍pending，未运行/未合入。

这是第二次未通过的final复审。下次送审前请在work提供“为何原检查漏检→不变量/实际路径→设计或oracle调整→永久反例/合法控制→当前剩余边界”的关闭表，尤其处理alias传播、pre-scan未信任索引、recipe全调用链与取消状态观察，而不是仅新增测试总数。本轮保留已关闭F05/F06和真进程恢复证据；下一轮只检查受影响内容与明确尚pending项。
