"""Source-derived external-input bindings, with a non-regenerating association guard."""
import copy
import hashlib
from pathlib import Path

from module_catalog import ROOT,load


def normalize(value):
    if isinstance(value,list):return [normalize(v) for v in value]
    if isinstance(value,dict):
        result={}
        for key,item in value.items():
            if key=='path' and isinstance(item,str):
                path=Path(item.removeprefix('\\\\?\\')).resolve()
                if not path.is_relative_to(ROOT.resolve()):raise ValueError('legacy wrapper input escapes repository')
                result[key]=path.relative_to(ROOT.resolve()).as_posix()
            else:result[key]=normalize(item)
        return result
    return value


def shape(value):
    if isinstance(value,list):return [shape(v) for v in value]
    if isinstance(value,dict):
        if 'path' in value:return {'path':value['path']}
        return {k:shape(v) for k,v in value.items() if k not in ('comparison','expected_error_substring_sha256')}
    return value


def associations(legacy,vm):
    rows=[{'kind':'vm-cache','test':r['test'],'id':r['cache_id']} for r in vm['declared_wrapper_bindings']]
    rows += [{'kind':'legacy-trace','test':r['test'],'children':shape(r['children'])}
             for r in legacy['wrapper_inventory']['bindings']]
    return sorted(rows,key=lambda r:(r['test'],r['kind']))


def check_files(value):
    if isinstance(value,list):
        for item in value:check_files(item)
    elif isinstance(value,dict):
        if 'path' in value:
            path=(ROOT/value['path']).resolve()
            if not path.is_relative_to(ROOT.resolve()):raise ValueError('wrapper file escaped repository')
            if value.get('sha256'):
                if hashlib.sha256(path.read_bytes()).hexdigest()!=value['sha256']:raise ValueError('wrapper source/oracle fingerprint stale')
            elif value.get('read_error') and path.exists():raise ValueError('unreadable existing wrapper input cannot be admitted')
        else:
            for item in value.values():check_files(item)


def apply(catalog,verify=False):
    inputs=catalog['input_inventory'];legacy=inputs['legacy-inputs'];vm=inputs['vm-cache-inputs']
    inventory=legacy['wrapper_inventory']
    contract=load(ROOT/'scripts/test-governance/legacy-wrapper-contract.json')
    if contract.get('schema')!='test-legacy-wrapper-associations/1':raise ValueError('wrapper association contract missing')
    actual={(r['test'],r['kind']):r for r in associations(legacy,vm)}
    for expected in contract['required']:
        if actual.get((expected['test'],expected['kind']))!=expected:
            raise ValueError(f"original wrapper/input association lost: {expected['test']}")
    for source in inventory['sources']:
        if hashlib.sha256((ROOT/source['path']).read_bytes()).hexdigest()!=source['sha256']:
            raise ValueError('legacy wrapper AST source stale')
    bindings={r['test']:r['children'] for r in inventory['bindings']}
    registered={r['test']:r for r in catalog['obligations'] if r['recipe']=='language' and r['binary']=='auto-lang'}
    for row in inventory['unresolved']:
        if row['test'] in registered and not registered[row['test']]['ignored']:
            raise ValueError(f"unresolved applicable original wrapper: {row}")
    for name,row in registered.items():
        expected=bindings.get(name,[])
        if verify:
            if row.get('legacy_children',[])!=expected:raise ValueError('wrapper child binding lost/altered')
        elif expected:row['legacy_children']=copy.deepcopy(expected)
        if expected:check_files(expected)
        paths=[]
        def collect(value):
            if isinstance(value,list):
                for item in value:collect(item)
            elif isinstance(value,dict):
                if 'path' in value:paths.append(value['path'])
                else:
                    for item in value.values():collect(item)
        collect(expected)
        if paths:
            wanted=sorted(set(paths))
            if verify:
                if row.get('input_paths')!=wanted:raise ValueError('wrapper file consumer edges lost')
            else:row['input_paths']=wanted
