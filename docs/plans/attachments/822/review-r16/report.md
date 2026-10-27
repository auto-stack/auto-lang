
### 2026-10-27 独立最终复审 R16

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立合同/源码/行为核验，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: f1d31632b1e625deb125142864e1ad032cb9870d | bookkeeping_commit: 9a2964804 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 14b6a9df2e52a46c72f49789e1dcd91818d11da5

F-822-R15-01 CLOSED：decoded entry name/source_snapshot均在execute前核对；独立运行正式cache_compat七控7/7绿，含两错配拒绝与诚实恢复。owner重盖章机制经源码核实，不要求给无自证owner的wire增字段。但提交的“profile/schema由通用ABC门保证”论证不完整：通用门按payload自己的profile选择版本，并非按collections请求强制profile。

F-822-R16-01(P1，恢复profile身份残余；AC-06/V-08/09，T-05/T-06，SD-03)：collections restore未绑定实际wire/profile到固定collections身份。独立反例先构造合法collections scalar module(main返回7、snapshot123、types=[I32,Fn])得到请求types_fp；将同一module改为合法core-i32 profile/owner schema，保相同types/entry/snapshot，编码为AVAB，然后以collections的A22 key、plan822/schema/semantics封套写真实ManagedStore。restore_collections返回Ok(Success I32(7),steps2)，拒绝oracle红(exit101)。同一程序保持collections AVAD v3的诚实store正控绿。两例types_fp/entry/snapshot均匹配，故不是重演已关闭的R14/R15例。

既定规则：§5.3 profile/wire/schema身份矩阵、V-08依赖变化不复用与V-09旧profile独立、SD-03世代隔离；不是要求通用verify_abc_bytes不再支持core-i32，也不是要求core旧消费者改变。建立原因：abc_verify::verify_abc按decoded.profile选合法tuple；mir_verify对collections/data/owned核schema，但core分支无反向expected schema约束，重盖章collections owner不能代替检查actual profile。restore仅比较types_fp/entry/snapshot后execute，因此合法core穿透。本轮仅声称已实测core→collections混淆，不冒称其他profile均可穿透。

重放：附件 replay.rs临时放既有树probe/tests/review_822_r16.rs；cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r16 -- --nocapture。原始日志profile.log.txt；1合法对照绿/1拒绝反例红，无产品构建warning。关闭oracle：恢复入口在dispatch前拒绝实际profile/wire不符，合法collections恢复及既有R14/R15拒绝保持，核通用门与请求身份的所有实际适用轴。不要改变通用旧profile合同。

acceptance_results: AC-06 fail、AC-07 fail；R15入口/快照子面已闭，AC-01..05/09保留原证据边界，未查项不冒领pass；AC-08 pending。SD-03恢复/隔离完整主张仍不批准，其他SD沿既有边界。冻结SD hash=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739；依赖/配置未变，hash及最终HEAD/cleanliness见input-hashes。无canonical/ledger发布、实现修复或整档运行。

next: 同树work修F-822-R16-01，T-05/T-06重开，executing5/9，其余保留；定向正反控提交后再审。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面档仍pending，tf仅主检出批量。本轮未合入/归档/清树。
