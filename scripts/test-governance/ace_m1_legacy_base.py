"""Strict original BASE source completions and VM values; scenarios stay separate."""
import hashlib
import re
from ace_m1_arch_acceptance import EvidenceScope,exact_keys
from ace_m1_case_evidence import CaseCopy,admit_case_bindings,require
from ace_m1_legacy import CORPUS,frozen_ledger,typed_equal
from ace_m1_legacy_execution import execution_summary
from ace_m1_scalar_evidence import scalar_observation

CONTRACT='crates/auto-lang/test/cases/ace-m1-expanded/legacy-base-contract.json'
CONTRACT_SHA = '6f052ba15a1765625177df159ac5387b7c0efa8871a1406c2bdb961e2d41ccd7'
SUITE='crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad'
VALUE_CASES=('B01','B02','B03','B04','B05','B06','B07','B09','B10','B12')
TRAP_CASES=('B13','B14','B15','B20')
TRAP_NAMES={'overflow':'Overflow','div-by-zero':'DivByZero','rem-overflow':'RemOverflow'}
DIAGNOSTICS={'B16':('bind','source.undefined',b'missing',0,7),
             'B17':('verify','source.type-mismatch',b'true',0,4),
             'B18':('verify','verify.immutable-write',b'x = 2',0,5),
             'B19':('verify','verify.no-return-path',b'fn main',3,4)}
OLD_DIAGNOSTICS={'B16':'resolve','B17':'type','B18':'assign-immutable','B19':'missing-return'}
VM_VALUE_CASES=VALUE_CASES+('K01','K03','K06','R02','R05','R06','R08','T05','V01','V02','V04','V05','V06','V07')
# Independent original binding taxonomy -> frozen legacy completion channel.
# K04 was approved as a parse rejection; it proves no borrow-checker rule.
VM_FAILURES={'B14':('trap','div-by-zero','run','DivisionByZero'),
             'B20':('trap','div-by-zero','run','DivisionByZero'),
             'B16':('diagnostic','resolve','type','auto_link_E0401'),
             'T07':('diagnostic','invalid-scalar','parse','auto_lexer_E0001'),
             # PLAN-998 T-06 reception (941 round 121): the fn-type grammar
             # admission supersedes the frozen parse channel — the borrow-
             # escaping closure now rejects at the designed typed channels.
             'K04':('diagnostic','borrow-escape','run','RuntimeError')}
DEPENDENCIES=('experimental/ace/target/release/av-worker.exe',
              'experimental/ace/target/release/mn-worker.exe',
              'target/debug/auto-case.exe','target/debug/test-case-pilot.exe')


def value_contracts(inputs):
    required=sorted(':'.join((r['ns'],r['id'],r['path'])) for r in frozen_ledger(inputs) if r['required'])
    rows=[line.split('\t') for line in inputs.read_bytes(CORPUS+'binding.tsv').decode().splitlines()
          if line and not line.startswith('#')]
    values={r[0]:r for r in rows if r[1]=='corpus' and r[0] in VALUE_CASES}
    typed_equal(set(values),set(VALUE_CASES),'original BASE value denominator')
    manifest,_=inputs.read_json(CORPUS+'source-manifest.json');contracts={}
    for name,row in values.items():
        typed_equal(row[3],'value','original BASE value family')
        require(all('corpus:'+name+':'+target in required for target in ('ac','av')),'original BASE required targets')
        path=CORPUS+'src/'+name+'.at';digest=manifest['corpus_src']['files'][name+'.at'];inputs.sha256(path,digest)
        contracts[name]=dict(source_file=path,source_sha256=digest,value=int(row[4]))
    return required,contracts


