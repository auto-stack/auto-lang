"""Original text source AV/native observations, including real diagnosed failures."""
import base64
import binascii
import hashlib
import json
import re
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import AdmissionError, EvidenceScope, counter, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _duration, _fnv1a64, _counts
from ace_m1_text_evidence import text_contracts, ORACLES

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/text-q4-contract.json'
CONTRACT_SHA  = '3326238f02437a2150075703191107ef4cba974e4bbe6c7a62152a4223637d71'
SUITE = 'crates/auto-lang/test/cases/ace-m1-expanded/text-q4.suite.ad'
DEPENDENCIES = ('experimental/ace/target/release/av-worker.exe',
 'experimental/ace/target/release/mn-worker.exe',
 'experimental/ace/target/runtime-release/ac_text_runtime.lib',
 'experimental/ace/crates/av/src/text_runtime.rs',
 'experimental/ace/crates/text-runtime/src/lib.rs',
 'experimental/ace/crates/native/src/driver.rs',
 'experimental/ace/crates/native/src/config.rs',
 'experimental/ace/crates/native/src/cache.rs',
 'experimental/ace/crates/native/src/link_driver.rs',
 'experimental/ace/crates/av/src/owned_driver.rs',
 'experimental/ace/crates/av-core/src/contract.rs')
NATIVE_COUNTERS = 'codegen_emit_units codegen_emit_wrappers coff_objects_persisted ledger_peak_io ledger_peak_mir-symbol ledger_peak_staging ledger_peak_total link_invocations link_manifests_persisted mir_units_split mn_runs run_invocations store_puts'.split()
NATIVE_SPANS = 'cache_lookup codegen coff_persist frontend link link_identity manifest_persist mir_lower mir_split mir_verify run unit_fp'.split()
AV_TRAPS = {'TextIndex':'av.trap.str-index', 'InvalidScalar':'av.trap.char-invalid'}
# Hand-derived runtime operations for these three original source programs:
# literal allocation -> failed access -> release; invalid scalar has no buffer.
TRAP_TRACES = {'v03_char_at_negative':b'LIT len=3\nLIT id=1\nRT-FAIL code=-10\nFREE id=1\n',
 'v03_slice_oob':b'LIT len=6\nLIT id=1\nRT-FAIL code=-10\nFREE id=1\n',
 'v03_codepoint_surrogate':b'FROM-CP cp=55296\nRT-FAIL code=-8\n'}


def native_events(raw):
    require(type(raw) is bytes and len(raw) <= 1 << 20, 'bounded native text events')
    try: text = raw.decode('ascii', errors='strict')
    except UnicodeError: raise ValueError('non-ASCII native text events')
    require(not text or text.endswith('\n'), 'unterminated native event')
    allocated, freed, traps = set(), set(), []
    for line in text.splitlines():
        alloc = re.fullmatch(r'(LIT|CLONE|CONCAT) id=([0-9]{1,20})', line)
        free = re.fullmatch(r'FREE id=([0-9]{1,20})', line)
        failure = re.fullmatch(r'RT-FAIL code=(-[0-9]{1,10})', line)
        if alloc:
            identity = counter(int(alloc[2]), 'native allocation id');require(identity not in allocated, 'duplicate native allocation')
            allocated.add(identity)
        elif free:
            identity = counter(int(free[1]), 'native free id');require(identity in allocated and identity not in freed, 'foreign/duplicate native release')
            freed.add(identity)
        elif failure: traps.append(int(failure[1]))
        else:
            require(any(re.fullmatch(pattern, line) for pattern in (
                r'(LIT|CLONE) len=[0-9]+', r'TEXT-SCALARS n=[0-9]+',
                r'CHAR-AT i=-?[0-9]+ v=[0-9]+', r'FROM-CP cp=-?[0-9]+',
                r'SLICE s=-?[0-9]+ e=-?[0-9]+ len=[0-9]+',
                r'CONCAT a=[0-9]+ b=[0-9]+')),
                'unknown native text event')
    require(allocated == freed, 'unreleased native text allocation')
    require(len(traps) <= 1, 'multiple native runtime failures')
    return traps


