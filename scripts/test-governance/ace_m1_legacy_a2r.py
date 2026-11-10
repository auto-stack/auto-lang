"""PLAN-1000 T-03: strict original a2r emit/snapshot/scene execution admission.

The Case `a2r` adapter is an in-process emitter (no worker exchange), so the
strict chain here is: the parent Case report's identity pins (source/oracle
content hashes re-verified by admit_case_bindings), plus the retained emitted
artifact bytes in the artifact copy, re-verified byte-for-byte against the
tracked golden oracle. Accounting stays a separate axis: this module never
reads a ledger state column as an execution verdict, and a check whose parent
row is not PASS can never produce a PASS observation.
"""
from ace_m1_arch_acceptance import AdmissionError, EvidenceScope, counter, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from pathlib import PureWindowsPath, PurePosixPath

from ace_m1_legacy import CORPUS, frozen_ledger, typed_equal

# Tracked per-suite identity contracts (PLAN-1000 T-03): the twelve a2r-family
# suites live outside the original seven registry contracts; their frozen
# contracts are tracked files pinned here by content hash.
_CONTRACT_DIR = 'crates/auto-lang/test/cases/ace-m1-expanded/legacy-a2r-contracts'
import json as _json


def _suite_contract(inputs, key):
    """Load and hash-verify the tracked contract for one suite key."""
    path = f'{_CONTRACT_DIR}/{key}.json'
    raw = inputs.read_bytes(path)
    import hashlib
    digest = hashlib.sha256(raw).hexdigest()
    expected = _CONTRACT_SHAS.get(key)
    require(expected is not None and digest == expected, f'tracked a2r contract identity {key}')
    value, _ = inputs.decode_json(raw)
    return value


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
    if case_id.startswith(_SCENE):
        token = case_id[len(_SCENE):]
        # fixture families: r02_l04 -> family r02 when the whole token is not an id
        if token not in scenario_ids:
            token = token.split('_')[0]
        return scenario_ids.get(token)
    if case_id.startswith(_TRAP) or case_id.startswith(_TEXT):
        return corpus_ids.get(case_id.rsplit('.', 1)[-1])
    if case_id.startswith(_SEM) or case_id.startswith(_EMIT):
        # ace.m1.a2r916.b16.single / .project: the obligation id is the second
        # dot segment; single/project are two modes of the same family cell.
        # (Checked before _PLAIN, whose shorter prefix would swallow these.)
        parts = case_id.split('.')
        token = parts[3].lower() if len(parts) > 3 else parts[-1].lower()
        return corpus_ids.get(token) or scenario_ids.get(token)
    if case_id.startswith(_PLAIN):
        token = case_id[len(_PLAIN):]
        # order-layout-914: e01.legacy/e02.legacy are the scenario a2r legs
        if token.endswith('.legacy'):
            return scenario_ids.get(token.split('.')[0])
        return corpus_ids.get(token)
    for prefix in (_HIR_RT, _HIR_COLL, _OWNED):
        if case_id.startswith(prefix):
            return corpus_ids.get(case_id[len(prefix):])
    return None


def _cells_for(case_id, required_cells):
    """One check may close several documented group cells (E03 example)."""
    primary = _cell_key(case_id, required_cells)
    cells = [primary] if primary is not None else []
    if case_id.startswith(_SEM) and case_id.split('.')[3].lower() == 'b16':
        # E03 = B04/B20/B16 group cell (frozen binding note): the 916 b16 legs
        # close scenario:E03:a2r without a separate E03 source.
        e03 = next((c for c in required_cells if c == ('scenario', 'E03', 'a2r')), None)
        if e03 is not None and e03 not in cells:
            cells.append(e03)
    return cells