def value_observation(inputs,scope,wire,check,parent,name,target,contract):
    from ace_m1_first_slice import _repo_relative,retain_parent_status
    require(name in VALUE_CASES and target in ('ac','av'),'finite original BASE value cell')
    typed_equal((check['case'],check['check']),('ace.m1.'+name.lower(),'ac-native' if target=='ac' else 'av'),'original BASE case/check')
    typed_equal(check['expected_exit'],contract['value'],'approved original BASE value')
    typed_equal(check['completion'],{'kind':'success'},'original BASE completion')
    require(check['state']=='RUN' and check['adapter']=='probe' and check['action']=='run','selected original BASE execution')
    typed_equal(check['declared_probe_steps'],[{'op':'run'}],'original BASE one run')
    typed_equal([_repo_relative(inputs,p) for p in check['input_paths']],[contract['source_file']],'original BASE input path')
    require(len(check['source_files'])==1,'original BASE single source')
    typed_equal(check['source_files'][0]['sha256'],contract['source_sha256'],'original BASE source bytes')
    recipe='ac/mir-native-scalar-v1' if target=='ac' else 'ac-av/core-run-v1'
    typed_equal(check['recipe'],recipe,'original BASE target recipe')
    options=check['probe_bindings']['native' if target=='ac' else 'av']
    typed_equal((options['entry'],options['caps']),('d_main',[]),'original BASE entry/capabilities')
    return retain_parent_status(scalar_observation(scope,wire,check),parent)


def diagnostic_span(source,name):
    require(name in DIAGNOSTICS,'finite original BASE diagnostic anchor')
    _,_,needle,offset,length=DIAGNOSTICS[name]
    require(source.count(needle)==1,'unique original BASE diagnostic anchor')
    start=source.index(needle)+offset
    return dict(start=start,end=start+length)


def other_contracts(inputs):
    required={':'.join((r['ns'],r['id'],r['path'])) for r in frozen_ledger(inputs) if r['required']}
    rows=[line.split('\t') for line in inputs.read_bytes(CORPUS+'binding.tsv').decode().splitlines() if line and not line.startswith('#')]
    manifest,_=inputs.read_json(CORPUS+'source-manifest.json');out={}
    for row in rows:
        name=row[0]
        if row[1]!='corpus' or name not in TRAP_CASES+tuple(DIAGNOSTICS)+VM_VALUE_CASES:continue
        kind,value=row[3:5]
        if name in TRAP_CASES:
            typed_equal(kind,'trap','original BASE trap family');require(value in TRAP_NAMES,'original BASE trap taxonomy');value=TRAP_NAMES[value]
        elif name in DIAGNOSTICS:
            typed_equal((kind,value),('diagnostic',OLD_DIAGNOSTICS[name]),'original BASE diagnostic family');value=DIAGNOSTICS[name][1]
        else:typed_equal(kind,'value','original VM value family')
        path=CORPUS+'src/'+name+'.at';digest=manifest['corpus_src']['files'][name+'.at'];inputs.sha256(path,digest)
        out[name]=dict(family='legacy-base',case=name,profile='scalar',caps='[]',trace='false',marks='',source_file=path,source_sha256=digest,oracle_kind=kind,oracle_value=value)
    typed_equal(set(out),set(TRAP_CASES+tuple(DIAGNOSTICS)+VM_VALUE_CASES),'original BASE non-value and VM source set')
    for name in TRAP_CASES+tuple(DIAGNOSTICS):require(all('corpus:'+name+':'+t in required for t in ('ac','av')),'original BASE non-value required')
    for name in VM_VALUE_CASES:require('corpus:'+name+':avm' in required,'original VM value required')
    return out


def nonvalue_observation(inputs,scope,wire,check,parent,name,target,contract):
    from ace_m1_first_slice import _repo_relative,retain_parent_status
    from ace_m1_scalar_q4_evidence import scalar_q4_observation,TRAPS
    require(name in TRAP_CASES+tuple(DIAGNOSTICS) and target in ('ac','av'),'finite BASE non-value cell')
    typed_equal((check['case'],check['check']),('ace.m1.'+name.lower(),'ac-native' if target=='ac' else 'av'),'BASE negative case/check')
    require(check['adapter']=='probe' and check['action']=='run' and check['state']=='RUN','selected BASE non-value run')
    typed_equal(check['declared_probe_steps'],[{'op':'run'}],'BASE non-value one run')
    typed_equal([_repo_relative(inputs,p) for p in check['input_paths']],[contract['source_file']],'BASE negative source path')
    require(len(check['source_files'])==1,'BASE non-value single source');typed_equal(check['source_files'][0]['sha256'],contract['source_sha256'],'BASE negative source bytes')
    typed_equal(check['recipe'],'ac/mir-native-scalar-v1' if target=='ac' else 'ac-av/core-run-v1','BASE negative target')
    options=check['probe_bindings']['native' if target=='ac' else 'av'];typed_equal((options['entry'],options['caps']),('d_main',[]),'BASE negative entry/caps')
    trap=name in TRAP_CASES
    if target=='ac':typed_equal(options['diagnostics'],'source-v1' if name in ('B17','B18','B19') else None,'original BASE diagnostic mode')
    completion={'kind':'success'} if trap and target=='ac' else dict(kind='error',stage='run' if trap else 'typecheck' if target=='ac' and name!='B16' else 'parse',code=TRAPS[contract['oracle_value']] if trap else 'mn.unsupported.frontend' if target=='ac' and name=='B16' else contract['oracle_value'],message=None)
    typed_equal(check['completion'],completion,'independent original BASE completion')
    typed_equal(check['expected_exit'],70 if trap and target=='ac' else None,'BASE non-value declared exit')
    return retain_parent_status(scalar_q4_observation(scope,wire,check,contract,legacy_base=True),parent)


