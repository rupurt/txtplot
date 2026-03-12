use super::{CellCanvas, CellRenderer};
use colored::Color;

impl<R: CellRenderer> CellCanvas<R> {
    pub fn rect(&mut self, x: isize, y: isize, w: usize, h: usize, color: Option<Color>) {
        let x1 = x + w as isize - 1;
        let y1 = y + h as isize - 1;
        self.line_screen(x, y, x1, y, color);
        self.line_screen(x1, y, x1, y1, color);
        self.line_screen(x1, y1, x, y1, color);
        self.line_screen(x, y1, x, y, color);
    }

    pub fn rect_filled(&mut self, x: isize, y: isize, w: usize, h: usize, color: Option<Color>) {
        let max_y = y + h as isize;
        for cy in y..max_y {
            self.line_screen(x, cy, x + w as isize - 1, cy, color);
        }
    }

    pub fn circle(&mut self, xc: isize, yc: isize, r: isize, color: Option<Color>) {
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;

        let mut draw_octants = |cx: isize, cy: isize, x: isize, y: isize| {
            let points = [
                (cx + x, cy + y),
                (cx - x, cy + y),
                (cx + x, cy - y),
                (cx - x, cy - y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx + y, cy - x),
                (cx - y, cy - x),
            ];

            for (px, py) in points {
                if px >= 0 && py >= 0 {
                    self.set_pixel(px as usize, py as usize, color);
                }
            }
        };

        draw_octants(xc, yc, x, y);
        while y >= x {
            x += 1;
            if d > 0 {
                y -= 1;
                d += 4 * (x - y) + 10;
            } else {
                d += 4 * x + 6;
            }
            draw_octants(xc, yc, x, y);
        }
    }

    pub fn circle_filled(&mut self, xc: isize, yc: isize, r: isize, color: Option<Color>) {
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;

        let mut draw_lines = |cx: isize, cy: isize, x: isize, y: isize| {
            self.line(cx - x, cy + y, cx + x, cy + y, color);
            self.line(cx - x, cy - y, cx + x, cy - y, color);
            self.line(cx - y, cy + x, cx + y, cy + x, color);
            self.line(cx - y, cy - x, cx + y, cy - x, color);
        };

        draw_lines(xc, yc, x, y);
        while y >= x {
            x += 1;
            if d > 0 {
                y -= 1;
                d += 4 * (x - y) + 10;
            } else {
                d += 4 * x + 6;
            }
            draw_lines(xc, yc, x, y);
        }
    }

    pub fn set_char(&mut self, col: usize, row: usize, c: char, color: Option<Color>) {
        let inverted_row = self.height.saturating_sub(1).saturating_sub(row);
        if col < self.width && inverted_row < self.height {
            let idx = self.idx(col, inverted_row);
            self.text_layer[idx] = Some(c);
            if let Some(col_val) = color {
                self.colors[idx] = Some(col_val);
            }
        }
    }
}