def owned_i32(value):
    exact_keys(value, ('result', 'live_after_release', 'allocs', 'frees'), 'AV owned result')
    typed_equal(value['live_after_release'], 0, 'AV post-release live bytes')
    allocated, freed = set(), set()
    require(type(value['allocs']) is list and type(value['frees']) is list, 'AV owned events')
    for event in value['allocs']:
        exact_keys(event, ('id', 'bytes'), 'AV allocation')
        identity=counter(event['id'], 'AV allocation id');counter(event['bytes'], 'AV allocation bytes')
        require(identity not in allocated, 'duplicate AV allocation');allocated.add(identity)
    for event in value['frees']:
        exact_keys(event, ('id',), 'AV release')
        identity=counter(event['id'], 'AV free id')
        require(identity in allocated and identity not in freed, 'foreign/duplicate AV release');freed.add(identity)
    require(allocated == freed, 'missing AV release')
    return canonical_i32(value['result'])


def canonical_i32(result):
    exact_keys(result, ('kind','bits'), 'AV canonical value')
    typed_equal(result['kind'], 'i32', 'AV entry scalar kind')
    bits=result['bits'];require(type(bits) is str and re.fullmatch('[0-9a-f]{1,8}',bits) is not None, 'AV exact i32 bits')
    number=int(bits,16);typed_equal(bits,format(number,'x'),'AV canonical i32 bits')
    return number if number < 1 << 31 else number - (1 << 32)