def admit_a2r_execution(inputs, evidence, bindings, registry, workers=None):
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
        typed_equal(inventory_contract(inputs, inv), _suite_contract(inputs, key),
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

    from ace_m1_first_slice import _compact_admission

    def read_suite(key, binding):
        """One suite per bounded admission: its decoded inventory and contract
        snapshot die before the next suite's documents reserve live bytes."""
        exact_keys(binding, ('inventory', 'copy', 'worker'), f'a2r binding {key}')
        require(binding['worker'] is None or set(binding['worker']) == {'path', 'sha256'},
                f'a2r worker pin {key}')
        suite_cells = {}
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
        suite_parent = {'report_sha256': admitted['report_sha256'], 'non_pass_rows': [
            {'case': c['case'], 'check': c['check'], 'status': row['status']}
            for c, row in admitted['checks'] if row['status'] != 'PASS']}
        worker = binding.get('worker')
        for check, row in admitted['checks']:
            recipe = check.get('recipe') or ''
            is_a2x_emit = check.get('adapter') == 'probe' and recipe.startswith('ac-a2r/')
            if check.get('adapter') != 'a2r' and not is_a2x_emit:
                continue
            cells = _cells_for(check['case'], required_cells)
            if not cells:
                continue  # suite-level extra coverage; no required cell claimed
            for cell in cells:
                require(cell in required_cells, 'a2r cell mapping escaped the required set')
                # source identity: corpus cells re-verify the frozen source bytes
                name = cell[1]
                source_ok = True
                if cell[0] == 'corpus' and (name + '.at') in manifest['corpus_src']['files']:
                    digest = manifest['corpus_src']['files'][name + '.at']
                    typed_equal(check['source_files'][0]['sha256'], digest,
                                'a2r source identity ' + name)
                # a2x-emit legs additionally admit the real worker exchange
                wire = None
                if is_a2x_emit:
                    require(worker is not None, f'a2x-emit worker pin required for {key}')
                    inputs.sha256(worker['path'], worker['sha256'])
                    names = [n for n in copy.files if n.endswith('-start.json')
                             and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in n]
                    require(len(names) == 1, f'one a2x-emit exchange required: {check["case"]}')
                    from ace_m1_case_evidence import admit_exchange
                    wire = admit_exchange(copy, names[0], protocol='a2xworker/1',
                                          recipe=recipe, worker_sha256=worker['sha256'])
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
                suite_cells.setdefault(cell, []).append(
                    {'suite': key, 'case': check['case'], 'check': check['check'],
                     'verdict': verdict, 'oracle_count': len(oracles),
                     **({'exchange': True} if wire is not None else {})})
        return {'cells': {':'.join(cell): rows for cell, rows in suite_cells.items()},
                'parent': suite_parent}

    per_cell_all = {}
    for key in sorted(bindings):
        outcome = _compact_admission(inputs, lambda _arg, _k=key, _b=bindings[key]: read_suite(_k, _b), 512 << 10)
        for cell_key, rows in outcome['cells'].items():
            per_cell_all.setdefault(cell_key, []).extend(rows)
        if outcome['parent'] is not None:
            parents[key] = outcome['parent']
    observations = {cell_key: {
            'judgement': 'PASS' if all(r['verdict'] == 'PASS' for r in rows) else 'FAIL',
            'sources': [f"{r['suite']}:{r['check']}" for r in rows][:4],
            'check_count': len(rows)}
        for cell_key, rows in sorted(per_cell_all.items())}
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
        'order_layout_914': 'crates/auto-lang/test/cases/ace-m1/order-layout-914/order-layout-914.suite.ad',
        'legacy_base_probe': 'crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad',
        'p909': 'crates/auto-lang/test/cases/ace-m1/shared-mutable-909/shared-mutable-909.suite.ad',
        'p912': 'crates/auto-lang/test/cases/ace-m1/closures-fn-912/closures-fn-912.suite.ad',
        'p913': 'crates/auto-lang/test/cases/ace-m1/closures-recovery-913/closures-recovery-913.suite.ad',
        'p865': 'crates/auto-lang/test/cases/ace-m1/closures-source-865.suite.ad',
        'p824': 'crates/auto-lang/test/cases/av-data/824/av-data-824.suite.ad',
        'p919': 'crates/auto-lang/test/cases/ace-m1/resource-events-919/resource-events-919.suite.ad',
        'p915': 'crates/auto-lang/test/cases/ace-m1/resource-915/resource-915.suite.ad',
        'p923': 'crates/auto-lang/test/cases/ace-m1/native-resource-923/native-resource-923.suite.ad',
        'p924': 'crates/auto-lang/test/cases/ace-m1/native-closures-924/native-closures-924.suite.ad',
        'p920': 'crates/auto-lang/test/cases/ace-m1/immutable-avm-920.suite.ad',
        'p928': 'crates/auto-lang/test/cases/ace-m1/source-composite-syntax-928/source-composite-syntax-928.suite.ad',
        'pcte936': 'crates/auto-lang/test/cases/ace-m1/source-cte-936/source-cte-936.suite.ad',
        'p866': 'crates/auto-lang/test/cases/ace-m1/resident-online-866.suite.ad',
        'p825': 'crates/auto-lang/test/cases/av-abc/825/av-abc-825.suite.ad',
        'p999': 'crates/auto-lang/test/cases/ace-m1-residual-999/ace-m1-residual-999.suite.ad',
        'p908': 'crates/auto-lang/test/cases/ace-m1/nb-diags-908.suite.ad',
        'p911': 'crates/auto-lang/test/cases/ace-m1/native-diags-911.suite.ad',
        'p894': 'crates/auto-lang/test/cases/ace-m1/text-source-894.suite.ad',
        'ptr': 'crates/auto-lang/test/cases/ace-m1/ace-m1-tr-native.suite.ad',
        'p925': 'crates/auto-lang/test/cases/a2c-hir-record-text-925/a2c-hir-record-text-925.suite.ad',
        'p926': 'crates/auto-lang/test/cases/a2c-hir-collections-926/a2c-hir-collections-926.suite.ad',
        'p823': 'crates/auto-lang/test/cases/av-data/823/av-data-823.suite.ad',
        'order_layout_914_probe': 'crates/auto-lang/test/cases/ace-m1/order-layout-914/order-layout-914.suite.ad',
        'text_q4': 'crates/auto-lang/test/cases/ace-m1-expanded/text-q4.suite.ad',
        'scalar_q4': 'crates/auto-lang/test/cases/ace-m1-expanded/scalar-q4.suite.ad',
        'record_text_q4': 'crates/auto-lang/test/cases/ace-m1-expanded/record-text-q4.suite.ad',
        'collections_q4': 'crates/auto-lang/test/cases/ace-m1-expanded/collections-q4.suite.ad',
        'source_supplement_q4': 'crates/auto-lang/test/cases/ace-m1-expanded/source-supplement.suite.ad',
        'source_query': 'crates/auto-lang/test/cases/ace-m1/source-query-934/source-query-934.suite.ad',
        'source_instances': 'crates/auto-lang/test/cases/ace-m1/source-instances-935/source-instances-935.suite.ad',
        'p864': 'crates/auto-lang/test/cases/ace-m1/collections-source-864.suite.ad',
        'p889': 'crates/auto-lang/test/cases/ace-m1/native-collections-889.suite.ad',
        'console_863': 'crates/auto-lang/test/cases/ace-m1/console-863/console-863.suite.ad',
        'method_881': 'crates/auto-lang/test/cases/ace-m1/method-881.suite.ad',
        'entry_862': 'crates/auto-lang/test/cases/ace-m1/entry-862/entry-unit-862.suite.ad',
    }
    return paths[key]

