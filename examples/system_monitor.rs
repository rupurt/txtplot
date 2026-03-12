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
// FIX: CpuExt and SystemExt were removed; they no longer exist in sysinfo 0.30+
use sysinfo::System;
use txtplot::ChartContext;

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    color: Color,
}

fn main() -> io::Result<()> {
    // 1. Initial setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    // Initialize the monitoring system
    let mut sys = System::new_all();

    // History buffers for the charts
    let history_len = 100;
    let mut cpu_history: VecDeque<f64> = VecDeque::from(vec![0.0; history_len]);
    let mut ram_history: VecDeque<f64> = VecDeque::from(vec![0.0; history_len]);

    // Particles (workload)
    let mut particles: Vec<Particle> = Vec::new();

    // FIX: explicit `usize` typing so we can use numeric helper methods
    let mut particle_count_target: usize = 100;

    let (mut cols, mut rows) = terminal::size()?;
    let width = (cols as usize).saturating_sub(2);
    let height = (rows as usize).saturating_sub(4);

    let mut chart = ChartContext::new(width, height);
    let mut running = true;
    let mut last_sys_update = Instant::now();

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
                    KeyCode::Char('+') => particle_count_target += 500,
                    // Now this works because the type is known to be usize
                    KeyCode::Char('-') => {
                        particle_count_target = particle_count_target.saturating_sub(500)
                    }
                    _ => {}
                }
            }
        }

        // --- C. SYSTEM LOGIC (every 500ms) ---
        if last_sys_update.elapsed() >= Duration::from_millis(500) {
            sys.refresh_cpu();
            sys.refresh_memory();

            // CPU Global Usage
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            cpu_history.pop_front();
            cpu_history.push_back(cpu_usage as f64);

            // RAM Usage
            let used_mem = sys.used_memory();
            let total_mem = sys.total_memory();
            // Avoid division by zero if total_mem is 0 (rare, but possible under virtualization)
            let ram_percent = if total_mem > 0 {
                (used_mem as f64 / total_mem as f64) * 100.0
            } else {
                0.0
            };
            ram_history.pop_front();
            ram_history.push_back(ram_percent);

            last_sys_update = Instant::now();
        }

        // --- D. PARTICLE LOGIC (stress simulation) ---
        let w_px = chart.canvas.pixel_width() as f64;
        let h_px = chart.canvas.pixel_height() as f64;
        // Define the simulation area (bottom half)
        let sim_top = h_px / 2.0;
        let sim_height = h_px / 2.0;

        // Adjust the particle count
        while particles.len() < particle_count_target {
            particles.push(Particle {
                x: rand::random::<f64>() * w_px,
                y: sim_top + rand::random::<f64>() * sim_height,
                vx: (rand::random::<f64>() - 0.5) * 2.0,
                vy: (rand::random::<f64>() - 0.5) * 2.0,
                color: Color::Green, // Default
            });
        }
        if particles.len() > particle_count_target {
            particles.truncate(particle_count_target);
        }

        // Particle physics and rendering
        for p in &mut particles {
            p.x += p.vx;
            p.y += p.vy;

            // Bounce off the edges of the lower region
            if p.x <= 0.0 || p.x >= w_px {
                p.vx *= -1.0;
            }
            if p.y <= sim_top || p.y >= h_px {
                p.vy *= -1.0;
            }

            // Color by velocity (simple heat map)
            let speed = (p.vx.powi(2) + p.vy.powi(2)).sqrt();
            p.color = if speed > 1.5 { Color::Red } else { Color::Cyan };

            // Visual clamp
            let dx = p.x.clamp(0.0, w_px - 1.0) as usize;
            let dy = p.y.clamp(sim_top, h_px - 1.0) as usize;

            chart.canvas.set_pixel_screen(dx, dy, Some(p.color));
        }

        // --- E. CHART RENDERING (dashboard) ---

        // Horizontal divider
        chart.canvas.line_screen(
            0,
            sim_top as isize,
            w_px as isize,
            sim_top as isize,
            Some(Color::White),
        );
        // Vertical divider (top half)
        chart.canvas.line_screen(
            (w_px / 2.0) as isize,
            0,
            (w_px / 2.0) as isize,
            sim_top as isize,
            Some(Color::White),
        );

        // Helper for drawing charts inside a specific box
        let draw_mini_chart = |chart: &mut ChartContext,
                               data: &VecDeque<f64>,
                               x_offset: f64,
                               width: f64,
                               color: Color| {
            let height = sim_top; // Box height is the upper half

            // Draw the line
            let step_x = width / data.len() as f64;
            let mut prev_x = x_offset;
            let mut prev_y = height - (data[0] / 100.0 * height); // 100.0 is the max value (100%)

            for (i, &val) in data.iter().enumerate().skip(1) {
                let curr_x = x_offset + (i as f64 * step_x);
                // Invert Y because screen coordinates grow downward
                // Y = Base - (NormalizedValue * AvailableHeight)
                let curr_y = height - (val / 100.0 * (height - 2.0));

                chart.canvas.line_screen(
                    prev_x as isize,
                    prev_y as isize,
                    curr_x as isize,
                    curr_y as isize,
                    Some(color),
                );
                prev_x = curr_x;
                prev_y = curr_y;
            }
        };

        // Draw CPU (left)
        draw_mini_chart(&mut chart, &cpu_history, 0.0, w_px / 2.0, Color::Yellow);
        let cpu_txt = format!("CPU: {:.1}%", cpu_history.back().unwrap_or(&0.0));
        chart.text(&cpu_txt, 0.02, 0.05, Some(Color::Yellow));

        // Draw RAM (right)
        draw_mini_chart(
            &mut chart,
            &ram_history,
            w_px / 2.0,
            w_px / 2.0,
            Color::Magenta,
        );
        let ram_txt = format!("RAM: {:.1}%", ram_history.back().unwrap_or(&0.0));
        chart.text(&ram_txt, 0.52, 0.05, Some(Color::Magenta));

        // Particle info
        let part_txt = format!("STRESS TEST: {} Particles (+/- to change)", particles.len());
        chart.text(&part_txt, 0.02, 0.52, Some(Color::Green));

        // Output final
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart
            .canvas
            .render_with_options(true, Some("SYSTEM MONITOR & STRESS TEST"));
        print!("{}", output.replace('\n', "\r\n"));
        io::stdout().flush()?;

        std::thread::sleep(Duration::from_millis(30));
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
