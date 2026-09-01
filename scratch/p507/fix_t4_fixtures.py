# Plan 507 T10 —— align T4 fixtures with handler-owns-state Toggle semantics.
import io
import sys

p = 'crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs'
s = io.open(p, encoding='utf-8').read()

BSN = '\\' + 'n'  # backslash + n (2 chars) — matches Rust source escapes

old1 = ('"widget CB {' + BSN + '    model { var ok bool = false }' + BSN
        + '    view {' + BSN + '        checkbox (checked: .ok) { onclick: .Toggle }' + BSN
        + '    }' + BSN + '}' + BSN + '"')
new1 = ('"widget CB {' + BSN + '    model { var ok bool = false }' + BSN
        + '    view {' + BSN + '        checkbox (checked: .ok) {}' + BSN
        + '    }' + BSN + '}' + BSN + '"')
if old1 not in s:
    print('FAIL fixture1', file=sys.stderr)
    sys.exit(1)
s = s.replace(old1, new1)

old2 = ('    on { .T -> { .n += 1 } }')
new2 = ('    on { .T -> { .n += 1 .ok = !.ok } }')
if old2 not in s:
    print('FAIL fixture2', file=sys.stderr)
    sys.exit(1)
s = s.replace(old2, new2)

old3 = ('view { switch (checked: .on) { onclick: .Nope } }')
new3 = ('view { switch (checked: .on) {} }')
if old3 not in s:
    print('FAIL fixture3', file=sys.stderr)
    sys.exit(1)
s = s.replace(old3, new3)
s = s.replace('''        // 轨道 + 滑块（无 handler 名 .Nope 缺 msg？零参 token 仍登记——
        // 派发由 on_with_input 容错）。滑块 x 左侧。''',
'''        // 轨道 + 滑块（无 handler → 自动翻转路径）。滑块 x 左侧。''')

old4 = ('view { radio (checked: .pick) { onclick: .Nop } }')
new4 = ('view { radio (checked: .pick) {} }')
if old4 not in s:
    print('FAIL fixture4', file=sys.stderr)
    sys.exit(1)
s = s.replace(old4, new4)

with io.open(p, 'w', encoding='utf-8', newline='\n') as f:
    f.write(s)
print('fixtures aligned')
