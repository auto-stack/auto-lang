"""Admit one real instance/checked-child CT version axis; no meta authority."""
from ace_m1_arch_acceptance import EvidenceScope, counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal
from ace_m1_ct_mechanisms import COUNTERS, LOG_FIELDS, CHILD_BINARY, CHILD_TEST, native_facts

ARCH = 'ARCH-04/instance-child-ct-version'
TRUE = 'crates/auto-lang/test/cases/ace-m1/source-cte-936/src/pick40.at'
FALSE = 'crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/pick42.at'
CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/ct-child-contract.json'
CONTRACT_SHA = '67c051ebb7b927e397b301dfa6c94465df5c654ad32ebd9ead45f6b4dd2472bd'


def validate_payload(document, source_true, source_false):
    exact_keys(document, ('schema','source_true_hex','source_false_hex','owner_values','instance_before',
                         'instance_after','cold','edited','hit','published_keys'), 'CT checked-child observation')
    typed_equal(document['schema'], 'ace.m1.ct-child-versions/1', 'CT checked-child schema')
    typed_equal(document['source_true_hex'], source_true.hex(), 'original child source')
    typed_equal(document['source_false_hex'], source_false.hex(), 'independent flag-edit source')
    require(source_true.count(b'return true') == 1, 'unique original flag body')
    typed_equal(source_true.replace(b'return true',b'return false',1), source_false, 'only checked child changes')
    values = document['owner_values']
    require(type(values) is list and len(values)==3 and all(type(v) is int for v in values), 'three actual owner values')
    for name in ('instance_before','instance_after'):
        i = document[name]
        exact_keys(i, ('logical','key','template_hex','concrete_hex','signature','body','demands'), 'actual template instance')
        for field in ('logical','key','signature','body'):hash_value(i[field], 'instance '+field)
        counter(i['demands'], 'actual instance demands')
        typed_equal((i['template_hex'],i['concrete_hex']), (b'pick'.hex(),b'pick_i32'.hex()), 'concrete instance owner')
    for name in ('cold','edited','hit'):
        state=document[name];exact_keys(state, (*COUNTERS,'exec_log'), 'CT child state')
        for field in COUNTERS:counter(state[field], 'CT child '+field)
        require(type(state['exec_log']) is list and len(state['exec_log'])<=6, 'bounded child execution log')
        for row in state['exec_log']:
            exact_keys(row,LOG_FIELDS,'child AV boundary event');hash_value(row['key_hash'],'child key')
            for field in ('attempt','steps','wall_ms'):counter(row[field],'child event '+field)
            require(type(row['from_cache']) is bool,'actual child cache indicator')
            require(row['outcome_hex'] in (b'ok'.hex(),b'hit'.hex()),'typed child success/hit')
    keys=document['published_keys']
    require(type(keys) is list and len(keys)==4,'four actual successful publications')
    for row in keys:
        exact_keys(row, ('key_hash','callee_hex','logical','signature','body','args_hex'), 'published child execution key')
        for field in ('key_hash','logical','signature','body'):hash_value(row[field], 'published key '+field)
    def counts(name,want):return tuple(document[name][f] for f in COUNTERS)==want
    cold,edited,hit=(document[name]['exec_log'] for name in ('cold','edited','hit'))
    passed=(values==[40,42,42] and document['instance_before']==document['instance_after']
            and document['instance_before']['demands']==1
            and counts('cold',(2,0,2,0)) and len(cold)==2
            and counts('edited',(4,0,4,0)) and len(edited)==4 and edited[:2]==cold
            and counts('hit',(4,2,4,0)) and len(hit)==6 and hit[:4]==edited)
    if passed:
        for index,row in enumerate(edited):
            passed &= (row['attempt']==index+1 and row['steps']>0 and row['from_cache'] is False
                       and row['outcome_hex']==b'ok'.hex() and row['key_hash']==keys[index]['key_hash'])
        for index,row in enumerate(hit[4:]):
            old=edited[index+2]
            passed &= (row['attempt']==0 and row['from_cache'] is True and row['outcome_hex']==b'hit'.hex()
                       and row['key_hash']==old['key_hash'] and row['steps']==old['steps'] and row['wall_ms']==old['wall_ms'])
        for first,second,callee,args in ((keys[0],keys[2],b'flag',b''),(keys[1],keys[3],b'pick_i32',b'i32:40,i32:42')):
            passed &= (first['callee_hex']==second['callee_hex']==callee.hex()
                       and first['args_hex']==second['args_hex']==args.hex()
                       and first['logical']==second['logical'] and first['signature']==second['signature']
                       and first['body']!=second['body'] and first['key_hash']!=second['key_hash'])
    return {ARCH:'PASS' if passed else 'FAIL'}


def admit(inputs, evidence, binding):
    exact_keys(binding, ('root',), 'CT child bindings')
    inputs.sha256(CONTRACT,CONTRACT_SHA);declaration,_=inputs.read_json(CONTRACT)
    typed_equal(declaration['schema'],'ace.m1.ct-child-contract/1','CT child contract schema')
    typed_equal(declaration['mechanisms'],[ARCH],'one child version obligation')
    typed_equal(declaration['execution'],'UNOBSERVED','child declaration is not execution')
    scope=EvidenceScope(evidence.path(binding['root']+'/observations.json').parent,inputs.budget)
    def judge(document,_sum42,_sum43):
        return validate_payload(document,inputs.read_bytes(TRUE),inputs.read_bytes(FALSE))
    result=native_facts(inputs,scope,declaration['source_pins'],judge,binary_name=CHILD_BINARY,test_name=CHILD_TEST)
    result['meta_instances']='checked-child-effective-version only; other instance/meta requirements UNOBSERVED'
    return result
