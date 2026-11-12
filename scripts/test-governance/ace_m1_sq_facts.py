"""Actual SQ type authority and restore-domain evidence; no producer launch."""
from ace_m1_arch_acceptance import EvidenceScope, counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_ct_mechanisms import native_facts
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/sq-fact-contract.json'
CONTRACT_SHA = 'af93847d9a25b4bfec812c5a639bc62d6621bf2d59b75ed8ce22e130120720cc'
SOURCE = 'crates/auto-lang/test/cases/ace-m1-expanded/query-instances/fact-authority.at'
BINARY = 'ace-source-query::architecture_facts_941'
TEST = 'ordinary_source_fact_authority_and_restore_identity_941'
ARCH = ('ARCH-02/solver-authority', 'ARCH-02/facts-before-hir')
COUNTERS = ('solver_runs','sig_hits','sig_misses','body_hits','body_misses','rejections','diagnostic_verdicts')


def state(value):
    exact_keys(value, COUNTERS, 'actual SQ counters')
    for key in COUNTERS: counter(value[key], 'SQ '+key)


def counts(value, expected):
    state(value)
    return tuple(value[key] for key in COUNTERS) == expected


def applied(rows, hit):
    require(type(rows) is list and len(rows) == 3, 'three source function applications')
    ok = True
    versions = []
    for row,name,exprs,bindings in zip(rows,('leaf','twin','main'),(4,4,2),(1,1,0)):
        exact_keys(row, ('name_hex','sig_hit','body_hit','body_version','exprs','bindings'), 'real SQ fact application')
        typed_equal(row['name_hex'],name.encode().hex(),'actual fact owner order')
        hash_value(row['body_version'],'SQ structural body version')
        for key in ('sig_hit','body_hit'): require(type(row[key]) is bool,'typed SQ application hit')
        for key in ('exprs','bindings'): counter(row[key],'actual applied '+key)
        ok &= row['sig_hit'] is hit and row['body_hit'] is hit and row['exprs'] == exprs and row['bindings'] == bindings
        versions.append(row['body_version'])
    return ok and versions[0] == versions[1] and versions[0] != versions[2]


def restored(row):
    exact_keys(row, ('verify_admitted','phase','admitted_bodies','hir_blake3','leaf_logical'), 'actual SQ restore result')
    require(type(row['verify_admitted']) is bool,'typed fact restore admission')
    require(row['phase'] in ('verified','restore-refused'),'actual restore terminal phase')
    counter(row['admitted_bodies'],'restore supplied bodies')
    hash_value(row['leaf_logical'],'fact page logical identity')
    if row['phase'] == 'verified': hash_value(row['hir_blake3'],'actual verified restored HIR')
    else: typed_equal(row['hir_blake3'],'','no HIR from refused restore')


def validate_payload(document, source):
    exact_keys(document, ('schema','source_hex','own_logical','denied','cold','hit','legacy_hir_blake3',
                          'poisoned','own_restore','foreign'), 'actual SQ observation')
    typed_equal(document['schema'],'ace.m1.sq-fact-authority/1','SQ observer schema')
    typed_equal(document['source_hex'],source.hex(),'original frozen source bytes')
    hash_value(document['own_logical'],'actual current owner logical')
    hash_value(document['legacy_hir_blake3'],'actual independent legacy HIR digest')
    denied = document['denied']; exact_keys(denied,('outcome','counters'),'cold disabled result')
    exact_keys(denied['outcome'],('phase','query_hex','owner_hex'),'actual solver disabled query')
    denied_ok = (denied['outcome'] == dict(phase='query-disabled',query_hex=b'infer-body'.hex(),owner_hex=b'leaf'.hex())
                 and counts(denied['counters'],(0,0,1,0,1,1,0)))
    accepted = []
    for name,expected,hit in (('cold',(3,0,3,0,3,0,0),False),('hit',(3,3,3,3,3,0,0),True)):
        row = document[name]; exact_keys(row,('hir_blake3','counters','applied'),'actual SQ compile')
        hash_value(row['hir_blake3'],'actual facts HIR digest')
        accepted.append(counts(row['counters'],expected) and applied(row['applied'],hit)
                        and row['hir_blake3'] == document['legacy_hir_blake3'])
    poison = document['poisoned']; exact_keys(poison,('outcome','counters'),'poisoned type result')
    exact_keys(poison['outcome'],('phase','diagnostics'),'actual poison bind verdict')
    ds = poison['outcome']['diagnostics']; require(type(ds) is list and 1 <= len(ds) <= 16,'bounded poisoned fact diagnostics')
    for d in ds:
        exact_keys(d,('stage','code_hex'),'actual bind diagnostic')
        require(type(d['stage']) is str and type(d['code_hex']) is str,'typed bind diagnostic')
    poison_ok = (poison['outcome']['phase'] == 'bind-refused'
                 and all(d == dict(stage='bind',code_hex=b'source.fact-mismatch'.hex()) for d in ds)
                 and counts(poison['counters'],(0,0,3,3,0,0,0)))
    own = document['own_restore']; restored(own)
    own_ok = (own['verify_admitted'] is True and own['phase'] == 'verified' and own['admitted_bodies'] == 3
               and own['hir_blake3'] == document['legacy_hir_blake3'] and own['leaf_logical'] == document['own_logical'])
    foreign = document['foreign']; require(type(foreign) is list and len(foreign) == 3,'three separate foreign domains')
    foreign_ok = True
    identities = {document['own_logical']}
    for row,axis in zip(foreign,('package','module','owner')):
        exact_keys(row,('axis','seed_refused','restore'),'foreign identity control')
        typed_equal(row['axis'],axis,'finite foreign identity axis')
        require(type(row['seed_refused']) is bool,'actual seed refusal boolean')
        restored(row['restore']); foreign_page = row['restore']
        require(foreign_page['leaf_logical'] not in identities,'actual different logical domain')
        identities.add(foreign_page['leaf_logical'])
        foreign_ok &= (row['seed_refused'] is True and foreign_page['verify_admitted'] is False
                       and foreign_page['phase'] == 'restore-refused' and foreign_page['admitted_bodies'] == 0
                       and foreign_page['hir_blake3'] == '')
    return {ARCH[0]:'PASS' if denied_ok and all(accepted) and poison_ok else 'FAIL',
            ARCH[1]:'PASS' if own_ok and foreign_ok and poison_ok else 'FAIL'}


def admit(inputs, evidence, binding):
    exact_keys(binding,('root',),'SQ fact binding')
    inputs.sha256(CONTRACT,CONTRACT_SHA)
    declaration,_ = inputs.read_json(CONTRACT)
    typed_equal(declaration['schema'],'ace.m1.sq-fact-contract/1','SQ fact declaration')
    typed_equal(declaration['mechanisms'],list(ARCH),'two original SQ mechanisms')
    typed_equal(declaration['execution'],'UNOBSERVED','declaration is not execution')
    scope = EvidenceScope(evidence.path(binding['root']+'/observations.json').parent,inputs.budget)
    def judge(document,_sum42,_sum43): return validate_payload(document,inputs.read_bytes(SOURCE))
    result = native_facts(inputs,scope,declaration['source_pins'],judge,binary_name=BINARY,test_name=TEST,group_name='ace-semantics')
    result['schema'] = 'ace.m1.sq-fact-admission/1'
    result['other_architecture'] = 'Other SQ/instance/CT/restore obligations remain UNOBSERVED'
    result['can_claim_architecture_pass'] = False
    return result
