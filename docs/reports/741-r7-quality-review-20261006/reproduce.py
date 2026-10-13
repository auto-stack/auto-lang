from pathlib import Path
import sys,json,tempfile,importlib.util,re
repo=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(exist_ok=False)
s=importlib.util.spec_from_file_location("p741_gate_probe",repo/"docs/reports/741-phase7-lifecycle-fixture-nav/fixture_tests.py");m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
text=(repo/"docs/plans/archive/741-ac-hir-native-core.md").read_text(encoding="utf8");results={}
with tempfile.TemporaryDirectory(prefix="p741-review-") as td:
 root=Path(td)
 for name in ["valid-archive","stale-module-nav","missing-readme-link","wrong-module-target","wrong-readme-target","wrong-ledger-target"]:
  ok,stale=m.build_sandbox_repos(root/name,text);r=stale if name=="stale-module-nav" else ok
  wrong=r/"docs/plans/archive/743-acc-bootstrap-hir-contract.md";wrong.write_text("unrelated archived Plan",encoding="utf8")
  if name=="missing-readme-link":(r/"experimental/ac-core/README.md").write_text("# no plan link",encoding="utf8")
  if name=="wrong-module-target":
   p=r/"docs/specs/auto-ac/plans.md";p.write_text(p.read_text(encoding="utf8").replace("](../../plans/archive/741-ac-hir-native-core.md)","](../../plans/archive/743-acc-bootstrap-hir-contract.md)"),encoding="utf8")
  if name=="wrong-readme-target":(r/"experimental/ac-core/README.md").write_text("[741-ac-hir-native-core.md](../../docs/plans/archive/743-acc-bootstrap-hir-contract.md)",encoding="utf8")
  if name=="wrong-ledger-target":
   p=r/".autoos/specs.json";d=json.loads(p.read_text(encoding="utf8"));d["sections"][0]["items"][0]["file"]="docs/plans/archive/743-acc-bootstrap-hir-contract.md";p.write_text(json.dumps(d),encoding="utf8")
  code,log=m.run_assert(r/"docs/plans/archive/741-ac-hir-native-core.md","archive",r);results[name]={"exit":code,"output":log};print(name,"exit",code)
 for label in ["executing-partial","execution_done-complete","reviewed-complete"]:
  a=m.convert_to_active(text);a=m.set_fm(a,"status",label.split("-")[0])
  if label.startswith("executing"):
   a=re.sub(r"^- \[x\] \*\*T-41\*\*","- [ ] **T-41**",a,flags=re.M);a=m.set_fm(a,"current_step","40")
  ok,_=m.build_sandbox_repos(root/label,a,nav_delivered=False);before=m.REPO;m.REPO=ok
  try:lines,n=m.run_matrix("active ("+label+")",a,root/(label+"-matrix"))
  finally:m.REPO=before
  results[label]={"executed":n,"lines":lines};print(label,"executed",n)
(out/"results.json").write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding="utf8")
