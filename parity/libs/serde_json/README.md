# serde_json Replication (subset)

**Upstream:** serde_json crate v1.0
**Scope:** `Value` (null/bool/num/str/arr/obj), `parse()` and `to_string()`.
Does NOT include: serialization derives, Serde trait, streaming parser, error position tracking.
**Auto features tested:** recursive data structures (tag/enum), pattern matching (is), generics, string parsing.

## API

- `parse(input str) Result[str, str]` — parse JSON string, return canonical JSON or Err
- `to_string(input str) str` — serialize (parse + re-emit canonical form)

## Representation note

Auto's VM cannot express recursive `tag`/`enum` types. The parsed Value is represented
by its canonical (compact) JSON text — a `str`. `parse` is a real recursive-descent parser
that validates and re-emits canonical form; `to_string` does the same.

## Known divergences

See `parity/docs/known-divergences.md` for VM limitations around recursive types.