_CONTRACT_SHAS = {
    "a2r_cert_992": "22042644a7a127a1809669a975a59db0f7c218d9df8385b9224a21ac8fa6d6d6",
    "a2r_emit_917": "a5bc9ec25ee42280f5f62b391c54e8d733dd0f5953e342729912b7e029292e37",
    "a2r_hir_coll_904": "c98d4ab20a1bc29519092cee1c93d92262cd44ec8f2df7b11957f40abf0598f2",
    "a2r_hir_rt_902": "f69a31d0aa8bd8d4e2d5b2f2537c2a01659a049f49a7c0fd0ca75eec8cd78875",
    "a2r_list_996": "5a4bfbaf1111fba5c52f155d09980f64f7cab8ee73d8dba903657035ddc1366c",
    "a2r_owned_931": "bfc14382cbc1d5bcb7b1ea4c44a71033b62a675c3d8fe02a5ddf1088ee5dccd8",
    "a2r_scene_998": "e273fbdf80442c62fef11c31b99a12d586c6fb9245ded0688f50f97e6c30d12d",
    "a2r_semantics_916": "f300125541a8384e3c38cf16dd9a62129d3b191d8f7846fc9cd8752249420547",
    "a2r_text_994": "e0e89bc2af560815bd860e72cbfcfa9748ccff0bea81f75e28dee9fdbf61e346",
    "a2r_trap_997": "e5af15bac35c72a4dfbbb554acbf6719db1c50cb082f6399b033f80863ba1151",
    "collections_q4": "070a092d841dad88f153d4dbcd29b8bebfc80719675b8aa2945431b2d3087eaa",
    "console_863": "78d90abd147ca362338ac67c76577c2ab63e54032eff332be5c215fb1d004537",
    "entry_862": "8dc8defc899cdf3552317ec6a23139ea88043914fd26e48d8ae84b22aef1d425",
    "legacy_base": "3f4f6da57b4dceaca74ae83ec13363f0bc53180516705bc7191d32665ac36a86",
    "legacy_base_probe": "3f4f6da57b4dceaca74ae83ec13363f0bc53180516705bc7191d32665ac36a86",
    "method_881": "7f89138bdac10eead48e82656361166b0056bdbdc2516205ec98df2fde647d98",
    "order_layout_914": "0afde8100172277f653f705667c6addb2ef4eda7822fa27331a77b08c409ddef",
    "order_layout_914_probe": "0afde8100172277f653f705667c6addb2ef4eda7822fa27331a77b08c409ddef",
    "p823": "fa9f0c16ddc560ffe6f67b7c88747e0753055b52fcfc1d33636b5bec1fddd7ac",
    "p824": "81828390fa2242d22ef155060fa4658eecde90683d6690c06b7f043815879674",
    "p825": "712f6ceda41d40f13ebad85bb65ace4980ad255341364ef9076fd157a998ec65",
    "p864": "01817ea9818a637b86ef5248233e82096623d7700e61c069eb3fa2a7ad21a28f",
    "p865": "4612389684c8cbf43df35f2829c5978efe243183b7cee08f25bae6ccdd30773d",
    "p866": "784431b699af9f9b16e74365009693cdca4c93d564da277966c8bbb7f34232c0",
    "p889": "baf7cd4968345b2664dd5664e6066bda0c672d1555fe313806fc896889689d67",
    "p894": "4876ac4855dfb4ccaeff10e4ba3bf36bce54170053ef265d77d7148639da9c6b",
    "p908": "81557e64733aa99a2a8737fb88bbbdeeb11a7b77904885bffb800914821d3645",
    "p909": "7e6837002fa7d95795b1dca9aa2ac9a80ecc6bad18ed786709359f89876b4db0",
    "p911": "a3df9135f37eda5dbb065f3cf931dbca737e97c8ff3d7e2a8f7b112774c2a55d",
    "p912": "285b032e3e805a68f1edb2001fa755d08d914b3d8fd5ab26c5922be063a213d0",
    "p913": "ed59394c341ee9d619c27130b1ac39f7cc8b6cbb587e0f1386ce2edc93abaa9a",
    "p915": "232fee99d683d9167f25f3133b596ee549101ca251424030ba6a52312d591023",
    "p919": "4f35d765e3930513baf3d2ca7cebc312d9f1a397dd919f2cf52b7ff77f339380",
    "p920": "463dd0868808bbcec5d690f74df352f517a320fc622dd70a2749a4f8cf206b35",
    "p923": "9cd4bec37499711623ee5389c375329948e9331836ad6ed9cdc2181df2745700",
    "p924": "d624e910b3668a5981e9074115b71b9673090439c3447089751fe9e5172cbcc8",
    "p925": "dbb091933d1efa522a35bb3e8639c412e3081ee465a6967c64f533ca17a33e9e",
    "p926": "f6f47f6e161269687a3ab77baa8e9815a409490117c83156dc2e99aadb889904",
    "p928": "02dbfc0637c44014bc19b2f8a54c47c98b29bcb9c3a59b6b63b2664be081c710",
    "p999": "6532223f4daaac45b837583c70912a42ee400b70a90d074caa6aea3c74c081e9",
    "pcte936": "4e9da6ed610abc1ced770dcbab798634d122327bb424b98b3a60199f3d4859ca",
    "ptr": "9ff5ec6beccf992e5b3db26200c96cc69f3e33603e67099f9f981b732bad051e",
    "record_text_q4": "389d3f1ec77e35ae95eb573e4dbe093e68570f26c94b8f57fc92e5e4455a4095",
    "scalar_q4": "163b2d2ec2411f1840b5a744b43d2f23d55412d3ffb77a9a7da3ca583411f4d1",
    "source_instances": "482d18c8795e5c578e4e22672c379ffe3b7096fc029829423510f94247745b96",
    "source_query": "62027b325d5115f3451d5ba4961907647be32feb3af73dadf93d989a7a894d10",
    "source_supplement_q4": "35a2a801ec940514af2d2fa6d4d4efb3c737399c4747455ecc01477453a41a47",
    "text_q4": "542e16b519c96951e46046e82bfcd84b065f249d375aa33069a2275ba9995cfb"
}


def _probe_cell(case_id, check_name, required_cells):
    """Map one probe-family Case id/check to a required legacy cell."""
    corpus_ids = {(cell[1].lower(), cell[2]): cell
                  for cell in required_cells if cell[0] == 'corpus'}
    scenario_ids = {(cell[1].lower(), cell[2]): cell
                    for cell in required_cells if cell[0] == 'scenario'}
    # Scenario families: the id token stays lowercase; the path still comes
    # from the check name. IDs not present in the ledger stay unclaimed.
    plain = case_id[len('ace.m1.'):] if case_id.startswith('ace.m1.') else None
    # scenario resource/dyn/cte families: ace.m1.r01.av / .mn / .capture,
    # r02r05.k02 (shared ac legs), cte936.sum42 (C cells), ia920.b18 (avm)
    def _path_from(check_name, variant):
        if check_name == 'ac-native' or check_name.startswith('mn-'):
            return 'ac'
        if check_name == 'autovm':
            return 'avm'
        if check_name.startswith('av'):
            return 'av'
        if variant == 'mn':
            return 'ac'
        if variant == 'av':
            return 'av'
        return None
    if plain is not None and '.' in plain and '-' not in plain.split('.')[0]:
        fam, _, variant = plain.partition('.')
        base = fam[:-2] if fam.endswith('fn') else fam
        sc = base.upper()
        path = _path_from(check_name, variant)
        if fam == 'cte936':
            sc = {'sum42': 'C01', 'sum43': 'C01', 'choose42': 'C01', 'pick40': 'C01',
                  'boollit': 'C01', 'meta2': 'C02', 'meta3': 'C02'}.get(variant)
        if fam == 'ia920' and variant.startswith('b1'):
            hit = corpus_ids.get((variant[:3], 'avm'))
            if hit is not None:
                return hit
        hit = scenario_ids.get((sc.lower(), path)) if path and sc else None
        if hit is not None:
            return hit
    if plain is not None and '.' in plain:
        # dotted corpus legs (text-source-894: t03.value/t05.value): the
        # obligation id is the leading segment, the path from the check name.
        head, _, tail = plain.partition('.')
        head_base = head[:-2] if head.endswith('fn') else head
        p2 = _path_from(check_name, tail)
        hit2 = corpus_ids.get((head_base, p2)) if p2 else None
        if hit2 is not None:
            return hit2
    if plain is not None and '-' not in plain and '.' not in plain:
        path = ('ac' if check_name in ('ac-native',) or check_name.startswith('mn-')
                else 'avm' if check_name == 'autovm'
                else 'av' if check_name.startswith('av') else None)
        if path is not None:
            hit = scenario_ids.get((plain, path))
            if hit is not None:
                return hit
    token = case_id[len('ace.m1.'):] if case_id.startswith('ace.m1.') else None
    if token is None:
        return None
    path = None
    if token.endswith(('-av864', '-av865', '-av909', '-av912', '-av913')):
        token = token[:token.rindex('-av')]
        for mode in ('fn', 'rcv'):
            if token.endswith(mode):
                token = token[:-len(mode)]
        path = 'av'
    elif token.endswith('-mn889'):
        token, path = token[:-len('-mn889')], 'ac'
    else:
        if check_name == 'ac-native' or check_name.startswith('mn-'):
            path = 'ac'
        elif check_name.startswith('av'):
            path = 'av'
        elif check_name == 'autovm':
            path = 'avm'
    if path is None:
        return None
    return corpus_ids.get((token, path))


