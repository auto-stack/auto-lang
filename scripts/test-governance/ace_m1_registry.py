"""Index registered obligations without pretending the full universe is closed."""
import hashlib

from ace_m1_arch_acceptance import exact_keys
from ace_m1_arch_obligations import CONTRACT as ARCH, CONTRACT_SHA as ARCH_SHA, REQUIRED as ARCH_REQUIRED
from ace_m1_budget_contracts import CONTRACT as BUDGET, CONTRACT_SHA as BUDGET_SHA
from ace_m1_case_evidence import require
from ace_m1_legacy import frozen_ledger, typed_equal
from ace_m1_legacy_obligations import MAP, MAP_SHA
from ace_m1_legacy_base import CONTRACT as LEGACY_BASE,CONTRACT_SHA as LEGACY_BASE_SHA
from ace_m1_obligation_catalog import Q4_CATALOG, Q4_CATALOG_SHA, Q4_FIELDS, Q4_TARGETS, read_tsv
from ace_m1_text_evidence import CONTRACT as TEXT_REFERENCE, CONTRACT_SHA as TEXT_REFERENCE_SHA
from ace_m1_text_q4_evidence import CONTRACT as TEXT_Q4, CONTRACT_SHA as TEXT_Q4_SHA
from ace_m1_cte_evidence import REFERENCE_CONTRACT as CT_REFERENCE, REFERENCE_SHA as CT_REFERENCE_SHA
from ace_m1_scalar_q4_evidence import CONTRACT as SCALAR_Q4, CONTRACT_SHA as SCALAR_Q4_SHA
from ace_m1_scalar_cert_evidence import CONTRACT as SCALAR_REFERENCE, CONTRACT_SHA as SCALAR_REFERENCE_SHA
from ace_m1_record_text_evidence import CONTRACT as RT_Q4, CONTRACT_SHA as RT_Q4_SHA, REFERENCE as RT_REFERENCE, REFERENCE_SHA as RT_REFERENCE_SHA

from ace_m1_collections_evidence import CONTRACT as COLL_Q4, CONTRACT_SHA as COLL_Q4_SHA, REFERENCE as COLL_REFERENCE, REFERENCE_SHA as COLL_REFERENCE_SHA

from ace_m1_query_evidence import CONTRACTS as QUERY_CONTRACTS, REFERENCE as QUERY_REFERENCE, REFERENCE_SHA as QUERY_REFERENCE_SHA

from ace_m1_source_supplement import CATALOG as SUPPLEMENT, CATALOG_SHA as SUPPLEMENT_SHA, CONTRACT as SUPPLEMENT_Q4, CONTRACT_SHA as SUPPLEMENT_Q4_SHA, source_contracts as supplement_contracts, REFERENCE as SUPPLEMENT_REFERENCE, REFERENCE_SHA as SUPPLEMENT_REFERENCE_SHA

from ace_m1_original_data_sources import CATALOG as ORIGINAL_DATA, CATALOG_SHA as ORIGINAL_DATA_SHA, source_contracts as original_data_contracts, CONTRACT as ORIGINAL_DATA_Q4, CONTRACT_SHA as ORIGINAL_DATA_Q4_SHA, REFERENCE as ORIGINAL_DATA_REFERENCE, REFERENCE_SHA as ORIGINAL_DATA_REFERENCE_SHA

from ace_m1_data_diagnostics import CONTRACT as DATA_DIAGNOSTICS, CONTRACT_SHA as DATA_DIAGNOSTICS_SHA

from ace_m1_ct_mechanisms import CONTRACT as CT_MECHANISMS, CONTRACT_SHA as CT_MECHANISMS_SHA
from ace_m1_ct_child import CONTRACT as CT_CHILD, CONTRACT_SHA as CT_CHILD_SHA
from ace_m1_ct_meta import CONTRACT as CT_META, CONTRACT_SHA as CT_META_SHA
from ace_m1_ct_restore import CONTRACT as CT_RESTORE, CONTRACT_SHA as CT_RESTORE_SHA
from ace_m1_sq_facts import CONTRACT as SQ_FACTS, CONTRACT_SHA as SQ_FACTS_SHA
from ace_m1_syntax_arch import CONTRACT as SYNTAX_ARCH, CONTRACT_SHA as SYNTAX_ARCH_SHA

from ace_m1_legacy_arch import CONTRACT as LEGACY_ARCH, CONTRACT_SHA as LEGACY_ARCH_SHA

