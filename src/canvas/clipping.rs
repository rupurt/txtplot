use super::BrailleCanvas;
use colored::Color;

impl BrailleCanvas {
    fn compute_outcode(&self, x: isize, y: isize) -> u8 {
        let mut code = 0;
        let w = self.pixel_width() as isize;
        let h = self.pixel_height() as isize;

        if x < 0 {
            code |= 1;
        } else if x >= w {
            code |= 2;
        }
        if y < 0 {
            code |= 4;
        } else if y >= h {
            code |= 8;
        }
        code
    }

    fn bresenham(
        &mut self,
        mut x0: isize,
        mut y0: isize,
        mut x1: isize,
        mut y1: isize,
        color: Option<Color>,
        cartesian: bool,
    ) {
        let w = self.pixel_width() as isize;
        let h = self.pixel_height() as isize;

        let mut outcode0 = self.compute_outcode(x0, y0);
        let mut outcode1 = self.compute_outcode(x1, y1);
        let mut accept = false;

        loop {
            if (outcode0 | outcode1) == 0 {
                accept = true;
                break;
            } else if (outcode0 & outcode1) != 0 {
                break;
            } else {
                let outcode_out = if outcode0 != 0 { outcode0 } else { outcode1 };
                let mut x = 0;
                let mut y = 0;

                if outcode_out & 8 != 0 {
                    x = x0 + (x1 - x0) * (h - 1 - y0) / (y1 - y0);
                    y = h - 1;
                } else if outcode_out & 4 != 0 {
                    x = x0 + (x1 - x0) * -y0 / (y1 - y0);
                    y = 0;
                } else if outcode_out & 2 != 0 {
                    y = y0 + (y1 - y0) * (w - 1 - x0) / (x1 - x0);
                    x = w - 1;
                } else if outcode_out & 1 != 0 {
                    y = y0 + (y1 - y0) * -x0 / (x1 - x0);
                    x = 0;
                }

                if outcode_out == outcode0 {
                    x0 = x;
                    y0 = y;
                    outcode0 = self.compute_outcode(x0, y0);
                } else {
                    x1 = x;
                    y1 = y;
                    outcode1 = self.compute_outcode(x1, y1);
                }
            }
        }

        if !accept {
            return;
        }

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if cartesian {
                self.set_pixel(x as usize, y as usize, color);
            } else {
                self.set_pixel_screen(x as usize, y as usize, color);
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, color: Option<Color>) {
        self.bresenham(x0, y0, x1, y1, color, true);
    }

    pub fn line_screen(
        &mut self,
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        color: Option<Color>,
    ) {
        self.bresenham(x0, y0, x1, y1, color, false);
    }
}
