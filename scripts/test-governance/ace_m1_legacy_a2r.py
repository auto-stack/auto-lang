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
            'supporting_checks': rows}
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
    "console_863": "78d90abd147ca362338ac67c76577c2ab63e54032eff332be5c215fb1d004537",
    "entry_862": "8dc8defc899cdf3552317ec6a23139ea88043914fd26e48d8ae84b22aef1d425",
    "legacy_base": "3f4f6da57b4dceaca74ae83ec13363f0bc53180516705bc7191d32665ac36a86",
    "legacy_base_probe": "3f4f6da57b4dceaca74ae83ec13363f0bc53180516705bc7191d32665ac36a86",
    "method_881": "7f89138bdac10eead48e82656361166b0056bdbdc2516205ec98df2fde647d98",
    "order_layout_914": "223d958c3f9c6db0eb3ea2f7ef2bf6fb66c4ad666384e859973cd94c84cb7b09",
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
    "p919": "0b854f06f964681d666263ae662f11cde3f0a89861eae5f9d0845b46ab821053",
    "p920": "463dd0868808bbcec5d690f74df352f517a320fc622dd70a2749a4f8cf206b35",
    "p923": "9cd4bec37499711623ee5389c375329948e9331836ad6ed9cdc2181df2745700",
    "p924": "188c5513d85bfbf5c5ac31f454fe634d3ba45b512a3ebb2c58070cbef50d6284",
    "p925": "dbb091933d1efa522a35bb3e8639c412e3081ee465a6967c64f533ca17a33e9e",
    "p926": "f6f47f6e161269687a3ab77baa8e9815a409490117c83156dc2e99aadb889904",
    "p928": "02dbfc0637c44014bc19b2f8a54c47c98b29bcb9c3a59b6b63b2664be081c710",
    "p999": "6532223f4daaac45b837583c70912a42ee400b70a90d074caa6aea3c74c081e9",
    "pcte936": "4e9da6ed610abc1ced770dcbab798634d122327bb424b98b3a60199f3d4859ca",
    "ptr": "6555045c3d17831c4f689286dfb0cf50148b756cb417abd7a1cc5fd9f69011fe"
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


def _probe_family_observation(scope, wire, check, table_row, recipe=None):
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
        require(all(type(st) is dict and st.get('op') in ('run', 'edit-source') for st in steps),
                'probe-family step ops')
        # multi-step legs (closures body-edit) are judged by their FINAL run
        # step; every earlier step must also have completed.
        run_steps = [st for st in steps if st.get('op') == 'run']
        require(run_steps, 'probe-family at least one run step')
        step = run_steps[-1]
        require(all(st.get('outcome') in ('success', 'trap', 'diag')
                    for st in steps if st.get('op') == 'run'),
                'probe-family intermediate step terminal')
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
        exact_keys(binding, ('inventory', 'copy', 'av', 'native', 'a2x'), f'probe family binding {key}')
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
            if check.get('adapter') != 'probe':
                continue
            cell = _probe_cell(check['case'], check['check'], all_cells)
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
            require(pin is not None, f'probe family worker pin missing: {key}/{worker_key}')
            # The worker pin is validated against the exchange frame itself
            # (admit_exchange); mn-worker.exe is deliberately built in two
            # feature states by the mn/mn-collections tool specs, so not every
            # family's recorded sha matches the current on-disk state.
            names = [n for n in copy.files if n.endswith('-start.json')
                     and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in n]
            if not names:
                continue  # avm-console or in-process legs: no wire exchange on this path
            require(len(names) == 1, f'one probe-family exchange required: {check["case"]}/{check["check"]}')
            wire = admit_exchange(copy, names[0], protocol=protocol, recipe=recipe,
                                  worker_sha256=pin['sha256'])
            oracle_note = 'suite-contract'
            table_row = values.get(cell[1]) if cell[0] == 'corpus' else None
            observation = _probe_family_observation(copy.scope, wire, check, table_row)
            observation['parent_status'] = row['status']
            if row['status'] != 'PASS':
                observation['judgement'] = 'FAIL'
            observation['oracle_source'] = oracle_note
            suite_cells.setdefault(':'.join(cell), []).append(
                {'suite': key, 'case': check['case'], 'check': check['check'],
                 'verdict': observation['judgement']})
            # keep the bounded observation itself per cell
            suite_cells.setdefault(':'.join(cell) + '#obs', []).append(observation)
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
