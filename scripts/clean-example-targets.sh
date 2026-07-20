#!/bin/bash
# Remove build artifacts from example projects to free disk space.
#
# Usage: bash scripts/clean-example-targets.sh

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Cleaning example target directories..."

for d in \
    "$REPO_ROOT"/examples/api-example/rust/target \
    "$REPO_ROOT"/examples/component-gallery/vue/src-tauri/target \
    "$REPO_ROOT"/examples/unified-demo/vue/src-tauri/target \
    "$REPO_ROOT"/examples/ui/*/gen/rust/target
do
    if [ -d "$d" ]; then
        echo "  removing $d"
        rm -rf "$d"
    fi
done

echo "Done."
