import re

variants = []
for line in open('data/ (相对 docs/specs/aavm/)opcode.rs', encoding='utf-8'):
    m = re.match(r'\s+([A-Z][A-Z0-9_]*)\s*=\s*(0x[0-9A-Fa-f]+|\d+)', line)
    if m:
        variants.append((m.group(1), m.group(2)))

engine = open('data/ (相对 docs/specs/aavm/)engine.rs', encoding='utf-8').read()

UI_KW = ['WIDGET','VIEW','SCENE','STYLE','UI_','_UI']
CONC_KW = ['SPAWN','FUTURE','AWAIT','POLL','CHANNEL','ACTOR']

def classify(name):
    n = name.upper()
    if any(k in n for k in UI_KW):
        return '剔除(UI)'
    if any(k in n for k in CONC_KW):
        return '剔除(并发/actor,432按需恢复)'
    if f'OpCode::{n}' in engine:
        return '移植'
    return '仅声明'

from collections import Counter
c = Counter()
rows = []
for name, val in variants:
    d = classify(name)
    c[d] += 1
    rows.append((name, val, d))
with open('data/ (相对 docs/specs/aavm/)opcode_table.csv', 'w', encoding='utf-8') as f:
    f.write("opcode,value,disposition\n")
    for r in rows: f.write(",".join(r) + "\n")
print(dict(c))
