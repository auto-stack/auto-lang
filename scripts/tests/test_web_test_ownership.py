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
