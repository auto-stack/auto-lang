#!/usr/bin/env python3
"""Fixed Cargo packages/Rust namespaces, dispatched to the original runners.

No leaf catalog, source discovery cache, or inferred consumer graph is consumed.
"""
from __future__ import annotations

import argparse
import contextlib
from collections import defaultdict
import hashlib
import json
import os
from pathlib import Path
import subprocess
import uuid
import sys
import tempfile
import time
import tomllib

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'scripts/test-governance'))
from process_capture import capture
from output_admission import admit_output, audit_suite_outputs


def write(path, value):
    temporary = path.with_name(path.name + '.tmp-' + uuid.uuid4().hex)
    try:
        with temporary.open('x', encoding='utf-8', newline='\n') as stream:
            stream.write(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def configuration(path=None):
    raw = (path or ROOT / 'scripts/test-groups.toml').read_bytes()
    value = tomllib.loads(raw.decode('utf-8'))
    value['_sha256'] = hashlib.sha256(raw).hexdigest()
    groups = value['group']
    names = [g['name'] for g in groups]
    if value['schema'] != 1 or len(names) != len(set(names)) or not names:
        raise ValueError('invalid/duplicate fixed groups')
    for g in groups:
        if g['runner'] not in ('nextest', 'python', 'case') or g['domain'] not in ('legacy', 'ace'):
            raise ValueError('unknown fixed runner/domain')
        if 'timeout_seconds' in g and (type(g['timeout_seconds']) is not int or g['timeout_seconds'] <= 0):
            raise ValueError('invalid fixed group timeout_seconds')
        workspace = (ROOT / g['workspace']).resolve()
        if not workspace.is_relative_to(ROOT):
            raise ValueError('workspace escapes repository')
        if g['domain'] != 'ace' and (g['workspace'].startswith('experimental/') or g['tools']):
            raise ValueError('ACE tools/workspace in a legacy group')
    return value, {g['name']: g for g in groups}


def select(names, value, groups):
    if not names:
        names = value['default']
    if 'all' in names:
        if names != ['all']:
            raise ValueError('all cannot be mixed with another group')
        names = [name for name, g in groups.items() if g.get('all', name != 'daily')]
    sets = value.get('sets', {})
    names = [member for name in names for member in sets.get(name, [name])]
    unknown = set(names) - groups.keys()
    if unknown:
        raise ValueError('unknown fixed group: ' + ', '.join(sorted(unknown)))
    return [groups[n] for n in dict.fromkeys(names)]


def invocations(groups, test_name=None):
    merged = {}
    for g in groups:
        key = (g['runner'], g['workspace'], tuple(g['args']), g['domain'], tuple(g['tools']),
               g.get('file'), g.get('suite'), g.get('check'), bool(g.get('respect_default_filter')), g.get('timeout_seconds'))
        if key not in merged:
            merged[key] = {**g, 'groups': [], 'expressions': []}
        merged[key]['groups'].append(g['name'])
        merged[key]['expressions'].append(g['expression'])
    for g in merged.values():
        expression = ' or '.join('(' + e + ')' for e in dict.fromkeys(g['expressions']))
        if os.environ.get('T3_MILESTONE') != '1':
            expression = '(' + expression + ') and not test(=tests::aavm2_t3::test_aavm2_t3_tower_milestone)'
        if test_name:
            # Deliberate navigation inside the fixed selected groups, not a global scan.
            if not all(c.isalnum() or c in '_:-./' for c in test_name):
                raise ValueError('unsupported test-name characters')
            expression = '(' + expression + ') and test(' + test_name + ')'
        g['expression'] = expression
    return list(merged.values())


def owned_target_override():
    value = os.environ.get('CARGO_TARGET_DIR')
    if not value:
        return None
    target = Path(value).resolve()
    if not target.is_relative_to(ROOT):
        raise ValueError('CARGO_TARGET_DIR must stay inside this checkout')
    return target


def nextest_args(group, ignored=False):
    args = [*group['args'], '--ignore-default-filter', '-E', group['expression']]
    if group['workspace'].startswith('experimental/'):
        # ACE tests/workers share this tree; batch remains a distinct target.
        # Native runtime profiles/triples remain explicit at their real consumers.
        args += ['--target-dir', str(ROOT / ('experimental/ace/target-batch' if group['groups'] == ['ace-batch'] else 'experimental/ace/target'))]
    elif owned_target_override():
        args += ['--target-dir', str(owned_target_override())]
    if 'aavm' in group['groups'] and os.environ.get('T3_MILESTONE') == '1':
        args[args.index('--config-file') + 1] = '.config/nextest-t3.toml'
    # Daily retains nextest's original default-filter and original applicability.
    if group['groups'] == ['daily'] or group.get('respect_default_filter'):
        args.remove('--ignore-default-filter')
    if ignored:
        args += ['--run-ignored', 'only']
    if os.name == 'nt' and len(subprocess.list2cmdline(args).encode('utf-16-le')) > 48000:
        raise ValueError('fixed group exceeds Windows argv budget; split Cargo targets explicitly')
    return args


def enumerate_rust(document, ignored=False, root=None):
    root = ROOT if root is None else Path(root).resolve()
    active, excluded = {}, []
    for binary, suite in document['rust-suites'].items():
        if suite.get('status') == 'skipped':
            continue
        if suite.get('cwd') and not Path(suite['cwd']).resolve().is_relative_to(root):
            raise ValueError('native test cwd escapes selected checkout')
        for name, case in suite['testcases'].items():
            if case['filter-match']['status'] != 'matches':
                continue
            row = {'binary': binary, 'test': name, 'ignored': case['ignored']}
            if bool(case['ignored']) != bool(ignored):
                excluded.append(row)
                continue
            event_name = suite['package-name'] + '::' + suite['binary-name'] + '$' + name
            if event_name in active:
                raise ValueError('duplicate actual nextest identity')
            active[event_name] = row
    return active, excluded


def terminal_results(raw, expected, allowed_skips=()):
    actual = {}
    skipped = []
    running = set()
    max_parallel = 0
    overlaps = []
    for line in raw.decode('utf-8', errors='replace').splitlines():
        try:
            row = json.loads(line)
        except ValueError:
            continue
        if row.get('type') != 'test' or row.get('event') not in ('started', 'ok', 'failed', 'ignored'):
            continue
        name = row['name']
        if row['event'] == 'started':
            # libtest compatibility includes synthetic started -> ignored
            # pairs for tests filtered out before process execution.
            if name in allowed_skips and name not in expected:
                continue
            if name not in expected or name in running or name in actual:
                raise ValueError('unexpected/duplicate test start: ' + name)
            if running and len(overlaps) < 8:
                overlaps.append([sorted(running)[0], name])
            running.add(name)
            max_parallel = max(max_parallel, len(running))
            continue
        if row['event'] == 'ignored' and name in allowed_skips and name not in expected:
            if name in skipped:
                raise ValueError('duplicate unselected terminal: ' + name)
            skipped.append(name)
            continue
        if name not in expected or name in actual:
            raise ValueError('extra/duplicate terminal test result: ' + name)
        actual[name] = row
        running.discard(name)
    missing = set(expected) - actual.keys()
    errors = []
    if missing:
        errors.append('missing actual test terminals: ' + ', '.join(sorted(missing)))
    if any(r['event'] != 'ok' for r in actual.values()):
        errors.append('failed or unexpectedly skipped test')
    return {'results': actual, 'not_run': sorted(missing), 'native_unselected': skipped,
            'max_parallel_tests_observed': max_parallel, 'overlapping_tests': overlaps, 'errors': errors}


def invoke(argv, cwd, env, output, label, timeout=1800, preparation=False):
    result = capture(argv, cwd=cwd, env=env, timeout=timeout,
                     compiler_auxiliary_cleanup=preparation)
    (output / (label + '.stdout')).write_bytes(result.stdout)
    (output / (label + '.stderr')).write_bytes(result.stderr)
    record = {'argv': [str(a) for a in argv], 'cwd': str(cwd), 'exit': result.returncode,
              'meter': result.measurement, 'stdout': label + '.stdout', 'stderr': label + '.stderr',
              'effective_env_sha256': hashlib.sha256(json.dumps(sorted(env.items())).encode()).hexdigest()}
    write(output / (label + '.command.json'), record)
    if result.returncode:
        sys.stderr.write(result.stderr.decode('utf-8', errors='replace')[-5000:])
    return result, record


def source_bindings(document, expected, output):
    """Current selected wrappers' original input/check contract, shared by admission."""
    bound = [dict(r, recipe='language') for r in expected.values()]
    wrapper_names = ('tests::vm_file_tests::', 'tests::a2r_tests::', 'tests::a2c_tests::',
                     'tests::a2ts_tests::', 'tests::cookbook_vm_tests::')
    needs_bindings = any(r['binary'] == 'auto-lang' and r['test'].startswith(wrapper_names) for r in bound)
    if needs_bindings:
        from input_inventory import discover_inputs, attach_inputs, verify_inputs
        inventories = output / 'inputs'; inventories.mkdir(parents=True)
        records = discover_inputs(inventories, document)
        binding_set = {'obligations': bound}
        attach_inputs(binding_set, {'inputs': records}, inventories)
        # PLAN-1066 T-08: the document-identity cross-check is an ACTIVE gate
        # (it used to be an uncalled helper that re-derived oracles from the
        # retired sidecars).
        verify_inputs(binding_set)
        bound = binding_set['obligations']
    return bound


def run_rust(group, output, env, ignored=False, discover_only=False, archive=None):
    env = dict(env)
    env.pop('CARGO_TARGET_DIR', None)
    cwd = ROOT / group['workspace']
    if archive is not None:
        listing_argv = ['cargo', 'nextest', 'list', '--archive-file', archive['path'],
                        '--workspace-remap', str(cwd), '--ignore-default-filter',
                        '-E', group['expression'], '--run-ignored', 'all', '--message-format', 'json']
    else:
        args = nextest_args(group, ignored)
        # One real enumeration per compatible Cargo configuration, not per leaf.
        listing_args = args[:-2] if ignored else args
        listing_argv = ['cargo', 'nextest', 'list', *listing_args, '--run-ignored', 'all', '--message-format', 'json']
    listing, prep = invoke(listing_argv, cwd, env, output, 'discovery', preparation=True)
    if listing.returncode:
        raise ValueError('selected Cargo target discovery/build failed')
    document = json.loads(listing.stdout)
    expected, excluded = enumerate_rust(document, ignored)
    if group['domain'] == 'legacy':
        ace_prefixes = ('test_cases::ac', 'test_cases::abi::', 'test_cases::sema::', 'test_cases::ace_',
                        'tests::ac_', 'tests::ace_', 'tests::av_', 'tests::probe_adapter', 'tests::a2r_hir_892')
        if any(row['test'].startswith(ace_prefixes) for row in expected.values()):
            raise ValueError('ACE host tests leaked into a legacy group; fix feature/target ownership before execution')
    if discover_only:
        return {'outcome': 'DISCOVERED', 'executed': 0, 'selected': list(expected.values()),
                'excluded': excluded, 'preparation': prep}
    if not expected:
        if excluded or (group['expressions'] == ['all()'] and not any(s.get('testcases') for s in document['rust-suites'].values())):
            return {'outcome': 'NOT_APPLICABLE', 'executed': 0, 'excluded': excluded, 'preparation': prep}
        raise ValueError('fixed group selected zero actual tests')
    # Preserve original wrapper child identities using their existing read-only
    # inventories, only when selected tests actually consume those wrappers.
    bound = source_bindings(document, expected, output)
    trace = output / 'trace'; trace.mkdir()
    trace_id = 'fixed-invocation'
    env = {**env, 'AUTO_TEST_TRACE_DIR': str(trace.resolve()), 'AUTO_TEST_TRACE_MODULE': trace_id}
    env = {**env, 'NEXTEST_EXPERIMENTAL_LIBTEST_JSON': '1'}
    if archive is not None:
        # Prepared run: no build inputs are supplied to nextest, only the
        # source-bound archive plus the run-time expression.
        argv = ['cargo', 'nextest', 'run', '--archive-file', archive['path'],
                '--workspace-remap', str(cwd), '--archive-format', 'tar-zst',
                '--ignore-default-filter', '-E', group['expression'],
                '--no-fail-fast', '--no-tests', 'fail',
                '--message-format', 'libtest-json-plus', '--message-format-version', '0.1']
        if ignored:
            argv += ['--run-ignored', 'only']
    else:
        argv = ['cargo', 'nextest', 'run', *args, '--no-fail-fast', '--no-tests', 'fail',
                '--message-format', 'libtest-json-plus', '--message-format-version', '0.1']
    # Native resources/nextest isolation remain in the original configuration.
    result, execution = invoke(argv, cwd, env, output, 'run', timeout=group.get('timeout_seconds') or (14400 if os.environ.get('T3_MILESTONE') == '1' else 1800))
    known = {suite['package-name'] + '::' + suite['binary-name'] + '$' + name
             for suite in document['rust-suites'].values() for name in suite.get('testcases', {})}
    terminals = terminal_results(result.stdout, expected, known - expected.keys())
    from execution_trace import collect, validate_bindings
    children = collect(trace, trace_id)
    child_validation = validate_bindings(bound, children)
    if not child_validation['valid']:
        terminals['errors'].extend(child_validation['errors'])
    return {'outcome': 'PASS' if result.returncode == 0 and not terminals['errors'] else 'FAIL',
            'executed': len(terminals['results']), 'selected': list(expected.values()),
            'excluded': excluded, 'preparation': prep, 'execution': execution,
            'bound_inputs': bound, 'child_observations': children, 'child_validation': child_validation, **terminals}


def run_python(group, output, env, discover_only=False):
    path = ROOT / group['file']
    if not path.resolve().is_relative_to(ROOT):
        raise ValueError('Python target escapes repository')
    base = [sys.executable, str(ROOT / 'scripts/test-governance/python_tests.py'), '--file', str(path)]
    listing, _ = invoke([*base, '--list'], ROOT, env, output, 'discovery')
    if listing.returncode:
        raise ValueError('Python target discovery failed')
    declared = json.loads(listing.stdout)['tests']
    if discover_only:
        return {'outcome': 'DISCOVERED', 'executed': 0, 'selected': declared}
    active = [r['id'] for r in declared if not r['ignored']]
    if not active:
        return {'outcome': 'NOT_APPLICABLE', 'executed': 0, 'excluded': declared}
    argv = [*base]
    for name in active:
        argv += ['--case', name]
    # Windows argv relief: hand the exact case list via a file when the
    # inline form would cross the OS command-line limit (PLAN-1000: the
    # arch acceptance suite outgrew the 32K ceiling).
    import subprocess as _sp
    if os.name == 'nt' and len(_sp.list2cmdline(argv).encode('utf-16-le')) > 30000:
        case_file = output / 'python-cases.txt'
        case_file.write_text(chr(10).join(active), encoding='utf-8')
        argv = [*base, '--case-file', str(case_file)]
    result, execution = invoke(argv, ROOT, env, output, 'run')
    reports = [json.loads(line) for line in result.stdout.decode().splitlines() if line.startswith('{')]
    if len(reports) != 1 or reports[0].get('selected') != active or reports[0].get('executed') != len(active):
        raise ValueError('Python execution incomplete or missing original result')
    report = reports[0]
    valid = result.returncode == 0 and not any(report[k] for k in ('skipped', 'failures', 'errors'))
    return {'outcome': 'PASS' if valid else 'FAIL', 'execution': execution, 'executed': len(active), 'report': report}


def run_case(group, output, env, prepared, discover_only=False, source_state=None):
    from prepare_tools import prepare, case_tools
    from case_artifacts import create_runtime, archive_runtime
    extra, _ = prepare(ROOT, ['case-parent'], output / 'tools', prepared, source_state)
    env = {**env, **extra}
    binary = str(ROOT / 'target/debug' / ('auto-case.exe' if os.name == 'nt' else 'auto-case'))
    suite = str(ROOT / group['suite'])
    inventory, _ = invoke([binary, 'inventory', '--suite', suite], ROOT, env, output, 'inventory')
    if inventory.returncode:
        raise ValueError('TestCase inventory failed')
    inventory_document = json.loads(inventory.stdout)
    if inventory_document.get('schema') != 'test-case-inventory/1' or not inventory_document.get('checks'):
        raise ValueError('unsupported or empty TestCase inventory')
    checks = inventory_document['checks']
    # PLAN-1078: an optional per-group `check` selector narrows the view to
    # the named check ids (e.g. the public corpus AV view runs `--check av`
    # only; legacy-reference rows stay with the explicit avm comparison).
    check_filter = group.get('check')
    if check_filter:
        checks = [c for c in checks if c['check'] == check_filter]
    if discover_only:
        return {'outcome': 'DISCOVERED', 'executed': 0, 'selected': checks}
    if not any(row['state'] == 'RUN' for row in checks):
        return {'outcome': 'NOT_APPLICABLE', 'executed': 0, 'excluded': checks}
    tools = case_tools([{'adapter': r['adapter'], 'case_recipe': r['recipe']} for r in checks])
    if group['domain'] != 'ace' and set(tools) - {'case-parent'}:
        raise ValueError('legacy suite requested ACE tools; correct its explicit group')
    extra, receipts = prepare(ROOT, tools, output / 'tools', prepared, source_state)
    env.update(extra)
    runtime = create_runtime()
    run_argv = [binary, 'run', '--suite', suite, '--out', str(runtime.path)]
    if check_filter:
        run_argv += ['--check', check_filter]
    result, execution = invoke(run_argv, ROOT, env, output, 'run')
    report = json.loads((runtime.path / 'report.json').read_text(encoding='utf-8'))
    overrides = json.loads((ROOT / 'scripts/test-governance/case-control-oracles.json').read_text())['overrides']
    expectations = {(c['case'], c['check']): 'PASS' for c in checks if c['state'] == 'RUN'}
    for row in overrides:
        if any(c['suite'] == row['suite'] for c in checks) and (row['case'], row['check']) in expectations:
            expectations[(row['case'], row['check'])] = row['expected_status']
    actual = {(r['case'], r['check']): r for r in report['results'] if r['status'] != 'NOT_RUN'}
    selected_report_rows = [r for r in report['results'] if r['status'] != 'NOT_RUN']
    valid = len(actual) == len(selected_report_rows) and actual.keys() == expectations.keys()
    valid = valid and not report['load_errors'] and not report['coverage_gaps']
    for key, status in expectations.items():
        row = actual.get(key, {})
        valid = valid and row.get('status') == status and (status != 'FAIL' or bool(row.get('mismatches')))
        original = next(c for c in checks if (c['case'], c['check']) == key)
        valid = valid and row.get('identity', {}).get('source') == original['source_sha256']
        valid = valid and row.get('identity', {}).get('oracle') == original['oracle_sha256']
    expected_exit = 1 if 'FAIL' in expectations.values() else 0
    if check_filter:
        # Explicit per-check selection: the suite report's exit code is 1
        # whenever NOT_RUN rows exist (the F-04 partial-run gate), so the
        # view gate judges only the selected rows' status — a selected FAIL
        # is still red through the per-row check above.
        valid = valid and not any(row.get('status') == 'FAIL' for row in actual.values())
    else:
        valid = valid and result.returncode == expected_exit
    artifacts = archive_runtime(runtime, output, 0)
    artifacts['report'] = str(Path(artifacts['root']) / 'report.json')
    return {'outcome': 'PASS' if valid else 'FAIL', 'execution': execution,
            'executed': len(actual), 'checks': report['results'], 'artifacts': artifacts, 'tools': receipts}


def checkout_state():
    """Run provenance and a write-during-test guard, not a discovery/cache identity."""
    scope = ['crates', 'test', 'experimental', 'scripts', '.cargo', '.config', 'Cargo.toml', 'Cargo.lock']
    head = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD']).strip()
    diff = subprocess.check_output(['git', '-C', str(ROOT), 'diff', 'HEAD', '--binary', '--', *scope])
    paths = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '--others', '--exclude-standard', '--', *scope]).decode().splitlines()
    pending = [(p, hashlib.sha256((ROOT / p).read_bytes()).hexdigest()) for p in paths if (ROOT / p).is_file()]
    return hashlib.sha256(head + diff + json.dumps(pending).encode()).hexdigest()


