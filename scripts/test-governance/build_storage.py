#!/usr/bin/env python3
"""Preview/reclaim only explicitly owned idle Cargo caches in this checkout.

PLAN-910 extension: exact managed generated-consumer cache roots
(<ROOT>/target/generated-consumers/<ctx8>/) are admitted when their
context.json proves this checkout is the producer and no live build/run
lease (pid alive) holds them. Every other refusal (reparse, evidence, active
compilers, execution lease, foreign roots) is unchanged.

PLAN-1001 extension: --incremental mode governs Cargo *incremental* cache
units (target/<profile>/incremental, target/<triple>/<profile>/incremental)
with inventory -> preview (policy-bound snapshot manifest, zero deletion)
-> apply-manifest (maintenance-window locks + identity re-verification +
per-unit failure receipts). The whole-unit subtree is the governance unit;
rustc-internal session names are never parsed for expiry. Foreign owner
roots need dual proof: explicit policy registration AND a real workspace
manifest. The legacy exact-root CLI is unchanged."""
import argparse
import contextlib
import ctypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]


def reparse(path):
    info = path.lstat()
    return path.is_symlink() or bool(getattr(info, 'st_file_attributes', 0) & 0x400)


GENERATED_CONSUMERS = 'generated-consumers'
CONTEXT_SCHEMA = 'generated-build-context/v1'


def process_alive(pid):
    """Windows liveness via tasklist (no psutil dependency); unknown -> alive."""
    if os.name != 'nt':
        return Path('/proc', str(pid)).exists()
    try:
        out = subprocess.check_output(['tasklist', '/FI', 'PID eq %d' % pid, '/NH', '/FO', 'CSV'], text=True)
    except (OSError, subprocess.SubprocessError):
        return True
    return ('"%d"' % pid) in out


def generated_consumer_root(target):
    """PLAN-910: an exact managed generated-consumer cache root owned by THIS
    checkout: <ROOT>/target/generated-consumers/<ctx8>/ with a schema-v1
    context.json whose producer_root is this checkout. Anything else (another
    producer's root, a missing/foreign record, a nested path) is refused."""
    base = (ROOT / 'target' / GENERATED_CONSUMERS).resolve()
    if target.parent != base:
        return False
    record = target / 'context.json'
    if not record.is_file():
        raise ValueError('generated-consumer root without context.json is unowned: ' + str(target))
    data = json.loads(record.read_text(encoding='utf-8'))
    if data.get('schema') != CONTEXT_SCHEMA:
        raise ValueError('generated-consumer root has an unknown context schema: ' + str(data.get('schema')))
    # Generator-written context.json records producer roots verbatim (\\?\); compare
    # through the same canonical form the foreign-root path uses (normalize_root).
    if normalize_root(data.get('producer_root', '')) != normalize_root(ROOT):
        raise ValueError('generated-consumer root belongs to another producer: ' + str(data.get('producer_root')))
    if data.get('context_id') != target.name:
        raise ValueError('generated-consumer root name does not match its context id')
    return True


def refuse_live_leases(target):
    """A build/run lease whose pid is still alive proves the root is in use;
    stale leases (dead pid) are reported and ignored, never deleted here."""
    lease_dir = target / '.lease'
    if not lease_dir.is_dir():
        return []
    stale = []
    for lease in sorted(lease_dir.glob('*.json')):
        try:
            data = json.loads(lease.read_text(encoding='utf-8'))
            pid = int(data.get('pid'))
        except (ValueError, TypeError, OSError) as error:
            raise ValueError('unreadable lease (fail closed): ' + str(lease)) from error
        if process_alive(pid):
            raise ValueError('live build/run lease pid %d (%s) holds this root' % (pid, data.get('purpose')))
        stale.append(str(lease))
    return stale


