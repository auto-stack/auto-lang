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


def owner_protocol_ids(full, name, catalog):
    """Preserve the real original denominator; added protocol IDs are synthetic."""
    floor = full.tier_minimums()[name]['minimum']
    if name in ('language', 'ui-full'):
        ids, selected = full.baseline_for('tf', name)
        if name == 'language':
            additions = sorted(r['binary']+'\t'+r['test'] for r in catalog['obligations']
                               if r['recipe']=='language' and not r['ignored'] and not r.get('conditions'))
        else:
            additions = ['protocol-additions\tadditional_'+str(i) for i in range(floor)]
        for identity in additions:
            if len(selected) >= floor: break
            selected.add(identity)
        return ids | selected, selected
    package = 'auto-ui-compiler' if name == 'ui-tu' else 'protocol-owner'
    ids = {package+'\tvalid_control_'+str(i) for i in range(floor)}
    return ids, ids.copy()


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
        import importlib.util
        spec = importlib.util.spec_from_file_location('protocol_baseline', ROOT/'scripts/run-full-regression.py')
        full = importlib.util.module_from_spec(spec); spec.loader.exec_module(full)
        _, language_selected = owner_protocol_ids(full, 'language', actual)
        item = next(m for m in actual['modules'] if m['recipe']=='language' and not m['conditions'])
        module = copy.deepcopy(item)
        module.update(id='language/protocol-control',
                      obligations=sorted('language|'+i.replace('\t','|',1) for i in language_selected))
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
        traces = {'invocations':[]}
        for r in rows:
            metadata = ([{'family':'legacy-vm-cache', **r['legacy_input']}] if r.get('legacy_input') else [])
            metadata += r.get('legacy_children', [])
            traces['invocations'].extend({'metadata':m, 'rust_test':r['test'], 'complete':True,
                                          'observation':{'status':'PASS'}} for m in metadata)
        row.update(listing=file_proof(listing), output=file_proof(log), child_observations=traces,
                   child_validation=validate_bindings(rows,traces))
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
            man = '4'*40
            names = ['auto-web', 'auto-web-contract', 'auto-lang-core', 'auto-generation-core', 'a2r-std', 'auto-media-core', 'auto-tool-execution',
                     *('sdk-dep-'+str(i) for i in range(13))]
            manifest = {'schema': 'web-source/1', 'candidate': False, 'sources': sources,
                        'source_sha256': fhash(files), 'files': files, 'tool_producer': {'source_commit': '3'*40},
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
                elif name == 'independent-installed-closure': row['detail'] = {'local_packages': 20}
                elif name == 'interrupted-generation-recovered': row['detail'] = {'checkpoint': 'public Web workspace phase'}
                else:
                    log.write_text((''.join(f'PASS [ 0.1s] sdk-binary sdk_case_{i}\n' for i in range(217))+'Summary [ 1s] 217 tests run: 217 passed, 2 skipped\n') if name == 'independent-sdk-tests' else 'real command output\n')
                    code = 2 if name in ('failed-swap', 'missing-resource-rejected', 'bad-hash-rejected', 'bad-version-rejected') else 1 if name.endswith('public-rejection') else 0
                    row.update(command=['tool', '--source', str(source)], exit=code, expected_exit=code,
                               log=str(log), log_sha256=h(log))
                checks.append(row)
            receipt = {'schema': 'web-repository-distribution/1', 'outcome': 'pass', 'candidate': False, 'pending': [],
                       'consumer_sources': {'auto-man': man},
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
                self.assertTrue(validate_distribution_receipt(receipt,path,{'auto-lang':root/'lang','auto-web':root/'web','auto-man':root/'man'}))
                self.assertEqual(git.call_count,6)
                self.assertTrue(any('crates/auto-lang-core' in call.args[0] for call in git.call_args_list))
            with patch('subprocess.check_output', side_effect=[b'', b'', b'', b'', b'core/Cargo.lock\n', b'']):
                with self.assertRaisesRegex(ValueError, 'CLI consumer inputs differ'):
                    validate_distribution_receipt(receipt,path,{'auto-lang':root/'lang','auto-web':root/'web','auto-man':root/'man'})
            wrong_consumer = copy.deepcopy(receipt)
            wrong_consumer['consumer_sources']['auto-man'] = manifest['tool_producer']['source_commit']
            with self.assertRaisesRegex(ValueError, 'producer cohort differs'):
                validate_distribution_receipt(wrong_consumer, path)
            original_manifest = (source/'source-manifest.json').read_bytes()
            missing = copy.deepcopy(manifest)
            missing['source_closure']['packages'] = [r for r in missing['source_closure']['packages'] if r['name'] != 'auto-tool-execution']
            (source/'source-manifest.json').write_text(json.dumps(missing))
            matching = copy.deepcopy(receipt)
            next(r for r in matching['checks'] if r['id'] == 'independent-installed-closure')['detail']['local_packages'] = 19
            with self.assertRaisesRegex(ValueError, 'source closure'):
                validate_distribution_receipt(matching, path)
            (source/'source-manifest.json').write_bytes(original_manifest)
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

    @contextlib.contextmanager
    def four_owner_protocol(self):
        from web_delivery import file_proof, read_proof
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
                ids, active = owner_protocol_ids(full, name, catalog)
                listing = directory/(name+'-list.json')
                suites = {}
                for identity in sorted(ids):
                    binary,test = identity.split('\t',1)
                    suites.setdefault(binary, {'testcases':{}})['testcases'][test] = {
                        'ignored':identity not in active, 'filter-match':{'status':'matches'}}
                listing.write_text(''.join(i.split('\t',1)[1]+(': test\n' if i in active else ': ignore\n') for i in sorted(ids))
                                   if package else json.dumps({'rust-suites':suites}))
                options = contract.get('options',[])
                row = {'leg':name,'cwd':str(contract['cwd']),'list_exit':0,'run_exit':0,
                       'selected':len(active),'selected_ids':sorted(active), 'listing':file_proof(listing),
                       'list_command':['cargo',*contract['list']] if package else ['cargo','nextest','list',*options,'--message-format','json']}
                if name=='language':
                    row.update(module_directory=str(directory), module_exit=0,
                               module_command=full.module_queue_command(root,'tf',[],True,directory,False,True),
                               module_selection=file_proof(directory/'selection.json'),
                               module_results=file_proof(directory/'execution/results.json'))
                else:
                    log = directory/(name+'-run.log')
                    log.write_text(''.join('test '+i.split('\t',1)[1]+' ... ok\n' for i in sorted(active))+
                                   f'test result: ok. {len(active)} passed; 0 failed; 0 ignored;\n' if package else
                                   ''.join('PASS [ 0.1s] '+i.replace('\t',' ',1)+'\n' for i in sorted(active))+
                                   f'Summary [ 0.1s] {len(active)} tests run: {len(active)} passed\n')
                    row.update(run_command=['cargo',*contract['run']] if package else ['cargo','nextest','run',*options,'--no-fail-fast'],
                               run_output=file_proof(log))
                rows.append(row)
            receipt = {'tier':'tf','batch':True,'mode':'four-repository','required_owners':list(REQUIRED_OWNERS),
                       'identities':identities,'identities_after':copy.deepcopy(identities), 'outcome':'pass',
                       'filters':[],'list_only':False,'required_legs':list(REQUIRED_LEGS),'results':rows}
            yield receipt, full, directory

    def test_four_owner_verdict_rejects_partial_missing_duplicate_and_changed_sources(self):
        with self.four_owner_protocol() as (receipt, full, directory):
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

    def test_owner_denominators_and_migration_credit_survive_public_consumption(self):
        from web_delivery import file_proof, read_proof
        import importlib.util
        with self.four_owner_protocol() as (receipt, full, directory):
            rows, identities = receipt["results"], receipt["identities"]
            # Exercise the real public consumer too. Only its external Git
            # fingerprint is substituted, as for the module protocol fixture;
            # source/resource/Cargo and every receipt validator remain real.
            import types
            spec = importlib.util.spec_from_file_location('public_owner_denominator',
                       Path(__file__).resolve().parents[1]/'web-repository-fixture.py')
            public = importlib.util.module_from_spec(spec); spec.loader.exec_module(public)
            lang = Path(__file__).resolve().parents[2]
            def public_check(value):
                gate, output = directory/'gate.json', directory/'public-result.json'
                gate.write_text(json.dumps(value))
                args = types.SimpleNamespace(lang_repo=str(lang), web_repo=str(lang.parent/'auto-web'),
                    ui_repo=str(lang.parent/'auto-ui'), man_repo=str(lang.parent/'auto-man'),
                    down_repo=str(lang.parent/'auto-down'), phase='source-switched', gate_receipt=gate,
                    distribution_receipt=None, require_complete=True, output=output)
                def identity(path):
                    name = Path(path).name
                    return identities['language' if name=='auto-lang' else name.removeprefix('auto-')]
                with patch.object(public.full, 'identity', side_effect=identity):
                    self.assertEqual(public.run(args), 2)  # Distribution deliberately pending.
                report = json.loads(output.read_text())
                return next(r for r in report['checks'] if r['id']=='four-owner-actual-execution')
            self.assertEqual(public_check(receipt)['status'], 'PASS')
            # Actual libtest --list prints even original ignored tests as
            # ': test'. Their same-source ui-full flags retain N/A, not PASS.
            ui = next(r for r in rows if r['leg']=='ui-full')
            single = next(r for r in rows if r['leg']=='ui-tu')
            expected, required = full.baseline_for('tf','ui-full')
            ignored = sorted(i for i in expected-required if '\tui_gen::' in i)
            self.assertTrue(ignored)
            single_list = Path(single['listing']['path'])
            with single_list.open('a') as stream:
                stream.write(''.join(i.split('\t',1)[1]+': test\n' for i in ignored))
            single.update(listing=file_proof(single_list), condition_excluded=ignored)
            self.assertTrue(validate_four_receipt(receipt))
            single['condition_excluded'] = ignored + [single['selected_ids'][0]]
            with self.assertRaisesRegex(ValueError,'original ignored identities differ'):
                validate_four_receipt(receipt)
            single['condition_excluded'] = ignored
            for name, mutation in [('ui-full','omit'), ('ui-full','ignore'), ('ui-full','filter'),
                                   ('language','omit'), ('man-t','omit'), ('ui-tu','omit')]:
                row = next(r for r in rows if r['leg']==name)
                saved = copy.deepcopy(row)
                listing = Path(row['listing']['path']); original = listing.read_bytes()
                log = Path(row['run_output']['path']) if name!='language' else None
                original_log = log.read_bytes() if log else None
                package = 'auto-ui-compiler' if name=='ui-tu' else None
                victim = (sorted(full.baseline_for('tf',name)[1])[0] if name in ('language','ui-full')
                          else row['selected_ids'][0])
                binary, test = victim.split('\t',1)
                try:
                    if package:
                        listing.write_text(''.join(line+'\n' for line in original.decode().splitlines()
                                                   if line != test+': test'))
                        listed, active = full.libtest_selection(listing.read_text(),package)
                        active -= full.libtest_original_ignores(listed,package,json.loads(read_proof(ui['listing'])))
                    else:
                        document = json.loads(original)
                        case = document['rust-suites'][binary]['testcases']
                        if mutation=='omit': del case[test]
                        elif mutation=='ignore': case[test]['ignored']=True
                        else: case[test]['filter-match']['status']='mismatch'
                        listing.write_text(json.dumps(document))
                        _, active = full.selection(document)
                    self.assertEqual(len(active), saved['selected']-1, 'mutation must remove one selected ID')
                    row.update(listing=file_proof(listing), selected=len(active), selected_ids=sorted(active),
                               migrated_original_required=1000000)  # Never accepted as migration credit.
                    if log:
                        log.write_text(''.join('test '+i.split('\t',1)[1]+' ... ok\n' for i in sorted(active))+
                                       f'test result: ok. {len(active)} passed; 0 failed; 0 ignored;\n' if package else
                                       ''.join('PASS [ 0.1s] '+i.replace('\t',' ',1)+'\n' for i in sorted(active))+
                                       f'Summary [ 0.1s] {len(active)} tests run: {len(active)} passed\n')
                        row['run_output']=file_proof(log)
                    with self.subTest(owner=name, mutation=mutation), self.assertRaisesRegex(
                            ValueError, 'baseline coverage lost|below frozen minimum'):
                        validate_four_receipt(receipt)
                    if (name,mutation) in [('ui-full','omit'),('man-t','omit')]:
                        self.assertEqual(public_check(receipt)['status'],'FAIL')
                finally:
                    listing.write_bytes(original)
                    if log: log.write_bytes(original_log)
                    row.clear();row.update(saved)
            self.assertTrue(validate_four_receipt(receipt))
            # Source-derived migration credit is legal only with actual passed
            # destination IDs. Presence in a catalog/selection is insufficient.
            executed = {i for r in rows if r['leg'] in MODULE_LEGS for i in r['selected_ids']}
            for name in ('language','ui-full'):
                row = next(r for r in rows if r['leg']==name)
                ids, active = full.selection(json.loads(read_proof(row['listing'])))
                _, credit = full.validate_owner_selection('tf',name,ids,active,full.tier_minimums(),
                                   with_web=True,executed_migrations=executed)
                self.assertGreater(credit,0)
                with self.assertRaisesRegex(ValueError,'migrated original required tests were not executed'):
                    full.validate_owner_selection('tf',name,ids,active,full.tier_minimums(),
                                                  with_web=True,executed_migrations=set())

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

    def test_python_and_negative_case_proofs_preserve_original_oracles(self):
        from module_catalog import ROOT,load
        from web_delivery import validate_module_result,file_proof
        from execution_trace import validate_bindings
        from run_selected import validate_case_result
        from recipe_runtime import list_command,python_run_command
        catalog=load(ROOT/'scripts/test-governance/modules.json')
        lookup={r['id']:r for r in catalog['obligations']}
        python=next(m for m in catalog['modules'] if m['recipe']=='infra-modules')
        negative=next(m for m in catalog['modules'] if catalog['recipes'][m['recipe']].get('runner')=='case-suite'
                      and any(lookup[i]['expected_status']=='FAIL' for i in m['obligations']))
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            for module in (python,negative):
                rows=[lookup[i] for i in module['obligations']]
                recipe=catalog['recipes'][module['recipe']]
                listing,log=root/'list.json',root/'run.log'
                command=list_command(recipe,ROOT)
                row={'module':module['id'],'obligations':sorted(module['obligations']),
                     'applicable':sorted(module['obligations']),'condition_environment':{},'condition_excluded':[],
                     'outcome':'PASS','exit':0,'cwd':str(ROOT),'list_command':command,
                     'child_observations':{'invocations':[]},'child_validation':validate_bindings(rows,{'invocations':[]})}
                if module is python:
                    listing.write_text(json.dumps({'schema':'python-unittest-discovery/1','tests':[{'id':r['test'],'ignored':False} for r in rows]}))
                    proof={'schema':'python-unittest-result/1','selected':[r['test'] for r in rows],
                           'executed':len(rows),'skipped':0,'failures':0,'errors':0}
                    log.write_text(json.dumps(proof))
                    row.update(command=python_run_command(recipe,ROOT,rows),execution=proof)
                else:
                    checks=[]
                    for r in rows:
                        checks.append({**{k:r[k] for k in ('adapter','profile','source_sha256','oracle_sha256','case_file_sha256','span','product_from','state')},
                                       'suite':r['suite_id'],'case':r['case_id'],'check':r['check_id'],'recipe':r['case_recipe'],
                                       'case_file':str(ROOT/r['case_file'])})
                    listing.write_text(json.dumps({'schema':'test-case-inventory/1','checks':checks}))
                    raw_exit=1
                    results=[{'case':r['case_id'],'check':r['check_id'],'status':r['expected_status'],
                              'mismatches':['deliberate negative oracle'] if r['expected_status']=='FAIL' else [],
                              'identity':{'source':r['source_sha256'],'oracle':r['oracle_sha256']}} for r in rows]
                    case=root/'module-case.json'
                    case.write_text(json.dumps({'schema':'test-case-module-result/1','scope':'case',
                                    'selected_gate_exit':raw_exit,'selected_report':{'results':results,'load_errors':[],'coverage_gaps':[]},'unselected':[]}))
                    log.write_text('original case exit1; negative meta-oracle satisfied\n')
                    row.update(command=[command[0],'module-case',*command[2:],'--case',rows[0]['case_id'],
                                        '--out',str(root/'original-runtime'),'--artifact-out',str(root.resolve())],
                               case_artifacts={'original_runtime_root':str(root/'original-runtime')},case_report=file_proof(case),
                               execution=validate_case_result(case,rows,raw_exit))
                    self.assertTrue(row['execution']['assertions_satisfied'])
                row.update(listing=file_proof(listing),output=file_proof(log))
                validate_module_result(catalog,module,row)
                bad=copy.deepcopy(row);bad['command'].append('only-one')
                with self.assertRaisesRegex(ValueError,'command differs'): validate_module_result(catalog,module,bad)
                if module is negative:
                    changed=json.loads(case.read_text());changed['selected_report']['results'][0]['identity']['oracle']='0'*64
                    case.write_text(json.dumps(changed));bad=copy.deepcopy(row);bad['case_report']=file_proof(case)
                    with self.assertRaisesRegex(ValueError,'atomic oracles'): validate_module_result(catalog,module,bad)

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
