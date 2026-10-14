"""Independent R2 combinations and external input probes; only owned temp copies mutate."""
import json, shutil, tempfile, stat, os, tomllib, sys
from pathlib import Path
from replay import cli, git, commit, summarize, digest, cargo_oracle

IMPL = Path('D:/autostack/.wt/lang-777/auto-lang')
SCRIPT = IMPL / 'scripts/modularity-audit.py'
EVIDENCE = Path(__file__).resolve().parent
container = Path(tempfile.mkdtemp(prefix='plan777-r2-')).resolve()
results = []
production = tomllib.loads((IMPL / '.autoos/module-boundaries.toml').read_text(encoding='utf-8'))
def fresh(name):
    repo = container / name
    shutil.copytree(IMPL / 'test/fixtures/modularity/audit', repo)
    git(repo, 'init', '-q')
    return repo
def freeze(repo):
    commit(repo, 'freeze input configuration')
    q = cli(SCRIPT, repo, '--refresh-baseline')
    assert q['exit_code'] == 0, q
def external(name, package):
    target = container / name
    shutil.copytree(IMPL / 'test/fixtures/modularity/audit/crates/fixture-lang-sdk', target)
    manifest = target / 'Cargo.toml'
    manifest.write_text(manifest.read_text().replace('fixture-lang-sdk', package), encoding='utf-8')
    return target
try:
    # Real production domain table and rule, without adding a missing mapping for auto-ui.
    for package, expected in [('auto-ui', 2), ('auto-man', 2), ('external-sdk', 0)]:
        repo = fresh('external-' + package)
        target = external(package, package)
        cfgpath = repo / '.autoos/module-boundaries.toml'
        cfg = cfgpath.read_text(encoding='utf-8')
        members = dict(production['domains']['members'])
        members['fixture-lang'] = 'lang-monolith'
        members['fixture-lang-sdk'] = 'lang-core'
        members['fixture-ui'] = members['fixture-ui-kit'] = 'ui'
        begin = cfg.index('[domains.members]')
        end = cfg.index('[[rules.forbidden_declared]]')
        cfg = cfg[:begin] + '[domains.members]\n' + ''.join(
            json.dumps(k) + ' = ' + json.dumps(v) + '\n' for k, v in members.items()) + '\n' + cfg[end:]
        rule = next(r for r in production['rules']['forbidden_declared'] if r['id'] == 'out-of-root-edges-must-be-optional')
        encoded = '\n[[rules.forbidden_declared]]\n' + ''.join(
            k + ' = ' + json.dumps(v) + '\n' for k, v in rule.items())
        cfgpath.write_text(cfg + encoded, encoding='utf-8')
        freeze(repo)
        manifest = repo / 'crates/fixture-lang/Cargo.toml'
        before = manifest.read_text()
        after = before + '\n[dependencies]\n' + package + ' = { path = "../../../' + package + '" }\n'
        manifest.write_text(after, encoding='utf-8')
        run = cli(SCRIPT, repo, '--mode', 'strict')
        oracle = cargo_oracle(repo)
        results.append({'id': 'S01-' + package, 'expected_exit': expected,
                        'production_rule': rule, 'domain': members.get(package, 'unclassified'),
                        'mutation': {'before': before, 'after': after}, 'actual': summarize(run),
                        'cargo_oracle': oracle, 'contract_pass': run['exit_code'] == expected})
        # The external manifest was read to classify this dependency, hence is audit input.
        if package == 'external-sdk':
            destination = target / 'Cargo.toml'
            saved = destination.read_text()
            before_hash = digest(destination)
            run = cli(SCRIPT, repo, '--mode', 'observe', '--output', str(destination))
            results.append({'id': 'S02-external-manifest-output', 'expected': 'nonzero, external input unchanged',
                            'input': saved, 'before_sha256': before_hash, 'after_sha256': digest(destination),
                            'actual': summarize(run), 'output_prefix': destination.read_text()[:200],
                            'contract_pass': run['exit_code'] != 0 and digest(destination) == before_hash})
    repo = fresh('mixed-target-inherited')
    manifest = repo / 'Cargo.toml'
    manifest.write_text(manifest.read_text() + '\n[workspace.dependencies]\nalias-ui = { package="fixture-ui-kit", path="crates/fixture-ui-kit" }\n', encoding='utf-8')
    freeze(repo)
    manifest = repo / 'crates/fixture-lang/Cargo.toml'
    before = manifest.read_text()
    after = before + '\n[target.\'cfg(windows)\'.build-dependencies]\nalias-ui = { workspace=true }\n' + '\n[target.\'cfg(unix)\'.dev-dependencies]\nalias-ui = { workspace=true }\n'
    manifest.write_text(after, encoding='utf-8')
    run = cli(SCRIPT, repo, '--mode', 'strict')
    violations = (run['report'] or {}).get('strict', {}).get('violations', [])
    results.append({'id': 'S03-target-workspace-alias-build-dev', 'expected': 'only Windows build UI forbidden',
                    'mutation': {'before': before, 'after': after}, 'actual': summarize(run),
                    'cargo_oracle': cargo_oracle(repo), 'contract_pass': run['exit_code'] == 2 and len(violations) == 1
                    and violations[0]['edge']['kind'] == 'build' and violations[0]['edge']['target'] == 'cfg(windows)'})
finally:
    assert container.parent == Path(tempfile.gettempdir()).resolve()
    def clear_readonly(function, path, exc):
        assert Path(path).resolve().is_relative_to(container)
        os.chmod(path, Path(path).stat().st_mode | stat.S_IWRITE)
        function(path)
    shutil.rmtree(container, onexc=clear_readonly)
(EVIDENCE / 'supplemental.jsonl').write_text(json.dumps({'implementation_head': git(IMPL, 'rev-parse', 'HEAD'), 'cases': results}, ensure_ascii=False) + '\n', encoding='utf-8')
for r in results:
    print(json.dumps({'id': r['id'], 'contract_pass': r['contract_pass'], 'expected': r.get('expected_exit', r.get('expected')), 'actual_exit': r['actual']['exit_code']}, ensure_ascii=False))
sys.exit(0 if all(r['contract_pass'] for r in results) else 1)
