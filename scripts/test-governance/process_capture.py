"""Bound discovery/preparation producers as well as the eventual test workload."""
import json
import os
import subprocess
import hashlib
from pathlib import Path

COMPILER_AUXILIARIES = Path(__file__).with_name('compiler-auxiliaries.json')


def compiler_auxiliaries_reaped(measured):
    """Only source-pinned compiler auxiliaries, after actual Job reap, qualify."""
    live = measured.get('live_after_exit', {})
    processes = live.get('processes', [])
    if (measured['exit'] != 0 or measured['error'] != 'descendants live after workload exit'
            or not live.get('cleanup_complete') or not processes
            or len(processes) != live.get('active')):
        return False
    policy = json.loads(COMPILER_AUXILIARIES.read_text(encoding='utf-8'))
    if policy.get('schema') != 'test-compiler-auxiliaries/1':
        raise ValueError('invalid compiler auxiliary policy')
    pins = {os.path.normcase(str(Path(row['path']).resolve())): row['sha256'] for row in policy['images']}
    for row in processes:
        if not row.get('image'):
            return False
        path = Path(row['image']).resolve()
        expected = pins.get(os.path.normcase(str(path)))
        if not expected or not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            return False
    return True


def capture(argv, *, cwd=None, env=None, timeout=1800, output_cap=64 << 20, error_cap=16 << 20,
            compiler_auxiliary_cleanup=False):
    argv = [os.fspath(value) for value in argv]
    if os.name != 'nt':
        from native_meter import workload_environment
        result = subprocess.run(argv, cwd=cwd, env=workload_environment(env), capture_output=True, timeout=timeout)
        result.measurement = None
        return result
    from native_meter import measure
    measured = measure(argv, cwd=cwd, env=env, timeout=timeout,
                       output_cap=output_cap, error_cap=error_cap)
    stdout, stderr = measured.pop('stdout'), measured.pop('stderr')
    # An exited parent is not successful preparation while descendants remain.
    code = 124 if measured['error'] else measured['exit']
    if compiler_auxiliary_cleanup and compiler_auxiliaries_reaped(measured):
        # Keep the original meter error and actual image/PID/cleanup evidence.
        # This explicit preparation-only exception never changes workload gates.
        measured['accepted_compiler_auxiliary_cleanup'] = True
        code = 0
    if measured['error']:
        stderr += ('\n[process-tree infrastructure] ' + json.dumps(measured, ensure_ascii=False) + '\n').encode()
    result = subprocess.CompletedProcess(argv, code, stdout, stderr)
    result.measurement = measured
    return result


def checked_output(argv, *, cwd=None, env=None, text=False, timeout=1800):
    result = capture(argv, cwd=cwd, env=env, timeout=timeout)
    if result.returncode:
        raise subprocess.CalledProcessError(result.returncode, argv, result.stdout, result.stderr)
    return result.stdout.decode('utf-8') if text else result.stdout
