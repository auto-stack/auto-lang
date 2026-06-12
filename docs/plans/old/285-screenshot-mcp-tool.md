# Plan 285: Add `autoui_screenshot` MCP Tool

## Status: ✅ Implemented

## Summary

Added a screenshot capture MCP tool to the AutoUI protocol. When an AI agent calls `autoui_screenshot`, the iced desktop app captures the current window as a PNG file and returns the file path. Claude can then use its `Read` tool to view the screenshot.

## Architecture

Uses the same `SharedState` → iced `update()` pattern as `needs_bounds` (Plan 282):

```
MCP thread                           iced main thread
    │                                     │
    │ request_screenshot()                │
    │ → oneshot::Sender in SharedState    │
    ├────────────────────────────────────►│
    │                                     │ window::oldest()
    │                                     │   .then(window::screenshot)
    │                                     │   .then(save_png + reply)
    │  ◄─────────────────────────────────┤
    │ recv() → file path                  │
```

## Files Modified

### `crates/auto-lang/Cargo.toml`
- Added `image = { version = "0.25", optional = true }` dependency
- Added `dep:image` to `ui-iced` feature

### `crates/auto-lang/src/ui/mcp_server.rs`
- Added `ScreenshotRequest` struct with `reply_tx: Sender<Result<String, String>>`
- Added `screenshot_request: Option<ScreenshotRequest>` to `SharedState`
- Added `request_screenshot()` and `take_screenshot_request()` methods
- Added `autoui_screenshot` tool definition (no params, readOnly, idempotent)
- Added `tool_screenshot()` handler: locks SharedState, stores request, waits up to 10s for reply

### `crates/auto-lang/src/ui/iced/renderer.rs`
- Added `screenshot_request: RefCell<Option<ScreenshotRequest>>` to `DynamicState`
- In `dynamic_view()`: picks up pending screenshot request from MCP SharedState
- In `update()`: executes screenshot via `iced::window::oldest()` → `iced::window::screenshot(id)` → `save_screenshot_png()`
- Added `save_screenshot_png()`: converts RGBA bytes to PNG via `image::RgbaImage`, saves to `tmp/autoui-screenshot-{timestamp}.png`

## Usage

1. Run an iced example: `auto run examples/ui/011-calculator`
2. Connect MCP client to port 9247
3. Call `autoui_screenshot` tool
4. Read the returned file path to view the PNG screenshot
