//! AshMenu — adaptive completion menu implementing reedline::Menu
//!
//! Replaces reedline's ColumnarMenu with:
//! - Adaptive layout (compact grid vs descriptive list)
//! - Type-based coloring by CompletionKind
//! - Half-screen pager
//! - Built-in search filtering

use reedline::menu_functions::{completer_input, replace_in_buffer};
use reedline::{
    Completer, Editor, Menu, MenuEvent, Painter, Suggestion,
};

use super::layout::{self, Layout, LayoutMode};
use super::render;
use crate::completions::CompletionKind;

/// AshMenu configuration
#[derive(Debug, Clone)]
pub struct AshMenuConfig {
    pub name: String,
    /// Max visible lines (0 = auto, 50% of terminal height)
    pub max_visible_lines: u16,
    /// Minimum column width in compact grid mode
    pub min_column_width: usize,
    /// Column padding in compact grid mode
    pub column_padding: usize,
}

impl Default for AshMenuConfig {
    fn default() -> Self {
        Self {
            name: "ash_menu".to_string(),
            max_visible_lines: 0,
            min_column_width: 12,
            column_padding: 2,
        }
    }
}

/// Adaptive completion menu
pub struct AshMenu {
    config: AshMenuConfig,
    /// Menu name (used by reedline to match events)
    menu_name: String,
    /// Menu indicator
    marker: String,
    /// Is the menu currently active?
    active: bool,
    /// Only use buffer difference for completion
    only_buffer_difference: bool,
    /// Cached suggestion values from completer
    values: Vec<Suggestion>,
    /// Kind metadata for each suggestion (parallel to values)
    kinds: Vec<CompletionKind>,
    /// Description metadata for each suggestion (parallel to values)
    descriptions: Vec<Option<String>>,
    /// Currently selected index
    selected: usize,
    /// Rows skipped (for paging)
    skip_rows: u16,
    /// Cached layout
    layout: Option<Layout>,
    /// Terminal width at last layout computation
    terminal_width: u16,
    /// Event to process
    event: Option<MenuEvent>,
    /// String collected after menu activation
    input: Option<String>,
    /// Minimum rows to always show
    min_rows: u16,
}

impl Default for AshMenu {
    fn default() -> Self {
        Self::new(AshMenuConfig::default())
    }
}

impl AshMenu {
    pub fn new(config: AshMenuConfig) -> Self {
        let menu_name = config.name.clone();
        Self {
            menu_name,
            marker: "| ".to_string(),
            config,
            active: false,
            only_buffer_difference: false,
            values: Vec::new(),
            kinds: Vec::new(),
            descriptions: Vec::new(),
            selected: 0,
            skip_rows: 0,
            layout: None,
            terminal_width: 80,
            event: None,
            input: None,
            min_rows: 3,
        }
    }

    /// Get total number of values
    fn total_values(&self) -> usize {
        self.values.len()
    }

    /// Reset selection to first item
    fn reset_position(&mut self) {
        self.selected = 0;
        self.skip_rows = 0;
    }

    /// Move to next item
    fn move_next(&mut self) {
        if self.total_values() == 0 {
            return;
        }
        self.selected = (self.selected + 1) % self.total_values();
    }

    /// Move to previous item
    fn move_previous(&mut self) {
        if self.total_values() == 0 {
            return;
        }
        self.selected = match self.selected.checked_sub(1) {
            Some(i) => i,
            None => self.total_values() - 1,
        };
    }

    /// Move up one row (in grid: go back `cols` items)
    fn move_up(&mut self) {
        let cols = self.get_columns();
        if cols == 0 {
            return;
        }
        if self.selected >= cols {
            self.selected -= cols;
        } else {
            // Wrap to last row in same column
            let total = self.total_values();
            if total > 0 {
                let last_row_start = ((total - 1) / cols) * cols;
                let col = self.selected % cols;
                self.selected = (last_row_start + col).min(total - 1);
            }
        }
    }

    /// Move down one row
    fn move_down(&mut self) {
        let cols = self.get_columns();
        let total = self.total_values();
        if cols == 0 || total == 0 {
            return;
        }
        let new_idx = self.selected + cols;
        if new_idx < total {
            self.selected = new_idx;
        } else {
            // Wrap to first row in same column
            self.selected = self.selected % cols;
        }
    }

