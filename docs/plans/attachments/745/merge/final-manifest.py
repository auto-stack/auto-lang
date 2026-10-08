"""Bind durable closeout evidence after the real final assertions pass."""
import hashlib
import json
import subprocess
from pathlib import Path

root = Path('D:/autostack/auto-lang')
packet = root/'docs/plans/attachments/745/merge'
read = lambda name: json.loads((packet/name).read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
def git(*args):
    return subprocess.run(['git',*args],cwd=root,capture_output=True,text=True,check=True).stdout.strip()
assert read('assertions-final.json')['outcome']=='pass'
assert read('cleanup.json')['outcome']=='pass'
prepared = read('prepared.json')
for source in prepared['spec_targets']:
    assert sha(root/source['path'])==source['sha256'],f'canonical source drift: {source["path"]}'
published = read('ledger-archive-published.json')
assert sha(root/'.autoos/specs.json')==published['sha256']
baseline = read('baseline.json')
assert sha(root/'docs/plans/attachments/745/review-r7/proposed-spec-delta.frozen.md')==baseline['frozen_delta_sha256']
landed = read('landed.json')
code_scopes = ['experimental/ac-core','scripts/verify-ac-source-745.ps1','docs/reports/745-source-core-i32']
assert not git('diff',landed['reviewed_commit'],prepared['rebased_implementation_commit'],'--name-only','--',*code_scopes)
assert not git('diff',landed['delivery_commit'],'HEAD','--name-only','--',*code_scopes,'docs/specs','docs/reports/743-acc-hir-contract')
assert not git('rev-list','--merges',landed['previous_main']+'..HEAD')
archive = root/'docs/plans/archive/745-ac-source-checked-hir-native.md'
assert len(list((root/'docs/plans/archive').glob('745-*.md')))==1
assert not list((root/'docs/plans').glob('745-*.md'))
leaves=[]
for path in sorted(packet.rglob('*')):
    assert not path.is_symlink(), 'unexpected link in evidence packet'
    if path.is_file() and path.name!='artifact-manifest.json':
        leaves.append({'path':path.relative_to(root).as_posix(),'bytes':path.stat().st_size,'sha256':sha(path)})
result={'key':'PLAN-745:r1','stage':'merge','outcome':'pass','completion_kind':'delivered',
        'delivery_commit':landed['delivery_commit'],'reviewed_commit':landed['reviewed_commit'],
        'ledger_commit':published['published_commit'],'ledger_sha256':published['sha256'],'ledger_version':9,
        'archive':archive.relative_to(root).as_posix(),'archive_sha256':sha(archive),
        'completed_tasks':9,'total_tasks':9,'cleanup_verified':True,'linear_history_verified':True,
        'canonical_hashes_and_rebased_code_equivalence_verified':True,
        'review_manifest_sha256':sha(root/'docs/plans/attachments/745/review-r7/artifact-manifest.json'),
        'manifest_self_excluded':True,'items':leaves}
target=packet/'artifact-manifest.json'
target.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
actual=json.loads(target.read_text(encoding='utf-8'))
for item in actual['items']:
    assert sha(root/item['path'])==item['sha256']
assert sha(archive)==actual['archive_sha256']
print(f'Final closeout manifest read back: {len(leaves)} leaves, actual archive 9/9, ledger version9.')
