use colored::Color;
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::thread; // <-- FIX: thread import was missing
use std::time::{Duration, Instant};
use txtplot::ChartContext;

// ============================================================================
// BASIC 3D MATH AND PHYSICS ENGINE
// ============================================================================

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
    fn add(self, o: Vec3) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    fn sub(self, o: Vec3) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
    fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    fn normalize(self) -> Self {
        let l = self.norm();
        if l > 0.0 {
            Self::new(self.x / l, self.y / l, self.z / l)
        } else {
            self
        }
    }
}

// Euler rotation functions (3D rotation matrix)
// They rotate a point in space around the X, Y, or Z axes.
fn rotate_x(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}
fn rotate_y(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}
fn rotate_z(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
}

// ============================================================================
// GRAPHICS ENGINE: Z-BUFFER AND ID-BUFFER
// ============================================================================
struct ZBuffer {
    w: usize,
    h: usize,
    z: Vec<f64>,
    id: Vec<Option<usize>>, // Tracks which planet drew each pixel (for mouse picking)
}
impl ZBuffer {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            z: vec![f64::INFINITY; w * h],
            id: vec![None; w * h],
        }
    }
    fn clear(&mut self) {
        self.z.fill(f64::INFINITY);
        self.id.fill(None);
    }
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    // Depth test: if the new point is closer (depth < stored z),
    // update the screen. Otherwise it stays hidden behind what was already drawn.
    fn test_and_set(&mut self, x: usize, y: usize, depth: f64, body_id: Option<usize>) -> bool {
        let i = self.idx(x, y);
        if depth < self.z[i] {
            self.z[i] = depth;
            self.id[i] = body_id; // Record the planet that owns this pixel
            true
        } else {
            false
        }
    }
}

// ============================================================================
// CELESTIAL MECHANICS
// ============================================================================

/// Solves Kepler's Equation (M = E - e * sin(E)) using Newton-Raphson.
/// Converts "Mean Anomaly" (time) into "Eccentric Anomaly" (geometric position).
fn solve_kepler(m: f64, e: f64) -> f64 {
    let mut e_est = m;
    for _ in 0..5 {
        e_est = e_est - (e_est - e * e_est.sin() - m) / (1.0 - e * e_est.cos());
    }
    e_est
}

/// Represents a celestial body with physically inspired properties.
/// Astronomical values are scaled for graphical visualization.
struct CelestialBody {
    name: &'static str,
    parent: Option<usize>, // Index of the body it orbits around (e.g. the Moon orbits Earth)
    color: Color,
    radius: f64, // Volumetric radius of the body

    // --- KEPLERIAN ORBITAL ELEMENTS (define orbit shape and orientation) ---
    a: f64,     // Semi-major axis: orbit size (average distance from the center).
    e: f64,     // Eccentricity: how flattened the ellipse is (0 = perfect circle, <1 = ellipse)
    i: f64,     // Orbital inclination: vertical angle relative to the reference plane (ecliptic).
    omega: f64, // Longitude of the ascending node: where the orbit crosses upward through the reference plane.
    w: f64,     // Argument of perihelion: orientation of the ellipse within its own plane.
    w_dot: f64, // Perihelion precession: how much the ellipse rotates over time (relativity/perturbations).
    m0: f64,    // Initial mean anomaly: planet position in the orbit at t=0.
    n: f64,     // Mean motion: orbital angular speed (radians per unit time).

    // --- ROTATION AND ORIENTATION ELEMENTS ("wobble" of the planet) ---
    axial_tilt: f64, // Base obliquity: axis tilt (e.g. Earth ~23.5° -> 0.41 radians).
    rot_rate: f64,   // Rotation rate: how fast the planet spins on its own axis (day length).
    prec_rate: f64,  // Equinox precession: the top-like wobble of the spin axis.
    nut_amp: f64,    // Nutation amplitude: small nodding motion on top of precession.
    nut_rate: f64,   // Nutation frequency.
    cw_amp: f64,     // Chandler wobble amplitude: tiny oscillation of the poles.
    cw_rate: f64,    // Chandler wobble frequency.

