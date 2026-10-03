#!/usr/bin/env python3
"""Targeted VM gallery checks for key components + screenshots."""
import json, urllib.request, re, time
from pathlib import Path

PORT = 4288

def call(n, a=None, timeout=60):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        f"http://127.0.0.1:{PORT}/mcp",
        data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"},
    )
    res = json.loads(urllib.request.urlopen(r, timeout=timeout).read().decode())
    if "error" in res:
        raise RuntimeError(res["error"])
    return res.get("result", {}).get("content", [{}])[0].get("text", "")

def snap():
    return call("autoui_snapshot", {"mode": "rendered"})

def route():
    t = call("autoui_state", {"fields": ["__current_route"]})
    m = re.search(r'__current_route: "([^"]*)"', t)
    return m.group(1) if m else ""

TARGETS = [
    ("Column", "/col"),
    ("Center", "/center"),
    ("Grid", "/grid"),
    ("Button", "/button"),
    ("CodeEditor", "/code-editor"),
    ("Label", "/label"),
    ("RadioGroup", "/radiogroup"),
    ("Textarea", "/textarea"),
    ("DonutChart", "/donut-chart"),
    ("Carousel", "/carousel"),
    ("DataTable", "/datatable"),
    ("FileTree", "/filetree"),
    ("Command", "/command"),
    ("Combobox", "/combobox"),
    ("ToggleGroup", "/togglegroup"),
    ("Menubar", "/menubar"),
    ("Sidebar", "/sidebar"),
    ("Dialog", "/dialog"),
    ("Toast", "/toast"),
    ("Form", "/form"),
]

def analyze(label, s):
    texts = re.findall(r'text #vnode_\d+ "([^"]{3,90})"', s)
    nbtn = len(re.findall(r"\bbutton #", s))
    ntext = len(texts)
    nimg = s.count("[Image]")
    nsvg = len(re.findall(r"\b(svg|path|rect|circle|line|polyline|image) #", s, re.I))
    ninput = len(re.findall(r"\binput #", s))
    flags = []
    empty_pv = 0
    lines = s.splitlines()
    for j, ln in enumerate(lines):
        if "min-h-[100px]" in ln or "min-h-24" in ln:
            inner = "\n".join(lines[j + 1 : j + 6])
            if "text #" not in inner and "button #" not in inner and "input #" not in inner:
                empty_pv += 1
    if empty_pv:
        flags.append(f"EMPTY_PREVIEW:{empty_pv}")
    if label.endswith("Chart") or label == "DonutChart":
        if not re.search(r"M \d+|series|seg|path|arc|circle", s, re.I):
            flags.append("NO_CHART_GEOM")
    if label in ("Command", "Combobox"):
        if nbtn < 6 or ninput < 1:
            flags.append("WEAK_WIDGET")
    if label == "ToggleGroup" and nbtn < 6:
        flags.append("WEAK_WIDGET")
    if label == "DataTable" and ntext < 30:
        flags.append("THIN_TABLE")
    return {
        "label": label, "len": len(s), "btn": nbtn, "text": ntext,
        "img": nimg, "svg": nsvg, "input": ninput,
        "flags": flags, "empty_pv": empty_pv,
        "head": " | ".join(texts[:10]),
    }

results = []
for label, want in TARGETS:
    got = False
    rec = None
    for attempt in range(3):
        try:
            s = snap()
            cur = route()
            if cur == want:
                got = True
                break
            m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
            if not m:
                # retry after short wait
                time.sleep(0.5)
                s = snap()
                m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
            if not m:
                rec = {"label": label, "want": want, "status": "NO_NAV_BTN"}
                break
            call("autoui_action", {"element_id": m.group(1), "action": "press"})
            for _ in range(80):
                time.sleep(0.25)
                if route() == want:
                    got = True
                    break
            if got:
                break
        except Exception as e:
            rec = {"label": label, "want": want, "status": f"ERR:{e}"}
            time.sleep(1)
    if rec and rec.get("status"):
        results.append(rec)
        print(f"{label:16s} {rec['status']}", flush=True)
        continue
    time.sleep(0.2)
    s = snap()
    cur = route()
    a = analyze(label, s)
    a["want"] = want
    a["cur"] = cur
    a["ok_route"] = cur == want
    if not a["ok_route"]:
        a.setdefault("flags", []).append(f"ROUTE_MISMATCH:{cur}")
    results.append(a)
    mark = " **" if a.get("flags") else ""
    print(
        f"{label:16s} route={cur:16s} ok={a['ok_route']} len={a['len']:5d} "
        f"btn={a['btn']:3d} text={a['text']:3d} in={a['input']} svg={a['svg']} "
        f"{a['flags']}{mark}",
        flush=True,
    )
    try:
        call("autoui_screenshot", {"name": f"vm_{label.lower()}", "baseline": True})
    except Exception as e:
        print(f"  shot err {e}", flush=True)

Path("D:/autostack/auto-lang/.tmp-vm-targeted.json").write_text(
    json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8"
)
print("\n==== PROBLEMS ====")
for r in results:
    if r.get("flags") or r.get("status") or not r.get("ok_route", True):
        print(json.dumps(r, ensure_ascii=False)[:400])
