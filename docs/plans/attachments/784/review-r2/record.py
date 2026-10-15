from pathlib import Path
root=Path('D:/autostack/auto-lang')
p=root/'docs/plans/784-repo-extraction-legacy-preservation.md'
s=p.read_text(encoding='utf-8')
assert 'status: execution_done' in s and 'current_step: 5' in s
s=s.replace('status: execution_done','status: executing',1).replace('current_step: 5','current_step: 2',1)
for task in ['T-02','T-03','T-05']:
 token=f'| [x] | {task} |';assert token in s;s=s.replace(token,f'| [ ] | {task} |',1)
entry='''2026-10-15 stage=review | review_scope=final | reviewer_identity=当前Codex会话（规划/R1作者、非实施者；独立恢复及自选反例，无额外agent） | plan_id=PLAN-784 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-784/auto-lang | branch=plan-784-dev | reviewed_commit=1994280805a524659ca3119ab9845e453a2cbc4a | base_commit=7d51845ffcca4c886698caa947cbaca93bf7fdca | dependency_revisions=auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e,auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | spec_inputs/SD hash=[fingerprint](attachments/784/review-r2/fingerprint.json)、[冻结SD](attachments/784/review-r2/spec-delta-reviewed.md) | acceptance_results=AC01/02 partial,AC03 pass（相关输入hash与R1相同复用证据）,AC04 fail,AC05 pending | findings=F784-R1-01/02/03均closed；新增F784-R2-01(P2/blocking:快照身份缺refs，HEAD不变而新增标签后仍reuse旧bundle、独立恢复缺标签) | evidence=[R2报告](attachments/784/review-r2/REVIEW.md)、[原始结果](attachments/784/review-r2/results.json)、[可复放控制](attachments/784/review-r2/run.py) | 正控=真实两仓快照独立恢复11/29refs与3/23tags、tree/WIP/保留文件hash相等；永久drill7/7、独立A→B保留与同态reuse、三类源文件变更/删除6反控、SD全5链接、diff-check/三树guard通过 | 保护=两原仓/全部真实载荷/实现tracked hash与HEAD前后未变；未修实现/发布规范/修改ledger/merge/清理 | tasks=保留T-01/T-04，重开T-02/T-03/T-05，2/7 | pending_closeout_gates=Category A无Rust整档；T-07沉淀/归档/准备树交接/执行树guard清理pending | next=work同树修复R2-01、补refs状态控制并提交；两次final失败，下次readiness先提供根因及快照身份/manifest/bundle一致性的覆盖调整，再review。

'''
assert '## 10. 待澄清事项' in s
s=s.replace('## 10. 待澄清事项',entry+'## 10. 待澄清事项',1);p.write_text(s,encoding='utf-8')
p=root/'docs/plans/KNOWN-DEBT-AND-RISKS.md';s=p.read_text(encoding='utf-8')
entry='''### Plan 784（2026-10-15，final review R2）

审面 `1994280805a524659ca3119ab9845e453a2cbc4a`；**needs_fix / executing 2/7**。R1三项均closed，新增一项blocking；历史R1记录保留。
[R2完整报告](attachments/784/review-r2/REVIEW.md)、[原始结果](attachments/784/review-r2/results.json)、[冻结SD](attachments/784/review-r2/spec-delta-reviewed.md)。

| id | 级别 | 缺口 |
|---|---|---|
| F784-R2-01 | P2 / blocking | 新快照身份只含HEAD/文件状态，忽略refs；同HEAD新增指向独立提交的标签后preserve仍exit0/reuse旧bundle，真实恢复缺该标签。须将完整refs→SHA纳入身份或明确拒绝变化，补新增/移动/删除控制并断言实际bundle可恢复性。 |

两真实快照独立恢复11/29refs、3/23tags与tree一致；A→B旧快照保留、三类源文件变更/删除6反控、drill7/7及规范5链接通过。原仓/实际payload/实现树未变，无真实数据损失证据。
保留T-01/T-04，重开T-02/T-03/T-05；同树修复提交再审。两次final失败，下次readiness须说明根因与状态身份/manifest/bundle覆盖调整。Category A未跑cargo整档；未merge/发布规范/清理。T07交接、沉淀、归档与guard清理pending；债务登记不批准延后本合同缺口。

'''
assert '### Plan 784（2026-10-15，final review R2）' not in s
p.write_text(entry+s,encoding='utf-8')
print('Recorded R2 needs_fix; R1 findings closed; F784-R2-01 open; 2/7.')
