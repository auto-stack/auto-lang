#!/usr/bin/env python3
"""PLAN-781 T-17/T-18: Reader 10MiB 导入预算验收驱动（V-17 成功链 + V-19 拒写）。

合同：docs/plans/781-reader-runtime-prerequisites.md §6 V-17/V-19 +
决策报告 reports/781-budget-decision.md（r2：导入 profile = 双 env）。

真实链路：只读 Reader 源码（--reader-dir），隔离 split-VM HTTP 实例
（--no-merge -B <port> + AUTO_READER_DATA=reader001-review-*），真实
/api/library/import（本地 path 导入）→ 受管副本逐字节（sha256）→
TOC/章节 API 逐字（manifest 边车哈希）→ 冷重启同库一致性。

用例：
  v17_txt / v17_md      10MiB 导入成功 + 副本 hash + 逐章核对（抽首/中/尾+全量计数）
  v17_cold_restart      冷重启（进程 reap → 同库重启）TOC/章节不变
  v19_low_budget        非空库 + 低预算 → 指名资源拒绝，库 bytes 不变、无半写
  v19_bad_encoding      非法 UTF-8 → decode_error，库不变
  v14_dual_instance     （T-18 段）默认实例拒 + 1G 实例成，双实例不串线

用法：
  python scripts/tools/test_reader_import_budget.py [--auto-bin <exe>] \
      [--reader-dir <path>] [--out-dir <dir>] [--only v17_txt,v19_low_budget]

fixture：test/fixtures/reader-import-budget/（gen_fixtures.py 确定性生成，
缺席自动生成）。退出码 0=全过，2=有 FAIL，3=环境不可用。
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
FIXTURE_DIR = os.path.join(REPO_ROOT, "test", "fixtures", "reader-import-budget")
HEALTH = "/__auto/health/live"
IMPORT_PROFILE = {"AUTO_HTTP_STEP_BUDGET": "1000000000",
                  "AUTO_HTTP_REQUEST_TIMEOUT_MS": "300000"}

results = []
raw = {}


def record(case, ok, detail=""):
    results.append({"case": case, "ok": bool(ok), "detail": str(detail)})
    print(("[+] " if ok else "[-] ") + f"{case}: {detail}")


def pick_free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


class Instance:
    """隔离 split-VM Reader 实例（阶段 A 同款启动/就绪/reap 纪律）。"""

    def __init__(self, tag, auto_bin, reader_dir, base_env=None):
        self.tag = tag
        self.auto_bin = auto_bin
        self.reader_dir = reader_dir
        self.port = pick_free_port()
        self.data_dir = os.path.join(os.environ.get("P781_T17_ROOT", "D:/p781-t17"),
                                     f"reader001-review-{tag}-{int(time.time())}")
        os.makedirs(self.data_dir, exist_ok=True)
        self.err_path = self.data_dir + "-stderr.log"
        self.proc = None
        self._errf = None
        self.base_env = base_env or {}

    def launch(self, timeout=150):
        env = dict(os.environ, **self.base_env)
        env.setdefault("AUTO_READER_DATA", self.data_dir)
        self._errf = open(self.err_path, "w", encoding="utf-8", errors="replace")
        self.proc = subprocess.Popen(
            [self.auto_bin, "run", "-r", "vm", "--no-merge", "-B", str(self.port)],
            cwd=self.reader_dir, env=env,
            stdout=self._errf, stderr=subprocess.STDOUT)
        t0 = time.monotonic()
        while time.monotonic() - t0 < timeout:
            try:
                with urllib.request.urlopen(
                        f"http://127.0.0.1:{self.port}{HEALTH}", timeout=2) as r:
                    if r.status == 200:
                        break
            except Exception:
                pass
            if self.proc.poll() is not None:
                return False
            time.sleep(0.5)
        else:
            return False
        # 前端就绪门：front Init 的 /api/books 返回 200（决策报告 r3：长忙
        # handler 占住派发泵会饿死共居前端的内部调用且 Reader 前端视 Init
        # 失败为致命——导入前必须确认前端已安定，属驱动卫生非掩盖）。
        t1 = time.monotonic()
        while time.monotonic() - t1 < 30:
            try:
                with urllib.request.urlopen(
                        f"http://127.0.0.1:{self.port}/api/books", timeout=3) as r:
                    if r.status == 200:
                        return True
            except Exception:
                pass
            if self.proc.poll() is not None:
                    return False
            time.sleep(0.5)
        return False

    def http(self, method, path, payload=None, timeout=600):
        req = urllib.request.Request(
            f"http://127.0.0.1:{self.port}{path}",
            data=json.dumps(payload).encode() if payload is not None else None,
            headers={"Content-Type": "application/json"}, method=method)
        t0 = time.monotonic()
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                return r.status, r.read().decode("utf-8", "replace"), time.monotonic() - t0
        except urllib.error.HTTPError as e:
            return e.code, e.read().decode("utf-8", "replace"), time.monotonic() - t0
        except Exception as e:  # noqa: BLE001
            return None, f"transport: {e}", time.monotonic() - t0

    def import_book(self, path, force=False, timeout=600):
        return self.http("POST", "/api/library/import",
                         {"path": path, "author": "p781-verify", "force": force}, timeout)

    @staticmethod
    def parse_reply(body):
        """Reader str 响应双重编码（阶段 A T-12 钉死）：外层 JSON 字符串包
        内层业务 JSON。两刀解析；失败返回 None。"""
        try:
            inner = json.loads(body)
            if isinstance(inner, str):
                return json.loads(inner)
            return inner
        except (json.JSONDecodeError, ValueError):
            return None

    def books_json_path(self):
        return os.path.join(self.data_dir, "library.json")

    def managed_root(self):
        return os.path.join(self.data_dir, "books")

    def stderr_has(self, pattern):
        try:
            with open(self.err_path, encoding="utf-8", errors="replace") as f:
                return bool(re.search(pattern, f.read()))
        except OSError:
            return False

    def reap(self):
        if not self.proc:
            return
        try:
            self.proc.terminate()
            self.proc.wait(timeout=5)
        except Exception:
            try:
                subprocess.run(["taskkill", "/PID", str(self.proc.pid), "/T", "/F"],
                               capture_output=True)
            except Exception:
                pass
        if self._errf:
            self._errf.close()
            self._errf = None


def ensure_fixtures():
    man_path = os.path.join(FIXTURE_DIR, "manifest.json")
    if not os.path.exists(man_path) or not os.path.exists(
            os.path.join(FIXTURE_DIR, "book-10mib.txt")):
        print("[*] generating fixtures (deterministic)…")
        subprocess.run([sys.executable, os.path.join(FIXTURE_DIR, "gen_fixtures.py")],
                       check=True)
    with open(man_path, encoding="utf-8") as f:
        return json.load(f)["fixtures"]




def http_get_retry(inst, path, tries=3, timeout=600):
    """长 GET（toc/chapter = 全量重解析，170-220s 忙段）传输级重试——r3
    记录的环境脆弱性（共居前端/进程生命期非确定）驱动卫生；内容断言不放松。"""
    last = (None, "no attempt", 0.0)
    for i in range(tries):
        st, body, dt = inst.http("GET", path, timeout=timeout)
        if st is not None:
            return st, body, dt
        last = (st, body, dt)
        time.sleep(3)
    return last

def toc_count(toc):
    """toc 响应形态：list 直读；dict 取 chapters。"""
    if isinstance(toc, list):
        return len(toc)
    if isinstance(toc, dict):
        return len(toc.get("chapters", []))
    return -1

def book_manifest(manifests, fname):
    return next(m for m in manifests if m["file"] == fname)


def verify_book_content(inst, case_prefix, man, bid, deep=False):
    """副本 hash + TOC 计数 + 逐章核对（默认抽首/中/尾；deep=全量）。"""
    # 受管副本逐字节：索引记录的 managed_path 指向 data 目录内副本。
    books = None
    try:
        with open(inst.books_json_path(), encoding="utf-8") as f:
            books = json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        record(f"{case_prefix}_index_readable", False, f"books.json: {e}")
        return
    entry = next((b for b in books.get("books", []) if b.get("book_id") == bid), None)
    record(f"{case_prefix}_index_entry", entry is not None,
           f"entry={'yes' if entry else 'MISSING'}")
    if not entry:
        return
    mp = entry.get("managed_path", "")
    ok_hash = os.path.exists(mp) and sha256_file(mp) == man["sha256"]
    record(f"{case_prefix}_managed_copy_bytes", ok_hash,
           f"sha256 match={ok_hash} path={os.path.basename(mp)}")
    # TOC 计数。
    st, toc_body, _ = http_get_retry(inst, f"/api/library/books/{bid}/toc")
    toc = inst.parse_reply(toc_body)
    n_got = toc_count(toc) if toc else -1
    record(f"{case_prefix}_toc_count", st == 200 and n_got == man["chapter_count"],
           f"status={st} chapters={n_got} want={man['chapter_count']}")
    # 逐章：抽样 or 全量；title + body sha256（manifest 口径 = trim 后）。
    chs = man["chapters"]
    sample = chs if deep else [chs[0], chs[len(chs) // 2], chs[-1]]
    bad = []
    for c in sample:
        st, body, _ = http_get_retry(
            inst, f"/api/library/chapter?book_id={bid}&number={c['number']}")
        ch = inst.parse_reply(body) if st == 200 else None
        if not ch:
            bad.append((c["number"], f"status={st}"))
            continue
        got_sha = hashlib.sha256(ch.get("body", "").encode("utf-8")).hexdigest()
        if got_sha != c["body_sha256"] or ch.get("title") != c["title"]:
            bad.append((c["number"],
                        f"title={ch.get('title')!r} sha={got_sha[:10]}≠{c['body_sha256'][:10]}"))
    record(f"{case_prefix}_chapter_bodies_{'all' if deep else 'sample'}",
           not bad, f"{len(sample)} checked, bad={bad[:3]}")


def run_v17(fmt, manifests, auto_bin, reader_dir, cold_restart=False):
    fname = f"book-10mib.{fmt}"
    man = book_manifest(manifests, fname)
    tag = f"v17-{fmt}"
    inst = Instance(tag, auto_bin, reader_dir, IMPORT_PROFILE)
    try:
        if not inst.launch():
            record(f"{tag}_instance", False, "backend not live")
            return
        record(f"{tag}_instance", True, f"port={inst.port}")
        fx = os.path.join(FIXTURE_DIR, fname)
        st, body, dt = inst.import_book(fx)
        reply = inst.parse_reply(body) if body else None
        ok = st == 200 and reply and reply.get("code") == "ok"
        record(f"{tag}_import_ok", ok, f"status={st} dt={dt:.1f}s reply={str(reply)[:90]}")
        raw[f"{tag}_import"] = {"status": st, "elapsed_s": round(dt, 1), "reply": reply}
        if not ok:
            return
        bid = reply.get("book_id", "")
        # T-26（F-781-R03 修正）：meta 索引（阶段 E T-25）落地后章 GET
        # = O(章) 窗读——deep 全章逐字 oracle 从 ~67h 降为分钟级，必选。
        verify_book_content(inst, tag, man, bid, deep=True)
        if cold_restart:
            # 冷启动：reap 本 run 拥有的进程 → 同库重启 → TOC/中章不变。
            old_pid = inst.proc.pid
            inst.reap()
            time.sleep(1.5)
            inst2 = Instance(f"{tag}-cold", auto_bin, reader_dir, IMPORT_PROFILE)
            # 复用同一 data 目录（Instance 会建新目录——手动指到旧的；
            # launch 的 env 以 self.data_dir 为准）。
            shutil.rmtree(inst2.data_dir, ignore_errors=True)
            inst2.data_dir = inst.data_dir
            if not inst2.launch():
                record(f"{tag}_cold_restart_live", False, "relaunch not live")
                return
            new_pid = inst2.proc.pid
            st, body, _ = http_get_retry(inst2, f"/api/library/books/{bid}/toc")
            toc = inst2.parse_reply(body) if body else None
            n = toc_count(toc) if toc else -1
            record(f"{tag}_cold_restart_live", new_pid != old_pid and st == 200,
                   f"pid {old_pid}→{new_pid} toc_status={st} chapters={n}")
            # T-26：冷重启后全章复扫（deep——索引随库持久，重启零重解析）。
            verify_book_content(inst2, f"{tag}_cold", man, bid, deep=True)
            inst2.reap()
            return
    finally:
        inst.reap()


def run_v19(manifests, auto_bin, reader_dir):
    """非空库 + 低预算 10MiB → 拒写；坏编码 → decode_error；库不变。"""
    tiny = book_manifest(manifests, "book-tiny.txt")
    big = book_manifest(manifests, "book-10mib.txt")
    inst = Instance("v19-low", auto_bin, reader_dir,
                    {"AUTO_HTTP_STEP_BUDGET": "1000000",
                     "AUTO_HTTP_REQUEST_TIMEOUT_MS": "300000"})
    try:
        if not inst.launch():
            record("v19_instance", False, "backend not live")
            return
        # 前提：小书导入成功（库非空 + 通道可用正对照）。
        st, body, _ = inst.import_book(os.path.join(FIXTURE_DIR, tiny["file"]))
        r = inst.parse_reply(body) if body else None
        record("v19_seed_small_ok", st == 200 and r and r.get("code") == "ok",
               f"status={st} reply={str(r)[:80]}")
        idx_before = open(inst.books_json_path(), "rb").read() if os.path.exists(inst.books_json_path()) else b""
        managed_before = sorted(os.listdir(inst.managed_root()))

        # 低预算 10MiB：指名资源拒绝，无业务副作用。
        st, body, dt = inst.import_book(os.path.join(FIXTURE_DIR, big["file"]))
        budget_named = "per-request cumulative budget" in body
        record("v19_low_budget_rejects",
               st == 500 and budget_named, f"status={st} dt={dt:.1f}s named={budget_named}")
        idx_after = (open(inst.books_json_path(), "rb").read()
                     if os.path.exists(inst.books_json_path()) else b"")
        managed_after = sorted(os.listdir(inst.managed_root()))
        record("v19_index_bytes_unchanged", idx_before == idx_after,
               f"bytes {len(idx_before)}=={len(idx_after)}")
        record("v19_no_managed_partial_write", managed_before == managed_after,
               f"dirs {managed_before} == {managed_after}")

        # 坏编码：decode_error 业务回执（既有路径），库不变。
        st, body, _ = inst.import_book(os.path.join(FIXTURE_DIR, "book-bad.bin"))
        r = inst.parse_reply(body) if body else None
        record("v19_bad_encoding_rejected",
               st == 200 and r and r.get("code") == "decode_error",
               f"status={st} code={r.get('code') if r else None}")
        idx_final = (open(inst.books_json_path(), "rb").read()
                     if os.path.exists(inst.books_json_path()) else b"")
        record("v19_library_unchanged_final", idx_final == idx_before,
               f"index bytes unchanged={idx_final == idx_before}")
    finally:
        inst.reap()



def run_v15_boot(auto_bin, reader_dir):
    """V-15：非法 env 宿主启动 fail-fast——指名错误 + 非零退出（真实 CLI）。"""
    for raw in ["0", "-5", "1.5", "abc", "99999999999999999999", "5000000001"]:
        env = dict(os.environ, AUTO_HTTP_STEP_BUDGET=raw,
                   AUTO_READER_DATA=os.environ.get("P781_T17_ROOT", "D:/p781-t17") + "/boot-neg")
        try:
            r = subprocess.run([auto_bin, "run", "-r", "vm", "--no-merge"],
                               cwd=reader_dir, env=env, capture_output=True,
                               text=True, timeout=60)
            named = "AUTO_HTTP_STEP_BUDGET" in (r.stdout + r.stderr)
            record(f"v15_boot_{raw[:12]}", r.returncode != 0 and named,
                   f"rc={r.returncode} named={named}")
        except subprocess.TimeoutExpired:
            record(f"v15_boot_{raw[:12]}", False, "timeout (fail-fast must be immediate)")

def run_v14_dual(manifests, auto_bin, reader_dir):
    """（T-18 段）默认实例拒 + 1G 实例成，双实例同跑不串线。"""
    big = book_manifest(manifests, "book-10mib.txt")
    fx = os.path.join(FIXTURE_DIR, big["file"])
    # 默认实例（无预算 env）。
    a = Instance("v14-default", auto_bin, reader_dir,
                 {"AUTO_HTTP_REQUEST_TIMEOUT_MS": "300000"})
    b = Instance("v14-1g", auto_bin, reader_dir, IMPORT_PROFILE)
    try:
        if not (a.launch() and b.launch()):
            record("v14_dual_live", False, f"a={a.port} b={b.port}")
            return
        # 先让 1G 实例导入成功（152s 级），期间默认实例保持默认边界。
        stb, bodyb, dtb = b.import_book(fx)
        rb = b.parse_reply(bodyb) if bodyb else None
        record("v14_1g_succeeds", stb == 200 and rb and rb.get("code") == "ok",
               f"status={stb} dt={dtb:.1f}s")
        # 默认实例同文件 → 10M 默认边界耗尽（决策报告锚点复核）。
        sta, bodya, dta = a.import_book(fx)
        named = "instruction budget exhausted" in bodya
        record("v14_default_rejects", sta == 500 and named,
               f"status={sta} dt={dta:.1f}s default-10M-exhausted={named}")
        # 库隔离：默认实例库无 1G 实例的书。
        idx = (open(a.books_json_path(), encoding="utf-8").read()
               if os.path.exists(a.books_json_path()) else "")
        bid = (rb or {}).get("book_id", "")
        record("v14_no_cross_instance", not (bid and bid in idx),
               f"default-lib contains 1g book={bid in idx if bid else 'n/a'}")
        # 256KiB 边界锚点复核（默认档成功）。
        small = book_manifest(manifests, "book-small.txt")
        st2, body2, _ = a.import_book(os.path.join(FIXTURE_DIR, small["file"]))
        r2 = a.parse_reply(body2) if body2 else None
        record("v14_default_small_still_ok", st2 == 200 and r2 and r2.get("code") == "ok",
               f"status={st2}")
    finally:
        a.reap()
        b.reap()




def run_w03(manifests, auto_bin, reader_dir):
    """W-03（T-26/F-781-R03）：损坏章负例——受管副本章窗内字节变异后，
    deep 全章 oracle 必须检出（body_sha256 失配）；未变异章不受扰。
    meta 索引窗读意味着变异只影响所属章窗（偏移漂移不波及他章）。
    用独立 doomed 实例（副本被改，库不回收）。"""
    man = book_manifest(manifests, "book-10mib.txt")
    inst = Instance("w03-corrupt", auto_bin, reader_dir, IMPORT_PROFILE)
    try:
        if not inst.launch():
            record("w03_instance", False, "backend not live")
            return
        fx = os.path.join(FIXTURE_DIR, man["file"])
        st, body, _ = inst.import_book(fx)
        reply = inst.parse_reply(body) if body else None
        if not (st == 200 and reply and reply.get("code") == "ok"):
            record("w03_import", False, f"status={st}")
            return
        record("w03_import", True, "ok")
        bid = reply.get("book_id", "")
        with open(inst.books_json_path(), encoding="utf-8") as f:
            entry = next((b for b in json.load(f).get("books", [])
                          if b.get("book_id") == bid), None)
        mp = entry.get("managed_path", "")
        size = os.path.getsize(mp)
        # 变异点：~2/3 处的一个字节（落在某章窗内；ASCII 空格 ↔ 'X'）。
        off = int(size * 2 / 3)
        with open(mp, "r+b") as f:
            f.seek(off)
            orig = f.read(1)
            f.seek(off)
            f.write(b"X" if orig != b"X" else b"Y")
        # 定位受影响章：manifest 顺序扫描累计字节（章体近似——以内容
        # sha 失配为准，不要求精确归章）。
        bad = scan_chapters(inst, bid, man)
        record("w03_corrupted_chapter_detected", bool(bad),
               f"bad={bad[:3]} corrupt_at={off}")
        # 未受扰区：首章（变异点 2/3 处不可能落在首章窗）必须仍逐字一致。
        st2, body2, _ = http_get_retry(inst, f"/api/library/chapter?book_id={bid}&number=1")
        ch = inst.parse_reply(body2) if st2 == 200 else None
        c1 = man["chapters"][0]
        ok1 = ch and hashlib.sha256(
            ch.get("body", "").encode("utf-8")).hexdigest() == c1["body_sha256"]
        record("w03_intact_chapter_unaffected", bool(ok1), "chapter 1 identical")
    finally:
        inst.reap()


def scan_chapters(inst, bid, man):
    """全章 body_sha256 扫描，返回失配 [(number, reason)]。"""
    bad = []
    for c in man["chapters"]:
        st, body, _ = http_get_retry(
            inst, f"/api/library/chapter?book_id={bid}&number={c['number']}")
        ch = inst.parse_reply(body) if st == 200 else None
        if not ch:
            bad.append((c["number"], f"status={st}"))
            continue
        got = hashlib.sha256(ch.get("body", "").encode("utf-8")).hexdigest()
        if got != c["body_sha256"]:
            bad.append((c["number"], "sha-mismatch"))
    return bad


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--auto-bin",
                    default=os.environ.get("AUTO_BIN",
                                           os.path.join(REPO_ROOT, "..", "auto-man", "target/debug/auto.exe")))
    ap.add_argument("--reader-dir", default=r"D:\autostack\auto-reader")
    ap.add_argument("--out-dir", default=os.environ.get("P781_T17_OUT", "D:/p781-t17/out"))
    ap.add_argument("--only", default="", help="逗号分隔用例过滤（v17_txt,…）")
    args = ap.parse_args()
    os.makedirs(args.out_dir, exist_ok=True)
    if not os.path.exists(args.auto_bin):
        print(f"[-] auto bin missing: {args.auto_bin}")
        return 3
    if not os.path.isdir(args.reader_dir):
        print(f"[-] reader dir missing: {args.reader_dir}")
        return 3

    manifests = ensure_fixtures()
    only = set(x for x in args.only.split(",") if x)
    legs = []
    if not only or {"v17_txt", "v17_cold_restart"} & only:
        legs.append(("v17_txt",
                     lambda: run_v17("txt", manifests, args.auto_bin, args.reader_dir,
                                     cold_restart=(not only or "v17_cold_restart" in only))))
    if not only or "v17_md" in only:
        legs.append(("v17_md",
                     lambda: run_v17("md", manifests, args.auto_bin, args.reader_dir)))
    if not only or "v19_low_budget" in only:
        legs.append(("v19_low_budget", lambda: run_v19(manifests, args.auto_bin, args.reader_dir)))
    if not only or "v14_dual_instance" in only:
        legs.append(("v14_dual_instance",
                     lambda: run_v14_dual(manifests, args.auto_bin, args.reader_dir)))
    if not only or "v15_boot" in only:
        legs.append(("v15_boot", lambda: run_v15_boot(args.auto_bin, args.reader_dir)))
    if not only or "w03_corrupt" in only:
        legs.append(("w03_corrupt", lambda: run_w03(manifests, args.auto_bin, args.reader_dir)))
    # 单腿异常不杀全家（此前一次解析崩溃吞掉后续三腿——4 分钟导入学费）。
    for name, leg in legs:
        try:
            leg()
        except Exception as e:  # noqa: BLE001
            record(f"driver_error_{name}", False, repr(e))

    cli_sha = sha256_file(args.auto_bin) if os.path.exists(args.auto_bin) else "n/a"
    report = {"cli": args.auto_bin, "cli_sha256": cli_sha,
              "import_profile": IMPORT_PROFILE,
              "fixtures": [{"file": m["file"], "sha256": m["sha256"],
                            "bytes": m["bytes"],
                            "chapters": m.get("chapter_count")} for m in manifests],
              "results": results, "raw": raw}
    with open(os.path.join(args.out_dir, "reader_import_budget_report.json"), "w",
              encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    failed = [r for r in results if not r["ok"]]
    print("=" * 64)
    print(f"[{'+' if not failed else '-'}] {len(results) - len(failed)}/{len(results)} "
          f"checks passed → {args.out_dir}")
    return 2 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
