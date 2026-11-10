"""Required original method/record/text byte variants; registration is not evidence."""
import json
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import Q4_CATALOG,Q4_CATALOG_SHA,Q4_FIELDS,read_tsv,CASE_CATALOG,CASE_CATALOG_SHA
from ace_m1_source_supplement import source_contracts as scalar_supplement
CATALOG='crates/auto-lang/test/cases/ace-m1-expanded/original-data-source-contracts.tsv'
CATALOG_SHA='dbf34b87c819aa673027f3c9992220e687ed49a8c226112678ee516bd863ded5'
FIELDS='family case profile caps source_file source_sha256 oracle_kind oracle_value origin_case_file origin_case_sha256 origin_case_id origin_group original_checks'.split()
values={'M01':0,'M02':57,'M03':75,'M04':135,'M05':42,'M06':52,'M07':84,'M08':42,'R01':19,'R02':9,'R03':23,'R04':51,'R05':42,'R06':21,'T01':38,'T02':27,'T04':128512}
negative={'M09':'method-called-as-static','M10':'immutable-write','M11':'alias-mut-self','M12':'reserved-owned-method','R09':'record-type-mismatch','R10':'use-after-move','T06':'borrow-escape','T07':'invalid-scalar'}
profiles={**{c:'record' for c in ['M01','M03','M04','M07','M08','M09','M10','M11','R01','R02','R03','R06','R09']},**{c:'text-record' for c in ['M02','M05','M06','M12','R04','R05','R10']},**{c:'text' for c in ['T01','T02','T04','T06','T07','T08']}}


def source_contracts(inputs):
    inputs.sha256(CATALOG,CATALOG_SHA)
    inputs.sha256(CASE_CATALOG,CASE_CATALOG_SHA)
    inputs.sha256(Q4_CATALOG,Q4_CATALOG_SHA)
    prior={row['source_sha256'] for row in read_tsv(inputs,Q4_CATALOG,Q4_FIELDS)}
    prior.update(row['source_sha256'] for row in scalar_supplement(inputs).values())
    seen={};hashes=set()
    for row in read_tsv(inputs,CATALOG,FIELDS):
        case=row['case'];require(case in profiles and case not in seen,'original data source identity')
        method=case.startswith('M')
        family='original-method-881' if method else 'original-record-text'
        group='case-ace-m1-method-881' if method else 'case-ace-m1-tr-native'
        origin='crates/auto-lang/test/cases/ace-m1/'+('method-881.case.ad' if method else 't-r-native.case.ad')
        kind,oracle=('value',str(values[case])) if case in values else ('diagnostic',negative[case]) if case in negative else ('trap','TextIndex')
        expected=dict(family=family,profile=profiles[case],source_file='crates/auto-lang/test/cases/ace-m1/src/'+case+'.at',
                      oracle_kind=kind,oracle_value=oracle,origin_case_file=origin,origin_case_id='ace.m1.'+case.lower(),origin_group=group)
        for key,want in expected.items():typed_equal(row[key],want,'original data source '+key)
        typed_equal(json.loads(row['caps']),[] if profiles[case]=='record' else ['ac.runtime.text.v1'],'original data source capabilities')
        typed_equal(json.loads(row['original_checks']),['ac-native'] if method and case in negative else ['ac-native','av'],'original data source check provenance')
        inputs.sha256(row['source_file'],row['source_sha256']);inputs.sha256(origin,row['origin_case_sha256'])
        require(row['source_sha256'] not in prior|hashes,'original data variant must not alias an existing source')
        hashes.add(row['source_sha256']);seen[case]=row
    typed_equal(set(seen),set(profiles),'complete 26 original data source variants')
    return seen


# Only positive/trap observations enter this adapter; all eight original
# diagnostic sources retain their separately required target keys.
from ace_m1_arch_acceptance import EvidenceScope,exact_keys
from ace_m1_case_evidence import CaseCopy,admit_case_bindings
from ace_m1_text_q4_evidence import text_observation,DEPENDENCIES,AV_TRAPS
from pathlib import PurePosixPath
CONTRACT='crates/auto-lang/test/cases/ace-m1-expanded/original-data-q4-contract.json'
CONTRACT_SHA = 'c19142b748fc69c1cfba68fc78a3451076ebb9e574da69fd9bd2727b63ff9e79'
SUITE='crates/auto-lang/test/cases/ace-m1-expanded/original-data-q4.suite.ad'
# T08 binds a str view to a static literal, not an owning Str buffer.
# hir_adapter StrLit preserves the borrowed destination; only the failed
# slice calls rt_fail(-10). ac_rt_own_from_lit and free are never invoked.
T08_NATIVE_TRACE=b'RT-FAIL code=-10\n'