def vm_result_observation(result,stdout,expected):
    require(type(result) is bytes and type(stdout) is bytes and 0<len(result)<=11,'bounded legacy VM result text')
    try:text=result.decode('ascii')
    except UnicodeError as error:
        from ace_m1_arch_acceptance import AdmissionError
        raise AdmissionError('legacy VM result is not canonical integer text') from error
    require(re.fullmatch(r'0|-?[1-9][0-9]*',text) is not None,'canonical legacy VM integer text')
    value=int(text);require(-(1<<31)<=value<(1<<31) and type(expected) is int,'legacy VM numeric value oracle')
    return dict(schema='ace.m1.legacy-vm-value/1',expected=expected,observed=dict(kind='numeric-result-text',text=text,value=value),
                result_sha256=hashlib.sha256(result).hexdigest(),stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                type_or_bits_authority=False,stdout_oracle=False,judgement='PASS' if value==expected else 'FAIL')


def vm_value_observation(inputs,copy,check,row,name,contract):
    from ace_m1_first_slice import _repo_relative,retain_parent_status
    require(name in VM_VALUE_CASES,'finite original VM numeric cell')
    typed_equal((check['case'],check['check'],check['adapter'],check['action'],check['state']),('ace.m1.'+name.lower(),'autovm','avm','run','RUN'),'original VM numeric entry')
    typed_equal(check['completion'],{'kind':'success'},'original VM numeric completion')
    typed_equal([_repo_relative(inputs,p) for p in check['input_paths']],[contract['source_file']],'original VM source path')
    typed_equal(check['source_files'][0]['sha256'],contract['source_sha256'],'original VM source identity')
    products={a['path']:a for a in row.get('artifacts',[])}
    require(set(products)=={'result.txt','stdout.txt'} and len(row['artifacts'])==2,'real legacy VM result/stdout products required')
    raw={}
    for kind,ref in products.items():
        rel=copy.path_from_original(ref['stored']);require('/'+check['case']+'/autovm/' in rel and rel.endswith('/'+kind),'original VM product owner')
        typed_equal(ref['sha256'],copy.files[rel]['sha256'],'VM product hash');typed_equal(ref['bytes'],copy.files[rel]['bytes'],'VM product size')
        raw[kind]=copy.bytes(rel)
    result=vm_result_observation(raw['result.txt'],raw['stdout.txt'],int(contract['oracle_value']))
    result['source_sha256']=contract['source_sha256']
    return retain_parent_status(result,row['status'])


def vm_failure_contracts(inputs):
    ledger={(r['ns'],r['id'],r['path']):r for r in frozen_ledger(inputs) if r['required']}
    rows=[line.split('\t') for line in inputs.read_bytes(CORPUS+'binding.tsv').decode().splitlines() if line and not line.startswith('#')]
    manifest,_=inputs.read_json(CORPUS+'source-manifest.json');out={}
    for row in rows:
        name=row[0]
        if row[1]!='corpus' or name not in VM_FAILURES:continue
        kind,value,stage,code=VM_FAILURES[name]
        typed_equal(tuple(row[3:5]),(kind,value),'original VM failure binding taxonomy')
        require(('corpus',name,'avm') in ledger,'original required VM failure cell')
        typed_equal(ledger[('corpus',name,'avm')]['token'],'ok-parse-reject' if name=='K04' else 'ok','approved legacy rejection scope')
        path=CORPUS+'src/'+name+'.at';digest=manifest['corpus_src']['files'][name+'.at'];inputs.sha256(path,digest)
        out[name]=dict(source_file=path,source_sha256=digest,stage=stage,code=code,
                       scope='typed-language-failure')
    typed_equal(set(out),set(VM_FAILURES),'finite original VM failure set')
    return out


