"""Read tracked discovery mappings, keeping execution and mapping gaps explicit."""
import csv
import io
import tomllib

from ace_m1_arch_acceptance import AdmissionError, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

BASE = 'crates/auto-lang/test/cases/ace-m1-expanded/'
CASE_CATALOG = BASE + 'case-obligations.tsv'
SCALAR_CATALOG = BASE + 'scalar-obligations.tsv'
Q4_CATALOG = BASE + 'q4-source-contracts.tsv'
Q4_CATALOG_SHA = '71b889228dee5074c81494bad1df5deb525dca2d9c670cf662dfce45ebfdf170'
Q4_FIELDS = 'family case profile caps source_file source_sha256 oracle_kind oracle_value marks owner origin source_status required_targets execution'.split()
Q4_TARGETS = ['AC-native','AV','ACE-A2R-HIR','ACE-A2C-HIR']
COMPOSITE_CATALOG = BASE + 'composite-obligations.tsv'
COMPOSITE_CATALOG_SHA = 'da392a3fc69239babd61f3fcd2c6aaf2cef8daa7c2ba47edda66a65504a63687'
COMPOSITE_SOURCES = {'experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs': '642b51d5fa365ea426aed00fc2460591aa5974a6ca77036505f13e110cf92ad2', 'experimental/ace/crates/a2x-worker/tests/cases/a2c_rt.rs': 'ee3288ff6d0f2c425fb21173ab7d4b7801034f34b87f7c212212d4fb12c3476d', 'experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_cert.rs': '43a9fdfae88925888d4ce9335bd57d1c7415df3c02ce14e8efa04bf90e60d81d'}
COMPOSITE_FIELDS = 'family case profile caps source_file source_sha256 oracle_kind oracle_value reference_file reference_sha256 reference_status owner formal_group execution'.split()
SCALAR_TABLE = 'experimental/ace/crates/a2x-worker/tests/cases/scalar_semantics.rs'
SCALAR_TABLE_SHA = '9007cdd1ecfbac5b9fcd73e7629983bb2ff35fac1fff8a63a740fec6079861b3'
CASE_CATALOG_SHA = 'ae3ad83a4af30279d0791133b05c396590165c4326ce461e50dbf73134370ee7'
SCALAR_CATALOG_SHA = '403819b35df9ca3b02b67d41e10fe46d6ff4115820eab70299d7ae6b2b9e7552'
CASE_FIELDS = 'group suite case check case_file case_file_sha256 contract_sha256 effective_check_sha256 source_sha256 oracle_sha256 adapter action recipe grammar declared_ac_budget state skip_owner skip_reason execution evidence_owner mechanism_or_probe_budget group_suite suite_sha256 snapshot_inputs'.split()
SCALAR_FIELDS = 'case profile source_sha256 oracle_kind oracle_value marks trace rust_debug_runs rust_release_runs c_runs rust_entry c_entry owner formal_group budget_source source_file'.split()


def read_tsv(scope, path, fields):
    raw = scope.read_bytes(path)
    scope.reserve(len(raw) * 6 + 4096)
    reader = csv.DictReader(io.StringIO(raw.decode('utf-8', errors='strict')), delimiter='\t', strict=True)
    typed_equal(reader.fieldnames, fields, 'tracked catalog columns')
    try:
        rows = list(reader)
    except csv.Error as error:
        raise AdmissionError('invalid tracked TSV catalog') from error
    for row in rows:
        exact_keys(row, fields, 'tracked catalog row')
        require(all(type(value) is str for value in row.values()), 'malformed catalog field')
    return rows


