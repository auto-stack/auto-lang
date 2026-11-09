"""PLAN-1000 T-03: strict original a2r emit/snapshot/scene execution admission.

The Case `a2r` adapter is an in-process emitter (no worker exchange), so the
strict chain here is: the parent Case report's identity pins (source/oracle
content hashes re-verified by admit_case_bindings), plus the retained emitted
artifact bytes in the artifact copy, re-verified byte-for-byte against the
tracked golden oracle. Accounting stays a separate axis: this module never
reads a ledger state column as an execution verdict, and a check whose parent
row is not PASS can never produce a PASS observation.
"""
from ace_m1_arch_acceptance import AdmissionError, EvidenceScope, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from pathlib import PureWindowsPath, PurePosixPath

from ace_m1_legacy import CORPUS, frozen_ledger, typed_equal

# cell-key extraction per suite family: (case-id prefix, namespace hint)
_SCENE = 'ace.m1.a2rscene.'
_TRAP = 'ace.m1.a2rtrap.'
_TEXT = 'ace.m1.a2rt.'
_PLAIN = 'ace.m1.'
_HIR_RT = 'a2r-hir-record-text-902.'
_HIR_COLL = 'a2r-hir-collections-904.'
_OWNED = 'a2r-owned-temporaries-931.'
_SEM = 'ace.m1.a2r916.'
_EMIT = 'ace.m1.a2r917.'


def _cell_key(case_id, required_cells):
    """Map one Case id to at most one required legacy cell (ns, id, path=a2r)."""
    corpus_ids = {cell[1].lower(): cell for cell in required_cells if cell[0] == 'corpus'}
    scenario_ids = {cell[1].lower(): cell for cell in required_cells if cell[0] == 'scenario'}
    token = None
    if case_id.startswith(_SCENE):
        token = case_id[len(_SCENE):]
        # fixture families: r02_l04 -> family r02 when the whole token is not an id
        if token not in scenario_ids:
            token = token.split('_')[0]
        return scenario_ids.get(token)
    if case_id.startswith(_TRAP) or case_id.startswith(_TEXT):
        token = case_id.rsplit('.', 1)[-1]
        return corpus_ids.get(token)
    if case_id.startswith(_SEM) or case_id.startswith(_EMIT):
        # ace.m1.a2r916.b16.single / .project: the obligation id is the second
        # dot segment; single/project are two modes of the same family cell.
        # (Checked before _PLAIN, whose shorter prefix would swallow these.)
        parts = case_id.split('.')
        token = parts[3].lower() if len(parts) > 3 else parts[-1].lower()
        if token in corpus_ids:
            return corpus_ids[token]
        if token in scenario_ids:
            return scenario_ids[token]
        return None
    if case_id.startswith(_PLAIN):
        token = case_id[len(_PLAIN):]
        return corpus_ids.get(token)
    for prefix in (_HIR_RT, _HIR_COLL, _OWNED):
        if case_id.startswith(prefix):
            token = case_id[len(prefix):]
            return corpus_ids.get(token)
    if case_id.startswith(_SEM) or case_id.startswith(_EMIT):
        # ace.m1.a2r916.b16.single / .project: the obligation id is the second
        # dot segment; single/project are two modes of the same family cell.
        parts = case_id.split('.')
        token = parts[3].lower() if len(parts) > 3 else parts[-1].lower()
        if token in corpus_ids:
            return corpus_ids[token]
        if token in scenario_ids:
            return scenario_ids[token]
        return None
    return None


