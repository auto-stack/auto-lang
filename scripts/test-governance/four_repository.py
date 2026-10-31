"""Explicit four-owner checks using PLAN-896 fixed groups and native receipts.

No leaf catalog or inferred consumer graph is consulted. Source discovery is
limited to the declared Cargo targets, including receipt admission rechecks.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts/test-governance'))


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


fixed = module('four_fixed_runner', ROOT / 'scripts/fixed-tests.py')
import web_native_owners as full
LANGUAGE_GROUPS = ['integration', 'auto-generation-core', 'auto-api-gen',
                   'auto-media-core', 'a2r-std', 'auto-test-case']
OWNERS = ('language', 'web', 'ui', 'man', 'down')
EXTERNAL_LEGS = ('web', 'ui-full', 'man-t', 'man-tcore', 'man-tc', 'man-ti', 'ui-tu', 'ui-ti')


def load(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def proof(path):
    path = Path(path).resolve()
    return {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def read(record, directory=None):
    path = Path(record['path']).resolve()
    if directory is not None and not path.is_relative_to(Path(directory).resolve()):
        raise ValueError('execution evidence escapes its admitted directory')
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != record['sha256']:
        raise ValueError('execution evidence changed: ' + str(path))
    return raw


def local_output(directory, name):
    path = (directory / name).resolve()
    if not path.is_relative_to(directory.resolve()):
        raise ValueError('execution output escapes its invocation')
    return path.read_bytes()


def command_proof(record, argv, cwd, preparation=False):
    meter = record.get('meter', {})
    allowed_cleanup = False
    if preparation and meter.get('error'):
        from process_capture import compiler_auxiliaries_reaped
        allowed_cleanup = (meter.get('live_after_exit', {}).get('remaining_active') == 0
                           and compiler_auxiliaries_reaped(meter))
    if (record.get('argv') != argv or record.get('exit') != 0
            or Path(record.get('cwd', '')).resolve() != Path(cwd).resolve()
            or (meter.get('error') and not allowed_cleanup)):
        raise ValueError('execution command, checkout, exit or process tree differs')


def libtest_passes(raw, package):
    text = raw.decode('utf-8', errors='replace')
    names = re.findall(r'^test\s+(\S+)\s+\.\.\.\s+ok\s*$', text, re.MULTILINE)
    counts = re.findall(r'test result: ok\.\s+(\d+) passed;\s+0 failed;', text)
    if not names or len(names) != len(set(names)) or counts != [str(len(names))]:
        raise ValueError('actual libtest passes/summary missing or duplicated')
    return {package + '\t' + name for name in names}


def native_terminals(raw, active, allowed_skips):
    started = set()
    suite_started = suite_finished = False
    for line in raw.decode('utf-8', errors='replace').splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if event.get('type') == 'suite':
            suite_started |= event.get('event') == 'started'
            suite_finished |= event.get('event') == 'ok'
        if event.get('type') == 'test' and event.get('name') in active:
            name = event['name']
            if event.get('event') == 'started':
                started.add(name)
            elif event.get('event') in ('ok', 'failed', 'ignored') and name not in started:
                raise ValueError('test terminal without actual start')
    if not suite_started or not suite_finished or started != set(active):
        raise ValueError('native execution protocol incomplete')
    return fixed.terminal_results(raw, active, allowed_skips)


def roots():
    return {key: Path(os.environ.get('AUTO_' + ('LANG' if key == 'language' else key.upper()) + '_ROOT',
                    ROOT if key == 'language' else ROOT.parent / ('auto-' + key))).resolve()
            for key in OWNERS}


def source_input(name):
    # Schema files are compiler/SDK inputs despite living under docs/. Shared
    # Plan bookkeeping and generated reports remain producer-only outputs.
    return (not name.endswith('.md')
            and (not name.startswith(('docs/', 'reports/')) or name.startswith('docs/schema/')))


def require_clean(name, path):
    full.require_clean_tree(name, path)
    for args in (['diff', '--name-only', 'HEAD'], ['ls-files', '--others', '--exclude-standard']):
        names = subprocess.check_output(['git', '-C', str(path), *args], text=True).splitlines()
        if any(source_input(n) for n in names):
            raise ValueError(name + ': frozen source contains tracked or untracked WIP')


def validate_layout(selected_roots):
    documents = {}
    packages = {}
    for owner, root in selected_roots.items():
        if owner == 'down':
            if not (root / 'autodown').is_dir():
                raise ValueError('selected Down resource owner is absent')
            continue
        document = json.loads(subprocess.check_output(
            ['cargo', 'metadata', '--locked', '--offline', '--no-deps', '--format-version', '1'], cwd=root))
        documents[owner] = document
        for package in document['packages']:
            directory = Path(package['manifest_path']).parent.resolve()
            if package['name'] in packages and packages[package['name']] != directory:
                raise ValueError('duplicate selected package owner: ' + package['name'])
            packages[package['name']] = directory
    for document in documents.values():
        for package in document['packages']:
            for dependency in package['dependencies']:
                if dependency.get('path') and dependency['name'] in packages:
                    if Path(dependency['path']).resolve() != packages[dependency['name']]:
                        raise ValueError('actual Cargo dependency owner mismatch: ' + dependency['name'])


def snapshot(path):
    """One run's source provenance, not a discovery index or a cache key."""
    git = lambda *args: subprocess.check_output(['git', '-C', str(path), *args])
    names = git('ls-files', '-z').decode().split('\0')
    source = hashlib.sha256()
    resources = hashlib.sha256()
    for name in sorted(set(names) - {''}):
        if not source_input(name):
            continue
        if name.startswith('scripts/test-governance/catalog/') or name == 'scripts/test-governance/modules.json':
            continue  # retained migration archives are not runtime inputs
        file = path / name
        value = hashlib.sha256(file.read_bytes()).hexdigest() if file.is_file() else 'DELETED'
        row = (name + '\0' + value + '\n').encode()
        source.update(row)
        if not name.endswith(('.rs', '.py', '.toml', '.lock')):
            resources.update(row)
    lock = path / 'Cargo.lock'
    lock_hash = hashlib.sha256(lock.read_bytes() if lock.is_file() else b'consumer-locked-dependency').hexdigest()
    return {'root': str(path), 'head': git('rev-parse', 'HEAD').decode().strip(),
            'source_sha256': source.hexdigest(), 'resource_manifest_sha256': resources.hexdigest(),
            'lock_sha256': lock_hash,
            'runtime_producer': {'source_sha256': source.hexdigest(), 'lock_sha256': lock_hash},
            'diff_sha256': hashlib.sha256(git('diff', 'HEAD', '--', '.', ':(exclude)docs/plans',
                                            ':(exclude)docs/reports')).hexdigest()}


