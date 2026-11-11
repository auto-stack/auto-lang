"""Strict S936 disk restore slice. Other restore owners remain unobserved."""
import os
import re
from ace_m1_arch_acceptance import EvidenceScope, counter, exact_keys
from ace_m1_case_evidence import require
from ace_m1_ct_mechanisms import COUNTERS, native_facts
from ace_m1_ct_meta import hex_text, state as meta_state
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/ct-restore-contract.json'
CONTRACT_SHA = '8131ae35ffecebe7e04bc0e77edbd23e7f0befc6c7b3f8744d526bb08b9de4f2'
BINARY = 'ace-source-cte::architecture_restore_941'
TEST = 'ordinary_source_disk_restore_first_edits_and_repair_941'
SOURCES = ('crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum42.at',) + tuple(
    'crates/auto-lang/test/cases/ace-m1-expanded/ct-mechanisms/restore-'+s+'.at'
    for s in ('args', 'trap', 'repair', 'second'))
PHASES = ('cold', 'first_edit', 'restored', 'failed', 'preserved', 'repaired',
          'second_edit', 'repair_restore', 'missing', 'fallback')
SLICE = ('ct/restore-first-good-edit', 'ct/restore-first-bad-repair-second-edit',
         'ct/restore-missing-page-cold')


def state(value):
    exact_keys(value, (*COUNTERS, 'read_probes', 'read_bytes', 'read_decodes', 'exec_log'), 'disk CT service state')
    # Reuse the strict successful-log validator after excluding the one typed
    # failing attempt; retain its identity and exact outcome for the checks below.
    log = value['exec_log']
    require(type(log) is list and len(log) <= 3, 'bounded restore execution log')
    converted = []
    for event in log:
        require(type(event) is dict, 'typed restore boundary event')
        outcome = event.get('outcome_hex')
        require(outcome in (b'ok'.hex(), b'hit'.hex(), b'cte.trap/av.trap.div-by-zero'.hex()), 'typed restore execution outcome')
        converted.append(dict(event, outcome_hex=b'ok'.hex()) if outcome == b'cte.trap/av.trap.div-by-zero'.hex() else event)
    meta_state({**{k:value[k] for k in COUNTERS}, 'probes':value['read_probes'], 'exec_log':converted})
    for name in ('read_probes', 'read_bytes', 'read_decodes'): counter(value[name], 'restore '+name)


