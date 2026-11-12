"""Architecture source contracts; locators never authorize execution PASS."""
import tomllib

from ace_m1_arch_acceptance import exact_keys
from ace_m1_budget_contracts import POLICIES
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/architecture-obligations.json'
CONTRACT_SHA = 'e4449a5ab684fb175a763b1479415a6b15043f530d109b1d7aa5527d6f975afd'
SUFFIXES = (
    'all-grammars single-frontdoor nested-body-reuse source-position-freshness public-first-edit',
    'solver-authority facts-before-hir positive-negative-deps real-instances effective-implementation required-instance-pages',
    'all-declared-q4 source-order-and-traps shared-ir-boundary source-vs-typed certification-two-legs',
    'cold-real-av-and-hit callee-body-change meta-args-and-negative instance-child-ct-version frozen-hir-four-targets unsupported-and-resources',
    'producer-and-io-zero method-closure-body-edits signature-layout-meta restore-immediate-good-edit bad-first-repair-second-restore all-required-pages atomic-publication-and-late historical-cross-project-ownership',
    'resource-facts-authority every-allocation-fault-and-retry failed-source-mut-out-atomicity all-domain-budgets abi-original-three-directions d1-source-vs-typed d2-real-source-imports d3-resources-and-callback',
    'one-pid-one-entry last-owner-gc original-churn-and-samples',
    'approved-old-conservation old38-cert-separation strict-all-fields-and-paths original-execution-terminal daily-cert-axis required-and-no-empty cost-memory-and-phase fresh-union-spec-review')
REQUIRED = tuple(f'ARCH-{index:02}/{suffix}' for index, names in enumerate(SUFFIXES, 1) for suffix in names.split())
OBSERVATIONS = frozenset('budget counters facts identity input instances io lifetime oracle producer publication raw-terminal restore'.split())
PENDING = {'foreign':'PLAN-938', 'foreign-resource':'PLAN-939'}
ROW_FIELDS = ('id', 'arch', 'owner', 'evidence_owner', 'entries', 'budget_domains', 'verification_ids',
              'closing_oracle', 'negative_control', 'required_observations', 'execution')


def _names(value, allowed, label):
    require(type(value) is list and bool(value), label + ' list required')
    require(all(type(item) is str and item in allowed for item in value), 'unknown ' + label)
    require(len(set(value)) == len(value), 'duplicate ' + label)


def reduce_architecture(observations):
    """Pure AND for already admitted verdicts and isolated controls, never a reader."""
    require(type(observations) is dict and observations.keys() <= set(REQUIRED), 'foreign architecture obligation')
    require(all(type(value) is str and value in ('PASS', 'FAIL', 'UNOBSERVED')
                for value in observations.values()), 'invalid architecture verdict')
    rows = [{'id':key, 'verdict':observations.get(key, 'UNOBSERVED')} for key in REQUIRED]
    return {'required':len(rows), 'rows':rows, 'complete':all(row['verdict'] == 'PASS' for row in rows)}


def validate_contract(inputs, document):
    exact_keys(document, ('schema', 'plan', 'scope', 'entry_semantics', 'entries', 'obligations', 'execution'), 'architecture contract')
    typed_equal(document['schema'], 'ace.m1.arch-obligation-contract/1', 'architecture schema')
    typed_equal(document['plan'], 'PLAN-941:r1', 'architecture revision')
    typed_equal(document['execution'], 'UNOBSERVED', 'locator is not execution')
    for field in ('scope', 'entry_semantics'):
        require(type(document[field]) is str and bool(document[field]), 'architecture scope required')
    require(type(document['entries']) is dict and bool(document['entries']), 'architecture entries required')
    groups = {row['name']:row for row in tomllib.loads(inputs.read_bytes('scripts/test-groups.toml').decode())['group']}
    pending = {}
    for key, entry in document['entries'].items():
        require(type(entry) is dict and 'kind' in entry, 'architecture entry kind required')
        if entry['kind'] == 'provider-pending':
            exact_keys(entry, ('kind', 'owner', 'contract'), 'pending provider')
            require(key in PENDING, 'foreign pending provider')
            typed_equal(entry['owner'], PENDING[key], 'pending provider owner')
            require(type(entry['contract']) is str and bool(entry['contract']), 'pending provider contract')
            pending[key] = entry['owner']
            continue
        require(entry.get('group') in groups, 'unknown architecture group')
        group = groups[entry['group']]
        if entry['kind'] == 'case-suite':
            exact_keys(entry, ('kind', 'group', 'suite_file', 'source_sha256'), 'architecture Case entry')
            require(group['runner'] == 'case', 'architecture Case runner')
            typed_equal(entry['suite_file'], group['suite'], 'architecture formal suite')
            path = entry['suite_file']
        else:
            typed_equal(entry['kind'], 'native-module', 'architecture entry kind')
            exact_keys(entry, ('kind', 'group', 'binary', 'module', 'source_file', 'source_sha256'), 'architecture native entry')
            require(group['runner'] == 'nextest' and group['workspace'] == 'experimental/ace', 'architecture native runner')
            require(type(entry['binary']) is str and type(entry['module']) is str and bool(entry['module']), 'architecture native locator')
            package = entry['binary'].split('::')[0]
            args = group['args']
            require(any(args[i:i + 2] == ['-p', package] for i in range(len(args) - 1)), 'architecture package not in group')
            path = entry['source_file']
        inputs.sha256(path, entry['source_sha256'])
    typed_equal(pending, PENDING, 'complete pending providers')
    require(type(document['obligations']) is list, 'architecture rows required')
    seen, used, verifications = set(), set(), set()
    for row in document['obligations']:
        exact_keys(row, ROW_FIELDS, 'architecture obligation')
        key = row['id']
        require(type(key) is str and key in REQUIRED and key not in seen, 'foreign/duplicate architecture obligation')
        seen.add(key)
        typed_equal(row['arch'], key.split('/')[0], 'architecture family')
        typed_equal(row['evidence_owner'], 'PLAN-941', 'architecture evidence owner')
        typed_equal(row['execution'], 'UNOBSERVED', 'architecture declarations cannot supply PASS')
        for field in ('owner', 'closing_oracle', 'negative_control'):
            require(type(row[field]) is str and bool(row[field]), 'architecture oracle/owner required')
        _names(row['entries'], document['entries'], 'architecture entry reference')
        _names(row['budget_domains'], POLICIES, 'architecture budget domain')
        _names(row['verification_ids'], {f'V-{i:02}' for i in range(1, 19)}, 'architecture verification')
        _names(row['required_observations'], OBSERVATIONS, 'architecture observation kind')
        used.update(row['entries'])
        verifications.update(row['verification_ids'])
    typed_equal(seen, set(REQUIRED), 'complete architecture denominator')
    typed_equal(used, set(document['entries']), 'architecture entry closure')
    typed_equal(verifications, {f'V-{i:02}' for i in range(1, 19)}, 'architecture verification closure')
    return {'schema':'ace.m1.arch-obligation-discovery/1', 'entry_count':len(used), 'pending_providers':pending,
            'obligations':[{key:row[key] for key in ('id', 'owner', 'entries', 'budget_domains', 'verification_ids')}
                           for row in document['obligations']], 'execution':'UNOBSERVED', 'executed':0,
            'gate':reduce_architecture({}), 'can_claim_architecture_pass':False}


def admit_architecture_contract(inputs):
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    document, digest = inputs.read_json(CONTRACT)
    return validate_contract(inputs, document) | {'contract_sha256':digest}
