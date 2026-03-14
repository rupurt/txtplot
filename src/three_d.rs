use crate::canvas::{CellCanvas, CellRenderer};
use colored::Color;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = self.norm();
        if length > 0.0 {
            Self::new(self.x / length, self.y / length, self.z / length)
        } else {
            self
        }
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
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

pub fn rotate_z(v: Vec3, angle: f64) -> Vec3 {
    let (sin_a, cos_a) = angle.sin_cos();
    Vec3::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a, v.z)
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn from_canvas<R: CellRenderer>(canvas: &CellCanvas<R>) -> Self {
        Self::new(canvas.pixel_width(), canvas.pixel_height())
    }

    pub fn clear(&mut self) {
        self.depth.fill(f64::INFINITY);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    near_plane: f64,
    center_x_ratio: f64,
    center_y_ratio: f64,
    scale_x: f64,
    scale_y: f64,
}

impl Projection {
    pub const fn new(
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

/// A camera that orbits around a target point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f64,
    pub pitch: f64,
    pub yaw: f64,
}

impl OrbitCamera {
    pub fn new(target: Vec3, distance: f64, pitch: f64, yaw: f64) -> Self {
        Self {
            target,
            distance,
            pitch,
            yaw,
        }
    }

    /// Computes the camera position in world space.
    pub fn position(&self) -> Vec3 {
        let (sin_p, cos_p) = self.pitch.sin_cos();
        let (sin_y, cos_y) = self.yaw.sin_cos();

        let rel_pos = Vec3::new(
            self.distance * cos_p * sin_y,
            self.distance * sin_p,
            self.distance * cos_p * cos_y,
        );

        self.target + rel_pos
    }

    /// Transforms a world-space point to camera-space.
    pub fn transform(&self, point: Vec3) -> Vec3 {
        let pos = self.position();
        let forward = (self.target - pos).normalize();
        let up_world = Vec3::new(0.0, 1.0, 0.0);
        let right = forward.cross(up_world).normalize();
        let up = right.cross(forward).normalize();

        let rel = point - pos;
        Vec3::new(rel.dot(right), rel.dot(up), rel.dot(forward))
    }

    /// Projects a world-space point directly to screen coordinates.
    pub fn project(
        &self,
        point: Vec3,
        canvas_width: f64,
        canvas_height: f64,
        projection: Projection,
    ) -> Option<(isize, isize, f64)> {
        let cam_point = self.transform(point);
        project_with_projection(cam_point, canvas_width, canvas_height, projection)
    }
}

/// Stores object IDs for picking/interaction.
#[derive(Clone, Debug, PartialEq)]
pub struct IdBuffer {
    width: usize,
    height: usize,
    ids: Vec<Option<u32>>,
}

impl IdBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            ids: vec![None; width * height],
        }
    }

    pub fn from_canvas<R: CellRenderer>(canvas: &CellCanvas<R>) -> Self {
        Self::new(canvas.pixel_width(), canvas.pixel_height())
    }

    pub fn clear(&mut self) {
        self.ids.fill(None);
    }

    pub fn get(&self, x: usize, y: usize) -> Option<u32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.ids[y * self.width + x]
    }

    fn set(&mut self, x: usize, y: usize, id: u32) {
        let idx = y * self.width + x;
        self.ids[idx] = Some(id);
    }
}

pub fn project_with_projection(
    v_cam: Vec3,
    canvas_width: f64,
    canvas_height: f64,
    projection: Projection,
) -> Option<(isize, isize, f64)> {
    if v_cam.z <= projection.near_plane {
        return None;
    }

    let center_x = canvas_width * projection.center_x_ratio;
    let center_y = canvas_height * projection.center_y_ratio;
    let screen_x = center_x + (v_cam.x / v_cam.z) * projection.scale_x;
    let screen_y = center_y + (v_cam.y / v_cam.z) * projection.scale_y;

    Some((
        screen_x.round() as isize,
        screen_y.round() as isize,
        v_cam.z,
    ))
}

pub fn project_to_screen(
    v_cam: Vec3,
    canvas_width: f64,
    canvas_height: f64,
    scale: f64,
) -> Option<(isize, isize, f64)> {
    project_with_projection(
        v_cam,
        canvas_width,
        canvas_height,
        Projection::new(0.06, 0.5, 0.5, scale * 2.0, scale),
    )
}

