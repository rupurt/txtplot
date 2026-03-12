use colored::Color;
use txtplot::ChartContext;

const STRIKE_MIN: f64 = 0.78;
const STRIKE_MAX: f64 = 1.22;
const EXPIRY_MIN: f64 = 0.05;
const EXPIRY_MAX: f64 = 2.00;
const SURFACE_ROWS: usize = 22;
const SURFACE_COLS: usize = 28;

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn add(self, other: Vec3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

struct ZBuffer {
    width: usize,
    height: usize,
    depth: Vec<f64>,
}

impl ZBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            depth: vec![f64::INFINITY; width * height],
        }
    }

    fn test_and_set(&mut self, x: usize, y: usize, depth: f64) -> bool {
        let idx = y * self.width + x;
        if depth < self.depth[idx] {
            self.depth[idx] = depth;
            true
        } else {
            false
        }
    }
}

fn rotate_x(v: Vec3, angle: f64) -> Vec3 {
    let (sin_a, cos_a) = angle.sin_cos();
    Vec3::new(v.x, v.y * cos_a - v.z * sin_a, v.y * sin_a + v.z * cos_a)
}

fn rotate_y(v: Vec3, angle: f64) -> Vec3 {
    let (sin_a, cos_a) = angle.sin_cos();
    Vec3::new(v.x * cos_a - v.z * sin_a, v.y, v.x * sin_a + v.z * cos_a)
}

fn project_to_screen(
    v: Vec3,
    width_px: f64,
    height_px: f64,
    scale: f64,
) -> Option<(isize, isize, f64)> {
    if v.z <= 0.2 {
        return None;
    }

    let sx = width_px * 0.5 + (v.x / v.z) * scale;
    let sy = height_px * 0.58 - (v.y / v.z) * scale;
    Some((sx.round() as isize, sy.round() as isize, v.z))
}

fn plot_z(
    chart: &mut ChartContext,
    zbuf: &mut ZBuffer,
    x: isize,
    y: isize,
    depth: f64,
    color: Color,
) {
    if x < 0 || y < 0 {
        return;
    }

    let ux = x as usize;
    let uy = y as usize;

    if ux >= zbuf.width || uy >= zbuf.height {
        return;
    }

    if zbuf.test_and_set(ux, uy, depth) {
        chart.canvas.set_pixel_screen(ux, uy, Some(color));
    }
}

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

fn line_z(
    chart: &mut ChartContext,
    zbuf: &mut ZBuffer,
    start: (isize, isize, f64),
    end: (isize, isize, f64),
    color: Color,
) {
    let steps = (end.0 - start.0).abs().max((end.1 - start.1).abs()).max(1) as usize;

    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let x = start.0 as f64 + (end.0 - start.0) as f64 * t;
        let y = start.1 as f64 + (end.1 - start.1) as f64 * t;
        let z = start.2 + (end.2 - start.2) * t;
        plot_z(
            chart,
            zbuf,
            x.round() as isize,
            y.round() as isize,
            z,
            color,
        );
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
                    project_to_screen(pa, width_px, height_px, 62.0),
                    project_to_screen(pb, width_px, height_px, 62.0),
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
                    project_to_screen(pa, width_px, height_px, 62.0),
                    project_to_screen(pb, width_px, height_px, 62.0),
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
            project_to_screen(pa, width_px, height_px, 62.0),
            project_to_screen(pb, width_px, height_px, 62.0),
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
        if let Some(peak2d) = project_to_screen(peak, width_px, height_px, 62.0) {
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
