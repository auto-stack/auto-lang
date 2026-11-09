"""Original daily emission linked to the independently admitted certified fileset.

This reader starts no tools. A daily emit is not an execution oracle; the caller
must separately admit the actual/reference certification for the same files.
"""
from pathlib import PurePosixPath
import re
import os

from ace_m1_arch_acceptance import counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_cert_evidence import admit_command, _relative
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _fnv1a64


def daily_emission(scope, inputs, certification_prefix, pin, producer):
    target = pin['target']
    require(target in ('rust', 'c'), 'daily target')
    original = pin['request']
    profile = original['profile']
    require(profile in ('core', 'scalar', 'record', 'text', 'text-record', 'collections'), 'finite daily source profile')
    prefix = str(PurePosixPath(certification_prefix).parent / ('daily-' + target))
    # Only the original captured argv supplies the daily nonce. No response,
    # child verdict, certificate echo or inferred Case Debug text supplies it.
    record, _ = scope.read_json(prefix + '.command.json')
    argv = record.get('argv')
    require(type(argv) is list and len(argv) == 2 and type(argv[1]) is str, 'original daily argv')
    request, _ = scope.decode_json(argv[1].encode())
    coll=profile=='collections'
    exact_keys(request, ('op', 'source_path', 'profile', 'entry', 'caps', 'out_dir', 'nonce') + (('mode',) if coll else ()), 'original daily request')
    nonce = request['nonce']
    require(type(nonce) is str and re.fullmatch('[0-9a-f]{48,64}', nonce) is not None,
            'original daily nonce')
    require(nonce != original['nonce'], 'daily and certification must be distinct operations')
    expected = {key:original[key] for key in ('source_path', 'profile', 'entry', 'caps')}
    if coll:expected['mode']=original['mode']
    expected.update(op='daily-emit' if target == 'rust' else 'daily-emit-c',
                    out_dir=str(scope.root.joinpath(*prefix.split('/'))), nonce=nonce)
    response, receipts = admit_command(scope, prefix + '.command.json', expected, producer)
    inputs.sha256(_relative(inputs, producer['worker_path']), producer['worker_sha256'])
    source_path = _relative(inputs, original['source_path'])
    inputs.sha256(source_path, pin['source_sha256'])
    source = inputs.read_bytes(source_path)
    # This finite Windows capture uses one owned launch process and the worker.
    # Reaped Job process count exposes extra child launches independently of
    # the worker's own spawn counters. No compiler is allowed in this daily leg.
    typed_equal(record['meter']['job_processes'], 2, 'daily owned launcher plus worker only')
    common = {'status', 'tool', 'protocol', 'recipe', 'mode', 'nonce', 'profile',
              'input_hash', 'source_sha256', 'entry_fn', 'manifest_sha256', 'files', 'spawned_rustc'}
    exact_keys(response, common | ({'target', 'spawned_cl'} if target == 'c' else set()), 'daily response')
    rt = profile in ('record','text','text-record')
    recipe = ('ac-a2r/' if target == 'rust' else 'ac-a2c/') + ('hir-collections-emit-v1' if coll else 'hir-record-text-emit-v1' if rt else 'hir-scalar-emit-v1')
    emitted_profile = ('auto.source.'+profile+'.draft' if rt or coll else 'auto.source.'+profile+'-i32.draft') if target == 'rust' else profile
    for key, want in dict(status='ok', tool='a2x-worker', protocol='a2xworker/1', recipe=recipe,
                          mode=expected['op'], nonce=nonce, profile=emitted_profile, entry_fn='fn_main',
                          input_hash=_fnv1a64(source), source_sha256=pin['source_sha256'], spawned_rustc=0).items():
        typed_equal(response[key], want, 'daily response ' + key)
    if target == 'c':
        typed_equal(response['target'], 'c', 'daily C target')
        typed_equal(response['spawned_cl'], 0, 'daily C spawn count')
    golden = pin['actual_files']
    names = {'main.rs', 'a2x_runtime.rs', 'generated.rs'} if target == 'rust' else {
        'main.c', 'a2x_c_coll_runtime.h' if coll else 'a2x_c_rt_runtime.h' if rt else 'a2x_c_runtime.h', 'generated.c'}
    exact_keys(golden, names | {'manifest.json'}, 'frozen certified fileset')
    for name, digest in golden.items(): scope.sha256(prefix + '/' + name, digest)
    # The daily output directory is dedicated to this emission. Reject an
    # unlisted executable, subdirectory or link without following it; inspect
    # at most the four declared members plus the first disallowed entry.
    directory = scope.path(prefix + '/manifest.json').parent
    seen_members = set()
    with os.scandir(directory) as entries:
        for entry in entries:
            require(len(seen_members) < len(golden) and entry.name in golden
                    and entry.is_file(follow_symlinks=False), 'undeclared daily output')
            seen_members.add(entry.name)
    typed_equal(seen_members, set(golden), 'complete daily output directory')
    typed_equal(response['manifest_sha256'], golden['manifest.json'], 'daily independently frozen manifest')
    manifest, _ = scope.read_json(prefix + '/manifest.json')
    schema = 'auto.a2x.' + ('rust' if target == 'rust' else 'c') + ('.collections' if coll else '.record-text' if rt else '') + '.fileset/1'
    exact_keys(manifest, {'schema', 'input_hash', 'entry', 'files'} | ({'source_sha256'} if coll else set()) | ({'execution_config'} if coll and target=='rust' else set()) | (
        {'source_profile', 'hir_domain', 'entry_fn', 'target'} if target == 'rust' else {'functions'}), 'daily manifest')
    typed_equal(manifest['schema'], schema, 'daily fileset family')
    typed_equal(manifest['input_hash'], _fnv1a64(source), 'daily manifest source hash')
    typed_equal(manifest['entry'], 'main', 'daily entry')
    if coll:typed_equal(manifest['source_sha256'],pin['source_sha256'],'daily collections full source')
    if target == 'rust':
        typed_equal(manifest['source_profile'], emitted_profile, 'daily Rust source profile')
        typed_equal(manifest['hir_domain'], 'collections-draft' if coll else profile+'-i32-draft', 'daily HIR domain')
        typed_equal(manifest['entry_fn'], 'fn_main', 'daily emitted entry')
        expected_target = dict(lang='rust', edition='2021', backend='auto-a2x-rust', backend_version='0.1.0')
        if rt or coll:expected_target['resource_runtime']='a2x-coll/1' if coll else 'a2x-rt/1'
        if coll:
            from ace_m1_cert_evidence import collections_execution_config
            typed_equal(manifest['execution_config'],collections_execution_config(original['mode']),'daily original collections configuration')
        typed_equal(manifest['target'], expected_target, 'daily Rust backend')
    else:
        functions = manifest['functions']
        require(type(functions) is list and bool(functions), 'daily C function metadata')
        seen = set()
        for function in functions:
            exact_keys(function, ('decl', 'sem_fp', 'origin'), 'daily C function')
            name = function['decl']
            require(type(name) is str and bool(name) and name not in seen, 'daily duplicate function')
            seen.add(name); hash_value(function['sem_fp'], 'daily function fingerprint')
            span = function['origin']
            require(type(span) is list and len(span) == 2 and counter(span[0], 'origin start')
                    <= counter(span[1], 'origin end') <= len(source), 'daily source span')
    for value, fields, label in ((response['files'], ('path', 'sha256', 'bytes'), 'response'),
                                 (manifest['files'], ('path', 'sha256'), 'manifest')):
        require(type(value) is list and len(value) == len(names), 'daily complete ' + label + ' files')
        seen = set()
        for ref in value:
            exact_keys(ref, fields, 'daily ' + label + ' file')
            name = ref['path']
            require(type(name) is str and name in names and name not in seen, 'daily foreign/duplicate file')
            seen.add(name); typed_equal(ref['sha256'], golden[name], 'daily certified file identity')
            if label == 'response':
                typed_equal(ref['bytes'], scope.path(prefix + '/' + name).stat().st_size, 'daily emitted bytes')
        typed_equal(seen, names, 'daily complete fileset')
    return dict(schema='ace.m1.daily-emission/1', target=target, command_sha256=receipts['command_sha256'],
                response_sha256=receipts['response_sha256'], manifest_sha256=golden['manifest.json'],
                nonce=nonce, compiler_spawn_zero=True, judgement='PASS')


def certified_daily_emission(scope, inputs, prefix, pin, producer, certification):
    """Both legs are required; certification alone never supplies daily evidence."""
    daily = daily_emission(scope, inputs, prefix, pin, producer)
    return dict(certification, daily_command_sha256=daily['command_sha256'],
                daily_response_sha256=daily['response_sha256'], daily_nonce=daily['nonce'],
                daily_compiler_spawn_zero=daily['compiler_spawn_zero'], daily_judgement=daily['judgement'])
