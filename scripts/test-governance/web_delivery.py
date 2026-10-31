"""Independent Web distribution and real Core recovery receipt admission."""
import hashlib
import json
import re
from pathlib import Path
from governance_io import load, digest

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
    if (len(package_names) != len(set(package_names)) or len(package_names) != 20
            or not {'auto-web', 'auto-web-contract', 'auto-lang-core', 'auto-generation-core', 'a2r-std', 'auto-media-core', 'auto-tool-execution'}.issubset(package_names)
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
    # The immutable transaction artifact's revision identifies that script,
    # not the independently built current Man Core CLI and its dependency lock.
    consumers = receipt.get('consumer_sources', {})
    if set(consumers) != {'auto-man'} or not re.fullmatch('[0-9a-f]{40}', consumers['auto-man']):
        raise ValueError('current Core CLI consumer source missing')
    new_sources = {**receipt['sources'], **consumers}
    if source_roots is not None:
        if 'auto-man' not in source_roots:
            raise ValueError('current Core CLI consumer owner missing')
        root = source_roots['auto-man']
        scopes = ['Cargo.toml', 'Cargo.lock', 'core', 'crates', 'scripts', 'LICENSE']
        changed = subprocess.check_output(['git', '-C', str(root), 'diff', '--name-only', consumers['auto-man'], '--', *scopes])
        added = subprocess.check_output(['git', '-C', str(root), 'ls-files', '--others', '--exclude-standard', '--', *scopes])
        if changed.strip() or added.strip():
            raise ValueError('current Core CLI consumer inputs differ')
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
