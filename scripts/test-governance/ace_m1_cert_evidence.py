"""Read original explicit scalar certification commands and raw observations.

Preparation/compilation belongs to the existing producer, never this reader.
Expectations are frozen current inputs supplied before certification, not cert
verdicts or a child's echoed oracle. Domain disagreement is refused explicitly.
"""
import hashlib
import json
from pathlib import PureWindowsPath
import re

from ace_m1_arch_acceptance import counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _duration, _fnv1a64


def _relative(scope, absolute):
    require(type(absolute) is str, 'absolute input name must be text')
    try:
        relative = PureWindowsPath(absolute).relative_to(PureWindowsPath(str(scope.root))).as_posix()
    except ValueError as error:
        from ace_m1_arch_acceptance import AdmissionError
        raise AdmissionError('command input lies outside the authorized root') from error
    scope.path(relative)
    return relative


def _raw_ref(scope, ref, relative):
    exact_keys(ref, ('path', 'sha256', 'bytes'), 'command raw file')
    typed_equal(ref['path'], relative, 'command raw path')
    counter(ref['bytes'], 'command raw bytes')
    path = scope.sha256(relative, ref['sha256'])
    require(path.stat().st_size == ref['bytes'], 'command raw byte count mismatch')
    return scope.read_bytes(relative)


def admit_command(scope, command_path, request, producer, *, expected_exit=0):
    require(type(expected_exit) is int and expected_exit in (0,2),
            'finite original command terminal contract')
    record, receipt_sha = scope.read_json(command_path)
    exact_keys(record, ('schema', 'argv', 'cwd', 'exit', 'meter', 'worker_sha256',
                        'producer_commit', 'stdout', 'stderr'), 'command receipt')
    typed_equal(record['schema'], 'ace.m1.command/1', 'command schema')
    for key in ('cwd', 'worker_sha256', 'producer_commit'):
        typed_equal(record[key], producer[key], f'current producer {key}')
    hash_value(record['worker_sha256'], 'worker SHA')
    require(type(record['producer_commit']) is str
            and re.fullmatch('[0-9a-f]{40}', record['producer_commit']) is not None,
            'producer revision must be a full commit')
    argv = record['argv']
    require(type(argv) is list and len(argv) == 2 and type(argv[1]) is str,
            'explicit worker command requires one JSON request')
    typed_equal(argv[0], producer['worker_path'], 'current worker path')
    actual_request, _ = scope.decode_json(argv[1].encode())
    typed_equal(actual_request, request, 'frozen original request')
    require(type(record['exit']) is int and record['exit'] == expected_exit, 'command did not complete cleanly')
    meter = record['meter']
    exact_keys(meter, ('argv', 'exit', 'error', 'wall_ms', 'workload_cpu_ms', 'meter_cpu_ms',
                       'total_cpu_ms', 'peak_process_bytes', 'peak_job_bytes', 'job_processes'),
               'original Job meter')
    typed_equal(meter['argv'], argv, 'meter original argv')
    require(type(meter['exit']) is int and meter['exit'] == expected_exit and meter['error'] is None,
            'command Job failed or did not reap descendants')
    for key in ('wall_ms', 'workload_cpu_ms', 'meter_cpu_ms', 'total_cpu_ms'):
        _duration(meter[key], 'Job ' + key)
    require(meter['wall_ms'] <= 360_000, 'certification exceeds original 360s wall')
    require(abs(meter['total_cpu_ms'] - meter['workload_cpu_ms'] - meter['meter_cpu_ms']) < 0.01,
            'inconsistent original CPU accounting')
    for key in ('peak_process_bytes', 'peak_job_bytes', 'job_processes'):
        counter(meter[key], 'Job ' + key)
    require(meter['job_processes'] > 0, 'missing original Job process accounting')
    require(meter['peak_process_bytes'] > 0 and meter['peak_job_bytes'] >= meter['peak_process_bytes'],
            'missing/inconsistent original Job memory accounting')
    require(command_path.endswith('.command.json'), 'invalid command receipt suffix')
    prefix = command_path.removesuffix('.command.json')
    stdout = _raw_ref(scope, record['stdout'], prefix + '.stdout')
    stderr = _raw_ref(scope, record['stderr'], prefix + '.stderr')
    require(stderr == b'', 'worker emitted infrastructure stderr')
    response, response_sha = scope.decode_json(stdout)
    return response, {'command_sha256': receipt_sha, 'response_sha256': response_sha,
                      'wall_ms': meter['wall_ms']}


