"""Original SQ/instance source consumers, with typed source diagnostics."""
import hashlib
import json
import os
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import EvidenceScope, AdmissionError, counter, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import Q4_CATALOG,Q4_CATALOG_SHA,Q4_FIELDS,read_tsv
from ace_m1_scalar_q4_evidence import scalar_q4_observation

CONTRACTS = {'source-query': {'path': 'crates/auto-lang/test/cases/ace-m1-expanded/source-query-q4-contract.json', 'sha256': 'ebaeb34be0d3193174e0cdc57f0d9d5ad7057e60e3396139646369884a3691d7', 'suite': 'crates/auto-lang/test/cases/ace-m1/source-query-934/source-query-934.suite.ad', 'checks': 14}, 'source-instances': {'path': 'crates/auto-lang/test/cases/ace-m1-expanded/source-instances-q4-contract.json', 'sha256': 'ec5f37f2f6265d485f62222cac9551088e3f59781772386e484f1f1ce6864d64', 'suite': 'crates/auto-lang/test/cases/ace-m1/source-instances-935/source-instances-935.suite.ad', 'checks': 12}}
DEPENDENCIES = ('experimental/ace/target/release/av-worker.exe', 'experimental/ace/target/release/mn-worker.exe', 'experimental/ace/target/release/a2x-worker.exe', 'experimental/ace/target/runtime-dev/ace_trace_runtime.lib', 'experimental/ace/crates/av/src/driver.rs', 'experimental/ace/crates/native/src/driver.rs', 'experimental/ace/crates/a2x-store/src/lib.rs', 'experimental/ace/crates/source-query/src/consumer.rs', 'experimental/ace/crates/source-query/src/produce.rs', 'experimental/ace/crates/source-query/src/instances.rs', 'experimental/ace/crates/source-query/src/instance_host.rs', 'experimental/ace/crates/source-query/src/facts.rs', 'experimental/ace/crates/source-query/src/host.rs')
PREFIX = {'source-query':'ace.m1.sq934.','source-instances':'ace.m1.inst935.'}
ORACLES = {'source-query':{'leaf42':('value','42'),'leaf43':('value','43'),'named7':('value','7'),'trace-ab7':('value','7'),
                         'neg-bool-plus':('diagnostic','source.type-mismatch'),'neg-not-int':('diagnostic','source.type-mismatch')},
           'source-instances':{'choose42':('value','42'),'choose0':('value','0'),'expected-type':('value','42'),
                               'cross-wrap':('value','40'),'recursion':('value','40'),'neg-hetero':('diagnostic','type-mismatch')}}


def source_contracts(inputs):
    inputs.sha256(Q4_CATALOG,Q4_CATALOG_SHA);rows={}
    for row in read_tsv(inputs,Q4_CATALOG,Q4_FIELDS):
        family,case=row['family'],row['case']
        if family not in CONTRACTS:continue
        require(case in ORACLES[family] and (family,case) not in rows,'original SQ/instances source identity')
        typed_equal((row['oracle_kind'],row['oracle_value']),ORACLES[family][case],'independent source oracle')
        typed_equal(row['profile'],'scalar','original concrete scalar domain')
        caps=['ac.source.instances.scalar/v1'] if family=='source-instances' else ['hir.test.trace.v1'] if case=='trace-ab7' else []
        typed_equal(inputs.decode_json(row['caps'].encode())[0],caps,'original source capabilities')
        marks=['mark_a','mark_b'] if case=='trace-ab7' else []
        typed_equal(inputs.decode_json(row['marks'].encode())[0],marks,'independent original marks')
        inputs.sha256(row['source_file'],row['source_sha256'])
        rows[(family,case)]=dict(row,trace='true' if marks else 'false',marks=','.join(marks))
    typed_equal(set(rows),{(family,case) for family,cases in ORACLES.items() for case in cases},'all twelve original query/instance sources')
    return rows


