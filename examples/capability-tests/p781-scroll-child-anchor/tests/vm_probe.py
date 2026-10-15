"""
p781-scroll-child-anchor VM(MCP) 验证驱动 —— PLAN-781 阶段B T-05 原生腿。

V-01~06 全矩阵（合同 = docs/plans/reports/781-interface-decision.md）：
  V-01 真实定位：Init 零驱动滚动定位（V-03 联动）+ 双 4000 字长段（中/英）
      顶对齐起始区域可见（intersection 顶 ≈ pane viewport 顶）。
  V-02 寻址：非连续 key（p2/p5/p8/p12/...，空行过滤语义）键→行身份。
  V-03 时序：Init 先发目标布局后可见；双章换绑（if 双 pane 分支共享句柄）
      ——旧 key 新请求 invalid:not_found、新 key applied。
  V-04 负例：空 key / 缺 key / 越界等价 / 幽灵句柄 not_bound / 同 pane 重复
      （后写覆盖=文档序最后）/ 跨 pane 同 key（主 pane 不动）。
  V-05 重排：字号 large / 窄宽度组合下新请求按新布局定位；已可见不跳。
  V-06 兼容：旧 offset 族回归；applied 后用户滚动不被旧请求拉回；
      managed pane → error:unsupported。

纪律：仅生产事件（button press 经 MCP action）；恢复/定位零驱动滚动；
几何 = autoui_visibility（真实布局帧）；offset = autoui_scroll_state
（按 controller 过滤）。证据（几何 JSON/截图/CLI hash）→ P781_T05_OUT。

用法：python tests/vm_probe.py [--archive]
AUTO_BIN 覆盖 auto 可执行档（默认取 worktree target/debug/auto.exe）。
"""
import hashlib
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.request

POLL = 0.4
OUT = os.environ.get("P781_T05_OUT",
                     os.path.join(os.path.dirname(os.path.abspath(__file__)), "evidence-vm"))


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def default_auto_bin() -> str:
    env = os.environ.get("AUTO_BIN")
    if env:
        return env
    for cand in (r"D:\autostack\.wt\lang-781\auto-lang\target\debug\auto.exe", "auto"):
        if os.sep not in cand or os.path.exists(cand):
            return cand
    return "auto"


