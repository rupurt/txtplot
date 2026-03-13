use super::{CellCanvas, CellRenderer, TextStyle};
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
    pub fill_char: Option<char>,
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self {
            border_color: Some(Color::White),
            background_color: None,
            title_color: Some(Color::White),
            title_background: None,
            fill_char: Some(' '),
        }
    }
}

impl<R: CellRenderer> CellCanvas<R> {
    fn apply_text_style_impl(&mut self, idx: usize, style: TextStyle, write_background: bool) {
        if let Some(color) = style.foreground {
            self.colors[idx] = Some(color);
        }
        if write_background || style.background.is_some() {
            self.background_colors[idx] = style.background;
        }
        self.text_intensity[idx] = style.intensity;
    }

    pub fn set_char_screen_styled(&mut self, col: usize, row: usize, c: char, style: TextStyle) {
        if col >= self.width || row >= self.height {
            return;
        }

        let idx = self.idx(col, row);
        self.text_layer[idx] = Some(c);
        self.apply_text_style_impl(idx, style, false);
    }

    pub fn set_char_screen(&mut self, col: usize, row: usize, c: char, color: Option<Color>) {
        self.set_char_screen_styled(col, row, c, TextStyle::from(color));
    }

    pub fn text_screen_styled(&mut self, col: usize, row: usize, text: &str, style: TextStyle) {
        for (offset, ch) in text.chars().enumerate() {
            let target_col = col.saturating_add(offset);
            if target_col >= self.width {
                break;
            }
            self.set_char_screen_styled(target_col, row, ch, style);
        }
    }

    pub fn text_screen(&mut self, col: usize, row: usize, text: &str, color: Option<Color>) {
        self.text_screen_styled(col, row, text, TextStyle::from(color));
    }

    pub fn label_screen_styled(&mut self, col: usize, row: usize, text: &str, style: TextStyle) {
        for (offset, ch) in text.chars().enumerate() {
            let target_col = col.saturating_add(offset);
            if target_col >= self.width {
                break;
            }

            let idx = self.idx(target_col, row);
            self.text_layer[idx] = Some(ch);
            self.apply_text_style_impl(idx, style, true);
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
        self.label_screen_styled(
            col,
            row,
            text,
            TextStyle {
                foreground,
                background,
                ..TextStyle::default()
            },
        );
    }

    pub fn clear_rect_screen(&mut self, rect: CellRect) {
        let start_col = rect.col.min(self.width);
        let end_col = rect.end_col().min(self.width);
        let start_row = rect.row.min(self.height);
        let end_row = rect.end_row().min(self.height);

        for row in start_row..end_row {
            for col in start_col..end_col {
                let idx = self.idx(col, row);
                self.buffer[idx] = R::Cell::default();
            }
        }
    }

    pub fn fill_cell_rect_screen(&mut self, rect: CellRect, background: Option<Color>) {
        self.fill_cell_rect_screen_styled(rect, None, TextStyle { background, ..TextStyle::default() });
    }

    pub fn fill_cell_rect_screen_styled(&mut self, rect: CellRect, fill_char: Option<char>, style: TextStyle) {
        let start_col = rect.col.min(self.width);
        let end_col = rect.end_col().min(self.width);
        let start_row = rect.row.min(self.height);
        let end_row = rect.end_row().min(self.height);

        for row in start_row..end_row {
            for col in start_col..end_col {
                let idx = self.idx(col, row);
                if let Some(c) = fill_char {
                    self.text_layer[idx] = Some(c);
                }
                self.apply_text_style_impl(idx, style, true);
            }
        }
    }

    pub fn panel_screen(&mut self, rect: CellRect, title: Option<&str>, style: PanelStyle) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        self.clear_rect_screen(rect);
        self.fill_cell_rect_screen_styled(
            rect,
            style.fill_char,
            TextStyle {
                background: style.background_color,
                ..TextStyle::default()
            },
        );

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
