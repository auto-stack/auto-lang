"""Bound discovery/preparation producers as well as the eventual test workload."""
import json
import os
import subprocess


def capture(argv, *, cwd=None, env=None, timeout=1800, output_cap=64 << 20, error_cap=16 << 20):
    argv = [os.fspath(value) for value in argv]
    if os.name != 'nt':
        result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, timeout=timeout)
        result.measurement = None
        return result
    from native_meter import measure
    measured = measure(argv, cwd=cwd, env=env, timeout=timeout,
                       output_cap=output_cap, error_cap=error_cap)
    stdout, stderr = measured.pop('stdout'), measured.pop('stderr')
    # An exited parent is not successful preparation while descendants remain.
    code = 124 if measured['error'] else measured['exit']
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
