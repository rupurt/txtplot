#![allow(dead_code)]

use colored::Color;

use crate::support::three_d::{rotate_x, rotate_y, rotate_z, Vec3};

/// Solves Kepler's equation (M = E - e * sin(E)) using Newton-Raphson.
pub fn solve_kepler(m: f64, e: f64) -> f64 {
    let mut e_est = m;
    for _ in 0..5 {
        e_est = e_est - (e_est - e * e_est.sin() - m) / (1.0 - e * e_est.cos());
    }
    e_est
}

/// Represents a celestial body with physically inspired properties.
/// Astronomical values are scaled for graphical visualization.
pub struct CelestialBody {
    pub name: &'static str,
    pub parent: Option<usize>,
    pub color: Color,
    pub radius: f64,
    pub a: f64,
    pub e: f64,
    pub i: f64,
    pub omega: f64,
    pub w: f64,
    pub w_dot: f64,
    pub m0: f64,
    pub n: f64,
    pub axial_tilt: f64,
    pub rot_rate: f64,
    pub prec_rate: f64,
    pub nut_amp: f64,
    pub nut_rate: f64,
    pub cw_amp: f64,
    pub cw_rate: f64,
    pub ring_inner: f64,
    pub ring_outer: f64,
    pub ring_color: Color,
    pub is_star: bool,
}

impl CelestialBody {
    /// Computes the planet center position in 3D space at time `t`.
    /// The position is relative to its parent (Sun or another planet).
    pub fn get_local_orbit_pos(&self, t: f64) -> Vec3 {
        if self.a == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let m = self.m0 + self.n * t;
        let e_anom = solve_kepler(m, self.e);
        let nu = 2.0 * (((1.0 + self.e) / (1.0 - self.e)).sqrt() * (e_anom / 2.0).tan()).atan();
        let r = self.a * (1.0 - self.e * e_anom.cos());

        let current_w = self.w + self.w_dot * t;

        let mut p = Vec3::new(r * nu.cos(), 0.0, r * nu.sin());
        p = rotate_y(p, current_w);
        p = rotate_x(p, self.i);
        p = rotate_y(p, self.omega);
        p
    }

    /// Computes a world-space sphere vertex and its rotated normal for lighting.
    pub fn get_vertex_data(&self, local_v: Vec3, t: f64, absolute_orbit_pos: Vec3) -> (Vec3, Vec3) {
        let mut normal = local_v;
        let mut v = Vec3::new(
            local_v.x * self.radius,
            local_v.y * self.radius,
            local_v.z * self.radius,
        );

        let cw_x = self.cw_amp * (self.cw_rate * t).cos();
        let cw_z = self.cw_amp * (self.cw_rate * t).sin();
        v = rotate_x(v, cw_x);
        v = rotate_z(v, cw_z);
        normal = rotate_x(normal, cw_x);
        normal = rotate_z(normal, cw_z);

        v = rotate_y(v, self.rot_rate * t);
        normal = rotate_y(normal, self.rot_rate * t);

        let current_tilt = self.axial_tilt + self.nut_amp * (self.nut_rate * t).cos();
        v = rotate_x(v, current_tilt);
        normal = rotate_x(normal, current_tilt);

        v = rotate_y(v, self.prec_rate * t);
        normal = rotate_y(normal, self.prec_rate * t);

        v = v.add(absolute_orbit_pos);
        (v, normal.normalize())
    }

    #[allow(dead_code)]
    pub fn get_ring_pos(&self, local_v: Vec3, t: f64, absolute_orbit_pos: Vec3) -> Vec3 {
        let mut v = local_v;
        let current_tilt = self.axial_tilt + self.nut_amp * (self.nut_rate * t).cos();
        v = rotate_x(v, current_tilt);
        v = rotate_y(v, self.prec_rate * t);
        v.add(absolute_orbit_pos)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the example dataset is kept as explicit orbital-element literals"
)]
pub fn body(
    name: &'static str,
    p: Option<usize>,
    c: Color,
    r: f64,
    a: f64,
    e: f64,
    i: f64,
    om: f64,
    w: f64,
    wd: f64,
    m: f64,
    n: f64,
    tilt: f64,
    rot: f64,
    pr: f64,
    ri: f64,
    ro: f64,
    rc: Color,
    star: bool,
) -> CelestialBody {
    CelestialBody {
        name,
        parent: p,
        color: c,
        radius: r,
        a,
        e,
        i,
        omega: om,
        w,
        w_dot: wd,
        m0: m,
        n,
        axial_tilt: tilt,
        rot_rate: rot,
        prec_rate: pr,
        nut_amp: 0.0,
        nut_rate: 0.0,
        cw_amp: 0.0,
        cw_rate: 0.0,
        ring_inner: ri,
        ring_outer: ro,
        ring_color: rc,
        is_star: star,
    }
}
