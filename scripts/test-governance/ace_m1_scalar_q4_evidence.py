"""Frozen scalar source observations; exit 70 alone never proves a named trap."""
import base64
import binascii
import hashlib
import json
import re
from pathlib import PurePosixPath, PureWindowsPath

from ace_m1_arch_acceptance import AdmissionError, EvidenceScope, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import _counts, _duration, _fnv1a64

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/scalar-q4-contract.json'
CONTRACT_SHA  = '66f5a66c4b4f0bc3f5846b82ca39f35d28f284b51d18eae6a06d553f8708bf84'
SUITE = 'crates/auto-lang/test/cases/ace-m1-expanded/scalar-q4.suite.ad'
TRACE_CAP = 'hir.test.trace.v1'
TRAPS = {'Overflow':'av.trap.int-overflow', 'DivByZero':'av.trap.div-by-zero',
         'DivOverflow':'av.trap.div-overflow', 'RemOverflow':'av.trap.rem-overflow',
         'RemByZero':'av.trap.div-by-zero'}
# The AV owner explicitly shares the division-zero trap for remainder by zero.
# Native's unlabelled exit 70 has no corresponding semantic authority.
# PLAN-1000 F-1000-01 (owner option a): the mn machine's own bounded trap
# taxonomy (typed-runtime reasons, exposed response-side by the trap-probe
# observation build — mn.trap.* names, never an AV-engine inference). The
# oracle translation shares codes exactly like the AV owner does: rem-by-zero
# is the division-zero trap, rem MIN/-1 the division-overflow trap; the
# int-overflow family carries the four checked-arithmetic names.
NATIVE_TRAPS = {'mn.trap.div-zero','mn.trap.div-overflow','mn.trap.add-overflow',
                'mn.trap.sub-overflow','mn.trap.mul-overflow','mn.trap.neg-overflow'}
EXPECTED_NATIVE_TRAPS = {
    'Overflow': {'mn.trap.add-overflow','mn.trap.sub-overflow','mn.trap.mul-overflow','mn.trap.neg-overflow'},
    'DivByZero': {'mn.trap.div-zero'},
    'DivOverflow': {'mn.trap.div-overflow'},
    'RemOverflow': {'mn.trap.div-overflow'},
    'RemByZero': {'mn.trap.div-zero'},
}


def _hex(raw):
    require(type(raw) is str and re.fullmatch(r'(?:[0-9a-f]{2})*', raw) is not None,
            'canonical scalar process bytes required')
    return bytes.fromhex(raw)


