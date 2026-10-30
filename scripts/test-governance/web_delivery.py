"""Four-owner receipt checks over the existing, actually executed module union."""
import hashlib
import json
import re
from pathlib import Path
from module_catalog import load, digest
from test_identity_migrations import records, key, media_records

WEB_LEGS = ("web-unit", "web-http", "web-generation")
MODULE_LEGS = (*WEB_LEGS, 'media-unit')
REQUIRED_OWNERS = ("language", "web", "ui", "man")
REQUIRED_LEGS = ('language', 'ui-full', 'man-t', 'man-tcore', 'man-tc', 'man-ti', 'ui-tu', 'ui-ti', *MODULE_LEGS)

DISTRIBUTION_IDS = {
    'independent-sdk-tests', 'fresh-install', 'independent-installed-closure',
    'external-fresh-lock', 'external-locked-build', 'fresh-vm-runtime',
    'interrupted-generation-recovered', 'relocated-vm-runtime',
    'wrong-producer-version-public-rejection', 'wrong-producer-version-old-runtime',
    'failed-swap', 'successful-upgrade', 'legacy-three-owner-recovery',
    *(prefix + '-' + step for prefix in ('fresh', 'after-interruption', 'relocated', 'after-failed-swap', 'after-upgrade')
      for step in ('public-generation', 'generated-build', 'generated-locked-replay', 'generated-runtime')),
    *(mode + '-' + step for mode in ('missing-producer-version', 'managed-source-hash', 'missing-managed-receipt')
      for step in ('public-rejection', 'old-runtime')),
    *(mode + '-' + step for mode in ('missing-resource', 'bad-hash', 'bad-version')
      for step in ('rejected', 'old-runtime')),
}


