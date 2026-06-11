//! Layout engine — automatically chooses between compact grid and descriptive list

use unicode_width::UnicodeWidthStr;

/// Resolved layout parameters
#[derive(Debug, Clone)]
pub struct Layout {
    /// Which layout mode to use
    pub mode: LayoutMode,
    /// Total rows needed to display all items
    pub total_rows: u16,
}

/// Layout strategy
#[derive(Debug, Clone)]
pub enum LayoutMode {
    /// Compact multi-column grid (no descriptions)
    CompactGrid {
        columns: u16,
        col_width: usize,
    },
    /// Descriptive list with name + description columns
    DescriptiveList {
        name_width: usize,
    },
}

/// Choose the best layout based on suggestion data and terminal width.
///
/// - All items have no description → `CompactGrid`
/// - Any item has description → `DescriptiveList`
pub fn choose_layout(
    items: &[(String, Option<String>, crate::completions::CompletionKind)],
    terminal_width: u16,
    min_column_width: usize,
    column_padding: usize,
) -> Layout {
    let has_any_desc = items.iter().any(|(_, desc, _)| desc.is_some());

    if has_any_desc {
        let name_width = items
            .iter()
            .map(|(name, _, _)| UnicodeWidthStr::width(name.as_str()))
            .max()
            .unwrap_or(20)
            .min(40);

        let total_rows = items.len() as u16;

        Layout {
            mode: LayoutMode::DescriptiveList { name_width },
            total_rows,
        }
    } else {
        let max_item_width = items
            .iter()
            .map(|(name, _, _)| UnicodeWidthStr::width(name.as_str()))
            .max()
            .unwrap_or(min_column_width)
            .max(min_column_width);

        let cell_width = max_item_width + column_padding;
        let columns = if cell_width > 0 {
            ((terminal_width as usize) / cell_width).max(1) as u16
        } else {
            4
        };

        let total_rows = if columns > 0 {
            (items.len() as u16 + columns - 1) / columns
        } else {
            items.len() as u16
        };

        Layout {
            mode: LayoutMode::CompactGrid {
                columns,
                col_width: cell_width,
            },
            total_rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completions::CompletionKind;

    #[test]
    fn test_compact_grid_layout() {
        let items = vec![
            ("src/".to_string(), None, CompletionKind::Directory),
            ("main.rs".to_string(), None, CompletionKind::File),
            ("lib.rs".to_string(), None, CompletionKind::File),
        ];
        let layout = choose_layout(&items, 80, 12, 2);
        assert!(matches!(layout.mode, LayoutMode::CompactGrid { .. }));
        assert!(layout.total_rows <= 3);
    }

    #[test]
    fn test_descriptive_list_layout() {
        let items = vec![
            (
                "build".to_string(),
                Some("Compile the project".to_string()),
                CompletionKind::Command,
            ),
            (
                "run".to_string(),
                Some("Run the binary".to_string()),
                CompletionKind::Command,
            ),
        ];
        let layout = choose_layout(&items, 80, 12, 2);
        assert!(matches!(layout.mode, LayoutMode::DescriptiveList { .. }));
        assert_eq!(layout.total_rows, 2);
    }

    #[test]
    fn test_single_item() {
        let items = vec![("ls".to_string(), None, CompletionKind::Command)];
        let layout = choose_layout(&items, 80, 12, 2);
        assert!(matches!(layout.mode, LayoutMode::CompactGrid { .. }));
        assert_eq!(layout.total_rows, 1);
    }
}