@contextlib.contextmanager
def execution_lease():
    active = ROOT / 'target/test-groups/active'
    # PLAN-1014 T-05（B-07，D-1001-R3）：获取=原子独占 mkdir；遇存量租约先验
    # 证归属/身份，仅本树+确认死亡才在互斥重验下恢复；活/未知/外树一律拒绝。
    while True:
        try:
            active.mkdir(parents=True, exist_ok=False)
            break
        except FileExistsError:
            _reclaim_dead_lease(active)
    try:
        (active / 'owner.json').write_text(json.dumps({'pid': os.getpid(), 'root': str(ROOT)}))
    except BaseException:
        with contextlib.suppress(OSError):
            active.rmdir()
        raise
    try:
        yield
    finally:
        try:
            (active / 'owner.json').unlink()
            active.rmdir()
        except OSError as error:
            raise ValueError(f'fixed-test lease cleanup failed for {active}: {error}') from error


def _lease_owner(path):
    """Read a lease owner document; anything unreadable fails closed."""
    try:
        data = json.loads(path.read_text(encoding='utf-8'))
        return int(data['pid']), str(data['root'])
    except (ValueError, TypeError, KeyError, OSError) as error:
        raise ValueError(
            f'existing execution lease is unreadable/invalid (fail closed; verify no '
            f'runner is active, then remove manually): {path}') from error