def admit_source(inputs,evidence,binding,workers):
    from ace_m1_first_slice import _inventory_for_suite,_compact_admission,_repo_relative,_wires,retain_parent_status
    exact_keys(binding,('inventory','copy','dependencies_before','dependencies_after','tool_capture'),'record/text execution bindings')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT)
    contracts={(row['family'],row['case']):row for row in source_contracts(inputs).values() if row['oracle_kind']!='diagnostic'};snapshots=[]
    for field,label in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        record,_=evidence.read_json(binding[field]);exact_keys(record,('schema','snapshot','source_pins'),'record/text dependency snapshot')
        typed_equal(record['schema'],'ace.m1.rt-q4-dependencies/1','record/text dependency schema')
        typed_equal(record['snapshot'],label,'record/text dependency phase');exact_keys(record['source_pins'],DEPENDENCIES,'record/text dependencies')
        for name,digest in record['source_pins'].items():inputs.sha256(name,digest)
        snapshots.append(record['source_pins'])
    typed_equal(*snapshots,'stable record/text execution dependencies')
    for key,name in [('av','av-worker.exe'),('native','mn-worker.exe')]:
        typed_equal(snapshots[0]['experimental/ace/target/release/'+name],workers[key],'record/text original worker')
    exact_keys(binding['tool_capture'],('path','sha256'),'original data tool capture')
    evidence.sha256(binding['tool_capture']['path'],binding['tool_capture']['sha256'])
    parent,_=evidence.read_json(binding['tool_capture']['path'])
    typed_equal(parent.get('groups'),['case-ace-m1-original-data-q4-941'],'original data formal fixed group')
    typed_equal(parent.get('executed'),36,'original data formal fixed denominator')
    required_tools={'av':DEPENDENCIES[0],'mn':DEPENDENCIES[1],'text-runtime':DEPENDENCIES[2]};captured={}
    require(type(parent.get('tools')) is list,'original data prepared tool list')
    for tool in parent['tools']:
        if tool.get('tool') in required_tools and 'path' in tool:
            name=required_tools[tool['tool']]
            typed_equal(_repo_relative(inputs,tool['path']),name,'original data prepared path')
            typed_equal(tool['sha256'],snapshots[0][name],'original data prepared dependency')
            if tool['tool'] in captured:typed_equal(captured[tool['tool']],tool['sha256'],'consistent original data tool preparation')
            captured[tool['tool']]=tool['sha256']
    typed_equal(set(captured),set(required_tools),'all original data production dependencies')
    inventory=_inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inventory)
    cases={'ace.m1.original-data.'+case.lower():row for (family,case),row in contracts.items()}
    seen,observations,non_pass=set(),{},[]
    for check,parent in admitted['checks']:
        require(check['case'] in cases and check['check'] in ('av','ac-native'),'foreign record/text execution')
        contract=cases[check['case']];av=check['check']=='av';target='AV' if av else 'AC-native'
        slot='q4-source/'+contract['family']+'/'+contract['case']+'/'+target
        require(slot not in seen,'duplicate record/text execution');seen.add(slot)
        recipe=('ac-av/data-run-v1' if contract['profile']=='record' else 'ac-av/owned-run-v1') if av else 'ac/mir-native-scalar-v1'
        typed_equal(check['recipe'],recipe,'original record/text domain recipe')
        typed_equal([_repo_relative(inputs,p) for p in check['input_paths']],[contract['source_file']],'original record/text source path')
        typed_equal(check['source_files'][0]['sha256'],contract['source_sha256'],'original record/text source bytes')
        kind,oracle=contract['oracle_kind'],contract['oracle_value']
        typed_equal(check['expected_exit'],int(oracle) if kind=='value' else None if av else 70,'independent record/text result')
        completion={'kind':'error','stage':'run','code':AV_TRAPS[oracle],'message':None} if av and kind=='trap' else {'kind':'success'}
        typed_equal(check['completion'],completion,'independent record/text completion')
        options=check['probe_bindings']['av' if av else 'native'];typed_equal(options['entry'],'d_main','record/text source entry')
        caps,_=inputs.decode_json(contract['caps'].encode());typed_equal(options['caps'],caps,'record/text original capabilities')
        require(check['state']=='RUN' and check['adapter']=='probe' and check['action']=='run','record/text selected actual run')
        typed_equal(check['declared_probe_steps'],[{'op':'run'}],'record/text declared run')
        def read_one(_):
            wire=_wires(copy,check,'avworker/1' if av else 'mnworker/1',recipe,workers['av' if av else 'native'])
            require(len(wire)==1,'one original record/text exchange');wire=wire[0]
            owner=PurePosixPath(wire['start_path']).parent.parent
            require(len(owner.parts)==3 and owner.parts[1:]==(check['case'],check['check']),'record/text exchange owner')
            request,_=copy.scope.decode_json(wire['request'])
            from pathlib import PureWindowsPath
            typed_equal(PureWindowsPath(request['store_root']),copy.original.joinpath(*owner.parts,'store' if av else 'mn-store'),'record/text store owner')
            if not av:typed_equal(PureWindowsPath(request['workdir']),copy.original.joinpath(*owner.parts,'mn-work'),'record/text work owner')
            return retain_parent_status(text_observation(copy.scope,wire,check,contract,record_family=True,trap_trace=T08_NATIVE_TRACE if contract['case']=='T08' else None),parent['status'])
        observations[slot]=_compact_admission(inputs,read_one,32<<10)
        if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
    typed_equal(seen,{'q4-source/'+family+'/'+case+'/'+target for family,case in contracts for target in ('AV','AC-native')},'complete original record/text source execution set')
    return dict(schema='ace.m1.original-data-source-observations/1',required=36,observations=observations,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                semantic_evidence_complete=all(v['judgement']=='PASS' for v in observations.values()),
                full_resource_and_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)