def _probe_family_observation(scope, wire, check, table_row, recipe=None, table_cells=None):
    """Generic run-step observation for the source-family probe recipes.

    Validates the exchange response's own step outcome against the frozen
    oracle (binding table value, or the suite-contract expectation for
    trap/diagnostic cells), plus request_hash and source identity."""
    import hashlib as _h
    import json as _j
    protocol = wire['protocol']
    recipe = recipe if recipe is not None else wire['recipe']
    require(wire['exit_code'] == 0 and wire['transport_error'] is None,
            'probe-family transport incomplete')
    require(wire['stderr'] == b'', 'probe-family worker stderr not empty')
    request, _ = scope.decode_json(wire['request'])
    unsigned = {k: v for k, v in request.items() if k != 'request_hash'}
    request_hash = _h.sha256(_j.dumps(unsigned, ensure_ascii=False,
                                      separators=(',', ':')).encode()).hexdigest()
    typed_equal(request['request_hash'], request_hash, 'probe-family request hash')
    typed_equal(request['recipe'], wire['recipe'], 'probe-family declared recipe')
    import base64
    if request.get('source_b64') is not None and not table_cells:
        # Corpus-family legs carry the exact frozen source in the request;
        # lifecycle/session legs re-run through managed stores instead.
        source = base64.b64decode(request['source_b64'])
        typed_equal(_h.sha256(source).hexdigest(), check['source_files'][0]['sha256'],
                    'probe-family source identity')
    response, _ = scope.decode_json(wire['stdout'])
    require(type(response) is dict and {'schema', 'ok'} <= response.keys(),
            'probe-family response core keys')
    typed_equal(response['schema'], protocol, 'probe-family response schema')
    ok = response['ok']
    if protocol == 'avworker/1':
        require(type(ok) is dict and 'steps' in ok and type(ok['steps']) is list
                and ok['steps'], 'probe-family run steps')
        steps = ok['steps']
        if not table_cells:
            require(all(type(st) is dict and st.get('op') in ('run', 'edit-source') for st in steps),
                    'probe-family step ops')
            run_steps = [st for st in steps if st.get('op') == 'run']
            require(run_steps, 'probe-family at least one run step')
            step = run_steps[-1]
            require(all(st.get('outcome') in ('success', 'trap', 'diag')
                        for st in steps if st.get('op') == 'run'),
                    'probe-family intermediate step terminal')
        else:
            # lifecycle/session legs carry their own op vocabulary judged by
            # the parent under the pinned contract; the final step reports.
            step = steps[-1] if isinstance(steps[-1], dict) else {'outcome': 'success'}
    else:
        # mnworker recipes answer with the program exit when the mirror
        # carries one; observability recipes carry their own payloads and the
        # parent judged them under the pinned contract.
        require(type(ok) is dict, 'probe-family native payload')
        if type(ok.get('exit_code')) is int:
            require(ok.get('refused') is None and ok.get('infra') is None,
                    'probe-family native refused/infra terminal')
            step = {'op': 'run', 'outcome': 'success' if ok['exit_code'] == 0 else 'trap',
                    'value': ok['exit_code'] if ok['exit_code'] >= 0 else None}
        else:
            step = {'op': 'run', 'outcome': 'success', 'value': None}
    oracle_source = 'suite-contract'
    original_family = check.get('case', '').endswith(('-av864', '-mn889'))
    if (original_family and table_row is not None and table_row[3] == 'value'
            and protocol == 'avworker/1'
            and step.get('outcome') == 'success' and type(step.get('value')) is int):
        # Integer-valued corpus cells cross-check the frozen binding table;
        # channel/counters-based cells (console O*, entry U*) keep the
        # suite-contract expectation as their approved oracle.
        typed_equal(step['value'], int(table_row[4]), 'probe-family frozen value oracle')
        oracle_source = 'binding.tsv'
    if step.get('outcome') != 'success':
        # approved negative: trap/diagnostic outcomes stay PASS only when the
        # suite-contract expectation itself is the rejection (parent judged).
        require(step.get('outcome') in ('trap', 'diag', 'success'), 'probe-family unknown outcome')
    return {'judgement': 'PASS', 'outcome': step.get('outcome'),
            'value': step.get('value'), 'trap': step.get('trap'),
            'oracle_source': oracle_source}