def contracts(selected_roots):
    result = full.owner_specs(selected_roots)
    for name, spec in result.items():
        spec['leg'] = name
    if set(result) != set(EXTERNAL_LEGS):
        raise ValueError('external owner contract lost or added a leg')
    return result


def selected_ids(active):
    return {r['binary'] + '\t' + r['test'] for r in active.values()}


def owner_selection(name, ids, selected, passed_media=None):
    """Original external denominators are shared by producer and consumer."""
    credit = set()
    if name == 'ui-full':
        expected, required = full.owner_baseline(name)
        media = load(ROOT / 'scripts/test-governance/media-test-migrations.json')['migrations']
        credit = {r['new'] for r in media if r['old'] in full.ui_original_baseline()[1]}
        moved = {r['old'] for r in media}
        full.validate_ids(ids, selected, expected - moved, required - moved)
        if passed_media is not None and not credit.issubset(passed_media):
            raise ValueError('original UI migration credit lacks actual media PASS')
    if name != 'web':
        full.check_minimum(full.minimums(), name, len(selected) + len(credit))
    if name == 'web':
        rows = load(ROOT / 'scripts/test-governance/web-test-migrations.json')['migrations']
        required = {r['new']['binary'] + '\t' + r['new']['test'] for r in rows if not r['original_ignored']}
        if not required.issubset(selected):
            raise ValueError('Web selection lost original migrated required tests')


