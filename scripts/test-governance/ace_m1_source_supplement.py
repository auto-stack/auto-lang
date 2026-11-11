"""Four original 892/921 scalar variants omitted from the initial open catalog."""
from pathlib import PurePosixPath, PureWindowsPath
import json
from ace_m1_arch_acceptance import EvidenceScope,exact_keys
from ace_m1_case_evidence import CaseCopy,admit_case_bindings,require
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import Q4_CATALOG,Q4_CATALOG_SHA,Q4_FIELDS,read_tsv
from ace_m1_scalar_q4_evidence import scalar_q4_observation,TRAPS,TRACE_CAP

CATALOG='crates/auto-lang/test/cases/ace-m1-expanded/source-supplement-contracts.tsv'
CATALOG_SHA='f0642c510f36ac682c71a1ad66228f4ca103162388f8dd3b7c124a2e5c66b6b6'
FIELDS='family case profile caps source_file source_sha256 oracle_kind oracle_value marks trace origin_case_file origin_case_sha256 origin_case_id origin_check origin_group'.split()
CONTRACT='crates/auto-lang/test/cases/ace-m1-expanded/source-supplement-q4-contract.json'
CONTRACT_SHA = '4e33debd92b12e6fd916b15c024e4d72c67b76e47f97b75966bc8f2c1dfdce6e'
SUITE='crates/auto-lang/test/cases/ace-m1-expanded/source-supplement.suite.ad'
ORIGINALS={'c01':('921','C01','value','42'), 'c02':('921','C02','trap','DivOverflow'),
           'c03':('921','C03','value','61'), 'named-01':('892','NAMED-01','value','23')}
REFERENCE='crates/auto-lang/test/cases/ace-m1-expanded/source-supplement-reference-contracts.tsv'
REFERENCE_SHA='e0aa3c236ea541624fbc31661fc739d0d99989ae4de5ca41a8a51dac0ea559ec'


def source_contracts(inputs):
    inputs.sha256(CATALOG,CATALOG_SHA)
    inputs.sha256(Q4_CATALOG,Q4_CATALOG_SHA)
    original_hashes={row['source_sha256'] for row in read_tsv(inputs,Q4_CATALOG,Q4_FIELDS)}
    rows={};source_hashes=set()
    for row in read_tsv(inputs,CATALOG,FIELDS):
        case=row['case'];require(case in ORIGINALS and case not in rows,'original scalar supplement identity')
        owner,stem,kind,value=ORIGINALS[case]
        root='crates/auto-lang/test/cases/'+('a2c-hir-scalar-921' if owner=='921' else 'a2r-hir-892')
        expected=dict(family='source-original-scalar',profile='scalar',source_file=root+'/src/'+stem+'.at',
                      oracle_kind=kind,oracle_value=value,origin_case_file=root+'/'+stem+'.case.ad',origin_check='a2x-emit',
                      origin_case_id='a2c-hir-scalar-921.'+case if owner=='921' else 'a2r-hir-892.named-01-source-order',
                      origin_group='case-a2c-hir-scalar-921' if owner=='921' else 'unregistered-a2r-hir-892',
                      trace='true' if case=='named-01' else 'false',marks='mark_b,mark_a' if case=='named-01' else '')
        for key,want in expected.items():typed_equal(row[key],want,'original source supplement '+key)
        typed_equal(json.loads(row['caps']),[TRACE_CAP] if case=='named-01' else [],'original source supplement capabilities')
        inputs.sha256(row['source_file'],row['source_sha256']);inputs.sha256(row['origin_case_file'],row['origin_case_sha256'])
        require(row['source_sha256'] not in original_hashes|source_hashes,'supplement must be a distinct source byte variant')
        source_hashes.add(row['source_sha256']);rows[case]=row
    typed_equal(set(rows),set(ORIGINALS),'all four original source variants')
    return rows


