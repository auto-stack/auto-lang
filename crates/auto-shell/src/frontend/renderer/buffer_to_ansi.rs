//! Buffer → ANSI string conversion bridge
//!
//! Converts a ratatui `Buffer` (grid of `Cell`s with style info) into an ANSI
//! colored string that can be fed to reedline's `menu_string()` or printed
//! directly to the terminal.
//!
//! This is the key technical bridge: ratatui widgets render to `Buffer` in
//! memory, then this module converts the styled cells to ANSI escape sequences
//! via `nu-ansi-term`.

use nu_ansi_term::Color as AnsiColor;
use ratatui_core::buffer::Buffer;
use ratatui_core::style::{Color, Modifier};

/// Convert a ratatui `Buffer` to an ANSI-colored string.
///
/// Each row of cells is rendered as a line. Cells with default style produce
/// plain text; cells with fg/bg/modifier produce ANSI escape sequences.
/// A reset sequence is appended after each styled run.
pub fn buffer_to_ansi(buf: &Buffer) -> String {
    let mut output = String::new();
    let width = buf.area.width as usize;
    let height = buf.area.height as usize;

    for row in 0..height {
        for col in 0..width {
            let cell = &buf.content[row * width + col];
            let style = cell_style_to_ansi(cell.fg, cell.bg, cell.modifier);
            output.push_str(&style.paint(cell.symbol()).to_string());
        }
        // Add newline between rows, but not after the last row
        if row < height - 1 {
            output.push('\n');
        }
    }

    output
}

/// Convert a ratatui `Buffer` to a plain-text string (no ANSI escapes).
///
/// Useful for testing and for environments that don't support ANSI.
pub fn buffer_to_plain(buf: &Buffer) -> String {
    let mut output = String::new();
    let width = buf.area.width as usize;
    let height = buf.area.height as usize;

    for row in 0..height {
        for col in 0..width {
            let cell = &buf.content[row * width + col];
            output.push_str(cell.symbol());
        }
        if row < height - 1 {
            output.push('\n');
        }
    }

    output
}

/// Map ratatui fg/bg/Modifier to a nu-ansi-term `Style`.
fn cell_style_to_ansi(fg: Color, bg: Color, modifier: Modifier) -> nu_ansi_term::Style {
    let mut style = nu_ansi_term::Style::new();

    // Foreground color
    style = apply_fg(style, fg);

    // Background color
    style = apply_bg(style, bg);

    // Modifiers
    if modifier.intersects(Modifier::BOLD) {
        style = style.bold();
    }
    if modifier.intersects(Modifier::DIM) {
        style = style.dimmed();
    }
    if modifier.intersects(Modifier::ITALIC) {
        style = style.italic();
    }
    if modifier.intersects(Modifier::UNDERLINED) {
        style = style.underline();
    }
    if modifier.intersects(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK) {
        style = style.blink();
    }
    if modifier.intersects(Modifier::REVERSED) {
        style = style.reverse();
    }
    if modifier.intersects(Modifier::HIDDEN) {
        style = style.hidden();
    }
    if modifier.intersects(Modifier::CROSSED_OUT) {
        style = style.strikethrough();
    }

    style
}

fn apply_fg(style: nu_ansi_term::Style, color: Color) -> nu_ansi_term::Style {
    match color {
        Color::Reset => style,
        Color::Black => style.fg(AnsiColor::Black),
        Color::Red => style.fg(AnsiColor::Red),
        Color::Green => style.fg(AnsiColor::Green),
        Color::Yellow => style.fg(AnsiColor::Yellow),
        Color::Blue => style.fg(AnsiColor::Blue),
        Color::Magenta => style.fg(AnsiColor::Purple),
        Color::Cyan => style.fg(AnsiColor::Cyan),
        Color::Gray => style.fg(AnsiColor::DarkGray), // ratatui Gray = ANSI dark gray (8)
        Color::DarkGray => style.fg(AnsiColor::DarkGray),
        Color::LightRed => style.fg(AnsiColor::LightRed),
        Color::LightGreen => style.fg(AnsiColor::LightGreen),
        Color::LightYellow => style.fg(AnsiColor::LightYellow),
        Color::LightBlue => style.fg(AnsiColor::LightBlue),
        Color::LightMagenta => style.fg(AnsiColor::LightMagenta),
        Color::LightCyan => style.fg(AnsiColor::LightCyan),
        Color::White => style.fg(AnsiColor::White),
        Color::Rgb(r, g, b) => style.fg(AnsiColor::Rgb(r, g, b)),
        Color::Indexed(n) => style.fg(AnsiColor::Fixed(n)),
    }
}

