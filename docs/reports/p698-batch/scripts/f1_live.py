"""PLAN-698 F-1 live check: SSE client disconnect reclaims the forwarder thread.

Starts 017 VM server, opens two SSE subscriptions that each disconnect,
samples the server process thread count before/during/after. With the fix
the count returns to baseline; without it each subscription leaks a thread.
"""
import json
import os
import re
import subprocess
import sys
import time
import urllib.request

AUTO = r"D:/autostack/.wt/lang-698/auto-lang/target/debug/auto.exe"
APP = r"D:/autostack/.wt/lang-698/auto-lang/examples/ui/017-chat"
LOG = open(r"D:/autostack/.wt/lang-698/scratch-w1/f1_live.log", "w", encoding="utf-8")


def threads_of(pid):
    out = subprocess.run(
        ["powershell", "-Command",
         f"(Get-Process -Id {pid}).Threads.Count"],
        capture_output=True, text=True).stdout.strip()
    try:
        return int(out.splitlines()[-1])
    except (ValueError, IndexError):
        return -1


def main():
    env = dict(os.environ)
    proc = subprocess.Popen([AUTO, "run", "-r", "vm", "--server", "vm", "-B", "18527"],
                            cwd=APP, env=env, stdout=LOG, stderr=subprocess.STDOUT)
    try:
        # wait for the API to answer
        deadline = time.time() + 60
        up = False
        while time.time() < deadline:
            try:
                urllib.request.urlopen("http://127.0.0.1:18527/api/contacts", timeout=2)
                up = True
                break
            except Exception:
                time.sleep(2)
        if not up:
            print("RESULT: server never came up")
            return 2
        time.sleep(3)
        base = threads_of(proc.pid)
        print(f"[baseline] threads={base}")

        # two SSE subscriptions, each disconnects after ~2s
        for i in (1, 2):
            c = subprocess.Popen(
                ["curl", "-sN", "--max-time", "2",
                 "http://127.0.0.1:18527/api/stream"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            c.wait()
        time.sleep(3)
        during = threads_of(proc.pid)
        print(f"[after 2 disconnects +3s] threads={during}")

        # two more with longer window, then settle
        for i in (3, 4):
            c = subprocess.Popen(
                ["curl", "-sN", "--max-time", "2",
                 "http://127.0.0.1:18527/api/stream"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            c.wait()
        time.sleep(5)
        after = threads_of(proc.pid)
        print(f"[after 4 disconnects +5s] threads={after}")

        verdict = "RECLAIMED" if after <= base + 2 else "LEAK"
        print(f"RESULT: {verdict} (base={base} final={after})")
        return 0
    finally:
        proc.terminate()
        LOG.close()


if __name__ == "__main__":
    sys.exit(main())