def inspect(path, retired_source=None):
    lexical = Path(path).absolute()
    for parent in [lexical, *lexical.parents]:
        if os.path.lexists(parent) and reparse(parent):
            raise ValueError('reparse point/ancestor refused: ' + str(parent))
    target = lexical.resolve(strict=True)
    allowed = target in {ROOT / 'target', ROOT / 'target/test-tools/regular', ROOT / 'target/test-tools/batch', ROOT / 'experimental/ace/target-batch'} or (
        target.is_relative_to(ROOT / 'experimental') and target.name == 'target'
        and (target.parent / 'Cargo.toml').is_file())
    stale_leases = []
    if not allowed and generated_consumer_root(target):
        allowed = True
        stale_leases = refuse_live_leases(target)
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
    # PLAN-1014 T-05: a live fixed-test lease still refuses (its pid is checked,
    # not mere existence); a dead-owner leftover is reported as stale and the
    # read-only inspect proceeds — the next fixed-tests run reclaims it before
    # executing (fixed-tests._reclaim_dead_lease owns recovery, never deletion here).
    lease_dir = ROOT / 'target/test-groups/active'
    if lease_dir.exists():
        try:
            lease_pid = int(json.loads(
                (lease_dir / 'owner.json').read_text(encoding='utf-8'))['pid'])
        except (ValueError, TypeError, KeyError, OSError) as error:
            raise ValueError('fixed-test execution lease present but unreadable '
                             '(fail closed): ' + str(lease_dir)) from error
        if process_alive(lease_pid):
            raise ValueError('fixed-test execution/measurement lease is active (pid %d)' % lease_pid)
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
            'reclaimable_file_data_bytes': sum(v['bytes'] for v in unique.values() if v['inside'] == v['links']),
            'stale_leases_ignored': stale_leases}


def foreign_compiler_scope(cwd, argv, environment):
    """Only fully observed foreign paths and explicit targets permit cleanup."""
    cwd = Path(cwd).resolve(strict=True)
    root = ROOT.resolve()
    root_text = str(root).replace('\\', '/').casefold()
    def overlaps(path):
        path = Path(path)
        if path.drive and not path.is_absolute():
            raise ValueError('drive-relative process path ownership is unknown')
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
        if ('/' in value or '\\' in value or Path(value).drive) and overlaps(value):
            raise ValueError('inherited compiler tool references this checkout')
    # Opaque compiler/linker instructions can hide paths. Do not build a
    # response-file or flags resolver in a cache cleaner: postpone instead.
    for name, value in environment.items():
        if value and ('RUSTFLAGS' in name or 'RUSTDOCFLAGS' in name or name in ('CFLAGS', 'CXXFLAGS', 'LDFLAGS')):
            raise ValueError('live compiler has opaque inherited flags; postpone cleanup')
        if value and name.startswith('CARGO_') and name.endswith(('_LINKER', '_RUSTC', '_RUSTDOC', '_RUSTC_WRAPPER', '_RUSTC_WORKSPACE_WRAPPER')) and ('/' in value or '\\' in value or Path(value).drive) and overlaps(value):
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
        if previous in path_options or '/' in candidate or '\\' in candidate or Path(candidate).drive or Path(candidate).suffix.lower() in ('.rs', '.toml', '.rlib', '.rmeta', '.lib', '.exe', '.obj', '.pdb', '.dll'):
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


# ============================================================
# PLAN-1001: incremental-cache lifecycle (restricted mode)
# ============================================================

INCREMENTAL = 'incremental'
POLICY_SCHEMA = 'incremental-cache-policy/v1'
MANIFEST_SCHEMA = 'incremental-cache-manifest/v1'
# T-02 (2026-11-10, cargo 1.98.0 / Windows): cargo's build-directory mutex
# lives at PROFILE level, and a second producer really blocks on it. Hold
# every present profile lock file before touching a unit.
PROFILE_LOCKS = ('.cargo-lock', '.cargo-build-lock', '.cargo-artifact-lock')
REQUIRED_LOCK = '.cargo-lock'
# Separately owned cache roots nested under an admitted target root: their
# incremental units belong to their own root, never to the parent's.
NESTED_OWNED_DIRS = ('generated-consumers', 'test-tools')
EVIDENCE_NAMES = ('results.json', 'report.json')

