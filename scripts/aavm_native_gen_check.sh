#!/usr/bin/env bash
# P532 W3(附录 B 修订,2026-09-04 用户裁定):原生代际对拍 runner。
# 步骤 7(一代对拍判定表)+ 步骤 9(自编译代际对拍)的判定载体。
#
# 形态(零解释器嵌套,每代原生二进制):
#   exe¹ = a2r(aavm.at+lib):⑤腿 build_aavm_rust_bin 形态(宿主 merged
#         转译剥 use 平铺目录 + prelude/shims/harness → cargo build),
#         内容寻址缓存于 %TEMP%/aavm2-bin-<hash>-*/
#   一代判据:exe¹ 跑 corpus 代表集输出 == 宿主 auto.exe oracle
#            (⑤腿 test_aavm2_compile_corpus 已证 58/58 全量;本表固化
#            附录 B 代表集八件的逐件判定留档)
#   exe² = cargo(exe¹ --trans 转译的 aavm.at+lib 拼合源 + shims):
#            "exe¹ 编译 aavm.at+lib → a2r → exe" 的产品化形态
#   二代判据:exe² 跑代表集 == exe¹;exe² --trans 输出 == exe¹ --trans
#            (自编译转译固定点,≥两代一致即自举闭合)
#
# 判定表落盘:scratch/p532/native_gen_table.md(stdout 同步打印)
#
# 用法:bash scripts/aavm_native_gen_check.sh [--skip-gen1] [--skip-gen2]
#       --skip-gen1:跳过⑤腿触发,直接用缓存最新 exe¹(⑤腿已跑过的快路径)
#       --skip-gen2:跳过 exe¹ --trans 慢路径(解释执行拼合源,小时级),
#         直接用 $TEMP/p532_gen2_main.rs 既有产物跑二代判定段
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
TABLE="$ROOT/scratch/p532/native_gen_table.md"
mkdir -p "$(dirname "$TABLE")"
trap 'rm -rf "$TMP"' EXIT

AUTO="$ROOT/target/release/auto.exe"
[ -f "$AUTO" ] || AUTO="$ROOT/target/release/auto"
[ -f "$AUTO" ] || { echo "[gen] host auto.exe not found (cargo build --release first)"; exit 2; }

CORPUS="$ROOT/crates/auto-lang/test/vm/aavm2/corpus_m4"
# 附录 B 代表集(tower{1,2,3}.at 同款八件)
REP="b01_hello b07_fib b08_strcat b13_eval_print_true b27_arr_literal b30_arr_loop b46_list_basic b58_str_methods"

# ── exe¹:⑤腿原生二进制 ──────────────────────────────────────────
if [ "${1:-}" != "--skip-gen1" ]; then
    echo "[gen] triggering leg-5 build (test_aavm2_compile_corpus; cached if fresh)..."
    (cd "$ROOT" && cargo test -p auto-lang --lib --features test-vm-files \
        test_aavm2_compile_corpus -- --test-threads=1 >/dev/null 2>&1) \
        || { echo "[gen] leg-5 test FAILED (see cargo output)"; exit 1; }
fi
# mktemp -d 已占 TMP;直接在系统临时目录里找最新的 aavm2_bin
# P572 T5b:exe¹ 选取改按 mtime 最新(⑤腿内容寻址缓存可能同时存在多个
# 新于 codegen.at 的目录——旧 find|head-1 按目录遍历序取首个,不确定,
# 曾取到修复前旧版致固定点误判;ls -t 确定取最新构建)。
SYS_TMP="$(dirname "$TMP")"
EXE1="$(find "$SYS_TMP" -maxdepth 4 -path '*aavm2-bin-*' -name 'aavm2_bin.exe' -newer "$ROOT/auto/lib/codegen.at" 2>/dev/null | xargs ls -t 2>/dev/null | head -1)"
[ -n "$EXE1" ] || EXE1="$(find "$SYS_TMP" -maxdepth 4 -path '*aavm2-bin-*' -name 'aavm2_bin.exe' 2>/dev/null | xargs ls -t 2>/dev/null | head -1)"
[ -n "$EXE1" ] && [ -f "$EXE1" ] || { echo "[gen] exe1 (aavm2_bin) not found — run leg-5 first"; exit 2; }
echo "[gen] exe1 = $EXE1"

