"""Read the original 937 query/edited-query seam, including parent Job facts.

This does not implement ABI lowering or certify a foreign library. Restart,
foreign-contract generations and library provenance remain separate obligations.
"""
import base64
import hashlib
import json
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import counter, exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal


def _u32(value, where):
    require(counter(value, where) < 1 << 32, where + ' exceeds u32')
    return value


def _diagnostic_matches(diagnostic, completion):
    code = diagnostic['code']
    stage = ('parse' if code == 'deck' else 'admit' if code.startswith('admission/')
             else 'run' if code == 'pe-exit' else 'infra')
    matcher = completion['message']
    message = diagnostic['message']
    matches = stage == completion['stage'] and (completion['code'] is None or code == completion['code'])
    if matcher is not None:
        exact_keys(matcher, ('kind', 'text'), 'ABI diagnostic matcher')
        require(matcher['kind'] in ('exact', 'contains') and type(matcher['text']) is str,
                'unknown ABI diagnostic matcher')
        matches = matches and (message == matcher['text'] if matcher['kind'] == 'exact'
                               else matcher['text'] in message)
    return matches


def _bridge_decks(expected, source):
    """Shared declared-steps replay: deck identity, leg, unique edit matches."""
    typed_equal(expected['recipe'], 'abi/ca-d1-bridge-v1', 'bridge recipe')
    require(expected['adapter'] == 'probe' and expected['action'] == 'run', 'not an ABI probe')
    require(type(source) is bytes and len(expected['source_files']) == 1, 'one current deck required')
    typed_equal(hashlib.sha256(source).hexdigest(), expected['source_files'][0]['sha256'], 'current ABI deck')
    exact_keys(expected['abi_options'], ('leg',), 'ABI options')
    require(expected['abi_options']['leg'] in ('source', 'typed-self', 'typed-n2c', 'typed-c2n'),
            'unknown bridge leg')
    steps = expected['abi_steps']
    require(type(steps) is list and bool(steps), 'zero ABI steps')
    deck, decks = source, []
    for step in steps:
        require(type(step) is dict and 'op' in step, 'invalid declared ABI step')
        if step['op'] == 'edit-source':
            exact_keys(step, ('op', 'from', 'to'), 'ABI edit')
            require(all(type(step[k]) is str for k in ('from', 'to')) and bool(step['from']), 'invalid ABI edit')
            text = deck.decode('utf-8', errors='replace')
            require(text.count(step['from']) == 1, 'ABI edit requires exactly one current match')
            deck = text.replace(step['from'], step['to'], 1).encode()
        else:
            exact_keys(step, ('op',), 'ABI query')
            typed_equal(step['op'], 'query', 'this slice does not admit restart or other operations')
            decks.append(deck)
    return decks


def _bridge_completion_oracles(expected, decks):
    """Shared completion/oracle denominator admission."""
    oracles = expected['probe_oracles']
    require(type(oracles) is list and len(oracles) in (0, len(decks)), 'ABI oracle denominator mismatch')
    completion = expected['completion']
    if completion.get('kind') == 'success':
        exact_keys(completion, ('kind',), 'ABI success oracle')
    else:
        exact_keys(completion, ('kind', 'stage', 'code', 'message'), 'ABI diagnostic oracle')
        typed_equal(completion['kind'], 'error', 'ABI completion')
        require(completion['stage'] in ('parse', 'admit', 'run'), 'infra is not an ABI language oracle')
    return oracles, completion


