"""Replay frozen independent cases in an ignored scratch package in the Plan tree."""
import argparse, os, subprocess, sys
from pathlib import Path
parser=argparse.ArgumentParser()
parser.add_argument('--implementation',required=True)
args=parser.parse_args()
root=Path(args.implementation).resolve()
evidence=Path(__file__).resolve().parent
scratch=root/'scratch/review778-r1-replay'
assert scratch.resolve().is_relative_to(root)
scratch.mkdir(parents=True,exist_ok=True)
(scratch/'tests').mkdir(exist_ok=True)
manifest='[package]\nname="review778-r1"\nversion="0.1.0"\nedition="2021"\n[dependencies]\n'
manifest+='auto-lang={path='+repr(str(root/'crates/auto-lang').replace('\\','/'))+'}\n'
manifest+='ui-dialect-consumer={path='+repr(str(root/'test/fixtures/modularity/ui-dialect-consumer').replace('\\','/'))+'}\n[workspace]\n'
(scratch/'Cargo.toml').write_text(manifest,encoding='utf-8')
(scratch/'tests/independent.rs').write_bytes((evidence/'independent.rs').read_bytes())
env=dict(os.environ)
env['CARGO_TARGET_DIR']=str(root/'test/fixtures/modularity/ui-dialect-consumer/target')
with (evidence/'replay-latest.log').open('w',encoding='utf-8') as log:
    q=subprocess.run(['cargo','test','--manifest-path',str(scratch/'Cargo.toml'),'--test','independent','--','--nocapture'],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
print('exit_code='+str(q.returncode))
print('\n'.join((evidence/'replay-latest.log').read_text().splitlines()[-32:]))
sys.exit(q.returncode)
