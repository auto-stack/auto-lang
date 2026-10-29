#!/usr/bin/env bash
# measure-l2-873.sh — PLAN-873 T-03 L2 增量三控制站点驱动（M873 协议 §5/§6 的受管实现）
#
# 用法（在组内任意树执行均可，站点表内置三仓路径；建议在 auto-lang 树跑）:
#   bash scripts/measure-l2-873.sh transition <lang|ui|man>          # incr0→incr1 过渡重建（记录为 transition，不计样本）
#   bash scripts/measure-l2-873.sh baseline  <lang|ui|man> [N]       # 无改动热跑对照 ×N（默认3）
#   bash scripts/measure-l2-873.sh site      <site-id>   [N]        # 单站点电池：mtime×N + 函数体×N（每样本复位+核hash+restore构建）
#   bash scripts/measure-l2-873.sh list                               # 站点表
#
# 站点（M873 §6 配额：语言≥4 / UI≥2 / Man≥1，合计 7 个真实函数体站点）:
#   函数体变化 = 在锚点行（唯一、以 { 结尾的函数签名行）后插入一行
#   `    let _m873_probe = 0x873u32;`（纯内部语句，不改签名与公共 API）。
#
# 纪律（协议 §2/§3/§5/§6）:
#   - 每条 cargo 命令 env -u RUSTC_WRAPPER -u SCCACHE_DIR（共享 sccache 零接触）
#   - CARGO_INCREMENTAL=1（L2 默认；过渡重建单独记录）
#   - CARGO_TARGET_DIR 显式指 .measure/target-*（target 不落树内）
#   - 每条计时命令前 rustc-only 污染守卫：发现任何 rustc.exe 进程即中止（exit 98）
#     （cargo.exe 长驻 dev-server 按 T-02 裁定不算编译争抢，仅记上下文）
#   - 函数体样本前后：git diff 仅含预期站点（+1 行）；复位后 sha256 与 pin 态一致、树干净
#   - 日志 = .measure/logs/<sample-id>.log，头部含样本id/pin/控制类型/env/命令，尾部含墙钟/退出码/重编单元
set -uo pipefail

GROUP_DIR="D:/autostack/.wt/lang-873"
MEASURE="$GROUP_DIR/.measure"
LOGS="$MEASURE/logs"
LANG_DIR="$GROUP_DIR/auto-lang"
UI_DIR="$GROUP_DIR/auto-ui"
MAN_DIR="$GROUP_DIR/auto-man"
PROBE="let _m873_probe = 0x873u32;"

# 站点表: site-id | repo-dir | target-dir | cargo 参数 | 站点文件(仓内相对) | 锚点行(唯一)
site_table() {
  cat <<'EOF'
lang-parser|LANG|check -p auto-lang|crates/auto-lang-core/src/parser/expressions.rs|fn parse_path_expr(&mut self) -> AutoResult<Expr> {
lang-codegen|LANG|check -p auto-lang|crates/auto-lang-core/src/vm/codegen/expr_ops.rs|pub(super) fn expr_ops(&mut self, expr: &Expr) -> AutoResult<()> {
lang-engine|LANG|check -p auto-lang|crates/auto-lang-core/src/vm/engine/ops_arith.rs|) -> Result<StepResult, VMError> {
lang-trans|LANG|check -p auto-lang|crates/auto-lang-core/src/trans/python.rs|fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
ui-compiler|UI|check --workspace|crates/auto-ui-compiler/src/ui_gen/api.rs|pub fn transpile_aura(source: &str, output_path: Option<&str>) -> Result<String, String> {
ui-runtime|UI|check --workspace|crates/auto-ui-runtime/src/ui/iced/renderer.rs|fn change_line(&mut self, line: usize) {
man-auto|MAN|check --workspace|crates/auto/src/cmd_bp.rs|fn registry() -> BlueprintRegistry {
EOF
}

resolve_repo() { # LANG|UI|MAN -> 目录 target-dir
  case "$1" in
    LANG) echo "$LANG_DIR|$MEASURE/target-lang" ;;
    UI)   echo "$UI_DIR|$MEASURE/target-ui" ;;
    MAN)  echo "$MAN_DIR|$MEASURE/target-man" ;;
  esac
}

repo_args() { # 各仓 check 入口（与 T-02 L1 入口一致）
  case "$1" in
    LANG) echo "check -p auto-lang" ;;
    UI)   echo "check --workspace" ;;
    MAN)  echo "check --workspace" ;;
  esac
}