def admit_case_catalog(inputs, rows):
    require(len(rows) == 511, 'original formal check denominator changed')
    config = tomllib.loads(inputs.read_bytes('scripts/test-groups.toml').decode('utf-8'))
    groups = {group['name']:group for group in config['group']}
    identities, suites, manual = set(), set(), set()
    verified_files = {}
    for row in rows:
        key = (row['suite'], row['case'], row['check'])
        require(key not in identities, 'duplicate formal case/check obligation')
        identities.add(key)
        suites.add(row['suite'])
        if row['group'] == 'unregistered-a2r-hir-892':
            typed_equal(row['group_suite'], 'crates/auto-lang/test/cases/a2r-hir-892/a2r-hir-892.suite.ad', 'named original manual suite')
            manual.add(row['group_suite'])
        else:
            require(row['group'] in groups and groups[row['group']]['runner'] == 'case', 'missing original formal group')
            typed_equal(groups[row['group']]['suite'], row['group_suite'], 'formal group suite input')
        require(row['execution'] == 'UNOBSERVED', 'discovery mapping cannot claim execution')
        require(row['evidence_owner'] == 'PLAN-941' and row['state'] in ('RUN', 'SKIP'), 'invalid discovery owner/state')
        for name in ('case_file_sha256', 'contract_sha256', 'effective_check_sha256', 'source_sha256', 'oracle_sha256'):
            hash_value(row[name], 'formal mapping ' + name)
        path, digest = row['case_file'], row['case_file_sha256']
        if path in verified_files:
            typed_equal(digest, verified_files[path], 'one current case-file identity')
        else:
            inputs.sha256(path, digest)
            verified_files[path] = digest
        suite_path = row['group_suite']
        if suite_path in verified_files:
            typed_equal(row['suite_sha256'], verified_files[suite_path], 'one current suite identity')
        else:
            inputs.sha256(suite_path, row['suite_sha256'])
            verified_files[suite_path] = row['suite_sha256']
        snapshots, _ = inputs.decode_json(row['snapshot_inputs'].encode('utf-8'))
        require(type(snapshots) is list, 'case snapshot inputs must be a list')
        names = set()
        for ref in snapshots:
            exact_keys(ref, ('path', 'sha256'), 'case snapshot input')
            require(type(ref['path']) is str and ref['path'] not in names, 'duplicate case snapshot input')
            names.add(ref['path'])
            hash_value(ref['sha256'], 'snapshot input SHA256')
            if ref['path'] in verified_files:
                typed_equal(ref['sha256'], verified_files[ref['path']], 'shared current input identity')
            else:
                inputs.sha256(ref['path'], ref['sha256'])
                verified_files[ref['path']] = ref['sha256']
    require(len(suites) == 35, 'original formal suite denominator changed')
    return {'suites': len(suites), 'checks': len(rows), 'current_source_documents': len(verified_files),
            'manual_suites': sorted(manual), 'manual_owner':'PLAN-941 explicit original auto-case invocation',
            'execution': 'UNOBSERVED'}


def admit_scalar_catalog(inputs, rows):
    inputs.sha256(SCALAR_TABLE, SCALAR_TABLE_SHA)
    require(len(rows) == 61 and len({row['case'] for row in rows}) == 61, 'original scalar denominator changed')
    profiles, kinds = {}, {}
    rust_runs = c_runs = 0
    for row in rows:
        hash_value(row['source_sha256'], 'scalar mapping source')
        typed_equal(row['source_file'], BASE+'scalar-src/'+row['case']+'.at', 'original scalar fixture path')
        inputs.sha256(row['source_file'], row['source_sha256'])
        require(row['profile'] in ('core', 'scalar') and row['oracle_kind'] in ('value', 'trap'), 'unknown scalar contract')
        require(row['trace'] in ('true', 'false') and row['rust_debug_runs'] == row['c_runs'] == '1'
                and row['rust_release_runs'] in ('0', '1'), 'original scalar execution configuration changed')
        typed_equal(row['rust_entry'], 'ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain', 'original Rust corpus entry')
        typed_equal(row['c_entry'], 'ace-a2x-worker::conformance::a2c_scalar_semantics::c_full_corpus_true_chain', 'original C corpus entry')
        typed_equal(row['formal_group'], 'ace-a2x', 'original scalar group')
        if row['oracle_kind'] == 'value':
            value = row['oracle_value']
            require(len(value) <= 11, 'invalid scalar oracle size')
            parsed = int(value)
            require(str(parsed) == value and -(1 << 31) <= parsed < (1 << 31), 'invalid scalar i32 oracle')
        else:
            require(row['oracle_value'] in ('Overflow', 'DivByZero', 'DivOverflow', 'RemOverflow', 'RemByZero'), 'unknown scalar trap')
        profiles[row['profile']] = profiles.get(row['profile'], 0) + 1
        kinds[row['oracle_kind']] = kinds.get(row['oracle_kind'], 0) + 1
        rust_runs += 1 + int(row['rust_release_runs'])
        c_runs += 1
    typed_equal(profiles, {'core':40, 'scalar':21}, 'original scalar profiles')
    typed_equal(kinds, {'value':53, 'trap':8}, 'original scalar oracle families')
    require(rust_runs == 69 and c_runs == 61, 'original scalar configuration denominator changed')
    return {'cases':61, 'profiles':profiles, 'oracles':kinds, 'declared_rust_runs':rust_runs,
            'declared_c_runs':c_runs, 'execution':'UNOBSERVED'}


