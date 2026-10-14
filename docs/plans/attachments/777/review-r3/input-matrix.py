"""R3 independent output-input identity matrix. Hardlinks are only in owned temp copies."""
import json, os, shutil, stat, sys, tempfile
from pathlib import Path
from replay import cli, git, commit, digest, summarize
IMPL = Path('D:/autostack/.wt/lang-777/auto-lang')
SCRIPT = IMPL / 'scripts/modularity-audit.py'
OUT = Path(__file__).resolve().parent
container = Path(tempfile.mkdtemp(prefix='plan777-inputs-r3-')).resolve()
cases = []
try:
    for label, rel in [('config', '.autoos/module-boundaries.toml'),
                       ('manifest', 'crates/fixture-lang/Cargo.toml'),
                       ('source', 'crates/fixture-lang/src/lib.rs')]:
        root = container / label
        shutil.copytree(IMPL / 'test/fixtures/modularity/audit', root)
        git(root, 'init', '-q')
        commit(root, 'valid source before output alias')
        original = root / rel
        alias = container / (label + '-output.json')
        os.link(original, alias)
        assert original.samefile(alias)
        before = original.read_bytes()
        before_hash = digest(original)
        q = cli(SCRIPT, root, '--mode', 'observe', '--output', str(alias))
        cases.append({'id': 'M01-hardlink-' + label, 'input_path': rel,
                      'input': before.decode('utf-8'), 'alias_samefile': True,
                      'expected': 'nonzero exit and unchanged input bytes',
                      'actual': summarize(q), 'before_sha256': before_hash,
                      'after_sha256': digest(original),
                      'contract_pass': q['exit_code'] != 0 and original.read_bytes() == before})
    root = container / 'positive'
    shutil.copytree(IMPL / 'test/fixtures/modularity/audit', root)
    git(root, 'init', '-q')
    commit(root, 'valid independent output')
    before = digest(root / 'crates/fixture-lang/src/lib.rs')
    destination = container / 'separate-report.json'
    q = cli(SCRIPT, root, '--mode', 'observe', '--output', str(destination))
    cases.append({'id': 'M02-independent-output', 'expected': 'exit0, report valid, source unchanged',
                  'actual': summarize(q), 'contract_pass': q['exit_code'] == 0
                  and json.loads(destination.read_text())['schema'].startswith('modularity-audit')
                  and before == digest(root / 'crates/fixture-lang/src/lib.rs')})
finally:
    assert container.parent == Path(tempfile.gettempdir()).resolve()
    def clear_readonly(function, path, exc):
        assert Path(path).resolve().is_relative_to(container)
        os.chmod(path, Path(path).stat().st_mode | stat.S_IWRITE)
        function(path)
    shutil.rmtree(container, onexc=clear_readonly)
(OUT / 'input-matrix.jsonl').write_text(json.dumps({'head': git(IMPL, 'rev-parse', 'HEAD'), 'cases': cases}, ensure_ascii=False) + '\n', encoding='utf-8')
for c in cases:
    print(json.dumps({'id': c['id'], 'exit_code': c['actual']['exit_code'], 'contract_pass': c['contract_pass']}))
sys.exit(0 if all(c['contract_pass'] for c in cases) else 1)
