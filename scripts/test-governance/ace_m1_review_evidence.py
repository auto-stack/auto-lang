"""PLAN-1005 (F-1005-02): pre-signature review-credential admission.

The single review-owned architecture row (ARCH-08/fresh-union-spec-review)
can ONLY be supplied by a pre-signature review credential R produced by an
actual independent review of the frozen candidate. This consumer verifies R's
structure, its binding to the frozen code commit, the pre-signature candidate
composition (a separate, already-terminal report whose facts must match the
current composition), the frozen row-rule/registry identities, the audited
coverage, and the acyclic reference shape. It never accepts a self-reported
PASS: the credential must bind the candidate's hash, and the FINAL report
references the credential — never the other way round.

Independence itself is procedural: the review stage records the real reviewer
identity in the shared Plan; a hash or a `reviewer=independent` string never
proves it (no PKI is built). Group PASS, implementation self-checks, entry
gates, mechanism families, and section injection can never close this row.
"""
from ace_m1_arch_acceptance import exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

REVIEW_ROW = 'ARCH-08/fresh-union-spec-review'
CREDENTIAL_FIELDS = ('schema', 'plan', 'plan_revision', 'frozen_commit', 'candidate',
                     'row_rules_sha256', 'registry_sha256', 'audited', 'reviewer',
                     'coverage', 'findings', 'verdict')
CANDIDATE_FIELDS = ('bindings_sha256', 'pre_signature_report_sha256')
COVERAGE_FIELDS = ('acceptance_criteria', 'verifications', 'spec_deltas')
PLAN = 'PLAN-1005'
REVISION = 1


