use super::{CellCanvas, CellRenderer};
use colored::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRect {
    pub col: usize,
    pub row: usize,
    pub width: usize,
    pub height: usize,
}

impl CellRect {
    pub const fn new(col: usize, row: usize, width: usize, height: usize) -> Self {
        Self {
            col,
            row,
            width,
            height,
        }
    }

    fn end_col(self) -> usize {
        self.col.saturating_add(self.width)
    }

    fn end_row(self) -> usize {
        self.row.saturating_add(self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelStyle {
    pub border_color: Option<Color>,
    pub background_color: Option<Color>,
    pub title_color: Option<Color>,
    pub title_background: Option<Color>,
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self {
            border_color: Some(Color::White),
            background_color: None,
            title_color: Some(Color::White),
            title_background: None,
        }
    }
}

impl<R: CellRenderer> CellCanvas<R> {
    pub fn set_char_screen(&mut self, col: usize, row: usize, c: char, color: Option<Color>) {
        if col >= self.width || row >= self.height {
            return;
        }

        let idx = self.idx(col, row);
        self.text_layer[idx] = Some(c);
        if let Some(color) = color {
            self.colors[idx] = Some(color);
        }
    }

    pub fn text_screen(&mut self, col: usize, row: usize, text: &str, color: Option<Color>) {
        for (offset, ch) in text.chars().enumerate() {
            let target_col = col.saturating_add(offset);
            if target_col >= self.width {
                break;
            }
            self.set_char_screen(target_col, row, ch, color);
        }
    }

    pub fn label_screen(
        &mut self,
        col: usize,
        row: usize,
        text: &str,
        foreground: Option<Color>,
        background: Option<Color>,
    ) {
        for (offset, ch) in text.chars().enumerate() {
            let target_col = col.saturating_add(offset);
            if target_col >= self.width {
                break;
            }

            self.set_cell_background_impl(target_col, row, background);
            self.set_char_screen(target_col, row, ch, foreground);
        }
    }

    pub fn fill_cell_rect_screen(&mut self, rect: CellRect, background: Option<Color>) {
        let start_col = rect.col.min(self.width);
        let end_col = rect.end_col().min(self.width);
        let start_row = rect.row.min(self.height);
        let end_row = rect.end_row().min(self.height);

        for row in start_row..end_row {
            for col in start_col..end_col {
                self.set_cell_background_impl(col, row, background);
            }
        }
    }

    pub fn panel_screen(&mut self, rect: CellRect, title: Option<&str>, style: PanelStyle) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        self.fill_cell_rect_screen(rect, style.background_color);

        let start_col = rect.col.min(self.width);
        let end_col = rect.end_col().min(self.width);
        let start_row = rect.row.min(self.height);
        let end_row = rect.end_row().min(self.height);

        if start_col >= end_col || start_row >= end_row {
            return;
        }

        if let Some(border_color) = style.border_color {
            let last_col = end_col - 1;
            let last_row = end_row - 1;

            if start_row == last_row {
                for col in start_col..=last_col {
                    self.set_char_screen(col, start_row, '─', Some(border_color));
                }
            } else if start_col == last_col {
                for row in start_row..=last_row {
                    self.set_char_screen(start_col, row, '│', Some(border_color));
                }
            } else {
                self.set_char_screen(start_col, start_row, '┌', Some(border_color));
                self.set_char_screen(last_col, start_row, '┐', Some(border_color));
                self.set_char_screen(start_col, last_row, '└', Some(border_color));
                self.set_char_screen(last_col, last_row, '┘', Some(border_color));

                for col in start_col + 1..last_col {
                    self.set_char_screen(col, start_row, '─', Some(border_color));
                    self.set_char_screen(col, last_row, '─', Some(border_color));
                }

                for row in start_row + 1..last_row {
                    self.set_char_screen(start_col, row, '│', Some(border_color));
                    self.set_char_screen(last_col, row, '│', Some(border_color));
                }
            }
        }

        let Some(title) = title.filter(|title| !title.is_empty()) else {
            return;
        };

        let inner_width = rect.width.saturating_sub(2);
        if inner_width == 0 {
            return;
        }

        let label: String = format!(" {title} ").chars().take(inner_width).collect();
        let foreground = style.title_color.or(style.border_color);
        let background = style.title_background.or(style.background_color);
        self.label_screen(
            start_col.saturating_add(1),
            start_row,
            &label,
            foreground,
            background,
        );
    }
}
