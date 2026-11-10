"""Original record/text mode and base sources; semantic and mode axes distinct."""
from pathlib import PurePosixPath

from ace_m1_arch_acceptance import EvidenceScope, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import (BASE, Q4_CATALOG, Q4_CATALOG_SHA, Q4_FIELDS,
    COMPOSITE_CATALOG, COMPOSITE_CATALOG_SHA, COMPOSITE_FIELDS, read_tsv)
from ace_m1_text_q4_evidence import text_observation, DEPENDENCIES, AV_TRAPS

REFERENCE = BASE+'record-text-reference-contracts.tsv'
REFERENCE_SHA = '7869cb3729409f4dce41bb5018e51bd0fab694d4a90d18ac74866dd923b78f8e'
CONTRACT = BASE+'record-text-q4-contract.json'
CONTRACT_SHA = 'b765ead131f474cd46932a62b092c60c7556889e54e4ac0df0f87d5a8ca500c6'
SUITE = BASE+'record-text-q4.suite.ad'
FIELDS = 'family case profile caps source_file source_sha256 oracle_kind oracle_value rust_reference rust_reference_sha256 c_reference c_reference_sha256 rust_debug_runs rust_release_runs rust_release_flags c_runs execution'.split()
FAMILIES = ('record-text-cert-925','record-text-base')
RELEASE_CASES = {'r07','r08','t03','t05'}
RELEASE_SOURCE = 'experimental/ace/crates/a2x-worker/tests/cases/record_text_902.rs'
RELEASE_SOURCE_SHA = '49bb4f15b6191cbce2ec1ee2edcd20e99be351aa3832ec1a01ad28614d466acb'
TRACES = {'T_CHAR_AT_NEG':b'LIT len=3\nLIT id=1\nRT-FAIL code=-10\nFREE id=1\n',
          'T_SLICE_OOB':b'LIT len=6\nLIT id=1\nRT-FAIL code=-10\nFREE id=1\n',
          'T_SURROGATE':b'FROM-CP cp=55296\nRT-FAIL code=-8\n'}


def reference_contracts(inputs):
    for path,digest in [(REFERENCE,REFERENCE_SHA),(Q4_CATALOG,Q4_CATALOG_SHA),
                        (COMPOSITE_CATALOG,COMPOSITE_CATALOG_SHA),(RELEASE_SOURCE,RELEASE_SOURCE_SHA)]:
        inputs.sha256(path,digest)
    expected={(row['family'],row['case']):row for row in read_tsv(inputs,Q4_CATALOG,Q4_FIELDS)
              if row['family'] in FAMILIES}
    originals={(row['family'],row['case']):row for row in read_tsv(inputs,COMPOSITE_CATALOG,COMPOSITE_FIELDS)
               if row['family']==FAMILIES[0]}
    typed_equal(len(expected),16,'original record/text source count')
    seen={}
    for row in read_tsv(inputs,REFERENCE,FIELDS):
        key=(row['family'],row['case']);require(key in expected and key not in seen,'foreign/duplicate record/text reference')
        source=expected[key]
        for name in ('profile','caps','source_file','source_sha256','oracle_kind','oracle_value'):
            typed_equal(row[name],source[name],'original record/text '+name)
        require(row['profile'] in ('record','text','text-record'),'record/text source profile')
        caps,_=inputs.decode_json(row['caps'].encode());require(caps in ([],['ac.runtime.text.v1']),'record/text original caps')
        require(row['oracle_kind']=='value' or row['oracle_kind']=='trap' and row['oracle_value'] in AV_TRAPS,'record/text original oracle')
        stem=row['family']+'--'+row['case']
        typed_equal(row['rust_reference'],BASE+'record-text-reference/'+stem+'.rs','independent record/text Rust reference')
        if key in originals:
            typed_equal((row['c_reference'],row['c_reference_sha256']),
                        (originals[key]['reference_file'],originals[key]['reference_sha256']),'unchanged original C reference')
        else:typed_equal(row['c_reference'],BASE+'record-text-reference/'+stem+'.c','independent base C reference')
        for target in ('rust','c'):inputs.sha256(row[target+'_reference'],row[target+'_reference_sha256'])
        inputs.sha256(row['source_file'],row['source_sha256'])
        release=row['family']==FAMILIES[1] and row['case'] in RELEASE_CASES
        typed_equal((row['rust_debug_runs'],row['rust_release_runs'],row['c_runs']),('1','1' if release else '0','1'),'original record/text execution configurations')
        flags,_=inputs.decode_json(row['rust_release_flags'].encode());typed_equal(flags,['-O'] if release else [],'original 902 release flags')
        if release:
            original='crates/auto-lang/test/cases/ace-m1/src/'+row['case'].upper()+'.at'
            inputs.sha256(original,row['source_sha256'])
        typed_equal(row['execution'],'UNOBSERVED','reference registration is not execution')
        seen[key]=row
    typed_equal(set(seen),set(expected),'complete original record/text reference set')
    return seen


