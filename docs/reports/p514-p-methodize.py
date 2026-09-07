import re, sys

W = "D:/autostack/auto-lang/.worktrees/plan-514-dev"
p = W + "/auto/lib/parser.at"
s = open(p, encoding="utf-8").read()

# 1. type P gains methods (after the fields block's closing brace)
old_type = """type P {
    toks List<Token>
    pos int
    err str
    scopes List<List<Binding>>
    globals List<Binding>
    depth int
    decls List<Binding>
}"""
new_type = """type P {
    toks List<Token>
    pos int
    err str
    scopes List<List<Binding>>
    globals List<Binding>
    depth int
    decls List<Binding>

    // ── Plan 514 W3(γ4 试点):P 游标自有操作族入 type 体 ──
    // 映射:原自由函数 p_kind/p_text/.../pop_scope;`err` 字段与
    // p_err 撞名 → 方法名 fail;语法产生式(parse_*)与纯表函数
    // (is_comment_kind/p_op/...)保留自由函数(待澄清①缺省)。

    fn kind() TokenKind {
        if .pos >= .toks.len() {
            return TokenKind.EOF
        }
        return .toks.get(.pos).kind
    }

    fn text() str {
        if .pos >= .toks.len() {
            return ""
        }
        return .toks.get(.pos).text
    }

    fn line() int {
        if .pos >= .toks.len() {
            return 0
        }
        return .toks.get(.pos).line
    }

    // 原始前瞻(不跳注释;与基线 lexer.pushback 前瞻同构,corpus 括号内无注释)
    fn peek(n int) TokenKind {
        var i = .pos + n
        if i >= .toks.len() {
            return TokenKind.EOF
        }
        return .toks.get(i).kind
    }

    // Parser::next 直译:前进一 token 并跳过注释 token
    fn next() {
        .pos = .pos + 1
        var scanning = true
        for scanning {
            if .pos >= .toks.len() {
                scanning = false
            } else if is_comment_kind(.toks.get(.pos).kind) {
                .pos = .pos + 1
            } else {
                scanning = false
            }
        }
    }

    fn fail(msg str) {
        if .err == "" {
            .err = "PARSE-ERROR:" + .line().str() + ":" + msg + " <" + kind_name(.kind()) + ">"
        }
    }

    fn expect(k TokenKind) {
        if .err != "" {
            return
        }
        if .kind() == k {
            .next()
        } else {
            .fail("expected " + kind_name(k) + ", got ")
        }
    }

    fn skip_empty_lines() int {
        var count = 0
        while .kind() == TokenKind.Newline {
            count = count + 1
            self.next()
        }
        return count
    }

    fn push_scope() {
        var sc List<Binding> = List.new()
        .scopes.push(sc)
        .depth = .depth + 1
    }

    fn pop_scope() {
        .depth = .depth - 1
    }

    fn bind(name str, ty str) {
        if .depth > 0 {
            // 提升+写回:VM 引用语义下 set 写回同一引用(无操作);
            // a2r 值语义下写回修改后的副本(VM/a2r 可观察状态一致)
            var sc = .scopes.get(.depth - 1)
            sc.push(Binding(name, ty))
            .scopes.set(.depth - 1, sc)
        } else {
            .globals.push(Binding(name, ty))
        }
    }

    fn lookup(name str) str {
        var s = .depth - 1
        while s >= 0 {
            var scope List<Binding> = .scopes.get(s)
            var i = scope.len() - 1
            while i >= 0 {
                if scope.get(i).name == name {
                    return scope.get(i).ty
                }
                i = i - 1
            }
            s = s - 1
        }
        var gi = 0
        while gi < .globals.len() {
            if .globals.get(gi).name == name {
                return .globals.get(gi).ty
            }
            gi = gi + 1
        }
        return ""
    }

    // 类型/枚举注册表查询(D38b):name → 声明期 Display 快照
    fn decl_lookup(name str) str {
        var i = 0
        while i < .decls.len() {
            if .decls.get(i).name == name {
                return .decls.get(i).ty
            }
            i = i + 1
        }
        return ""
    }

    fn decl_register(name str, display str) {
        .decls.push(Binding(name, display))
    }
}"""
assert old_type in s, "type P block not found"
s = s.replace(old_type, new_type)

# 2. delete the now-method free fns (exact bodies from the original file)
def cut(src, header):
    i = src.find(header)
    assert i >= 0, header
    # find the end: next "\n}\n" at brace depth 0 — simple scan
    j = i
    depth = 0
    started = False
    while j < len(src):
        c = src[j]
        if c == '{':
            depth += 1; started = True
        elif c == '}':
            depth -= 1
            if started and depth == 0:
                break
        j += 1
    # consume trailing newlines
    k = j + 1
    while k < len(src) and src[k] == '\n':
        k += 1
    return src[:i] + src[k:]

for hdr in [
    "fn p_kind(p P) TokenKind {",
    "fn p_text(p P) str {",
    "fn p_line(p P) int {",
    "fn p_peek(p P, n int) TokenKind {",
    "fn p_next(mut p P) {",
    "fn p_err(mut p P, msg str) {",
    "fn p_expect(mut p P, k TokenKind) {",
    "fn skip_empty_lines(mut p P) int {",
    "fn push_scope(mut p P) {",
    "fn pop_scope(mut p P) {",
    "fn p_bind(mut p P, name str, ty str) {",
    "fn p_lookup(p P, name str) str {",
]:
    s = cut(s, hdr)

# p_decl_lookup / p_decl_register were BEFORE type P in the file — cut them too
for hdr in [
    "fn p_decl_lookup(p P, name str) str {",
    "fn p_decl_register(mut p P, name str, display str) {",
]:
    s = cut(s, hdr)

open(p, "w", encoding="utf-8", newline="\n").write(s)
print("parser.at type+deletes done")

# 3. flip call sites across all lib files
FLIPS = [
    (r'\bp_kind\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.kind()'),
    (r'\bp_text\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.text()'),
    (r'\bp_line\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.line()'),
    (r'\bp_next\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.next()'),
    (r'\bp_peek\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.peek('),
    (r'\bp_err\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.fail('),
    (r'\bp_expect\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.expect('),
    (r'\bskip_empty_lines\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.skip_empty_lines()'),
    (r'\bpush_scope\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.push_scope()'),
    (r'\bpop_scope\(([A-Za-z_][A-Za-z0-9_]*)\)', r'\1.pop_scope()'),
    (r'\bp_bind\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.bind('),
    (r'\bp_lookup\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.lookup('),
    (r'\bp_decl_lookup\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.decl_lookup('),
    (r'\bp_decl_register\(([A-Za-z_][A-Za-z0-9_]*),\s*', r'\1.decl_register('),
]
import glob
for f in glob.glob(W + "/auto/lib/*.at"):
    t = open(f, encoding="utf-8").read()
    orig = t
    for pat, rep in FLIPS:
        t = re.sub(pat, rep, t)
    if t != orig:
        open(f, "w", encoding="utf-8", newline="\n").write(t)
        print("flipped:", f.split("/")[-1])
print("done")
