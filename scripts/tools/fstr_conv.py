
import re, sys, io

IDENT = r'[A-Za-z_]\w*'
FIELD = IDENT + r'(?:\.' + IDENT + r')*(?:\([^()]*(?:\([^()]*\)[^()]*)*\))?'

def seg_ok(s):
    s = s.strip()
    if re.fullmatch(r'\d+', s): return ('n', s)
    if re.fullmatch(FIELD, s):
        if '{' in s or '}' in s: return None
        return ('e', s)
    return None

def lit_parse(s):
    s = s.strip()
    if len(s) < 2 or not (s.startswith('"') and s.endswith('"')): return None
    inner = s[1:-1]
    if '$' in inner or '"' in inner or chr(92) in inner: return None
    return inner

def split_top(rhs):
    parts = []; cur = ''; in_str = False
    for ch in rhs:
        if in_str:
            cur += ch
            if ch == '"': in_str = False
        else:
            if ch == '"':
                in_str = True; cur += ch
            elif ch == '+':
                parts.append(cur); cur = ''
            else:
                cur += ch
    parts.append(cur)
    return [p.strip() for p in parts]

def try_line(line):
    raw = line.rstrip("\n")
    m = re.match(r'^(\s*)((?:var|let)\s+\w+(?:\s+List<[\w<>]+>)?\s*=\s*|(\w+)\s*=\s*)(.*?)(;?)\s*$', raw)
    if not m: return None
    indent, decl, bare, rhs, semi = m.groups()
    if not (' + "' in rhs or '" +' in rhs): return None
    parts = split_top(rhs)
    if len(parts) < 2: return None
    segs = []
    for p in parts:
        if p.startswith('"'):
            t = lit_parse(p)
            if t is None: return None
            segs.append(('t', t))
        else:
            t = seg_ok(p)
            if t is None: return None
            segs.append(t)
    if not any(k in ('e','n') for k,_ in segs): return None
    fs = 'f"'
    for k, v in segs:
        if k == 't':
            fs += v.replace('{', '{{').replace('}', '}}')
        else:
            fs += '${' + v + '}'
    fs += '"'
    prefix = decl if decl else (bare + ' = ' if bare else '')
    return indent + prefix + fs + semi

def main(path, dry=True):
    out = []; converted = 0
    for line in io.open(path, encoding="utf-8"):
        r = try_line(line)
        if r is not None and r != line.rstrip("\n"):
            converted += 1
            out.append(r + "\n")
        else:
            out.append(line)
    if not dry:
        io.open(path, "w", encoding="utf-8", newline="\n").write("".join(out))
    print(path + ": converted " + str(converted))

def run_batch(files, apply):
    for f in files:
        main(f, dry=not apply)

if __name__ == "__main__":
    main(sys.argv[1], dry=(len(sys.argv) < 3))
