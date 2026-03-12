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