def _terminal(value, where):
    exact_keys(value, ('exit_code', 'stdout_hex', 'stderr_hex'), where)
    require(type(value['exit_code']) is int and value['exit_code'] == 0,
            f'{where}: failed compile/run terminal')
    for role in ('stdout_hex', 'stderr_hex'):
        raw = value[role]
        require(type(raw) is str and len(raw) % 2 == 0 and len(raw) <= 2 << 20
                and all(c in '0123456789abcdef' for c in raw), f'{where}: invalid raw {role}')


def _toolchain(value, target):
    if target == 'rust':
        exact_keys(value, ('kind', 'version_vv', 'version_vv_sha256', 'flags'), 'Rust compiler identity')
        typed_equal(value['kind'], 'rustc', 'Rust compiler kind')
        require(type(value['version_vv']) is str and bool(value['version_vv']), 'observed Rust compiler version')
        typed_equal(value['version_vv_sha256'], hashlib.sha256(value['version_vv'].encode()).hexdigest(),
                    'observed Rust compiler version bytes')
        typed_equal(value['flags'], ['--edition', '2021'], 'original Rust compiler flags')
    else:
        exact_keys(value, ('kind', 'version', 'version_sha256', 'vcvars_sha256', 'flags'), 'C compiler identity')
        typed_equal(value['kind'], 'cl', 'C compiler kind')
        require(type(value['version']) is str and bool(value['version']), 'observed C compiler version')
        # The original locator names these fields sha256 but hashes them with
        # BLAKE3. Preserve its identity without pretending to recompute SHA256.
        hash_value(value['version_sha256'], 'original C locator version digest')
        hash_value(value['vcvars_sha256'], 'original C locator setup digest')
        typed_equal(value['flags'], ['-nologo', '-std:c11', '-TC', '-W4', '-WX', '-O2', '-MT'],
                    'original C compiler flags')


def process_matches(observations, stdout_hex, stderr_hex):
    """Each original leg must match the independent bytes, including trap names."""
    exact_keys(observations, ('schema', 'actual', 'reference'), 'cert process observations')
    typed_equal(observations['schema'], 'ace.a2x.cert-execution/1', 'cert process schema')
    matches = []
    for leg in ('actual', 'reference'):
        exact_keys(observations[leg], ('compile', 'run'), leg + ' process pair')
        for phase in ('compile', 'run'):
            _terminal(observations[leg][phase], leg + ' ' + phase)
        run = observations[leg]['run']
        matches.append(run['stdout_hex'] == stdout_hex and run['stderr_hex'] == stderr_hex)
    return matches


def scalar_certification(scope, inputs, prefix, pin, producer):
    return _certification(scope, inputs, prefix, pin, producer)


def text_certification(scope, inputs, prefix, pin, producer, contract):
    """The caller supplies a current, independently admitted text reference row."""
    typed_equal(contract['profile'], 'text', 'supported text certification profile')
    typed_equal(contract['caps'], '[]', 'supported text certification capabilities')
    require(contract['oracle_kind'] in ('value', 'trap'), 'text oracle kind')
    return _certification(scope, inputs, prefix, pin, producer, contract, text_family=True)


def cte_certification(scope, inputs, prefix, pin, producer, contract):
    """Only the current finite source-CT reference contract supplies authority."""
    typed_equal(contract['profile'], 'scalar', 'source CT profile')
    typed_equal(contract['oracle_kind'], 'value', 'source CT value oracle')
    caps, _ = inputs.decode_json(contract['caps'].encode())
    require(type(caps) is list and caps and caps[0] == 'ac.source.cte.scalar/v1'
            and all(type(cap) is str and cap in
            ('ac.source.cte.scalar/v1', 'cte.meta.fields/v1', 'ac.source.instances.scalar/v1')
            for cap in caps) and len(set(caps)) == len(caps), 'finite source CT capabilities')
    return _certification(scope, inputs, prefix, pin, producer, contract)


