from pathlib import Path
import os,subprocess,time,json
repo=Path('D:/autostack/.wt/lang-741/auto-lang')
out=Path(__file__).parent
env={**os.environ,'CARGO_TARGET_DIR':'D:/autostack/auto-lang/target','CARGO_BUILD_JOBS':'6'}
results=[]
for name,args in [('root-check',['check','-p','auto-lang']),('root-t',['t','--no-fail-fast']),('root-tv',['tv'])]:
    print('RUN',name,flush=True);started=time.monotonic()
    with (out/(name+'.txt')).open('wb') as log:
        proc=subprocess.run(['cargo',*args],cwd=repo,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=2400)
    result={'gate':name,'exit':proc.returncode,'seconds':round(time.monotonic()-started,3),'command':['cargo',*args]}
    results.append(result);print(json.dumps(result),flush=True)
    (out/'root-gates.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf8')
