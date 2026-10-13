import importlib.util, json, subprocess, sys, shutil, hashlib
from pathlib import Path
repo=Path(sys.argv[1]).resolve()
out=Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=True)
tool=repo/"scripts/acc_inventory.py"
spec=importlib.util.spec_from_file_location("acc_inventory_review",tool)
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
def run(args):
    p=subprocess.run([sys.executable,"-B",str(tool),*map(str,args)],capture_output=True,text=True,encoding="utf-8",timeout=30)
    return dict(exit=p.returncode,stdout=p.stdout,stderr=p.stderr)
results={}
results["current_check"]=run(["--root",repo,"--output","docs/reports/743-acc-hir-contract","--check"])
for name, source in {
    "multiline-string":'fn host() {\n var text = "\n  pretend()\n "\n return 0\n}\n',
    "same-name-method":'type Meter {\n fn len() int { return 1 }\n static fn new() Meter { return Meter() }\n}\nfn demo(x Meter) int {\n var m = Meter.new()\n return x.len()\n}\n',
}.items():
    (out/(name+".at")).write_text(source,encoding="utf-8")
    results[name]=mod.scan_module(name+".at",source).to_json()
for label in ("a","b"):
    results["real_write_"+label]=run(["--root",repo,"--output",out/("generated-"+label),"--write"])
a=json.loads((out/"generated-a/source-manifest.json").read_text(encoding="utf-8"))
b=json.loads((out/"generated-b/source-manifest.json").read_text(encoding="utf-8"))
old=json.loads((repo/"docs/reports/743-acc-hir-contract/source-manifest.json").read_text(encoding="utf-8"))
for obj in (a,b,old):obj["source_identity"]["head_commit_audit_only"]=None
results["determinism"]={"two_writes_identical":(out/"generated-a/source-manifest.json").read_bytes()==(out/"generated-b/source-manifest.json").read_bytes(),"normalized_manifest_matches_committed":a==old,"summary_matches_committed":(out/"generated-a/scan-summary.md").read_bytes()==(repo/"docs/reports/743-acc-hir-contract/scan-summary.md").read_bytes()}
fixture=repo/"scripts/tests/fixtures/acc-inventory/repo"
for name, data in {
    "unbound-decision":{"format_version":1,"decisions":[{"id":"MD-UNBOUND","subject":"auto/lib/token.at","kind":"module-role","conclusion":"adapt","evidence":["auto/lib/token.at:2"],"bound_input_hashes":{}}]},
    "missing-evidence":{"format_version":1,"decisions":[{"id":"MD-MISSING","subject":"auto/lib/token.at","kind":"module-role","conclusion":"adapt","evidence":["auto/lib/does-not-exist.at:999"],"bound_input_hashes":{}}]},
    "empty-decisions":{"format_version":1,"decisions":[]},
}.items():
    root=out/name/"repo"; report=out/name/"report"
    shutil.copytree(fixture,root)
    results[name+"_write"]=run(["--root",root,"--output",report,"--write"])
    (report/"manual-decisions.json").write_text(json.dumps(data),encoding="utf-8")
    results[name+"_check"]=run(["--root",root,"--output",report,"--check"])
p=subprocess.run([sys.executable,"-B","-m","unittest","discover","-s",str(repo/"scripts/tests"),"-p","test_acc_inventory.py","-v"],capture_output=True,text=True,encoding="utf-8",timeout=45)
(out/"tests.txt").write_text(p.stdout+p.stderr,encoding="utf-8")
results["tests"]={"exit":p.returncode,"summary":p.stderr[-120:]}
plan=repo/"docs/plans/743-acc-bootstrap-hir-contract.md"
if not plan.is_file():plan=repo/"docs/plans/archive/743-acc-bootstrap-hir-contract.md"
results["hashes"]={str(path.relative_to(repo)).replace("\\","/"):hashlib.sha256(path.read_bytes()).hexdigest() for path in [tool,repo/"scripts/tests/test_acc_inventory.py",plan,repo/"docs/specs/auto-acc/project.md",repo/"docs/specs/auto-hir/stage-contract.md",repo/"docs/specs/auto-hir/project.md",repo/"docs/specs/auto-ac/project.md",repo/"docs/design/strategy/auto-acc-bootstrap-contract.md",repo/"docs/reports/743-acc-hir-contract/proposed-spec-delta.md",repo/"docs/reports/743-acc-hir-contract/manual-decisions.json"]}
(out/"results.json").write_text(json.dumps(results,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({k:v for k,v in results.items() if k!="hashes"},ensure_ascii=False,indent=2))
