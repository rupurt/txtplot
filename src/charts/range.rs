use super::{AxisScale, CellChartContext};
use crate::canvas::CellRenderer;

impl<R: CellRenderer> CellChartContext<R> {
    pub fn get_auto_range(points: &[(f64, f64)], padding: f64) -> ((f64, f64), (f64, f64)) {
        Self::get_auto_range_scaled(points, padding, AxisScale::Linear, AxisScale::Linear)
    }

    pub fn get_auto_range_scaled(
        points: &[(f64, f64)],
        padding: f64,
        x_scale: AxisScale,
        y_scale: AxisScale,
    ) -> ((f64, f64), (f64, f64)) {
        let valid_points: Vec<(f64, f64)> = points
            .iter()
            .filter_map(|&(x, y)| Some((x_scale.transform(x)?, y_scale.transform(y)?)))
            .collect();

        if valid_points.is_empty() {
            return (Self::default_range(x_scale), Self::default_range(y_scale));
        }

        let (min_x, max_x) = valid_points
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (x, _)| {
                (min.min(*x), max.max(*x))
            });

        let (min_y, max_y) = valid_points
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (_, y)| {
                (min.min(*y), max.max(*y))
            });

        (
            Self::expand_range(min_x, max_x, padding, x_scale),
            Self::expand_range(min_y, max_y, padding, y_scale),
        )
    }

    fn default_range(scale: AxisScale) -> (f64, f64) {
        match scale {
            AxisScale::Linear => (0.0, 1.0),
            AxisScale::Log10 => (1.0, 10.0),
        }
    }

    fn expand_range(min: f64, max: f64, padding: f64, scale: AxisScale) -> (f64, f64) {
        let range = if (max - min).abs() < 1e-9 {
            1.0
        } else {
            max - min
        };
        let min = scale.inverse_transform(min - range * padding);
        let max = scale.inverse_transform(max + range * padding);
        (min, max)
    }

    pub(super) fn transformed_range(scale: AxisScale, range: (f64, f64)) -> Option<(f64, f64)> {
        let min = scale.transform(range.0)?;
        let max = scale.transform(range.1)?;
        Some(if min <= max { (min, max) } else { (max, min) })
    }

    pub(super) fn normalized_axis_position(
        scale: AxisScale,
        value: f64,
        range: (f64, f64),
    ) -> Option<f64> {
        let value = scale.transform(value)?;
        let (min, max) = Self::transformed_range(scale, range)?;
        let span = (max - min).max(1e-9);
        Some(((value - min) / span).clamp(0.0, 1.0))
    }

    pub(super) fn axis_ticks(scale: AxisScale, range: (f64, f64)) -> Vec<f64> {
        match scale {
            AxisScale::Linear => {
                let (min, max) = range;
                let step = (max - min) / 3.0;
                vec![min, min + step, min + step * 2.0, max]
            }
            AxisScale::Log10 => Self::log_ticks(range),
        }
    }

    fn log_ticks(range: (f64, f64)) -> Vec<f64> {
        let (min, max) = match Self::transformed_range(AxisScale::Log10, range) {
            Some((min, max)) => (10f64.powf(min), 10f64.powf(max)),
            None => return Vec::new(),
        };

        let min_exp = min.log10().floor() as i32;
        let max_exp = max.log10().ceil() as i32;
        let powers: Vec<f64> = (min_exp..=max_exp)
            .map(|exp| 10f64.powi(exp))
            .filter(|value| *value >= min && *value <= max)
            .collect();

        if powers.len() >= 2 {
            return Self::downsample_ticks(&powers, 5);
        }

        let min_t = min.log10();
        let max_t = max.log10();
        let step = (max_t - min_t) / 3.0;
        let ticks: Vec<f64> = (0..=3)
            .map(|i| 10f64.powf(min_t + step * i as f64))
            .collect();

        Self::dedup_ticks(ticks)
    }

    fn downsample_ticks(ticks: &[f64], max_ticks: usize) -> Vec<f64> {
        if ticks.len() <= max_ticks {
            return ticks.to_vec();
        }

        let last_index = ticks.len() - 1;
        let sampled: Vec<f64> = (0..max_ticks)
            .map(|i| {
                let ratio = i as f64 / (max_ticks - 1) as f64;
                let index = (ratio * last_index as f64).round() as usize;
                ticks[index]
            })
            .collect();

        Self::dedup_ticks(sampled)
    }

    fn dedup_ticks(ticks: Vec<f64>) -> Vec<f64> {
        let mut deduped = Vec::with_capacity(ticks.len());
        for tick in ticks {
            let is_duplicate = deduped
                .last()
                .map(|last| (last - tick).abs() < 1e-9)
                .unwrap_or(false);
            if !is_duplicate {
                deduped.push(tick);
            }
        }
        deduped
    }

    pub(super) fn format_tick(scale: AxisScale, value: f64) -> String {
        match scale {
            AxisScale::Linear => format!("{value:.1}"),
            AxisScale::Log10 => Self::format_log_tick(value),
        }
    }

    fn format_log_tick(value: f64) -> String {
        if !value.is_finite() {
            return "NaN".to_string();
        }

        if value <= 0.0 {
            return format!("{value:.1}");
        }

        let exp = value.log10().round() as i32;
        let exact_power = 10f64.powi(exp);

        if (value - exact_power).abs() / exact_power.max(1.0) < 1e-9 {
            return match exp {
                -2 => "0.01".to_string(),
                -1 => "0.1".to_string(),
                0 => "1".to_string(),
                1 => "10".to_string(),
                2 => "100".to_string(),
                _ => format!("1e{exp}"),
            };
        }

        Self::format_compact(value)
    }

    fn format_compact(value: f64) -> String {
        let abs = value.abs();
        let raw = if abs >= 1000.0 || (abs > 0.0 && abs < 0.1) {
            format!("{value:.1e}")
                .replace("e+0", "e")
                .replace("e+", "e")
                .replace("e-0", "e-")
        } else if abs >= 10.0 {
            format!("{value:.1}")
        } else {
            format!("{value:.2}")
        };

        Self::trim_trailing_zeros(raw)
    }

    fn trim_trailing_zeros(mut value: String) -> String {
        if let Some(exp_index) = value.find('e') {
            let exponent = value.split_off(exp_index);
            let trimmed = Self::trim_decimal(value);
            return format!("{trimmed}{exponent}");
        }

        Self::trim_decimal(value)
    }

    fn trim_decimal(mut value: String) -> String {
        if value.contains('.') {
            while value.ends_with('0') {
                value.pop();
            }
            if value.ends_with('.') {
                value.pop();
            }
        }
        value
    }
}
