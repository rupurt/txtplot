mod clipping;
mod composition;
mod pixels;
mod primitives;
mod render;
mod renderer;

#[cfg(test)]
mod tests;

use colored::Color;
use std::marker::PhantomData;

pub use renderer::{BrailleRenderer, CellRenderer, QuadrantRenderer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorBlend {
    /// Overwrites the cell's previous color.
    Overwrite,
    /// Keeps the first color assigned to the cell (does not overwrite it).
    KeepFirst,
}

pub struct CellCanvas<R: CellRenderer> {
    pub width: usize,
    pub height: usize,
    pub blend_mode: ColorBlend,
    plot_left_inset_px: usize,
    plot_bottom_inset_px: usize,
    buffer: Vec<R::Cell>,
    colors: Vec<Option<Color>>,
    text_layer: Vec<Option<char>>,
    _renderer: PhantomData<R>,
}

pub type BrailleCanvas = CellCanvas<BrailleRenderer>;
pub type QuadrantCanvas = CellCanvas<QuadrantRenderer>;

impl<R: CellRenderer> CellCanvas<R> {
    #[inline]
    fn idx(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    fn set_pixel_impl(&mut self, px: usize, py: usize, color: Option<Color>) {
        if px >= self.pixel_width() || py >= self.pixel_height() {
            return;
        }

        let index = self.idx(px / R::CELL_WIDTH, py / R::CELL_HEIGHT);
        R::set_subpixel(
            &mut self.buffer[index],
            px % R::CELL_WIDTH,
            py % R::CELL_HEIGHT,
        );

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

        let index = self.idx(px / R::CELL_WIDTH, py / R::CELL_HEIGHT);
        R::unset_subpixel(
            &mut self.buffer[index],
            px % R::CELL_WIDTH,
            py % R::CELL_HEIGHT,
        );
        if R::is_empty(self.buffer[index]) {
            self.colors[index] = None;
        }
    }
}
