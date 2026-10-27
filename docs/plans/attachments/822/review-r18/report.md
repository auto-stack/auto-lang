
### 2026-10-27 独立最终复审 R18

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立核合同/实际路径/反例，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 7de289d47e92c4866857c4ae38579dab5cc4e16b | bookkeeping_commit: cf987d4f7 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: a26b8e71c3760e0538cb654d7c39e5a548c3f104

F-822-R17-01 PARTIAL：内部List<i32>→List<Str>布局更新与List参数接口错配拒绝两控独立2/2绿；真实session、生产计数、未改单元Arc、旧snapshot/失败后可用均有断言。新增host_grant_lifecycle独立1/1绿，建立required capabilities与host grants分离的输入，但仅测试重新提交受限beta，未覆盖同一候选程序的carried beta；完整授权不变量未闭。

F-822-R18-01(P1，R17修复接线遗漏；AC-09/V-13子case3，T-09/T-06，SD-06)：CollectionsUpdateSession::apply_revision只在Some(submitted module)分支检查host_grants；None分支直接复用已有FunctionCode，之后link/execute/commit完全不核其required capabilities。独立真实入口：先grant TRACE_CAPABILITY，安装main=alpha(+10)、beta=beta_unit_trace(1)成功11；撤销为[]后，①只提交main=alpha(+20)，旧受限beta被复用，新revision2成功21、rebuilt1/gate.pass；②空提交，新revision2成功11、rebuilt0/gate.pass。两次均执行并安装了当前grant不覆盖的候选程序。保持grant的同一carried形态正控成功21，复用beta Arc且live0。独立拒绝oracle红(exit101；一个测试内两形态均记录)，不是只凭测试覆盖计数推断。

既定规则：§5.4输入required capabilities与当前host grants分开，优化不绕过授权；§5全链增量及V-13授权事实变化须精确失效/拒绝；R17关闭oracle亦明确要求撤销/不足拒绝、失败不安装。此处要求的是新的apply_revision候选，不撤销已经绑定旧授权事实的历史snapshot，不把旧snapshot可用控制判为错。原因已由分支及真实行为确立，未声称本例执行了实际外部trace副作用（beta仅声明required capability）；缺授权候选仍准入/执行/安装本身已违反门禁。

复现：附件replay.rs临时放既有树probe/tests/review_822_r18.rs，cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r18 review_r18 -- --nocapture；1合法正控绿/1拒绝oracle红含两个失败形态，日志carried-grants.log.txt。关闭范围：候选完整受检闭包（submitted+carried）的required capabilities对当前grants核验，撤销/不足时不得执行安装；重新grant、无关grant变化/无需cap程序及原layout/body/身份控制保持。检查多submitted顺序与拒绝前生产副作用的边界，避免局部循环检查冒充“整次revision在任何生产前拒绝”；不要求全局回归或无关性能重跑。

acceptance_results: AC-09 partial/fail（layout子面已补，授权carried不变量失败）；AC-07 fail；AC-01..06保留原证据边界，R14..16恢复修复不因本diff失效；AC-08 merge-owned pending。SD-06完整依赖/授权主张仍待关闭；其余SD沿既有边界，touched_goals空理由保留。冻结SD/hash见input-hashes。无canonical/ledger写入、实现修复、整档/aggregate acceptance。

next: 同树work修F-822-R18-01并完成R17完整关闭，T-09/T-06重开，executing5/9，其他任务保留；定向提交后再审。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门仍pending，tf仅主检出批量。本轮未合入/归档/清树。
