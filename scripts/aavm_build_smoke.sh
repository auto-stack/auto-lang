#!/usr/bin/env bash
# Plan 523 W4-13:a2r 产品路径冒烟(auto build 的转译→构建→运行核心链)
#
# 形态:临时 cargo 工程 ← `auto trans`(主 a2r)→ cargo build → 运行 →
#       输出与 aavm2_bin(参考语义承载)对拍。语料抽验 b07/b34(计划口径)。
#
# 注(2026-09-03):`auto build`(pac)全管线在最小工程上的 rust target
# 生成缺口与 examples/api-example 前端 strict 校验存量红登记为债
# (KNOWN-DEBT P523-1);本脚本为其 a2r 核心链的常态化 CI 位。
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AUTO="$ROOT/target/release/auto.exe"
[ -f "$AUTO" ] || AUTO="$ROOT/target/release/auto"
SMOKE="$(mktemp -d)"
trap 'rm -rf "$SMOKE"' EXIT

run_case() {
  local name="$1" at="$2" expect="$3"
  local d="$SMOKE/$name"
  mkdir -p "$d/src"
  cp "$at" "$d/src/main.at"
  (cd "$d" && "$AUTO" trans --path src/main.at rust >/dev/null); mv "$d/src/main.a2r.rs" "$d/src/main.rs"
  cat > "$d/Cargo.toml" <<EOF
[package]
name = "smoke_$name"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
EOF
  (cd "$d" && cargo build --release >/dev/null 2>&1)
  local out
  out="$("$d/target/release/smoke_$name.exe")"
  if [ "$out" == "$expect" ]; then
    echo "SMOKE $name PASS (output == $(
      echo "$expect" | tr '\n' ' '))"
  else
    echo "SMOKE $name FAIL"
    echo "--- expect ---"; echo "$expect"
    echo "--- got ---";    echo "$out"
    exit 1
  fi
}

# b07_fib:期望 55
run_case b07_fib "$ROOT/crates/auto-lang/test/vm/aavm2/corpus_m4/b07_fib.at" "55"
# b34_struct_basic:期望 10/20(中阶 struct 族代表)
run_case b34_struct_basic "$ROOT/crates/auto-lang/test/vm/aavm2/corpus_m4/b34_struct_basic.at" "$(printf '10\n20')"
echo "aavm build smoke: all PASS"
