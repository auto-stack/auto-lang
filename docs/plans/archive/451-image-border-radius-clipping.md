# Plan 451: Image Border-Radius Clipping & Square Avatar Background Fix in VM Mode

**Status**: Completed  
**Scope**: `crates/auto-lang/src/ui/iced/renderer.rs`  
**Dependencies**: Plan 408, Plan 409, Plan 442  

---

## 1. Context & Motivation

In `examples/ui/004-profile-card`, the profile avatar has:
```auto
image (src: .avatar_url) {
    style: "w-20 h-20 rounded-full border-4 border-gray-200 shadow-md"
}
```
In Vue mode, this renders a circular avatar neatly overlapping the top banner.
In VM mode (`auto run -r vm`), the avatar background previously showed a white square, and the image corners were not clipped, cutting a square notch into the gradient banner.

### Root Causes
1. **Hardcoded Container Background**: The `container` wrapping `iced::widget::image` hardcoded `background: Some(iced::Background::Color(iced::Color::WHITE))`.
2. **Iced Image Widget Limitation**: `container.clip(true)` in Iced only clips an axis-aligned rectangle. Raster images (PNG/JPEG) are rectangular textures, so corners stick out of circular or rounded borders.

---

## 2. Technical Design

1. **Raster Image Alpha Masking**:
   - For raster images with `border_radius > 0.0`:
   - Decode image into `image::RgbaImage`.
   - Calculate SDF distance for each pixel to the rounded rectangle / circle.
   - Set alpha to 0 (with anti-aliasing) outside the boundary.
   - Create `Handle::from_rgba`.
2. **Container Background Normalization**:
   - Only set container background if `is.background_color` is explicitly specified.
3. **Border-Box Insetting**:
   - Inner image widget dimensions are inset by `2 * border_width` so image content abuts the inner edge of the border without painting over it.
4. **Handle Cache Keying**:
   - Key the handle cache with radius and size parameter (`url#r=...`) so textures are cached properly per radius.

---

## 3. Tasks

- [x] Create git worktree `.worktree/plan-451`
- [x] Implement raster image alpha masking and container background fix in `crates/auto-lang/src/ui/iced/renderer.rs`
- [x] Run `cargo test -p auto-lang --lib` (3211 passed, 0 failed)
- [x] Run `cargo test -p auto-lang --test docs_gen` (4 passed, 0 failed)
- [x] Run `autoui-verifier` to capture new VM screenshot for `004-profile-card` and verify visual parity against Vue
- [x] Independent review gate: ensure zero warnings, no leftover debug hacks
- [x] Archive plan to `docs/plans/archive/451-image-border-radius-clipping.md`, merge worktree, and update `.next-id`

---

## 4. Verification Results

- **Unit tests**: `cargo test -p auto-lang --lib` passed (3211 passed, 0 failed, 88 ignored).
- **Docs consistency**: `cargo test -p auto-lang --test docs_gen` passed (4 passed, 0 failed).
- **Dual-backend verification**: Verified `examples/ui/004-profile-card` via `test_vm_mcp.py`. `004_vm_initial.png` shows a circular avatar cleanly overlapping the top gradient banner with circular 4px border and shadow, matching Vue mode.
