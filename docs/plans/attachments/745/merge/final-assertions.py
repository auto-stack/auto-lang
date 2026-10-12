"""Actual archive/cleanup gate for PLAN-745, with meaningful identity controls."""
import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path

root = Path('D:/autostack/auto-lang').resolve()
packet = root/'docs/plans/attachments/745/merge'
archive = root/'docs/plans/archive/745-ac-source-checked-hir-native.md'
active = root/'docs/plans/745-ac-source-checked-hir-native.md'

def git(*args):
    p = subprocess.run(['git',*args],cwd=root,capture_output=True,text=True,check=True)
    return p.stdout.strip()

def require(condition,message):
    if not condition:
        raise AssertionError(message)

def prose(text):
    return re.sub(r'(?ms)^```[^\n]*\n.*?^```\s*$', '', text)

def check_links(path,text):
    for m in re.finditer(r'\[[^\]]*\]\(([^)]+)\)',prose(text)):
        value = m[1].strip('<>')
        if re.match(r'https?://|#|codex:',value) or re.search(r'[{}]',value):
            continue
        value = value.split('#',1)[0]
        require((path.parent/value).resolve().exists(),f'broken link {path}: {value}')

def check_navigation(module,text):
    rows = [r for r in text.splitlines() if re.match(r'^\| 745 \|',r)]
    require(len(rows)==1,f'{module}: expected one 745 row')
    require('delivered' in rows[0],f'{module}: delivery state missing')
    matches = re.findall(r'\[[^\]]+\]\(([^)]+)\)',rows[0])
    candidates = [(root/f'docs/specs/{module}'/p).resolve() for p in matches if '745-' in p]
    require(candidates == [archive],f'{module}: wrong Plan identity')

def check_ledger(data):
    records = {i['id']:(s['id'],i) for s in data['sections'] for i in s['items']}
    expected = {'P741-2':('designs','docs/specs/auto-ac/project.md'), 'P743-1':('designs','docs/specs/auto-acc/project.md'), 'P743-2':('designs','docs/specs/auto-hir/stage-contract.md'), 'P745-1':('designs','docs/specs/auto-ac/source-core-i32.md'), 'P745-2':('tests','docs/specs/auto-ac/source-core-i32.md'), 'P745-3':('reviews',archive.relative_to(root).as_posix()),'P745-4':('reports',archive.relative_to(root).as_posix())}
    require(data['project']=='auto-lang','wrong ledger workspace')
    for ident,(section,file) in expected.items():
        require(ident in records,f'missing ledger ID {ident}')
        observed,item = records[ident]
        require(observed==section and item['file']==file,f'wrong ledger identity {ident}')
        require((root/file).is_file(),f'missing ledger source {ident}')
        require(item['provenance']['plan']=='PLAN-745' and item['provenance']['revision']==1,f'wrong provenance {ident}')
        if section not in ('reviews','reports'):
            require(item['source_sha256']==hashlib.sha256((root/file).read_bytes()).hexdigest(),f'stale current ledger source {ident}')

def check_plan(text,phase):
    require(re.search(r'(?m)^plan_id: PLAN-745$',text),'wrong Plan identity')
    require(re.search(r'(?m)^status: archived$',text),'not actually archived')
    require(re.search(r'(?m)^plan_revision: 1$',text),'wrong revision')
    rows = re.findall(r'(?m)^\| (T-\d\d) \| \[([ x])\] \|',text)
    require(len(rows)==9 and len({r[0] for r in rows})==9,'task identity/count mismatch')
    completed = sum(mark=='x' for _,mark in rows)
    wanted = 9 if phase=='final' else 8
    require(completed==wanted,f'completed task count {completed} != {wanted}')
    require(re.search(fr'(?m)^current_step: {wanted}$',text) and re.search(r'(?m)^total_steps: 9$',text),'frontmatter/task count mismatch')
    require('PLAN-745:r1' in text and 'completion_kind' in text,'merge receipt missing')

def controls(text,data):
    outcomes=[]
    bad=data.copy()
    bad=copy.deepcopy(data)
    next(i for s in bad['sections'] for i in s['items'] if i['id']=='P745-3')['file']='docs/plans/archive/743-acc-bootstrap-hir-contract.md'
    try:
        check_ledger(bad)
    except AssertionError:
        outcomes.append('wrong-ledger-plan-rejected')
    else:
        raise AssertionError('wrong ledger identity falsely accepted')
    nav=(root/'docs/specs/auto-ac/plans.md').read_text(encoding='utf-8')
    wrong=nav.replace('745-ac-source-checked-hir-native.md','743-acc-bootstrap-hir-contract.md')
    try:
        check_navigation('auto-ac',wrong)
    except AssertionError:
        outcomes.append('wrong-navigation-plan-rejected')
    else:
        raise AssertionError('wrong navigation identity falsely accepted')
    wrong=text.replace('current_step: 9','current_step: 8')
    try:
        check_plan(wrong,'final')
    except AssertionError:
        outcomes.append('wrong-final-task-count-rejected')
    else:
        raise AssertionError('wrong final count falsely accepted')
    return outcomes

ap=argparse.ArgumentParser()
ap.add_argument('--phase',choices=('archived','final'),required=True)
args=ap.parse_args()
require(archive.is_file() and not active.exists(),'archive location/state mismatch')
text=archive.read_text(encoding='utf-8')
check_plan(text,args.phase)
check_links(archive,text)
landed=json.loads((packet/'landed.json').read_text(encoding='utf-8-sig'))
subprocess.run(['git','merge-base','--is-ancestor',landed['delivery_commit'],'HEAD'],cwd=root,check=True)
for module in ('auto-hir','auto-ac','auto-acc'):
    p=root/f'docs/specs/{module}/plans.md'
    check_navigation(module,p.read_text(encoding='utf-8'))
for value in ('docs/specs/auto-ac/source-core-i32.md','docs/specs/auto-ac/project.md','docs/specs/auto-hir/stage-contract.md','docs/specs/auto-acc/project.md'):
    p=root/value
    check_links(p,p.read_text(encoding='utf-8'))
data=json.loads((root/'.autoos/specs.json').read_text(encoding='utf-8'))
check_ledger(data)
if args.phase=='final':
    require(not Path('D:/autostack/.wt/lang-745').exists(),'group remains')
    require('D:/autostack/.wt/lang-745/auto-lang' not in git('worktree','list','--porcelain'),'tree still registered')
    p=subprocess.run(['git','show-ref','--verify','refs/heads/plan-745-dev'],cwd=root,capture_output=True)
    require(p.returncode!=0,'development branch remains')
    controls_result=controls(text,data)
else:
    require(Path('D:/autostack/.wt/lang-745/auto-lang').is_dir(),'cleanup pending tree absent unexpectedly')
    controls_result=[]
result={'phase':args.phase,'outcome':'pass','archive':str(archive),'delivery_commit':landed['delivery_commit'],'completed_tasks':9 if args.phase=='final' else 8,'links_and_identities_verified':True,'canonical_sources_current':True,'ledger_ids_verified':['P741-2','P743-1','P743-2','P745-1','P745-2','P745-3','P745-4'],'controls':controls_result}
(packet/f'assertions-{args.phase}.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(f'PLAN-745 actual {args.phase} assertions passed; negative controls={len(controls_result)}.')