    // --- RINGS ---
    #[allow(dead_code)]
    ring_inner: f64, // Inner ring radius (0 if none)
    #[allow(dead_code)]
    ring_outer: f64, // Outer ring radius
    #[allow(dead_code)]
    ring_color: Color,
    is_star: bool, // Whether it emits its own light (no dark side)
}

impl CelestialBody {
    /// Computes the planet center position in 3D space at time `t`.
    /// The position is RELATIVE to its parent (Sun or another planet).
    fn get_local_orbit_pos(&self, t: f64) -> Vec3 {
        // If it has no orbit radius (it is the Sun), it stays at the center.
        if self.a == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        // 1. Ellipse calculations (2D within an orbital plane)
        let m = self.m0 + self.n * t;
        let e_anom = solve_kepler(m, self.e);
        let nu = 2.0 * (((1.0 + self.e) / (1.0 - self.e)).sqrt() * (e_anom / 2.0).tan()).atan(); // True anomaly
        let r = self.a * (1.0 - self.e * e_anom.cos()); // Distance from the focus

        // 2. Apply perihelion precession (orbit rotation over time)
        let current_w = self.w + self.w_dot * t;

        // 3. 3D transform: apply orbital tilt and orientation in space
        let mut p = Vec3::new(r * nu.cos(), 0.0, r * nu.sin());
        p = rotate_y(p, current_w); // Rotate the ellipse within its own plane
        p = rotate_x(p, self.i); // Tilt the orbit relative to the ecliptic
        p = rotate_y(p, self.omega); // Rotate the orbit around the Sun
        p
    }

    /// Computes the world-space position of a sphere vertex,
    /// applying axial deformations before moving it onto the orbit.
    /// Returns: (final world position, rotated vertex normal for lighting)
    fn get_vertex_data(&self, local_v: Vec3, t: f64, absolute_orbit_pos: Vec3) -> (Vec3, Vec3) {
        // The normal starts as the original spherical position (pointing outward)
        let mut normal = local_v;
        let mut v = Vec3::new(
            local_v.x * self.radius,
            local_v.y * self.radius,
            local_v.z * self.radius,
        );

        // A. Chandler wobble (local polar oscillation)
        let cw_x = self.cw_amp * (self.cw_rate * t).cos();
        let cw_z = self.cw_amp * (self.cw_rate * t).sin();
        v = rotate_x(v, cw_x);
        v = rotate_z(v, cw_z);
        normal = rotate_x(normal, cw_x);
        normal = rotate_z(normal, cw_z);

        // B. Daily rotation (spin around the local Y axis)
        v = rotate_y(v, self.rot_rate * t);
        normal = rotate_y(normal, self.rot_rate * t);

        // C. Obliquity and nutation (axis tilt)
        let current_tilt = self.axial_tilt + self.nut_amp * (self.nut_rate * t).cos();
        v = rotate_x(v, current_tilt);
        normal = rotate_x(normal, current_tilt);

        // D. Equinox precession (top-like spin)
        v = rotate_y(v, self.prec_rate * t);
        normal = rotate_y(normal, self.prec_rate * t);

        // E. Finally, move the point onto its orbital position
        v = v.add(absolute_orbit_pos);
        (v, normal.normalize())
    }

    /// Same as get_vertex_pos, but for ring particles.
    #[allow(dead_code)]
    fn get_ring_pos(&self, local_v: Vec3, t: f64, absolute_orbit_pos: Vec3) -> Vec3 {
        let mut v = local_v;
        // Rings do not spin at the planet rate, but they do share its tilt and precession.
        let current_tilt = self.axial_tilt + self.nut_amp * (self.nut_rate * t).cos();
        v = rotate_x(v, current_tilt);
        v = rotate_y(v, self.prec_rate * t);
        v.add(absolute_orbit_pos)
    }
}

// ============================================================================
// HELPER FUNCTIONS - GEOMETRY GENERATION
// ============================================================================

/// Creates a point cloud forming a 3D sphere from latitude/longitude sampling.
fn make_sphere_points(lat_steps: usize, lon_steps: usize) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(lat_steps * lon_steps);
    for i in 0..lat_steps {
        let v = i as f64 / (lat_steps - 1).max(1) as f64;
        let theta = v * std::f64::consts::PI;
        let st = theta.sin();
        let ct = theta.cos();
        for j in 0..lon_steps {
            let u = j as f64 / lon_steps as f64;
            let phi = u * std::f64::consts::TAU;
            pts.push(Vec3::new(st * phi.cos(), ct, st * phi.sin()));
        }
    }
    pts
}