def admit_a2r_execution(inputs, evidence, bindings, registry):
    """Admit every bound a2r suite run and reduce per-cell observations.

    bindings: {suite_key: {'inventory': {'path','sha256'}, 'copy': <scope dir>}}
    registry: the expanded registry document carrying the frozen identity
    contracts for each suite key (same shape as the seven original suites).
    """
    from ace_m1_first_slice import _inventory_for_suite, _compact_admission
    ledger = frozen_ledger(inputs)
    required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in ledger if row['required'])
    required_cells = {(row['ns'], row['id'], row['path']) for row in ledger
                      if row['required'] and row['path'] == 'a2r'}
    manifest, _ = inputs.read_json(CORPUS + 'source-manifest.json')
    observations, parents, per_cell = {}, {}, {}

    def _admit_inventory(key, binding):
        """Same re-hash discipline as _inventory_for_suite, allowing the
        multi-file project sources the original seven suites never had."""
        from ace_m1_first_slice import inventory_contract
        ref = binding['inventory']
        exact_keys(ref, ('path', 'sha256'), f'a2r inventory reference {key}')
        evidence.sha256(ref['path'], ref['sha256'])
        inv, _ = evidence.read_json(ref['path'])
        require(inv.get('schema') == 'test-case-inventory/2', 'formal identity inventory required')
        fixture = inv['suite_path']
        if fixture.startswith('\\\\?\\'):
            fixture = fixture[4:]
        typed_equal(PureWindowsPath(fixture).relative_to(
            PureWindowsPath(str(inputs.root))).as_posix(), _suite_for(key),
            f'frozen suite identity {key}')
        typed_equal(inventory_contract(inputs, inv), registry['inventory_contracts'][key],
                    f'tracked independent typed contract {key}')
        inputs.sha256(PureWindowsPath(fixture).relative_to(
            PureWindowsPath(str(inputs.root))).as_posix(), inv['suite_sha256'])
        for check in inv['checks']:
            case_rel = PureWindowsPath(check['case_file']).relative_to(
                PureWindowsPath(str(inputs.root))).as_posix()
            inputs.sha256(case_rel, check['case_file_sha256'])
            for index, source in enumerate(check['source_files']):
                if index < len(check['input_paths']):
                    name = check['input_paths'][index]
                    if name.startswith('\\\\?\\'):
                        name = name[4:]
                    inputs.sha256(PureWindowsPath(name).relative_to(
                        PureWindowsPath(str(inputs.root))).as_posix(), source['sha256'])
            base = inv['fixture_base']
            if base.startswith('\\\\?\\'):
                base = base[4:]
            base_rel = PureWindowsPath(base).relative_to(PureWindowsPath(str(inputs.root))).as_posix()
            for artifact in check['oracle_artifacts']:
                inputs.sha256(PurePosixPath(base_rel, artifact['file']).as_posix(), artifact['sha256'])
        return inv

    def read_one(_):
        for key, binding in sorted(bindings.items()):
            exact_keys(binding, ('inventory', 'copy'), f'a2r binding {key}')
            inv = _admit_inventory(key, binding)
            copy = CaseCopy(EvidenceScope(evidence.path(binding['copy'] + '/artifact-copy.json').parent,
                                          inputs.budget))
            from ace_m1_first_slice import _repo_relative
            fixture = inv['fixture_base']
            if fixture.startswith('\\\\?\\'):
                fixture = fixture[4:]
            base_relative = PureWindowsPath(fixture).relative_to(
                PureWindowsPath(str(inputs.root))).as_posix()
            admitted = admit_case_bindings(copy, inv)
            parents[key] = {'report_sha256': admitted['report_sha256'], 'non_pass_rows': [
                {'case': c['case'], 'check': c['check'], 'status': row['status']}
                for c, row in admitted['checks'] if row['status'] != 'PASS']}
            for check, row in admitted['checks']:
                if check.get('adapter') != 'a2r':
                    continue
                cell = _cell_key(check['case'], required_cells)
                if cell is None:
                    continue  # suite-level extra coverage; no required cell claimed
                require(cell in required_cells, 'a2r cell mapping escaped the required set')
                # source identity: corpus cells re-verify the frozen source bytes
                name = cell[1]
                source_ok = True
                if cell[0] == 'corpus' and (name + '.at') in manifest['corpus_src']['files']:
                    digest = manifest['corpus_src']['files'][name + '.at']
                    typed_equal(check['source_files'][0]['sha256'], digest,
                                'a2r source identity ' + name)
                # oracle: every declared golden artifact re-hashed in the repo,
                # and the retained emitted bytes compared byte-for-byte.
                oracles = []
                for artifact in check.get('oracle_artifacts', ()):
                    golden = inputs.read_bytes(PurePosixPath(base_relative, artifact['file']).as_posix())
                    emitted_name = next((n for n in copy.files
                                         if n.endswith('/' + check['case'] + '/' + check['check']
                                                       + '/actual/' + artifact['path'])), None)
                    require(emitted_name is not None,
                            f'a2r emitted artifact missing: {check["case"]}/{check["check"]}/{artifact["path"]}')
                    emitted_bytes = copy.scope.read_bytes(emitted_name)
                    require(emitted_bytes == golden,
                            f'a2r golden byte-compare failed: {check["case"]}/{check["check"]}')
                    oracles.append({'golden': artifact['file'], 'sha256': artifact['sha256']})
                verdict = 'PASS' if (row['status'] == 'PASS' and source_ok) else 'FAIL'
                per_cell.setdefault(cell, []).append(
                    {'suite': key, 'case': check['case'], 'check': check['check'],
                     'verdict': verdict, 'oracles': oracles})
        result = {}
        for cell, rows in sorted(per_cell.items()):
            result[':'.join(cell)] = {
                'judgement': 'PASS' if all(r['verdict'] == 'PASS' for r in rows) else 'FAIL',
                'supporting_checks': rows}
        return result

    observations = _compact_admission(inputs, read_one, 2 << 20)
    from ace_m1_legacy_execution import execution_summary
    summary = execution_summary(required, observations)
    summary['source_case_reports'] = parents
    summary['claimed_cells'] = len(observations)
    return summary


def _suite_for(key):
    # The suite path is authoritative in the registry contract; this fallback
    # keeps the module self-describing for governance tests.
    paths = {
        'a2r_cert_992': 'crates/auto-lang/test/cases/ace-m1/legacy-a2r-cert-992.suite.ad',
        'a2r_text_994': 'crates/auto-lang/test/cases/ace-m1/legacy-a2r-text-994.suite.ad',
        'a2r_list_996': 'crates/auto-lang/test/cases/ace-m1/legacy-a2r-list-996.suite.ad',
        'a2r_trap_997': 'crates/auto-lang/test/cases/ace-m1/legacy-a2r-trap-997.suite.ad',
        'a2r_scene_998': 'crates/auto-lang/test/cases/ace-m1/legacy-a2r-resources-998.suite.ad',
        'a2r_semantics_916': 'crates/auto-lang/test/cases/ace-m1/a2r-semantics-916/a2r-semantics-916.suite.ad',
        'a2r_emit_917': 'crates/auto-lang/test/cases/ace-m1/a2r-emit-917/a2r-emit-917.suite.ad',
        'a2r_hir_rt_902': 'crates/auto-lang/test/cases/a2r-hir-record-text-902/a2r-hir-record-text-902.suite.ad',
        'a2r_hir_coll_904': 'crates/auto-lang/test/cases/a2r-hir-collections-904/a2r-hir-collections-904.suite.ad',
        'a2r_owned_931': 'crates/auto-lang/test/cases/ace-m1/a2r-owned-temporaries-931/a2r-owned-temporaries-931.suite.ad',
        'legacy_base': 'crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad',
    }
    return paths[key]