def scalar_corpus_certification(scope, inputs, prefix, pin, producer, contract):
    """Frozen original core/scalar corpus, including checked traps and marks."""
    require(contract['profile'] in ('core','scalar'), 'original scalar corpus profile')
    require(contract['oracle_kind'] in ('value','trap'), 'original scalar corpus oracle')
    caps, _ = inputs.decode_json(contract['caps'].encode())
    typed_equal(caps, ['hir.test.trace.v1'] if contract['trace']=='true' else [], 'original scalar trace capability')
    return _certification(scope, inputs, prefix, pin, producer, contract, scalar_family=True)


def query_certification(scope,inputs,prefix,pin,producer,contract):
    typed_equal(contract['profile'],'scalar','original query concrete domain')
    require(contract['family'] in ('source-query','source-instances') and contract['oracle_kind']=='value','finite original query certification domain')
    caps,_=inputs.decode_json(contract['caps'].encode())
    expected=['ac.source.instances.scalar/v1'] if contract['family']=='source-instances' else ['hir.test.trace.v1'] if contract['case']=='trace-ab7' else []
    typed_equal(caps,expected,'original query certification capabilities')
    return _certification(scope,inputs,prefix,pin,producer,contract,scalar_family=True)


def record_text_certification(scope,inputs,prefix,pin,producer,contract):
    require(contract['profile'] in ('record','text','text-record'),'original record/text profile')
    caps,_=inputs.decode_json(contract['caps'].encode())
    require(caps in ([],['ac.runtime.text.v1']),'original record/text capability set')
    require(contract['oracle_kind'] in ('value','trap'),'original record/text oracle')
    return _certification(scope,inputs,prefix,pin,producer,contract,record_family=True)


def collections_execution_config(mode):
    require(type(mode) is str and mode in ('debug','release'),'original collections config mode')
    return dict(budget_work_units=16777216,cancel=False,events=False,fault_alloc=None,mode=mode)


def collections_certificate_config(certificate,target,mode):
    require(target in ('rust','c'),'collections certificate target')
    expected=collections_execution_config(mode)
    typed_equal(certificate['execution_config'],json.dumps(expected,separators=(',',':')) if target=='rust' else expected,
                'original canonical collections execution configuration')


def collections_certification(scope,inputs,prefix,pin,producer,contract):
    typed_equal(contract['profile'],'collections','original collections certification profile')
    typed_equal(contract['caps'],'[]','original collections certification caps')
    typed_equal(contract['mode'],'debug','original default collections certification mode')
    require(contract['oracle_kind']=='value' or contract['oracle_kind']=='trap' and contract['oracle_value']=='ListIndex','original collections certification oracle')
    return _certification(scope,inputs,prefix,pin,producer,contract,collections_family=True)