    /// Move left one item
    fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else if self.total_values() > 0 {
            self.selected = self.total_values() - 1;
        }
    }

    /// Move right one item
    fn move_right(&mut self) {
        if self.total_values() > 0 {
            self.selected = (self.selected + 1) % self.total_values();
        }
    }

    /// Get number of columns in current layout
    fn get_columns(&self) -> usize {
        self.layout
            .as_ref()
            .map(|l| match &l.mode {
                LayoutMode::CompactGrid { columns, .. } => *columns as usize,
                LayoutMode::DescriptiveList { .. } => 1,
            })
            .unwrap_or(1)
    }

    /// Get total rows needed
    fn get_total_rows(&self) -> u16 {
        self.layout.as_ref().map(|l| l.total_rows).unwrap_or(0)
    }

    /// Compute layout based on current values and terminal width
    fn compute_layout(&mut self, terminal_width: u16) {
        self.terminal_width = terminal_width;
        if self.values.is_empty() {
            self.layout = None;
            return;
        }

        let items: Vec<(String, Option<String>, CompletionKind)> = self
            .values
            .iter()
            .zip(self.kinds.iter())
            .zip(self.descriptions.iter())
            .map(|((v, k), d)| (v.value.clone(), d.clone(), *k))
            .collect();

        self.layout = Some(layout::choose_layout(
            &items,
            terminal_width,
            self.config.min_column_width,
            self.config.column_padding,
        ));
    }

    /// Adjust skip_rows to keep selection visible
    fn adjust_scroll(&mut self, available_lines: u16) {
        if self.total_values() == 0 {
            return;
        }

        let cols = self.get_columns().max(1);
        let selected_row = self.selected / cols;

        if (selected_row as u16) < self.skip_rows {
            self.skip_rows = selected_row as u16;
        } else if (selected_row as u16) >= self.skip_rows + available_lines {
            self.skip_rows = selected_row as u16 - available_lines + 1;
        }
    }

    /// Infer completion kind from the suggestion value
    fn infer_kind(suggestion: &Suggestion) -> CompletionKind {
        let value = &suggestion.value;
        // Directory: ends with /
        if value.ends_with('/') {
            return CompletionKind::Directory;
        }
        // Variable: contains $ or starts with uppercase env var pattern
        if suggestion.description.as_ref().map_or(false, |d| d.contains("variable"))
            || value.starts_with('$')
        {
            return CompletionKind::Variable;
        }
        // Flag: starts with -
        if value.starts_with('-') {
            return CompletionKind::Flag;
        }
        // Default: Command
        CompletionKind::Command
    }

    /// Get the currently selected suggestion
    fn get_selected_value(&self) -> Option<Suggestion> {
        self.values.get(self.selected).cloned()
    }
}

impl Menu for AshMenu {
    fn name(&self) -> &str {
        &self.menu_name
    }

