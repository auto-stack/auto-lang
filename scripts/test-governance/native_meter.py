"""Windows process-tree CPU/peak accounting for prebuilt test workloads.

The workload starts suspended, enters a kill-on-close Job, then resumes.
Job accounting includes the workload parent and descendant processes. The
meter's own CPU is reported separately and added to total CPU; wall time starts
before launch and ends after bounded pipe/tree cleanup. No percent estimates.
"""
from __future__ import annotations
import ctypes
from ctypes import wintypes as W
import os
import subprocess
import threading
import time


class IO(ctypes.Structure):
    _fields_ = [(name, ctypes.c_ulonglong) for name in (
        "read_ops", "write_ops", "other_ops", "read_bytes", "write_bytes", "other_bytes")]


class Limits(ctypes.Structure):
    _fields_ = [("user", ctypes.c_longlong), ("job_user", ctypes.c_longlong),
                ("flags", W.DWORD), ("min_working", ctypes.c_size_t), ("max_working", ctypes.c_size_t),
                ("active_limit", W.DWORD), ("affinity", ctypes.c_size_t),
                ("priority", W.DWORD), ("scheduling", W.DWORD)]


class Extended(ctypes.Structure):
    _fields_ = [("basic", Limits), ("io", IO), ("process_limit", ctypes.c_size_t),
                ("job_limit", ctypes.c_size_t), ("peak_process", ctypes.c_size_t),
                ("peak_job", ctypes.c_size_t)]


class BasicAccounting(ctypes.Structure):
    _fields_ = [("user", ctypes.c_longlong), ("kernel", ctypes.c_longlong),
                ("period_user", ctypes.c_longlong), ("period_kernel", ctypes.c_longlong),
                ("faults", W.DWORD), ("processes", W.DWORD), ("active", W.DWORD),
                ("terminated", W.DWORD)]


class Accounting(ctypes.Structure):
    _fields_ = [("basic", BasicAccounting), ("io", IO)]


class ThreadEntry(ctypes.Structure):
    _fields_ = [("size", W.DWORD), ("usage", W.DWORD), ("tid", W.DWORD),
                ("pid", W.DWORD), ("base", W.LONG), ("delta", W.LONG), ("flags", W.DWORD)]


def api():
    if os.name != "nt":
        raise OSError("native process-tree metering is currently supported on Windows only")
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    signatures = {
        "CreateJobObjectW": ([ctypes.c_void_p, W.LPCWSTR], W.HANDLE),
        "SetInformationJobObject": ([W.HANDLE, ctypes.c_int, ctypes.c_void_p, W.DWORD], W.BOOL),
        "QueryInformationJobObject": ([W.HANDLE, ctypes.c_int, ctypes.c_void_p, W.DWORD, ctypes.c_void_p], W.BOOL),
        "AssignProcessToJobObject": ([W.HANDLE, W.HANDLE], W.BOOL),
        "TerminateJobObject": ([W.HANDLE, W.UINT], W.BOOL),
        "CloseHandle": ([W.HANDLE], W.BOOL),
        "CreateToolhelp32Snapshot": ([W.DWORD, W.DWORD], W.HANDLE),
        "Thread32First": ([W.HANDLE, ctypes.POINTER(ThreadEntry)], W.BOOL),
        "Thread32Next": ([W.HANDLE, ctypes.POINTER(ThreadEntry)], W.BOOL),
        "OpenThread": ([W.DWORD, W.BOOL, W.DWORD], W.HANDLE),
        "ResumeThread": ([W.HANDLE], W.DWORD),
        "GetCurrentProcess": ([], W.HANDLE),
        "OpenProcess": ([W.DWORD, W.BOOL, W.DWORD], W.HANDLE),
        "GetExitCodeProcess": ([W.HANDLE, ctypes.POINTER(W.DWORD)], W.BOOL),
        "GetProcessTimes": ([W.HANDLE, ctypes.POINTER(W.FILETIME), ctypes.POINTER(W.FILETIME),
                              ctypes.POINTER(W.FILETIME), ctypes.POINTER(W.FILETIME)], W.BOOL),
    }
    for name, (arguments, result) in signatures.items():
        function = getattr(kernel, name)
        function.argtypes, function.restype = arguments, result
    return kernel


def required(result, action):
    if not result:
        raise OSError(ctypes.get_last_error(), action)
    return result


def cpu_100ns(kernel):
    values = [W.FILETIME() for _ in range(4)]
    required(kernel.GetProcessTimes(kernel.GetCurrentProcess(), *(ctypes.byref(v) for v in values)),
             "GetProcessTimes(meter)")
    return sum((v.dwHighDateTime << 32) + v.dwLowDateTime for v in values[2:])


