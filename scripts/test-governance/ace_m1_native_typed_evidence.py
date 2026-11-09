"""Read the original one-shot native typed frame; never infer wide source support."""
import base64
import binascii
import hashlib
import json
import struct
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import AdmissionError, counter, exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _fnv1a64

KINDS = {1:'i32', 2:'bool', 3:'i64', 4:'u64', 5:'f32', 6:'f64'}
COUNTERS = ('verify_mir', 'unit_prepare', 'unit_fp_render', 'native_admit', 'emit_units',
            'persist_units', 'cache_hits', 'wrapper_builds', 'wrapper_hits',
            'link_invocations', 'run_invocations', 'whole_fallback')


def native_typed_observation(copy, wire, expected):
    recipe = 'ac/mir-native-typed-v1'
    typed_equal(expected['recipe'], recipe, 'native typed recipe')
    require(expected['adapter'] == 'probe' and expected['action'] == 'run' and expected['state'] == 'RUN',
            'native typed check not selected')
    typed_equal(expected['completion'], {'kind':'success'}, 'native typed completion')
    typed_equal(expected['declared_probe_steps'], [{'op':'run'}], 'native typed steps')
    options = expected['probe_bindings']['native']
    exact_keys(options, ('entry', 'caps', 'session', 'diagnostics', 'resource', 'declaration', 'typed'), 'native typed options')
    typed_equal({k:v for k,v in options.items() if k not in ('entry', 'typed')},
                {'caps':[], 'session':None, 'diagnostics':None, 'resource':None, 'declaration':None},
                'one-shot native typed options')
    exact_keys(options['typed'], ('fixture', 'args'), 'native typed declaration')
    typed_equal(wire['protocol'], 'mnworker/1', 'native typed protocol')
    typed_equal(wire['recipe'], recipe, 'native typed wire recipe')
    require(type(wire['exit_code']) is int and wire['exit_code'] == 0 and wire['transport_error'] is None
            and wire['stderr'] == b'', 'failed native typed transport')
    request, _ = copy.scope.decode_json(wire['request'])
    exact_keys(request, ('recipe', 'nonce', 'typed_fixture_b64', 'entry', 'args', 'store_root', 'workdir', 'request_hash'), 'native typed request')
    require(type(request['nonce']) is str and bool(request['nonce']), 'native typed nonce required')
    require(type(request['typed_fixture_b64']) is str, 'native typed fixture must be base64 text')
    try:
        source = base64.b64decode(request['typed_fixture_b64'], validate=True)
    except (ValueError, binascii.Error) as error:
        raise AdmissionError('invalid native typed fixture encoding') from error
    typed_equal(base64.b64encode(source).decode(), request['typed_fixture_b64'], 'canonical native fixture encoding')
    require(len(expected['source_files']) == 1, 'one native typed fixture required')
    digest = hashlib.sha256(source).hexdigest()
    typed_equal(digest, expected['source_files'][0]['sha256'], 'current native typed fixture')
    artifact = PurePosixPath(wire['start_path']).parent.parent
    require(len(artifact.parts) == 3 and artifact.parts[1:] == (expected['case'], expected['check']), 'native typed owner')
    for key, leaf in (('store_root', 'mn-typed-store'), ('workdir', 'mn-typed-work')):
        require(type(request[key]) is str, 'native typed directory must be text')
        typed_equal(PureWindowsPath(request[key]), copy.original.joinpath(*artifact.parts, leaf), 'native typed directory owner')
    wanted = {'recipe':recipe, 'nonce':request['nonce'], 'typed_fixture_b64':request['typed_fixture_b64'],
              'entry':options['entry'], 'args':options['typed']['args'],
              'store_root':request['store_root'], 'workdir':request['workdir']}
    request_hash = hashlib.sha256(json.dumps(wanted, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest()
    typed_equal(request, wanted | {'request_hash':request_hash}, 'independent native typed request')
    # This is the versioned JSON fixture, not Auto source. Only its declared
    # scalar signature is read; no MIR interpretation or lowering occurs here.
    fixture, _ = copy.scope.decode_json(source)
    typed_equal(fixture['schema'], 'mn-typed-fixture/1', 'native fixture schema')
    units = [u for u in fixture['units'] if u['name'] == options['entry']]
    require(len(units) == 1, 'native typed entry signature missing/duplicated')
    unit = units[0]
    require(unit['result'] in KINDS.values() and all(type(p) is list and len(p) == 2 and p[1] in KINDS.values()
            for p in unit['params']), 'unsupported native typed entry signature')
    signature = '(' + ','.join(p[1] for p in unit['params']) + ')->' + unit['result']
    sig_id = int(_fnv1a64(signature.encode()), 16) & 0xffffffff
    response, response_sha = copy.scope.decode_json(wire['stdout'])
    exact_keys(response, ('schema', 'nonce', 'request_hash', 'ok'), 'native typed response')
    for key, value in (('schema', 'mnworker/1'), ('nonce', request['nonce']), ('request_hash', request_hash)):
        typed_equal(response[key], value, 'native typed response ' + key)
    ok = response['ok']
    exact_keys(ok, ('recipe', 'worker_version', 'mn_abi', 'mn_target', 'counters', 'typed',
                    'manifest_fp', 'exit_code', 'stderr_hex', 'stdout_hex'), 'native typed result')
    for key, value in (('recipe', recipe), ('worker_version', 'mnworker/1.0'), ('mn_abi', 'msvc-x64/priv-typed-data-v1'),
                       ('mn_target', 'x86_64-pc-windows-msvc'), ('exit_code', 0), ('stderr_hex', '')):
        typed_equal(ok[key], value, 'native typed result ' + key)
    exact_keys(ok['counters'], COUNTERS, 'native typed counters')
    for key, value in ok['counters'].items():
        counter(value, 'native typed counter ' + key)
    typed_equal(ok['counters']['run_invocations'], 1, 'one-shot native execution')
    require(type(ok['manifest_fp']) is str and len(ok['manifest_fp']) == 16
            and all(c in '0123456789abcdef' for c in ok['manifest_fp']), 'native manifest identity')
    raw = ok['stdout_hex']
    require(type(raw) is str and len(raw) == 88 and all(c in '0123456789abcdef' for c in raw), 'native frame must be exactly 44 bytes')
    frame = bytes.fromhex(raw)
    magic, version, frame_sig, status, nonce, kind_code, bits, checksum = struct.unpack('<IIIIQ I Q Q', frame)
    require(magic == 0x4d4e5231 and version == 1 and checksum == int(_fnv1a64(frame[:36]), 16), 'native frame magic/version/checksum')
    # Original one-shot parent supplies a text nonce; the worker's numeric
    # frame nonce is therefore zero. Preserve this boundary, no freshness claim.
    require(frame_sig == sig_id and nonce == 0 and status == 0 and kind_code in KINDS, 'native typed frame identity/outcome')
    kind = KINDS[kind_code]
    if kind in ('i32', 'bool', 'f32'):
        require(bits <= 0xffffffff and (kind != 'bool' or bits in (0, 1)), 'native typed frame payload width')
    width = 16 if kind in ('i64', 'u64', 'f64') else 8 if kind == 'f32' else 0
    observed = {'kind':kind, 'bits':format(bits, '0' + str(width) + 'x')}
    typed_equal(ok['typed'], {'status':'value', 'nonce':nonce, 'sig_id':sig_id} | observed, 'native typed JSON must match original machine frame')
    oracle = expected['typed_oracle']
    exact_keys(oracle, ('kind', 'bits'), 'independent native typed oracle')
    return {'schema':'ace.m1.native-typed-observation/1', 'input_kind':'typed-fixture', 'fixture_sha256':digest,
            'response_sha256':response_sha, 'expected':oracle, 'observed':observed,
            'judgement':'PASS' if observed == oracle else 'FAIL', 'frame_nonce':nonce,
            'scope':'Original private typed ABI, exact bits; no wide source, public ABI or freshness claim',
            'metric_gaps':['parent-io', 'parent-memory', 'per-stage-time']}
