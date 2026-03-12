mod support;

use colored::Color;
use support::three_d::{
    line_z, plot_z, project_with_projection, rotate_x, rotate_y, Projection, Vec3, ZBuffer,
};
use txtplot::ChartContext;

const STRIKE_MIN: f64 = 0.78;
const STRIKE_MAX: f64 = 1.22;
const EXPIRY_MIN: f64 = 0.05;
const EXPIRY_MAX: f64 = 2.00;
const SURFACE_ROWS: usize = 22;
const SURFACE_COLS: usize = 28;

fn stamp_z(
    chart: &mut ChartContext,
    zbuf: &mut ZBuffer,
    x: isize,
    y: isize,
    depth: f64,
    color: Color,
) {
    for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        plot_z(chart, zbuf, x + dx, y + dy, depth - 0.02, color);
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn surface_color(vol: f64, min_vol: f64, max_vol: f64) -> Color {
    let t = ((vol - min_vol) / (max_vol - min_vol)).clamp(0.0, 1.0);
    let r = lerp(32.0, 255.0, t) as u8;
    let g = lerp(110.0, 208.0, t) as u8;
    let b = lerp(210.0, 64.0, t) as u8;
    Color::TrueColor { r, g, b }
}

fn volatility_surface(strike: f64, expiry: f64) -> f64 {
    let smile = 0.16 + 0.28 * (strike - 1.0).powi(2);
    let term = 0.07 * (1.0 - (-1.8 * expiry).exp());
    let skew = 0.05 * (1.0 - strike) * (-2.6 * expiry).exp();
    let ridge = 0.12 * (-(28.0 * (strike - 0.91).powi(2) + 4.5 * (expiry - 0.42).powi(2))).exp();

    smile + term + skew + ridge
}

fn gradient(strike: f64, expiry: f64) -> (f64, f64) {
    let ds = 0.005;
    let dt = 0.01;

    let dv_ds = (volatility_surface((strike + ds).clamp(STRIKE_MIN, STRIKE_MAX), expiry)
        - volatility_surface((strike - ds).clamp(STRIKE_MIN, STRIKE_MAX), expiry))
        / (2.0 * ds);
    let dv_dt = (volatility_surface(strike, (expiry + dt).clamp(EXPIRY_MIN, EXPIRY_MAX))
        - volatility_surface(strike, (expiry - dt).clamp(EXPIRY_MIN, EXPIRY_MAX)))
        / (2.0 * dt);

    (dv_ds, dv_dt)
}

fn ascent_path(start_strike: f64, start_expiry: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let mut strike = start_strike;
    let mut expiry = start_expiry;
    let mut path = Vec::with_capacity(steps + 1);

    for _ in 0..steps {
        let vol = volatility_surface(strike, expiry);
        path.push((strike, expiry, vol));

        let (grad_s, grad_t) = gradient(strike, expiry);
        let norm = (grad_s * grad_s + grad_t * grad_t).sqrt();
        if norm < 1e-7 {
            break;
        }

        strike = (strike + 0.018 * grad_s / norm).clamp(STRIKE_MIN, STRIKE_MAX);
        expiry = (expiry + 0.09 * grad_t / norm).clamp(EXPIRY_MIN, EXPIRY_MAX);
    }

    path.push((strike, expiry, volatility_surface(strike, expiry)));
    path
}

fn world_point(strike: f64, expiry: f64, vol: f64, lift: f64) -> Vec3 {
    let x = ((strike - (STRIKE_MIN + STRIKE_MAX) * 0.5) / (STRIKE_MAX - STRIKE_MIN)) * 7.0;
    let z = ((expiry - (EXPIRY_MIN + EXPIRY_MAX) * 0.5) / (EXPIRY_MAX - EXPIRY_MIN)) * 6.0;
    let y = (vol - 0.20) * 20.0 + lift;
    Vec3::new(x, y, z)
}

fn camera_space(point: Vec3) -> Vec3 {
    rotate_y(rotate_x(point, -0.72), 0.90).add(Vec3::new(0.0, -0.35, 10.5))
}

fn main() {
    let mut chart = ChartContext::new(74, 22);
    let width_px = chart.canvas.pixel_width() as f64;
    let height_px = chart.canvas.pixel_height() as f64;
    let mut zbuf = ZBuffer::new(chart.canvas.pixel_width(), chart.canvas.pixel_height());
    let projection = Projection::new(0.2, 0.5, 0.58, 62.0, -62.0);

    let mut surface = Vec::with_capacity(SURFACE_ROWS + 1);
    let mut min_vol = f64::INFINITY;
    let mut max_vol = f64::NEG_INFINITY;

    for row in 0..=SURFACE_ROWS {
        let t = row as f64 / SURFACE_ROWS as f64;
        let expiry = lerp(EXPIRY_MIN, EXPIRY_MAX, t);
        let mut strip = Vec::with_capacity(SURFACE_COLS + 1);

        for col in 0..=SURFACE_COLS {
            let u = col as f64 / SURFACE_COLS as f64;
            let strike = lerp(STRIKE_MIN, STRIKE_MAX, u);
            let vol = volatility_surface(strike, expiry);
            min_vol = min_vol.min(vol);
            max_vol = max_vol.max(vol);
            strip.push((strike, expiry, vol));
        }

        surface.push(strip);
    }

    for row in 0..surface.len() {
        for col in 0..surface[row].len() {
            if col + 1 < surface[row].len() {
                let a = surface[row][col];
                let b = surface[row][col + 1];
                let pa = camera_space(world_point(a.0, a.1, a.2, 0.0));
                let pb = camera_space(world_point(b.0, b.1, b.2, 0.0));
                let color = surface_color((a.2 + b.2) * 0.5, min_vol, max_vol);

                if let (Some(a2d), Some(b2d)) = (
                    project_with_projection(pa, width_px, height_px, projection),
                    project_with_projection(pb, width_px, height_px, projection),
                ) {
                    line_z(&mut chart, &mut zbuf, a2d, b2d, color);
                }
            }

            if row + 1 < surface.len() {
                let a = surface[row][col];
                let b = surface[row + 1][col];
                let pa = camera_space(world_point(a.0, a.1, a.2, 0.0));
                let pb = camera_space(world_point(b.0, b.1, b.2, 0.0));
                let color = surface_color((a.2 + b.2) * 0.5, min_vol, max_vol);

                if let (Some(a2d), Some(b2d)) = (
                    project_with_projection(pa, width_px, height_px, projection),
                    project_with_projection(pb, width_px, height_px, projection),
                ) {
                    line_z(&mut chart, &mut zbuf, a2d, b2d, color);
                }
            }
        }
    }

    let path = ascent_path(1.17, 1.65, 18);
    for window in path.windows(2) {
        let a = window[0];
        let b = window[1];
        let pa = camera_space(world_point(a.0, a.1, a.2, 0.10));
        let pb = camera_space(world_point(b.0, b.1, b.2, 0.10));

        if let (Some(a2d), Some(b2d)) = (
            project_with_projection(pa, width_px, height_px, projection),
            project_with_projection(pb, width_px, height_px, projection),
        ) {
            line_z(
                &mut chart,
                &mut zbuf,
                (a2d.0, a2d.1, a2d.2 - 0.05),
                (b2d.0, b2d.1, b2d.2 - 0.05),
                Color::BrightYellow,
            );
            stamp_z(
                &mut chart,
                &mut zbuf,
                a2d.0,
                a2d.1,
                a2d.2 - 0.08,
                Color::BrightMagenta,
            );
        }
    }

    if let Some(last) = path.last() {
        let peak = camera_space(world_point(last.0, last.1, last.2, 0.12));
        if let Some(peak2d) = project_with_projection(peak, width_px, height_px, projection) {
            stamp_z(
                &mut chart,
                &mut zbuf,
                peak2d.0,
                peak2d.1,
                peak2d.2 - 0.10,
                Color::White,
            );
        }
    }

    chart.text("3D vol surface", 0.36, 0.96, Some(Color::White));
    chart.text(
        "yellow = gradient ascent path",
        0.26,
        0.91,
        Some(Color::BrightYellow),
    );
    chart.text("strike", 0.10, 0.07, Some(Color::BrightBlack));
    chart.text("expiry", 0.80, 0.07, Some(Color::BrightBlack));
    chart.text("vol", 0.06, 0.82, Some(Color::BrightBlack));

    println!(
        "{}",
        chart
            .canvas
            .render_with_options(true, Some("TXTPLOT 3D SURFACE EXAMPLE"))
    );
}
