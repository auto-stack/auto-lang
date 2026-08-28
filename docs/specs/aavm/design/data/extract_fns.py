import re, sys

def extract(path):
    lines = open(path, encoding='utf-8').read().splitlines()
    fn_re = re.compile(r'^(\s*)(?:pub(?:\([^)]*\))? )?(?:async )?(?:unsafe )?(?:const )?fn (\w+)')
    hits = []
    for i, line in enumerate(lines):
        m = fn_re.match(line)
        if m:
            hits.append((m.group(2), len(m.group(1)), i + 1))
    out = []
    for k, (name, indent, s) in enumerate(hits):
        e = hits[k + 1][2] - 1 if k + 1 < len(hits) else len(lines)
        out.append((name, indent, s, e))
    return out

def classify(name):
    n = name.lower()
    if any(k in n for k in ['widget','store','scene','route','msg','onevent','on_event','tag','grid','cover','vue','view','aura','style','css']):
        return 'ui'
    if 'task' in n:
        return 'task'
    return 'core'

path = sys.argv[1]
fns = extract(path)
print("fn_name,indent,start,end,lines,kind")
for name, indent, s, e in fns:
    print(f"{name},{indent},{s},{e},{e-s+1},{classify(name)}")
