"""Original twenty collections source observations; no execution or synthetic ledger."""
import base64
import binascii
import hashlib
import json
import re
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import AdmissionError, EvidenceScope, counter, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import (BASE, Q4_CATALOG, Q4_CATALOG_SHA, Q4_FIELDS,
    COMPOSITE_CATALOG, COMPOSITE_CATALOG_SHA, COMPOSITE_FIELDS, read_tsv)
from ace_m1_scalar_evidence import _duration, _fnv1a64, _counts

CONTRACT = BASE+'collections-q4-contract.json'
CONTRACT_SHA = 'b31cc462bc0b9c9be45a105569e2074ba710f02f6dabd42946efc5c2ee8d45ec'
SUITE = BASE+'collections-q4.suite.ad'
AV_RECIPE = 'ac-av/collections-source-run-v1'
MN_RECIPE = 'ac/mir-native-collections-v1'
RUNTIME_STAMP = 'experimental/ace/target/x86_64-pc-windows-msvc/runtime-release/ace_collections_runtime.identity.json'
DEPENDENCIES = ('experimental/ace/target/release/av-worker.exe', 'experimental/ace/target/release/mn-collections-worker.exe', 'experimental/ace/target/x86_64-pc-windows-msvc/runtime-release/ace_collections_runtime.lib', 'experimental/ace/target/x86_64-pc-windows-msvc/runtime-release/ace_collections_runtime.identity.json', 'experimental/ace/crates/av/src/collections_864_support.rs', 'experimental/ace/crates/native/src/collections_driver.rs', 'experimental/ace/crates/native/src/collections_runtime.rs', 'experimental/ace/crates/native/src/collections_incremental.rs', 'experimental/ace/crates/native/src/collections_layout.rs', 'experimental/ace/crates/native/src/collections_lower.rs', 'experimental/ace/crates/native/src/link_driver.rs', 'experimental/ace/crates/av-core/src/contract.rs')
COUNTERS = 'codegen_emit_units codegen_emit_wrappers coff_objects_persisted ledger_peak_io ledger_peak_mir-symbol ledger_peak_staging ledger_peak_total link_invocations link_manifests_persisted mir_units_encoded run_invocations runtime_cargo_invocations store_puts'.split()
SPANS = 'cache_lookup codegen coff_persist cold_closure_admit frontend link link_identity manifest_persist mir_encode mir_persist resident_admit resident_meta run runtime_persist runtime_prepare source_persist unit_fp'.split()
MN_FIELDS = ('recipe worker_version mn_abi worker_memory mn_target collections_abi collections_codegen counters spans allocation_meter runtime_identity link_identity op exit_code stderr_hex stdout_hex manifest_fp manifest_id wall_ms changed_fn codegen_units cache_hits reused_history_unit refused infra unaffected_stage_delta whole_module_stage_delta fallback_whole_lower parsed_functions bodies ledger_peaks').split()
SUMMARY_KEYS = 'process_private_base process_private_end process_commit_peak process_commit_delta process_sample_ok live bytes guest peak allocs frees invalid arena_live arena_commit arena_peak arena_invalid registered_bytes managed_commit_peak text_metadata'.split()


def source_contracts(inputs):
    inputs.sha256(Q4_CATALOG,Q4_CATALOG_SHA);inputs.sha256(COMPOSITE_CATALOG,COMPOSITE_CATALOG_SHA)
    rows={row['case']:row for row in read_tsv(inputs,Q4_CATALOG,Q4_FIELDS) if row['family']=='collections-926'}
    original={row['case']:row for row in read_tsv(inputs,COMPOSITE_CATALOG,COMPOSITE_FIELDS) if row['family']=='collections-926'}
    typed_equal(set(rows),set(original),'original complete collections set');typed_equal(len(rows),20,'twenty original collections sources')
    for case,row in rows.items():
        for key in ('profile','caps','source_file','source_sha256','oracle_kind','oracle_value'):
            typed_equal(row[key],original[case][key],'original collections '+key)
        typed_equal(row['profile'],'collections','collections source domain');typed_equal(row['caps'],'[]','original collections capabilities')
        require(row['oracle_kind']=='value' or case=='L06' and row['oracle_kind']=='trap' and row['oracle_value']=='ListIndex','original collections outcome')
        inputs.sha256(row['source_file'],row['source_sha256'])
    return rows


