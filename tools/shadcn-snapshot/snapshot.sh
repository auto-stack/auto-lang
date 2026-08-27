#!/usr/bin/env bash
# PLAN-457: replayable shadcn-vue component snapshot fetcher.
#
# Usage: snapshot.sh <scratch-dir> [component-name ...]
#   Fetches each catalog component (default style, matching
#   crates/auto-man/assets/shadcn-ui/SNAPSHOT.md) into a scratch Vite-less
#   project via `pnpm dlx shadcn-vue@latest add`, one item per invocation so
#   a single registry miss doesn't abort the whole batch.
#
# Afterwards copy <scratch-dir>/src/components/ui/* into
# crates/auto-man/assets/shadcn-ui/ and re-apply the Sonner icon renames
# (see SNAPSHOT.md).

set -u

SCRATCH="${1:?usage: snapshot.sh <scratch-dir> [component ...]}"
shift || true

CATALOG=(
  button input textarea checkbox switch select tabs dialog tooltip slider
  radio-group progress badge skeleton card avatar table separator scroll-area
  label alert sonner dropdown-menu popover sheet breadcrumb accordion
  alert-dialog command form navigation-menu sidebar stepper calendar carousel
  combobox context-menu drawer hover-card number-field pagination pin-input
  tags-input toggle-group aspect-ratio button-group chart chart-area chart-bar
  chart-line chart-donut collapsible input-group input-otp kbd menubar
  native-select range-calendar resizable auto-complete
)
ITEMS=("${@:-${CATALOG[@]}}")

mkdir -p "$SCRATCH/src/assets"
cd "$SCRATCH"

cat > components.json <<'EOF'
{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "default",
  "typescript": true,
  "tailwind": {
    "config": "tailwind.config.cjs",
    "css": "src/assets/index.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}
EOF

cat > package.json <<'EOF'
{
  "name": "shadcn-snapshot-scratch",
  "version": "0.0.0",
  "private": true,
  "type": "module"
}
EOF

touch tailwind.config.cjs src/assets/index.css

cat > tsconfig.json <<'EOF'
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["src/**/*", "src/**/*.vue"]
}
EOF

pass=0; fail=0
for n in "${ITEMS[@]}"; do
  if pnpm dlx shadcn-vue@latest add "$n" --yes --overwrite >/dev/null 2>&1; then
    echo "PASS $n"; pass=$((pass+1))
  else
    echo "FAIL $n"; fail=$((fail+1))
  fi
done
echo "done: $pass pass, $fail fail"
