#!/usr/bin/env python3
"""PLAN-890 four-repository source, ID, execution and delivery receipt checks."""
from __future__ import annotations
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'scripts/test-governance'))
from test_identity_migrations import validate_sources, validate_resources, validate_historical_obligations
from web_delivery import validate_distribution_receipt
import four_repository

spec = importlib.util.spec_from_file_location('full_regression', ROOT / 'scripts/run-full-regression.py')
full = importlib.util.module_from_spec(spec)
spec.loader.exec_module(full)


def resolved_roots(args):
    lang = Path(args.lang_repo or os.environ.get('AUTO_LANG_ROOT') or ROOT).resolve()
    result = {'lang': lang}
    for name in ('web', 'ui', 'man', 'down'):
        explicit = getattr(args, name+'_repo') or os.environ.get('AUTO_'+name.upper()+'_ROOT')
        sibling = lang.parent / ('auto-'+name)
        result[name] = Path(explicit).resolve() if explicit else (sibling if sibling.is_dir() else Path('D:/autostack/auto-'+name))
    return result


def run(args):
    roots = resolved_roots(args)
    receipt = {'schema': 'four-repository-fixture/1', 'plan': 'PLAN-890', 'phase': args.phase,
               'sources': {}, 'checks': [], 'pending': []}
    def check(name, action):
        try:
            detail = action()
            receipt['checks'].append({'id': name, 'status': 'PASS', 'detail': detail})
        except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
            receipt['checks'].append({'id': name, 'status': 'FAIL', 'detail': str(error)})

    for name, path in roots.items():
        check('identity-'+name, lambda name=name, path=path: receipt['sources'].setdefault(name, four_repository.snapshot(path)))
    check('migration-original-source-identities', lambda: validate_sources(roots['lang']) or 'Original Git source/hash and target test declarations match')
    check('original-resource-identities', lambda: validate_resources(roots['lang']) or 'All 17 original resources match their declared unique owner/hash')
    # Static provenance here; actual destination enumeration is mandatory in the
    # fixed four-owner receipt below, without consulting the retired leaf catalog.
    historical = json.loads((roots['lang']/'scripts/test-governance/web-historical-obligations.json').read_text(encoding='utf-8'))
    check('historical-Web-test-identities', lambda: validate_historical_obligations(
        {'obligations': [{'id': row['target']} for row in historical['entries']]}, roots['lang'])
        or '696 original tier rows / 116 unique IDs; actual destinations checked by execution gate')

    def closure():
        data = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=roots['web']))
        packages = {p['id']: p for p in data['packages']}
        nodes = {n['id']: n for n in data['resolve']['nodes']}
        seeds = [p['id'] for p in data['packages'] if p['name'] in ('auto-web', 'auto-web-contract')]
        seen, todo = set(), list(seeds)
        while todo:
            identity = todo.pop()
            if identity in seen: continue
            seen.add(identity)
            todo.extend(d['pkg'] for d in nodes[identity]['deps'])
        forbidden = [packages[i]['name'] for i in seen if packages[i]['name'].startswith('auto-ui')
                     or packages[i]['name'] in ('auto-lang', 'auto-api-gen', 'auto-man', 'auto-man-runtime', 'tauri')]
        if forbidden: raise ValueError('forbidden Web dependency closure: '+repr(forbidden))
        contract = next(p['id'] for p in packages.values() if p['name'] == 'auto-web-contract')
        pure, todo = set(), [contract]
        while todo:
            identity = todo.pop()
            if identity in pure: continue
            pure.add(identity)
            todo.extend(d['pkg'] for d in nodes[identity]['deps'])
        if any(packages[i]['name'] in ('auto-lang-core', 'auto-web', 'axum', 'hyper', 'tokio')
               or packages[i]['name'].startswith(('auto-ui', 'auto-man')) for i in pure):
            raise ValueError('pure contract closure contains an implementation')
        expected = {'auto-lang-core': roots['lang']/ 'crates/auto-lang-core',
                    'auto-web-contract': roots['web']/ 'crates/auto-web-contract',
                    'auto-generation-core': roots['lang']/ 'crates/auto-generation-core'}
        for name, directory in expected.items():
            found = [p for p in packages.values() if p['name'] == name]
            if len(found) != 1 or Path(found[0]['manifest_path']).parent.resolve() != directory.resolve():
                raise ValueError('missing/duplicate/wrong physical source: '+name)
        return {'packages': len(seen), 'pure_contract_packages': len(pure), 'forbidden': forbidden}
    check('independent-web-dependency-closure', closure)

    if args.gate_receipt:
        def gate():
            gate_receipt = json.loads(args.gate_receipt.read_text(encoding='utf-8'))
            four_repository.validate(gate_receipt, args.output.parent / (args.output.stem + '-admission'))
            expected = {('language' if name == 'lang' else name): four_repository.snapshot(path) for name, path in roots.items()}
            if gate_receipt['identities_after'] != expected:
                raise ValueError('gate source/lock/resource/producer hash differs from the actual selected repositories')
            return {'path': str(args.gate_receipt), 'required_legs': list(four_repository.EXTERNAL_LEGS),
                    'language_groups': gate_receipt['language_groups']}
        check('four-owner-actual-execution', gate)
    else:
        receipt['pending'].append('four-owner-actual-execution: supply the frozen --with-web gate receipt (T-12)')
    if args.distribution_receipt:
        def distribution():
            result = json.loads(args.distribution_receipt.read_text(encoding='utf-8'))
            validate_distribution_receipt(result, args.distribution_receipt,
                                          {'auto-lang': roots['lang'], 'auto-web': roots['web'], 'auto-man': roots['man']})
            return str(args.distribution_receipt)
        check('independent-distribution-and-recovery', distribution)
    else:
        receipt['pending'].append('independent-distribution-and-recovery: T-09 receipt required')
    failed = sum(c['status'] == 'FAIL' for c in receipt['checks'])
    receipt['summary'] = {'passed': len(receipt['checks'])-failed, 'failed': failed, 'pending': len(receipt['pending'])}
    receipt['outcome'] = 'fail' if failed else 'partial' if receipt['pending'] else 'pass'
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2, ensure_ascii=False)+'\n', encoding='utf-8')
    print(json.dumps(receipt['summary'])+' '+str(args.output))
    return 2 if failed or args.require_complete and receipt['pending'] else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['run'])
    parser.add_argument('--phase', choices=['source-switched'], default='source-switched')
    for name in ('lang', 'web', 'ui', 'man', 'down'): parser.add_argument('--'+name+'-repo')
    parser.add_argument('--gate-receipt', type=Path)
    parser.add_argument('--distribution-receipt', type=Path)
    parser.add_argument('--require-complete', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    return run(parser.parse_args())


if __name__ == '__main__':
    sys.exit(main())
