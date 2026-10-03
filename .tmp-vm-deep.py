import json, urllib.request, re, time

def call(n, a=None, timeout=45):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp",
        data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"},
    )
    res = json.loads(urllib.request.urlopen(r, timeout=timeout).read().decode())
    return res.get("result", {}).get("content", [{}])[0].get("text", "")

def snap():
    return call("autoui_snapshot", {"mode": "rendered"})

def route():
    t = call("autoui_state", {"fields": ["__current_route"]})
    m = re.search(r'__current_route: "([^"]*)"', t)
    return m.group(1) if m else ""

def goto(label, want):
    for _ in range(3):
        s = snap()
        if route() == want:
            return s
        m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
        if m:
            try:
                call("autoui_action", {"element_id": m.group(1), "action": "press"})
            except Exception:
                pass
        for _ in range(60):
            time.sleep(0.25)
            if route() == want:
                return snap()
    return snap()

# Command
print("==== COMMAND ====")
s = goto("Command", "/command")
print("route", route(), "len", len(s))
# print content-ish texts
texts = re.findall(r'text #vnode_\d+ "([^"]{2,60})"', s)
print("texts:", texts)
print("inputs:", re.findall(r'input #vnode_\d+[^\n]*', s)[:5])
print("command-ish kinds:", re.findall(r'\b(Command|command|Listbox|Combobox|Palette)\b', s)[:20])
open("D:/autostack/auto-lang/.tmp-vm-command.txt","w",encoding="utf-8").write(s)

print("\n==== COMBOBOX ====")
s = goto("Combobox", "/combobox")
print("route", route(), "len", len(s))
texts = re.findall(r'text #vnode_\d+ "([^"]{2,60})"', s)
print("texts:", texts)
print("inputs:", re.findall(r'input #vnode_\d+[^\n]*', s)[:5])
open("D:/autostack/auto-lang/.tmp-vm-combobox.txt","w",encoding="utf-8").write(s)

print("\n==== DONUT ====")
s = goto("DonutChart", "/donut-chart")
print("route", route(), "len", len(s))
texts = re.findall(r'text #vnode_\d+ "([^"]{2,60})"', s)
print("texts:", texts)
# geometry?
print("has arc/path/M:", bool(re.search(r'M \d+|arc|path|d=', s, re.I)))
print("img count", s.count("[Image]"))
open("D:/autostack/auto-lang/.tmp-vm-donut.txt","w",encoding="utf-8").write(s)

print("\n==== AREA (compare) ====")
s = goto("AreaChart", "/area-chart")
print("route", route(), "len", len(s))
print("has M path", bool(re.search(r'M \d+', s)))
print("texts sample", re.findall(r'text #vnode_\d+ "([^"]{2,40})"', s)[:15])