die() { echo "FATAL: $*" >&2; exit 1; }

foreign_rustc() { # 输出当前 rustc.exe 进程列表（无则空）
  powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='rustc.exe'\" | ForEach-Object { \"{0} {1}\" -f \$_.ProcessId, \$_.CommandLine }" 2>/dev/null
}

cargo_procs() { # 上下文：cargo.exe 进程
  powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='cargo.exe'\" | ForEach-Object { \"{0} {1}\" -f \$_.ProcessId, \$_.CommandLine }" 2>/dev/null
}

guard_pollution() { # 计时命令前置守卫
  local procs; procs="$(foreign_rustc)"
  if [ -n "$procs" ]; then
    { echo "pollution_detected=$(date '+%Y-%m-%d %H:%M:%S')"; echo "foreign_rustc:"; echo "$procs"; } >> "$LOGS/pollution-$(date '+%Y%m%d-%H%M').md"
    echo "POLLUTION: foreign rustc running, abort ($1)" >&2
    exit 98
  fi
}

load_pct() { powershell -NoProfile -Command "(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average" 2>/dev/null; }

sha_of() { sha256sum "$1" | cut -d' ' -f1; }

# run_check <sample-id> <repo-dir> <target-dir> <control-tag> <file-rel-or--> <cargo 参数...>
run_check() {
  local id="$1" repo="$2" target="$3" ctl="$4" filerel="$5"; shift 5
  LOG="$LOGS/$id.log"
  guard_pollution "$id"
  local fsha="-"
  [ "$filerel" != "-" ] && fsha="$(sha_of "$repo/$filerel")"
  {
    echo "sample_id=$id"
    echo "datetime=$(date '+%Y-%m-%d %H:%M:%S')"
    echo "repo=$repo"
    echo "pin=$(git -C "$repo" rev-parse HEAD 2>/dev/null)"
    echo "control=$ctl file=$filerel sha256=$fsha"
    echo "env: RUSTC_WRAPPER=<unset> CARGO_INCREMENTAL=1 CARGO_TARGET_DIR=$target"
    echo "load_pct_pre=$(load_pct)"
    echo "cargo_context: $(cargo_procs | tr '\n' ';')"
    echo "cmd: cargo $*"
    echo "start=$(date '+%H:%M:%S.%3N')"
  } > "$LOG"
  local start end wall rc
  start=$(date +%s.%N)
  ( cd "$repo" || exit 97
    exec env -u RUSTC_WRAPPER -u SCCACHE_DIR \
      CARGO_INCREMENTAL=1 CARGO_TARGET_DIR="$target" cargo "$@"
  ) >> "$LOG" 2>&1
  rc=$?
  end=$(date +%s.%N)
  wall=$(awk -v e="$end" -v s="$start" 'BEGIN{printf "%.1f", e-s}')
  local units
  units=$(grep -oE "(Checking|Compiling) [a-zA-Z0-9_-]+" "$LOG" | sort -u | sed 's/^[A-Za-z]* //' | tr '\n' ',' | sed 's/,$//')
  {
    echo "end=$(date '+%H:%M:%S.%3N')"
    echo "wall_seconds=$wall"
    echo "exit=$rc"
    echo "rebuilt_units=${units:-none}"
  } >> "$LOG"
  echo "[$id] ctl=$ctl wall=${wall}s exit=$rc units=${units:-none}"
  return $rc
}

assert_clean_tree() { # <repo-dir> <期望HEAD简写或'-'>
  local repo="$1" want="$2" st head
  st=$(git -C "$repo" status --porcelain)
  [ -z "$st" ] || die "$repo tree not clean: $st"
  head=$(git -C "$repo" rev-parse HEAD)
  if [ "$want" != "-" ]; then
    [ "$head" = "$want" ] || die "$repo HEAD $head != pin $want"
  fi
}