_k32 = None


def allocated_bytes(path):
    """On-disk allocation of one file, or None when unreadable.

    Never substitute the logical length for a failed measurement: callers
    count unreadable files separately and demote the unit to 'unknown'."""
    global _k32
    if os.name == 'nt':
        if _k32 is None:
            from ctypes import wintypes
            _k32 = ctypes.WinDLL('kernel32', use_last_error=True)
            _k32.GetCompressedFileSizeW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(wintypes.DWORD)]
            _k32.GetCompressedFileSizeW.restype = wintypes.DWORD
            _k32._wintypes = wintypes
        wintypes = _k32._wintypes
        high = wintypes.DWORD(0)
        low = _k32.GetCompressedFileSizeW(str(path), ctypes.byref(high))
        if low == 0xFFFFFFFF and ctypes.get_last_error() != 0:
            return None
        return (high.value << 32) | low
    info = os.stat(path)
    blocks = getattr(info, 'st_blocks', None)
    return None if blocks is None else blocks * 512


def toolchain_fingerprint():
    return {'cargo': subprocess.check_output(['cargo', '--version'], text=True).strip(),
            'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()}


def normalize_root(path):
    """Canonical comparison form. Strips Windows verbatim (\\\\?\\) prefixes:
    generator-written context.json records producer roots verbatim."""
    text = str(path)
    if text.startswith('\\\\?\\UNC\\'):
        text = '\\\\' + text[8:]
    elif text.startswith('\\\\?\\'):
        text = text[4:]
    return str(Path(text).resolve(strict=False)).replace('\\', '/').casefold()


def load_policy(path):
    """Strict policy: exact schema, explicit owners, dual proof for foreign
    roots, watermarks with rationale. No extra keys, no embedded paths
    beyond registered owner roots."""
    try:
        data = json.loads(Path(path).read_text(encoding='utf-8'))
    except (OSError, ValueError) as error:
        raise ValueError('policy unreadable: ' + str(error)) from error
    if not isinstance(data, dict) or data.get('schema') != POLICY_SCHEMA:
        raise ValueError('policy schema mismatch: ' + str(data.get('schema') if isinstance(data, dict) else None))
    allowed = {'schema', 'owners', 'watermarks', 'notes'}
    if set(data) - allowed:
        raise ValueError('policy carries unknown keys: ' + str(sorted(set(data) - allowed)))
    owners = data.get('owners')
    if not isinstance(owners, list) or not owners:
        raise ValueError('policy needs a non-empty owners list')
    registered = {}
    for owner in owners:
        if not isinstance(owner, dict) or set(owner) - {'owner', 'root', 'rationale', 'foreign'}:
            raise ValueError('policy owner entry malformed: ' + json.dumps(owner)[:120])
        if not owner.get('owner') or not owner.get('root') or not owner.get('rationale'):
            raise ValueError('policy owner needs owner/root/rationale')
        registered[normalize_root(owner['root'])] = dict(owner)
    watermarks = data.get('watermarks', {})
    if not isinstance(watermarks, dict):
        raise ValueError('policy watermarks must be an object keyed by owner')
    for key, mark in watermarks.items():
        if not isinstance(mark, dict) or set(mark) - {'high_bytes', 'low_bytes', 'rationale'}:
            raise ValueError('watermark entry malformed for ' + key)
        if (mark.get('high_bytes') is not None or mark.get('low_bytes') is not None) and not mark.get('rationale'):
            raise ValueError('watermark needs a rationale for ' + key)
        for name in ('high_bytes', 'low_bytes'):
            value = mark.get(name)
            if value is not None and (not isinstance(value, int) or isinstance(value, bool) or value < 0):
                raise ValueError('watermark %s must be a non-negative integer for %s' % (name, key))
    return {'registered_roots': registered, 'watermarks': watermarks,
            'sha256': hashlib.sha256(Path(path).read_bytes()).hexdigest()}


def admit_incremental_root(target, policy):
    """Cache-root admission for the incremental mode. Same this-checkout
    admission rules as inspect(); foreign roots additionally need explicit
    policy registration AND a real workspace manifest (dual proof)."""
    lexical = Path(target).absolute()
    for parent in [lexical, *lexical.parents]:
        if os.path.lexists(parent) and reparse(parent):
            raise ValueError('reparse point/ancestor refused: ' + str(parent))
    resolved = lexical.resolve(strict=True)
    meta = {'leases': [], 'kind': 'this-checkout'}
    allowed = resolved in {ROOT / 'target', ROOT / 'target/test-tools/regular', ROOT / 'target/test-tools/batch',
                           ROOT / 'experimental/ace/target-batch'} or (
        resolved.is_relative_to(ROOT / 'experimental') and resolved.name == 'target'
        and (resolved.parent / 'Cargo.toml').is_file())
    if not allowed and generated_consumer_root(resolved):
        allowed = True
        meta['leases'] = refuse_live_leases(resolved)
    if not allowed and policy is not None:
        entry = policy['registered_roots'].get(normalize_root(resolved))
        # Foreign root: registration + real workspace manifest (dual proof).
        # A non-foreign registration alone never admits a path outside the
        # this-checkout rules above — policy is a scope list, not a key.
        if entry is not None and entry.get('foreign') and resolved.name == 'target' \
                and (resolved.parent / 'Cargo.toml').is_file():
            allowed = True
            meta['kind'] = 'foreign:' + entry['owner']
        if not allowed:
            # Registered workspace's managed generated-consumer context:
            # admission chains through that workspace's registration plus the
            # context's own producer proof (still dual proof).
            for candidate in policy['registered_roots'].values():
                if not candidate.get('foreign'):
                    continue
                foreign_target = Path(candidate['root'])
                gc_base = foreign_target / GENERATED_CONSUMERS
                if resolved.parent != gc_base or not (foreign_target.parent / 'Cargo.toml').is_file():
                    continue
                record = resolved / 'context.json'
                try:
                    data = json.loads(record.read_text(encoding='utf-8'))
                except (OSError, ValueError) as error:
                    raise ValueError('foreign generated-consumer context unreadable: ' + str(record)) from error
                if data.get('schema') != CONTEXT_SCHEMA or data.get('context_id') != resolved.name \
                        or normalize_root(data.get('producer_root', '')) != normalize_root(foreign_target.parent):
                    raise ValueError('foreign generated-consumer context does not prove its producer: ' + str(resolved))
                allowed = True
                meta['kind'] = 'foreign-generated:' + candidate['owner']
                meta['leases'] = refuse_live_leases(resolved)
                break
    if not allowed:
        raise ValueError('not a registered incremental-cache owner root: ' + str(resolved))
    meta['root'] = str(resolved)
    return meta


def find_incremental_units(root_path):
    """Profile-level incremental subtrees of an admitted cache root.

    Only <profile>/incremental and <triple>/<profile>/incremental shapes are
    units; anything deeper belongs to another unit's subtree. Separately
    owned nested roots are pruned so their units are never double-counted."""
    root_path = Path(root_path)
    units = []
    for folder, dirs, _names in os.walk(root_path, followlinks=False):
        relative = Path(folder).relative_to(root_path)
        depth = 0 if relative == Path('.') else len(relative.parts)
        if depth == 0:
            dirs[:] = [d for d in dirs if d not in NESTED_OWNED_DIRS]
            continue
        if os.path.basename(folder) == INCREMENTAL and 2 <= depth <= 3:
            units.append({'path': str(Path(folder).resolve(strict=True)),
                          'profile': relative.parts[-1] if relative.parts[-1] != INCREMENTAL else relative.parts[-2],
                          'triple': relative.parts[0] if len(relative.parts) >= 3 else None})
            dirs[:] = []
            continue
        if depth >= 3:
            dirs[:] = []
    return units


def measure_incremental_unit(unit_path):
    """Measure one unit: logical + allocated bytes, file-id identity, hard
    links, mtime range (display only), and a change-detecting tree digest.
    Refuses reparse points and evidence files inside the unit."""
    unit_path = Path(unit_path)
    files = 0
    logical = 0
    allocated = 0
    unknown_alloc = 0
    id_links = {}
    oldest = newest = None
    digest_rows = []
    for folder, dirs, names in os.walk(unit_path, followlinks=False):
        for name in [*dirs, *names]:
            child = Path(folder) / name
            if reparse(child):
                raise ValueError('reparse point inside incremental unit: ' + str(child))
        for name in names:
            child = Path(folder) / name
            if child.name in EVIDENCE_NAMES or 'receipts' in child.parts:
                raise ValueError('evidence mixed into incremental unit; retain/route it first: ' + str(child))
            info = child.stat()
            files += 1
            logical += info.st_size
            alloc = allocated_bytes(child)
            if alloc is None:
                unknown_alloc += 1
            else:
                allocated += alloc
            key = (info.st_dev, info.st_ino)
            entry = id_links.setdefault(key, {'bytes': info.st_size, 'alloc': alloc,
                                              'inside': 0, 'links': info.st_nlink})
            entry['inside'] += 1
            mtime = info.st_mtime_ns
            oldest = mtime if oldest is None or mtime < oldest else oldest
            newest = mtime if newest is None or mtime > newest else newest
            digest_rows.append((child.relative_to(unit_path).as_posix(), info.st_ino, info.st_size, mtime))
    digest = hashlib.sha256('\n'.join('%s|%d|%d|%d' % row for row in sorted(digest_rows)).encode()).hexdigest()
    return {'files': files, 'logical_bytes': logical, 'allocated_bytes': allocated,
            'unknown_alloc_files': unknown_alloc, 'file_ids': id_links, 'tree_digest': digest,
            'mtime_min_ns': oldest, 'mtime_max_ns': newest}


def lenient_size(path):
    """Best-effort accounting for a unit whose strict measurement refused
    (evidence/reparse inside). The unit stays protected; these numbers only
    feed protected-bytes bookkeeping and never enable deletion."""
    files = logical = allocated = unknown = 0
    for folder, _dirs, names in os.walk(path, followlinks=False):
        for name in names:
            try:
                info = (Path(folder) / name).stat()
            except OSError:
                continue
            files += 1
            logical += info.st_size
            alloc = allocated_bytes(Path(folder) / name)
            if alloc is None:
                unknown += 1
            else:
                allocated += alloc
    return files, logical, allocated, unknown


def incremental_inventory(policy):
    """Inventory every policy-registered owner root. Units are measured
    once; cross-unit hard links are deduplicated globally; statuses are
    protected / unknown / eligible-idle. Read-only."""
    units = []
    failures = []
    for key in sorted(policy['registered_roots']):
        owner = policy['registered_roots'][key]
        root_arg = Path(owner['root'])  # registered spelling; the key is only a lookup id
        try:
            meta = admit_incremental_root(root_arg, policy)
            found = find_incremental_units(meta['root'])
        except ValueError as error:
            failures.append({'owner': owner['owner'], 'root': str(root_arg), 'status': 'protected', 'reason': str(error)})
            continue
        for unit in found:
            row = {'owner': owner['owner'], 'root': meta['root'], 'kind': meta['kind'],
                   'stale_leases_ignored': meta['leases'], **unit}
            try:
                measured = measure_incremental_unit(unit['path'])
            except ValueError as error:
                files, logical, allocated, unknown = lenient_size(unit['path'])
                row.update({'status': 'protected', 'refusal_reason': str(error),
                            'accounting': 'lenient', 'files': files,
                            'logical_bytes': logical, 'allocated_bytes': allocated,
                            'unknown_alloc_files': unknown,
                            'file_ids': {}, 'tree_digest': None})
                failures.append({k: row[k] for k in ('owner', 'root', 'path', 'status', 'refusal_reason')})
                units.append(row)
                continue
            row.update(measured)
            row['status'] = 'unknown' if measured['unknown_alloc_files'] else 'eligible-idle'
            units.append(row)
    # Cross-unit hard-link dedup: a file id referenced by more than one unit
    # is not reclaimable by deleting any single unit.
    owners_by_id = {}
    for row in units:
        for file_id in row['file_ids']:
            owners_by_id.setdefault(file_id, []).append(row['path'])
    shared = {fid: paths for fid, paths in owners_by_id.items() if len(paths) > 1}
    for row in units:
        ids = row['file_ids']
        row['shared_file_ids'] = sum(1 for fid in ids if fid in shared)
        row['reclaimable_allocated_upper_bound'] = sum(
            entry['alloc'] or 0 for fid, entry in ids.items()
            if entry['alloc'] is not None and entry['inside'] == entry['links'] and fid not in shared)
        del row['file_ids']  # identity lives in the digest; raw maps are not serialized
    return {'units': units, 'refused': failures, 'shared_file_ids_across_units': len(shared)}


def watermark_report(policy, inventory):
    """Watermarks sort and alert only: report protected bytes and shortfall,
    never authorize crossing a protection."""
    report = {}
    for owner in sorted({unit['owner'] for unit in inventory['units']} | {r['owner'] for r in inventory['refused']}):
        rows = [u for u in inventory['units'] if u['owner'] == owner]
        eligible = sum(u['allocated_bytes'] for u in rows if u['status'] == 'eligible-idle')
        protected = sum(u['allocated_bytes'] for u in rows if u['status'] != 'eligible-idle')
        entry = {'eligible_idle_allocated_bytes': eligible, 'non_eligible_allocated_bytes': protected,
                 'refused_roots': sum(1 for f in inventory['refused'] if f['owner'] == owner),
                 'high_bytes': None, 'low_bytes': None, 'shortfall_bytes': None}
        mark = policy['watermarks'].get(owner)
        if mark:
            entry.update({k: mark.get(k) for k in ('high_bytes', 'low_bytes')})
            if mark.get('high_bytes') is not None:
                total = eligible + protected
                entry['above_high_watermark'] = total > mark['high_bytes']
                entry['shortfall_bytes'] = max(0, total - mark['high_bytes'] - eligible)
        report[owner] = entry
    return report


def build_manifest(policy):
    inventory = incremental_inventory(policy)
    manifest = {'schema': MANIFEST_SCHEMA, 'generated_at': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                'policy_sha256': policy['sha256'],
                'toolchain': toolchain_fingerprint(),
                'zero_deletion': True,
                'units': [{k: v for k, v in unit.items()
                           if k != 'stale_leases_ignored' or v}
                          for unit in inventory['units']],
                'refused': inventory['refused'],
                'shared_file_ids_across_units': inventory['shared_file_ids_across_units'],
                'watermarks': watermark_report(policy, inventory)}
    return manifest, inventory


def retention_hold_paths():
    """Re-read the retention registry before any deletion. Missing registry
    is an anomaly, not consent: fail closed."""
    registry = ROOT / '.retention' / 'policy.json'
    if not registry.is_file():
        raise ValueError('retention registry missing; cannot prove absence of user holds')
    try:
        data = json.loads(registry.read_text(encoding='utf-8'))
    except (OSError, ValueError) as error:
        raise ValueError('retention registry unreadable: ' + str(error)) from error
    held = []
    for hold in data.get('holds', []) or []:
        path = hold.get('path') if isinstance(hold, dict) else None
        if path:
            held.append(str(path))
    for scope in data.get('scopes', []) or []:
        if isinstance(scope, dict) and 'held' in str(scope.get('status', '')):
            if scope.get('path'):
                held.append(str(scope['path']))
    return held


def acquire_profile_locks(unit_rows):
    """Hold every cargo profile mutex of the affected roots in a fixed
    order. A lock held by a live producer makes the whole apply refuse
    before any deletion (fail closed, never wait silently)."""
    handles = []
    locked = []
    try:
        for profile_dir in sorted({str(Path(u['path']).parent) for u in unit_rows}):
            primary = Path(profile_dir) / REQUIRED_LOCK
            if not primary.is_file():
                raise ValueError('cargo profile lock file missing; cannot establish mutex: ' + str(primary))
            for name in PROFILE_LOCKS:
                candidate = Path(profile_dir) / name
                if not candidate.is_file():
                    continue
                fd = os.open(str(candidate), os.O_RDWR)
                try:
                    _lock_byte(fd)
                except OSError as error:
                    os.close(fd)
                    raise ValueError('cargo lock held by another producer (%s): %s' % (candidate, error)) from error
                handles.append(fd)
                locked.append(str(candidate))
    except ValueError:
        for fd in handles:
            with contextlib.suppress(OSError):
                _unlock_byte(fd)
                os.close(fd)
        raise
    return handles, locked


def _lock_byte(fd):
    if os.name == 'nt':
        import msvcrt
        os.lseek(fd, 0, os.SEEK_SET)
        msvcrt.locking(fd, msvcrt.LK_NBLCK, 1)
    else:
        import fcntl
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)


