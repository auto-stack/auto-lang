# Improve Tab Completion in Auto-Shell

## Goal
Modify `auto-shell` completion to autofill the longest common substring when multiple candidates match (e.g. `ls au` -> `ls auto` if candidates are `auto` and `auto-shell`).

## Analysis
Currently, `auto-shell` returns all matching candidates to `reedline`, which triggers the selection menu immediately if there's ambiguity.
To achieve "autofill longest common prefix", we need to intercept the completions in `ShellCompleter::complete`.
Behavior logic:
1. Get all valid completions.
2. Calculate Longest Common Prefix (LCP) of their replacement strings.
3. Compare LCP with the current input word.
    - If LCP is longer than current input: Return ONLY the LCP as a suggestion. This forces `reedline` to extend the input to the LCP without opening the menu.
    - If LCP is same length (or we are unique): Return all candidates. `reedline` will handle uniqueness (fill) or ambiguity (menu) as normal.

## Proposed Changes

### Auto-Shell
#### [repl.rs](file:///d:/autostack/auto-lang/auto-shell/src/repl.rs)
- Enable `with_partial_completions(true)` or `Edit(Complete)` logic.
- Configure `ColumnarMenu` with Grid layout (Vertical).
- Implement cycling keybinding (Debugging "stuck at end" issue).

#### [reedline.rs](file:///d:/autostack/auto-lang/auto-shell/src/completions/reedline.rs)
- Clean up duplicate descriptions.r function.
### src/completions/reedline.rs
- Add `longest_common_prefix` helper function.
- Modify `complete` method:
    - Compute LCP.
    - Implementation check: `if completions.len() > 1 && lcp.len() > input.len() { return vec![lcp_suggestion] }`.

## Verification
- User Scenario: `ls au` (candidates: `auto`, `auto-shell`).
    - Before: Menu shows `auto`, `auto-shell`. Input stays `ls au`.
    - After: Input becomes `ls auto`. Menu does *not* show (yet).
    - Press TAB again: Input is `ls auto`. LCP is `auto`. Menu shows `auto`, `auto-shell`.
- Unit tests in `reedline.rs` to verify LCP logic.
