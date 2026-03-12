mod axes;
mod overlays;
mod range;
mod series;

#[cfg(test)]
mod tests;

use crate::canvas::{
    BrailleRenderer, CellCanvas, CellRenderer, HalfBlockRenderer, QuadrantRenderer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisScale {
    Linear,
    Log10,
}

impl AxisScale {
    fn transform(self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }

        match self {
            Self::Linear => Some(value),
            Self::Log10 if value > 0.0 => Some(value.log10()),
            Self::Log10 => None,
        }
    }

    fn inverse_transform(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::Log10 => 10f64.powf(value),
        }
    }
}

pub struct CellChartContext<R: CellRenderer> {
    pub canvas: CellCanvas<R>,
    background_mask: Vec<R::Cell>,
    x_scale: AxisScale,
    y_scale: AxisScale,
}

pub type BrailleChartContext = CellChartContext<BrailleRenderer>;
pub type ChartContext = CellChartContext<BrailleRenderer>;
pub type HalfBlockChartContext = CellChartContext<HalfBlockRenderer>;
pub type QuadrantChartContext = CellChartContext<QuadrantRenderer>;

#[derive(Clone, Copy)]
struct PlotGeometry {
    width_px: usize,
    height_px: usize,
    left_inset_px: usize,
    bottom_inset_px: usize,
}

impl PlotGeometry {
    fn from_canvas<R: CellRenderer>(canvas: &CellCanvas<R>) -> Self {
        let (left_inset_px, bottom_inset_px) = canvas.plot_insets();
        Self {
            width_px: canvas.pixel_width(),
            height_px: canvas.pixel_height(),
            left_inset_px,
            bottom_inset_px,
        }
    }
}

#[derive(Clone, Copy)]
struct PlotScales {
    x: AxisScale,
    y: AxisScale,
}

impl CellChartContext<BrailleRenderer> {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_dimensions(width, height)
    }
}

impl<R: CellRenderer> CellChartContext<R> {
    pub fn with_dimensions(width: usize, height: usize) -> Self {
        let canvas = CellCanvas::<R>::new(width, height);
        Self::from_canvas(canvas)
    }

    pub fn from_canvas(canvas: CellCanvas<R>) -> Self {
        let cell_count = canvas.width * canvas.height;
        Self {
            canvas,
            background_mask: vec![R::Cell::default(); cell_count],
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
        }
    }

    pub fn set_x_scale(&mut self, scale: AxisScale) -> &mut Self {
        self.x_scale = scale;
        self
    }

    pub fn set_y_scale(&mut self, scale: AxisScale) -> &mut Self {
        self.y_scale = scale;
        self
    }

    pub fn set_scales(&mut self, x_scale: AxisScale, y_scale: AxisScale) -> &mut Self {
        self.x_scale = x_scale;
        self.y_scale = y_scale;
        self
    }

    pub fn x_scale(&self) -> AxisScale {
        self.x_scale
    }

    pub fn y_scale(&self) -> AxisScale {
        self.y_scale
    }

    fn plot_geometry(&self) -> PlotGeometry {
        PlotGeometry::from_canvas(&self.canvas)
    }

    fn plot_scales(&self) -> PlotScales {
        PlotScales {
            x: self.x_scale,
            y: self.y_scale,
        }
    }
}
