# r6a受影响修复核查待回写收据

2026-10-21 stage:review(r6a explicit affected checks) | review_scope:final（仅用户具名r5a三例及删除/插入边界/复合核查；其他必需work仍未完成，不授完整最终pass，历史review计数保持） | PLAN-828 r2 | outcome:needs_fix | reviewer:Codex /root，独立于实施 | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | commit:5bdc4d1f5fb71255b418291190f23a775c296a1f | base:0b936f900c223018c3c6edc48ddcdca51fb4d381 | evidence:永久24/24、原17/17绿；独立30例27过3失败。r5a原三失败例通过，但其已通过撤销正控回归(102..75应70..75)；新增部分删除插入文本(102..91应86..91)、扩长替换内部删除污染前置函数(25..14应25..30)失败；独立暂停旧snapshot恢复控制通过。fmt过，旧日志EOF diff-check2、warnings保持 | findings:F04-i/j/k原例闭，F04-l新修复回归、m/n完整不变式未闭；其他既有通过及明确OPEN保持 | report:docs/plans/attachments/828/r6a-check/report.md | next:原树work先明确mint/新增片段来源关系及整个编辑区间分解投影、插删替换和撤销规则，再实现/回放所有旧正反控；继续executing，完整AC/SD及merge-owned冻结门/沉淀/归档/guard pending。

仅新增原树核验材料；未写主检出计划/KNOWN-DEBT/canonical/ledger/合并，未修Rust实现、未跑整档或合入。后续按最新主计划增量回写，保留全部旧证据，勿覆盖主计划为旧工作树副本。