CONTRACTS = {'legacy_arch':{'path':LEGACY_ARCH,'sha256':LEGACY_ARCH_SHA}, 'syntax_arch':{'path':SYNTAX_ARCH,'sha256':SYNTAX_ARCH_SHA}, 'sq_facts':{'path':SQ_FACTS,'sha256':SQ_FACTS_SHA}, 'ct_restore':{'path':CT_RESTORE,'sha256':CT_RESTORE_SHA}, 'ct_meta':{'path':CT_META,'sha256':CT_META_SHA}, 'ct_child':{'path':CT_CHILD,'sha256':CT_CHILD_SHA}, 'ct_mechanisms':{'path':CT_MECHANISMS,'sha256':CT_MECHANISMS_SHA}, 'legacy':{'path':MAP, 'sha256':MAP_SHA},
             'legacy_base':{'path':LEGACY_BASE,'sha256':LEGACY_BASE_SHA},
             'q4':{'path':Q4_CATALOG, 'sha256':Q4_CATALOG_SHA},
             'architecture':{'path':ARCH, 'sha256':ARCH_SHA},
             'budget':{'path':BUDGET, 'sha256':BUDGET_SHA},
             'text_reference':{'path':TEXT_REFERENCE, 'sha256':TEXT_REFERENCE_SHA},
             'cte_reference':{'path':CT_REFERENCE, 'sha256':CT_REFERENCE_SHA},
             'text_q4':{'path':TEXT_Q4, 'sha256':TEXT_Q4_SHA},
             'scalar_q4':{'path':SCALAR_Q4, 'sha256':SCALAR_Q4_SHA},
             'scalar_reference':{'path':SCALAR_REFERENCE, 'sha256':SCALAR_REFERENCE_SHA},
             'record_text_q4':{'path':RT_Q4, 'sha256':RT_Q4_SHA},
             'record_text_reference':{'path':RT_REFERENCE, 'sha256':RT_REFERENCE_SHA},
             'collections_q4':{'path':COLL_Q4,'sha256':COLL_Q4_SHA},
             'collections_reference':{'path':COLL_REFERENCE,'sha256':COLL_REFERENCE_SHA},
             'query_source_q4':{key:QUERY_CONTRACTS['source-query'][key] for key in ('path','sha256')},
             'instances_source_q4':{key:QUERY_CONTRACTS['source-instances'][key] for key in ('path','sha256')},
             'query_reference':{'path':QUERY_REFERENCE,'sha256':QUERY_REFERENCE_SHA},
             'source_supplement_catalog':{'path':SUPPLEMENT,'sha256':SUPPLEMENT_SHA},
             'source_supplement_q4':{'path':SUPPLEMENT_Q4,'sha256':SUPPLEMENT_Q4_SHA},
             'source_supplement_reference':{'path':SUPPLEMENT_REFERENCE,'sha256':SUPPLEMENT_REFERENCE_SHA},
             'original_data_sources':{'path':ORIGINAL_DATA,'sha256':ORIGINAL_DATA_SHA},
             'original_data_q4':{'path':ORIGINAL_DATA_Q4,'sha256':ORIGINAL_DATA_Q4_SHA},
             'original_data_reference':{'path':ORIGINAL_DATA_REFERENCE,'sha256':ORIGINAL_DATA_REFERENCE_SHA},
             'original_data_diagnostics':{'path':DATA_DIAGNOSTICS,'sha256':DATA_DIAGNOSTICS_SHA}}


def registered_keys(inputs, preview_required):
    old = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in frozen_ledger(inputs) if row['required'])
    q4 = read_tsv(inputs, Q4_CATALOG, Q4_FIELDS) + list(supplement_contracts(inputs).values()) + list(original_data_contracts(inputs).values())
    keys = (['legacy/' + key for key in old]
            + ['q4-source/' + row['family'] + '/' + row['case'] + '/' + target for row in q4 for target in Q4_TARGETS]
            + ['architecture/' + key for key in ARCH_REQUIRED]
            + ['preview/' + key for key in preview_required])
    require(len(keys) == len(set(keys)), 'duplicate registered obligation')
    return sorted(keys)


# PLAN-941 T-03: the closed required universe. Registered obligations are
# derived live from the pinned frozen contracts (registered_keys above); the
# only obligations outside those contracts are the explicitly-owned awaiting-D
# promises of PLAN-938/939 (work-stage acceptance criteria carried verbatim in
# the registry's awaiting_d section). Both plans are undelivered (938
# executing, 939 drafting): an awaiting-D row never carries a verdict, never
# discharges the old cert-gap or scalar reds, and closes only through its
# archived D + 941 consumption + its own strict reader admission.
_AWAITING_FIELDS = ('key', 'owner', 'promise', 'direction', 'close')
_DELIVERED_FIELDS = _AWAITING_FIELDS + ('delivered_commit',)
_AWAITING_OWNERS = ('PLAN-938', 'PLAN-939')
# Completeness of the promise surface itself (V-01: 漏一项即拒): the two
# undelivered plans' work-stage acceptance criteria are each fully enumerated;
# dropping, adding or re-owning any row refuses the universe.
_AWAITING_EXPECTED = {}  # PLAN-939 delivered (dfe5a1002, round 113)
_DELIVERED_EXPECTED = {'PLAN-938': 7, 'PLAN-939': 7}
_REGISTERED_DOMAINS = ('legacy', 'q4-source', 'architecture', 'preview')


