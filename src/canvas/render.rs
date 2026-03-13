use super::{CellCanvas, CellRenderer, TextIntensity};
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

    fn write_ansi_background_color<W: Write>(w: &mut W, color: Color) -> fmt::Result {
        match color {
            Color::Black => w.write_str("\x1b[40m"),
            Color::Red => w.write_str("\x1b[41m"),
            Color::Green => w.write_str("\x1b[42m"),
            Color::Yellow => w.write_str("\x1b[43m"),
            Color::Blue => w.write_str("\x1b[44m"),
            Color::Magenta => w.write_str("\x1b[45m"),
            Color::Cyan => w.write_str("\x1b[46m"),
            Color::White => w.write_str("\x1b[47m"),
            Color::BrightBlack => w.write_str("\x1b[100m"),
            Color::BrightRed => w.write_str("\x1b[101m"),
            Color::BrightGreen => w.write_str("\x1b[102m"),
            Color::BrightYellow => w.write_str("\x1b[103m"),
            Color::BrightBlue => w.write_str("\x1b[104m"),
            Color::BrightMagenta => w.write_str("\x1b[105m"),
            Color::BrightCyan => w.write_str("\x1b[106m"),
            Color::BrightWhite => w.write_str("\x1b[107m"),
            Color::TrueColor { r, g, b } => write!(w, "\x1b[48;2;{};{};{}m", r, g, b),
        }
    }

    fn write_style<W: Write>(
        w: &mut W,
        foreground: Option<Color>,
        background: Option<Color>,
        intensity: TextIntensity,
        last_foreground: &mut Option<Color>,
        last_background: &mut Option<Color>,
        last_intensity: &mut TextIntensity,
    ) -> fmt::Result {
        if foreground == *last_foreground
            && background == *last_background
            && intensity == *last_intensity
        {
            return Ok(());
        }

        let needs_reset = (foreground.is_none() && last_foreground.is_some())
            || (background.is_none() && last_background.is_some());

        if needs_reset {
            w.write_str("\x1b[0m")?;
            if let Some(bg) = background {
                Self::write_ansi_background_color(w, bg)?;
            }
            if let Some(fg) = foreground {
                Self::write_ansi_color(w, fg)?;
            }
            if intensity != TextIntensity::Normal {
                Self::write_ansi_intensity(w, intensity)?;
            }
        } else {
            if background != *last_background {
                if let Some(bg) = background {
                    Self::write_ansi_background_color(w, bg)?;
                }
            }
            if foreground != *last_foreground {
                if let Some(fg) = foreground {
                    Self::write_ansi_color(w, fg)?;
                }
            }
            if intensity != *last_intensity {
                if *last_intensity != TextIntensity::Normal && intensity != TextIntensity::Normal {
                    w.write_str("\x1b[22m")?;
                }
                if intensity == TextIntensity::Normal {
                    w.write_str("\x1b[22m")?;
                } else {
                    Self::write_ansi_intensity(w, intensity)?;
                }
            }
        }

        *last_foreground = foreground;
        *last_background = background;
        *last_intensity = intensity;
        Ok(())
    }

    fn write_ansi_intensity<W: Write>(w: &mut W, intensity: TextIntensity) -> fmt::Result {
        match intensity {
            TextIntensity::Normal => w.write_str("\x1b[22m"),
            TextIntensity::Bold => w.write_str("\x1b[1m"),
            TextIntensity::Dim => w.write_str("\x1b[2m"),
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

        let mut last_foreground: Option<Color> = None;
        let mut last_background: Option<Color> = None;
        let mut last_intensity = TextIntensity::Normal;

        for row in 0..self.height {
            if show_border {
                w.write_char('│')?;
            }

            for col in 0..self.width {
                let idx = self.idx(col, row);
                let appearance = R::appearance(
                    self.buffer[idx],
                    self.colors[idx],
                    self.background_colors[idx],
                    self.text_layer[idx],
                );

                Self::write_style(
                    w,
                    appearance.foreground,
                    appearance.background,
                    self.text_intensity[idx],
                    &mut last_foreground,
                    &mut last_background,
                    &mut last_intensity,
                )?;

                w.write_char(appearance.glyph)?;
            }

            if last_foreground.is_some()
                || last_background.is_some()
                || last_intensity != TextIntensity::Normal
            {
                w.write_str("\x1b[0m")?;
                last_foreground = None;
                last_background = None;
                last_intensity = TextIntensity::Normal;
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
                let idx = self.idx(col, row);
                let glyph = self.text_layer[idx].unwrap_or_else(|| {
                    R::glyph(self.buffer[idx])
                });
                out.push(glyph);
            }
            out.push('\n');
        }
        out
    }
}
