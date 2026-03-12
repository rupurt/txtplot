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
use txtplot::ChartContext;

fn main() -> io::Result<()> {
    // Initial setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let (mut cols, mut rows) = terminal::size()?;
    let width = (cols as usize).saturating_sub(4);
    let height = (rows as usize).saturating_sub(4);

    let mut chart = ChartContext::new(width, height);
    let mut running = true;

    // --- SIMULATION STATE ---
    let mut phase: f64 = 0.0;

    // Wave buffer (oscilloscope)
    // Store enough points to fill the pixel width
    let max_points = chart.canvas.pixel_width();
    let mut wave_buffer: VecDeque<f64> = VecDeque::with_capacity(max_points);
    // Fill with initial zeros
    for _ in 0..max_points {
        wave_buffer.push_back(0.0);
    }

    // Spectrum buffer (bars)
    let num_bars = 30;
    let mut spectrum: Vec<f64> = vec![0.0; num_bars];

    let start_time = Instant::now();

    while running {
        // 1. Resize Check
        let (nc, nr) = terminal::size()?;
        if nc != cols || nr != rows {
            cols = nc;
            rows = nr;
            let w = (cols as usize).saturating_sub(4);
            let h = (rows as usize).saturating_sub(4);
            chart = ChartContext::new(w, h);
            // Resize the buffer if the width changes
            let new_max = chart.canvas.pixel_width();
            if new_max > wave_buffer.len() {
                wave_buffer.resize(new_max, 0.0);
            } else {
                wave_buffer.truncate(new_max);
            }
        } else {
            chart.canvas.clear();
        }

        // 2. Input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if code == KeyCode::Char('q') || code == KeyCode::Esc {
                    running = false;
                }
            }
        }

        // 3. UPDATE DATA (simulate a complex audio signal)
        phase += 0.15;

        // Build a signal from 3 sine waves
        let signal = (phase).sin() * 0.5        // Low frequency (bass)
                   + (phase * 3.0).sin() * 0.3  // Midrange
                   + (phase * 7.0).sin() * 0.1; // Highs (noise)

        // Scroll: drop the oldest sample, push the newest
        wave_buffer.pop_front();
        wave_buffer.push_back(signal);

        // Simulate a frequency spectrum (procedural animation)
        for (i, value) in spectrum.iter_mut().enumerate().take(num_bars) {
            // Fake Perlin-style noise using sine waves
            let freq_val = ((phase * 0.5 + i as f64 * 0.5).sin() + 1.0) / 2.0;
            // Smoothing (lerp) so the bars do not jump wildly
            *value = *value * 0.8 + freq_val * 0.2;
        }

        // 4. RENDERING
        let w_px = chart.canvas.pixel_width();
        let h_px = chart.canvas.pixel_height();
        let h_half = h_px / 2; // Split the screen in half

        // --- A) TOP HALF: OSCILLOSCOPE (line) ---
        let mut points = Vec::with_capacity(w_px);
        for (i, &val) in wave_buffer.iter().enumerate() {
            // Map X across the full width
            let x = i as f64;
            // Map Y into the upper half (0 to h_half)
            // Centered around h_half / 2
            let center_y = (h_px as f64) * 0.75;
            let amplitude = (h_px as f64) * 0.2;
            let y = center_y + val * amplitude;

            points.push((x, y));
        }

        // Draw the upper grid
        chart.canvas.line_screen(
            0,
            h_half as isize,
            w_px as isize,
            h_half as isize,
            Some(Color::White),
        ); // Divider
        chart.text("CH-A: WAVEFORM", 0.02, 0.55, Some(Color::Cyan));

        // Draw the wave manually (faster than chart.line_chart for data already in px)
        for w in points.windows(2) {
            chart.canvas.line_screen(
                w[0].0 as isize,
                w[0].1 as isize,
                w[1].0 as isize,
                w[1].1 as isize,
                Some(Color::BrightCyan),
            );
        }

        // --- B) BOTTOM HALF: SPECTRUM ANALYZER (bars) ---
        let bar_w = w_px / num_bars;
        let floor_y = (h_px as f64 * 0.45) as isize; // Base of the bars

        chart.text("CH-B: SPECTRUM", 0.02, 0.95, Some(Color::Magenta));

        for (i, &val) in spectrum.iter().enumerate() {
            let bar_h = (val * (h_px as f64 * 0.4)) as isize;
            let x_start = (i * bar_w) as isize;
            let x_end = x_start + (bar_w as isize).max(1) - 1; // Space between bars

            // Height-based gradient color
            let color = if val > 0.8 {
                Color::Red
            } else if val > 0.5 {
                Color::Yellow
            } else {
                Color::Magenta
            };

            for x in x_start..x_end {
                // Draw a vertical line from the floor upward
                // In screen coordinates Y grows downward, so:
                // floor_y is the bottom, floor_y - bar_h is the top
                chart
                    .canvas
                    .line_screen(x, floor_y, x, floor_y - bar_h, Some(color));
            }
        }

        // --- C) INFORMATION OVERLAY ---
        let uptime = start_time.elapsed().as_secs();
        let info = format!(
            "REC [●] | T: {:02}:{:02} | FPS: High",
            uptime / 60,
            uptime % 60
        );
        chart.text(&info, 0.7, 0.05, Some(Color::Red));

        // Output
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart
            .canvas
            .render_with_options(true, Some("AUDIO VISUALIZER SIMULATION"));
        print!("{}", output.replace('\n', "\r\n"));
        io::stdout().flush()?;

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS cap
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
