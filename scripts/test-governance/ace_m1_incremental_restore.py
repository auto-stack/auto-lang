#!/usr/bin/env python3
"""PLAN-941 T-06: the incremental/restore measurement section.

Composes the ARCH-05 rows from execution evidence that is already strictly
admitted elsewhere: the frozen full-group gate document (per-test nextest
events, admitted by content hash) plus optional single-group companion
documents (ace-semantics carries the 934/935 facts/instance-page families,
ace-comptime carries the meta dependency-edit family) and the composed
CT-restore/architecture observation sections. Coverage is row-by-row honest
— a row is PASS only when every mapped test/observation really passed inside
the admitted documents; partial coverage names exactly what is missing; the
section never converts a partial map into a domain-gate pass and never
substitutes typed-bridge recovery for ordinary source recovery (V-05).
"""
from ace_m1_arch_acceptance import AdmissionError, exact_keys

GATE_SCHEMA = 'fixed-test-run/1'
GROUPS = ('ace-source', 'ace-syntax', 'infra-ace-m1-arch')

# Row map. `tests` entries are substrings of test names inside the admitted
# source-group result events (a family prefix covers every test of that
# family); `observations` are '<family>/<key>' references into the composed
# ct-restore section or the architecture gate rows; `partial_missing` names
# what still lacks evidence — its presence makes the row PARTIAL even when
# all mapped evidence passed.
ROWS = (
    {'id': 'ARCH-05/restore-immediate-good-edit',
     'tests': ('conformance$source_restore_929_t02::t02_first_lambda_edit_after_restore_exact_production',
               'conformance$source_restore_929_t02::t02_restore_chain_continues_across_edits',
               'conformance$source_positions_941::cold_lambda_structural_positions_equal_fresh',
               'conformance$source_positions_941::restored_first_lambda_structural_positions_equal_fresh'),
     'observations': (), 'partial_missing': ()},
    {'id': 'ARCH-05/bad-first-repair-second-restore',
     'tests': ('conformance$source_restore_929_t02::t02_first_edit_failure_zero_mutation_then_repair',
               'conformance$source_restore_929_t02::t02_restore_rejection_matrix',
               'conformance$source_positions_941::first_failure_repair_then_restore_uses_shifted_current_position'),
     'observations': (), 'partial_missing': ()},
    {'id': 'ARCH-05/producer-and-io-zero',
     'tests': ('conformance$source_units_928::',
               'conformance$source_units_928_t09::',
               'conformance$source_restore_929_t02::t02_restore_itself_is_zero_production',
               'conformance$source_restore_929_t02::t02_owner_edit_after_restore_same_text_lambda_zero_production',
               'conformance$source_dependency_edits_941::v04_callee_body_edit_caller_zero_io',
               'conformance$source_dependency_edits_941::v04_method_family_body_edits_zero_io',
               'conformance$source_dependency_edits_941::v04_closure_variable_length_subbody_edit_precise',
               'conformance$source_dependency_edits_941::v04_edits_keep_fresh_equivalence_and_restorable_identity',
               'meta_capability_941$revoked_capability_cannot_probe_but_regrant_hits_old_page_941'),
     'observations': (),
     'partial_missing': ()},
    {'id': 'ARCH-05/method-closure-body-edits',
     'tests': ('conformance$source_units_928::', 'conformance$source_units_928_t09::',
               'conformance$source_dependency_edits_941::v04_method_family_body_edits_zero_io',
               'conformance$source_dependency_edits_941::v04_closure_variable_length_subbody_edit_precise',
               'conformance$source_dependency_edits_941::v04_signature_edit_precise_invalidation_zero_io',
               'conformance$source_dependency_edits_941::v04_layout_decl_change_precise_dependents_zero_io',
               'meta_capability_941$valid_shape_edits_and_instance_child_still_execute_941'),
     'observations': (),
     'partial_missing': ()},
    {'id': 'ARCH-05/signature-layout-meta',
     'tests': ('conformance$source_dependency_edits_941::v04_signature_edit_precise_invalidation_zero_io',
               'conformance$source_dependency_edits_941::v04_layout_decl_change_precise_dependents_zero_io',
               'meta_capability_941$revoked_capability_cannot_probe_but_regrant_hits_old_page_941',
               'meta_capability_941$valid_shape_edits_and_instance_child_still_execute_941',
               'meta_capability_941$nested_and_callee_demands_require_current_capability_941'),
     'observations': (), 'partial_missing': ()},
    {'id': 'ARCH-05/all-required-pages',
     'tests': ('ace-source-query::restore_seam_929$facts_restore_lane_runs_on_the_929_restore_seam',
               'ace-source-query::restore_seam_929$facts_repair_after_the_bad_edit_restores_a_clean_page',
               'ace-source-query::instances_restore_controls_935$instances_restore_pages_only_bind_and_first_edits',
               'ace-source-query::instances_restore_controls_935$anti_tampered_instance_page_type_fails_real_bind',
               'ace-source-query::instances_restore_controls_935$t07_corrupted_instance_page_refused',
               'ace-source-query::instances_restore_controls_935$anti_foreign_owner_page_refused',
               'ace-abi::conformance$scalar_bridge$bridge_restore_deep_verifies_without_producers',
               'ace-abi::conformance$scalar_bridge$bridge_restore_negative_controls',
               'ace-abi::conformance$source_foreign$source_foreign_provider_symbol_index_uses_shared_budget_before_rebind_and_repairs',
               'ace-abi::conformance$cache_turnaround$abi_cache_foreign_toolchain_generation_not_served'),
     'observations': ('ct_restore/ct/restore-missing-page-cold',
                      'ct_restore/ct/restore-first-good-edit'),
     'partial_missing': ('939 opaque resource/callback pages (awaiting D)',)},
    # Cross-project ownership discipline per page family: SQ fact pages
    # refuse foreign owners explicitly (restore_verify/bind, F-941-09); CT
    # pages are owner-scoped by construction (owner_logical enters the
    # ExecKey hash — a foreign package cannot address another owner's page,
    # t02_09); foreign pages are provider/generation-scoped (a foreign
    # toolchain generation is never served; persisted foreign HIR is
    # untrusted and re-verified against the current environment).
    {'id': 'ARCH-05/historical-cross-project-ownership',
     'tests': ('restore_identity_941$f94109_foreign_package_refused_before_fact_bind',
               'restore_identity_941$f94109_foreign_module_refused_before_fact_bind',
               'restore_identity_941$f94109_same_shape_foreign_owner_refused_before_fact_bind',
               'restore_identity_941$f94109_own_pages_and_source_position_drift_keep_fresh_hir',
               'sq_source_facts_934::f94109_av_foreign_package_refused_old_42_survives',
               'sq_source_facts_934::f94109_av_foreign_module_refused_old_42_survives',
               'sq_source_facts_934::f94109_av_same_shape_foreign_owner_refused_old_42_survives',
               't02_09_foreign_package_never_addresses_another_owners_page',
               'cache_turnaround::abi_cache_foreign_toolchain_generation_not_served',
               'persisted_foreign_hir_is_untrusted_and_wrong_signature_is_rejected'),
     'observations': ('architecture/ARCH-02/facts-before-hir',
                      'architecture/ARCH-02/solver-authority'),
     'partial_missing': ()},
    {'id': 'ARCH-05/atomic-publication-and-late',
     'tests': ('ace-source-query::instance_host_935$single_flight_dedup_and_atomic_install',
               'ace-source-query::instance_host_935$commit_revalidation_keeps_old_set',
               'ace-source-query::instance_host_935$aba_history_reuse_requires_equal_content',
               'conformance$source_units_928::t05_candidate_transaction_lifecycle',
               'conformance$source_units_928_t09::t09_stale_candidate_rejected_zero_mutation',
               'conformance$source_units_928_t09::t09_foreign_candidate_rejected_zero_mutation'),
     'observations': (),
     'partial_missing': ()},
)


