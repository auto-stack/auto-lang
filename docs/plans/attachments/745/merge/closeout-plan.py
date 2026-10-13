"""Shared Plan bookkeeping only; actual publication/cleanup is separately verified."""
import json
import os
import re
import sys
from pathlib import Path

root = Path('D:/autostack/auto-lang')
packet = root/'docs/plans/attachments/745/merge'
archive = root/'docs/plans/archive/745-ac-source-checked-hir-native.md'
old = root/'docs/plans/745-ac-source-checked-hir-native.md'
read = lambda name: json.loads((packet/name).read_text(encoding='utf-8-sig'))
mode = sys.argv[1]
assert mode in ('archive','final')
assert archive.is_file() and not old.exists(), 'git mv must precede archive bookkeeping'
text = archive.read_text(encoding='utf-8')
if mode == 'archive':
    assert 'status: reviewed' in text and 'current_step: 8' in text
    publication = read('ledger-published.json')
    assert publication['checkpoint']=='ledger_refreshed' and publication['main_tip_equal']
    def relocate(m):
        value = m[2]
        if re.match(r'https?://|#|codex:',value) or re.search(r'[{}]',value) or Path(value).is_absolute():
            return m[0]
        path, separator, anchor = value.partition('#')
        target = (old.parent/path).resolve()
        assert target.exists(), f'pre-archive broken source: {value}'
        return m[1]+Path(os.path.relpath(target,archive.parent)).as_posix()+(separator+anchor if separator else '')+')'
    # The Plan's links are all inline; no link in code fences is a live reference.
    parts = re.split(r'(?ms)(^```[^\n]*\n.*?^```\s*$)',text)
    for index in range(0,len(parts),2):
        parts[index] = re.sub(r'(\[[^\]]*\]\()([^)]+)\)',relocate,parts[index])
    text = ''.join(parts).replace('status: reviewed','status: archived',1)
    receipt = f'''2026-10-08 stage=merge，PLAN-745:r1：交付已验证，归档后清理待执行。

- `completion_kind`: delivered；用户显式 /auto-plan:merge 授权，落点 v0.6-dev，继续使用原专属树；未重开实施或改合同。
- `prepared`: 原终审 bb8cb4b4330d505059ea20e7bfe561a93c1051f3、原base eea7a80618d4d5e75749d666b7609a9a87925bbd、auto-down fba6563ed2148ce85e68208863159b4ccccac710；冻结SD SHA256 cd027eae6395f8903f41f1623c20af49d67ec5fdf09631b30f52b8f5485bcdb2。canonical与人工层审计均原树提交，恢复ref refs/auto-plan/745/reviewed保留。
- `landed`: 2653e0088be1307119bb106ab0f82a4f4dc004f2 已 ff-only 落地且当时主检出HEAD实际相等；24实现提交rebase全等价，无冲突/merge commit。原树当日verify745 13步74例通过，主检出check及strict47通过。
- `ledger_refreshed`: Git跟踪 .autoos/specs.json，原树提交后ff-only发布 {publication['published_commit']}；准确读回 version7、SHA256 {publication['sha256']}；复用P741-2/P743-1/P743-2，新增稳定P745-1..4，不相关项保持。归档history引用更新在原树另行提交后核验，未预称完成。
- `spec_sync`: auto-ac/source-core-i32及三阶段/能力卡片、overview与三plans导航；index实际生成无内容变化。MD408/409维持required/unknown并追加有限证据；五个失鲜人工绑定逐项审计，strict47新鲜。touched_goals=[]，完整语言/ACC主体/自举仍open。
- `batch_regression`: 到期tf/tt/tb主检出串行完成；5994/5342/5027运行，31/34/76失败保留，根生产/config无745 diff，不称全绿。两时序控制各1/1隔离通过；外部book金样可变及固定auto-term路径观察公开登记，详见[合入报告](../attachments/745/merge/MERGE.md)。未找到regress技能，依AGENTS明确档位执行。
- `archive_location`: docs/plans/archive/745-ac-source-checked-hir-native.md；status archived、8/9，T09清理待完成；实际链接/导航/账本/计数门禁待history引用发布后执行，不预记pass。
- `cleaned`: pending，原树/只读依赖树保留至真实归档门禁通过；须fresh guard clean、Git clean/全部落地及目标归属后移除树/合并分支/普通空组。
- `evidence`: [合入报告](../attachments/745/merge/MERGE.md)、merge/landed.json、ledger-published.json、binding-audit.json、批量原始日志及外仓审计；原第七轮证据仍绑定原review SHA。

'''
    assert '## 10. 待澄清事项' in text
    text = text.replace('## 10. 待澄清事项',receipt+'## 10. 待澄清事项',1)
    debt = root/'docs/plans/KNOWN-DEBT-AND-RISKS.md'
    debt.write_text(debt.read_text(encoding='utf-8').replace('](745-ac-source-checked-hir-native.md)','](archive/745-ac-source-checked-hir-native.md)'),encoding='utf-8')
else:
    cleanup = read('cleanup.json')
    gate = read('assertions-archived.json')
    assert cleanup['outcome']=='pass' and gate['outcome']=='pass'
    assert not Path('D:/autostack/.wt/lang-745').exists()
    assert 'current_step: 8' in text and '| T-09 | [ ] |' in text
    text = text.replace('current_step: 8','current_step: 9',1).replace('| T-09 | [ ] |','| T-09 | [x] |',1)
    publication = read('ledger-archive-published.json')
    receipt = f'''2026-10-08 stage=merge，PLAN-745:r1：**pass**，completion_kind: delivered，AC09已完成。

- `archived`: 实际归档身份、相对链接、三模块导航、canonical来源SHA、ledger七项身份及8/9待清理计数已通过assertions-archived.json；无假archive路径或错误Plan替代。
- `ledger_refreshed`: 原树归档history引用更新，仅P745-3/4改变，ff-only发布 {publication['published_commit']}；主检出准确读回version9、SHA256 {publication['sha256']}，canonical项仍指Specs，其余记录保持。
- `cleaned`: 原树auto-lang与组内只读auto-down分别fresh wt-guard clean、无reparse、Git clean、HEAD落地/依赖fba保持后移除；plan-745-dev已用merged branch删除，组内空后普通目录删除，Git登记与磁盘均确认无残留。恢复ref refs/auto-plan/745/reviewed与主检出ignored原始ledger备份有意保留。详见[清理收据](../attachments/745/merge/cleanup.json)。
- `acceptance_results`: AC01..08沿用第七轮独立pass并经当日整合验证，AC09真实closeout pass；T01..09全[x]、9/9。最终实际门禁再查归档/链接/ledger/清理，另含错误ledger Plan、错误745导航与错误终态计数三负控；结果保存assertions-final.json，合入packet SHA清单保存artifact-manifest.json。
- `limits`: 根批量档未全绿；环境/可变外部book金样观察已公开，外仓修改保留，未把重新生成金样当oracle。后续完整语言、ABI及ACC主体未交付；无未经批准延期或workaround。

'''
    text = text.replace('## 10. 待澄清事项',receipt+'## 10. 待澄清事项',1)
archive.write_text(text,encoding='utf-8')
print(f'Plan actual {mode} bookkeeping saved.')