def reference_contracts(inputs):
    from ace_m1_scalar_cert_evidence import FIELDS as REFERENCE_FIELDS
    inputs.sha256(REFERENCE,REFERENCE_SHA)
    originals=source_contracts(inputs);seen={}
    for row in read_tsv(inputs,REFERENCE,REFERENCE_FIELDS):
        case=row['case'];require(case in originals and case not in seen,'original source reference identity')
        source=originals[case]
        for key in ('source_file','source_sha256','profile','caps','oracle_kind','oracle_value'):
            typed_equal(row[key],source[key],'original source reference '+key)
        typed_equal(json.loads(row['marks']),source['marks'].split(',') if source['marks'] else [],'original source reference marks')
        for key,want in [('rust_debug_runs','1'),('rust_release_runs','0'),('c_runs','1'),('execution','UNOBSERVED')]:
            typed_equal(row[key],want,'original source reference '+key)
        for target,ext in [('rust','rs'),('c','c')]:
            expected=('crates/auto-lang/test/cases/a2c-hir-scalar-921/reference-C01.c' if case=='c01' and target=='c'
                      else 'crates/auto-lang/test/cases/a2r-hir-892/reference-NAMED-01.rs' if case=='named-01' and target=='rust'
                      else 'crates/auto-lang/test/cases/ace-m1-expanded/source-supplement-reference/'+case+'.'+ext)
            typed_equal(row[target+'_reference'],expected,'original independent reference path')
            inputs.sha256(expected,row[target+'_reference_sha256'])
        seen[case]=dict(row,trace=source['trace'])
    typed_equal(set(seen),set(originals),'all four original independent references')
    return seen


