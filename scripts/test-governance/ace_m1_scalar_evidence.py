"""Independent scalar observation reader for the real Q4 AV/native slice.

Only these two frozen scalar recipes are supported here. Emit/certification,
incremental state, foreign calls and milestone reduction have separate readers.
"""
import base64
import binascii
import hashlib
import json
import math

from ace_m1_arch_acceptance import AdmissionError, counter, exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal


def _duration(value, where):
    require(type(value) in (int, float) and math.isfinite(value) and value >= 0,
            f'{where}: expected a finite nonnegative duration')


def _counts(value, where):
    require(type(value) is dict, f'{where}: counter map required')
    for key, count in value.items():
        require(type(key) is str and bool(key), f'{where}: invalid counter name')
        counter(count, f'{where}.{key}')


def _fnv1a64(raw):
    value = 0xcbf29ce484222325
    for byte in raw:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f'{value:016x}'


def scalar_observation(scope, wire, expected):
    """Compare each observation with its independently resolved current oracle.

    `expected` is the current /2 identity inventory row, not a child report.
    The caller must first admit exchange bytes and pin its current worker.
    No Case status, pairwise equality, or echoed worker verdict is consulted.
    """
    recipe = expected['recipe']
    require(recipe in ('ac-av/core-run-v1', 'ac/mir-native-scalar-v1'),
            'unsupported scalar observation recipe')
    protocol = 'avworker/1' if recipe == 'ac-av/core-run-v1' else 'mnworker/1'
    typed_equal(wire['protocol'], protocol, 'scalar protocol')
    typed_equal(wire['recipe'], recipe, 'scalar recipe')
    require(type(wire['exit_code']) is int and wire['exit_code'] == 0
            and wire['transport_error'] is None, 'incomplete/failed scalar transport')
    require(wire['stderr'] == b'', 'scalar worker stderr is not empty')
    request, _ = scope.decode_json(wire['request'])
    common = {'recipe', 'nonce', 'source_b64', 'entry', 'caps', 'store_root', 'request_hash'}
    exact_keys(request, common | ({'steps'} if protocol == 'avworker/1' else {'workdir'}),
               'scalar request')
    typed_equal(request['recipe'], recipe, 'declared request recipe')
    require(type(request['nonce']) is str and bool(request['nonce']), 'missing request nonce')
    binding = expected.get('probe_bindings', {}).get('av' if protocol == 'avworker/1' else 'native')
    caps = binding['caps'] if binding is not None else []
    require(type(caps) is list and len(set(caps)) == len(caps) and all(
        type(cap) is str and cap in ('ac.source.cte.scalar/v1', 'cte.meta.fields/v1',
                                    'ac.source.instances.scalar/v1') for cap in caps),
        'unsupported scalar capabilities')
    typed_equal(request['entry'], binding['entry'] if binding is not None else 'd_main', 'scalar declared entry')
    typed_equal(request['caps'], caps, 'scalar declared capabilities')
    require(type(request['store_root']) is str and bool(request['store_root']), 'invalid store name')
    if protocol == 'avworker/1':
        typed_equal(request['steps'], [{'op': 'run'}], 'scalar request steps')
    else:
        require(type(request['workdir']) is str and bool(request['workdir']), 'invalid native work name')
    # The original parents hash the ordered serde_json request BEFORE adding
    # request_hash. This supported slice has strings, arrays and integer-only
    # options, so Python's compact UTF-8 serialization has the same bytes.
    unsigned = {key: value for key, value in request.items() if key != 'request_hash'}
    request_hash = hashlib.sha256(json.dumps(unsigned, ensure_ascii=False,
                                            separators=(',', ':')).encode()).hexdigest()
    typed_equal(request['request_hash'], request_hash, 'independently recomputed request hash')
    require(type(request['source_b64']) is str, 'scalar source must be base64 text')
    try:
        source = base64.b64decode(request['source_b64'], validate=True)
    except (ValueError, binascii.Error) as error:
        raise AdmissionError('invalid scalar source encoding') from error
    require(base64.b64encode(source).decode() == request['source_b64'], 'noncanonical scalar source encoding')
    require(len(expected['source_files']) == 1, 'scalar observation needs one current source file')
    typed_equal(hashlib.sha256(source).hexdigest(), expected['source_files'][0]['sha256'],
                'current source bytes')
    oracle = expected['expected_exit']
    require(type(oracle) is int and -(1 << 31) <= oracle < (1 << 31),
            'independent scalar i32 value oracle required')
    response, _ = scope.decode_json(wire['stdout'])
    exact_keys(response, ('schema', 'nonce', 'request_hash', 'ok'), 'scalar success envelope')
    for key, want in (('schema', protocol), ('nonce', request['nonce']), ('request_hash', request_hash)):
        typed_equal(response[key], want, f'scalar response {key}')
    ok = response['ok']
    if protocol == 'avworker/1':
        exact_keys(ok, ('recipe', 'steps', 'worker_version', 'av_core_version'), 'AV scalar ok')
        typed_equal(ok['worker_version'], 'avworker/1.0', 'AV worker version')
        typed_equal(ok['av_core_version'], 'av.core-i32/1', 'AV scalar core')
        require(type(ok['steps']) is list and len(ok['steps']) == 1, 'one AV scalar run required')
        step = ok['steps'][0]
        exact_keys(step, ('op', 'outcome', 'value', 'channel', 'trap', 'diag', 'counters',
                          'spans', 'source_hash', 'wall_ms', 'trace'), 'AV scalar step')
        for key, want in (('op', 'run'), ('outcome', 'success'), ('channel', 'value'),
                          ('trap', None), ('diag', None), ('trace', [])):
            typed_equal(step[key], want, f'AV scalar {key}')
        typed_equal(step['source_hash'], _fnv1a64(source), 'AV source FNV')
        require(type(step['value']) is int and -(1 << 31) <= step['value'] < (1 << 31),
                'AV scalar observation must be i32')
        observed = step['value']
        _counts(step['counters'], 'AV counters')
        require(type(step['spans']) is list, 'AV spans must be a list')
        names = set()
        for span in step['spans']:
            require(type(span) is list and len(span) == 2 and type(span[0]) is str
                    and bool(span[0]) and span[0] not in names, 'invalid/duplicate AV span')
            names.add(span[0])
            _duration(span[1], 'AV span')
        _duration(step['wall_ms'], 'AV wall')
    else:
        exact_keys(ok, ('recipe', 'worker_version', 'mn_abi', 'mn_target', 'counters',
                        'spans', 'manifest_fp', 'exit_code', 'stderr_hex', 'stdout_hex'), 'native scalar ok')
        for key, want in (('worker_version', 'mnworker/1.0'),
                          ('mn_abi', 'msvc-x64/priv-i32-bool-v1'),
                          ('mn_target', 'x86_64-pc-windows-msvc'),
                          ('stderr_hex', ''), ('stdout_hex', '')):
            typed_equal(ok[key], want, f'native scalar {key}')
        require(type(ok['exit_code']) is int and -(1 << 31) <= ok['exit_code'] < (1 << 31),
                'native scalar observation must be i32')
        observed = ok['exit_code']
        _counts(ok['counters'], 'native counters')
        _counts(ok['spans'], 'native spans')
        require(type(ok['manifest_fp']) is str and len(ok['manifest_fp']) == 16
                and all(c in '0123456789abcdef' for c in ok['manifest_fp']), 'invalid native manifest identity')
    typed_equal(ok['recipe'], recipe, 'scalar response recipe')
    return {'schema': 'ace.m1.scalar-observation/1', 'recipe': recipe,
            'source_sha256': hashlib.sha256(source).hexdigest(),
            'request_sha256': hashlib.sha256(wire['request']).hexdigest(),
            'response_sha256': hashlib.sha256(wire['stdout']).hexdigest(),
            'expected': oracle, 'observed': observed,
            'judgement': 'PASS' if observed == oracle else 'FAIL'}