def admit_source(inputs,evidence,binding,workers):
    from ace_m1_first_slice import _inventory_for_suite,_compact_admission,_repo_relative,_wires,retain_parent_status
    exact_keys(binding,('inventory','copy','dependencies_before','dependencies_after'),'record/text execution bindings')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT)
    contracts=reference_contracts(inputs);snapshots=[]
    for field,label in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        record,_=evidence.read_json(binding[field]);exact_keys(record,('schema','snapshot','source_pins'),'record/text dependency snapshot')
        typed_equal(record['schema'],'ace.m1.rt-q4-dependencies/1','record/text dependency schema')
        typed_equal(record['snapshot'],label,'record/text dependency phase');exact_keys(record['source_pins'],DEPENDENCIES,'record/text dependencies')
        for name,digest in record['source_pins'].items():inputs.sha256(name,digest)
        snapshots.append(record['source_pins'])
    typed_equal(*snapshots,'stable record/text execution dependencies')
    for key,name in [('av','av-worker.exe'),('native','mn-worker.exe')]:
        typed_equal(snapshots[0]['experimental/ace/target/release/'+name],workers[key],'record/text original worker')
    inventory=_inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inventory)
    cases={'ace.m1.rtq4.'+family+'.'+case.lower():row for (family,case),row in contracts.items()}
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
            return retain_parent_status(text_observation(copy.scope,wire,check,contract,record_family=True,trap_trace=TRACES.get(contract['case'])),parent['status'])
        observations[slot]=_compact_admission(inputs,read_one,32<<10)
        if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
    typed_equal(seen,{'q4-source/'+family+'/'+case+'/'+target for family,case in contracts for target in ('AV','AC-native')},'complete original record/text source execution set')
    return dict(schema='ace.m1.record-text-source-observations/1',required=32,observations=observations,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                semantic_evidence_complete=all(v['judgement']=='PASS' for v in observations.values()),
                full_resource_and_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)


def admit_certifications(inputs,evidence,binding_path,release_binding=None):
    from ace_m1_first_slice import _compact_admission,_repo_relative
    from ace_m1_cert_evidence import record_text_certification
    from ace_m1_daily_evidence import certified_daily_emission
    contracts=reference_contracts(inputs);document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'record/text certification bindings');producer=document['producer']
    exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'record/text certification producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','record/text actual producer')
    typed_equal(producer['cwd'],str(inputs.root),'record/text producer worktree')
    require(type(document['rows']) is list,'record/text certification rows')
    release=None
    if release_binding is not None:
        from ace_m1_scalar_release_evidence import admit_scalar_release
        release_contracts={case:row for (family,case),row in contracts.items() if row['rust_release_runs']=='1'}
        pins={}
        for row in document['rows']:
            exact_keys(row,('family','case','prefix','pin'),'record/text release certification row')
            key=(row['family'],row['case']);require(key in contracts and type(row['pin']) is dict,'record/text release certified source')
            require(row['pin'].get('target') in ('rust','c'),'record/text release certified target')
            if key[0]=='record-text-base' and key[1] in release_contracts and row['pin']['target']=='rust':
                require(key[1] not in pins,'duplicate record/text release certification');pins[key[1]]=row['pin']
        typed_equal(set(pins),set(release_contracts),'original release default certifications')
        release=admit_scalar_release(inputs,evidence,release_binding,release_contracts,pins,record_text=True)
    seen,observations=set(),{}
    for row in document['rows']:
        exact_keys(row,('family','case','prefix','pin'),'record/text certification row')
        key=(row['family'],row['case']);require(key in contracts and type(row['pin']) is dict,'record/text certification identity')
        pin=row['pin'];exact_keys(pin,('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'record/text certification pins')
        target=pin['target'];require(target in ('rust','c') and (*key,target) not in seen,'duplicate/foreign record/text certification target');seen.add((*key,target))
        contract=contracts[key];typed_equal(pin['identity_request'],{'op':'identity','certification_target':target},'record/text compiler identity target')
        typed_equal(pin['request']['case_id'],'941/record-text/'+key[0]+'/'+key[1],'record/text original certification case')
        path=PurePosixPath(row['prefix']);home=PurePosixPath(binding_path).parent
        require(path.parent.parent==home/(key[0]+'--'+key[1]) and path.name==target
                and path.parent.name.startswith('attempt-') and path.parent.name[8:].isdigit()
                and int(path.parent.name[8:])>0,'record/text certification attempt owner')
        value=_compact_admission(inputs,lambda _:certified_daily_emission(evidence,inputs,row['prefix'],pin,producer,
            record_text_certification(evidence,inputs,row['prefix'],pin,producer,contract)),128<<10)
        from ace_m1_scalar_cert_evidence import required_variants
        value=required_variants(value,contract,target,release['observations'].get(key[1]) if release is not None and key[0]=='record-text-base' else None)
        observations['q4-source/'+key[0]+'/'+key[1]+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=value
    typed_equal(seen,{(family,case,target) for family,case in contracts for target in ('rust','c')},'complete record/text certification set')
    result=dict(schema='ace.m1.record-text-certification-observations/1',required=32,observations=observations,
                contract_sha256=REFERENCE_SHA,bindings_sha256=digest,
                actual_default_certifications_complete=all(v['default_certification_judgement']=='PASS' for v in observations.values()),
                required_extra_rust_release_runs=4,rust_release_evidence='UNOBSERVED' if release is None else 'PASS' if release['complete'] else 'FAIL',
                certification_complete=all(v['judgement']=='PASS' for v in observations.values()),
                daily_admission='PASS' if all(v['daily_judgement']=='PASS' for v in observations.values()) else 'FAIL',can_claim_expanded_m1=False)

    if release is not None:result['rust_release_execution']=release
    return result