# ── 一代判定表(步骤 7)──────────────────────────────────────────
echo "" > "$TABLE"
echo "# P532 原生代际对拍判定表($(date '+%F %T'))" >> "$TABLE"
echo "" >> "$TABLE"
echo "exe¹: $EXE1" >> "$TABLE"
echo "" >> "$TABLE"
echo "## 一代(exe¹ vs 宿主 oracle,代表集)" >> "$TABLE"
echo "| corpus | exe¹==host |" >> "$TABLE"
echo "|---|---|" >> "$TABLE"
gen1_fail=0
for c in $REP; do
    f="$CORPUS/$c.at"
    [ -f "$f" ] || { echo "| $c | MISSING |" >> "$TABLE"; gen1_fail=1; continue; }
    # 宿主 CLI 输出剥 banner(前 3 行 -----/Running/-----;⑤腿参考
    # run_with_capture 无 banner,同口径)
    (cd "$ROOT" && "$AUTO" "$f" 2>&1) | tail -n +4 | sed -e 's/[ 	]*$//' | grep -v '^$' > "$TMP/host_$c.txt"
    "$EXE1" "$f" 2>&1 | sed -e 's/[ 	]*$//' | grep -v '^$' > "$TMP/g1_$c.txt"
    # ⑤腿 trim_end 同口径:双侧剥尾空白/空行后逐行比
    if diff -q "$TMP/host_$c.txt" "$TMP/g1_$c.txt" >/dev/null 2>&1; then
        echo "| $c | PASS |" >> "$TABLE"; echo "[gen1] $c PASS"
    else
        echo "| $c | FAIL |" >> "$TABLE"; echo "[gen1] $c FAIL"
        diff "$TMP/host_$c.txt" "$TMP/g1_$c.txt" | head -5
        gen1_fail=1
    fi
done
[ $gen1_fail -eq 0 ] && echo "一代代表集:8/8 PASS" >> "$TABLE" || echo "一代代表集:存在 FAIL" >> "$TABLE"

# ── exe²:exe¹ 转译 lib → cargo ─────────────────────────────────
echo "[gen] building exe2 (exe1 --trans self-compile)..."
# P572 T5b:拼合源改为 lib 七文件(剥 use)——aavm.at 不入 exe² 构建输入。
# 原因:aavm.at 全部内容即 CLI main,拼入则与 harness 追加的 main 重复
# (E0428;⑤腿 exe¹ 同款=lib-only+harness main,此处镜像);aavm.at 的
# IO.read_line 入口面由 aavm_at_mode 测试(531 形态,宿主)覆盖。转译
# 固定点用同一拼合源(exe² 体==exe¹ --trans(本文件)的自再现闭环)。
python - "$ROOT" "$TMP" <<'PYEOF'
import sys, os
root, tmp = sys.argv[1], sys.argv[2]
files = ['token','lexer','parser','typeinfo','codegen','engine','a2r']
out = []
for f in files:
    for line in open(os.path.join(root, 'auto/lib', f + '.at'), encoding='utf-8'):
        if line.lstrip().startswith('use auto.lib.'):
            continue
        out.append(line)
    out.append('\n')
open(os.path.join(tmp, 'concat.at'), 'w', encoding='utf-8', newline='\n').write(''.join(out))
PYEOF
if [ "${2:-}" = "--skip-gen2" ] && [ -s "$TEMP/p532_gen2_main.rs" ]; then
    cp "$TEMP/p532_gen2_main.rs" "$TMP/gen2_body.rs"
    echo "[gen] reusing $TEMP/p532_gen2_main.rs (skip-gen2)"
else
"$EXE1" --trans "$TMP/concat.at" > "$TMP/gen2_body.rs" 2> "$TMP/gen2_err.txt" \
    || { echo "[gen] exe1 --trans FAILED:"; head -3 "$TMP/gen2_err.txt"; exit 1; }
fi
[ -s "$TMP/gen2_body.rs" ] || { echo "[gen] exe1 --trans EMPTY output"; exit 1; }
echo "[gen] transpiled body: $(wc -c < "$TMP/gen2_body.rs") bytes"

