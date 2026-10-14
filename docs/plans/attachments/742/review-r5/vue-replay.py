import hashlib, json, os, pathlib, shutil, socket, subprocess, time, urllib.request, uuid

out = pathlib.Path(__file__).resolve().parent
app = pathlib.Path('D:/autostack/auto-notes')
binary = app/'rust-workspace/target/debug/auto-notes-back.exe'
node = shutil.which('node')
assert node
root = pathlib.Path('D:/autostack/tmp/p742-review-r5')/('vue-'+uuid.uuid4().hex[:10])
root.mkdir(parents=True)
for port in [17818,17819]:
    with socket.socket() as s:
        s.bind(('127.0.0.1',port))
env = dict(os.environ, AUTO_HTTP_PORT='17819', NOTES_TEST_MODE='1', NOTES_SEED='1', NOTES_DATA_DIR=str(root), NOTES_URL='http://127.0.0.1:17818')
env.pop('AUTO_HTTP_SERVICE_JSON',None)
procs=[]
logs=[]
def start(argv, cwd, label):
    log=open(out/f'vue-{label}.log','wb');logs.append(log)
    p=subprocess.Popen(argv,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
    procs.append(p)
    return p
def wait(url, proc):
    end=time.monotonic()+40
    while time.monotonic()<end:
        if proc.poll() is not None: raise RuntimeError(f'process {proc.pid} exited {proc.returncode}')
        try:
            with urllib.request.urlopen(url, timeout=3) as r: return r.read().decode('utf-8')
        except Exception: time.sleep(.25)
    raise RuntimeError(f'readiness timeout {url}')
try:
    back=start([str(binary)],str(app),'backend')
    initial=wait('http://127.0.0.1:17819/api/v1/notes',back)
    front=start([node,str(app/'gen/front/vue/node_modules/vite/bin/vite.js'),'--host','127.0.0.1','--port','17818','--strictPort'],str(app/'gen/front/vue'),'frontend')
    wait('http://127.0.0.1:17818/',front)
    identity=dict(backend_pid=back.pid,frontend_pid=front.pid,backend_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),data_dir=str(root),initial_notes=json.loads(initial),notes_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=app,text=True).strip())
    (out/'vue-identity.json').write_text(json.dumps(identity,ensure_ascii=False,indent=2),encoding='utf-8')
    test=start([node,str(app/'tests/node_modules/@playwright/test/cli.js'),'test','smoke.spec.ts','--reporter=line','--output',str(root/'playwright-output')],str(app/'tests'),'smoke')
    code=test.wait(timeout=150)
    print(f'Vue smoke exit={code}')
    if code: raise SystemExit(code)
finally:
    for p in reversed(procs):
        if p.poll() is None: p.terminate()
        try:p.wait(timeout=4)
        except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
    for log in logs:log.close()
