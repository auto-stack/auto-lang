#!/usr/bin/env bash
# assert-style.sh — PLAN-617 style/contract assertions for 030-video-player.
#
# Reusable, re-runnable checker behind AC-01 / AC-02 / AC-04 / AC-17.
# Run from the example root (examples/ui/030-video-player) or pass the dir.
#
# Usage:  bash tests/assert-style.sh [example_dir]
# Exit:   0 = all clean, 1 = violations found (printed), 2 = usage error
#
# Design note: this asserts on NON-COMMENT source lines only. The .at files
# are heavily commented, and comments legitimately mention the banned tokens
# (e.g. "去掉 backdrop-blur"), so a naive grep would false-positive.

set -uo pipefail

DIR="${1:-$(pwd)}"
FRONT="$DIR/src/front"
BACK="$DIR/src/back"

if [ ! -d "$FRONT" ]; then
  echo "error: $FRONT not found (pass the example dir)" >&2
  exit 2
fi

FAIL=0

# Strip // comments and /* */ blocks, then grep. Keeps assertions honest.
src_lines() {
  # shellcheck disable=SC2016
  sed -e 's://.*::' "$@" 2>/dev/null
}

check_absent() {
  local id="$1" pattern="$2" label="$3" scope="$4" flags="${5:--E}"
  local hits
  hits=$(find $scope -name '*.at' -print0 2>/dev/null \
         | xargs -0 -r -I{} sh -c 'sed -e "s://.*::" "{}"' 2>/dev/null \
         | grep -a -n "$flags" "$pattern" 2>/dev/null)
  if [ -n "$hits" ]; then
    local n
    n=$(printf '%s\n' "$hits" | wc -l | tr -d ' ')
    printf '%-8s %-34s FAIL  (%s hit%s)\n' "$id" "$label" "$n" "$([ "$n" = 1 ] || echo s)"
    printf '%s\n' "$hits" | head -6 | sed 's/^/           | /'
    FAIL=1
  else
    printf '%-8s %-34s ok\n' "$id" "$label"
  fi
}

check_count() {
  local id="$1" pattern="$2" label="$3" max="$4" scope="$5"
  local n
  n=$(find $scope -name '*.at' -print0 2>/dev/null \
      | xargs -0 -r -I{} sh -c 'sed -e "s://.*::" "{}"' 2>/dev/null \
      | grep -oE "$pattern" 2>/dev/null | wc -l | tr -d ' ')
  if [ "$n" -gt "$max" ]; then
    printf '%-8s %-34s FAIL  (%s > max %s)\n' "$id" "$label" "$n" "$max"
    FAIL=1
  else
    printf '%-8s %-34s ok    (%s <= %s)\n' "$id" "$label" "$n" "$max"
  fi
}

echo "=== PLAN-617 style assertions on: $DIR ==="
echo
echo "-- AC-01 flat visual (VM-unsupported / demo-ish classes must be gone) --"
check_absent AC-01 'backdrop-'              'backdrop-*'            "$FRONT"
check_absent AC-01 'shadow-'                'shadow-*'              "$FRONT"
check_absent AC-01 'rounded-(xl|2xl|3xl)'   'rounded-xl/2xl/3xl'    "$FRONT"
check_absent AC-01 'animate-'               'animate-*'             "$FRONT"
check_absent AC-01 'hover:scale'            'hover:scale-*'         "$FRONT"
check_absent AC-01 'flex-wrap'              'flex-wrap'             "$FRONT"
check_absent AC-01 '(from|via|to)-[a-z]+-'  'gradients from/via/to'  "$FRONT"

echo
echo "-- AC-02 button discipline --"
check_count  AC-02 'bg-primary' 'bg-primary occurrences' 4 "$FRONT"

echo
echo "-- AC-04 no emoji in UI strings --"
# Emoji / pictograph / dingbat ranges. Covers the ones actually in use today.
# Needs PCRE2 (-P): the \x{...} escapes are not ERE. ugrep/GNU grep both support it.
check_absent AC-04 '[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}\x{2B00}-\x{2BFF}\x{FE0F}]' \
                   'emoji / pictographs' "$FRONT" -P

echo
echo "-- AC-17 no fabricated media metadata --"
check_absent AC-17 '(resolution|codec|bitrate)\s*:' 'fabricated resolution/codec/bitrate' "$BACK"

echo
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS (all style/contract assertions clean)"
  exit 0
else
  echo "RESULT: FAIL (violations above) — expected BEFORE the rewrite (T-01 baseline)"
  exit 1
fi