def vm_failure_observation(inputs,copy,check,row,name,contract):
    from ace_m1_first_slice import _repo_relative,retain_parent_status
    require(name in VM_FAILURES,'finite original VM failure cell')
    typed_equal((check['case'],check['check'],check['adapter'],check['action'],check['state']),
                ('ace.m1.'+name.lower(),'autovm','avm','run','RUN'),'original VM failure entry')
    typed_equal(check['completion'],dict(kind='error',stage=contract['stage'],code=contract['code'],message=None),'independent original VM failure completion')
    typed_equal([_repo_relative(inputs,p) for p in check['input_paths']],[contract['source_file']],'original VM failure source path')
    typed_equal(check['source_files'],[dict(virtual_path='main.at',sha256=contract['source_sha256'])],'original VM failure source identity')
    products=row.get('artifacts',[])
    require(type(products) is list and len(products)==1,'one real typed VM failure product required')
    ref=products[0];exact_keys(ref,('path','stored','sha256','bytes'),'typed VM failure product')
    typed_equal(ref['path'],'failure.json','typed VM failure logical product')
    rel=copy.path_from_original(ref['stored'])
    typed_equal(rel.split('/')[-3:],[check['case'],'autovm','failure.json'],'typed VM failure product owner')
    typed_equal(ref['sha256'],copy.files[rel]['sha256'],'typed VM failure hash')
    typed_equal(ref['bytes'],copy.files[rel]['bytes'],'typed VM failure size')
    require(type(ref['bytes']) is int and 0<ref['bytes']<=16<<10,'bounded typed VM failure product')
    actual,digest=copy.json(rel)
    exact_keys(actual,('schema','adapter','action','outcome','source_sha256','entry','entry_sha256','stage','code'),'typed VM failure facts')
    for key,value in dict(schema='autolang.avm-failure/1',adapter='avm',action='run',outcome='language-error',
                          source_sha256=check['source_sha256'],entry='main.at',entry_sha256=contract['source_sha256']).items():
        typed_equal(actual[key],value,'typed VM failure '+key)
    require(type(actual['stage']) is str and actual['stage'] in ('parse','type','run'),'typed VM failure stage')
    code=actual['code']
    require(code is None or (type(code) is str and re.fullmatch(r'[A-Za-z][A-Za-z0-9_.:-]{0,127}',code) is not None),'typed VM failure code')
    result=dict(schema='ace.m1.legacy-vm-failure/1',source_sha256=contract['source_sha256'],artifact_sha256=digest,
                expected=dict(stage=contract['stage'],code=contract['code']),observed=dict(stage=actual['stage'],code=code),
                authority=contract['scope'],semantic_borrow_rule_authority=False,
                judgement='FAIL' if actual['stage']!=contract['stage'] else 'UNOBSERVED' if code is None else 'PASS' if code==contract['code'] else 'FAIL')
    if code is None:result['gap']='typed legacy VM failure has no stable code'
    return retain_parent_status(result,row['status'])


