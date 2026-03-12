use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use sysinfo::System; // Reminder: sysinfo 0.30+
use txtplot::ChartContext;

fn main() -> io::Result<()> {
    // 1. Initial setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let mut sys = System::new_all();

    // Graphics buffers
    let history_len = 100;
    let mut cpu_history: VecDeque<f64> = VecDeque::from(vec![0.0; history_len]);

    // Stress configuration
    // "Complexity" = how many trigonometric operations we do per pixel
    let mut complexity: usize = 10;
    let mut time_val: f64 = 0.0;

    let (mut cols, mut rows) = terminal::size()?;
    let width = (cols as usize).saturating_sub(2);
    let height = (rows as usize).saturating_sub(4);

    let mut chart = ChartContext::new(width, height);
    let mut running = true;
    let mut last_sys_update = Instant::now();
    let mut fps = 0;
    let mut frames = 0;
    let mut last_fps_time = Instant::now();

    while running {
        // --- A. SIZE MANAGEMENT ---
        let (nc, nr) = terminal::size()?;
        if nc != cols || nr != rows {
            cols = nc;
            rows = nr;
            let w = (cols as usize).saturating_sub(2);
            let h = (rows as usize).saturating_sub(4);
            chart = ChartContext::new(w, h);
        } else {
            chart.canvas.clear();
        }

        // --- B. INPUT ---
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    // Increase the complexity in large steps
                    KeyCode::Char('+') => complexity = (complexity + 50).min(5000),
                    KeyCode::Char('-') => complexity = complexity.saturating_sub(50).max(1),
                    _ => {}
                }
            }
        }

        // --- C. CPU MONITORING ---
        if last_sys_update.elapsed() >= Duration::from_millis(500) {
            sys.refresh_cpu();
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            cpu_history.pop_front();
            cpu_history.push_back(cpu_usage as f64);
            last_sys_update = Instant::now();
        }

        // --- D. "MATH CORE" RENDERING (bottom half) ---
        let w_px = chart.canvas.pixel_width();
        let h_px = chart.canvas.pixel_height();
        let sim_top = h_px / 2;

        // This is where we burn CPU
        // Iterate over every pixel in the lower half
        for y in sim_top..h_px {
            for x in 0..w_px {
                // Normalized coordinates
                let u = x as f64 * 0.05;
                let v = y as f64 * 0.05;

                let mut val = 0.0;

                // STRESS LOOP: run heavy operations `complexity` times per pixel
                // Rust in release mode will try to optimize this, but because the result depends on `i`,
                // it still has to compute it.
                for i in 0..complexity {
                    // An intentionally expensive function: sines, cosines, roots, and powers
                    let factor = i as f64 * 0.1;
                    val += (u + time_val + factor).sin() * (v + time_val).cos();
                }

                // Normalize the result visually to choose a color
                // (Use modulo to create color bands)
                let color_idx = (val.abs() as usize) % 5;
                let color = match color_idx {
                    0 => Color::Blue,
                    1 => Color::Cyan,
                    2 => Color::Green,
                    3 => Color::Yellow,
                    _ => Color::Red,
                };

                chart.canvas.set_pixel_screen(x, y, Some(color));
            }
        }

        // --- E. TOP CHART (monitor) ---
        // Divider line
        chart.canvas.line_screen(
            0,
            sim_top as isize,
            w_px as isize,
            sim_top as isize,
            Some(Color::White),
        );

        // Draw the CPU chart
        let chart_w = w_px as f64;
        let chart_h = sim_top as f64;
        let step = chart_w / cpu_history.len() as f64;

        let mut prev_x = 0.0;
        let mut prev_y = chart_h - (cpu_history[0] / 100.0 * (chart_h - 2.0));

        for (i, &cpu) in cpu_history.iter().enumerate().skip(1) {
            let curr_x = i as f64 * step;
            let curr_y = chart_h - (cpu / 100.0 * (chart_h - 2.0));

            // Line color changes with load
            let line_col = if cpu > 80.0 {
                Color::Red
            } else if cpu > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            chart.canvas.line_screen(
                prev_x as isize,
                prev_y as isize,
                curr_x as isize,
                curr_y as isize,
                Some(line_col),
            );
            prev_x = curr_x;
            prev_y = curr_y;
        }

        // Text
        let cpu_val = cpu_history.back().unwrap_or(&0.0);
        let cpu_txt = format!("CPU USAGE: {:.1}%", cpu_val);
        chart.text(&cpu_txt, 0.02, 0.05, Some(Color::White));

        let stress_txt = format!("MATH LOAD: {} ops/pixel", complexity);
        chart.text(&stress_txt, 0.02, 0.55, Some(Color::BrightRed));

        let fps_txt = format!("FPS: {}", fps);
        chart.text(&fps_txt, 0.85, 0.05, Some(Color::Cyan));

        // Render
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart
            .canvas
            .render_with_options(true, Some("CPU TORTURE TEST"));
        print!("{}", output.replace('\n', "\r\n"));
        io::stdout().flush()?;

        // Time management
        time_val += 0.1;
        frames += 1;
        if last_fps_time.elapsed() >= Duration::from_secs(1) {
            fps = frames;
            frames = 0;
            last_fps_time = Instant::now();
        }

        // Do not sleep the thread so it can run as fast as possible
        // If you want to simulate a steady 60 FPS and see whether CPU usage rises, uncomment:
        // let frame_time = frame_start.elapsed();
        // if frame_time < Duration::from_millis(16) {
        //    std::thread::sleep(Duration::from_millis(16) - frame_time);
        // }
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