def _unlock_byte(fd):
    if os.name == 'nt':
        import msvcrt
        os.lseek(fd, 0, os.SEEK_SET)
        with contextlib.suppress(OSError):
            msvcrt.locking(fd, msvcrt.LK_UNLCK, 1)
    else:
        import fcntl
        with contextlib.suppress(OSError):
            fcntl.flock(fd, fcntl.LOCK_UN)


def run_incremental(args):
    policy = load_policy(args.policy) if args.policy else None
    if args.inventory:
        meta = admit_incremental_root(args.inventory, policy)
        units = find_incremental_units(meta['root'])
        rows = []
        for unit in units:
            row = {'owner': policy['registered_roots'][normalize_root(args.inventory)]['owner']
                   if policy and normalize_root(args.inventory) in policy['registered_roots'] else meta['kind'],
                   'root': meta['root'], **unit}
            try:
                row.update(measure_incremental_unit(unit['path']))
                row['status'] = 'unknown' if row['unknown_alloc_files'] else 'eligible-idle'
            except ValueError as error:
                row.update({'status': 'protected', 'refusal_reason': str(error)})
            row.pop('file_ids', None)
            rows.append(row)
        result = {'schema': MANIFEST_SCHEMA, 'mode': 'inventory', 'root': meta['root'],
                  'stale_leases_ignored': meta['leases'], 'units': rows}
        Path(args.output).write_text(json.dumps(result, indent=1), encoding='utf-8')
        print(json.dumps(result, indent=1))
        return 0
    if args.preview:
        manifest, _inventory = build_manifest(policy)
        Path(args.output).write_text(json.dumps(manifest, indent=1), encoding='utf-8')
        print(json.dumps({'mode': 'preview', 'units': len(manifest['units']),
                          'eligible': sum(1 for u in manifest['units'] if u['status'] == 'eligible-idle'),
                          'refused': len(manifest['refused']), 'output': str(args.output)}))
        return 0
    return apply_incremental_manifest(args, policy)


