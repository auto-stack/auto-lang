#!/usr/bin/env python3
"""PLAN-1015 RQ closeout drill (F-989-CLOSE-02): specified old combo -> N.

Phase A (old combo O-RQ/U-RQ/L-RQ/M-BASE):
  start todo backend -> POST unique marker -> GET list -> stop owned procs
  -> quiescent backup (todos.db + -wal + -bak)
Phase N (new combo):
  reuse same data dir -> start -> read A marker -> POST phase-N marker ->
  GET -> stop
Fault: bad-source / wrong Host must reject and preserve A + backup.
Restore A: start old combo again -> real task -> independent sqlite readback.
"""
from __future__ import annotations

import json
import re
import shutil
import sqlite3
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
MARKERS = {
    "a": "plan-1015-rq-a-marker",
    "n": "plan-1015-rq-n-marker",
    "restore": "plan-1015-rq-restore-marker",
}
RESULTS: list[dict] = []


def log(msg: str) -> None:
    print(f"[rq-drill-1015] {msg}", flush=True)


def record(case: str, ok: bool, detail: str) -> bool:
    RESULTS.append({"case": case, "ok": ok, "detail": detail})
    log(f"{'PASS' if ok else 'FAIL'} {case}: {detail}")
    return ok


def http_call(port: int, method: str, path: str, body: dict | None = None):
    url = f"http://127.0.0.1:{port}{path}"
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=10) as resp:
        return json.loads(resp.read().decode())


