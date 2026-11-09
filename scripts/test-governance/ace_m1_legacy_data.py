"""Join admitted data observations to independent, frozen corpus obligations.

Only these fourteen old sources and their AC/AV legs are eligible. Method
variants, scenario rows, legacy VM and certification axes remain separate.
This module never reads child verdicts as an oracle or launches a producer.
"""
from ace_m1_arch_acceptance import hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import CORPUS, frozen_ledger, typed_equal

VALUES = ('R01', 'R02', 'R03', 'R04', 'R05', 'R06', 'T01', 'T02', 'T04')
DIAGNOSTICS = {
    'R09': ('nominal-type', 'verify', 'source.type-mismatch'),
    'R10': ('use-after-move', 'verify', 'verify.use-after-move'),
    'T06': ('borrow-escape', 'verify', 'verify.borrow-escape'),
    'T07': ('invalid-scalar', 'text', 'text.invalid-scalar'),
}
CASES = VALUES + ('T08',) + tuple(DIAGNOSTICS)


def contracts(inputs):
    from ace_m1_original_data_sources import source_contracts
    required = sorted(':'.join((r['ns'], r['id'], r['path']))
                      for r in frozen_ledger(inputs) if r['required'])
    rows = [line.split('\t') for line in inputs.read_bytes(CORPUS + 'binding.tsv').decode().splitlines()
            if line and not line.startswith('#')]
    # R01 is also a scenario name. Never key the old table by its ID alone.
    rows = {r[0]: r for r in rows if r[1] == 'corpus' and r[0] in CASES}
    typed_equal(set(rows), set(CASES), 'original record/text corpus denominator')
    manifest, _ = inputs.read_json(CORPUS + 'source-manifest.json')
    current = source_contracts(inputs)
    out = {}
    for name, row in rows.items():
        kind, old_value = row[3:5]
        if name in VALUES:
            typed_equal(kind, 'value', 'original record/text value family')
            value = str(int(old_value))
            typed_equal(value, old_value, 'canonical original numeric oracle')
        elif name == 'T08':
            typed_equal((kind, old_value), ('trap', 'slice-oob'), 'original T08 taxonomy')
            value = 'TextIndex'
        else:
            typed_equal((kind, old_value), ('diagnostic', DIAGNOSTICS[name][0]), 'original diagnostic taxonomy')
            value = 'record-type-mismatch' if name == 'R09' else old_value
        path = CORPUS + 'src/' + name + '.at'
        digest = manifest['corpus_src']['files'][name + '.at']
        inputs.sha256(path, digest)
        source = current[name]
        typed_equal((source['family'], source['source_file'], source['source_sha256'],
                     source['oracle_kind'], source['oracle_value']),
                    ('original-record-text', path, digest, kind, value), 'same original source and oracle')
        require(all('corpus:' + name + ':' + t in required for t in ('ac', 'av')), 'original required data targets')
        out[name] = dict(kind=kind, old_value=old_value, value=value, source_sha256=digest)
    return required, out


def observation(name, target, rule, source):
    """Rejudge facts already admitted by the strict, target-specific wire reader."""
    require(name in CASES and target in ('ac', 'av'), 'finite original data cell')
    diagnostic = name in DIAGNOSTICS
    typed_equal(source['schema'], 'ace.m1.original-data-diagnostic/1' if diagnostic
                else 'ace.m1.text-source-observation/1', 'admitted original data observation schema')
    typed_equal(source['source_sha256'], rule['source_sha256'], 'same frozen old source bytes')
    for field in ('request_sha256', 'response_sha256'):
        hash_value(source[field], 'original data wire ' + field)
    expected = dict(kind=rule['kind'], value=rule['value'])
    if diagnostic:
        _, stage, code = DIAGNOSTICS[name]
        expected = dict(kind='diagnostic', stage=stage, code=code)
    typed_equal(source['expected'], expected, 'independent old oracle agrees with source contract')
    fact = source['observed']
    require(type(fact) is dict, 'original data observation facts')
    require(source['judgement'] in ('PASS', 'FAIL', 'UNOBSERVED'), 'original data source verdict')
    parent = source['parent_status']
    require(type(parent) is str and bool(parent), 'original data parent status')
    verdict = 'UNOBSERVED'
    if diagnostic:
        require(source['diagnostic_authority'] in ('OBSERVED', 'UNOBSERVED'), 'typed diagnostic authority')
        if source['diagnostic_authority'] == 'OBSERVED':
            verdict = 'PASS' if (fact.get('kind'), fact.get('stage'), fact.get('code')) == (
                'diagnostic', 'frontend' if target == 'av' else stage, code) else 'FAIL'
    elif name in VALUES:
        verdict = 'PASS' if fact.get('kind') == 'value' and type(fact.get('value')) is int and fact['value'] == int(rule['old_value']) else 'FAIL'
    elif fact.get('kind') == 'trap':
        # Native exit 70 alone cannot name a trap. T08 has the actual rt_fail(-10)
        # trace in its strict source admission; do not generalize this mapping.
        verdict = ('PASS' if fact.get('code') == 'av.trap.str-index' else 'FAIL') if target == 'av' else (
            'PASS' if type(fact.get('runtime_code')) is int and fact['runtime_code'] == -10
            and type(fact.get('exit')) is int and fact['exit'] == 70 else 'UNOBSERVED')
    else:
        verdict = 'FAIL'
    if source['judgement'] == 'UNOBSERVED':
        verdict = 'UNOBSERVED'
    if source['judgement'] == 'FAIL' or parent != 'PASS':
        verdict = 'FAIL'
    return dict(schema='ace.m1.legacy-data-observation/1', source_sha256=rule['source_sha256'],
                request_sha256=source['request_sha256'], response_sha256=source['response_sha256'],
                expected=dict(kind=rule['kind'], value=rule['old_value']), observed=fact,
                source_judgement=source['judgement'], parent_status=parent, judgement=verdict,
                source_observation='q4-source/original-record-text/' + name + '/' + ('AV' if target == 'av' else 'AC-native'))


def admit(inputs, data, diagnostics):
    required, rules = contracts(inputs)
    observations, pending = {}, {}
    for report, names, schema in ((data, VALUES + ('T08',), 'ace.m1.original-data-source-observations/1'),
                                  (diagnostics, tuple(DIAGNOSTICS), 'ace.m1.original-data-diagnostics/1')):
        if report is None:
            continue
        typed_equal(report['schema'], schema, 'strict source report domain')
        for name in names:
            for target, label in (('ac', 'AC-native'), ('av', 'AV')):
                slot = 'q4-source/original-record-text/' + name + '/' + label
                require(slot in report['observations'], 'complete admitted original data source domain')
                item = observation(name, target, rules[name], report['observations'][slot])
                key = 'corpus:' + name + ':' + target
                (pending if item['judgement'] == 'UNOBSERVED' else observations)[key] = item
    return dict(required=required, observations=observations, pending_observations=pending)
