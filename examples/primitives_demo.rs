use colored::Color;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write as IoWrite};
use std::time::{Duration, Instant};
use txtplot::{canvas::ColorBlend, ChartContext};

#[derive(Clone, Copy)]
enum ShapeKind {
    RectFilled,
    CircleFilled,
    Line,
    Eraser, // Punches holes through the other shapes using unset_pixel
}

struct BouncingShape {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    size: f64,
    kind: ShapeKind,
    color: Option<Color>,
}

impl BouncingShape {
    fn update(&mut self, w: f64, h: f64) {
        self.x += self.vx;
        self.y += self.vy;

        // Allow shapes to go slightly off-screen to demonstrate clipping
        let bounds_margin = self.size + 10.0;

        if self.x < -bounds_margin || self.x > w + bounds_margin {
            self.vx *= -1.0;
        }
        if self.y < -bounds_margin || self.y > h + bounds_margin {
            self.vy *= -1.0;
        }
    }

    fn draw(&self, chart: &mut ChartContext) {
        let px = self.x as isize;
        let py = self.y as isize;
        let s = self.size as isize;

        match self.kind {
            ShapeKind::RectFilled => {
                // Still draws even if half the shape is off-screen thanks to clipping
                chart.canvas.rect_filled(
                    px - s,
                    py - s,
                    (s * 2) as usize,
                    (s * 2) as usize,
                    self.color,
                );
            }
            ShapeKind::CircleFilled => {
                chart.canvas.circle_filled(px, py, s, self.color);
            }
            ShapeKind::Line => {
                // A long line crossing through the center
                chart.canvas.line_screen(
                    px - s * 2,
                    py - s * 2,
                    px + s * 2,
                    py + s * 2,
                    self.color,
                );
                chart.canvas.line_screen(
                    px - s * 2,
                    py + s * 2,
                    px + s * 2,
                    py - s * 2,
                    self.color,
                );
            }
            ShapeKind::Eraser => {
                // MASK: use unset_pixel to punch through what has already been drawn
                let r2 = s * s;
                for dy in -s..=s {
                    for dx in -s..=s {
                        if dx * dx + dy * dy <= r2 {
                            let ex = px + dx;
                            let ey = py + dy;
                            if ex >= 0 && ey >= 0 {
                                chart.canvas.unset_pixel_screen(ex as usize, ey as usize);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout().lock(); // Lock stdout for maximum throughput
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let (mut cols, mut rows) = terminal::size()?;
    let mut chart = ChartContext::new((cols - 4) as usize, (rows - 4) as usize);

    // Reusable String buffer to avoid per-frame allocations (zero-allocation loop)
    let mut render_buffer = String::with_capacity(cols as usize * rows as usize * 15);

    let mut shapes = vec![
        BouncingShape {
            x: 20.0,
            y: 20.0,
            vx: 1.5,
            vy: 1.1,
            size: 25.0,
            kind: ShapeKind::CircleFilled,
            color: Some(Color::Blue),
        },
        BouncingShape {
            x: 80.0,
            y: 40.0,
            vx: -1.2,
            vy: 1.8,
            size: 20.0,
            kind: ShapeKind::RectFilled,
            color: Some(Color::Red),
        },
        BouncingShape {
            x: 50.0,
            y: 10.0,
            vx: 2.0,
            vy: -1.5,
            size: 15.0,
            kind: ShapeKind::CircleFilled,
            color: Some(Color::Green),
        },
        BouncingShape {
            x: 10.0,
            y: 60.0,
            vx: 2.5,
            vy: 0.5,
            size: 30.0,
            kind: ShapeKind::Line,
            color: Some(Color::BrightYellow),
        },
        // THE ERASER: punches transparent circles through the earlier shapes
        BouncingShape {
            x: 60.0,
            y: 30.0,
            vx: -1.0,
            vy: -1.0,
            size: 12.0,
            kind: ShapeKind::Eraser,
            color: None,
        },
    ];

    let mut running = true;
    let mut blend_mode = ColorBlend::Overwrite;
    let mut frame_count = 0;
    let mut last_fps_time = Instant::now();
    let mut current_fps = 0;

    while running {
        // --- 1. FPS CALCULATION ---
        frame_count += 1;
        if last_fps_time.elapsed().as_secs() >= 1 {
            current_fps = frame_count;
            frame_count = 0;
            last_fps_time = Instant::now();
        }

        // --- 2. RESIZE ---
        let (nc, nr) = terminal::size()?;
        if nc != cols || nr != rows {
            cols = nc;
            rows = nr;
            chart = ChartContext::new((cols - 4) as usize, (rows - 4) as usize);
        }

        // --- 3. INPUT ---
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    KeyCode::Char('b') => {
                        // Change the color blending policy in real time
                        blend_mode = match blend_mode {
                            ColorBlend::Overwrite => ColorBlend::KeepFirst,
                            ColorBlend::KeepFirst => ColorBlend::Overwrite,
                        };
                    }
                    _ => {}
                }
            }
        }

        // --- 4. LOGIC AND DRAWING ---
        chart.canvas.clear();
        chart.canvas.blend_mode = blend_mode; // Apply the selected mode

        let pw = chart.canvas.pixel_width() as f64;
        let ph = chart.canvas.pixel_height() as f64;

        // Draw a background grid so the eraser effect is easier to read
        chart.draw_grid(10, 5, Some(Color::BrightBlack));

        for shape in &mut shapes {
            shape.update(pw, ph);
            shape.draw(&mut chart);
        }

        // --- 5. HUD ---
        let blend_str = match blend_mode {
            ColorBlend::Overwrite => "OVERWRITE (Latest Wins)",
            ColorBlend::KeepFirst => "KEEP_FIRST (First Wins)",
        };
        let hud_text = format!(
            " FPS: {} | Blend [B]: {} | Quit [Q] ",
            current_fps, blend_str
        );
        chart.text(&hud_text, 0.02, 0.02, Some(Color::White));

        // --- 6. OPTIMIZED ZERO-ALLOCATION RENDERING ---
        execute!(stdout, cursor::MoveTo(0, 0))?;

        render_buffer.clear(); // Clear the buffer but KEEP the allocated memory

        // Write directly into the String buffer
        chart
            .canvas
            .render_to(
                &mut render_buffer,
                true,
                Some("TXTPLOT PRIMITIVES & BLENDING"),
            )
            .expect("Failed to format canvas");

        // Flush to the terminal while adapting line endings for raw mode
        write!(stdout, "{}", render_buffer.replace('\n', "\r\n"))?;
        stdout.flush()?;

        // Frame rate control (~60 FPS)
        std::thread::sleep(Duration::from_millis(16));
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
