"""PLAN-536 T5 reactive flow: select session → UI send → observe reply without re-select."""
import json, sys, time, urllib.request

PORT = json.load(open(r"D:/autostack/.wt/lang-536/auto-lang/scratch/p536_t5_musk_vm.log.meta.json"))["port"]
URL = f"http://127.0.0.1:{PORT}/mcp"
_rid = 1

def call(tool, args=None):
    global _rid
    req = {"jsonrpc": "2.0", "id": _rid, "method": "tools/call",
           "params": {"name": tool, "arguments": args or {}}}
    _rid += 1
    r = urllib.request.Request(URL, data=json.dumps(req).encode(),
                               headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(r, timeout=15) as resp:
        res = json.loads(resp.read().decode())
        if "error" in res:
            raise RuntimeError(f"{tool}: {res['error']}")
        return res.get("result", {})

def text(tool, args=None):
    return call(tool, args).get("content", [{}])[0].get("text", "")

def texts_in_view():
    """All rendered text lines from the styled vtree snapshot."""
    snap = text("autoui_snapshot")
    out = []
    for line in snap.splitlines():
        s = line.strip()
        if s.startswith("text #"):
            # text #id "content"
            if '"' in s:
                out.append(s.split('"', 1)[1].rsplit('"', 1)[0])
    return out

# 1) find my session button in the list
sess_id = "f07c9ace541172de5bc3a23a"
snap = text("autoui_snapshot")
lines = snap.splitlines()
target = None
for i, line in enumerate(lines):
    if "p536 reactive probe ping" in line:
        # walk back to nearest button node id
        for j in range(i, max(0, i - 12), -1):
            if lines[j].strip().startswith("button #"):
                target = lines[j].split("#", 1)[1].split()[0]
                break
        break
print("[session button]", target)
if not target:
    print("[-] session not in list yet"); sys.exit(2)

# 2) click it (select session)
print("[click]", text("autoui_click", {"element_id": target})[:120])
time.sleep(2.0)

# 3) find composer textarea and type via autoui_type (declared focusable)
snap = text("autoui_snapshot")
comp = None
for line in snap.splitlines():
    if "textarea #" in line or ("textbox" in line and "#" in line):
        comp = line.split("#", 1)[1].split()[0]
        break
print("[composer]", comp)

# fallback: use state-inspect approach — find by autoui_type on 'composer' known id shape
if comp:
    print("[type]", text("autoui_type", {"element_id": comp, "text": "second probe: reply with pong too", "clear_first": True})[:120])
else:
    print("[!] no textarea found; snapshot head:")
    print(snap[:1500])
    sys.exit(3)

time.sleep(1.0)
# 4) press Enter (keyboard) to send
print("[enter]", text("autoui_keyboard", {"key": "Enter"})[:120])

# 5) observe WITHOUT re-selecting: poll snapshot texts for 'pong'
t0 = time.time()
seen = ""
deadline = 90
while time.time() - t0 < deadline:
    time.sleep(3)
    body = " | ".join(texts_in_view())
    if "pong" in body.lower():
        print(f"[REPLY VISIBLE] after {time.time()-t0:.0f}s without re-select")
        seen = body
        break
    seen = body
else:
    print("[-] no reply visible in 90s")
print("[canvas tail]", seen[-600:] if seen else "(empty)")
