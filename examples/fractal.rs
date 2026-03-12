use colored::*;
use std::io::{self, Write};
//use txtplot::{ChartContext, ChartOptions};
use txtplot::ChartContext;
// Define constants for the resolution
const WIDTH: usize = 80;
const HEIGHT: usize = 40;

fn main() {
    loop {
        // --- Main Menu ---
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!(
            "{}",
            "=== TXTPLOT FRACTAL EXPLORER ===".bright_cyan().bold()
        );
        println!("Select an option:");
        println!("1. {}", "Mandelbrot Set".green());
        println!("2. {}", "Julia Set".green());
        println!("3. {}", "Lorenz Attractor (Auto-scale)".yellow());
        println!("4. {}", "Barnsley Fern (Auto-scale)".yellow());
        println!("q. Exit");
        print!("\nOption > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => draw_mandelbrot(),
            "2" => draw_julia(),
            "3" => draw_lorenz(),
            "4" => draw_barnsley(),
            "q" => break,
            _ => continue,
        }

        println!("\nPress Enter to continue...");
        let mut _s = String::new();
        io::stdin().read_line(&mut _s).unwrap();
    }
}

// --- 1. MANDELBROT (Pixel by Pixel) ---
fn draw_mandelbrot() {
    let mut chart = ChartContext::new(WIDTH, HEIGHT);

    // Virtual pixel dimensions (2x width, 4x height)
    let w_px = (WIDTH * 2) as f64;
    let h_px = (HEIGHT * 4) as f64;

    // Complex plane range
    let min_x = -2.5;
    let max_x = 1.0;
    let min_y = -1.2;
    let max_y = 1.2;

    for py in 0..(h_px as usize) {
        for px in 0..(w_px as usize) {
            // Map the pixel to a complex coordinate
            let x0 = min_x + (px as f64 / w_px) * (max_x - min_x);
            let y0 = min_y + (py as f64 / h_px) * (max_y - min_y);

            let mut x = 0.0;
            let mut y = 0.0;
            let mut iteration = 0;
            let max_iteration = 50;

            while x * x + y * y <= 4.0 && iteration < max_iteration {
                let xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                iteration += 1;
            }

            // Color according to the iteration count
            if iteration < max_iteration {
                let color = if iteration < 10 {
                    Color::Blue
                } else if iteration < 20 {
                    Color::Cyan
                } else {
                    Color::White
                };
                chart.canvas.set_pixel(px, py, Some(color));
            }
        }
    }

    println!(
        "{}",
        chart.canvas.render_with_options(true, Some("Mandelbrot"))
    );
}

// --- 2. JULIA (Pixel by Pixel) ---
fn draw_julia() {
    let mut chart = ChartContext::new(WIDTH, HEIGHT);
    let w_px = (WIDTH * 2) as f64;
    let h_px = (HEIGHT * 4) as f64;

    // Constant C parameter for Julia
    let c_re = -0.7;
    let c_im = 0.27015;

    let zoom = 1.0;
    let move_x = 0.0;
    let move_y = 0.0;

    for py in 0..(h_px as usize) {
        for px in 0..(w_px as usize) {
            let mut new_re = 1.5 * (px as f64 - w_px / 2.0) / (0.5 * zoom * w_px) + move_x;
            let mut new_im = (py as f64 - h_px / 2.0) / (0.5 * zoom * h_px) + move_y;

            let mut i = 0;
            let max_iter = 50;

            while i < max_iter {
                let old_re = new_re;
                let old_im = new_im;

                new_re = old_re * old_re - old_im * old_im + c_re;
                new_im = 2.0 * old_re * old_im + c_im;

                if (new_re * new_re + new_im * new_im) > 4.0 {
                    break;
                }
                i += 1;
            }

            if i < max_iter {
                let color = match i % 3 {
                    0 => Color::Magenta,
                    1 => Color::Red,
                    _ => Color::BrightRed,
                };
                chart.canvas.set_pixel(px, py, Some(color));
            }
        }
    }
    println!(
        "{}",
        chart.canvas.render_with_options(true, Some("Julia Set"))
    );
}

// --- 3. LORENZ (Vectorial / Auto-scale) ---
fn draw_lorenz() {
    let mut chart = ChartContext::new(WIDTH, HEIGHT);

    // System parameters
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    let mut x = 0.1;
    let mut y = 0.0;
    let mut z = 0.0;
    let dt = 0.01;

    let mut points = Vec::new();

    // Generate the trajectory
    for _ in 0..3000 {
        let dx = sigma * (y - x);
        let dy = x * (rho - z) - y;
        let dz = x * y - beta * z;

        x += dx * dt;
        y += dy * dt;
        z += dz * dt;

        // Project onto the X-Z plane (the butterfly shape reads more clearly there)
        points.push((x, z));
    }

    // Use line_chart, which automatically computes the data min/max
    // and scales everything to fit the canvas.
    chart.line_chart(&points, Some(Color::BrightGreen));

    // Add a title
    chart.text("X-Z Plane", 0.05, 0.9, Some(Color::White));

    println!(
        "{}",
        chart
            .canvas
            .render_with_options(true, Some("Lorenz Attractor"))
    );
}

// --- 4. BARNSLEY FERN (Scatter / Auto-scale) ---
use rand::Rng;
fn draw_barnsley() {
    let mut chart = ChartContext::new(WIDTH, HEIGHT);
    let mut rng = rand::thread_rng();

    let mut x = 0.0;
    let mut y = 0.0;
    let mut points = Vec::new();

    for _ in 0..10000 {
        let r: f64 = rng.r#gen();
        let next_x;
        let next_y;

        if r < 0.01 {
            next_x = 0.0;
            next_y = 0.16 * y;
        } else if r < 0.86 {
            next_x = 0.85 * x + 0.04 * y;
            next_y = -0.04 * x + 0.85 * y + 1.6;
        } else if r < 0.93 {
            next_x = 0.2 * x - 0.26 * y;
            next_y = 0.23 * x + 0.22 * y + 1.6;
        } else {
            next_x = -0.15 * x + 0.28 * y;
            next_y = 0.26 * x + 0.24 * y + 0.44;
        }
        x = next_x;
        y = next_y;

        points.push((x, y));
    }

    // scatter also uses internal autoscaling based on the provided data
    chart.scatter(&points, Some(Color::Green));

    println!(
        "{}",
        chart
            .canvas
            .render_with_options(true, Some("Barnsley Fern"))
    );
}
