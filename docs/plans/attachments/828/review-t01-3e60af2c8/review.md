# PLAN-828 r3 / T-01 受影响面复审：needs_fix

受审提交 `3e60af2c8aa2c287633f4700b377714e94c4ff4f`，基面 `47894c2e6747c8c06032be7a3d40bf00b90a204d`；同一工作树 `D:/autostack/.wt/lang-828/auto-lang`、分支 `plan-828-dev`，入口/收尾均检查 clean。

**结论：T-01 尚未完成验收，先回同树 work 修复，不直接进入 T-02。** 有三项 P2：未改 body 被深拷贝/重新生产、结构门不能识别真实旧管线工作、ac-core 新增 private_interfaces 警告。D-02 的无 merged 方向可保留，但当前只量化旧现状；D-02/D-03 的 T-01 最小可行性及候选环境决策仍需补齐。Plan 保持 r3、executing 0/11，F-01/F-02 OPEN，F-05b 已验证范围 CLOSED。

这是 final reviewer 角色对 T-01 的定向检查，不是整份最终验收或阶段落地批准。当前上下文没有实施此代码；此前已做静态合同就绪核对，本次在同一源码基线上补了独立运行控制。来源/配置/Spec/产物指纹、原始计划与工作决策均冻结，见 [baseline.json](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/baseline.json)。用户的测试摘要作为待核声明，本次运行结果单列如下。

## T01-C01 / P2：单 body 发射仍复制并重新生产未改 body

**既有规则**：AC-01/02，§5.2/5.6/5.8，C01-01/05；允许接口目录与必要绑定元数据，不允许全部 stub Program 的 body/HIR 重建。T-01/D-01 要为后续提供符合该边界的接缝。

**位置**：[resolve.rs:828](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:828)、[adapter.rs:1981](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:1981)、[adapter.rs:2266](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:2266)。

`stub_resolved` 先 clone 整个拥有式 ResolvedFn，随后清 body/局部变量；plan_module_scoped 对所有函数（含 changed）调用它。render 仍为全部函数写 body 文档，随后全部 bind 成 HIR。这与“字段级签名脚手架、没有未改 body 工作”的描述不符。

**可重放输入**：[frontend-controls.rs](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/frontend-controls.rs)，`t01_review_one_emission_must_not_regenerate_unchanged_body_documents`。真实三函数 helper/main/tail，helper 的 n+1 改 n+2；main 有局部 x，避免空表观察。实际输出：
- receipt=`real_body_plans:1, scaffold_fns:2`；
- `planned_fns=3`；
- 本次新生成的 Atom body 头为 helper/main/tail 三项；
- 新 bind 的 HIR bodies=3；单 body 生成断言实际 3、期望 1，FAIL。

该断言针对本次实际生成的 body 文档；不要求未来共享声明/上下文视图必须只有一个条目。深 clone 由源码建立，未声称已测量其峰值或耗时。合法 changed-body 冷编 dump/verify 对照、合法非首位递归函数均 PASS，故不是把所有新入口都判不可用。

**修复/关闭**：owner=T-01/D-01，T-02/03 消费。保留可用的 resolve_fn/check_body_in 规则抽取；从签名字段或受检共享声明上下文供 callee 查询，去掉未改 body 深 clone 及热路径全部 body 的计划/发射/绑定重建。新接口仍走适用 verifier 私有凭证，不另建 parser 或授 trusted。用真实 emission/body producer 数据与冷编/合法控制关闭；仅移除 clone、把空 body 改名 scaffold 或修改计数都不够。若现有 binder 接缝尚不支持，先完成有 owner 的具体方案/首例，不将它带入 T-02 后再掩盖。

## T01-C02 / P2：结构门会放过实际执行的旧 synth 管线

**既有规则**：C01-01/05、§5.8、T-01/D-01：真实旧管线工作必须令同一结构门红，缺观测另行 fail-closed。

**位置**：[adapter.rs:278](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:278)、[resolve.rs:735](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:735)、[contract test:99](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/probe/tests/function_incremental_828_contract.rs:99)。

receipt 的 real_body_plans 固定为1；env_walk_stats 只描述该 retained env 的显式调用，冷 N/N 还由消费者设置。原负控调用 gate(None,None)，只能证明缺收据时拒绝，不能证明真实全量工作会被观察到。

**可重放输入**：`t01_review_real_old_pipeline_must_invalidate_same_structural_gate`。
1. 实际跑冷前端，初始化 retained env 及合法 recheck/emit，取得真实 receipt 和 (3,3)；未伪造 receipt/counter。
2. 在同一工作观察窗真正执行 compile_program_parts，输入为 changed helper + main 空 body 的旧 synth，实际 planned_fns=2。
3. 再计算与提交测试相同的 gate 谓词，仍为 true，FAIL。

这是验收 gate 的漏检，不冒称已有正式生产报告在此窗口发布了错误 PASS。独立控制将“实际额外旧管线工作”与“缺收据”分开，原始五测试 5/5 不能覆盖此问题。

**修复/关闭**：owner=T-01/D-01/T-02 观测接缝。把观察挂到真实 resolve/typecheck/HIR producer/bind 与适用复制边界，按函数身份区分 body 工作和接口目录。合法热路径同门绿，真实强制旧管线保持观察器可用时同门因额外工作红，缺必需观察另拒绝。验证输入/产物与观察窗绑定；不能通过永远不给旧路径 receipt、手填数字或只修改 gate 的期望值收口。

## T01-C03 / P2：新增 CheckedEnv 字段触发编译警告

**既有规则**：仓库独立审核健康检查要求处理编译警告；T-01 新接口不能把本轮新增问题归到已有四个 probe warning。

**位置**：[resolve.rs:698](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:698)：`pub(crate) sigs: BTreeMap<String, FnSig>`，而 FnSig 只在模块私有。