class Client:
    def __init__(self, port):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, name, args=None):
        req = {"jsonrpc": "2.0", "id": self._id, "method": "tools/call",
               "params": {"name": name, "arguments": args or {}}}
        self._id += 1
        r = urllib.request.Request(self.url, data=json.dumps(req).encode(),
                                   headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(r, timeout=15) as resp:
            res = json.loads(resp.read().decode())
            if "error" in res:
                raise RuntimeError(f"{name}: {res['error']}")
            return res.get("result", {})

    def text(self, name, args=None):
        return "".join(p.get("text", "") for p in self.call(name, args).get("content", []))

    def tool_json(self, name, args=None):
        return json.loads(self.text(name, args))


class Driver:
    def __init__(self, cli: Client):
        self.cli = cli
        self.checks = []          # (name, ok, detail)
        self.raw = {}             # evidence blobs

    def check(self, name, ok, detail=""):
        self.checks.append((name, bool(ok), str(detail)))
        print(("[+] " if ok else "[-] ") + f"{name}: {detail}")

    # -- lookups ----------------------------------------------------------
    def vtree(self):
        return self.cli.text("autoui_vtree", {})

    def snapshot(self):
        return self.cli.text("autoui_snapshot", {})

    def _id_of(self, line):
        for tok in line.replace("#", " ").split():
            if tok.startswith("aura_") or tok.startswith("vnode_"):
                return tok
        return None

    def find_button(self, label):
        vt = self.vtree()
        idx = vt.find(f'"{label}"')
        if idx < 0:
            return None
        hdrs = [m for m in re.finditer(r"button (vnode_\d+|aura_\d+) \{", vt[:idx])]
        return hdrs[-1].group(1) if hdrs else None

    def find_col_with(self, needle):
        """needle 文本所在行（col 包装）的节点 id。"""
        vt = self.vtree()
        idx = vt.find(needle)
        if idx < 0:
            return None
        hdrs = [m for m in re.finditer(r"col (vnode_\d+|aura_\d+) \{", vt[:idx])]
        return hdrs[-1].group(1) if hdrs else None

    def find_pane_node(self, content_needle):
        """包含 content_needle 的 scroll-pane（vtree scrollable 行）节点 id。"""
        vt = self.vtree()
        idx = vt.find(content_needle)
        if idx < 0:
            return None
        hdrs = [m for m in re.finditer(r"(?:scrollable|scroll-pane) (vnode_\d+|aura_\d+) \{", vt[:idx])]
        return hdrs[-1].group(1) if hdrs else None

    # -- actions / reads --------------------------------------------------
    def press(self, label, deadline=8.0):
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            bid = self.find_button(label)
            if bid:
                res = self.cli.call("autoui_action", {"element_id": bid, "action": "press"})
                txt = "".join(p.get("text", "") for p in res.get("content", []))
                if "status: ok" in txt:
                    return True
            time.sleep(POLL)
        return False

    def reply(self, deadline=8.0):
        """读 R=<receipt> 显示（快照文本）。"""
        t0 = time.monotonic()
        while time.monotonic() - t0 < deadline:
            m = re.search(r"R=(\S+)", self.snapshot())
            if m:
                return m.group(1)
            time.sleep(POLL)
        return None

    def press_reply(self, label, want, deadline=8.0):
        ok = self.press(label)
        got = self.reply() if ok else None
        return got == want, f"pressed={ok} got={got} want={want}"

    def expect_reply(self, label, want, deadline=12.0):
        """press 一次 + 轮询回执直至 want——状态查询经 drain/prime 节拍
        （pending → 终态）有真实时间窗，一次性读会撞 pending。"""
        ok = self.press(label)
        t0 = time.monotonic()
        got = None
        while time.monotonic() - t0 < deadline:
            got = self.reply()
            if got == want:
                return True, f"pressed={ok} got={got}"
            time.sleep(POLL)
        return False, f"pressed={ok} got={got} want={want}"

    def pane_state(self, controller):
        # F-781-D01 勘定：工具参数名是 "pane"（schema 单源）；早期误发
        # "filter" 被静默忽略 → 无过滤全集按字典序排序，@scrollctl:1 句柄
        # （'@' < 字母）排首位，panes[0] 读到 std pane 的恒 0 offset——
        # 「iced scroll 全局失效」实为读错 pane 的观测假象。
        return self.cli.tool_json("autoui_scroll_state", {"pane": controller})

    def main_offset_y(self):
        st = self.pane_state("main-pane")
        # 按 key 精确寻址（不依赖排序；防御任何过滤面退化）。
        panes = [p for p in st.get("panes", [])
                 if p.get("key") == "scroll_ctl_main-pane"]
        return panes[0]["offset"][1] if panes else None

    def visible(self, row_needle, deadline=12.0, pane_pin=None):
        """轮询：needle 行可见（真实布局帧 visibility）。返回 payload 或 None。"""
        t0 = time.monotonic()
        last = None
        while time.monotonic() - t0 < deadline:
            cid = self.find_col_with(row_needle)
            if cid:
                args = {"element_id": cid}
                if pane_pin:
                    args["pane"] = pane_pin
                try:
                    v = self.cli.tool_json("autoui_visibility", args)
                    last = v
                    if v.get("visible"):
                        return v
                except Exception as e:  # noqa: BLE001
                    last = {"err": str(e)}
            time.sleep(POLL)
        return last


    def trace(self, seconds, label):
        """采样 (t, offset, frame, tick) —— 失败窗口归因用。"""
        out = []
        t0 = time.monotonic()
        while time.monotonic() - t0 < seconds:
            try:
                st = self.pane_state("main-pane")
                p = st.get("panes", [{}])[0]
                out.append({"t": round(time.monotonic() - t0, 2),
                            "off": p.get("offset"), "frame": st.get("frame"),
                            "tick": st.get("scroll_tick")})
            except Exception as e:  # noqa: BLE001
                out.append({"t": round(time.monotonic() - t0, 2), "err": str(e)[:80]})
            time.sleep(0.3)
        self.raw[f"trace_{label}"] = out
        return out

    def shot(self, name):
        try:
            res = self.cli.text("autoui_screenshot", {"name": name})
            # 路径可能带 \\?\ 前缀或包裹文本——容忍形态后剥离 verbatim 前缀。
            m = re.search(r"([A-Za-z]:\\[^\"']+?autoui-screenshot-\d+\.png)", res)
            return m.group(1).replace("\\\\?\\", "") if m else None
        except Exception:  # noqa: BLE001
            return None


def wait_ready(cli, deadline=40.0):
    t0 = time.monotonic()
    while time.monotonic() - t0 < deadline:
        try:
            if "tree:" in cli.text("autoui_snapshot", {}):
                return True
        except Exception:  # noqa: BLE001
            pass
        time.sleep(0.5)
    return False


def wait_first_collection(cli, deadline=20.0):
    t0 = time.monotonic()
    while time.monotonic() - t0 < deadline:
        try:
            ss = cli.tool_json("autoui_scroll_state", {})
            if ss.get("frame", 0) > 0 and ss.get("panes"):
                return ss
        except Exception:  # noqa: BLE001
            pass
        time.sleep(POLL)
    return None


def main():
    app_dir = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
    auto_bin = default_auto_bin()
    port = pick_free_port()
    os.makedirs(OUT, exist_ok=True)
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] {auto_bin} run -r vm @ {app_dir} (mcp:{port})")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)

    cli = Client(port)
    try:
        if not wait_ready(cli):
            print("[-] VM MCP not ready")
            return 2
        if not wait_first_collection(cli):
            print("[-] no bounds collection")
            return 2
        d = Driver(cli)

        # ── V-03/V-01：Init 先发目标 —— 零驱动滚动下 c3 可见 + applied ──
        v = d.visible("V03-c3", pane_pin="main-pane")
        d.check("v03_init_target_visible", v and v.get("visible"),
                f"intersection={v.get('intersection') if v else None}")
        d.raw["v03_init_vis_c3"] = v
        ok, det = d.expect_reply("QC3", "applied", deadline=15.0)
        d.check("v03_init_state_applied", ok, det)

        # ── V-01：双长段。中文 c50（顶对齐：高于视口 → 起始区域可见）──
        off0 = d.main_offset_y()
        ok, det = d.press_reply("GoC50", "queued")
        d.check("v01_c50_queued", ok, det)
        v50 = d.visible("V01-C-LONG-START", pane_pin="main-pane", deadline=15.0)
        d.check("v01_c50_visible", v50 and v50.get("visible"),
                f"intersection={v50.get('intersection') if v50 else None}")
        if v50 and v50.get("intersection") and v50.get("pane_chain"):
            vp = v50["pane_chain"][-1]["viewport_rect"]
            inter = v50["intersection"]
            tall = v50["target_rect"][3] > vp[3]
            top_aligned = abs(inter[1] - vp[1]) <= 2.0
            d.check("v01_c50_tall_top_aligned", tall and top_aligned,
                    f"target_h={v50['target_rect'][3]:.0f} vp_h={vp[3]:.0f} "
                    f"inter_y={inter[1]:.1f} vp_y={vp[1]:.1f}")
        d.raw["v01_vis_c50"] = v50

        # 英文 a80
        ok, det = d.press_reply("GoA80", "queued")
        d.check("v01_a80_queued", ok, det)
        va = d.visible("V01-A-LONG-START", pane_pin="main-pane", deadline=15.0)
        d.check("v01_a80_visible", va and va.get("visible"),
                f"intersection={va.get('intersection') if va else None}")
        d.raw["v01_vis_a80"] = va

        # ── V-02：非连续 key 寻址（p8 行身份）──
        ok, det = d.press_reply("GoP8", "queued")
        d.check("v02_p8_queued", ok, det)
        vp8 = d.visible("V02-p8", pane_pin="main-pane")
        d.check("v02_p8_row_visible", vp8 and vp8.get("visible"),
                f"intersection={vp8.get('intersection') if vp8 else None}")
        d.raw["v02_vis_p8"] = vp8

        # ── V-04：负例族 ──
        off_pre = d.main_offset_y()
        d.trace(1.0, "v04_pre")
        ok, det = d.press_reply("GoMiss", "queued")
        d.check("v04_missing_queued", ok, det)
        ok, det = d.expect_reply("QMiss", "invalid:not_found")
        d.check("v04_missing_state", ok, det)
        ok, det = d.press_reply("GoOOB", "queued")
        d.check("v04_oob_queued", ok, det)
        off_now = d.main_offset_y()
        if off_now != off_pre:
            d.trace(3.0, "v04_post")
        d.check("v04_invalid_no_scroll", off_pre is not None and off_now == off_pre,
                f"off {off_pre} -> {off_now}")
        ok, det = d.press_reply("GoEmptyK", "error:empty_key")
        d.check("v04_empty_key", ok, det)
        ok, det = d.press_reply("GoBadH", "error:bad_handle")
        d.check("v04_bad_handle", ok, det)
        ok, det = d.press_reply("GoGhost", "queued")
        d.check("v04_ghost_queued", ok, det)
        ok, det = d.expect_reply("QGhost", "invalid:not_bound")
        d.check("v04_ghost_not_bound", ok, det)
        # 同 pane 重复：后写覆盖 → 文档序最后（dup-B）可见
        off_pre = d.main_offset_y()
        ok, det = d.press_reply("GoDup", "queued")
        d.check("v04_dup_queued", ok, det)
        vdup = d.visible("dup-B-late", pane_pin="main-pane")
        d.check("v04_dup_last_wins", vdup and vdup.get("visible"),
                f"intersection={vdup.get('intersection') if vdup else None}")
        d.raw["v04_vis_dupB"] = vdup
        # 跨 pane 同 key：GoSideDup 滚 side pane 的 dup；main 不动
        off_main_pre = d.main_offset_y()
        ok, det = d.press_reply("GoSideDup", "queued")
        d.check("v04_side_dup_queued", ok, det)
        vsd = d.visible("side-dup-row", pane_pin="side-pane")
        d.check("v04_side_dup_visible", vsd and vsd.get("visible"),
                f"intersection={vsd.get('intersection') if vsd else None}")
        off_main_now = d.main_offset_y()
        d.check("v04_no_cross_pane_scroll", off_main_pre == off_main_now,
                f"main off {off_main_pre} -> {off_main_now}")
        ok, det = d.press_reply("GoSideOnly", "queued")
        d.check("v04_side_only_queued", ok, det)
        vso = d.visible("side-only-row", pane_pin="side-pane")
        d.check("v04_side_only_visible", vso and vso.get("visible"),
                f"intersection={vso.get('intersection') if vso else None}")
        # QUnknown（从未请求的 (handle,key)）
        ok, det = d.press_reply("QUnknown", "unknown")
        d.check("v04_unknown_state", ok, det)

        # ── 阶段D W-01（F-781-R01）：标准 controller 句柄（Init 经
        # scroll_controller() 构造，native "@scrollctl:N"）+ 含冒号 key——
        # 旧实现合成 id 首冒号错拆 → not_found；修复后必须真实定位 applied。
        ok, det = d.press_reply("GoStd", "queued")
        d.check("w01_std_handle_queued", ok, det)
        vstd = d.visible("STD-colon-row")
        d.check("w01_std_colon_key_located", vstd and vstd.get("visible"),
                f"intersection={vstd.get('intersection') if vstd else None}")
        d.raw["w01_std_vis"] = vstd
        ok, det = d.expect_reply("QStd", "applied", deadline=15.0)
        d.check("w01_std_state_applied", ok, det)

        # ── V-03：双章换绑 ──
        ok = d.press("SwitchCh")
        time.sleep(1.2)
        d.check("v03_switch_pressed", ok, "chapter 2 pane bound")
        ok, det = d.press_reply("GoC50", "queued")
        d.check("v03_old_key_new_request_queued", ok, det)
        ok, det = d.expect_reply("QC50", "invalid:not_found", deadline=15.0)
        d.check("v03_old_key_not_found_on_ch2", ok, det)
        off_ch2_pre = d.main_offset_y()
        ok, det = d.press_reply("GoQ5", "queued")
        d.check("v03_new_key_queued", ok, det)
        ok, det = d.expect_reply("QQ5", "applied", deadline=15.0)
        d.check("v03_new_key_state_applied", ok, det)
        vq5 = d.visible("ch2-q5", pane_pin="main-pane", deadline=15.0)
        d.check("v03_new_key_visible", vq5 and vq5.get("visible"),
                f"intersection={vq5.get('intersection') if vq5 else None}")
        d.raw["v03_vis_q5"] = vq5

        # ── V-05：重排（换回章1 → 字号 large → 窄宽）──
        d.press("SwitchCh")
        time.sleep(1.2)
        d.press("FontL")
        time.sleep(0.8)
        ok, det = d.press_reply("GoA80", "queued")
        d.check("v05_large_a80_queued", ok, det)
        va2 = d.visible("V01-A-LONG-START", pane_pin="main-pane", deadline=15.0)
        d.check("v05_large_a80_visible", va2 and va2.get("visible"),
                f"intersection={va2.get('intersection') if va2 else None}")
        d.raw["v05_vis_a80_large"] = va2
        # 已可见不跳：GoC3 后再次 GoC3 → offset 不变
        d.press_reply("GoC3", "queued")
        v_c3 = d.visible("V03-c3", pane_pin="main-pane", deadline=12.0)
        off_a = d.main_offset_y()
        d.raw["v05_vis_c3"] = v_c3
        if v_c3 and v_c3.get("visible"):
            d.trace(1.5, "v05_c3_first")
        d.press_reply("GoC3", "queued")
        time.sleep(0.8)
        off_b = d.main_offset_y()
        if off_a != off_b:
            d.trace(2.5, "v05_c3_second")
        d.check("v05_already_visible_stable", v_c3 and v_c3.get("visible")
                and off_a is not None and off_a == off_b,
                f"off {off_a} -> {off_b}")
        d.press("SetNarrow")
        time.sleep(0.8)
        ok, det = d.press_reply("GoC50", "queued")
        d.check("v05_narrow_c50_queued", ok, det)
        v50n = d.visible("V01-C-LONG-START", pane_pin="main-pane", deadline=15.0)
        d.check("v05_narrow_c50_visible", v50n and v50n.get("visible"),
                f"intersection={v50n.get('intersection') if v50n else None}")
        d.raw["v05_vis_c50_narrow"] = v50n

        # ── V-06：旧 offset 族回归 + 不回拉 + managed ──
        d.press("OldStart")
        time.sleep(2.6)
        off_start = d.main_offset_y()
        d.check("v06_old_start_zero", off_start == 0, f"off={off_start}")
        d.press("OldBy")
        time.sleep(2.6)
        off_by = d.main_offset_y()
        d.check("v06_old_by_moves", off_by is not None and abs(off_by - 320.0) < 30,
                f"off={off_by} (want ~320)")
        d.press("OldEnd")
        time.sleep(2.6)
        off_end = d.main_offset_y()
        d.check("v06_old_end_large", off_end is not None and off_end > off_by,
                f"off={off_end}")
        # applied 后用户滚动（MCP scroll = 生产动作面）不被旧请求拉回
        d.press_reply("GoC50", "queued")
        v_anchor = d.visible("V01-C-LONG-START", pane_pin="main-pane", deadline=15.0)
        off_anchor = d.main_offset_y()
        pane_node = d.find_pane_node("V01-C-LONG-START")
        user_ok = False
        if pane_node:
            self_txt = d.cli.text("autoui_action",
                                  {"element_id": pane_node, "action": "scroll", "value": -1200})
            user_ok = "ok" in self_txt
        time.sleep(2.8)
        off_user = d.main_offset_y()
        time.sleep(1.2)
        off_user2 = d.main_offset_y()
        d.check("v06_no_pullback_after_user_scroll",
                v_anchor and v_anchor.get("visible") and user_ok
                and off_user is not None and off_user != off_anchor
                and off_user2 == off_user,
                f"user_scroll={user_ok} anchor_off={off_anchor} user_off={off_user} stable={off_user2 == off_user}")
        ok, det = d.press_reply("GoMgd", "error:unsupported")
        d.check("v06_managed_unsupported", ok, det)

        # ── 证据 ──
        shot_paths = []
        for nm in ("p781vm_final",):
            p = d.shot(nm)
            if p:
                shot_paths.append(p)
        try:
            cli_hash = hashlib.sha256(open(auto_bin, "rb").read()).hexdigest()
        except OSError:
            cli_hash = "unavailable"
        report = {
            "leg": "vm",
            "auto_bin": auto_bin,
            "auto_bin_sha256": cli_hash,
            "checks": [{"name": n, "ok": o, "detail": t} for n, o, t in d.checks],
            "raw": d.raw,
            "screenshots": shot_paths,
        }
        with open(os.path.join(OUT, "vm_report.json"), "w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
        for i, sp in enumerate(shot_paths):
            try:
                import shutil
                shutil.copyfile(sp, os.path.join(OUT, f"vm_shot_{i}.png"))
            except OSError:
                pass

        bad = [c for c in d.checks if not c[1]]
        print("=" * 60)
        if bad:
            print(f"[-] FAILURES ({len(bad)}): " + "; ".join(c[0] for c in bad))
            return 2
        print(f"[+] ALL {len(d.checks)} P781 VM CHECKS PASSED")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()
        try:
            subprocess.run(["taskkill", "/PID", str(proc.pid), "/T", "/F"],
                           capture_output=True)
        except Exception:  # noqa: BLE001
            pass


if __name__ == "__main__":
    sys.exit(main())
