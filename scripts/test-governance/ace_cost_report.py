#!/usr/bin/env python3
"""PLAN-893/1060 read-only cost attribution and regular-module budget over real
fixed-driver run outputs.

Two read-only modes:

* policy (default, PLAN-893): attributes the frozen 13 formal-policy items over
  `cargo t ... --out <dir>` artifacts (results.json, *.command.json meters,
  libtest-json-plus terminal events) and rejects green claims without them.
* regular (PLAN-1060): judges one complete ordinary ACE module invocation per
  `--run` against the 60 s target / 120 s acceptance budget from the driver's
  full warm wall clock, accepting ordinary tests that the formal policy does not
  cover. Exit 0 = all GREEN/WITHIN_BUDGET, 1 = OVER_BUDGET or functional FAIL,
  2 = missing/stale/ambiguous evidence (never green).
* session (PLAN-1098 T-08): judges several ordinary modules together, keeps the
  per-module warm budget/terminal/STALE rules above, and adds the session-only
  observations - the added/removed native identity diff against `--baseline`
  plus the admission rule that a NEW identity must finish within
  `--new-item-cap-seconds` (default 10 s). No new meter is introduced.

Never starts tests, builds, jobs or meters, and never rewrites run artifacts
(reports go to a fresh --out path only). The only process it may spawn is `git`
for source-identity verification; no build, runner or case process is started.

Phase model (command granularity, mirrors the driver's sequential shape):
  preparation_build_discovery  discovery/inventory.command (cargo build + nextest list / inventory)
  tool_preparation             tools/*.command.json (case runner tool builds)
  requested_work               run.command (the actual workload)
Intra-run phases (parent-side verdict, persist/GC, copy, cleanup) are not
observable from command artifacts and are reported as UNOBSERVED with the
boundary stated — never as zero. Walls of sequential commands compose the
observed window; CPU stays per command; peaks are maxed, never summed.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT = Path(__file__).resolve().parents[2]
POLICY_DEFAULT = Path(__file__).resolve().parent / 'ace-test-policy.json'
CASE_ORACLES_DEFAULT = Path(__file__).resolve().parent / 'case-control-oracles.json'
GROUPS_CONFIG = ROOT / 'scripts/test-groups.toml'
FIXED_TESTS = ROOT / 'scripts/fixed-tests.py'

INTRA_RUN_PHASES = ('parent_side_verdict', 'persist_gc', 'copy', 'cleanup')

REGULAR_EXIT_OK = 0
REGULAR_EXIT_OVER = 1
REGULAR_EXIT_UNVERIFIABLE = 2

_FIXED_MODULE_CACHE = {}


def load_policy_items(document):
    if document.get('schema') != 'ace-test-policy/1':
        raise ValueError('unsupported policy schema: ' + str(document.get('schema')))
    items = document.get('items')
    if not isinstance(items, list) or not items:
        raise ValueError('policy carries no items')
    native = {}
    for item in items:
        key = (item['native']['binary'], item['native']['test'])
        if key in native:
            raise ValueError('duplicate native identity in policy: ' + item['id'])
        native[key] = item
    return native


def load_policy(path):
    document = json.loads(Path(path).read_text(encoding='utf-8'))
    return document, load_policy_items(document)


def load_case_oracles(repo_root=None):
    """TestCase controls' original expected statuses (the driver's own rule).

    Some delivered suites carry one-check meta-oracles whose expected status is
    FAIL; a report that only accepted PASS would call those red. This re-derives
    the driver's acceptance, so the budget verdict never trusts the parent exit
    code but also never invents a failure the contract does not have.
    """
    path = Path(repo_root or ROOT) / 'scripts/test-governance/case-control-oracles.json'
    if not path.is_file():
        return {}
    try:
        document = json.loads(path.read_text(encoding='utf-8'))
    except ValueError:
        return {}
    return {(row.get('case'), row.get('check')): row.get('expected_status', 'PASS')
            for row in document.get('overrides') or []}


def fixed_tests_module(repo_root):
    """The repository's own loader, never a second catalog."""
    root = Path(repo_root).resolve()
    module = _FIXED_MODULE_CACHE.get(str(root))
    if module is not None:
        return module
    spec = importlib.util.spec_from_file_location('fixed_tests_local', root / 'scripts/fixed-tests.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    _FIXED_MODULE_CACHE[str(root)] = module
    return module


def frozen_invocation(repo_root, group_name):
    """Current fixed configuration's invocation for one group: expression/runner.

    Returns None when the group is unknown or the configuration cannot be read;
    callers must then keep the weaker recorded-evidence checks instead of
    inventing a selection.
    """
    try:
        module = fixed_tests_module(repo_root)
        _, groups = module.configuration()
        if group_name not in groups:
            return None
        selected = module.select([group_name], {'default': [group_name], 'sets': {}}, groups)
        merged = module.invocations(selected)
        if len(merged) != 1:
            return None
        return {'expression': merged[0]['expression'], 'runner': merged[0]['runner'],
                'workspace': merged[0]['workspace'], 'args': merged[0]['args']}
    except Exception:  # noqa: BLE001 - caller reports the weaker boundary
        return None


def read_commands(index_dir):
    """Command records in this invocation index, keyed by phase kind."""
    commands = {'discovery': [], 'run': [], 'tools': []}
    for path in sorted(index_dir.glob('*.command.json')):
        record = json.loads(path.read_text(encoding='utf-8'))
        if path.name in ('discovery.command.json', 'inventory.command.json'):
            commands['discovery'].append(record)
        elif path.name == 'run.command.json':
            commands['run'].append(record)
        else:
            commands['tools'].append(record)
    tools_dir = index_dir / 'tools'
    if tools_dir.is_dir():
        for path in sorted(tools_dir.glob('**/*.command.json')):
            commands['tools'].append(json.loads(path.read_text(encoding='utf-8')))
        # prepare_tools records a bare measurement next to each tool build log
        # (<tool>-build.meter.json); count it as tool preparation, never as zero.
        for path in sorted(tools_dir.glob('**/*.meter.json')):
            measurement = json.loads(path.read_text(encoding='utf-8'))
            commands['tools'].append({'argv': [path.name], 'cwd': str(path.parent), 'exit': 0,
                                      'meter': measurement})
    return commands


def event_prefix_map(index_dir):
    """Selected binary id -> the real nextest event prefix (package::binary)."""
    path = Path(index_dir) / 'discovery.stdout'
    if not path.is_file():
        return {}
    try:
        document = json.loads(path.read_text(encoding='utf-8'))
    except ValueError:
        return {}
    mapping = {}
    for key, suite in (document.get('rust-suites') or {}).items():
        package, binary = suite.get('package-name'), suite.get('binary-name')
        if package and binary:
            mapping[key] = package + '::' + binary
    return mapping


def terminal_events(stdout_path):
    """Libtest-json-plus terminal events (ok/failed/ignored) with exec_time."""
    events = []
    if not stdout_path.is_file():
        return events
    for line in stdout_path.read_text(encoding='utf-8', errors='replace').splitlines():
        try:
            row = json.loads(line)
        except ValueError:
            continue
        if row.get('type') == 'test' and row.get('event') in ('ok', 'failed', 'ignored'):
            events.append(row)
    return events


def meter_of(record):
    meter = record.get('meter') or {}
    return {
        'wall_ms': meter.get('wall_ms'),
        'workload_cpu_ms': meter.get('workload_cpu_ms'),
        'meter_cpu_ms': meter.get('meter_cpu_ms'),
        'peak_job_bytes': meter.get('peak_job_bytes'),
        'job_processes': meter.get('job_processes'),
        'exit': record.get('exit'),
        'error': meter.get('error') or record.get('error'),
    }


def phase_view(commands):
    """Attribute observed command meters to phase buckets; UNOBSERVED stays named."""
    phases = {}
    roles = (('discovery', 'preparation_build_discovery'),
             ('run', 'requested_work'),
             ('tools', 'tool_preparation'))
    for kind, name in roles:
        meters = [meter_of(record) for record in commands[kind]]
        phases[name] = {
            'status': 'OBSERVED' if meters else 'UNOBSERVED',
            'command_count': len(meters),
            'wall_ms': sum(m['wall_ms'] or 0 for m in meters) if meters else None,
            'workload_cpu_ms': sum(m['workload_cpu_ms'] or 0 for m in meters) if meters else None,
            'peak_job_bytes': max((m['peak_job_bytes'] for m in meters if m['peak_job_bytes'] is not None), default=None),
            'exits': [m['exit'] for m in meters],
        }
    for phase in INTRA_RUN_PHASES:
        phases[phase] = {'status': 'UNOBSERVED',
                         'boundary': 'inside the run.command job tree; no command-granularity marker exists'}
    return phases


def verify_run(result, commands, events):
    """Verdict integrity: a green claim must be backed by real execution artifacts.

    Rust/nextest rows need a terminal event per executed test; case rows lean
    on the driver's TestCase admission (their authority is the case report),
    so only the run artifact and executed count are required here.
    """
    problems = []
    outcome = result.get('outcome')
    selected = result.get('selected') or []
    errors = result.get('errors') or []
    executed = result.get('executed')
    is_case_row = 'checks' in result or 'artifacts' in result
    if errors:
        problems.append('driver errors: ' + '; '.join(map(str, errors)))
    if outcome == 'PASS':
        if not commands['run']:
            problems.append('PASS without run.command.json (no real execution artifact)')
        elif not selected and not is_case_row:
            problems.append('PASS with zero selected tests')
        elif not executed:
            problems.append('PASS with executed=0 (fake pass)')
        elif not is_case_row:
            ok_events = [e for e in events if e.get('event') == 'ok']
            if len(ok_events) < executed:
                problems.append('PASS without a terminal ok event per executed test (%d events / %d executed)'
                                % (len(ok_events), executed))
    if outcome not in ('PASS', 'NOT_APPLICABLE', 'DISCOVERED'):
        problems.append('driver outcome ' + str(outcome))
    for record in commands['run']:
        meter = meter_of(record)
        if meter['exit'] == 124 or meter['error']:
            problems.append('canceled/timed-out workload: exit=%s error=%s' % (meter['exit'], meter['error']))
    return problems


def staleness(summary, repo_root):
    """A run recorded on a different checkout state/config is STALE, not reusable."""
    flags = []
    root = Path(repo_root)
    try:
        current = fixed_tests_module(root).checkout_state()
    except Exception as error:  # noqa: BLE001 - surfaced, never swallowed
        return ['checkout state unavailable: %s' % error]
    if summary.get('source_state') and summary['source_state'] != current:
        flags.append('source_state differs from current checkout (run recorded on another tree state)')
    config_hash = summary.get('config_sha256')
    if config_hash:
        actual = hashlib.sha256((root / 'scripts/test-groups.toml').read_bytes()).hexdigest()
        if actual != config_hash:
            flags.append('test-groups.toml changed since the run (config_sha256 mismatch)')
    return flags


def attribute_run(run_dir, policy_items):
    run_dir = Path(run_dir)
    summary = json.loads((run_dir / 'results.json').read_text(encoding='utf-8'))
    index_dirs = sorted(p for p in run_dir.iterdir() if p.is_dir() and p.name.isdigit())
    indexes, problems = [], []
    for result, index_dir in zip(summary.get('results', []), index_dirs):
        commands = read_commands(index_dir)
        events = terminal_events(index_dir / 'run.stdout')
        problems.extend(verify_run(result, commands, events))
        for row in result.get('selected') or []:
            key = (row['binary'], row['test'])
            if key not in policy_items:
                problems.append('selected test not covered by policy: %s$%s' % key)
        # a selected-but-unmatched policy item on a slow-list run is not an error:
        # the report only claims attribution for what the policy freezes.
        indexes.append({'result': {k: result.get(k) for k in ('outcome', 'executed', 'groups', 'tools')},
                        'commands': commands, 'events': events})
    merged = {kind: [r for index in indexes for r in index['commands'][kind]]
              for kind in ('discovery', 'run', 'tools')}
    attribution = {
        'run_dir': str(run_dir),
        'selected_groups': summary.get('selected_groups'),
        'wall_seconds_driver': summary.get('wall_seconds'),
        'phases': phase_view(merged),
        'tests': {},
        'resources_driver': summary.get('resources'),
    }
    # Case rows record tool preparation as receipts (build_seconds/exit) inside
    # the result, not as full job command meters — surface them honestly.
    receipts = [r for index in indexes for r in (index['result'].get('tools') or [])]
    if receipts:
        attribution['phases']['tool_preparation'] = {
            'status': 'OBSERVED-RECEIPTS',
            'command_count': len(receipts),
            'wall_ms': sum(r.get('build_seconds') or 0 for r in receipts) * 1000.0,
            'exits': [r.get('exit') for r in receipts],
            'boundary': 'receipts carry build_seconds/exit only, not job-tree CPU/peak meters',
        }
    for index in indexes:
        for event in index['events']:
            attribution['tests'].setdefault(event['name'], {})[event['event']] = {
                'exec_time_ms': event['exec_time'] * 1000.0 if 'exec_time' in event else None}
    attribution['policy_matches'] = {}
    for item in policy_items.values():
        event_key = item['native']['binary'] + '$' + item['native']['test']
        if event_key in attribution['tests']:
            attribution['policy_matches'][item['id']] = {
                'native': event_key, 'terminals': attribution['tests'][event_key]}
    attribution['problems'] = problems
    return attribution


def render_markdown(attributions, stale_flags_per_run):
    lines = ['# ACE cost attribution (read-only)', '']
    for attribution, stale in zip(attributions, stale_flags_per_run):
        lines.append('## Run `%s`' % attribution['run_dir'])
        lines.append('')
        lines.append('- groups: %s' % ', '.join(attribution['selected_groups'] or []))
        lines.append('- driver wall: %ss' % attribution['wall_seconds_driver'])
        for name, phase in attribution['phases'].items():
            if phase.get('status') == 'UNOBSERVED' and 'boundary' in phase:
                lines.append('- phase **%s**: UNOBSERVED (%s)' % (name, phase['boundary']))
            elif phase.get('status') == 'UNOBSERVED':
                lines.append('- phase **%s**: UNOBSERVED (no such command in this run)' % name)
            elif phase.get('status') == 'OBSERVED-RECEIPTS':
                lines.append('- phase **%s**: wall=%sms exits=%s (%s)' % (
                    name, phase['wall_ms'], phase['exits'], phase['boundary']))
            else:
                lines.append('- phase **%s**: wall=%sms cpu=%sms peak=%sB exits=%s' % (
                    name, phase['wall_ms'], phase['workload_cpu_ms'], phase['peak_job_bytes'], phase['exits']))
        for name, terminals in sorted(attribution['tests'].items()):
            times = {k: v['exec_time_ms'] for k, v in terminals.items()}
            lines.append('- test `%s`: %s' % (name, times))
        for kid, item in sorted(attribution['policy_matches'].items()):
            lines.append('- item %s: %s terminals=%s' % (kid, item['native'], sorted(item['terminals'])))
        if stale:
            lines.append('- STALE: %s' % '; '.join(stale))
        if attribution['problems']:
            lines.append('- PROBLEMS (verdict cannot be green):')
            lines.extend('  - %s' % p for p in attribution['problems'])
        else:
            lines.append('- no verdict problems')
        lines.append('')
    return '\n'.join(lines) + '\n'


# --------------------------------------------------------------------------- #
# PLAN-1060 ordinary-module budget mode (read-only)
# --------------------------------------------------------------------------- #

def _numeric_run_dirs(run_dir):
    return sorted((p for p in Path(run_dir).iterdir() if p.is_dir() and p.name.isdigit()),
                  key=lambda p: int(p.name))


def regular_module(run_dir, repo_root=None, budget_seconds=120.0, target_seconds=60.0,
                   case_oracles=None):
    """Judge one complete ordinary module invocation against the warm budget.

    Evidence integrity failures are `missing_evidence` (exit 2), functional
    failures are `problems` (exit 1), and only a complete, fresh, passing
    single-group invocation gets a budget status. `case_oracles` re-derives the
    TestCase controls' expected statuses (the driver's own acceptance rule)
    instead of blindly trusting the parent exit code.
    """
    run_dir = Path(run_dir)
    repo_root = Path(repo_root) if repo_root is not None else ROOT
    if case_oracles is None:
        case_oracles = load_case_oracles(repo_root)
    record = {
        'run_dir': str(run_dir),
        'module': None,
        'status': 'UNVERIFIABLE',
        'budget_seconds': budget_seconds,
        'target_seconds': target_seconds,
        'warm_wall_seconds': None,
        'identity': {},
        'selection': {},
        'phases': {},
        'tests': [],
        'case_checks': None,
        'stale': [],
        'problems': [],
        'missing_evidence': [],
        'boundary': ('warm wall = the driver invocation wall clock (preparation, discovery/admission, '
                     'execution, parent verdict, copy and cleanup); first cold build belongs in --cold-run'),
    }
    summary_path = run_dir / 'results.json'
    if not summary_path.is_file():
        record['missing_evidence'].append('no results.json in the run directory')
        return record
    summary = json.loads(summary_path.read_text(encoding='utf-8'))
    if summary.get('schema') != 'fixed-test-run/1':
        record['missing_evidence'].append('unsupported run schema: %s' % summary.get('schema'))
        return record
    record['identity'] = {
        'source_state': summary.get('source_state'),
        'config_sha256': summary.get('config_sha256'),
        'wall_seconds': summary.get('wall_seconds'),
        'selected_groups': summary.get('selected_groups'),
        'invocations': [{k: inv.get(k) for k in ('runner', 'workspace', 'expression', 'args', 'groups')}
                        for inv in summary.get('invocations') or []],
    }
    if summary.get('error'):
        record['problems'].append('driver error: %s' % summary['error'])

    groups = summary.get('selected_groups') or []
    if len(groups) != 1:
        record['missing_evidence'].append(
            'a module budget needs exactly one fixed group per invocation (recorded: %s); '
            'a combined invocation wall cannot be copied onto each module' % (groups or 'none'))
        return record
    module = groups[0]
    record['module'] = module

    results = summary.get('results') or []
    index_dirs = _numeric_run_dirs(run_dir)
    if not results:
        record['missing_evidence'].append('no driver results rows')
        return record
    if len(results) != len(index_dirs):
        record['missing_evidence'].append(
            'results rows (%d) != numeric run directories (%d); a zip pairing would truncate modules'
            % (len(results), len(index_dirs)))
        return record
    for position, directory in enumerate(index_dirs):
        if directory.name != str(position):
            record['missing_evidence'].append(
                'run directory %s is not the sequential index %d' % (directory.name, position))
        stored = directory / 'result.json'
        if not stored.is_file():
            record['missing_evidence'].append('run directory %s carries no result.json' % directory.name)
            continue
        if json.loads(stored.read_text(encoding='utf-8')).get('groups') != results[position].get('groups'):
            record['missing_evidence'].append(
                'run directory %s result does not match results.json row %d' % (directory.name, position))
    result = results[0]
    commands = read_commands(index_dirs[0])
    events = terminal_events(index_dirs[0] / 'run.stdout')

    record['identity']['outcome'] = result.get('outcome')
    record['identity']['executed'] = result.get('executed')
    record['selection'] = {
        'selected': len(result.get('selected') or result.get('checks') or []),
        'executed': result.get('executed'),
        'excluded': len(result.get('excluded') or []),
        'groups': result.get('groups'),
    }
    if result.get('errors'):
        record['problems'].extend('driver errors: %s' % error for error in result['errors'])
    outcome = result.get('outcome')
    if outcome == 'NOT_APPLICABLE':
        record['missing_evidence'].append(
            'driver outcome NOT_APPLICABLE: every original condition was excluded, so this is not a module pass')
    elif outcome != 'PASS':
        record['problems'].append('driver outcome %s' % outcome)

    if not commands['run']:
        record['missing_evidence'].append('no run.command.json (no real execution artifact)')
    # A delivered suite whose controls expect FAIL exits 1 by contract; the
    # expected workload exit is derived from those controls, never assumed 0.
    case_checks = result.get('checks') or []
    expected_case_exit = 1 if any(
        case_oracles.get((row.get('case'), row.get('check')), 'PASS') != 'PASS'
        for row in case_checks) else 0
    for command in commands['run']:
        meter = meter_of(command)
        if meter['exit'] == 124:
            record['problems'].append('canceled/timed-out workload: exit=124')
        elif meter['error']:
            record['problems'].append('run command error: %s' % meter['error'])
        elif meter['exit'] not in (0, None, expected_case_exit):
            record['problems'].append('run command exit=%s (expected %s)'
                                      % (meter['exit'], expected_case_exit))

    is_case_row = 'checks' in result or 'artifacts' in result
    executed = result.get('executed') or 0
    if executed <= 0:
        record['missing_evidence'].append('executed=0 (fake pass): no test body actually ran')
    if is_case_row:
        checks = result.get('checks') or []
        # The TestCase runner records a native per-check duration in its own
        # report, referenced by the driver result; read it instead of guessing.
        durations = {}
        report_path = (result.get('artifacts') or {}).get('report')
        if report_path:
            try:
                case_document = json.loads(Path(report_path).read_text(encoding='utf-8'))
            except (OSError, ValueError):
                case_document = None
            for row in (case_document or {}).get('results') or []:
                durations[(row.get('case'), row.get('check'))] = row.get('duration_ms')
        record['case_durations'] = 'OBSERVED' if durations else 'UNOBSERVED'
        record['case_checks'] = [{'case': row.get('case'), 'check': row.get('check'),
                                  'status': row.get('status'),
                                  'duration_ms': durations.get((row.get('case'), row.get('check')))}
                                 for row in checks]
        record['tests'] = sorted(
            ({'name': '%s%%%s' % (row.get('case'), row.get('check')),
              'event': row.get('status'),
              'exec_time_ms': row.get('duration_ms')}
             for row in record['case_checks']),
            key=lambda row: -(row.get('exec_time_ms') or 0.0))
        if not checks:
            record['missing_evidence'].append('case row carries no per-check results')
        if len(checks) != executed:
            record['missing_evidence'].append(
                'case checks (%d) != executed (%d)' % (len(checks), executed))
        failed = []
        for row in checks:
            expected = case_oracles.get((row.get('case'), row.get('check')), 'PASS')
            if row.get('status') != expected:
                failed.append((row, expected))
        record['case_controls'] = len(case_oracles)
        if failed:
            first, expected = failed[0]
            record['problems'].append(
                '%d case check(s) differ from the original control expectation '
                '(first: %s%%%s status=%s expected=%s)'
                % (len(failed), first.get('case'), first.get('check'),
                   first.get('status'), expected))
    else:
        selected = result.get('selected') or []
        if not selected:
            record['missing_evidence'].append('zero selected tests')
        seen = {}
        for event in events:
            seen.setdefault(event['name'], []).append(event.get('event'))
        duplicates = sorted(name for name, kinds in seen.items()
                            if len([kind for kind in kinds if kind != 'ignored']) > 1)
        if duplicates:
            record['missing_evidence'].append(
                'duplicate terminal events for the same native identity: %s' % ', '.join(duplicates[:4]))
        expected = {row['binary'] + '$' + row['test'] for row in selected}
        # nextest records the selected binary id, while its terminal events use
        # <package>::<binary-name>; both forms are legitimate identities here.
        prefixes = event_prefix_map(index_dirs[0])
        observed = set(seen)
        unmatched, matched = [], set()
        for row in selected:
            candidates = {row['binary'] + '$' + row['test'],
                          prefixes.get(row['binary'], row['binary']) + '$' + row['test']}
            hit = candidates & observed
            if not hit:
                unmatched.append(row['binary'] + '$' + row['test'])
                continue
            matched |= hit
            if all(kind == 'ignored' for name in hit for kind in seen[name]):
                record['missing_evidence'].append(
                    'selected test was skipped by the runner: %s' % sorted(hit)[0])
        if unmatched:
            record['missing_evidence'].append(
                'missing actual terminal for %d selected test(s): %s'
                % (len(unmatched), ', '.join(sorted(unmatched)[:4])))
        # `ignored` terminals for identities outside the selection are the
        # runner's legitimate skips (excluded/ignored tests); an ok/failed
        # terminal outside the selection would mean the real set differed.
        extra = sorted(name for name in (observed - matched - expected)
                       if any(kind != 'ignored' for kind in seen[name]))
        if extra:
            record['missing_evidence'].append(
                'terminal events for unselected identities: %s' % ', '.join(extra[:4]))
        record['ignored_outside_selection'] = sorted(
            name for name in (observed - matched - expected)
            if all(kind == 'ignored' for kind in seen[name]))
        ok_events = [event for event in events if event.get('event') == 'ok']
        failed_events = [event for event in events if event.get('event') == 'failed']
        if failed_events:
            record['problems'].append('%d failed test(s) (first: %s)'
                                      % (len(failed_events), failed_events[0]['name']))
        if outcome == 'PASS' and len(ok_events) < executed:
            record['missing_evidence'].append(
                'PASS without one ok terminal per executed test (%d events / %d executed)'
                % (len(ok_events), executed))
        if not result.get('not_run') == [] and result.get('not_run'):
            record['missing_evidence'].append('not_run identities: %s' % ', '.join(result['not_run'][:4]))
        record['tests'] = sorted(
            ({'name': name, 'event': kinds[-1],
              'exec_time_ms': next((event['exec_time'] * 1000.0 for event in events
                                    if event['name'] == name and 'exec_time' in event), None)}
             for name, kinds in seen.items()),
            key=lambda row: -(row['exec_time_ms'] or 0.0))

    record['phases'] = phase_view(commands)
    reused_tools = [row for row in (result.get('tools') or []) if row.get('reused')]
    if reused_tools and not commands['tools']:
        record['phases']['tool_preparation'] = {
            'status': 'REUSED',
            'command_count': len(reused_tools),
            'wall_ms': 0.0,
            'exits': [],
            'boundary': ('identity-bound receipts matched (source state, rustc, tool config, '
                         'artifact hashes); no build was executed'),
        }
    warm = summary.get('wall_seconds')
    if not isinstance(warm, (int, float)) or warm <= 0:
        record['missing_evidence'].append('no usable driver wall_seconds')
        warm = None
    else:
        record['warm_wall_seconds'] = float(warm)
        observed_ms = sum((phase.get('wall_ms') or 0.0) for phase in record['phases'].values()
                          if phase.get('status') in ('OBSERVED', 'OBSERVED-RECEIPTS'))
        if observed_ms > warm * 1000.0 + 1000.0:
            record['missing_evidence'].append(
                'observed command walls (%.0fms) exceed the driver wall (%.0fms): window boundary unclear'
                % (observed_ms, warm * 1000.0))

    # Identity: the completed invocation must be the frozen group, not a
    # --test-name subset of it, and it must be fresh.
    frozen = frozen_invocation(repo_root, module)
    invocations = summary.get('invocations') or []
    recorded = next((inv for inv in invocations if module in (inv.get('groups') or [])), None)
    if recorded is None:
        record['missing_evidence'].append('no invocation record for group %s' % module)
    else:
        record['identity']['recorded_expression'] = recorded.get('expression')
        if summary.get('test_name'):
            record['missing_evidence'].append(
                'invocation used --test-name (%s): a subset cannot certify the complete module'
                % summary['test_name'])
        elif frozen is not None and recorded.get('expression') != frozen['expression']:
            record['missing_evidence'].append(
                'recorded selection differs from the frozen group expression (subset or navigation)')
        elif frozen is None:
            record['missing_evidence'].append(
                'group %s is not in the current fixed configuration; complete selection unverifiable' % module)
        if frozen is not None and recorded.get('runner') != frozen['runner']:
            record['missing_evidence'].append(
                'runner %s != frozen runner %s' % (recorded.get('runner'), frozen['runner']))
    record['stale'] = staleness(summary, repo_root)
    if record['stale']:
        record['missing_evidence'].extend(record['stale'])

    if record['missing_evidence']:
        record['status'] = 'UNVERIFIABLE'
    elif record['problems']:
        record['status'] = 'FAIL'
    elif record['warm_wall_seconds'] is None:
        record['status'] = 'UNVERIFIABLE'
    elif record['warm_wall_seconds'] <= target_seconds:
        record['status'] = 'GREEN'
    elif record['warm_wall_seconds'] <= budget_seconds:
        record['status'] = 'WITHIN_BUDGET'
    else:
        record['status'] = 'OVER_BUDGET'
    return record


def regular_cold(record, cold_dir, repo_root=None):
    """Attach first-invocation cold preparation (T_cold/T_first_total), unbudgeted."""
    cold = regular_module(cold_dir, repo_root=repo_root)
    record['cold'] = {
        'run_dir': cold['run_dir'],
        'module': cold['module'],
        'warm_wall_seconds': cold['warm_wall_seconds'],
        'status': cold['status'],
        'missing_evidence': cold['missing_evidence'],
        'problems': cold['problems'],
        'boundary': ('first cold Cargo/worker build on this tree; excluded from the module budget and '
                     'reported separately, never hidden'),
    }
    if cold['module'] is not None and record.get('module') is not None and cold['module'] != record['module']:
        record['missing_evidence'].append(
            'cold run recorded for %s, warm run for %s' % (cold['module'], record['module']))
    if cold['warm_wall_seconds'] and record.get('warm_wall_seconds'):
        record['first_total_seconds'] = cold['warm_wall_seconds'] + record['warm_wall_seconds']
    if cold['missing_evidence'] or cold['problems']:
        record['missing_evidence'].append('cold preparation record is not a valid run')
    return record


def regular_exit_code(records):
    statuses = {record['status'] for record in records}
    if statuses & {'UNVERIFIABLE', 'NOT_APPLICABLE'}:
        return REGULAR_EXIT_UNVERIFIABLE
    if statuses & {'OVER_BUDGET', 'FAIL'}:
        return REGULAR_EXIT_OVER
    return REGULAR_EXIT_OK


def render_regular_markdown(records):
    lines = ['# ACE ordinary-module budget (read-only)', '',
             'Warm wall = the driver invocation wall clock; target/budget are per complete module.',
             '']
    for record in records:
        lines.append('## `%s` — %s' % (record.get('module') or 'unknown', record['status']))
        lines.append('')
        lines.append('- run: `%s`' % record['run_dir'])
        warm = record.get('warm_wall_seconds')
        lines.append('- T_warm: %s s (target %ss / budget %ss)' % (
            'UNOBSERVED' if warm is None else '%.3f' % warm,
            record['target_seconds'], record['budget_seconds']))
        if record.get('first_total_seconds'):
            lines.append('- T_cold: %s s; T_first_total: %.3f s' % (
                record['cold']['warm_wall_seconds'], record['first_total_seconds']))
        identity = record.get('identity') or {}
        lines.append('- identity: source_state=%s config_sha256=%s outcome=%s executed=%s' % (
            identity.get('source_state'), identity.get('config_sha256'),
            identity.get('outcome'), identity.get('executed')))
        selection = record.get('selection') or {}
        lines.append('- selection: selected=%s executed=%s excluded=%s' % (
            selection.get('selected'), selection.get('executed'), selection.get('excluded')))
        phases = record.get('phases') or {}
        for name in ('preparation_build_discovery', 'tool_preparation', 'requested_work',
                     'parent_side_verdict', 'persist_gc', 'copy', 'cleanup'):
            phase = phases.get(name)
            if not phase:
                continue
            if phase.get('status') == 'UNOBSERVED':
                lines.append('- phase **%s**: UNOBSERVED (%s)' % (name, phase.get('boundary', 'no such command')))
            elif phase.get('status') == 'REUSED':
                lines.append('- phase **%s**: REUSED %d tool receipt(s), no build (%s)'
                             % (name, phase.get('command_count') or 0, phase.get('boundary')))
            else:
                lines.append('- phase **%s**: wall=%sms cpu=%sms peak=%sB exits=%s' % (
                    name, phase.get('wall_ms'), phase.get('workload_cpu_ms'),
                    phase.get('peak_job_bytes'), phase.get('exits')))
        tests = record.get('tests') or []
        if tests:
            lines.append('- slowest tests:')
            for row in tests[:15]:
                lines.append('  - `%s` %s %s' % (row['name'], row['event'],
                                                 'UNOBSERVED' if row['exec_time_ms'] is None
                                                 else '%.1fms' % row['exec_time_ms']))
            if len(tests) > 15:
                lines.append('  - (+%d more)' % (len(tests) - 15))
        checks = record.get('case_checks')
        if checks:
            bad = [row for row in checks if row.get('status') != 'PASS']
            timed = [row for row in checks if row.get('duration_ms') is not None]
            lines.append('- case checks: %d (%d not PASS before the original control overrides; '
                         '%d override(s) honoured); per-check split: %s'
                         % (len(checks), len(bad), record.get('case_controls') or 0,
                            ('%d/%d observed by the runner report' % (len(timed), len(checks)))
                            if timed else 'UNOBSERVED (no per-check duration in the runner report)'))
        if record.get('stale'):
            lines.append('- STALE: %s' % '; '.join(record['stale']))
        if record.get('missing_evidence'):
            lines.append('- MISSING/UNVERIFIABLE (exit 2):')
            lines.extend('  - %s' % item for item in record['missing_evidence'])
        if record.get('problems'):
            lines.append('- PROBLEMS (functional, exit 1):')
            lines.extend('  - %s' % item for item in record['problems'])
        lines.append('- boundary: %s' % record.get('boundary'))
        lines.append('')
    return '\n'.join(lines) + '\n'


SESSION_BASELINE_SCHEMA = 'ace-session-baseline/1'
SESSION_REPORT_SCHEMA = 'ace-session-report/1'


def session_identities(records):
    """Native identity -> observed warm exec time (ms), from judged modules."""
    identities = {}
    for record in records:
        for row in record.get('tests') or []:
            identities[row['name']] = row.get('exec_time_ms')
    return identities


def write_session_baseline(records, path):
    """Persist the current session's identities as the next session's baseline."""
    Path(path).write_text(json.dumps({'schema': SESSION_BASELINE_SCHEMA,
                                      'identities': session_identities(records)},
                                     ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def load_session_baseline(path):
    document = json.loads(Path(path).read_text(encoding='utf-8'))
    if document.get('schema') != SESSION_BASELINE_SCHEMA:
        raise ValueError('unsupported session baseline schema: %s' % document.get('schema'))
    identities = document.get('identities')
    if not isinstance(identities, dict):
        raise ValueError('session baseline carries no identities map')
    return identities


def session_report(run_dirs, repo_root=None, budget_seconds=120.0, target_seconds=60.0,
                   baseline=None, new_item_cap_seconds=10.0, case_oracles=None):
    """Judge several ordinary modules together and diff them against a baseline.

    Session scope adds only what one module cannot see:

    * added/removed native identities versus the previous session, and
    * the admission rule for NEW items: a newly appeared identity must finish
      within `new_item_cap_seconds` (default 10 s); an old identity keeps the
      group budget instead. Terminal events, the per-module warm budget and the
      STALE identity check are reused from `regular_module`, never re-derived.
    """
    run_dirs = [Path(run_dir) for run_dir in run_dirs]
    records = [regular_module(run_dir, repo_root=repo_root, budget_seconds=budget_seconds,
                              target_seconds=target_seconds, case_oracles=case_oracles)
               for run_dir in run_dirs]
    current = session_identities(records)
    added, removed, baseline_error = [], [], None
    if baseline is not None:
        try:
            previous = load_session_baseline(baseline)
        except (OSError, ValueError) as error:
            baseline_error = 'unreadable session baseline: %s' % error
            previous = {}
        added = sorted(set(current) - set(previous))
        removed = sorted(set(previous) - set(current))
    violations = []
    for name in added:
        observed = current.get(name)
        if observed is None:
            violations.append({'identity': name, 'exec_time_ms': None,
                               'reason': 'new identity has no observed exec time'})
        elif observed > new_item_cap_seconds * 1000.0:
            violations.append({'identity': name, 'exec_time_ms': observed,
                               'reason': 'new identity exceeds %.0fs' % new_item_cap_seconds})
    unverifiable = [record['module'] for record in records if record['status'] == 'UNVERIFIABLE']
    over = [record['module'] for record in records
            if record['status'] in ('OVER_BUDGET', 'FAIL')]
    if baseline_error or unverifiable:
        status = 'UNVERIFIABLE'
    elif over or violations:
        status = 'OVER_BUDGET'
    else:
        status = 'GREEN'
    return {'schema': SESSION_REPORT_SCHEMA, 'mode': 'session', 'status': status,
            'budget_seconds': budget_seconds, 'target_seconds': target_seconds,
            'new_item_cap_seconds': new_item_cap_seconds,
            'baseline': None if baseline is None else str(baseline),
            'baseline_error': baseline_error,
            'identities': len(current), 'modules': records,
            'added': added, 'removed': removed, 'new_item_violations': violations,
            'unverifiable_modules': unverifiable, 'over_budget_modules': over}


def render_session_markdown(document):
    lines = ['# ACE session cost report (read-only)', '',
             'Status: **%s**' % document['status'], '',
             '- modules: %d; identities: %d' % (len(document['modules']), document['identities']),
             '- budget: target %ss / accept %ss; NEW item cap: %.0fs'
             % (document['target_seconds'], document['budget_seconds'],
                document['new_item_cap_seconds'])]
    if document.get('baseline'):
        lines.append('- baseline: `%s` (added=%d, removed=%d)'
                     % (document['baseline'], len(document['added']), len(document['removed'])))
    if document.get('baseline_error'):
        lines.append('- baseline error: %s' % document['baseline_error'])
    lines.append('')
    lines.append('| module | status | T_warm (s) | target | budget | executed | stale |')
    lines.append('|---|---|---:|---:|---:|---:|---|')
    for record in document['modules']:
        warm = record.get('warm_wall_seconds')
        lines.append('| %s | %s | %s | %ss | %ss | %s | %s |' % (
            record.get('module') or 'unknown', record['status'],
            'UNOBSERVED' if warm is None else '%.3f' % warm,
            record['target_seconds'], record['budget_seconds'],
            (record.get('identity') or {}).get('executed'),
            'yes' if record.get('stale') else 'no'))
    if document.get('added'):
        lines.append('')
        lines.append('## Added identities (%d)' % len(document['added']))
        for name in document['added']:
            lines.append('- `%s`' % name)
    if document.get('removed'):
        lines.append('')
        lines.append('## Removed identities (%d)' % len(document['removed']))
        for name in document['removed']:
            lines.append('- `%s`' % name)
    if document.get('new_item_violations'):
        lines.append('')
        lines.append('## New-item cap violations (%d)' % len(document['new_item_violations']))
        for row in document['new_item_violations']:
            lines.append('- `%s` %s: %s' % (
                row['identity'],
                'UNOBSERVED' if row['exec_time_ms'] is None else '%.1fms' % row['exec_time_ms'],
                row['reason']))
    problems = [(record.get('module'), record.get('problems') or [],
                 record.get('missing_evidence') or []) for record in document['modules']]
    if any(module_problems for _, module_problems, _ in problems):
        lines.append('')
        lines.append('## Module problems')
        for module, module_problems, _ in problems:
            for item in module_problems:
                lines.append('- `%s`: %s' % (module, item))
    if any(missing for _, _, missing in problems):
        lines.append('')
        lines.append('## Missing / unverifiable evidence (exit 2)')
        for module, _, missing in problems:
            for item in missing:
                lines.append('- `%s`: %s' % (module, item))
    lines.append('')
    lines.append('Boundary: session scope reuses the per-module warm budget, terminal-event '
                 'integrity and STALE check; it adds only the identity diff and the new-item '
                 'cap. No new meter, no re-run, no artifact rewrite.')
    return '\n'.join(lines) + '\n'


def session_exit_code(document):
    if document['status'] == 'UNVERIFIABLE':
        return REGULAR_EXIT_UNVERIFIABLE
    if document['status'] == 'OVER_BUDGET':
        return REGULAR_EXIT_OVER
    return REGULAR_EXIT_OK


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', action='append', required=True, type=Path,
                        help='a fixed-driver --out directory (results.json inside)')
    parser.add_argument('--mode', choices=('policy', 'regular', 'session'), default='policy',
                        help='policy = frozen 13-item attribution (default); regular = one ordinary '
                             'module budget; session = several modules + identity diff/new-item cap')
    parser.add_argument('--budget-seconds', type=float, default=120.0,
                        help='regular mode acceptance upper bound (complete warm module invocation)')
    parser.add_argument('--target-seconds', type=float, default=60.0,
                        help='regular mode target (GREEN at or below it)')
    parser.add_argument('--cold-run', action='append', type=Path, default=[],
                        help='regular mode first cold preparation run (reported, never budgeted)')
    parser.add_argument('--baseline', type=Path,
                        help='session mode: previous session-baseline JSON for the added/removed diff')
    parser.add_argument('--baseline-out', type=Path,
                        help="session mode: write this session's identities as a fresh baseline")
    parser.add_argument('--new-item-cap-seconds', type=float, default=10.0,
                        help='session mode: an ADDED identity must finish within this cap')
    parser.add_argument('--policy', type=Path, default=POLICY_DEFAULT)
    parser.add_argument('--repo', type=Path, default=ROOT,
                        help='repo root used for staleness comparison')
    parser.add_argument('--out', required=True, type=Path,
                        help='fresh report path (markdown); refuses to overwrite')
    parser.add_argument('--json-out', type=Path,
                        help='optional machine-readable companion (also fresh)')
    args = parser.parse_args(argv)
    if args.out.exists():
        parser.error('report output already exists; preserve previous reports: ' + str(args.out))
    if args.json_out and args.json_out.exists():
        parser.error('json report output already exists: ' + str(args.json_out))
    if args.mode == 'session' and args.baseline_out and args.baseline_out.exists():
        parser.error('baseline output already exists; keep the previous baseline: '
                     + str(args.baseline_out))
    if args.mode != 'session' and (args.baseline or args.baseline_out):
        parser.error('--baseline/--baseline-out belong to --mode session')
    for run_dir in list(args.run) + list(args.cold_run):
        if not (run_dir / 'results.json').is_file():
            parser.error('not a fixed-driver run directory: ' + str(run_dir))
    if args.mode == 'session':
        document = session_report(args.run, repo_root=args.repo,
                                  budget_seconds=args.budget_seconds,
                                  target_seconds=args.target_seconds,
                                  baseline=args.baseline,
                                  new_item_cap_seconds=args.new_item_cap_seconds)
        args.out.write_text(render_session_markdown(document), encoding='utf-8')
        if args.json_out:
            args.json_out.write_text(json.dumps(document, ensure_ascii=False, indent=2) + '\n',
                                     encoding='utf-8')
        if args.baseline_out:
            write_session_baseline(document['modules'], args.baseline_out)
        print(str(args.out))
        return session_exit_code(document)
    if args.mode == 'regular':
        if args.cold_run and len(args.run) != 1:
            parser.error('--cold-run pairs one cold record with exactly one warm --run')
        records = [regular_module(run_dir, repo_root=args.repo,
                                  budget_seconds=args.budget_seconds,
                                  target_seconds=args.target_seconds)
                   for run_dir in args.run]
        if args.cold_run:
            records = [regular_cold(record, args.cold_run[0], repo_root=args.repo)
                       for record in records]
        document = {'schema': 'ace-regular-budget-report/1', 'mode': 'regular',
                    'budget_seconds': args.budget_seconds,
                    'target_seconds': args.target_seconds,
                    'modules': records}
        args.out.write_text(render_regular_markdown(records), encoding='utf-8')
        if args.json_out:
            args.json_out.write_text(json.dumps(document, ensure_ascii=False, indent=2) + '\n',
                                     encoding='utf-8')
        print(str(args.out))
        return regular_exit_code(records)
    _, policy_items = load_policy(args.policy)
    attributions, stale_all = [], []
    for run_dir in args.run:
        attributions.append(attribute_run(run_dir, policy_items))
        stale_all.append(staleness(json.loads((run_dir / 'results.json').read_text(encoding='utf-8')), args.repo))
    document = {'schema': 'ace-cost-report/1', 'runs': attributions, 'staleness': stale_all}
    args.out.write_text(render_markdown(attributions, stale_all), encoding='utf-8')
    if args.json_out:
        args.json_out.write_text(json.dumps(document, ensure_ascii=False, indent=2), encoding='utf-8')
    print(str(args.out))
    return 0


if __name__ == '__main__':
    sys.exit(main())