def native_source_diagnostic(response,source,contract,*,data_family=False,legacy_base=False):
    """Read the original 911 cold fact; no rendered-message diagnostic parsing."""
    require(type(data_family) is bool and type(legacy_base) is bool and not (data_family and legacy_base),'finite native diagnostic source family')
    if legacy_base:
        from ace_m1_legacy_base import DIAGNOSTICS,diagnostic_span as legacy_span
        require(contract['case'] in ('B17','B18','B19') and contract['oracle_kind']=='diagnostic','finite original BASE native diagnostic')
    profile='scalar';attempted=['scalar'];surface=[]
    if data_family:
        from ace_m1_data_diagnostics import ORACLES,diagnostic_span
        require(contract['case'] in ORACLES and contract['oracle_kind']=='diagnostic','finite original data diagnostic')
        profile=ORACLES[contract['case']][2]
        # PLAN-999 (T-03) reception: the ladder attempts a Core side face
        # between scalar and the monotone rungs — it appears in `attempted`
        # but never lists a surface rejection (failures never terminate).
        attempted=['scalar','core','record','text-record'][:{'record':3,'text-record':4}[profile]]
        surface=[dict(profile=p,codes=['source.unsupported']) for p in attempted[:-1] if p!='core']
    snap=response['source_error']
    exact_keys(snap,('schema','mode','producer','profile','attempted','surface_rejections','path','source_sha256','source_len','span_basis','causes','primary','operation'),'native typed source fact')
    digest=hashlib.sha256(source).hexdigest()
    for key,want in [('schema','mn.source-diagnostic/1'),('mode','source-v1'),('producer','ace-core.source'),('profile',profile),
                     ('attempted',attempted),('surface_rejections',surface),('path','mn.at'),('source_sha256',digest),('source_len',len(source)),('span_basis','utf8-byte')]:
        typed_equal(snap[key],want,'native original diagnostic '+key)
    typed_equal(snap['operation'],dict(kind='cold',step=None,base_sha256=None,candidate_sha256=digest,session_revision=None,edit=None),'native source fact operation binding')
    require(type(snap['causes']) is list and 1<=len(snap['causes'])<=64,'original native diagnostic cause cap')
    typed_equal(snap['primary'],0,'compiler first diagnostic is primary')
    for cause in snap['causes']:
        exact_keys(cause,('stage','code','message','span'),'native source cause')
        require(cause['stage'] in ('text','bind','verify'),'native raw source stage')
        for key in ('code','message'):
            require(type(cause[key]) is str and 0<len(cause[key].encode())<=4096,'original native source message cap')
        span=cause['span'];exact_keys(span,('start','end'),'native source span')
        require(counter(span['start'],'source span start')<=counter(span['end'],'source span end')<=len(source),'native source span bounds')
        try:source[:span['start']].decode('utf-8');source[:span['end']].decode('utf-8')
        except UnicodeError as error:raise AdmissionError('native source span is not a UTF-8 boundary') from error
    # Iterative encoding counts the original 64KiB payload cap without a second
    # retained serialized snapshot; individual strings were bounded above.
    require(sum(len(chunk.encode()) for chunk in json.JSONEncoder(separators=(',',':')).iterencode(snap))<=64<<10,'original native source fact payload cap')
    first=snap['causes'][0];legacy=response['diagnostic'];exact_keys(legacy,('code','message'),'legacy projection of typed source fact')
    typed_equal(legacy['code'],'mn.unsupported.frontend','native legacy projection code')
    typed_equal(legacy['message'],first['stage'].capitalize()+'/'+first['code']+': '+first['message'],'native primary projection linkage')
    if data_family:
        stage,code,_=ORACLES[contract['case']]
        expected_span=diagnostic_span(source,contract['case'])
    elif legacy_base:
        stage,code=DIAGNOSTICS[contract['case']][:2];expected_span=legacy_span(source,contract['case'])
        require(type(expected_span) is dict,'original BASE diagnostic span oracle required')
    else:
        stage='verify' if contract['family']=='source-query' else 'bind';code=contract['oracle_value']
        needle=b'!1' if contract['case']=='neg-not-int' else b'true'
        require(source.count(needle)==1,'original diagnostic operand is unique')
        start=source.index(needle)+(1 if needle==b'!1' else 0);end=start+(1 if needle==b'!1' else len(needle))
        expected_span=dict(start=start,end=end)
    matches=first['stage']==stage and first['code']==code and (expected_span is None or first['span']==expected_span)
    verdict='UNOBSERVED' if matches and data_family and expected_span is None else 'PASS' if matches else 'FAIL'
    return dict(kind='diagnostic',stage=first['stage'],code=first['code'],message=first['message'],span=first['span'],source_sha256=digest), verdict


