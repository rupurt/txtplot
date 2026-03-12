use colored::Color;
use std::fmt::{self, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorBlend {
    /// Overwrites the cell's previous color.
    Overwrite,
    /// Keeps the first color assigned to the cell (does not overwrite it).
    KeepFirst,
}

pub struct BrailleCanvas {
    pub width: usize,
    pub height: usize,
    pub blend_mode: ColorBlend,
    plot_left_inset_px: usize,
    plot_bottom_inset_px: usize,
    buffer: Vec<u8>,
    colors: Vec<Option<Color>>,
    text_layer: Vec<Option<char>>,
}

impl BrailleCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            blend_mode: ColorBlend::Overwrite,
            plot_left_inset_px: 0,
            plot_bottom_inset_px: 0,
            buffer: vec![0u8; size],
            colors: vec![None; size],
            text_layer: vec![None; size],
        }
    }

    #[inline]
    pub fn pixel_width(&self) -> usize {
        self.width * 2
    }

    #[inline]
    pub fn pixel_height(&self) -> usize {
        self.height * 4
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
        self.colors.fill(None);
        self.text_layer.fill(None);
        self.plot_left_inset_px = 0;
        self.plot_bottom_inset_px = 0;
    }

    pub fn set_plot_insets(&mut self, left_px: usize, bottom_px: usize) {
        self.plot_left_inset_px = left_px.min(self.pixel_width().saturating_sub(1));
        self.plot_bottom_inset_px = bottom_px.min(self.pixel_height().saturating_sub(1));
    }

    pub fn plot_insets(&self) -> (usize, usize) {
        (self.plot_left_inset_px, self.plot_bottom_inset_px)
    }

    pub(crate) fn cell_masks(&self) -> &[u8] {
        &self.buffer
    }

    /// Replaces entire cells whenever `top` has content. This allows
    /// curves to sit above a grid without mixing both glyphs.
    pub fn overlay(&mut self, top: &BrailleCanvas) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");

        for idx in 0..self.buffer.len() {
            if top.buffer[idx] != 0 || top.text_layer[idx].is_some() {
                self.buffer[idx] = top.buffer[idx];
                self.colors[idx] = top.colors[idx];
                self.text_layer[idx] = top.text_layer[idx];
            }
        }
    }

    pub(crate) fn merge(&mut self, top: &BrailleCanvas) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");

        for idx in 0..self.buffer.len() {
            if top.buffer[idx] != 0 {
                self.buffer[idx] |= top.buffer[idx];
                if top.colors[idx].is_some() {
                    self.colors[idx] = top.colors[idx];
                }
            }

            if let Some(ch) = top.text_layer[idx] {
                self.text_layer[idx] = Some(ch);
                if top.colors[idx].is_some() {
                    self.colors[idx] = top.colors[idx];
                }
            }
        }
    }

    pub(crate) fn overlay_without_background(
        &mut self,
        top: &BrailleCanvas,
        background_mask: &[u8],
    ) {
        assert_eq!(self.width, top.width, "canvas width mismatch");
        assert_eq!(self.height, top.height, "canvas height mismatch");
        assert_eq!(
            self.buffer.len(),
            background_mask.len(),
            "background mask size mismatch"
        );

        for (idx, background) in background_mask
            .iter()
            .copied()
            .enumerate()
            .take(self.buffer.len())
        {
            if top.buffer[idx] == 0 && top.text_layer[idx].is_none() {
                continue;
            }

            let preserved_color = self.colors[idx];
            let existing_foreground = self.buffer[idx] & !background;

            self.buffer[idx] &= !background;
            self.buffer[idx] |= top.buffer[idx];

            if let Some(ch) = top.text_layer[idx] {
                self.text_layer[idx] = Some(ch);
            }

            let keep_existing_color = top.text_layer[idx].is_none()
                && existing_foreground != 0
                && preserved_color.is_some()
                && existing_foreground.count_ones() >= top.buffer[idx].count_ones();

            if keep_existing_color {
                self.colors[idx] = preserved_color;
            } else if top.colors[idx].is_some() || self.buffer[idx] == 0 {
                self.colors[idx] = top.colors[idx];
            }
        }
    }

    // --- Coordinate Helpers ---

    #[inline]
    fn idx(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    #[inline]
    fn get_mask(sub_x: usize, sub_y: usize) -> u8 {
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

    fn set_pixel_impl(&mut self, px: usize, py: usize, color: Option<Color>) {
        if px >= self.pixel_width() || py >= self.pixel_height() {
            return;
        }

        let index = self.idx(px / 2, py / 4);
        self.buffer[index] |= Self::get_mask(px % 2, py % 4);

        if let Some(c) = color {
            match self.blend_mode {
                ColorBlend::Overwrite => self.colors[index] = Some(c),
                ColorBlend::KeepFirst => {
                    if self.colors[index].is_none() {
                        self.colors[index] = Some(c);
                    }
                }
            }
        }
    }

    fn unset_pixel_impl(&mut self, px: usize, py: usize) {
        if px >= self.pixel_width() || py >= self.pixel_height() {
            return;
        }
        let index = self.idx(px / 2, py / 4);
        self.buffer[index] &= !Self::get_mask(px % 2, py % 4);
        if self.buffer[index] == 0 {
            self.colors[index] = None;
        }
    }

    // --- Basic Public Drawing API ---

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Option<Color>) {
        let inverted_y = self.pixel_height().saturating_sub(1).saturating_sub(y);
        self.set_pixel_impl(x, inverted_y, color);
    }

    pub fn set_pixel_screen(&mut self, x: usize, y: usize, color: Option<Color>) {
        self.set_pixel_impl(x, y, color);
    }

    pub fn unset_pixel(&mut self, x: usize, y: usize) {
        let inverted_y = self.pixel_height().saturating_sub(1).saturating_sub(y);
        self.unset_pixel_impl(x, inverted_y);
    }

    pub fn unset_pixel_screen(&mut self, x: usize, y: usize) {
        self.unset_pixel_impl(x, y);
    }

    pub fn toggle_pixel_screen(&mut self, x: usize, y: usize, color: Option<Color>) {
        if x >= self.pixel_width() || y >= self.pixel_height() {
            return;
        }
        let index = self.idx(x / 2, y / 4);
        let mask = Self::get_mask(x % 2, y % 4);

        if (self.buffer[index] & mask) != 0 {
            self.unset_pixel_impl(x, y);
        } else {
            self.set_pixel_impl(x, y, color);
        }
    }

    // --- Clipped Primitives (Cohen-Sutherland) ---

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

        // Cohen-Sutherland Clipping
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
                    x = x0 + (x1 - x0) * (0 - y0) / (y1 - y0);
                    y = 0;
                } else if outcode_out & 2 != 0 {
                    y = y0 + (y1 - y0) * (w - 1 - x0) / (x1 - x0);
                    x = w - 1;
                } else if outcode_out & 1 != 0 {
                    y = y0 + (y1 - y0) * (0 - x0) / (x1 - x0);
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

    // --- Full 2D Primitives ---

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
                d = d + 4 * (x - y) + 10;
            } else {
                d = d + 4 * x + 6;
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
                d = d + 4 * (x - y) + 10;
            } else {
                d = d + 4 * x + 6;
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

    // --- Optimized Rendering (Zero Allocation per Frame) ---

    /// Static helper to avoid allocating `colored` strings in the standard format.
    fn write_ansi_color<W: Write>(w: &mut W, color: Color) -> fmt::Result {
        match color {
            Color::Black => w.write_str("\x1b[30m"),
            Color::Red => w.write_str("\x1b[31m"),
            Color::Green => w.write_str("\x1b[32m"),
            Color::Yellow => w.write_str("\x1b[33m"),
            Color::Blue => w.write_str("\x1b[34m"),
            Color::Magenta => w.write_str("\x1b[35m"),
            Color::Cyan => w.write_str("\x1b[36m"),
            Color::White => w.write_str("\x1b[37m"),
            Color::BrightBlack => w.write_str("\x1b[90m"),
            Color::BrightRed => w.write_str("\x1b[91m"),
            Color::BrightGreen => w.write_str("\x1b[92m"),
            Color::BrightYellow => w.write_str("\x1b[93m"),
            Color::BrightBlue => w.write_str("\x1b[94m"),
            Color::BrightMagenta => w.write_str("\x1b[95m"),
            Color::BrightCyan => w.write_str("\x1b[96m"),
            Color::BrightWhite => w.write_str("\x1b[97m"),
            Color::TrueColor { r, g, b } => write!(w, "\x1b[38;2;{};{};{}m", r, g, b),
        }
    }

    pub fn render_to<W: Write>(
        &self,
        w: &mut W,
        show_border: bool,
        title: Option<&str>,
    ) -> fmt::Result {
        if let Some(t) = title {
            writeln!(w, "{:^width$}", t, width = self.width + 2)?;
        }

        if show_border {
            w.write_char('┌')?;
            for _ in 0..self.width {
                w.write_char('─')?;
            }
            w.write_char('┐')?;
            w.write_char('\n')?;
        }

        let mut last_color: Option<Color> = None;

        for row in 0..self.height {
            if show_border {
                w.write_char('│')?;
            }

            for col in 0..self.width {
                let idx = self.idx(col, row);

                let char_to_print = if let Some(c) = self.text_layer[idx] {
                    c
                } else {
                    let mask = self.buffer[idx];
                    std::char::from_u32(0x2800 + mask as u32).unwrap_or(' ')
                };

                let current_color = self.colors[idx];

                if current_color != last_color {
                    match current_color {
                        Some(c) => Self::write_ansi_color(w, c)?,
                        None => w.write_str("\x1b[0m")?,
                    }
                    last_color = current_color;
                }

                w.write_char(char_to_print)?;
            }

            if last_color.is_some() {
                w.write_str("\x1b[0m")?;
                last_color = None;
            }

            if show_border {
                w.write_char('│')?;
            }
            w.write_char('\n')?;
        }

        if show_border {
            w.write_char('└')?;
            for _ in 0..self.width {
                w.write_char('─')?;
            }
            w.write_char('┘')?;
        }

        Ok(())
    }

    pub fn render_with_options(&self, show_border: bool, title: Option<&str>) -> String {
        let mut out = String::with_capacity(self.width * self.height * 2 + 100);
        let _ = self.render_to(&mut out, show_border, title);
        out
    }

    pub fn render(&self) -> String {
        self.render_with_options(true, None)
    }

    pub fn render_no_color(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height + self.height);
        for row in 0..self.height {
            for col in 0..self.width {
                let mask = self.buffer[self.idx(col, row)];
                out.push(std::char::from_u32(0x2800 + mask as u32).unwrap_or(' '));
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::BrailleCanvas;
    use colored::Color;

    #[test]
    fn overlay_replaces_existing_braille_cells() {
        let mut background = BrailleCanvas::new(1, 1);
        background.line_screen(0, 0, 0, 3, None);

        let mut foreground = BrailleCanvas::new(1, 1);
        foreground.line_screen(0, 0, 1, 0, None);

        let mut direct_merge = BrailleCanvas::new(1, 1);
        direct_merge.line_screen(0, 0, 0, 3, None);
        direct_merge.line_screen(0, 0, 1, 0, None);

        let foreground_render = foreground.render_no_color();
        background.overlay(&foreground);

        assert_eq!(background.render_no_color(), foreground_render);
        assert_ne!(background.render_no_color(), direct_merge.render_no_color());
    }

    #[test]
    fn overlay_without_background_prefers_foreground_over_background() {
        let mut background = BrailleCanvas::new(1, 1);
        background.line(0, 0, 1, 0, Some(Color::White));
        let mask = background.cell_masks().to_vec();

        let mut foreground = BrailleCanvas::new(1, 1);
        foreground.line(0, 0, 1, 3, Some(Color::Cyan));

        background.overlay_without_background(&foreground, &mask);

        assert_eq!(background.colors[0], Some(Color::Cyan));
        assert_eq!(background.render_no_color(), foreground.render_no_color());
    }

    #[test]
    fn overlay_without_background_keeps_first_foreground_color() {
        let mut canvas = BrailleCanvas::new(1, 1);
        canvas.line_screen(0, 0, 0, 3, Some(Color::Green));

        let mut overlay = BrailleCanvas::new(1, 1);
        overlay.line_screen(1, 0, 1, 3, Some(Color::Magenta));

        canvas.overlay_without_background(&overlay, &[0]);

        assert_eq!(canvas.colors[0], Some(Color::Green));
        assert_eq!(canvas.render_no_color(), "⣿\n");
    }
}