pub fn make_sphere_points(lat_steps: usize, lon_steps: usize) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(lat_steps * lon_steps);
    for lat in 0..lat_steps {
        let v = lat as f64 / (lat_steps - 1).max(1) as f64;
        let theta = v * std::f64::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        for lon in 0..lon_steps {
            let u = lon as f64 / lon_steps as f64;
            let phi = u * std::f64::consts::TAU;
            let (sin_phi, cos_phi) = phi.sin_cos();
            points.push(Vec3::new(
                sin_theta * cos_phi,
                cos_theta,
                sin_theta * sin_phi,
            ));
        }
    }
    points
}

pub fn make_torus_rings(
    major_radius: f64,
    minor_radius: f64,
    major_steps: usize,
    minor_steps: usize,
) -> Vec<Vec<Vec3>> {
    let mut rings = Vec::with_capacity(major_steps);
    for major in 0..major_steps {
        let u = major as f64 / major_steps as f64 * std::f64::consts::TAU;
        let (sin_u, cos_u) = u.sin_cos();
        let mut ring = Vec::with_capacity(minor_steps + 1);
        for minor in 0..=minor_steps {
            let v = minor as f64 / minor_steps as f64 * std::f64::consts::TAU;
            let (sin_v, cos_v) = v.sin_cos();
            let x = (major_radius + minor_radius * cos_v) * cos_u;
            let y = minor_radius * sin_v;
            let z = (major_radius + minor_radius * cos_v) * sin_u;
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

pub fn make_circle_3d(center: Vec3, radius: f64, steps: usize) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(steps);
    for i in 0..steps {
        let angle = i as f64 / steps as f64 * std::f64::consts::TAU;
        let (sin_a, cos_a) = angle.sin_cos();
        points.push(center + Vec3::new(cos_a * radius, 0.0, sin_a * radius));
    }
    points
}

pub struct Box3D {
    pub vertices: [Vec3; 8],
    pub edges: [(usize, usize); 12],
}

pub fn make_box_3d(center: Vec3, width: f64, height: f64, depth: f64) -> Box3D {
    let w2 = width / 2.0;
    let h2 = height / 2.0;
    let d2 = depth / 2.0;

    let vertices = [
        center + Vec3::new(-w2, -h2, -d2),
        center + Vec3::new(w2, -h2, -d2),
        center + Vec3::new(w2, h2, -d2),
        center + Vec3::new(-w2, h2, -d2),
        center + Vec3::new(-w2, -h2, d2),
        center + Vec3::new(w2, -h2, d2),
        center + Vec3::new(w2, h2, d2),
        center + Vec3::new(-w2, h2, d2),
    ];

    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];

    Box3D { vertices, edges }
}

pub fn plot_z<R: CellRenderer>(
    canvas: &mut CellCanvas<R>,
    zbuf: &mut ZBuffer,
    x: isize,
    y: isize,
    depth: f64,
    color: Color,
) {
    if x < 0 || y < 0 {
        return;
    }

    let x = x as usize;
    let y = y as usize;
    if x >= zbuf.width || y >= zbuf.height {
        return;
    }

    if zbuf.test_and_set(x, y, depth) {
        canvas.set_pixel_screen(x, y, Some(color));
    }
}

pub fn line_z<R: CellRenderer>(
    canvas: &mut CellCanvas<R>,
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
            canvas,
            zbuf,
            x.round() as isize,
            y.round() as isize,
            z,
            color,
        );
    }
}

pub fn plot_z_id<R: CellRenderer>(
    canvas: &mut CellCanvas<R>,
    zbuf: &mut ZBuffer,
    idbuf: &mut IdBuffer,
    x: isize,
    y: isize,
    depth: f64,
    color: Color,
    id: u32,
) {
    if x < 0 || y < 0 {
        return;
    }

    let x = x as usize;
    let y = y as usize;
    if x >= zbuf.width || y >= zbuf.height {
        return;
    }

    if zbuf.test_and_set(x, y, depth) {
        canvas.set_pixel_screen(x, y, Some(color));
        idbuf.set(x, y, id);
    }
}