def file_proof(path):
    path = Path(path).resolve()
    return {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def read_proof(proof):
    if not isinstance(proof, dict) or not proof.get('path') or not re.fullmatch('[0-9a-f]{64}', proof.get('sha256', '')):
        raise ValueError('missing execution output path/hash')
    raw = Path(proof['path']).read_bytes()
    if hashlib.sha256(raw).hexdigest() != proof['sha256']:
        raise ValueError('execution output identity changed: ' + proof['path'])
    return raw


def execution_ids(raw, package=None):
    text = raw.decode('utf-8', errors='replace')
    if package:
        names = re.findall(r'^test\s+(\S+)\s+\.\.\.\s+ok\s*$', text, re.MULTILINE)
        ids = [package + '\t' + name for name in names]
        summaries = re.findall(r'test result: ok\.\s+(\d+) passed;\s+0 failed;', text)
        counts = [int(n) for n in summaries]
    else:
        names = re.findall(r'^\s*PASS(?:\s*\+\s*LEAK)?\s+\[[^\]]*\](?:\s+\(\s*\d+/\d+\))?\s+(\S+)\s+(\S+)\s*$', text, re.MULTILINE)
        ids = [binary + '\t' + test for binary, test in names]
        summaries = re.findall(r'Summary\s+\[[^\]]+\]\s+(\d+) tests? run:\s+(\d+) passed', text)
        counts = [int(a) for a, b in summaries if a == b]
    if len(ids) != len(set(ids)) or counts != [len(ids)] or not ids:
        raise ValueError('actual test IDs/summary missing, duplicated or unsuccessful')
    return set(ids)


def listed_ids(raw, package=None):
    if package:
        text = raw.decode('utf-8', errors='replace')
        return {package + '\t' + line[:-6].strip() for line in text.splitlines() if line.endswith(': test')}
    document = json.loads(raw)
    return {binary + '\t' + name for binary, suite in document['rust-suites'].items()
            for name, row in suite['testcases'].items()
            if row['filter-match']['status'] == 'matches' and not row['ignored']}


def module_execution(directory):
    """Reconcile stored execution with the selected source-bound atomic IDs."""
    from module_catalog import verify_selection, source_identity, ROOT
    directory = Path(directory)
    selection_path = directory / 'selection.json'
    selection = load(selection_path)
    catalog = load(selection['catalog_path'])
    if digest(catalog) != digest(load(ROOT/'scripts/test-governance/modules.json')):
        # The public tm runner deliberately refreshes into an ephemeral catalog.
        # Recheck its source contracts rather than requiring published bytes.
        from catalog_contract import verify_catalog
        verify_catalog(catalog, ROOT)
    verify_selection(catalog, selection, source_identity(ROOT))
    report = load(directory / 'execution/results.json')
    if (not report.get('complete') or not report.get('gate_passed')
            or report.get('selection_sha256') != file_proof(selection_path)['sha256']
            or report.get('catalog_sha256') != digest(catalog)
            or report.get('source_identity') != selection['source_identity']):
        raise ValueError('incomplete, failed or stale module execution')
    results = {r['module']: r for r in report['results']}
    if len(results) != len(report['results']) or set(results) != set(selection['modules']):
        raise ValueError('missing, extra or duplicate actual module result')
    for module in catalog['modules']:
        if module['id'] in results:
            validate_module_result(catalog, module, results[module['id']])
    return selection, catalog, results


def validate_module_result(catalog, module, row):
    """One leaf's actual producer/atomic proof; caller owns full selection gates."""
    from module_catalog import discover, ROOT
    from run_selected import applicability, exact_filter, execution_settings, validate_python_result, validate_case_result
    from repository_layout import workspace_root
    import os
    lookup = {r['id']: r for r in catalog['obligations']}
    recipe = catalog['recipes'][module['recipe']]
    rows = [lookup[i] for i in module['obligations']]
    if row.get('obligations') != sorted(module['obligations']):
        raise ValueError('actual module obligation identity differs')
    keys = set(recipe.get('required_env', {}))
    for obligation in rows:
        for condition in obligation.get('conditions', []):
            keys.update(condition.get('required_env', {}))
    env = {**os.environ, **recipe.get('env', {})}
    condition_environment = {key: env.get(key) for key in sorted(keys)}
    if row.get('condition_environment') != condition_environment:
        raise ValueError('execution condition environment differs from frozen invocation')
    excluded = applicability(rows, condition_environment, recipe.get('required_env'))
    expected = sorted(set(module['obligations']) - {r['id'] for r in excluded})
    if row.get('applicable') != expected:
        raise ValueError('actual applicable IDs omit/duplicate an unconditional obligation')
    if excluded:
        if row.get('outcome') != 'NOT_APPLICABLE' or row.get('condition_excluded') != excluded or row.get('exit') is not None:
            raise ValueError('condition exclusion differs from original source conditions')
        return
    if row.get('outcome') != 'PASS' or row.get('exit') != 0 or row.get('condition_excluded') != []:
        raise ValueError('required module was not successfully executed')
    from execution_trace import validate_bindings
    bindings = validate_bindings(rows, row.get('child_observations', {'invocations': []}))
    if row.get('child_validation') != bindings or not bindings['valid']:
        raise ValueError('required original child observations are incomplete')
    if Path(row.get('cwd', '')).resolve() != workspace_root(ROOT, recipe):
        raise ValueError('module execution cwd differs from declared owner')
    output = read_proof(row.get('output'))
    listing = read_proof(row.get('listing'))
    listed = discover(json.loads(listing), module['recipe'], allow_filtered_skipped=True)
    runner = recipe.get('runner', 'nextest')
    if runner in ('python-unittest', 'case-suite'):
        listed = [r for r in listed if r['id'] in set(expected)]
    if sorted(r['id'] for r in listed if r['selected_at_discovery'] and not r['ignored']) != expected:
        raise ValueError('actual discovery omitted or changed module IDs')
    fields = ('suite_id', 'case_id', 'check_id', 'adapter', 'case_recipe', 'profile',
              'source_sha256', 'oracle_sha256', 'case_file_sha256', 'case_file',
              'product_from', 'state') if runner == 'case-suite' else ('package_path', 'package')
    for discovered in listed:
        if discovered['id'] in lookup and any(discovered.get(key) != lookup[discovered['id']].get(key) for key in fields):
            raise ValueError('actual discovered source/profile identity differs from its obligation')
    if runner == 'nextest':
        switches = [*execution_settings(recipe, module)[0], '--ignore-default-filter', '-E', exact_filter(rows)]
        serial = ['--test-threads', '1'] if 'th' in module.get('legacy_tiers', []) or any(r['test'].startswith('vm::ffi::stdlib::http::') for r in rows) else []
        if (row.get('list_command') != ['cargo', 'nextest', 'list', *switches, '--message-format', 'json']
                or row.get('command') != ['cargo', 'nextest', 'run', *switches, *serial, '--no-fail-fast', '--no-tests', 'fail']):
            raise ValueError('module list/run command differs from complete exact selection')
        if execution_ids(output) != {i.split('|', 1)[1].replace('|', '\t', 1) for i in expected}:
            raise ValueError('actual passed test IDs differ from required complete module')
    elif runner == 'python-unittest':
        from recipe_runtime import list_command, python_run_command
        if (row.get('list_command') != list_command(recipe, ROOT)
                or row.get('command') != python_run_command(recipe, ROOT, [r for r in listed if not r['ignored']])):
            raise ValueError('Python list/run command differs from complete selection')
        proof = validate_python_result(Path(row['output']['path']), [r['test'] for r in listed if not r['ignored']])
        if proof != row.get('execution') or proof['failures'] or proof['errors']:
            raise ValueError('Python atomic execution failed or differs')
    elif runner == 'case-suite':
        from recipe_runtime import list_command
        command = list_command(recipe, ROOT)
        expected_command = [command[0], 'module-case', *command[2:], '--case', rows[0]['case_id'],
                            '--out', row['case_artifacts']['original_runtime_root'], '--artifact-out', str(Path(row['output']['path']).resolve().parent)]
        if row.get('list_command') != command or row.get('command') != expected_command:
            raise ValueError('case list/run command differs from original complete case')
        # The raw case report is retained independently of its summary.
        read_proof(row.get('case_report'))
        proof = validate_case_result(Path(row['case_report']['path']), rows, row['execution']['raw_exit'])
        if proof != row['execution'] or not proof['assertions_satisfied']:
            raise ValueError('case execution failed its original atomic oracles')
    else:
        raise ValueError('unknown module execution kind')


def validate_distribution_receipt(receipt, path, source_roots=None):
    """A banner alone cannot certify fresh/locked/relocation/recovery delivery."""
    path = Path(path).resolve()
    if (receipt.get('schema') != 'web-repository-distribution/1' or receipt.get('outcome') != 'pass'
            or receipt.get('candidate') is not False or receipt.get('pending') or receipt.get('error')):
        raise ValueError('partial/failed/unfrozen Web distribution receipt')
    rows = receipt.get('checks', [])
    ids = [r['id'] for r in rows]
    if len(ids) != len(set(ids)) or not DISTRIBUTION_IDS.issubset(ids):
        raise ValueError('missing/duplicate required distribution execution')
    if receipt.get('summary') != {'executed': len(rows), 'passed': len(rows), 'failed': 0, 'pending': 0}:
        raise ValueError('distribution executed/passed totals differ')
    def file_identity(file, expected):
        file = Path(file)
        if not re.fullmatch('[0-9a-f]{64}', expected or '') or hashlib.sha256(file.read_bytes()).hexdigest() != expected:
            raise ValueError('distribution output identity differs: ' + str(file))
        return file
    for row in rows:
        if row.get('status') != 'PASS': raise ValueError('required distribution check failed/unexecuted')
        detail_only = (row['id'].endswith('runtime') or row['id'] in
                       ('independent-installed-closure', 'interrupted-generation-recovered', 'legacy-three-owner-recovery'))
        reuse = row['id'] == 'independent-sdk-tests' and 'reused_receipt' in row.get('detail', {})
        if not detail_only and not reuse:
            expected = 2 if row['id'] in ('failed-swap', 'missing-resource-rejected', 'bad-hash-rejected', 'bad-version-rejected') else 1 if row['id'].endswith('public-rejection') else 0
            if (not isinstance(row.get('command'), list) or not row['command']
                    or not all(isinstance(v, str) and v for v in row['command'])
                    or row.get('exit') != expected or row.get('expected_exit') != expected
                    or not row.get('log')):
                raise ValueError('distribution command result differs from original oracle')
            file_identity(row['log'], row.get('log_sha256'))
        detail = row.get('detail', {})
        if row['id'].endswith(('runtime',)):
            if (detail.get('exact_json') != 890 or detail.get('shutdown') != 202 or detail.get('exit') != 0
                    or detail.get('port_released') is not True or not re.fullmatch('[0-9a-f]{64}', detail.get('binary_sha256', ''))):
                raise ValueError('distribution runtime did not prove exact response/shutdown/release')
            file_identity(path.parent / 'logs' / (row['id'] + '.log'), detail.get('log_sha256'))
    by_id = {r['id']: r for r in rows}
    install_command = by_id['fresh-install']['command']
    source = Path(install_command[install_command.index('--source') + 1])
    manifest = json.loads((source / 'source-manifest.json').read_text(encoding='utf-8'))
    file_map_hash = hashlib.sha256(json.dumps(manifest['files'], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    if (manifest.get('schema') != 'web-source/1' or manifest.get('candidate') is not False
            or manifest.get('sources') != receipt.get('sources')
            or manifest.get('source_sha256') != receipt.get('source_sha256') or file_map_hash != receipt.get('source_sha256')):
        raise ValueError('distribution source cohort differs')
    if source_roots is not None:
        import subprocess
        scopes = {'auto-lang': ['Cargo.toml', 'stdlib/auto', 'stdlib/assembly-providers.json', 'docs/schema/aura.at',
                               *manifest['source_closure']['root_inputs'], 'vendor/autodown-test-parser',
                               *(f'examples/ui/{name}/{part}' for name in ('015-notes', '017-chat', '023-realworld') for part in ('src/back', 'pac.at')), 'LICENSE'],
                  'auto-web': ['Cargo.toml', 'Cargo.lock', '.config', 'scripts',
                               'test/fixtures/standalone-consumer', 'README.md', 'LICENSE']}
        for package in manifest['source_closure']['packages']:
            if package['source_repo'] not in scopes or package['source_path'] != 'crates/' + package['name']:
                raise ValueError('distribution package source owner/path differs')
            scopes[package['source_repo']].append(package['source_path'])
        for name, paths in scopes.items():
            root = source_roots[name]
            changed = subprocess.check_output(['git', '-C', str(root), 'diff', '--name-only',
                                               manifest['sources'][name], '--', *paths])
            added = subprocess.check_output(['git', '-C', str(root), 'ls-files', '--others',
                                             '--exclude-standard', '--', *paths])
            if changed.strip() or added.strip():
                raise ValueError('distribution payload inputs differ from the selected current owner: ' + name)
    package_names = [p['name'] for p in manifest.get('source_closure', {}).get('packages', [])]
    if (len(package_names) != len(set(package_names)) or len(package_names) != 19
            or not {'auto-web', 'auto-web-contract', 'auto-lang-core', 'auto-generation-core', 'a2r-std', 'auto-media-core'}.issubset(package_names)
            or by_id['independent-installed-closure'].get('detail', {}).get('local_packages') != len(package_names)):
        raise ValueError('independent installed source closure missing/duplicated/different')
    for name, record in manifest['files'].items():
        selected = (source / name).resolve()
        if not selected.is_relative_to(source.resolve()): raise ValueError('distribution source escapes payload')
        file_identity(selected, record['sha256'])
        if selected.stat().st_size != record['bytes']: raise ValueError('distribution source size differs')
    tested = by_id['independent-sdk-tests']
    if 'command' in tested:
        test_log = Path(tested['log'])
    else:
        reused = tested.get('detail', {})
        previous = file_identity(reused['reused_receipt'], reused['sha256'])
        previous = json.loads(previous.read_text(encoding='utf-8'))
        sdk_files = {name.removeprefix('sdk/'): value for name, value in manifest['files'].items() if name.startswith('sdk/')}
        sdk_hash = hashlib.sha256(json.dumps(sdk_files, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        if (previous.get('schema') != 'web-independent-tests/1' or previous.get('outcome') != 'pass'
                or previous.get('passed') != 217 or previous.get('original_skipped') != 2
                or previous.get('sdk_inputs_sha256') != sdk_hash
                or previous.get('config_sha256') != manifest['files']['sdk/.config/nextest.toml']['sha256']):
            raise ValueError('reused SDK execution source/config identity differs')
        file_identity(previous['log'], previous['log_sha256'])
        test_log = Path(previous['log'])
    raw = test_log.read_bytes()
    if (not re.search(r'Summary\s+\[[^\]]+\]\s+217 tests run: 217 passed, 2 skipped', raw.decode('utf-8', errors='replace'))
            or len(execution_ids(raw)) != 217):
        raise ValueError('independent SDK actual test IDs/original ignored execution differs')
    legacy = by_id['legacy-three-owner-recovery']['detail']
    legacy_path = file_identity(legacy['receipt'], legacy['sha256'])
    legacy = json.loads(legacy_path.read_text(encoding='utf-8'))
    legacy_ids = {r['id'] for r in legacy.get('checks', [])}
    required_legacy = {'old-core-locked-build', 'new-core-locked-build', 'old-core-install', 'new-core-install',
        'old-installed-runtime', 'new-installed-runtime', *(f'after-{phase}-failure-old-runtime' for phase in ('prepare', 'verify', 'swap'))}
    if (legacy.get('schema') != 'web-legacy-upgrade/1' or legacy.get('outcome') != 'pass'
            or not required_legacy.issubset(legacy_ids) or len(legacy_ids) != len(legacy['checks'])
            or any(r.get('status') != 'PASS' for r in legacy['checks'])):
        raise ValueError('old/new Core upgrade or actual recovery consumption missing')
    for row in legacy['checks']:
        command_required = row['id'].startswith('archive-') or row['id'].endswith('-core-locked-build')
        if command_required and (not isinstance(row.get('command'), list) or not row['command']
                                 or row.get('exit') != 0 or not row.get('log')):
            raise ValueError('legacy source/build command proof missing or unsuccessful')
        if command_required or row['id'].endswith('runtime'):
            if not row.get('log'): raise ValueError('legacy execution output missing')
            file_identity(row['log'], row.get('log_sha256'))
        if row['id'].endswith('-core-install') and not re.fullmatch('[0-9a-f]{64}', row.get('manifest_sha256', '')):
            raise ValueError('legacy install identity missing')
        if row['id'].endswith('runtime') and (row.get('exact_json') != 'legacy-890' or row.get('shutdown') != 202
                or row.get('exit') != 0 or row.get('port_released') is not True
                or not re.fullmatch('[0-9a-f]{64}', row.get('binary_sha256', ''))):
            raise ValueError('legacy recovery did not prove real response/shutdown/release')
    legacy_by_id = {r['id']: r for r in legacy['checks']}
    old_sources = {'auto-lang': '674871e2a3bcc1a9cf594ac7446f3c1d1129ecbb',
                   'auto-man': 'e6cc06fbc496f366edde7929abfd49254ec13647'}
    new_sources = {**receipt['sources'], 'auto-man': manifest['tool_producer']['source_commit']}
    if legacy_by_id['old-core-install'].get('sources') != old_sources or legacy_by_id['new-core-install'].get('sources') != new_sources:
        raise ValueError('legacy/new Core producer cohort differs')
    snapshots = legacy.get('snapshots', [])
    expected_snapshots = {(repo, revision) for cohort in (old_sources, new_sources) for repo, revision in cohort.items()}
    if {(s['repo'], s['revision']) for s in snapshots} != expected_snapshots or len(snapshots) != 5:
        raise ValueError('legacy source/history restoration identities missing or duplicate')
    for snapshot in snapshots:
        file_identity(snapshot['archive'], snapshot['archive_sha256'])
        label = Path(snapshot['path']).parent.name
        archive_id = 'archive-' + label + '-' + snapshot['repo']
        archive = legacy_by_id.get(archive_id, {})
        if (label not in ('old', 'new') or archive.get('exit') != 0 or not archive.get('command')
                or snapshot['revision'] not in archive['command'] or snapshot['archive'] not in archive['command']):
            raise ValueError('legacy archive lacks its actual revision-bound producer command')
    return True


def collect_web_legs(directory, list_only=False):
    directory = Path(directory)
    if list_only:
        selection = load(directory / 'selection.json')
        catalog = load(selection['catalog_path'])
        results = {}
    else:
        selection, catalog, results = module_execution(directory)
    selected = set(selection['modules'])
    legs = []
    for recipe in MODULE_LEGS:
        modules = [m for m in catalog['modules'] if m['recipe'] == recipe]
        if not modules or any(m['id'] not in selected for m in modules):
            raise ValueError(f'{recipe}: missing required complete modules')
        wanted = {i for m in modules for i in m['obligations']}
        if not wanted or not wanted.issubset(set(selection['obligations'])):
            raise ValueError(f'{recipe}: zero or missing selected obligations')
        execution = []
        for module in modules:
            if list_only:
                continue
            row = results.get(module['id'])
            if row is None or set(row['obligations']) != set(module['obligations']):
                raise ValueError(f'{recipe}: missing/changed actual module execution')
            if row['outcome'] not in ('PASS', 'NOT_APPLICABLE') or (row['outcome'] == 'PASS' and row['exit'] != 0):
                raise ValueError(f'{recipe}: unsuccessful required execution')
            if row['outcome'] == 'NOT_APPLICABLE' and not module['conditions']:
                raise ValueError(f'{recipe}: unconditional module was not executed')
            execution.append(row)
        if not list_only and not any(r['outcome'] == 'PASS' for r in execution):
            raise ValueError(f'{recipe}: required leg never executed')
        executed_ids = {i for row in execution if row['outcome'] == 'PASS' for i in row['applicable']}
        legs.append({'leg': recipe, 'owner': 'language' if recipe == 'media-unit' else 'web', 'kind': 'complete-module-union',
                     'ids': sorted(wanted), 'declared': len(wanted), 'selected_ids': sorted(executed_ids),
                     'selected': len(wanted) if list_only else len(executed_ids), 'execution': execution,
                     'list_only': list_only, 'run_exit': None if list_only else 0,
                     'selection_path': str(directory / 'selection.json'),
                     'selection_sha256': hashlib.sha256((directory / 'selection.json').read_bytes()).hexdigest()})
    all_ids = set(selection['obligations'])
    required = {key(r['new']) for r in records() if r['new']['recipe'] == 'web-unit'}
    required.update('media-unit|' + r['new'].replace('\t', '|') for r in media_records())
    if not required.issubset(all_ids):
        raise ValueError('migrated original test missing from four-owner selection')
    return legs


def validate_four_receipt(receipt):
    if receipt.get('mode') != 'four-repository' or receipt.get('required_owners') != list(REQUIRED_OWNERS):
        raise ValueError('not a four-repository receipt')
    if receipt.get('list_only') or receipt.get('filters') or receipt.get('outcome') != 'pass':
        raise ValueError('partial/failed receipt cannot certify four-repository delivery')
    before, after = receipt.get('identities', {}), receipt.get('identities_after', {})
    if before != after or not set((*REQUIRED_OWNERS, 'down')).issubset(before):
        raise ValueError('required owner/dependency identity missing or changed')
    for owner in (*REQUIRED_OWNERS, 'down'):
        value = before[owner]
        for name in ('root', 'head', 'source_sha256', 'resource_manifest_sha256', 'lock_sha256', 'runtime_producer'):
            if not value.get(name):
                raise ValueError(f'{owner}: missing {name} identity')
    if (set(receipt.get('required_legs', [])) != set(REQUIRED_LEGS)
            or len(receipt.get('required_legs', [])) != len(REQUIRED_LEGS)):
        raise ValueError('required leg contract absent')
    results = receipt.get('results', [])
    names = [r['leg'] for r in results]
    if len(names) != len(set(names)) or not set(receipt['required_legs']).issubset(names):
        raise ValueError('duplicate/missing required legs')
    for name in receipt['required_legs']:
        row = next(r for r in results if r['leg'] == name)
        if row.get('run_exit') != 0 or not row.get('selected'):
            raise ValueError(f'{name}: required leg unexecuted or selected zero tests')
    # Consume the same owner recipes as the producer, not self-declared options.
    import importlib.util
    spec = importlib.util.spec_from_file_location('four_owner_recipe_contract', Path(__file__).parents[1]/'run-full-regression.py')
    owner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owner)
    if receipt.get('tier') != 'tf' or receipt.get('batch') is not True:
        raise ValueError('complete owner execution recipe missing')
    roots = {name: Path(before[name]['root']).resolve() for name in REQUIRED_OWNERS}
    contracts = {name: {'cwd': root, 'kind': 'nextest', 'options': options}
                 for name, root, options in owner.legs(receipt['tier'], roots['language'], roots['ui'], True)}
    contracts.update({s['leg']: s for s in owner.batch_extra_legs(roots['language'], roots['ui'], roots['man'])})
    by_name = {r['leg']: r for r in results}
    owner_selections = {}
    executed_migrations = set()
    minimums = owner.tier_minimums()
    for name in REQUIRED_LEGS:
        if name in MODULE_LEGS:
            continue
        row, contract = by_name[name], contracts[name]
        package = contract.get('package') if contract['kind'] == 'libtest' else None
        options = contract.get('options', [])
        list_command = ['cargo', *contract['list']] if package else ['cargo', 'nextest', 'list', *options, '--message-format', 'json']
        if (Path(row.get('cwd', '')).resolve() != Path(contract['cwd']).resolve()
                or row.get('list_exit') != 0 or row.get('list_command') != list_command):
            raise ValueError(f'{name}: actual owner/list command missing or changed')
        listing = read_proof(row.get('listing'))
        selected = listed_ids(listing, package)
        ids, source_selected = (owner.libtest_selection(listing.decode('utf-8'), package) if package
                                else owner.selection(json.loads(listing)))
        if name == 'ui-tu':
            excluded = owner.libtest_original_ignores(ids, package,
                            json.loads(read_proof(by_name['ui-full'].get('listing'))))
            if row.get('condition_excluded', []) != sorted(excluded):
                raise ValueError('ui-tu: original ignored identities differ')
            selected -= excluded
            source_selected -= excluded
        if not selected or row.get('selected_ids') != sorted(selected) or row['selected'] != len(selected):
            raise ValueError(f'{name}: exact selected test identities differ')
        if source_selected != selected:
            raise ValueError(f'{name}: owner selection parsing differs')
        owner.validate_owner_selection('tf', name, ids, selected, minimums, with_web=True)
        owner_selections[name] = (ids, selected)
    # Reject a lost source denominator before expensive execution proof checks.
    for name, (ids, selected) in owner_selections.items():
        row, contract = by_name[name], contracts[name]
        package = contract.get('package') if contract['kind'] == 'libtest' else None
        options = contract.get('options', [])
        if name == 'language':
            directory = Path(row.get('module_directory', ''))
            for key, relative in [('module_selection', 'selection.json'), ('module_results', 'execution/results.json')]:
                read_proof(row.get(key))
                if Path(row[key]['path']).resolve() != (directory/relative).resolve():
                    raise ValueError('language module proof points to a different union')
            selection, catalog, execution = module_execution(directory)
            expected_command = owner.module_queue_command(roots['language'], 'tf', [], True, directory, False, True)
            if row.get('module_command') != expected_command or row.get('module_exit') != 0:
                raise ValueError('language complete module invocation missing or changed')
            from module_command import schedule_seed_modules
            scheduled = schedule_seed_modules(catalog, roots['language']/'scripts/test-governance/periodic-four-repository-schedule.json')
            if set(selection['modules']) != set(scheduled):
                raise ValueError('four-owner module union omitted a required periodic recipe/leaf')
            covered = {i.split('|', 1)[1].replace('|', '\t', 1) for i in selection['obligations'] if i.startswith('language|')}
            if not selected.issubset(covered):
                raise ValueError('legacy language selection absent from executed/condition-excluded module union')
            actual = collect_web_legs(directory)
            for leg in actual:
                if by_name[leg['leg']] != leg:
                    raise ValueError('Web/media leg differs from actual complete module execution')
                executed_migrations.update(leg['selected_ids'])
        else:
            expected = ['cargo', *contract['run']] if package else ['cargo', 'nextest', 'run', *options, '--no-fail-fast']
            if row.get('run_command') != expected:
                raise ValueError(f'{name}: actual run command missing, filtered or changed')
            if execution_ids(read_proof(row.get('run_output')), package) != selected:
                raise ValueError(f'{name}: actual passed tests differ from complete selection')
    for name, (ids, selected) in owner_selections.items():
        owner.validate_owner_selection('tf', name, ids, selected, minimums, with_web=True,
                                       executed_migrations=executed_migrations)
    return True
