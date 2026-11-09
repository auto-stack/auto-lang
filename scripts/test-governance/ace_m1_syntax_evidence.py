"""Read the original syntax/parse slice and compare independent case oracles.

This reader does not parse Auto source or reproduce the syntax engine. It
admits the original request/response and verifies byte reconstruction and typed
diagnostic oracles. Incremental/resource obligations have their own evidence.
"""
import base64
import binascii
import hashlib
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import AdmissionError, counter, exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal


def _source(raw):
    require(type(raw) is str, 'syntax source must be base64 text')
    try:
        value = base64.b64decode(raw, validate=True)
    except (ValueError, binascii.Error) as error:
        raise AdmissionError('invalid syntax source encoding') from error
    require(base64.b64encode(value).decode() == raw, 'noncanonical syntax source encoding')
    return value


def syntax_parse_observation(copy, wire, expected):
    scope = copy.scope
    require(expected['adapter'] == 'syntax' and expected['action'] == 'parse',
            'this slice only admits original syntax/parse')
    options = expected['ac_options']
    exact_keys(options, ('profile', 'entry', 'cache', 'grammar', 'capabilities', 'budget'),
               'current syntax options')
    grammar = options['grammar']
    require(type(grammar) is str and grammar.startswith('auto-source@'), 'current grammar identity required')
    for field in ('profile', 'entry', 'cache'):
        require(type(options[field]) is str and bool(options[field]), 'invalid current syntax ' + field)
    require(type(options['capabilities']) is list
            and all(type(cap) is str for cap in options['capabilities']), 'invalid syntax capabilities')
    typed_equal(wire['protocol'], 'acworker/1', 'syntax protocol')
    typed_equal(wire['recipe'], 'syntax/' + grammar, 'syntax recipe')
    require(type(wire['exit_code']) is int and wire['exit_code'] == 0
            and wire['transport_error'] is None and wire['stderr'] == b'', 'failed syntax transport')
    lines = wire['request'].splitlines()
    require(len(lines) == 3, 'one complete begin/request/end syntax exchange required')
    frames = [scope.decode_json(line)[0] for line in lines]
    typed_equal(frames[0], {'op': 'begin', 'schema': 'acworker/1'}, 'syntax begin')
    exact_keys(frames[1], ('op', 'request'), 'syntax request frame')
    typed_equal(frames[1]['op'], 'request', 'syntax request operation')
    typed_equal(frames[2], {'op': 'end'}, 'syntax end')
    request = frames[1]['request']
    exact_keys(request, ('schema', 'kind', 'nonce', 'request_hash', 'input', 'profile', 'entry',
                         'budget', 'store_root', 'grammar', 'capabilities', 'support_lib',
                         'edits', 'cache', 'project', 'include_hir', 'scratch'), 'syntax request')
    for key, want in (('schema', 'acworker/1'), ('kind', 'parse'), ('profile', options['profile']),
                      ('entry', options['entry']), ('grammar', grammar),
                      ('cache', options['cache']), ('capabilities', options['capabilities']),
                      ('store_root', None), ('support_lib', None), ('edits', None), ('project', 'ac-case'),
                      ('include_hir', bool(expected['oracle_artifacts']))):
        typed_equal(request[key], want, 'current syntax request ' + key)
    require(type(request['nonce']) is str and bool(request['nonce']), 'missing syntax nonce')
    exact_keys(options['budget'], ('build_ms', 'run_ms', 'total_ms'), 'declared syntax budget')
    budget = {key: options['budget'][key] if options['budget'][key] is not None else default
              for key, default in (('build_ms', 60000), ('run_ms', 5000), ('total_ms', 70000))}
    for key, value in budget.items():
        counter(value, 'syntax budget ' + key)
    typed_equal(request['budget'], budget, 'current syntax budget')
    exact_keys(request['input'], ('kind', 'source_b64'), 'syntax input')
    typed_equal(request['input']['kind'], 'source', 'syntax input kind')
    source = _source(request['input']['source_b64'])
    require(len(expected['source_files']) == 1, 'one current syntax input required')
    source_sha = hashlib.sha256(source).hexdigest()
    typed_equal(source_sha, expected['source_files'][0]['sha256'], 'current syntax source bytes')
    artifact = PurePosixPath(wire['start_path']).parent.parent
    require(len(artifact.parts) == 3 and artifact.parts[1:] == (expected['case'], expected['check']),
            'syntax exchange belongs to another case/check')
    require(type(request['scratch']) is str, 'syntax scratch name required')
    typed_equal(PureWindowsPath(request['scratch']), copy.original.joinpath(*artifact.parts, 'work'),
                'syntax scratch owner')
    # Preserve the actual display spelling used by the original parent hash;
    # canonical namespace was checked above. Never open the old absolute root.
    suffix = '\\' + expected['case'] + '\\' + expected['check'] + '\\work'
    require(request['scratch'].endswith(suffix), 'unsupported syntax scratch spelling')
    old_run = request['scratch'][:-len(suffix)]
    identity = '|'.join(('acworker/1', 'parse', source_sha, options['profile'], options['entry'],
                         str(budget['build_ms']), str(budget['run_ms']), str(budget['total_ms']),
                         request['scratch'], options['cache'], old_run + '\\cache-store'))
    request_hash = hashlib.sha256(identity.encode()).hexdigest()
    typed_equal(request['request_hash'], request_hash, 'independently recomputed syntax request hash')
    response, response_sha = scope.decode_json(wire['stdout'])
    exact_keys(response, ('schema', 'nonce', 'request_hash', 'ok'), 'syntax language envelope')
    for key, want in (('schema', 'acworker/1'), ('nonce', request['nonce']), ('request_hash', request_hash)):
        typed_equal(response[key], want, 'syntax response ' + key)
    ok = response['ok']
    exact_keys(ok, ('diagnostics', 'hir_b64', 'artifacts', 'run', 'trace', 'toolchain',
                    'cache', 'metric_gaps'), 'syntax observation')
    typed_equal(ok['hir_b64'], None, 'syntax has no backend HIR payload')
    typed_equal(ok['run'], None, 'syntax must not impersonate execution')
    tools = dict.fromkeys(('lld_path', 'lld_sha256', 'sdk_um', 'sdk_um_sha256',
                           'ac_core_version', 'worker_version', 'backend'), '')
    tools['worker_exe_sha256'] = wire['worker_sha256']
    typed_equal(ok['toolchain'], tools, 'syntax-only worker identity')
    diagnostics = ok['diagnostics']
    require(type(diagnostics) is list, 'syntax diagnostics must be a list')
    for diagnostic in diagnostics:
        exact_keys(diagnostic, ('stage', 'code', 'message', 'span_start', 'span_end'), 'syntax diagnostic')
        for key in ('stage', 'code', 'message'):
            require(type(diagnostic[key]) is str and bool(diagnostic[key]), 'invalid syntax diagnostic ' + key)
        a, b = (counter(diagnostic[key], key) for key in ('span_start', 'span_end'))
        require(a <= b <= len(source), 'syntax diagnostic span outside current source')
    completion = expected['completion']
    require(type(completion) is dict and type(completion.get('kind')) is str, 'current completion oracle required')
    if completion['kind'] == 'success':
        exact_keys(completion, ('kind',), 'syntax success oracle')
        matches = [not diagnostics]
    else:
        exact_keys(completion, ('kind', 'stage', 'code', 'message'), 'syntax diagnostic oracle')
        typed_equal(completion['kind'], 'error', 'syntax completion kind')
        first = diagnostics[0] if diagnostics else {}
        matches = [bool(diagnostics) and first.get('stage') == completion['stage']
                   and (completion['code'] is None or first.get('code') == completion['code'])]
        matcher = completion['message']
        if matcher is not None:
            exact_keys(matcher, ('kind', 'text'), 'syntax message oracle')
            require(matcher['kind'] in ('exact', 'contains') and type(matcher['text']) is str,
                    'unsupported syntax diagnostic matcher')
            actual = first.get('message', '')
            matches.append(actual == matcher['text'] if matcher['kind'] == 'exact'
                           else matcher['text'] in actual)
    require(type(ok['artifacts']) is list, 'syntax artifacts must be a list')
    actual = {}
    for ref in ok['artifacts']:
        exact_keys(ref, ('kind', 'logical', 'path', 'bytes', 'sha256'), 'syntax artifact')
        typed_equal(ref['kind'], 'syntax', 'syntax artifact kind')
        name = ref['logical']
        require(type(name) is str and name in {'reconstructed.at', 'outline.json', 'cst.canonical'}
                and name not in actual, 'extra/duplicate syntax artifact')
        relative = copy.path_from_original(ref['path'])
        typed_equal(relative, artifact.as_posix() + '/work/' + name, 'syntax artifact owner')
        path = copy.verify(relative)
        counter(ref['bytes'], 'syntax artifact bytes')
        typed_equal(ref['bytes'], path.stat().st_size, 'syntax artifact byte count')
        typed_equal(ref['sha256'], copy.files[relative]['sha256'], 'syntax artifact content')
        actual[name] = ref['sha256']
    require(actual.keys() == {'reconstructed.at', 'outline.json', 'cst.canonical'}, 'incomplete syntax artifacts')
    modes = expected['artifact_modes']
    require(type(modes) is list and len(modes) == len(expected['oracle_artifacts']), 'incomplete artifact modes')
    seen = set()
    for oracle, mode in zip(expected['oracle_artifacts'], modes):
        exact_keys(mode, ('path', 'compare'), 'syntax artifact mode')
        typed_equal(mode['path'], oracle['path'], 'current artifact mode identity')
        typed_equal(mode['compare'], 'bytes', 'this slice requires exact-byte oracle')
        require(oracle['path'] in actual and oracle['path'] not in seen, 'missing/duplicate syntax artifact oracle')
        seen.add(oracle['path'])
        matches.append(actual[oracle['path']] == oracle['sha256'])
    require(type(ok['trace']) is list and type(ok['metric_gaps']) is list, 'invalid syntax metrics shape')
    for span in ok['trace']:
        exact_keys(span, ('stage', 'wall_ms'), 'syntax span')
        require(type(span['stage']) is str and bool(span['stage']), 'invalid syntax stage')
        counter(span['wall_ms'], 'syntax span wall')
    require(all(type(gap) is str and bool(gap) for gap in ok['metric_gaps']), 'invalid metric gap')
    cache = ok['cache']
    exact_keys(cache, ('mode', 'hits', 'counters'), 'syntax cache record')
    typed_equal(cache['mode'], 'syntax:' + grammar + ':revision-0', 'fresh parse cache identity')
    require(type(cache['hits']) is list and type(cache['counters']) is list, 'invalid syntax cache records')
    for pairs, numeric in ((cache['hits'], False), (cache['counters'], True)):
        names = set()
        for pair in pairs:
            require(type(pair) is list and len(pair) == 2 and type(pair[0]) is str
                    and bool(pair[0]) and pair[0] not in names, 'invalid/duplicate syntax metric key')
            names.add(pair[0])
            if numeric:
                require(counter(pair[1], 'syntax counter') <= (1 << 32) - 1, 'syntax counter overflow')
            else:
                require(type(pair[1]) is str, 'syntax cache hit value must be text')
    # Preserve admitted request and refusal identities for the architecture
    # consumer. The full diagnostic list remains in the hashed original wire;
    # only its primary identity is retained in this bounded observation.
    primary = None if not diagnostics else {key: diagnostics[0][key]
        for key in ('stage', 'code', 'span_start', 'span_end')}
    return {'schema': 'ace.m1.syntax-parse-observation/2', 'source_sha256': source_sha,
            'response_sha256': response_sha, 'grammar': grammar,
            'case_id': expected['case'], 'check_id': expected['check'],
            'request_profile': request['profile'], 'entry': request['entry'],
            'capabilities': request['capabilities'], 'worker_sha256': wire['worker_sha256'],
            'primary_diagnostic': primary, 'reconstructed_sha256': actual['reconstructed.at'],
            'diagnostics': len(diagnostics), 'artifact_oracles': len(seen),
            'metric_gaps': ok['metric_gaps'], 'counters': cache['counters'],
            'judgement': 'PASS' if all(matches) else 'FAIL',
            'scope': 'parse, source reconstruction and diagnostic oracle; no incremental/performance claim'}
