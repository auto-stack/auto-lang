#!/usr/bin/env bash
set -euo pipefail
exec pwsh.exe -NoProfile -NonInteractive -File "$(dirname "$0")/wt-guard.ps1" "$@"
