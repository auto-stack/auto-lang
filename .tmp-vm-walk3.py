#!/usr/bin/env python3
"""Walk widgets-gallery VM — re-resolve sidebar button ids each page."""
import json, urllib.request, re, time
from pathlib import Path

PORT = 4288

def call(name, args=None, timeout=30):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": name, "arguments": args or {}}}
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

def find_btn(s, label):
    m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
    return m.group(1) if m else None

order = [
    "Home", "Row", "Column", "Center", "Flex", "Alignment", "Absolute",
    "Spacing", "Sizing", "Scroll", "Position", "Responsive", "Grid", "GridSpan",
    "Button", "Calendar", "Checkbox", "CodeEditor", "Combobox", "DatePicker",
    "Form", "Input", "Label", "RadioGroup", "Select", "Slider", "Switch",
    "Textarea", "Toggle", "ToggleGroup",
    "LineChart", "BarChart", "AreaChart", "DonutChart", "FlowDiagram",
    "AspectRatio", "Avatar", "Badge", "Card", "Carousel", "DataTable",
    "Skeleton", "Table", "TreeView", "FileTree",
    "Alert", "Progress", "Sonner", "Toast", "Tooltip",
    "Breadcrumb", "Command", "Menubar", "NavigationMenu", "Pagination",
    "ScrollArea", "Separator", "Sidebar", "Tabs",
    "Accordion", "AlertDialog", "Collapsible", "ContextMenu", "Dialog",
    "Drawer", "DropdownMenu", "HoverCard", "Popover", "Sheet",
]

# capture home as baseline chrome
home = snap()
home_texts = set(re.findall(r'text #vnode_\d+ "([^"]{3,90})"', home))
# chrome = texts that appear in ALL pages (sidebar/header) — fill later

results = []
prev_texts = home_texts
prev_label = "(home)"

for i, label in enumerate(order):
    t0 = time.time()
    try:
        s = snap()
        vid = find_btn(s, label)
        if not vid:
            results.append({"label": label, "status": "NO_NAV_BTN"})
            print(f"[{i:02d}] {label:16s} NO_NAV_BTN", flush=True)
            continue
        if label != "Home":
            call("autoui_action", {"element_id": vid, "action": "press"})
            time.sleep(0.35)
            s = snap()
        vid2 = find_btn(s, label)  # confirm sidebar still there
    except Exception as e:
        results.append({"label": label, "status": f"ERR:{e}"})
        print(f"[{i:02d}] {label:16s} ERR {e}", flush=True)
        continue

    texts = set(re.findall(r'text #vnode_\d+ "([^"]{3,90})"', s))
    new_texts = sorted(texts - home_texts)
    # also compare to previous page to detect stuck nav
    stuck = (label != "Home") and (texts == prev_texts)

    nbtn = len(re.findall(r"\bbutton #", s))
    ntext = len(re.findall(r"\btext #", s))
    nimg = s.count("[Image]")
    nsvg = len(re.findall(r"\b(svg|path|rect|circle|line|polyline|image) #", s, re.I))

    flags = []
    if stuck:
        flags.append("STUCK_NAV")
    if nbtn <= 4 and ntext < 10:
        flags.append("FEW_NODES")
    # empty preview heuristic
    empty_pv = 0
    lines = s.splitlines()
    for j, ln in enumerate(lines):
        if "min-h-[100px]" in ln or "min-h-24" in ln:
            inner = "\n".join(lines[j + 1 : j + 6])
            if "text #" not in inner and "button #" not in inner and "input" not in inner:
                empty_pv += 1
    if empty_pv:
        flags.append(f"EMPTY_PREVIEW:{empty_pv}")
    if label.endswith("Chart"):
        if not re.search(r"M \d+|path|series|seg", s, re.I):
            flags.append("NO_CHART_GEOM")
    if label in ("Dialog", "Sheet", "Drawer", "Popover", "Command", "DataTable",
                 "Carousel", "CodeEditor", "Combobox", "Menubar"):
        if nbtn < 4:
            flags.append("FEW_TRIGGERS")

    rec = {
        "label": label,
        "len": len(s),
        "btn": nbtn,
        "text": ntext,
        "img": nimg,
        "svg": nsvg,
        "new_n": len(new_texts),
        "new": new_texts[:10],
        "stuck": stuck,
        "flags": flags,
        "dt": round(time.time() - t0, 2),
    }
    results.append(rec)
    prev_texts = texts
    prev_label = label
    mark = " **" if flags else ""
    print(
        f"[{i:02d}] {label:16s} len={len(s):5d} btn={nbtn:3d} text={ntext:3d} "
        f"new={len(new_texts):3d} svg={nsvg:2d} {rec['dt']}s {flags}{mark}",
        flush=True,
    )

Path("D:/autostack/auto-lang/.tmp-vm-walk.json").write_text(
    json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8"
)
print("\n==== SUSPECTS ====")
for r in results:
    if r.get("flags") or r.get("status"):
        print(json.dumps({k: r[k] for k in r if k in (
            "label", "status", "flags", "new", "stuck", "btn", "text", "len"
        )}, ensure_ascii=False)[:350])