def admit_source(inputs,evidence,binding,workers):
    from ace_m1_first_slice import _inventory_for_suite,_compact_admission,_repo_relative,_wires,retain_parent_status
    exact_keys(binding,('dependencies_before','dependencies_after','families'),'query/instances source binding')
    exact_keys(binding['families'],CONTRACTS,'original query/instance families');contracts=source_contracts(inputs);snapshots=[]
    for key,phase in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        snap,_=evidence.read_json(binding[key]);exact_keys(snap,('schema','snapshot','source_pins'),'source dependency snapshot')
        typed_equal(snap['schema'],'ace.m1.query-instances-dependencies/1','query/instances dependency schema');typed_equal(snap['snapshot'],phase,'source snapshot phase')
        exact_keys(snap['source_pins'],DEPENDENCIES,'original source dependencies')
        for path,digest in snap['source_pins'].items():inputs.sha256(path,digest)
        snapshots.append(snap['source_pins'])
    typed_equal(*snapshots,'stable actual source dependencies')
    for key,path in [('av',DEPENDENCIES[0]),('native',DEPENDENCIES[1])]:typed_equal(snapshots[0][path],workers[key],'current actual source consumer')
    observations={};seen=set();parents={};emit_only=[]
    for family,spec in CONTRACTS.items():
        original=binding['families'][family];exact_keys(original,('inventory','copy'),'source family binding')
        inputs.sha256(spec['path'],spec['sha256']);frozen,_=inputs.read_json(spec['path'])
        inventory=_inventory_for_suite(inputs,evidence,original['inventory'],spec['suite'],frozen)
        typed_equal(len(inventory['checks']),spec['checks'],'original complete fixed suite')
        copy=CaseCopy(EvidenceScope(evidence.path(original['copy']+'/artifact-copy.json').parent,inputs.budget));admitted=admit_case_bindings(copy,inventory);non_pass=[]
        for check,parent in admitted['checks']:
            if parent['status']!='PASS':non_pass.append(dict(case=check['case'],check=check['check'],status=parent['status']))
            if check['check'] in ('a2r-emit','a2c-emit'):
                require(family=='source-query' and check['case']=='ace.m1.sq934.leaf42','only original leaf42 golden checks are emit-only')
                emit_only.append(dict(case=check['case'],check=check['check'],status=parent['status'],semantic_authority=False));continue
            require(check['case'].startswith(PREFIX[family]) and check['check'] in ('av','ac-native'),'foreign source semantic check')
            case=check['case'].removeprefix(PREFIX[family]);require((family,case) in contracts,'original source semantic case');row=contracts[(family,case)]
            av=check['check']=='av';slot='q4-source/'+family+'/'+case+'/'+('AV' if av else 'AC-native')
            require(slot not in seen,'duplicate source target');seen.add(slot)
            typed_equal([_repo_relative(inputs,name) for name in check['input_paths']],[row['source_file']],'original source path')
            typed_equal(check['source_files'][0]['sha256'],row['source_sha256'],'original source bytes')
            typed_equal(check['recipe'],'ac-av/core-run-v1' if av else 'ac/mir-native-scalar-v1','original source recipe')
            typed_equal(check['expected_exit'],int(row['oracle_value']) if row['oracle_kind']=='value' else None,'independent source value')
            completion=dict(kind='success') if row['oracle_kind']=='value' else dict(kind='error',stage='parse' if av else 'typecheck' if family=='source-query' else 'resolve',code=row['oracle_value'],message=None)
            typed_equal(check['completion'],completion,'original source completion')
            options=check['probe_bindings']['av' if av else 'native'];typed_equal(options['entry'],'d_main','original source entry');typed_equal(options['caps'],json.loads(row['caps']),'original source capabilities')
            if not av and row['oracle_kind']=='diagnostic':typed_equal(options['diagnostics'],'source-v1','original native typed diagnostic mode')
            typed_equal(check['declared_probe_steps'],[{'op':'run'}],'one original source operation');require(check['state']=='RUN' and check['adapter']=='probe' and check['action']=='run','original selected source operation')
            def read_one(_):
                wire=_wires(copy,check,'avworker/1' if av else 'mnworker/1',check['recipe'],workers['av' if av else 'native']);require(len(wire)==1,'one original source exchange');wire=wire[0]
                owner=PurePosixPath(wire['start_path']).parent.parent;require(len(owner.parts)==3 and owner.parts[1:]==(check['case'],check['check']),'original source exchange owner')
                request,_=copy.scope.decode_json(wire['request']);typed_equal(PureWindowsPath(request['store_root']),copy.original.joinpath(*owner.parts,'store' if av else 'mn-store'),'source store owner')
                if not av:typed_equal(PureWindowsPath(request['workdir']),copy.original.joinpath(*owner.parts,'mn-work'),'source work owner')
                return retain_parent_status(scalar_q4_observation(copy.scope,wire,check,row,query_family=True),parent['status'])
            observations[slot]=_compact_admission(inputs,read_one,32<<10)
        parents[family]=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass)
    typed_equal(seen,{'q4-source/'+family+'/'+case+'/'+target for family,case in contracts for target in ('AV','AC-native')},'all twelve original source two-target observations')
    typed_equal(len(emit_only),2,'original additional golden-only checks')
    return dict(schema='ace.m1.query-instances-source-observations/1',required=24,observations=observations,source_case_reports=parents,
                golden_emission_only=emit_only,semantic_evidence_complete=all(row['judgement']=='PASS' for row in observations.values()),
                query_fact_authority_and_restore='UNOBSERVED',instance_materialization_and_restore='UNOBSERVED',can_claim_expanded_m1=False)


