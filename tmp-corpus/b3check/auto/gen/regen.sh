#!/bin/bash
# Regen + deploy for auto-os-config Auto sources (Plan 006; jade-garden
# front/auto/gen/regen.sh pattern, simplified: no gen-side typecheck — the
# authoritative gate is the host `npm run build` (vue-tsc + vite) + e2e).
# Lives in gitignored auto/gen/. Usage: bash auto/gen/regen.sh
set -e
cd "$(dirname "$0")/.."   # -> auto/
AUTO=${AUTO:-D:/autostack/auto-lang/target/debug/auto.exe}
shopt -s nullglob

mkdir -p gen
# Clean generated components/stores first: `auto build` does NOT remove
# outputs of deleted widgets — without this, stale SFCs from removed .at
# files keep getting re-deployed (bit us with ProbeB.vue after Phase 5).
rm -rf gen/front/vue/src/components gen/front/vue/src/stores
"$AUTO" build -d . --gen-only 2>&1 | tee gen/build.log
if grep -q "Warning: Failed to compile" gen/build.log; then
  echo "!!! BUILD HAD COMPILE WARNINGS — aborting deploy" >&2
  exit 1
fi
# Parse errors print differently (and --gen-only still exits 0) — grep both.
if grep -qE "Parse error|error\[|Error:" gen/build.log; then
  echo "!!! BUILD HAD PARSE/COMPILE ERRORS — aborting deploy" >&2
  exit 1
fi

# Deploy store composables: rewrite the codegen's imports to the host tree —
# @/lib/api -> handwritten transport; @/ext/... -> the auto tree (stores sit
# one level deeper than components: ../../../ = repo root).
mkdir -p ../src/stores/auto
for f in gen/front/vue/src/stores/use*Store.ts; do
  base=$(basename "$f")
  sed -e "s|@/lib/api|../../lib/api|g" \
      -e "s|@/ext/src/front/utils/|../../../auto/src/front/utils/|g" \
      -e "s|@/ext/src/lib/api|../../../src/lib/api|g" \
      "$f" > "../src/stores/auto/${base}"
done

# Deploy components: rewrite the codegen's @/ext/... imports to point into
# the auto tree (../../ = repo root from src/components/). @/ext/src/lib/api
# (direct api imports in components) maps to the host transport layer.
mkdir -p ../src/components
# Plan 007: vm-only widgets (and the vm root App) are never used by the web
# app and import stores via a path the vue codegen emits incorrectly —
# remove stale copies and skip them in the web deployment. They only need to
# run on the vm track (auto run -r vm).
VM_ONLY="App.vue ConfigEditorVm.vue DaemonViewVm.vue CollectionBrowserVm.vue"
for vm in $VM_ONLY; do rm -f "../src/components/${vm}"; done
for f in gen/front/vue/src/components/*.vue; do
  base=$(basename "$f")
  case " $VM_ONLY " in *" $base "*) continue ;; esac
  sed -e "s|@/ext/src/front/utils/|../../auto/src/front/utils/|g" \
      -e "s|@/ext/src/lib/api|../../lib/api|g" \
      -e "s|@/stores/use|../stores/auto/use|g" \
      "$f" > "../src/components/${base}"
done

echo "REGEN OK — components: $(find ../src/components -maxdepth 1 -name '*.vue' | wc -l), stores: $(find ../src/stores/auto -maxdepth 1 -name '*.ts' 2>/dev/null | wc -l)"