def scalar_q4_observation(scope, wire, check, contract, *, query_family=False, legacy_base=False):
    require(type(query_family) is bool and type(legacy_base) is bool and not (query_family and legacy_base),'finite source observation family')
    if legacy_base:
        from ace_m1_legacy_base import TRAP_CASES,DIAGNOSTICS
        require(contract['family']=='legacy-base' and contract['case'] in TRAP_CASES+tuple(DIAGNOSTICS),'finite original BASE non-value')
    if query_family:require(contract['family'] in ('source-query','source-instances'),'original query/instance source family')
    negative=(query_family or legacy_base) and contract['oracle_kind']=='diagnostic'
    typed_negative=negative and not (legacy_base and contract['case']=='B16' and check['recipe']=='ac/mir-native-scalar-v1')
    require(contract['oracle_kind'] in (('value','diagnostic') if query_family else ('trap','diagnostic') if legacy_base else ('value','trap')) and contract['trace'] in ('true','false'),
            'scalar Q4 independent oracle domain')
    require(contract['oracle_kind']!='trap' or contract['oracle_value'] in TRAPS,
            'scalar Q4 independent trap taxonomy')
    av = check['recipe'] == 'ac-av/core-run-v1'
    recipe = 'ac-av/core-run-v1' if av else 'ac/mir-native-scalar-v1'
    protocol = 'avworker/1' if av else 'mnworker/1'
    typed_equal(check['recipe'], recipe, 'scalar Q4 recipe')
    typed_equal(wire['protocol'], protocol, 'scalar Q4 protocol')
    typed_equal(wire['recipe'], recipe, 'scalar Q4 wire recipe')
    require(type(wire['exit_code']) is int and wire['exit_code'] == 0
            and wire['transport_error'] is None and wire['stderr'] == b'',
            'scalar Q4 incomplete transport')
    request, _ = scope.decode_json(wire['request'])
    exact_keys(request, {'recipe','nonce','source_b64','entry','caps','store_root','request_hash'}
               | ({'steps'} if av else {'workdir'} | ({'diagnostics'} if typed_negative else set())), 'scalar Q4 request')
    require(type(request['nonce']) is str and bool(request['nonce']), 'scalar Q4 nonce')
    typed_equal(request['recipe'], recipe, 'scalar Q4 declared recipe')
    typed_equal(request['entry'], 'd_main', 'scalar Q4 entry')
    caps = [TRACE_CAP] if contract['trace'] == 'true' else []
    if query_family:
        caps,_=scope.decode_json(contract['caps'].encode())
        require(caps in ([[],[TRACE_CAP]] if contract['family']=='source-query' else [['ac.source.instances.scalar/v1']]),'original query/instance capabilities')
    typed_equal(request['caps'], caps, 'scalar Q4 independent capabilities')
    if typed_negative and not av:typed_equal(request['diagnostics'],'source-v1','original strict source diagnostic mode')
    for key in ('store_root',) if av else ('store_root','workdir'):
        require(type(request[key]) is str and bool(request[key]), 'scalar Q4 output owner')
    if av: typed_equal(request['steps'], [{'op':'run'}], 'scalar Q4 one run')
    unsigned = {key:value for key,value in request.items() if key != 'request_hash'}
    digest = hashlib.sha256(json.dumps(unsigned, ensure_ascii=False, separators=(',',':')).encode()).hexdigest()
    typed_equal(request['request_hash'], digest, 'scalar Q4 recomputed request hash')
    require(type(request['source_b64']) is str, 'scalar Q4 encoded source')
    try: source = base64.b64decode(request['source_b64'], validate=True)
    except (ValueError, binascii.Error) as error: raise AdmissionError('invalid scalar Q4 source') from error
    typed_equal(base64.b64encode(source).decode(), request['source_b64'], 'scalar Q4 canonical source')
    typed_equal(hashlib.sha256(source).hexdigest(), contract['source_sha256'], 'scalar Q4 independent source')
    response, _ = scope.decode_json(wire['stdout'])
    required = {'schema','nonce','request_hash','ok'}
    if typed_negative and not av:exact_keys(response,required|{'source_error','diagnostic'},'typed native source diagnostic envelope')
    else:
        require(type(response) is dict and required <= response.keys() <= required | ({'diagnostic'} if not av else set()),
                'scalar Q4 response envelope')
    for key,want in [('schema',protocol),('nonce',request['nonce']),('request_hash',digest)]:
        typed_equal(response[key],want,'scalar Q4 response '+key)
    ok = response['ok']
    expected = {'kind':contract['oracle_kind'], 'value':contract['oracle_value']}
    marks = [{'mark_a':'a','mark_b':'b'}[mark] for mark in contract['marks'].split(',') if mark]
    judgement, reason = 'FAIL', None
    if av:
        exact_keys(ok, ('recipe','steps','worker_version','av_core_version'), 'AV scalar Q4 ok')
        typed_equal(ok['worker_version'],'avworker/1.0','AV scalar Q4 version')
        typed_equal(ok['av_core_version'],'av.core-i32/1','AV scalar Q4 core')
        require(type(ok['steps']) is list and len(ok['steps']) == 1, 'AV scalar Q4 step count')
        step = ok['steps'][0]
        exact_keys(step, ('op','outcome','value','channel','trap','diag','counters','spans','source_hash','wall_ms','trace'), 'AV scalar Q4 step')
        typed_equal(step['op'],'run','AV scalar Q4 step')
        typed_equal(step['source_hash'],_fnv1a64(source),'AV scalar Q4 source FNV')
        _counts(step['counters'],'AV scalar Q4 counters');_duration(step['wall_ms'],'AV scalar Q4 wall')
        require(type(step['spans']) is list,'AV scalar Q4 spans')
        seen = set()
        for span in step['spans']:
            require(type(span) is list and len(span)==2 and type(span[0]) is str and bool(span[0])
                    and span[0] not in seen,'AV scalar Q4 unique span')
            seen.add(span[0]);_duration(span[1],'AV scalar Q4 span duration')
        require(type(step['trace']) is list and all(type(v) is str and v in ('a','b') for v in step['trace']),
                'AV scalar Q4 trace')
        trace = step['trace']
        if step['outcome'] == 'diag':
            typed_equal((step['value'],step['channel'],step['trap']),(None,None,None),'AV scalar Q4 diagnostic result')
            exact_keys(step['diag'],('stage','code','message'),'AV scalar Q4 diagnostic')
            require(all(type(v) is str and bool(v) for v in step['diag'].values()),'AV scalar Q4 diagnostic text')
            observed = {'kind':'diagnostic', **step['diag']}
            if negative:
                typed_equal(step['counters'],{'frontend_reject':1,'requests':1},'AV original source rejection counters')
                judgement='PASS' if step['diag']['stage']=='frontend' and step['diag']['code']==expected['value'] and trace==[] else 'FAIL'
        elif step['outcome'] == 'trap':
            typed_equal((step['value'],step['channel'],step['diag']),(None,None,None),'AV scalar Q4 trap result')
            require(step['trap'] in set(TRAPS.values()),'AV scalar Q4 named trap')
            observed = {'kind':'trap','code':step['trap']}
            judgement = 'PASS' if expected['kind']=='trap' and TRAPS[expected['value']]==step['trap'] and trace==marks else 'FAIL'
        else:
            typed_equal((step['outcome'],step['channel'],step['trap'],step['diag']),('success','value',None,None),'AV scalar Q4 value result')
            require(type(step['value']) is int and -(1<<31)<=step['value']<(1<<31),'AV scalar Q4 i32')
            observed = {'kind':'value','value':step['value']}
            judgement = 'PASS' if expected['kind']=='value' and step['value']==int(expected['value']) and trace==marks else 'FAIL'
    else:
        common = {'recipe','worker_version','mn_abi','mn_target','counters','spans'}
        if typed_negative:common |= {'diagnostics_mode','diagnostics_schema'}
        diagnosed = 'diagnostic' in response
        # PLAN-1000 F-1000-01: a trapped production run may carry the exact
        # trap-probe observation key set (trap/trap_reason/trap_observation)
        # — everything else keeps the frozen key shape.
        trap_keys = {'trap','trap_reason','trap_observation'}
        ok_keys = common | ({'manifest_fp','exit_code','stderr_hex','stdout_hex'}
                            | (trap_keys if not diagnosed and type(ok) is dict
                               and ok.get('trap') is not None else set()))
        exact_keys(ok, common if diagnosed else ok_keys, 'native scalar Q4 ok')
        for key,want in [('worker_version','mnworker/1.0'),('mn_abi','msvc-x64/priv-i32-bool-v1'),('mn_target','x86_64-pc-windows-msvc')]:
            typed_equal(ok[key],want,'native scalar Q4 '+key)
        _counts(ok['counters'],'native scalar Q4 counters');_counts(ok['spans'],'native scalar Q4 spans')
        trace = []
        if diagnosed:
            diag = response['diagnostic'];exact_keys(diag,('code','message'),'native scalar Q4 diagnostic')
            require(all(type(v) is str and bool(v) for v in diag.values()),'native scalar Q4 diagnostic text')
            observed = {'kind':'diagnostic',**diag}
            if typed_negative:
                from ace_m1_query_evidence import native_source_diagnostic
                typed_equal(ok['diagnostics_mode'],'source-v1','native strict diagnostics mode')
                typed_equal(ok['diagnostics_schema'],'mn.source-diagnostic/1','native strict diagnostics schema')
                typed_equal(ok['counters'],{'frontend_reject':1},'native pre-execution source rejection')
                exact_keys(ok['spans'],('frontend',),'native source rejection span')
                observed,judgement=native_source_diagnostic(response,source,contract,legacy_base=legacy_base)
            elif legacy_base and negative:
                typed_equal(diag['code'],'mn.unsupported.frontend','original BASE untyped refusal code')
                require(len(diag['message'].encode())<=4096,'BASE untyped diagnostic message cap')
                typed_equal(ok['counters'],{'frontend_reject':1},'BASE untyped pre-execution rejection')
                exact_keys(ok['spans'],('frontend',),'BASE untyped rejection span')
                judgement,reason='UNOBSERVED','original BASE native refusal lacks requested typed source facts; rendered message is not authority'
        else:
            from ace_m1_text_q4_evidence import NATIVE_COUNTERS, NATIVE_SPANS
            exact_keys(ok['counters'],NATIVE_COUNTERS,'native scalar Q4 production counters')
            exact_keys(ok['spans'],NATIVE_SPANS,'native scalar Q4 production spans')
            typed_equal(ok['counters']['link_invocations'],1,'native scalar Q4 actual link')
            typed_equal(ok['counters']['run_invocations'],1,'native scalar Q4 actual run')
            require(type(ok['manifest_fp']) is str and re.fullmatch('[0-9a-f]{16}',ok['manifest_fp']) is not None,'native scalar Q4 manifest')
            require(type(ok['exit_code']) is int and -(1<<31)<=ok['exit_code']<(1<<31),'native scalar Q4 i32')
            stdout, stderr = _hex(ok['stdout_hex']), _hex(ok['stderr_hex'])
            observed = {'kind':'process-exit','value':ok['exit_code'],'stdout_hex':ok['stdout_hex'],'stderr_hex':ok['stderr_hex']}
            # PLAN-1000 F-1000-01: an optional response-side NAMED trap
            # (trap-probe observation build; production bytes unchanged).
            # Present only as the exact three-field set, with the machine's
            # own bounded taxonomy and a bounded reason code.
            named_trap = ok.get('trap')
            if named_trap is not None:
                require(type(named_trap) is str and named_trap in NATIVE_TRAPS,'native scalar Q4 machine trap name')
                require(type(ok.get('trap_reason')) is int and 1 <= ok['trap_reason'] <= 6,'native scalar Q4 bounded trap reason')
                typed_equal(ok.get('trap_observation'),'trap-probe/1','native scalar Q4 trap observation mode')
                observed = dict(observed,trap=named_trap,trap_reason=ok['trap_reason'])
            if expected['kind']=='value':
                judgement = 'PASS' if ok['exit_code']==int(expected['value']) and stdout==b'' and stderr==''.join(marks).encode() else 'FAIL'
            elif ok['exit_code']==70 and stdout==stderr==b'':
                if named_trap is None:
                    judgement,reason='UNOBSERVED','native exit 70 lacks an independent named-trap observation'
                elif expected['kind']!='trap' or named_trap not in EXPECTED_NATIVE_TRAPS.get(expected['value'],set()):
                    judgement='FAIL'
                else:
                    judgement='PASS'
    typed_equal(ok['recipe'],recipe,'scalar Q4 response recipe')
    result = dict(schema='ace.m1.scalar-q4-observation/1',source_sha256=contract['source_sha256'],
                  request_sha256=hashlib.sha256(wire['request']).hexdigest(),response_sha256=hashlib.sha256(wire['stdout']).hexdigest(),
                  expected=expected,observed=observed,expected_marks=marks,observed_av_marks=trace if av else None,judgement=judgement)
    if reason: result['reason']=reason
    return result


