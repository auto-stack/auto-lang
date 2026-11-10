"""Original accounting conservation and old certification guard, never execution."""
import hashlib
import math
from pathlib import Path

from ace_m1_arch_acceptance import EvidenceScope, counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import admit_accounting, typed_equal

ARCH = ('ARCH-08/approved-old-conservation', 'ARCH-08/old38-cert-separation')
CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/legacy-architecture-contract.json'
CONTRACT_SHA = '08172653fd630452d80f0c2330672ea9d9ac983646f957f5118d196dac5982d9'
SCRIPT = 'scripts/verify-ace-m1-852.ps1'
METER_FIELDS = ('argv', 'exit', 'error', 'wall_ms', 'workload_cpu_ms', 'meter_cpu_ms',
                'total_cpu_ms', 'peak_process_bytes', 'peak_job_bytes', 'job_processes')


def validate_terminal(command, stdout, stderr, repo, root):
    exact_keys(command, ('schema', 'argv', 'cwd', 'exit', 'meter', 'stdout', 'stderr'), 'old accounting command')
    typed_equal(command['schema'], 'ace.m1.legacy-accounting-command/1', 'old accounting command schema')
    argv = command['argv']
    require(type(argv) is list and len(argv) == 10 and all(type(v) is str for v in argv), 'exact original accounting command')
    require(Path(argv[0]).is_absolute() and Path(argv[0]).name.lower() in ('pwsh.exe', 'powershell.exe'), 'explicit PowerShell accounting tool')
    typed_equal(argv[1:], ['-NoProfile', '-File', str(repo / SCRIPT), '-Scope', 'acceptance',
                          '-RepoRoot', str(repo), '-OutDir', str(root)], 'original accounting-only invocation')
    typed_equal(command['cwd'], str(repo), 'original accounting cwd')
    typed_equal(command['stdout'], 'producer.stdout', 'original accounting stdout path')
    typed_equal(command['stderr'], 'producer.stderr', 'original accounting stderr path')
    meter = command['meter']
    exact_keys(meter, METER_FIELDS, 'accounting original Job terminal')
    typed_equal(meter['argv'], argv, 'metered actual accounting invocation')
    typed_equal(command['exit'], 0, 'original accounting exit (integrity only)')
    typed_equal(meter['exit'], 0, 'accounting original process exit')
    typed_equal(meter['error'], None, 'accounting original Job terminal error')
    for name in ('wall_ms', 'workload_cpu_ms', 'meter_cpu_ms', 'total_cpu_ms'):
        require(type(meter[name]) in (int, float) and math.isfinite(meter[name]) and meter[name] >= 0,
                'actual finite accounting ' + name)
    require(meter['wall_ms'] > 0 and meter['wall_ms'] <= 30000, 'original accounting capture deadline')
    require(abs(meter['total_cpu_ms'] - meter['workload_cpu_ms'] - meter['meter_cpu_ms']) < .001,
            'accounting CPU attribution')
    for name in ('peak_process_bytes', 'peak_job_bytes', 'job_processes'):
        require(counter(meter[name], 'actual accounting ' + name) > 0, 'actual accounting meter not empty')
    require(meter['peak_process_bytes'] <= meter['peak_job_bytes'], 'actual accounting process/Job peaks')
    require(not stderr, 'original accounting stderr is not empty')
    text = stdout.decode('utf-8', errors='strict')
    require(text.count('==== ace.m1.acceptance/2 ====') == 1 and text.count('can_claim_m1: True') == 1,
            'raw original accounting terminal and honest claim line')
    # PLAN-992..998 reception (941 round 120): all three certification domains
    # are closed on the old gate's own accounting (value 38 + trap 4 + scene
    # 6), so its claim honestly flips True. The retained guard is the strict
    # receipt-validation clause — a claim without strictly validated
    # trap/resource-scenario receipts must never appear here.
    require('Integrity PASS (accounting valid)' in text and
            'CERT: trap+scene-validated(4 trap + 6 resource-scenario receipt(s)); value axis: supply -BuildIdentity or run snapshot/receipts' in text,
            'raw integrity PASS must retain the certification receipt guard')


def admit(inputs, evidence, binding, parent_accounting):
    exact_keys(binding, ('root',), 'old conservation mechanism binding')
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    contract, _ = inputs.read_json(CONTRACT)
    exact_keys(contract, ('schema', 'mechanisms', 'source_pins', 'execution'), 'old architecture declaration')
    typed_equal(contract['schema'], 'ace.m1.legacy-architecture-contract/1', 'old architecture schema')
    typed_equal(contract['mechanisms'], list(ARCH), 'finite old architecture obligations')
    typed_equal(contract['execution'], 'UNOBSERVED', 'declaration cannot supply execution')
    pins = contract['source_pins']
    require(type(pins) is dict and bool(pins), 'old accounting producer closure')
    for path, digest in pins.items():
        inputs.sha256(path, digest)
    scope = EvidenceScope(evidence.path(binding['root'] + '/ace-m1-acceptance.json').parent, inputs.budget)
    before, before_sha = scope.read_json('source-pins-before.json')
    after, after_sha = scope.read_json('source-pins-after.json')
    exact_keys(before, ('commit', 'source_pins'), 'original accounting before inputs')
    exact_keys(after, ('commit', 'source_pins'), 'original accounting after inputs')
    typed_equal(before, after, 'accounting actual source window stability')
    require(type(before['commit']) is str and len(before['commit']) == 40 and
            all(c in '0123456789abcdef' for c in before['commit']), 'actual accounting producer revision')
    typed_equal(before['source_pins'], pins, 'complete accounting actual source closure')
    command, command_sha = scope.read_json('command.json')
    stdout, stderr = scope.read_bytes('producer.stdout'), scope.read_bytes('producer.stderr')
    scope.reserve(4 * len(stdout) + 4096)  # Reserve Unicode copies before decoding.
    validate_terminal(command, stdout, stderr, inputs.root, scope.root)
    admitted = admit_accounting(scope, 'ace-m1-acceptance.json', inputs)
    # The actual original report is the SAME report already used by the parent;
    # no alternate ledger, newer HIR certificates, or summary can replace it.
    for key in ('accounting_valid', 'required', 'na', 'accepted_diff', 'future', 'required_sha256',
                'cert_gaps', 'can_claim_m1', 'receipt_sha256'):
        typed_equal(admitted[key], parent_accounting[key], 'same original admitted accounting ' + key)
    # PLAN-992..998 reception (941 round 120): the old gate's own 313-door
    # accounting now claims (certs 10 -> 38 across three domains); the
    # expanded-M1 claim remains this package's separate, stricter gate.
    typed_equal(admitted['can_claim_m1'], True, 'old gate claims after full domain closure')
    require(len(admitted['cert_gaps']) == 0,
            'no required A2R cert gaps remain after the 998 resource-scenario reception')
    hash_value(admitted['receipt_sha256'], 'actual original accounting raw identity')
    return dict(schema='ace.m1.legacy-architecture-admission/1',
                observations={key:'PASS' for key in ARCH},
                source_evidence=dict(producer_commit=before['commit'], command_sha256=command_sha,
                    original_before_sha256=before_sha, original_after_sha256=after_sha,
                    raw_accounting_sha256=admitted['receipt_sha256'],
                    raw_stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                    raw_stderr_sha256=hashlib.sha256(stderr).hexdigest(), original_exit=command['exit'],
                    old_certification_gaps=len(admitted['cert_gaps']), original_claim=True),
                legacy_execution='UNOBSERVED', can_claim_architecture_pass=False)