def _admit_companion_group(evidence, ref, expected_group):
    """Admit an optional single-group companion gate document (semantics,
    comptime, abi or av) by content hash; returns its per-test events."""
    exact_keys(ref, ('path', 'sha256'), f'incremental-restore {expected_group} gate binding')
    evidence.sha256(ref['path'], ref['sha256'])
    # Per-document transient release: full-group documents (e.g. ace-av's
    # thousand-plus events) reserve their raw bytes only while being checked;
    # only the extracted events survive the admission.
    before = evidence.budget.reserved
    document, _ = evidence.read_json(ref['path'])
    try:
        if document.get('schema') != GATE_SCHEMA or document.get('gate_passed') is not True:
            raise AdmissionError(f'{expected_group} gate document did not pass')
        results = document.get('results')
        if type(results) is not list or len(results) != 1 or results[0].get('groups') != [expected_group]:
            raise AdmissionError(f'{expected_group} gate document groups mismatch')
        if results[0].get('outcome') != 'PASS' or type(results[0].get('executed')) is not int or results[0]['executed'] <= 0:
            raise AdmissionError(f'{expected_group} gate document did not execute green')
        events = results[0].get('results', {})
        if type(events) is not dict:
            raise AdmissionError(f'{expected_group} group carries no per-test events')
        for name, event in events.items():
            if type(name) is not str or type(event) is not dict or event.get('event') != 'ok':
                raise AdmissionError(f'malformed {expected_group} per-test event')
    finally:
        evidence.budget.release_transient(evidence.budget.reserved - before)
    return events


