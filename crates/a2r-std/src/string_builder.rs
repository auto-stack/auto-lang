//! StringBuilder — Auto-to-Rust runtime support for the Auto VM `StringBuilder`
//! type.
//!
//! The Auto VM exposes `StringBuilder` as a built-in type with the methods
//! `new(capacity)`, `append(str)`, `append_char(code)`, `build()`, `len()`, and
//! `clear()`. The a2r transpiler emits references to this type verbatim into
//! transpiled Rust, so this module provides a Rust-native implementation with
//! the identical API surface so that the transpiled code compiles and runs.
//!
//! Auto's `append_char(code)` accepts a Unicode code point expressed as an
//! `i32` (matching the VM's int-based char representation), so this type stores
//! a `char` internally but exposes `append_char` with an `i32` argument.

/// A growable, owned string builder mirroring the Auto VM `StringBuilder` type.
#[derive(Debug, Clone)]
pub struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    /// Create a new, empty `StringBuilder` with the given reserved capacity.
    ///
    /// Matches Auto's `StringBuilder.new(capacity)`. The capacity is an `i32`
    /// in Auto; we treat anything non-positive as "use default capacity".
    pub fn new(capacity: i32) -> Self {
        let cap = if capacity > 0 { capacity as usize } else { 0 };
        StringBuilder {
            buffer: String::with_capacity(cap),
        }
    }

    /// Append a string slice to the buffer.
    ///
    /// Matches Auto's `sb.append(s)`.
    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Append a single character given as a Unicode code point (`i32`).
    ///
    /// Matches Auto's `sb.append_char(code)`. Non-character code points and
    /// out-of-range values are replaced with U+FFFD REPLACEMENT CHARACTER,
    /// mirroring the VM's defensive behaviour for invalid codes.
    pub fn append_char(&mut self, code: i32) {
        let c = char::from_u32(code as u32).unwrap_or('\u{FFFD}');
        self.buffer.push(c);
    }

    /// Return the accumulated `String` without consuming the builder.
    ///
    /// Matches Auto's `sb.build()`: the VM StringBuilder is **not** consumed by
    /// build() — the same builder can be built, appended to further, and built
    /// again. Plan 368 (consumer-mode parity): take `&self` + clone so the a2r
    /// backend mirrors the VM's non-consuming semantics (taking `self` would
    /// move the builder and break any `.at` source that calls build() more than
    /// once on the same builder, or build() after a conditional append path).
    pub fn build(&self) -> String {
        self.buffer.clone()
    }

    /// Return the current length of the buffer in bytes.
    ///
    /// Matches Auto's `sb.len()`.
    pub fn len(&self) -> i32 {
        self.buffer.len() as i32
    }

    /// Return whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear the buffer, leaving it empty with its capacity retained.
    ///
    /// Matches Auto's `sb.clear()`.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        StringBuilder {
            buffer: String::new(),
        }
    }
}
