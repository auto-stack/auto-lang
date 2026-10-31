#!/usr/bin/env python3
"""Preview/reclaim only explicitly owned idle Cargo caches in this checkout."""
import argparse
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def reparse(path):
    info = path.lstat()
    return path.is_symlink() or bool(getattr(info, 'st_file_attributes', 0) & 0x400)


def inspect(path, retired_source=None):
    lexical = Path(path).absolute()
    for parent in [lexical, *lexical.parents]:
        if os.path.lexists(parent) and reparse(parent):
            raise ValueError('reparse point/ancestor refused: ' + str(parent))
    target = lexical.resolve(strict=True)
    allowed = target in {ROOT / 'target', ROOT / 'target/test-tools/regular', ROOT / 'target/test-tools/batch', ROOT / 'experimental/ace/target-batch'} or (
        target.is_relative_to(ROOT / 'experimental') and target.name == 'target'
        and (target.parent / 'Cargo.toml').is_file())
    if not allowed and retired_source and target.is_relative_to(ROOT / 'experimental') and target.name == 'target':
        # A migrated manifest is no longer present at its old cache root.
        # Prove this exact checkout owned that Cargo workspace in an ancestor;
        # this is cleanup provenance, never a test-selection catalog.
        revision = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', '--verify', retired_source + '^{commit}'], text=True).strip()
        subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor', revision, 'HEAD'], check=True, capture_output=True)
        relative = (target.parent / 'Cargo.toml').relative_to(ROOT).as_posix()
        manifest = subprocess.check_output(['git', '-C', str(ROOT), 'show', revision + ':' + relative], text=True, stderr=subprocess.PIPE)
        import tomllib
        historical = tomllib.loads(manifest)
        allowed = 'package' in historical or 'workspace' in historical
    if not allowed:
        raise ValueError('not an exact Cargo cache owned by this checkout: ' + str(target))
    if (ROOT / 'target/test-groups/active').exists():
        raise ValueError('fixed-test execution/measurement lease is active')
    files = []
    for folder, dirs, names in os.walk(target, followlinks=False):
        for name in [*dirs, *names]:
            child = Path(folder) / name
            if reparse(child):
                raise ValueError('reparse point inside cache: ' + str(child))
            if child.is_file():
                if child.name in ('results.json', 'report.json') or 'receipts' in child.parts:
                    raise ValueError('evidence mixed into cache; retain/route it first: ' + str(child))
                files.append(child)
    unique = {}
    for file in files:
        info = file.stat()
        key = (info.st_dev, info.st_ino)
        value = unique.setdefault(key, {'bytes': info.st_size, 'inside': 0, 'links': info.st_nlink})
        value['inside'] += 1
    return {'path': str(target), 'files': len(files),
            'unique_file_bytes': sum(v['bytes'] for v in unique.values()),
            'reclaimable_file_data_bytes': sum(v['bytes'] for v in unique.values() if v['inside'] == v['links'])}


