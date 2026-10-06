from pathlib import Path
import re,json,hashlib
R=Path('D:/autostack/auto-lang')
P=R/'docs/plans/741-ac-hir-native-core.md'
s=P.read_text(encoding='utf8')
st={}
for state,tid in re.findall(r'^- \[([ x])\] \*\*(T-\d+)\*\*',s,re.M):st.setdefault(tid,set()).add(state)
assert len(st)==38 and sum(v=={'x'} for v in st.values())==26 and all(len(v)==1 for v in st.values())
assert 'current_step: 26\ntotal_steps: 38' in s and 'plan_revision: 6' in s and 'status: executing' in s
assert not (R/'docs/plans/archive/741-ac-hir-native-core.md').exists()
assert sum(p.name=='741-ac-hir-native-core.md' for p in (R/'docs/plans').rglob('741-*.md'))==1
def checklinks(p):
    text=p.read_text(encoding='utf8');text=re.sub(r'```.*?```','',text,flags=re.S);text=re.sub(r'`[^`]*`','',text)
    count=0
    for target in re.findall(r'\]\(([^)#\s]+)(?:#[^)]*)?\)',text):
        if '://' in target or target.startswith('/'):continue
        assert (p.parent/target).resolve().exists(), (p,target)
        count+=1
    return count
result={'active_plan_links':checklinks(P),'review_report_links':checklinks(R/'docs/reports/741-r5-quality-review-20261006/REVIEW.md'),'current':26,'total':38,'revision':6}
for mod in ('auto-ac','auto-hir'):
    p=R/f'docs/specs/{mod}/plans.md';row=[ln for ln in p.read_text(encoding='utf8').splitlines() if ln.startswith('| 741 |')]
    assert len(row)==1 and 'plans/741-ac-hir-native-core.md' in row[0] and 'executing' in row[0]
assert 'docs/plans/741-ac-hir-native-core.md' in (R/'experimental/ac-core/README.md').read_text(encoding='utf8')
print(json.dumps(result,indent=2))
