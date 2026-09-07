# P572 T1: AA2R 转译超线性复现器(阶梯形态)
#
# 用途:重建 P532 会话(2026-09-06)的最小化阶梯——切 codegen.at L82-819
# 的 type CG 完整区,重建 `type CG { fields + static new + 前 N 方法 }` 源,
# 对每个 N 用 exe¹(aavm2_bin.exe)--trans 计时。
#
# 悬崖锚点(P532 实测):N=27 秒级 / N=28(加入 emit_store)>7min 不完。
#
# 用法:
#   python scratch/p572/ladder.py <exe1_path> [--n-list 26,27,28] [--timeout 90]
#     [--out DIR] [--keep]
#   exe1_path 缺省取 %TEMP% 下最新 aavm2-bin-*/target/release/aavm2_bin.exe
#   --n-list   逗号分隔的 N 集合(默认 1..28)
#   --timeout  单次转译秒上限,超时记 TIMEOUT 并杀进程(默认 90)
#   --out      源/产物落盘目录(默认 %TEMP%/p572_ladder)
#   --keep     保留每次转译产物 .rs(默认只留计时)
#
# 退出码:所有 N 均在 timeout 内完成=0;任一 TIMEOUT=3(即超线性红)。

import argparse
import glob
import os
import re
import subprocess
import sys
import time

CODEGEN_AT = 'auto/lib/codegen.at'

# type CG 方法起始行(2026-09-06 快照,worktree 校验防漂移;行号 1-based)
# starts[0]=static new(168);starts[1..28]=前 28 方法,第 28 个=emit_store(677)
STARTS = [168, 194, 209, 222, 230, 242, 254, 271, 291, 302, 314, 356, 376, 398,
          415, 428, 435, 444, 467, 493, 512, 528, 563, 591, 608, 625, 643, 659,
          677, 694, 715, 729, 737, 773]
TYPE_OPEN = 82   # pub type CG {
REGION_END = 833  # type CG 体末(cg_items_has@834 之前的闭合区)

METHOD_NAMES = ['new(static)', 'import_symbol', 'is_module', 'gkey', 'is_global',
                'tys_lookup', 'field_idx', 'field_ty', 'const_lookup', 'ctor_lookup',
                'link_symbol', 'loop_enter', 'loop_exit', 'loop_jump', 'fail',
                'emit', 'line', 'push_scope', 'pop_scope', 'max_alive_idx',
                'pop_scope_silent', 'add_var_reuse', 'add_var', 'var_arr',
                'var_ty', 'var_ty2', 'var_gtor', 'lookup', 'emit_store']


def find_default_exe1():
    tmp = os.environ.get('TEMP', '/tmp')
    cands = sorted(
        glob.glob(os.path.join(tmp, 'aavm2-bin-*', 'target', 'release',
                               'aavm2_bin.exe')),
        key=os.path.getmtime, reverse=True)
    return cands[0] if cands else None


def load_starts(root):
    """从真实 codegen.at 重算方法起始行,与 STARTS 快照比对防漂移。"""
    lines = open(os.path.join(root, CODEGEN_AT), encoding='utf-8').read().split('\n')
    starts = []
    for i, l in enumerate(lines[TYPE_OPEN - 1:REGION_END], start=TYPE_OPEN):
        if re.match(r'^    (static )?fn ', l):
            starts.append(i)
    if starts != STARTS:
        print(f'[ladder] codegen.at 方法布局漂移:快照 {STARTS}\n'
              f'[ladder] 实际 {starts}\n'
              f'[ladder] (修复后行号变动属预期——按实际值更新 STARTS 快照即可)')
    return lines, starts


def build_n_source(lines, starts, n, tyname='CG'):
    """重建 `type <tyname> { fields + static new + 前 n 方法 }` 源。"""
    body = lines[TYPE_OPEN - 1:starts[1] - 1]      # fields
    body += lines[starts[0] - 1:starts[1] - 1]      # static new
    stop = REGION_END + 1
    for k in range(1, n + 1):
        end = starts[k + 1] if k + 1 < len(starts) else stop
        body += lines[starts[k] - 1:end - 1]
    if tyname != 'CG':
        body = [re.sub(r'\bCG\b', tyname, l) for l in body]
        body[0] = body[0].replace('pub type CG', f'pub type {tyname}')
    return '\n'.join(body) + '\n}\n'


def run_one(exe1, src_path, timeout):
    t0 = time.time()
    try:
        p = subprocess.run([exe1, '--trans', src_path], capture_output=True,
                           timeout=timeout)
        dt = time.time() - t0
        return dt, p.returncode, len(p.stdout), p.stderr[:200].decode('utf-8', 'replace')
    except subprocess.TimeoutExpired:
        return time.time() - t0, None, 0, 'TIMEOUT'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe1', nargs='?', default=None)
    ap.add_argument('--n-list', default=','.join(str(i) for i in range(1, 29)))
    ap.add_argument('--timeout', type=float, default=90)
    ap.add_argument('--out', default=None)
    ap.add_argument('--keep', action='store_true')
    ap.add_argument('--tyname', default='CG')
    ap.add_argument('--file', default=None,
                    help='非阶梯模式:对给定 .at 文件单次 --trans 计时')
    args = ap.parse_args()

    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    exe1 = args.exe1 or find_default_exe1()
    if not exe1 or not os.path.isfile(exe1):
        print('[ladder] exe1 not found (aavm2_bin.exe); run leg-5 first')
        return 2
    outdir = args.out or os.path.join(os.environ.get('TEMP', '/tmp'), 'p572_ladder')
    os.makedirs(outdir, exist_ok=True)

    if args.file:
        dt, rc, outlen, err = run_one(exe1, args.file, args.timeout)
        status = 'OK' if rc == 0 else ('TIMEOUT' if rc is None else f'EXIT{rc}:{err}')
        print(f'{os.path.basename(args.file):<24} {os.path.getsize(args.file)//1024:>7}KB '
              f'{dt:>9.2f}s  {status}  out={outlen}B')
        return 0 if rc == 0 else 3

    lines, starts = load_starts(root)
    print(f'[ladder] exe1 = {exe1}')
    print(f'[ladder] out = {outdir}  timeout = {args.timeout}s')
    print(f'{"N":>3} {"last_method":<16} {"src_KB":>7} {"sec":>9}  status')
    any_timeout = False
    for n in (int(x) for x in args.n_list.split(',')):
        src = build_n_source(lines, starts, n, args.tyname)
        sp = os.path.join(outdir, f'n{n}.at')
        open(sp, 'w', encoding='utf-8', newline='\n').write(src)
        dt, rc, outlen, err = run_one(exe1, sp, args.timeout)
        status = 'OK' if rc == 0 else ('TIMEOUT' if rc is None else f'EXIT{rc}:{err}')
        if rc is None:
            any_timeout = True
        print(f'{n:>3} {METHOD_NAMES[n]:<16} {len(src)//1024:>7} {dt:>9.2f}  {status}')
        if args.keep and rc == 0:
            open(sp[:-3] + '.rs', 'wb').write(
                subprocess.run([exe1, '--trans', sp], capture_output=True).stdout)
    return 3 if any_timeout else 0


if __name__ == '__main__':
    sys.exit(main())
