use super::BrailleCanvas;
use colored::Color;

impl BrailleCanvas {
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

    pub fn toggle_pixel_screen(&mut self, x: usize, y: usize, color: Option<Color>) {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return;
        }

        let index = self.idx(x / 2, y / 4);
        let mask = Self::get_mask(x % 2, y % 4);

        if (self.buffer[index] & mask) != 0 {
            self.unset_pixel_impl(x, y);
        } else {
            self.set_pixel_impl(x, y, color);
        }
    }
}
