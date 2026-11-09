"""Original scalar corpus certification; release obligations remain explicit."""
from pathlib import PurePosixPath

from ace_m1_arch_acceptance import exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import BASE, SCALAR_CATALOG, SCALAR_FIELDS, read_tsv, admit_scalar_catalog

CONTRACT = BASE+'scalar-reference-contracts.tsv'
CONTRACT_SHA = '4d4c5a3904f0a5955855f6127aa388ba6ae1a14cc4df8858b00b05aa6d53a094'
FIELDS = 'case source_file source_sha256 profile caps oracle_kind oracle_value marks rust_debug_runs rust_release_runs c_runs rust_reference rust_reference_sha256 c_reference c_reference_sha256 execution'.split()


def required_variants(certificate, contract, target, release_observation=None):
    require(target in ('rust','c'),'scalar certification variant target')
    require(contract['rust_release_runs'] in ('0','1'),'scalar original release count')
    require(certificate['judgement'] in ('PASS','FAIL'),'scalar default certification verdict')
    release=target=='rust' and contract['rust_release_runs']=='1'
    value=dict(certificate,default_certification_judgement=certificate['judgement'],
               required_rust_release=release,rust_release_evidence='UNOBSERVED' if release else 'NOT_REQUIRED')
    if release:
        if release_observation is None:
            if value['judgement']=='PASS':value['judgement']='UNOBSERVED'
        else:
            require(release_observation['judgement'] in ('PASS','FAIL'),'scalar release observation verdict')
            value['rust_release_evidence']=release_observation['judgement']
            value['rust_release_receipt_sha256']=release_observation['receipt_sha256']
            value['judgement']='PASS' if certificate['judgement']==release_observation['judgement']=='PASS' else 'FAIL'
    return value


def scalar_reference_contracts(inputs):
    inputs.sha256(CONTRACT,CONTRACT_SHA)
    original=read_tsv(inputs,SCALAR_CATALOG,SCALAR_FIELDS);admit_scalar_catalog(inputs,original)
    expected={row['case']:row for row in original}
    rows=read_tsv(inputs,CONTRACT,FIELDS)
    seen={}
    for row in rows:
        case=row['case']
        require(case in expected and case not in seen,'foreign/duplicate scalar reference')
        source=expected[case]
        for key in ('source_file','source_sha256','profile','oracle_kind','oracle_value','rust_debug_runs','rust_release_runs','c_runs'):
            typed_equal(row[key],source[key],'original scalar reference '+key)
        caps,_=inputs.decode_json(row['caps'].encode());marks,_=inputs.decode_json(row['marks'].encode())
        typed_equal(caps,['hir.test.trace.v1'] if source['trace']=='true' else [],'original scalar reference capabilities')
        typed_equal(marks,source['marks'].split(',') if source['marks'] else [],'original scalar reference marks')
        typed_equal(row['execution'],'UNOBSERVED','reference registration is not execution')
        for target,ext in [('rust','rs'),('c','c')]:
            typed_equal(row[target+'_reference'],BASE+'scalar-reference/'+case+'.'+ext,'independent scalar reference location')
            inputs.sha256(row[target+'_reference'],row[target+'_reference_sha256'])
        seen[case]=dict(row,trace=source['trace'])
    typed_equal(set(seen),set(expected),'complete original 61-source reference set')
    return seen


def admit_scalar_certifications(inputs,evidence,binding_path,release_binding=None):
    from ace_m1_first_slice import _compact_admission, _repo_relative
    from ace_m1_daily_evidence import certified_daily_emission
    from ace_m1_cert_evidence import scalar_corpus_certification
    contracts=scalar_reference_contracts(inputs)
    document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'scalar corpus certification bindings')
    producer=document['producer']
    exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'scalar corpus producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','scalar corpus actual producer')
    typed_equal(producer['cwd'],str(inputs.root),'scalar corpus producer worktree')
    require(type(document['rows']) is list,'scalar corpus certification rows')
    release=None
    if release_binding is not None:
        from ace_m1_scalar_release_evidence import admit_scalar_release
        pins={}
        for row in document['rows']:
            exact_keys(row,('case','prefix','pin'),'scalar release certification row')
            require(row['case'] in contracts and type(row['pin']) is dict,'scalar release certification identity')
            exact_keys(row['pin'],('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'scalar release certification pins')
            require(row['pin']['target'] in ('rust','c'),'scalar release certification target')
            if row['pin']['target']=='rust':
                require(row['case'] not in pins,'duplicate release certification source')
                pins[row['case']]=row['pin']
        require({case for case,row in contracts.items() if row['rust_release_runs']=='1'}<=pins.keys(),
                'release default certifications missing')
        release=admit_scalar_release(inputs,evidence,release_binding,contracts,pins)
    seen,observations=set(),{}
    for row in document['rows']:
        exact_keys(row,('case','prefix','pin'),'scalar corpus certification row')
        case,pin=row['case'],row['pin']
        require(case in contracts and type(pin) is dict,'unknown scalar certification case')
        exact_keys(pin,('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'scalar certification pins')
        target=pin['target'];require(target in ('rust','c') and (case,target) not in seen,'foreign/duplicate scalar certification target');seen.add((case,target))
        contract=contracts[case]
        typed_equal(pin['request']['case_id'],'941/scalar/'+case,'scalar original certification identity')
        typed_equal(pin['identity_request'],{'op':'identity','certification_target':target},'scalar original toolchain target')
        # Attempts preserve failed raw commands; each accepted row names exactly
        # one immutable attempt under this case, with daily and cert adjacent.
        path=PurePosixPath(row['prefix']);parent=PurePosixPath(binding_path).parent
        require(path.parent.parent==parent/case and path.name==target
                and path.parent.name.startswith('attempt-') and path.parent.name[8:].isdigit()
                and int(path.parent.name[8:])>0,'scalar certification attempt owner')
        value=_compact_admission(inputs,lambda _: certified_daily_emission(evidence,inputs,row['prefix'],pin,producer,
            scalar_corpus_certification(evidence,inputs,row['prefix'],pin,producer,contract)),128<<10)
        value=required_variants(value,contract,target,release['observations'].get(case) if release is not None else None)
        key='q4-source/scalar/'+case+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')
        observations[key]=value
    typed_equal(seen,{(case,target) for case in contracts for target in ('rust','c')},'complete original 122 scalar certifications')
    result=dict(schema='ace.m1.scalar-certification-observations/1',required=122,contract_sha256=CONTRACT_SHA,
                bindings_sha256=digest,observations=observations,
                actual_default_certifications_complete=all(v['default_certification_judgement']=='PASS' for v in observations.values()),
                required_extra_rust_release_runs=8,rust_release_evidence='UNOBSERVED' if release is None else 'PASS' if release['complete'] else 'FAIL',
                certification_complete=all(v['judgement']=='PASS' for v in observations.values()),
                daily_admission='PASS' if all(v['daily_judgement']=='PASS' for v in observations.values()) else 'FAIL',
                can_claim_expanded_m1=False)
    if release is not None:result['rust_release_execution']=release
    return result