def admit_certifications(inputs,evidence,binding_path):
    from ace_m1_first_slice import _compact_admission,_repo_relative
    from ace_m1_daily_evidence import certified_daily_emission
    from ace_m1_cert_evidence import scalar_corpus_certification
    contracts=reference_contracts(inputs);document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'original source certification bindings')
    producer=document['producer'];exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'original source certification producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','original source actual producer')
    typed_equal(producer['cwd'],str(inputs.root),'original source producer worktree')
    require(type(document['rows']) is list,'original source certification rows')
    seen=set();observations={}
    for row in document['rows']:
        exact_keys(row,('case','prefix','pin'),'original source certification row')
        case,pin=row['case'],row['pin'];require(case in contracts and type(pin) is dict,'original source certification identity')
        exact_keys(pin,('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'original source certification pins')
        target=pin['target'];require(target in ('rust','c') and (case,target) not in seen,'duplicate/foreign original source certification target');seen.add((case,target))
        typed_equal(pin['request']['case_id'],'941/source-original-scalar/'+case,'original source certification case')
        typed_equal(pin['identity_request'],{'op':'identity','certification_target':target},'original source certification toolchain target')
        path=PurePosixPath(row['prefix']);parent=PurePosixPath(binding_path).parent
        require(path.parent.parent==parent/case and path.name==target and path.parent.name.startswith('attempt-')
                and path.parent.name[8:].isdigit() and int(path.parent.name[8:])>0,'original source certification attempt owner')
        value=_compact_admission(inputs,lambda _:certified_daily_emission(evidence,inputs,row['prefix'],pin,producer,
            scalar_corpus_certification(evidence,inputs,row['prefix'],pin,producer,contracts[case])),128<<10)
        observations['q4-source/source-original-scalar/'+case+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=value
    typed_equal(seen,{(case,target) for case in contracts for target in ('rust','c')},'complete original eight certifications')
    return dict(schema='ace.m1.original-source-certification-observations/1',required=8,contract_sha256=REFERENCE_SHA,
                bindings_sha256=digest,observations=observations,
                certification_complete=all(v['judgement']=='PASS' for v in observations.values()),
                daily_admission='PASS' if all(v['daily_judgement']=='PASS' for v in observations.values()) else 'FAIL',
                can_claim_expanded_m1=False)


def admit_source(inputs, evidence, binding, workers):
    from ace_m1_first_slice import _compact_admission, _inventory_for_suite, _repo_relative, _wires, retain_parent_status
    exact_keys(binding,('inventory','copy','tool_capture'),'scalar Q4 binding')
    exact_keys(binding['tool_capture'],('path','sha256'),'scalar Q4 original tool capture')
    evidence.sha256(binding['tool_capture']['path'],binding['tool_capture']['sha256'])
    tool_record,_=evidence.read_json(binding['tool_capture']['path'])
    typed_equal(tool_record.get('groups'),['case-ace-m1-source-supplement-941'],'scalar Q4 original fixed group')
    typed_equal(tool_record.get('executed'),8,'scalar Q4 original fixed denominator')
    tool_paths={'av':'experimental/ace/target/release/av-worker.exe',
                'mn':'experimental/ace/target/release/mn-worker.exe',
                'trace-runtime':'experimental/ace/target/runtime-dev/ace_trace_runtime.lib'}
    captured={}
    require(type(tool_record.get('tools')) is list,'scalar Q4 original prepared tools')
    for ref in tool_record['tools']:
        if ref.get('tool') in tool_paths and 'path' in ref:
            path=tool_paths[ref['tool']]
            typed_equal(_repo_relative(inputs,ref['path']),path,'scalar Q4 original prepared path')
            inputs.sha256(path,ref['sha256'])
            if ref['tool'] in captured:typed_equal(captured[ref['tool']],ref['sha256'],'scalar Q4 consistent tool capture')
            captured[ref['tool']]=ref['sha256']
    typed_equal(set(captured),set(tool_paths),'scalar Q4 complete prepared dependencies')
    typed_equal((captured['av'],captured['mn']),(workers['av'],workers['native']),'scalar Q4 independently pinned workers')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT)
    inventory = _inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    rows=list(source_contracts(inputs).values())
    cases={'ace.m1.original-scalar.'+row['case'].lower():row for row in rows}
    require(len(cases)==4,'unique original scalar source variants')
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inventory)
    seen,observations,non_pass=set(),{},[]
    for check,parent in admitted['checks']:
        require(check['case'] in cases and check['check'] in ('av','ac-native'),'foreign scalar Q4 check')
        contract=cases[check['case']];av=check['check']=='av';target='AV' if av else 'AC-native'
        slot='q4-source/source-original-scalar/'+contract['case']+'/'+target
        require(slot not in seen,'duplicate scalar Q4 observation');seen.add(slot)
        typed_equal(check['recipe'],'ac-av/core-run-v1' if av else 'ac/mir-native-scalar-v1','scalar Q4 declared recipe')
        typed_equal(_repo_relative(inputs,check['input_paths'][0]),contract['source_file'],'scalar Q4 original source path')
        typed_equal(check['source_files'][0]['sha256'],contract['source_sha256'],'scalar Q4 original source hash')
        options=check['probe_bindings']['av' if av else 'native']
        typed_equal(options['entry'],'d_main','scalar Q4 declared entry')
        typed_equal(options['caps'],[TRACE_CAP] if contract['trace']=='true' else [],'scalar Q4 declared capabilities')
        typed_equal(check['declared_probe_steps'],[{'op':'run'}],'scalar Q4 declared run')
        require(check['state']=='RUN' and check['adapter']=='probe' and check['action']=='run','selected scalar Q4 execution')
        typed_equal(check['expected_exit'],int(contract['oracle_value']) if contract['oracle_kind']=='value' else None if av else 70,'scalar Q4 independent value')
        typed_equal(check['completion'],{'kind':'error','stage':'run','code':TRAPS[contract['oracle_value']],'message':None} if av and contract['oracle_kind']=='trap' else {'kind':'success'},'scalar Q4 independent completion')
        def read_one(_):
            wires=_wires(copy,check,'avworker/1' if av else 'mnworker/1',check['recipe'],workers['av' if av else 'native'])
            require(len(wires)==1,'one original scalar Q4 exchange')
            wire=wires[0];owner=PurePosixPath(wire['start_path']).parent.parent
            require(len(owner.parts)==3 and owner.parts[1:]==(check['case'],check['check']),'scalar Q4 exchange owner')
            request,_=copy.scope.decode_json(wire['request'])
            typed_equal(PureWindowsPath(request['store_root']),copy.original.joinpath(*owner.parts,'store' if av else 'mn-store'),'scalar Q4 store owner')
            if not av:typed_equal(PureWindowsPath(request['workdir']),copy.original.joinpath(*owner.parts,'mn-work'),'scalar Q4 native work owner')
            return retain_parent_status(scalar_q4_observation(copy.scope,wire,check,contract),parent['status'])
        observations[slot]=_compact_admission(inputs,read_one,32<<10)
        if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
    typed_equal(seen,{'q4-source/source-original-scalar/'+row['case']+'/'+target for row in rows for target in ('AV','AC-native')},'complete four-source scalar supplement')
    counts={state:sum(v['judgement']==state for v in observations.values()) for state in ('PASS','FAIL','UNOBSERVED')}
    return dict(schema='ace.m1.source-supplement-observations/1',required=8,observations=observations,counts=counts,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                semantic_evidence_complete=counts['PASS']==8,certification_targets='UNOBSERVED',budgets_and_full_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)