fn apply_bg(style: nu_ansi_term::Style, color: Color) -> nu_ansi_term::Style {
    match color {
        Color::Reset => style,
        Color::Black => style.on(AnsiColor::Black),
        Color::Red => style.on(AnsiColor::Red),
        Color::Green => style.on(AnsiColor::Green),
        Color::Yellow => style.on(AnsiColor::Yellow),
        Color::Blue => style.on(AnsiColor::Blue),
        Color::Magenta => style.on(AnsiColor::Purple),
        Color::Cyan => style.on(AnsiColor::Cyan),
        Color::Gray => style.on(AnsiColor::DarkGray),
        Color::DarkGray => style.on(AnsiColor::DarkGray),
        Color::LightRed => style.on(AnsiColor::LightRed),
        Color::LightGreen => style.on(AnsiColor::LightGreen),
        Color::LightYellow => style.on(AnsiColor::LightYellow),
        Color::LightBlue => style.on(AnsiColor::LightBlue),
        Color::LightMagenta => style.on(AnsiColor::LightMagenta),
        Color::LightCyan => style.on(AnsiColor::LightCyan),
        Color::White => style.on(AnsiColor::White),
        Color::Rgb(r, g, b) => style.on(AnsiColor::Rgb(r, g, b)),
        Color::Indexed(n) => style.on(AnsiColor::Fixed(n)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::{Color, Modifier, Style};

    /// Helper: create a 3×2 buffer with specific content and styles.
    fn make_buffer() -> Buffer {
        let area = Rect::new(0, 0, 3, 2);
        let mut buf = Buffer::empty(area);

        // Row 0: "ABC" with bold red fg
        buf.set_string(0, 0, "ABC", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        // Row 1: "XYZ" plain (default style)
        buf.set_string(0, 1, "XYZ", Style::default());

        buf
    }

    #[test]
    fn test_buffer_to_plain() {
        let buf = make_buffer();
        let plain = buffer_to_plain(&buf);
        assert_eq!(plain, "ABC\nXYZ");
    }

    #[test]
    fn test_buffer_to_ansi_contains_text() {
        let buf = make_buffer();
        let ansi = buffer_to_ansi(&buf);
        // Should contain the text characters
        assert!(ansi.contains('A'));
        assert!(ansi.contains('Z'));
        // Should contain ANSI escape for bold red
        assert!(ansi.contains("\x1b["));
    }

    #[test]
    fn test_buffer_to_ansi_has_newline_between_rows() {
        let buf = make_buffer();
        let ansi = buffer_to_ansi(&buf);
        // Split by newline should give 2 "lines" (may contain ANSI escapes)
        let lines: Vec<&str> = ansi.split('\n').collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_empty_buffer() {
        let area = Rect::new(0, 0, 0, 0);
        let buf = Buffer::empty(area);
        let plain = buffer_to_plain(&buf);
        assert_eq!(plain, "");
        let ansi = buffer_to_ansi(&buf);
        assert_eq!(ansi, "");
    }

    #[test]
    fn test_single_cell_buffer() {
        let area = Rect::new(0, 0, 1, 1);
        let mut buf = Buffer::empty(area);
        buf.set_string(0, 0, "X", Style::default().fg(Color::Green));

        let plain = buffer_to_plain(&buf);
        assert_eq!(plain, "X");

        let ansi = buffer_to_ansi(&buf);
        assert!(ansi.contains('X'));
        assert!(ansi.contains("\x1b["));
    }

    #[test]
    fn test_rgb_color() {
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        buf.set_string(0, 0, "Hi", Style::default().fg(Color::Rgb(255, 0, 128)));

        let ansi = buffer_to_ansi(&buf);
        // Each cell is individually styled, so check characters separately
        assert!(ansi.contains('H'));
        assert!(ansi.contains('i'));
        // RGB uses 38;2;r;g;b sequence
        assert!(ansi.contains("38;2;255;0;128"));
    }

    #[test]
    fn test_reversed_modifier() {
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        buf.set_string(0, 0, "RV", Style::default().add_modifier(Modifier::REVERSED));

        let ansi = buffer_to_ansi(&buf);
        // Each cell is individually styled
        assert!(ansi.contains('R'));
        assert!(ansi.contains('V'));
        assert!(ansi.contains("\x1b[7m")); // ANSI reverse
    }
}