REFERENCE='crates/auto-lang/test/cases/ace-m1-expanded/original-data-reference-contracts.tsv'
REFERENCE_SHA='92cbb23d49bc7ed9cb108055dc696c88b6b8f2a7b53e2536888d9972fdde17aa'


def reference_contracts(inputs):
    from ace_m1_record_text_evidence import FIELDS as REFERENCE_FIELDS
    inputs.sha256(REFERENCE,REFERENCE_SHA)
    originals={row['case']:row for row in source_contracts(inputs).values() if row['oracle_kind']!='diagnostic'}
    seen={}
    for row in read_tsv(inputs,REFERENCE,REFERENCE_FIELDS):
        case=row['case'];require(case in originals and case not in seen,'original data reference identity')
        for key in ('family','profile','caps','source_file','source_sha256','oracle_kind','oracle_value'):
            typed_equal(row[key],originals[case][key],'original data independent reference '+key)
        for key,want in [('rust_debug_runs','1'),('rust_release_runs','0'),('rust_release_flags','[]'),('c_runs','1'),('execution','UNOBSERVED')]:
            typed_equal(row[key],want,'original data reference configuration '+key)
        for target,ext in [('rust','rs'),('c','c')]:
            expected='crates/auto-lang/test/cases/ace-m1-expanded/original-data-reference/'+case+'.'+ext
            typed_equal(row[target+'_reference'],expected,'original data hand-written reference path')
            inputs.sha256(expected,row[target+'_reference_sha256'])
        seen[case]=row
    typed_equal(set(seen),set(originals),'complete 18 original data independent references')
    return seen


def certification_coverage(contracts,observations,diagnostics=()):
    expected={'q4-source/'+row['family']+'/'+row['case']+'/'+target
              for row in contracts.values() for target in ('ACE-A2R-HIR','ACE-A2C-HIR')}
    # PLAN-1000 16 diagnostic legs: the refused sources certify NEGATIVELY
    # (typed frontdoor refusals on the same daily channel as the six query
    # negatives); their slots join the same coverage contract.
    expected|={'q4-source/'+row['family']+'/'+row['case']+'/'+target
               for row in diagnostics for target in ('ACE-A2R-HIR','ACE-A2C-HIR')}
    require(type(observations) is dict and observations.keys()<=expected,'foreign original data certification')
    require(all(type(row) is dict and row.get('judgement') in ('PASS','FAIL') for row in observations.values()),'original data completed certification verdict')
    missing=sorted(expected-observations.keys())
    complete=dict(observations)
    for key in missing:
        complete[key]=dict(judgement='UNOBSERVED',reason='No completed actual/reference certification supplied; failed or missing attempts cannot authorize PASS.')
    return complete,missing


