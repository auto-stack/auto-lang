#!/usr/bin/env python3
"""Plan 823: fail closed on incomplete, mislabelled or stale typed evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

BINDING = 'crates/auto-lang/test/cases/ace-m1/binding.tsv'
MANIFEST = 'docs/plans/attachments/823/852-validation/evidence-identity.json.txt'
REQUIRED = [
    BINDING,
    'Cargo.lock', '.cargo/config.toml',
    'experimental/ace/Cargo.toml', 'experimental/ace/Cargo.lock',
    'experimental/ace/crates/artifacts/Cargo.toml', 'experimental/ace/crates/artifacts/Cargo.lock',
    'crates/auto-lang/test/cases/av-data/823/av-data-823.suite.ad',
    'crates/auto-lang/test/cases/av-data/823/av-closures-first-case.case.ad',
    'crates/auto-lang/test/cases/av-data/823/src/first-case.at',
    'crates/auto-lang/src/tests/av_closures_823_tests.rs',
    'crates/auto-lang/src/tests/ac_av_probe_tests.rs',
    'crates/auto-lang-core/src/test_cases/ac_av.rs',
    'crates/auto-test-case/src/contract.rs',
    'experimental/ace/crates/artifacts/src/store/gc.rs',
    'experimental/ace/crates/artifacts/src/store/mod.rs',
    'experimental/ace/crates/av/Cargo.toml',
    'experimental/ace/crates/av-core/Cargo.toml',
    'experimental/ace/crates/artifacts/tests/plan823_gc_metadata.rs',
    *['experimental/ace/crates/av-core/src/' + name + '.rs' for name in
      ('abc', 'abc_verify', 'av', 'closures', 'contract', 'mir_verify', 'ownership', 'program_snapshot')],
    'experimental/ace/crates/av/src/closures_fixture.rs',
    'experimental/ace/crates/av/src/closures_session.rs',
    'experimental/ace/crates/av/src/formal_campaign.rs',
    'experimental/ace/crates/av/src/formal_worker.rs',
    'experimental/ace/crates/av/src/load256.rs',
    'experimental/ace/crates/av/src/bin/av-worker.rs',
    *['experimental/ace/crates/av/tests/av_closures_823_' + name + '.rs' for name in
      ('first_case', 'semantics', 'resources', 'cache_compat', 'incremental', 'recipe')],
    'scripts/verify-av-closures-823.ps1',
    'scripts/verify-av-closures-823-bindings.py',
]
K_IDS = [f'K{i:02}' for i in range(1, 7)]
SCENARIOS = ['R05', 'I01', 'I02', 'I04', 'I05', 'I06', 'I07', 'I10', 'I11', 'I12']
MARKERS = {'I01': 'closures session body-edit identity', 'I02': 'closures body-only edit',
           'I04': 'env-layout drift', 'I05': 'historical admitted-code identity hit',
           'I06': 'cross-project identical artifact identity', 'I07': 'paused live closure',
           'I10': '512 distinct persisted body revisions', 'I11': '823 r9: 10 typed',
           'I12': 'judge lie-controls'}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(root, check_dependencies=True):
    doc = json.loads((root / MANIFEST).read_text(encoding='utf-8'))
    files = doc.get('files', {})
    if set(files) != set(REQUIRED) or doc.get('plan_revision') != 10:
        raise ValueError('required evidence identity set or plan revision differs')
    for name in REQUIRED:
        if digest(root / name) != files[name]:
            raise ValueError(f'stale evidence input: {name}')
    if check_dependencies:
        deps = doc.get('dependencies', {})
        if set(deps) != {'auto-down', 'auto-ui'}:
            raise ValueError('dependency identity set differs')
        for name, expected in deps.items():
            directory = root.parent / name
            rev = subprocess.run(['git', '-C', str(directory), 'rev-parse', 'HEAD'], check=True, text=True, capture_output=True).stdout.strip()
            dirty = subprocess.run(['git', '-C', str(directory), 'status', '--porcelain'], check=True, text=True, capture_output=True).stdout
            if rev != expected or dirty:
                raise ValueError(f'stale or dirty dependency: {name}')
    rows = {}
    for line in (root / BINDING).read_text(encoding='utf-8').splitlines():
        if not line or line.startswith('#'):
            continue
        cols = line.split('\t')
        if len(cols) != 12:
            raise ValueError('binding row must have 12 fields')
        key = (cols[0], cols[1])
        if key in rows:
            raise ValueError(f'duplicate binding: {key}')
        rows[key] = cols
    for ident in K_IDS:
        row = rows.get((ident, 'corpus'))
        if row is None or row[6] != 'ok:av-closures-823' or 'typed evidence' not in row[11]:
            raise ValueError(f'{ident} must be an explicitly typed 823 binding')
    row = rows.get(('R05', 'scenario'))
    if row is None or row[6] != 'ok:av-closures-823' or 'typed evidence' not in row[11]:
        raise ValueError('R05 must be an explicitly typed 823 binding')
    for ident in SCENARIOS[1:]:
        row = rows.get((ident, 'scenario'))
        if row is None or MARKERS[ident] not in row[11]:
            raise ValueError(f'missing scenario binding or 823 evidence: {ident}')
    return True


def self_test(root):
    with tempfile.TemporaryDirectory(prefix='plan823-bindings-') as tmp:
        sample = Path(tmp)
        for name in REQUIRED + [MANIFEST]:
            dst = sample / name
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(root / name, dst)
        verify(sample, check_dependencies=False)
        binding = sample / BINDING
        original = binding.read_text(encoding='utf-8')
        manifest = sample / MANIFEST
        def refresh_binding():
            obj = json.loads(manifest.read_text(encoding='utf-8'))
            obj['files'][BINDING] = digest(binding)
            manifest.write_text(json.dumps(obj), encoding='utf-8')
        def must_fail(label):
            try:
                verify(sample, check_dependencies=False)
            except (ValueError, OSError, KeyError):
                return
            raise AssertionError(f'{label} unexpectedly passed')
        binding.write_text('\n'.join(x for x in original.splitlines() if not x.startswith('K02\tcorpus\t')) + '\n', encoding='utf-8')
        refresh_binding()
        must_fail('missing required row even with updated hash')
        lines = original.splitlines()
        for i, line in enumerate(lines):
            if line.startswith('K01\tcorpus\t'):
                lines[i] = line.replace('typed evidence', 'source evidence')
        binding.write_text('\n'.join(lines) + '\n', encoding='utf-8')
        refresh_binding()
        must_fail('mislabelled source even with updated hash')
        binding.write_text(original, encoding='utf-8')
        refresh_binding()
        changed = sample / 'experimental/ace/crates/artifacts/src/store/gc.rs'
        changed.write_bytes(changed.read_bytes() + b'\n// stale evidence control\n')
        must_fail('changed code with stale evidence')
    return True


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    try:
        verify(args.root)
        if args.self_test:
            self_test(args.root)
        print('PASS: 823 typed bindings and input identities' + ('; negative controls' if args.self_test else ''))
        return 0
    except (ValueError, OSError, KeyError, AssertionError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f'FAIL: 823 typed binding/identity: {error}', file=sys.stderr)
        return 1

if __name__ == '__main__':
    sys.exit(main())
