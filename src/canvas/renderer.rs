use super::ColorBlend;
use colored::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellAppearance {
    pub glyph: char,
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

pub trait CellRenderer {
    type Cell: Copy + Default + PartialEq + Eq;

    const CELL_WIDTH: usize;
    const CELL_HEIGHT: usize;

    fn set_subpixel(cell: &mut Self::Cell, sub_x: usize, sub_y: usize);
    fn unset_subpixel(cell: &mut Self::Cell, sub_x: usize, sub_y: usize);
    fn is_subpixel_set(cell: Self::Cell, sub_x: usize, sub_y: usize) -> bool;

    fn merge_cell(cell: &mut Self::Cell, top: Self::Cell);
    fn subtract_mask(cell: &mut Self::Cell, mask: Self::Cell);
    fn without_mask(cell: Self::Cell, mask: Self::Cell) -> Self::Cell;
    fn subpixel_count(cell: Self::Cell) -> u32;
    fn is_empty(cell: Self::Cell) -> bool;
    fn glyph(cell: Self::Cell) -> char;

    fn apply_subpixel(
        cell: &mut Self::Cell,
        sub_x: usize,
        sub_y: usize,
        color: Option<Color>,
        blend_mode: ColorBlend,
        foreground: &mut Option<Color>,
        _background: &mut Option<Color>,
    ) {
        Self::set_subpixel(cell, sub_x, sub_y);

        if let Some(color) = color {
            match blend_mode {
                ColorBlend::Overwrite => *foreground = Some(color),
                ColorBlend::KeepFirst => {
                    if foreground.is_none() {
                        *foreground = Some(color);
                    }
                }
            }
        }
    }

    fn clear_subpixel(
        cell: &mut Self::Cell,
        sub_x: usize,
        sub_y: usize,
        foreground: &mut Option<Color>,
        _background: &mut Option<Color>,
    ) {
        Self::unset_subpixel(cell, sub_x, sub_y);
        if Self::is_empty(*cell) {
            *foreground = None;
        }
    }

    fn appearance(
        cell: Self::Cell,
        foreground: Option<Color>,
        background: Option<Color>,
        text: Option<char>,
    ) -> CellAppearance {
        let glyph = text.unwrap_or_else(|| {
            if background.is_some() && Self::is_empty(cell) {
                ' '
            } else {
                Self::glyph(cell)
            }
        });

        CellAppearance {
            glyph,
            foreground,
            background,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct BrailleRenderer;

impl BrailleRenderer {
    fn mask(sub_x: usize, sub_y: usize) -> u8 {
        match (sub_x, sub_y) {
            (0, 0) => 0x01,
            (1, 0) => 0x08,
            (0, 1) => 0x02,
            (1, 1) => 0x10,
            (0, 2) => 0x04,
            (1, 2) => 0x20,
            (0, 3) => 0x40,
            (1, 3) => 0x80,
            _ => 0,
        }
    }
}

impl CellRenderer for BrailleRenderer {
    type Cell = u8;

    const CELL_WIDTH: usize = 2;
    const CELL_HEIGHT: usize = 4;

    fn set_subpixel(cell: &mut Self::Cell, sub_x: usize, sub_y: usize) {
        *cell |= Self::mask(sub_x, sub_y);
    }

    fn unset_subpixel(cell: &mut Self::Cell, sub_x: usize, sub_y: usize) {
        *cell &= !Self::mask(sub_x, sub_y);
    }

    fn is_subpixel_set(cell: Self::Cell, sub_x: usize, sub_y: usize) -> bool {
        (cell & Self::mask(sub_x, sub_y)) != 0
    }

    fn merge_cell(cell: &mut Self::Cell, top: Self::Cell) {
        *cell |= top;
    }

    fn subtract_mask(cell: &mut Self::Cell, mask: Self::Cell) {
        *cell &= !mask;
    }

    fn without_mask(cell: Self::Cell, mask: Self::Cell) -> Self::Cell {
        cell & !mask
    }

    fn subpixel_count(cell: Self::Cell) -> u32 {
        cell.count_ones()
    }

    fn is_empty(cell: Self::Cell) -> bool {
        cell == 0
    }

    fn glyph(cell: Self::Cell) -> char {
        std::char::from_u32(0x2800 + cell as u32).unwrap_or(' ')
    }
}

#[derive(Clone, Copy, Default)]
pub struct QuadrantRenderer;

impl QuadrantRenderer {
    fn mask(sub_x: usize, sub_y: usize) -> u8 {
        match (sub_x, sub_y) {
            (0, 0) => 0x01,
            (1, 0) => 0x02,
            (0, 1) => 0x04,
            (1, 1) => 0x08,
            _ => 0,
        }
    }

    fn glyph_for_mask(cell: u8) -> char {
        match cell {
            0x00 => ' ',
            0x01 => '▘',
            0x02 => '▝',
            0x03 => '▀',
            0x04 => '▖',
            0x05 => '▌',
            0x06 => '▞',
            0x07 => '▛',
            0x08 => '▗',
            0x09 => '▚',
            0x0A => '▐',
            0x0B => '▜',
            0x0C => '▄',
            0x0D => '▙',
            0x0E => '▟',
            0x0F => '█',
            _ => ' ',
        }
    }
}

impl CellRenderer for QuadrantRenderer {
    type Cell = u8;

    const CELL_WIDTH: usize = 2;
    const CELL_HEIGHT: usize = 2;

    fn set_subpixel(cell: &mut Self::Cell, sub_x: usize, sub_y: usize) {
        *cell |= Self::mask(sub_x, sub_y);
    }

    fn unset_subpixel(cell: &mut Self::Cell, sub_x: usize, sub_y: usize) {
        *cell &= !Self::mask(sub_x, sub_y);
    }

    fn is_subpixel_set(cell: Self::Cell, sub_x: usize, sub_y: usize) -> bool {
        (cell & Self::mask(sub_x, sub_y)) != 0
    }

    fn merge_cell(cell: &mut Self::Cell, top: Self::Cell) {
        *cell |= top;
    }

    fn subtract_mask(cell: &mut Self::Cell, mask: Self::Cell) {
        *cell &= !mask;
    }

    fn without_mask(cell: Self::Cell, mask: Self::Cell) -> Self::Cell {
        cell & !mask
    }

    fn subpixel_count(cell: Self::Cell) -> u32 {
        cell.count_ones()
    }

    fn is_empty(cell: Self::Cell) -> bool {
        cell == 0
    }

    fn glyph(cell: Self::Cell) -> char {
        Self::glyph_for_mask(cell)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct HalfBlockCell {
    mask: u8,
    top: Option<Color>,
    bottom: Option<Color>,
}

#[derive(Clone, Copy, Default)]
pub struct HalfBlockRenderer;

impl HalfBlockRenderer {
    const TOP: u8 = 0x01;
    const BOTTOM: u8 = 0x02;
    const FULL: u8 = Self::TOP | Self::BOTTOM;

    fn mask(sub_y: usize) -> u8 {
        match sub_y {
            0 => Self::TOP,
            1 => Self::BOTTOM,
            _ => 0,
        }
    }

    fn set_half_color(slot: &mut Option<Color>, color: Color, blend_mode: ColorBlend) {
        match blend_mode {
            ColorBlend::Overwrite => *slot = Some(color),
            ColorBlend::KeepFirst => {
                if slot.is_none() {
                    *slot = Some(color);
                }
            }
        }
    }
}

impl CellRenderer for HalfBlockRenderer {
    type Cell = HalfBlockCell;

    const CELL_WIDTH: usize = 1;
    const CELL_HEIGHT: usize = 2;

    fn set_subpixel(cell: &mut Self::Cell, _sub_x: usize, sub_y: usize) {
        cell.mask |= Self::mask(sub_y);
    }

    fn unset_subpixel(cell: &mut Self::Cell, _sub_x: usize, sub_y: usize) {
        match Self::mask(sub_y) {
            Self::TOP => {
                cell.mask &= !Self::TOP;
                cell.top = None;
            }
            Self::BOTTOM => {
                cell.mask &= !Self::BOTTOM;
                cell.bottom = None;
            }
            _ => {}
        }
    }

    fn is_subpixel_set(cell: Self::Cell, _sub_x: usize, sub_y: usize) -> bool {
        (cell.mask & Self::mask(sub_y)) != 0
    }

    fn merge_cell(cell: &mut Self::Cell, top: Self::Cell) {
        if (top.mask & Self::TOP) != 0 {
            cell.mask |= Self::TOP;
            cell.top = top.top;
        }

        if (top.mask & Self::BOTTOM) != 0 {
            cell.mask |= Self::BOTTOM;
            cell.bottom = top.bottom;
        }
    }

    fn subtract_mask(cell: &mut Self::Cell, mask: Self::Cell) {
        if (mask.mask & Self::TOP) != 0 {
            cell.mask &= !Self::TOP;
            cell.top = None;
        }

        if (mask.mask & Self::BOTTOM) != 0 {
            cell.mask &= !Self::BOTTOM;
            cell.bottom = None;
        }
    }

    fn without_mask(cell: Self::Cell, mask: Self::Cell) -> Self::Cell {
        let mut result = cell;
        Self::subtract_mask(&mut result, mask);
        result
    }

    fn subpixel_count(cell: Self::Cell) -> u32 {
        cell.mask.count_ones()
    }

    fn is_empty(cell: Self::Cell) -> bool {
        cell.mask == 0
    }

    fn glyph(cell: Self::Cell) -> char {
        match cell.mask {
            0 => ' ',
            Self::TOP => '▀',
            Self::BOTTOM => '▄',
            Self::FULL => '█',
            _ => ' ',
        }
    }

    fn apply_subpixel(
        cell: &mut Self::Cell,
        _sub_x: usize,
        sub_y: usize,
        color: Option<Color>,
        blend_mode: ColorBlend,
        _foreground: &mut Option<Color>,
        _background: &mut Option<Color>,
    ) {
        Self::set_subpixel(cell, 0, sub_y);

        if let Some(color) = color {
            match Self::mask(sub_y) {
                Self::TOP => Self::set_half_color(&mut cell.top, color, blend_mode),
                Self::BOTTOM => Self::set_half_color(&mut cell.bottom, color, blend_mode),
                _ => {}
            }
        }
    }

    fn clear_subpixel(
        cell: &mut Self::Cell,
        _sub_x: usize,
        sub_y: usize,
        _foreground: &mut Option<Color>,
        _background: &mut Option<Color>,
    ) {
        Self::unset_subpixel(cell, 0, sub_y);
    }

    fn appearance(
        cell: Self::Cell,
        foreground: Option<Color>,
        background: Option<Color>,
        text: Option<char>,
    ) -> CellAppearance {
        if let Some(glyph) = text {
            return CellAppearance {
                glyph,
                foreground,
                background,
            };
        }

        match (
            (cell.mask & Self::TOP) != 0,
            (cell.mask & Self::BOTTOM) != 0,
        ) {
            (false, false) => CellAppearance {
                glyph: ' ',
                foreground: None,
                background,
            },
            (true, false) => CellAppearance {
                glyph: '▀',
                foreground: cell.top.or(foreground),
                background,
            },
            (false, true) => CellAppearance {
                glyph: '▄',
                foreground: cell.bottom.or(foreground),
                background,
            },
            (true, true) => {
                if cell.top.is_some() && cell.bottom.is_some() && cell.top != cell.bottom {
                    CellAppearance {
                        glyph: '▀',
                        foreground: cell.top,
                        background: cell.bottom,
                    }
                } else {
                    CellAppearance {
                        glyph: '█',
                        foreground: cell.top.or(cell.bottom).or(foreground),
                        background: None,
                    }
                }
            }
        }
    }
}