def _reclaim_dead_lease(active):
    """Validate an existing lease and recover it only when it belongs to this
    tree and its owner pid is confirmed dead (mutual exclusion via the atomic
    rename itself, re-validated inside the quarantine)."""
    from build_storage import process_alive
    pid, root = _lease_owner(active / 'owner.json')
    if Path(root).resolve() != ROOT:
        raise ValueError(
            f'existing execution lease belongs to another tree ({root}); refusing to touch it')
    if process_alive(pid):
        raise ValueError(
            f'fixed-test execution lease held by live pid {pid} (root {root}); not reclaiming')
    entries = sorted(entry.name for entry in active.iterdir())
    if entries != ['owner.json']:
        raise ValueError(
            f'existing execution lease carries unknown content {entries}; refusing to recover')
    quarantine = active.with_name(f'active.dead-{pid}')
    if quarantine.exists():
        raise ValueError(
            f'dead-lease quarantine already exists; inspect and remove manually: {quarantine}')
    try:
        os.rename(active, quarantine)
    except FileNotFoundError:
        return  # a concurrent session reclaimed it first; retry the mkdir
    try:
        # A racing new owner may have taken the directory between the read and
        # the rename; restore it untouched and let the retry loop re-validate.
        reclaimed_pid, reclaimed_root = _lease_owner(quarantine / 'owner.json')
        if (reclaimed_pid, Path(reclaimed_root).resolve()) != (pid, ROOT):
            os.rename(quarantine, active)
            return
    except BaseException:
        with contextlib.suppress(OSError):
            os.rename(quarantine, active)
        raise
    import shutil
    shutil.rmtree(quarantine)