def _certification(scope, inputs, prefix, pin, producer, contract=None, *, text_family=False, scalar_family=False, record_family=False, collections_family=False):
    rt = text_family or record_family
    kind = contract['oracle_kind'] if contract is not None else 'value'
    expected_oracle = contract['oracle_value'] if contract is not None else None
    caps = inputs.decode_json(contract['caps'].encode())[0] if contract is not None else []
    profile = contract['profile'] if scalar_family or record_family or collections_family else 'text' if rt else 'scalar'
    hir_domain = 'collections-draft' if collections_family else profile+'-i32-draft'
    target = pin['target']
    require(target in ('rust', 'c'), 'unsupported certification target')
    worker_relative = _relative(inputs, producer['worker_path'])
    inputs.sha256(worker_relative, producer['worker_sha256'])
    request = pin['request']
    exact_keys(request, ('op', 'source_path', 'profile', 'entry', 'caps', 'out_dir',
                         'reference_path', 'oracle_kind', 'case_id', 'nonce',
                         'oracle_value' if kind == 'value' else 'oracle_trap') + (('mode',) if collections_family else ()),
               'scalar certification request')
    for key, want in (('op', 'certify' if target == 'rust' else 'certify-c'),
                      ('profile', profile), ('entry', 'main'), ('caps', caps), ('oracle_kind', kind)):
        typed_equal(request[key], want, f'scalar certification {key}')
    if collections_family:typed_equal(request['mode'],contract['mode'],'frozen collections mode')
    if kind == 'value':
        require(type(request['oracle_value']) is int and -(1 << 31) <= request['oracle_value'] < (1 << 31),
                'independent scalar i32 oracle required')
        oracle = str(request['oracle_value'])
    else:
        allowed_traps = ('Overflow','DivByZero','DivOverflow','RemOverflow','RemByZero') if scalar_family else ('ListIndex',) if collections_family else ('TextIndex','InvalidScalar')
        require(request['oracle_trap'] in allowed_traps, 'registered trap identity required')
        oracle = request['oracle_trap']
    if contract is not None:
        typed_equal(oracle, expected_oracle, 'independent registered oracle')
    require(type(request['case_id']) is str and bool(request['case_id'])
            and type(request['nonce']) is str and bool(request['nonce']), 'missing request identity')
    typed_equal(pin['expected_stdout_hex'], f'{kind}:{oracle}\n'.encode().hex(),
                'independent oracle bytes')
    marks = inputs.decode_json(contract['marks'].encode())[0] if scalar_family else []
    require(type(marks) is list and all(type(mark) is str and mark in ('mark_a','mark_b') for mark in marks),
            'independent scalar side-effect oracle')
    typed_equal(pin['expected_stderr_hex'], ''.join(mark+'\n' for mark in marks).encode().hex(), 'scalar side-effect oracle')
    source_path = _relative(inputs, request['source_path'])
    reference_path = _relative(inputs, request['reference_path'])
    if contract is not None:
        typed_equal(source_path, contract['source_file'], 'registered source')
        typed_equal(reference_path, contract[target + '_reference'], 'registered independent reference')
        typed_equal(pin['source_sha256'], contract['source_sha256'], 'registered source hash')
        typed_equal(pin['reference_sha256'], contract[target + '_reference_sha256'], 'registered reference hash')
    inputs.sha256(source_path, pin['source_sha256'])
    inputs.sha256(reference_path, pin['reference_sha256'])
    source = inputs.read_bytes(source_path)
    # The output directory is only interpreted beneath the authorized root;
    # it is not an arbitrary absolute path supplied by a certificate.
    output_prefix = PureWindowsPath(request['out_dir']).relative_to(PureWindowsPath(str(scope.root))).as_posix()
    typed_equal(output_prefix, prefix, 'frozen output directory')
    identity, _ = admit_command(scope, str(PureWindowsPath(prefix).parent / ('identity-' + target)).replace('\\', '/')
                                + '.command.json', pin['identity_request'], producer)
    exact_keys(identity, ('status', 'tool', 'protocol', 'recipe', 'recipes', 'backend',
                          'backend_version', 'edition', 'targets', 'c_backend', 'c_standard',
                          'certification_toolchain'), 'explicit current tool identity')
    for key, want in (('status', 'ok'), ('tool', 'a2x-worker'), ('protocol', 'a2xworker/1')):
        typed_equal(identity[key], want, 'current identity ' + key)
    for key, want in {'recipe':'ac-a2r/hir-scalar-emit-v1', 'backend':'auto-a2x-rust',
                      'backend_version':'0.1.0', 'edition':'2021', 'targets':['rust', 'c'],
                      'c_backend':'auto-a2x-c', 'c_standard':'c11'}.items():
        typed_equal(identity[key], want, 'current worker identity ' + key)
    typed_equal(identity['recipes'], ['ac-a2r/hir-scalar-emit-v1', 'ac-a2r/hir-record-text-emit-v1',
                'ac-a2r/hir-collections-emit-v1', 'ac-a2c/hir-scalar-emit-v1',
                'ac-a2c/hir-record-text-emit-v1'], 'current declared worker recipes')
    _toolchain(identity['certification_toolchain'], target)
    typed_equal(identity['certification_toolchain'], pin['toolchain'], 'pre-frozen compiler identity')
    response, receipts = admit_command(scope, prefix + '.command.json', request, producer)
    common = {'status', 'tool', 'protocol', 'recipe', 'mode', 'case', 'verdict',
              'cert_sha256', 'execution_observations'}
    rust_fields = {'actual_observed', 'reference_observed', 'oracle', 'marks_match_reference',
                   'actual_wall_ms', 'reference_wall_ms', 'files'}
    c_fields = {'target', 'nonce', 'oracle_kind', 'oracle_value', 'actual_stdout',
                'reference_stdout', 'cert_path', 'spawned_cl', 'spawned_rustc'}
    exact_keys(response, common | (rust_fields if target == 'rust' else c_fields), 'certification response')
    recipe = ('ac-a2r/' if target == 'rust' else 'ac-a2c/') + ('hir-collections-emit-v1' if collections_family else 'hir-record-text-emit-v1' if rt else 'hir-scalar-emit-v1')
    case = request['case_id'] if target == 'rust' else ('a2c-hir-926/' if collections_family else 'a2c-hir-925/' if rt else 'a2c-hir-921/') + request['case_id']
    for key, want in (('status', 'ok'), ('tool', 'a2x-worker'), ('protocol', 'a2xworker/1'),
                      ('recipe', recipe), ('mode', request['op']), ('case', case)):
        typed_equal(response[key], want, 'certification response ' + key)
    if target == 'c':
        typed_equal(response['nonce'], request['nonce'], 'C certification nonce')
        typed_equal(response['target'], 'c', 'C certification target')
        typed_equal(response['cert_path'], str(scope.root.joinpath(*prefix.split('/'), 'case.cert.json')),
                    'C cert path')
        typed_equal(response['spawned_cl'], 2, 'C original compile count')
        typed_equal(response['spawned_rustc'], 0, 'C foreign compiler count')
    certificate, cert_sha = scope.read_json(prefix + '/case.cert.json')
    typed_equal(cert_sha, response['cert_sha256'], 'certificate content')
    fields = {'schema', 'recipe', 'case', 'profile', 'hir_domain', 'entry', 'verdict',
              'oracle_kind', 'oracle_value', 'input_hash', 'source_sha256', 'actual_files',
              'wrapper_sha256', 'reference_sha256', 'backend', 'backend_version',
              'shared_view_version', 'protocol', 'toolchain', 'capabilities'}
    exact_keys(certificate, fields | ({'golden_sha256', 'execution_config'} if target == 'rust' else {'standard'} | ({'execution_config'} if collections_family else set())),
               'scalar certificate')
    schema = ('ace.a2r-hir.record-text.cert/1' if target == 'rust' else 'ace.a2c-hir.record-text.cert/1') if rt else (
        'ace.a2r-hir.cert/1' if target == 'rust' else 'ace.a2c-hir.scalar.cert/1')
    if collections_family:schema='ace.a2r-hir.collections.cert/1' if target=='rust' else 'ace.a2c-hir.collections.cert/1'
    for key, want in (('schema', schema),
                      ('recipe', recipe), ('case', case), ('entry', 'main'), ('oracle_kind', kind),
                      ('oracle_value', oracle), ('input_hash', _fnv1a64(source)),
                      ('source_sha256', pin['source_sha256']), ('reference_sha256', pin['reference_sha256']),
                      ('backend', 'auto-a2x-rust' if target == 'rust' else 'auto-a2x-c'),
                      ('backend_version', '0.1.0'), ('shared_view_version', '0.1.0'),
                      ('protocol', 'a2x-observation/1'), ('toolchain', pin['toolchain']), ('capabilities', caps)):
        typed_equal(certificate[key], want, 'scalar certificate ' + key)
    if target == 'rust':
        typed_equal(certificate['golden_sha256'], None, 'no substituted golden')
        if not collections_family:typed_equal(certificate['execution_config'], None, 'scalar configuration')
    else:
        typed_equal(certificate['standard'], 'c11', 'C language standard')
    if collections_family:collections_certificate_config(certificate,target,contract['mode'])
    golden = pin['actual_files']
    names = ({'main.rs', 'a2x_runtime.rs', 'generated.rs', 'manifest.json'} if target == 'rust'
             else {'main.c', 'a2x_c_coll_runtime.h' if collections_family else 'a2x_c_rt_runtime.h' if rt else 'a2x_c_runtime.h', 'generated.c', 'manifest.json'})
    require(type(golden) is dict and golden.keys() == names, 'complete frozen scalar fileset required')
    for name, digest in golden.items():
        require(type(name) is str and '/' not in name and '\\' not in name, 'flat scalar fileset required')
        scope.sha256(prefix + '/' + name, digest)
    actual_files = certificate['actual_files']
    require(type(actual_files) is list, 'certificate actual files must be a list')
    seen = set()
    for ref in actual_files:
        exact_keys(ref, ('path', 'sha256'), 'certificate actual file')
        name = ref['path']
        require(type(name) is str and name in golden and name != 'manifest.json' and name not in seen,
                'missing/extra/duplicate actual file binding')
        typed_equal(ref['sha256'], golden[name], 'independent actual file hash')
        seen.add(name)
    require(seen == golden.keys() - {'manifest.json'}, 'certificate omits actual files')
    manifest, _ = scope.read_json(prefix + '/manifest.json')
    if rt or collections_family:
        common_manifest = {'schema', 'input_hash', 'entry', 'files'}
        if collections_family:common_manifest |= {'source_sha256'} | ({'execution_config'} if target=='rust' else set())
        exact_keys(manifest, common_manifest | ({'source_profile', 'hir_domain', 'entry_fn', 'target'}
                   if target == 'rust' else {'functions'}), 'text fileset manifest')
        typed_equal(manifest['schema'], 'auto.a2x.' + ('rust' if target == 'rust' else 'c')
                    + ('.collections.fileset/1' if collections_family else '.record-text.fileset/1'), 'text manifest family')
        typed_equal(manifest['input_hash'], _fnv1a64(source), 'text manifest source identity')
        typed_equal(manifest['entry'], 'main', 'text manifest entry')
        if collections_family:
            typed_equal(manifest['source_sha256'],pin['source_sha256'],'collections full source identity')
            if target=='rust':typed_equal(manifest['execution_config'],collections_execution_config(contract['mode']),'collections manifest config identity')
        typed_equal(manifest['files'], actual_files, 'manifest and certificate complete fileset')
        if target == 'rust':
            typed_equal(manifest['entry_fn'], 'fn_main', 'text emitted entry')
            typed_equal(manifest['target'], {'lang':'rust', 'edition':'2021', 'backend':'auto-a2x-rust',
                        'backend_version':'0.1.0', 'resource_runtime':'a2x-coll/1' if collections_family else 'a2x-rt/1'}, 'text target identity')
        else:
            require(type(manifest['functions']) is list and bool(manifest['functions']), 'text manifest functions')
            decls = set()
            for function in manifest['functions']:
                exact_keys(function, ('decl', 'sem_fp', 'origin'), 'text emitted function')
                name = function['decl']
                require(type(name) is str and bool(name) and name not in decls, 'duplicate text function')
                decls.add(name)
                hash_value(function['sem_fp'], 'text function fingerprint')
                span = function['origin']
                require(type(span) is list and len(span) == 2 and counter(span[0], 'origin start')
                        <= counter(span[1], 'origin end') <= len(source), 'text function origin')
    if target == 'rust':
        typed_equal(certificate['profile'], manifest['source_profile'], 'actual profile identity')
        typed_equal(certificate['hir_domain'], manifest['hir_domain'], 'actual HIR domain identity')
        typed_equal(manifest['hir_domain'], hir_domain, 'current HIR domain')
        if scalar_family:
            typed_equal(manifest['source_profile'], 'auto.source.'+profile+'-i32.draft', 'original scalar profile')
        if rt or collections_family:
            typed_equal(manifest['source_profile'], 'auto.source.'+profile+'.draft', 'current record/text profile')
        elif contract is not None and not scalar_family:
            typed_equal(manifest['source_profile'], 'auto.source.scalar-i32.draft', 'current CT scalar profile')
    else:
        # The original C scalar fileset /1 has no profile/domain fields.
        # Its pinned producer binds the actual CheckedBundle domain into cert.
        typed_equal(certificate['profile'], profile, 'C source profile')
        typed_equal(certificate['hir_domain'], hir_domain, 'C HIR domain')
    wrapper = 'main.rs' if target == 'rust' else 'main.c'
    typed_equal(certificate['wrapper_sha256'], golden[wrapper], 'complete actual wrapper')
    reference_copy = 'reference.rs' if target == 'rust' else '_reference/reference.c'
    scope.sha256(prefix + '/' + reference_copy, pin['reference_sha256'])
    for value, where in ((response['verdict'], 'response'), (certificate['verdict'], 'certificate')):
        require(type(value) is str and value in ('PASS', 'REJECTED'), 'invalid ' + where + ' verdict type')
    if target == 'rust':
        for field in ('actual_wall_ms', 'reference_wall_ms'):
            counter(response[field], field)
            require(response[field] <= 150_000, 'original compile/run leg budget exceeded')
        require(type(response['marks_match_reference']) is bool, 'invalid marks metadata type')
        rows = response['files']
        require(type(rows) is list and len(rows) == 3, 'incomplete Rust response fileset')
        response_files = set()
        for ref in rows:
            exact_keys(ref, ('path', 'sha256', 'bytes'), 'Rust response file')
            name = ref['path']
            require(type(name) is str and name in names - {'manifest.json'} and name not in response_files,
                    'extra/duplicate Rust response file')
            typed_equal(ref['sha256'], golden[name], 'Rust response file hash')
            typed_equal(ref['bytes'], scope.path(prefix + '/' + name).stat().st_size, 'Rust response file bytes')
            response_files.add(name)
        typed_equal(response['oracle'], f'{kind}:{oracle}', 'Rust oracle echo')
    else:
        typed_equal(response['oracle_kind'], kind, 'C oracle kind echo')
        typed_equal(response['oracle_value'], oracle, 'C oracle value echo')
    observations = response['execution_observations']
    matches = process_matches(observations, pin['expected_stdout_hex'], pin['expected_stderr_hex'])
    for leg in ('actual', 'reference'):
        raw = observations[leg]['run']
        display = bytes.fromhex(raw['stdout_hex']).decode('utf-8', errors='replace').strip()
        typed_equal(response[leg + ('_observed' if target == 'rust' else '_stdout')], display,
                    leg + ' displayed observation linkage')
    if target == 'rust':
        a = observations['actual']['run']['stderr_hex']
        b = observations['reference']['run']['stderr_hex']
        # Original Rust display trims lossy text before comparing marks.
        displayed = lambda h: bytes.fromhex(h).decode('utf-8', errors='replace').strip()
        typed_equal(response['marks_match_reference'], displayed(a) == displayed(b), 'marks/raw linkage')
    # Neither a child PASS nor its oracle/marks echo supplies positive authority;
    # explicit rejection is never promoted even if partial observations agree.
    matches.append(response['verdict'] == 'PASS' and certificate['verdict'] == 'PASS')
    return {'schema': 'ace.m1.collections-certification/1' if collections_family else 'ace.m1.text-certification/1' if rt else 'ace.m1.scalar-certification/1', 'target': target,
            'certificate_sha256': cert_sha, **receipts,
            'actual_matches_oracle': matches[0], 'reference_matches_oracle': matches[1],
            'judgement': 'PASS' if all(matches) else 'FAIL'}