本次 Cargo 定向重放实际报告 `private_interfaces`，指出字段可见性大于其类型。基面没有新增的 CheckedEnv/字段，属于本提交新增。

**修复/关闭**：owner=T-01。按真正消费需要收窄字段可见性或使内部类型具有一致可见性，不为消警告盲目扩大公共 API。重放受影响编译/定向检查确认此 warning 消失。原四个 probe warning 与 shim-metadata doc warning 仍按既有 F-14/历史归因处理；不是本轮新引入，不能一并算作本轮回归。

## D-02 / D-03 与验收边界

| 子项 | 本轮判断 | 下一证据 |
|---|---|---|
| D-01 resolve/typecheck 接缝 | 有可复用价值；有效 body、类型错恢复、签名漂移控制通过；整个 D-01 尚未满足 | 先闭 C01/C02；保留已通过控制 |
| D-02 方向/旧现状观察 | PARTIAL。当前三入口仍 dispatch merged；代码身份变化负控成立，但不是新单元派发正常绿/实际副本红的同门证明 | T-01 给出局部 type/const/slot/PC、self/import 调用与 snapshot/source-map 的实际消费接缝，补双函数可行性首例和绑定拒绝控制；完整 C02 矩阵仍由原 T-05/06/T-08 交付 |
| D-03 所有权 | 原 session/PreparedEdit/snapshot/PausedRun 的对应与回归保留；新增 CheckedEnv 尚未接 session | 明确成功 recheck 如何作为候选、prepare 不提前发布、失败/取消/迟到如何处置、commit 一次更新和 close 释放；禁止全 env clone 抵消 F-01。当前不存在的接线不登记为已复现污染 |
| F-05b | 原样九控制 PASS，无本轮新回归 | 新环境/dispatch 后按影响再验，不重开无关已闭部分 |
| §5.4 / C02-04 link 失败原子性 | 仍是原 T-06 必需验证；之前就绪报告已指出来源元数据在 link 前被修改的时序 | 后续须进入真实失败分支；本次未注入该分支，不添加未复现的运行结果 |

本次没有要求提前完整交付 T-05/06；D-02/D-03 仍需完成 T-01 原先登记的接口决策/有界首例，不以“后面落地”替代其前置证据。其余 F-09/10/11/12/14 及 A07/正式性能资源门保持原 owner 和 OPEN。

## 本轮实际验证

| 检查 | 结果 / raw |
|---|---|
| Cargo contract 目标，locked，5个原测试 | **5/5**，exit0；[输出](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/original-contract-output.txt) |
| 独立 frontend-controls | **2pass / 2fail**，exit101；两失败对应 C01/C02，两个合法控制通过；[输出](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/frontend-controls-output.txt) |
| r23a 原关闭输入，filter close_ | **9/9**，exit0，87 filtered不计通过；含最后 PausedRun 释放与实际取消；[输出](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/close-output.txt) |
| 六个修改 Rust 文件 rustfmt --check，skip_children | exit0；[输出](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/fmt-output.txt) |
| git diff --check base..HEAD | exit0；[输出](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/diff-check-output.txt) |

原命令：

    cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_contract -- --test-threads=1 --nocapture
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/run-controls.ps1
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/run-controls.ps1 -Source close-replay.rs -Filter close_

Cargo compiler-artifact/build-script-executed 元数据精确选择本轮 rlib/native 路径，见 [清单](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/cargo-artifacts.jsonl)、[库哈希](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/linked-libraries.json)。初次 helper 因两个 rlib 拒绝在编译前停止，已保留 [原因](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/runner-initial-output.txt)；没有按最新时间随意取库。一次 cargo fmt --all 选入无关生成源/旧示例而失败，归为检查范围问题，改用六文件定向检查，见 [记录](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/fmt-scope-note.txt)；两次都只检查，未改源码。

未重复 r1 86、主17、core22、旧62/798兼容全集，也不宣称复现这些摘要。当前所需的两个结构缺口已有独立失败，F-05b 已重放；完整 scalar/前端替换兼容、资源/正式门及后续矩阵保持 pending。这里的集合有重叠，不合计独立测试总数。未执行整档 t/tv/tt/tb/tu/taa/tf。

## Spec、状态与下一步

冻结 [SD-01..06](D:/autostack/auto-lang/docs/plans/attachments/828/review-t01-3e60af2c8/spec-delta-input.md)，SHA256 `371a9298c285eb8a0d7309b77298ce9bdadb932e626cc82b7df3af7be3759623`；目标路径和 GOAL-002 保留。SD-01/02/05 的新函数前端/结构证据未过，SD-04 的本轮 D-02 决策仍部分；原 F-05b 受测寿命范围通过，不给整份 delta 授 PASS。canonical、ledger、目标/profile/P95/预算和既有 AC/V/C/T/SD 语义均未修改，revision仍3。

下一轮同树 work：
1. 一轮处理三项 C01..03，保留两个合法控制及原合同五例，不只改测试期望。
2. 同时补齐本轮 D-02/D-03 前置决策/最小首例，更新 interface-decision 的超出证据描述及 frontend“5/5”数字冲突。
3. 按 [现有GLM任务单](D:/autostack/auto-lang/docs/plans/attachments/828/contract-r3/readiness-20261021/glm-next-round.md)实施；本报告的三项正式 findings 与精确重放是新增交接输入。提交完整 SHA、关闭表、实际选择数/断言/raw及残余边界后再做 T-01 受影响面检查。
4. T-01继续未勾选、executing0/11；T-02不开始全面换接，整份最终审核与 merge-owned 冻结门、Spec/归档/guard仍 pending。登记缺口不授权延期或降级；没有重复请求相同范围批准。

