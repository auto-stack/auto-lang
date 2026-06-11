//! Rendering functions for compact grid and descriptive list modes

use nu_ansi_term::ansi::RESET;

use super::layout::LayoutMode;
use super::style;

/// Render the menu as a string with ANSI colors.
///
/// `items` is (display, description, kind, is_selected) for each visible item.
/// Returns the complete menu string with `\r\n` line endings.
pub fn render_menu(
    items: &[(String, Option<String>, crate::completions::CompletionKind, bool)],
    mode: &LayoutMode,
    _terminal_width: u16,
    use_ansi_coloring: bool,
) -> String {
    if items.is_empty() {
        if use_ansi_coloring {
            format!(
                "{}NO RECORDS FOUND{}",
                style::selected_overlay().prefix(),
                RESET
            )
        } else {
            "NO RECORDS FOUND".to_string()
        }
    } else {
        match mode {
            LayoutMode::CompactGrid { columns, col_width } => {
                render_compact_grid(items, *columns, *col_width, use_ansi_coloring)
            }
            LayoutMode::DescriptiveList { name_width } => {
                render_descriptive_list(items, *name_width, use_ansi_coloring)
            }
        }
    }
}

/// Render compact multi-column grid
fn render_compact_grid(
    items: &[(String, Option<String>, crate::completions::CompletionKind, bool)],
    columns: u16,
    col_width: usize,
    use_ansi_coloring: bool,
) -> String {
    let columns = columns.max(1) as usize;
    let mut result = String::new();

    for (i, (display, _, kind, selected)) in items.iter().enumerate() {
        if use_ansi_coloring {
            let base_style = style::kind_color(*kind);
            let text = if *selected {
                style::selected_overlay().paint(display.as_str()).to_string()
            } else {
                base_style.paint(display.as_str()).to_string()
            };

            // Pad to column width (using display width, not byte length)
            let display_width = unicode_width::UnicodeWidthStr::width(display.as_str());
            let padding = col_width.saturating_sub(display_width);
            result.push_str(&text);
            result.push_str(&" ".repeat(padding));
        } else {
            let marker = if *selected { ">" } else { " " };
            let display_width = unicode_width::UnicodeWidthStr::width(display.as_str());
            let padding = col_width.saturating_sub(display_width + 1);
            result.push_str(marker);
            result.push_str(display);
            result.push_str(&" ".repeat(padding));
        }

        // New line at column boundary
        if (i + 1) % columns == 0 || i == items.len() - 1 {
            result.push_str("\r\n");
        }
    }

    result
}

/// Render descriptive list with name + description
fn render_descriptive_list(
    items: &[(String, Option<String>, crate::completions::CompletionKind, bool)],
    name_width: usize,
    use_ansi_coloring: bool,
) -> String {
    let mut result = String::new();

    for (display, description, kind, selected) in items {
        if use_ansi_coloring {
            let base_style = style::kind_color(*kind);
            let name = if *selected {
                style::selected_overlay().paint(display.as_str()).to_string()
            } else {
                base_style.paint(display.as_str()).to_string()
            };

            let display_width = unicode_width::UnicodeWidthStr::width(display.as_str());
            let padding = name_width.saturating_sub(display_width) + 2;

            result.push_str(&name);
            result.push_str(&" ".repeat(padding));

            if let Some(desc) = description {
                let desc_style = if *selected {
                    style::selected_overlay()
                } else {
                    nu_ansi_term::Color::Yellow.normal()
                };
                result.push_str(&format!("{}{}", desc_style.paint(desc), RESET));
            }
            result.push_str(RESET);
        } else {
            let marker = if *selected { ">" } else { " " };
            let display_width = unicode_width::UnicodeWidthStr::width(display.as_str());
            let padding = name_width.saturating_sub(display_width) + 2;

            result.push_str(marker);
            result.push_str(display);
            result.push_str(&" ".repeat(padding));
            if let Some(desc) = description {
                result.push_str(desc);
            }
        }
        result.push_str("\r\n");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completions::CompletionKind;

    #[test]
    fn test_render_compact_grid_ansi() {
        let items = vec![
            ("src/".to_string(), None, CompletionKind::Directory, false),
            ("main.rs".to_string(), None, CompletionKind::File, true),
            ("lib.rs".to_string(), None, CompletionKind::File, false),
        ];
        let mode = LayoutMode::CompactGrid {
            columns: 3,
            col_width: 14,
        };
        let result = render_menu(&items, &mode, 80, true);
        assert!(result.contains("src/"));
        assert!(result.contains("main.rs"));
        assert!(result.contains("lib.rs"));
        assert!(result.contains("\r\n"));
    }

    #[test]
    fn test_render_descriptive_list_ansi() {
        let items = vec![
            (
                "build".to_string(),
                Some("Compile project".to_string()),
                CompletionKind::Command,
                false,
            ),
            (
                "run".to_string(),
                Some("Run binary".to_string()),
                CompletionKind::Command,
                true,
            ),
        ];
        let mode = LayoutMode::DescriptiveList { name_width: 10 };
        let result = render_menu(&items, &mode, 80, true);
        assert!(result.contains("build"));
        assert!(result.contains("Compile project"));
        assert!(result.contains("run"));
        assert!(result.contains("Run binary"));
    }

    #[test]
    fn test_render_no_ansi() {
        let items = vec![
            ("ls".to_string(), None, CompletionKind::Command, false),
        ];
        let mode = LayoutMode::CompactGrid {
            columns: 1,
            col_width: 10,
        };
        let result = render_menu(&items, &mode, 80, false);
        assert!(result.contains("ls"));
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_render_empty() {
        let items: Vec<(String, Option<String>, CompletionKind, bool)> = vec![];
        let mode = LayoutMode::CompactGrid {
            columns: 1,
            col_width: 10,
        };
        let result = render_menu(&items, &mode, 80, true);
        assert!(result.contains("NO RECORDS FOUND"));
    }
}