def validate_payload(document, sources):
    exact_keys(document, ('schema', 'sink', 'owner_values', 'sources_hex', 'store_root_hex',
                         'trap_code_hex', 'missing_code_hex', *PHASES), 'disk restore observation')
    typed_equal(document['schema'], 'ace.m1.ct-disk-restore/1', 'CT disk restore schema')
    typed_equal(document['sink'], 'ManagedEnvelopeSink/S936', 'existing actual managed CT sink')
    require(type(sources) is list and len(sources) == 5 and all(type(s) is bytes for s in sources), 'finite source byte set')
    typed_equal(document['sources_hex'], [s.hex() for s in sources], 'actual restore source bytes')
    for name in ('store_root_hex', 'trap_code_hex', 'missing_code_hex'): hex_text(document[name], 'restore '+name)
    root = bytes.fromhex(document['store_root_hex']).decode().replace('\\', '/').removeprefix('//?/')
    # PLAN-1005: the registered scope root is parameterized the same way the
    # observer is (AUTO_PLAN_941_SCOPE_ROOT, default keeps the 941 root); the
    # store must still live inside the canonicalized registered scope.
    scope_root = os.environ.get('AUTO_PLAN_941_SCOPE_ROOT', 'D:/autostack/plan-artifacts/plan-941').replace('\\', '/')
    require(re.fullmatch(re.escape(scope_root) + r'/ct-restore-store-[0-9]+-[0-9]+', root) is not None, 'owned actual restore store scope')
    values = document['owner_values']
    require(type(values) is list and len(values) == 11 and all(type(v) is int and -(1<<31) <= v < (1<<31) for v in values), 'eleven actual checked owner values')
    for phase in PHASES: state(document[phase])
    def counts(phase, expected, reads):
        s = document[phase]
        return (tuple(s[k] for k in COUNTERS) == expected and s['read_probes'] == s['read_decodes'] == reads
                and (s['read_bytes'] > 0 if reads else s['read_bytes'] == 0))
    def event(row, key, attempt, outcome):
        return (row['key_hash'] == key and row['attempt'] == attempt and row['outcome_hex'] == outcome.encode().hex()
                and row['from_cache'] is (outcome == 'hit') and (row['steps'] > 0 if outcome in ('ok','hit') else row['steps'] == 0))
    logs = {p:document[p]['exec_log'] for p in PHASES}
    lengths = dict(cold=1, first_edit=1, restored=2, failed=1, preserved=2, repaired=3,
                   second_edit=1, repair_restore=2, missing=0, fallback=1)
    structural = all(len(logs[p]) == n for p,n in lengths.items())
    if not structural: return dict.fromkeys(SLICE, 'FAIL')
    good, args, bad, repair, second = (logs[p][-1]['key_hash'] for p in ('cold','first_edit','failed','repaired','second_edit'))
    keys_distinct = len({good,args,bad,repair,second}) == 5
    good_ok = (keys_distinct and values[:4] == [42]*4 and counts('cold',(1,0,1,0),0)
               and counts('first_edit',(1,0,1,0),0) and counts('restored',(0,2,0,0),2)
               and event(logs['cold'][0],good,1,'ok') and event(logs['first_edit'][0],args,1,'ok')
               and event(logs['restored'][0],good,0,'hit') and event(logs['restored'][1],args,0,'hit')
               and all(logs['restored'][i][k] == logs[p][0][k] for i,p in enumerate(('cold','first_edit')) for k in ('steps','wall_ms')))
    trap = 'cte.trap/av.trap.div-by-zero'
    repair_ok = (keys_distinct and values[4:10] == [42,42,43,44,43,44]
                 and document['trap_code_hex'] == trap.encode().hex()
                 and counts('failed',(1,0,0,1),0) and counts('preserved',(1,1,0,1),1)
                 and counts('repaired',(2,1,1,1),1) and counts('second_edit',(1,0,1,0),0)
                 and counts('repair_restore',(0,2,0,0),2)
                 and event(logs['failed'][0],bad,1,trap)
                 and logs['preserved'][:1] == logs['failed'] and event(logs['preserved'][1],good,0,'hit')
                 and logs['repaired'][:2] == logs['preserved'] and event(logs['repaired'][2],repair,2,'ok')
                 and event(logs['second_edit'][0],second,1,'ok')
                 and event(logs['repair_restore'][0],repair,0,'hit') and event(logs['repair_restore'][1],second,0,'hit')
                 and all(logs['repair_restore'][i][k] == logs[p][-1][k] for i,p in enumerate(('repaired','second_edit')) for k in ('steps','wall_ms')))
    missing_ok = (values[-1] == 42 and document['missing_code_hex'] == b'cte.infra/av-forbidden'.hex()
                  and counts('missing',(0,0,0,1),0) and not logs['missing']
                  and counts('fallback',(1,0,1,1),0) and event(logs['fallback'][0],good,1,'ok'))
    return {key:'PASS' if ok else 'FAIL' for key,ok in zip(SLICE,(good_ok,repair_ok,missing_ok))}


def admit(inputs, evidence, binding):
    exact_keys(binding, ('root',), 'CT disk restore binding')
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    declaration,_ = inputs.read_json(CONTRACT)
    typed_equal(declaration['schema'], 'ace.m1.ct-restore-contract/1', 'CT restore declaration')
    typed_equal(declaration['slices'], list(SLICE), 'finite CT-only restore slice')
    typed_equal(declaration['execution'], 'UNOBSERVED', 'declaration is not execution')
    scope = EvidenceScope(evidence.path(binding['root']+'/observations.json').parent, inputs.budget)
    def judge(document, _sum42, _sum43): return validate_payload(document, [inputs.read_bytes(p) for p in SOURCES])
    result = native_facts(inputs, scope, declaration['source_pins'], judge, binary_name=BINARY, test_name=TEST)
    result['managed_restore'] = 'S936 executed envelopes only; all other required restore owners remain UNOBSERVED'
    result['architecture_obligations'] = dict.fromkeys((
        'ARCH-05/restore-immediate-good-edit', 'ARCH-05/bad-first-repair-second-restore',
        'ARCH-05/all-required-pages'), 'UNOBSERVED')
    result['can_claim_architecture_pass'] = False
    return result
