#!/usr/bin/env python3
"""VM gallery walk: poll __current_route after each press, then analyze content."""
import json, urllib.request, re, time
from pathlib import Path

PORT = 4288
ROUTE_OF = {
    "Home": "/", "Row": "/row", "Column": "/col", "Center": "/center",
    "Flex": "/flex", "Alignment": "/alignment", "Absolute": "/absolute",
    "Spacing": "/spacing", "Sizing": "/sizing", "Scroll": "/scroll",
    "Position": "/position", "Responsive": "/responsive", "Grid": "/grid",
    "GridSpan": "/grid-span",
    "Button": "/button", "Calendar": "/calendar", "Checkbox": "/checkbox",
    "CodeEditor": "/code-editor", "Combobox": "/combobox", "DatePicker": "/datepicker",
    "Form": "/form", "Input": "/input", "Label": "/label", "RadioGroup": "/radiogroup",
    "Select": "/select", "Slider": "/slider", "Switch": "/switch",
    "Textarea": "/textarea", "Toggle": "/toggle", "ToggleGroup": "/togglegroup",
    "LineChart": "/line-chart", "BarChart": "/bar-chart", "AreaChart": "/area-chart",
    "DonutChart": "/donut-chart", "FlowDiagram": "/flow-diagram",
    "AspectRatio": "/aspectratio", "Avatar": "/avatar", "Badge": "/badge",
    "Card": "/card", "Carousel": "/carousel", "DataTable": "/datatable",
    "Skeleton": "/skeleton", "Table": "/table", "TreeView": "/treeview",
    "FileTree": "/filetree",
    "Alert": "/alert", "Progress": "/progress", "Sonner": "/sonner",
    "Toast": "/toast", "Tooltip": "/tooltip",
    "Breadcrumb": "/breadcrumb", "Command": "/command", "Menubar": "/menubar",
    "NavigationMenu": "/navigationmenu", "Pagination": "/pagination",
    "ScrollArea": "/scrollarea", "Separator": "/separator", "Sidebar": "/sidebar",
    "Tabs": "/tabs",
    "Accordion": "/accordion", "AlertDialog": "/alertdialog",
    "Collapsible": "/collapsible", "ContextMenu": "/contextmenu",
    "Dialog": "/dialog", "Drawer": "/drawer", "DropdownMenu": "/dropdownmenu",
    "HoverCard": "/hovercard", "Popover": "/popover", "Sheet": "/sheet",
}

def call(n, a=None, timeout=45):
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
    return m.group(1) if m else t.strip()

def find_btn(s, label):
    # prefer button whose inspect has __navigate — but first match in sidebar
    # skip buttons inside preview by taking the LAST match (sidebar often after content?
    # actually sidebar is first). Take first.
    m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
    return m.group(1) if m else None

results = []
for label, want in ROUTE_OF.items():
    t0 = time.time()
    try:
        s = snap()
        cur = route()
        if cur != want:
            vid = find_btn(s, label)
            if not vid:
                results.append({"label": label, "want": want, "status": "NO_NAV"})
                print(f"{label:16s} NO_NAV", flush=True)
                continue
            call("autoui_action", {"element_id": vid, "action": "press"})
            # poll route
            ok = False
            for _ in range(40):  # up to ~8s
                time.sleep(0.2)
                try:
                    if route() == want:
                        ok = True
                        break
                except Exception:
                    pass
            if not ok:
                # one more try with longer wait
                time.sleep(1.5)
                ok = route() == want
            time.sleep(0.15)
            s = snap()
            cur = route()
        else:
            ok = True
            s = snap()
            cur = route()
    except Exception as e:
        results.append({"label": label, "want": want, "status": f"ERR:{e}"})
        print(f"{label:16s} ERR {e}", flush=True)
        continue

    texts = re.findall(r'text #vnode_\d+ "([^"]{3,90})"', s)
    nbtn = len(re.findall(r"\bbutton #", s))
    ntext = len(texts)
    nimg = s.count("[Image]")
    nsvg = len(re.findall(r"\b(svg|path|rect|circle|line|polyline|image) #", s, re.I))

    flags = []
    if cur != want:
        flags.append(f"ROUTE_MISMATCH:{cur}")
    if nbtn <= 3 and ntext < 8:
        flags.append("FEW_NODES")
    empty_pv = 0
    lines = s.splitlines()
    for j, ln in enumerate(lines):
        if "min-h-[100px]" in ln or "min-h-24" in ln:
            inner = "\n".join(lines[j + 1 : j + 6])
            if "text #" not in inner and "button #" not in inner and "input #" not in inner:
                empty_pv += 1
    if empty_pv:
        flags.append(f"EMPTY_PREVIEW:{empty_pv}")
    if label.endswith("Chart"):
        if not re.search(r"M \d+|series|seg|path", s, re.I):
            flags.append("NO_CHART_GEOM")
    # specific empties
    if label in ("Command", "Combobox", "ToggleGroup", "Carousel", "CodeEditor"):
        if nbtn < 5:
            flags.append("FEW_WIDGET_NODES")

    rec = {
        "label": label, "want": want, "cur": cur, "ok_route": cur == want,
        "len": len(s), "btn": nbtn, "text": ntext, "img": nimg, "svg": nsvg,
        "flags": flags, "dt": round(time.time() - t0, 2),
        "head": " | ".join(texts[:8]),
    }
    results.append(rec)
    mark = " **" if flags else ""
    print(
        f"{label:16s} route={cur:16s} ok={rec['ok_route']} len={len(s):5d} "
        f"btn={nbtn:3d} text={ntext:3d} svg={nsvg:2d} {rec['dt']}s {flags}{mark}",
        flush=True,
    )

Path("D:/autostack/auto-lang/.tmp-vm-walk.json").write_text(
    json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8"
)
print("\n==== PROBLEMS ====")
for r in results:
    if r.get("flags") or r.get("status") or not r.get("ok_route", True):
        print(json.dumps({k: r.get(k) for k in
            ("label", "want", "cur", "status", "flags", "btn", "text", "len", "head")
            if k in r or r.get(k) is not None}, ensure_ascii=False)[:400])