def bridge_query_observations(copy, wires, expected, source):
    """Reconcile all declared query steps; no child PASS supplies authority."""
    decks = _bridge_decks(expected, source)
    require(type(wires) is list and len(wires) == len(decks) and bool(wires), 'ABI query denominator mismatch')
    oracles, completion = _bridge_completion_oracles(expected, decks)
    artifact = PurePosixPath(wires[0]['start_path']).parent.parent
    require(len(artifact.parts) == 3 and artifact.parts[1:] == (expected['case'], expected['check']),
            'ABI evidence belongs to another case/check')
    sample_path = artifact.as_posix() + '/abi-request-samples.jsonl'
    lines = copy.bytes(sample_path).splitlines()
    require(len(lines) == len(wires), 'ABI parent sample denominator mismatch')
    samples = [copy.scope.decode_json(line)[0] for line in lines]
    matched, outcomes, nonces = [], [], set()
    for index, (wire, deck, sample) in enumerate(zip(wires, decks, samples)):
        typed_equal(wire['protocol'], 'abiworker/1', 'ABI protocol')
        typed_equal(wire['recipe'], expected['recipe'], 'ABI wire recipe')
        typed_equal(PurePosixPath(wire['start_path']).parent.parent, artifact, 'ABI sample owner')
        require(type(wire['exit_code']) is int and wire['exit_code'] == 0
                and wire['transport_error'] is None and wire['stderr'] == b'', 'failed ABI transport')
        request, _ = copy.scope.decode_json(wire['request'])
        exact_keys(request, ('recipe', 'nonce', 'deck_b64', 'leg', 'store_root', 'steps', 'request_hash'), 'ABI request')
        nonce = request['nonce']
        require(type(nonce) is str and bool(nonce) and nonce not in nonces, 'duplicate/missing ABI nonce')
        nonces.add(nonce)
        wanted = {'recipe': expected['recipe'], 'nonce': nonce,
                  'deck_b64': base64.b64encode(deck).decode(), 'leg': expected['abi_options']['leg'],
                  'store_root': request['store_root'], 'steps': [{'op': 'query'}]}
        require(type(request['store_root']) is str, 'invalid ABI store root')
        typed_equal(PureWindowsPath(request['store_root']), copy.original.joinpath(*artifact.parts, 'store'), 'ABI store owner')
        request_hash = hashlib.sha256(json.dumps(wanted, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest()
        typed_equal(request, wanted | {'request_hash': request_hash}, 'independent current ABI request')
        response, response_sha = copy.scope.decode_json(wire['stdout'])
        exact_keys(response, ('schema', 'nonce', 'request_hash', 'ok'), 'ABI response')
        for key, value in (('schema', 'abiworker/1'), ('nonce', nonce), ('request_hash', request_hash)):
            typed_equal(response[key], value, 'ABI response ' + key)
        ok = response['ok']
        exact_keys(ok, ('recipe', 'steps', 'restart_pending', 'worker_version'), 'ABI result')
        typed_equal(ok['recipe'], expected['recipe'], 'ABI result recipe')
        typed_equal(ok['restart_pending'], False, 'query cannot authorize restart')
        typed_equal(ok['worker_version'], 'abi-worker/0.1.0', 'ABI worker version')
        require(type(ok['steps']) is list and len(ok['steps']) == 1, 'exactly one original query result required')
        result = ok['steps'][0]
        required = {'op', 'outcome', 'cache', 'compute_calls', 'meter'}
        require(type(result) is dict and required <= result.keys() <= required | {'diagnostics', 'pe_exit', 'stages'}, 'ABI step fields')
        typed_equal(result['op'], 'query', 'ABI result operation')
        require(result['outcome'] in ('ok', 'diags', 'infra', 'resource'), 'unknown ABI outcome')
        require(result['cache'] in ('published', 'hit', 'miss', 'stale', '-'), 'invalid ABI cache observation')
        _u32(result['compute_calls'], 'ABI compute calls')
        diagnostics = result.get('diagnostics', [])
        require(type(diagnostics) is list, 'ABI diagnostics must be a list')
        for diagnostic in diagnostics:
            exact_keys(diagnostic, ('code', 'message'), 'ABI diagnostic')
            require(all(type(diagnostic[k]) is str and bool(diagnostic[k]) for k in diagnostic), 'ABI diagnostic text')
        pe_exit = result.get('pe_exit')
        require(pe_exit is None or type(pe_exit) is int and -(1 << 31) <= pe_exit < (1 << 31), 'invalid ABI exit')
        stages = {}
        require(type(result.get('stages', [])) is list, 'ABI stages must be a list')
        for stage in result.get('stages', []):
            exact_keys(stage, ('name', 'wall_ns'), 'ABI stage')
            require(type(stage['name']) is str and bool(stage['name']) and stage['name'] not in stages, 'duplicate/invalid ABI stage')
            stages[stage['name']] = counter(stage['wall_ns'], 'ABI stage wall') // 1_000_000
        meter = result['meter']
        exact_keys(meter, ('hot_bytes', 'scratch_peak_disk_bytes', 'rejections', 'gc_runs', 'gc_deleted_chunks', 'gc_freed_bytes'), 'ABI worker meter')
        for name, value in meter.items():
            (_u32 if name in ('rejections', 'gc_runs', 'gc_deleted_chunks') else counter)(value, name)
        exact_keys(sample, ('seq', 'op', 'ok', 'edit_ms', 'request_ms', 'outcome', 'pe_exit', 'cache', 'compute_calls',
                            'stages', 'meter', 'peak_mem_bytes', 'peak_job_mem_bytes', 'io_read_bytes', 'io_write_bytes',
                            'error', 'request_nonce'), 'ABI parent sample')
        for name in ('seq', 'edit_ms', 'request_ms', 'peak_mem_bytes', 'peak_job_mem_bytes', 'io_read_bytes', 'io_write_bytes'):
            counter(sample[name], 'parent ABI ' + name)
        typed_equal(sample['seq'], index, 'ABI sample sequence')
        typed_equal(sample['request_nonce'], nonce, 'ABI sample nonce')
        for name, value in (('op', 'query'), ('outcome', result['outcome']), ('pe_exit', pe_exit), ('cache', result['cache']),
                            ('compute_calls', result['compute_calls']), ('stages', stages)):
            typed_equal(sample[name], value, 'ABI sample ' + name)
        typed_equal(sample['meter'], {('scratch_disk_bytes' if k == 'scratch_peak_disk_bytes' else k): v
                                     for k, v in meter.items()}, 'ABI sample meter')
        require(type(sample['ok']) is bool and (sample['error'] is None or type(sample['error']) is str), 'ABI sample judgement shape')
        semantic = (result['outcome'] == 'ok' and not diagnostics and pe_exit == 0
                    if completion['kind'] == 'success' else result['outcome'] == 'diags'
                    and any(_diagnostic_matches(d, completion) for d in diagnostics))
        matches = [semantic]
        if oracles:
            oracle = oracles[index]
            exact_keys(oracle, ('cache', 'compute_calls', 'bindings_sha256', 'stages_include'), 'ABI step oracle')
            require(oracle['bindings_sha256'] is None, 'bridge slice has no independent bindings admission')
            if oracle['cache'] is not None: matches.append(result['cache'] == oracle['cache'])
            if oracle['compute_calls'] is not None: matches.append(result['compute_calls'] == _u32(oracle['compute_calls'], 'expected compute'))
            require(type(oracle['stages_include']) is list and all(type(n) is str for n in oracle['stages_include']), 'invalid ABI stage oracle')
            matches.append(set(oracle['stages_include']) <= stages.keys())
        # A parent rejection is never promoted; an echoed true does not establish correctness.
        matches.append(sample['ok'] is True and sample['error'] is None)
        matched.append(all(matches))
        outcomes.append({'response_sha256': response_sha, 'cache': result['cache'], 'compute_calls': result['compute_calls'],
                         'outcome': result['outcome'], 'stages_observed': 'stages' in result, 'parent_job': {k:sample[k] for k in
                          ('request_ms', 'edit_ms', 'peak_mem_bytes', 'peak_job_mem_bytes', 'io_read_bytes', 'io_write_bytes')}})
    return {'schema': 'ace.m1.bridge-query-observation/1', 'queries': outcomes,
            'judgement': 'PASS' if all(matched) else 'FAIL',
            'scope': 'declared query/oracle and parent Job observations; no restart, foreign provenance or performance claim'}


# PLAN-941 round 120 (D937-final store seam): the bridge suite's case parent
# drives the ABI worker through the store (journal/blocks/bridge-scratch).
# Worker-exchange frames, nonces and request-hash chaining are not observable
# on this seam and are not claimed here; the observation consumes the parent
# Job samples plus the store's real product surface.
_STORE_SAMPLE_FIELDS = ('seq', 'op', 'ok', 'edit_ms', 'request_ms', 'outcome', 'pe_exit', 'cache',
                        'compute_calls', 'stages', 'meter', 'peak_mem_bytes', 'peak_job_mem_bytes',
                        'io_read_bytes', 'io_write_bytes', 'error')


def bridge_store_query_observations(copy, expected, source):
    """Reconcile declared query steps over the store seam; no parent PASS supplies authority."""
    decks = _bridge_decks(expected, source)
    oracles, completion = _bridge_completion_oracles(expected, decks)
    suffix = f"/{expected['case']}/{expected['check']}/abi-request-samples.jsonl"
    sample_paths = sorted(name for name in copy.files if name.endswith(suffix))
    require(len(sample_paths) == 1, 'ABI store sample must belong to exactly one run of this check')
    lines = copy.bytes(sample_paths[0]).splitlines()
    require(bool(lines) and len(lines) == len(decks), 'ABI parent sample denominator mismatch')
    samples = [copy.scope.decode_json(line)[0] for line in lines]
    base = sample_paths[0][:-len('/abi-request-samples.jsonl')]
    matched, outcomes = [], []
    for index, (deck, sample) in enumerate(zip(decks, samples)):
        exact_keys(sample, _STORE_SAMPLE_FIELDS, 'ABI store parent sample')
        for name in ('seq', 'edit_ms', 'request_ms', 'peak_mem_bytes', 'peak_job_mem_bytes',
                     'io_read_bytes', 'io_write_bytes'):
            counter(sample[name], 'parent ABI ' + name)
        typed_equal(sample['seq'], index, 'ABI sample sequence')
        typed_equal(sample['op'], 'query', 'ABI sample operation')
        _u32(sample['compute_calls'], 'ABI store compute calls')
        require(sample['outcome'] in ('ok', 'diags', 'infra', 'resource'), 'unknown ABI outcome')
        require(sample['cache'] in ('published', 'hit', 'miss', 'stale', '-'), 'invalid ABI cache observation')
        pe_exit = sample['pe_exit']
        require(pe_exit is None or type(pe_exit) is int and -(1 << 31) <= pe_exit < (1 << 31),
                'invalid ABI exit')
        stages = sample['stages']
        require(type(stages) is dict and all(type(name) is str and bool(name) and counter(value, 'ABI stage wall') >= 0
                                             for name, value in stages.items()), 'invalid ABI stage map')
        meter = sample['meter']
        exact_keys(meter, ('hot_bytes', 'scratch_disk_bytes', 'rejections', 'gc_runs',
                           'gc_deleted_chunks', 'gc_freed_bytes'), 'ABI worker meter')
        for name, value in meter.items():
            (_u32 if name in ('rejections', 'gc_runs', 'gc_deleted_chunks') else counter)(value, name)
        require(type(sample['ok']) is bool and (sample['error'] is None or type(sample['error']) is str),
                'ABI sample judgement shape')
        semantic = (sample['outcome'] == 'ok' and sample['error'] is None and pe_exit in (None, 0)
                    if completion['kind'] == 'success' else sample['outcome'] == 'diags')
        matches = [semantic, sample['ok'] is True]
        if oracles:
            oracle = oracles[index]
            exact_keys(oracle, ('cache', 'compute_calls', 'bindings_sha256', 'stages_include'), 'ABI step oracle')
            require(oracle['bindings_sha256'] is None, 'bridge slice has no independent bindings admission')
            if oracle['cache'] is not None: matches.append(sample['cache'] == oracle['cache'])
            if oracle['compute_calls'] is not None:
                matches.append(sample['compute_calls'] == _u32(oracle['compute_calls'], 'expected compute'))
            require(type(oracle['stages_include']) is list and all(type(n) is str for n in oracle['stages_include']),
                    'invalid ABI stage oracle')
            matches.append(set(oracle['stages_include']) <= stages.keys())
        matched.append(all(matches))
        outcomes.append({'cache': sample['cache'], 'compute_calls': sample['compute_calls'],
                         'outcome': sample['outcome'], 'parent_job': {k: sample[k] for k in
                          ('request_ms', 'edit_ms', 'peak_mem_bytes', 'peak_job_mem_bytes',
                           'io_read_bytes', 'io_write_bytes')}})
    # The store seam must actually have been used; queries that computed must
    # leave their real compilation products in bridge-scratch (a parse-stage
    # rejection compiles nothing — its cache axis is '-' — and has no scratch).
    require(base + '/store/journal.db' in copy.files, 'ABI store journal missing')
    scratch = [name for name in copy.files if name.startswith(base + '/store/bridge-scratch/')]
    if any(sample['cache'] != '-' for sample in samples):
        require(bool(scratch), 'ABI bridge-scratch products missing')
        leg = expected['abi_options']['leg']
        if leg == 'source':
            require(any(name.endswith('/generated.c') for name in scratch)
                    and any(name.endswith('/a2x_c_runtime.h') for name in scratch)
                    and any(name.endswith('/bridge_source_caller.c') for name in scratch),
                    'source-leg compile products missing from bridge-scratch')
        else:
            lane = {'typed-self': '/self/', 'typed-n2c': '/n2c/', 'typed-c2n': '/c2n/'}[leg]
            require(any(lane in name and name.endswith('.obj') for name in scratch),
                    'typed-leg object products missing from bridge-scratch')
    return {'schema': 'ace.m1.bridge-store-query-observation/1', 'queries': outcomes,
            'judgement': 'PASS' if all(matched) else 'FAIL',
            'scope': 'declared query/oracle and parent Job observations over the store seam '
                     '(journal/bridge-scratch products); no exchange-frame, nonce/request-hash, '
                     'restart, foreign provenance or performance claim'}
