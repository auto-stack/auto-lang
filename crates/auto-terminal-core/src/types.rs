//! Owned terminal value contract (PLAN-808 SD-01).
//!
//! These types cross the VM / UI / external-consumer boundary. They are
//! deliberately plain owned data: no UI registry references, no VM stack
//! types, no raw DLL pointers. The scalar color decoding is the single copy
//! of the engine's `(kind << 24) | value` wire format.

/// Cell color — scalar mirror of the engine's ANSI color surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TerminalColor {
    Default,
    /// 0-255 palette (0-15 correspond to the named colors).
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// One grid cell (owned projection of one engine cell).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: TerminalColor,
    pub bg: TerminalColor,
}

impl TerminalCell {
    pub fn plain(ch: char) -> Self {
        Self {
            ch,
            fg: TerminalColor::Default,
            bg: TerminalColor::Default,
        }
    }
}

/// Grid geometry in cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TerminalGeometry {
    pub cols: u16,
    pub rows: u16,
}

/// Resize safety clamp (legacy 014 guard: 1-column geometry triggers the
/// engine's full-history reflow pathology; see autoterm DEBTS #15).
pub const MIN_RESIZE_COLS: u16 = 2;
pub const MAX_RESIZE_COLS: u16 = 500;
pub const MAX_RESIZE_ROWS: u16 = 200;

/// Engine palette slots: `[0]` = default fg, `[1]` = default bg,
/// `[2..18]` = base16.
pub const PALETTE_SLOTS: usize = 18;

/// Decode one engine style scalar: `(kind << 24) | value` with kind
/// 0=Default, 1=Indexed (0-255), 2=RGB (`0xRRGGBB`); anything else is
/// Default. This is the only copy of the bit-field algorithm (PLAN-808
/// T-01 §4.3).
pub fn decode_style_color(v: u32) -> TerminalColor {
    let kind = v >> 24;
    let value = v & 0x00FF_FFFF;
    match kind {
        1 => TerminalColor::Indexed(value as u8),
        2 => TerminalColor::Rgb((value >> 16) as u8, (value >> 8) as u8, value as u8),
        _ => TerminalColor::Default,
    }
}

/// One harvested viewport snapshot. `lines` is the text layer (the
/// `engine_rows` return value); `cells` is the styled layer the UI adapter
/// projects; `cursor` is `Some` only when the engine reported a visible
/// cursor on this harvest (the service cache keeps the last value otherwise).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TerminalSnapshot {
    pub lines: Vec<String>,
    pub cells: Vec<Vec<TerminalCell>>,
    pub cursor: Option<(i32, i32)>,
    /// Engine scrollback rows at harvest time.
    pub history: i32,
    /// Engine display offset at harvest time (0 = live bottom).
    pub display_offset: i32,
}

/// Frozen cursor/viewport cache of a handle. Captured when a handle is freed
/// so the UI/VM can keep the legacy post-free observable values after the
/// original provider is replaced or dropped (R2-F02). This is a value copy:
/// it never contains a DLL pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerminalRetiredCache {
    pub cursor_row: i64,
    pub cursor_col: i64,
    pub viewport_cols: i64,
    pub viewport_rows: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_wire_format_decoding() {
        assert_eq!(decode_style_color(0), TerminalColor::Default);
        assert_eq!(decode_style_color(1 << 24 | 7), TerminalColor::Indexed(7));
        assert_eq!(
            decode_style_color(2 << 24 | 0x10_20_30),
            TerminalColor::Rgb(0x10, 0x20, 0x30)
        );
        // Unknown kinds and the 0xFFFF_FFFF sentinel fall back to Default.
        assert_eq!(decode_style_color(3 << 24 | 5), TerminalColor::Default);
        assert_eq!(decode_style_color(0xFFFF_FFFF), TerminalColor::Default);
    }
}
