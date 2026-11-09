"""Admit the original AV data typed-entry face without inventing wide source.

The supplied Auto file is a request carrier. The selected typed fixture is
bound by its entry id, current worker and the original parent's exact bit table.
"""
import base64
import binascii
import hashlib
import json
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import AdmissionError, exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _duration, _fnv1a64

WIDTHS = {'i64':16, 'u64':16, 'f64':16, 'f32':8, 'i32':8, 'bool':1}


def av_typed_observation(copy, wire, expected, oracle):
    exact_keys(oracle, ('fixture', 'kind', 'bits'), 'independent typed oracle')
    require(type(oracle['fixture']) is str and bool(oracle['fixture']), 'typed fixture id required')
    typed_equal(expected['recipe'], 'ac-av/data-run-v1', 'AV typed recipe')
    require(expected['adapter'] == 'probe' and expected['action'] == 'run' and expected['state'] == 'RUN',
            'typed fixture not selected')
    typed_equal(expected['completion'], {'kind':'success'}, 'typed completion')
    options = expected['probe_bindings']['av']
    exact_keys(options, ('entry', 'caps', 'session', 'profile', 'resource', 'invalidation'), 'typed AV options')
    entry = 'typed:' + oracle['fixture']
    typed_equal(options, {'entry':entry, 'caps':[], 'session':None, 'profile':None,
                          'resource':None, 'invalidation':None}, 'original typed fixture options')
    typed_equal(expected['declared_probe_steps'], [{'op':'run'}], 'typed declared steps')
    typed_equal(wire['protocol'], 'avworker/1', 'typed wire protocol')
    typed_equal(wire['recipe'], expected['recipe'], 'typed wire recipe')
    require(type(wire['exit_code']) is int and wire['exit_code'] == 0 and wire['transport_error'] is None
            and wire['stderr'] == b'', 'failed typed transport')
    request, _ = copy.scope.decode_json(wire['request'])
    exact_keys(request, ('recipe', 'nonce', 'source_b64', 'entry', 'caps', 'store_root', 'steps',
                         'typed_fixture', 'request_hash'), 'typed request')
    require(type(request['nonce']) is str and bool(request['nonce']), 'typed nonce required')
    require(type(request['source_b64']) is str, 'typed carrier must be base64 text')
    try:
        source = base64.b64decode(request['source_b64'], validate=True)
    except (ValueError, binascii.Error) as error:
        raise AdmissionError('invalid typed carrier encoding') from error
    typed_equal(base64.b64encode(source).decode(), request['source_b64'], 'canonical typed carrier encoding')
    require(len(expected['source_files']) == 1, 'one current typed carrier required')
    source_sha = hashlib.sha256(source).hexdigest()
    typed_equal(source_sha, expected['source_files'][0]['sha256'], 'current typed request carrier')
    artifact = PurePosixPath(wire['start_path']).parent.parent
    require(len(artifact.parts) == 3 and artifact.parts[1:] == (expected['case'], expected['check']),
            'typed request belongs to another case/check')
    require(type(request['store_root']) is str, 'typed store root required')
    typed_equal(PureWindowsPath(request['store_root']), copy.original.joinpath(*artifact.parts, 'store'),
                'typed store owner')
    wanted = {'recipe':expected['recipe'], 'nonce':request['nonce'], 'source_b64':request['source_b64'],
              'entry':entry, 'caps':[], 'store_root':request['store_root'], 'steps':[{'op':'run'}],
              'typed_fixture':oracle['fixture']}
    request_hash = hashlib.sha256(json.dumps(wanted, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest()
    typed_equal(request, wanted | {'request_hash':request_hash}, 'independent typed request')
    response, response_sha = copy.scope.decode_json(wire['stdout'])
    exact_keys(response, ('schema', 'nonce', 'request_hash', 'ok'), 'typed response')
    for key, value in (('schema', 'avworker/1'), ('nonce', request['nonce']), ('request_hash', request_hash)):
        typed_equal(response[key], value, 'typed response ' + key)
    ok = response['ok']
    exact_keys(ok, ('recipe', 'steps', 'worker_version', 'av_core_version'), 'typed result')
    for key, value in (('recipe', expected['recipe']), ('worker_version', 'avworker/1.0'), ('av_core_version', 'av.data/1')):
        typed_equal(ok[key], value, 'typed result ' + key)
    require(type(ok['steps']) is list and len(ok['steps']) == 1, 'one typed run required')
    step = ok['steps'][0]
    exact_keys(step, ('op', 'outcome', 'value', 'typed', 'trap', 'diag', 'source_hash', 'wall_ms'), 'typed step')
    for key, value in (('op', 'run'), ('outcome', 'success'), ('value', None), ('trap', None), ('diag', None)):
        typed_equal(step[key], value, 'typed step ' + key)
    # The original typed branch hashes its entry identity, not the .at carrier.
    typed_equal(step['source_hash'], _fnv1a64(entry.encode()), 'original typed entry identity')
    _duration(step['wall_ms'], 'typed run wall')
    typed = step['typed']
    exact_keys(typed, ('kind', 'bits'), 'typed value')
    kind, bits = typed['kind'], typed['bits']
    require(type(kind) is str and kind in WIDTHS, 'unknown typed value kind')
    require(type(bits) is str and 0 < len(bits) <= WIDTHS[kind]
            and all(char in '0123456789abcdef' for char in bits), 'typed bits must be exact lowercase hex text')
    require(kind != 'bool' or bits in ('0', '1'), 'invalid typed bool bits')
    return {'schema':'ace.m1.av-typed-observation/1', 'input_kind':'typed-fixture',
            'fixture':oracle['fixture'], 'request_carrier_sha256':source_sha, 'response_sha256':response_sha,
            'expected':{'kind':oracle['kind'], 'bits':oracle['bits']}, 'observed':typed,
            'judgement':'PASS' if typed == {'kind':oracle['kind'], 'bits':oracle['bits']} else 'FAIL',
            'metric_gaps':['producer-counters', 'parent-io', 'parent-memory'],
            'scope':'Exact typed tag/bits only; no wide source or performance claim'}
