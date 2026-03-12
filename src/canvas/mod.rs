mod clipping;
mod composition;
mod pixels;
mod primitives;
mod render;

#[cfg(test)]
mod tests;

use colored::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorBlend {
    /// Overwrites the cell's previous color.
    Overwrite,
    /// Keeps the first color assigned to the cell (does not overwrite it).
    KeepFirst,
}

pub struct BrailleCanvas {
    pub width: usize,
    pub height: usize,
    pub blend_mode: ColorBlend,
    plot_left_inset_px: usize,
    plot_bottom_inset_px: usize,
    buffer: Vec<u8>,
    colors: Vec<Option<Color>>,
    text_layer: Vec<Option<char>>,
}

impl BrailleCanvas {
    #[inline]
    fn idx(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    #[inline]
    fn get_mask(sub_x: usize, sub_y: usize) -> u8 {
        match (sub_x, sub_y) {
            (0, 0) => 0x01,
            (1, 0) => 0x08,
            (0, 1) => 0x02,
            (1, 1) => 0x10,
            (0, 2) => 0x04,
            (1, 2) => 0x20,
            (0, 3) => 0x40,
            (1, 3) => 0x80,
            _ => 0,
        }
    }

    fn set_pixel_impl(&mut self, px: usize, py: usize, color: Option<Color>) {
        if px >= self.pixel_width() || py >= self.pixel_height() {
            return;
        }

        let index = self.idx(px / 2, py / 4);
        self.buffer[index] |= Self::get_mask(px % 2, py % 4);

        if let Some(c) = color {
            match self.blend_mode {
                ColorBlend::Overwrite => self.colors[index] = Some(c),
                ColorBlend::KeepFirst => {
                    if self.colors[index].is_none() {
                        self.colors[index] = Some(c);
                    }
                }
            }
        }
    }

    fn unset_pixel_impl(&mut self, px: usize, py: usize) {
        if px >= self.pixel_width() || py >= self.pixel_height() {
            return;
        }

        let index = self.idx(px / 2, py / 4);
        self.buffer[index] &= !Self::get_mask(px % 2, py % 4);
        if self.buffer[index] == 0 {
            self.colors[index] = None;
        }
    }
}
