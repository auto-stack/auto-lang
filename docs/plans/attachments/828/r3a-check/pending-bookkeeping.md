# 待回写的r3a修复核查收据

用户此前要求主检出由其他会话合并时不干预；本轮继续将证据留在828工作树，不覆盖其旧Plan副本，不写主检出计划/KNOWN-DEBT或canonical/ledger。

2026-10-21 stage:review(r3a bounded repair/readiness check) | review_scope:final（因work承认必需OPEN，仅就绪及受影响核查，不授完整验收pass；R1/R2及r2a历史保留） | PLAN-828 r2 | outcome:needs_fix | reviewer:Codex /root，独立于实施 | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | commit:bb2380908ea7873be922f67288cf3aba39f82fc5 | base:db295803a1f45c4ec0b7ac528dc16167e0c39cce | evidence:永久13/13、原17/17通过，独立上轮15/15过；新增5相邻例5失败（快照体内映射/连续编辑/区间end、intrinsic引用身份交换/effect变更）；fmt过，diff-check日志EOF空行exit2，dead_code仍在 | findings:F04原a/b/c闭但d/e/f OPEN，F06旧intrinsic替换例闭但引用绑定身份和effect依赖仍OPEN，均原合同完整不变式未闭；F03/07/13与其他已验证原例保持，未完成任务不变 | report:docs/plans/attachments/828/r3a-check/report.md | next:原树work先给完整可组合source-map/实际引用依赖表示的诊断和消费矩阵，完成修复与全套定向回放再送审；继续executing，merge-owned冻结门/Spec沉淀/归档/guard仍pending | 未写主检出/修实施/整档/合入。