def universe_rows(inputs, preview_required, awaiting, delivered):
    """Complete universe: registered keys plus the delivered-D rows (the
    archived PLAN-938 D received into the 941 tree) and the still-awaiting
    PLAN-939 promises. A delivered row keeps its promise/close surface and
    names its archived delivery commit; neither state carries a verdict."""
    # Round 113: every promise is delivered — the awaiting section may be empty.
    require(type(awaiting) is list, 'awaiting-D section must be a list')
    require(type(delivered) is list and delivered, 'delivered-D section must be a non-empty list')
    rows = [dict(key=key, domain=key.split('/', 1)[0], state='registered')
            for key in registered_keys(inputs, preview_required)]
    for row in awaiting:
        exact_keys(row, _AWAITING_FIELDS, 'awaiting-D obligation')
        for field in _AWAITING_FIELDS:
            require(type(row[field]) is str and row[field].strip(), f'awaiting-D {field} must be a non-empty string')
        require(row['key'].startswith('awaiting-d/'), 'awaiting-D key namespace')
        require(row['owner'] in _AWAITING_OWNERS, 'awaiting-D owner plan')
        rows.append(dict(key=row['key'], domain='awaiting-d', state='awaiting-d', owner=row['owner'],
                         promise=row['promise'], direction=row['direction'], close=row['close']))
    for row in delivered:
        exact_keys(row, _DELIVERED_FIELDS, 'delivered-D obligation')
        for field in _DELIVERED_FIELDS:
            require(type(row[field]) is str and row[field].strip(), f'delivered-D {field} must be a non-empty string')
        require(row['key'].startswith('awaiting-d/'), 'delivered-D key namespace')
        require(row['owner'] in _AWAITING_OWNERS, 'delivered-D owner plan')
        rows.append(dict(key=row['key'], domain='awaiting-d', state='delivered-d', owner=row['owner'],
                         promise=row['promise'], direction=row['direction'], close=row['close'],
                         delivered_commit=row['delivered_commit']))
    require(len(rows) == len({row['key'] for row in rows}), 'duplicate universe obligation')
    foreign = {row['domain'] for row in rows} - set(_REGISTERED_DOMAINS) - {'awaiting-d'}
    require(not foreign, 'foreign universe domain')
    owners, delivered_owners = {}, {}
    for row in rows:
        if row['state'] == 'awaiting-d':
            owners[row['owner']] = owners.get(row['owner'], 0) + 1
        if row['state'] == 'delivered-d':
            delivered_owners[row['owner']] = delivered_owners.get(row['owner'], 0) + 1
    require(owners == _AWAITING_EXPECTED, 'awaiting-D promise surface incomplete')
    require(delivered_owners == _DELIVERED_EXPECTED, 'delivered-D surface incomplete')
    return rows


def universe_gate(rows):
    """Universe closure: complete enumeration with an explicit owner for every
    obligation. Evidence completeness and the M1 claim stay separate gates —
    closure never marks an awaiting-D promise observed or delivered."""
    closed = bool(rows) and all(
        row['state'] in ('registered', 'awaiting-d', 'delivered-d')
        and (row['state'] == 'registered' or (row.get('owner') and row.get('close')))
        and (row['state'] != 'delivered-d' or row.get('delivered_commit'))
        for row in rows)
    owners, delivered_owners = {}, {}
    for row in rows:
        if row['state'] == 'awaiting-d':
            owners[row['owner']] = owners.get(row['owner'], 0) + 1
        if row['state'] == 'delivered-d':
            delivered_owners[row['owner']] = delivered_owners.get(row['owner'], 0) + 1
    return {'universe_count': len(rows),
            'registered': sum(row['state'] == 'registered' for row in rows),
            'awaiting_d': sum(row['state'] == 'awaiting-d' for row in rows),
            'delivered_d': sum(row['state'] == 'delivered-d' for row in rows),
            'awaiting_owners': owners,
            'delivered_owners': delivered_owners,
            'required_universe_closed': closed,
            'note': 'Closure enumerates every obligation with an explicit owner; '
                    'registered_observations_complete and can_claim_expanded_m1 remain separate gates.'}


