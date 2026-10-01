import os,sys,subprocess,time,re,json,importlib.util,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('autoui_driver',root/'.agents/skills/autoui-verifier/scripts/test_vm_mcp.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
app=Path(sys.argv[1]).resolve();dest=Path(sys.argv[2]).resolve();dest.parent.mkdir(parents=True,exist_ok=True)
port=m.pick_free_port();env=dict(os.environ,AUTOUI_MCP_PORT=str(port),AUTO_BACKEND_IMPL='vm',AUTO_VM_MERGE='1',AUTO_LANG_ROOT=str(root))
auto_bin=os.environ.get('AUTO_BIN',str(root/'target/debug/auto.exe'))
if not Path(auto_bin).is_file():auto_bin='D:/autostack/auto-lang/target/debug/auto.exe'
log=dest.with_suffix('.log').open('w',encoding='utf-8');proc=subprocess.Popen([auto_bin,'run','-r','vm','--merged'],cwd=app,env=env,stdout=log,stderr=subprocess.STDOUT)
try:
 client=m.AutoUiMcpClient(port); deadline=time.monotonic()+35; snap=''
 while time.monotonic()<deadline:
  if proc.poll() is not None:raise RuntimeError('app exited '+str(proc.returncode))
  try:
   snap=client.snapshot()
   if 'tree:' in snap:break
  except Exception:pass
  time.sleep(.4)
 if 'tree:' not in snap:raise RuntimeError('no UI snapshot within 35s')
 dest.with_suffix('.snapshot.txt').write_text(snap,encoding='utf-8')
 time.sleep(float(os.environ.get('CAPTURE_SETTLE','1')))
 actions=json.loads(os.environ.get('CAPTURE_ACTIONS','[]'))
 action_log=[]
 for action in actions:
  snap=client.snapshot()
  if action['type']=='press':
   matches=re.findall(r'button #(vnode_\d+) "([^"\n]*)"',snap)
   vid=next(v for v,label in matches if action['text'] in label)
   result=client.press(vid)
  elif action['type']=='type':
   vid=re.findall(r'input #(vnode_\d+)',snap)[action.get('index',0)]
   result=client.type_text(vid,action['text'])
  elif action['type']=='keyboard':result=client.keyboard(action['key'],action.get('modifiers'))
  elif action['type']=='event':
   vid=None;current=None;hits=[]
   for line in snap.splitlines():
    hit=re.search(r'^\s*\w+ #(vnode_\d+)',line)
    if hit:current=hit.group(1)
    if 'onclick: '+action['event'] in line:hits.append(current)
   if hits:vid=hits[action.get('index',0)]
   if not vid:raise RuntimeError('event not visible: '+action['event'])
   result=client.press(vid)
  elif action['type']=='cell':
   vid=re.findall(r'col #(vnode_\d+).*?\{\s*onclick: \.Paint\(' + str(action['index']) + r'\)',snap,re.S)[0]
   result=client.press(vid)
  else:raise ValueError(action)
  action_log.append({'action':action,'result':result})
  time.sleep(action.get('wait',.6))
 dest.with_suffix('.actions.json').write_text(json.dumps(action_log,ensure_ascii=False,indent=2),encoding='utf-8')
 dest.with_suffix('.snapshot.txt').write_text(client.snapshot(),encoding='utf-8')
 receipt=client.screenshot(dest.stem,baseline=False,save_path=str(dest))
 print(receipt,flush=True)
 match=re.search(r'Screenshot saved to: (.+\.png)',receipt)
 if match:shutil.copyfile(match.group(1).replace('\\\\?\\',''),dest)
 if not dest.is_file():raise RuntimeError('capture did not produce a file')
 print('CAPTURED',app.name,'PID',proc.pid,'port',port,flush=True)
finally:
 proc.terminate()
 try:proc.wait(timeout=3)
 except subprocess.TimeoutExpired:proc.kill()
 log.close()
