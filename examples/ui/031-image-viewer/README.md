# 031 Image Viewer

031-image-viewer is the AutoUI read-only image viewer example. It keeps file
metadata and viewport transforms in the front model; image bytes remain behind
the shared media URI and ImageSurface runtime.

## Run modes

```text
# Vue + Rust backend
auto run

# VM merged (AutoUI MCP)
auto run -r vm

# Rust UI generation (merged mode is the default)
auto run -r rust --server rust
```

The Vue action file and deterministic fixtures live under tests/. The release
performance harness is tests/perf_release.ps1 and writes a local JSON report.
The control API is metadata-only: open, snapshot, navigate, request-view,
close, and stats never return encoded pixels or base64.

## Scope

Supported formats are static JPEG, PNG, and WebP. The UI covers open file or
directory, fit/width/1:1, zoom, pan, rotation, fullscreen, keyboard controls,
loading/error/empty states, and an 80 ms settle timer.