PREPARED_SCHEMA = 'fixed-test-prepared/1'
PREPARED_ROOT = ROOT / 'target/test-groups/prepared'


def find_matching_manifest(names):
    """Newest verified in-tree preparation that covers every requested ACE group."""
    if not PREPARED_ROOT.is_dir():
        return None
    wanted = set(names)
    candidates = sorted(PREPARED_ROOT.glob('*/prepared.json'),
                        key=lambda path: path.stat().st_mtime, reverse=True)
    for path in candidates:
        try:
            manifest = json.loads(path.read_text(encoding='utf-8'))
            verify_prepared(manifest)
        except (ValueError, OSError):
            continue
        covered = {name for entry in manifest.get('invocations', []) for name in entry.get('groups', [])}
        if wanted <= covered:
            return path
    return None


def archive_args(group):
    """Group args with run-only options removed (archive builds, it never runs)."""
    out, skip = [], False
    for value in group['args']:
        if skip:
            skip = False
            continue
        if value == '-E':
            skip = True
            continue
        out.append(value)
    return out


def build_archive(group, output, env, index):
    cwd = ROOT / group['workspace']
    target = (output / ('archive-%d.tar.zst' % index)).resolve()
    result, record = invoke(['cargo', 'nextest', 'archive', *archive_args(group),
                             '--archive-file', str(target)],
                            cwd, env, output, 'archive-%d' % index, preparation=True)
    if result.returncode:
        raise ValueError('selected Cargo target archive preparation failed')
    return {'path': str(target), 'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
            'bytes': target.stat().st_size, 'argv': record['argv']}