def _hash(value,label):
    require(type(value) is str and re.fullmatch('[0-9a-f]{64}',value) is not None,label)


def native_summary(raw):
    require(type(raw) is bytes and 0<len(raw)<=1<<20 and raw.endswith(b'\n'),'complete bounded collections runtime summary')
    try:lines=raw.decode('ascii',errors='strict').splitlines()
    except UnicodeError as error:raise AdmissionError('non-ASCII collections summary') from error
    trap=None;values={}
    boundary=next((i for i,line in enumerate(lines) if line.startswith('MN-COLL ')),len(lines))
    prefix,lines=lines[:boundary],lines[boundary:]
    if prefix and prefix[-1].startswith('MN-COLL-TRAP code='):
        match=re.fullmatch(r'MN-COLL-TRAP code=([0-9]{1,20})',prefix.pop());require(match is not None,'collections trap syntax');trap=counter(int(match[1]),'collections trap')
    from ace_m1_text_q4_evidence import native_events
    text_events=('\n'.join(prefix)+'\n').encode() if prefix else b''
    typed_equal(native_events(text_events),[],'no embedded text runtime failure in original collections sources')
    require(len(lines)==len(SUMMARY_KEYS)+2,'complete collections summary fields')
    for line,key in zip(lines,SUMMARY_KEYS):
        match=re.fullmatch('MN-COLL '+key+r'=([0-9]{1,20})',line);require(match is not None,'original collections runtime field '+key)
        values[key]=counter(int(match[1]),'collections runtime '+key)
    def fields(line,prefix,keys):
        match=re.fullmatch(prefix+''.join(' '+key+r'=([0-9]{1,20})' for key in keys),line)
        require(match is not None,'original embedded text runtime summary '+prefix)
        return {key:counter(int(value),'embedded text '+key) for key,value in zip(keys,match.groups())}
    text=fields(lines[-2],'SUMMARY',['attempts','allocs','frees','live','live_bytes','peak'])
    frees=fields(lines[-1],'FREESTAT',['free_attempts','frees_ok','invalid'])
    for key in ('live','bytes','guest','invalid','arena_live','arena_commit','arena_invalid','registered_bytes','text_metadata'):
        typed_equal(values[key],0,'collections post-run cleanup '+key)
    typed_equal(values['allocs'],values['frees'],'collections owners released')
    typed_equal(values['process_sample_ok'],1,'collections process measurement present')
    typed_equal(values['process_commit_delta'],max(0,values['process_commit_peak']-values['process_private_base']),'collections private delta')
    for key in ('live','live_bytes'):typed_equal(text[key],0,'embedded text cleanup')
    typed_equal(text['allocs'],sum(re.fullmatch(r'(LIT|CLONE|CONCAT) id=[0-9]+',line) is not None for line in prefix),'embedded text actual allocation events')
    typed_equal(text['allocs'],text['frees'],'embedded text owners released');typed_equal(frees['invalid'],0,'embedded text invalid release')
    typed_equal(frees['frees_ok'],text['frees'],'embedded text successful release count')
    return trap,values


