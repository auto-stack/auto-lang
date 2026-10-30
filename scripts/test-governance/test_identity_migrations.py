"""Revision-cited test moves preserve identities and applicability, never floors."""
from pathlib import Path
import json
import hashlib
import re

PATH = Path(__file__).with_name("web-test-migrations.json")


def media_records(root=None, catalog=None):
    """The UI photo tests moved to the existing neutral language media owner."""
    from process_capture import checked_output
    root = Path(root) if root else PATH.parents[2]
    document = json.loads((root / 'scripts/test-governance/media-test-migrations.json').read_text(encoding='utf-8'))
    rows = document['migrations']
    if (document.get('schema') != 'media-test-migrations/1' or document.get('plan') != 890
            or len(rows) != 10 or len({r['old'] for r in rows}) != 10
            or len({r['new'] for r in rows}) != 10):
        raise ValueError('original photo test identities lost/duplicated')
    cache = {}
    actual = None if catalog is None else {r['id']: r for r in catalog['obligations']}
    for row in rows:
        if (row['source'] != 'ui-full' or row['target'] != 'media-unit'
                or row['original_ignored'] is not False or not row.get('reason')
                or row['source_path'] != 'crates/auto-ui-runtime/src/ui/photo_service.rs'
                or row['target_path'] != 'crates/auto-media-core/src/photo_service.rs'):
            raise ValueError('invalid original photo test owner/applicability')
        identity = row['source_commit'] + ':' + row['source_path']
        if identity not in cache:
            cache[identity] = checked_output(['git', '-C', str(root.parent / 'auto-ui'), 'show', identity])
        raw = cache[identity]
        name = row['old'].split('\t')[1].split('::')[-1]
        if (hashlib.sha256(raw).hexdigest() != row['source_sha256']
                or ('fn ' + name + '(').encode() not in raw
                or row['new'] != 'auto-media-core\tphoto_service::tests::' + name
                or ('fn ' + name + '(') not in (root / row['target_path']).read_text(encoding='utf-8')):
            raise ValueError('original photo test source/destination identity differs')
        target = 'media-unit|' + row['new'].replace('\t', '|')
        if actual is not None and (target not in actual or actual[target]['ignored']):
            raise ValueError('original photo test missing/suppressed in media module')
    return rows


def key(row):
    return f"{row['recipe']}|{row['binary']}|{row['test']}"


def records(path=PATH):
    if not path.is_file():
        return []
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("schema") != "test-identity-migrations/1" or document.get("plan") != 890:
        raise ValueError("unsupported Web test migration contract")
    rows = document["migrations"]
    if len({key(r['old']) for r in rows}) != len(rows) or len({key(r['new']) for r in rows}) != len(rows):
        raise ValueError("duplicate test migration source or destination")
    for row in rows:
        if (not row.get("reason") or not row.get("source_commit") or not row.get("source_path")
                or not row.get("target_path") or not row.get("source_sha256")
                or type(row.get("original_ignored")) is not bool
                or not re.fullmatch(r'[0-9a-f]{64}', row['source_sha256'])
                or not re.fullmatch(r'[0-9a-f]{40}', row['source_commit'])):
            raise ValueError("test migration lacks original source/applicability evidence")
        if row['new']['recipe'] not in ("language", "web-unit"):
            raise ValueError("unknown test migration destination")
    return rows


def validate_sources(root, path=None):
    from process_capture import checked_output
    from repository_layout import relative_path
    path = path or Path(root) / 'scripts/test-governance/web-test-migrations.json'
    cache = {}
    for row in records(path):
        source = row['source_path']
        if not source.startswith('crates/') or '..' in Path(source).parts:
            raise ValueError('migration original source escapes language repository')
        identity = (row['source_commit'], source)
        if identity not in cache:
            cache[identity] = checked_output(['git', '-C', str(root), 'show', ':'.join(identity)])
        raw = cache[identity]
        if hashlib.sha256(raw).hexdigest() != row['source_sha256']:
            raise ValueError('migration original source hash differs')
        target = Path(root) / row['target_path']
        relative_path(root, target)
        if ('fn '+row['old']['test'].split('::')[-1]+'(').encode() not in raw:
            raise ValueError('migration old test absent from its cited source')
        if ('fn '+row['new']['test'].split('::')[-1]+'(') not in target.read_text(encoding='utf-8'):
            raise ValueError('migration destination test absent from its source')


def translated(identity):
    return {key(r['old']): key(r['new']) for r in records()}.get(identity, identity)


def legacy_ids(contract, tier):
    return {translated(f"language|{row['binary']}|{row['test']}")
            for row in contract['rows'] if tier in row['selected_in']}


def validate_resources(root, path=None):
    from repository_layout import relative_path
    path = path or Path(root) / 'scripts/test-governance/web-resource-ownership.json'
    if not path.is_file():
        recipes = Path(root) / 'scripts/test-governance/recipes.json'
        if recipes.is_file() and 'web-unit' in json.loads(recipes.read_text(encoding='utf-8')):
            raise ValueError('required Web resource ownership contract missing')
        return
    document = json.loads(path.read_text(encoding='utf-8'))
    rows = document['entries']
    if (document.get('schema') != 'web-resource-ownership/1' or len(rows) != 17
            or len({r['original_path'] for r in rows}) != len(rows)
            or len({r['path'] for r in rows}) != len(rows)):
        raise ValueError('original resource contract lost/duplicated inputs')
    for row in rows:
        source = Path(root) / row['path']
        relative_path(root, source)
        if not source.is_file() or hashlib.sha256(source.read_bytes()).hexdigest() != row['sha256']:
            raise ValueError('original resource missing or changed: '+row['path'])
        if row.get('owner') == 'web' and (Path(root) / row['original_path']).exists():
            raise ValueError('moved Web resource still has an old duplicate owner')


def validate_historical_obligations(catalog, root, path=None):
    from process_capture import checked_output
    path = path or Path(root) / 'scripts/test-governance/web-historical-obligations.json'
    if not path.is_file():
        recipes = Path(root) / 'scripts/test-governance/recipes.json'
        if recipes.is_file() and 'web-unit' in json.loads(recipes.read_text(encoding='utf-8')):
            raise ValueError('required historical Web obligation contract missing')
        return
    document = json.loads(path.read_text(encoding='utf-8'))
    rows = document['entries']
    if (document.get('schema') != 'web-original-obligations/1'
            or document['historical_rows'] != 696 or len(rows) != 116
            or len({r['old'] for r in rows}) != len(rows)
            or len({r['target'] for r in rows}) != len(rows)):
        raise ValueError('historical Web obligation loss/duplication')
    actual = {r['id'] for r in catalog['obligations']}
    total = 0
    for tier, expected_hash in document['original_manifest_hashes'].items():
        original = checked_output(['git', '-C', str(root), 'show',
                    document['source_commit']+':scripts/full-regression-'+tier+'-ids.txt'])
        if hashlib.sha256(original).hexdigest() != expected_hash:
            raise ValueError('historical manifest identity differs')
        identities = set(original.decode('utf-8').splitlines())
        selected = [r for r in rows if tier in r['historical_tiers']]
        if any(r['old'] not in identities or r['target'] not in actual for r in selected):
            raise ValueError('original Web ID omitted from its source/destination')
        total += len(selected)
    if total != document['historical_rows']:
        raise ValueError('historical tier membership count differs')
