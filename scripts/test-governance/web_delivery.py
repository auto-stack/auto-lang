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


def validate_distribution_receipt(receipt, path):
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
        if 'command' in row:
            expected = 2 if row['id'] in ('failed-swap', 'missing-resource-rejected', 'bad-hash-rejected', 'bad-version-rejected') else 1 if row['id'].endswith('public-rejection') else 0
            if not row['command'] or row.get('exit') != expected or row.get('expected_exit') != expected:
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
        text = Path(tested['log']).read_text(encoding='utf-8', errors='replace')
        if not re.search(r'Summary\s+\[[^\]]+\]\s+217 tests run: 217 passed, 2 skipped', text):
            raise ValueError('independent SDK applicable/original ignored execution differs')
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
        if 'log' in row: file_identity(row['log'], row['log_sha256'])
        if 'command' in row and (not row['command'] or row.get('exit') != 0):
            raise ValueError('legacy source/build command did not actually pass')
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
    selection = load(directory / 'selection.json')
    catalog = load(selection['catalog_path'])
    if (selection.get('scope') != 'module' or selection.get('catalog_sha256') != digest(catalog)
            or len(selection['modules']) != len(set(selection['modules']))
            or len(selection['obligations']) != len(set(selection['obligations']))):
        raise ValueError('module selection identity/uniqueness differs')
    selected = set(selection['modules'])
    report = None if list_only else load(directory / 'execution/results.json')
    if report is not None and (not report.get('complete') or not report.get('gate_passed')):
        raise ValueError('incomplete/failed module execution cannot certify Web legs')
    results = {} if list_only else {r['module']: r for r in report['results']}
    if report is not None and len(results) != len(report['results']):
        raise ValueError('duplicate actual module result')
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
            if row['outcome'] == 'PASS':
                command = row.get('command', [])
                if command[:3] != ['cargo', 'nextest', 'run'] or '--no-tests' not in command:
                    raise ValueError(f'{recipe}: no actual nextest execution command')
                index = next(i for i, m in enumerate(catalog['modules']) if m['id'] == module['id'])
                log = directory / 'execution' / f'{index}-run.log'
                raw = log.read_bytes()
                counts = re.findall(r'Summary\s+\[[^\]]+\]\s+(\d+) tests? run:\s+(\d+) passed', raw.decode('utf-8', errors='replace'))
                if len(counts) != 1 or tuple(map(int, counts[0])) != (len(row['applicable']), len(row['applicable'])) or not row['applicable']:
                    raise ValueError(f'{recipe}: actual executed/passed count differs from selected applicable IDs')
                row = {**row, 'output_path': str(log), 'output_sha256': hashlib.sha256(raw).hexdigest()}
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
    return True
