# Fix Stdlib Compilation Errors

## Goal
Fix compilation errors in `stdlib/auto/dstr.at`, `builder.at`, `hashmap.at`, and `hashset.at` preventing `a2c-stdlib` from working. The primary issue appears to be missing imports for types like `List`.

## Proposed Changes

### stdlib/auto
#### [MODIFY] [dstr.at](file:///d:/autostack/auto-lang/stdlib/auto/dstr.at)
- Add `use auto::data::List` or qualify `List` as `auto::data::List`.

#### [MODIFY] [builder.at](file:///d:/autostack/auto-lang/stdlib/auto/builder.at)
- Add necessary imports (likely `use auto::dstr`).

### stdlib/auto/data
#### [MODIFY] [hashmap.at](file:///d:/autostack/auto-lang/stdlib/auto/data/hashmap.at)
- Add necessary imports if any.

#### [MODIFY] [hashset.at](file:///d:/autostack/auto-lang/stdlib/auto/data/hashset.at)
- Add necessary imports if any.

## Verification
- Run `cargo run -p auto -- a2c-stdlib` and verify it succeeds or moves past the current errors.
