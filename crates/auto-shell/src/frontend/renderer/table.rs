//! Ratatui Table rendering for structured shell output
//!
//! Converts `Value::Array` (of objects) into a bordered, styled table using
//! ratatui's `Table` widget, rendered to a `Buffer` and then to ANSI string
//! via `buffer_to_ansi()`.

use auto_val::Value;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::text::Text;
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::{Block, Padding};
use ratatui_widgets::borders::BorderType;
use ratatui_widgets::table::{Cell, Row, Table};

use super::buffer_to_ansi;
use crate::config::IconStyle;

/// Render a `Value::Array` (of objects) as a bordered ANSI table string.
///
/// Uses the default icon style (`Plain`). Returns `None` if the value is not a
/// table-compatible array.
pub fn render_table(value: &Value, term_width: u16) -> Option<String> {
    render_table_with(value, term_width, IconStyle::default())
}

/// Render a table with a specific [`IconStyle`] for file-listing rows.
pub fn render_table_with(
    value: &Value,
    term_width: u16,
    icons: IconStyle,
) -> Option<String> {
    let arr = match value {
        Value::Array(a) => a,
        _ => return None,
    };

    if arr.is_empty() {
        return None;
    }

    // All elements must be objects
    let all_objects = arr.iter().all(|v| matches!(v, Value::Obj(_)));
    if !all_objects {
        return None;
    }

    // Collect columns (all unique keys across all objects)
    let mut columns = collect_columns(arr);
    if columns.is_empty() {
        return None;
    }

    // File listings (have both `name` and `type`): prepend an icon column so
    // directories and files are visually distinct at a glance. The icon is
    // purely presentational — it does not pollute the underlying data, so
    // `ls | select name` etc. stay clean. Skipped when icon style is `Off`.
    let is_file_listing = columns.iter().any(|c| c == "name")
        && columns.iter().any(|c| c == "type");
    if is_file_listing && icons != IconStyle::Off {
        columns.insert(0, "icon".to_string());
    }

    // Calculate column widths based on content
    let col_widths = calculate_column_widths(arr, &columns, term_width);

    // Build header row
    let header = Row::new(columns.iter().enumerate().map(|(_i, col)| {
        let display_name = column_display_name(col);
        Cell::from(Text::styled(
            display_name,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Gray),
        ))
    }));

    // Build data rows
    let rows: Vec<Row> = arr
        .iter()
        .enumerate()
        .map(|(row_idx, item)| {
            // Row-level file context: type + name, used to color the Name column
            // (dirs → blue) and to compute the icon column.
            let (row_type, row_name): (Option<String>, String) = if let Value::Obj(obj) = item {
                let t = match obj.get("type") {
                    Some(Value::Str(s)) => Some(s.to_string()),
                    _ => None,
                };
                let n = match obj.get("name") {
                    Some(Value::Str(s)) => s.to_string(),
                    _ => String::new(),
                };
                (t, n)
            } else {
                (None, String::new())
            };

            let cells: Vec<Cell> = columns
                .iter()
                .enumerate()
                .map(|(_col_idx, col)| {
                    // Synthetic icon column — not in the data.
                    if col == "icon" {
                        let icon = file_icon(row_type.as_deref(), &row_name, icons);
                        let style = cell_style(&row_name, "name", row_type.as_deref());
                        return Cell::from(Text::styled(icon, style));
                    }

                    let text = if let Value::Obj(obj) = item {
                        if let Some(v) = obj.get(col.as_str()) {
                            format_cell_value(&v)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    // Apply per-cell styling based on content + row context.
                    let style = cell_style(&text, col, row_type.as_deref());
                    Cell::from(Text::styled(text, style))
                })
                .collect();

            // Subtle zebra striping: even rows get a dark background
            let row_style = if row_idx % 2 == 0 {
                Style::default().bg(Color::Indexed(234))
            } else {
                Style::default()
            };

            Row::new(cells).style(row_style)
        })
        .collect();

    // Build constraints
    let constraints: Vec<Constraint> = col_widths
        .iter()
        .enumerate()
        .map(|(i, &w)| {
            if i == col_widths.len() - 1 {
                // Last column fills remaining space
                Constraint::Min(w)
            } else {
                Constraint::Length(w)
            }
        })
        .collect();

    // Calculate total height: border(2) + header(1) + data_rows
    let total_height = 2 + 1 + rows.len() as u16;
    let area = Rect::new(0, 0, term_width, total_height);

    let table = Table::new(rows, constraints)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Indexed(240)))
                .padding(Padding::horizontal(1)),
        )
        .header(header)
        .column_spacing(2);

    let mut buf = Buffer::empty(area);
    table.render(area, &mut buf);

    Some(buffer_to_ansi(&buf))
}