def admit_composite_catalog(inputs, rows):
    for path, digest in COMPOSITE_SOURCES.items():
        inputs.sha256(path, digest)
    require(len(rows) == 41, 'original composite supplement denominator changed')
    seen, families, kinds, missing_references = set(), {}, {}, []
    for row in rows:
        key = (row['family'], row['case'])
        require(key not in seen, 'duplicate original composite source/oracle')
        seen.add(key)
        require(row['family'] in ('record-text-cert-925', 'text-char-925', 'collections-926'), 'unknown composite family')
        require(row['profile'] in ('record', 'text', 'text-record', 'collections'), 'unknown composite profile')
        name = row['family'] + '--' + row['case']
        typed_equal(row['source_file'], BASE + 'composite-src/' + name + '.at', 'original composite fixture path')
        inputs.sha256(row['source_file'], row['source_sha256'])
        caps, _ = inputs.decode_json(row['caps'].encode())
        require(type(caps) is list and all(type(cap) is str and cap in ('ac.runtime.text.v1', 'hir.test.trace.v1')
                for cap in caps) and len(set(caps)) == len(caps), 'unknown/duplicate composite capability')
        if row['oracle_kind'] == 'value':
            value = row['oracle_value']
            require(len(value) <= 11 and str(int(value)) == value and -(1 << 31) <= int(value) < (1 << 31), 'composite value oracle')
        else:
            require(row['oracle_kind'] == 'trap' and row['oracle_value'] in ('ListIndex', 'TextIndex', 'InvalidScalar'), 'composite trap oracle')
        if row['reference_file']:
            typed_equal(row['reference_file'], BASE + 'composite-reference/' + name + '.c', 'independent C reference path')
            typed_equal(row['reference_status'], 'independent-handwritten-c', 'original reference identity')
            inputs.sha256(row['reference_file'], row['reference_sha256'])
        else:
            require(row['reference_sha256'] == '' and row['reference_status'] == 'original-test-declares-no-reference'
                    and row['family'] == 'text-char-925', 'missing reference cannot masquerade as certification')
            missing_references.append('/'.join(key))
        typed_equal(row['formal_group'], 'ace-a2x', 'original composite responsibility group')
        typed_equal(row['owner'], 'PLAN-941 evidence; PLAN-925/926 capability', 'composite ownership')
        typed_equal(row['execution'], 'UNOBSERVED', 'discovery cannot claim composite execution')
        families[row['family']] = families.get(row['family'], 0) + 1
        kinds[row['oracle_kind']] = kinds.get(row['oracle_kind'], 0) + 1
    typed_equal(families, {'record-text-cert-925':11, 'text-char-925':10, 'collections-926':20}, 'original complete supplement families')
    typed_equal(kinds, {'value':34, 'trap':7}, 'original composite oracle axes')
    require(len(missing_references) == 10, 'original reference availability changed')
    return {'cases':41, 'families':families, 'oracles':kinds, 'independent_c_references':31,
            'reference_gaps':missing_references, 'reference_gap_owner':'PLAN-941',
            'execution':'UNOBSERVED', 'executed':0}


