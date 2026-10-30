"""Counterexamples for PLAN-890's cross-repository test boundaries."""
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
from repository_layout import relative_path, workspace_root
from module_catalog import discover
import copy
import json
from test_identity_migrations import records, key, PATH
from module_catalog import digest
from web_delivery import collect_web_legs, validate_four_receipt, WEB_LEGS, MODULE_LEGS, REQUIRED_OWNERS, REQUIRED_LEGS


class WebOwnershipTests(unittest.TestCase):
    def test_distribution_rejects_partial_missing_steps_changed_sources_and_unexecuted_commands(self):
        from web_delivery import validate_distribution_receipt, DISTRIBUTION_IDS
        import hashlib
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root/'source'
            source.mkdir()
            logs = root/'logs'
            logs.mkdir()
            h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
            fhash = lambda d: hashlib.sha256(json.dumps(d, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
            payload = source/'sdk/.config/nextest.toml'
            payload.parent.mkdir(parents=True)
            payload.write_text('test-threads = 1\n')
            files = {'sdk/.config/nextest.toml': {'bytes': payload.stat().st_size, 'sha256': h(payload)}}
            sources = {'auto-lang': '1'*40, 'auto-web': '2'*40}
            man = '3'*40
            names = ['auto-web', 'auto-web-contract', 'auto-lang-core', 'auto-generation-core', 'a2r-std', 'auto-media-core',
                     *('sdk-dep-'+str(i) for i in range(13))]
            manifest = {'schema': 'web-source/1', 'candidate': False, 'sources': sources,
                        'source_sha256': fhash(files), 'files': files, 'tool_producer': {'source_commit': man},
                        'source_closure': {'packages': [{'name': n} for n in names]}}
            (source/'source-manifest.json').write_text(json.dumps(manifest))
            old = {'auto-lang': '674871e2a3bcc1a9cf594ac7446f3c1d1129ecbb', 'auto-man': 'e6cc06fbc496f366edde7929abfd49254ec13647'}
            new = {**sources, 'auto-man': man}
            legacy = {'schema': 'web-legacy-upgrade/1', 'outcome': 'pass', 'checks': [], 'snapshots': []}
            for label, pins in [('old', old), ('new', new)]:
                for repo, revision in pins.items():
                    archive = root/(label+'-'+repo+'.tar')
                    archive.write_bytes((repo+revision).encode())
                    legacy['snapshots'].append({'repo': repo, 'revision': revision, 'path': str(root/label/repo),
                                                'archive': str(archive), 'archive_sha256': h(archive)})
                    legacy['checks'].append({'id': 'archive-'+label+'-'+repo, 'status': 'PASS', 'exit': 0,
                                             'command': ['git', 'archive', str(archive), revision]})
                legacy['checks'] += [{'id': label+'-core-locked-build', 'status': 'PASS', 'exit': 0, 'command': ['cargo', 'build', '--locked']},
                                     {'id': label+'-core-install', 'status': 'PASS', 'sources': pins}]
            for name in ['old-installed-runtime', 'new-installed-runtime', *('after-'+p+'-failure-old-runtime' for p in ('prepare', 'verify', 'swap'))]:
                legacy['checks'].append({'id': name, 'status': 'PASS', 'exact_json': 'legacy-890', 'shutdown': 202,
                                          'exit': 0, 'port_released': True, 'binary_sha256': '4'*64})
            legacy_path = root/'legacy.json'
            legacy_path.write_text(json.dumps(legacy))
            checks = []
            self.assertEqual(len(DISTRIBUTION_IDS), 45)
            for name in sorted(DISTRIBUTION_IDS):
                row = {'id': name, 'status': 'PASS'}
                log = logs/(name+'.log')
                if name.endswith('runtime'):
                    log.write_text('actual runtime output\n')
                    row['detail'] = {'exact_json': 890, 'shutdown': 202, 'exit': 0, 'port_released': True,
                                     'binary_sha256': '5'*64, 'log_sha256': h(log)}
                elif name == 'legacy-three-owner-recovery': row['detail'] = {'receipt': str(legacy_path), 'sha256': h(legacy_path)}
                elif name == 'independent-installed-closure': row['detail'] = {'local_packages': 19}
                elif name == 'interrupted-generation-recovered': row['detail'] = {'checkpoint': 'public Web workspace phase'}
                else:
                    log.write_text('Summary [ 1s] 217 tests run: 217 passed, 2 skipped\n' if name == 'independent-sdk-tests' else 'real command output\n')
                    code = 2 if name in ('failed-swap', 'missing-resource-rejected', 'bad-hash-rejected', 'bad-version-rejected') else 1 if name.endswith('public-rejection') else 0
                    row.update(command=['tool', '--source', str(source)], exit=code, expected_exit=code,
                               log=str(log), log_sha256=h(log))
                checks.append(row)
            receipt = {'schema': 'web-repository-distribution/1', 'outcome': 'pass', 'candidate': False, 'pending': [],
                       'sources': sources, 'source_sha256': fhash(files), 'checks': checks,
                       'summary': {'executed': 45, 'passed': 45, 'failed': 0, 'pending': 0}}
            path = root/'receipt.json'
            validate_distribution_receipt(receipt, path)
            for mutation in ('partial', 'candidate', 'missing', 'duplicate', 'zero-executed', 'not-run', 'wrong-runtime', 'wrong-log'):
                bad = copy.deepcopy(receipt)
                if mutation == 'partial': bad['outcome'] = 'partial'
                elif mutation == 'candidate': bad['candidate'] = True
                elif mutation == 'missing': bad['checks'].pop()
                elif mutation == 'duplicate': bad['checks'].append(bad['checks'][-1])
                elif mutation == 'zero-executed': bad['summary']['executed'] = 0
                elif mutation == 'not-run': next(r for r in bad['checks'] if 'command' in r)['exit'] = None
                elif mutation == 'wrong-runtime': next(r for r in bad['checks'] if r['id'].endswith('runtime'))['detail']['exact_json'] = 0
                else: next(r for r in bad['checks'] if 'command' in r)['log_sha256'] = '0'*64
                with self.subTest(mutation=mutation), self.assertRaises(ValueError): validate_distribution_receipt(bad, path)
            payload.write_text('changed selected source')
            with self.assertRaises(ValueError): validate_distribution_receipt(receipt, path)

    def test_original_photo_tests_cannot_disappear_or_be_ignored_at_new_owner(self):
        from test_identity_migrations import media_records
        rows = media_records()
        catalog = {'obligations': [{'id': 'media-unit|' + r['new'].replace('\t', '|'),
                                    'ignored': False} for r in rows]}
        media_records(catalog=catalog)
        missing = copy.deepcopy(catalog)
        missing['obligations'].pop()
        suppressed = copy.deepcopy(catalog)
        suppressed['obligations'][0]['ignored'] = True
        for bad in (missing, suppressed):
            with self.assertRaises(ValueError): media_records(catalog=bad)

    def test_four_owner_cargo_paths_reject_a_valid_foreign_web_or_sdk_source(self):
        import importlib.util
        script = Path(__file__).resolve().parents[1]/'run-full-regression.py'
        spec = importlib.util.spec_from_file_location('web_owner_full', script)
        full = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(full)
        with tempfile.TemporaryDirectory() as folder:
            lang, web = Path(folder)/'auto-lang', Path(folder)/'auto-web'
            document = {'packages': [
                {'name': 'auto', 'dependencies': [{'name': 'auto-web', 'path': str(web/'crates/auto-web')}]},
                {'name': 'auto-web', 'dependencies': [{'name': 'auto-lang-core', 'path': str(lang/'crates/auto-lang-core')}]},
                {'name': 'auto-lang-core', 'dependencies': [{'name': 'auto-web-contract', 'path': str(web/'crates/auto-web-contract')}]}]}
            full.validate_web_paths([document], lang, web)
            for package in range(3):
                bad = copy.deepcopy(document)
                bad['packages'][package]['dependencies'][0]['path'] = str(Path(folder)/'foreign/crates/auto-web')
                with self.subTest(package=package), self.assertRaises(ValueError):
                    full.validate_web_paths([bad], lang, web)
            bad = copy.deepcopy(document)
            bad['packages'].append({'name': 'auto-web-contract', 'dependencies': [
                {'name': 'auto-lang-core', 'path': str(lang/'crates/auto-lang-core')}]})
            with self.assertRaises(ValueError): full.validate_web_paths([bad], lang, web)

    def test_missing_or_changed_original_resources_and_duplicate_inputs_fail(self):
        from test_identity_migrations import validate_resources
        import hashlib
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)/'auto-lang'
            root.mkdir()
            rows = []
            for i in range(17):
                path = root/f'fixture-{i}.at'
                path.write_bytes(b'original input')
                rows.append({'original_path': path.name, 'path': path.name,
                             'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
            document = {'schema': 'web-resource-ownership/1', 'entries': rows}
            manifest = root/'resources.json'
            manifest.write_text(json.dumps(document))
            validate_resources(root, manifest)
            source = root/'fixture-0.at'
            source.write_bytes(b'changed input')
            with self.assertRaises(ValueError): validate_resources(root, manifest)
            source.unlink()
            with self.assertRaises(ValueError): validate_resources(root, manifest)
            source.write_bytes(b'original input')
            document['entries'][1] = copy.deepcopy(document['entries'][0])
            manifest.write_text(json.dumps(document))
            with self.assertRaises(ValueError): validate_resources(root, manifest)
            manifest.unlink()
            (root/'scripts/test-governance').mkdir(parents=True)
            (root/'scripts/test-governance/recipes.json').write_text('{"web-unit":{}}')
            with self.assertRaises(ValueError): validate_resources(root, manifest)

    def test_four_owner_verdict_rejects_partial_missing_duplicate_and_changed_sources(self):
        identities = {name: {'root': '/'+name, 'head': 'f'*40, 'source_sha256': 'a'*64,
                       'resource_manifest_sha256': 'b'*64, 'lock_sha256': 'c'*64,
                       'runtime_producer': {'source_sha256': 'a'*64}} for name in (*REQUIRED_OWNERS, 'down')}
        receipt = {'mode': 'four-repository', 'required_owners': list(REQUIRED_OWNERS),
                   'identities': identities, 'identities_after': copy.deepcopy(identities),
                   'outcome': 'pass', 'filters': [], 'list_only': False,
                   'required_legs': list(REQUIRED_LEGS),
                   'results': [{'leg': name, 'run_exit': 0, 'selected': 1} for name in REQUIRED_LEGS]}
        self.assertTrue(validate_four_receipt(receipt))
        for mutation in ('list-only', 'filtered', 'missing-owner', 'missing-hash', 'changed-hash',
                         'missing-leg', 'duplicate-leg', 'zero-selection', 'not-run', 'failure'):
            bad = copy.deepcopy(receipt)
            if mutation == 'list-only': bad['list_only'] = True
            elif mutation == 'filtered': bad['filters'] = ['only-one']
            elif mutation == 'missing-owner': del bad['identities']['web']; del bad['identities_after']['web']
            elif mutation == 'missing-hash': del bad['identities']['web']['lock_sha256']; del bad['identities_after']['web']['lock_sha256']
            elif mutation == 'changed-hash': bad['identities_after']['web']['source_sha256'] = 'changed'
            elif mutation == 'missing-leg': bad['results'].pop()
            elif mutation == 'duplicate-leg': bad['results'].append(bad['results'][-1])
            elif mutation == 'zero-selection': bad['results'][-1]['selected'] = 0
            elif mutation == 'not-run': bad['results'][-1]['run_exit'] = None
            elif mutation == 'failure': bad['results'][-1]['run_exit'] = 1
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate_four_receipt(bad)

    def test_web_legs_require_real_complete_module_results_and_executed_counts(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            (directory/'execution').mkdir()
            moved = [key(r['new']) for r in records()]
            from test_identity_migrations import media_records
            media = ['media-unit|' + r['new'].replace('\t', '|') for r in media_records()]
            ids = {name: (moved if name == 'web-unit' else media if name == 'media-unit' else [name+'|auto-web|actual']) for name in MODULE_LEGS}
            modules = [{'id': name+'/leaf', 'recipe': name, 'obligations': ids[name], 'conditions': []} for name in MODULE_LEGS]
            catalog = {'modules': modules}
            catalog_path = directory/'catalog.json'
            catalog_path.write_text(json.dumps(catalog))
            selection = {'scope': 'module', 'catalog_path': str(catalog_path), 'catalog_sha256': digest(catalog),
                         'modules': [m['id'] for m in modules], 'obligations': sorted(i for values in ids.values() for i in values)}
            results = {'complete': True, 'gate_passed': True, 'results': [
                {'module': m['id'], 'obligations': m['obligations'], 'applicable': m['obligations'],
                 'outcome': 'PASS', 'exit': 0, 'command': ['cargo', 'nextest', 'run', '--no-tests', 'fail']} for m in modules]}
            def save(selected, executed):
                (directory/'selection.json').write_text(json.dumps(selected))
                (directory/'execution/results.json').write_text(json.dumps(executed))
                for i, m in enumerate(modules):
                    (directory/'execution'/f'{i}-run.log').write_text(f'Summary [ 0.1s] {len(m["obligations"])} tests run: {len(m["obligations"])} passed\n')
            save(selection, results)
            self.assertEqual([r['leg'] for r in collect_web_legs(directory)], list(MODULE_LEGS))
            for mutation in ('missing-result', 'duplicate-result', 'missing-selected-id', 'partial-execution', 'list-instead-of-run', 'filtered-after-list'):
                selected, executed = copy.deepcopy(selection), copy.deepcopy(results)
                if mutation == 'missing-result': executed['results'].pop()
                elif mutation == 'duplicate-result': executed['results'].append(executed['results'][-1])
                elif mutation == 'missing-selected-id': selected['obligations'].pop()
                elif mutation == 'partial-execution': executed['complete'] = False
                elif mutation == 'list-instead-of-run': executed['results'][-1]['command'][2] = 'list'
                save(selected, executed)
                if mutation == 'filtered-after-list': (directory/'execution/0-run.log').write_text('Summary [ 0.1s] 1 test run: 1 passed\n')
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    collect_web_legs(directory)

    def test_migrated_original_conditions_and_missing_ids_are_not_rebaselined(self):
        from legacy_catalog import annotate
        migration = records()[0]
        old, new = migration['old'], migration['new']
        contract = {'schema': 'test-legacy-membership/1', 'platform': sys.platform,
                    'rows': [{'binary': old['binary'], 'test': old['test'], 'ignored': migration['original_ignored'],
                              'selected_in': ['t', 'tf']}], 'extra_conditions': [], 'additional_ids': []}
        row = {**new, 'id': key(new), 'ignored': migration['original_ignored']}
        rows = [copy.deepcopy(row)]
        annotate(rows, contract)
        self.assertEqual(rows[0]['legacy_tiers'], ['t', 'tf'])
        with self.assertRaises(ValueError):
            annotate([], contract)
        row['ignored'] = not row['ignored']
        with self.assertRaises(ValueError):
            annotate([row], contract)

    def test_duplicate_migration_destination_and_bad_source_identity_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'migrations.json'
            document = json.loads(PATH.read_text(encoding='utf-8'))
            document['migrations'][1]['new'] = document['migrations'][0]['new']
            path.write_text(json.dumps(document))
            with self.assertRaises(ValueError):
                records(path)
            document = json.loads(PATH.read_text(encoding='utf-8'))
            document['migrations'][0]['source_sha256'] = 'unknown'
            path.write_text(json.dumps(document))
            with self.assertRaises(ValueError):
                records(path)

    def test_declared_web_owner_cannot_escape_to_another_sibling(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "auto-lang"
            root.mkdir()
            web = Path(folder) / "auto-web"
            web.mkdir()
            recipe = {"workspace": "../auto-web", "repository": "web"}
            self.assertEqual(workspace_root(root, recipe), web.resolve())
            self.assertEqual(relative_path(root, web / "crates/auto-web"), "../auto-web/crates/auto-web")
            for bad in ({"workspace": "../auto-web"},
                        {"workspace": "../auto-ui", "repository": "web"},
                        {"workspace": "../auto-web/../../outside", "repository": "web"},
                        {"workspace": ".", "repository": "unknown"}):
                with self.subTest(recipe=bad), self.assertRaises(ValueError):
                    workspace_root(root, bad)
            with self.assertRaises(ValueError):
                relative_path(root, Path(folder) / "foreign/crates/auto-web")

    def test_external_runtime_enumeration_requires_registered_web_recipe_and_source(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "auto-lang"
            web = Path(folder) / "auto-web/crates/auto-web"
            document = {"rust-suites": {"auto-web": {
                "cwd": str(web), "package-name": "auto-web", "status": "listed",
                "testcases": {"http_owned_shutdown": {"ignored": False, "filter-match": {"status": "matches"}}}}}}
            with patch("module_catalog.ROOT", root):
                row = discover(document, "web-unit")[0]
                self.assertEqual(row["package_path"], "../auto-web/crates/auto-web")
                with self.assertRaises(ValueError):
                    discover(document, "language")
                document["rust-suites"]["auto-web"]["cwd"] = str(Path(folder) / "foreign/crates/auto-web")
                with self.assertRaises(ValueError):
                    discover(document, "web-unit")


if __name__ == "__main__":
    unittest.main()