/// Column display names (capitalize known columns)
fn column_display_name(col: &str) -> String {
    match col {
        // The icon column has no text header (icons are self-explanatory).
        "icon" => String::new(),
        "permissions" => "Permissions".to_string(),
        "owner" => "Owner".to_string(),
        "size" => "Size".to_string(),
        "modified" => "Modified".to_string(),
        "name" => "Name".to_string(),
        "type" => "Type".to_string(),
        _ => {
            // Capitalize first letter
            let mut c = col.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

/// Collect all unique column keys from an array of objects, sorted by priority.
fn collect_columns(arr: &auto_val::Array) -> Vec<String> {
    let mut columns: Vec<String> = Vec::new();
    for item in arr.iter() {
        if let Value::Obj(obj) = item {
            for (key, _) in obj.iter() {
                let key_str = key.to_string();
                if !columns.contains(&key_str) {
                    columns.push(key_str);
                }
            }
        }
    }

    // Sort by priority (same logic as format_array_as_table)
    columns.sort_by(|a, b| {
        let has_long_format =
            a == "permissions" || a == "owner" || b == "permissions" || b == "owner";

        if has_long_format {
            let priority = ["permissions", "owner", "size", "modified", "name"];
            let a_pos = priority.iter().position(|&p| p == a).unwrap_or(usize::MAX);
            let b_pos = priority.iter().position(|&p| p == b).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos).then_with(|| a.cmp(b))
        } else {
            let priority = ["name", "type", "size", "modified"];
            let a_pos = priority.iter().position(|&p| p == a).unwrap_or(usize::MAX);
            let b_pos = priority.iter().position(|&p| p == b).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos).then_with(|| a.cmp(b))
        }
    });

    columns
}

/// Calculate column widths based on content.
/// Returns widths for each column (last column uses remaining space).
fn calculate_column_widths(
    arr: &auto_val::Array,
    columns: &[String],
    term_width: u16,
) -> Vec<u16> {
    let border_overhead = 2 + 2; // left + right border chars
    let spacing_overhead = (columns.len().saturating_sub(1)) as u16 * 2; // column_spacing=2
    let available = term_width.saturating_sub(border_overhead + spacing_overhead);

    let mut widths: Vec<u16> = columns
        .iter()
        .map(|col| {
            // The synthetic icon column holds a double-width glyph; fix its
            // width and never let the shrink pass below it.
            if col == "icon" {
                return 2u16;
            }
            let header_width = column_display_name(col).len() as u16;
            let max_data_width = arr.iter().fold(0u16, |max, item| {
                if let Value::Obj(obj) = item {
                    if let Some(v) = obj.get(col.as_str()) {
                        return max.max(format_cell_value(&v).len() as u16);
                    }
                }
                max
            });
            // Add padding (1 space each side)
            header_width.max(max_data_width) + 2
        })
        .collect();

    // Clamp total width to available space
    let total: u16 = widths.iter().sum::<u16>();
    if total > available {
        // Shrink columns proportionally, keeping minimum of 4 chars.
        // (saturating_sub avoids underflow on narrow columns like the icon col.)
        let excess = total - available;
        let mut remaining = excess;
        for (col, w) in columns.iter().zip(widths.iter_mut()).rev() {
            if remaining == 0 {
                break;
            }
            if col == "icon" {
                continue; // never shrink the fixed-width icon column
            }
            let shrink = (*w).saturating_sub(4).min(remaining);
            *w -= shrink;
            remaining -= shrink;
        }
    }

    widths
}

/// Format a Value for table cell display (no extra quotes for strings).
fn format_cell_value(val: &Value) -> String {
    match val {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Uint(u) => u.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Nil => "nil".to_string(),
        Value::Null => "null".to_string(),
        Value::Void => "void".to_string(),
        _ => val.to_string(),
    }
}

/// Apply per-cell styling based on content + the row's `type` context.
///
/// Coloring for file listings is by **file type / extension**, not git status:
///   - Directory Name → LightBlue (matches the Type column, so dirs stand out
///     from files at a glance).
///   - File Name → by extension: `.at`/`.rs` green, `.exe`/`.dll` light cyan,
///     `.toml`/`.json`/`.yaml` yellow.
///   - The literal "dir" value in the Type column → LightBlue.
///   - The Permissions column → DarkGray, so the noisy `rwx` string recedes
///     and **Name** becomes the visual center of the listing.
fn cell_style(text: &str, col: &str, row_type: Option<&str>) -> Style {
    if col == "name" {
        // Directory name → blue (matches Type column) for clear dir/file contrast.
        if row_type == Some("dir") {
            return Style::default().fg(Color::LightBlue);
        }
        // File name → color by extension.
        if text.ends_with(".at") || text.ends_with(".rs") {
            return Style::default().fg(Color::Green);
        }
        if text.ends_with(".exe") || text.ends_with(".dll") {
            return Style::default().fg(Color::LightCyan);
        }
        if text.ends_with(".toml") || text.ends_with(".json") || text.ends_with(".yaml") {
            return Style::default().fg(Color::Yellow);
        }
        return Style::default();
    }

    // Permissions column → dim gray so it recedes; Name is the visual center.
    if col == "permissions" {
        return Style::default().fg(Color::DarkGray);
    }

    // Type column: the "dir" value → blue.
    if text == "dir" {
        return Style::default().fg(Color::LightBlue);
    }

    Style::default()
}

/// Pick a leading icon glyph for a file-listing row, honoring the configured
/// [`IconStyle`].
///
/// Currently distinguishes **directory vs file** only. The `file_icon_by_name`
/// helpers are extension points for per-extension icons later.
///
/// - `Plain`: single-width geometric glyphs (■/□) — render at normal cell
///   height in every terminal. Color (dir → blue, files → by extension) carries
///   the rest of the distinction.
/// - `NerdFont`: Nerd Font PUA glyphs (single-cell, normal height) — requires a
///   Nerd Font installed in the terminal.
/// - `Emoji`: standard Unicode emoji (📁/📄) — only use if the terminal renders
///   emoji at cell height (many don't, inflating row height).
/// - `Off`: no icon (the caller skips the icon column entirely before reaching
///   here, but this returns a safe fallback regardless).
fn file_icon(row_type: Option<&str>, name: &str, icons: IconStyle) -> &'static str {
    match icons {
        IconStyle::Emoji => match row_type {
            Some("dir") => "📁",
            _ => file_icon_by_name_emoji(name),
        },
        IconStyle::NerdFont => match row_type {
            Some("dir") => "\u{F07C}", // nf-fa-folder
            _ => file_icon_by_name_nerd(name),
        },
        IconStyle::Off => "",
        IconStyle::Plain => match row_type {
            Some("dir") => "■", // filled square — directory (container)
            _ => file_icon_by_name_plain(name),
        },
    }
}

