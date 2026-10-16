"""Independent T01-T05 review probes. Writes only a new owned temp directory.
Run: python <this file> --tree D:/autostack/.wt/lang-788/auto-lang
The mock worker tests the parent's declared trust boundary, not compiler correctness.
"""
import argparse, base64, hashlib, json, os, pathlib, re, subprocess, tempfile, time

p = argparse.ArgumentParser()
p.add_argument('--tree', required=True)
a = p.parse_args()
tree = pathlib.Path(a.tree)
root = pathlib.Path(tempfile.mkdtemp(prefix='p788-review-r1-'))
worker = tree / 'experimental/ac-case-worker/target/debug/ac-case-worker.exe'
runner = tree / 'target/debug/auto-case.exe'
results = []

def observe(name, data):
    results.append({'probe': name, **data})
    print(json.dumps(results[-1], ensure_ascii=False), flush=True)

def real_worker(name, **changes):
    scratch = root / name
    scratch.mkdir()
    req = dict(schema='acworker/1', kind='build', nonce=name, request_hash=name,
               input={'kind':'source','source_b64':base64.b64encode(b'fn main() int { return 3 }').decode()},
               profile='core', entry='main', budget={'build_ms':1,'run_ms':5000,'total_ms':1},
               scratch=str(scratch), store_root=str(scratch/'store'), cache='fresh', project=name, include_hir=False)
    req.update(changes)
    data = '\n'.join(map(json.dumps, [{'op':'begin','schema':'acworker/1'}, {'op':'request','request':req}, {'op':'end'}]))+'\n'
    t = time.monotonic()
    proc = subprocess.run([str(worker)], input=data, text=True, capture_output=True, timeout=30)
    response = json.loads(proc.stdout.strip().splitlines()[-1])
    observe(name, dict(wall_ms=round((time.monotonic()-t)*1000), response=response))

real_worker('budget-one-ms')
real_worker('backend-missing-entry', entry='absent', budget={'build_ms':60000,'run_ms':5000,'total_ms':70000})
real_worker('negative-trace', kind='check', input={'kind':'source','source_b64':base64.b64encode(b'fn main() int { return absent }').decode()})
shared_store=str(root/'shared-store')
real_worker('hir-golden-cold', store_root=shared_store, cache='reuse', include_hir=True, budget={'build_ms':60000,'run_ms':5000,'total_ms':70000})
real_worker('hir-golden-hot', store_root=shared_store, cache='reuse', include_hir=True, budget={'build_ms':60000,'run_ms':5000,'total_ms':70000})

# A deliberately untrusted worker: correct identity echo, wrong schema,
# nonzero process exit, optional outside-scratch artifact, and missing run.
mock_src = root/'mock.rs'
mock_src.write_text(r'''
use std::io::{self, BufRead};
fn field(s: &str, key: &str) -> String {
    let needle = format!("\"{}\":\"", key);
    s.split(&needle).nth(1).unwrap().split('"').next().unwrap().to_string()
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        if !line.contains("\"request\":") { continue; }
        let n = field(&line, "nonce"); let h = field(&line, "request_hash");
        let ok = std::fs::read_to_string(std::env::var("MOCK_OK_FILE").unwrap()).unwrap();
        let schema=std::env::var("MOCK_SCHEMA").unwrap();
        println!("{{\"schema\":\"{}\",\"nonce\":\"{}\",\"request_hash\":\"{}\",\"ok\":{}}}", schema, n, h, ok);
        std::process::exit(std::env::var("MOCK_EXIT").unwrap().parse().unwrap());
    }
}
''', encoding='utf-8')
mock_exe = root/'mock.exe'
subprocess.run(['rustc',str(mock_src),'-o',str(mock_exe)],check=True,capture_output=True)
okfile=root/'ok.json'

