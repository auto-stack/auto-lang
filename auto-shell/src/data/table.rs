//! Table rendering for shell output
//!
//! Provides structured table display with alignment and color support.

use indexmap::IndexMap;
use nu_ansi_term::{Color, Style};

/// Alignment for table columns
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Right,
    Center,
}

/// Column definition
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub width: usize,
    pub align: Align,
    pub style: Option<Style>,
}

impl Column {
    /// Create a new column
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            width: 0,
            align: Align::Left,
            style: None,
        }
    }

    /// Set column width
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set column alignment
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Set column style (color)
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Get column header with styling
    pub fn render_header(&self) -> String {
        let text = self.pad_text(&self.name);
        if let Some(style) = &self.style {
            style.paint(text).to_string()
        } else {
            text
        }
    }

    /// Render a cell value
    pub fn render_cell(&self, value: &str) -> String {
        let text = self.pad_text(value);
        if let Some(style) = &self.style {
            style.paint(text).to_string()
        } else {
            text
        }
    }

    /// Pad text according to alignment
    fn pad_text(&self, text: &str) -> String {
        let text_len = text.chars().count();
        if text_len >= self.width {
            return text.chars().take(self.width).collect::<String>();
        }

        let padding = self.width - text_len;
        match self.align {
            Align::Left => format!("{}{}", text, " ".repeat(padding)),
            Align::Right => format!("{}{}", " ".repeat(padding), text),
            Align::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
            }
        }
    }
}

/// Table structure for rendering tabular data
#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// Create a new table
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Add a column
    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Add a row
    pub fn add_row(mut self, row: Vec<String>) -> Self {
        self.rows.push(row);
        self
    }

    /// Auto-calculate column widths based on content
    pub fn calculate_widths(&mut self) {
        for (col_idx, column) in self.columns.iter_mut().enumerate() {
            let mut max_width = column.name.chars().count();

            // Check all rows
            for row in &self.rows {
                if let Some(cell) = row.get(col_idx) {
                    max_width = max_width.max(cell.chars().count());
                }
            }

            column.width = max_width;
        }
    }

    /// Render table as string
    pub fn render(&self) -> String {
        let mut lines = Vec::new();

        // Render header
        let header: Vec<String> = self.columns.iter()
            .map(|col| col.render_header())
            .collect();
        lines.push(header.join("  "));

        // Render separator (if more than one column)
        if self.columns.len() > 1 {
            let separator: Vec<String> = self.columns.iter()
                .map(|col| "-".repeat(col.width))
                .collect();
            lines.push(separator.join("  "));
        }

        // Render rows
        for row in &self.rows {
            let cells: Vec<String> = self.columns.iter()
                .enumerate()
                .map(|(col_idx, col)| {
                    let cell_value = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                    col.render_cell(cell_value)
                })
                .collect();
            lines.push(cells.join("  "));
        }

        lines.join("\n")
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// File entry for directory listing
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

impl FileEntry {
    /// Get style for file type
    pub fn style(&self) -> Style {
        if self.is_dir {
            Color::Blue.bold()
        } else if self.name.ends_with(".at") || self.name.ends_with(".rs") {
            Color::Green.normal()
        } else if self.name.ends_with(".exe") || self.name.ends_with(".dll") {
            Color::Cyan.bold()
        } else {
            Style::default()
        }
    }

    /// Format file size
    pub fn format_size(&self) -> String {
        if let Some(size) = self.size {
            if size < 1024 {
                format!("{}B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1}K", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        } else {
            String::from("-")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let col = Column::new("Name").width(10).align(Align::Left);
        assert_eq!(col.name, "Name");
        assert_eq!(col.width, 10);
        assert_eq!(col.align, Align::Left);
    }

    #[test]
    fn test_column_padding_left() {
        let col = Column::new("Test").width(10).align(Align::Left);
        assert_eq!(col.pad_text("hi"), "hi        ");
    }

    #[test]
    fn test_column_padding_right() {
        let col = Column::new("Test").width(10).align(Align::Right);
        assert_eq!(col.pad_text("hi"), "        hi");
    }

    #[test]
    fn test_column_padding_center() {
        let col = Column::new("Test").width(10).align(Align::Center);
        assert_eq!(col.pad_text("hi"), "    hi    ");
    }

    #[test]
    fn test_column_truncation() {
        let col = Column::new("Test").width(5).align(Align::Left);
        assert_eq!(col.pad_text("hello world"), "hello");
    }

    #[test]
    fn test_table_creation() {
        let table = Table::new()
            .add_column(Column::new("Name").width(10))
            .add_column(Column::new("Size").width(8))
            .add_row(vec!["file.txt".to_string(), "1.2K".to_string()])
            .add_row(vec!["doc.pdf".to_string(), "45.0K".to_string()]);

        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_table_render() {
        let mut table = Table::new()
            .add_column(Column::new("Name").width(8))
            .add_column(Column::new("Size").width(6))
            .add_row(vec!["file.txt".to_string(), "1.2K".to_string()])
            .add_row(vec!["doc.pdf".to_string(), "45K".to_string()]);

        let output = table.render();
        println!("Output:\n{}", output);
        // Header should have proper spacing
        assert!(output.contains("Name"));
        assert!(output.contains("Size"));
        // Data rows should contain the values
        assert!(output.contains("file.txt"));
        assert!(output.contains("1.2K"));
        assert!(output.contains("doc.pdf"));
        assert!(output.contains("45K"));
    }

    #[test]
    fn test_file_entry_style() {
        let entry = FileEntry {
            name: "test.rs".to_string(),
            is_dir: false,
            size: Some(1024),
            modified: None,
        };

        let style = entry.style();
        // Green for .rs files
        assert!(format!("{:?}", style).contains("Green"));
    }

    #[test]
    fn test_file_entry_format_size() {
        let entry = FileEntry {
            name: "test.txt".to_string(),
            is_dir: false,
            size: Some(1536), // 1.5K
            modified: None,
        };

        assert_eq!(entry.format_size(), "1.5K");
    }

    #[test]
    fn test_file_entry_directory_style() {
        let entry = FileEntry {
            name: "src".to_string(),
            is_dir: true,
            size: None,
            modified: None,
        };

        let style = entry.style();
        // Blue bold for directories
        assert!(format!("{:?}", style).contains("Blue"));
    }
}