def admit(inputs,evidence,binding,workers,previous=None):
    from ace_m1_first_slice import _inventory_for_suite,_compact_admission,_repo_relative,_wires
    exact_keys(binding,('inventory','copy','tool_capture','dependencies_before','dependencies_after'),'original BASE bindings')
    inputs.sha256(CONTRACT,CONTRACT_SHA);frozen,_=inputs.read_json(CONTRACT)
    snapshots=[]
    for field,label in [('dependencies_before','before-execution'),('dependencies_after','after-execution')]:
        d,_=evidence.read_json(binding[field]);exact_keys(d,('schema','snapshot','source_pins'),'BASE dependency snapshot')
        typed_equal(d['schema'],'ace.m1.legacy-base-dependencies/1','BASE dependency schema');typed_equal(d['snapshot'],label,'BASE dependency phase')
        exact_keys(d['source_pins'],DEPENDENCIES,'BASE dependency set')
        for name,digest in d['source_pins'].items():inputs.sha256(name,digest)
        snapshots.append(d['source_pins'])
    typed_equal(*snapshots,'stable BASE dependencies')
    for key,name in [('av',DEPENDENCIES[0]),('native',DEPENDENCIES[1])]:typed_equal(workers[key],snapshots[0][name],'original BASE worker')
    exact_keys(binding['tool_capture'],('path','sha256'),'BASE formal capture')
    evidence.sha256(binding['tool_capture']['path'],binding['tool_capture']['sha256']);parent,_=evidence.read_json(binding['tool_capture']['path'])
    typed_equal(parent.get('groups'),['case-ace-m1-base'],'BASE fixed group');typed_equal(parent.get('executed'),97,'original BASE full checks')
    prepared={}
    require(type(parent.get('tools')) is list,'BASE preparation metadata')
    for tool in parent['tools']:
        if 'path' in tool and tool.get('tool') in ('av','mn','case-parent'):
            path=_repo_relative(inputs,tool['path']);require(path in DEPENDENCIES,'foreign BASE prepared path')
            typed_equal(tool['sha256'],snapshots[0][path],'BASE prepared binary');prepared[path]=tool['sha256']
    typed_equal(set(prepared),set(DEPENDENCIES),'all original BASE prepared tools')
    inv=_inventory_for_suite(inputs,evidence,binding['inventory'],SUITE,frozen)
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inv);required,contracts=value_contracts(inputs);others=other_contracts(inputs);vm_failures=vm_failure_contracts(inputs)
    selected=[(c,row) for c,row in admitted['checks'] if
              (c['case'] in {'ace.m1.'+n.lower() for n in VALUE_CASES+TRAP_CASES+tuple(DIAGNOSTICS)} and c['check'] in ('ac-native','av')) or
              (c['case'] in {'ace.m1.'+n.lower() for n in VM_VALUE_CASES+tuple(VM_FAILURES)} and c['check']=='autovm')]
    wanted={('ace.m1.'+n.lower(),t) for n in VALUE_CASES+TRAP_CASES+tuple(DIAGNOSTICS) for t in ('ac-native','av')}|{('ace.m1.'+n.lower(),'autovm') for n in VM_VALUE_CASES+tuple(VM_FAILURES)}
    typed_equal({(c['case'],c['check']) for c,_ in selected},wanted,'original BASE source check set');require(len(selected)==65,'original BASE source check count')
    observations={};pending={}
    for check,row in selected:
        name=check['case'].split('.')[-1].upper();target='avm' if check['check']=='autovm' else 'ac' if check['check']=='ac-native' else 'av'
        def read_one(_):
            if target=='avm':return vm_failure_observation(inputs,copy,check,row,name,vm_failures[name]) if name in VM_FAILURES else vm_value_observation(inputs,copy,check,row,name,others[name])
            wire=_wires(copy,check,'mnworker/1' if target=='ac' else 'avworker/1',check['recipe'],workers['native' if target=='ac' else 'av'])
            require(len(wire)==1,'one original BASE exchange')
            return value_observation(inputs,copy.scope,wire[0],check,row['status'],name,target,contracts[name]) if name in VALUE_CASES else nonvalue_observation(inputs,copy.scope,wire[0],check,row['status'],name,target,others[name])
        item=_compact_admission(inputs,read_one,32<<10);cell='corpus:'+name+':'+target
        (pending if item['judgement']=='UNOBSERVED' else observations)[cell]=item
    previous={} if previous is None else previous
    require(not previous.keys() & observations.keys(),'duplicate original BASE required observations')
    result=execution_summary(required,previous|observations)
    result['pending_observations']=pending
    result['source_case_report']=dict(report_sha256=admitted['report_sha256'],non_pass_rows=[
        dict(case=c['case'],check=c['check'],status=row['status']) for c,row in admitted['checks'] if row['status']!='PASS'])
    return result
