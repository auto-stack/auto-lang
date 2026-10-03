#!/usr/bin/env python3
import json, urllib.request, re, time
from pathlib import Path

PORT = 4288

def call(name, args=None, timeout=20):
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
    return call("autoui_action", {"element_id": vid.lstrip("#"), "action": "press"})

home = snap()
Path("D:/autostack/auto-lang/.tmp-vm-home.txt").write_text(home, encoding="utf-8")
pat = re.compile(r'button #vnode_(\d+) "([^"]+)"')
btns = {}
for m in pat.finditer(home):
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

results = []
for i, label in enumerate(order):
    vid = btns.get(label)
    if not vid:
        results.append({"label": label, "status": "NO_NAV_BTN"})
        print(f"[{i:02d}] {label:16s} NO_NAV_BTN", flush=True)
        continue
    try:
        press(vid)
        time.sleep(0.30)
        s = snap()
    except Exception as e:
        results.append({"label": label, "status": f"ERR {e}"})
        print(f"[{i:02d}] {label:16s} ERR {e}", flush=True)
        continue
    n = len(s)
    nbtn = len(re.findall(r"\bbutton #", s))
    ntext = len(re.findall(r"\btext #", s))
    nsvg = len(re.findall(r"\b(svg|path|rect|circle|image) #", s, re.I))
    nimg = s.count("[Image]")
    flags = []
    if n < 1500:
        flags.append("SHORT")
    if nbtn <= 2 and ntext < 5:
        flags.append("FEW_NODES")
    low = s.lower()
    if "unsupported" in low or "not supported" in low or "未实现" in s:
        flags.append("UNSUPPORTED_TEXT")
    if nimg > 15:
        flags.append(f"MANY_IMG:{nimg}")
    # empty-ish preview: container with only style and no kids in next lines
    empty_blocks = 0
    lines = s.splitlines()
    for j, line in enumerate(lines):
        if re.search(r"(container|col|row|div) #vnode_\d+ \{\s*$", line):
            # look ahead 3 lines for only closing or style-only
            window = "\n".join(lines[j + 1 : j + 5])
            if re.match(r"\s*(style:[^\n]*\n)?\s*\}", window):
                empty_blocks += 1
    if empty_blocks:
        flags.append(f"EMPTY_BLOCKS:{empty_blocks}")
    titles = re.findall(r'text #vnode_\d+ "([^"]{2,40})"', s)
    head = " / ".join(titles[:4])
    rec = {"label": label, "len": n, "btn": nbtn, "text": ntext,
           "svg": nsvg, "img": nimg, "flags": flags, "head": head}
    results.append(rec)
    mark = " **" if flags else ""
    print(f"[{i:02d}] {label:16s} len={n:5d} btn={nbtn:3d} text={ntext:3d} "
          f"svg={nsvg:2d} img={nimg:2d} {flags}{mark}", flush=True)

Path("D:/autostack/auto-lang/.tmp-vm-walk.json").write_text(
    json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8"
)
print("DONE", len(results))
print("SUSPECTS:")
for r in results:
    if r.get("flags") or r.get("status"):
        print(" ", r)