def resume(kernel, pid):
    snapshot = kernel.CreateToolhelp32Snapshot(4, 0)
    if snapshot in (None, ctypes.c_void_p(-1).value):
        raise OSError("thread snapshot unavailable")
    count = 0
    try:
        entry = ThreadEntry()
        entry.size = ctypes.sizeof(entry)
        more = kernel.Thread32First(snapshot, ctypes.byref(entry))
        while more:
            if entry.pid == pid:
                handle = required(kernel.OpenThread(2, False, entry.tid), "OpenThread")
                try:
                    if kernel.ResumeThread(handle) == 0xFFFFFFFF:
                        raise OSError("ResumeThread failed")
                    count += 1
                finally:
                    kernel.CloseHandle(handle)
            more = kernel.Thread32Next(snapshot, ctypes.byref(entry))
    finally:
        kernel.CloseHandle(snapshot)
    if not count:
        raise OSError("no suspended workload thread found")


def measure(argv, *, cwd=None, env=None, timeout=600, output_cap=16 << 20, error_cap=1 << 20):
    kernel = api()
    start = time.perf_counter()
    meter_before = cpu_100ns(kernel)
    job = required(kernel.CreateJobObjectW(None, None), "CreateJobObjectW")
    child = None
    threads = []
    output, errors = bytearray(), bytearray()
    overflow = threading.Event()
    reader_errors = []
    failure = None
    accounting, memory = Accounting(), Extended()
    try:
        limits = Extended()
        limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        required(kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)),
                 "SetInformationJobObject")
        child = subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 creationflags=0x4 | 0x08000000)
        required(kernel.AssignProcessToJobObject(job, int(child._handle)), "AssignProcessToJobObject")

        def read(pipe, target, cap):
            try:
                while True:
                    block = pipe.read1(16384)
                    if not block:
                        break
                    if len(target) + len(block) > cap:
                        overflow.set()
                        break
                    target.extend(block)
            except OSError as error:
                reader_errors.append(str(error))
            finally:
                pipe.close()

        for pipe, target, cap in ((child.stdout, output, output_cap), (child.stderr, errors, error_cap)):
            thread = threading.Thread(target=read, args=(pipe, target, cap), daemon=True)
            thread.start()
            threads.append(thread)
        resume(kernel, child.pid)
        while child.poll() is None:
            if time.perf_counter() - start >= timeout:
                failure = "deadline"
                break
            if overflow.is_set():
                failure = "output cap"
                break
            time.sleep(0.002)
        if failure:
            required(kernel.TerminateJobObject(job, 124), "TerminateJobObject")
        child.wait(timeout=5)
        # Query after process exit, BEFORE closing the still-accountable Job.
        settle = time.monotonic() + 2
        while True:
            required(kernel.QueryInformationJobObject(job, 8, ctypes.byref(accounting),
                      ctypes.sizeof(accounting), None), "job CPU/I/O accounting")
            if not accounting.basic.active or time.monotonic() >= settle:
                break
            time.sleep(0.002)
        required(kernel.QueryInformationJobObject(job, 9, ctypes.byref(memory),
                  ctypes.sizeof(memory), None), "job peak memory")
        if accounting.basic.active:
            failure = failure or "descendants live after workload exit"
            required(kernel.TerminateJobObject(job, 124), "reap descendant tree")
    finally:
        # The job owns descendants even if setup, sampling or parsing fails.
        if child is not None and child.poll() is None:
            kernel.TerminateJobObject(job, 124)
            child.kill()
            child.wait(timeout=5)
        kernel.CloseHandle(job)
        grace = time.monotonic() + 2
        for thread in threads:
            thread.join(max(0, grace-time.monotonic()))
        if any(t.is_alive() for t in threads):
            raise OSError("meter reader cleanup deadline exceeded")
    if reader_errors:
        failure = failure or "reader error: " + "; ".join(reader_errors)
    if overflow.is_set():
        failure = failure or "output cap"
    own = cpu_100ns(kernel) - meter_before
    guest = accounting.basic.user + accounting.basic.kernel
    return {"argv": list(argv), "exit": child.returncode, "error": failure,
            "wall_ms": (time.perf_counter()-start)*1000,
            "workload_cpu_ms": guest/10000, "meter_cpu_ms": own/10000,
            "total_cpu_ms": (guest+own)/10000,
            "peak_process_bytes": memory.peak_process, "peak_job_bytes": memory.peak_job,
            "job_processes": accounting.basic.processes,
            "stdout": bytes(output), "stderr": bytes(errors)}