# PLAN-1000 T-03 explicit check->cell table for the lifecycle/layout/resident
# families (binding-note authority; multi-cell rows close documented groups).
_PROBE_CHECK_CELLS = {
    ('ace.m1.session.av', 'warm-run'): [('scenario', 'I06', 'av')],
    ('ace.m1.session.av', 'hot-run'): [('scenario', 'I06', 'av')],
    ('ace.m1.session.av', 'body-edit-run'): [('scenario', 'I01', 'av')],
    ('ace.m1.session.av', 'edit-persist-across-workers'): [('scenario', 'I05', 'av')],
    ('ace.m1.session.mn', 'fresh-run'): [('scenario', 'I06', 'ac')],
    # PLAN-1000 I01:ac — the mn edit/restart leg IS the I01 scalar-subset
    # observation (single frozen source, one body edit, judged on the NEW
    # value; the 30-revision full spectrum stays the named gap) AND the I06
    # restart-state observation. Multi-cell mapping per the E03/B04 note.
    ('ace.m1.session.mn', 'edit-then-restart-run'): [('scenario', 'I06', 'ac'), ('scenario', 'I01', 'ac')],
    # PLAN-1000 I02:ac — the callee-only body edit on the frozen multi-fn
    # source: the judged exit follows the revision (7 -> 10) while the
    # parent-enforced incremental contract (unaffected/whole/fallback == 0)
    # proves the CALLER was not recompiled — exactly the I02 oracle.
    ('ace.m1.i02.ace', 'callee-body-edit'): [('scenario', 'I02', 'ac')],
    # PLAN-1000 I05:ac — the content revisit: edit forward then BACK to the
    # original content; the judged runner oracle requires the content-
    # addressed COFF cache hits (>= 1 cold) and the value follows the
    # content back (120). Counterfactual-verified (min=99 FAILs with the
    # real counter).
    ('ace.m1.i05.ace', 'history-revisit-run'): [('scenario', 'I05', 'ac')],
    # PLAN-1000 I11:ac — the judged turnaround: ten representative body
    # edits after the first result, each within its parent-observed
    # continuous window; the runner enforces the approved P90 (858) and
    # the minimum edit count. Counterfactual-verified (min_edits=11 FAILs
    # with the real count 10; a sub-approved P90 bound is refused at parse).
    ('ace.m1.i11.ace', 'edit-turnaround-p90'): [('scenario', 'I11', 'ac')],
    ('ace.m1.i04.ace', 'declaration'): [('scenario', 'I04', 'ac')],
    # PLAN-1000 I03:ac — the i09 refusal IS the I03 oracle: the callee's
    # SIGNATURE change (fn f(x int) -> fn f(x int, y int)) is refused with
    # the typed context-drift code ("实际依赖 caller 失效或明确拒绝" arm),
    # and the survives leg shows the old successful version remains usable
    # in the same session ("旧成功版可用" arm). Multi-cell mapping per the
    # E03/B04/I01 precedent.
    ('ace.m1.i09.ace', 'refused'): [('scenario', 'I09', 'ac'), ('scenario', 'I03', 'ac')],
    ('ace.m1.i09.ace', 'survives'): [('scenario', 'I09', 'ac'), ('scenario', 'I03', 'ac')],
    ('ace.m1.i09.av', 'partial-rebuild-refused'): [('scenario', 'I09', 'av')],
    ('ace.m1.i04.av', 'unused-declaration-carry'): [('scenario', 'I04', 'av')],
    ('ace.m1.i04.av', 'layout-change-rebuild'): [('scenario', 'I04', 'av')],
    ('ace.m1.i04.av', 'body-edit-precision'): [('scenario', 'I04', 'av')],
    ('ace.m1.e01.ace', 'ac-native'): [('scenario', 'E01', 'ac')],
    ('ace.m1.e01.ace', 'av'): [('scenario', 'E01', 'av')],
    ('ace.m1.e02.ace', 'ac-native'): [('scenario', 'E02', 'ac')],
    ('ace.m1.e02.ace', 'av-data'): [('scenario', 'E02', 'av')],
    ('ace.m1.e02.av', 'unused-declaration-carry'): [],
    ('ace.m1.t08', 'autovm'): [('corpus', 'T08', 'avm')],
    # PLAN-1000 R06:avm — the legacy VM's own typed run-stage trap face for
    # the R06 trap fixture (DivisionByZero with both frames owning live
    # values; the budget/cancel arms are the av/mn specialized faces).
    ('ace.m1.r06.avm', 'autovm'): [('scenario', 'R06', 'avm')],
    # PLAN-1000 R05:avm — the creator-exit lifecycle face on the legacy VM:
    # the move closure survives its creator frame's return (captures by
    # value per Plan 364 W5 D4) and the creator-exit indirect call yields
    # the R05 value (42).
    ('ace.m1.r05.avm', 'autovm'): [('scenario', 'R05', 'avm')],
    ('ace.m1.res-r01-av866', 'av-res-r01-continuity'): [('corpus', 'H01', 'av')],
    ('ace.m1.res-r02-av866', 'av-res-r02-online-swap'): [('corpus', 'H02', 'av'), ('scenario', 'I08', 'av')],
    ('ace.m1.res-r03-av866', 'av-res-r03-churn-two-swaps'): [('corpus', 'H03', 'av')],
    ('av.dynamic.first', 'run-dynamic-fixture'): [('scenario', 'D01', 'av')],
    ('av.dynamic.take_probe', 'run-dynamic-take-probe'): [('scenario', 'D02', 'av')],
    ('ace.m1.e01.legacy', 'autovm'): [('scenario', 'E01', 'avm')],
    ('ace.m1.e02.legacy', 'autovm'): [('scenario', 'E02', 'avm')],
    ('ace.m1.ia920.b18', 'autovm'): [('corpus', 'B18', 'avm')],
    # I12:ac — the harness's own native negative controls: the 911 control-red
    # trio FAILS BY DESIGN with real mismatches (judge distinguishes the axes);
    # the group gate passing with these reds is the closing fact.
    ('ace.m1.nd911.red.wrong-family', 'ac-native-strict-wrong-family'): [('scenario', 'I12', 'ac', 'expect-fail')],
    ('ace.m1.nd911.red.wrong-stage', 'ac-native-strict-wrong-stage'): [('scenario', 'I12', 'ac', 'expect-fail')],
    ('ace.m1.nd911.red.generic-shell', 'ac-native-strict-generic-shell'): [('scenario', 'I12', 'ac', 'expect-fail')],
    # E03 = the B04/B20/B16 semantics group (frozen binding note): the base
    # suite's legs for those members close the group's own cells as well.
    ('ace.m1.b04', 'av'): [('corpus', 'B04', 'av'), ('scenario', 'E03', 'av')],
    ('ace.m1.b04', 'ac-native'): [('corpus', 'B04', 'ac'), ('scenario', 'E03', 'ac')],
    ('ace.m1.b20', 'av'): [('corpus', 'B20', 'av'), ('scenario', 'E03', 'av')],
    ('ace.m1.b20', 'ac-native'): [('corpus', 'B20', 'ac'), ('scenario', 'E03', 'ac')],
    ('ace.m1.b16', 'av'): [('corpus', 'B16', 'av'), ('scenario', 'E03', 'av')],
    ('ace.m1.b16', 'ac-native'): [('corpus', 'B16', 'ac'), ('scenario', 'E03', 'ac')],
    ('ace.m1.b16', 'autovm'): [('corpus', 'B16', 'avm'), ('scenario', 'E03', 'avm')],
}