def foreign_compiler_scope(cwd, argv, environment):
    """Only fully observed foreign paths and explicit targets permit cleanup."""
    cwd = Path(cwd).resolve(strict=True)
    root = ROOT.resolve()
    root_text = str(root).replace('\\', '/').casefold()
    def overlaps(path):
        path = Path(path)
        path = (path if path.is_absolute() else cwd / path).resolve()
        return path == root or path.is_relative_to(root) or root.is_relative_to(path)
    if cwd == root or cwd.is_relative_to(root):
        raise ValueError('live compiler cwd references this checkout')
    # Environment remains inherited by nested compilation even if the outer
    # Cargo has a CLI target override. Never let that override hide own paths.
    if any(root_text in str(value).replace('\\', '/').casefold() for value in environment.values()):
        raise ValueError('live compiler environment references this checkout')
    env_target = environment.get('CARGO_TARGET_DIR')
    if env_target and overlaps(env_target):
        raise ValueError('inherited compiler target overlaps this checkout')
    for name in ('RUSTC', 'RUSTDOC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CC', 'CXX', 'AR', 'LD'):
        value = environment.get(name, '')
        if ('/' in value or '\\' in value) and overlaps(value):
            raise ValueError('inherited compiler tool references this checkout')
    # Opaque compiler/linker instructions can hide paths. Do not build a
    # response-file or flags resolver in a cache cleaner: postpone instead.
    for name, value in environment.items():
        if value and ('RUSTFLAGS' in name or 'RUSTDOCFLAGS' in name or name in ('CFLAGS', 'CXXFLAGS', 'LDFLAGS')):
            raise ValueError('live compiler has opaque inherited flags; postpone cleanup')
        if value and name.startswith('CARGO_') and name.endswith(('_LINKER', '_RUSTC', '_RUSTDOC', '_RUSTC_WRAPPER', '_RUSTC_WORKSPACE_WRAPPER')) and ('/' in value or '\\' in value) and overlaps(value):
            raise ValueError('inherited Cargo tool references this checkout')
        if value and name.startswith('CARGO_') and name.endswith('_RUNNER'):
            raise ValueError('live compiler has opaque runner configuration; postpone cleanup')
    if any('@' in value for value in argv):
        raise ValueError('live compiler response-file ownership is unknown')
    path_options = {'--manifest-path', '--target-dir', '--out-dir', '--out', '--sysroot', '-o', '-L'}
    previous = None
    for value in argv:
        if root_text in str(value).replace('\\', '/').casefold():
            raise ValueError('live compiler argv references this checkout')
        if value == '--config' or value.startswith('--config='):
            raise ValueError('custom live Cargo configuration ownership is unknown')
        candidate = value.rsplit('=', 1)[-1].strip('"\'')
        if candidate.startswith('/out:'):
            candidate = candidate[5:]
        if previous in path_options or '/' in candidate or '\\' in candidate or Path(candidate).suffix.lower() in ('.rs', '.toml', '.rlib', '.rmeta', '.lib', '.exe', '.obj', '.pdb', '.dll'):
            try:
                if overlaps(candidate):
                    raise ValueError('live compiler path argument overlaps this checkout')
            except OSError as error:
                raise ValueError('live compiler path ownership is unknown') from error
        previous = value
    target = None
    for index, value in enumerate(argv):
        if value == '--':
            break  # following values are test/program arguments, never Cargo options
        if value == '--target-dir':
            if index + 1 >= len(argv):
                raise ValueError('unknown compiler target directory')
            target = argv[index + 1]
        elif value.startswith('--target-dir='):
            target = value.split('=', 1)[1]
    target = target or env_target
    if not target:
        raise ValueError('live compiler has no explicit target; postpone cleanup')
    if overlaps(target):
        raise ValueError('live compiler target overlaps this checkout')
    target = Path(target)
    target = (target if target.is_absolute() else cwd / target).resolve()
    return {'cwd': str(cwd), 'target_dir': str(target)}


def refuse_active_processes():
    if os.name != 'nt':
        # Automatic removal is Windows-only until another process ownership oracle is implemented.
        raise ValueError('automatic apply requires Windows process ownership inspection')
    command = ['powershell', '-NoProfile', '-NonInteractive', '-Command',
               'Get-CimInstance Win32_Process | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress']
    rows = json.loads(subprocess.check_output(command, text=True))
    if isinstance(rows, dict):
        rows = [rows]
    for row in rows:
        if row['Name'].lower() in ('cargo.exe', 'rustc.exe', 'rust-lld.exe', 'cargo-nextest.exe'):
            # CIM has no reliable cwd/environment. Only an actual read of the
            # same user's process can prove its explicit target is foreign;
            # absence/access failures stay closed. Never print its environment.
            try:
                import psutil
            except ImportError as error:
                raise ValueError('live compiler ownership unavailable; postpone cleanup') from error
            try:
                process = psutil.Process(row['ProcessId'])
                if process.name().lower() not in ('cargo.exe', 'rustc.exe', 'rust-lld.exe', 'cargo-nextest.exe'):
                    continue
                scope = foreign_compiler_scope(process.cwd(), process.cmdline(), process.environ())
            except psutil.NoSuchProcess:
                continue
            except psutil.AccessDenied as error:
                raise ValueError('live compiler ownership unreadable; postpone cleanup') from error
            print(json.dumps({'foreign_compiler_kept': row['ProcessId'], **scope}), flush=True)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('caches', nargs='+', type=Path)
    ap.add_argument('--retired-source', help='ancestor revision proving an exact migrated Cargo cache owner')
    ap.add_argument('--apply', action='store_true')
    ap.add_argument('--idle-confirmed', action='store_true', help='owner confirms no reserved measurement/consumer')
    args = ap.parse_args(argv)
    if args.apply and not args.idle_confirmed:
        ap.error('--apply requires the cache owner to confirm --idle-confirmed')
    rows = [inspect(path, args.retired_source) for path in args.caches]
    if len({r['path'] for r in rows}) != len(rows):
        raise ValueError('duplicate cache targets')
    print(json.dumps({'mode': 'apply' if args.apply else 'preview', 'caches': rows}, indent=2), flush=True)
    if args.apply:
        refuse_active_processes()
        for row in rows:
            # Revalidate immediately before mutation; partial failures propagate, never report success.
            inspect(Path(row['path']), args.retired_source)
            shutil.rmtree(row['path'])
            print(json.dumps({'removed': row['path'], 'unique_bytes_removed_from_scope': row['unique_file_bytes'],
                              'file_data_bytes_reclaimed': row['reclaimable_file_data_bytes']}), flush=True)
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print('cache cleanup refused/failed: ' + str(error), file=sys.stderr)
        sys.exit(2)