def admit_scalar_q4(inputs, evidence, binding, workers):
    from ace_m1_first_slice import _compact_admission, _inventory_for_suite, _repo_relative, _wires, retain_parent_status
    from ace_m1_obligation_catalog import SCALAR_CATALOG, SCALAR_FIELDS, read_tsv, admit_scalar_catalog
    exact_keys(binding,('inventory','copy','tool_capture'),'scalar Q4 binding')
    exact_keys(binding['tool_capture'],('path','sha256'),'scalar Q4 original tool capture')
    evidence.sha256(binding['tool_capture']['path'],binding['tool_capture']['sha256'])
    tool_record,_=evidence.read_json(binding['tool_capture']['path'])
    typed_equal(tool_record.get('groups'),['case-ace-m1-scalar-q4-941'],'scalar Q4 original fixed group')
    typed_equal(tool_record.get('executed'),122,'scalar Q4 original fixed denominator')
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
    rows=read_tsv(inputs,SCALAR_CATALOG,SCALAR_FIELDS);admit_scalar_catalog(inputs,rows)
    cases={'ace.m1.scalarq4.'+row['case'].lower():row for row in rows}
    require(len(cases)==61,'unique scalar Q4 cases')
    copy=CaseCopy(EvidenceScope(evidence.path(binding['copy']+'/artifact-copy.json').parent,inputs.budget))
    admitted=admit_case_bindings(copy,inventory)
    seen,observations,non_pass=set(),{},[]
    for check,parent in admitted['checks']:
        require(check['case'] in cases and check['check'] in ('av','ac-native'),'foreign scalar Q4 check')
        contract=cases[check['case']];av=check['check']=='av';target='AV' if av else 'AC-native'
        slot='q4-source/scalar/'+contract['case']+'/'+target
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
    typed_equal(seen,{'q4-source/scalar/'+row['case']+'/'+target for row in rows for target in ('AV','AC-native')},'complete 61-source scalar Q4 set')
    counts={state:sum(v['judgement']==state for v in observations.values()) for state in ('PASS','FAIL','UNOBSERVED')}
    return dict(schema='ace.m1.scalar-q4-observations/1',required=122,observations=observations,counts=counts,
                source_case_report=dict(report_sha256=admitted['report_sha256'],non_pass_rows=non_pass),
                semantic_evidence_complete=counts['PASS']==122,certification_targets='UNOBSERVED',budgets_and_full_lifecycle='UNOBSERVED',can_claim_expanded_m1=False)
