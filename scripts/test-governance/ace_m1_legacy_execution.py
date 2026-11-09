"""Original required-cell execution admission; accounting is never execution.

The first supported slice is the original B08/B11 native/AV requirements.
Other cells remain explicitly unobserved, including every old A2R cert gap.
"""
from ace_m1_arch_acceptance import exact_keys
from ace_m1_case_evidence import admit_case_bindings, admit_exchange, require
from ace_m1_legacy import CORPUS, frozen_ledger, typed_equal
from ace_m1_scalar_evidence import scalar_observation

WIDE_AV = {'W01':'i64-min', 'W02':'u64-max', 'W03':'neg-zero',
           'W04':'fneg-nan', 'W05':'f32-nan-payload'}


def execution_summary(required, observations):
    require(type(required) is list and len(required) == 313 and len(set(required)) == 313,
            'original required denominator must contain 313 unique cells')
    require(type(observations) is dict and observations.keys() <= set(required), 'foreign legacy execution cell')
    states = {key:value['judgement'] for key, value in observations.items()}
    require(all(value in ('PASS', 'FAIL') for value in states.values()), 'unknown legacy execution verdict')
    missing = sorted(set(required) - observations.keys())
    failed = sorted(key for key, verdict in states.items() if verdict == 'FAIL')
    return {'required':313, 'observed':len(states), 'passed':sum(v == 'PASS' for v in states.values()),
            'failed_cells':failed, 'unobserved_cells':missing,
            'complete':not missing and not failed,
            'verdict':'FAIL' if failed else 'UNOBSERVED' if missing else 'PASS',
            'observations':observations}


def admit_scalar_slice(copy, inventory, inputs, workers):
    ledger = frozen_ledger(inputs)
    required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in ledger if row['required'])
    manifest, _ = inputs.read_json(CORPUS + 'source-manifest.json')
    # Parse the approved data table, not Auto source or child echoes. Its hash
    # and complete shape have already been checked by frozen_ledger.
    rows = [line.split('\t') for line in inputs.read_bytes(CORPUS + 'binding.tsv').decode().splitlines()
            if line.strip() and not line.startswith('#')]
    oracles = {row[0]:row for row in rows if row[1] == 'corpus' and row[0] in ('B08', 'B11')}
    require(oracles.keys() == {'B08', 'B11'}, 'original B08/B11 contract missing')
    admitted = admit_case_bindings(copy, inventory)
    expected_keys = {('ace.m1.legacy941.' + name.lower(), check)
                     for name in ('B08', 'B11') for check in ('ac-native', 'av')}
    require({(check['case'], check['check']) for check, _ in admitted['checks']} == expected_keys,
            'original scalar slice denominator changed')
    observations = {}
    for check, row in admitted['checks']:
        name = 'B08' if check['case'] == 'ace.m1.legacy941.b08' else 'B11'
        target = 'ac' if check['check'] == 'ac-native' else 'av'
        cell = 'corpus:' + name + ':' + target
        require(cell in required, 'not an original required cell')
        contract = oracles[name]
        typed_equal(contract[3], 'value', 'original scalar oracle family')
        typed_equal(check['expected_exit'], int(contract[4]), 'original approved scalar oracle')
        typed_equal(check['completion'], {'kind':'success'}, 'original scalar completion')
        require(check['adapter'] == 'probe' and check['action'] == 'run' and check['state'] == 'RUN',
                'original scalar execution is not selected')
        digest = manifest['corpus_src']['files'][name + '.at']
        inputs.sha256(CORPUS + 'src/' + name + '.at', digest)
        require(len(check['source_files']) == 1, 'original scalar source required')
        typed_equal(check['source_files'][0]['sha256'], digest, 'exact original source bytes')
        recipe = 'ac/mir-native-scalar-v1' if target == 'ac' else 'ac-av/core-run-v1'
        protocol = 'mnworker/1' if target == 'ac' else 'avworker/1'
        typed_equal(check['recipe'], recipe, 'original required target identity')
        names = [name for name in copy.files if name.endswith('-start.json')
                 and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in name]
        require(len(names) == 1, 'one original scalar execution required')
        wire = admit_exchange(copy, names[0], protocol=protocol, recipe=recipe,
                              worker_sha256=workers['native' if target == 'ac' else 'av'])
        observation = scalar_observation(copy.scope, wire, check)
        observation['parent_status'] = row['status']
        # Child PASS is not positive authority; a parent rejection is retained.
        if row['status'] != 'PASS':
            observation['judgement'] = 'FAIL'
        observations[cell] = observation
    result = execution_summary(required, observations)
    result['report_sha256'] = admitted['report_sha256']
    return result


