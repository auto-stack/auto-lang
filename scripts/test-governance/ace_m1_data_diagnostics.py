"""Original data source rejections; never infer codes from rendered messages."""
import base64
import binascii
import hashlib
import json
from pathlib import PurePosixPath,PureWindowsPath
from ace_m1_arch_acceptance import EvidenceScope,AdmissionError,exact_keys
from ace_m1_case_evidence import CaseCopy,admit_case_bindings,require
from ace_m1_legacy import typed_equal
from ace_m1_original_data_sources import source_contracts
from ace_m1_scalar_evidence import _fnv1a64,_duration,_counts
from ace_m1_text_q4_evidence import DEPENDENCIES,canonical_i32,owned_i32

CONTRACT='crates/auto-lang/test/cases/ace-m1-expanded/original-data-diagnostics-contract.json'
CONTRACT_SHA='ab4ebfef1227b9ed9a446c9614cf984ec86d7c3967d56e1c682468c53ddc477e'
SUITE='crates/auto-lang/test/cases/ace-m1-expanded/original-data-diagnostics.suite.ad'
ORACLES={'M09':('bind','source.not-callable','record'),
         'M10':('verify','verify.immutable-write','record'),
         'M11':('verify','verify.mut-conflict','record'),
         'M12':('bind','source.duplicate','text-record'),
         'R09':('verify','source.type-mismatch','record'),
         'R10':('verify','verify.use-after-move','text-record'),
         'T06':('verify','verify.borrow-escape','text-record'),
         'T07':('text','text.invalid-scalar','text-record')}


def diagnostic_span(source,case):
    """Source anchors follow resolver/typecheck/verifier diagnostic ownership."""
    if case=='T06':
        # text_resources anchors borrow-escape at the HIR block. adapter
        # joins live statement spans through the first terminator; both
        # legacy parser and CST project .to at the type word end, excluding
        # ')'. Derive from independent source anchors, never response spans.
        first=b'let s Str = "temporary"';last=b'return s.to(str)'
        require(source.count(first)==1 and source.count(last)==1,
                'unique original borrow-escape block anchors')
        start=source.index(first)
        end=source.index(last)+len(b'return s.to(str')
        require(start<end,'ordered original borrow-escape live statements')
        return dict(start=start,end=end)
    needles={'M09':(b'P.get(p)',2,3),'M10':(b'q.v = n + 1',0,11),
             'M11':(b'p.mix(p.mut, 3)',0,15),'M12':(b'fn clone',3,5),
             'R09':(b'B(42)',0,5),'R10':(b'a.code',0,1),'T07':(b'\\u{D800}',0,8)}
    require(case in needles,'finite original data diagnostic span')
    needle,offset,length=needles[case]
    require(source.count(needle)==1,'unique original data diagnostic anchor')
    start=source.index(needle)+offset
    return dict(start=start,end=start+length)


