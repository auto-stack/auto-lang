import hashlib, json, os, pathlib, socket, subprocess, time, urllib.request, uuid

out = pathlib.Path(__file__).resolve().parent
root = pathlib.Path('D:/autostack/tmp/p742-review-r4') / ('s10-' + uuid.uuid4().hex[:10])
root.mkdir(parents=True)
binary = pathlib.Path('D:/autostack/auto-notes/rust-workspace/target/debug/auto-notes-back.exe')
cli = pathlib.Path('D:/autostack/.wt/lang-742/auto-lang/target/debug/auto.exe')
sha = lambda p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
with socket.socket() as s:
    s.bind(('127.0.0.1', 0))
    port = s.getsockname()[1]
env = dict(os.environ, AUTO_HTTP_PORT=str(port), NOTES_TEST_MODE='1', NOTES_SEED='0', NOTES_DATA_DIR=str(root))
env.pop('AUTO_HTTP_SERVICE_JSON', None)
events = []
def record(stage, **data):
    events.append(dict(stage=stage, **data))
    (out/'s10-independent.json').write_text(json.dumps(events, ensure_ascii=False, indent=2), encoding='utf-8')
def request(path, body=None):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(f'http://127.0.0.1:{port}'+path, data=data, headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req, timeout=5) as r:
        return json.loads(r.read())
def launch(label):
    log = open(out/f's10-{label}.log', 'wb')
    proc = subprocess.Popen([str(binary)], cwd='D:/autostack/auto-notes', env=env, stdout=log, stderr=subprocess.STDOUT)
    record('launch', label=label, pid=proc.pid, binary=str(binary), binary_sha256=sha(binary), generator_sha256=sha(cli), data_dir=str(root), port=port)
    end = time.monotonic()+30
    while time.monotonic() < end:
        try:
            request('/api/v1/notes')
            return proc, log
        except Exception:
            if proc.poll() is not None:
                log.close()
                raise RuntimeError(f'backend exited {proc.returncode}')
            time.sleep(.2)
    proc.kill(); proc.wait(timeout=5); log.close()
    raise RuntimeError('backend readiness timeout')
proc = None
log = None
try:
    proc, log = launch('before')
    setup = dict(data_dir=str(root), legacy_file='')
    record('setup-before', request=setup, response=request('/api/v1/test/setup', setup))
    body = dict(request_id='r4-'+uuid.uuid4().hex, title='p742-r4-survivor', body='durable review payload', folder='inbox', tags=[])
    created = request('/api/v1/notes', body)
    assert created['ok'], created
    nid = created['doc']['note_id']
    record('create', request=body, response=created)
    before = request('/api/v1/notes')
    assert any(n['note_id']==nid and n['body']==body['body'] for n in before), before
    db = root/'inbox.db'
    record('pre-kill', pid=proc.pid, notes=before, storage=str(db), storage_sha256=sha(db))
    old_pid = proc.pid
    proc.kill(); proc.wait(timeout=5); log.close(); proc=None; log=None
    record('forced-kill', pid=old_pid, operation='Popen.kill (Windows TerminateProcess)', exited=True)
    try:
        request('/api/v1/notes')
    except Exception as e:
        record('down-confirmed', port=port, error=repr(e))
    else:
        raise AssertionError('endpoint remained available after killing owned process')
    proc, log = launch('after')
    assert proc.pid != old_pid
    record('setup-after', request=setup, response=request('/api/v1/test/setup', setup))
    after = request('/api/v1/notes')
    assert any(n['note_id']==nid and n['title']==body['title'] and n['body']==body['body'] and n['state']=='active' for n in after), after
    record('post-restart', pid=proc.pid, notes=after, storage=str(db), storage_sha256=sha(db), outcome='pass')
    print('S10 independent forced-kill / down / same-binary restart / same-dir durable read PASS')
finally:
    if proc is not None:
        proc.kill(); proc.wait(timeout=5)
    if log is not None:
        log.close()