REFERENCE = 'crates/auto-lang/test/cases/ace-m1-expanded/query-reference-contracts.tsv'
REFERENCE_SHA = '469274867db24637eaa1b6b8a1eb086ec0514b59fdb06d0335ed9b5e9c90ba6b'
REFERENCE_FIELDS = 'family case profile caps source_file source_sha256 oracle_kind oracle_value marks rust_reference rust_reference_sha256 c_reference c_reference_sha256 rust_runs rust_release_runs rust_release_flags c_runs execution'.split()
RELEASE_SOURCE = 'experimental/ace/crates/a2x-worker/tests/cases/choose_instances_935.rs'
RELEASE_SOURCE_SHA = '9b9d28847d96f87ccf82eb81f6b4d396a7c9f2182670bbc539fd9d648f37b77b'


def reference_contracts(inputs):
    inputs.sha256(REFERENCE,REFERENCE_SHA);inputs.sha256(RELEASE_SOURCE,RELEASE_SOURCE_SHA);sources=source_contracts(inputs);seen={}
    for row in read_tsv(inputs,REFERENCE,REFERENCE_FIELDS):
        key=(row['family'],row['case']);require(key in sources and sources[key]['oracle_kind']=='value' and key not in seen,'original positive query reference identity')
        for field in ('profile','caps','source_file','source_sha256','oracle_kind','oracle_value'):
            typed_equal(row[field],sources[key][field],'independent query reference '+field)
        typed_equal(json.loads(row['marks']),sources[key]['marks'].split(',') if sources[key]['marks'] else [],'independent query reference marks')
        for target,ext in [('rust','rs'),('c','c')]:
            typed_equal(row[target+'_reference'],'crates/auto-lang/test/cases/ace-m1-expanded/query-reference/'+key[0]+'--'+key[1]+'.'+ext,'independent query reference path')
            inputs.sha256(row[target+'_reference'],row[target+'_reference_sha256'])
        release=key[0]=='source-instances' and key[1] in ('choose42','choose0')
        typed_equal((row['rust_runs'],row['rust_release_runs'],json.loads(row['rust_release_flags']),row['c_runs'],row['execution']),('1','1' if release else '0',['-O'] if release else [],'1','UNOBSERVED'),'original query/instance configurations')
        seen[key]=dict(row,trace=sources[key]['trace'])
    typed_equal(set(seen),{key for key,row in sources.items() if row['oracle_kind']=='value'},'all nine positive query references')
    return seen


