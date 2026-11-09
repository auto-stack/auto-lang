"""Admit explicit read-only parent discovery; never infer execution from it."""
from ace_m1_arch_acceptance import counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _duration

SOURCES = (
    'crates/auto-lang-core/src/test_cases/ac_av.rs',
    'crates/auto-lang-core/src/test_cases/ac_av/contract_inventory.rs',
    'crates/auto-lang-core/src/test_cases/ac_mn.rs',
    'crates/auto-lang-core/src/test_cases/abi.rs',
    'crates/auto-lang-core/src/test_cases/inventory.rs',
    'crates/auto-lang/src/bin/auto-case.rs',
    'crates/auto-lang-core/src/test_cases/inventory_probe.rs',
)
PRODUCER = 'target/debug/auto-case.exe'


def admit_parent_contracts(inputs, evidence, command_path, frozen):
    exact_keys(frozen, ('snapshot', 'source_sha256'), 'tracked parent contracts')
    exact_keys(frozen['source_sha256'], SOURCES, 'original parent sources')
    for path, digest in frozen['source_sha256'].items():
        inputs.sha256(path, digest)
    record, command_sha = evidence.read_json(command_path)
    exact_keys(record, ('schema', 'argv', 'cwd', 'exit', 'producer_sha256', 'stdout', 'stderr', 'meter'),
               'parent discovery capture')
    typed_equal(record['schema'], 'test-parent-contract-capture/1', 'parent capture schema')
    argv = [str(inputs.root / PRODUCER), 'parent-inventory']
    typed_equal(record['argv'], argv, 'original read-only discovery invocation')
    typed_equal(record['cwd'], str(inputs.root), 'parent discovery root')
    typed_equal(record['exit'], 0, 'parent discovery exit')
    inputs.sha256(PRODUCER, record['producer_sha256'])
    meter = record['meter']
    exact_keys(meter, ('argv', 'exit', 'error', 'wall_ms', 'workload_cpu_ms', 'meter_cpu_ms',
                       'total_cpu_ms', 'peak_process_bytes', 'peak_job_bytes', 'job_processes'),
               'original parent discovery meter')
    typed_equal(meter['argv'], argv, 'meter invocation')
    typed_equal(meter['exit'], 0, 'meter exit')
    typed_equal(meter['error'], None, 'meter terminal')
    for key in ('wall_ms', 'workload_cpu_ms', 'meter_cpu_ms', 'total_cpu_ms'):
        _duration(meter[key], 'parent discovery ' + key)
    require(abs(meter['total_cpu_ms'] - meter['workload_cpu_ms'] - meter['meter_cpu_ms']) < 1e-6,
            'parent discovery CPU attribution mismatch')
    for key in ('peak_process_bytes', 'peak_job_bytes', 'job_processes'):
        counter(meter[key], 'parent discovery ' + key)
    require(meter['peak_job_bytes'] >= meter['peak_process_bytes'], 'invalid parent discovery memory attribution')
    raw = {}
    for role in ('stdout', 'stderr'):
        ref = record[role]
        exact_keys(ref, ('path', 'bytes', 'sha256'), 'parent discovery raw reference')
        hash_value(ref['sha256'], 'parent discovery digest')
        counter(ref['bytes'], 'parent discovery bytes')
        path = evidence.sha256(ref['path'], ref['sha256'])
        typed_equal(path.stat().st_size, ref['bytes'], 'parent discovery byte count')
        raw[role] = evidence.read_bytes(ref['path'])
    typed_equal(raw['stderr'], b'', 'parent discovery stderr')
    snapshot, payload_sha = evidence.decode_json(raw['stdout'])
    exact_keys(snapshot, ('schema', 'scope', 'executed', 'parents'), 'parent discovery snapshot')
    for key, value in (('schema', 'test-parent-contract-inventory/1'), ('scope', 'discovery'), ('executed', 0)):
        typed_equal(snapshot[key], value, 'parent snapshot ' + key)
    exact_keys(snapshot['parents'], ('av', 'native', 'abi'), 'parent domains')
    for key, protocol, source in (('av', 'avworker/1', SOURCES[0]),
                                  ('native', 'mnworker/1', SOURCES[2]), ('abi', 'abiworker/1', SOURCES[3])):
        parent = snapshot['parents'][key]
        exact_keys(parent, ('source', 'protocol', 'limits', 'tables') if key == 'av'
                   else ('source', 'protocol', 'limits'), 'original ' + key + ' parent')
        typed_equal(parent['source'], source, 'parent source identity')
        typed_equal(parent['protocol'], protocol, 'parent protocol identity')
    # The original typed tables are the authority; a file with matching outer
    # hashes cannot override the independently tracked expectations.
    typed_equal(snapshot, frozen['snapshot'], 'original parent tables and budgets')
    av = snapshot['parents']['av']
    counts = {}
    for name, table in av['tables'].items():
        exact_keys(table, ('columns', 'rows'), 'original parent table')
        require(type(table['columns']) is list and bool(table['columns'])
                and all(type(field) is str for field in table['columns'])
                and len(set(table['columns'])) == len(table['columns']), 'invalid parent table columns')
        require(type(table['rows']) is list and bool(table['rows']), 'empty original parent table')
        require(all(type(row) is list and len(row) == len(table['columns']) for row in table['rows']),
                'parent table row shape')
        names = [row[0] for row in table['rows']]
        require(all(type(name) is str and bool(name) for name in names) and len(set(names)) == len(names),
                'duplicate original parent case identity')
        counts[name] = len(table['rows'])
    return {'schema':'ace.m1.parent-contract-admission/1', 'status':'DISCOVERY', 'executed':0,
            'command_sha256':command_sha, 'payload_sha256':payload_sha,
            'producer_sha256':record['producer_sha256'], 'source_sha256':frozen['source_sha256'],
            'tables':counts, 'limits':{key:parent['limits'] for key, parent in snapshot['parents'].items()},
            'note':'Original declared limits/oracles; runtime budget, result, lifecycle and performance gates remain separate.'}