pub fn line_z_id<R: CellRenderer>(
    canvas: &mut CellCanvas<R>,
    zbuf: &mut ZBuffer,
    idbuf: &mut IdBuffer,
    start: (isize, isize, f64),
    end: (isize, isize, f64),
    color: Color,
    id: u32,
) {
    let steps = (end.0 - start.0).abs().max((end.1 - start.1).abs()).max(1) as usize;

    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let x = start.0 as f64 + (end.0 - start.0) as f64 * t;
        let y = start.1 as f64 + (end.1 - start.1) as f64 * t;
        let z = start.2 + (end.2 - start.2) * t;
        plot_z_id(
            canvas,
            zbuf,
            idbuf,
            x.round() as isize,
            y.round() as isize,
            z,
            color,
            id,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        line_z_id, plot_z, plot_z_id, project_with_projection, IdBuffer, OrbitCamera, Projection,
        Vec3, ZBuffer,
    };
    use crate::BrailleCanvas;
    use colored::Color;

    #[test]
    fn projection_maps_camera_center_to_canvas_center() {
        let point = Vec3::new(0.0, 0.0, 2.0);
        let projection = Projection::new(0.1, 0.5, 0.5, 20.0, 10.0);

        let projected = project_with_projection(point, 80.0, 40.0, projection);

        assert_eq!(projected, Some((40, 20, 2.0)));
    }

    #[test]
    fn zbuffer_keeps_nearer_point_color() {
        let mut canvas = BrailleCanvas::new(1, 1);
        let mut zbuf = ZBuffer::from_canvas(&canvas);

        plot_z(&mut canvas, &mut zbuf, 0, 0, 2.0, Color::Blue);
        plot_z(&mut canvas, &mut zbuf, 0, 0, 1.0, Color::Red);

        let rendered = canvas.render_with_options(false, None);
        assert!(rendered.contains("\x1b[31m"));
        assert!(!rendered.contains("\x1b[34m"));
    }

    #[test]
    fn orbit_camera_position_respects_distance() {
        let cam = OrbitCamera::new(Vec3::new(0.0, 0.0, 0.0), 10.0, 0.0, 0.0);
        let pos = cam.position();
        assert!((pos.z - 10.0).abs() < 1e-9);
    }

    #[test]
    fn id_buffer_records_picking_ids() {
        let mut canvas = BrailleCanvas::new(1, 1);
        let mut zbuf = ZBuffer::from_canvas(&canvas);
        let mut idbuf = IdBuffer::from_canvas(&canvas);

        plot_z_id(
            &mut canvas,
            &mut zbuf,
            &mut idbuf,
            0,
            0,
            1.0,
            Color::White,
            42,
        );

        assert_eq!(idbuf.get(0, 0), Some(42));
    }

    #[test]
    fn line_z_id_records_ids() {
        let mut canvas = BrailleCanvas::new(5, 1);
        let mut zbuf = ZBuffer::from_canvas(&canvas);
        let mut idbuf = IdBuffer::from_canvas(&canvas);

        line_z_id(
            &mut canvas,
            &mut zbuf,
            &mut idbuf,
            (0, 0, 1.0),
            (4, 0, 1.0),
            Color::White,
            7,
        );

        assert_eq!(idbuf.get(0, 0), Some(7));
        assert_eq!(idbuf.get(2, 0), Some(7));
        assert_eq!(idbuf.get(4, 0), Some(7));
    }

    #[test]
    fn id_buffer_respects_z_depth() {
        let mut canvas = BrailleCanvas::new(1, 1);
        let mut zbuf = ZBuffer::from_canvas(&canvas);
        let mut idbuf = IdBuffer::from_canvas(&canvas);

        plot_z_id(
            &mut canvas,
            &mut zbuf,
            &mut idbuf,
            0,
            0,
            10.0,
            Color::White,
            1,
        );
        plot_z_id(
            &mut canvas,
            &mut zbuf,
            &mut idbuf,
            0,
            0,
            5.0,
            Color::White,
            2,
        );

        assert_eq!(idbuf.get(0, 0), Some(2));
    }
}
