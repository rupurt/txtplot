#![allow(dead_code)]

use colored::Color;
use txtplot::ChartContext;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn add(self, other: Vec3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(self, other: Vec3) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

pub fn rotate_x(v: Vec3, angle: f64) -> Vec3 {
    let (sin_a, cos_a) = angle.sin_cos();
    Vec3::new(v.x, v.y * cos_a - v.z * sin_a, v.y * sin_a + v.z * cos_a)
}

pub fn rotate_y(v: Vec3, angle: f64) -> Vec3 {
    let (sin_a, cos_a) = angle.sin_cos();
    Vec3::new(v.x * cos_a - v.z * sin_a, v.y, v.x * sin_a + v.z * cos_a)
}

pub struct ZBuffer {
    width: usize,
    height: usize,
    depth: Vec<f64>,
}

impl ZBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            depth: vec![f64::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.depth.fill(f64::INFINITY);
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

#[derive(Clone, Copy)]
pub struct Projection {
    near_plane: f64,
    center_x_ratio: f64,
    center_y_ratio: f64,
    scale_x: f64,
    scale_y: f64,
}

impl Projection {
    pub fn new(
        near_plane: f64,
        center_x_ratio: f64,
        center_y_ratio: f64,
        scale_x: f64,
        scale_y: f64,
    ) -> Self {
        Self {
            near_plane,
            center_x_ratio,
            center_y_ratio,
            scale_x,
            scale_y,
        }
    }
}

pub fn project_with_projection(
    v_cam: Vec3,
    canvas_w: f64,
    canvas_h: f64,
    projection: Projection,
) -> Option<(isize, isize, f64)> {
    if v_cam.z <= projection.near_plane {
        return None;
    }

    let cx = canvas_w * projection.center_x_ratio;
    let cy = canvas_h * projection.center_y_ratio;
    let sx = cx + (v_cam.x / v_cam.z) * projection.scale_x;
    let sy = cy + (v_cam.y / v_cam.z) * projection.scale_y;

    Some((sx.round() as isize, sy.round() as isize, v_cam.z))
}

pub fn project_to_screen(
    v_cam: Vec3,
    canvas_w: f64,
    canvas_h: f64,
    scale: f64,
) -> Option<(isize, isize, f64)> {
    project_with_projection(
        v_cam,
        canvas_w,
        canvas_h,
        Projection::new(0.06, 0.5, 0.5, scale * 2.0, scale),
    )
}

pub fn make_sphere_points(lat_steps: usize, lon_steps: usize) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(lat_steps * lon_steps);
    for i in 0..lat_steps {
        let v = i as f64 / (lat_steps - 1).max(1) as f64;
        let theta = v * std::f64::consts::PI;
        let st = theta.sin();
        let ct = theta.cos();
        for j in 0..lon_steps {
            let u = j as f64 / lon_steps as f64;
            let phi = u * std::f64::consts::TAU;
            let (sp, cp) = phi.sin_cos();
            pts.push(Vec3::new(st * cp, ct, st * sp));
        }
    }
    pts
}

pub fn make_torus_rings(
    r_major: f64,
    r_minor: f64,
    u_steps: usize,
    v_steps: usize,
) -> Vec<Vec<Vec3>> {
    let mut rings = Vec::with_capacity(u_steps);
    for i in 0..u_steps {
        let u = i as f64 / u_steps as f64 * std::f64::consts::TAU;
        let (su, cu) = u.sin_cos();
        let mut ring = Vec::with_capacity(v_steps + 1);
        for j in 0..=v_steps {
            let v = j as f64 / v_steps as f64 * std::f64::consts::TAU;
            let (sv, cv) = v.sin_cos();
            let x = (r_major + r_minor * cv) * cu;
            let y = r_minor * sv;
            let z = (r_major + r_minor * cv) * su;
            ring.push(Vec3::new(x, y, z));
        }
        rings.push(ring);
    }
    rings
}

pub fn make_triangle() -> [Vec3; 3] {
    [
        Vec3::new(-1.2, -0.8, 0.0),
        Vec3::new(1.2, -0.8, 0.0),
        Vec3::new(0.0, 1.3, 0.0),
    ]
}

pub fn plot_z(
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

pub fn line_z(
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