def registered_gate(keys, observations, universe_closed=False):
    """Pure verdict reducer, called only after the owning strict readers.

    The reducer alone never closes the universe (an isolated all-PASS control
    stays open); closure flows only from the composed universe admission."""
    require(type(observations) is dict and observations.keys() <= set(keys), 'foreign registered observation')
    require(all(type(value) is str and value in ('PASS', 'FAIL', 'UNOBSERVED')
                for value in observations.values()), 'invalid registered verdict')
    rows = [{'id':key, 'verdict':observations.get(key, 'UNOBSERVED')} for key in keys]
    counts = {state:sum(row['verdict'] == state for row in rows) for state in ('PASS', 'FAIL', 'UNOBSERVED')}
    # Open scope is an independent gate even for an isolated all-PASS control.
    return {'registered_rows':rows, 'registered_count':len(keys), 'counts':counts,
            'registered_observations_complete':bool(keys) and counts['PASS'] == len(keys),
            'required_universe_closed':universe_closed, 'complete':False}


def compose_registered_inventory(inputs, registry, report, preview_required):
    exact_keys(registry['expanded_contracts'], CONTRACTS, 'registered source contracts')
    typed_equal(registry['expanded_contracts'], CONTRACTS, 'registered source contract identities')
    for ref in CONTRACTS.values():
        inputs.sha256(ref['path'], ref['sha256'])
    typed_equal(registry['full_registry_closed'], True, 'closed full registry universe')
    keys = registered_keys(inputs, preview_required)
    observations = {'preview/' + row['id']:row['verdict'] for row in report['first_slice_rows']}
    # Only strict legacy reader observations supply old execution verdicts.
    for key, row in report['legacy'].get('execution_summary', {}).get('observations', {}).items():
        observations['legacy/' + key] = row['judgement']
    for row in report.get('architecture_execution', report['architecture_obligations'])['gate']['rows']:
        observations['architecture/' + row['id']] = row['verdict']
    # PLAN-941 T-06/T-04: the incremental/restore measurement section is the
    # execution-side source for the ARCH-05 registered obligations (its row
    # ids are exactly those keys). A PASS measurement upgrades the reducer's
    # placeholder UNOBSERVED verdict; PARTIAL or UNOBSERVED measurements keep
    # UNOBSERVED (real-but-incomplete coverage never claims the obligation)
    # and never downgrade an admitted mechanism-side PASS. The one genuine
    # conflict — a mechanism-side FAIL against a measurement claiming full
    # PASS coverage — is rejected rather than silently resolved. A foreign
    # measurement id reaches the registered gate's foreign-observation
    # rejection below.
    for row in report.get('incremental_restore_measurements', {}).get('rows', []):
        key = 'architecture/' + row['id']
        verdict = 'PASS' if row['verdict'] == 'PASS' else 'UNOBSERVED'
        prior = observations.get(key)
        require(prior != 'FAIL' or verdict != 'PASS',
                f'measurement verdict conflicts with admitted mechanism observation: {key}')
        if prior in (None, 'UNOBSERVED'):
            observations[key] = verdict
    for key, row in report.get('source_cte_execution', {}).get('observations', {}).items():
        require(key not in observations, 'duplicate registered execution')
        observations[key] = row['judgement']
    for key, row in report.get('text_q4_execution', {}).get('observations', {}).items():
        require(key not in observations, 'duplicate registered text source execution')
        observations[key] = row['judgement']
    for key, row in report.get('scalar_q4_execution', {}).get('observations', {}).items():
        require(key not in observations, 'duplicate registered scalar source execution')
        observations[key] = row['judgement']
    for key, row in report.get('text_certifications', {}).get('observations', {}).items():
        require(key not in observations, 'duplicate registered certification')
        observations[key] = row['judgement']
    for key, row in report.get('source_cte_certifications', {}).get('observations', {}).items():
        require(key not in observations, 'duplicate registered CT certification')
        observations[key] = row['judgement']
    for key, row in report.get('scalar_certifications', {}).get('observations', {}).items():
        require(key not in observations, 'duplicate registered scalar certification')
        observations[key] = row['judgement']
    for field in ('original_data_diagnostics','original_data_certifications','original_data_execution','source_supplement_certifications','source_supplement_execution','record_text_execution','record_text_certifications','collections_execution','collections_certifications','query_instances_execution','query_certifications','query_frontdoor_refusals'):
        for key,row in report.get(field,{}).get('observations',{}).items():
            require(key not in observations,'duplicate registered record/text observation')
            observations[key]=row['judgement']
    universe = universe_gate(universe_rows(inputs, preview_required, registry['awaiting_d'], registry['delivered_d']))
    return registered_gate(keys, observations, universe['required_universe_closed']) | {
        'schema':'ace.m1.registered-obligation-index/2',
        'registered_keys_sha256':hashlib.sha256(('\n'.join(keys) + '\n').encode()).hexdigest(),
        'contracts':CONTRACTS,
        'required_universe':universe,
        'note':'Registered index plus the closed awaiting-D universe; remaining original source variants and actual domain readers are required. Counts are not the final M1 denominator.'}
