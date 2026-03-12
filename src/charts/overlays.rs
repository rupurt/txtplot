use super::ChartContext;
use crate::canvas::BrailleCanvas;

impl ChartContext {
    pub(super) fn draw_foreground_overlay<F>(&mut self, draw: F)
    where
        F: FnOnce(&mut BrailleCanvas),
    {
        let mut overlay = BrailleCanvas::new(self.canvas.width, self.canvas.height);
        overlay.blend_mode = self.canvas.blend_mode;
        draw(&mut overlay);
        self.canvas
            .overlay_without_background(&overlay, &self.background_mask);
    }

    pub(super) fn draw_background_overlay<F>(&mut self, draw: F)
    where
        F: FnOnce(&mut BrailleCanvas),
    {
        let mut overlay = BrailleCanvas::new(self.canvas.width, self.canvas.height);
        overlay.blend_mode = self.canvas.blend_mode;
        draw(&mut overlay);
        self.canvas.merge(&overlay);
        for (mask, cell) in self.background_mask.iter_mut().zip(overlay.cell_masks()) {
            *mask |= *cell;
        }
    }
}
