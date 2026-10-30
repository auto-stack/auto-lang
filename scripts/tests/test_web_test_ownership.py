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
import hashlib
import contextlib
import json
from test_identity_migrations import records, key, PATH
from module_catalog import digest
from web_delivery import collect_web_legs, validate_four_receipt, WEB_LEGS, MODULE_LEGS, REQUIRED_OWNERS, REQUIRED_LEGS


@contextlib.contextmanager
def complete_module_protocol(folder, include_language=False):
    """Controlled protocol files, not a claim that synthetic logs ran tests.

    Only the external Git/environment fingerprint is substituted. All receipt
    admission, original ID/condition conservation and output parsing is real.
    """
    from module_catalog import load, ROOT
    from run_selected import applicability, exact_filter, execution_settings
    from web_delivery import file_proof
    actual = load(ROOT/'scripts/test-governance/modules.json')
    lookup = {r['id']: r for r in actual['obligations']}
    modules = copy.deepcopy([m for m in actual['modules'] if m['recipe'] in MODULE_LEGS])
    if include_language:
        item = next(m for m in actual['modules'] if m['recipe']=='language' and not m['conditions'])
        module = copy.deepcopy(item)
        module.update(id='language/protocol-control', obligations=[item['obligations'][0]])
        modules.append(module)
    for module in modules:
        module['consumers'] = []
    root = Path(folder)/'auto-lang'
    directory = root/'protocol'
    (directory/'execution').mkdir(parents=True)
    (root/'scripts/test-governance').mkdir(parents=True)
    recipes = {m['recipe']: actual['recipes'][m['recipe']] for m in modules}
    ids = {i for m in modules for i in m['obligations']}
    identity = {'code':'a'*64}
    catalog = {'schema':'test-modules/1', 'recipes':recipes, 'modules':modules,
               'obligations':[lookup[i] for i in sorted(ids)], 'source_identity':identity}
    path = root/'scripts/test-governance/modules.json'
    path.write_text(json.dumps(catalog))
    (path.parent/'periodic-four-repository-schedule.json').write_text(json.dumps({'schema':'test-periodic-module-schedule/1','recipes':list(recipes)}))
    selection = {'schema':'test-module-selection/1', 'scope':'module', 'execution_allowed':True,
                 'catalog_path':str(path), 'catalog_sha256':digest(catalog), 'source_identity':identity,
                 'modules':[m['id'] for m in modules], 'obligations':sorted(ids)}
    results = []
    for index, module in enumerate(modules):
        rows = [lookup[i] for i in module['obligations']]
        excluded = applicability(rows, {}, recipes[module['recipe']].get('required_env'))
        row = {'module':module['id'], 'obligations':sorted(module['obligations']),
               'applicable':[] if excluded else sorted(module['obligations']),
               'condition_environment':{}, 'condition_excluded':excluded,
               'outcome':'NOT_APPLICABLE' if excluded else 'PASS', 'exit':None if excluded else 0}
        results.append(row)
        if excluded: continue
        switches = [*execution_settings(recipes[module['recipe']], module)[0], '--ignore-default-filter','-E',exact_filter(rows)]
        row['cwd'] = str(root.parent/'auto-web' if module['recipe'] in WEB_LEGS else root)
        row['list_command'] = ['cargo','nextest','list',*switches,'--message-format','json']
        row['command'] = ['cargo','nextest','run',*switches,'--no-fail-fast','--no-tests','fail']
        suites = {}
        for r in rows:
            suite = suites.setdefault(r['binary'], {'cwd':str((root/r['package_path']).resolve()),
                        'package-name':r['package'], 'status':'listed', 'testcases':{}})
            suite['testcases'][r['test']] = {'ignored':False,'filter-match':{'status':'matches'}}
        listing = directory/'execution'/f'{index}-list.json'
        listing.write_text(json.dumps({'rust-suites':suites}))
        log = directory/'execution'/f'{index}-run.log'
        log.write_text(''.join(f"PASS [ 0.1s] {r['binary']} {r['test']}\n" for r in rows)+
                       f'Summary [ 0.1s] {len(rows)} tests run: {len(rows)} passed\n')
        from execution_trace import validate_bindings
        row.update(listing=file_proof(listing), output=file_proof(log), child_observations={"invocations":[]},
                   child_validation=validate_bindings(rows,{"invocations":[]}))
    report = {'complete':True,'gate_passed':True,'results':results}
    def save(selected=selection, executed=report):
        (directory/'selection.json').write_text(json.dumps(selected))
        executed.update(selection_sha256=file_proof(directory/'selection.json')['sha256'],
                        catalog_sha256=digest(catalog), source_identity=identity)
        (directory/'execution/results.json').write_text(json.dumps(executed))
    save()
    with patch('module_catalog.ROOT',root), patch('module_catalog.source_identity',return_value=identity):
        yield root, directory, catalog, selection, report, save


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
                        'source_closure': {'root_inputs':[], 'packages': [{'name': n, 'source_repo':'auto-web' if n in ('auto-web','auto-web-contract') else 'auto-lang', 'source_path':'crates/'+n} for n in names]}}
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
                                             'command': ['git', 'archive', str(archive), revision], 'log':str(archive), 'log_sha256':h(archive)})
                legacy['checks'] += [{'id': label+'-core-locked-build', 'status': 'PASS', 'exit': 0, 'command': ['cargo', 'build', '--locked'], 'log':str(archive), 'log_sha256':h(archive)},
                                     {'id': label+'-core-install', 'status': 'PASS', 'sources': pins, 'manifest_sha256':'8'*64}]
            for name in ['old-installed-runtime', 'new-installed-runtime', *('after-'+p+'-failure-old-runtime' for p in ('prepare', 'verify', 'swap'))]:
                legacy['checks'].append({'id': name, 'status': 'PASS', 'exact_json': 'legacy-890', 'shutdown': 202,
                                          'exit': 0, 'port_released': True, 'binary_sha256': '4'*64, 'log':str(archive), 'log_sha256':h(archive)})
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
                    log.write_text((''.join(f'PASS [ 0.1s] sdk-binary sdk_case_{i}\n' for i in range(217))+'Summary [ 1s] 217 tests run: 217 passed, 2 skipped\n') if name == 'independent-sdk-tests' else 'real command output\n')
                    code = 2 if name in ('failed-swap', 'missing-resource-rejected', 'bad-hash-rejected', 'bad-version-rejected') else 1 if name.endswith('public-rejection') else 0
                    row.update(command=['tool', '--source', str(source)], exit=code, expected_exit=code,
                               log=str(log), log_sha256=h(log))
                checks.append(row)
            receipt = {'schema': 'web-repository-distribution/1', 'outcome': 'pass', 'candidate': False, 'pending': [],
                       'sources': sources, 'source_sha256': fhash(files), 'checks': checks,
                       'summary': {'executed': 45, 'passed': 45, 'failed': 0, 'pending': 0}}
            path = root/'receipt.json'
            validate_distribution_receipt(receipt, path)
            for mutation in ('partial', 'candidate', 'missing', 'duplicate', 'zero-executed', 'not-run', 'wrong-runtime', 'wrong-log', 'missing-command', 'missing-exit', 'missing-output'):
                bad = copy.deepcopy(receipt)
                if mutation == 'partial': bad['outcome'] = 'partial'
                elif mutation == 'candidate': bad['candidate'] = True
                elif mutation == 'missing': bad['checks'].pop()
                elif mutation == 'duplicate': bad['checks'].append(bad['checks'][-1])
                elif mutation == 'zero-executed': bad['summary']['executed'] = 0
                elif mutation == 'not-run': next(r for r in bad['checks'] if 'command' in r)['exit'] = None
                elif mutation == 'wrong-runtime': next(r for r in bad['checks'] if r['id'].endswith('runtime'))['detail']['exact_json'] = 0
                elif mutation == 'missing-command': next(r for r in bad['checks'] if r['id']=='external-locked-build').pop('command')
                elif mutation == 'missing-exit': next(r for r in bad['checks'] if r['id']=='failed-swap').pop('exit')
                elif mutation == 'missing-output': next(r for r in bad['checks'] if r['id']=='successful-upgrade').pop('log')
                else: next(r for r in bad['checks'] if 'command' in r)['log_sha256'] = '0'*64
                with self.subTest(mutation=mutation), self.assertRaises(ValueError): validate_distribution_receipt(bad, path)
            with patch('subprocess.check_output',return_value=b'') as git:
                self.assertTrue(validate_distribution_receipt(receipt,path,{'auto-lang':root/'lang','auto-web':root/'web'}))
                self.assertEqual(git.call_count,4)
                self.assertTrue(any('crates/auto-lang-core' in call.args[0] for call in git.call_args_list))
            with patch('subprocess.check_output',return_value=b'crates/auto-lang-core/src/script_entry.rs\n'):
                with self.assertRaisesRegex(ValueError,'payload inputs differ'):
                    validate_distribution_receipt(receipt,path,{'auto-lang':root/'lang','auto-web':root/'web'})
            original_legacy=legacy_path.read_bytes()
            for target in ('old-core-locked-build','new-core-locked-build','archive-old-auto-lang'):
                changed=copy.deepcopy(legacy)
                next(r for r in changed['checks'] if r['id']==target).pop('command')
                legacy_path.write_text(json.dumps(changed))
                bad=copy.deepcopy(receipt)
                next(r for r in bad['checks'] if r['id']=='legacy-three-owner-recovery')['detail']['sha256']=h(legacy_path)
                with self.subTest(legacy_missing_command=target), self.assertRaises(ValueError):
                    validate_distribution_receipt(bad,path)
                legacy_path.write_bytes(original_legacy)
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
        from web_delivery import file_proof
        import importlib.util
        spec = importlib.util.spec_from_file_location('protocol_full', Path(__file__).resolve().parents[1]/'run-full-regression.py')
        full = importlib.util.module_from_spec(spec); spec.loader.exec_module(full)
        with tempfile.TemporaryDirectory() as folder, complete_module_protocol(folder, True) as fixture:
            root, directory, catalog, selection, report, save = fixture
            identities = {name: {'root':str(root if name=='language' else root.parent/('auto-'+name)),
                           'head':'f'*40,'source_sha256':'a'*64,'resource_manifest_sha256':'b'*64,
                           'lock_sha256':'c'*64,'runtime_producer':{'source_sha256':'a'*64}} for name in (*REQUIRED_OWNERS,'down')}
            contracts = {name:{'cwd':cwd,'kind':'nextest','options':options} for name,cwd,options in full.legs('tf',root,root.parent/'auto-ui',True)}
            contracts.update({r['leg']:r for r in full.batch_extra_legs(root,root.parent/'auto-ui',root.parent/'auto-man')})
            rows = collect_web_legs(directory)
            for name in REQUIRED_LEGS:
                if name in MODULE_LEGS: continue
                contract = contracts[name]
                package = contract.get('package') if contract['kind']=='libtest' else None
                if name=='language':
                    obligation = next(r for r in catalog['obligations'] if r['recipe']=='language')
                    binary,test = obligation['binary'],obligation['test']
                else: binary,test = package or 'owner-test-binary','valid_control'
                listing = directory/(name+'-list.json')
                listing.write_text(test+': test\n' if package else json.dumps({'rust-suites':{binary:{'testcases':{test:{'ignored':False,'filter-match':{'status':'matches'}}}}}}))
                options = contract.get('options',[])
                row = {'leg':name,'cwd':str(contract['cwd']),'list_exit':0,'run_exit':0,
                       'selected':1,'selected_ids':[binary+'\t'+test], 'listing':file_proof(listing),
                       'list_command':['cargo',*contract['list']] if package else ['cargo','nextest','list',*options,'--message-format','json']}
                if name=='language':
                    row.update(module_directory=str(directory), module_exit=0,
                               module_command=full.module_queue_command(root,'tf',[],True,directory,False,True),
                               module_selection=file_proof(directory/'selection.json'),
                               module_results=file_proof(directory/'execution/results.json'))
                else:
                    log = directory/(name+'-run.log')
                    log.write_text(f'test {test} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n' if package else
                                   f'PASS [ 0.1s] {binary} {test}\nSummary [ 0.1s] 1 test run: 1 passed\n')
                    row.update(run_command=['cargo',*contract['run']] if package else ['cargo','nextest','run',*options,'--no-fail-fast'],
                               run_output=file_proof(log))
                rows.append(row)
            receipt = {'tier':'tf','batch':True,'mode':'four-repository','required_owners':list(REQUIRED_OWNERS),
                       'identities':identities,'identities_after':copy.deepcopy(identities), 'outcome':'pass',
                       'filters':[],'list_only':False,'required_legs':list(REQUIRED_LEGS),'results':rows}
            self.assertTrue(validate_four_receipt(receipt))
            for mutation in ('list-only', 'filtered', 'missing-owner', 'missing-hash', 'changed-hash',
                             'missing-leg', 'duplicate-leg', 'zero-selection', 'not-run', 'failure',
                             'no-execution-proof', 'short-ids', 'changed-log', 'filtered-command', 'missing-cwd'):
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
                elif mutation == 'no-execution-proof': bad['results']=[{'leg':r['leg'],'run_exit':0,'selected':1} for r in bad['results']]
                elif mutation == 'short-ids': bad['results'][-1]['selected_ids']=[]
                elif mutation == 'changed-log': bad['results'][-1]['run_output']['sha256']='0'*64
                elif mutation == 'filtered-command': bad['results'][-1]['run_command'].append('only-one-test')
                elif mutation == 'missing-cwd': bad['results'][-1].pop('cwd')
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    validate_four_receipt(bad)

    def test_web_legs_require_real_complete_module_results_and_executed_counts(self):
        from web_delivery import file_proof
        with tempfile.TemporaryDirectory() as folder, complete_module_protocol(folder) as fixture:
            root, directory, catalog, selection, results, save = fixture
            self.assertEqual([r['leg'] for r in collect_web_legs(directory)], list(MODULE_LEGS))
            for mutation in ('missing-result','duplicate-result','missing-selected-id','partial-execution',
                             'list-instead-of-run','filtered-after-list','short-applicable','duplicate-applicable',
                             'unexplained-exclusion','changed-output'):
                selected, executed = copy.deepcopy(selection), copy.deepcopy(results)
                row = next(r for r in executed['results'] if r['outcome']=='PASS' and len(r['applicable'])>1)
                if mutation=='missing-result': executed['results'].pop()
                elif mutation=='duplicate-result': executed['results'].append(executed['results'][-1])
                elif mutation=='missing-selected-id': selected['obligations'].pop()
                elif mutation=='partial-execution': executed['complete']=False
                elif mutation=='list-instead-of-run': row['command'][2]='list'
                elif mutation=='short-applicable':
                    row['applicable'].pop()
                    # Consistent shortened per-ID output and total cannot excuse a missing obligation.
                    log=Path(row['output']['path']); original=log.read_bytes()
                    log.write_text(''.join('PASS [ 0.1s] '+i.split('|',1)[1].replace('|',' ',1)+'\n' for i in row['applicable'])+
                                   f"Summary [ 0.1s] {len(row['applicable'])} tests run: {len(row['applicable'])} passed\n")
                    row['output']=file_proof(log)
                elif mutation=='duplicate-applicable': row['applicable'].append(row['applicable'][0])
                elif mutation=='unexplained-exclusion':
                    row.update(outcome='NOT_APPLICABLE',exit=None,applicable=[],condition_excluded=[])
                elif mutation in ('filtered-after-list','changed-output'):
                    row['output']['sha256']='0'*64
                save(selected,executed)
                with self.subTest(mutation=mutation), self.assertRaises(ValueError): collect_web_legs(directory)
                if mutation=='short-applicable': log.write_bytes(original)
            save(selection,results)
            self.assertEqual(len(collect_web_legs(directory)),4)

    def test_ephemeral_catalog_rechecks_source_contracts_at_consumption(self):
        with tempfile.TemporaryDirectory() as folder, complete_module_protocol(folder) as fixture:
            root,directory,catalog,selection,report,save=fixture
            refreshed=copy.deepcopy(catalog)
            refreshed['refresh_provenance']='new runtime discovery, unchanged complete protocol'
            path=directory/'ephemeral.json';path.write_text(json.dumps(refreshed))
            selection.update(catalog_path=str(path),catalog_sha256=digest(refreshed))
            save()
            report['catalog_sha256']=digest(refreshed)
            (directory/'execution/results.json').write_text(json.dumps(report))
            with patch('catalog_contract.verify_catalog') as verify:
                self.assertEqual(len(collect_web_legs(directory)),4)
                verify.assert_called_once_with(refreshed,root)
            with patch('catalog_contract.verify_catalog',side_effect=ValueError('source obligation missing')):
                with self.assertRaisesRegex(ValueError,'source obligation missing'): collect_web_legs(directory)

    def test_public_fixture_rejects_no_execution_proof_without_a_fake_pass(self):
        import importlib.util
        import subprocess
        script=Path(__file__).resolve().parents[1]/'web-repository-fixture.py'
        spec=importlib.util.spec_from_file_location('public_fixture_repair',script)
        fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
        lang=Path(__file__).resolve().parents[2]
        roots={name:lang if name=='language' else lang.parent/('auto-'+name) for name in (*REQUIRED_OWNERS,'down')}
        identities={name:fixture.full.identity(root) for name,root in roots.items()}
        receipt={'tier':'tf','batch':True,'mode':'four-repository','required_owners':list(REQUIRED_OWNERS),
                 'identities':identities,'identities_after':copy.deepcopy(identities),'outcome':'pass',
                 'filters':[],'list_only':False,'required_legs':list(REQUIRED_LEGS),
                 'results':[{'leg':name,'run_exit':0,'selected':1} for name in REQUIRED_LEGS]}
        with tempfile.TemporaryDirectory() as folder:
            gate,output=Path(folder)/'gate.json',Path(folder)/'result.json'
            gate.write_text(json.dumps(receipt))
            result=subprocess.run([sys.executable,str(script),'run','--lang-repo',str(lang),
                                   '--gate-receipt',str(gate),'--require-complete','--output',str(output)],
                                  cwd=lang,capture_output=True,text=True,timeout=60)
            self.assertEqual(result.returncode,2,result.stdout+result.stderr)
            report=json.loads(output.read_text())
            check=next(r for r in report['checks'] if r['id']=='four-owner-actual-execution')
            self.assertEqual(check['status'],'FAIL',check)
            self.assertIn('owner/list command',check['detail'])
            self.assertEqual(report['outcome'],'fail')

    def test_periodic_consumers_accept_the_complete_current_recipe_manifest(self):
        from module_catalog import load,ROOT
        from module_command import schedule_seed_modules
        catalog=load(ROOT/'scripts/test-governance/modules.json')
        # Catalog refresh follows changed source; manifests already contain every
        # current source recipe and both consumer entrypoints must retain them.
        recipes=load(ROOT/'scripts/test-governance/recipes.json')
        probe={**catalog,'recipes':recipes}
        for name in ('periodic-module-schedule.json','periodic-four-repository-schedule.json'):
            path=ROOT/'scripts/test-governance'/name
            ids=schedule_seed_modules(probe,path)
            self.assertEqual(set(ids),{m['id'] for m in catalog['modules']})
            self.assertEqual(set(load(path)['recipes']),set(recipes))

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