def admit_certifications(inputs,evidence,binding_path,release_binding=None):
    from ace_m1_first_slice import _compact_admission,_repo_relative
    from ace_m1_cert_evidence import query_certification
    from ace_m1_daily_evidence import certified_daily_emission
    from ace_m1_scalar_cert_evidence import required_variants
    contracts=reference_contracts(inputs);document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'query certification binding');producer=document['producer']
    exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'query certification producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','original query certification producer')
    typed_equal(producer['cwd'],str(inputs.root),'query certification worktree');require(type(document['rows']) is list,'query certifications')
    release=None
    if release_binding is not None:
        from ace_m1_scalar_release_evidence import admit_scalar_release
        release_contracts={case:row for (family,case),row in contracts.items() if row['rust_release_runs']=='1'};pins={}
        for row in document['rows']:
            exact_keys(row,('family','case','prefix','pin'),'original query release certification row')
            key=(row['family'],row['case']);require(key in contracts and type(row['pin']) is dict,'query release certified source')
            require(row['pin'].get('target') in ('rust','c'),'query release certified target')
            if key[0]=='source-instances' and key[1] in release_contracts and row['pin']['target']=='rust':
                require(key[1] not in pins,'duplicate query release certification');pins[key[1]]=row['pin']
        typed_equal(set(pins),set(release_contracts),'original query release default certifications')
        release=admit_scalar_release(inputs,evidence,release_binding,release_contracts,pins,query_family=True)
    seen=set();observations={}
    for row in document['rows']:
        exact_keys(row,('family','case','prefix','pin'),'query certification row');key=(row['family'],row['case']);pin=row['pin']
        require(key in contracts and type(pin) is dict,'original query certification source')
        exact_keys(pin,('target','identity_request','request','toolchain','source_sha256','reference_sha256','actual_files','expected_stdout_hex','expected_stderr_hex'),'query certification pins')
        target=pin['target'];require(target in ('rust','c') and (*key,target) not in seen,'duplicate/foreign query certification target');seen.add((*key,target))
        typed_equal(pin['identity_request'],{'op':'identity','certification_target':target},'query original compiler target')
        typed_equal(pin['request']['case_id'],'941/query/'+key[0]+'/'+key[1],'original query case identity')
        path=PurePosixPath(row['prefix']);home=PurePosixPath(binding_path).parent
        require(path.parent.parent==home/(key[0]+'--'+key[1]) and path.name==target and path.parent.name.startswith('attempt-') and path.parent.name[8:].isdigit() and int(path.parent.name[8:])>0,'query owned attempt')
        value=_compact_admission(inputs,lambda _:certified_daily_emission(evidence,inputs,row['prefix'],pin,producer,
            query_certification(evidence,inputs,row['prefix'],pin,producer,contracts[key])),128<<10)
        value=required_variants(value,contracts[key],target,release['observations'].get(key[1]) if release is not None and key[0]=='source-instances' else None)
        observations['q4-source/'+key[0]+'/'+key[1]+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=value
    typed_equal(seen,{(family,case,target) for family,case in contracts for target in ('rust','c')},'all eighteen positive query certifications')
    result=dict(schema='ace.m1.query-certification-observations/1',required=18,observations=observations,
                contract_sha256=REFERENCE_SHA,bindings_sha256=digest,
                default_certification_complete=all(row['default_certification_judgement']=='PASS' for row in observations.values()),
                required_extra_rust_release_runs=2,rust_release_evidence='UNOBSERVED' if release is None else 'PASS' if release['complete'] else 'FAIL',
                certification_complete=all(row['judgement']=='PASS' for row in observations.values()),
                query_and_instance_mechanisms='UNOBSERVED',can_claim_expanded_m1=False)

    if release is not None:result['rust_release_execution']=release
    return result


def untyped_frontdoor_observation(response):
    # The original error/detail API has no typed stage, source hash or operation
    # identity. A rendered diagnostic alone never closes the semantic target.
    exact_keys(response,('status','detail'),'original A2X frontdoor refusal')
    typed_equal(response['status'],'error','original A2X refusal status')
    require(type(response['detail']) is str and 0<len(response['detail'].encode())<=1<<20,'bounded original refusal detail')
    return dict(observed=dict(kind='untyped-frontdoor-refusal',detail=response['detail']),judgement='UNOBSERVED',
                reason='A2X error/detail lacks typed diagnostic/source/operation identity; rendered message is not semantic authority')


def typed_frontdoor_observation(response,request,contract,source,stage=None,code=None):
    """PLAN-1000 F-1000-02 (GAP-941-A2X-DIAG-01 closure): the a2x worker's
    refusal now carries the TYPED diagnostic channel — structured frontend
    facts (stage/code/message/span per cause, source identity, primary
    index). The FIRST cause's stage/code is the semantic authority; the
    rendered `detail` string is only the legacy projection and must
    re-derive from the causes exactly (no free-text parsing either way).
    `stage`/`code` default to the query families' oracle mapping; callers
    with their own diagnostic authority (the original-data families use
    the same ORACLES table as their execution legs) pass it explicitly."""
    exact_keys(response,('status','detail','diagnostics'),'typed A2X frontdoor refusal')
    typed_equal(response['status'],'error','typed A2X refusal status')
    snap=response['diagnostics']
    exact_keys(snap,('schema','mode','producer','profile','path','source_sha256','source_len','causes','primary'),'typed A2X source fact')
    for key,want in [('schema','a2x.source-diagnostic/1'),('mode','typed-negative/1'),('producer','a2x-worker'),('profile',request['profile'])]:
        typed_equal(snap[key],want,'typed A2X diagnostic '+key)
    typed_equal(snap['path'],request['source_path'],'typed A2X source path identity')
    typed_equal(snap['source_sha256'],contract['source_sha256'],'typed A2X source bytes identity')
    counter(snap['source_len'],'typed A2X source length')
    typed_equal(snap['source_len'],len(source),'typed A2X source length equality')
    require(type(snap['causes']) is list and 1<=len(snap['causes'])<=64,'typed A2X cause cap')
    typed_equal(snap['primary'],0,'compiler first diagnostic is primary')
    for cause in snap['causes']:
        exact_keys(cause,('stage','code','message','span'),'typed A2X cause')
        require(cause['stage'] in ('text','bind','verify','capability','backend','link','run'),'typed A2X diagnostic stage')
        for key in ('code','message'):
            require(type(cause[key]) is str and 0<len(cause[key].encode())<=4096,'typed A2X diagnostic text bound')
        span=cause['span'];exact_keys(span,('start','end'),'typed A2X cause span')
        require(counter(span['start'],'typed A2X span start')<=counter(span['end'],'typed A2X span end')<=len(source),'typed A2X span bounds')
        try:source[:span['start']].decode('utf-8');source[:span['end']].decode('utf-8')
        except UnicodeError as error:raise AdmissionError('typed A2X span is not a UTF-8 boundary') from error
    # the legacy projection must re-derive from the typed causes verbatim
    prefix='a2x.compile: ' if request['op'] in ('daily-emit-c','certify-c') else 'frontend diagnostics: '
    rendered=prefix+'; '.join(f"{c['code']}: {c['message']}" for c in snap['causes'])
    typed_equal(response['detail'],rendered,'typed A2X legacy projection linkage')
    first=snap['causes'][0]
    if stage is None:stage='verify' if contract['family']=='source-query' else 'bind'
    if code is None:code=contract['oracle_value']
    matches=first['stage']==stage and first['code']==code
    return dict(observed=dict(kind='typed-frontdoor-refusal',stage=first['stage'],code=first['code'],
                              source_sha256=contract['source_sha256'],primary=0,causes=len(snap['causes'])),
                judgement='PASS' if matches else 'FAIL',
                **({} if matches else {'reason':'typed A2X first cause does not match the independent oracle'}))


def admit_frontdoor_refusals(inputs,evidence,binding_path):
    from ace_m1_first_slice import _compact_admission,_repo_relative
    from ace_m1_cert_evidence import admit_command,_relative
    sources=source_contracts(inputs);document,digest=evidence.read_json(binding_path)
    exact_keys(document,('producer','rows'),'original A2X negative source bindings');producer=document['producer']
    exact_keys(producer,('producer_commit','worker_path','worker_sha256','cwd'),'original A2X negative producer')
    typed_equal(_repo_relative(inputs,producer['worker_path']),'experimental/ace/target/release/a2x-worker.exe','original negative producer')
    typed_equal(producer['cwd'],str(inputs.root),'negative source producer worktree');inputs.sha256('experimental/ace/target/release/a2x-worker.exe',producer['worker_sha256'])
    require(type(document['rows']) is list,'negative source rows');seen=set();observations={}
    for row in document['rows']:
        exact_keys(row,('family','case','target','prefix','request','source_sha256'),'original negative source row')
        key=(row['family'],row['case']);target=row['target'];require(key in sources and sources[key]['oracle_kind']=='diagnostic' and target in ('rust','c') and (*key,target) not in seen,'duplicate/foreign negative source');seen.add((*key,target));contract=sources[key]
        prefix=row['prefix'];typed_equal(PurePosixPath(prefix),PurePosixPath(binding_path).parent/(key[0]+'--'+key[1])/target,'negative source owner')
        request=row['request'];exact_keys(request,('op','source_path','profile','entry','caps','out_dir','nonce'),'original negative emission request')
        for field,want in [('op','daily-emit' if target=='rust' else 'daily-emit-c'),('profile','scalar'),('entry','main'),('caps',json.loads(contract['caps']))]:typed_equal(request[field],want,'original negative request '+field)
        typed_equal(_relative(inputs,request['source_path']),contract['source_file'],'original negative source path')
        typed_equal(row['source_sha256'],contract['source_sha256'],'independent negative source hash');inputs.sha256(contract['source_file'],row['source_sha256'])
        typed_equal(request['out_dir'],str(evidence.root.joinpath(*PurePosixPath(prefix).parts)),'negative source output owner')
        require(type(request['nonce']) is str and len(request['nonce'])==48 and all(c in '0123456789abcdef' for c in request['nonce']),'original negative nonce')
        require(not os.path.lexists(evidence.path(prefix+'.command.json').parent/target),'refused source produced an output entry')
        def read_one(_):
            response,receipts=admit_command(evidence,prefix+'.command.json',request,producer,expected_exit=2)
            record,_=evidence.read_json(prefix+'.command.json');typed_equal(record['meter']['job_processes'],2,'negative emit launcher and worker only')
            # PLAN-1000 F-1000-02: the typed negative channel (when the
            # worker reports it) supplies the semantic authority; the plain
            # two-key refusal stays the honest UNOBSERVED observation.
            if type(response) is dict and 'diagnostics' in response:
                source=inputs.read_bytes(contract['source_file'])
                observation=typed_frontdoor_observation(response,request,contract,source)
            else:
                observation=untyped_frontdoor_observation(response)
            return dict(observation,source_sha256=contract['source_sha256'],**receipts)
        observations['q4-source/'+key[0]+'/'+key[1]+'/'+('ACE-A2R-HIR' if target=='rust' else 'ACE-A2C-HIR')]=_compact_admission(inputs,read_one,32<<10)
    typed_equal(seen,{(family,case,target) for (family,case),row in sources.items() if row['oracle_kind']=='diagnostic' for target in ('rust','c')},'all six original negative emit entries')
    typed_complete=all(v['judgement']=='PASS' and v.get('observed',{}).get('kind')=='typed-frontdoor-refusal'
                       for v in observations.values())
    return dict(schema='ace.m1.query-negative-frontdoor-observations/1',required=6,observations=observations,bindings_sha256=digest,
                gap='GAP-941-A2X-DIAG-01',typed_diagnostic_evidence_complete=typed_complete,can_claim_expanded_m1=False)