def collections_observation(scope,wire,check,contract,runtime):
    av=check['recipe']==AV_RECIPE
    typed_equal(check['recipe'],AV_RECIPE if av else MN_RECIPE,'original collections recipe')
    protocol='avworker/1' if av else 'mnworker/1'
    typed_equal(wire['protocol'],protocol,'collections protocol');typed_equal(wire['recipe'],check['recipe'],'collections wire recipe')
    require(type(wire['exit_code']) is int and wire['exit_code']==0 and wire['transport_error'] is None and wire['stderr']==b'','complete collections process exchange')
    request,_=scope.decode_json(wire['request'])
    exact_keys(request,{'recipe','nonce','source_b64','entry','caps','store_root','request_hash'}|({'steps'} if av else {'workdir'}),'collections request')
    require(type(request['nonce']) is str and bool(request['nonce']),'collections request nonce')
    for key,want in [('recipe',check['recipe']),('entry','d_main'),('caps',[])]:typed_equal(request[key],want,'collections request '+key)
    if av:typed_equal(request['steps'],[{'op':'run'}],'one collections source operation')
    unsigned={key:value for key,value in request.items() if key!='request_hash'}
    digest=hashlib.sha256(json.dumps(unsigned,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
    typed_equal(request['request_hash'],digest,'collections request hash')
    require(type(request['source_b64']) is str,'collections source frame')
    try:source=base64.b64decode(request['source_b64'],validate=True)
    except (ValueError,binascii.Error) as error:raise AdmissionError('invalid collections source frame') from error
    typed_equal(base64.b64encode(source).decode(),request['source_b64'],'canonical collections source frame')
    typed_equal(hashlib.sha256(source).hexdigest(),contract['source_sha256'],'original collections source')
    response,response_sha=scope.decode_json(wire['stdout']);exact_keys(response,('schema','nonce','request_hash','ok'),'collections response envelope')
    for key,want in [('schema',protocol),('nonce',request['nonce']),('request_hash',digest)]:typed_equal(response[key],want,'collections response '+key)
    ok=response['ok'];kind=contract['oracle_kind'];oracle=contract['oracle_value'];matches=False
    if av:
        exact_keys(ok,('recipe','steps','worker_version','av_core_version'),'AV collections result')
        typed_equal(ok['worker_version'],'avworker/1.0','AV collections worker');typed_equal(ok['av_core_version'],'av.collections/1','AV collections profile')
        require(type(ok['steps']) is list and len(ok['steps'])==1,'one AV collections step');step=ok['steps'][0]
        exact_keys(step,('op','outcome','value','collections','trap','diag','source_hash','wall_ms'),'AV collections source step')
        typed_equal(step['op'],'run','AV source operation');typed_equal(step['source_hash'],_fnv1a64(source),'AV source FNV');_duration(step['wall_ms'],'AV collections wall')
        typed_equal(step['collections'],None,'source collections is not synthetic fixture ledger')
        if step['outcome']=='success':
            require(type(step['value']) is int and -(1<<31)<=step['value']<(1<<31),'AV collections i32')
            typed_equal(step['trap'],None,'AV value trap');typed_equal(step['diag'],None,'AV value diagnostic')
            observed=dict(kind='value',value=step['value']);matches=kind=='value' and step['value']==int(oracle)
        elif step['outcome']=='trap':
            typed_equal(step['value'],None,'AV trap value');typed_equal(step['diag'],None,'AV trap diagnostic')
            require(type(step['trap']) is str and bool(step['trap']),'AV named collections trap')
            observed=dict(kind='trap',code=step['trap']);matches=kind=='trap' and step['trap']=='av.trap.list-index'
        else:
            typed_equal(step['outcome'],'diag','AV language outcome')
            for key in ('value','trap'):typed_equal(step[key],None,'AV diagnostic '+key)
            exact_keys(step['diag'],('stage','code','message'),'AV source diagnostic')
            require(all(type(v) is str for v in step['diag'].values()) and bool(step['diag']['message']),'AV source diagnostic fields')
            observed=dict(kind='diagnostic',**step['diag'])
    else:
        diagnostic='diagnostic' in ok
        identity='recipe worker_version mn_abi worker_memory mn_target collections_abi collections_codegen counters spans'.split()
        exact_keys(ok,identity+['diagnostic'] if diagnostic else MN_FIELDS,'native collections result')
        for key,want in [('worker_version','mnworker/1.0'),('mn_abi','msvc-x64/priv-i32-bool-v1'),('mn_target','x86_64-pc-windows-msvc'),('collections_abi','mn.collections.cell/3')]:typed_equal(ok[key],want,'native collections '+key)
        _hash(ok['collections_codegen'],'native collections codegen identity')
        memory=ok['worker_memory'];exact_keys(memory,('base','current','peak','delta','ok'),'native worker memory');_counts(memory,'native memory')
        typed_equal(memory['ok'],1,'native memory measurement');typed_equal(memory['delta'],max(0,memory['peak']-memory['base']),'native worker private delta')
        if diagnostic:
            typed_equal(ok['counters'],{},'native diagnosed counters');typed_equal(ok['spans'],{},'native diagnosed spans')
            exact_keys(ok['diagnostic'],('code','message'),'native source diagnostic');require(all(type(v) is str and bool(v) for v in ok['diagnostic'].values()),'native diagnostic fields')
            observed=dict(kind='diagnostic',**ok['diagnostic'])
        else:
            typed_equal(ok['op'],'run','native source operation');typed_equal(ok['runtime_identity'],runtime,'same independently captured runtime identity')
            require(type(ok['link_identity']) is list and len(ok['link_identity'])==3,'native original link identity')
            _hash(ok['link_identity'][0],'native linker identity');typed_equal(ok['link_identity'][1],'um','native SDK domain');_hash(ok['link_identity'][2],'native kernel identity')
            exact_keys(ok['counters'],COUNTERS,'native cold source counters');exact_keys(ok['spans'],SPANS,'native cold source spans');_counts(ok['counters'],'native counters');_counts(ok['spans'],'native spans')
            for key in ('link_invocations','run_invocations','codegen_emit_wrappers','link_manifests_persisted'):typed_equal(ok['counters'][key],1,'native actual '+key)
            typed_equal(ok['counters']['runtime_cargo_invocations'],0,'runtime prepared before source operation')
            _hash(ok['manifest_fp'],'native manifest fingerprint');require(type(ok['manifest_id']) is str and re.fullmatch('art[0-9a-f]{64}',ok['manifest_id']) is not None,'native manifest artifact identity')
            require(type(ok['exit_code']) is int and -(1<<31)<=ok['exit_code']<(1<<31),'native signed i32 result');_duration(ok['wall_ms'],'native wall')
            typed_equal(ok['stdout_hex'],'','native source stdout');raw=ok['stderr_hex'];require(type(raw) is str and len(raw)<=2<<20 and re.fullmatch('(?:[0-9a-f]{2})*',raw) is not None,'native complete stderr')
            trap,summary=native_summary(bytes.fromhex(raw))
            for key in ('changed_fn','refused','infra'):typed_equal(ok[key],None,'native cold source '+key)
            typed_equal(ok['reused_history_unit'],False,'native cold source is not history reuse')
            for key in ('codegen_units','cache_hits','unaffected_stage_delta','whole_module_stage_delta','fallback_whole_lower'):counter(ok[key],'native cold observation '+key)
            typed_equal(ok['codegen_units'],ok['counters']['codegen_emit_units'],'native codegen axis')
            require(type(ok['parsed_functions']) is dict and type(ok['bodies']) is dict,'native source body work');_counts(ok['parsed_functions'],'native parsed functions')
            for name,stages in ok['bodies'].items():
                require(type(name) is str and bool(name) and type(stages) is list and len(stages)==4,'native body stage axes')
                for value in stages:counter(value,'native body stage count')
            exact_keys(ok['allocation_meter'],('current','peak','total_peak','enabled'),'native allocation meter')
            for values,label in [(ok['ledger_peaks'],'native ledger'),(ok['allocation_meter'].get('current'),'native current allocation'),(ok['allocation_meter'].get('peak'),'native peak allocation')]:
                require(type(values) is list and len(values)==3,label)
                for value in values:counter(value,label)
            meter=ok['allocation_meter'];exact_keys(meter,('current','peak','total_peak','enabled'),'native allocation meter');counter(meter['total_peak'],'native total allocation');typed_equal(meter['enabled'],True,'native allocation measurement enabled')
            observed=dict(kind='trap',runtime_code=trap,exit=ok['exit_code']) if trap is not None else dict(kind='value',value=ok['exit_code'])
            matches=(kind=='trap' and trap==21 and ok['exit_code']==70) if trap is not None else kind=='value' and ok['exit_code']==int(oracle)
    typed_equal(ok['recipe'],check['recipe'],'collections actual recipe')
    return dict(schema='ace.m1.collections-source-observation/1',source_sha256=contract['source_sha256'],
                request_sha256=hashlib.sha256(wire['request']).hexdigest(),response_sha256=response_sha,
                expected=dict(kind=kind,value=oracle),observed=observed,judgement='PASS' if matches else 'FAIL')


def admit_source(inputs,evidence,binding,workers):
    from ace_m1_first_slice import _inventory_for_suite,_compact_admission,_repo_relative,_wires,retain_parent_status
    exact_keys(binding,('inventory','copy','dependencies_before','dependencies_after'),'collections source binding')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT);contracts=source_contracts(inputs);snapshots=[]
    for key,phase in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        snapshot,_=evidence.read_json(binding[key]);exact_keys(snapshot,('schema','snapshot','source_pins'),'collections source dependencies')
        typed_equal(snapshot['schema'],'ace.m1.collections-q4-dependencies/1','collections dependency schema');typed_equal(snapshot['snapshot'],phase,'collections dependency phase');exact_keys(snapshot['source_pins'],DEPENDENCIES,'collections dependency closure')
        for path,digest in snapshot['source_pins'].items():inputs.sha256(path,digest)
        snapshots.append(snapshot['source_pins'])
    typed_equal(*snapshots,'stable original collections execution dependencies')
    typed_equal(snapshots[0][DEPENDENCIES[0]],workers['av'],'current original AV producer')
    runtime,_=inputs.read_json(RUNTIME_STAMP);exact_keys(runtime,('abi','build_key','content_hash'),'original runtime build stamp')
    typed_equal(runtime['abi'],'mn.collections.runtime/3','original collections runtime ABI')
    for key in ('build_key','content_hash'):_hash(runtime[key],'runtime '+key)
    inventory=_inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget));admitted=admit_case_bindings(copy,inventory)
    cases={'ace.m1.collq4.'+case.lower():row for case,row in contracts.items()};seen=set();observations={};non_pass=[]
    for check,parent in admitted['checks']:
        require(check['case'] in cases and check['check'] in ('av','ac-native'),'foreign collections source execution')
        row=cases[check['case']];av=check['check']=='av';slot='q4-source/collections-926/'+row['case']+'/'+('AV' if av else 'AC-native')
        require(slot not in seen,'duplicate collections execution');seen.add(slot)
        typed_equal(check['recipe'],AV_RECIPE if av else MN_RECIPE,'original collections domain recipe')
        typed_equal([_repo_relative(inputs,name) for name in check['input_paths']],[row['source_file']],'original collections source path')
        typed_equal(check['source_files'][0]['sha256'],row['source_sha256'],'original collections source bytes')
        typed_equal(check['expected_exit'],int(row['oracle_value']) if row['oracle_kind']=='value' else None,'original collections oracle')
        completion={'kind':'success'} if row['oracle_kind']=='value' else {'kind':'error','stage':'run','code':('av' if av else 'mn')+'.trap.list-index','message':None}
        typed_equal(check['completion'],completion,'original collections completion')
        options=check['probe_bindings']['av' if av else 'native'];typed_equal(options['entry'],'d_main','collections entry');typed_equal(options['caps'],[],'collections original capabilities')
        typed_equal(check['declared_probe_steps'],[{'op':'run'}],'declared source run');require(check['state']=='RUN' and check['adapter']=='probe' and check['action']=='run','selected original source execution')
        def read_one(_):
            wire=_wires(copy,check,'avworker/1' if av else 'mnworker/1',check['recipe'],snapshots[0][DEPENDENCIES[0] if av else DEPENDENCIES[1]])
            require(len(wire)==1,'one collections actual exchange');wire=wire[0];owner=PurePosixPath(wire['start_path']).parent.parent
            require(len(owner.parts)==3 and owner.parts[1:]==(check['case'],check['check']),'collections exchange owner')
            request,_=copy.scope.decode_json(wire['request'])
            typed_equal(PureWindowsPath(request['store_root']),copy.original.joinpath(*owner.parts,'store' if av else 'mn-store'),'collections store owner')
            if not av:typed_equal(PureWindowsPath(request['workdir']),copy.original.joinpath(*owner.parts,'mn-work'),'collections work owner')
            return retain_parent_status(collections_observation(copy.scope,wire,check,row,runtime),parent['status'])
        observations[slot]=_compact_admission(inputs,read_one,32<<10)
        if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
    typed_equal(seen,{'q4-source/collections-926/'+case+'/'+target for case in contracts for target in ('AV','AC-native')},'complete twenty-source two-target observations')
    return dict(schema='ace.m1.collections-source-observations/1',required=40,observations=observations,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                semantic_evidence_complete=all(row['judgement']=='PASS' for row in observations.values()),
                full_resources_and_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)


