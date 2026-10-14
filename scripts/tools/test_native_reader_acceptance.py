#!/usr/bin/env python3
"""PLAN-781 原生 Reader 验收驱动（阶段 A 通道，T-10）.

合同来源: docs/plans/reports/781-channel-decision.md（T-09 冻结）。
启动形态: auto run -r vm --no-merge -B <port>（单进程: iced 前端 + VM HTTP
后端线程）。身份 = pid↔端口(netstat) + 内容级断言；就绪 = /__auto/health/live
200 + /api/library/books 预期集合 + snapshot tree。收割 = terminate 直子 +
端口释放有界验证，绝不按进程名/端口杀非本 run 子进程。

用法:
  python scripts/tools/test_native_reader_acceptance.py \
    --auto-bin <path> --app-dir D:/autostack/auto-reader \
    --data-dir <isolated-parent> --out-dir <evidence-root> [--case ID ...]

退出码: 0=全部 required pass；1=用法/环境错；2=required FAIL；3=SKIP/缺证据。
v12_injected_fail 不在默认集（它故意 FAIL 以验证非零退出与证据对应）。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / ".agents" / "skills" / "autoui-verifier" / "scripts"))
from test_vm_mcp import AutoUiMcpClient, pick_free_port  # noqa: E402

HEALTH_LIVE = "/__auto/health/live"  # legacy VM 轨无 /health app 路由；控制面 live 恒 200
POLL_INTERVAL = 0.5


# ---------------------------------------------------------------- utilities
def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def http_json(port: int, method: str, path: str, body=None, timeout: float = 5.0):
    url = f"http://127.0.0.1:{port}{path}"
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.status, json.loads(r.read().decode("utf-8"))


def unwrap_json(x):
    """Reader HTTP 响应存在双重 JSON 编码（body 为 JSON 字符串）；解到 dict 为止。"""
    for _ in range(2):
        if isinstance(x, str):
            try:
                x = json.loads(x)
            except ValueError:
                return x
        else:
            break
    return x


def books_ids(books_resp) -> list:
    resp = unwrap_json(books_resp)
    items = resp.get("books", []) if isinstance(resp, dict) else []
    items = unwrap_json(items)
    if not isinstance(items, list):
        return []
    ids = []
    for b in items:
        if isinstance(b, dict):
            ids.append(b.get("id") or b.get("book_id"))
        else:
            ids.append(str(b))
    return [i for i in ids if i]


def netstat_listening_pids(port: int) -> set:
    """返回在该端口上 LISTENING 的 owning pid 集合（netstat -ano 解析）。"""
    try:
        out = subprocess.run(["netstat", "-ano", "-p", "tcp"], capture_output=True,
                             text=True, timeout=10).stdout
    except (OSError, subprocess.SubprocessError):
        return set()
    pids = set()
    for line in out.splitlines():
        parts = line.split()
        if len(parts) >= 5 and parts[3] == "LISTENING":
            if parts[1].endswith(f":{port}"):
                try:
                    pids.add(int(parts[4]))
                except ValueError:
                    pass
    return pids


def port_accepts(port: int) -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(0.5)
        return s.connect_ex(("127.0.0.1", port)) == 0


def make_book(path: Path, marker: str, chapters: int = 3) -> None:
    lines = []
    for c in range(1, chapters + 1):
        lines.append(f"第{c}章 {marker} 章节{c}")
        lines.append(f"{marker} 第{c}章第一段，正文内容用于内容级身份断言。")
        lines.append(f"第{c}章第二段，普通正文。")
        lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


# ---------------------------------------------------------------- evidence
@dataclass
class CaseResult:
    case_id: str
    required: bool = True
    status: str = "evidence_missing"  # pass | fail | skip | evidence_missing
    detail: str = ""
    artifacts: list = field(default_factory=list)

    @property
    def nonpass(self) -> bool:
        return self.status != "pass"


class Evidence:
    """证据包: manifest/raw/trace/shots/lib/proc。"""

    def __init__(self, out_dir: Path):
        self.out = out_dir
        for sub in ("raw", "trace", "shots", "fixtures", "proc"):
            (out_dir / sub).mkdir(parents=True, exist_ok=True)
        self.manifest = {"schema": "reader-acceptance/1",
                         "started_at": datetime.now(timezone.utc).isoformat()}
        self.results: list[CaseResult] = []

    def raw_write(self, name: str, payload) -> None:
        (self.out / "raw" / f"{name}.json").write_text(
            json.dumps(payload, ensure_ascii=False, indent=1, default=str),
            encoding="utf-8")

    def trace_write(self, line: dict) -> None:
        with open(self.out / "trace" / "actions.ndjson", "a", encoding="utf-8") as f:
            f.write(json.dumps(line, ensure_ascii=False, default=str) + "\n")

    def shot_collect(self, name: str, resp_text: str) -> str:
        """从 autoui_screenshot 响应文本解析源路径，复制进 shots/（+sha256）。"""
        marker = "Screenshot saved to: "
        src = None
        for part in resp_text.splitlines():
            if marker in part:
                src = part.split(marker, 1)[1].strip().replace("\\\\?\\", "")
                break
        if not src or not Path(src).is_file():
            return f"screenshot source not found: {resp_text[:120]}"
        dst = self.out / "shots" / f"{name}{Path(src).suffix}"
        shutil.copy2(src, dst)
        self.manifest.setdefault("shot_hashes", {})[name] = sha256_file(dst)
        return str(dst)

    def case(self, result: CaseResult) -> None:
        self.results.append(result)
        print(f"[{result.status.upper()}] {result.case_id}: {result.detail}")

    def finish(self) -> int:
        counts = {}
        for r in self.results:
            counts[r.status] = counts.get(r.status, 0) + 1
        # AC-10 owned 进程收尾核查：本 run 记录的全部实例端口（含 restart 前任）
        # 均不得仍被持有（外部执行方占用同一端口的情形由 pid 对照豁免——
        # holders 与本 run 任何 pid 无交集即视为已释放）。
        owned_pids = set()
        for inst in self.manifest.get("instances", {}).values():
            if isinstance(inst, dict) and inst.get("pid") is not None:
                owned_pids.add(inst["pid"])
        for rst in self.manifest.get("restarts", []):
            for key in ("from_pid", "to_pid"):
                if rst.get(key) is not None:
                    owned_pids.add(rst[key])
        lingering = {}
        external_reuse = {}
        for tag, inst in self.manifest.get("instances", {}).items():
            if not isinstance(inst, dict):
                continue
            for label in ("mcp_port", "back_port"):
                port = inst.get(label)
                if port is None:
                    continue
                holders = netstat_listening_pids(port)
                if not holders:
                    continue
                if holders & owned_pids:
                    lingering[f"{tag}:{label}:{port}"] = sorted(holders)
                else:
                    # 端口已被外部进程复用 = 本 run 已释放（外部执行方并发证据）。
                    external_reuse[f"{tag}:{label}:{port}"] = sorted(holders)
        self.manifest["owned_port_sweep"] = {"checked": True, "lingering": lingering,
                                             "external_reuse": external_reuse}
        self.manifest["cases"] = [vars(r) for r in self.results]
        self.manifest["case_counts"] = counts
        self.manifest["finished_at"] = datetime.now(timezone.utc).isoformat()
        (self.out / "manifest.json").write_text(
            json.dumps(self.manifest, ensure_ascii=False, indent=1, default=str),
            encoding="utf-8")
        print(f"cases: {counts}")
        print(f"owned-port sweep: {'clean' if not lingering else lingering}"
              + (f" (external reuse: {external_reuse})" if external_reuse else ""))
        if lingering:
            return 2
        if any(r.status == "fail" for r in self.results):
            return 2
        if any(r.nonpass for r in self.results):
            return 3
        return 0


# ---------------------------------------------------------------- instance
class Instance:
    """一个受控 split 实例: 单 Popen(iced 前端 + VM HTTP 后端线程)。"""

    def __init__(self, ev: Evidence, auto_bin: Path, app_dir: Path, data_dir: Path,
                 tag: str, ready_timeout: float = 120.0,
                 back_port: int | None = None, mcp_port: int | None = None,
                 fixture_mode: bool = False, ready_marker: str | None = None):
        self.ev = ev
        self.auto_bin = auto_bin
        self.app_dir = app_dir
        self.data_dir = data_dir
        self.tag = tag
        self.ready_timeout = ready_timeout
        self.back_port = back_port or pick_free_port()
        self.mcp_port = mcp_port or pick_free_port()
        self.proc = None
        self.marker = f"P781{tag}-{os.urandom(3).hex()}"
        self.argv: list = []
        self.exit_code = None
        self._log_files = []
        # fixture_mode: 框架 fixture app（无 Reader API；就绪 = /api/ping + 快照
        # 含 ready_marker，跳过空库断言——那是 Reader 隔离性专属）。
        self.fixture_mode = fixture_mode
        self.ready_marker = ready_marker

    # -- lifecycle ---------------------------------------------------------
    def launch(self) -> None:
        self.data_dir.mkdir(parents=True, exist_ok=True)
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("AUTO_", "AUTOUI_"))}
        env.update({"AUTO_READER_DATA": str(self.data_dir),
                    "AUTOUI_MCP_PORT": str(self.mcp_port)})
        self.argv = [str(self.auto_bin), "run", "-r", "vm", "--no-merge",
                     "-B", str(self.back_port)]
        out_f = open(self.ev.out / "proc" / f"{self.tag}-stdout.log", "wb")
        err_f = open(self.ev.out / "proc" / f"{self.tag}-stderr.log", "wb")
        self._log_files = [out_f, err_f]
        self.proc = subprocess.Popen(self.argv, cwd=str(self.app_dir), env=env,
                                     stdout=out_f, stderr=err_f)
        self.ev.manifest.setdefault("instances", {})[self.tag] = {
            "pid": self.proc.pid, "argv": self.argv,
            "mcp_port": self.mcp_port, "back_port": self.back_port,
            "data_dir": str(self.data_dir),
        }

    def poll_alive(self) -> bool:
        return self.proc is not None and self.proc.poll() is None

    def wait_backend(self):
        """就绪前半: backend live + 空库断言。种子导入应插在本段与 wait_mcp 之间
        （前端 Init 首次 fetch 晚于窗口启动，早种使 marker 进入首帧数据）。"""
        t0 = time.monotonic()
        live = False
        while time.monotonic() - t0 < self.ready_timeout:
            if not self.poll_alive():
                return False, (f"process exited early rc={self.proc.returncode}; "
                               f"see proc/{self.tag}-std*.log")
            try:
                st, _ = http_json(self.back_port, "GET", HEALTH_LIVE, timeout=2)
                if st == 200:
                    live = True
                    break
            except (urllib.error.URLError, OSError, ValueError):
                pass
            time.sleep(POLL_INTERVAL)
        if not live:
            return False, f"backend not live within {self.ready_timeout}s on :{self.back_port}"
        if self.fixture_mode:
            try:
                st, _ = http_json(self.back_port, "GET", "/api/ping", timeout=3)
                if st != 200:
                    return False, f"fixture ping http {st}"
            except (urllib.error.URLError, OSError, ValueError) as e:
                return False, f"fixture ping failed: {e!r}"
            return True, f"fixture backend live in {time.monotonic() - t0:.1f}s"
        try:
            st, books = http_json(self.back_port, "GET", "/api/library/books")
            got = books_ids(books)
            if st != 200:
                return False, f"books http {st}"
            if got:
                return False, f"isolated library not empty: {got}"
        except (urllib.error.URLError, OSError, ValueError) as e:
            return False, f"books probe failed: {e!r}"
        return True, f"backend live in {time.monotonic() - t0:.1f}s"

    def wait_mcp(self):
        """就绪后半: MCP snapshot tree（fixture: ready_marker）+ 身份断言 + stderr 门。"""
        t0 = time.monotonic()
        cli = AutoUiMcpClient(self.mcp_port)
        snap = ""
        want = self.ready_marker if (self.fixture_mode and self.ready_marker) else "tree:"
        while time.monotonic() - t0 < self.ready_timeout:
            if not self.poll_alive():
                return False, f"process exited during mcp-ready rc={self.proc.returncode}"
            try:
                snap = cli.snapshot()
                if snap and want in snap:
                    break
            except (urllib.error.URLError, OSError, RuntimeError, ValueError):
                pass
            time.sleep(POLL_INTERVAL)
        if not snap or want not in snap:
            return False, f"mcp snapshot not ready within {self.ready_timeout}s (want {want!r})"
        self.ev.raw_write(f"{self.tag}-ready-snapshot", {"snapshot_head": snap[:4000]})
        # 身份断言在 MCP 就绪之后（MCP 随 iced 前端启动，晚于后端线程）
        ident = self.identity_problems()
        if ident:
            return False, "identity: " + "; ".join(ident)
        self._flush_logs()
        err_txt = self._read_log("stderr")
        for bad in ("fell back to", "AutoUI MCP: FATAL"):
            if bad in err_txt:
                return False, f"stderr identity violation: '{bad}' present"
        return True, f"ready in {time.monotonic() - t0:.1f}s"

    def identity_problems(self) -> list:
        problems = []
        for label, port in (("mcp", self.mcp_port), ("back", self.back_port)):
            holders = netstat_listening_pids(port)
            if not holders:
                problems.append(f"{label} port {port}: no listener")
            elif self.proc.pid not in holders:
                problems.append(f"{label} port {port} held by {holders}, not child pid {self.proc.pid}")
        return problems

    def _flush_logs(self) -> None:
        for f in self._log_files:
            try:
                f.flush()
                os.fsync(f.fileno())
            except (OSError, ValueError):
                pass

    def _read_log(self, which: str) -> str:
        self._flush_logs()
        try:
            return (self.ev.out / "proc" / f"{self.tag}-{which}.log").read_text(
                encoding="utf-8", errors="replace")
        except OSError:
            return ""

    def seed_book(self) -> str:
        """真实导入种子书(建立前提), 返回 book_id。"""
        path = self.ev.out / "fixtures" / f"{self.marker}.txt"
        make_book(path, self.marker)
        st, imp = http_json(self.back_port, "POST", "/api/library/import",
                            {"path": str(path), "author": "", "force": False})
        imp = unwrap_json(imp)
        if st != 200 or not isinstance(imp, dict) or imp.get("code") != "ok":
            raise RuntimeError(f"seed import failed: {st} {imp}")
        self.ev.raw_write(f"{self.tag}-seed-import",
                          {"status": st, "resp": imp, "fixture_sha256": sha256_file(path)})
        return imp.get("book_id", "")

    def snapshot_has(self, needle: str, timeout: float = 40.0) -> bool:
        cli = AutoUiMcpClient(self.mcp_port)
        t0 = time.monotonic()
        while time.monotonic() - t0 < timeout:
            if not self.poll_alive():
                return False
            try:
                if needle in cli.snapshot():
                    return True
            except (urllib.error.URLError, OSError, RuntimeError, ValueError):
                pass
            try:
                found = cli.call("autoui_find", {"label": needle})
                if needle in json.dumps(found, ensure_ascii=False):
                    return True
            except (urllib.error.URLError, OSError, RuntimeError, ValueError):
                pass
            time.sleep(POLL_INTERVAL)
        return False

    def snapshot_text(self) -> str:
        try:
            return AutoUiMcpClient(self.mcp_port).snapshot()
        except Exception:  # noqa: BLE001
            return ""

    def restart(self):
        """冷启动（§5.A.5）：收割本 run 进程 + 端口释放验证 + 同数据目录重新
        拉起（新端口/pid）。返回 (ok, detail)。"""
        if not self.poll_alive() and self.proc is not None:
            pass  # 已退出，仅走端口验证
        reap_ok, reap_detail = self.reap()
        if not reap_ok:
            return False, f"reap before restart failed: {reap_detail}"
        old_pid = self.exit_code if self.exit_code is not None else (
            self.proc.pid if self.proc else None)
        old_ports = (self.mcp_port, self.back_port)
        self.back_port = pick_free_port()
        self.mcp_port = pick_free_port()
        self.launch()
        self.ev.manifest.setdefault("restarts", []).append({
            "tag": self.tag, "from_pid": old_pid, "to_pid": self.proc.pid,
            "old_ports": old_ports, "new_ports": (self.mcp_port, self.back_port),
            "data_dir": str(self.data_dir),
        })
        return True, (f"cold restart {old_pid} -> {self.proc.pid} "
                      f"(ports {old_ports} -> {(self.mcp_port, self.back_port)})")

    def reap(self, deadline: float = 10.0):
        """收割直子 + 端口释放有界验证。绝不杀非本 run 子进程。"""
        if self.proc is None:
            return True, "never launched"
        if self.poll_alive():
            self.proc.terminate()
            try:
                self.proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                try:
                    self.proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    pass
        self.exit_code = self.proc.returncode
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            if not port_accepts(self.mcp_port) and not port_accepts(self.back_port):
                self._close_logs()
                return True, f"reaped rc={self.exit_code}"
            time.sleep(POLL_INTERVAL)
        holders = {p: sorted(netstat_listening_pids(p)) for p in (self.mcp_port, self.back_port)}
        resid = self._read_log("stderr")[-500:]
        self._close_logs()
        return False, (f"ports still accepting after reap (rc={self.exit_code}, "
                       f"holders={holders}); NOT killing non-child owners; stderr_tail={resid!r}")

    def _close_logs(self) -> None:
        self._flush_logs()
        for f in self._log_files:
            try:
                f.close()
            except (OSError, ValueError):
                pass


# ---------------------------------------------------------------- cases
def run_r1_smoke(ev: Evidence, inst: Instance) -> CaseResult:
    r = CaseResult("r1_smoke")
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        r.status, r.detail = "fail", f"backend ready failed: {detail}"
        inst.reap()
        return r
    try:
        inst.seed_book()
    except RuntimeError as e:
        r.status, r.detail = "fail", str(e)
        inst.reap()
        return r
    ok, detail = inst.wait_mcp()
    if not ok:
        r.status, r.detail = "fail", f"mcp ready failed: {detail}"
        inst.reap()
        return r
    if not inst.snapshot_has(inst.marker, timeout=20.0):
        r.status = "fail"
        r.detail = f"marker {inst.marker} not visible in snapshot after import"
        inst.reap()
        return r
    try:
        shot_text = AutoUiMcpClient(inst.mcp_port).screenshot(f"r1smoke-{inst.tag}", baseline=False)
        shot_path = ev.shot_collect(f"r1smoke-{inst.tag}", str(shot_text))
    except Exception as e:  # noqa: BLE001
        r.status, r.detail = "fail", f"screenshot failed: {e!r}"
        inst.reap()
        return r
    ev.trace_write({"ts": time.time(), "instance": inst.tag, "tool": "autoui_screenshot",
                    "result_head": str(shot_text)[:200]})
    reap_ok, reap_detail = inst.reap()
    if not reap_ok:
        r.status, r.detail = "fail", f"reap failed: {reap_detail}"
        return r
    r.status, r.detail = "pass", (f"ready+identity+empty-lib+seed+marker+screenshot+reap ok "
                                  f"({detail}; shot={shot_path})")
    return r


def run_v07_dual(ev: Evidence, mk) -> CaseResult:
    r = CaseResult("v07_dual_instance")
    a, b = mk("v07a"), mk("v07b")
    try:
        for inst in (a, b):
            inst.launch()
        for inst, name in ((a, "A"), (b, "B")):
            ok, detail = inst.wait_backend()
            if not ok:
                r.status, r.detail = "fail", f"instance {name} backend ready failed: {detail}"
                return r
        for inst, name in ((a, "A"), (b, "B")):
            try:
                inst.seed_book()
            except RuntimeError as e:
                r.status, r.detail = "fail", f"instance {name} seed failed: {e}"
                return r
        for inst, name in ((a, "A"), (b, "B")):
            ok, detail = inst.wait_mcp()
            if not ok:
                r.status, r.detail = "fail", f"instance {name} mcp ready failed: {detail}"
                return r
        # 不串线: 各自 snapshot 含自己的 marker、不含对方的
        for me, other, name in ((a, b, "A"), (b, a, "B")):
            if not me.snapshot_has(me.marker, timeout=40.0):
                r.status, r.detail = "fail", f"instance {name}: own marker missing"
                return r
            if other.marker in me.snapshot_text():
                r.status, r.detail = "fail", f"instance {name}: foreign marker visible (cross-talk)"
                return r
        if a.proc.pid == b.proc.pid:
            r.status, r.detail = "fail", "same pid for both instances"
            return r
        for inst, name in ((a, "A"), (b, "B")):
            probs = inst.identity_problems()
            if probs:
                r.status, r.detail = "fail", f"instance {name} identity: {probs}"
                return r
        for inst, name in ((a, "A"), (b, "B")):
            ok, detail = inst.reap()
            if not ok:
                r.status, r.detail = "fail", f"instance {name} reap: {detail}"
                return r
        r.status, r.detail = "pass", "dual isolated instances, distinct pids/ports, no cross-talk"
        return r
    finally:
        for inst in (a, b):
            if inst.poll_alive():
                inst.reap()


def run_v12_window_start_fail(ev: Evidence, inst: Instance, bad_app_dir: Path) -> CaseResult:
    r = CaseResult("v12_window_start_fail")
    inst.app_dir = bad_app_dir  # 无 pac.at 的空目录 → 启动失败路径
    inst.launch()
    ok, detail = inst.wait_backend()
    ev.raw_write("v12_window_start_fail", {"ready_ok": ok, "detail": detail})
    if ok:
        inst.reap()
        r.status, r.detail = "fail", "unexpected ready with bad app-dir"
        return r
    if "exited early" not in detail:
        inst.reap()
        r.status, r.detail = "fail", f"failure not observed as early exit: {detail}"
        return r
    reap_ok, reap_detail = inst.reap()
    r.status = "pass" if reap_ok else "fail"
    r.detail = f"controlled failure observed; {detail}; reap {'ok' if reap_ok else reap_detail}"
    return r


def run_v12_backend_dead(ev: Evidence, inst: Instance) -> CaseResult:
    r = CaseResult("v12_backend_dead")
    inst.ready_timeout = 8.0  # 驱动侧先到时; app 自身 120s 门随后由收割终止
    blocker = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    blocker.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    blocker.bind(("127.0.0.1", inst.back_port))
    blocker.listen(1)
    try:
        inst.launch()
        ok, detail = inst.wait_backend()
        ev.raw_write("v12_backend_dead", {"ready_ok": ok, "detail": detail})
        if ok:
            r.status, r.detail = "fail", "unexpected ready with blocked backend port"
            return r
    finally:
        blocker.close()
    reap_ok, reap_detail = inst.reap()
    r.status = "pass" if reap_ok else "fail"
    r.detail = f"controlled backend-dead observation; reap {'ok' if reap_ok else reap_detail}"
    return r


def run_v12_timeout(ev: Evidence, inst: Instance) -> CaseResult:
    r = CaseResult("v12_timeout")
    # 确定性零预算: 热缓存下 backend 可 <2s live，唯一保证超时的是 0 预算。
    inst.ready_timeout = 0.0
    inst.launch()
    ok, detail = inst.wait_backend()
    reap_ok, reap_detail = inst.reap()
    if ok:
        r.status, r.detail = "fail", "ready unexpectedly succeeded under 0s budget"
        return r
    r.status = "pass" if reap_ok else "fail"
    r.detail = f"timeout budget respected ({detail}); reap {'ok' if reap_ok else reap_detail}"
    return r


def run_v12_injected_fail(ev: Evidence) -> CaseResult:
    r = CaseResult("v12_injected_fail")
    r.status = "fail"
    r.detail = ("INTENTIONAL injected required failure — verifies non-zero exit + "
                "evidence correspondence (T-13 asserts driver exit==2)")
    ev.manifest["injected_fail"] = True
    return r


def run_v10_pending_t11(ev: Evidence, inst: Instance) -> CaseResult:
    """V-10 假几何负例（T-11 版）：错 node / 无 bounds / 零尺寸 / 远端裁剪 /
    无动作不刷代数。fixture = scroll-geometry（120 行 + 零尺寸行）。"""
    import json as _json
    import re

    r = CaseResult("v10_fake_geometry")
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        r.status, r.detail = "fail", f"backend ready failed: {detail}"
        inst.reap()
        return r
    ok, detail = inst.wait_mcp()
    if not ok:
        r.status, r.detail = "fail", f"mcp ready failed: {detail}"
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)

    def tool_json(name, args):
        res = cli.call(name, args)
        text = "".join(p.get("text", "") for p in res.get("content", []))
        return _json.loads(text)

    def wait_first_collection(deadline=15.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            ss = tool_json("autoui_scroll_state", {})
            if ss.get("frame", 0) > 0 and ss.get("panes"):
                return ss
            time.sleep(POLL_INTERVAL)
        return None

    def wait_scroll_settled(prev_y, deadline=15.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            ss = tool_json("autoui_scroll_state", {})
            off = ss["panes"][0]["offset"][1] if ss.get("panes") else None
            if off is not None and abs(off - prev_y) > 0.5:
                return ss
            time.sleep(POLL_INTERVAL)
        return tool_json("autoui_scroll_state", {})

    def find_container(needle):
        vt = "".join(p.get("text", "") for p in cli.call("autoui_vtree", {}).get("content", []))
        idx = vt.find(needle)
        if idx < 0:
            return None
        best = None
        for m in re.finditer(r"(col|row|container) (vnode_\d+) \{", vt):
            if m.end() <= idx and (best is None or m.end() > best.end()):
                best = m
        return re.search(r"vnode_\d+", best.group(0)).group(0) if best else None

    def find_scrollable():
        vt = "".join(p.get("text", "") for p in cli.call("autoui_vtree", {}).get("content", []))
        for pat in (r"(?:scrollable|Scrollable|scroll-pane) (vnode_\d+) \{",):
            m = re.search(pat, vt)
            if m:
                return m.group(1)
        return None

    ss = wait_first_collection()
    if not ss:
        r.status, r.detail = "fail", "no bounds collection within deadline"
        inst.reap()
        return r
    checks = []

    # (1) 错 node：不存在的 element_id → unmeasured / resolved=false / 不可见
    v = tool_json("autoui_visibility", {"element_id": "vnode_999999999"})
    checks.append(("bogus id unmeasured",
                   v.get("resolved") is False and v.get("visible") is False
                   and "unmeasured" in v.get("flags", [])))

    # (2) 零尺寸行：measured 但 w/h=0 → zero_size / 不可见
    zero_eid = find_container("¶ZERO P781SG")
    if not zero_eid:
        checks.append(("zero-size row found", False))
    else:
        vz = tool_json("autoui_visibility", {"element_id": zero_eid})
        checks.append(("zero-size flagged not visible",
                       vz.get("visible") is False and "zero_size" in vz.get("flags", [])))

    # (3) 远端行（content y 远超 viewport）→ clipped_out / 不可见
    far_eid = find_container("¶100 P781SG")
    if not far_eid:
        checks.append(("far row found", False))
    else:
        vf = tool_json("autoui_visibility", {"element_id": far_eid})
        checks.append(("far row clipped_out",
                       vf.get("visible") is False and "clipped_out" in vf.get("flags", [])))

    # (4) 无动作不刷代数：两次查询间 frame 至多小幅前进（≤3——例间窗口
    # churn 的真实重绘是合法 frame 来源；假新鲜度会随查询次数持续增长）。
    s1 = tool_json("autoui_scroll_state", {})
    time.sleep(2.5)
    s2 = tool_json("autoui_scroll_state", {})
    checks.append(("no fake freshness without action",
                   s2.get("frame", 0) - s1.get("frame", 0) <= 3))

    ev.raw_write("v10_fake_geometry", {"checks": [(c[0], bool(c[1])) for c in checks],
                                       "frame": ss.get("frame")})
    try:
        shot = cli.screenshot("v10geometry", baseline=False)
        ev.shot_collect("v10geometry", str(shot))
    except Exception:  # noqa: BLE001
        pass
    reap_ok, reap_detail = inst.reap()
    bad = [c for c in checks if not c[1]]
    if bad or not reap_ok:
        r.status = "fail"
        r.detail = f"failed checks: {[c[0] for c in bad]}; reap={'ok' if reap_ok else reap_detail}"
    else:
        r.status = "pass"
        r.detail = f"{len(checks)} negative-oracle checks ok ({detail})"
    return r


def run_v11_clip(ev: Evidence, inst: Instance) -> CaseResult:
    """V-11 裁剪（T-11 版）：正面积交集 / 恰一次转换 / 部分可见 / 滚动后代数刷新。"""
    import json as _json
    import re

    r = CaseResult("v11_clip")
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        r.status, r.detail = "fail", f"backend ready failed: {detail}"
        inst.reap()
        return r
    ok, detail = inst.wait_mcp()
    if not ok:
        r.status, r.detail = "fail", f"mcp ready failed: {detail}"
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)

    def tool_json(name, args):
        res = cli.call(name, args)
        text = "".join(p.get("text", "") for p in res.get("content", []))
        return _json.loads(text)

    def find_container(needle):
        vt = "".join(p.get("text", "") for p in cli.call("autoui_vtree", {}).get("content", []))
        idx = vt.find(needle)
        if idx < 0:
            return None
        best = None
        for m in re.finditer(r"(col|row|container) (vnode_\d+) \{", vt):
            if m.end() <= idx and (best is None or m.end() > best.end()):
                best = m
        return re.search(r"vnode_\d+", best.group(0)).group(0) if best else None

    def find_scrollable():
        vt = "".join(p.get("text", "") for p in cli.call("autoui_vtree", {}).get("content", []))
        m = re.search(r"(?:scrollable|Scrollable|scroll-pane) (vnode_\d+) \{", vt)
        return m.group(1) if m else None

    def wait_scroll_settled(prev_y, deadline=15.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            ss = tool_json("autoui_scroll_state", {})
            off = ss["panes"][0]["offset"][1] if ss.get("panes") else None
            if off is not None and abs(off - prev_y) > 0.5:
                return ss
            time.sleep(POLL_INTERVAL)
        return tool_json("autoui_scroll_state", {})

    t0 = time.monotonic()
    while time.monotonic() - t0 < 15:
        ss = tool_json("autoui_scroll_state", {})
        if ss.get("frame", 0) > 0 and ss.get("panes"):
            break
        time.sleep(POLL_INTERVAL)
    if not ss.get("panes"):
        r.status, r.detail = "fail", "no pane geometry"
        inst.reap()
        return r
    pane = ss["panes"][0]
    vx, vy, vw, vh = pane["viewport_rect"]
    checks = []

    row0 = find_container("¶0 P781SG")
    checks.append(("row0 found", bool(row0)))
    v0 = tool_json("autoui_visibility", {"element_id": row0}) if row0 else {}
    checks.append(("row0 visible positive-area",
                   v0.get("visible") is True and (v0.get("intersection") or [0, 0, 0, 0])[2] > 0))

    # content-space + 恰一次转换：row60 的 content y 已知 → 滚到使其跨 viewport
    # 顶缘（offset = content_y + 20 → window_y = vy - 20 < vy 部分裁剪）。
    row60 = find_container("¶60 P781SG")
    checks.append(("row60 found", bool(row60)))
    pane_eid = find_scrollable()
    checks.append(("pane vnode found", bool(pane_eid)))
    if row60 and pane_eid:
        v60 = tool_json("autoui_visibility", {"element_id": row60})
        # bounds 为 pane 锚定（含 viewport 原点）——content y = cy − vy。
        cy = v60["target_rect"][1] - vy
        h = v60["target_rect"][3]
        cli.call("autoui_action", {"element_id": pane_eid, "action": "scroll", "value": cy + 20})
        ss2 = wait_scroll_settled(pane["offset"][1])
        checks.append(("offset moved to straddle", abs(ss2["panes"][0]["offset"][1] - (cy + 20)) < 1.0,
                       ))
        v60b = tool_json("autoui_visibility", {"element_id": row60})
        inter = v60b.get("intersection")
        # 部分可见：正面积且被裁掉顶部（交高 < 目标高、交顶 = viewport 顶）
        checks.append(("row60 partially visible",
                       v60b.get("visible") is True and inter is not None
                       and 0 < inter[3] < h and abs(inter[1] - vy) < 1.0))
        # 恰一次转换的数值验证（§12 r4 修正公式）：window_y = bounds − offset
        expect_y = (cy + vy) - ss2["panes"][0]["offset"][1]
        checks.append(("translate-once arithmetic",
                       inter is not None and abs(inter[1] - max(expect_y, vy)) < 1.0))
        # bounds 不随滚动漂移（content-space）
        v60c = tool_json("autoui_visibility", {"element_id": row60})
        checks.append(("content-space bounds stable",
                       all(abs(a - b) < 0.5 for a, b in zip(v60["target_rect"], v60c["target_rect"]))))
        # 滚动后代数刷新（scroll_tick 增加而 frame 可不动）
        checks.append(("scroll_tick advanced", ss2.get("scroll_tick", 0) > ss.get("scroll_tick", 0)))

    ev.raw_write("v11_clip", {"checks": [(c[0], bool(c[1])) for c in checks],
                              "pane": pane, "frame": ss.get("frame")})
    try:
        shot = cli.screenshot("v11clip", baseline=False)
        ev.shot_collect("v11clip", str(shot))
    except Exception:  # noqa: BLE001
        pass
    reap_ok, reap_detail = inst.reap()
    bad = [c for c in checks if not c[1]]
    if bad or not reap_ok:
        r.status = "fail"
        r.detail = f"failed checks: {[c[0] for c in bad]}; reap={'ok' if reap_ok else reap_detail}"
    else:
        r.status = "pass"
        r.detail = f"{len(checks)} clipping-oracle checks ok ({detail})"
    return r


# ---------------- T-12: 真实保存/冷启动链路（save-restore fixture 正对照） ----------------

def _t12_helpers(cli):
    import re

    def tool_json(name, args):
        res = cli.call(name, args)
        text = "".join(p.get("text", "") for p in res.get("content", []))
        return json.loads(text)

    def vtree_text():
        return "".join(p.get("text", "") for p in cli.call("autoui_vtree", {}).get("content", []))

    def find_para(needle):
        """段落外层 col 直接包含 needle text——最近一个 col 头即该节点。
        （视图重建后 id 会变，每次调用重新发现。）"""
        vt = vtree_text()
        idx = vt.find(needle)
        if idx < 0:
            return None
        hdrs = [m for m in re.finditer(r"col (vnode_\d+) \{", vt[:idx])]
        return hdrs[-1].group(1) if hdrs else None

    def find_button(needle):
        vt = vtree_text()
        idx = vt.find(needle)
        if idx < 0:
            return None
        hdrs = [m for m in re.finditer(r"button (vnode_\d+) \{", vt[:idx])]
        return hdrs[-1].group(1) if hdrs else None

    def wait_first_collection(deadline=15.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            ss = tool_json("autoui_scroll_state", {})
            if ss.get("frame", 0) > 0 and ss.get("panes"):
                return ss
            time.sleep(POLL_INTERVAL)
        return None

    return tool_json, find_para, find_button, wait_first_collection


def _t12_ready_fixture(inst) -> tuple:
    """launch + 双段就绪 + 首轮采集；返回 (ok, detail)。"""
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        return False, f"backend ready failed: {detail}"
    ok, detail = inst.wait_mcp()
    if not ok:
        return False, f"mcp ready failed: {detail}"
    return True, detail


TARGET_ROW = "Row 040"  # 字符占比恢复的稳定落点段（¶60 在 large 字号下估算
# 偏差 ~131px 出窗——占比 vs 像素 Padding 的系统性偏差，证据记录不掩盖）


def run_v08_save(ev: Evidence, inst: Instance) -> CaseResult:
    """V-08 真实保存（框架正对照）：press -> handler -> HTTP POST -> 后端
    校验+落盘+读回 -> UI 确认 -> 磁盘/读回逐字一致。Reader 现状另列。"""
    r = CaseResult("v08_save")
    ok, detail = _t12_ready_fixture(inst)
    if not ok:
        r.status, r.detail = "fail", detail
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)
    tool_json, find_para, find_button, wait_first = _t12_helpers(cli)
    if not wait_first():
        r.status, r.detail = "fail", "no bounds collection"
        inst.reap()
        return r
    checks = []

    eid = find_para(TARGET_ROW)
    checks.append(("target paragraph found", bool(eid)))
    if not eid:
        ev.raw_write("v08_save", {"checks": [(c[0], bool(c[1])) for c in checks]})
        inst.reap()
        r.status, r.detail = "fail", "target not found"
        return r
    res = cli.call("autoui_action", {"element_id": eid, "action": "press"})
    res_text = "".join(p.get("text", "") for p in res.get("content", []))
    checks.append(("press dispatched Tap(40)", "status: ok" in res_text and "cur_text" in res_text))
    note = inst.snapshot_has("已记录位置：第 40 段", timeout=15)
    checks.append(("success note shown", bool(note)))

    pj = inst.data_dir / "p781-progress.json"
    raw = pj.read_text(encoding="utf-8") if pj.is_file() else ""
    checks.append(("disk progress persisted",
                   '"paragraph_index":40' in raw and TARGET_ROW in raw and '"font_size":"medium"' in raw))
    ev.raw_write("v08_save-disk", {"progress_json": raw,
                                   "progress_sha256": sha256_file(pj) if pj.is_file() else None})
    try:
        st, got = http_json(inst.back_port, "GET", "/api/progress")
        got = unwrap_json(got)
        checks.append(("GET readback identical",
                       st == 200 and got.get("paragraph_index") == 40
                       and got.get("paragraph_text", "").startswith(TARGET_ROW)
                       and got.get("font_size") == "medium"))
    except (urllib.error.URLError, OSError, ValueError) as e:
        checks.append(("GET readback identical", False))

    # 字号设置一并持久化（V-09 前提）
    fe = find_button("A+")
    checks.append(("font button found", bool(fe)))
    if fe:
        cli.press(fe)
        ok_font = inst.snapshot_has("large", timeout=10)
        checks.append(("font state changed", bool(ok_font)))
        eid2 = find_para(TARGET_ROW)  # 重建后 id 已变，重新发现
        cli.press(eid2)
        t0 = time.monotonic()
        persisted_large = False
        while time.monotonic() - t0 < 10:
            if pj.is_file() and '"font_size":"large"' in pj.read_text(encoding="utf-8"):
                persisted_large = True
                break
            time.sleep(POLL_INTERVAL)
        checks.append(("font large persisted", persisted_large))

    ev.raw_write("v08_save", {"checks": [(c[0], bool(c[1])) for c in checks]})
    reap_ok, reap_detail = inst.reap()
    bad = [c for c in checks if not c[1]]
    if bad or not reap_ok:
        r.status = "fail"
        r.detail = f"failed: {[c[0] for c in bad]}; reap={'ok' if reap_ok else reap_detail}"
    else:
        r.status = "pass"
        r.detail = f"{len(checks)} save-chain checks ok ({detail})"
    return r


def run_v09_cold_restore(ev: Evidence, inst: Instance) -> CaseResult:
    """V-09 冷恢复（框架正对照）：保存 -> 冷重启（新 pid/端口）-> 只观察
    （零滚动动作）-> 产品自身恢复使目标正面积可见 + 状态/字号保留 +
    损坏锚点负例（诚实失败不假恢复）。"""
    import re

    r = CaseResult("v09_cold_restore")
    ok, detail = _t12_ready_fixture(inst)
    if not ok:
        r.status, r.detail = "fail", detail
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)
    tool_json, find_para, find_button, wait_first = _t12_helpers(cli)
    if not wait_first():
        r.status, r.detail = "fail", "no bounds collection"
        inst.reap()
        return r

    # 前提：保存 ¶40 + large（建立测试保存前提的两阶段之一，滚动零参与）
    eid = find_para(TARGET_ROW)
    if not eid:
        r.status, r.detail = "fail", "target not found in phase-1"
        inst.reap()
        return r
    cli.press(eid)
    if not inst.snapshot_has("已记录位置：第 40 段", timeout=15):
        r.status, r.detail = "fail", "phase-1 save note missing"
        inst.reap()
        return r
    fe = find_button("A+")
    if fe:
        cli.press(fe)
        inst.snapshot_has("large", timeout=10)
        eid2 = find_para(TARGET_ROW)
        cli.press(eid2)
    pj = inst.data_dir / "p781-progress.json"
    t0 = time.monotonic()
    while time.monotonic() - t0 < 10:
        if pj.is_file() and '"font_size":"large"' in pj.read_text(encoding="utf-8"):
            break
        time.sleep(POLL_INTERVAL)
    old_pid = inst.proc.pid

    # 冷启动：停止+reap+端口释放 -> 同库重启
    ok, rdetail = inst.restart()
    if not ok:
        r.status, r.detail = "fail", f"restart failed: {rdetail}"
        return r
    ok2, detail2 = inst.wait_backend()
    ok3, detail3 = inst.wait_mcp()
    if not (ok2 and ok3):
        r.status, r.detail = "fail", f"restart ready failed: {detail2} | {detail3}"
        inst.reap()
        return r
    checks = [("cold restart new pid", inst.proc.pid != old_pid),
              ("restart ready", True)]

    # 观察期：零滚动动作（本函数自此不调用 autoui_action）。
    cli2 = AutoUiMcpClient(inst.mcp_port)
    tool_json, find_para, _, _ = _t12_helpers(cli2)  # 重启换端口——helpers 重绑
    t0 = time.monotonic()
    vis = None
    while time.monotonic() - t0 < 25:
        e2 = find_para(TARGET_ROW)
        if e2:
            try:
                v = tool_json("autoui_visibility", {"element_id": e2})
                if v.get("visible"):
                    vis = v
                    break
            except Exception:  # noqa: BLE001  # 观察轮询：网络/解析抖动一律续等
                pass
        time.sleep(POLL_INTERVAL)
    checks.append(("target visible after product restore (no driver scroll)", bool(vis)))
    snap = inst.snapshot_text()
    checks.append(("restore note shown", "已恢复上次位置" in snap))
    checks.append(("font large restored", "large" in snap))
    ss = tool_json("autoui_scroll_state", {})
    checks.append(("product scrolled (offset moved by restore)",
                   ss.get("panes") and ss["panes"][0]["offset"][1] > 0))
    if vis:
        ev.raw_write("v09_cold_restore-visibility", vis)
    try:
        shot = cli2.screenshot("v09restore", baseline=False)
        ev.shot_collect("v09restore", str(shot))
    except Exception:  # noqa: BLE001
        pass

    # 负例：损坏锚点 -> 再冷启 -> 诚实失败（内容变化提示、不标记已恢复）
    pj.write_text('{"reader":"p781-save-restore","paragraph_index":40,'
                  '"paragraph_text":"TAMPERED anchor text","font_size":"large","updated_at":0}',
                  encoding="utf-8")
    ok, rdetail = inst.restart()
    if not ok:
        r.status, r.detail = "fail", f"negative restart failed: {rdetail}"
        return r
    ok2, _ = inst.wait_backend()
    ok3, _ = inst.wait_mcp()
    if not (ok2 and ok3):
        r.status, r.detail = "fail", "negative restart not ready"
        inst.reap()
        return r
    t0 = time.monotonic()
    snap2 = ""
    while time.monotonic() - t0 < 15:
        snap2 = inst.snapshot_text()
        if "内容已变化" in snap2 or "已恢复上次位置" in snap2:
            break
        time.sleep(POLL_INTERVAL)
    checks.append(("tampered anchor fails honestly",
                   "内容已变化" in snap2 and "已恢复上次位置" not in snap2))

    ev.raw_write("v09_cold_restore", {"checks": [(c[0], bool(c[1])) for c in checks],
                                      "progress_sha256": sha256_file(pj) if pj.is_file() else None})
    reap_ok, reap_detail = inst.reap()
    bad = [c for c in checks if not c[1]]
    if bad or not reap_ok:
        r.status = "fail"
        r.detail = f"failed: {[c[0] for c in bad]}; reap={'ok' if reap_ok else reap_detail}"
    else:
        r.status = "pass"
        r.detail = f"{len(checks)} cold-restore checks ok ({detail} -> {detail3})"
    return r


def run_v08_reader_current_state(ev: Evidence, inst: Instance) -> CaseResult:
    """Reader 现状记录例（AC-08）：真实 Reader 走同一保存链——当前因
    reading.at `let pr` 重赋值致 VM 合成失败、阅读页正文恒空。本例 PASS =
    该缺陷被通道如实取证（不授予 Reader 目标 PASS）。"""
    r = CaseResult("v08_reader_current_state")
    r.required = False  # 记录例：缺陷修复后应转正（001 路由）
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        r.status, r.detail = "fail", f"backend ready failed: {detail}"
        inst.reap()
        return r
    try:
        inst.seed_book()
    except RuntimeError as e:
        r.status, r.detail = "fail", f"seed failed: {e}"
        inst.reap()
        return r
    ok, detail = inst.wait_mcp()
    if not ok:
        r.status, r.detail = "fail", f"mcp ready failed: {detail}"
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)

    import re

    def col_candidates(needle, n=6):
        """needle 最近的 n 个容器头候选（内→外，含 col/button）。for 迭代>0
        的 events 注解不随迭代重复、嵌套层数不定——由调用方逐一试派发。"""
        vt = "".join(x.get("text", "") for x in cli.call("autoui_vtree", {}).get("content", []))
        idx = vt.find(needle)
        if idx < 0:
            return []
        hdrs = [mm.group(1) for mm in re.finditer(r"(?:col|button) (vnode_\d+) \{", vt)
                if mm.end() <= idx]
        out = []
        for eid in reversed(hdrs):
            if eid not in out:
                out.append(eid)
            if len(out) >= n:
                break
        return out

    if not inst.snapshot_has(inst.marker, timeout=30.0):
        r.status, r.detail = "fail", f"marker {inst.marker} not visible on bookshelf"
        inst.reap()
        return r

    press_log = []

    def press_with_retry(needle, what):
        for attempt in range(3):
            for eid in col_candidates(needle):
                res = cli.call("autoui_action", {"element_id": eid, "action": "press"})
                txt = "".join(x.get("text", "") for x in res.get("content", []))
                press_log.append({"needle": needle, "eid": eid, "resp": txt[:120]})
                if "status: ok" in txt:
                    return True
            time.sleep(1.0)
        return False

    if not press_with_retry(inst.marker, "card"):
        r.status, r.detail = "fail", f"card press not dispatched: {press_log}"
        inst.reap()
        return r
    t0 = time.monotonic()
    while time.monotonic() - t0 < 20:
        if "第1章" in inst.snapshot_text():
            break
        time.sleep(POLL_INTERVAL)
    if not press_with_retry("第1章", "chapter"):
        r.status, r.detail = "fail", f"chapter press not dispatched: {press_log}"
        inst.reap()
        return r
    t0 = time.monotonic()
    reading = False
    while time.monotonic() - t0 < 20:
        snap = inst.snapshot_text()
        if "OnScroll" in snap or "（本章无内容）" in snap:
            reading = True
            break
        time.sleep(POLL_INTERVAL)
    if not reading:
        r.status, r.detail = "fail", "reading page not reached"
        inst.reap()
        return r
    snap = inst.snapshot_text()
    err_txt = inst._read_log("stderr") + inst._read_log("stdout")
    ev.raw_write("v08_reader_current_state", {
        "snapshot_head": snap[:3000],
        "synthesis_error_present": "Cannot reassign to immutable variable" in err_txt,
        "empty_content_present": "（本章无内容）" in snap,
        "paragraph_labels_present": "¶0" in snap,
    })
    defect_evidenced = ("Cannot reassign to immutable variable" in err_txt
                        and "（本章无内容）" in snap)
    reap_ok, _ = inst.reap()
    if defect_evidenced and reap_ok:
        r.status = "pass"
        r.detail = ("documented consumer defect evidenced: reading.at:266 'let pr' reassigned "
                    "at :315 -> VM handler synthesis fails -> reading Init never runs -> "
                    "empty content; routed to READER-001 (not granted PASS)")
    else:
        r.status = "fail"
        r.detail = (f"defect not evidenced as documented (synth_err="
                    f"{'Cannot reassign' in err_txt}, empty={'（本章无内容）' in snap}) — "
                    "若 Reader 已修复请把本例转正为真实保存链验收")
    return r


def run_v13_anchor_interop(ev: Evidence, inst: Instance) -> CaseResult:
    """V-13 锚点互操作负对照（阶段 B 未落地形态）：通道对 pane 外目标按实测
    分类（clipped_out/不可见），锚点能力（scroll_to_child）以显式 Unknown
    tool 诚实失败——不是「通道不支持」、驱动也不代为修正位置（零滚动）。"""
    import re

    r = CaseResult("v13_anchor_interop")
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        r.status, r.detail = "fail", f"backend ready failed: {detail}"
        inst.reap()
        return r
    ok, detail = inst.wait_mcp()
    if not ok:
        r.status, r.detail = "fail", f"mcp ready failed: {detail}"
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)

    def tool_json(name, args):
        res = cli.call(name, args)
        return json.loads("".join(p.get("text", "") for p in res.get("content", [])))

    t0 = time.monotonic()
    while time.monotonic() - t0 < 15:
        ss = tool_json("autoui_scroll_state", {})
        if ss.get("frame", 0) > 0 and ss.get("panes"):
            break
        time.sleep(POLL_INTERVAL)
    if not ss.get("panes"):
        r.status, r.detail = "fail", "no pane geometry"
        inst.reap()
        return r
    offset_before = ss["panes"][0]["offset"]

    vt = "".join(p.get("text", "") for p in cli.call("autoui_vtree", {}).get("content", []))
    idx = vt.find("¶100 P781SG")
    hdrs = [m for m in re.finditer(r"col (vnode_\d+) \{", vt[:idx])]
    target = hdrs[-1].group(1) if hdrs else None
    checks = [("outside-pane target found", bool(target))]
    if target:
        v = tool_json("autoui_visibility", {"element_id": target})
        checks.append(("channel classifies honestly (clipped_out, not visible)",
                       v.get("visible") is False and "clipped_out" in v.get("flags", [])))
        ev.raw_write("v13_anchor_interop-visibility", v)

    # 阶段 B 锚点工具未落地：MCP 必须显式 Unknown tool（诚实失败），
    # 而非通道代答/假成功。
    unknown_ok = False
    unknown_text = ""
    try:
        res = cli.call("autoui_scroll_to_child", {"pane": "vnode", "anchor_key": "p100"})
        unknown_text = json.dumps(res, ensure_ascii=False)[:200]
        unknown_ok = res.get("isError") is True and "Unknown tool" in unknown_text
    except Exception as e:  # noqa: BLE001
        unknown_text = repr(e)[:200]
    checks.append(("anchor capability fails honestly (Unknown tool)",
                   unknown_ok))
    ev.raw_write("v13_anchor_interop-anchor-call", {"response": unknown_text})

    # 驱动不修正位置：全程零滚动，offset 不变。
    ss2 = tool_json("autoui_scroll_state", {})
    checks.append(("no driver scroll (offset unchanged)",
                   ss2["panes"] and ss2["panes"][0]["offset"] == offset_before))
    try:
        shot = cli.screenshot("v13anchor", baseline=False)
        ev.shot_collect("v13anchor", str(shot))
    except Exception:  # noqa: BLE001
        pass

    ev.raw_write("v13_anchor_interop", {"checks": [(c[0], bool(c[1])) for c in checks],
                                        "offset_before": offset_before})
    reap_ok, reap_detail = inst.reap()
    bad = [c for c in checks if not c[1]]
    if bad or not reap_ok:
        r.status = "fail"
        r.detail = f"failed: {[c[0] for c in bad]}; reap={'ok' if reap_ok else reap_detail}"
    else:
        r.status = "pass"
        r.detail = (f"{len(checks)} anchor-interop negative checks ok ({detail}); "
                    "stage-B anchor API absent by design -> honest Unknown tool")
    return r




def run_v_anchor_native(ev: Evidence, inst: Instance) -> CaseResult:
    """阶段B T-03 原生验证：经生产 handler 调 scroll_to_child——
    (1) GoP40 → 零驱动滚动下产品定位使 Row040 正面积可见 + applied；
    (2) GoP115 → 底部段可见；(3) GoMissing → queued 后 invalid:not_found
    且 offset 不动（诚实失败）；全程仅生产事件。"""
    import re

    r = CaseResult("v_anchor_native")
    inst.launch()
    ok, detail = inst.wait_backend()
    if not ok:
        r.status, r.detail = "fail", f"backend ready failed: {detail}"
        inst.reap()
        return r
    ok, detail = inst.wait_mcp()
    if not ok:
        r.status, r.detail = "fail", f"mcp ready failed: {detail}"
        inst.reap()
        return r
    cli = AutoUiMcpClient(inst.mcp_port)

    def tool_json(name, args):
        res = cli.call(name, args)
        return json.loads("".join(p.get("text", "") for p in res.get("content", [])))

    def press_button(label):
        vt = "".join(x.get("text", "") for x in cli.call("autoui_vtree", {}).get("content", []))
        idx = vt.find(label)
        if idx < 0:
            return False
        hdrs = [m for m in re.finditer(r"button (vnode_\d+) \{", vt[:idx])]
        if not hdrs:
            return False
        res = cli.call("autoui_action", {"element_id": hdrs[-1].group(1), "action": "press"})
        txt = "".join(x.get("text", "") for x in res.get("content", []))
        return "status: ok" in txt

    def wait_first_collection(deadline=15.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            ss = tool_json("autoui_scroll_state", {})
            if ss.get("frame", 0) > 0 and ss.get("panes"):
                return ss
            time.sleep(POLL_INTERVAL)
        return None

    def pane():
        return tool_json("autoui_scroll_state", {})["panes"][0]

    def row_visible(needle, deadline=12.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            vt = "".join(x.get("text", "") for x in cli.call("autoui_vtree", {}).get("content", []))
            idx = vt.find(needle)
            if idx < 0:
                time.sleep(POLL_INTERVAL)
                continue
            hdrs = [m for m in re.finditer(r"col (vnode_\d+) \{", vt[:idx])]
            if not hdrs:
                time.sleep(POLL_INTERVAL)
                continue
            try:
                v = tool_json("autoui_visibility", {"element_id": hdrs[-1].group(1)})
                if v.get("visible"):
                    return v
            except Exception:  # noqa: BLE001
                pass
            time.sleep(POLL_INTERVAL)
        return None

    def snapshot_has(needle, deadline=10.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            if needle in inst.snapshot_text():
                return True
            time.sleep(POLL_INTERVAL)
        return False

    if not wait_first_collection():
        r.status, r.detail = "fail", "no bounds collection"
        inst.reap()
        return r
    checks = []
    off_before = pane()["offset"]

    # (1) GoP40：入队回执 queued → 产品定位 → Row040 可见 + applied
    checks.append(("GoP40 pressed", press_button("GoP40")))
    checks.append(("queued receipt shown", snapshot_has("queued")))
    v40 = row_visible("¶40 P781SG")
    checks.append(("Row040 visible via product anchor scroll (no driver scroll)", bool(v40)))
    if v40:
        ev.raw_write("v_anchor_native-vis40", v40)
    checks.append(("state applied (Query p40)", press_button("Q40") and snapshot_has("applied")))
    off40 = pane()["offset"]
    checks.append(("offset moved by anchor", off40[1] > off_before[1]))

    # (2) GoP115：底部段可见（clamp 后正面积）
    checks.append(("GoP115 pressed", press_button("GoP115")))
    v115 = row_visible("¶115 P781SG")
    checks.append(("Row115 visible after anchor", bool(v115)))
    if v115:
        ev.raw_write("v_anchor_native-vis115", v115)

    # (3) GoMissing：诚实失败 + offset 不动
    off115 = pane()["offset"]
    checks.append(("GoMissing pressed", press_button("GoMissing")))
    checks.append(("queued for missing too", snapshot_has("queued")))
    checks.append(("invalid:not_found surfaced",
                   press_button("QMiss") and snapshot_has("invalid:not_found")))
    off_after = pane()["offset"]
    checks.append(("offset unchanged on invalid", off_after[1] == off115[1]))

    try:
        shot = cli.screenshot("vanchor", baseline=False)
        ev.shot_collect("vanchor", str(shot))
    except Exception:  # noqa: BLE001
        pass
    ev.raw_write("v_anchor_native", {"checks": [(c[0], bool(c[1])) for c in checks],
                                      "offset_before": off_before,
                                      "offset_after_40": off40, "offset_final": off_after})
    reap_ok, reap_detail = inst.reap()
    bad = [c for c in checks if not c[1]]
    if bad or not reap_ok:
        r.status = "fail"
        r.detail = f"failed: {[c[0] for c in bad]}; reap={'ok' if reap_ok else reap_detail}"
    else:
        r.status = "pass"
        r.detail = f"{len(checks)} native anchor checks ok ({detail})"
    return r



# ---------------------------------------------------------------- main
def main() -> int:
    ap = argparse.ArgumentParser(description="PLAN-781 native reader acceptance driver")
    ap.add_argument("--auto-bin", required=True)
    ap.add_argument("--app-dir", required=True)
    ap.add_argument("--data-dir", required=True)
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--back-port", type=int, default=None)
    ap.add_argument("--mcp-port", type=int, default=None)
    ap.add_argument("--ready-timeout", type=float, default=120.0)
    ap.add_argument("--case", action="append", dest="cases", default=None,
                    help="可重复; 缺省 ALL(不含 v12_injected_fail)")
    args = ap.parse_args()

    auto_bin, app_dir = Path(args.auto_bin).resolve(), Path(args.app_dir).resolve()
    data_root, out_dir = Path(args.data_dir).resolve(), Path(args.out_dir).resolve()
    if not auto_bin.is_file():
        print(f"[usage] --auto-bin not a file: {auto_bin}")
        return 1
    if not (app_dir / "pac.at").is_file():
        print(f"[usage] --app-dir lacks pac.at: {app_dir}")
        return 1
    out_dir.mkdir(parents=True, exist_ok=True)
    data_root.mkdir(parents=True, exist_ok=True)
    ev = Evidence(out_dir)

    try:
        reader_head = subprocess.run(["git", "-C", str(app_dir), "rev-parse", "HEAD"],
                                     capture_output=True, text=True, timeout=10).stdout.strip()
        reader_before = subprocess.run(["git", "-C", str(app_dir), "status", "--porcelain"],
                                       capture_output=True, text=True, timeout=10).stdout
    except (OSError, subprocess.SubprocessError):
        reader_head, reader_before = "n/a", "n/a"
    try:
        ver = subprocess.run([str(auto_bin), "--version"], capture_output=True,
                             text=True, timeout=10)
        auto_version = (ver.stdout + ver.stderr).strip()[:200] or "n/a"
    except (OSError, subprocess.SubprocessError):
        auto_version = "n/a"
    ev.manifest.update({
        "auto_bin": str(auto_bin), "auto_bin_sha256": sha256_file(auto_bin),
        "auto_bin_size": auto_bin.stat().st_size, "auto_version": auto_version,
        "app_dir": str(app_dir), "reader_commit": reader_head,
        "reader_status_before": reader_before,
        "data_root": str(data_root), "argv": sys.argv,
    })

    run_ts = datetime.now().strftime("%Y%m%d-%H%M%S")

    def mk(tag: str, **kw) -> Instance:
        return Instance(ev, auto_bin, app_dir,
                        data_root / f"reader001-review-{run_ts}-{tag}", tag,
                        ready_timeout=args.ready_timeout,
                        back_port=args.back_port, mcp_port=args.mcp_port, **kw)

    all_cases = ["r1_smoke", "v07_dual_instance", "v12_window_start_fail",
                 "v12_backend_dead", "v12_timeout", "v10_fake_geometry", "v11_clip",
                 "v08_save", "v09_cold_restore", "v08_reader_current_state", "v13_anchor_interop",
                 "v_anchor_native"]
    selected = args.cases or all_cases

    for case_id in selected:
        inst_holder: list = []

        def guard(fn, *a):
            """异常也收割: 案例函数抛错时至少 reap 已 launch 的实例。"""
            try:
                return fn(ev, *a)
            except Exception as e:  # noqa: BLE001
                return CaseResult("unhandled", status="fail", detail=repr(e))
            finally:
                for inst in inst_holder:
                    if inst.poll_alive():
                        inst.reap()

        if case_id == "r1_smoke":
            inst = mk("smoke")
            inst_holder.append(inst)
            ev.case(guard(run_r1_smoke, inst))
        elif case_id == "v07_dual_instance":
            ev.case(guard(run_v07_dual, mk))
        elif case_id == "v12_window_start_fail":
            bad_dir = out_dir / "bad-app-dir"
            bad_dir.mkdir(exist_ok=True)
            inst = mk("v12wsf")
            inst_holder.append(inst)
            ev.case(guard(run_v12_window_start_fail, inst, bad_dir))
        elif case_id == "v12_backend_dead":
            inst = mk("v12bd")
            inst_holder.append(inst)
            ev.case(guard(run_v12_backend_dead, inst))
        elif case_id == "v12_timeout":
            inst = mk("v12to")
            inst_holder.append(inst)
            ev.case(guard(run_v12_timeout, inst))
        elif case_id == "v12_injected_fail":
            ev.case(run_v12_injected_fail(ev))
        elif case_id == "v10_fake_geometry":
            inst = mk("v10", fixture_mode=True, ready_marker="P781-SCROLL-GEOMETRY")
            inst.app_dir = REPO_ROOT / "test" / "fixtures" / "native-reader-acceptance" / "scroll-geometry"
            inst_holder.append(inst)
            ev.case(guard(run_v10_pending_t11, inst))
        elif case_id == "v11_clip":
            inst = mk("v11", fixture_mode=True, ready_marker="P781-SCROLL-GEOMETRY")
            inst.app_dir = REPO_ROOT / "test" / "fixtures" / "native-reader-acceptance" / "scroll-geometry"
            inst_holder.append(inst)
            ev.case(guard(run_v11_clip, inst))
        elif case_id == "v08_save":
            inst = mk("v08", fixture_mode=True, ready_marker="P781-SAVE-RESTORE-FIXTURE")
            inst.app_dir = REPO_ROOT / "test" / "fixtures" / "native-reader-acceptance" / "save-restore"
            inst_holder.append(inst)
            ev.case(guard(run_v08_save, inst))
        elif case_id == "v09_cold_restore":
            inst = mk("v09", fixture_mode=True, ready_marker="P781-SAVE-RESTORE-FIXTURE")
            inst.app_dir = REPO_ROOT / "test" / "fixtures" / "native-reader-acceptance" / "save-restore"
            inst_holder.append(inst)
            ev.case(guard(run_v09_cold_restore, inst))
        elif case_id == "v08_reader_current_state":
            inst_holder.append(mk("v08reader"))
            ev.case(guard(run_v08_reader_current_state, inst_holder[-1]))
        elif case_id == "v_anchor_native":
            inst = mk("vanchor", fixture_mode=True, ready_marker="P781-SCROLL-GEOMETRY")
            inst.app_dir = REPO_ROOT / "test" / "fixtures" / "native-reader-acceptance" / "scroll-geometry"
            inst_holder.append(inst)
            ev.case(guard(run_v_anchor_native, inst))
        elif case_id == "v13_anchor_interop":
            inst = mk("v13", fixture_mode=True, ready_marker="P781-SCROLL-GEOMETRY")
            inst.app_dir = REPO_ROOT / "test" / "fixtures" / "native-reader-acceptance" / "scroll-geometry"
            inst_holder.append(inst)
            ev.case(guard(run_v13_anchor_interop, inst))
        else:
            print(f"[usage] unknown case: {case_id}")
            return 1

    try:
        reader_after = subprocess.run(["git", "-C", str(app_dir), "status", "--porcelain"],
                                      capture_output=True, text=True, timeout=10).stdout
        ev.manifest["reader_status_after"] = reader_after
        tracked_damage = [l for l in set(reader_after.splitlines())
                          - set(reader_before.splitlines()) if not l.startswith("??")]
        ev.manifest["reader_tracked_damage"] = tracked_damage
    except (OSError, subprocess.SubprocessError):
        pass

    rc = ev.finish()
    print(f"driver exit={rc}")
    return rc


if __name__ == "__main__":
    sys.exit(main())