def admit_review_credential(inputs, evidence, binding, composition, frozen_commits=()):
    """binding: {'credential': {'path','sha256'}, 'pre_signature_candidate': {'path','sha256'}}.

    `composition` is the current in-progress composition dict (must already
    carry the non-review architecture rows, admission sources, bindings sha).
    `frozen_commits` are the producer commits recorded by the composition's
    admitted evidence gates; the credential must bind one of them.
    Returns the credential admission supplying exactly the review row."""
    from ace_m1_arch_obligations import REQUIRED
    exact_keys(binding, ('credential', 'pre_signature_candidate'), 'review credential binding')
    candidate_ref = binding['pre_signature_candidate']
    exact_keys(candidate_ref, ('path', 'sha256'), 'pre-signature candidate reference')
    candidate, candidate_digest = evidence.read_json(candidate_ref['path'])
    if candidate_digest != candidate_ref['sha256']:
        raise ValueError(f'STALE artifact content: {candidate_ref["path"]}')

    # The pre-signature candidate is terminal and unsigned: every non-review
    # architecture row it reports must equal the current composition's rows.
    require(isinstance(candidate, dict), 'candidate must be an object')
    candidate_arch = candidate.get('architecture_execution')
    require(isinstance(candidate_arch, dict), 'candidate architecture section required')
    candidate_rows = {row['id']: row['verdict'] for row in candidate_arch.get('gate', {}).get('rows', [])}
    current_rows = {row['id']: row['verdict'] for row in composition['architecture_execution']['gate']['rows']}
    require(REVIEW_ROW in candidate_rows and candidate_rows[REVIEW_ROW] == 'UNOBSERVED',
            'pre-signature candidate must leave the review row unobserved')
    for key, verdict in candidate_rows.items():
        if key != REVIEW_ROW:
            typed_equal(current_rows.get(key), verdict, f'candidate row drift: {key}')
    require(candidate.get('can_claim_expanded_m1') is False,
            'pre-signature candidate is never signable on its own')
    typed_equal(candidate.get('admission_sources'), composition['admission_sources'],
                'candidate admission-source identity')

    ref = binding['credential']
    exact_keys(ref, ('path', 'sha256'), 'review credential reference')
    credential, digest = evidence.read_json(ref['path'])
    if digest != ref['sha256']:
        raise ValueError(f'STALE artifact content: {ref["path"]}')
    exact_keys(credential, CREDENTIAL_FIELDS, 'pre-signature review credential')
    typed_equal(credential['schema'], 'ace.m1.pre-signature-review-credential/1', 'credential schema')
    typed_equal(credential['plan'], PLAN, 'credential plan identity')
    require(type(credential['plan_revision']) is int and credential['plan_revision'] == REVISION,
            'credential plan revision')
    frozen = credential['frozen_commit']
    require(type(frozen) is str and len(frozen) == 40 and all(c in '0123456789abcdef' for c in frozen),
            'credential frozen commit')
    require(frozen in frozen_commits,
            'credential frozen commit must be the code the admitted evidence was captured on')

    candidate_block = credential['candidate']
    exact_keys(candidate_block, CANDIDATE_FIELDS, 'credential candidate binding')
    hash_value(candidate_block['bindings_sha256'], 'credential bindings sha')
    hash_value(candidate_block['pre_signature_report_sha256'], 'credential candidate sha')
    # The pre-signature candidate was composed WITHOUT the credential binding;
    # its bindings identity binds the credential verbatim from the candidate
    # document — never the final composition's necessarily-larger bindings
    # document. Acyclic by construction (§5.3): R binds the candidate, the
    # final report references R's own hash.
    typed_equal(candidate.get('parent_bindings_sha256'), candidate_block['bindings_sha256'],
                'credential binds the actual pre-signature bindings')
    typed_equal(candidate_block['pre_signature_report_sha256'], candidate_digest,
                'credential binds the actual pre-signature candidate')

    hash_value(credential['row_rules_sha256'], 'credential row-rules sha')
    hash_value(credential['registry_sha256'], 'credential registry sha')
    from ace_m1_arch_entry_gates import RECIPES as ROW_RULES, RECIPES_SHA
    typed_equal(credential['row_rules_sha256'], RECIPES_SHA, 'credential binds the frozen row rules')
    import hashlib
    registry_sha = hashlib.sha256(inputs.read_bytes('crates/auto-lang/test/cases/ace-m1-expanded/registry.json')).hexdigest()
    typed_equal(credential['registry_sha256'], registry_sha, 'credential binds the current registry')

    audited = credential['audited']
    exact_keys(audited, ('arch_rows', 'registered_keys', 'excluded_row'), 'credential audited set')
    require(type(audited['arch_rows']) is int and audited['arch_rows'] == len(REQUIRED) - 1,
            'credential audited every non-review architecture row')
    require(type(audited['registered_keys']) is int and audited['registered_keys'] == 1006,
            'credential audited the full registered denominator')
    typed_equal(audited['excluded_row'], REVIEW_ROW, 'the single excluded row is the review row')

    reviewer = credential['reviewer']
    exact_keys(reviewer, ('role', 'context', 'method'), 'credential reviewer record')
    for field in ('role', 'context', 'method'):
        require(type(reviewer[field]) is str and reviewer[field].strip(), f'reviewer {field} required')
    typed_equal(reviewer['role'], 'independent-final-review', 'credential review role')

    coverage = credential['coverage']
    exact_keys(coverage, COVERAGE_FIELDS, 'credential coverage record')
    typed_equal(coverage['acceptance_criteria'], [f'AC-{index:02}' for index in range(1, 10)],
                'credential covers every acceptance criterion')
    typed_equal(coverage['verifications'], [f'V-{index:02}' for index in range(1, 14)],
                'credential covers every verification case')
    typed_equal(coverage['spec_deltas'], ['SD-01', 'SD-02', 'SD-03'], 'credential covers every spec delta')

    findings = credential['findings']
    require(type(findings) is list, 'credential findings required')
    for finding in findings:
        exact_keys(finding, ('id', 'disposition', 'note'), 'credential finding')
        require(type(finding['id']) is str and finding['id'], 'finding id')
        require(type(finding['disposition']) is str and finding['disposition'] in ('resolved', 'accepted-hold'),
                'finding disposition')
        require(type(finding['note']) is str and finding['note'], 'finding note')
    require(all(f['disposition'] == 'resolved' for f in findings),
            'unresolved findings keep the credential unsignable')

    typed_equal(credential['verdict'], 'pass', 'credential verdict')
    return {'schema': 'ace.m1.review-credential-admission/1',
            'credential_sha256': digest,
            'frozen_commit': frozen,
            'pre_signature_candidate_sha256': candidate_digest,
            'observations': {REVIEW_ROW: 'PASS'},
            'can_claim_architecture_pass': False,
            'note': 'Mechanical verification only; reviewer independence is the recorded review process, not a hash.'}
