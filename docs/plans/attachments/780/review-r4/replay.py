"""Build the current oracle library and run frozen review probes inside the Plan tree."""
import argparse,json,subprocess,sys
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--implementation',required=True);args=p.parse_args()
root=Path(args.implementation).resolve();out=Path(__file__).resolve().parent
scratch=root/'scratch/review780-r1';assert scratch.resolve().is_relative_to(root)
scratch.mkdir(parents=True,exist_ok=True)
source=out/'independent.rs'
(scratch/'independent.rs').write_bytes(source.read_bytes())
with (out/'oracle-build.jsonl').open('w',encoding='utf-8') as stdout,(out/'oracle-build.log').open('w',encoding='utf-8') as stderr:
    q=subprocess.run(['cargo','build','-p','auto-man','--lib','--locked','--message-format=json'],cwd=root,stdout=stdout,stderr=stderr)
if q.returncode: print('oracle build failed');sys.exit(q.returncode)
libs={};native_paths=set()
for line in (out/'oracle-build.jsonl').read_text().splitlines():
    try:r=json.loads(line)
    except json.JSONDecodeError:continue
    if r.get('reason')=='build-script-executed':native_paths.update(r.get('linked_paths',[]))
    if r.get('reason')=='compiler-artifact' and r['target']['name'] in ('auto_man','auto_project_model'):
        for name in r['filenames']:
            if name.endswith('.rlib'):libs[r['target']['name']]=name
assert set(libs)=={'auto_man','auto_project_model'},libs
exe=scratch/'independent.exe'
cmd=['rustc','--edition','2021','--test',str(scratch/'independent.rs'),'-L','dependency='+str(root/'target/debug/deps'),'-o',str(exe)]
for crate,path in libs.items():cmd+=['--extern',crate+'='+path]
for path in sorted(native_paths):cmd+=['-L',path]
with (out/'probe-build.log').open('w',encoding='utf-8') as log:q=subprocess.run(cmd,cwd=root,stdout=log,stderr=subprocess.STDOUT)
if q.returncode: print('probe build failed');sys.exit(q.returncode)
with (out/'independent.log').open('w',encoding='utf-8') as log:q=subprocess.run([str(exe),'--nocapture'],cwd=root,stdout=log,stderr=subprocess.STDOUT)
(out/'probe-run.json').write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'libraries':libs,'compile_command':cmd,'exit_code':q.returncode})+'\n',encoding='utf-8')
print('\n'.join((out/'independent.log').read_text().splitlines()[-38:]))
sys.exit(q.returncode)
