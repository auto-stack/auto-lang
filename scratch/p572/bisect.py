# P572 T2: 最小触发集二分
#
# 阶段1(leave-one-out):基底=28 方法全集(引爆);对 i=1..27 逐一剔除,
#   剔除后不再引爆(FAST)的方法 = 必要成分。
# 阶段2(贪心收缩):从 {必要成分 ∪ emit_store} 出发,逐一再试剔除
#   (全集中必要的成分在小集中可能冗余),直到不可再减 → 最小引爆集。
#
# 用法:python scratch/p572/bisect.py [--timeout 20] [--phase 1|2|all]

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ladder import (CODEGEN_AT, METHOD_NAMES, TYPE_OPEN, REGION_END, STARTS,
                    find_default_exe1, load_starts, run_one)


def build_skip_source(lines, starts, skip):
    """fields + static new + 方法 {1..28} \ skip(1-based 方法号集合)。"""
    body = lines[TYPE_OPEN - 1:starts[1] - 1]
    body += lines[starts[0] - 1:starts[1] - 1]
    stop = REGION_END + 1
    for k in range(1, 29):
        if k in skip:
            continue
        end = starts[k + 1] if k + 1 < len(starts) else stop
        body += lines[starts[k] - 1:end - 1]
    return '\n'.join(body) + '\n}\n'


def explodes(exe1, lines, starts, skip, outdir, tag, timeout):
    sp = os.path.join(outdir, f'bis_{tag}.at')
    open(sp, 'w', encoding='utf-8', newline='\n').write(
        build_skip_source(lines, starts, skip))
    dt, rc, _, err = run_one(exe1, sp, timeout)
    return rc is None, dt, rc


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--timeout', type=float, default=20)
    ap.add_argument('--phase', default='all')
    args = ap.parse_args()

    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    exe1 = find_default_exe1()
    outdir = os.path.join(os.environ.get('TEMP', '/tmp'), 'p572_ladder')
    os.makedirs(outdir, exist_ok=True)
    lines, starts = load_starts(root)
    print(f'[bisect] exe1 = {exe1}  timeout = {args.timeout}s')

    necessary = set()
    if args.phase in ('1', 'all'):
        print('== 阶段1:28 方法基底逐一减一(i=1..27) ==')
        for i in range(1, 28):
            e, dt, rc = explodes(exe1, lines, starts, {i}, outdir,
                                 f'loo{i}', args.timeout)
            tag = 'EXPLODES' if e else f'fast({rc})'
            mark = '' if e else '  ← 剔除即修复 → 必要'
            print(f'  -m{i:<2} {METHOD_NAMES[i]:<16} {dt:7.2f}s {tag}{mark}')
            if not e:
                necessary.add(i)
        print(f'阶段1 必要成分(剔除即修复): '
              f'{sorted(necessary)} = {[METHOD_NAMES[i] for i in sorted(necessary)]}')

    if args.phase in ('2', 'all'):
        if args.phase == '2' and not necessary:
            necessary = {int(x) for x in
                         (os.environ.get('P572_NECESSARY') or '').split(',') if x}
        print('== 阶段2:贪心收缩(最小集再逐一试剔) ==')
        cur = set(necessary) | {28}
        e, dt, _ = explodes(exe1, lines, starts, set(range(1, 29)) - cur,
                            outdir, 'seed', args.timeout)
        print(f'  种子集(必要∪emit_store)={sorted(cur)}: '
              f'{"EXPLODES" if e else f"NOT exploding ({dt:.2f}s)"}')
        if not e:
            print('  种子集已不引爆——必要成分在全集语境下的交互才是触发面;'
                  '最小集需从全集方向收缩,改用阶段1结果+回加法')
            return 1
        changed = True
        while changed:
            changed = False
            for i in sorted(cur - {28}):
                trial = cur - {i}
                e, dt, rc = explodes(exe1, lines, starts,
                                     set(range(1, 29)) - trial,
                                     outdir, 'shrink', args.timeout)
                if e:
                    print(f'  -m{i:<2} {METHOD_NAMES[i]:<16} 可剔(仍引爆)')
                    cur = trial
                    changed = True
                else:
                    print(f'  -m{i:<2} {METHOD_NAMES[i]:<16} 必留({dt:.2f}s)')
        print(f'最小引爆集 = {sorted(cur)} = {[METHOD_NAMES[i] for i in sorted(cur)]}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
