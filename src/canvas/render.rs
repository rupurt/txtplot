use super::{CellCanvas, CellRenderer};
use colored::Color;
use std::fmt::{self, Write};

impl<R: CellRenderer> CellCanvas<R> {
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
                    R::glyph(self.buffer[idx])
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
                out.push(R::glyph(self.buffer[self.idx(col, row)]));
            }
            out.push('\n');
        }
        out
    }
}
