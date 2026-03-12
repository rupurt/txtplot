use super::{CellCanvas, CellRenderer};
use colored::Color;

impl<R: CellRenderer> CellCanvas<R> {
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Option<Color>) {
        let inverted_y = self.pixel_height().saturating_sub(1).saturating_sub(y);
        self.set_pixel_impl(x, inverted_y, color);
    }

    pub fn set_pixel_screen(&mut self, x: usize, y: usize, color: Option<Color>) {
        self.set_pixel_impl(x, y, color);
    }

    pub fn unset_pixel(&mut self, x: usize, y: usize) {
        let inverted_y = self.pixel_height().saturating_sub(1).saturating_sub(y);
        self.unset_pixel_impl(x, inverted_y);
    }

    pub fn unset_pixel_screen(&mut self, x: usize, y: usize) {
        self.unset_pixel_impl(x, y);
    }

    pub fn set_cell_background(&mut self, col: usize, row: usize, color: Option<Color>) {
        let inverted_row = self.height.saturating_sub(1).saturating_sub(row);
        self.set_cell_background_impl(col, inverted_row, color);
    }

    pub fn set_cell_background_screen(&mut self, col: usize, row: usize, color: Option<Color>) {
        self.set_cell_background_impl(col, row, color);
    }

    pub fn toggle_pixel_screen(&mut self, x: usize, y: usize, color: Option<Color>) {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return;
        }

        let index = self.idx(x / R::CELL_WIDTH, y / R::CELL_HEIGHT);
        if R::is_subpixel_set(self.buffer[index], x % R::CELL_WIDTH, y % R::CELL_HEIGHT) {
            self.unset_pixel_impl(x, y);
        } else {
            self.set_pixel_impl(x, y, color);
        }
    }
}
