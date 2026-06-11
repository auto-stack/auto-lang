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

/// Render a `Value::Array` (of objects) as a bordered ANSI table string.
///
/// Returns `None` if the value is not a table-compatible array.
pub fn render_table(value: &Value, term_width: u16) -> Option<String> {
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
    let columns = collect_columns(arr);
    if columns.is_empty() {
        return None;
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
            let cells: Vec<Cell> = columns
                .iter()
                .enumerate()
                .map(|(_col_idx, col)| {
                    let text = if let Value::Obj(obj) = item {
                        if let Some(v) = obj.get(col.as_str()) {
                            format_cell_value(&v)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    // Apply per-cell styling based on content
                    let style = cell_style(&text, col);
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
        // Shrink columns proportionally, keeping minimum of 4 chars
        let excess = total - available;
        let mut remaining = excess;
        for w in widths.iter_mut().rev() {
            if remaining == 0 {
                break;
            }
            let shrink = (*w - 4).min(remaining);
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

/// Apply per-cell styling based on content.
fn cell_style(text: &str, _col: &str) -> Style {
    // Directory names get light blue
    if text.ends_with('/') || text.ends_with('\\') {
        return Style::default().fg(Color::LightBlue);
    }

    // File type coloring
    if text.ends_with(".at") || text.ends_with(".rs") {
        return Style::default().fg(Color::Green);
    }
    if text.ends_with(".exe") || text.ends_with(".dll") {
        return Style::default().fg(Color::LightCyan);
    }
    if text.ends_with(".toml") || text.ends_with(".json") || text.ends_with(".yaml") {
        return Style::default().fg(Color::Yellow);
    }

    // "dir" type value
    if text == "dir" {
        return Style::default().fg(Color::LightBlue);
    }

    Style::default()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Should contain border characters
        assert!(output.contains('╭') || output.contains('┌'));
        assert!(output.contains('╰') || output.contains('└'));
        // Should contain data
        assert!(output.contains("file.txt"));
        assert!(output.contains("src/"));
        // Should contain header
        assert!(output.contains("Name"));
    }
}