def prepare_plan(plan, output, summary, args):
    """Explicit preparation: build every selected configuration once, run nothing."""
    from prepare_tools import prepare, tool_identity, toolchain_marker
    before = checkout_state()
    summary['source_state'] = before
    output.mkdir(parents=True, exist_ok=True)
    write(output / 'selection.json', summary)
    tool_document = json.loads((ROOT / 'scripts/test-governance/toolchains.json').read_text(encoding='utf-8'))
    tools = tool_document['tools']
    prepared = set()
    manifest = {'schema': PREPARED_SCHEMA, 'driver': 'scripts/fixed-tests.py',
                'config_sha256': summary['config_sha256'], 'source_state': before,
                'toolchain': toolchain_marker(), 'created_at': time.strftime('%Y-%m-%dT%H:%M:%S'),
                'invocations': []}
    started = time.monotonic()
    entries = []
    # Phase 1 — archives. `cargo nextest archive` builds with nextest's `test`
    # profile into the same target dir, which REBUILDS (and rewrites) any
    # binary a tool preparation just produced. Build every archive first, then
    # prepare tools, so tool artifact hashes describe the final bytes.
    for index, group in enumerate(plan):
        destination = output / str(index)
        destination.mkdir(exist_ok=True)
        env = {**os.environ, 'AUTO_TEST_DOMAIN': group['domain']}
        env.pop('CARGO_TARGET_DIR', None)
        entry = {'index': index, 'groups': group['groups'], 'runner': group['runner'],
                 'workspace': group['workspace'], 'domain': group['domain'],
                 'expression': group['expression'],
                 'expressions': group.get('expressions') or [group['expression']],
                 'tools': group['tools'], 'tool_env': {},
                 'file': group.get('file'), 'suite': group.get('suite'),
                 'timeout_seconds': group.get('timeout_seconds')}
        if group['runner'] == 'nextest':
            entry['archive'] = build_archive(group, destination, env, index)
        entries.append(entry)
    # Phase 2 — tools (and their content-bound warm-up), after every archive.
    for entry in entries:
        destination = output / str(entry['index'])
        extra, receipts = prepare(ROOT, entry['tools'], destination / 'tools', prepared, before)
        entry['tool_env'] = extra
        entry['receipts'] = receipts
        entry['tool_identity'] = {name: tool_identity(tools[name], (ROOT / tools[name]['target']).resolve(), before)
                                  for name in entry['tools']}
        manifest['invocations'].append(entry)
        write(destination / 'prepared.json', entry)
    manifest['prepared_seconds'] = time.monotonic() - started
    write(output / 'prepared.json', manifest)
    print('PREPARED: ' + str(len(manifest['invocations'])) + ' invocation(s) -> ' + str(output / 'prepared.json'))
    return 0


