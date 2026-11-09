"""Read original optimized scalar compile/run observations, never launch them."""
import json
import re
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import exact_keys
from ace_m1_case_evidence import require
from ace_m1_cert_evidence import _terminal, _toolchain
from ace_m1_legacy import typed_equal

NAME = 'scalar_release_941::original_eight_release_variants'
BINARY = 'ace-a2x-worker::conformance'
SOURCE = 'experimental/ace/crates/a2x-worker/tests/cases/scalar_release_941.rs'
FLAGS = ['--edition','2021','-O','-C','debug-assertions=off']
DEPENDENCIES = (
    'experimental/ace/target/debug/a2x-worker.exe', SOURCE,
    'experimental/ace/crates/a2x-worker/tests/conformance.rs',
    'experimental/ace/crates/a2x-worker/tests/cases/scalar_semantics.rs',
    'crates/auto-lang/test/cases/ace-m1-expanded/scalar-reference-contracts.tsv',
    'experimental/ace/crates/core/src/link.rs', '.config/nextest-full.toml')


RT_NAME = 'record_text_release_941::original_four_release_variants'
RT_SOURCE = 'experimental/ace/crates/a2x-worker/tests/cases/record_text_release_941.rs'
RT_FLAGS = ['--edition','2021','-O']
RT_DEPENDENCIES = (DEPENDENCIES[0],RT_SOURCE,SOURCE,DEPENDENCIES[2],
    'experimental/ace/crates/a2x-worker/tests/cases/record_text_902.rs',
    'crates/auto-lang/test/cases/ace-m1-expanded/record-text-reference-contracts.tsv',
    *DEPENDENCIES[-2:])


QUERY_NAME = 'query_release_941::original_two_release_variants'
QUERY_SOURCE = 'experimental/ace/crates/a2x-worker/tests/cases/query_release_941.rs'
QUERY_FLAGS = ['--edition','2021','-O']
QUERY_DEPENDENCIES = (DEPENDENCIES[0],QUERY_SOURCE,SOURCE,DEPENDENCIES[2],
    'experimental/ace/crates/a2x-worker/tests/cases/choose_instances_935.rs',
    'crates/auto-lang/test/cases/ace-m1-expanded/query-reference-contracts.tsv',*DEPENDENCIES[-2:])


def _mode(record_text,query_family=False):
    require(type(record_text) is bool and type(query_family) is bool and not(record_text and query_family),'finite release observation domain')
    if query_family:return QUERY_NAME,QUERY_SOURCE,QUERY_FLAGS,QUERY_DEPENDENCIES,'query',2
    return (RT_NAME,RT_SOURCE,RT_FLAGS,RT_DEPENDENCIES,'record-text',4) if record_text else (NAME,SOURCE,FLAGS,DEPENDENCIES,'scalar',8)


def _path(actual, expected, where):
    require(type(actual) is str,'release path must be text')
    typed_equal(PureWindowsPath(actual),PureWindowsPath(expected),where)


def release_phase(value, argv, cwd, maximum_ms):
    exact_keys(value,('argv','cwd','deadline_ms','wall_ms','terminal'),'release captured process')
    require(type(value['argv']) is list and len(value['argv'])==len(argv),'release process argv count')
    for actual,expected in zip(value['argv'],argv):
        require(type(actual) is str,'release process argument text')
        if PureWindowsPath(expected).is_absolute():_path(actual,expected,'release process path argument')
        else:typed_equal(actual,expected,'release exact process argument')
    _path(value['cwd'],cwd,'release process working directory')
    for key in ('deadline_ms','wall_ms'):
        require(type(value[key]) is int and 0<=value[key]<=maximum_ms,'release phase original budget')
    require(value['deadline_ms']>0 and value['wall_ms']<=value['deadline_ms'],'release phase deadline')
    _terminal(value['terminal'],'release raw process terminal')
    return value['terminal']


def _reference(evidence, ref):
    exact_keys(ref,('path','sha256'),'release document reference')
    evidence.sha256(ref['path'],ref['sha256'])
    return evidence.read_json(ref['path'])[0]