/// Per-file-name icon — Plain (extension point).
fn file_icon_by_name_plain(_name: &str) -> &'static str {
    // TODO(future): match on extension, e.g. png/jpg → "▦", zip/tar → "▣", …
    //   (keep single-width, non-emoji to avoid row-height inflation)
    "□" // outline square — regular file
}

/// Per-file-name icon — Emoji (extension point).
fn file_icon_by_name_emoji(_name: &str) -> &'static str {
    "📄"
}

/// Per-file-name icon — Nerd Font (extension point).
fn file_icon_by_name_nerd(_name: &str) -> &'static str {
    // TODO(future): rs → nf-dev-rust, py → nf-dev-python, png → nf-fa-image, …
    "\u{F15B}" // nf-fa-file
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Strip CSI ANSI escapes so multi-char text assertions work despite
    /// `buffer_to_ansi` emitting styling per buffer cell.
    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                while let Some(csi) = chars.next() {
                    if csi.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn test_column_display_name() {
        assert_eq!(column_display_name("name"), "Name");
        assert_eq!(column_display_name("size"), "Size");
        assert_eq!(column_display_name("modified"), "Modified");
        assert_eq!(column_display_name("custom_field"), "Custom_field");
    }

    #[test]
    fn test_format_cell_value() {
        assert_eq!(format_cell_value(&Value::str("hello")), "hello");
        assert_eq!(format_cell_value(&Value::Int(42)), "42");
        assert_eq!(format_cell_value(&Value::Bool(true)), "true");
        assert_eq!(format_cell_value(&Value::Void), "void");
    }

    #[test]
    fn test_render_table_not_array() {
        let val = Value::str("hello");
        assert!(render_table(&val, 80).is_none());
    }

    #[test]
    fn test_render_table_empty_array() {
        let val = Value::Array(auto_val::Array::new());
        assert!(render_table(&val, 80).is_none());
    }

    #[test]
    fn test_render_table_mixed_types() {
        let arr = auto_val::Array::from_vec(vec![Value::Int(1), Value::str("hello")]);
        let val = Value::Array(arr);
        assert!(render_table(&val, 80).is_none());
    }

    #[test]
    fn test_render_table_with_objects() {
        use auto_val::Obj;

        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("file.txt"));
        obj1.set("type", Value::str("file"));
        obj1.set("size", Value::Int(1024));

        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("src/"));
        obj2.set("type", Value::str("dir"));
        obj2.set("size", Value::Void);

        let arr = auto_val::Array::from_vec(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let val = Value::Array(arr);

        let result = render_table(&val, 60);
        assert!(result.is_some());

        let output = result.unwrap();
        let plain = strip_ansi(&output);
        // Should contain border characters
        assert!(output.contains('╭') || output.contains('┌'));
        assert!(output.contains('╰') || output.contains('└'));
        // Should contain data (strip ANSI: text is per-cell styled, not contiguous)
        assert!(plain.contains("file.txt"));
        assert!(plain.contains("src/"));
        // Should contain header
        assert!(plain.contains("Name"));
    }

    #[test]
    fn test_file_icon_dir_vs_file() {
        use crate::config::IconStyle;
        // Plain (default): single-width geometric glyphs.
        assert_eq!(file_icon(Some("dir"), "anything", IconStyle::Plain), "■");
        assert_eq!(file_icon(Some("file"), "readme.md", IconStyle::Plain), "□");
        assert_eq!(file_icon(None, "readme.md", IconStyle::Plain), "□");
        // Emoji.
        assert_eq!(file_icon(Some("dir"), "x", IconStyle::Emoji), "📁");
        assert_eq!(file_icon(Some("file"), "x", IconStyle::Emoji), "📄");
        // Nerd Font (PUA codepoints).
        assert_eq!(file_icon(Some("dir"), "x", IconStyle::NerdFont), "\u{F07C}");
        assert_eq!(file_icon(Some("file"), "x", IconStyle::NerdFont), "\u{F15B}");
        // Off.
        assert_eq!(file_icon(Some("dir"), "x", IconStyle::Off), "");
    }

    #[test]
    fn test_render_table_file_listing_has_icons() {
        use auto_val::Obj;

        let mut obj1 = Obj::new();
        obj1.set("name", Value::str("src"));
        obj1.set("type", Value::str("dir"));

        let mut obj2 = Obj::new();
        obj2.set("name", Value::str("main.rs"));
        obj2.set("type", Value::str("file"));

        let arr = auto_val::Array::from_vec(vec![Value::Obj(obj1), Value::Obj(obj2)]);
        let val = Value::Array(arr);

        let output = render_table(&val, 60).unwrap();
        let plain = strip_ansi(&output);
        // Icon column present: dir → ■, file → □ (single-width, non-emoji)
        assert!(output.contains('■'), "missing dir icon: {output}");
        assert!(output.contains('□'), "missing file icon: {output}");
        // Names still render (strip ANSI: per-cell styled).
        assert!(plain.contains("src"));
        assert!(plain.contains("main.rs"));
    }

    #[test]
    fn test_render_table_non_file_listing_has_no_icon_column() {
        // A table without a `type` column is not a file listing → no icon column.
        use auto_val::Obj;

        let mut obj = Obj::new();
        obj.set("name", Value::str("widget"));
        obj.set("value", Value::Int(7));

        let arr = auto_val::Array::from_vec(vec![Value::Obj(obj)]);
        let val = Value::Array(arr);

        let output = render_table(&val, 60).unwrap();
        assert!(!output.contains('📁'));
        assert!(!output.contains('📄'));
    }
}
