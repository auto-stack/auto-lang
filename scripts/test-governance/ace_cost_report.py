#!/usr/bin/env python3
"""PLAN-893 read-only cost attribution over real fixed-driver run outputs.

Reads existing `cargo t ... --out <dir>` artifacts (results.json,
*.command.json meters, libtest-json-plus terminal events, TestCase report
references); correlates them with scripts/test-governance/ace-test-policy.json.
Never starts tests, builds, jobs or meters, and never rewrites run artifacts
(reports go to a fresh --out path only).

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
GROUPS_CONFIG = ROOT / 'scripts/test-groups.toml'
FIXED_TESTS = ROOT / 'scripts/fixed-tests.py'

INTRA_RUN_PHASES = ('parent_side_verdict', 'persist_gc', 'copy', 'cleanup')


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
    return commands


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
        spec = importlib.util.spec_from_file_location('fixed_tests_local', root / 'scripts/fixed-tests.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        current = module.checkout_state()
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


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', action='append', required=True, type=Path,
                        help='a fixed-driver --out directory (results.json inside)')
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
    _, policy_items = load_policy(args.policy)
    attributions, stale_all = [], []
    for run_dir in args.run:
        if not (run_dir / 'results.json').is_file():
            parser.error('not a fixed-driver run directory: ' + str(run_dir))
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