def text_observation(scope, wire, check, contract, *, record_family=False, trap_trace=None):
    require(not record_family or contract.get('profile') in ('record','text','text-record'),
            'record/text original source profile')
    av=check['recipe'] in ('ac-av/owned-run-v1','ac-av/data-run-v1')
    data=record_family and contract['profile']=='record' and av
    av_recipe='ac-av/data-run-v1' if data else 'ac-av/owned-run-v1'
    typed_equal(check['recipe'], av_recipe if av else 'ac/mir-native-scalar-v1', 'text execution recipe')
    protocol='avworker/1' if av else 'mnworker/1'
    typed_equal(wire['protocol'],protocol,'text protocol');typed_equal(wire['recipe'],check['recipe'],'text wire recipe')
    require(type(wire['exit_code']) is int and wire['exit_code']==0 and wire['transport_error'] is None
            and wire['stderr']==b'', 'incomplete text worker exchange')
    request,_=scope.decode_json(wire['request'])
    exact_keys(request, {'recipe','nonce','source_b64','entry','caps','store_root','request_hash'} | ({'steps'} if av else {'workdir'}), 'text request')
    require(type(request['nonce']) is str and bool(request['nonce']), 'text request nonce')
    caps=scope.decode_json(contract['caps'].encode())[0] if record_family else []
    typed_equal(request['recipe'],check['recipe'],'text request recipe');typed_equal(request['entry'],'d_main','text entry');typed_equal(request['caps'],caps,'text capabilities')
    for key in ('store_root',) if av else ('store_root','workdir'):
        require(type(request[key]) is str and bool(request[key]), 'text owned directory')
    if av: typed_equal(request['steps'],[{'op':'run'}],'one source AV execution')
    unsigned={k:v for k,v in request.items() if k!='request_hash'}
    digest=hashlib.sha256(json.dumps(unsigned,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
    typed_equal(request['request_hash'],digest,'text request hash')
    require(type(request['source_b64']) is str, 'text source frame')
    try:source=base64.b64decode(request['source_b64'],validate=True)
    except (ValueError,binascii.Error) as error:raise AdmissionError('invalid text source encoding') from error
    typed_equal(base64.b64encode(source).decode(),request['source_b64'],'canonical text source frame')
    typed_equal(hashlib.sha256(source).hexdigest(),contract['source_sha256'],'original text source identity')
    response,response_sha=scope.decode_json(wire['stdout'])
    exact_keys(response, {'schema','nonce','request_hash','ok'} | ({'diagnostic'} if not av and 'diagnostic' in response else set()), 'text execution envelope')
    for key,want in [('schema',protocol),('nonce',request['nonce']),('request_hash',digest)]:typed_equal(response[key],want,'text response '+key)
    ok=response['ok'];kind=contract['oracle_kind'];oracle=contract['oracle_value'];matches=False
    if av:
        exact_keys(ok,('recipe','steps','worker_version','av_core_version'),'AV owned envelope')
        typed_equal(ok['worker_version'],'avworker/1.0','AV worker version');typed_equal(ok['av_core_version'],'av.data/1' if data else 'av.owned/1','AV source profile')
        require(type(ok['steps']) is list and len(ok['steps'])==1,'one AV owned terminal')
        payload='typed' if data else 'owned'
        fields={'op','outcome','value',payload,'trap','diag','source_hash','wall_ms'} | ({'trace'} if data else set())
        step=ok['steps'][0];exact_keys(step,fields,'AV source step')
        if data:typed_equal(step['trace'],[],'AV record source trace')
        typed_equal(step['op'],'run','AV source operation');typed_equal(step['source_hash'],_fnv1a64(source),'AV source FNV');_duration(step['wall_ms'],'AV wall')
        outcome=step['outcome'];require(outcome in ('success','trap','diag'),'AV language outcome')
        if outcome=='success':
            typed_equal(step['trap'],None,'AV success trap');typed_equal(step['diag'],None,'AV success diagnostic')
            value=canonical_i32(step['typed']) if data else owned_i32(step['owned']);typed_equal(step['value'],value,'AV canonical and scalar observation')
            observed={'kind':'value','value':value};matches=kind=='value' and value==int(oracle)
        elif outcome=='trap':
            typed_equal(step['value'],None,'AV trap value');typed_equal(step[payload],None,'AV trap source result');typed_equal(step['diag'],None,'AV trap diagnostic')
            require(type(step['trap']) is str and bool(step['trap']),'AV trap code')
            observed={'kind':'trap','code':step['trap']};matches=kind=='trap' and step['trap']==AV_TRAPS[oracle]
        else:
            for key in ('value',payload,'trap'):typed_equal(step[key],None,'AV diagnostic '+key)
            diag=step['diag'];exact_keys(diag,('stage','code','message'),'AV diagnostic')
            require(diag['stage'] in ('frontend','verify') and all(type(diag[k]) is str and bool(diag[k]) for k in ('code','message')),'AV diagnostic fields')
            observed={'kind':'diagnostic',**diag}
    else:
        common={'recipe','worker_version','mn_abi','mn_target','counters','spans'}
        diagnosed='diagnostic' in response
        exact_keys(ok, common if diagnosed else common | {'manifest_fp','exit_code','stderr_hex','stdout_hex'}, 'native text result')
        for key,want in [('worker_version','mnworker/1.0'),('mn_abi','msvc-x64/priv-i32-bool-v1'),('mn_target','x86_64-pc-windows-msvc')]:typed_equal(ok[key],want,'native '+key)
        exact_keys(ok['counters'], ['frontend_reject'] if diagnosed else NATIVE_COUNTERS,'native original counters')
        exact_keys(ok['spans'], ['frontend'] if diagnosed else NATIVE_SPANS,'native original spans')
        _counts(ok['counters'],'native counters');_counts(ok['spans'],'native spans')
        if diagnosed:
            typed_equal(ok['counters']['frontend_reject'],1,'native frontend rejection')
            diag=response['diagnostic'];exact_keys(diag,('code','message'),'native diagnostic')
            require(all(type(v) is str and bool(v) for v in diag.values()),'native diagnosed fields')
            observed={'kind':'diagnostic',**diag}
        else:
            require(type(ok['exit_code']) is int and -(1<<31)<=ok['exit_code']<(1<<31),'native i32 exit')
            require(type(ok['manifest_fp']) is str and re.fullmatch('[0-9a-f]{16}',ok['manifest_fp']) is not None,'native manifest identity')
            typed_equal(ok['stdout_hex'],'','native text stdout')
            raw=ok['stderr_hex'];require(type(raw) is str and len(raw)<=2<<20 and re.fullmatch('(?:[0-9a-f]{2})*',raw) is not None,'native raw stderr')
            events=bytes.fromhex(raw);traps=native_events(events)
            for key in ('link_invocations','mn_runs','run_invocations'):typed_equal(ok['counters'][key],1,'real native '+key)
            if traps:
                observed={'kind':'trap','runtime_code':traps[0],'exit':ok['exit_code']}
                expected_trace=trap_trace if record_family else TRAP_TRACES.get(contract['case'])
                matches=kind=='trap' and ok['exit_code']==70 and traps==[-10 if oracle=='TextIndex' else -8] and events==expected_trace
            else:
                observed={'kind':'value','value':ok['exit_code']};matches=kind=='value' and ok['exit_code']==int(oracle)
    typed_equal(ok['recipe'],check['recipe'],'text result recipe')
    return dict(schema='ace.m1.text-source-observation/1',source_sha256=contract['source_sha256'],
                request_sha256=hashlib.sha256(wire['request']).hexdigest(),response_sha256=response_sha,
                expected={'kind':kind,'value':oracle},observed=observed,judgement='PASS' if matches else 'FAIL')


def admit_text_q4(inputs,evidence,binding,workers):
    from ace_m1_first_slice import _inventory_for_suite,_wires,_repo_relative,retain_parent_status
    exact_keys(binding,('inventory','copy','dependencies_before','dependencies_after'),'text source execution binding')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT)
    contracts=text_contracts(inputs)
    before,_=evidence.read_json(binding['dependencies_before']);after,_=evidence.read_json(binding['dependencies_after'])
    for record,label in [(before,'before-execution'),(after,'after-execution')]:
        exact_keys(record,('schema','producer_commit','source_pins','snapshot'),'text runtime dependency snapshot')
        typed_equal(record['schema'],'ace.m1.text-q4-dependencies/1','dependency schema');typed_equal(record['snapshot'],label,'dependency snapshot phase')
        require(type(record['producer_commit']) is str and re.fullmatch('[0-9a-f]{40}',record['producer_commit']) is not None,'dependency producer revision')
        exact_keys(record['source_pins'],DEPENDENCIES,'text dependency closure')
        for name,digest in record['source_pins'].items():inputs.sha256(name,digest)
    typed_equal(before['source_pins'],after['source_pins'],'dependency identity across actual execution')
    typed_equal(before['producer_commit'],after['producer_commit'],'dependency execution revision')
    for key,name in [('av','av-worker.exe'),('native','mn-worker.exe')]:typed_equal(before['source_pins']['experimental/ace/target/release/'+name],workers[key],'current text worker dependency')
    inventory=_inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inventory);seen=set();observations={};non_pass=[]
    for check,parent in admitted['checks']:
        case=check['case'].removeprefix('ace.m1.textq4.')
        require(case in contracts and check['case']=='ace.m1.textq4.'+case and check['check'] in ('av','ac-native'),'foreign text source execution')
        av=check['check']=='av';target='AV' if av else 'AC-native';slot='q4-source/text-char-925/'+case+'/'+target
        require(slot not in seen,'duplicate text source target');seen.add(slot)
        contract=contracts[case];typed_equal([_repo_relative(inputs,name) for name in check['input_paths']],[contract['source_file']],'original text file')
        kind,oracle=ORACLES[case];typed_equal(contract['oracle_kind'],kind,'independent text kind');typed_equal(contract['oracle_value'],oracle,'independent text oracle')
        completion={'kind':'error','stage':'run','code':AV_TRAPS[oracle],'message':None} if av and kind=='trap' else {'kind':'success'}
        typed_equal(check['completion'],completion,'declared text source completion')
        typed_equal(check['expected_exit'],None if av and kind=='trap' else int(oracle) if kind=='value' else 70,'declared text value or original native trap exit')
        options=check['probe_bindings']['av' if av else 'native']
        typed_equal(options['entry'],'d_main','text source entry');typed_equal(options['caps'],[],'text source capabilities')
        wires=_wires(copy,check,'avworker/1' if av else 'mnworker/1',check['recipe'],workers['av' if av else 'native'])
        require(len(wires)==1,'one original text source exchange')
        typed_equal(check['declared_probe_steps'],[{'op':'run'}],'declared text run')
        require(check['adapter']=='probe' and check['action']=='run' and check['state']=='RUN','selected text source execution')
        wire=wires[0];artifact=PurePosixPath(wire['start_path']).parent.parent
        require(len(artifact.parts)==3 and artifact.parts[1:]==(check['case'],check['check']),'text exchange owner')
        request,_=copy.scope.decode_json(wire['request'])
        typed_equal(PureWindowsPath(request['store_root']),copy.original.joinpath(*artifact.parts,'store' if av else 'mn-store'),'text store owner')
        if not av:typed_equal(PureWindowsPath(request['workdir']),copy.original.joinpath(*artifact.parts,'mn-work'),'native text work owner')
        observations[slot]=retain_parent_status(text_observation(copy.scope,wires[0],check,contract),parent['status'])
        if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
    typed_equal(seen,{'q4-source/text-char-925/'+case+'/'+target for case in ORACLES for target in ('AV','AC-native')},'complete original ten-source execution set')
    return dict(schema='ace.m1.text-q4-observations/1',required=20,observations=observations,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                semantic_evidence_complete=all(v['judgement']=='PASS' for v in observations.values()),
                budgets_and_full_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)
