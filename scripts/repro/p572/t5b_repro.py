# P572 T5b: exe² 5 缺陷之 3 个 AA2R 发射缺口的最小复现器(TDD 红→绿)
#
# R1(E0308):process.args() 类型缺失 → argv.get(1) 入 &str 参无 .as_str()
# R2(E0382):非 Copy 结构体 ident 入 .push() 无 .clone()
# R3(E0423):IO.read_line() 发射点号形(mod IO 下 E0423)
#
# 用法:python scratch/p572/t5b_repro.py [exe1_path]
# 判定:每源输出检查修复后期望形;当前(未修复)全部落 BAD。
# 退出码:任一 BAD=1(红);全 GREEN=0。

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ladder import find_default_exe1, run_one

R1 = '''fn pick(s str) str {
    return s
}
fn main() {
    var argv = process.args()
    if argv.len() > 1 {
        print(pick(argv.get(1)))
        return
    }
}
'''

R2 = '''pub type P {
    x int
}
fn main() {
    var xs List<P> = List.new()
    var mc = P(1)
    xs.push(mc)
    print(mc.x)
}
'''

R3 = '''fn main() {
    var h = IO.read_line()
    print(h.parse_int())
}
'''

CHECKS = [
    ('R1_e0308_as_str', R1, '.as_str()', 'argv[(1) as usize].clone())'),
    ('R2_e0382_push_clone', R2, 'xs.push(mc.clone())', 'xs.push(mc)'),
    ('R3_e0423_io_colon', R3, 'IO::read_line()', 'IO.read_line()'),
]


def main():
    exe1 = sys.argv[1] if len(sys.argv) > 1 else find_default_exe1()
    outdir = os.path.join(os.environ.get('TEMP', '/tmp'), 'p572_t5b')
    os.makedirs(outdir, exist_ok=True)
    print(f'[t5b] exe1 = {exe1}')
    any_bad = False
    for name, src, want, bad in CHECKS:
        sp = os.path.join(outdir, name + '.at')
        open(sp, 'w', encoding='utf-8', newline='\n').write(src)
        dt, rc, outlen, err = run_one(exe1, sp, 30)
        out = ''
        if rc == 0:
            out = open(os.path.join(outdir, name + '.rs'), 'wb')
        if rc == 0:
            import subprocess
            r = subprocess.run([exe1, '--trans', sp], capture_output=True)
            out = r.stdout.decode('utf-8', 'replace')
        if rc != 0:
            print(f'{name:<22} EXIT{rc} {err[:60]}')
            any_bad = True
            continue
        has_want = want in out
        has_bad = bad in out and not has_want
        if has_want:
            print(f'{name:<22} GREEN(含期望形 {want!r})')
        elif has_bad:
            print(f'{name:<22} BAD(仍是缺陷形 {bad!r})')
            any_bad = True
        else:
            print(f'{name:<22} ??? 两者皆无(人工检查 {sp})')
            any_bad = True
    return 1 if any_bad else 0


if __name__ == '__main__':
    sys.exit(main())