def verify_prepared(manifest):
    """Fail closed with a named preparation error; never rebuild or fall back."""
    from prepare_tools import tool_identity, toolchain_marker
    value, _groups = configuration()
    if manifest.get('schema') != PREPARED_SCHEMA:
        raise ValueError('PREPARATION_REQUIRED: unsupported prepared manifest schema')
    if manifest.get('config_sha256') != value['_sha256']:
        raise ValueError('PREPARATION_REQUIRED: fixed configuration changed since preparation')
    current = checkout_state()
    if manifest.get('source_state') != current:
        raise ValueError('PREPARATION_REQUIRED: source/input state changed since preparation')
    if manifest.get('toolchain') != toolchain_marker():
        raise ValueError('PREPARATION_REQUIRED: rustc toolchain changed since preparation')
    tools = json.loads((ROOT / 'scripts/test-governance/toolchains.json').read_text(encoding='utf-8'))['tools']
    for entry in manifest.get('invocations', []):
        identities = entry.get('tool_identity') or {}
        for name in entry.get('tools', []):
            expected = identities.get(name)
            if expected is None:
                raise ValueError('PREPARATION_REQUIRED: missing tool identity for ' + name)
            target = (ROOT / tools[name]['target']).resolve()
            if tool_identity(tools[name], target, current) != expected:
                raise ValueError('PREPARATION_REQUIRED: tool configuration changed: ' + name)
        for row in entry.get('receipts', []):
            if not row.get('path'):
                # warm-up receipts carry no artifact path; the builder's own
                # content-bound stamp validates the library at run time.
                continue
            path = Path(row['path'])
            if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != row['sha256']:
                raise ValueError('PREPARATION_REQUIRED: tool artifact missing or tampered: ' + str(path))
        archive = entry.get('archive')
        if archive:
            path = Path(archive['path'])
            if not path.is_file():
                raise ValueError('PREPARATION_REQUIRED: archive missing: ' + str(path))
            if hashlib.sha256(path.read_bytes()).hexdigest() != archive['sha256']:
                raise ValueError('PREPARATION_REQUIRED: archive changed since preparation: ' + str(path))


