"""Capture independent production runs and read-only fingerprints."""
import hashlib, json, subprocess, sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
ROOT = Path('D:/autostack/.wt/lang-777/auto-lang')
OUT = Path(__file__).resolve().parent
def run(args):
    q = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', timeout=240)
    return q
def git(*args):
    return run(['git', *args]).stdout.strip()
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def inputs():
    files = git('ls-files').splitlines()
    return {p: digest(ROOT / p) for p in files if (ROOT / p).is_file()}
before = inputs()
head = git('rev-parse', 'HEAD')
status = git('status', '--porcelain')
script = str(ROOT / 'scripts/modularity-audit.py')
jobs = [('help.txt', ['--help']), ('self-test.txt', ['--self-test']),
        ('observe-a.jsonl', ['--mode', 'observe']), ('observe-b.jsonl', ['--mode', 'observe']),
        ('strict.jsonl', ['--mode', 'strict'])]
def invoke(job):
    name, args = job
    q = run([sys.executable, script, *args])
    (OUT / name).write_text(q.stdout, encoding='utf-8')
    (OUT / (name + '.stderr.txt')).write_text(q.stderr, encoding='utf-8')
    return {'file': name, 'command': [sys.executable, script, *args], 'exit_code': q.returncode,
            'sha256': digest(OUT / name)}
with ThreadPoolExecutor(max_workers=3) as pool:
    results = list(pool.map(invoke, jobs))
after = inputs()
record = {'head_before': head, 'head_after': git('rev-parse', 'HEAD'), 'branch': git('branch', '--show-current'),
          'status_before': status, 'status_after': git('status', '--porcelain'), 'inputs_unchanged': before == after,
          'tracked_count': len(before), 'input_sha256': before,
          'spec_sha256': digest(Path('D:/autostack/auto-lang/docs/specs/design/repository-layout.md')),
          'auto_down_head': run(['git', '-C', str(ROOT.parent / 'auto-down'), 'rev-parse', 'HEAD']).stdout.strip(),
          'python': sys.version, 'cargo': run(['cargo', '--version']).stdout.strip()}
(OUT / 'fingerprints.jsonl').write_text(json.dumps(record, ensure_ascii=False) + '\n', encoding='utf-8')
(OUT / 'primary-runs.jsonl').write_text('\n'.join(json.dumps(x) for x in results) + '\n', encoding='utf-8')
print(json.dumps({'runs': results, 'head': head, 'inputs_unchanged': before == after, 'status_after': record['status_after']}, ensure_ascii=False))
