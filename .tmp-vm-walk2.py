#!/usr/bin/env python3
"""Walk widgets-gallery VM pages via AutoUI MCP; flag empty/broken previews."""
import json, urllib.request, re, time
from pathlib import Path

PORT = 4288

def call(name, args=None, timeout=25):
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

def press(vid):
    return call("autoui_action", {"element_id": vid, "action": "press"})

home = snap()
btns = {}
for m in re.finditer(r'button #(vnode_\d+) "([^"]+)"', home):
    btns.setdefault(m.group(2), m.group(1))

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
for k in btns:
    if "kitchen" in k.lower() or "sink" in k.lower():
        order.append(k)

def extract_content(snap_text):
    """Content = texts that are NOT in the sidebar/header chrome.
    Heuristic: after the first 'Layout' group header of sidebar ends,
    or all unique long texts minus home chrome."""
    return snap_text

home_texts = set(re.findall(r'text #vnode_\d+ "([^"]{4,80})"', home))
home_len = len(home)

results = []
for i, label in enumerate(order):
    vid = btns.get(label)
    if not vid:
        results.append({"label": label, "status": "NO_NAV"})
        print(f"[{i:02d}] {label:16s} NO_NAV", flush=True)
        continue
    try:
        press(vid)
        time.sleep(0.32)
        s = snap()
    except Exception as e:
        results.append({"label": label, "status": f"ERR:{e}"})
        print(f"[{i:02d}] {label:16s} ERR {e}", flush=True)
        continue

    texts = set(re.findall(r'text #vnode_\d+ "([^"]{4,80})"', s))
    new_texts = sorted(texts - home_texts)
    nbtn = len(re.findall(r"\bbutton #", s))
    ntext = len(re.findall(r"\btext #", s))
    nimg = s.count("[Image]")
    nsvg = len(re.findall(r"\b(svg|path|rect|circle|line|polyline) #", s, re.I))
    # custom widget tags often appear as kind or bare names
    custom = re.findall(r"\b(AreaChart|BarChart|LineChart|DonutChart|FlowDiagram|FileTree|TreeView|DataTable|CodeEditor|Combobox|Command|Carousel|Menubar|ToggleGroup)\b", s)

    flags = []
    route_ok = True
    # detect navigation: __current_route in action result or new content
    if not new_texts and abs(len(s) - home_len) < 200:
        flags.append("NAV_STUCK")
        route_ok = False
    if nbtn <= 3 and ntext < 8:
        flags.append("FEW_NODES")
    # empty preview: look for container immediately closed with no kids near preview styles
    empty_pv = 0
    lines = s.splitlines()
    for j, ln in enumerate(lines):
        if "min-h-[100px]" in ln or "min-h-24" in ln or "preview" in ln.lower():
            # if next non-style line is just }, empty
            k = j + 1
            body = []
            while k < len(lines) and k < j + 8:
                body.append(lines[k])
                if lines[k].strip() == "}":
                    break
                k += 1
            inner = "\n".join(body)
            if inner.count("{") == 0 and "text #" not in inner and "button #" not in inner:
                empty_pv += 1
    if empty_pv:
        flags.append(f"EMPTY_PREVIEW:{empty_pv}")
    # chart geometry: area/line should have path-ish or long style path strings
    if label.endswith("Chart") or label == "AreaChart":
        has_geom = bool(re.search(r'M \d+|path|seg|series', s, re.I))
        if not has_geom:
            flags.append("NO_CHART_GEOM")
        # specifically area chart path in state was 'M 40 ...' as text? or in snapshot?
        if "M 40" not in s and "M 550" not in s and "path" not in s.lower():
            if "NO_CHART_GEOM" not in flags:
                flags.append("NO_PATH_DATA")
    # overlay components should still have trigger buttons
    if label in ("Dialog", "Sheet", "Drawer", "Popover", "HoverCard", "AlertDialog",
                 "DropdownMenu", "ContextMenu", "Command"):
        if nbtn < 3:
            flags.append("FEW_TRIGGERS")

    rec = {
        "label": label,
        "len": len(s),
        "btn": nbtn,
        "text": ntext,
        "img": nimg,
        "svg": nsvg,
        "new_text_n": len(new_texts),
        "new_texts": new_texts[:12],
        "custom": custom[:8],
        "flags": flags,
    }
    results.append(rec)
    mark = " **" if flags else ""
    print(
        f"[{i:02d}] {label:16s} len={len(s):5d} btn={nbtn:3d} text={ntext:3d} "
        f"new={len(new_texts):3d} img={nimg:3d} svg={nsvg:2d} {flags}{mark}",
        flush=True,
    )

Path("D:/autostack/auto-lang/.tmp-vm-walk.json").write_text(
    json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8"
)
print("\n==== SUSPECTS ====")
for r in results:
    if r.get("flags") or r.get("status"):
        print(json.dumps(r, ensure_ascii=False)[:300])