def admit_certifications(inputs,evidence,binding_path):
    from ace_m1_first_slice import _compact_admission,_repo_relative
    from ace_m1_cert_evidence import record_text_certification
    from ace_m1_daily_evidence import certified_daily_emission
    contracts=reference_contracts(inputs);document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'original data certification bindings')
    producer=document['producer'];exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'original data certification producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','original data actual producer')
    typed_equal(producer['cwd'],str(inputs.root),'original data actual worktree')
    require(type(document['rows']) is list,'original data certification row list')
    import os
    from ace_m1_data_diagnostics import ORACLES
    from ace_m1_query_evidence import typed_frontdoor_observation
    from ace_m1_cert_evidence import admit_command,_relative
    diagnostics=[row for row in source_contracts(inputs).values() if row['oracle_kind']=='diagnostic']
    observations={};seen=set()
    for row in document['rows']:
        if 'refusal' in row:
            # PLAN-1000 16 diagnostic legs: the negative certification — a
            # typed frontdoor refusal of the diagnostic source on the daily
            # channel (the exact F-1000-02 capture pattern), certified when
            # the first cause's stage/code equals the family's own
            # diagnostic authority (the same ORACLES table the AV/native
            # diagnostic legs verify against).
            exact_keys(row,('family','case','target','prefix','request','source_sha256'),'original data negative certification row')
            case,target=row['case'],row['target']
            contract=next((r for r in diagnostics if r['case']==case),None)
            require(contract is not None and row['family']==contract['family'],'original data diagnostic source')
            require(target in ('rust','c') and (case,target) not in seen,'duplicate/foreign original data negative target');seen.add((case,target))
            request=row['request']
            exact_keys(request,('op','source_path','profile','entry','caps','out_dir','nonce'),'original data negative request')
            stage,code,profile=ORACLES[case]
            for field,want in [('op','daily-emit' if target=='rust' else 'daily-emit-c'),('profile',profile),('entry','main'),('caps',json.loads(contract['caps']))]:
                typed_equal(request[field],want,'original data negative request '+field)
            typed_equal(_relative(inputs,request['source_path']),contract['source_file'],'original data negative source path')
            typed_equal(row['source_sha256'],contract['source_sha256'],'independent negative source hash');inputs.sha256(contract['source_file'],row['source_sha256'])
            prefix=row['prefix'];home=PurePosixPath(binding_path).parent
            typed_equal(PurePosixPath(prefix),home/(contract['family']+'--'+case)/target,'negative original data owner')
            require(type(request['out_dir']) is str and request['out_dir']==str(evidence.root.joinpath(*PurePosixPath(prefix).parts)),'negative original data output owner')
            require(type(request['nonce']) is str and len(request['nonce'])==48 and all(c in '0123456789abcdef' for c in request['nonce']),'original data negative nonce')
            require(not os.path.lexists(evidence.path(prefix+'.command.json').parent/target),'refused diagnostic produced an output entry')
            def read_negative(_):
                response,receipts=admit_command(evidence,prefix+'.command.json',request,producer,expected_exit=2)
                record,_=evidence.read_json(prefix+'.command.json');typed_equal(record['meter']['job_processes'],2,'negative emit launcher and worker only')
                source=inputs.read_bytes(contract['source_file'])
                return dict(typed_frontdoor_observation(response,request,contract,source,stage=stage,code=code),
                            source_sha256=contract['source_sha256'],**receipts)
            observations['q4-source/'+contract['family']+'/'+case+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=_compact_admission(inputs,read_negative,32<<10)
            continue
        exact_keys(row,('family','case','prefix','pin'),'original data certification row')
        case,pin=row['case'],row['pin'];require(case in contracts and type(pin) is dict,'original data certified source')
        contract=contracts[case];typed_equal(row['family'],contract['family'],'original data certification family')
        exact_keys(pin,('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'original data certification pins')
        target=pin['target'];require(target in ('rust','c') and (case,target) not in seen,'duplicate/foreign original data certification target');seen.add((case,target))
        typed_equal(pin['identity_request'],{'op':'identity','certification_target':target},'original data compiler identity target')
        typed_equal(pin['request']['case_id'],'941/original-data/'+contract['family']+'/'+case,'original data certification case')
        path=PurePosixPath(row['prefix']);home=PurePosixPath(binding_path).parent
        require(path.parent.parent==home/(contract['family']+'--'+case) and path.name==target
                and path.parent.name.startswith('attempt-') and path.parent.name[8:].isdigit()
                and int(path.parent.name[8:])>0,'original data certification attempt owner')
        value=_compact_admission(inputs,lambda _:certified_daily_emission(evidence,inputs,row['prefix'],pin,producer,
            record_text_certification(evidence,inputs,row['prefix'],pin,producer,contract)),128<<10)
        observations['q4-source/'+contract['family']+'/'+case+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=value
    supplied=len(observations);observations,missing=certification_coverage(contracts,observations,diagnostics)
    return dict(schema='ace.m1.original-data-certifications/1',required=36+2*len(diagnostics),supplied=supplied,missing_certification_slots=missing,
                observations=observations,contract_sha256=REFERENCE_SHA,bindings_sha256=digest,
                certification_complete=not missing and all(v['judgement']=='PASS' for v in observations.values()),
                daily_admission='UNOBSERVED' if missing else 'PASS' if all(v['daily_judgement']=='PASS' for v in observations.values()) else 'FAIL',
                can_claim_expanded_m1=False)
