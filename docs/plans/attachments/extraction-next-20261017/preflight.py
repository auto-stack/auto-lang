"""Read-only contract checks; writes only this batch's evidence receipts."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

root = Path(__file__).resolve().parents[4]
out = Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
git = lambda *args: subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()
design = 'docs/design/37-v06-modularization-and-repo-boundaries.md'
route = 'docs/plans/reports/v06-extraction-execution-roadmap.md'
report = 'docs/plans/reports/extraction-next-parallel-order-20261017.md'

if '--snapshot' in sys.argv:
    files = [
        'AGENTS.md', 'Cargo.toml', 'crates/auto-lang/Cargo.toml',
        'crates/auto-man/Cargo.toml', 'crates/auto-lang/src/parser_bridge.rs',
        'crates/auto-lang/src/dialect.rs', 'crates/auto-lang/src/dialect/ui.rs',
        'crates/auto-lang/src/parser.rs', 'crates/auto-lang/src/session.rs',
        'crates/auto-lang/src/ast.rs', 'crates/auto-lang/src/vm/native.rs',
        'crates/auto-lang/src/vm/native_catalog.rs',
        'crates/auto-lang/src/vm/host_capabilities.rs',
        'crates/auto-lang/src/ui/code_editor/mod.rs',
        'crates/auto-man/src/asset.rs', 'crates/auto-man/src/automan.rs',
        'crates/auto-man/src/util.rs', 'crates/auto-man/src/exporter/iar.rs',
        'crates/auto-man/src/exporter/ghs.rs',
        'crates/auto-gen-core/src/generate.rs', '.autoos/module-boundaries.toml',
        'docs/specs/overview.md', 'docs/specs/goals.md',
        'docs/specs/auto-lang/frontend/overview.md',
        'docs/specs/auto-lang/frontend/design/dialect-system.md',
        'docs/specs/auto-lang/vm/overview.md', 'docs/specs/auto-lang/ui/overview.md',
        'docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md',
        'docs/specs/auto-lang/interpreter/design/template-evaluation.md',
        'docs/specs/auto-man/project.md', 'docs/specs/auto-gen-core/project.md',
        'docs/specs/package-resolver/project.md',
        'docs/specs/repository-boundaries/generated-ui-distribution.md',
        'docs/plans/799-frontend-ast-model-extraction.md',
        'docs/plans/archive/800-man-dependency-model-extraction.md',
        'docs/plans/archive/792-generated-ui-independent-build.md',
        'docs/plans/archive/794-scroll-host-provider.md',
        design, route, report,
    ]
    for directory in ['crates/auto-lang/src/ast',
                      'crates/auto-lang/src/ui/code_editor/core',
                      'crates/auto-man/assets/templates',
                      'crates/auto-man/assets/builders/ghs',
                      'crates/auto-man/assets/builders/iar',
                      'test/fixtures/modularity/ui-dialect-consumer']:
        files += [p.relative_to(root).as_posix() for p in (root/directory).rglob('*')
                  if p.is_file() and 'target' not in p.parts and p.suffix != '.lock']
    data = {
        'date': '2026-10-17',
        'initial_commit': '143aff0a258fdca8bda9a3b8068dc8a9cced623c',
        'reconciled_commit': git('rev-parse', 'HEAD'),
        'concurrent_update': '800 delivered 02b11c4b4, archived 8/8 and cleaned; 799 executing, not delivered',
        'inputs': {p: sha(root/p) for p in sorted(set(files))},
        'absence_at_snapshot': ['crates/auto-ast/Cargo.toml', 'docs/specs/auto-ast/project.md'],
        'design_before_batch_sha256': hashlib.sha256(subprocess.check_output(
            ['git', '-C', str(root), 'show', '143aff0a2:'+design])).hexdigest(),
        'scope': 'Plan/design bookkeeping only; no Cargo/docs_gen/implementation/agent/worktree/old-repository changes',
    }
    (out/'inputs.json').write_text(json.dumps(data, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')

data = json.loads((out/'inputs.json').read_text(encoding='utf-8'))
for rel, digest in data['inputs'].items():
    assert sha(root/rel) == digest, f'stale input: {rel}'
for rel in data['absence_at_snapshot']:
    assert not (root/rel).exists(), f'prerequisite changed, reconcile: {rel}'

allocation = json.loads((out/'allocation.json').read_text(encoding='utf-8'))
allplans = [p for directory in [root/'docs/plans', root/'docs/plans/archive']
            for p in directory.glob('*.md') if re.match(r'^\d{3}-', p.name)]
rows = []
for item in allocation:
    n = item['id']
    p = root/item['path']
    s = p.read_text(encoding='utf-8')
    assert f'plan_id: PLAN-{n}\n' in s
    for field in ['status: drafting', 'plan_revision: 1', 'current_step: 0', 'total_steps: 8']:
        assert field+'\n' in s, (n, field)
    assert [int(x) for x in re.findall(r'^## (\d+)\.', s, re.M)] == list(range(11))
    assert re.findall(r'^\| \[ \] \| (T-\d+) \|', s, re.M) == [f'T-{i:02}' for i in range(1, 9)]
    assert re.findall(r'^\| AC-\d+ \| (work|review|merge) \|', s, re.M) == ['work']*4+['review','merge']
    assert re.findall(r'^\| \[ \] \| T-\d+ \| (work|review|merge) \|', s, re.M) == ['work']*6+['review','merge']
    assert len([q for q in allplans if q.name.startswith(f'{n}-')]) == 1
    assert '### 规范增量' in s and 'merge门pending' in s
    deps = [int(x) for x in re.findall(r'PLAN-(\d+)', re.search(r'^depends_on: (.*)$', s, re.M)[1])]
    states = {}
    for d in deps:
        matches = [q for q in allplans if q.name.startswith(f'{d}-')]
        assert len(matches) == 1
        state = re.search(r'^status: (\w+)', matches[0].read_text(encoding='utf-8'), re.M)[1]
        assert state == 'archived' or (n == 801 and d == 799 and state == 'executing'), (n,d,state)
        states[d] = state
    assert ('outcome: blocked' if n == 801 else 'outcome: pass') in s
    rows.append({'plan': n, 'revision': 1, 'tasks': 8, 'criteria': 6,
                 'outcome': 'blocked' if n == 801 else 'pass',
                 'prerequisites': states, 'sha256': sha(p)})

checked_links = 0
for rel in [a['path'] for a in allocation]+[report, route, (out/'preflight.md').relative_to(root).as_posix()]:
    p = root/rel
    for target in re.findall(r'\]\(([^)]+)\)', p.read_text(encoding='utf-8')):
        if '://' in target:
            continue
        destination = p.parent/target.split('#')[0]
        # This receipt is generated after validation; no other missing target is allowed.
        assert destination.exists() or destination.resolve() == (out/'preflight.json').resolve(), (rel,target)
        checked_links += 1
ids = [int(re.match(r'^(\d{3})-', p.name)[1]) for p in allplans]
# Historical archive prefixes predate the atomic allocator and include duplicates.
# New IDs were checked individually above; do not expand this batch into archive renumbering.
legacy_duplicates = sorted(n for n in set(ids) if ids.count(n) > 1)
assert int((root/'docs/plans/.next-id').read_text().strip()) == max(ids)+1
assert not (root/'docs/plans/.new-plan.lock').exists()
changed = [a['path'] for a in allocation]+[design,route,report,'docs/plans/.next-id',out.relative_to(root).as_posix()]
check = subprocess.run(['git','-C',str(root),'diff','--check','--',*changed],capture_output=True,text=True)
assert check.returncode == 0, check.stdout+check.stderr
result = {'stage':'new', 'contract_preflight':'pass', 'date':'2026-10-17',
          'source_commit':data['reconciled_commit'], 'plans':rows,
          'parallel_now':[799,802,803], 'hard_gate': '799 archived -> 801 work',
          'local_links_checked':checked_links, 'input_files_checked':len(data['inputs']),
          'checks':'New IDs unique; sections/tasks/AC/owners, actual dependency states, local links, SHA256, diff whitespace',
          'historical_duplicate_prefixes_unchanged':legacy_duplicates,
          'not_executed':'No implementation or runtime/Cargo tests; drafting contract validation only'}
(out/'preflight.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,ensure_ascii=False))
