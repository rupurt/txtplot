use colored::Color;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use rand::{rngs::ThreadRng, Rng};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use txtplot::ChartContext;

/// Demo: Fireworks / particle system (PRO autoadjust + user burst control + FPS meter + sleep toggle)
///
/// Controls:
///   - q / Esc : quit
///   - + / =   : increase rockets per launch tick (burst)
///   - -       : decrease burst (min 1)
///   - s       : toggle sleep (ON/OFF) for stress/benchmarking
///
/// Notes:
///   - Coordinates: (0,0) top-left.
///   - FPS meter acts as CPU/terminal stress monitor.

#[derive(Clone, Copy, Debug)]
enum FireworkKind {
    Burst,
    Ring,
    Fountain,
    Willow,
}

#[derive(Clone, Debug)]
struct Rocket {
    x: f64,
    y: f64,
    vx: f64, // px/frame (approx), integrated with *dt*60
    vy: f64, // px/frame
    fuse: f64,
    color: Color,
    kind: FireworkKind,
    trail: VecDeque<(f64, f64)>,
}

#[derive(Clone, Debug)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64, // px/frame
    vy: f64, // px/frame
    life: f64,
    fade: f64,
    color: Color,
    sparkle: bool,
}

fn palette(rng: &mut ThreadRng) -> Color {
    match rng.gen_range(0..9) {
        0 => Color::Red,
        1 => Color::Yellow,
        2 => Color::Green,
        3 => Color::Cyan,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::BrightRed,
        7 => Color::BrightCyan,
        _ => Color::BrightMagenta,
    }
}

fn kind_pick(rng: &mut ThreadRng) -> FireworkKind {
    match rng.gen_range(0..10) {
        0..=4 => FireworkKind::Burst,
        5..=6 => FireworkKind::Ring,
        7..=8 => FireworkKind::Willow,
        _ => FireworkKind::Fountain,
    }
}

fn clamp_to_screen(x: f64, y: f64, w: usize, h: usize) -> Option<(usize, usize)> {
    if x.is_finite() && y.is_finite() {
        let xi = x.round() as isize;
        let yi = y.round() as isize;
        if xi >= 0 && yi >= 0 && (xi as usize) < w && (yi as usize) < h {
            return Some((xi as usize, yi as usize));
        }
    }
    None
}

/// PRO: spawn rocket whose apex lands inside a visible band.
/// Uses "per frame" velocities compatible with: pos += v * dt * 60
fn spawn_rocket_pro(rng: &mut ThreadRng, w_px: usize, h_px: usize) -> Rocket {
    let x = rng.gen_range((w_px as f64) * 0.15..(w_px as f64) * 0.85);
    let y0 = (h_px as f64) - 2.0;

    let color = palette(rng);
    let kind = kind_pick(rng);

    // Lateral drift
    let vx = rng.gen_range(-0.95..0.95);

    // Choose a visible band for the apex (top portion of the screen)
    let band_top = (h_px as f64) * 0.18; // 18%
    let band_bottom = (h_px as f64) * 0.38; // 38%
    let target_apex_y = rng.gen_range(band_top..band_bottom);

    // Reference physics (same as rocket update)
    let dt_assumed = 1.0 / 60.0;
    let gravity = 18.0;
    let g_frame = gravity * dt_assumed * 0.55;

    // dy = y0 - y_apex (positive)
    let mut dy = y0 - target_apex_y;
    dy = dy.clamp((h_px as f64) * 0.20, (h_px as f64) * 0.82);

    // dy = v0^2/(2*g_frame) -> v0 = -sqrt(2*g_frame*dy)
    let mut vy = -((2.0 * g_frame * dy).max(0.001)).sqrt();

    // Small variation
    vy *= rng.gen_range(0.92..1.06);

    // Time to apex: t = |v0|/g_frame
    let t_apex_frames = (vy.abs() / g_frame).max(12.0);
    let t_apex_secs = t_apex_frames / 60.0;

    // Fuse near apex + jitter
    let mut fuse = rng.gen_range(t_apex_secs * 0.82..t_apex_secs * 1.05);

    if matches!(kind, FireworkKind::Willow) {
        fuse *= rng.gen_range(0.85..0.98);
    }

    // Fountain: better near the ground and short fuse
    let mut y = y0;
    if matches!(kind, FireworkKind::Fountain) {
        y = (h_px as f64) - rng.gen_range(1.0..8.0);
        vy = rng.gen_range(-(h_px as f64) * 0.10..-(h_px as f64) * 0.06);
        fuse = rng.gen_range(0.35..0.75);
    }

    Rocket {
        x,
        y,
        vx,
        vy,
        fuse,
        color,
        kind,
        trail: VecDeque::with_capacity(24),
    }
}