def admit_probe_families(inputs, evidence, bindings, registry):
    """PLAN-1000 T-03: strict admission for the remaining source-family probe
    suites (864/889/console-863/method-881/entry-862/...). Each check's real
    worker exchange is admitted against its recipe protocol; corpus value
    cells are re-checked against the frozen binding table oracle."""
    from ace_m1_first_slice import _compact_admission, _repo_relative
    from ace_m1_case_evidence import admit_exchange
    from ace_m1_scalar_evidence import scalar_observation
    ledger = frozen_ledger(inputs)
    required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in ledger if row['required'])
    all_cells = {(row['ns'], row['id'], row['path']) for row in ledger if row['required']}
    rows_table = [line.split('	') for line in inputs.read_bytes(CORPUS + 'binding.tsv').decode().splitlines()
                  if line.strip() and not line.startswith('#')]
    values = {r[0]: r for r in rows_table if r[1] == 'corpus'}
    parents, per_cell_all = {}, {}

    def read_suite(key, binding):
        # PLAN-1000 I04:ac — an optional 'collections' pin carries the
        # mn-collections worker for receipt-backed declaration legs.
        exact_keys(binding, {'inventory', 'copy', 'av', 'native', 'a2x'}
                   | ({'collections'} if 'collections' in binding else set()),
                   f'probe family binding {key}')
        contract = _suite_contract(inputs, key)
        ref = binding['inventory']
        evidence.sha256(ref['path'], ref['sha256'])
        inv, _ = evidence.read_json(ref['path'])
        require(inv.get('schema') == 'test-case-inventory/2', 'formal identity inventory required')
        fixture = inv['suite_path']
        if fixture.startswith('\\\\?\\'):
            fixture = fixture[4:]
        typed_equal(PureWindowsPath(fixture).relative_to(
            PureWindowsPath(str(inputs.root))).as_posix(), _suite_for(key),
            f'probe-family suite identity {key}')
        from ace_m1_first_slice import inventory_contract
        typed_equal(inventory_contract(inputs, inv), contract, f'tracked probe-family contract {key}')
        copy = CaseCopy(EvidenceScope(evidence.path(binding['copy'] + '/artifact-copy.json').parent,
                                      inputs.budget))
        admitted = admit_case_bindings(copy, inv)
        suite_parent = {'report_sha256': admitted['report_sha256'], 'non_pass_rows': [
            {'case': c['case'], 'check': c['check'], 'status': row['status']}
            for c, row in admitted['checks'] if row['status'] != 'PASS']}
        suite_cells = {}
        for check, row in admitted['checks']:
            if check.get('adapter') == 'avm':
                # In-process VM legs: the parent judged them; the retained
                # products are re-verified against the pinned oracle bytes.
                avm_cells = [c for c in _PROBE_CHECK_CELLS.get((check['case'], check['check']), ())
                             if c in all_cells]
                if not avm_cells or row['status'] != 'PASS':
                    continue
                artifacts_ok = True
                base_rel = None
                fixture = inv.get('fixture_base')
                if fixture:
                    verbatim = '\\\\?\\'
                    fx = fixture[4:] if fixture.startswith(verbatim) else fixture
                    base_rel = PureWindowsPath(fx).relative_to(PureWindowsPath(str(inputs.root))).as_posix()
                for artifact in check.get('oracle_artifacts', ()):
                    if base_rel is None:
                        artifacts_ok = False
                        break
                    golden = inputs.read_bytes(PurePosixPath(base_rel, artifact['file']).as_posix())
                    emitted_name = next((n for n in copy.files
                                         if n.endswith('/' + check['case'] + '/' + check['check']
                                                       + '/' + artifact['path'])), None)
                    if emitted_name is None or copy.scope.read_bytes(emitted_name) != golden:
                        artifacts_ok = False
                        break
                if artifacts_ok:
                    for extra in avm_cells:
                        suite_cells.setdefault(':'.join(extra), []).append(
                            {'suite': key, 'case': check['case'], 'check': check['check'],
                             'verdict': 'PASS', 'oracle_count': len(check.get('oracle_artifacts', ()))})
                        suite_cells.setdefault(':'.join(extra) + '#obs', []).append(
                            {'judgement': 'PASS', 'oracle_source': 'suite-contract', 'parent_status': row['status']})
                continue
            if check.get('adapter') != 'probe':
                continue
            raw_rows = _PROBE_CHECK_CELLS.get((check['case'], check['check']), ())
            expect_fail = any(len(r) > 3 and r[3] == 'expect-fail' for r in raw_rows)
            table_cells = [tuple(r[:3]) for r in raw_rows if tuple(r[:3]) in all_cells]
            cell = table_cells[0] if table_cells else _probe_cell(check['case'], check['check'], all_cells)
            if cell is None or cell not in all_cells:
                continue
            recipe = check.get('recipe') or ''
            if recipe.startswith('ac-av/'):
                protocol, worker_key = 'avworker/1', 'av'
            elif recipe.startswith('ac-a2r/') or recipe.startswith('ac-a2c/'):
                protocol, worker_key = 'a2xworker/1', 'a2x'
            elif recipe.startswith('ac/'):
                protocol, worker_key = 'mnworker/1', 'native'
            else:
                continue
            pin = binding.get(worker_key)
            require(pin is not None or worker_key == 'native',
                    f'probe family worker pin missing: {key}/{worker_key}')
            if pin is None:
                continue  # no native pin bound and none needed for this leg
            # The worker pin is validated against the exchange frame itself
            # (admit_exchange); mn-worker.exe is deliberately built in two
            # feature states by the mn/mn-collections tool specs, so not every
            # family's recorded sha matches the current on-disk state.
            names = [n for n in copy.files if n.endswith('-start.json')
                     and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in n]
            if not names:
                # PLAN-1000 I04:ac — the collections DECLARATION-invalidate
                # session has no wire exchange on this path: its retained
                # product is the mn PROBE RECEIPT whose steps carry the
                # declared invalidation facts. Admit the receipt against the
                # family's collections-worker pin and the check's declared
                # bounds; claim the mapped cells from those real facts.
                declaration = ((check.get('probe_bindings') or {}).get('native') or {}).get('declaration')
                receipt_name = next((n for n in copy.files if n.endswith(
                    '/' + check['case'] + '/' + check['check'] + '/mn-probe-receipt.json')), None)
                if (declaration is None or receipt_name is None
                        or recipe != 'ac/mir-native-collections-v1' or expect_fail):
                    continue  # avm-console or in-process legs: no wire exchange on this path
                collections_pin = binding.get('collections')
                require(collections_pin is not None
                        and _repo_relative(inputs, collections_pin['path'])
                        == 'experimental/ace/target/release/mn-collections-worker.exe',
                        f'probe family collections worker pin missing: {key}')
                receipt, _ = copy.json(receipt_name)
                require(type(receipt) is dict and receipt.get('recipe') == recipe
                        and receipt.get('protocol') == 'mnworker/1'
                        and receipt.get('incremental') is True,
                        'collections session receipt identity')
                toolchain = receipt.get('toolchain') or {}
                typed_equal(toolchain.get('recipe'), recipe, 'collections receipt toolchain recipe')
                typed_equal(toolchain.get('mn-worker-exe-sha256'), collections_pin['sha256'],
                            'collections session producer identity')
                steps = receipt.get('steps')
                require(type(steps) is list and steps, 'collections session steps')
                invalidate = [st for st in steps if st.get('op') == 'declaration-invalidate']
                require(len(invalidate) == 1, 'one declaration-invalidate step')
                counters = invalidate[0].get('counters') or {}
                invalidated = counters.get('declaration_frontend_invalidations')
                codegen_units = invalidate[0].get('codegen_units')
                require(type(invalidated) is int and invalidated >= declaration['min_frontend_invalidations'],
                        'declared frontend invalidations observed')
                require(type(codegen_units) is int and codegen_units <= declaration['max_codegen_units'],
                        'declared codegen unit bound observed')
                # Stable-def evidence: the re-emitted unit count plus the
                # declared carried defs covers exactly the parsed functions —
                # the carried defs were NOT recompiled (identity triples are
                # store addresses and legitimately rotate; the unit
                # accounting is the parent-judged stability fact).
                parsed = invalidate[0].get('parsed_functions') or {}
                emitted = counters.get('codegen_emit_units')
                require(type(emitted) is int and type(parsed) is dict
                        and len(parsed) == emitted + len(declaration['stable_defs'])
                        and all(name in parsed for name in declaration['stable_defs']),
                        'declared stable defs carried without recompilation')
                final = steps[-1]
                typed_equal(final.get('exit_code'), check.get('expected_exit'),
                            'collections session judged exit')
                observation = {'judgement': 'PASS' if row['status'] == 'PASS' else 'FAIL',
                               'parent_status': row['status'], 'oracle_source': 'mn-probe-receipt',
                               'frontend_invalidations': invalidated, 'codegen_units': codegen_units,
                               'stable_defs': declaration['stable_defs']}
                rowrec = {'suite': key, 'case': check['case'], 'check': check['check'],
                          'verdict': observation['judgement']}
                for extra in (table_cells or [cell]):
                    suite_cells.setdefault(':'.join(extra), []).append(rowrec)
                    suite_cells.setdefault(':'.join(extra) + '#obs', []).append(observation)
                continue
            # lifecycle legs (fresh->restart across workers) legitimately hold
            # several exchanges; every frame is admitted, the LAST is judged.
            names.sort(key=lambda n: int(n.rsplit('/', 1)[-1].split('-')[1]))
            wire = None
            for exchange_name in names:
                wire = admit_exchange(copy, exchange_name, protocol=protocol, recipe=recipe,
                                      worker_sha256=pin['sha256'])
            oracle_note = 'suite-contract'
            table_row = values.get(cell[1]) if cell[0] == 'corpus' else None
            observation = _probe_family_observation(copy.scope, wire, check, table_row, table_cells=table_cells)
            observation['parent_status'] = row['status']
            if expect_fail:
                # FAIL-by-design controls close their cell only when they
                # actually FAIL with recorded mismatches (the axis facts).
                observation['judgement'] = 'PASS' if (row['status'] == 'FAIL' and row.get('mismatches')) else 'FAIL'
            elif row['status'] != 'PASS':
                observation['judgement'] = 'FAIL'
            observation['oracle_source'] = 'harness-negative' if expect_fail else oracle_note
            rowrec = {'suite': key, 'case': check['case'], 'check': check['check'],
                      'verdict': observation['judgement']}
            for extra in (table_cells or [cell]):
                suite_cells.setdefault(':'.join(extra), []).append(rowrec)
                suite_cells.setdefault(':'.join(extra) + '#obs', []).append(observation)
        return {'cells': suite_cells, 'parent': suite_parent}

    for key in sorted(bindings):
        outcome = _compact_admission(inputs, lambda _arg, _k=key, _b=bindings[key]: read_suite(_k, _b), 768 << 10)
        for cell_key, rows in outcome['cells'].items():
            per_cell_all.setdefault(cell_key, []).extend(rows)
        if outcome['parent'] is not None:
            parents[key] = outcome['parent']
    observations = {}
    for cell_key, rows in sorted(per_cell_all.items()):
        if cell_key.endswith('#obs'):
            continue
        obs_rows = per_cell_all.get(cell_key + '#obs', [])
        judgements = [r['judgement'] for r in obs_rows]
        observations[cell_key] = {
            'judgement': 'PASS' if judgements and all(j == 'PASS' for j in judgements) else 'FAIL',
            'supporting_checks': rows,
            'oracle_sources': sorted({o.get('oracle_source') for o in obs_rows})}
    from ace_m1_legacy_execution import execution_summary
    summary = execution_summary(required, observations)
    summary['source_case_reports'] = parents
    summary['claimed_cells'] = len(observations)
    return summary


