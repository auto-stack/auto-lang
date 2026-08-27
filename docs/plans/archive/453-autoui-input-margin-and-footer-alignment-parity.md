# Plan 453: AutoUI Input Margin Support & Footer Alignment Parity

**Status**: Completed  
**Scope**: `crates/auto-lang/src/ui/iced/renderer.rs`, `examples/ui/005-login`  
**Dependencies**: Plan 452  

---

## 1. Background & Discrepancy Analysis
In `examples/ui/005-login`, comparison between Vue mode (`auto run`) and VM mode (`auto run -r vm`) revealed the following UI layout discrepancies:

1. **Email/Password label to Input gap inconsistency**:
   - `input` has `style: "... mt-2 ..."`. In Vue (HTML/CSS), `mt-2` gives 8px margin-top between the label (`Email` / `Password`) and the input box.
   - In VM mode, `AbstractView::Input` in `IntoIcedElement` and `render_dynamic_view` did not call `wrap_with_margin(el, &iced_style)`. Therefore, any `mt-*`, `mb-*`, `ml-*`, `mr-*` on input elements was silently ignored, resulting in 0px gap between label and input in VM mode.
2. **Footer Row vertical baseline alignment**:
   - In `app.at`: `text "Don't have an account? "` had `mt-4`, while `a "Sign up"` had no margin.
   - In VM mode, `wrap_with_margin` applied top padding to the first text item, pushing it 16px down, while `"Sign up"` stayed at top padding 0px, causing the two pieces of text to be vertically misaligned in the row.
   - Proper Tailwind/CSS structure requires `mt-4` on the parent `row` container (`justify-center items-center mt-4`), keeping both text items on the same baseline while spacing the entire footer 16px below the button.
3. **General Widget Margin Support Audit**:
   - Also applied `wrap_with_margin` to `AbstractView::Textarea` and `AbstractView::Checkbox` in both `IntoIcedElement` and `render_dynamic_view`.

---

## 2. Tasks

- [x] Task 1: Create dedicated git worktree `.worktree/plan-453`
- [x] Task 2: Apply `wrap_with_margin` to `AbstractView::Input`, `AbstractView::Textarea`, and `AbstractView::Checkbox` in both `IntoIcedElement` and `render_dynamic_view` in `renderer.rs`
- [x] Task 3: Adjust footer row styling in `examples/ui/005-login/src/front/app.at` to place `mt-4` on the row container and refine inline link spacing
- [x] Task 4: Run test suites (`cargo test -p auto-lang --lib`, `cargo test -p auto-lang --test docs_gen`)
- [x] Task 5: Use `autoui-verifier` to capture and compare updated dual-backend screenshots
- [x] Task 6: Independent Review Gate, archive plan, merge to `master`, remove worktree, and bump `.next-id`

---

## 3. Verification Results

- `cargo test -p auto-lang --lib`: 3,214 passed, 0 failed, 89 ignored.
- `cargo test -p auto-lang --test docs_gen`: 4 passed, 0 failed.
- Dual-backend AutoUI verification:
  - Input label-to-input gap (8px from `mt-2`) is now identical across Vue and VM backends.
  - Footer row text and link are horizontally centered, on the same baseline, with exact spacing (`mr-1`).
