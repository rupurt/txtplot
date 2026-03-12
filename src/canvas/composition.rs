use super::{BrailleCanvas, ColorBlend};

impl BrailleCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            blend_mode: ColorBlend::Overwrite,
            plot_left_inset_px: 0,
            plot_bottom_inset_px: 0,
            buffer: vec![0u8; size],
            colors: vec![None; size],
            text_layer: vec![None; size],
        }
    }

    #[inline]
    pub fn pixel_width(&self) -> usize {
        self.width * 2
    }

    #[inline]
    pub fn pixel_height(&self) -> usize {
        self.height * 4
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
        self.colors.fill(None);
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

    pub(crate) fn cell_masks(&self) -> &[u8] {
        &self.buffer
    }

    /// Replaces entire cells whenever `top` has content. This allows
    /// curves to sit above a grid without mixing both glyphs.
    pub fn overlay(&mut self, top: &BrailleCanvas) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");

        for idx in 0..self.buffer.len() {
            if top.buffer[idx] != 0 || top.text_layer[idx].is_some() {
                self.buffer[idx] = top.buffer[idx];
                self.colors[idx] = top.colors[idx];
                self.text_layer[idx] = top.text_layer[idx];
            }
        }
    }

    pub(crate) fn merge(&mut self, top: &BrailleCanvas) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");

        for idx in 0..self.buffer.len() {
            if top.buffer[idx] != 0 {
                self.buffer[idx] |= top.buffer[idx];
                if top.colors[idx].is_some() {
                    self.colors[idx] = top.colors[idx];
                }
            }

            if let Some(ch) = top.text_layer[idx] {
                self.text_layer[idx] = Some(ch);
                if top.colors[idx].is_some() {
                    self.colors[idx] = top.colors[idx];
                }
            }
        }
    }

    pub(crate) fn overlay_without_background(
        &mut self,
        top: &BrailleCanvas,
        background_mask: &[u8],
    ) {
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
            if top.buffer[idx] == 0 && top.text_layer[idx].is_none() {
                continue;
            }

            let preserved_color = self.colors[idx];
            let existing_foreground = self.buffer[idx] & !background;

            self.buffer[idx] &= !background;
            self.buffer[idx] |= top.buffer[idx];

            if let Some(ch) = top.text_layer[idx] {
                self.text_layer[idx] = Some(ch);
            }

            let keep_existing_color = top.text_layer[idx].is_none()
                && existing_foreground != 0
                && preserved_color.is_some()
                && existing_foreground.count_ones() >= top.buffer[idx].count_ones();

            if keep_existing_color {
                self.colors[idx] = preserved_color;
            } else if top.colors[idx].is_some() || self.buffer[idx] == 0 {
                self.colors[idx] = top.colors[idx];
            }
        }
    }
}
