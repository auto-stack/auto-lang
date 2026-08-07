#!/bin/sh
# Cargo `rustc-wrapper` for Linux/macOS: proxy to sccache when available,
# otherwise pass through to rustc directly. Activated by setting
# RUSTC_WRAPPER to this script's path (see scripts/README-sccache.md).
#
# Cargo invokes:  sccache-wrap.sh  <rustc>  <rustc-args...>
# so "$@" already starts with the rustc executable, which is exactly what
# sccache expects as its first argument. `exec` replaces this process so the
# child's exit code becomes this script's exit code.

if command -v sccache >/dev/null 2>&1; then
    exec sccache "$@"
else
    exec "$@"
fi
