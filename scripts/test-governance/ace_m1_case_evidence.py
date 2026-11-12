"""Read-only Case identity/copy/wire admission; no domain or milestone verdict.

Expected inventories and worker identities come from the caller's frozen current
inputs. Neither a child PASS nor its echoed source/tool hashes supply authority.
"""
from pathlib import PurePosixPath, PureWindowsPath
import re

from ace_m1_arch_acceptance import AdmissionError, counter, exact_keys, hash_value
from ace_m1_legacy import typed_equal


def require(condition, message):
    if not condition:
        raise AdmissionError(message)


class CaseCopy:
    """Map old absolute *names* into the authorized copy; never open old paths."""
    def __init__(self, scope, manifest='artifact-copy.json'):
        self.scope = scope
        # PLAN-1013 A-1013-02 knife 3a: the manifest is a one-shot input —
        # release its conservative decode estimate once the index is built
        # (self.files stays live; the estimate over-reserves the dict).
        raw = scope.read_bytes(manifest)
        nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
        record, self.manifest_sha256 = scope.decode_json(raw)
        exact_keys(record, ('schema', 'original_runtime_root', 'files',
                            'runtime_cleanup_complete', 'note'), 'artifact copy')
        require(record['schema'] == 'test-case-artifact-copy/1', 'unknown artifact copy schema')
        require(record['runtime_cleanup_complete'] is True, 'runtime cleanup incomplete')
        require(type(record['note']) is str, 'artifact copy note must be text')
        old = record['original_runtime_root']
        require(type(old) is str and PureWindowsPath(old).is_absolute(), 'invalid original runtime name')
        self.original = PureWindowsPath(old)
        require(type(record['files']) is list, 'artifact copy files must be a list')
        self.files = {}
        for ref in record['files']:
            exact_keys(ref, ('path', 'bytes', 'sha256'), 'copied file')
            relative = ref['path']
            # Validate every indexed name without loading large cache/product files.
            scope.path(relative)
            require(relative not in self.files, 'duplicate copied artifact path')
            counter(ref['bytes'], 'copied file bytes')
            hash_value(ref['sha256'], 'copied file hash')
            self.files[relative] = ref
        scope.budget.release_transient(len(raw) + 1 + 7 * len(raw) + 256 * nodes + 4096)

    def path_from_original(self, old):
        require(type(old) is str, 'stored artifact must be a path string')
        try:
            relative = PureWindowsPath(old).relative_to(self.original).as_posix()
        except ValueError as error:
            raise AdmissionError('stored artifact lies outside the copied runtime') from error
        require(relative in self.files, 'stored artifact missing from copied manifest')
        return relative

    def verify(self, relative):
        require(relative in self.files, 'artifact not listed in copied manifest')
        ref = self.files[relative]
        path = self.scope.sha256(relative, ref['sha256'])
        require(path.stat().st_size == ref['bytes'], 'copied artifact byte count mismatch')
        return path

    def json(self, relative):
        self.verify(relative)
        return self.scope.read_json(relative)

    def bytes(self, relative):
        self.verify(relative)
        return self.scope.read_bytes(relative)

    def reference(self, directory, ref):
        exact_keys(ref, ('path', 'sha256', 'bytes'), 'exchange artifact reference')
        name = ref['path']
        require(type(name) is str and PurePosixPath(name).name == name,
                'exchange reference must be a filename in its own directory')
        relative = f'{directory}/{name}'
        require(relative in self.files, 'exchange artifact missing')
        typed_equal(ref, self.files[relative] | {'path': name}, 'exchange reference')
        return relative


