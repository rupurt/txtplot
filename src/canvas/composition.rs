use super::{CellCanvas, CellRenderer, ColorBlend};
use std::marker::PhantomData;

impl<R: CellRenderer> CellCanvas<R> {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            blend_mode: ColorBlend::Overwrite,
            plot_left_inset_px: 0,
            plot_bottom_inset_px: 0,
            buffer: vec![R::Cell::default(); size],
            colors: vec![None; size],
            background_colors: vec![None; size],
            text_layer: vec![None; size],
            _renderer: PhantomData,
        }
    }

    #[inline]
    pub fn pixel_width(&self) -> usize {
        self.width * R::CELL_WIDTH
    }

    #[inline]
    pub fn pixel_height(&self) -> usize {
        self.height * R::CELL_HEIGHT
    }

    pub fn clear(&mut self) {
        self.buffer.fill(R::Cell::default());
        self.colors.fill(None);
        self.background_colors.fill(None);
        self.text_layer.fill(None);
        self.plot_left_inset_px = 0;
        self.plot_bottom_inset_px = 0;
    }

    pub fn set_plot_insets(&mut self, left_px: usize, bottom_px: usize) {
        self.plot_left_inset_px = left_px.min(self.pixel_width().saturating_sub(1));
        self.plot_bottom_inset_px = bottom_px.min(self.pixel_height().saturating_sub(1));
    }

    pub fn plot_insets(&self) -> (usize, usize) {
        (self.plot_left_inset_px, self.plot_bottom_inset_px)
    }

    pub(crate) fn cell_masks(&self) -> &[R::Cell] {
        &self.buffer
    }

    /// Replaces entire cells whenever `top` has content. This allows
    /// curves to sit above a grid without mixing both glyphs.
    pub fn overlay(&mut self, top: &Self) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");

        for idx in 0..self.buffer.len() {
            if !R::is_empty(top.buffer[idx]) || top.text_layer[idx].is_some() {
                self.buffer[idx] = top.buffer[idx];
                self.colors[idx] = top.colors[idx];
                self.background_colors[idx] = top.background_colors[idx];
                self.text_layer[idx] = top.text_layer[idx];
            }
        }
    }

    pub(crate) fn merge(&mut self, top: &Self) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");

        for idx in 0..self.buffer.len() {
            if !R::is_empty(top.buffer[idx]) {
                R::merge_cell(&mut self.buffer[idx], top.buffer[idx]);
                if top.colors[idx].is_some() {
                    self.colors[idx] = top.colors[idx];
                }
                if top.background_colors[idx].is_some() {
                    self.background_colors[idx] = top.background_colors[idx];
                }
            }

            if let Some(ch) = top.text_layer[idx] {
                self.text_layer[idx] = Some(ch);
                if top.colors[idx].is_some() {
                    self.colors[idx] = top.colors[idx];
                }
                if top.background_colors[idx].is_some() {
                    self.background_colors[idx] = top.background_colors[idx];
                }
            }
        }
    }

    pub(crate) fn overlay_without_background(&mut self, top: &Self, background_mask: &[R::Cell]) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");
        assert_eq!(
            self.buffer.len(),
            background_mask.len(),
            "background mask size mismatch"
        );

        for (idx, background) in background_mask
            .iter()
            .copied()
            .enumerate()
            .take(self.buffer.len())
        {
            if R::is_empty(top.buffer[idx]) && top.text_layer[idx].is_none() {
                continue;
            }

            let preserved_color = self.colors[idx];
            let existing_foreground = R::without_mask(self.buffer[idx], background);

            R::subtract_mask(&mut self.buffer[idx], background);
            R::merge_cell(&mut self.buffer[idx], top.buffer[idx]);

            if let Some(ch) = top.text_layer[idx] {
                self.text_layer[idx] = Some(ch);
            }

            let keep_existing_color = top.text_layer[idx].is_none()
                && !R::is_empty(existing_foreground)
                && preserved_color.is_some()
                && R::subpixel_count(existing_foreground) >= R::subpixel_count(top.buffer[idx]);

            if keep_existing_color {
                self.colors[idx] = preserved_color;
            } else if top.colors[idx].is_some() || R::is_empty(self.buffer[idx]) {
                self.colors[idx] = top.colors[idx];
            }

            if top.background_colors[idx].is_some() || R::is_empty(self.buffer[idx]) {
                self.background_colors[idx] = top.background_colors[idx];
            }
        }
    }
}
