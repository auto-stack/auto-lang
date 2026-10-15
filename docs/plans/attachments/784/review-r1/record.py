from pathlib import Path
root=Path('D:/autostack/auto-lang')
p=root/'docs/plans/784-repo-extraction-legacy-preservation.md'
s=p.read_text(encoding='utf-8')
assert 'status: execution_done' in s and 'current_step: 5' in s
s=s.replace('status: execution_done','status: executing',1).replace('current_step: 5','current_step: 2',1)
for task in ('T-02','T-03','T-05'):
    token=f'| [x] | {task} |'
    assert token in s
    s=s.replace(token,f'| [ ] | {task} |',1)
record='''2026-10-15 stage=review | review_scope=final | reviewer_identity=当前 Codex 会话（规划作者、非本提交实施者；独立恢复及自选反例，无额外agent） | plan_id=PLAN-784 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-784/auto-lang | branch=plan-784-dev | reviewed_commit=3f084979dcabf65e363e19027df9dff0c4b6e0b9 | base_commit=7d51845ffcca4c886698caa947cbaca93bf7fdca | dependency_revisions=auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e,auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | spec_inputs/SD hash=[fingerprint](attachments/784/review-r1/fingerprint.json)、[冻结SD](attachments/784/review-r1/spec-delta-reviewed.md) | acceptance_results=AC01/02 partial,AC03 pass,AC04 fail,AC05 pending | findings=F784-R1-01(P1:重复preserve覆盖旧WIP备份),F784-R1-02(P2:verify漏检原ignored内容漂移),F784-R1-03(P3:SD三个失效链接) | evidence=[R1完整报告](attachments/784/review-r1/REVIEW.md)、[独立原始结果](attachments/784/review-r1/results.json)、[可复放反例](attachments/784/review-r1/run.py) | 正控=两仓独立恢复refs/tree/旧tag通过、真实源文件/备份hash一致、原drill4/4、两strict通过、三树guard clean、diff-check通过 | 保护=原仓/真实payload/提交实现前后未变；没有修实现/发布规范/修改ledger/merge/移树 | tasks=保留T-01/T-04，重开T-02/T-03/T-05，2/7 | pending_closeout_gates=Category A禁止cargo整档；T-07规范沉淀/归档/交接/guard清理仍pending | next=work原树修复三项、提交并更新覆盖与收据后再review；不可覆盖真实旧快照来关闭finding。

'''
assert '## 10. 待澄清事项' in s
s=s.replace('## 10. 待澄清事项',record+'## 10. 待澄清事项',1)
p.write_text(s,encoding='utf-8')
p=root/'docs/plans/KNOWN-DEBT-AND-RISKS.md'
s=p.read_text(encoding='utf-8')
entry='''### Plan 784（2026-10-15，final review R1）

审面 `3f084979dcabf65e363e19027df9dff0c4b6e0b9`；**needs_fix / executing 2/7**。真实两仓 bundle 已独立恢复；原仓与实际 payload 未变。发现交付工具的重跑保护和漂移校验缺口，不能签发 pass。
[R1完整报告](attachments/784/review-r1/REVIEW.md)、[原始结果](attachments/784/review-r1/results.json)、[冻结SD](attachments/784/review-r1/spec-delta-reviewed.md)。

| id | 级别 | 缺口 |
|---|---|---|
| F784-R1-01 | P1 / blocking | 同payload重复preserve仅复用bundle，却覆盖旧未提交work/head/diff及manifest；A→B反例exit0且A恢复点丢失。须在覆盖前拒绝输入漂移或建立不可覆盖的新快照。 |
| F784-R1-02 | P2 / blocking | verify只校对ignored/untracked备份，没有比较原文件；ignored源内容变化后仍exit0/ok。须检查明确保留源的存在/hash并加入变更与删除控制。 |
| F784-R1-03 | P3 / closeout | 新repository-boundaries规范三处../plans链接解析到不存在的docs/specs/plans；修复并重新冻结SD。 |

保留T-01/T-04，重开T-02/T-03/T-05；同树修复提交后复审。原drill4/4、两strict、三树guard及diff-check通过，不能替代上述独立反例。Category A未跑cargo整档；未merge或清理准备树。以上属于本合同未闭合项，登记债务不代表批准延后。

'''
assert '### Plan 784（2026-10-15，final review R1）' not in s
p.write_text(entry+s,encoding='utf-8')
print('Recorded needs_fix; retained T-01/T-04; reopened T-02/T-03/T-05; 2/7.')