fn explode(rng: &mut ThreadRng, rocket: &Rocket, out: &mut Vec<Particle>) {
    let (count, speed_min, speed_max, life_min, life_max, sparkle) = match rocket.kind {
        FireworkKind::Burst => (220, 3.0, 9.0, 0.8, 1.6, true),
        FireworkKind::Ring => (180, 4.0, 7.0, 0.8, 1.4, false),
        FireworkKind::Willow => (200, 2.0, 6.5, 1.6, 2.6, true),
        FireworkKind::Fountain => (120, 2.0, 6.0, 0.7, 1.2, true),
    };

    for i in 0..count {
        let a = rng.gen_range(0.0..std::f64::consts::TAU);
        let mut s = rng.gen_range(speed_min..speed_max);

        let (mut vx, mut vy) = match rocket.kind {
            FireworkKind::Ring => {
                let vx = a.cos() * s;
                let vy = a.sin() * s * 0.35;
                (vx, vy)
            }
            FireworkKind::Willow => {
                s *= rng.gen_range(0.6..1.0);
                let vx = a.cos() * s * 0.8;
                let vy = a.sin() * s * 0.8 - rng.gen_range(1.5..4.0);
                (vx, vy)
            }
            FireworkKind::Fountain => {
                let spread = rng.gen_range(-0.9..0.9);
                let vx = spread * s;
                let vy = -rng.gen_range(4.0..9.0) - rng.gen_range(0.0..2.0);
                (vx, vy)
            }
            FireworkKind::Burst => (a.cos() * s, a.sin() * s),
        };

        // Rocket momentum
        vx += rocket.vx * 0.15;
        vy += rocket.vy * 0.06;

        let is_sparkle = sparkle && (i % 9 == 0 || rng.gen_bool(0.07));
        let life = rng.gen_range(life_min..life_max);

        out.push(Particle {
            x: rocket.x,
            y: rocket.y,
            vx,
            vy,
            life,
            fade: life,
            color: if is_sparkle {
                Color::White
            } else {
                rocket.color
            },
            sparkle: is_sparkle,
        });
    }
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let (mut cols, mut rows) = terminal::size()?;
    let width = (cols as usize).saturating_sub(4);
    let height = (rows as usize).saturating_sub(4);
    let mut chart = ChartContext::new(width, height);

    let mut rng = rand::thread_rng();
    let mut rockets: Vec<Rocket> = Vec::with_capacity(64);
    let mut particles: Vec<Particle> = Vec::with_capacity(64_000);

    let mut running = true;
    let mut last = Instant::now();
    let start = Instant::now();

    // Spawn pacing
    let mut spawn_acc = 0.0;
    let mut auto_intensity = 0.85;

    // User control: rockets per spawn tick
    let mut burst_size: usize = 1;
    let burst_max: usize = 512;

    // Sleep toggle for max stress / benchmarking
    let mut sleep_enabled: bool = true;

    // FPS meter (smoothed)
    let mut fps: f64 = 0.0;
    let mut frame_ms_avg: f64 = 0.0;
    let mut frame_counter: u32 = 0;
    let mut fps_window_start = Instant::now();
    while running {
        let frame_begin = Instant::now();

        // Resize / clear
        let (nc, nr) = terminal::size()?;
        if nc != cols || nr != rows {
            cols = nc;
            rows = nr;
            let w = (cols as usize).saturating_sub(4);
            let h = (rows as usize).saturating_sub(4);
            chart = ChartContext::new(w, h);

            let area = (chart.canvas.pixel_width() * chart.canvas.pixel_height()).max(1) as f64;
            auto_intensity = (area / (180.0 * 60.0)).clamp(0.55, 1.35);
        } else {
            chart.canvas.clear();
        }

        // Input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,

                    // '+' often arrives as '=' in some layouts
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        burst_size = (burst_size + 1).min(burst_max);
                    }
                    KeyCode::Char('-') => {
                        burst_size = burst_size.saturating_sub(1).max(1);
                    }

                    // Toggle sleep (stress / benchmark mode)
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        sleep_enabled = !sleep_enabled;
                    }

                    _ => {}
                }
            }
        }

        // Time step
        let now = Instant::now();
        let mut dt = (now - last).as_secs_f64();
        last = now;
        dt = dt.clamp(0.0, 0.05);

        let w_px = chart.canvas.pixel_width();
        let h_px = chart.canvas.pixel_height();

        // Spawning rockets
        spawn_acc += dt;
        let rate = (1.3 * auto_intensity).clamp(0.8, 2.8);
        while spawn_acc > 1.0 / rate {
            spawn_acc -= 1.0 / rate;

            for _ in 0..burst_size {
                rockets.push(spawn_rocket_pro(&mut rng, w_px, h_px));
            }
        }

        // Physics
        let gravity = 18.0;
        let drag = 0.985;
        let floor = (h_px as f64) - 1.0;

        // Update rockets
        for r in rockets.iter_mut() {
            if r.trail.len() == r.trail.capacity() {
                r.trail.pop_front();
            }
            r.trail.push_back((r.x, r.y));

            r.fuse -= dt;
            r.x += r.vx * dt * 60.0;
            r.y += r.vy * dt * 60.0;

            r.vy += gravity * dt * 0.55;
            r.vx *= drag;
            r.vy *= drag;
        }

        // Explode rockets (failsafe ceiling)
        let mut i = 0;
        let ceiling_margin = (h_px as f64) * 0.06 + 2.0;
        while i < rockets.len() {
            let do_explode =
                rockets[i].fuse <= 0.0 || rockets[i].vy > -1.0 || rockets[i].y <= ceiling_margin;

            if do_explode {
                explode(&mut rng, &rockets[i], &mut particles);
                rockets.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // Update particles
        for p in particles.iter_mut() {
            p.life -= dt;

            p.x += p.vx * dt * 60.0;
            p.y += p.vy * dt * 60.0;
            p.vy += gravity * dt * 0.8;

            p.vx *= drag;
            p.vy *= drag;

            if p.sparkle && rng.gen_bool(0.25) {
                p.vx += rng.gen_range(-0.8..0.8);
                p.vy += rng.gen_range(-0.6..0.6);
            }

            if p.y >= floor {
                p.y = floor;
                p.vy *= -0.35;
                p.vx *= 0.72;
            }
        }
        particles.retain(|p| p.life > 0.0);

        // Draw rockets + trails
        for r in rockets.iter() {
            for (k, (tx, ty)) in r.trail.iter().enumerate() {
                if k % 2 == 0 {
                    continue;
                }
                if let Some((sx, sy)) = clamp_to_screen(*tx, *ty, w_px, h_px) {
                    chart
                        .canvas
                        .set_pixel_screen(sx, sy, Some(Color::BrightBlack));
                }
            }
            if let Some((sx, sy)) = clamp_to_screen(r.x, r.y, w_px, h_px) {
                chart.canvas.set_pixel_screen(sx, sy, Some(r.color));
                if sx + 1 < w_px {
                    chart
                        .canvas
                        .set_pixel_screen(sx + 1, sy, Some(Color::White));
                }
            }
        }

        // Draw particles
        for p in particles.iter() {
            let t = (p.life / p.fade).clamp(0.0, 1.0);
            let col = if p.sparkle {
                if t > 0.6 {
                    Color::BrightWhite
                } else if t > 0.3 {
                    Color::White
                } else {
                    Color::BrightBlack
                }
            } else if t > 0.66 {
                p.color
            } else if t > 0.33 {
                match p.color {
                    Color::Red => Color::BrightRed,
                    Color::Blue => Color::BrightBlue,
                    Color::Green => Color::BrightGreen,
                    Color::Cyan => Color::BrightCyan,
                    Color::Magenta => Color::BrightMagenta,
                    Color::Yellow => Color::BrightYellow,
                    _ => p.color,
                }
            } else {
                Color::BrightBlack
            };

            if let Some((sx, sy)) = clamp_to_screen(p.x, p.y, w_px, h_px) {
                chart.canvas.set_pixel_screen(sx, sy, Some(col));
                if !p.sparkle && rng.gen_bool(0.06) {
                    if sx + 1 < w_px {
                        chart.canvas.set_pixel_screen(sx + 1, sy, Some(col));
                    }
                    if sy + 1 < h_px {
                        chart.canvas.set_pixel_screen(sx, sy + 1, Some(col));
                    }
                }
            }
        }

        // FPS update (windowed + EMA smoothing)
        frame_counter += 1;
        let window_elapsed = fps_window_start.elapsed().as_secs_f64();
        if window_elapsed >= 0.5 {
            let inst_fps = frame_counter as f64 / window_elapsed;
            let alpha = 0.25;
            fps = if fps == 0.0 {
                inst_fps
            } else {
                fps + alpha * (inst_fps - fps)
            };

            fps_window_start = Instant::now();
            frame_counter = 0;
        }

        // Per-frame time (ms)
        let last_frame_ms = frame_begin.elapsed().as_secs_f64() * 1000.0;
        let alpha_ms = 0.12;
        frame_ms_avg = if frame_ms_avg == 0.0 {
            last_frame_ms
        } else {
            frame_ms_avg + alpha_ms * (last_frame_ms - frame_ms_avg)
        };

        // HUD
        let uptime = start.elapsed().as_secs();
        let hud = format!(
            "fps={:5.1} | frame={:5.1}ms avg={:5.1}ms | sleep={} | burst={} | rockets={} particles={} | {:02}:{:02} | +/- burst | s sleep | q/Esc",
            fps,
            last_frame_ms,
            frame_ms_avg,
            if sleep_enabled { "ON " } else { "OFF" },
            burst_size,
            rockets.len(),
            particles.len(),
            uptime / 60,
            uptime % 60
        );
        chart.text(&hud, 0.02, 0.06, Some(Color::White));
        chart.text(
            "Auto-apex band: 18%..38% height | Failsafe ceiling explode | Stress: raise burst; toggle sleep OFF for max load",
            0.02,
            0.12,
            Some(Color::BrightBlack),
        );

        // Output
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart.canvas.render_with_options(
            true,
            Some("PARTICLE SYSTEM — FIREWORKS (PRO + FPS + SLEEP)"),
        );
        print!("{}", output.replace('\n', "\r\n"));
        io::stdout().flush()?;

        // Sleep toggle
        if sleep_enabled {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
