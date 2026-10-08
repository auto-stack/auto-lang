"""Separate historical failure names from unresolved batch observations."""
import json
import re
import subprocess
from pathlib import Path

root=Path('D:/autostack/auto-lang')
packet=root/'docs/plans/attachments/745/merge'
historical={}
failure=re.compile(r'^\s*(?:FAIL|TIMEOUT) \[.+?\] \(.+?\) (auto-lang\S* .+)$',re.M)
plain=re.compile(r'^(auto-lang\S* [A-Za-z_][A-Za-z0-9_:]+)$',re.M)
paths=[]
for review in (root/'docs/plans/attachments/745').glob('review-r*'):
    paths.extend(review.glob('root-*.stderr.txt'))
    paths.extend(review.glob('daily-results.txt'))
    paths.extend(review.glob('historical-attribution.txt'))
for prefix in ('741-phase*','741-r*-quality-review*'):
    for folder in (root/'docs/reports').glob(prefix):
        paths.extend(folder.glob('*.txt'))
for p in paths:
    text=p.read_text(encoding='utf-8-sig',errors='replace')
    for name in set(failure.findall(text)+plain.findall(text)):
        historical.setdefault(name,[]).append(p.relative_to(root).as_posix())
current={}
for leg in ('tf','tt','tb'):
    path=packet/f'batch-{leg}.stderr.txt'
    if path.exists():
        text=path.read_text(encoding='utf-8-sig',errors='replace')
        names=sorted(set(failure.findall(text)))
        current[leg]={'summary':re.findall(r'^\s*Summary .+$',text,re.M),'failures':names,'historical_matches':{n:historical[n] for n in names if n in historical},'unmatched': [n for n in names if n not in historical]}
delivery=json.loads((packet/'landed.json').read_text(encoding='utf-8-sig'))
scopes=['crates','Cargo.toml','Cargo.lock','.cargo','.config','auto/lib','test/vm/aavm2','parity']
diff=subprocess.run(['git','diff',delivery['previous_main'],delivery['delivery_commit'],'--name-only','--',*scopes],cwd=root,capture_output=True,text=True,check=True).stdout.splitlines()
assert not diff, '745 introduced production changes outside its experimental workspace'
result={'historical_unique_names':len(historical),'sources_before_merge_only':True,'legs':current,'production_source_config_diff_pre_main_to_delivery':diff,'unmatched_is_not_automatically_plan_regression':True,'scope':'All 745 compiler implementation remains independent experimental/ac-core; root tf/tt/tb do not compile or call it. Root production/config/library/dependency manifests are exactly pre-landing main. Historical names support known-red attribution; unmatched names require controls/analysis, no all-green claim.'}
(packet/'batch-attribution.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
for leg, info in current.items():
    print(f'{leg}: failures={len(info["failures"])} historical={len(info["historical_matches"])} unmatched={info["unmatched"]}')