cmd_transition() { # incr0→incr1 过渡重建（不计时为样本）
  local rv; rv="$(resolve_repo "$1")"; local repo="${rv%%|*}" target="${rv#*|}"
  run_check "L2-$1-transition" "$repo" "$target" "transition" "-" $(repo_args "$1")
}

cmd_baseline() {
  local rv; rv="$(resolve_repo "$1")"; local repo="${rv%%|*}" target="${rv#*|}"
  local n="${2:-3}"
  assert_clean_tree "$repo" -
  local k
  for k in $(seq 1 "$n"); do
    run_check "L2-$1-base-S$k" "$repo" "$target" "nochange" "-" $(repo_args "$1")
  done
}

cmd_site() {
  local sid="$1" n="${2:-3}"
  local row; row="$(site_table | grep -m1 "^$sid|")" || die "unknown site $sid"
  local repo_key filerel anchor
  repo_key="$(echo "$row" | cut -d'|' -f2)"
  filerel="$(echo "$row" | cut -d'|' -f4)"
  anchor="$(echo "$row" | cut -d'|' -f5-)"
  local rv; rv="$(resolve_repo "$repo_key")"; local repo="${rv%%|*}" target="${rv#*|}"
  local pin_head; pin_head=$(git -C "$repo" rev-parse HEAD)
  assert_clean_tree "$repo" "$pin_head"
  local pin_sha; pin_sha="$(sha_of "$repo/$filerel")"
  echo "== site $sid: repo=$repo file=$filerel pin_sha=${pin_sha:0:12} =="
  local k
  for k in $(seq 1 "$n"); do
    # --- mtime 对照：仅改 mtime，内容 hash 必须不变 ---
    touch "$repo/$filerel"
    local s_after; s_after="$(sha_of "$repo/$filerel")"
    [ "$s_after" = "$pin_sha" ] || die "mtime control changed content?!"
    run_check "L2-$sid-mtime-S$k" "$repo" "$target" "mtime" "$filerel" $(repo_args "$repo_key") || die "mtime sample failed"
    # --- 函数体变化：锚点行后插入探针 ---
    awk -v anchor="$anchor" -v probe="$PROBE" '
      !ins && index($0, anchor) {
        print
        ind = $0; sub(/[^ ].*/, "", ind)
        print ind "    " probe
        ins = 1; next
      } { print }' "$repo/$filerel" > "$repo/$filerel.m873tmp"
    mv "$repo/$filerel.m873tmp" "$repo/$filerel"
    local diffstat
    diffstat=$(git -C "$repo" diff --numstat)
    [ "$diffstat" = "1	0	$filerel" ] || { git -C "$repo" checkout -- "$filerel"; die "probe diff unexpected: $diffstat"; }
    run_check "L2-$sid-body-S$k" "$repo" "$target" "body" "$filerel" $(repo_args "$repo_key") || { git -C "$repo" checkout -- "$filerel"; die "body sample failed"; }
    # --- 复位：checkout + 核 hash + 树干净，再 restore 构建回 pin 态（记录不计样本） ---
    git -C "$repo" checkout -- "$filerel"
    local s_reset; s_reset="$(sha_of "$repo/$filerel")"
    [ "$s_reset" = "$pin_sha" ] || die "reset sha mismatch"
    assert_clean_tree "$repo" "$pin_head"
    run_check "L2-$sid-restore-S$k" "$repo" "$target" "restore" "$filerel" $(repo_args "$repo_key") || die "restore check failed"
  done
  assert_clean_tree "$repo" "$pin_head"
  echo "== site $sid done, tree clean @${pin_head:0:9} =="
}

case "${1:-}" in
  transition) shift; cmd_transition "$@" ;;
  baseline)   shift; cmd_baseline "$@" ;;
  site)       shift; cmd_site "$@" ;;
  list)       site_table ;;
  *) sed -n '2,14p' "$0"; exit 64 ;;
esac