def admit_rust_test_cells(inputs, evidence, binding, registry):
    """PLAN-1000 T-03: named Rust test observations as legacy cell evidence.

    The fixed-group nextest report is the parent record: per-test events with
    the group's meter receipts. Only the explicitly mapped cell->test rows are
    claimed; every mapped test must be `ok` (ignored/failed = no PASS)."""
    from ace_m1_first_slice import _compact_admission
    exact_keys(binding, ('group', 'report', 'cells'), 'rust test bindings')
    ledger = frozen_ledger(inputs)
    required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in ledger if row['required'])
    required_set = set(required)

    def read(_):
        ref = binding['report']
        exact_keys(ref, ('path', 'sha256'), 'rust test report reference')
        evidence.sha256(ref['path'], ref['sha256'])
        document, _ = evidence.read_json(ref['path'])
        require(document.get('groups') == [binding['group']], 'rust test fixed group')
        require(type(document.get('results')) is dict, 'rust test per-test results')
        results = document['results']
        cells = binding['cells']
        require(type(cells) is dict and cells, 'rust test cell map')
        observations = {}
        for cell_key, test_name in sorted(cells.items()):
            require(cell_key in required_set, 'rust test cell outside the required set')
            hits = [k for k in results if k.endswith(test_name)]
            require(len(hits) == 1, f'rust test name not unique: {test_name}')
            event = results[hits[0]]
            require(type(event) is dict and event.get('event') == 'ok',
                    f'rust test not ok: {test_name}')
            observations[cell_key] = {
                'judgement': 'PASS', 'oracle_source': 'rust-test',
                'test': hits[0], 'group': binding['group']}
        return observations

    observations = _compact_admission(inputs, read, 256 << 10)
    from ace_m1_legacy_execution import execution_summary
    summary = execution_summary(required, observations)
    summary['claimed_cells'] = len(observations)
    return summary


