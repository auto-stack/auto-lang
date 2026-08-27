# sha2 Replication

**Upstream:** sha2 crate v0.10.8 (`Sha256`)
**Scope:** `sha256(input str) str` — SHA-256 digest of an ASCII / byte string, returned as a 64-char lowercase hex string.
**Auto features tested:** 32-bit arithmetic in a signed-only VM, bitwise operations on globals and locals, module-level globals, multi-block loops, large if-chain dispatch (K table / W table accessors), `StringBuilder`.

## API

- `sha256(input str) str` — SHA-256 of `input`, 64-char lowercase hex digest.

Inputs are read via `str.char_at(i)`, which for ASCII inputs (all NIST test
vectors) yields the same byte sequence as the Rust oracle's
`input.as_bytes()`. Non-ASCII / multi-byte UTF-8 inputs are out of scope.

## Implementation notes

SHA-256 is defined over 32-bit *unsigned* words, but the Auto VM has only a
signed `int`. The module proves the signed `int` is sufficient because:

1. The VM's `int` bit pattern is the full 32-bit two's-complement
   representation; a value with the top bit set (e.g. `0x80000000`) is carried
   correctly even though it reads back as negative.
2. Integer `+` wraps mod 2^32 exactly, so no manual modular reduction is
   needed for the compression adds.
3. `.shr(n)` is a LOGICAL (zero-filling, unsigned) right shift even when the
   top bit is set — which is what SHA-256 needs.
4. `.and` / `.or` / `.xor` operate on the full 32-bit pattern.

### Load-bearing VM workarounds

- **No 32-bit literals.** Hex literals `>= 0x80000000` parse to `0`. Every
  K/H constant is built from its four bytes via `mk32(b0,b1,b2,b3)`
  (`shl` + `or`), where each byte is a small literal in `[0,255]`.
- **No arrays.** The 64-entry round-constant table K is surfaced via a long
  if-chain (`k_at(i)`); the 64-entry message schedule W and the 64-byte block
  buffer are backed by module-level `var int` slots with `w_get`/`w_set` and
  `b_get`/`b_set` accessors.
- **`.shr(n)` for n >= 32 wraps** (it lowers to `u32::wrapping_shr`, so a
  shift by 32 is a shift by 0). The 64-bit message-length encoding guards
  `shf < 32` before shifting so the high length bytes are zero-filled
  correctly.
- **State lives in module-level globals** (`H0`..`H7`, `W0`..`W63`,
  `B0`..`B63`, `A`..`H`). The final digest is emitted by passing each `Hi`
  global to `append_hex_word`, which extracts the four big-endian bytes via
  `.shr`/`.and` and maps them through a hex-digit string.

## Known divergences

(none yet)
