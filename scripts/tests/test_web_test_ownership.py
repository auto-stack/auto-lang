"""Distribution consumer counterexamples; original full oracle retained."""
from pathlib import Path
import sys
import tempfile
import unittest
import copy
import json
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))

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