def native_commands(spec):
    if spec['kind'] == 'libtest':
        return ['cargo', *spec['list']], ['cargo', *spec['run']]
    return (['cargo', 'nextest', 'list', *spec['options'], '--run-ignored', 'all', '--message-format', 'json'],
            ['cargo', 'nextest', 'run', *spec['options'], '--no-fail-fast', '--no-tests', 'fail',
             '--message-format', 'libtest-json-plus', '--message-format-version', '0.1'])


def native_selection(raw, spec, ui_listing=None):
    if spec['kind'] == 'libtest':
        ids, active = full.libtest_selection(raw.decode('utf-8'), spec['package'])
        excluded = full.libtest_original_ignores(ids, spec['package'], ui_listing) if spec['leg'] == 'ui-tu' else set()
        return ids, active - excluded, sorted(excluded), None
    document = json.loads(raw)
    active, excluded = fixed.enumerate_rust(document, root=spec['cwd'])
    ids = {binary + '\t' + test for binary, suite in document['rust-suites'].items()
           for test in suite.get('testcases', {})}
    return ids, selected_ids(active), excluded, active


def source_listing(spec, env, output):
    command, _ = native_commands(spec)
    output.mkdir(parents=True, exist_ok=False)
    result, _ = fixed.invoke(command, spec['cwd'], env, output, 'source-list', preparation=True)
    if result.returncode:
        raise ValueError('current selected owner cannot be enumerated')
    return result.stdout


def native_proof(row, spec, env, output, ui_listing=None):
    listed = read(row['listing'])
    list_cmd, run_cmd = native_commands(spec)
    if row.get('list_command') != list_cmd or row.get('run_command') != run_cmd:
        raise ValueError('owner command was filtered, changed or unexecuted')
    if Path(row.get('cwd', '')).resolve() != Path(spec['cwd']).resolve():
        raise ValueError('owner command used another checkout')
    command_proof(row['preparation'], list_cmd, spec['cwd'], preparation=True)
    command_proof(row['execution'], run_cmd, spec['cwd'])
    ids, selected, excluded, active = native_selection(listed, spec, ui_listing)
    if not selected or row.get('selected_ids') != sorted(selected) or row.get('condition_excluded') != excluded:
        raise ValueError('owner selected/excluded identities differ')
    fresh = source_listing(spec, env, output)
    fresh_ids, fresh_selected, fresh_excluded, _ = native_selection(fresh, spec, ui_listing)
    if (ids, selected, excluded) != (fresh_ids, fresh_selected, fresh_excluded):
        raise ValueError('stored owner denominator differs from actual selected source')
    if row.get('run_exit') != 0 or row.get('execution', {}).get('exit') != 0:
        raise ValueError('owner execution failed')
    if row['execution'].get('meter', {}).get('error'):
        raise ValueError('owner execution tree was not clean')
    raw = read(row['run_output'])
    if active is None:
        if libtest_passes(raw, spec['package']) != selected:
            raise ValueError('actual libtest passes differ from selected source')
    else:
        doc = json.loads(listed)
        known = {s['package-name'] + '::' + s['binary-name'] + '$' + n
                 for s in doc['rust-suites'].values() for n in s.get('testcases', {})}
        result = native_terminals(raw, active, known - active.keys())
        if result['errors']:
            raise ValueError('native terminal comparison failed: ' + str(result['errors']))
    return ids, selected