REFERENCE = BASE+'collections-reference-contracts.tsv'
REFERENCE_SHA = 'f0b4adb0985304e6706215ff349f6f38a02af487d7318159ec8ca4772a7fdde1'
REFERENCE_FIELDS = 'family case profile caps source_file source_sha256 oracle_kind oracle_value rust_reference rust_reference_sha256 c_reference c_reference_sha256 mode rust_runs c_runs execution'.split()


def reference_contracts(inputs):
    inputs.sha256(REFERENCE,REFERENCE_SHA);sources=source_contracts(inputs)
    original={row['case']:row for row in read_tsv(inputs,COMPOSITE_CATALOG,COMPOSITE_FIELDS) if row['family']=='collections-926'}
    seen={}
    for row in read_tsv(inputs,REFERENCE,REFERENCE_FIELDS):
        case=row['case'];require(case in sources and case not in seen,'foreign/duplicate collections reference')
        for key in ('family','profile','caps','source_file','source_sha256','oracle_kind','oracle_value'):
            typed_equal(row[key],sources[case][key],'original collections reference '+key)
        typed_equal(row['rust_reference'],'crates/auto-lang/test/cases/a2r-hir-collections-904/reference-'+case+'.rs','original independent 904 Rust reference')
        typed_equal((row['c_reference'],row['c_reference_sha256']),(original[case]['reference_file'],original[case]['reference_sha256']),'original independent C reference')
        # The existing hand-written translations stay byte-exact; no emitted
        # source or echoed expected result supplies a reference program.
        for target in ('rust','c'):inputs.sha256(row[target+'_reference'],row[target+'_reference_sha256'])
        typed_equal((row['mode'],row['rust_runs'],row['c_runs'],row['execution']),('debug','1','1','UNOBSERVED'),'original default mode obligations')
        seen[case]=row
    typed_equal(set(seen),set(sources),'all twenty independent collections references')
    return seen