def admit_scalar_release(inputs,evidence,binding,contracts,pins,*,record_text=False,query_family=False):
    name,source,flags,dependencies,domain,count=_mode(record_text,query_family)
    from ace_m1_first_slice import _repo_relative, _compact_admission
    from ace_m1_scalar_cert_evidence import CONTRACT_SHA
    from ace_m1_obligation_catalog import SCALAR_TABLE_SHA
    if record_text:
        from ace_m1_record_text_evidence import REFERENCE_SHA,RELEASE_SOURCE_SHA
        CONTRACT_SHA,SCALAR_TABLE_SHA=REFERENCE_SHA,RELEASE_SOURCE_SHA
    elif query_family:
        from ace_m1_query_evidence import REFERENCE_SHA,RELEASE_SOURCE_SHA
        CONTRACT_SHA,SCALAR_TABLE_SHA=REFERENCE_SHA,RELEASE_SOURCE_SHA
    exact_keys(binding,('index','parent','discovery','dependencies_before','dependencies_after'),'release binding')
    index=_reference(evidence,binding['index'])
    exact_keys(index,('schema','flags','producer','required','cases','compiler_identity'),'release index')
    typed_equal(index['schema'],'ace.m1.'+domain+'-release-index/1','release schema')
    typed_equal(index['flags'],flags,'original release flags')
    typed_equal(index['required'],count,'original release count')
    producer=index['producer']
    exact_keys(producer,('role','source_sha256','table_sha256','reference_contract_sha256','worker_path','execution_path','rustc'),'release original producer')
    typed_equal(producer['role'],'ace-a2x-worker::conformance::'+name,'release original role')
    inputs.sha256(source,producer['source_sha256'])
    typed_equal(producer['table_sha256'],SCALAR_TABLE_SHA,'original source contract table')
    typed_equal(producer['reference_contract_sha256'],CONTRACT_SHA,'independent reference contract')
    _path(producer['worker_path'],str(inputs.root.joinpath(*dependencies[0].split('/'))),'release original worker')
    executable=_repo_relative(inputs,producer['execution_path'])
    require(PurePosixPath(executable).parent.as_posix()=='experimental/ace/target/debug/deps'
            and re.fullmatch(r'conformance-[0-9a-f]+\.exe',PurePosixPath(executable).name) is not None,
            'release original test executable')
    snapshots=[]
    for key,phase in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        snapshot,_=evidence.read_json(binding[key])
        exact_keys(snapshot,('schema','snapshot','source_pins'),'release dependency snapshot')
        typed_equal(snapshot['schema'],'ace.m1.'+domain+'-release-dependencies/1','release dependency schema')
        typed_equal(snapshot['snapshot'],phase,'release dependency phase')
        exact_keys(snapshot['source_pins'],set(dependencies)|{executable},'release whole producer dependencies')
        snapshots.append(snapshot['source_pins'])
    typed_equal(*snapshots,'release stable dependencies')
    for path_name,digest in snapshots[0].items():inputs.sha256(path_name,digest)
    parent=_reference(evidence,binding['parent'])
    exact_keys(parent,('outcome','executed','selected','excluded','preparation','execution','bound_inputs',
                       'child_observations','child_validation','results','not_run','native_unselected',
                       'max_parallel_tests_observed','overlapping_tests','errors','tools','groups'),
               'release original fixed-nextest result')
    require(type(parent['results']) is dict and type(parent['execution']) is dict,'release original parent records')
    typed_equal(parent.get('outcome'),'PASS','release original parent outcome')
    typed_equal(parent.get('groups'),['ace-a2x'],'release formal group')
    selected=parent.get('selected')
    require(type(selected) is list and bool(selected),'release original parent selection')
    identities=set()
    for row in selected:
        exact_keys(row,('binary','test','ignored'),'release original selected test')
        require(type(row['binary']) is str and bool(row['binary']) and type(row['test']) is str and bool(row['test']),
                'release original selected identity')
        typed_equal(row['ignored'],False,'release selected test must execute')
        key=(row['binary'],row['test']);require(key not in identities,'release duplicate parent selection');identities.add(key)
        result=parent['results'].get(row['binary']+'$'+row['test'])
        require(type(result) is dict and result.get('event')=='ok','release original selected terminal')
    require((BINARY,name) in identities,'original release producer not selected')
    typed_equal(parent.get('executed'),len(selected),'release original parent execution')
    require(parent.get('not_run')==[] and parent.get('errors')==[],'release parent omitted/failed work')
    typed_equal(parent['results'][BINARY+'$'+name]['event'],'ok','release original test terminal')
    require(parent['execution']['exit']==0 and type(parent['execution']['exit']) is int,'release original Job exit')
    typed_equal(PurePosixPath(binding['discovery']['path']),PurePosixPath(binding['parent']['path']).parent/'discovery.stdout','release original discovery path')
    discovery=_reference(evidence,binding['discovery'])
    require(type(discovery) is dict and type(discovery.get('rust-suites')) is dict
            and type(discovery['rust-suites'].get(BINARY)) is dict,'release original executable discovery')
    _path(discovery['rust-suites'][BINARY]['binary-path'],producer['execution_path'],'release executed producer')
    root=evidence.path(binding['index']['path']).parent
    rustc=producer['rustc'];require(type(rustc) is str and bool(rustc),'release original compiler')
    vv=release_phase(index['compiler_identity'],[rustc,'-vV'],str(root),120_000)
    typed_equal(vv['stderr_hex'],'','release compiler identity stderr')
    expected={case for case,row in contracts.items() if row['rust_release_runs']=='1'}
    typed_equal(len(expected),count,'release original required set')
    require(type(index['cases']) is list,'release case references')
    seen,observations=set(),{}
    for ref in index['cases']:
        exact_keys(ref,('case','path','sha256'),'release case reference')
        case=ref['case'];require(case in expected and case not in seen,'foreign/duplicate release case');seen.add(case)
        typed_equal(ref['path'],case+'/receipt.json','release case path')
        relative=(PurePosixPath(binding['index']['path']).parent/ref['path']).as_posix()
        evidence.sha256(relative,ref['sha256'])
        def read_one(_):
            record,_=evidence.read_json(relative)
            value=scalar_release_observation(inputs,evidence,root,producer,record,contracts[case],pins[case],vv,record_text=record_text,query_family=query_family)
            return dict(value,receipt_sha256=ref['sha256'])
        observations[case]=_compact_admission(inputs,read_one,32<<10)
    typed_equal(seen,expected,'complete original release observations')
    return dict(schema='ace.m1.'+domain+'-release-observations/1',required=count,observations=observations,
                complete=all(v['judgement']=='PASS' for v in observations.values()),
                parent_sha256=binding['parent']['sha256'],index_sha256=binding['index']['sha256'])


