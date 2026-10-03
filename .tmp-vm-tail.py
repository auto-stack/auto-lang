import json, urllib.request, re

def call(n, a=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp", data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"})
    return json.loads(urllib.request.urlopen(r, timeout=20).read().decode()
                     ).get("result", {}).get("content", [{}])[0].get("text", "")

try:
    print("route", call("autoui_state", {"fields": ["__current_route"]})[:100])
    s = call("autoui_snapshot", {"mode": "rendered"})
    idx = s.rfind("\nSheet")
    tail = s[idx:] if idx > 0 else s[-9000:]
    texts = re.findall(r'text #vnode_\d+ "([^"]{2,70})"', tail)
    print("n_texts", len(texts))
    print("content:", texts[:30])
    open("D:/autostack/auto-lang/.tmp-vm-tail.txt", "w", encoding="utf-8").write(tail[:15000])
except Exception as e:
    print("ERR", type(e), e)
