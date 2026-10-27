# PLAN-817 R6 final review — needs_fix

2026-10-23；Plan r3；review_scope=final；当前独立于实施的Codex审查上下文。worktree D:/autostack/.wt/lang-817/auto-lang，branch plan-817-dev，冻结实现c7ec7c076875714d031ab6aae20b23b6752845b8，base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d，本轮修复基面eb8f9bc03。起止HEAD未变/clean。使用主检出Plan而非worktree旧Plan；根因和覆盖调整收据已核。

## R5三个P1闭环

- F-817-R5-01：本轮正常进程模式和约定反控范围内关闭。spawn/warm消费child自报峰值，0/缺失拒绝；managed判决入门；新增TreeSampler以parent+已登记children按同tick求和。真实420MiB child、双child聚合、managed缩容、omit-peak反控通过。TreeSampler不枚举任意后代，只统计本bench注册的worker；本计划无native的性能查询路径可对应该范围。另：warm在首轮response后登记PID，warm-up树采样未完整覆盖；此为采样边界，未来扩原生/后代路径不得沿用该证据。
- F-817-R5-02：关闭。Meta恢复115函数/≥256KiB基线，field_count→meta_fold真实32callee依赖闭包，独立闭式2595n+141296；提交30样本meta P95=609.2ms（报告冻结）。不再使用上一轮缩载结果。
- F-817-R5-03：关闭。warm/spawn共用admit_response，步数先判、每步来源/op/outcome/精确binding/type/value/HIR、edit恰一次、峰值缺失/managed核对；11种真实wire变异和合法控制、暖流代表性变异已执行通过，未再把旧浅oracle当闭合。

实际运行：cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_turnaround_tests -- --test-threads=1：**11 passed,0 failed，263.68s**，串行结束后才执行额外资源反例，避免性能互扰。测试含正常30样本、spike、malformed、degrade三态、真实wire/stream变异、child内存/树聚合/managed/采样缺失。

## 新发现（原合同资源项，不是新增需求）

### F-817-R6-01 P1：缺少10秒请求累计截止时间（AC-03/05，T-03/05/08）

原Plan§3、contract-decision§7明确单请求墙钟<=10s（含调度/等待），不是仅每次AV<=2s。driver.query_step_inner无请求deadline，循环中每次执行均重新使用ExecLimits::default；正式AutoDown适配器SESSION_WALL为60s，不能替代10s请求门。

生产case-serve反例：request-wall.atomdata（267676字节，32query，低于4MiB/128节点上限），115helper各含80条有意义算术，32个不同实参调用cte_sum（不触发single-flight复用），只bind main.ct=q0、native off。独立expected=42+115*4780+81*6555=1080697。

实际request-wall-result.json：outcome=ok，ct_execute=32，wall_ms=22198.3679（约22.2s），managed=true、峰值369430528<384MiB，值/HIR正确。没有用语言trap或大输入异常混淆限时路径。4调用同源/全新store控制：four-query-control.json，outcome=ok、ct_execute=4、wall_ms=1938.1529，值相同。每单次合法但累计超限仍成功，不能由单次fuel/wall证明请求总门。

输入SHA256 BDEABF38E338C2581B0406C3A1CF574AE4223DBA7EAA601AC5821664DCAA3E90。重放：主检出Get-Content docs/plans/attachments/817/review-r6/request-wall.json -Raw | & D:/autostack/.wt/lang-817/auto-lang/experimental/ac-cte-probe/target/release/cte_probe_worker.exe case-serve。必须改store_root为新目录，否则32项已缓存会hit（不再是该miss反例）。输入生成最后多余marker导致一条Shell未知命令错误，但文件此前已完整写入；实际重放独立完成且来源hash/value/执行数均相符，不以生成命令退出码作验收。

关闭oracle：从请求准备/调度开始的同一累计deadline覆盖cache、编译/执行、consumer与必要等待；超10s受控resource/cancel/infra，不能成功发布当前请求/迟到结果；相同合法规模miss组合、多query/DAG与cache hit正控通过。不以把单调用cap更小或提高总门限替代。保留反例并验证资源分类，重放无需改变语言支持面。

### F-817-R6-02 P1：编解码预算预留顺序仍违反T-05（AC-05，T-05/08）

这是原Plan§3、§5.4及contract-decision§7“分配前预留——先于clone/decode/编码”的未覆盖路径，不重开已关闭的采样/benchmark判决。

已确认代码顺序：
1. driver.publish_env先store::serialize_envelope(env)生成Vec，再budget.reserve(Staging,payload.len)。超预算控制虽能返回红，真实编码分配已发生；随后publish_frozen_payload又payload.to_vec，未额外预留重叠副本。
2. driver cache_lookup先store::lookup_frozen→get_artifact载入payload→serde_json::from_slice完整反序列化，再serialize_envelope仅为测量held大小，最后budget.reserve(Hot)。恢复/decode/临时重编码均发生在预留前，无预算token传入store恢复。
3. one-shot worker stdin.read_to_end/JSON解码、stream read_line、base64 decode均在输入上限和driver台账之前；大于4MiB的请求直到deck parse才拒绝，无法证明受控输入解码有界。没有本轮实测巨量输入/宣称OOM，仅依据完整分配调用序列确认预留时序错误。

现有resources_spike_control_is_really_red用缩容后最终Diag/managed=false验证结果，不能验证真实分配之前已获额度；不能用进程峰值低于384MiB替代32MiB受管台账。

关闭oracle：各clone/decode/编码路径在实际分配前获得覆盖最大活跃重叠空间的预算；有界wire读入和base64大小预检；恢复的payload/decode先界定再分配；失败保持旧条目/lease。编码/解码/深输入/恢复四路用超额度反控及分配/预留顺序观测证明拒绝先于受管大分配，不能仅让末端ledger红。不要通过降低支持输入规模来绕过原合同。

## 总体验收/证据

整体结论needs_fix，R5三项本轮关闭，新增R6-01/02均属原资源合同缺口。AC-06的本轮性能/语义oracle反控通过；AC-03/05因上述问题未过，AC-08/09与完整交付不能pass；AC-01/02/04/07/11其余未在此轮重跑的证据保持pending/历史有效范围，不推定整体通过。未跑整档cargo t/tv/tt/tb/tu/taa。发现required新反例后返回work，不用重复所有正向矩阵覆盖资源缺口。

Spec delta冻结spec-delta-frozen.md（原文仍含owner-only旧表述、touched_goals空影响理由需最终收口），canonical只读auto-av/resident-function-install、auto-semantics/semantic-contract、auto-ac/artifact-contract、auto-test-case/architecture；依赖取冻结树内783/796/798/828/lock。没有修改实现、canonical Spec/ledger，没有合入/归档。merge-owned裸cargo t+tv+probe消费面pending，须最终review通过后执行。

后续同树work：T-07本轮R5验收闭合，T-05/T-08因R6重开/保持开放，T-03累计deadline缺口重新打开，T-11完整专项收口继续pending；完成任务5/11。下一冻结提交必须提供累计deadline与真实分配前预算的根因/覆盖调整及反例，而不是更多相同正向测试总数。
补充定向验证：cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_resources_tests resources_spike_control_is_really_red -- --exact：1 passed（0.04s），确认现有末端超预算反控仍绿；该断言未观测编码前分配，不能关闭R6-02。终检HEAD c7ec7c076且实现树clean。