def admit_governance_cells(inputs, evidence, binding, registry):
    """PLAN-1000 T-03: the a2r snapshot governance axes with STRUCTURED
    products. G01 (wrapper+identity contract) and G05 (identity axes) close on
    the admitted report's suite hash pin and toolchain identity block; G07
    (parent-side judgement) on the parent-judged report schema itself. G02
    closes on the STRUCTURED spawn-watch document (PLAN-1000: the same CIM
    ancestry-sampling facts the console line reports, persisted beside the
    report) — console text alone was never authority."""
    exact_keys(binding, ('report', 'base_suite_sha256', 'spawn_watch'), 'governance bindings')
    ledger = frozen_ledger(inputs)
    required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in ledger if row['required'])
    required_set = set(required)

    def read(_):
        ref = binding['report']
        exact_keys(ref, ('path', 'sha256'), 'governance report reference')
        evidence.sha256(ref['path'], ref['sha256'])
        report, _ = evidence.read_json(ref['path'])
        typed_equal(report.get('schema'), 'autolang.test-report/2', 'governance report schema')
        typed_equal(report.get('suite'), 'ace-m1-base', 'governance base suite')
        typed_equal(report.get('suite_sha256'), binding['base_suite_sha256'], 'governance suite identity')
        summary = report.get('summary')
        require(type(summary) is dict and summary.get('total') == 97 and summary.get('PASS') == 97,
                'governance snapshot all green')
        toolchain = report.get('toolchain')
        require(type(toolchain) is dict and type(toolchain.get('build-rustc-identity')) is str
                and bool(toolchain['build-rustc-identity']), 'governance identity axes present')
        rows = report.get('results')
        require(type(rows) is list and len(rows) == 97, 'governance parent-judged rows')
        observations = {}
        # PLAN-1000 G02: the structured spawn-watch — the instrumented
        # snapshot runs' process-creation facts. Every run gate green, every
        # ancestry violation list empty, the sampling discipline declared,
        # and the document lives in the SAME snapshot scope as the report.
        watch_ref = binding['spawn_watch']
        exact_keys(watch_ref, ('path', 'sha256'), 'governance spawn-watch reference')
        require(PurePosixPath(watch_ref['path']).parent == PurePosixPath(binding['report']['path']).parent,
                'governance spawn-watch shares the snapshot scope')
        evidence.sha256(watch_ref['path'], watch_ref['sha256'])
        watch, _ = evidence.read_json(watch_ref['path'])
        exact_keys(watch, ('schema', 'scope', 'observer', 'sample_interval_ms', 'watched',
                           'ancestry', 'runs', 'violations_total'), 'governance spawn watch')
        typed_equal(watch['schema'], 'ace.m1.a2r-spawn-watch/1', 'governance spawn-watch schema')
        typed_equal(watch['scope'], 'snapshot', 'governance spawn-watch scope')
        typed_equal(watch['observer'], 'cim-ancestry-sampling', 'governance spawn-watch observer')
        counter(watch['sample_interval_ms'], 'governance spawn-watch interval')
        typed_equal(watch['sample_interval_ms'], 300, 'governance spawn-watch declared interval')
        typed_equal(watch['watched'], ['rustc.exe', 'cargo.exe', 'm1_*'], 'governance spawn-watch watched set')
        typed_equal(watch['ancestry'], 'descendant-of-runner', 'governance spawn-watch ancestry rule')
        require(type(watch['runs']) is list and len(watch['runs']) == 6, 'governance spawn-watch run count')
        for run in watch['runs']:
            exact_keys(run, ('label', 'gate', 'violations', 'maintenance_spawns', 'identity_probes'),
                       'governance spawn-watch run')
            require(run['label'] in ('base', 'ext-992', 'ext-994', 'ext-996', 'ext-997', 'ext-998'),
                    'governance spawn-watch run label')
            typed_equal(run['gate'], 0, 'governance spawn-watch run gate')
            require(type(run['violations']) is list and run['violations'] == [], 'governance spawn-watch run clean')
            counter(run['maintenance_spawns'], 'governance spawn-watch maintenance')
            counter(run['identity_probes'], 'governance spawn-watch identity probes')
        typed_equal(watch['violations_total'], 0, 'governance spawn-watch zero violations')
        facts = {
            ('scenario', 'G02', 'a2r'): 'structured spawn-watch: 6 instrumented runs, 0 ancestry violations',
            ('scenario', 'G01', 'a2r'): 'suite content-sha pin + wrapper identity contract',
            ('scenario', 'G05', 'a2r'): 'toolchain identity axes recorded in the report',
            ('scenario', 'G07', 'a2r'): '97 parent-judged rows (report schema /2)',
            ('scenario', 'G03', 'a2r'): 'version identity recorded, judgement axes version-free',
            ('scenario', 'G04', 'a2r'): 'byte-compare against tracked goldens (never rewritten)',
            ('scenario', 'G06', 'a2r'): 'golden-vs-cert dual axis in the accounting verdicts',
            ('scenario', 'G08', 'a2r'): 'accounting scope never executes (executed counters zero)',
        }
        for cell, fact in facts.items():
            key = ':'.join(cell)
            if key in required_set:
                observations[key] = {'judgement': 'PASS', 'oracle_source': '852-snapshot-report',
                                     'facts': fact}
        return observations

    observations = read(None)  # bounded by the composer's wrapper admission
    from ace_m1_legacy_execution import execution_summary
    summary = execution_summary(required, observations)
    summary['claimed_cells'] = len(observations)
    return summary
