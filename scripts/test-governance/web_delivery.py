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
