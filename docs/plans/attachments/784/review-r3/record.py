from pathlib import Path
root=Path('D:/autostack/auto-lang')
p=root/'docs/plans/784-repo-extraction-legacy-preservation.md';s=p.read_text(encoding='utf-8')
assert 'status: execution_done' in s and 'current_step: 5' in s and '| [ ] | T-06 |' in s
s=s.replace('status: execution_done','status: reviewed',1).replace('current_step: 5','current_step: 6',1).replace('| [ ] | T-06 |','| [x] | T-06 |',1)
entry='''2026-10-15 stage=review | review_scope=final | reviewer_identity=当前Codex会话（规划/前两轮复审作者、非实施者；独立恢复与自选反控，无额外agent） | plan_id=PLAN-784 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-784/auto-lang | branch=plan-784-dev | reviewed_commit=d2f12ca2e37cab568041f2b548f6dad29e516dc4 | base_commit=7d51845ffcca4c886698caa947cbaca93bf7fdca | dependency_revisions=auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e,auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | spec_inputs/SD hash=[fingerprint](attachments/784/review-r3/fingerprint.json)、[冻结SD](attachments/784/review-r3/spec-delta-reviewed.md) | acceptance_results=AC01/02/03/04 pass,AC05 merge-owned pending | findings=F784-R1-01/02/03及F784-R2-01全部closed，无新增阻断；两连败诊断与refs/manifest/bundle覆盖调整已核对实际实现 | evidence=[R3报告](attachments/784/review-r3/REVIEW.md)、[19/19独立结果](attachments/784/review-r3/results.json)、[可复放控制](attachments/784/review-r3/run.py) | 验证=两真实新代快照独立恢复11/29refs与3/23tags、tree与保留文件hash一致；永久drill8/8；旧A→B/源变更删除6反控/refs新增移动删除及branch新建/有效旧bundle+同步hash仍被覆盖oracle拒绝全部通过；SD5链接、diff-check、三树guard通过；实现树和主检出使用交付baseline两strict均0违例2grandfathered | 保护=原仓/全部真实payload/实现tracked hash及HEAD前后不变，准备树仍隔离clean；主检出HEAD前进仅781簿记，不影响审面 | tasks=T-01..T-06完成，reviewed 6/7 | pending_closeout_gates=Category A禁止cargo整档，无Rust回归待补；T-07规范沉淀/ledger/归档/准备树明确交接/执行树guard后清理pending | next=merge核对冻结证据并完成T-07；本轮未merge/发布规范/改ledger/移树，review pass不等于交付完成。

'''
assert '## 10. 待澄清事项' in s
s=s.replace('## 10. 待澄清事项',entry+'## 10. 待澄清事项',1);p.write_text(s,encoding='utf-8')
p=root/'docs/plans/KNOWN-DEBT-AND-RISKS.md';s=p.read_text(encoding='utf-8')
entry='''### Plan 784（2026-10-15，final review R3 —— 全部闭合）

审面 `d2f12ca2e37cab568041f2b548f6dad29e516dc4`；**final pass / reviewed 6/7**。F784-R1-01/02/03和F784-R2-01全部closed，无新增阻断项，历史记录保留。
[R3报告与闭合表](attachments/784/review-r3/REVIEW.md)、[独立19/19结果](attachments/784/review-r3/results.json)、[冻结SD](attachments/784/review-r3/spec-delta-reviewed.md)。

两真实快照独立恢复11/29refs、3/23tags及tree/WIP等原字节；永久drill8/8、refs新增/移动/删除和branch新建、有效缺ref旧bundle+同步hash双反控、三类保留源变更/删除6反控均通过。两个strict均pass/0违例2grandfathered；三树guard、diff-check、规范5链接通过。原仓/实际payload/实现提交前后不变。

T07规范沉淀/ledger/归档、准备树明确交接与执行树guard后清理仍pending；review pass不等于merge或交付。Category A未跑也不要求补跑Rust整档。本轮未merge/发布规范/清理树；不得删除后续迁移用准备树及任一代恢复载荷。

'''
assert '### Plan 784（2026-10-15，final review R3' not in s
p.write_text(entry+s,encoding='utf-8')
print('Recorded final pass; all four findings closed; reviewed 6/7; T-07 merge-owned pending.')
