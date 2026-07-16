# regex Replication

**Upstream:** regex crate v1.10 (`regex::Regex::is_match` / `find`)
**Scope:** a simplified backtracking matcher for a useful regex subset.

Supported syntax:
- `.` — any single character
- `*` — zero or more (greedy)
- `+` — one or more (greedy)
- `?` — zero or one (greedy)
- `[abc]` — character class (membership)
- `[a-z]` — character range inside a class
- literal characters

## API

The Auto replication exposes matching as **free functions** that take the
pattern and input strings and return primitives only (the same shape as the
base64 / url replications).

- `is_match(pattern str, input str) int` — `1` if `pattern` matches anywhere in
  `input` (unanchored search), else `0`. Mirrors `regex::Regex::is_match`.
- `find(pattern str, input str) str` — the leftmost match as a string, or `""`
  if there is no match (a zero-width match also yields `""`). Mirrors the
  matched text of `regex::Regex::find`.

## Representation notes / divergences from the regex crate

This is a *simplified* matcher built to exercise the same input → output
mapping as the `regex` crate for the cases the tests cover. It is not a full
regex engine. Key, deliberate differences:

- **No AST.** The plan suggested a `tag Node` AST, but Auto's VM cannot express
  recursive sum types. The matcher works directly on the pattern string,
  parsing each "atom" (literal / `.` / `[...]`) on the fly by index. No
  intermediate tree is built.
- **Backtracking, not NFA.** The `regex` crate uses a Pike-VM NFA simulation;
  this matcher uses recursive backtracking. For greedy repetitions without
  alternation the two produce the same leftmost-longest result, so the test
  cases (chosen to be unambiguous) agree on both backends.
- **`is_match` returns `int`, not `bool`.** See "Implementation notes" below.
- **No anchors, alternation, captures, escapes, or repetition of groups.** The
  supported metacharacters are exactly the five listed above. Characters like
  `(`, `)`, `^`, `$`, `\` are treated as literals (matching themselves).

## Implementation notes (Auto VM workarounds)

The current Auto VM has four load-bearing constraints this library works
around (the same families hit by serde_json / url):

1. **Strings are passed as parameters, not module globals.** A module-level
   `var str` is unreadable via `.char_at` / `.len`: `.char_at(0)` returns the
   code of the *variable name's* first character and `.len()` returns `1`,
   regardless of the stored value. So `pattern` and `input` (and their
   precomputed lengths) are threaded through every helper as parameters. Only
   `MATCHED` and `LAST_END` are module globals, and they are plain `bool` /
   `int` (which survive recursion — only complex values like `StringBuilder`
   or user structs corrupt across frames).
2. **No `bool` returns across recursion.** Returning a `bool` up through
   recursive frames corrupts the value, so the recursive helpers
   (`match_here`, `match_star`) return nothing and set `MATCHED` / `LAST_END`
   on success — the same pattern serde_json uses with its `POS`/`N`/`ERR`
   globals.
3. **`is_match` returns `int` (1/0), not `bool`.** A `bool` crossing the module
   boundary to the caller reads back as a wrong value and compares with
   garbage. The Rust oracle asserts integers too, so both backends agree.
4. **Control flow is nested `if`/`else if`/`else` with one trailing return per
   function.** A block whose last statements are a function call followed by
   `return` can emit invalid bytecode (stack underflow / `InvalidOpCode(255)`),
   and an accumulator `var` assigned inside an `if` and returned at the end can
   lose the assignment. Each function therefore returns directly from each
   branch.

`str.char_at(i)` returns the code point as `int`, so all character comparisons
use integer codes (e.g. `.`=46, `*`=42, `[`=91, `]`=93, `-`=45); strings are
built with `StringBuilder`.

## a2r (transpiler) note

The library uses no module-level `var str` (only `var bool` / `var int`
globals), and its recursive helpers return `void` (no value crosses the
recursive frame), so it transpiles cleanly under the a2r fixes landed for
serde_json / url (boolean methods, `char_at`, `StringBuilder`, module-level
`var` → `Lazy<Mutex<T>>`).

## Known divergences

See `parity/docs/known-divergences.md`.
