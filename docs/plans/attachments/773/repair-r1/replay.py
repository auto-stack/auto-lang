"""PLAN-773 repair replay — adapted twin of review-r1/replay.py.

Counterexample inputs (contracts, sources, oracles, selectors, write-port
targets) are IDENTICAL to the reviewer's originals. The single adaptation:
normal-run outputs go to a SIBLING of the fixture root, because the F-05
fix makes the reviewer's original layout (out inside the suite dir) a
by-design contract error — that rejection is itself one of the controls.

Expected verdicts after the repair (was: PASS/exit-0 false greens):
  R01 success-satisfies-error  -> FAIL (exit 1)
  R02 oracle identity          -> differs before/after sidecar mutation
  R03 ignored observations     -> admission ERROR (exit 2)
  R04 snapshot-fail-blocks     -> producer FAIL, consumer NOT blocked
  R05 selector                 -> unselected leaves no other/result.txt;
                                  --adapter typo exits 2 before execution
  R06 out-overlap/json-out     -> exit 2, sentinel/case preserved
  R07 missing-oracle-lint      -> lint exit 2 with located error
  R08 error-stream-ignored     -> admission ERROR (exit 2)
  R09 expanded-checks          -> lint exit 2 (resource limit)
"""
"""Independent PLAN-773 controls; run with a freshly built worktree auto-case.

Writes only disposable fixtures under the given scratch directory. Never
points overwrite probes at repository source/oracles. Saves raw inputs/results.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser()
p.add_argument('--binary', required=True)
p.add_argument('--scratch', required=True)
p.add_argument('--evidence', required=True)
a = p.parse_args()
binary = str(Path(a.binary).resolve())
scratch = Path(a.scratch).resolve()
evidence = Path(a.evidence).resolve()
evidence.mkdir(parents=True, exist_ok=True)
results = []

def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding='utf-8', newline='\n')

def fixture(name, checks, source='1 + 2', extra_suite=''):
    root = scratch / name
    root.mkdir(parents=True, exist_ok=True)
    write(root / 's.suite.ad', '```test-suite\nsuite { schema: "autolang.suite/1", id: "review", include: ["c.case.ad"] ' + extra_suite + ' }\n```\n')
    write(root / 'c.case.ad', '```test-case\ncase { schema: "autolang.case/1", id: "review.case", source: { text: ' + json.dumps(source) + ', syntax: "auto", path: "main.at" }, entry: { mode: "script" }, checks: [' + checks + '] }\n```\n')
    return root

def out_dir_for(root):
    # Plan 773 F-05 fix: outputs live in a SIBLING of the fixture root.
    return root.parent / (root.name + '-out')


def invoke(name, root, command='run', extra=(), out=None):
    for file in root.iterdir():
        if file.is_file():
            target = evidence / 'invocation-inputs' / name / file.name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(file, target)
    args = [binary, command, '--suite', str(root / 's.suite.ad')]
    if command in ('run', 'proposal'):
        args += ['--out', str(out or out_dir_for(root))]
    args += list(extra)
    proc = subprocess.run(args, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=45)
    write(evidence / (name + '.stdout.txt'), proc.stdout)
    write(evidence / (name + '.stderr.txt'), proc.stderr)
    report = None
    report_path = (out or out_dir_for(root)) / 'report.json'
    if report_path.exists():
        try:
            report = json.loads(report_path.read_text(encoding='utf-8'))
        except json.JSONDecodeError:
            report = None  # e.g. the preserved ORACLE SENTINEL (R06)
        write(evidence / (name + '.report.json'), json.dumps(report, ensure_ascii=False, indent=2))
    row = {'id': name, 'command': args, 'exit': proc.returncode,
           'statuses': [(r['check'], r['status']) for r in (report or {}).get('results', [])]}
    results.append(row)
    print(json.dumps(row, ensure_ascii=False), flush=True)
    return report, proc

avm = '{ id: "vm", adapter: "avm", action: "run", expect: { completion: "success", result: { format: "legacy-display", text: "3" } } }'
emit = '{ id: "emit", adapter: "a2r", action: "emit", expect: { completion: "success", artifacts: [{ path: "main.rs", file: "expected.rs" }] } }'
root = fixture('control-avm', avm)
invoke('C01-avm-valid', root)
root = fixture('emit-success-error', '{ id: "emit", adapter: "a2r", action: "emit", expect: { completion: { stage: "run", message: { exact: "emit succeeded" } } } }', 'fn main() { print(1) }')
invoke('R01-success-satisfies-error', root)

root = fixture('oracle-identity', emit, 'fn main() { print(1) }')
write(root / 'expected.rs', '// deliberately wrong\n')
before, _ = invoke('R02-oracle-before', root)
actual = next(out_dir_for(root).rglob('actual/main.rs'))
shutil.copyfile(actual, root / 'expected.rs')
after, _ = invoke('R02-oracle-after', root)
results.append({'id': 'R02-identity-comparison', 'identity_equal': before['results'][0]['identity'] == after['results'][0]['identity'], 'expected_sha_before': hashlib.sha256(b'// deliberately wrong\n').hexdigest(), 'expected_sha_after': hashlib.sha256((root / 'expected.rs').read_bytes()).hexdigest()})

good = (root / 'expected.rs').read_bytes()
root = fixture('ignored-a2r-observations', emit.replace('completion: "success",', 'completion: "success", stdout: { text: "IMPOSSIBLE" }, stderr: { text: "IMPOSSIBLE" }, result: { format: "legacy-display", text: "IMPOSSIBLE" },'), 'fn main() { print(1) }')
(root / 'expected.rs').write_bytes(good)
invoke('R03-a2r-ignored-observations', root)
root = fixture('ignored-avm-artifacts', avm.replace('completion: "success",', 'completion: "success", artifacts: [{ path: "missing.bin", file: "missing-oracle.bin" }],'))
invoke('R03-avm-ignored-artifacts', root)

consumer = '{ id: "consumer", adapter: "avm", action: "run", product_from: "emit", expect: { completion: "success" } }'
root = fixture('product-failed-golden', emit + ',' + consumer, 'fn main() { print(1) }')
write(root / 'expected.rs', '// wrong\n')
invoke('R04-snapshot-fail-blocks-product', root)
(root / 'expected.rs').write_bytes(good)
invoke('R04-fake-consumer-pass', root)

root = fixture('selector', avm + ',' + avm.replace('"vm"', '"other"'))
invoke('R05-selector', root, extra=['--check', 'vm'])
results.append({'id': 'R05-unselected-actually-executed', 'other_result_files': [str(x) for x in out_dir_for(root).rglob('other/result.txt')]})
invoke('R05-invalid-selector', root, extra=['--adapter', 'typo'])

root = fixture('out-overlap', avm)
write(root / 'report.json', 'ORACLE SENTINEL\n')
invoke('R06-out-overlap', root, out=root)
results.append({'id': 'R06-overwrite-observed', 'sentinel_preserved': (root / 'report.json').read_text(encoding='utf-8') == 'ORACLE SENTINEL\n'})
root = fixture('json-overwrite', avm)
old_case = (root / 'c.case.ad').read_bytes()
invoke('R06-json-out-overwrites-input', root, extra=['--json-out', str(root / 'c.case.ad')])
results.append({'id': 'R06-input-changed', 'case_preserved': (root / 'c.case.ad').read_bytes() == old_case})

root = fixture('missing-oracle-lint', emit, 'fn main() { print(1) }')
invoke('R07-missing-oracle-lint', root, command='lint')
invoke('C02-missing-oracle-run-error', root)
root = fixture('strong-error-stream', '{ id: "vm", adapter: "avm", action: "run", expect: { completion: { stage: "parse", message: { contains: "" } }, stdout: { text: "IMPOSSIBLE" } } }', 'fn main(')
invoke('R08-error-stream-ignored', root)

# Overall expansion boundary: individual file/case/check counts stay legal.
root = scratch / 'expanded-limit'
root.mkdir(parents=True, exist_ok=True)
names = [f'c{i}.case.ad' for i in range(5)]
write(root / 's.suite.ad', '```test-suite\nsuite { schema: "autolang.suite/1", id: "expanded", include: ' + json.dumps(names) + ' }\n```\n')
for fi, name in enumerate(names):
    cases = []
    for ci in range(64):
        checks = ','.join(avm.replace('"vm"', json.dumps('v' + str(j))) for j in range(16))
        cases.append('```test-case\ncase { schema: "autolang.case/1", id: "c' + str(fi) + '-' + str(ci) + '", source: { text: "1+2", syntax: "auto", path: "main.at" }, entry: { mode: "expression" }, checks: [' + checks + '] }\n```\n')
    write(root / name, '\n'.join(cases))
invoke('R09-5120-expanded-checks-lint', root, command='lint')

write(evidence / 'results.json', json.dumps(results, ensure_ascii=False, indent=2))
for root in scratch.iterdir():
    if root.is_dir():
        for file in root.iterdir():
            if file.is_file():
                target = evidence / 'inputs' / root.name / file.name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(file, target)
print('Replay complete; raw inputs/output saved.', flush=True)
