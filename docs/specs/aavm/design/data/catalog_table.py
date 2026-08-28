import re
src = open('data/ (相对 docs/specs/aavm/)native_catalog.rs', encoding='utf-8').read()
# 条目形态:("auto.xxx.yyy", ID) 或 ("auto.xxx.yyy", NATIVE_X, fn, canonical)
entries = re.findall(r'\("((?:auto|io|py|rust)\.[^"]+)",\s*([A-Z_0-9]+|0x[0-9A-Fa-f]+|\d+)\)?', src)
seen = set()
rows = []
UI_KW = ('ui.', 'view.', 'store.', 'scene.', 'style.', 'task.', 'http.', 'ws.', 'websocket', 'async', 'await', 'spawn', 'timer.', 'actor')
def classify(name):
    n = name.lower()
    if any(n.startswith(k) or ('.'+k) in n for k in UI_KW):
        return '剔除(UI/异步)'
    return '核心'
from collections import Counter
c = Counter()
for name, ident in entries:
    if name in seen: continue
    seen.add(name)
    d = classify(name)
    c[d] += 1
    rows.append((name, ident, d))
with open('data/ (相对 docs/specs/aavm/)catalog_table.csv','w',encoding='utf-8') as f:
    f.write("native,ident,disposition\n")
    for r in rows: f.write(",".join(r)+"\n")
print(len(rows), dict(c))
print('ui sample:', [r[0] for r in rows if r[2]!='核心'][:12])
