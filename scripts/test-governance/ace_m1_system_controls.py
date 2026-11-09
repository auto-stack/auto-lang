#!/usr/bin/env python3
"""PLAN-941 T-10: the acceptance-system control-family admission.

The expanded acceptance system counts as delivered only when its own
failure-state control family really executed green inside an admitted gate
document. The named classes below pin the T-10 claims (V-14/V-15): the total
gate AND with named blockers, preview ineligibility, legacy exit-0/boolean
promotion refusals, forged green, empty-steps/fake-execution refusals,
declarations-never-prove-runtime, partial-coverage degradation, stale or
non-green document refusals, companion domain strictness, awaiting-D tamper
refusals, late/cancelled publication states, terminal/binary identity,
child-producer impersonation, infra-vs-language distinction and cancelled
meters. Controls prove the SYSTEM's rejection behavior only — they never
enter milestone evidence and never make can_claim true by themselves.
"""
from ace_m1_arch_acceptance import AdmissionError, exact_keys

GATE_SCHEMA = 'fixed-test-run/1'
GROUP = 'infra-ace-m1-arch'

# Named control classes → needles matched against the executed test names of
# the admitted governance gate document. Every class must match at least one
# really-executed green test; a missing class keeps the family incomplete.
CONTROL_MAP = (
    ('total-gate-named-blockers', 'test_isolated_fully_green_control_fires_and_names_every_blocker_class'),
    ('unknown-inputs-refused', 'test_reducer_rejects_unknown_missing_or_mistyped_inputs'),
    ('preview-never-claims', 'test_complete_bounded_slice_still_cannot_claim_milestone'),
    ('legacy-exit0-or-boolean-refused', 'test_cannot_promote_script_exit_or_boolean_to_claim'),
    ('forged-green-preserves-gaps', 'test_forged_legacy_green_never_closes_full_registry_or_preview'),
    ('empty-steps-fake-execution-refused', 'test_missing_duplicate_fake_execution_and_unknown_group_are_refused'),
    ('declarations-never-prove-runtime', 'test_real_meters_pass_but_declarations_keep_the_gate_open'),
    ('partial-coverage-degrades-row', 'test_dropping_a_mapped_test_degrades_only_its_row'),
    ('stale-or-nongreen-documents-refused', 'test_stale_foreign_or_nongreen_documents_are_rejected'),
    ('companion-domain-binding-strict', 'test_companion_bindings_merge_only_their_own_group_events'),
    ('awaiting-d-tamper-refused', 'test_universe_rejects_missing_tampered_or_foreign_awaiting_d_rows'),
    ('late-or-cancelled-publication-red', 'test_failed_publication_stale_repair_or_second_edit_is_red'),
    ('terminal-and-binary-identity-required', 'test_raw_terminal_and_actual_binary_identity_are_required_even_for_parent_pass'),
    ('child-impersonation-refused', 'test_native_child_producer_cannot_be_replaced_with_parent_or_foreign_test'),
    ('infra-vs-language-distinct', 'test_expected_language_diagnostic_is_distinct_from_infra'),
    ('cancelled-or-wrong-meter-refused', 'test_cancelled_and_wrong_type_meter_cannot_pass'),
)


def admit_system_controls(inputs, evidence, ref):
    """Admit the governance gate document by content hash, then verify the
    named control family really executed green inside it."""
    exact_keys(ref, ('path', 'sha256'), 'acceptance-system controls gate binding')
    evidence.sha256(ref['path'], ref['sha256'])
    document, _ = evidence.read_json(ref['path'])
    if document.get('schema') != GATE_SCHEMA or document.get('gate_passed') is not True:
        raise AdmissionError('acceptance-system controls gate document did not pass')
    results = document.get('results')
    if type(results) is not list:
        raise AdmissionError('acceptance-system controls gate document malformed')
    row = next((r for r in results if r.get('groups') == [GROUP]), None)
    if row is None:
        raise AdmissionError('acceptance-system controls gate document lacks the governance group')
    if row.get('outcome') != 'PASS' or type(row.get('executed')) is not int or row['executed'] <= 0:
        raise AdmissionError('acceptance-system controls group did not execute green')
    report = row.get('report')
    if not isinstance(report, dict):
        raise AdmissionError('acceptance-system controls group carries no python report')
    for key in ('schema', 'selected', 'executed', 'skipped', 'failures', 'errors'):
        if key not in report:
            raise AdmissionError('acceptance-system controls report malformed')
    selected = report['selected']
    if (type(selected) is not list or not selected
            or report['executed'] != len(selected) or row['executed'] != len(selected)
            or report['skipped'] or report['failures'] or report['errors']):
        raise AdmissionError('acceptance-system controls did not execute every selected test green')
    controls, missing = [], []
    for name, needle in CONTROL_MAP:
        matched = sorted(test for test in selected if needle in test)
        if matched:
            controls.append({'class': name, 'tests': matched})
        else:
            missing.append(name)
    return {'schema': 'ace.m1.acceptance-system-controls/1',
            'controls': controls,
            'gate': {'controls_complete': not missing},
            'missing': missing,
            'executed': len(selected),
            'note': 'Controls prove the acceptance system rejects each failure '
                    'state; they never supply milestone evidence.'}
