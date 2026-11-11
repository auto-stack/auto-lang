"""One actual meta argument/negative-dependency axis, never a whole ARCH pass."""
import re
from ace_m1_arch_acceptance import EvidenceScope, counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_ct_mechanisms import COUNTERS, LOG_FIELDS, META_BINARY, META_TEST, native_facts

ARCH = 'ARCH-04/meta-args-and-negative'
CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/ct-meta-contract.json'
CONTRACT_SHA = 'f4e1082acb53320dd3ea78e6337d8b94148d6436f0cd65ec7af62854fc8a91fc'
SOURCES = ('crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta2.at',
           'crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta3.at',
           'crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/has-absent.at',
           'crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/has-present.at',
           'crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/meta-only.at')

def hex_text(value,label):
    require(type(value) is str and len(value)<=512 and len(value)%2==0
            and re.fullmatch('[0-9a-f]*',value) is not None,label+' bounded UTF-8 hex')
    try:bytes.fromhex(value).decode('utf-8')
    except UnicodeDecodeError:require(False,label+' UTF-8')

def state(value):
    exact_keys(value, (*COUNTERS,'probes','exec_log'), 'actual meta service state')
    for name in (*COUNTERS,'probes'):counter(value[name], 'meta '+name)
    require(type(value['exec_log']) is list and len(value['exec_log'])<=3, 'bounded meta execute log')
    for row in value['exec_log']:
        exact_keys(row,LOG_FIELDS,'meta AV boundary');hash_value(row['key_hash'],'meta execution key')
        for name in ('attempt','steps','wall_ms'):counter(row[name],'meta log '+name)
        require(type(row['from_cache']) is bool, 'meta hit boolean')
        require(row['outcome_hex'] in (b'ok'.hex(),b'hit'.hex()),'meta execution outcome')

def counts(value, expected, probes):
    return tuple(value[name] for name in COUNTERS)==expected and value['probes']==probes

def axis(value, expected, callee, args):
    exact_keys(value,('values','meta_queries','control_code_hex','cold','hit','control','edited','published_keys'),'meta axis')
    require(type(value['values']) is list and len(value['values'])==3
            and all(type(v) is int and -(1<<31)<=v<(1<<31) for v in value['values']),'actual owner i32 values')
    hex_text(value['control_code_hex'],'meta control code')
    require(type(value['meta_queries']) is list and len(value['meta_queries'])==3,'actual meta counters')
    for count in value['meta_queries']:counter(count,'local meta work')
    for phase in ('cold','hit','control','edited'):state(value[phase])
    keys=value['published_keys'];require(type(keys) is list and len(keys)==2,'two actual successful meta-dependent publications')
    for key in keys:
        exact_keys(key,('key_hash','callee_hex','logical','signature','body','args_hex'),'published meta key')
        for name in ('key_hash','logical','signature','body'):hash_value(key[name],'meta key '+name)
        for name in ('callee_hex','args_hex'):hex_text(key[name],'meta '+name)
    cold,hit,control,edited=(value[p] for p in ('cold','hit','control','edited'))
    ok=(value['values']==expected and value['meta_queries']==[1,1,1]
        and value['control_code_hex']==b'cte.infra/av-forbidden'.hex()
        and counts(cold,(1,0,1,0),1) and counts(hit,(1,1,1,0),2)
        and counts(control,(1,1,1,1),3) and counts(edited,(2,1,2,1),4))
    a,b=keys
    ok &= (a['callee_hex']==b['callee_hex']==callee.hex() and a['logical']==b['logical']
           and a['signature']==b['signature'] and a['body']==b['body'] and a['key_hash']!=b['key_hash']
           and (a['args_hex'],b['args_hex'])==tuple(x.hex() for x in args))
    log=cold['exec_log'];hit_log=hit['exec_log'];last=edited['exec_log']
    ok &= len(log)==1 and len(hit_log)==2 and len(last)==3
    if ok:
        ok &= (log[0]['key_hash']==a['key_hash'] and log[0]['attempt']==1 and log[0]['steps']>0
               and log[0]['from_cache'] is False and log[0]['outcome_hex']==b'ok'.hex()
               and hit_log[0]==log[0] and hit_log[1]==dict(log[0],attempt=0,from_cache=True,outcome_hex=b'hit'.hex())
               and control['exec_log']==hit_log and last[:2]==hit_log and last[2]['key_hash']==b['key_hash']
               and last[2]['attempt']==2 and last[2]['steps']>0 and last[2]['from_cache'] is False and last[2]['outcome_hex']==b'ok'.hex())
    return ok

def validate_payload(document, sources):
    exact_keys(document,('schema','sources_hex','field','has','field_closed','has_closed','local'),'meta observation')
    typed_equal(document['schema'],'ace.m1.ct-meta-arguments/1','meta schema')
    typed_equal(document['sources_hex'],[s.hex() for s in sources],'five original meta source byte identities')
    require(len(sources)==5,'finite meta source set')
    for before,after in (sources[:2],sources[2:4]):
        typed_equal(before.replace(b'    y int\n',b'    y int\n    z int\n',1),after,'only nominal field addition changes')
    ok=axis(document['field'],[42,42,43],b'sum',(b'i32:2,i32:40',b'i32:3,i32:40'))
    ok=axis(document['has'],[42,42,40],b'pick',(b'bool:false',b'bool:true')) and ok
    for name in ('field_closed','has_closed'):
        closed=document[name];exact_keys(closed,('stage','code_hex','state'),'cold capability refusal')
        require(type(closed['stage']) is str and closed['stage'] in ('text','bind','verify','capability','backend','link','run'),'source diagnostic stage')
        hex_text(closed['code_hex'],'source diagnostic code')
        state(closed['state'])
        ok &= (closed['stage']=='bind' and closed['code_hex']==b'source.cte.bind'.hex()
               and counts(closed['state'],(0,0,0,0),0) and not closed['state']['exec_log'])
    local=document['local'];exact_keys(local,('value','meta_queries','state'),'meta-only demand')
    require(type(local['value']) is int and -(1<<31)<=local['value']<(1<<31),'meta-only i32');counter(local['meta_queries'],'local meta query')
    state(local['state'])
    ok &= (local['value']==2 and local['meta_queries']==1 and counts(local['state'],(0,0,0,0),0) and not local['state']['exec_log'])
    return {ARCH:'PASS' if ok else 'FAIL'}

def admit(inputs,evidence,binding):
    exact_keys(binding,('root',),'CT meta binding')
    inputs.sha256(CONTRACT,CONTRACT_SHA);declaration,_=inputs.read_json(CONTRACT)
    typed_equal(declaration['schema'],'ace.m1.ct-meta-contract/1','meta contract')
    typed_equal(declaration['mechanisms'],[ARCH],'one meta obligation')
    typed_equal(declaration['execution'],'UNOBSERVED','meta declaration is not execution')
    scope=EvidenceScope(evidence.path(binding['root']+'/observations.json').parent,inputs.budget)
    def judge(document,_sum42,_sum43):return validate_payload(document,[inputs.read_bytes(p) for p in SOURCES])
    result=native_facts(inputs,scope,declaration['source_pins'],judge,binary_name=META_BINARY,test_name=META_TEST)
    result['meta_instances']='meta arguments and absence only; template/meta composition remains UNOBSERVED'
    return result
