use super::{AxisScale, CellChartContext, PlotGeometry, PlotScales};
use crate::canvas::CellRenderer;
use colored::Color;
use std::f64::consts::PI;

fn map_coords_for_size<R: CellRenderer>(
    geometry: PlotGeometry,
    scales: PlotScales,
    point: (f64, f64),
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Option<(isize, isize)> {
    let x = scales.x.transform(point.0)?;
    let y = scales.y.transform(point.1)?;
    let (min_x, max_x) = CellChartContext::<R>::transformed_range(scales.x, x_range)?;
    let (min_y, max_y) = CellChartContext::<R>::transformed_range(scales.y, y_range)?;
    let range_x = (max_x - min_x).max(1e-9);
    let range_y = (max_y - min_y).max(1e-9);
    let drawable_width =
        (geometry.width_px.saturating_sub(1 + geometry.left_inset_px)).max(1) as f64;
    let drawable_height = (geometry
        .height_px
        .saturating_sub(1 + geometry.bottom_inset_px))
    .max(1) as f64;

    let px = geometry.left_inset_px as f64 + ((x - min_x) / range_x * drawable_width).round();
    let py = geometry.bottom_inset_px as f64 + ((y - min_y) / range_y * drawable_height).round();

    Some((px as isize, py as isize))
}

impl<R: CellRenderer> CellChartContext<R> {
    fn line_chart_with_ranges(
        &mut self,
        points: &[(f64, f64)],
        x_range: (f64, f64),
        y_range: (f64, f64),
        color: Option<Color>,
    ) {
        let geometry = self.plot_geometry();
        let scales = self.plot_scales();

        self.draw_foreground_overlay(|overlay| {
            for window in points.windows(2) {
                let (x0, y0) = window[0];
                let (x1, y1) = window[1];

                let Some(p0) =
                    map_coords_for_size::<R>(geometry, scales, (x0, y0), x_range, y_range)
                else {
                    continue;
                };
                let Some(p1) =
                    map_coords_for_size::<R>(geometry, scales, (x1, y1), x_range, y_range)
                else {
                    continue;
                };
                overlay.line(p0.0, p0.1, p1.0, p1.1, color);
            }
        });
    }

    pub fn scatter(&mut self, points: &[(f64, f64)], color: Option<Color>) {
        if points.is_empty() {
            return;
        }

        let (x_range, y_range) =
            Self::get_auto_range_scaled(points, 0.05, self.x_scale, self.y_scale);
        let geometry = self.plot_geometry();
        let scales = self.plot_scales();

        self.draw_foreground_overlay(|overlay| {
            for &(x, y) in points {
                let Some((px, py)) =
                    map_coords_for_size::<R>(geometry, scales, (x, y), x_range, y_range)
                else {
                    continue;
                };

                if px >= 0
                    && py >= 0
                    && (px as usize) < geometry.width_px
                    && (py as usize) < geometry.height_px
                {
                    overlay.set_pixel(px as usize, py as usize, color);
                }
            }
        });
    }

    pub fn line_chart(&mut self, points: &[(f64, f64)], color: Option<Color>) {
        if points.len() < 2 {
            return;
        }

        let (x_range, y_range) =
            Self::get_auto_range_scaled(points, 0.05, self.x_scale, self.y_scale);
        self.line_chart_with_ranges(points, x_range, y_range, color);
    }

    pub fn bar_chart(&mut self, values: &[(f64, Option<Color>)]) {
        if values.is_empty() {
            return;
        }

        let w_px = self.canvas.pixel_width();
        let h_px = self.canvas.pixel_height();
        let bar_width = (w_px / values.len()).max(1);

        match self.y_scale {
            AxisScale::Linear => {
                let max_val = values
                    .iter()
                    .filter_map(|(v, _)| if v.is_finite() { Some(*v) } else { None })
                    .fold(0.0f64, f64::max);

                if max_val <= 1e-9 {
                    return;
                }

                for (i, &(val, color)) in values.iter().enumerate() {
                    if !val.is_finite() || val <= 0.0 {
                        continue;
                    }

                    let normalized_h = (val / max_val * h_px as f64).round();
                    let bar_height = (normalized_h as usize).min(h_px);
                    let x_start = i * bar_width;
                    let x_end = (x_start + bar_width).min(w_px);
                    if x_start >= w_px {
                        break;
                    }

                    for x in x_start..x_end {
                        self.canvas
                            .line(x as isize, 0, x as isize, bar_height as isize, color);
                    }
                }
            }
            AxisScale::Log10 => {
                let transformed: Vec<(f64, Option<Color>)> = values
                    .iter()
                    .filter_map(|(value, color)| {
                        Some((AxisScale::Log10.transform(*value)?, *color))
                    })
                    .collect();

                if transformed.is_empty() {
                    return;
                }

                let min_val = transformed
                    .iter()
                    .map(|(value, _)| *value)
                    .fold(f64::INFINITY, f64::min);
                let max_val = transformed
                    .iter()
                    .map(|(value, _)| *value)
                    .fold(f64::NEG_INFINITY, f64::max);
                let range = (max_val - min_val).max(1e-9);

                for (i, &(raw_value, color)) in values.iter().enumerate() {
                    let Some(value) = AxisScale::Log10.transform(raw_value) else {
                        continue;
                    };

                    let normalized_h = (value - min_val) / range;
                    let bar_height =
                        (((normalized_h * h_px.saturating_sub(1) as f64).round() as usize) + 1)
                            .min(h_px);
                    let x_start = i * bar_width;
                    let x_end = (x_start + bar_width).min(w_px);
                    if x_start >= w_px {
                        break;
                    }

                    for x in x_start..x_end {
                        self.canvas
                            .line(x as isize, 0, x as isize, bar_height as isize, color);
                    }
                }
            }
        }
    }

    pub fn polygon(&mut self, vertices: &[(f64, f64)], color: Option<Color>) {
        if vertices.len() < 2 {
            return;
        }

        let normalized_polygon = self.x_scale == AxisScale::Linear
            && self.y_scale == AxisScale::Linear
            && vertices.iter().all(|&(x, y)| {
                x.is_finite()
                    && y.is_finite()
                    && (0.0..=1.0).contains(&x)
                    && (0.0..=1.0).contains(&y)
            });
        let (x_range, y_range) = if normalized_polygon {
            ((0.0, 1.0), (0.0, 1.0))
        } else {
            Self::get_auto_range_scaled(vertices, 0.05, self.x_scale, self.y_scale)
        };
        let geometry = self.plot_geometry();
        let scales = self.plot_scales();

        self.draw_foreground_overlay(|overlay| {
            for i in 0..vertices.len() {
                let (x0, y0) = vertices[i];
                let (x1, y1) = vertices[(i + 1) % vertices.len()];
                let Some(p0) =
                    map_coords_for_size::<R>(geometry, scales, (x0, y0), x_range, y_range)
                else {
                    continue;
                };
                let Some(p1) =
                    map_coords_for_size::<R>(geometry, scales, (x1, y1), x_range, y_range)
                else {
                    continue;
                };
                overlay.line(p0.0, p0.1, p1.0, p1.1, color);
            }
        });
    }

    pub fn pie_chart(&mut self, slices: &[(f64, Option<Color>)]) {
        let total: f64 = slices
            .iter()
            .filter_map(|(v, _)| {
                if v.is_finite() && *v > 0.0 {
                    Some(*v)
                } else {
                    None
                }
            })
            .sum();
        if total <= 1e-9 {
            return;
        }

        let w_px = self.canvas.pixel_width() as isize;
        let h_px = self.canvas.pixel_height() as isize;
        let cx = w_px / 2;
        let cy = h_px / 2;
        let radius = ((w_px.min(h_px).saturating_sub(1)) / 2).max(1);
        let mut current_angle = 0.0;

        for (value, color) in slices {
            if !value.is_finite() || *value <= 0.0 {
                continue;
            }

            let slice_angle = (value / total) * 2.0 * PI;
            let end_angle = current_angle + slice_angle;
            let end_x = cx + (radius as f64 * end_angle.cos()) as isize;
            let end_y = cy + (radius as f64 * end_angle.sin()) as isize;

            self.draw_foreground_overlay(|overlay| {
                overlay.line(cx, cy, end_x, end_y, *color);
            });
            current_angle = end_angle;
        }
    }

    pub fn draw_circle(&mut self, center: (f64, f64), radius_norm: f64, color: Option<Color>) {
        let w_px = self.canvas.pixel_width() as f64;
        let h_px = self.canvas.pixel_height() as f64;
        let min_dim = w_px.min(h_px);

        let r_px = (radius_norm * min_dim) as isize;
        let cx_px = (center.0 * (w_px - 1.0)) as isize;
        let cy_px = (center.1 * (h_px - 1.0)) as isize;

        self.draw_foreground_overlay(|overlay| {
            overlay.circle(cx_px, cy_px, r_px, color);
        });
    }

    pub fn plot_function<F>(&mut self, func: F, min_x: f64, max_x: f64, color: Option<Color>)
    where
        F: Fn(f64) -> f64,
    {
        let steps = self.canvas.pixel_width().saturating_sub(1).max(1);
        let Some(min_x_t) = self.x_scale.transform(min_x) else {
            return;
        };
        let Some(max_x_t) = self.x_scale.transform(max_x) else {
            return;
        };
        let mut points = Vec::with_capacity(steps + 1);

        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = self
                .x_scale
                .inverse_transform(min_x_t + t * (max_x_t - min_x_t));
            let y = func(x);
            if self.y_scale.transform(y).is_some() {
                points.push((x, y));
            }
        }
        if points.len() < 2 {
            return;
        }

        let (_, y_range) = Self::get_auto_range_scaled(&points, 0.05, self.x_scale, self.y_scale);
        self.line_chart_with_ranges(&points, (min_x, max_x), y_range, color);
    }
}