def admit_incremental_restore_measurements(inputs, evidence, ref, ct_restore, architecture_rows, semantics_ref=None, comptime_ref=None, abi_ref=None, av_ref=None):
    """Admit the gate document by content hash, then derive row coverage."""
    exact_keys(ref, ('path', 'sha256'), 'incremental-restore gate binding')
    evidence.sha256(ref['path'], ref['sha256'])
    document, _ = evidence.read_json(ref['path'])
    if document.get('schema') != GATE_SCHEMA:
        raise AdmissionError('unexpected gate document schema')
    if document.get('gate_passed') is not True:
        raise AdmissionError('incremental-restore gate document did not pass')
    results = document.get('results')
    if type(results) is not list or len(results) != len(GROUPS):
        raise AdmissionError('gate document group count mismatch')
    for result, group in zip(results, GROUPS):
        if result.get('groups') != [group]:
            raise AdmissionError('gate document groups do not match the frozen full groups')
        if result.get('outcome') != 'PASS' or type(result.get('executed')) is not int or result['executed'] <= 0:
            raise AdmissionError('gate document group did not execute green')
    events = results[0].get('results', {})
    if type(events) is not dict or not events:
        raise AdmissionError('source group carries no per-test events')
    for name, event in events.items():
        if type(name) is not str or type(event) is not dict or event.get('event') != 'ok':
            raise AdmissionError('malformed per-test event')
    if semantics_ref is not None:
        events = {**events, **_admit_companion_group(evidence, semantics_ref, 'ace-semantics')}
    if comptime_ref is not None:
        events = {**events, **_admit_companion_group(evidence, comptime_ref, 'ace-comptime')}
    if abi_ref is not None:
        events = {**events, **_admit_companion_group(evidence, abi_ref, 'ace-abi')}
    if av_ref is not None:
        events = {**events, **_admit_companion_group(evidence, av_ref, 'ace-av')}
    ct_verdicts = ct_restore.get('observations', {}) if isinstance(ct_restore, dict) else {}
    arch_verdicts = {row['id']: row['verdict'] for row in architecture_rows} if architecture_rows else {}
    rows = []
    for spec in ROWS:
        matched, missing_tests = [], []
        for needle in spec['tests']:
            hits = sorted(name for name in events if needle in name)
            if hits:
                matched.extend(hits)
            else:
                missing_tests.append(needle)
        observed, missing_observations = [], []
        for observation in spec['observations']:
            family, _, key = observation.partition('/')
            source = ct_verdicts if family == 'ct_restore' else arch_verdicts
            if source.get(key) == 'PASS':
                observed.append(observation)
            else:
                missing_observations.append(observation)
        covered = not missing_tests and not missing_observations
        verdict = 'PASS' if covered and not spec['partial_missing'] else (
            'PARTIAL' if covered or matched or observed else 'UNOBSERVED')
        rows.append({'id': spec['id'], 'verdict': verdict,
                     'evidence': {'tests': matched, 'observations': observed},
                     'missing': list(missing_tests) + list(missing_observations)
                                + list(spec['partial_missing'])})
    counts = {state: sum(row['verdict'] == state for row in rows)
              for state in ('PASS', 'PARTIAL', 'UNOBSERVED')}
    return {'schema': 'ace.m1.incremental-restore-measurements/1',
            'rows': rows,
            'gate': {'measurements_complete': counts['PASS'] == len(rows)},
            'counts': counts,
            'note': 'Row-by-row coverage from admitted execution evidence; typed-bridge '
                    'recovery never substitutes ordinary source recovery (V-05).'}
