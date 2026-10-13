from pathlib import Path
import re,json,hashlib,gzip
R=Path('D:/autostack/auto-lang');D=R/'docs/reports/743-r5-quality-review-20261006';P=R/'docs/plans/743-acc-bootstrap-hir-contract.md'
s=P.read_text(encoding='utf8');done=set(re.findall(r'^- \[x\] (T-\d+) ',s,re.M));lines=s.splitlines()
for i,line in enumerate(lines):
    m=re.match(r'^\| (T-\d+) \|',line)
    if m and any('[✅' in lines[j] for j in range(i+1,min(i+4,len(lines)))):done.add(m.group(1))
ids=set(re.findall(r'^- \[[ x]\] (T-\d+) ',s,re.M))|set(re.findall(r'^\| (T-\d+) \|',s,re.M))
assert len(ids)==38 and len(done)==26
assert 'status: executing\n' in s and 'plan_revision: 6\n' in s and 'current_step: 26\ntotal_steps: 38' in s
assert not (R/'docs/plans/archive/743-acc-bootstrap-hir-contract.md').exists()
assert len(list((R/'docs/plans').rglob('743-*.md')))==1
def links(p):
    fence=None;body=[]
    for line in p.read_text(encoding='utf8').splitlines():
        token=line.strip()[:3]
        if token in ('```','~~~'):
            fence=None if fence==token else token if fence is None else fence;continue
        if fence is None:body.append(line)
    text=re.sub(r'`[^`]*`','','\n'.join(body));count=0
    for target in re.findall(r'\]\(([^)#\s]+)(?:#[^)]*)?\)',text):
        if '://' in target or Path(target).is_absolute():continue
        assert (p.parent/target).resolve().exists(),(str(p),target)
        count+=1
    return count
for mod in ('auto-acc','auto-hir'):
    p=R/f'docs/specs/{mod}/plans.md';row=[ln for ln in p.read_text(encoding='utf8').splitlines() if ln.startswith('| 743 |')]
    assert len(row)==1 and 'executing' in row[0] and 'plans/743-acc-bootstrap-hir-contract.md' in row[0]
manifest=json.loads((D/'source-manifest.json').read_text(encoding='utf8'))
for p,h in manifest['hashes'].items():
    if p.startswith('docs/plans/'):continue
    assert hashlib.sha256((R/p).read_bytes()).hexdigest()==h,(p,'changed')
assert hashlib.sha256(gzip.decompress((D/'reviewed-plan-r5.md.gz').read_bytes())).hexdigest()==manifest['hashes']['docs/plans/archive/743-acc-bootstrap-hir-contract.md']
result={'plan_links':links(P),'report_links':links(D/'REVIEW.md'),'completed':26,'total':38,'revision':6,'product_code_canonical_and_frozen_inputs_unchanged':True}
print(json.dumps(result,indent=2))