    fn indicator(&self) -> &str {
        &self.marker
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn menu_event(&mut self, event: MenuEvent) {
        match &event {
            MenuEvent::Activate(_) => self.active = true,
            MenuEvent::Deactivate => {
                self.active = false;
                self.input = None;
            }
            _ => {}
        }
        self.event = Some(event);
    }

    fn can_quick_complete(&self) -> bool {
        true
    }

    fn can_partially_complete(
        &mut self,
        values_updated: bool,
        editor: &mut Editor,
        completer: &mut dyn Completer,
    ) -> bool {
        if !values_updated {
            self.update_values(editor, completer);
        }

        // Try partial completion: find common prefix
        if self.values.len() >= 2 {
            let first = &self.values[0].value;
            let mut common_len = first.len();
            for suggestion in &self.values[1..] {
                common_len = common_len.min(
                    first
                        .chars()
                        .zip(suggestion.value.chars())
                        .take_while(|(a, b)| a == b)
                        .count(),
                );
            }
            if common_len > 0 {
                let common: String = first.chars().take(common_len).collect();
                let span = self.values[0].span;
                let buf = editor.get_buffer();
                let start = span.start.min(buf.len());
                let end = span.end.min(buf.len());
                let new_buf = format!("{}{}{}", &buf[..start], &common, &buf[end..]);
                editor.edit_buffer(
                    |lb| lb.set_buffer(new_buf),
                    reedline::UndoBehavior::CreateUndoPoint,
                );
                self.update_values(editor, completer);
                return true;
            }
        }
        false
    }

    fn update_values(&mut self, editor: &mut Editor, completer: &mut dyn Completer) {
        let buf = editor.get_buffer();
        let pos = buf.len(); // Use end of buffer as insertion point

        let (input, input_pos) = completer_input(
            buf,
            pos,
            self.input.as_deref(),
            self.only_buffer_difference,
        );

        let (values, _base_ranges) = completer.complete_with_base_ranges(&input, input_pos);

        // Extract kinds and descriptions
        self.kinds = values.iter().map(|v| Self::infer_kind(v)).collect();
        self.descriptions = values
            .iter()
            .map(|v| v.description.clone())
            .collect();
        self.values = values;
        self.reset_position();
    }

    fn update_working_details(
        &mut self,
        editor: &mut Editor,
        completer: &mut dyn Completer,
        painter: &Painter,
    ) {
        if let Some(event) = self.event.take() {
            match event {
                MenuEvent::Activate(updated) => {
                    self.active = true;
                    self.reset_position();
                    self.input = if self.only_buffer_difference {
                        Some(editor.get_buffer().to_string())
                    } else {
                        None
                    };
                    if !updated {
                        self.update_values(editor, completer);
                    }
                }
                MenuEvent::Deactivate => self.active = false,
                MenuEvent::Edit(updated) => {
                    self.reset_position();
                    if !updated {
                        self.update_values(editor, completer);
                    }
                }
                MenuEvent::NextElement => self.move_next(),
                MenuEvent::PreviousElement => self.move_previous(),
                MenuEvent::MoveUp => self.move_up(),
                MenuEvent::MoveDown => self.move_down(),
                MenuEvent::MoveLeft => self.move_left(),
                MenuEvent::MoveRight => self.move_right(),
                MenuEvent::NextPage | MenuEvent::PreviousPage => {
                    let available = painter.remaining_lines_real().max(self.min_rows());
                    let rows = self.get_total_rows();
                    if rows > 0 {
                        let cols = self.get_columns().max(1);
                        let current_page_start = self.skip_rows;
                        let page_items = (available as usize) * cols;

                        match event {
                            MenuEvent::NextPage => {
                                let new_skip = (current_page_start as usize) + page_items;
                                if new_skip < self.total_values() {
                                    self.skip_rows = (new_skip / cols) as u16;
                                    self.selected = new_skip.min(self.total_values() - 1);
                                }
                            }
                            MenuEvent::PreviousPage => {
                                let new_skip = (current_page_start as usize)
                                    .saturating_sub(page_items);
                                self.skip_rows = (new_skip / cols) as u16;
                                self.selected = new_skip;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Compute layout based on current values
            self.compute_layout(painter.screen_width());

            // Adjust scroll to keep selected item visible
            let available_lines = painter.remaining_lines_real().max(self.min_rows());
            self.adjust_scroll(available_lines);
        }
    }

    fn replace_in_buffer(&self, editor: &mut Editor) {
        replace_in_buffer(self.get_selected_value(), editor);
    }

    fn menu_required_lines(&self, _terminal_columns: u16) -> u16 {
        let total_rows = self.get_total_rows();
        if total_rows == 0 {
            return 1; // "NO RECORDS FOUND"
        }
        total_rows.min(self.min_rows())
    }

    fn menu_string(&self, available_lines: u16, use_ansi_coloring: bool) -> String {
        if self.values.is_empty() {
            return render::render_menu(
                &[],
                &LayoutMode::CompactGrid {
                    columns: 1,
                    col_width: 10,
                },
                self.terminal_width,
                use_ansi_coloring,
            );
        }

        let layout = match &self.layout {
            Some(l) => l.clone(),
            None => return String::new(),
        };

        let cols = self.get_columns().max(1);
        let skip_items = (self.skip_rows as usize) * cols;
        let max_items = (available_lines as usize) * cols;

        // Build visible items with selection state
        let visible_items: Vec<(String, Option<String>, CompletionKind, bool)> = self
            .values
            .iter()
            .skip(skip_items)
            .take(max_items)
            .enumerate()
            .map(|(i, v)| {
                let idx = skip_items + i;
                (
                    v.value.clone(),
                    self.descriptions.get(idx).cloned().flatten(),
                    self.kinds
                        .get(idx)
                        .copied()
                        .unwrap_or(CompletionKind::Command),
                    idx == self.selected,
                )
            })
            .collect();

        render::render_menu(
            &visible_items,
            &layout.mode,
            self.terminal_width,
            use_ansi_coloring,
        )
    }

    fn min_rows(&self) -> u16 {
        self.min_rows
    }

    fn get_values(&self) -> &[Suggestion] {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ash_menu_default() {
        let menu = AshMenu::default();
        assert!(!menu.is_active());
        assert!(menu.values.is_empty());
        assert_eq!(menu.name(), "ash_menu");
    }

    #[test]
    fn test_navigation() {
        let mut menu = AshMenu::new(AshMenuConfig::default());
        // Simulate having 5 values
        menu.values = (0..5)
            .map(|i| Suggestion {
                value: format!("item{}", i),
                description: None,
                extra: None,
                span: reedline::Span { start: 0, end: 5 },
                append_whitespace: false,
                style: None,
                match_indices: None,
            })
            .collect();
        menu.kinds = vec![CompletionKind::Command; 5];
        menu.descriptions = vec![None; 5];
        menu.layout = Some(Layout {
            mode: LayoutMode::CompactGrid {
                columns: 3,
                col_width: 14,
            },
            total_rows: 2,
        });

        assert_eq!(menu.selected, 0);
        menu.move_next();
        assert_eq!(menu.selected, 1);
        menu.move_next();
        assert_eq!(menu.selected, 2);
        menu.move_next();
        assert_eq!(menu.selected, 3);
        menu.move_previous();
        assert_eq!(menu.selected, 2);

        // Wrap backward
        menu.move_previous();
        menu.move_previous();
        menu.move_previous();
        assert_eq!(menu.selected, 4); // Wrap to last
    }

    #[test]
    fn test_menu_string_empty() {
        let menu = AshMenu::default();
        let s = menu.menu_string(5, true);
        assert!(s.contains("NO RECORDS FOUND"));
    }

    #[test]
    fn test_menu_string_with_items() {
        let mut menu = AshMenu::new(AshMenuConfig::default());
        menu.values = vec![
            Suggestion {
                value: "ls".to_string(),
                description: None,
                extra: None,
                span: reedline::Span { start: 0, end: 2 },
                append_whitespace: false,
                style: None,
                match_indices: None,
            },
            Suggestion {
                value: "cd".to_string(),
                description: None,
                extra: None,
                span: reedline::Span { start: 0, end: 2 },
                append_whitespace: false,
                style: None,
                match_indices: None,
            },
        ];
        menu.kinds = vec![CompletionKind::Command, CompletionKind::Command];
        menu.descriptions = vec![None, None];
        menu.layout = Some(Layout {
            mode: LayoutMode::CompactGrid {
                columns: 2,
                col_width: 14,
            },
            total_rows: 1,
        });
        menu.terminal_width = 80;

        let s = menu.menu_string(5, true);
        assert!(s.contains("ls"));
        assert!(s.contains("cd"));
    }

    #[test]
    fn test_infer_kind() {
        let dir = Suggestion {
            value: "src/".to_string(),
            description: None,
            extra: None,
            span: reedline::Span { start: 0, end: 4 },
            append_whitespace: false,
            style: None,
            match_indices: None,
        };
        assert_eq!(AshMenu::infer_kind(&dir), CompletionKind::Directory);

        let flag = Suggestion {
            value: "--verbose".to_string(),
            description: None,
            extra: None,
            span: reedline::Span { start: 0, end: 9 },
            append_whitespace: false,
            style: None,
            match_indices: None,
        };
        assert_eq!(AshMenu::infer_kind(&flag), CompletionKind::Flag);
    }
}
