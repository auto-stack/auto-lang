# 031 Image Viewer Specification

## Contract

The front model contains session and image metadata plus fit, zoom, pan, and
rotation state. It does not contain file bytes, decoded pixels, a browser
decoder, or an application cache. Back Auto owns directory indexing, session
generation, navigation, settled-view requests, and the shared auto.image
mechanism seam.

The JSON control endpoints are:

- GET /api/viewer/health
- POST /api/viewer/open-file, /open-directory, /open-path
- GET /api/viewer/snapshot and /stats
- POST /api/viewer/navigate, /request-view, and /close

Pixels use the framework media route under /api/__auto/media. The control
plane carries only metadata and opaque tickets.

## Verification

The deterministic fixtures cover JPEG, PNG, WebP, alpha, EXIF-orientation
metadata, and corrupt input. Vue actions are driven by the standard
test_vue_playwright.mjs runner; VM merged uses test_vm_mcp.py. The Rust merged
release harness records timings, RSS, CPU, queue, cache, and shutdown fields.

## Known execution notes

The Plan 547 execution record keeps two environment/tooling observations:
the current auto CLI passes --merged through to cargo for the explicit Rust
command, and the reference release cold-start budget is not met on the current
machine. These do not change the source contract and are recorded in the plan
for independent review.

