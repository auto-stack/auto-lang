import json, urllib.request, re, time

def call(n, a=None, timeout=45):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp", data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"})
    res = json.loads(urllib.request.urlopen(r, timeout=timeout).read().decode())
    return res.get("result", {}).get("content", [{}])[0].get("text", "")

def snap():
    return call("autoui_snapshot", {"mode": "rendered"})

def route():
    t = call("autoui_state", {"fields": ["__current_route"]})
    m = re.search(r'__current_route: "([^"]*)"', t)
    return m.group(1) if m else ""

def goto(label, want, tries=2):
    for _ in range(tries):
        s = snap()
        if route() == want:
            return s
        m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
        if not m:
            time.sleep(0.4)
            continue
        try:
            call("autoui_action", {"element_id": m.group(1), "action": "press"})
        except Exception:
            pass
        for _ in range(50):
            time.sleep(0.25)
            if route() == want:
                return snap()
    return snap()

PAGES = [
    ("Accordion", "/accordion"),
    ("Calendar", "/calendar"),
    ("Breadcrumb", "/breadcrumb"),
    ("Pagination", "/pagination"),
    ("ScrollArea", "/scrollarea"),
    ("NavigationMenu", "/navigationmenu"),
    ("ContextMenu", "/contextmenu"),
    ("Sonner", "/sonner"),
    ("Form", "/form"),
    ("Dialog", "/dialog"),
    ("Drawer", "/drawer"),
    ("Sheet", "/sheet"),
    ("HoverCard", "/hovercard"),
    ("Popover", "/popover"),
    ("Carousel", "/carousel"),
    ("ToggleGroup", "/togglegroup"),
    ("CodeEditor", "/code-editor"),
    ("DataTable", "/datatable"),
]

def preview_empty(s):
    """Detect empty min-h preview boxes."""
    empty = 0
    lines = s.splitlines()
    for j, ln in enumerate(lines):
        if "min-h-[100px]" in ln or "min-h-24" in ln or "min-h-[80px]" in ln:
            inner = "\n".join(lines[j+1:j+7])
            if "text #" not in inner and "button #" not in inner and "input #" not in inner:
                empty += 1
    return empty

for label, want in PAGES:
    s = goto(label, want)
    cur = route()
    # content texts after the last Sheet nav item (sidebar tail)
    texts = re.findall(r'text #vnode_\d+ "([^"]{2,70})"', s)
    # drop pure chrome: first ~140 are sidebar
    content = texts[130:] if len(texts) > 150 else texts[80:]
    nbtn = len(re.findall(r"\bbutton #", s))
    ninput = len(re.findall(r"\binput #", s))
    emp = preview_empty(s)
    flags = []
    if cur != want:
        flags.append(f"ROUTE:{cur}")
    if emp:
        flags.append(f"EMPTY_PV:{emp}")
    if ninput == 0 and label in ("Form", "CodeEditor", "DataTable", "Pagination"):
        flags.append("NO_INPUT")
    # widget-specific
    body = " ".join(content)
    if label == "Accordion" and "Accordion" not in body and "item" not in body.lower():
        flags.append("NO_WIDGET_BODY")
    if label == "Calendar" and not re.search(r"Mon|Tue|Jan|day|week", body, re.I):
        flags.append("NO_CALENDAR_GRID")
    if label == "Pagination" and not re.search(r"page|next|prev|1|2", body, re.I):
        flags.append("NO_PAGER")
    print(f"{label:16s} {cur:16s} btn={nbtn:3d} in={ninput} emp={emp} "
          f"content_n={len(content)} {flags}", flush=True)
    print(f"   body: {' | '.join(content[:12])}", flush=True)
    try:
        call("autoui_screenshot", {"name": f"vm_{label.lower()}", "baseline": True})
    except Exception:
        pass