def apply_incremental_manifest(args, policy):
    """Consume exactly one preview snapshot. Validation failure of any unit
    aborts with zero deletion; IO failures mid-deletion are receipted per
    unit and the exit code stays non-zero."""
    try:
        manifest = json.loads(Path(args.apply_manifest).read_text(encoding='utf-8'))
    except (OSError, ValueError) as error:
        raise ValueError('apply manifest unreadable: ' + str(error)) from error
    if manifest.get('schema') != MANIFEST_SCHEMA:
        raise ValueError('apply manifest schema mismatch')
    recorded_policy = manifest.get('policy_sha256')
    if policy is None or policy['sha256'] != recorded_policy:
        raise ValueError('policy does not match the snapshot policy (sha mismatch); regenerate the preview')
    refuse_active_processes()
    held = retention_hold_paths()
    # Re-derive the eligible set from the CURRENT tree under the SAME policy:
    # a manifest line can never promote an unlisted/changed path to deletable.
    current = incremental_inventory(policy)
    current_eligible = {u['path']: u for u in current['units'] if u['status'] == 'eligible-idle'}
    selected = []
    for unit in manifest['units']:
        if unit.get('status') != 'eligible-idle':
            continue
        path = unit.get('path', '')
        live = current_eligible.get(path)
        if live is None:
            raise ValueError('manifest unit is no longer an eligible unit under the current tree: ' + path)
        if live['tree_digest'] != unit.get('tree_digest') or live['files'] != unit.get('files'):
            raise ValueError('manifest unit identity changed since the preview: ' + path)
        for hold in held:
            if Path(path).is_relative_to(Path(hold)):
                raise ValueError('retention hold covers this unit: ' + path)
        selected.append(live)
    handles = None
    receipt = {'schema': 'incremental-cache-apply/v1', 'manifest': str(args.apply_manifest),
               'policy_sha256': recorded_policy, 'deleted': [], 'failed': [], 'zero_delete': not selected}
    try:
        handles, locked = acquire_profile_locks(selected)
        receipt['locks_held'] = locked
        # Under the mutex, re-verify every identity once more.
        for live in selected:
            recheck = measure_incremental_unit(live['path'])
            if recheck['tree_digest'] != live['tree_digest'] or recheck['files'] != live['files']:
                raise ValueError('unit changed while locks were being acquired: ' + live['path'])
        for live in selected:
            try:
                shutil.rmtree(live['path'])
                receipt['deleted'].append({'path': live['path'], 'files': live['files'],
                                           'allocated_bytes_estimate': live['allocated_bytes'],
                                           'reclaimable_upper_bound': live['reclaimable_allocated_upper_bound']})
            except OSError as error:
                receipt['failed'].append({'path': live['path'], 'reason': str(error)})
    finally:
        if handles:
            for fd in handles:
                with contextlib.suppress(OSError):
                    _unlock_byte(fd)
                with contextlib.suppress(OSError):
                    os.close(fd)
    Path(args.output).write_text(json.dumps(receipt, indent=1), encoding='utf-8')
    print(json.dumps(receipt, indent=1))
    if receipt['failed']:
        print('apply finished with per-unit failures; deleted %d, failed %d'
              % (len(receipt['deleted']), len(receipt['failed'])), file=sys.stderr)
        return 3
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('caches', nargs='*', type=Path)
    ap.add_argument('--retired-source', help='ancestor revision proving an exact migrated Cargo cache owner')
    ap.add_argument('--apply', action='store_true')
    ap.add_argument('--incremental', action='store_true', help='PLAN-1001 incremental-cache lifecycle mode')
    ap.add_argument('--inventory', type=Path, help='(incremental) read-only inventory of one registered owner root')
    ap.add_argument('--preview', action='store_true', help='(incremental) policy-bound candidate snapshot; zero deletion')
    ap.add_argument('--apply-manifest', type=Path, help='(incremental) apply exactly one preview snapshot')
    ap.add_argument('--policy', type=Path, help='(incremental) policy.json: owners + watermarks + rationale')
    ap.add_argument('--output', type=Path, help='(incremental) where the preview/apply receipt is written')
    ap.add_argument('--idle-confirmed', action='store_true', help='owner confirms no reserved measurement/consumer')
    args = ap.parse_args(argv)
    if args.incremental:
        if args.caches or args.apply or args.retired_source:
            ap.error('--incremental mode takes no positional caches/--apply/--retired-source')
        actions = sum(1 for flag in (args.inventory, args.preview, args.apply_manifest) if flag)
        if actions != 1:
            ap.error('--incremental requires exactly one of --inventory/--preview/--apply-manifest')
        if (args.preview or args.apply_manifest) and args.policy is None:
            ap.error('--incremental preview/apply require --policy')
        if args.output is None:
            ap.error('--incremental requires --output')
        if args.apply_manifest and not args.idle_confirmed:
            ap.error('--apply-manifest requires the owner to confirm a maintenance window via --idle-confirmed')
        return run_incremental(args)
    if not args.caches:
        ap.error('exact cache roots are required (or pass --incremental)')
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