def admit_certifications(inputs,evidence,binding_path):
    from ace_m1_first_slice import _compact_admission,_repo_relative
    from ace_m1_cert_evidence import collections_certification
    from ace_m1_daily_evidence import certified_daily_emission
    contracts=reference_contracts(inputs);document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'collections certification binding');producer=document['producer']
    exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'collections producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','original collections producer')
    typed_equal(producer['cwd'],str(inputs.root),'collections producer worktree');require(type(document['rows']) is list,'collections certification rows')
    seen=set();observations={}
    for row in document['rows']:
        exact_keys(row,('family','case','prefix','pin'),'collections certification row');case=row['case'];pin=row['pin']
        typed_equal(row['family'],'collections-926','collections certification family');require(case in contracts and type(pin) is dict,'original collections certified source')
        exact_keys(pin,('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'collections certification pins')
        target=pin['target'];require(target in ('rust','c') and (case,target) not in seen,'duplicate/foreign collections target');seen.add((case,target))
        typed_equal(pin['identity_request'],{'op':'identity','certification_target':target},'collections compiler identity target')
        typed_equal(pin['request']['case_id'],'941/collections/collections-926/'+case,'original collections case identity')
        path=PurePosixPath(row['prefix']);home=PurePosixPath(binding_path).parent
        require(path.parent.parent==home/('collections-926--'+case) and path.name==target and path.parent.name.startswith('attempt-')
                and path.parent.name[8:].isdigit() and int(path.parent.name[8:])>0,'collections owned attempt')
        observations['q4-source/collections-926/'+case+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=_compact_admission(inputs,
            lambda _:certified_daily_emission(evidence,inputs,row['prefix'],pin,producer,
                collections_certification(evidence,inputs,row['prefix'],pin,producer,contracts[case])),128<<10)
    typed_equal(seen,{(case,target) for case in contracts for target in ('rust','c')},'complete original default collections certifications')
    return dict(schema='ace.m1.collections-certification-observations/1',required=40,observations=observations,
                contract_sha256=REFERENCE_SHA,bindings_sha256=digest,
                default_certification_complete=all(row['judgement']=='PASS' for row in observations.values()),
                configuration_key_controls='UNOBSERVED',optimized_rust_execution='UNOBSERVED',
                full_resources_and_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)
