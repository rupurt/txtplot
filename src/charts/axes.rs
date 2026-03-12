use super::ChartContext;
use colored::Color;

impl ChartContext {
    pub fn text(&mut self, text: &str, x_norm: f64, y_norm: f64, color: Option<Color>) {
        let w = self.canvas.width;
        let h = self.canvas.height;
        let cx = (x_norm * w.saturating_sub(1) as f64).round() as usize;
        let cy = (y_norm * h.saturating_sub(1) as f64).round() as usize;

        for (i, ch) in text.chars().enumerate() {
            if cx + i >= w {
                break;
            }
            self.canvas.set_char(cx + i, cy, ch, color);
        }
    }

    /// Draws the axes and computes intermediate ticks automatically.
    pub fn draw_axes(&mut self, x_range: (f64, f64), y_range: (f64, f64), color: Option<Color>) {
        let w_px = self.canvas.pixel_width() as isize;
        let h_px = self.canvas.pixel_height() as isize;
        self.canvas.set_plot_insets(1, 1);
        let (left_inset_px, bottom_inset_px) = self.canvas.plot_insets();

        self.draw_background_overlay(|overlay| {
            overlay.line(
                left_inset_px as isize,
                bottom_inset_px as isize,
                left_inset_px as isize,
                h_px - 1,
                color,
            );
            overlay.line(
                left_inset_px as isize,
                bottom_inset_px as isize,
                w_px - 1,
                bottom_inset_px as isize,
                color,
            );
        });

        let y_ticks = Self::axis_ticks(self.y_scale, y_range);
        for val in y_ticks {
            let Some(norm_y) = Self::normalized_axis_position(self.y_scale, val, y_range) else {
                continue;
            };
            self.text(&Self::format_tick(self.y_scale, val), 0.0, norm_y, color);
        }

        let x_ticks = Self::axis_ticks(self.x_scale, x_range);
        for val in x_ticks {
            let Some(norm_x) = Self::normalized_axis_position(self.x_scale, val, x_range) else {
                continue;
            };

            let label = Self::format_tick(self.x_scale, val);
            let margin = if self.canvas.width > 1 {
                (label.len().saturating_sub(1) as f64 / (self.canvas.width - 1) as f64).min(0.45)
            } else {
                0.0
            };
            let safe_x = norm_x.clamp(margin, 1.0 - margin);
            self.text(&label, safe_x, 0.0, color);
        }
    }

    pub fn draw_grid(&mut self, divs_x: usize, divs_y: usize, color: Option<Color>) {
        let w_px = self.canvas.pixel_width() as isize;
        let h_px = self.canvas.pixel_height() as isize;

        self.draw_background_overlay(|overlay| {
            for i in 1..divs_x {
                let x = (i as f64 / divs_x as f64 * w_px as f64).round() as isize;
                overlay.line(x, 0, x, h_px, color);
            }

            for i in 1..divs_y {
                let y = (i as f64 / divs_y as f64 * h_px as f64).round() as isize;
                overlay.line(0, y, w_px, y, color);
            }
        });
    }
}