def admit_q4_catalog(inputs, rows):
    require(len(rows) == 126, 'declared Q4 source denominator changed')
    seen, families, pending = set(), {}, []
    for row in rows:
        key = (row['family'], row['case'])
        require(key not in seen, 'duplicate Q4 source contract')
        seen.add(key)
        targets, _ = inputs.decode_json(row['required_targets'].encode())
        typed_equal(targets, Q4_TARGETS, 'four required targets cannot follow capability intersection')
        typed_equal(row['execution'], 'UNOBSERVED', 'source registration is not execution')
        require(bool(row['owner']) and bool(row['origin']), 'Q4 source owner/origin required')
        marks, _ = inputs.decode_json(row['marks'].encode())
        require(type(marks) is list and all(x in ('mark_a','mark_b') for x in marks), 'original trace oracle shape')
        typed_equal(row['source_status'], 'source-bound', 'Q4 source binding state')
        require(row['profile'] in ('core','scalar','record','text','text-record','collections'), 'unknown Q4 source profile')
        inputs.sha256(row['source_file'], row['source_sha256'])
        caps, _ = inputs.decode_json(row['caps'].encode())
        require(type(caps) is list and len(set(caps)) == len(caps) and all(type(x) is str and x in
                ('hir.test.trace.v1','ac.runtime.text.v1','ac.source.instances.scalar/v1',
                 'ac.source.cte.scalar/v1','cte.meta.fields/v1') for x in caps), 'Q4 capability contract')
        if row['family'] == 'source-cte':
            from ace_m1_cte_evidence import CT_CONTRACTS
            require(row['case'] in CT_CONTRACTS, 'unknown delivered CT case')
            name, value, extra = CT_CONTRACTS[row['case']]
            typed_equal(row['owner'], 'PLAN-936', 'CT source owner')
            typed_equal(row['profile'], 'scalar', 'CT source profile')
            typed_equal(caps, ['ac.source.cte.scalar/v1'] + extra, 'CT capabilities')
            typed_equal(row['source_file'], 'crates/auto-lang/test/cases/ace-m1/source-cte-936/src/' + name + '.at', 'CT actual source')
            typed_equal((row['oracle_kind'], row['oracle_value']), ('value', str(value)), 'delivered CT independent oracle')
        if row['oracle_kind'] == 'value':
            value = row['oracle_value']
            require(len(value) <= 11 and str(int(value)) == value and -(1 << 31) <= int(value) < (1 << 31), 'Q4 scalar value oracle')
        elif row['oracle_kind'] == 'trap':
            require(row['oracle_value'] in ('Overflow','DivByZero','DivOverflow','RemOverflow','RemByZero','TextIndex','InvalidScalar','ListIndex'), 'Q4 trap family')
        elif row['oracle_kind'] == 'diagnostic':
            require(row['family'] in ('source-query','source-instances') and row['oracle_value'] in
                    ('source.type-mismatch','type-mismatch'), 'original source diagnostic family')
        else:
            require(row['oracle_kind'] == 'pending-provider-oracle' and row['family'] == 'source-cte'
                    and row['case'] == 'bool' and row['oracle_value'] == '', 'unknown source oracle')
        families[row['family']] = families.get(row['family'], 0) + 1
    typed_equal(families, {'scalar':61,'record-text-cert-925':11,'text-char-925':10,'collections-926':20,
                           'record-text-base':5,'source-query':6,'source-instances':6,'source-cte':7}, 'declared Q4 source families')
    return {'sources':126, 'source_bound':126, 'pending_provider_sources':pending,
            'registered_required_target_cells':504, 'targets':Q4_TARGETS, 'families':families,
            'execution':'UNOBSERVED', 'executed':0, 'scope_closed':False,
            'note':'Source registration only; complete ARCH mapping and remaining original source variants are still pending.'}


def admit_catalogs(inputs):
    inputs.sha256(CASE_CATALOG, CASE_CATALOG_SHA)
    inputs.sha256(SCALAR_CATALOG, SCALAR_CATALOG_SHA)
    inputs.sha256(COMPOSITE_CATALOG, COMPOSITE_CATALOG_SHA)
    inputs.sha256(Q4_CATALOG, Q4_CATALOG_SHA)
    return {'formal_cases':admit_case_catalog(inputs, read_tsv(inputs, CASE_CATALOG, CASE_FIELDS)),
            'scalar':admit_scalar_catalog(inputs, read_tsv(inputs, SCALAR_CATALOG, SCALAR_FIELDS)),
            'composite':admit_composite_catalog(inputs, read_tsv(inputs, COMPOSITE_CATALOG, COMPOSITE_FIELDS)),
            'q4_sources':admit_q4_catalog(inputs, read_tsv(inputs, Q4_CATALOG, Q4_FIELDS)),
            'source_sha256':{CASE_CATALOG:CASE_CATALOG_SHA, SCALAR_CATALOG:SCALAR_CATALOG_SHA,
                             SCALAR_TABLE:SCALAR_TABLE_SHA, COMPOSITE_CATALOG:COMPOSITE_CATALOG_SHA, Q4_CATALOG:Q4_CATALOG_SHA} | COMPOSITE_SOURCES,
            'mapping_complete':False,
            'pending':['runtime budget observations, undeclared inner metrics and future provider limits',
                       'remaining ARCH obligations', 'old required-cell-to-execution closure']}
