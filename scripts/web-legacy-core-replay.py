#!/usr/bin/env python3
"""Replay a legal pre-Web Core install, upgrade failures and the Web-enabled Core.

The old three-repository architecture's Core profile deliberately consumes only
Man/language; the UI pin is recorded, not misrepresented as an executed UI leg.
All source snapshots come from real Git objects, with no fabricated Git history.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tarfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
GROUP = ROOT.parent
OLD_LANG = '674871e2a3bcc1a9cf594ac7446f3c1d1129ecbb'
OLD_MAN = 'e6cc06fbc496f366edde7929abfd49254ec13647'
OLD_UI = 'bce23b0f43a8dfa1d46360a47d3e98121061b56d'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--target-dir', type=Path, required=True)
    args = parser.parse_args()
    base, target = args.scratch.resolve(), args.target_dir.resolve()
    if base.exists(): raise ValueError('scratch must be new; preserve prior evidence')
    base.mkdir(parents=True)
    (base / 'logs').mkdir()
    receipt = {'schema': 'web-legacy-upgrade/1', 'outcome': 'fail', 'checks': [],
               'scope': 'old/new Core CLI install and real VM HTTP; no old Full/UI runtime claim',
               'observed_old_ui_pin': OLD_UI, 'snapshots': []}
    spec = importlib.util.spec_from_file_location('man_install', GROUP / 'auto-man/scripts/man-install.py')
    owner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owner)
    env = {k: v for k, v in os.environ.items() if not k.startswith(('AUTO_', 'CARGO_PROFILE_'))
           and k not in ('CARGO_TARGET_DIR', 'RUSTFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_ENCODED_RUSTFLAGS')}

    def command(name, argv, cwd):
        log = base / 'logs' / (name + '.log')
        with log.open('wb') as output:
            result = subprocess.run([str(a) for a in argv], cwd=cwd, env=env, stdout=output,
                                    stderr=subprocess.STDOUT, timeout=3600)
        receipt['checks'].append({'id': name, 'exit': result.returncode, 'command': list(map(str, argv)),
                                  'log': str(log), 'log_sha256': sha(log),
                                  'status': 'PASS' if result.returncode == 0 else 'FAIL'})
        if result.returncode: raise RuntimeError('command failed: ' + name)

    def archive(repo, revision, destination, paths):
        subprocess.run(['git', '-C', str(repo), 'cat-file', '-e', revision + '^{commit}'], check=True)
        archive_path = destination.with_suffix('.tar')
        command('archive-' + destination.parent.name + '-' + destination.name,
                ['git', '-C', repo, 'archive', '--format=tar', '--output', archive_path, revision, *paths], repo)
        destination.mkdir(parents=True)
        with tarfile.open(archive_path) as bundle:
            if any(not (m.isfile() or m.isdir()) for m in bundle.getmembers()):
                raise ValueError('source archive has a link or special input; not extracted')
            bundle.extractall(destination, filter='data')
        receipt['snapshots'].append({'repo': repo.name, 'revision': revision, 'path': str(destination),
                                     'archive': str(archive_path), 'archive_sha256': sha(archive_path)})

    install_root = base / 'installed'
    project = base / 'project'
    (project / 'src/back').mkdir(parents=True)
    (project / 'pac.at').write_text('name: "legacy890"\n')
    (project / 'src/back/api.at').write_text('#[api(method="GET",path="/value")]\npub fn value() str { "legacy-890" }\n')
    user_file = project / 'user.txt'
    user_file.write_text('preserved user content')
    user_hash = sha(user_file)

    def actual_service(name):
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0)); port = listener.getsockname()[1]
        config = json.dumps({'profile': 'development', 'listen': {'port': port}, 'limits': {'drain_timeout_ms': 100}})
        binary = install_root / owner.exe('auto')
        log = base / 'logs' / (name + '.log')
        with log.open('wb') as output:
            child = subprocess.Popen([str(binary), 'service', '--server', 'vm', '--dir', str(project),
                                      '--http-config-inline', config], cwd=project, stdout=output, stderr=subprocess.STDOUT,
                env={**env, 'AUTO_LANG_ROOT': str(install_root / 'sdk'), 'AUTO_WEB_ROOT': str(install_root / 'sdk'),
                     'AUTO_STDLIB_ROOT': str(install_root / 'sdk/stdlib/auto'), 'AUTO_EXECUTION_ENGINE': 'autovm'})
            try:
                deadline = time.monotonic() + 30
                while True:
                    if child.poll() is not None: raise RuntimeError(name + ': CLI exited before ready')
                    try:
                        with urllib.request.urlopen(f'http://127.0.0.1:{port}/value', timeout=3) as response:
                            assert response.status == 200 and response.read() == b'"legacy-890"'
                        break
                    except OSError:
                        if time.monotonic() > deadline: raise RuntimeError(name + ': readiness timed out')
                        time.sleep(.05)
                with urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{port}/__auto/shutdown', method='POST'), timeout=3) as response:
                    assert response.status == 202
                assert child.wait(timeout=10) == 0
            finally:
                if child.poll() is None: child.kill(); child.wait(timeout=10)
        with socket.socket() as listener: listener.bind(('127.0.0.1', port))
        assert sha(user_file) == user_hash
        receipt['checks'].append({'id': name, 'status': 'PASS', 'exact_json': 'legacy-890',
                                  'shutdown': 202, 'exit': 0, 'port_released': True,
                                  'binary_sha256': sha(binary), 'log': str(log), 'log_sha256': sha(log)})

    try:
        heads = {name: subprocess.check_output(['git', '-C', str(GROUP / name), 'rev-parse', 'HEAD'], text=True).strip()
                 for name in ('auto-lang', 'auto-man', 'auto-web')}
        subprocess.run(['git', '-C', str(GROUP / 'auto-ui'), 'cat-file', '-e', OLD_UI + '^{commit}'], check=True)
        for label, pins in [('old', {'auto-lang': OLD_LANG, 'auto-man': OLD_MAN}), ('new', heads)]:
            cohort = base / label
            cohort.mkdir()
            for name, revision in pins.items():
                paths = ['Cargo.toml', 'Cargo.lock', 'LICENSE', 'crates']
                if name == 'auto-lang': paths += ['stdlib', 'vendor', 'docs/schema']
                elif name == 'auto-man': paths += ['core']
                archive(GROUP / name, revision, cohort / name, paths)
            owner.MAN, owner.LANG = cohort / 'auto-man', cohort / 'auto-lang'
            owner.WEB, owner.UI, owner.DOWN = cohort / 'auto-web', cohort / 'auto-ui', cohort / 'auto-down'
            owner.head = lambda path, pins=pins, cohort=cohort: pins[path.resolve().relative_to(cohort.resolve()).parts[0]]
            command(label + '-core-locked-build', ['cargo', 'build', '--locked', '--manifest-path', owner.MAN / 'core/Cargo.toml',
                    '-p', 'auto', '--no-default-features', '--features', 'core', '--target-dir', target,
                    '--config', 'profile.dev.debug="line-tables-only"', '-j', '4'], owner.MAN)
            binary_directory = owner.MAN / 'core/target/debug'
            binary_directory.mkdir(parents=True, exist_ok=True)
            shutil.copy2(target / 'debug' / owner.exe('auto'), binary_directory / owner.exe('auto'))
            if label == 'new':
                before = owner.manifest_files(install_root)
                for failure in ('prepare', 'verify', 'swap'):
                    try: owner.install('core', install_root, failure, True)
                    except RuntimeError as error:
                        if str(error) != 'injected ' + failure + ' failure': raise
                    else: raise AssertionError('injected failure unexpectedly passed')
                    assert owner.manifest_files(install_root) == before
                    owner.verify(install_root, 'core')
                    actual_service('after-' + failure + '-failure-old-runtime')
            installed = owner.install('core', install_root, None, True)
            expected_owners = {'auto-lang', 'auto-man'} | ({'auto-web'} if label == 'new' else set())
            assert set(installed['sources']) == expected_owners
            assert not (install_root / 'sdk/crates/auto-lang').exists()
            assert not any((install_root / 'sdk/crates').glob('auto-ui*'))
            receipt['checks'].append({'id': label + '-core-install', 'status': 'PASS', 'sources': installed['sources'],
                                      'manifest_sha256': sha(install_root / 'install-manifest.json')})
            actual_service(label + '-installed-runtime')
        receipt['outcome'] = 'pass'
    except Exception as error:
        receipt['error'] = str(error)
        print(str(error), file=sys.stderr)
    finally:
        receipt['summary'] = {'executed': len(receipt['checks']), 'passed': sum(c['status'] == 'PASS' for c in receipt['checks']),
                              'failed': sum(c['status'] == 'FAIL' for c in receipt['checks'])}
        (base / 'legacy-receipt.json').write_text(json.dumps(receipt, indent=2, ensure_ascii=False) + '\n')
        print(json.dumps(receipt['summary']) + ' ' + receipt['outcome'])
    return 0 if receipt['outcome'] == 'pass' else 2


if __name__ == '__main__':
    sys.exit(main())