/// Projects a 3D coordinate (X, Y, Z) into a 2D screen pixel (X, Y) with perspective.
fn project_to_screen(v_cam: Vec3, w: f64, h: f64, scale: f64) -> Option<(isize, isize, f64)> {
    if v_cam.z <= 0.1 {
        return None;
    }
    let px = (v_cam.x / v_cam.z) * 2.0;
    let py = v_cam.y / v_cam.z;
    Some((
        (w / 2.0 + px * scale).round() as isize,
        (h / 2.0 + py * scale).round() as isize,
        v_cam.z,
    ))
}

fn plot_z(
    chart: &mut ChartContext,
    zb: &mut ZBuffer,
    x: isize,
    y: isize,
    z: f64,
    col: Color,
    id: Option<usize>,
) {
    if x < 0 || y < 0 {
        return;
    }
    let (ux, uy) = (x as usize, y as usize);
    if ux < zb.w && uy < zb.h && zb.test_and_set(ux, uy, z, id) {
        chart.canvas.set_pixel_screen(ux, uy, Some(col));
    }
}

// ============================================================================
// SOLAR SYSTEM SETUP
// ============================================================================

/// Helper function (macro-like) for instantiating bodies in one line.
///
/// Parameters:
/// - name: Body name (e.g. "Earth", "Halley").
/// - p:    Parent body index. `None` if it orbits the Sun. `Some(index)` if it is a moon.
/// - c:    Base terminal color of the body.
/// - r:    Visual radius (object size in the simulation).
///
/// --- Orbital Elements (Kepler) ---
/// - a:    Semi-major axis (average orbital distance).
/// - e:    Eccentricity (0 = perfect circle, <1 = ellipse, >1 = escape).
/// - i:    Orbital inclination relative to the ecliptic (in radians).
/// - om:   Longitude of the ascending node (uppercase omega).
/// - w:    Argument of perihelion (lowercase omega).
/// - wd:   Perihelion precession (w_dot - rate of orbital change over time).
/// - m:    Initial mean anomaly (starting position at t=0).
/// - n:    Mean motion (orbital speed).
///
/// --- Rotation / Axis Elements ---
/// - tilt: Obliquity (planet axis tilt).
/// - rot:  Rotation speed around its own axis (day length).
/// - pr:   Equinox precession (long-term axis wobble).
///
/// --- Ring System ---
/// - ri:   Inner ring radius (0.0 if none).
/// - ro:   Outer ring radius (0.0 if none).
/// - rc:   Ring color.
///
/// --- Lighting Properties ---
/// - star: `true` if it emits its own light (no dark side, e.g. the Sun).
///   `false` if it receives light (has a day/night side depending on Sun position).
#[expect(
    clippy::too_many_arguments,
    reason = "the example dataset is kept as explicit orbital-element literals"
)]
fn body(
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

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        cursor::Hide,
        terminal::Clear(ClearType::All),
        EnableMouseCapture
    )?;

    // ==================================================
    // SOLAR SYSTEM DATASET
    // ==================================================
    // IMPORTANT: A "parent" body must be defined BEFORE its moon/satellite (lower index).
    let bodies = vec![
        // SUN (0) - Center of mass. a=0. (`true` at the end marks it as a star)
        body(
            "Sun",
            None,
            Color::BrightYellow,
            2.5,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.1,
            0.5,
            0.0,
            0.0,
            0.0,
            Color::White,
            true,
        ),
        // INNER PLANETS
        body(
            "Mercury",
            None,
            Color::White,
            0.4,
            5.0,
            0.25,
            0.12,
            0.8,
            1.3,
            0.02,
            0.0,
            0.4,
            0.0,
            0.1,
            0.01,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Venus",
            None,
            Color::Yellow,
            0.75,
            8.0,
            0.05,
            0.06,
            1.3,
            0.9,
            0.01,
            1.5,
            0.25,
            3.09,
            -0.05,
            0.02,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        // EARTH AND MOON (Earth = index 3)
        body(
            "Earth",
            None,
            Color::Blue,
            0.8,
            12.0,
            0.1,
            0.0,
            1.0,
            0.0,
            0.015,
            0.0,
            0.15,
            0.41,
            5.0,
            0.05,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Moon",
            Some(3),
            Color::White,
            0.25,
            1.6,
            0.05,
            0.08,
            0.5,
            0.0,
            0.05,
            1.0,
            1.8,
            0.1,
            1.8,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        // MARS AND ITS MOONS (Mars = index 5)
        body(
            "Mars",
            None,
            Color::Red,
            0.5,
            16.0,
            0.2,
            0.03,
            0.5,
            2.0,
            0.008,
            1.0,
            0.08,
            0.44,
            4.8,
            0.03,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Phobos",
            Some(5),
            Color::White,
            0.15,
            0.8,
            0.01,
            0.01,
            0.0,
            0.0,
            0.0,
            0.0,
            4.0,
            0.0,
            4.0,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Deimos",
            Some(5),
            Color::BrightBlack,
            0.12,
            1.3,
            0.01,
            0.02,
            1.0,
            0.0,
            0.0,
            2.0,
            2.8,
            0.0,
            2.8,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        // JUPITER AND ITS GALILEAN MOONS (Jupiter = index 8)
        body(
            "Jupiter",
            None,
            Color::BrightMagenta,
            1.8,
            26.0,
            0.1,
            0.02,
            1.7,
            0.2,
            0.005,
            2.0,
            0.03,
            0.05,
            12.0,
            0.01,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Io",
            Some(8),
            Color::Yellow,
            0.2,
            2.4,
            0.0,
            0.01,
            0.0,
            0.0,
            0.0,
            0.0,
            3.5,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Europa",
            Some(8),
            Color::White,
            0.18,
            3.0,
            0.01,
            0.01,
            1.0,
            0.0,
            0.0,
            1.5,
            2.8,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Ganymede",
            Some(8),
            Color::BrightBlack,
            0.25,
            3.7,
            0.0,
            0.0,
            2.0,
            0.0,
            0.0,
            3.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        body(
            "Callisto",
            Some(8),
            Color::BrightBlack,
            0.22,
            4.6,
            0.0,
            0.0,
            3.0,
            0.0,
            0.0,
            4.5,
            1.4,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        // SATURN AND TITAN (Saturn = index 13)
        body(
            "Saturn",
            None,
            Color::BrightYellow,
            1.5,
            38.0,
            0.1,
            0.04,
            2.0,
            1.5,
            0.003,
            3.0,
            0.015,
            0.46,
            11.0,
            0.015,
            1.8,
            3.5,
            Color::Yellow,
            false,
        ),
        body(
            "Titan",
            Some(13),
            Color::BrightYellow,
            0.26,
            4.5,
            0.03,
            0.1,
            0.0,
            0.0,
            0.01,
            0.0,
            1.2,
            0.0,
            1.2,
            0.0,
            0.0,
            0.0,
            Color::White,
            false,
        ),
        // URANUS AND NEPTUNE
        body(
            "Uranus",
            None,
            Color::BrightCyan,
            1.1,
            52.0,
            0.1,
            0.01,
            1.2,
            1.1,
            0.002,
            4.0,
            0.007,
            1.71,
            -7.0,
            0.005,
            1.3,
            1.9,
            Color::BrightBlack,
            false,
        ),
        body(
            "Neptune",
            None,
            Color::Blue,
            1.0,
            68.0,
            0.05,
            0.03,
            2.3,
            0.5,
            0.001,
            5.0,
            0.004,
            0.49,
            7.5,
            0.003,
            0.0,
            0.0,
            Color::White,
            false,
        ),
    ];
    // ==================================================
    // INITIAL ENGINE STATE
    // ==================================================
    let mut cam_pos = Vec3::new(0.0, -20.0, -45.0); // Camera position in X, Y, Z
    let mut cam_pitch = 0.4_f64; // Look up/down
    let mut cam_yaw = 0.0_f64; // Look left/right
    let mut zoom = 1.0_f64; // Scale (FOV)

    let mut sim_time = 0.0_f64; // Elapsed cosmic time
    let mut time_scale = 0.1_f64; // How much time advances per frame
    let mut saved_time_scale = 0.1_f64; // For pause (space bar)

    let mut detail: i32 = 4; // Sphere mesh quality (LOD)
    let mut regen_mesh = true;
    let mut sphere_pts: Vec<Vec3> = Vec::new();

    let mut chart: Option<ChartContext> = None;
    let mut zbuf: Option<ZBuffer> = None;
    let mut last_term: (u16, u16) = (0, 0);
    let mut show_orbits = true;
    let mut show_help = false;

    // Mouse state for gesture detection
    let mut is_dragging = false;
    let mut last_mouse_pos: Option<(u16, u16)> = None;

    // Selection and tracking systems
    let mut selected_body: Option<usize> = None;
    let mut follow_body: Option<usize> = None;

    // Stress monitor
    let mut frames = 0;
    let mut last_fps_time = Instant::now();
    let mut current_fps = 0;
    //let mut drawn_vertices = 0;

    // ==================================================
    // MAIN LOOP
    // ==================================================
    loop {
        let loop_start = Instant::now();
        frames += 1;
        if last_fps_time.elapsed().as_secs() >= 1 {
            current_fps = frames;
            frames = 0;
            last_fps_time = Instant::now();
        }

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        if (cols, rows) != last_term || chart.is_none() {
            last_term = (cols, rows);
            let width = (cols as usize).saturating_sub(4);
            let height = (rows as usize).saturating_sub(6);
            let new_chart = ChartContext::new(width, height);
            zbuf = Some(ZBuffer::new(
                new_chart.canvas.pixel_width(),
                new_chart.canvas.pixel_height(),
            ));
            chart = Some(new_chart);
            regen_mesh = true;
        }

        if regen_mesh {
            let d = detail as usize;
            sphere_pts = make_sphere_points((10 + d * 4).min(140), (20 + d * 8).min(260));
            regen_mesh = false;
        }

        let chart_ref = chart.as_mut().unwrap();
        let zb = zbuf.as_mut().unwrap();
        // Keep the previous zbuffer frame around for tracking; clear it later instead
        //chart_ref.canvas.clear();
        //zb.clear();
        //drawn_vertices = 0;

        // --- INPUT ---
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // Keyboard events
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                            execute!(
                                stdout,
                                cursor::Show,
                                terminal::Clear(ClearType::All),
                                DisableMouseCapture
                            )?;
                            terminal::disable_raw_mode()?;
                            return Ok(());
                        }

                        // Movement cancels planet follow mode
                        (KeyCode::Char('w'), _) => {
                            cam_pos.z += 2.0 * cam_yaw.cos();
                            cam_pos.x += 2.0 * cam_yaw.sin();
                            follow_body = None;
                        }
                        (KeyCode::Char('s'), _) => {
                            cam_pos.z -= 2.0 * cam_yaw.cos();
                            cam_pos.x -= 2.0 * cam_yaw.sin();
                            follow_body = None;
                        }
                        (KeyCode::Char('a'), _) => {
                            cam_pos.x -= 2.0 * cam_yaw.cos();
                            cam_pos.z += 2.0 * cam_yaw.sin();
                            follow_body = None;
                        }
                        (KeyCode::Char('d'), _) => {
                            cam_pos.x += 2.0 * cam_yaw.cos();
                            cam_pos.z -= 2.0 * cam_yaw.sin();
                            follow_body = None;
                        }
                        (KeyCode::Char('e'), _) => cam_pos.y += 1.5,
                        (KeyCode::Char('c'), _) => cam_pos.y -= 1.5,

                        // Manual camera rotation
                        (KeyCode::Left, _) => cam_yaw += 0.1,
                        (KeyCode::Right, _) => cam_yaw -= 0.1,
                        (KeyCode::Up, _) => cam_pitch += 0.1,
                        (KeyCode::Down, _) => cam_pitch -= 0.1,

                        // Time control
                        (KeyCode::Char('i'), m) if m.contains(KeyModifiers::ALT) => {
                            time_scale += 0.01
                        }
                        (KeyCode::Char('i'), _) => time_scale += 0.05,
                        (KeyCode::Char('u'), m) if m.contains(KeyModifiers::ALT) => {
                            time_scale -= 0.01
                        }
                        (KeyCode::Char('u'), _) => time_scale -= 0.05,
                        (KeyCode::Char('p'), _) => time_scale = 0.0,

                        // Zoom and detail
                        (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => {
                            zoom = (zoom * 1.1).min(20.0)
                        }
                        (KeyCode::Char('-'), _) => zoom = (zoom / 1.1).max(0.1),
                        (KeyCode::Char('m'), _) => {
                            detail = (detail + 1).min(33);
                            regen_mesh = true;
                        }
                        (KeyCode::Char('n'), _) => {
                            detail = (detail - 1).max(1);
                            regen_mesh = true;
                        }

                        // Visual options and features
                        (KeyCode::Char('o'), _) => show_orbits = !show_orbits,
                        (KeyCode::Char('h'), _) | (KeyCode::Char('H'), _) => show_help = !show_help,

                        // More intuitive pause/resume with Space
                        (KeyCode::Char(' '), _) => {
                            if time_scale == 0.0 {
                                time_scale = saved_time_scale;
                            } else {
                                saved_time_scale = time_scale;
                                time_scale = 0.0;
                            }
                        }

                        _ => {}
                    }
                }
                // Mouse events
                Event::Mouse(me) => {
                    // Magnetic selection engine (tolerant hitbox)
                    let get_clicked_id = |col: u16, row: u16, zb: &ZBuffer| -> Option<usize> {
                        // Approximate terminal frame offset
                        let mouse_px = (col.saturating_sub(2) as isize) * 2;
                        let mouse_py = (row.saturating_sub(2) as isize) * 4;

                        let mut clicked_id = None;
                        let mut min_dist = 40.0; // Attraction radius of the "magnet" (40 Braille pixels of tolerance)
                        let mut closest_z = f64::INFINITY;

                        // Search window so we do not scan the entire screen
                        let min_x = (mouse_px - 40).max(0) as usize;
                        let max_x = (mouse_px + 40).min(zb.w as isize - 1) as usize;
                        let min_y = (mouse_py - 40).max(0) as usize;
                        let max_y = (mouse_py + 40).min(zb.h as isize - 1) as usize;

                        for py in min_y..=max_y {
                            for px in min_x..=max_x {
                                let idx = zb.idx(px, py);
                                if let Some(id) = zb.id[idx] {
                                    // Compute distance to the real mouse position
                                    let dx = px as f64 - mouse_px as f64;
                                    let dy = py as f64 - mouse_py as f64;
                                    let dist = (dx * dx + dy * dy).sqrt();

                                    // Keep the drawn pixel closest to the mouse
                                    if dist < min_dist {
                                        min_dist = dist;
                                        closest_z = zb.z[idx];
                                        clicked_id = Some(id);
                                    } else if (dist - min_dist).abs() < 1.0 && zb.z[idx] < closest_z
                                    {
                                        // Tie-breaker: if two are almost equal, pick the one in front (smaller Z)
                                        closest_z = zb.z[idx];
                                        clicked_id = Some(id);
                                    }
                                }
                            }
                        }
                        clicked_id
                    };

                    match me.kind {
                        // LEFT CLICK: drag camera or select
                        MouseEventKind::Down(MouseButton::Left) => {
                            is_dragging = true;
                            last_mouse_pos = Some((me.column, me.row));
                            // Select the touched body (even near misses are caught by the magnet)
                            selected_body = get_clicked_id(me.column, me.row, zb);
                        }
                        // RIGHT CLICK: follow planet
                        MouseEventKind::Down(MouseButton::Right) => {
                            let id = get_clicked_id(me.column, me.row, zb);
                            follow_body = id;
                            if id.is_some() {
                                // Following it also selects it automatically
                                selected_body = id;
                            }
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            is_dragging = false;
                            last_mouse_pos = None;
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            if is_dragging {
                                if let Some((lx, ly)) = last_mouse_pos {
                                    cam_yaw -= (me.column as f64 - lx as f64) * 0.015;
                                    cam_pitch += (me.row as f64 - ly as f64) * 0.015;
                                }
                                last_mouse_pos = Some((me.column, me.row));
                            }
                        }
                        // Mouse wheel: zoom or time speed (with Ctrl)
                        MouseEventKind::ScrollUp => {
                            if me.modifiers.contains(KeyModifiers::CONTROL) {
                                time_scale += 0.02;
                            } else {
                                zoom = (zoom * 1.1).min(25.0);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if me.modifiers.contains(KeyModifiers::CONTROL) {
                                time_scale -= 0.02;
                            } else {
                                zoom = (zoom / 1.1).max(0.05);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Clear the canvas now because the previous frame was used for tracking
        chart_ref.canvas.clear();
        zb.clear();
        let mut drawn_vertices = 0;

        // --- PHYSICAL STATE UPDATE ---
        sim_time += time_scale;
        let cw = chart_ref.canvas.pixel_width() as f64;
        let ch = chart_ref.canvas.pixel_height() as f64;
        let scale = (cw.min(ch) / 2.0) * zoom;

        // Absolute position calculation
        let mut abs_pos = vec![Vec3::new(0.0, 0.0, 0.0); bodies.len()];
        for (i, body) in bodies.iter().enumerate() {
            let mut p = body.get_local_orbit_pos(sim_time);
            if let Some(parent_idx) = body.parent {
                p = p.add(abs_pos[parent_idx]);
            }
            abs_pos[i] = p;
        }

        let camera_target_offset = if let Some(id) = follow_body {
            abs_pos[id]
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };

        let to_screen = |v_world: Vec3| -> Option<(isize, isize, f64)> {
            let mut v_cam = v_world.sub(camera_target_offset).sub(cam_pos);
            v_cam = rotate_y(v_cam, -cam_yaw);
            v_cam = rotate_x(v_cam, -cam_pitch);
            project_to_screen(v_cam, cw, ch, scale)
        };

        // --- UNIVERSE RENDERING ---
        let sun_pos = abs_pos[0];

        for (i, body) in bodies.iter().enumerate() {
            let orbit_pos = abs_pos[i];

            if show_orbits && body.a > 0.0 {
                let segments = 60;
                let mut prev_proj: Option<(isize, isize, f64)> = None;
                let parent_pos = if let Some(p_idx) = body.parent {
                    abs_pos[p_idx]
                } else {
                    sun_pos
                };

                for step_idx in 0..=segments {
                    let m = (step_idx as f64 / segments as f64) * std::f64::consts::TAU;
                    let e_anom = solve_kepler(m, body.e);
                    let nu = 2.0
                        * (((1.0 + body.e) / (1.0 - body.e)).sqrt() * (e_anom / 2.0).tan()).atan();
                    let r = body.a * (1.0 - body.e * e_anom.cos());
                    let current_w = body.w + body.w_dot * sim_time;

                    let mut p = Vec3::new(r * nu.cos(), 0.0, r * nu.sin());
                    p = rotate_y(p, current_w);
                    p = rotate_x(p, body.i);
                    p = rotate_y(p, body.omega);
                    p = p.add(parent_pos);

                    if let Some(proj) = to_screen(p) {
                        if let Some(prev) = prev_proj {
                            let dx = (proj.0 - prev.0).abs();
                            let dy = (proj.1 - prev.1).abs();
                            let steps = dx.max(dy).max(1) as i32;
                            for s in 0..=steps {
                                let t = s as f64 / steps as f64;
                                let xf = prev.0 as f64 + (proj.0 as f64 - prev.0 as f64) * t;
                                let yf = prev.1 as f64 + (proj.1 as f64 - prev.1 as f64) * t;
                                let zf = prev.2 + (proj.2 - prev.2) * t;
                                plot_z(
                                    chart_ref,
                                    zb,
                                    xf.round() as isize,
                                    yf.round() as isize,
                                    zf,
                                    Color::BrightBlack,
                                    None,
                                );
                            }
                        }
                        prev_proj = Some(proj);
                    } else {
                        prev_proj = None;
                    }
                }
            }

            // Spheres with Lambert shading
            for p0 in sphere_pts.iter() {
                let (v_world, normal) = body.get_vertex_data(*p0, sim_time, orbit_pos);

                if let Some((sx, sy, z)) = to_screen(v_world) {
                    drawn_vertices += 1;
                    let final_color;

                    if body.is_star {
                        final_color = body.color;
                    } else {
                        let light_dir = sun_pos.sub(v_world).normalize();
                        let intensity = normal.dot(light_dir);

                        if intensity > 0.4 {
                            final_color = body.color;
                        } else if intensity > 0.0 {
                            final_color = Color::BrightBlack;
                        } else {
                            continue;
                        }
                    }

                    plot_z(chart_ref, zb, sx, sy, z, final_color, Some(i));
                }
            }
        }

        // --- HUD AND MONITOR ---
        let ms = loop_start.elapsed().as_millis();

        // Render either the help menu or the normal HUD
        if show_help {
            chart_ref.text(
                "======= CONTROLS HELP =======",
                0.35,
                0.20,
                Some(Color::BrightYellow),
            );
            chart_ref.text(
                "[W/A/S/D] Camera Move (Cancels Follow)",
                0.35,
                0.25,
                Some(Color::White),
            );
            chart_ref.text("[E/C]     Camera Up/Down", 0.35, 0.30, Some(Color::White));
            chart_ref.text(
                "[Arrows]  Camera Look (Yaw/Pitch)",
                0.35,
                0.35,
                Some(Color::White),
            );
            chart_ref.text(
                "[U/I]     Adjust Time Speed (Also Ctrl+Wheel)",
                0.35,
                0.40,
                Some(Color::White),
            );
            chart_ref.text(
                "[Space]   Pause / Resume Time",
                0.35,
                0.45,
                Some(Color::White),
            );
            chart_ref.text(
                "[+/-]     Zoom (Also Mouse Wheel)",
                0.35,
                0.50,
                Some(Color::White),
            );
            chart_ref.text(
                "[M/N]     Increase/Decrease LOD Detail",
                0.35,
                0.55,
                Some(Color::White),
            );
            chart_ref.text(
                "[O]       Toggle Orbit Lines",
                0.35,
                0.60,
                Some(Color::White),
            );
            chart_ref.text(
                "[L-Click] Select Planet / Hold to Look",
                0.35,
                0.65,
                Some(Color::BrightGreen),
            );
            chart_ref.text(
                "[R-Click] Follow Planet",
                0.35,
                0.70,
                Some(Color::BrightGreen),
            );
            chart_ref.text(
                "[H]       Close this Help",
                0.35,
                0.75,
                Some(Color::BrightYellow),
            );
        } else {
            // Normal HUD
            let stress = format!(
                "Monitor: {} FPS | Latency: {}ms | 3D pixels processed: {}",
                current_fps, ms, drawn_vertices
            );
            chart_ref.text(&stress, 0.02, 0.02, Some(Color::Green));

            let control_txt = format!(
                "Time: {:.2} | LOD: {} | Zoom: {:.1} | Press [H] for Help",
                time_scale, detail, zoom
            );
            chart_ref.text(&control_txt, 0.02, 0.08, Some(Color::Cyan));

            // Tracking status shown clearly on screen (TRACK: XXXXX)
            let track_status = match follow_body {
                Some(id) => format!("TRACK: {}", bodies[id].name),
                None => "TRACK: NONE (Free)".to_string(),
            };

            if let Some(id) = selected_body {
                let b = &bodies[id];
                let dist = if id > 0 { abs_pos[id].norm() } else { 0.0 };
                let sel_txt = format!(
                    "> SELECTED: {} < | Distance: {:.1} AU | {}",
                    b.name, dist, track_status
                );
                chart_ref.text(&sel_txt, 0.02, 0.14, Some(Color::BrightYellow));
            } else {
                let default_txt = format!("L-Click: Select | R-Click: Follow | {}", track_status);
                chart_ref.text(&default_txt, 0.02, 0.14, Some(Color::BrightBlack));
            }
        }

        // --- DRAW TO TERMINAL ---
        execute!(stdout, cursor::MoveTo(0, 0))?;
        print!(
            "{}",
            chart_ref
                .canvas
                .render_with_options(true, Some("Professional Solar System Simulator"))
                .replace('\n', "\r\n")
        );
        stdout.flush()?;

        // Frame Pacing
        if ms < 16 {
            thread::sleep(Duration::from_millis(16 - ms as u64));
        }
    }
}