def diagnostic_observation(scope,wire,check,contract):
    av=check['check']=='av';data=contract['profile']=='record'
    recipe=('ac-av/data-run-v1' if data else 'ac-av/owned-run-v1') if av else 'ac/mir-native-scalar-v1'
    protocol='avworker/1' if av else 'mnworker/1'
    typed_equal((wire['protocol'],wire['recipe'],check['recipe']),(protocol,recipe,recipe),'original diagnostic producer domain')
    require(type(wire['exit_code']) is int and wire['exit_code']==0 and wire['transport_error'] is None
            and wire['stderr']==b'','original diagnostic transport terminal')
    request,_=scope.decode_json(wire['request'])
    exact_keys(request,{'recipe','nonce','source_b64','entry','caps','store_root','request_hash'}|
               ({'steps'} if av else {'workdir','diagnostics'}),'original data diagnostic request')
    typed_equal(request['recipe'],recipe,'original diagnostic recipe');typed_equal(request['entry'],'d_main','original diagnostic entry')
    typed_equal(request['caps'],json.loads(contract['caps']),'original diagnostic capabilities')
    require(type(request['nonce']) is str and bool(request['nonce']),'original diagnostic nonce')
    if av:typed_equal(request['steps'],[{'op':'run'}],'one original diagnostic operation')
    else:typed_equal(request['diagnostics'],'source-v1','explicit native typed diagnostic mode')
    for key in ('store_root',) if av else ('store_root','workdir'):
        require(type(request[key]) is str and bool(request[key]),'original diagnostic owned directory')
    unsigned={k:v for k,v in request.items() if k!='request_hash'}
    digest=hashlib.sha256(json.dumps(unsigned,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
    typed_equal(request['request_hash'],digest,'original diagnostic request identity')
    require(type(request['source_b64']) is str,'original diagnostic source frame')
    try:source=base64.b64decode(request['source_b64'],validate=True)
    except (ValueError,binascii.Error) as error:raise AdmissionError('invalid original diagnostic source') from error
    typed_equal(base64.b64encode(source).decode(),request['source_b64'],'canonical original diagnostic source')
    typed_equal(hashlib.sha256(source).hexdigest(),contract['source_sha256'],'independent original diagnostic source')
    response,response_sha=scope.decode_json(wire['stdout'])
    exact_keys(response,{'schema','nonce','request_hash','ok'}|(set() if av else {'diagnostic','source_error'}),'original diagnostic response')
    for key,want in [('schema',protocol),('nonce',request['nonce']),('request_hash',digest)]:
        typed_equal(response[key],want,'original diagnostic response '+key)
    ok=response['ok'];stage,code,_=ORACLES[contract['case']];authority='OBSERVED'
    if av:
        exact_keys(ok,('recipe','steps','worker_version','av_core_version'),'original AV diagnostic envelope')
        typed_equal(ok['worker_version'],'avworker/1.0','original AV version')
        typed_equal(ok['av_core_version'],'av.data/1' if data else 'av.owned/1','original AV source profile')
        require(type(ok['steps']) is list and len(ok['steps'])==1,'one original AV diagnostic terminal')
        step=ok['steps'][0];payload='typed' if data else 'owned'
        # The existing data diagnostic branch omits typed/trace; its
        # success branch includes both. Owned diagnostics retain owned:null.
        fields={'op','outcome','value','trap','diag','source_hash','wall_ms'}
        fields |= ({'typed','trace'} if step.get('outcome')!='diag' else set()) if data else {'owned'}
        exact_keys(step,fields,'original AV diagnostic step')
        typed_equal(step['op'],'run','original AV operation');typed_equal(step['source_hash'],_fnv1a64(source),'original AV source FNV')
        _duration(step['wall_ms'],'original AV diagnostic wall')
        if data and step['outcome']!='diag':typed_equal(step['trace'],[],'original AV negative trace')
        if step['outcome']=='diag':
            for key in ('value','trap') if data else ('value',payload,'trap'):
                typed_equal(step[key],None,'original AV diagnostic non-execution result')
            diag=step['diag'];exact_keys(diag,('stage','code','message'),'original AV diagnostic')
            require(diag['stage'] in ('frontend','verify') and type(diag['code']) is str
                    and type(diag['message']) is str and 0<len(diag['message'].encode())<=4096,'original AV diagnostic fields')
            observed=dict(kind='diagnostic' if diag['code'] else 'untyped-diagnostic',**diag)
            verdict='PASS' if diag['stage']=='frontend' and diag['code']==code else 'FAIL'
            if not diag['code']:
                authority='UNOBSERVED';verdict='UNOBSERVED'
        elif step['outcome']=='success':
            typed_equal((step['trap'],step['diag']),(None,None),'unexpected AV success fields')
            value=canonical_i32(step[payload]) if data else owned_i32(step[payload])
            typed_equal(step['value'],value,'unexpected AV canonical value')
            observed=dict(kind='unexpected-value',value=value);verdict='FAIL'
        else:
            typed_equal(step['outcome'],'trap','unexpected AV terminal kind')
            typed_equal((step['value'],step[payload],step['diag']),(None,None,None),'unexpected AV trap fields')
            require(type(step['trap']) is str and bool(step['trap']),'unexpected AV trap code')
            observed=dict(kind='unexpected-trap',code=step['trap']);verdict='FAIL'
    else:
        exact_keys(ok,('recipe','worker_version','mn_abi','mn_target','counters','spans','diagnostics_mode','diagnostics_schema'),'native original diagnosed result')
        for key,want in [('worker_version','mnworker/1.0'),('mn_abi','msvc-x64/priv-i32-bool-v1'),
                         ('mn_target','x86_64-pc-windows-msvc'),('diagnostics_mode','source-v1'),('diagnostics_schema','mn.source-diagnostic/1')]:
            typed_equal(ok[key],want,'native diagnostic '+key)
        typed_equal(ok['counters'],{'frontend_reject':1},'native rejected before codegen/link/run')
        exact_keys(ok['spans'],('frontend',),'native diagnostic spans');_counts(ok['spans'],'native diagnostic spans')
        from ace_m1_query_evidence import native_source_diagnostic
        observed,verdict=native_source_diagnostic(response,source,contract,data_family=True)
        if verdict=='UNOBSERVED':authority='UNOBSERVED'
    typed_equal(ok['recipe'],recipe,'original diagnostic result recipe')
    result=dict(schema='ace.m1.original-data-diagnostic/1',source_sha256=contract['source_sha256'],
                request_sha256=hashlib.sha256(wire['request']).hexdigest(),response_sha256=response_sha,
                expected=dict(kind='diagnostic',stage=stage,code=code),observed=observed,
                diagnostic_authority=authority,judgement=verdict)
    if authority=='UNOBSERVED':
        result['gap_id']='GAP-941-AV-DATA-DIAG-01' if av else 'GAP-941-DATA-BLOCK-SPAN-01'
        result['reason']='AV record diagnostic code is empty; rendered message is not authority.' if av else 'Independent exact T06 block-span oracle remains unverified.'
    return result


def admit_diagnostics(inputs,evidence,binding,workers):
    from ace_m1_first_slice import _inventory_for_suite,_compact_admission,_repo_relative,_wires,retain_parent_status
    exact_keys(binding,('inventory','copy','dependencies_before','dependencies_after','tool_capture'),'original data diagnostic binding')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT)
    contracts={case:row for case,row in source_contracts(inputs).items() if row['oracle_kind']=='diagnostic'}
    typed_equal(set(contracts),set(ORACLES),'all eight original data diagnostic sources')
    snapshots=[]
    for field,phase in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        record,_=evidence.read_json(binding[field]);exact_keys(record,('schema','snapshot','source_pins'),'original diagnostic dependency snapshot')
        typed_equal((record['schema'],record['snapshot']),('ace.m1.rt-q4-dependencies/1',phase),'original diagnostic dependency phase')
        exact_keys(record['source_pins'],DEPENDENCIES,'original diagnostic dependency closure')
        for name,digest in record['source_pins'].items():inputs.sha256(name,digest)
        snapshots.append(record['source_pins'])
    typed_equal(*snapshots,'stable original diagnostic dependencies')
    for key,path in [('av',DEPENDENCIES[0]),('native',DEPENDENCIES[1])]:typed_equal(snapshots[0][path],workers[key],'actual diagnostic worker')
    exact_keys(binding['tool_capture'],('path','sha256'),'diagnostic tool capture')
    evidence.sha256(binding['tool_capture']['path'],binding['tool_capture']['sha256']);parent,_=evidence.read_json(binding['tool_capture']['path'])
    typed_equal(parent.get('groups'),['case-ace-m1-original-data-diagnostics-941'],'original diagnostic formal group')
    typed_equal(parent.get('executed'),16,'original diagnostic formal denominator')
    required_tools={'av':DEPENDENCIES[0],'mn':DEPENDENCIES[1]};captured={}
    require(type(parent.get('tools')) is list,'original diagnostic prepared tools')
    for tool in parent['tools']:
        if tool.get('tool') in required_tools and 'path' in tool:
            path=required_tools[tool['tool']]
            typed_equal(_repo_relative(inputs,tool['path']),path,'original diagnostic prepared path')
            typed_equal(tool['sha256'],snapshots[0][path],'original diagnostic prepared SHA')
            captured[tool['tool']]=tool['sha256']
    typed_equal(set(captured),set(required_tools),'all original diagnostic prepared dependencies')
    inventory=_inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inventory);seen=set();observations={};non_pass=[]
    for check,parent in admitted['checks']:
        case=check['case'].removeprefix('ace.m1.original-data-diag.')
        require(case.upper() in contracts and check['case']=='ace.m1.original-data-diag.'+case
                and check['check'] in ('av','ac-native'),'original diagnostic check identity')
        contract=contracts[case.upper()];av=check['check']=='av'
        stage,code,_=ORACLES[contract['case']]
        typed_equal(check['completion'],dict(kind='error',stage='parse' if av else {'text':'parse','bind':'resolve','verify':'typecheck'}[stage],code=code,message=None),'frozen original diagnostic completion')
        typed_equal(check['expected_exit'],None,'diagnostic has no fabricated guest exit')
        typed_equal([_repo_relative(inputs,p) for p in check['input_paths']],[contract['source_file']],'original diagnostic source path')
        typed_equal(check['source_files'][0]['sha256'],contract['source_sha256'],'original diagnostic source bytes')
        options=check['probe_bindings']['av' if av else 'native']
        typed_equal(options['entry'],'d_main','original diagnostic entry');typed_equal(options['caps'],json.loads(contract['caps']),'original diagnostic source caps')
        if not av:typed_equal(options['diagnostics'],'source-v1','native explicit diagnostic mode')
        require(check['state']=='RUN' and check['adapter']=='probe' and check['action']=='run','selected original diagnostic execution')
        typed_equal(check['declared_probe_steps'],[{'op':'run'}],'original diagnostic operation')
        def read_one(_):
            wires=_wires(copy,check,'avworker/1' if av else 'mnworker/1',check['recipe'],workers['av' if av else 'native'])
            require(len(wires)==1,'one original diagnostic exchange');wire=wires[0]
            owner=PurePosixPath(wire['start_path']).parent.parent
            require(len(owner.parts)==3 and owner.parts[1:]==(check['case'],check['check']),'original diagnostic exchange owner')
            request,_=copy.scope.decode_json(wire['request'])
            typed_equal(PureWindowsPath(request['store_root']),copy.original.joinpath(*owner.parts,'store' if av else 'mn-store'),'original diagnostic store owner')
            if not av:typed_equal(PureWindowsPath(request['workdir']),copy.original.joinpath(*owner.parts,'mn-work'),'original diagnostic work owner')
            return retain_parent_status(diagnostic_observation(copy.scope,wire,check,contract),parent['status'])
        key='q4-source/'+contract['family']+'/'+contract['case']+'/'+('AV' if av else 'AC-native')
        require(key not in seen,'duplicate original diagnostic observation');seen.add(key)
        observations[key]=_compact_admission(inputs,read_one,32<<10)
        if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
    typed_equal(len(seen),16,'complete original diagnostic target set')
    return dict(schema='ace.m1.original-data-diagnostics/1',required=16,observations=observations,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                can_claim_expanded_m1=False)
