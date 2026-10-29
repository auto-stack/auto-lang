"""Real read-only input discovery, separate from executed test obligations."""
import hashlib
import os
from pathlib import Path
import subprocess

from module_catalog import ROOT,load


def discover_inputs(output,language):
    from prepare_tools import prepare
    prepare(ROOT.resolve(),["case-parent"],output/"tools",set())
    output=output.resolve()
    legacy=[str(ROOT/"target/debug"/("auto-case.exe" if os.name=="nt" else "auto-case")),
            "legacy-inventory","--test-root",str(ROOT/"crates/auto-lang/test")]
    binary=language["rust-suites"]["auto-lang"]["binary-path"]
    vm=[binary,"--exact","tests::vm_file_tests::inventory_vm_cache_read_only","--ignored"]
    records=[]
    for name,command,schema in [("legacy-inputs",legacy,"test-legacy-inventory/1"),
                                ("vm-cache-inputs",vm,"test-vm-cache-inventory/1")]:
        path=output/(name+".json")
        with (output/(name+".log")).open("wb") as log:
            if name=="legacy-inputs":
                with path.open("wb") as stream:result=subprocess.run(command,cwd=ROOT,stdout=stream,stderr=log,timeout=120)
            else:
                result=subprocess.run(command,cwd=ROOT,env={**os.environ,"AUTO_TEST_INVENTORY_OUT":str(path)},
                                      stdout=log,stderr=subprocess.STDOUT,timeout=120)
        if result.returncode:raise ValueError(f"read-only input discovery failed: {name}")
        value=load(path)
        if value.get("schema")!=schema or value.get("scope")!="discovery" or value.get("executed")!=0:
            raise ValueError("input metadata confused with body execution")
        records.append({"name":name,"path":path.name,"command":command,"exit":0,
                        "sha256":hashlib.sha256(path.read_bytes()).hexdigest()})
    return records


def attach_inputs(catalog,receipt,directory):
    def normalize(value,key=None):
        if isinstance(value,dict):return {k:normalize(v,k) for k,v in value.items()}
        if isinstance(value,list):return [normalize(v,key) for v in value]
        if isinstance(value,str) and key in ("path","source_path","root"):
            path=Path(value.removeprefix("\\\\?\\"))
            if path.is_absolute():
                path=path.resolve()
                if not path.is_relative_to(ROOT.resolve()):raise ValueError("input outside authorized repository")
                return path.relative_to(ROOT.resolve()).as_posix()
        return value
    inputs={}
    for record in receipt["inputs"]:
        path=directory/record["path"]
        if path.parent.resolve()!=directory.resolve() or record["exit"]!=0 or hashlib.sha256(path.read_bytes()).hexdigest()!=record["sha256"]:
            raise ValueError("input discovery receipt identity differs")
        inputs[record["name"]]=normalize(load(path))
    if set(inputs)!={"legacy-inputs","vm-cache-inputs"}:raise ValueError("incomplete input inventory")
    vm=inputs["vm-cache-inputs"]
    source=ROOT/"crates/auto-lang/src/tests/vm_file_tests.rs"
    if hashlib.sha256(source.read_bytes()).hexdigest()!=vm["wrapper_source_sha256"]:
        raise ValueError("VM wrapper declaration inventory stale")
    cases={row["id"]:row for row in vm["cases"]}
    wrappers={row["test"]:row for row in vm["declared_wrapper_bindings"]}
    registry={r['test']:r for r in catalog['obligations'] if r['recipe']=='language' and r['binary']=='auto-lang'}
    for binding in wrappers.values():
        binding['registered']=binding['test'] in registry
        binding['runtime_ignored']=registry[binding['test']]['ignored'] if binding['registered'] else None
    for row in catalog["obligations"]:
        if row["recipe"]=="language" and row["binary"]=="auto-lang" and row["test"] in wrappers:
            binding=wrappers[row["test"]]
            case=cases[binding["cache_id"]]
            if not binding["cache_present"] or binding["declared_ignore"]!=row["ignored"]:
                raise ValueError("VM declaration/runtime registry applicability differs")
            row["legacy_input"]={"kind":"vm-cache","id":case["id"],
                "source_sha256":case["source_sha256"],"oracle_sha256":case["oracle_sha256"]}
            path=Path(case["source_path"])
            stem=path.stem
            row["input_paths"]=[path.as_posix(),*[(path.parent/(stem+suffix)).as_posix()
                for suffix in (".expected.out",".expected.result",".expected.error") if (ROOT/path.parent/(stem+suffix)).is_file()]]
    catalog["input_inventory"]=inputs
    from wrapper_bindings import apply
    apply(catalog)


def verify_inputs(catalog):
    # Reconstruct bindings from the source-bound discovered metadata so a
    # selection cannot silently remove a wrapper's child identity.
    inputs=catalog["input_inventory"]
    from wrapper_bindings import apply
    apply(catalog,verify=True)
    contract=load(ROOT/'scripts/test-governance/input-contract.json')
    if contract.get('schema')!='test-input-universe/1':raise ValueError('unsupported input universe')
    vm=inputs["vm-cache-inputs"]
    legacy=inputs['legacy-inputs']
    if (set(contract['vm_cache_ids'])-{r['id'] for r in vm['cases']}
        or {tuple(r) for r in contract['legacy_ids']}-{(r['family'],r['id']) for r in legacy['cases']}):
        raise ValueError('frozen input universe lost')
    if hashlib.sha256((ROOT/"crates/auto-lang/src/tests/vm_file_tests.rs").read_bytes()).hexdigest()!=vm["wrapper_source_sha256"]:
        raise ValueError("input wrapper source stale")
    lookup={r["test"]:r for r in catalog["obligations"] if r["recipe"]=="language" and r["binary"]=="auto-lang"}
    cases={r["id"]:r for r in vm["cases"]}
    for case in cases.values():
        source=ROOT/case["source_path"]
        if hashlib.sha256(source.read_bytes()).hexdigest()!=case["source_sha256"]:
            raise ValueError("VM input source hash differs")
        oracle={"stdout":None,"result":None,"error_presence":source.with_suffix('.expected.error').is_file()}
        for key,suffix in (("stdout",".expected.out"),("result",".expected.result")):
            path=source.with_suffix(suffix)
            if path.is_file():oracle[key]=path.read_bytes().decode('utf-8')
        import json
        # serde_json preserve_order is enabled in this build. Keep the exact
        # producer order in its identity; compare reconstructed values first.
        if oracle!=case['oracle'] or hashlib.sha256(json.dumps(case['oracle'],separators=(',',':'),ensure_ascii=False).encode()).hexdigest()!=case["oracle_sha256"]:
            raise ValueError("VM input oracle hash differs")
    for binding in vm["declared_wrapper_bindings"]:
        row=lookup.get(binding["test"])
        case=cases[binding["cache_id"]]
        if binding['registered']!=(row is not None):raise ValueError("VM runtime registration changed")
        if row is None:continue
        if row['ignored']!=binding['runtime_ignored'] or row.get("legacy_input")!={"kind":"vm-cache","id":case["id"],"source_sha256":case["source_sha256"],"oracle_sha256":case["oracle_sha256"]}:
            raise ValueError("VM input binding lost or altered")
        source=Path(case["source_path"])
        paths=[source.as_posix(),*[source.with_suffix(suffix).as_posix() for suffix in
            ('.expected.out','.expected.result','.expected.error') if (ROOT/source.with_suffix(suffix)).is_file()]]
        if row.get("input_paths")!=paths:raise ValueError("VM input source/consumer edge lost")
