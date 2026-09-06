# P572 T2: 最小触发对的语句级形态归因
#
# 基底:{emit, emit_store} 二方法对(TIMEOUT 实证)。生成变体:
#   - emit_store 体逐语句删减(分支剪除/单语句化)
#   - emit 体替换(去 push/去 return 表达式)
#   - 参数形态变化(3参/1参)
# 每变体计时(timeout 短),输出矩阵 → 触发形态定位。

import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ladder import find_default_exe1, run_one

OUT = os.path.join(os.environ.get('TEMP', '/tmp'), 'p572_shapes')

FIELDS = '''pub type CG {
    ins List<I>
    n_args int
'''

EMIT_ORIG = '''
    fn emit(op OpCode, s str, n int) int {
    self.ins.push(I(op, s, n))
    return self.ins.len() - 1
}
'''

EMIT_BARE = '''
    fn emit(op OpCode, s str, n int) int {
    return 0
}
'''

EMIT_PUSH_ONLY = '''
    fn emit(op OpCode, s str, n int) int {
    self.ins.push(I(op, s, n))
    return 0
}
'''

EMIT_LEN_ONLY = '''
    fn emit(op OpCode, s str, n int) int {
    return self.ins.len() - 1
}
'''

EMIT_1PARAM = '''
    fn emit(s str) int {
    self.ins.push(I(OpCode.Line, s, 0))
    return self.ins.len() - 1
}
'''

STORE_ORIG = '''
    fn emit_store(idx int) {
    if idx < self.n_args {
        var sv = (128 + idx).str()
        self.emit(OpCode.StoreLocal, sv, 128 + idx)
        return
    }
    var rel = idx - self.n_args
    if rel == 0 {
        self.emit(OpCode.StoreLoc0, "0", 0)
    } else if rel == 1 {
        self.emit(OpCode.StoreLoc1, "1", 1)
    } else {
        var sv2 = rel.str()
        self.emit(OpCode.StoreLocal, sv2, rel)
    }
}
'''

STORE_FIRST_IF = '''
    fn emit_store(idx int) {
    if idx < self.n_args {
        var sv = (128 + idx).str()
        self.emit(OpCode.StoreLocal, sv, 128 + idx)
        return
    }
}
'''

STORE_ONE_CALL = '''
    fn emit_store(idx int) {
    self.emit(OpCode.StoreLocal, "0", 0)
}
'''

STORE_CALL_VARSTR = '''
    fn emit_store(idx int) {
    var sv = idx.str()
    self.emit(OpCode.StoreLocal, sv, idx)
}
'''

STORE_CALL_ARITH = '''
    fn emit_store(idx int) {
    self.emit(OpCode.StoreLocal, "0", 128 + idx)
}
'''

STORE_IF_ELSE_2CALL = '''
    fn emit_store(idx int) {
    if idx < self.n_args {
        self.emit(OpCode.StoreLocal, "0", 0)
    } else {
        self.emit(OpCode.StoreLoc0, "1", 1)
    }
}
'''

STORE_3SEQ_CALLS = '''
    fn emit_store(idx int) {
    self.emit(OpCode.StoreLocal, "0", 0)
    self.emit(OpCode.StoreLoc0, "1", 1)
    self.emit(OpCode.StoreLoc1, "2", 2)
}
'''

VARIANTS = {
    'orig':            (EMIT_ORIG, STORE_ORIG),
    'emit_bare':       (EMIT_BARE, STORE_ORIG),
    'emit_push_only':  (EMIT_PUSH_ONLY, STORE_ORIG),
    'emit_len_only':   (EMIT_LEN_ONLY, STORE_ORIG),
    'emit_1param':     (EMIT_1PARAM, STORE_ORIG),
    'store_first_if':  (EMIT_ORIG, STORE_FIRST_IF),
    'store_one_call':  (EMIT_ORIG, STORE_ONE_CALL),
    'store_call_var':  (EMIT_ORIG, STORE_CALL_VARSTR),
    'store_call_arith': (EMIT_ORIG, STORE_CALL_ARITH),
    'store_ifelse2':   (EMIT_ORIG, STORE_IF_ELSE_2CALL),
    'store_3seq':      (EMIT_ORIG, STORE_3SEQ_CALLS),
}


def main():
    os.makedirs(OUT, exist_ok=True)
    exe1 = find_default_exe1()
    timeout = float(sys.argv[1]) if len(sys.argv) > 1 else 10.0
    print(f'[shapes] exe1={exe1} timeout={timeout}s')
    for name, (emit_src, store_src) in VARIANTS.items():
        src = FIELDS + emit_src + store_src + '}\n'
        sp = os.path.join(OUT, f'{name}.at')
        open(sp, 'w', encoding='utf-8', newline='\n').write(src)
        dt, rc, outlen, err = run_one(exe1, sp, timeout)
        status = ('TIMEOUT' if rc is None else
                  (f'OK' if rc == 0 else f'EXIT{rc} {err[:60]}'))
        print(f'{name:<18} {dt:7.2f}s {status}')


if __name__ == '__main__':
    main()
