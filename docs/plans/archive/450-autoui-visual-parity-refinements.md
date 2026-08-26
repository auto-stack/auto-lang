# Plan 450: AutoUI Visual Parity Refinements (VM & Vue Alignment)

## 1. Context & Motivation
Inspection of `examples/ui/004-profile-card` revealed specific visual discrepancies between the Vue backend (`auto run`) and VM/Iced backend (`auto run -r vm`):
1. **Directional Border Radius**: `rounded-t-*`, `rounded-b-*`, `rounded-l-*`, `rounded-r-*` are not parsed in `class.rs`, causing header containers to render with 90° sharp corners.
2. **Text Box-Model Styling**: `AbstractView::Text` ignores container styles (`background_color`, `padding`, `border`, `border_radius`), preventing badge pills and inset paragraphs from rendering properly.
3. **Button Typography Defaults**: Buttons in VM default to 16px Regular (400) instead of shadcn-vue's `text-sm` (14px) and `font-medium` (500).

## 2. Tasks
- [x] Task 1: Add directional `rounded-t-*`, `rounded-b-*`, `rounded-l-*`, `rounded-r-*` parsing in `crates/auto-lang/src/ui/style/class.rs`
- [x] Task 2: Support negative margin tokens (`-mt-`, `-mb-`, `-ml-`, `-mr-`, `-m-`, `-mx-`, `-my-`) in `class.rs`
- [x] Task 3: Support 4-corner radii in `crates/auto-lang/src/ui/style/iced_adapter.rs` and `effective_border_radius()`
- [x] Task 4: Support `AbstractView::Text` container styling in `crates/auto-lang/src/ui/iced/renderer.rs`
- [x] Task 5: Align VM button default font size (14px) and weight (500) with shadcn-vue in `renderer.rs`
- [x] Task 6: Run test suite and dual-backend visual verification with `skills/autoui-verifier`
- [x] Task 7: Independent review gate, plan archiving, and merge

## 3. Verification & Results
- Unit tests: `cargo test -p auto-lang --lib` passed (3211 tests passed, 0 failed).
- Docs tests: `cargo test -p auto-lang --test docs_gen` passed (4 tests passed).
- Visual verification: Captured dual-backend screenshots `004_vm_initial.png` and `004_vue_initial.png`.
  - Header banner: `rounded-t-lg` correctly rounds top-left & top-right corners in VM mode.
  - Avatar: `-mt-10` shifts the avatar upwards over the header banner boundary.
  - Badge: "Full Stack Developer" text renders with pill background, padding, and rounded-full capsule.
  - Bio: Paragraph text renders with horizontal padding and relaxed line height.
  - Buttons: "Follow" and "Message" render with 14px Medium typography aligned with Vue shadcn defaults.