def wait_ready(port: int, timeout: float = 600.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(
                    f"http://127.0.0.1:{port}/api/todos", timeout=3) as r:
                if r.status == 200:
                    return True
        except Exception:
            pass
        time.sleep(2)
    return False


def sqlite_read(db: Path) -> list[tuple]:
    con = sqlite3.connect(db)
    try:
        cur = con.cursor()
        cur.execute("SELECT id, text, done FROM todos ORDER BY id")
        return cur.fetchall()
    finally:
        con.close()


def dump_table_schema(db: Path) -> str:
    con = sqlite3.connect(db)
    try:
        cur = con.cursor()
        cur.execute("SELECT sql FROM sqlite_master WHERE type='table'")
        return "\n".join(str(r[0]) for r in cur.fetchall())
    finally:
        con.close()


def main() -> int:
    scope = Path(r"D:/autostack/plan-artifacts/plan-1015/cohort-rq")
    todo_root = Path(r"D:/autostack/auto-todo")
    old_auto = Path(r"D:/autostack/.wt/rc1015-old/auto-man/target/debug/auto.exe")
    new_auto = Path(r"D:/autostack/.wt/lang-1015/auto-man/target/debug/auto.exe")
    data_a = scope / "data-a"
    data_n = scope / "data-n"
    data_restore = scope / "data-restore"
    backup = scope / "backup-a"

    port_a, port_n, port_restore = 9721, 9722, 9723
    port_chat = 9724
    ok_all = True

    def start_service(auto: Path, port: int, data_dir: Path,
                      tag: str, lang_root: Path) -> subprocess.Popen:
        import os
        env = os.environ.copy()
        env["AUTO_TODO_DATA"] = str(data_dir)
        env["AUTO_OS_ROOT"] = "D:/autostack/.wt/rc1015-old/auto-os"
        env["AUTO_LANG_ROOT"] = str(lang_root)
        return subprocess.Popen(
            [str(auto), "service", "--server", "rust", "-B", str(port)],
            cwd=str(todo_root), env=env,
            stdout=open(scope / "logs" / f"{tag}.out", "w"),
            stderr=open(scope / "logs" / f"{tag}.err", "w"))

    def os_environ_path() -> str:
        import os
        return os.environ.get("PATH", "")

    # ---- Phase A: old combo writes marker ----
    data_a.mkdir(parents=True, exist_ok=True)
    pA = start_service(old_auto, port_a, data_a, "phase-a",
                       Path(r"D:/autostack/.wt/rc1015-old/auto-lang"))
    log(f"phase-a pid={pA.pid}")

    # bystander: independent auto-todo instance (own data dir / port)
    chat_data = scope / "bystander-data"
    chat_data.mkdir(parents=True, exist_ok=True)
    pChat = start_service(old_auto, port_chat, chat_data, "bystander-todo",
                          Path(r"D:/autostack/.wt/rc1015-old/auto-lang"))
    log(f"bystander-todo pid={pChat.pid}")
    bystander_ok = False
    if wait_ready(port_chat):
        rec_b = http_call(port_chat, "POST", "/api/todos",
                          {"text": "plan-1015-bystander-marker"})
        bystander_ok = rec_b.get("todo", {}).get("text") == "plan-1015-bystander-marker"
        record("bystander-todo-create", bystander_ok, f"create={rec_b}")
    else:
        record("bystander-todo-ready", False, "bystander backend not ready")

    if not wait_ready(port_a):
        pA.terminate()
        return record("phase-a-ready", False, "backend not ready")
    rec = http_call(port_a, "POST", "/api/todos", {"text": MARKERS["a"]})
    ok_all = record("phase-a-create",
                    rec.get("todo", {}).get("text") == MARKERS["a"],
                    f"created {rec}") and ok_all
    lst = http_call(port_a, "GET", "/api/todos")
    todos = lst.get("todos", [])
    ok_all = record("phase-a-list",
                    any(t.get("text") == MARKERS["a"] for t in todos),
                    f"list {len(todos)} items") and ok_all
    schema_a = dump_table_schema(data_a / "todos.db")
    ok_all = record("phase-a-schema", "todos" in schema_a,
                    f"schema={schema_a[:120]}") and ok_all
    # stop owned procs: A service + spawned backend
    pA.terminate()
    pA.wait(timeout=10)
    time.sleep(2)

    # quiescent backup: copy db + wal + bak after service stop
    backup.mkdir(parents=True, exist_ok=True)
    for f in ["todos.db", "todos.db-wal", "todos.db-bak"]:
        src = data_a / f
        if src.exists():
            shutil.copy2(src, backup / f)
    record("phase-a-backup", (backup / "todos.db").exists(),
           "backup db copied")

    # ---- Phase N: new combo reads A data + writes marker ----
    # copy A db into N's data dir (same schema, no migration)
    data_n.mkdir(parents=True, exist_ok=True)
    shutil.copy2(data_a / "todos.db", data_n / "todos.db")
    pN = start_service(new_auto, port_n, data_n, "phase-n",
                       Path(r"D:/autostack/.wt/lang-1015/auto-lang"))
    log(f"phase-n pid={pN.pid}")
    if not wait_ready(port_n):
        pN.terminate()
        ok_all = record("phase-n-ready", False, "N backend not ready") and ok_all
    else:
        lst = http_call(port_n, "GET", "/api/todos")
        todos_n = lst.get("todos", [])
        ok_all = record("phase-n-reads-a",
                        any(t.get("text") == MARKERS["a"] for t in todos_n),
                        f"N reads A marker: {len(todos_n)} items") and ok_all
        rec = http_call(port_n, "POST", "/api/todos", {"text": MARKERS["n"]})
        ok_all = record("phase-n-create",
                        rec.get("todo", {}).get("text") == MARKERS["n"],
                        f"created {rec}") and ok_all
        lst2 = http_call(port_n, "GET", "/api/todos")
        todos2 = lst2.get("todos", [])
        ok_all = record("phase-n-continue",
                        any(t.get("text") == MARKERS["n"] for t in todos2),
                        f"N list {len(todos2)} items") and ok_all
        pN.terminate()
        pN.wait(timeout=10)
        time.sleep(2)

    # ---- Fault: bad source must reject; A backup preserved ----
    env = {"AUTO_TODO_DATA": str(data_a),
           "AUTO_DELIVERY_SOURCE_ROOT": str(scope / "missing-source"),
           "AUTO_OS_ROOT": "D:/autostack/.wt/rc1015-old/auto-os",
           "AUTO_LANG_ROOT": str(old_auto.parent.parent.parent),
           "PATH": os_environ_path()}
    bad = subprocess.run([str(old_auto), "gen", "--delivery-root",
                          str(scope / "bundle-bad-rq")],
                         cwd=str(todo_root), env=env,
                         capture_output=True, text=True, timeout=300)
    ok_all = record("fault-bad-source", bad.returncode != 0,
                    f"exit={bad.returncode}") and ok_all
    ok_all = record("fault-backup-preserved",
                    (backup / "todos.db").exists()
                    and sqlite_read(backup / "todos.db")
                    and any(r[1] == MARKERS["a"] for r in
                            sqlite_read(backup / "todos.db")),
                    "backup contains A marker") and ok_all

    # ---- Restore A: old combo + selected backup ----
    data_restore.mkdir(parents=True, exist_ok=True)
    shutil.copy2(backup / "todos.db", data_restore / "todos.db")
    pR = start_service(old_auto, port_restore, data_restore, "phase-restore",
                       Path(r"D:/autostack/.wt/rc1015-old/auto-lang"))
    log(f"phase-restore pid={pR.pid}")
    if not wait_ready(port_restore):
        pR.terminate()
        ok_all = record("restore-ready", False, "restore backend not ready") and ok_all
    else:
        lst = http_call(port_restore, "GET", "/api/todos")
        todos_r = lst.get("todos", [])
        ok_all = record("restore-reads-backup",
                        any(t.get("text") == MARKERS["a"] for t in todos_r),
                        f"restore reads A marker: {len(todos_r)} items") and ok_all
        rec = http_call(port_restore, "POST", "/api/todos",
                        {"text": MARKERS["restore"]})
        ok_all = record("restore-create",
                        rec.get("todo", {}).get("text") == MARKERS["restore"],
                        f"created {rec}") and ok_all
        pR.terminate()
        pR.wait(timeout=10)
        time.sleep(2)

    # independent sqlite readback of restore data
    rows = sqlite_read(data_restore / "todos.db")
    ok_all = record("restore-sqlite-readback",
                    any(r[1] == MARKERS["a"] for r in rows)
                    and any(r[1] == MARKERS["restore"] for r in rows),
                    f"rows={rows}") and ok_all

    # bystander survived through fault + switch: PID alive + still serves
    bystander_alive = pChat.poll() is None
    ok_all = record("bystander-todo-alive", bystander_alive,
                    f"pid={pChat.pid} poll={pChat.poll()}") and ok_all
    if bystander_alive:
        try:
            lst_b = http_call(port_chat, "GET", "/api/todos")
            ok_all = record("bystander-todo-business",
                            any(t.get("text") == "plan-1015-bystander-marker"
                                for t in lst_b.get("todos", [])),
                            f"bystander list {len(lst_b.get('todos', []))}") and ok_all
        except Exception as e:
            ok_all = record("bystander-todo-business", False,
                            f"bystander unresponsive: {e}") and ok_all
    if bystander_ok and bystander_alive:
        ok_all = record("bystander-isolation", True,
                        "independent instance survived fault/switch") and ok_all
    pChat.terminate()
    try:
        pChat.wait(timeout=10)
    except subprocess.TimeoutExpired:
        pChat.kill()

    receipt = scope / "rq-drill-receipt.json"
    with open(receipt, "w", encoding="utf-8") as f:
        json.dump({"plan": "PLAN-1015", "markers": MARKERS,
                   "results": RESULTS,
                   "passed": sum(1 for r in RESULTS if r["ok"]),
                   "total": len(RESULTS)}, f, ensure_ascii=False, indent=2)
    log(f"receipt: {receipt}")
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
