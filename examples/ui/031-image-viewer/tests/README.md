# 031-image-viewer deterministic fixtures

The fixture set is intentionally tiny and redistributable. It contains
1x1/2x1 JPEG, PNG and WebP files, an EXIF-orientation-labelled JPEG, an RGBA
PNG and a deliberately corrupt payload. The files are generated locally with
the image crate using fixed dimensions, colors and quality; no user media is
included.

All three verification paths use the same fixture names and metadata-only
control API:

- Vue: tests/vue-actions.json through the standard Playwright runner.
- VM: the standard AutoUI MCP runner.
- Rust merged: the generated native executable and the same media route.

The UI model stores only opaque media URIs and metadata. Fixture bytes stay in
the test directory and are never serialized into an Auto state value.