def plan_from_manifest(manifest):
    """Rebuild the invocation list from a verified manifest (run-only input)."""
    plan = []
    for entry in manifest['invocations']:
        group = {'groups': entry['groups'], 'runner': entry['runner'],
                 'workspace': entry['workspace'], 'domain': entry['domain'],
                 'expression': entry['expression'],
                 'expressions': entry.get('expressions') or [entry['expression']],
                 'tools': entry.get('tools', []), 'file': entry.get('file'),
                 'suite': entry.get('suite')}
        if entry.get('timeout_seconds'):
            group['timeout_seconds'] = entry['timeout_seconds']
        plan.append(group)
    return plan


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('groups', nargs='*')
    ap.add_argument('--list', action='store_true', help='read fixed names only; never build')
    ap.add_argument('--select-only', '--list-only', action='store_true', help='print static invocation plan; never build')
    ap.add_argument('--discover-only', action='store_true', help='explicit migration audit: build/list selected native targets, execute no bodies')
    ap.add_argument('--out', type=Path)
    ap.add_argument('--ignored', action='store_true', help='explicit ignored-only manual replay')
    ap.add_argument('--test-name', help='optional native test navigation inside the selected groups')
    ap.add_argument('--prepare-only', action='store_true',
                    help='build the selected configurations/tools into a prepared manifest; execute no bodies')
    ap.add_argument('--run-prepared', type=Path,
                    help='execute only, consuming a prepared manifest; never builds (named failure on drift)')
    args = ap.parse_args(argv)
    if args.prepare_only and args.run_prepared:
        ap.error('--prepare-only and --run-prepared are mutually exclusive')
    if args.prepare_only or args.run_prepared:
        if args.select_only or args.discover_only:
            ap.error('--prepare-only/--run-prepared cannot be combined with --select-only/--discover-only')
        if args.run_prepared and not args.out:
            ap.error('--run-prepared requires --out')
    value, groups = configuration()
    if args.list:
        if args.groups:
            ap.error('--list takes no groups')
        for name in value.get('sets', {}):
            print(name + ': ACE常规自动测试（独立workspace及语言私有协议桥接）')
        for name, g in groups.items():
            print(name + ': ' + g['title'])
        return 0
    try:
        owned_target_override()
    except ValueError as error:
        ap.error(str(error))
    selected = select(args.groups, value, groups)
    # 2026-10-31 ruling: bulk ignored-only replay is the aavm discipline's
    # explicit manual entry. PLAN-1000 I10: a NAMED single-test ignored
    # replay (--test-name) is the same explicit-manual-entry discipline for
    # stress legs that live inside other groups — never a bulk run.
    if args.ignored and any(g['name'] != 'aavm' for g in selected) and not args.test_name:
        ap.error('ignored-only replay must explicitly name aavm or a single test (--test-name)')
    plan = invocations(selected, args.test_name)
    summary = {'schema': 'fixed-test-run/1', 'selected_groups': [g['name'] for g in selected],
               'config_sha256': value['_sha256'],
               'invocations': plan, 'results': [], 'gate_passed': False}
    if args.select_only:
        if args.out and any(g['runner'] == 'case' for g in plan):
            ap.error('Case select-only prints to stdout; --out needs authoritative Case admission and is execution-only')
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        if args.out:
            output = admit_output(args.out); output.mkdir(parents=True, exist_ok=False)
            write(output / 'selection.json', summary)
        return 0
    (ROOT / 'target/test-groups').mkdir(parents=True, exist_ok=True)
    if args.prepare_only and not args.out:
        # Canonical per-tree preparation home, so the ordinary entry can find it.
        prepared_home = PREPARED_ROOT / checkout_state()[:16]
        prepared_home.mkdir(parents=True, exist_ok=True)
        output = prepared_home
    else:
        output = admit_output(args.out) if args.out else Path(tempfile.mkdtemp(prefix='run-', dir=ROOT / 'target/test-groups'))
    with execution_lease():
        if args.prepare_only:
            return prepare_plan(plan, output, summary, args)
        prepared_by_index = None
        manifest_path = args.run_prepared
        manifest = None
        if manifest_path is not None:
            if not manifest_path.is_file():
                raise ValueError('PREPARATION_REQUIRED: prepared manifest not found: ' + str(manifest_path))
            manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
            verify_prepared(manifest)
            plan = plan_from_manifest(manifest)
            prepared_by_index = {entry['index']: entry for entry in manifest['invocations']}
            summary['invocations'] = plan
        else:
            # Ordinary ACE entry: only a verified in-tree preparation may build-free run.
            ace_names = [name for group in plan for name in group['groups']
                         if group['domain'] == 'ace']
            # Explicit ignored-only manual replay keeps its own entry discipline.
            if ace_names and not args.discover_only and not args.ignored:
                manifest_path = find_matching_manifest(ace_names)
                if manifest_path is None:
                    raise ValueError(
                        'PREPARATION_REQUIRED: no verified prepared manifest covers ' +
                        ', '.join(ace_names) + '; run: python scripts/fixed-tests.py ' +
                        ' '.join(ace_names) + ' --prepare-only')
                manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
                verify_prepared(manifest)
                by_groups = {tuple(entry['groups']): entry for entry in manifest['invocations']}
                prepared_by_index = {}
                missing = []
                for index, group in enumerate(plan):
                    entry = by_groups.get(tuple(group['groups']))
                    if entry is not None:
                        prepared_by_index[index] = entry
                    elif group['domain'] == 'ace':
                        missing.append(group['groups'])
                if missing:
                    raise ValueError('PREPARATION_REQUIRED: prepared manifest lacks ' +
                                     ', '.join(','.join(names) for names in missing))
        if manifest is not None:
            summary['prepared_from'] = str(manifest_path)
            summary['prepared_source_state'] = manifest['source_state']
            summary['selected_groups'] = [name for entry in manifest['invocations'] for name in entry['groups']]
        return execute_plan(plan, output, summary, args, prepared_by_index)



