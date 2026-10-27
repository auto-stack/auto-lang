
### 2026-10-26 独立最终复审 R14

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施修复，从实际源码/合同/行为重建，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: fab5f492a74377d97c508e02dcea1916504bf8d2 | bookkeeping_commit: 8e5045941 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 5490d6678cadb6e71cc01a96474151b56f4754f4

F-822-R13-01 CLOSED：统一 build 错误出口显式 drain out/临时 children，List fault 在所有权转交前预留；本轮独立运行 partial_clone_rollback 两测绿，原深链反例返回受控 InstrSteps/live0，未再崩溃。预置 cancel 控制只证明入口拒绝，不冒称中途取消。既存 R14 原反例与 list-fault 日志保留，来源为入场已存在附件；本轮重新运行 cache 反例与正式 cache_compat 五控。

F-822-R14-01(P1，原恢复身份集成遗漏；AC-06/V-08，T-05/T-06)：restore_collections 以请求 types_fp 选 key，却仅 verify_abc_bytes 后直接 execute，没有比较载荷实际类型表指纹。真实 ManagedStore 中把合法 first-case 加一个 unused I64 类型（MIR/ABC均合法），以原 types_fp key+合法同owner/snapshot envelope存储；恢复实际返回 Success I32(0)/steps121。expected types-fp=46d81ee5fdbb84a2，payload=58dda3f6073c2aff，独立拒绝oracle失败(exit101)。违反§5.3类型/layout依赖与恢复不可信、V-08及SD-03。不是要求拒绝所有合法新类型，而是载荷与请求身份不匹配须在dispatch前拒绝。核查入口/owner/snapshot等相邻轴与载荷一致性，保合法恢复和真实依赖 clean miss；不改旧profile合同。

acceptance_results: AC-06 fail，AC-07 fail；AC-03/05 R13部分回滚缺陷关闭，其他条目保留既有证据边界，剩余未查项不冒领pass；AC-08 merge-owned pending。SD-03身份/恢复完整主张不批准，SD-01/02原R13阻塞已消除但本轮不授予全局pass；其余保持原边界。SD冻结hash=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739，Cargo.lock沿既有hash，未修改依赖/验证配置。证据见 attachments/822/review-r14/report.md、type-binding-replay.rs及日志。最终HEAD仍fab5f492a，tracked implementation clean；既存未跟踪review探针保留，探针unused import warning不归产品。无canonical/ledger发布、实现修复、整档或aggregate acceptance。

next: 同树work修F-822-R14-01，T-05/T-06重开，executing5/9；定向重放身份不匹配反例、合法恢复/clean miss及相邻身份正反控，提交后再审。最终pass后merge-owned冻结裸cargo t+tv+独立workspace有限合同档及实际触面门仍pending，tf仅主检出批量。未合入/归档/清树。
