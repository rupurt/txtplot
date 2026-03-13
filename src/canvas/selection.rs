use super::{BrailleRenderer, CellRenderer, HalfBlockRenderer, QuadrantRenderer};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RendererKind {
    #[default]
    Braille,
    HalfBlock,
    Quadrant,
}

impl RendererKind {
    pub const ALL: [Self; 3] = [Self::Braille, Self::HalfBlock, Self::Quadrant];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Braille => "braille",
            Self::HalfBlock => "halfblock",
            Self::Quadrant => "quadrant",
        }
    }

    pub const fn cell_dimensions(self) -> (usize, usize) {
        match self {
            Self::Braille => (BrailleRenderer::CELL_WIDTH, BrailleRenderer::CELL_HEIGHT),
            Self::HalfBlock => (
                HalfBlockRenderer::CELL_WIDTH,
                HalfBlockRenderer::CELL_HEIGHT,
            ),
            Self::Quadrant => (QuadrantRenderer::CELL_WIDTH, QuadrantRenderer::CELL_HEIGHT),
        }
    }

    pub const fn pixel_dimensions(self, width: usize, height: usize) -> (usize, usize) {
        let (cell_width, cell_height) = self.cell_dimensions();
        (width * cell_width, height * cell_height)
    }
}

impl fmt::Display for RendererKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for RendererKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "braille" | "b" => Ok(Self::Braille),
            "halfblock" | "half-block" | "half_block" | "half" | "hb" => Ok(Self::HalfBlock),
            "quadrant" | "quad" | "q" => Ok(Self::Quadrant),
            _ => Err("unknown renderer"),
        }
    }
}

#[macro_export]
macro_rules! with_renderer {
    ($kind:expr, |$renderer:ident| $body:block) => {{
        match $kind {
            $crate::RendererKind::Braille => {
                type $renderer = $crate::BrailleRenderer;
                $body
            }
            $crate::RendererKind::HalfBlock => {
                type $renderer = $crate::HalfBlockRenderer;
                $body
            }
            $crate::RendererKind::Quadrant => {
                type $renderer = $crate::QuadrantRenderer;
                $body
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::RendererKind;
    use crate::CellCanvas;

    #[test]
    fn renderer_kind_parses_aliases() {
        assert_eq!("braille".parse::<RendererKind>(), Ok(RendererKind::Braille));
        assert_eq!(
            "half-block".parse::<RendererKind>(),
            Ok(RendererKind::HalfBlock)
        );
        assert_eq!("quad".parse::<RendererKind>(), Ok(RendererKind::Quadrant));
    }

    #[test]
    fn renderer_kind_reports_pixel_dimensions() {
        assert_eq!(RendererKind::Braille.pixel_dimensions(3, 2), (6, 8));
        assert_eq!(RendererKind::HalfBlock.pixel_dimensions(3, 2), (3, 4));
        assert_eq!(RendererKind::Quadrant.pixel_dimensions(3, 2), (6, 4));
    }

    #[test]
    fn with_renderer_dispatches_to_concrete_canvas_type() {
        let pixel_height = with_renderer!(RendererKind::HalfBlock, |Renderer| {
            CellCanvas::<Renderer>::new(4, 3).pixel_height()
        });

        assert_eq!(pixel_height, 6);
    }
}