# prelude/shims(镜像 ⑤腿 vm_file_tests + 531 aavm_at_mode_tests 形态)+ harness
cat > "$TMP/gen2_prelude.rs" <<'RS'
#[allow(dead_code)]
pub struct File;
#[allow(dead_code)]
impl File {
    pub fn read_text(p: impl AsRef<std::path::Path>) -> String {
        std::fs::read_to_string(p).unwrap_or_default()
    }
}
#[allow(dead_code)]
struct ProcessShim;
#[allow(dead_code)]
impl ProcessShim {
    pub fn args(&self) -> Vec<String> { std::env::args().collect() }
}
#[allow(dead_code)]
const process: ProcessShim = ProcessShim;
#[allow(dead_code)]
mod IO {
    pub fn read_line() -> String {
        let mut buf = String::new();
        match std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut buf) {
            Ok(_) => buf.trim_end_matches('\n').trim_end_matches('\r').to_string(),
            Err(_) => String::new(),
        }
    }
}
#[allow(dead_code)]
trait A2rParseInt { fn parse_int(&self) -> i64; }
impl A2rParseInt for String {
    fn parse_int(&self) -> i64 { self.trim().parse::<i64>().unwrap_or(0) }
}
#[allow(dead_code)]
mod a2r_std {
    pub fn value_len<T>(v: &Vec<T>) -> i64 { v.len() as i64 }
}
RS
cat > "$TMP/gen2_harness.rs" <<'RS'
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // P572 T5b:三模式完整镜像 ⑤腿 exe¹ harness(--files/--trans/单文件)。
    if args.len() >= 3 && &args[1] == "--files" {
        let out = ev_run_files(&args[2]);
        print!("{}", out);
        return;
    }
    if args.len() >= 3 && &args[1] == "--trans" {
        let source = match std::fs::read_to_string(&args[2]) {
            Ok(s) => s,
            Err(e) => { eprintln!("read error: {}", e); std::process::exit(2); }
        };
        let out = ar_run(&source, 0);
        print!("{}", out);
        return;
    }
    if args.len() < 2 {
        eprintln!("usage: aavm2 <file.at>");
        std::process::exit(2);
    }
    let source = match std::fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => { eprintln!("read error: {}", e); std::process::exit(2); }
    };
    let out = ev_run(&source);
    print!("{}", out);
}
RS
PROJ="$TMP/gen2proj"
mkdir -p "$PROJ/src"
printf '[package]\nname = "aavm2_gen2"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n\n[dependencies]\n' > "$PROJ/Cargo.toml"
cat "$TMP/gen2_body.rs" "$TMP/gen2_prelude.rs" "$TMP/gen2_harness.rs" > "$PROJ/src/main.rs"
(cd "$PROJ" && cargo build --release) > "$TMP/gen2_build.log" 2>&1 \
    || { echo "[gen] exe2 cargo build FAILED:"; tail -20 "$TMP/gen2_build.log"; exit 1; }
EXE2="$PROJ/target/release/aavm2_gen2.exe"
echo "[gen] exe2 = $EXE2"

# ── 二代判定(步骤 9)────────────────────────────────────────────
echo "" >> "$TABLE"
echo "## 二代(exe² vs exe¹,代表集 + 转译固定点)" >> "$TABLE"
echo "| corpus | exe²==exe¹ |" >> "$TABLE"
echo "|---|---|" >> "$TABLE"
gen2_fail=0
for c in $REP; do
    f="$CORPUS/$c.at"
    [ -f "$f" ] || continue
    "$EXE2" "$f" 2>&1 | sed -e 's/[ 	]*$//' | grep -v '^$' > "$TMP/g2_$c.txt"
    if diff -q "$TMP/g1_$c.txt" "$TMP/g2_$c.txt" >/dev/null 2>&1; then
        echo "| $c | PASS |" >> "$TABLE"; echo "[gen2] $c PASS"
    else
        echo "| $c | FAIL |" >> "$TABLE"; echo "[gen2] $c FAIL"
        diff "$TMP/g1_$c.txt" "$TMP/g2_$c.txt" | head -5
        gen2_fail=1
    fi
done
# 转译固定点:exe² --trans == exe¹ --trans(同一拼合源)
"$EXE2" --trans "$TMP/concat.at" > "$TMP/gen2_trans2.rs" 2>&1
if diff -q "$TMP/gen2_body.rs" "$TMP/gen2_trans2.rs" >/dev/null 2>&1; then
    echo "" >> "$TABLE"; echo "转译固定点(exe¹ --trans == exe² --trans):PASS" >> "$TABLE"
    echo "[gen2] transpile fixed-point PASS"
else
    echo "" >> "$TABLE"; echo "转译固定点:FAIL(首分歧见下)" >> "$TABLE"
    echo "[gen2] transpile fixed-point FAIL:"
    diff "$TMP/gen2_body.rs" "$TMP/gen2_trans2.rs" | head -10
    gen2_fail=1
fi

echo "" >> "$TABLE"
echo "结论:一代 $([ $gen1_fail -eq 0 ] && echo PASS || echo FAIL) / 二代 $([ $gen2_fail -eq 0 ] && echo PASS || echo FAIL)(附录 B 原生代际形态;判定表完)" >> "$TABLE"
echo "[gen] table -> $TABLE"
[ $gen1_fail -eq 0 ] && [ $gen2_fail -eq 0 ] && { echo "[gen] ALL PASS"; exit 0; } || exit 1