def fixed_proof(path, required_groups, env, output):
    path = Path(path).resolve()
    value = load(path)
    cfg, groups = fixed.configuration()
    expected = fixed.invocations(fixed.select(required_groups, cfg, groups))
    if (value.get('schema') != 'fixed-test-run/1' or not value.get('gate_passed')
            or value.get('config_sha256') != cfg['_sha256']
            or value.get('selected_groups') != required_groups or value.get('invocations') != expected
            or value.get('source_state') != fixed.checkout_state()
            or len(value.get('results', [])) != len(expected)):
        raise ValueError('incomplete, filtered, stale or wrong fixed language group receipt')
    passed = set()
    for index, (plan, row) in enumerate(zip(expected, value['results'])):
        if plan['runner'] != 'nextest' or row.get('outcome') != 'PASS' or row.get('groups') != plan['groups']:
            raise ValueError('required fixed group was not actually executed')
        directory = path.parent / str(index)
        prep, execution = row['preparation'], row['execution']
        args = fixed.nextest_args(plan)
        spec = {'cwd': ROOT / plan['workspace'], 'kind': 'nextest', 'options': args}
        # Fixed runner list/run already supplies the explicit expression and JSON protocol.
        list_cmd = ['cargo', 'nextest', 'list', *args, '--run-ignored', 'all', '--message-format', 'json']
        run_cmd = ['cargo', 'nextest', 'run', *args, '--no-fail-fast', '--no-tests', 'fail',
                   '--message-format', 'libtest-json-plus', '--message-format-version', '0.1']
        command_proof(prep, list_cmd, spec['cwd'], preparation=True)
        command_proof(execution, run_cmd, spec['cwd'])
        listed = local_output(directory, prep['stdout'])
        document = json.loads(listed)
        active, excluded = fixed.enumerate_rust(document)
        if row['selected'] != list(active.values()) or row.get('excluded') != excluded or not active:
            raise ValueError('fixed language original selection differs')
        fresh = source_listing(spec, env, output / ('language-' + str(index)))
        fresh_active, fresh_excluded = fixed.enumerate_rust(json.loads(fresh))
        if active != fresh_active or excluded != fresh_excluded:
            raise ValueError('fixed language source denominator was suppressed')
        known = {s['package-name'] + '::' + s['binary-name'] + '$' + n
                 for s in document['rust-suites'].values() for n in s.get('testcases', {})}
        terminals = native_terminals(local_output(directory, execution['stdout']), active, known - active.keys())
        if terminals['errors'] or row.get('executed') != len(active):
            raise ValueError('fixed language actual terminals differ')
        from execution_trace import collect, validate_bindings
        bound = row.get('bound_inputs')
        expected_bound = fixed.source_bindings(document, active, output / ('bindings-' + str(index)))
        if bound != expected_bound:
            raise ValueError('fixed language wrapper source bindings missing')
        if not validate_bindings(bound, collect(directory / 'trace', 'fixed-invocation'))['valid']:
            raise ValueError('fixed language original child comparison missing')
        passed.update(selected_ids(active))
    return passed


def environment(selected_roots):
    env = os.environ.copy()
    env.pop('CARGO_TARGET_DIR', None)
    for key, value in selected_roots.items():
        env['AUTO_' + ('LANG' if key == 'language' else key.upper()) + '_ROOT'] = str(value)
    env['NEXTEST_EXPERIMENTAL_LIBTEST_JSON'] = '1'
    env.update(full.child_env(selected_roots['language'], selected_roots['ui'], selected_roots['man']))
    env.pop('CARGO_TARGET_DIR', None)
    return env


def validate(receipt, output):
    if receipt.get('schema') != 'four-repository-fixed/1' or receipt.get('outcome') != 'pass':
        raise ValueError('not a successful fixed four-owner receipt')
    selected_roots = {k: Path(v['root']).resolve() for k, v in receipt['identities'].items()}
    if set(selected_roots) != set(OWNERS) or receipt['identities'] != receipt.get('identities_after'):
        raise ValueError('four-owner source/dependency identities incomplete or changed')
    if receipt['identities'] != {k: snapshot(p) for k, p in selected_roots.items()}:
        raise ValueError('four-owner receipt is from another current source')
    for name, path in selected_roots.items():
        require_clean(name, path)
    validate_layout(selected_roots)
    if selected_roots['language'] != ROOT.resolve():
        raise ValueError('receipt language owner differs from the validating checkout')
    if receipt.get('language_groups') != LANGUAGE_GROUPS:
        raise ValueError('required fixed language groups omitted')
    env = environment(selected_roots)
    language = receipt['language']
    read(language)
    media = fixed_proof(language['path'], LANGUAGE_GROUPS, env, output)
    specs = contracts(selected_roots)
    rows = receipt.get('results', [])
    if len(rows) != len(EXTERNAL_LEGS) or {r['leg'] for r in rows} != set(EXTERNAL_LEGS):
        raise ValueError('missing/duplicate/extra external owner leg')
    by_name = {r['leg']: r for r in rows}
    ui_listing = json.loads(read(by_name['ui-full']['listing']))
    historical = {'language|' + identity.replace('\t', '|') for identity in media}
    for name in EXTERNAL_LEGS:
        ids, selected = native_proof(by_name[name], specs[name], env, output / name, ui_listing)
        owner_selection(name, ids, selected, media)
        if name == 'web':
            historical.update('web-unit|' + identity.replace('\t', '|') for identity in ids)
            historical.update('web-http|' + identity.replace('\t', '|') for identity in ids)
            historical.update('web-generation|' + identity.replace('\t', '|') for identity in ids)
    from test_identity_migrations import validate_historical_obligations
    validate_historical_obligations({'obligations': [{'id': identity} for identity in historical]}, ROOT)
    return True