def admit_case_bindings(copy, expected):
    """Reconcile the complete case denominator with a current /2 inventory.

    This does not rejudge execution and returns no PASS: domain readers must
    still bind raw requests/results and compare independent declared oracles.
    """
    require(expected.get('schema') == 'test-case-inventory/2', 'current identity inventory /2 required')
    require(expected.get('executed') == 0 and type(expected.get('executed')) is int,
            'inventory must not claim execution')
    require(type(expected.get('checks')) is list, 'inventory checks must be a list')
    # PLAN-1013 A-1013-02 knife 3a: the report is a one-shot input — release
    # its decode estimate once the admitted rows are extracted.
    copy.verify('report.json')
    raw = copy.scope.read_bytes('report.json')
    nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
    report, report_sha256 = copy.scope.decode_json(raw)
    exact_keys(report, ('schema', 'suite', 'suite_sha256', 'required_adapters', 'summary',
                        'toolchain', 'coverage_gaps', 'load_errors', 'results'), 'Case report')
    require(report['schema'] == 'autolang.test-report/2', 'unknown Case report schema')
    typed_equal(report['suite'], expected['suite'], 'suite name')
    typed_equal(report['suite_sha256'], expected['suite_sha256'], 'suite hash')
    require(report['coverage_gaps'] == [] and report['load_errors'] == [], 'incomplete Case load')
    require(type(report['required_adapters']) is list and type(report['toolchain']) is dict,
            'invalid Case header types')
    require(type(report['results']) is list, 'Case results must be a list')
    wanted = {}
    for check in expected['checks']:
        key = (check['case'], check['check'])
        require(key not in wanted, 'duplicate current check identity')
        wanted[key] = check
    seen, statuses = set(), {}
    admitted = []
    for row in report['results']:
        required = {'execution_key', 'case', 'check', 'adapter', 'action', 'tier',
                    'status', 'identity', 'duration_ms'}
        optional = {'detail', 'mismatches', 'artifacts', 'native'}
        require(type(row) is dict and required <= row.keys() <= required | optional,
                'unknown or missing Case row fields')
        require(type(row['case']) is str and type(row['check']) is str, 'invalid Case check name')
        key = (row['case'], row['check'])
        require(key in wanted and key not in seen, 'missing/extra/duplicate Case check identity')
        seen.add(key)
        check = wanted[key]
        typed_equal(row['execution_key'], ':'.join((expected['suite'], expected['suite_sha256'], *key)),
                    'execution key')
        for field in ('adapter', 'action', 'tier'):
            typed_equal(row[field], check[field], field)
        typed_equal(row['identity'], {
            'suite': expected['suite_sha256'], 'case_file': check['case_file_sha256'],
            'source': check['source_sha256'], 'oracle': check['oracle_sha256'],
            'contract': check['contract_sha256'],
        }, 'current Case identity')
        counter(row['duration_ms'], 'Case duration')
        require(row['status'] in ('PASS', 'FAIL', 'ERROR', 'SKIP'), 'unknown Case status')
        statuses[row['status']] = statuses.get(row['status'], 0) + 1
        if 'detail' in row:
            require(type(row['detail']) is str, 'invalid Case detail')
        if 'mismatches' in row:
            require(type(row['mismatches']) is list, 'invalid mismatches')
        require(type(row.get('artifacts', [])) is list, 'invalid Case artifacts')
        for artifact in row.get('artifacts', []):
            exact_keys(artifact, ('path', 'stored', 'sha256', 'bytes'), 'Case artifact')
            relative = copy.path_from_original(artifact['stored'])
            require(type(artifact['path']) is str and relative.endswith('/' + artifact['path']),
                    'logical artifact path does not match stored copy')
            typed_equal(artifact['bytes'], copy.files[relative]['bytes'], 'artifact bytes')
            typed_equal(artifact['sha256'], copy.files[relative]['sha256'], 'artifact SHA')
            copy.verify(relative)
        admitted.append((check, row))
    require(seen == wanted.keys(), 'Case report omits required checks')
    typed_equal(report['summary'], {'total': len(seen), **statuses}, 'Case summary')
    copy.scope.budget.release_transient(len(raw) + 1 + 7 * len(raw) + 256 * nodes + 4096)
    return {'schema': 'ace.m1.case-bindings/1', 'report_sha256': report_sha256,
            'checks': admitted, 'execution_judgement': 'UNOBSERVED'}


def admit_exchange(copy, start_path, *, protocol, recipe, worker_sha256):
    """Validate original byte/terminal linkage, without trusting child outcomes."""
    hash_value(worker_sha256, 'current worker hash')
    start, start_sha = copy.json(start_path)
    exact_keys(start, ('schema', 'exchange', 'phase', 'protocol', 'recipe',
                       'worker_sha256', 'request'), 'exchange start')
    identity = start['exchange']
    require(type(identity) is str and re.fullmatch(r'[0-9]+-[0-9]+', identity) is not None,
            'invalid exchange identity')
    require(PurePosixPath(start_path).name == f'{identity}-start.json', 'exchange start name mismatch')
    typed_equal({k: start[k] for k in ('schema', 'phase', 'protocol', 'recipe', 'worker_sha256')},
                {'schema': 'case-worker-exchange/1', 'phase': 'start', 'protocol': protocol,
                 'recipe': recipe, 'worker_sha256': worker_sha256}, 'current exchange identity')
    directory = PurePosixPath(start_path).parent.as_posix()
    terminal, _ = copy.json(f'{directory}/{identity}-terminal.json')
    exact_keys(terminal, ('schema', 'exchange', 'phase', 'start_sha256', 'exit_code',
                          'stdout', 'stderr', 'transport_error'), 'exchange terminal')
    typed_equal({k: terminal[k] for k in ('schema', 'exchange', 'phase', 'start_sha256')},
                {'schema': 'case-worker-exchange/1', 'exchange': identity,
                 'phase': 'terminal', 'start_sha256': start_sha}, 'exchange terminal binding')
    code = terminal['exit_code']
    require(code is None or type(code) is int and -(1 << 31) <= code < (1 << 31),
            'invalid worker exit code')
    error = terminal['transport_error']
    require(error is None or type(error) is str, 'invalid transport error')
    raw = {}
    for role, ref in (('request', start['request']), ('stdout', terminal['stdout']),
                      ('stderr', terminal['stderr'])):
        relative = copy.reference(directory, ref)
        require(ref['path'] == f'{identity}-{role}', 'exchange role name mismatch')
        raw[role] = copy.bytes(relative)
    return {'exchange': identity, 'protocol': protocol, 'recipe': recipe,
            'start_path': start_path, 'worker_sha256': worker_sha256,
            'exit_code': code, 'transport_error': error, **raw,
            'execution_judgement': 'UNOBSERVED'}
