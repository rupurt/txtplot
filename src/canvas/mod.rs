mod clipping;
mod composition;
mod pixels;
mod primitives;
mod render;
mod renderer;
mod selection;
mod text;
mod ui;

#[cfg(test)]
mod tests;

use colored::Color;
use std::marker::PhantomData;

pub use renderer::CellAppearance;
pub use renderer::{BrailleRenderer, CellRenderer, HalfBlockRenderer, QuadrantRenderer};
pub use selection::RendererKind;
pub use text::{TextIntensity, TextStyle};
pub use ui::{CellRect, PanelStyle};

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
    background_colors: Vec<Option<Color>>,
    text_layer: Vec<Option<char>>,
    text_intensity: Vec<TextIntensity>,
    _renderer: PhantomData<R>,
}

pub type BrailleCanvas = CellCanvas<BrailleRenderer>;
pub type HalfBlockCanvas = CellCanvas<HalfBlockRenderer>;
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
        R::apply_subpixel(
            &mut self.buffer[index],
            px % R::CELL_WIDTH,
            py % R::CELL_HEIGHT,
            color,
            self.blend_mode,
            &mut self.colors[index],
            &mut self.background_colors[index],
        );
    }

    fn unset_pixel_impl(&mut self, px: usize, py: usize) {
        if px >= self.pixel_width() || py >= self.pixel_height() {
            return;
        }

        let index = self.idx(px / R::CELL_WIDTH, py / R::CELL_HEIGHT);
        R::clear_subpixel(
            &mut self.buffer[index],
            px % R::CELL_WIDTH,
            py % R::CELL_HEIGHT,
            &mut self.colors[index],
            &mut self.background_colors[index],
        );
    }

    fn set_cell_background_impl(&mut self, col: usize, row: usize, color: Option<Color>) {
        if col >= self.width || row >= self.height {
            return;
        }

        let index = self.idx(col, row);
        self.background_colors[index] = color;
    }
}