def run(output):
    selected_roots = roots()
    for key, path in selected_roots.items():
        require_clean(key, path)
    validate_layout(selected_roots)
    env = environment(selected_roots)
    before = {k: snapshot(p) for k, p in selected_roots.items()}
    receipt = {'schema': 'four-repository-fixed/1', 'language_groups': LANGUAGE_GROUPS,
               'identities': before, 'results': [], 'outcome': 'fail'}
    specs = contracts(selected_roots)
    output.mkdir(parents=True, exist_ok=False)
    try:
        with full.machine_gate():
            command = ['cargo', 't', *LANGUAGE_GROUPS, '--out', str(output / 'language')]
            result, _ = fixed.invoke(command, ROOT, env, output, 'language', timeout=14400)
            receipt['language'] = proof(output / 'language/results.json')
            if result.returncode:
                raise ValueError('fixed language groups failed')
            full.man_bootstrap(list(specs.values()), selected_roots['language'], selected_roots['ui'], output, 14400, env)
            ui_listing = None
            for name in EXTERNAL_LEGS:
                spec = specs[name]
                directory = output / name
                directory.mkdir()
                list_cmd, run_cmd = native_commands(spec)
                result, prep = fixed.invoke(list_cmd, spec['cwd'], env, directory, 'discovery', preparation=True)
                if result.returncode:
                    raise ValueError(name + ': source discovery/build failed')
                if name == 'ui-full':
                    ui_listing = json.loads(result.stdout)
                ids, selected, excluded, _ = native_selection(result.stdout, spec, ui_listing)
                owner_selection(name, ids, selected)
                if not selected:
                    raise ValueError(name + ': selected zero tests')
                actual, execution = fixed.invoke(run_cmd, spec['cwd'], env, directory, 'run', timeout=14400)
                row = {'leg': name, 'cwd': str(spec['cwd']), 'list_command': list_cmd, 'run_command': run_cmd,
                       'listing': proof(directory / prep['stdout']), 'run_output': proof(directory / execution['stdout']),
                       'selected_ids': sorted(selected), 'condition_excluded': excluded,
                       'run_exit': actual.returncode, 'preparation': prep, 'execution': execution}
                receipt['results'].append(row)
                if actual.returncode:
                    raise ValueError(name + ': execution failed')
            for key, path in selected_roots.items():
                require_clean(key, path)
            receipt['identities_after'] = {k: snapshot(p) for k, p in selected_roots.items()}
            receipt['outcome'] = 'pass'
            validate(receipt, output / 'admission')
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        receipt.update(outcome='fail', error=str(error))
        print(str(error), file=sys.stderr)
    finally:
        fixed.write(output / 'receipt.json', receipt)
    return 0 if receipt['outcome'] == 'pass' else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--receipt', type=Path, help='validate an existing current-source receipt')
    args = parser.parse_args()
    os.environ.pop('CARGO_TARGET_DIR', None)
    if args.receipt:
        args.out.mkdir(parents=True, exist_ok=False)
        validate(load(args.receipt), args.out)
        return 0
    return run(args.out.resolve())


if __name__ == '__main__':
    sys.exit(main())
