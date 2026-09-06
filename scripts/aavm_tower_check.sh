#!/usr/bin/env bash
# P532 W2:塔顶自持 runner(附录 B 设计定案)
#
# 形态:三阶自持链对拍宿主 oracle——
#   R0 = auto tower1.at                (宿主直跑,oracle)
#   O1 = auto auto/aavm.at tower1.at   (aavm 编译 corpus 集合)
#   O2 = auto auto/aavm.at tower2.at   (aavm 编译 tower1(含 corpus))
#   O3 = auto auto/aavm.at tower3.at   (aavm 编译 tower2(嵌tower1(嵌corpus)))
# 判据:R0 == O1 == O2 == O3(逐字节);首个分歧位报告。
# 用法:bash scripts/aavm_tower_check.sh [--keep](--keep 保留中间产物)
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# 二进制解析序:env 覆盖 → 本检出 → 主检出(镜像仓内跨仓依赖序;
# 塔顶只需宿主 VM 语义,lib 一律取 $ROOT 的)
AUTO="${AUTO_LANG_BIN:-}"
[ -n "$AUTO" ] && [ -f "$AUTO" ] || AUTO="$ROOT/target/release/auto.exe"
[ -f "$AUTO" ] || AUTO="$ROOT/target/release/auto"
[ -f "$AUTO" ] || AUTO="D:/autostack/auto-lang/target/release/auto.exe"
[ -f "$AUTO" ] || AUTO="D:/autostack/auto-lang/target/release/auto"
TOWER="$ROOT/crates/auto-lang/test/vm/aavm2/tower"
TMP="$(mktemp -d)"
trap '[ "${1:-}" = "--keep" ] || rm -rf "$TMP"' EXIT

# 宿主模块路径存在存量非确定性(~5-10% 概率静默空输出,pre-P532,
# 宿主侧 HashMap 迭代序族——KNOWN-DEBT 候选登记):每阶重试至非空
# (上限 6 次),空输出不参与判定;重试次数留档。
run_stage() {
    local name="$1"; shift
    local out="$TMP/$name.txt"
    local tries=0
    while [ $tries -lt 6 ]; do
        tries=$((tries+1))
        ( cd "$ROOT" && "$AUTO" "$@" ) > "$out" 2>&1
        # 空输出(仅 banner)= 疑似宿主侧非确定性静默失败,重试
        if [ -s "$out" ] && grep -q "== b01_hello" "$out"; then
            echo "[tower] $name ok (tries=$tries)"
            return 0
        fi
    done
    echo "[tower] $name EMPTY after $tries tries (host nondeterminism?)"
    return 1
}

echo "[tower] R0 (host oracle)..."
run_stage R0 "$TOWER/tower1.at" || exit 1
echo "[tower] O1 (stage-1 self-host)..."
run_stage O1 "$ROOT/auto/aavm.at" "$TOWER/tower1.at" || exit 1
echo "[tower] O2 (stage-2)..."
run_stage O2 "$ROOT/auto/aavm.at" "$TOWER/tower2.at" || exit 1
echo "[tower] O3 (stage-3)..."
run_stage O3 "$ROOT/auto/aavm.at" "$TOWER/tower3.at" || exit 1

fail=0
for st in O1 O2 O3; do
    if diff -u "$TMP/R0.txt" "$TMP/$st.txt" > "$TMP/diff_$st.txt" 2>&1; then
        echo "[tower] $st == R0  PASS"
    else
        echo "[tower] $st != R0  FAIL (first divergence:)"
        head -12 "$TMP/diff_$st.txt"
        fail=1
    fi
done
if [ "$fail" = "0" ]; then
    echo "[tower] ALL STAGES PASS — self-host chain closed (R0==O1==O2==O3)"
else
    echo "[tower] FAILURES above; intermediates: $TMP (rerun with --keep)"
fi
exit $fail