def execute_plan(plan, output, summary, args, prepared_by_index=None):
    prepared = set()
    before = checkout_state()
    summary['source_state'] = before
    if hashlib.sha256((ROOT / 'scripts/test-groups.toml').read_bytes()).hexdigest() != summary['config_sha256']:
        raise ValueError('fixed configuration changed after selection')
    started = time.monotonic()
    cpu_started = time.process_time()
    suites = sorted({g['suite'] for g in plan if g['runner'] == 'case'})
    if suites:
        diagnostics = Path(tempfile.mkdtemp(prefix='fixed-admission-'))
        summary['output_admission'] = audit_suite_outputs(ROOT, suites, output, diagnostics, prepared)
    output.mkdir(parents=True, exist_ok=True)
    write(output / 'selection.json', summary)
    write(output / 'results.json', summary)
    for index, group in enumerate(plan):
        destination = output / str(index); destination.mkdir()
        env = {**os.environ, 'AUTO_TEST_DOMAIN': group['domain']}
        env.pop('CARGO_TARGET_DIR', None)
        entry = prepared_by_index.get(index) if prepared_by_index is not None else None
        try:
            from prepare_tools import prepare
            if entry is not None:
                # Prepared run: consume the verified tools/archives only.
                extra, receipts = entry.get('tool_env', {}), list(entry.get('receipts', []))
            else:
                extra, receipts = prepare(ROOT, [] if args.discover_only else group['tools'], destination / 'tools', prepared, before)
            env.update(extra)
            print('Running ' + ', '.join(group['groups']), flush=True)
            archive = (entry or {}).get('archive')
            if group['runner'] == 'nextest':
                result = run_rust(group, destination, env, args.ignored, args.discover_only, archive=archive)
            elif group['runner'] == 'python':
                result = run_python(group, destination, env, args.discover_only)
            else:
                result = run_case(group, destination, env, prepared, args.discover_only, before)
            result['tools'] = receipts + result.get('tools', [])
            if entry is not None:
                result['prepared'] = True
        except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
            result = {'outcome': 'FAIL', 'error': str(error)}
        result['groups'] = group['groups']
        write(destination / 'result.json', result)
        summary['results'].append(result)
        write(output / 'results.json', summary)
        print(result['outcome'] + ': ' + ', '.join(group['groups']), flush=True)
    summary['wall_seconds'] = time.monotonic() - started
    summary['discovery_complete'] = args.discover_only and len(summary['results']) == len(plan) and all(r['outcome'] == 'DISCOVERED' for r in summary['results'])
    summary['gate_passed'] = bool(summary['results']) and all(r['outcome'] in ('PASS', 'NOT_APPLICABLE') for r in summary['results']) and any(r['outcome'] == 'PASS' for r in summary['results'])
    if checkout_state() != before:
        summary['gate_passed'] = False
        summary['discovery_complete'] = False
        summary['error'] = 'code/config/inputs changed during execution'
    commands = []
    for path in output.rglob('*.command.json'):
        meter = json.loads(path.read_text()).get('meter')
        if meter:
            commands.append(meter)
    for path in output.rglob('*.meter.json'):
        commands.append(json.loads(path.read_text()))
    admission = summary.get('output_admission')
    if admission:
        for path in Path(admission['diagnostics']).rglob('*meter.json'):
            commands.append(json.loads(path.read_text()))
    summary['resources'] = {
        'measured_descendant_cpu_ms': sum(m['workload_cpu_ms'] for m in commands),
        'runner_cpu_ms': (time.process_time() - cpu_started) * 1000,
        'total_cpu_ms': sum(m['workload_cpu_ms'] for m in commands) + (time.process_time() - cpu_started) * 1000,
        'meter_cpu_ms': sum(m['meter_cpu_ms'] for m in commands),
        'peak_job_bytes': max((m['peak_job_bytes'] for m in commands), default=None),
        'scope': 'distinct sequential native workloads plus runner CPU (meter CPU already inside runner, not added twice); peak is max Job peak and excludes Python runner memory',
    }
    write(output / 'results.json', summary)
    print('Results: ' + str(output / 'results.json'))
    return 0 if summary['gate_passed'] or summary['discovery_complete'] else 1


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (ValueError, KeyError, OSError) as error:
        print('fixed tests: ' + str(error), file=sys.stderr)
        sys.exit(2)