def admit_av_typed_slice(copy, inventory, inputs, workers, parent_snapshot):
    from ace_m1_av_typed_evidence import av_typed_observation
    ledger = frozen_ledger(inputs)
    required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in ledger if row['required'])
    table = parent_snapshot['parents']['av']['tables']['data_typed']
    typed_equal(table['columns'], ['fixture', 'kind', 'bits'], 'original typed oracle columns')
    oracles = {row[0]:dict(zip(table['columns'], row)) for row in table['rows']}
    require(len(oracles) == len(table['rows']) == 8, 'original typed oracle denominator changed')
    rows = [line.split('\t') for line in inputs.read_bytes(CORPUS + 'binding.tsv').decode().splitlines()
            if line.strip() and not line.startswith('#')]
    old = {row[0]:row for row in rows if row[1] == 'corpus' and row[0] in WIDE_AV}
    require(old.keys() == WIDE_AV.keys(), 'original wide typed obligations missing')
    for name, fixture in WIDE_AV.items():
        require('corpus:' + name + ':av' in required, 'AV typed cell not required')
        typed_equal(old[name][3], 'value', 'original typed oracle family')
        typed_equal(old[name][4], oracles[fixture]['bits'], 'original wide bit oracle')
    admitted = admit_case_bindings(copy, inventory)
    selected = [(check, row) for check, row in admitted['checks'] if check['case'] == 'av.data.typed']
    require({check['check'] for check, _ in selected} == {'typed-' + name for name in oracles}
            and len(selected) == 8, 'original AV typed checks missing/duplicated')
    typed_observations = {}
    for check, row in selected:
        fixture = check['check'][len('typed-'):]
        names = [name for name in copy.files if name.endswith('-start.json')
                 and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in name]
        require(len(names) == 1, 'one original typed exchange required')
        wire = admit_exchange(copy, names[0], protocol='avworker/1', recipe='ac-av/data-run-v1',
                              worker_sha256=workers['av'])
        result = av_typed_observation(copy, wire, check, oracles[fixture])
        result['parent_status'] = row['status']
        if row['status'] != 'PASS':
            result['judgement'] = 'FAIL'
        typed_observations[fixture] = result
    observations = {'corpus:' + name + ':av':typed_observations[fixture] for name, fixture in WIDE_AV.items()}
    result = execution_summary(required, observations)
    result['typed_controls'] = typed_observations
    result['source_case_report'] = {'report_sha256':admitted['report_sha256'], 'non_pass_rows':[
        {'case':check['case'], 'check':check['check'], 'status':row['status']}
        for check, row in admitted['checks'] if row['status'] != 'PASS']}
    return result


def admit_native_typed_slice(copy, inventory, inputs, workers):
    from ace_m1_native_typed_evidence import native_typed_observation
    required = sorted(':'.join((row['ns'], row['id'], row['path']))
                      for row in frozen_ledger(inputs) if row['required'])
    rows = [line.split('\t') for line in inputs.read_bytes(CORPUS + 'binding.tsv').decode().splitlines()
            if line.strip() and not line.startswith('#')]
    old = {row[0]:row for row in rows if row[1] == 'corpus' and row[0] in WIDE_AV}
    require(old.keys() == WIDE_AV.keys(), 'original native wide obligations missing')
    admitted = admit_case_bindings(copy, inventory)
    selected = [(check, row) for check, row in admitted['checks']
                if check['case'] in {'ace.m1.' + name.lower() + '-mn891' for name in WIDE_AV}]
    require(len(selected) == 5, 'original native wide checks missing')
    observations = {}
    for check, row in selected:
        name = check['case'].split('.')[2].split('-')[0].upper()
        cell = 'corpus:' + name + ':ac'
        require(cell in required and cell not in observations, 'unknown/duplicate native required cell')
        typed_equal(old[name][3], 'value', 'original native wide oracle family')
        typed_equal(check['typed_oracle']['bits'], old[name][4], 'original native wide bit oracle')
        names = [path for path in copy.files if path.endswith('-start.json')
                 and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in path]
        require(len(names) == 1, 'one original native typed exchange required')
        wire = admit_exchange(copy, names[0], protocol='mnworker/1', recipe='ac/mir-native-typed-v1',
                              worker_sha256=workers['native'])
        result = native_typed_observation(copy, wire, check)
        result['parent_status'] = row['status']
        if row['status'] != 'PASS':
            result['judgement'] = 'FAIL'
        observations[cell] = result
    result = execution_summary(required, observations)
    result['source_case_report'] = {'report_sha256':admitted['report_sha256'], 'non_pass_rows':[
        {'case':check['case'], 'check':check['check'], 'status':row['status']}
        for check, row in admitted['checks'] if row['status'] != 'PASS']}
    return result
