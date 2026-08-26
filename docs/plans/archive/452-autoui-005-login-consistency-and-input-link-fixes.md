# Plan 452: AutoUI 005-login Consistency Verification & Cross-Backend Parity Fixes

**Status**: Completed  
**Scope**: `crates/auto-lang/src/ui/aura_view_builder.rs`, `crates/auto-lang/src/ui/iced/renderer.rs`, `examples/ui/005-login`  
**Dependencies**: Plan 450, Plan 451  

---

## 1. Context & Motivation
Running the `/autoui-verifier` workflow on `examples/ui/005-login` identified three key cross-backend discrepancies between the Vue backend (`auto run`) and VM/Iced backend (`auto run -r vm`):

1. **`<a>` / Link Element Missing in VM Mode**:
   - `005-login` contains `a "Sign up" { style: "text-sm text-blue-500 font-semibold underline px-3" }`.
   - In Vue mode, this generates an anchor/link element with full styling.
   - In VM mode, `aura_view_builder.rs` only handled `"text" | "label" | "h1" | "h2" | "h3" | "p" | "span"`. `"a"` fell through to unknown tag fallback and produced `View::Empty`.
2. **Forced Dark Background on Input Widgets**:
   - The card container in `005-login` is `bg-white`, while inputs specify `style: "w-full px-3 py-2 border rounded-lg mt-2"` without an explicit background color.
   - `aura_view_builder.rs` injected `bg-background` into default input styling, and `renderer.rs` fell back to `Color::Background` (`(9, 14, 26)` near-black in dark mode), rendering dark input rectangles inside a white card.
   - In Vue / standard Tailwind, inputs default to transparent background, properly showing the card's background.
3. **Password Masking Unimplemented in VM Mode**:
   - `input { type: "password" ... }` was not recognized as password mode in `aura_view_builder.rs`, and `renderer.rs` ignored the password parameter and did not call `.secure(true)` on `iced::widget::text_input`.

---

## 2. Tasks

- [x] Task 1: Create dedicated git worktree `.worktree/plan-452`
- [x] Task 2: Support `"a"`, `"link"`, `"h4"`, `"h5"`, `"h6"`, `"small"`, `"strong"`, `"em"`, `"b"`, `"i"` in `aura_view_builder.rs`
- [x] Task 3: Support `type: "password"` and transparent default background for inputs in `aura_view_builder.rs`
- [x] Task 4: Implement transparent input fallback and password masking (`.secure(true)`) in `renderer.rs`
- [x] Task 5: Enhance `005-login` submit handler with empty-field error validation to verify conditional rendering
- [x] Task 6: Run full test suite (`cargo test -p auto-lang --lib`, `cargo test -p auto-lang --test docs_gen`)
- [x] Task 7: Run dual-backend interactive verification using `autoui-verifier` and capture screenshots
- [x] Task 8: Independent review gate, archive plan, merge worktree, and bump `.next-id`

---

## 3. Verification Results

- **Unit tests**: `cargo test -p auto-lang --lib` passed (3213 passed, 0 failed, 88 ignored).
- **Docs consistency**: `cargo test -p auto-lang --test docs_gen` passed (4 passed, 0 failed).
- **Dual-backend verification**: Verified `examples/ui/005-login` via `autoui-verifier` across initial, error, typed, and submitted states:
  - Initial: Clean white card, transparent inputs with borders, blue "Sign up" link aligned with "Don't have an account?".
  - Error: Empty submit triggers conditional red error messages under email and password fields on both backends.
  - Typed: Typed input values update live, password characters are securely masked with bullet dots (`•••••••••`), and errors auto-clear.
  - Submitted: Filled submit passes validation on both backends.