def scalar_release_observation(inputs,evidence,root,producer,record,contract,pin,vv,*,record_text=False,query_family=False):
    _,_,flags,_,domain,_=_mode(record_text,query_family)
    from ace_m1_first_slice import _repo_relative
    exact_keys(record,('schema','case','profile','source_path','source_sha256','reference_path','reference_sha256',
                       'request','flags','actual_files','processes','daily','whole_wall_ms'),'release case observation')
    typed_equal(record['schema'],'ace.m1.'+domain+'-release-execution/1','release case schema')
    if record_text:
        require(contract['family']=='record-text-base' and contract['case'] in {'r07','r08','t03','t05'},'original record/text release set')
        typed_equal(json.loads(contract['rust_release_flags']),['-O'],'original record/text optimization flags')
    if query_family:
        require(contract['family']=='source-instances' and contract['case'] in {'choose42','choose0'},'original query release source set')
        typed_equal(json.loads(contract['rust_release_flags']),['-O'],'original query optimization flags')
    case=contract['case'];typed_equal(record['case'],case,'release case identity')
    typed_equal(record['profile'],contract['profile'],'release original profile')
    typed_equal(record['flags'],flags,'release exact flags')
    require(type(record['whole_wall_ms']) is int and 0<=record['whole_wall_ms']<=360_000,'release original whole budget')
    for key,name in [('source',contract['source_file']),('reference',contract['rust_reference'])]:
        typed_equal(_repo_relative(inputs,record[key+'_path']),name,'release independent '+key)
        typed_equal(record[key+'_sha256'],contract[key+'_sha256'] if key=='source' else contract['rust_reference_sha256'],'release '+key+' hash')
        inputs.sha256(name,record[key+'_sha256'])
    _toolchain(pin['toolchain'],'rust')
    typed_equal(pin['source_sha256'],contract['source_sha256'],'release certified source')
    typed_equal(pin['reference_sha256'],contract['rust_reference_sha256'],'release certified reference')
    expected_stdout=(contract['oracle_kind']+':'+contract['oracle_value']+'\n').encode().hex()
    expected_stderr=''.join(mark+'\n' for mark in ([] if record_text else json.loads(contract['marks']))).encode().hex()
    typed_equal(pin['expected_stdout_hex'],expected_stdout,'release independent stdout oracle')
    typed_equal(pin['expected_stderr_hex'],expected_stderr,'release independent marks oracle')
    typed_equal(vv['stdout_hex'],pin['toolchain']['version_vv'].encode().hex(),'release actual compiler version')
    directory=root/case;actual=directory/'actual';reference=directory/'reference'
    request=record['request']
    exact_keys(request,('op','source_path','profile','entry','caps','out_dir','nonce'),'release original emission request')
    for key,want in [('op','daily-emit'),('profile',contract['profile']),('entry','main'),('caps',json.loads(contract['caps']))]:
        typed_equal(request[key],want,'release emission '+key)
    _path(request['source_path'],record['source_path'],'release original emission source')
    _path(request['out_dir'],str(actual),'release complete output directory')
    require(type(request['nonce']) is str and re.fullmatch('[0-9a-f]{64}',request['nonce']) is not None,'release original nonce')
    daily=release_phase(record['daily'],[producer['worker_path'],json.dumps(request,ensure_ascii=False,separators=(',',':'))],str(actual),360_000)
    typed_equal(daily['stderr_hex'],'','release emitter stderr')
    evidence.reserve(len(daily['stdout_hex'])//2)
    response,_=evidence.decode_json(bytes.fromhex(daily['stdout_hex']))
    exact_keys(response,('status','tool','protocol','recipe','mode','nonce','profile','input_hash','source_sha256',
                         'entry_fn','manifest_sha256','files','spawned_rustc'),'release original emission response')
    for key,want in [('status','ok'),('tool','a2x-worker'),('protocol','a2xworker/1'),('recipe','ac-a2r/hir-record-text-emit-v1' if record_text else 'ac-a2r/hir-scalar-emit-v1'),
                     ('mode','daily-emit'),('nonce',request['nonce']),('profile','auto.source.'+contract['profile']+('.draft' if record_text else '-i32.draft')),
                     ('source_sha256',contract['source_sha256']),('entry_fn','fn_main'),('spawned_rustc',0)]:
        typed_equal(response[key],want,'release emission response '+key)
    from ace_m1_scalar_evidence import _fnv1a64
    typed_equal(response['input_hash'],_fnv1a64(inputs.read_bytes(contract['source_file'])),'release source FNV')
    exact_keys(pin['actual_files'],('main.rs','a2x_runtime.rs','generated.rs','manifest.json'),'release full certified file set')
    typed_equal(record['actual_files'],pin['actual_files'],'same complete certified release fileset')
    typed_equal(response['manifest_sha256'],pin['actual_files']['manifest.json'],'release original manifest')
    for name,digest in pin['actual_files'].items():
        evidence.sha256(actual.relative_to(evidence.root).as_posix()+'/'+name,digest)
    require(type(response['files']) is list and len(response['files'])==3,'release complete emitter file list')
    names=set()
    for ref in response['files']:
        exact_keys(ref,('path','sha256','bytes'),'release emitted file')
        name=ref['path'];require(name in pin['actual_files'] and name!='manifest.json' and name not in names,'release emitted file identity')
        names.add(name);typed_equal(ref['sha256'],pin['actual_files'][name],'release emitter original file hash')
        typed_equal(ref['bytes'],(actual/name).stat().st_size,'release emitter original file bytes')
    typed_equal(names,set(pin['actual_files'])-{'manifest.json'},'release complete emitter file set')
    copied_reference=reference.relative_to(evidence.root).as_posix()+'/main.rs'
    evidence.sha256(copied_reference,contract['rust_reference_sha256'])
    exact_keys(record['processes'],('actual','reference'),'release both process legs')
    matches=[]
    for leg,work in [('actual',actual),('reference',reference)]:
        pair=record['processes'][leg];exact_keys(pair,('compile','run'),'release process pair')
        exe=str(work/'release.exe')
        release_phase(pair['compile'],[producer['rustc'],*flags,'-o',exe,'main.rs'],str(work),120_000)
        terminal=release_phase(pair['run'],[exe],str(work),30_000)
        matches.append(terminal['stdout_hex']==pin['expected_stdout_hex'] and terminal['stderr_hex']==pin['expected_stderr_hex'])
    return dict(schema='ace.m1.'+domain+'-release-observation/1',source_sha256=contract['source_sha256'],
                reference_sha256=contract['rust_reference_sha256'],flags=flags,
                actual_matches_oracle=matches[0],reference_matches_oracle=matches[1],
                judgement='PASS' if all(matches) else 'FAIL')