def run_case(name, checks, ok=None, source='fn main() int { return 3 }', schema='acworker/1', worker_exit=0):
    fixture=root/name
    fixture.mkdir()
    (fixture/'suite.ad').write_text('```test-suite\nsuite { schema: "autolang.suite/2", id: "review", include: ["case.ad"] }\n```\n',encoding='utf-8')
    (fixture/'case.ad').write_text('```test-case\ncase { schema: "autolang.case/2", id: "review.case", source: { text: '+json.dumps(source)+', syntax: "auto", path: "main.at" }, entry: { mode: "program" }, checks: ['+checks+'] }\n```\n',encoding='utf-8')
    env=dict(os.environ, AC_CASE_WORKER=str(mock_exe if ok is not None else worker), MOCK_OK_FILE=str(okfile), MOCK_SCHEMA=schema, MOCK_EXIT=str(worker_exit))
    if ok is not None: okfile.write_text(json.dumps(ok),encoding='utf-8')
    report=root/(name+'.json')
    proc=subprocess.run([str(runner),'run','--suite',str(fixture/'suite.ad'),'--out',str(root/(name+'-out')),'--json-out',str(report)],env=env,text=True,capture_output=True,timeout=40)
    data={'stderr':proc.stderr,'stdout':proc.stdout}
    if report.exists():
        raw=report.read_text(encoding='utf-8')
        try: data=json.loads(raw)
        except json.JSONDecodeError as e:
            # Preserve the actual invalid report; use a repaired COPY only
            # for subsequent observations, never claim that the raw parses.
            repaired=re.sub(r'("duration_ms": \d+)\n',r'\1,\n',raw)
            repaired=re.sub(r',\s*([}\]])',r'\1',repaired)
            data={'raw_json_error':str(e),'repaired_copy':json.loads(repaired)}
    observe(name,dict(gate=proc.returncode,report=data))
    return fixture,report,env

valid_run={'diagnostics':[], 'artifacts':[], 'run':{'exit_code':3,'stdout':'','stderr':''},'trace':[], 'toolchain':{}}
run_case('wrong-schema', '{id:"run",adapter:"ac",action:"run",expect:{completion:"success",exit:3}}', valid_run, schema='acworker/999')
run_case('missing-run', '{id:"run",adapter:"ac",action:"run",expect:{completion:"success"}}', {})
run_case('nonzero-worker', '{id:"run",adapter:"ac",action:"run",expect:{completion:"success",exit:3}}', valid_run, worker_exit=7)
outside=root/'outside.bin'
outside.write_bytes(b'not a PE executable')
artifact=dict(kind='exe',logical='app.exe',path=str(outside),bytes=999,sha256=hashlib.sha256(outside.read_bytes()).hexdigest())
run_case('outside-artifact-invalid-consumer', '{id:"build",adapter:"ac",action:"build",expect:{completion:"success"}}, {id:"consume",adapter:"ac",action:"run",product_from:"build",expect:{completion:"success",exit:3}}', {'artifacts':[artifact]})
run_case('consumer-stream-rejected', '{id:"build",adapter:"ac",action:"build",expect:{completion:"success"}}, {id:"consume",adapter:"ac",action:"run",product_from:"build",expect:{completion:"success",stdout:{text:""}}}', {'artifacts':[artifact]})
fixture, report, env = run_case('real-native-report','{id:"run",adapter:"ac",action:"run",expect:{completion:"success",exit:3}}')
# Native JSON is currently malformed. Repair only a separate diagnostic
# copy so the stale identity probe can reach the identity comparison.
raw=report.read_text(encoding='utf-8')
try: json.loads(raw)
except json.JSONDecodeError:
    raw=re.sub(r'("duration_ms": \d+)\n',r'\1,\n',raw)
    raw=re.sub(r',\s*([}\]])',r'\1',raw)
    report=root/'repaired-for-stale.json'
    report.write_text(raw,encoding='utf-8')
env['AC_CASE_WORKER']=str(mock_exe)
proc=subprocess.run([str(runner),'stale','--suite',str(fixture/'suite.ad'),'--report',str(report)],env=env,text=True,capture_output=True,timeout=15)
observe('stale-replaced-worker',dict(gate=proc.returncode,stdout=proc.stdout,stderr=proc.stderr))
(root/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8')
print('Evidence directory: '+str(root))
