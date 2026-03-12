use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;
use txtplot::ChartContext;

// --- SPRITE ENGINE ---

#[derive(Clone)]
struct Sprite {
    width: usize,
    _height: usize,
    data: Vec<u8>, // 1 = pixel on, 0 = off
    color: Color,
}

impl Sprite {
    // Create a sprite from a visual ASCII string
    // '#' = pixel, ' ' = empty
    fn new(width: usize, height: usize, art: &str, color: Color) -> Self {
        let mut data = Vec::new();
        for char in art.chars() {
            if char == '#' {
                data.push(1);
            } else if char == '.' || char == ' ' {
                data.push(0);
            }
        }
        Self {
            width,
            _height: height,
            data,
            color,
        }
    }

    fn draw(&self, chart: &mut ChartContext, x_pos: f64, y_pos: f64) {
        let start_x = x_pos.round() as isize;
        let start_y = y_pos.round() as isize;

        for (i, &pixel) in self.data.iter().enumerate() {
            if pixel == 1 {
                // Compute local coordinates
                let lx = (i % self.width) as isize;
                let ly = (i / self.width) as isize;

                // Global screen coordinate
                let gx = start_x + lx;
                let gy = start_y + ly;

                // Draw using screen coordinates (0,0 top-left)
                // Basic bounds check (set_pixel_screen already protects us somewhat)
                if gx >= 0 && gy >= 0 {
                    chart
                        .canvas
                        .set_pixel_screen(gx as usize, gy as usize, Some(self.color));
                }
            }
        }
    }
}

// --- GAME ---

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let (mut cols, mut rows) = terminal::size()?;
    let width = (cols as usize).saturating_sub(4);
    let height = (rows as usize).saturating_sub(4);
    let mut chart = ChartContext::new(width, height);

    // 1. SPRITE DEFINITIONS (plain text art)

    // Ship (11x7 px)
    let ship_art = ".....#.....\
         ....###....\
         ....#.#....\
         ...##.##...\
         ..#######..\
         .##.###.##.\
         ##.......##";
    let ship_sprite = Sprite::new(11, 7, ship_art, Color::Cyan);

    // Alien Frame 1 (11x8 px)
    let alien1_art = "..#.....#..\
         ...#...#...\
         ..#######..\
         .##.###.##.\
         ###########\
         #.#######.#\
         #.#.....#.#\
         ...##.##...";
    let alien1 = Sprite::new(11, 8, alien1_art, Color::Green);

    // Alien Frame 2 (arms down)
    let alien2_art = "...#...#...\
         ..##...##..\
         ..#######..\
         .##.###.##.\
         ###########\
         #.#######.#\
         #.#.....#.#\
         ..##...##..";
    let alien2 = Sprite::new(11, 8, alien2_art, Color::BrightGreen);

    // Projectile
    let bullet_sprite = Sprite::new(1, 3, "#|#", Color::Yellow);

    // Game state
    let mut ship_x = (chart.canvas.pixel_width() / 2) as f64;
    let ship_y = (chart.canvas.pixel_height() - 10) as f64;

    let mut aliens_x = 10.0;
    let mut aliens_dir = 1.0;

    struct Bullet {
        x: f64,
        y: f64,
        active: bool,
    }
    let mut bullets: Vec<Bullet> = Vec::new();

    let mut running = true;
    let mut frame_count = 0;

    while running {
        // Resize check
        let (nc, nr) = terminal::size()?;
        if nc != cols || nr != rows {
            cols = nc;
            rows = nr;
            let w = (cols as usize).saturating_sub(4);
            let h = (rows as usize).saturating_sub(4);
            chart = ChartContext::new(w, h);
        } else {
            chart.canvas.clear();
        }

        // Input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    KeyCode::Left => ship_x -= 3.0, // Fast movement
                    KeyCode::Right => ship_x += 3.0,
                    KeyCode::Char(' ') => {
                        bullets.push(Bullet {
                            x: ship_x + 5.0, // Center of the ship
                            y: ship_y,
                            active: true,
                        });
                    }
                    _ => {}
                }
            }
        }

        // --- LOGIC ---

        // Move aliens
        aliens_x += aliens_dir * 0.5;
        if aliens_x > (chart.canvas.pixel_width() - 50) as f64 || aliens_x < 2.0 {
            aliens_dir *= -1.0;
        }

        // Move bullets
        for b in bullets.iter_mut() {
            if b.active {
                b.y -= 2.0; // Bullet speed
                if b.y < 0.0 {
                    b.active = false;
                }
            }
        }
        bullets.retain(|b| b.active);

        // --- RENDERING ---

        // 1. Draw bullets
        for b in &bullets {
            bullet_sprite.draw(&mut chart, b.x, b.y);
        }

        // 2. Draw player ship
        ship_sprite.draw(&mut chart, ship_x, ship_y);

        // 3. Draw the alien horde (animated)
        let current_alien = if (frame_count / 30) % 2 == 0 {
            &alien1
        } else {
            &alien2
        };

        for i in 0..5 {
            for j in 0..3 {
                let ax = aliens_x + (i as f64 * 14.0);
                let ay = 10.0 + (j as f64 * 10.0);
                current_alien.draw(&mut chart, ax, ay);
            }
        }

        // Output
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart
            .canvas
            .render_with_options(true, Some("SPRITE ENGINE DEMO (Left/Right/Space)"));
        print!("{}", output.replace('\n', "\r\n"));
        io::stdout().flush()?;

        frame_count += 1;
        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
