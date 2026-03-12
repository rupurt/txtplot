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
use std::collections::VecDeque;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};
use txtplot::ChartContext;

// ============================================================================
// 3D MATH
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
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
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

fn rotate_x(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}
fn rotate_y(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}
// ============================================================================
// GRAPHICS ENGINE (Z-BUFFER)
// ============================================================================
struct ZBuffer {
    w: usize,
    h: usize,
    z: Vec<f64>,
    id: Vec<Option<usize>>,
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
    fn test_and_set(&mut self, x: usize, y: usize, depth: f64, body_id: Option<usize>) -> bool {
        let i = self.idx(x, y);
        if depth < self.z[i] {
            self.z[i] = depth;
            self.id[i] = body_id;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// GRAVITY: NEWTONIAN N-BODY PHYSICS
// ============================================================================
const G: f64 = 1.0;

struct PhysicsBody {
    name: String,
    color: Color,
    radius: f64,
    is_star: bool,
    mass: f64,
    pos: Vec3,
    vel: Vec3,
    force: Vec3,
    rot_angle: f64,
    rot_rate: f64,
    trail: VecDeque<Vec3>,
}

impl PhysicsBody {
    fn get_vertex_data(&self, local_v: Vec3) -> (Vec3, Vec3) {
        let mut normal = local_v;
        let mut v = Vec3::new(
            local_v.x * self.radius,
            local_v.y * self.radius,
            local_v.z * self.radius,
        );

        v = rotate_y(v, self.rot_angle);
        normal = rotate_y(normal, self.rot_angle);

        (v.add(self.pos), normal.normalize())
    }
}

fn get_circular_orbit(
    center_mass: f64,
    center_pos: Vec3,
    center_vel: Vec3,
    distance: f64,
) -> (Vec3, Vec3) {
    let pos = center_pos.add(Vec3::new(distance, 0.0, 0.0));
    let v_mag = (G * center_mass / distance).sqrt();
    let vel = center_vel.add(Vec3::new(0.0, 0.0, v_mag));
    (pos, vel)
}

fn create_body(
    name: &str,
    c: Color,
    r: f64,
    star: bool,
    m: f64,
    state: (Vec3, Vec3),
    rot: f64,
) -> PhysicsBody {
    PhysicsBody {
        name: name.to_string(),
        color: c,
        radius: r,
        is_star: star,
        mass: m,
        pos: state.0,
        vel: state.1,
        force: Vec3::new(0.0, 0.0, 0.0),
        rot_angle: 0.0,
        rot_rate: rot,
        trail: VecDeque::with_capacity(150),
    }
}

// ============================================================================
// DRAWING FUNCTIONS
// ============================================================================
fn make_sphere_points(lat_steps: usize, lon_steps: usize) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(lat_steps * lon_steps);
    for i in 0..lat_steps {
        let v = i as f64 / (lat_steps - 1).max(1) as f64;
        let theta = v * std::f64::consts::PI;
        let (st, ct) = theta.sin_cos();
        for j in 0..lon_steps {
            let u = j as f64 / lon_steps as f64;
            let phi = u * std::f64::consts::TAU;
            pts.push(Vec3::new(st * phi.cos(), ct, st * phi.sin()));
        }
    }
    pts
}

fn project_to_screen(v_cam: Vec3, w: f64, h: f64, scale: f64) -> Option<(isize, isize, f64)> {
    if v_cam.z <= 1.5 {
        return None;
    } // Camera culling
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

fn line_z(
    chart: &mut ChartContext,
    zb: &mut ZBuffer,
    p1: (isize, isize, f64),
    p2: (isize, isize, f64),
    col: Color,
) {
    let min_x = p1.0.min(p2.0);
    let max_x = p1.0.max(p2.0);
    let min_y = p1.1.min(p2.1);
    let max_y = p1.1.max(p2.1);

    if max_x < 0 || min_x >= zb.w as isize || max_y < 0 || min_y >= zb.h as isize {
        return;
    }

    let dx = (p2.0 - p1.0).abs();
    let dy = (p2.1 - p1.1).abs();
    let steps = dx.max(dy).max(1) as i32;
    if steps > 1500 {
        return;
    }

    for s in 0..=steps {
        let t = s as f64 / steps as f64;
        let xf = p1.0 as f64 + (p2.0 as f64 - p1.0 as f64) * t;
        let yf = p1.1 as f64 + (p2.1 as f64 - p1.1 as f64) * t;
        let zf = p1.2 + (p2.2 - p1.2) * t;
        plot_z(
            chart,
            zb,
            xf.round() as isize,
            yf.round() as isize,
            zf,
            col,
            None,
        );
    }
}

// ============================================================================
// MAIN LOOP
// ============================================================================
fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        cursor::Hide,
        terminal::Clear(ClearType::All),
        EnableMouseCapture
    )?;

    // --- GRAVITATIONAL SYSTEM SETUP ---
    let mut bodies = Vec::new();

    let sun_mass = 10000.0;
    let sun_pos = Vec3::new(0.0, 0.0, 0.0);
    let sun_vel = Vec3::new(0.0, 0.0, 0.0);
    bodies.push(create_body(
        "Sun",
        Color::BrightYellow,
        2.5,
        true,
        sun_mass,
        (sun_pos, sun_vel),
        0.05,
    ));

    let earth_mass = 50.0;
    let (e_pos, e_vel) = get_circular_orbit(sun_mass, sun_pos, sun_vel, 30.0);
    bodies.push(create_body(
        "Earth",
        Color::Blue,
        0.8,
        false,
        earth_mass,
        (e_pos, e_vel),
        0.8,
    ));

    let (m_pos, m_vel) = get_circular_orbit(earth_mass, e_pos, e_vel, 2.5);
    bodies.push(create_body(
        "Moon",
        Color::White,
        0.3,
        false,
        0.5,
        (m_pos, m_vel),
        0.2,
    ));

    let jup_mass = 300.0;
    let (j_pos, j_vel) = get_circular_orbit(sun_mass, sun_pos, sun_vel, 60.0);
    let j_vel_elliptical = Vec3::new(j_vel.x, j_vel.y, j_vel.z * 0.9);
    bodies.push(create_body(
        "Jupiter",
        Color::BrightMagenta,
        1.8,
        false,
        jup_mass,
        (j_pos, j_vel_elliptical),
        1.5,
    ));

    // Store the original body count (for the Clear Asteroids `X` command)
    let base_body_count = bodies.len();

    // Simulation state
    let mut cam_pos = Vec3::new(0.0, -30.0, -100.0);
    let mut cam_pitch = 0.3_f64;
    let mut cam_yaw = 0.0_f64;
    let mut zoom = 1.0_f64;

    let mut time_scale = 0.1_f64;
    let mut saved_time_scale = 0.1_f64;
    let mut detail: i32 = 4;
    let mut regen_mesh = true;
    let mut sphere_pts: Vec<Vec3> = Vec::new();

    let mut chart: Option<ChartContext> = None;
    let mut zbuf: Option<ZBuffer> = None;
    let mut last_term: (u16, u16) = (0, 0);

    let mut selected_body: Option<usize> = None;
    let mut follow_body: Option<usize> = None;
    let mut asteroide_id_counter = 1;
    let mut show_help = false;
    let mut show_orbits = true;

    // Monitor
    let mut frames = 0;
    let mut last_fps_time = Instant::now();
    let mut current_fps = 0;
    let mut _drawn_vertices = 0;
    let mut is_dragging = false;
    let mut last_mouse_pos: Option<(u16, u16)> = None;

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
            let new_chart = ChartContext::new(
                (cols as usize).saturating_sub(4),
                (rows as usize).saturating_sub(6),
            );
            zbuf = Some(ZBuffer::new(
                new_chart.canvas.pixel_width(),
                new_chart.canvas.pixel_height(),
            ));
            chart = Some(new_chart);
            regen_mesh = true;
        }
        if regen_mesh {
            sphere_pts = make_sphere_points(
                (10 + detail as usize * 4).min(140),
                (20 + detail as usize * 8).min(260),
            );
            regen_mesh = false;
        }

        let chart_ref = chart.as_mut().unwrap();
        let zb = zbuf.as_mut().unwrap();

        // --- INPUT ---
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        execute!(
                            stdout,
                            cursor::Show,
                            terminal::Clear(ClearType::All),
                            DisableMouseCapture
                        )?;
                        terminal::disable_raw_mode()?;
                        return Ok(());
                    }
                    KeyCode::Char('w') => {
                        cam_pos.z += 2.0 * cam_yaw.cos();
                        cam_pos.x += 2.0 * cam_yaw.sin();
                        follow_body = None;
                    }
                    KeyCode::Char('s') => {
                        cam_pos.z -= 2.0 * cam_yaw.cos();
                        cam_pos.x -= 2.0 * cam_yaw.sin();
                        follow_body = None;
                    }
                    KeyCode::Char('a') => {
                        cam_pos.x -= 2.0 * cam_yaw.cos();
                        cam_pos.z += 2.0 * cam_yaw.sin();
                        follow_body = None;
                    }
                    KeyCode::Char('d') => {
                        cam_pos.x += 2.0 * cam_yaw.cos();
                        cam_pos.z -= 2.0 * cam_yaw.sin();
                        follow_body = None;
                    }
                    KeyCode::Char('e') => cam_pos.y += 1.5,
                    KeyCode::Char('c') => cam_pos.y -= 1.5,

                    KeyCode::Left => cam_yaw += 0.1,
                    KeyCode::Right => cam_yaw -= 0.1,
                    KeyCode::Up => cam_pitch += 0.1,
                    KeyCode::Down => cam_pitch -= 0.1,

                    KeyCode::Char('u') => time_scale -= 0.05,
                    KeyCode::Char('i') => time_scale += 0.05,
                    KeyCode::Char('+') | KeyCode::Char('=') => zoom = (zoom * 1.1).min(20.0),
                    KeyCode::Char('-') => zoom = (zoom / 1.1).max(0.1),
                    KeyCode::Char('m') => {
                        detail = (detail + 1).min(10);
                        regen_mesh = true;
                    }
                    KeyCode::Char('n') => {
                        detail = (detail - 1).max(1);
                        regen_mesh = true;
                    }

                    KeyCode::Char('o') => show_orbits = !show_orbits,
                    KeyCode::Char('f') => {
                        follow_body = selected_body;
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') => show_help = !show_help,

                    // PANIC BUTTON: clear asteroids
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        bodies.truncate(base_body_count);
                        asteroide_id_counter = 1;
                        if let Some(id) = selected_body {
                            if id >= base_body_count {
                                selected_body = None;
                            }
                        }
                        if let Some(id) = follow_body {
                            if id >= base_body_count {
                                follow_body = None;
                            }
                        }
                    }

                    KeyCode::Char(' ') => {
                        if time_scale == 0.0 {
                            time_scale = saved_time_scale;
                        } else {
                            saved_time_scale = time_scale;
                            time_scale = 0.0;
                        }
                    }
                    _ => {}
                }
            } else if let Event::Mouse(me) = event::read()? {
                let get_clicked_id = |c: u16, r: u16, zb: &ZBuffer| -> Option<usize> {
                    let mx = (c.saturating_sub(2) as isize) * 2;
                    let my = (r.saturating_sub(2) as isize) * 4;
                    let mut cl_id = None;
                    let mut min_dist = 40.0;
                    let mut close_z = f64::INFINITY;
                    for py in (my - 40).max(0) as usize..=(my + 40).min(zb.h as isize - 1) as usize
                    {
                        for px in
                            (mx - 40).max(0) as usize..=(mx + 40).min(zb.w as isize - 1) as usize
                        {
                            if let Some(id) = zb.id[zb.idx(px, py)] {
                                let dist = ((px as f64 - mx as f64).powi(2)
                                    + (py as f64 - my as f64).powi(2))
                                .sqrt();
                                if dist < min_dist {
                                    min_dist = dist;
                                    close_z = zb.z[zb.idx(px, py)];
                                    cl_id = Some(id);
                                } else if (dist - min_dist).abs() < 1.0
                                    && zb.z[zb.idx(px, py)] < close_z
                                {
                                    close_z = zb.z[zb.idx(px, py)];
                                    cl_id = Some(id);
                                }
                            }
                        }
                    }
                    cl_id
                };

                match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        is_dragging = true;
                        last_mouse_pos = Some((me.column, me.row));
                        selected_body = get_clicked_id(me.column, me.row, zb);
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        follow_body = get_clicked_id(me.column, me.row, zb);
                        if follow_body.is_some() {
                            selected_body = follow_body;
                        }
                    }
                    MouseEventKind::Down(MouseButton::Middle) => {
                        // ASTEROID LIMITER (max 150 bodies to preserve O(N^2) performance)
                        if bodies.len() < 150 {
                            let aim_dir =
                                rotate_y(rotate_x(Vec3::new(0.0, 0.0, 1.0), cam_pitch), cam_yaw);
                            let target_offset = if let Some(id) = follow_body {
                                bodies[id].pos
                            } else {
                                Vec3::new(0.0, 0.0, 0.0)
                            };

                            let spawn_pos = cam_pos.add(target_offset).add(aim_dir.mul(5.0));
                            let throw_vel = aim_dir.mul(25.0);

                            let ast_name = format!("Asteroid-{}", asteroide_id_counter);
                            asteroide_id_counter += 1;
                            bodies.push(create_body(
                                &ast_name,
                                Color::BrightRed,
                                0.4,
                                false,
                                50.0,
                                (spawn_pos, throw_vel),
                                2.0,
                            ));
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
        }

        chart_ref.canvas.clear();
        zb.clear();
        _drawn_vertices = 0;

        // --- N-BODY PHYSICS WITH VELOCITY VERLET (LEAPFROG) ---
        if time_scale > 0.0 {
            let substeps = 10;
            let dt = time_scale / (substeps as f64);

            for _ in 0..substeps {
                let n = bodies.len();

                // 1. First half-step (Verlet): update positions and precompute half-step velocity
                for b in bodies.iter_mut() {
                    if b.mass > 0.0 {
                        let acc = b.force.mul(1.0 / b.mass);
                        b.pos = b.pos.add(b.vel.mul(dt)).add(acc.mul(0.5 * dt * dt));
                        b.vel = b.vel.add(acc.mul(0.5 * dt)); // v(t + 0.5*dt)
                    }
                    b.force = Vec3::new(0.0, 0.0, 0.0); // Reset forces for the new calculation
                }

                // 2. Compute NEW forces at the newly updated positions
                for i in 0..n {
                    for j in i + 1..n {
                        let d = bodies[j].pos.sub(bodies[i].pos);
                        let dist_sq = d.dot(d);
                        let eps2 = 2.0 * 2.0; // ε^2 (ajusta ε a gusto)
                        let r2 = dist_sq + eps2;
                        let inv_r = 1.0 / r2.sqrt();
                        let inv_r3 = inv_r * inv_r * inv_r;
                        let force_vec = d.mul(G * bodies[i].mass * bodies[j].mass * inv_r3);
                        bodies[i].force = bodies[i].force.add(force_vec);
                        bodies[j].force = bodies[j].force.sub(force_vec);
                    }
                }

                // 3. Complete the velocity step with the new forces
                for b in bodies.iter_mut() {
                    if b.mass > 0.0 {
                        let acc = b.force.mul(1.0 / b.mass);
                        b.vel = b.vel.add(acc.mul(0.5 * dt)); // v(t + dt)
                    }
                    b.rot_angle += b.rot_rate * dt;
                }
            }

            for b in bodies.iter_mut() {
                if let Some(last) = b.trail.back() {
                    if b.pos.sub(*last).norm() > 0.8 {
                        b.trail.push_back(b.pos);
                        if b.trail.len() > 100 {
                            b.trail.pop_front();
                        }
                    }
                } else {
                    b.trail.push_back(b.pos);
                }
            }
        }

        // --- RENDERING ---
        let cw = chart_ref.canvas.pixel_width() as f64;
        let ch = chart_ref.canvas.pixel_height() as f64;
        let scale = (cw.min(ch) / 2.0) * zoom;
        let sun_pos = bodies[0].pos;

        let camera_target_offset = if let Some(id) = follow_body {
            bodies[id].pos
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };

        let to_screen = |v_world: Vec3| -> Option<(isize, isize, f64)> {
            let mut v_cam = v_world.sub(camera_target_offset).sub(cam_pos);
            v_cam = rotate_y(v_cam, -cam_yaw);
            v_cam = rotate_x(v_cam, -cam_pitch);
            project_to_screen(v_cam, cw, ch, scale)
        };

        for (i, body) in bodies.iter().enumerate() {
            if show_orbits {
                let mut prev_proj = None;
                for p in body.trail.iter() {
                    if let Some(proj) = to_screen(*p) {
                        if let Some(prev) = prev_proj {
                            let line_color = if body.is_star {
                                Color::Yellow
                            } else {
                                Color::BrightBlack
                            };
                            line_z(chart_ref, zb, prev, proj, line_color);
                        }
                        prev_proj = Some(proj);
                    } else {
                        prev_proj = None;
                    }
                }
            }

            for p0 in sphere_pts.iter() {
                let (v_world, normal) = body.get_vertex_data(*p0);
                if let Some((sx, sy, z)) = to_screen(v_world) {
                    _drawn_vertices += 1;
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
                "[O]       Toggle Orbit Trails",
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
                "[M-Click] FIRE ASTEROID (Max 150)",
                0.35,
                0.75,
                Some(Color::BrightRed),
            );
            chart_ref.text(
                "[X]       CLEAR ALL ASTEROIDS",
                0.35,
                0.80,
                Some(Color::BrightRed),
            );
            chart_ref.text(
                "[H]       Close this Help",
                0.35,
                0.85,
                Some(Color::BrightYellow),
            );
        } else {
            let stress = format!(
                "Monitor: {} FPS | Latency: {}ms | Bodies: {}/150",
                current_fps,
                ms,
                bodies.len()
            );
            chart_ref.text(&stress, 0.02, 0.02, Some(Color::Green));

            let control_txt = format!(
                "Time: {:.2} | LOD: {} | Zoom: {:.1} | Press [H] for Help",
                time_scale, detail, zoom
            );
            chart_ref.text(&control_txt, 0.02, 0.08, Some(Color::Cyan));

            let track_status = match follow_body {
                Some(id) => format!("TRACK: {}", bodies[id].name),
                None => "TRACK: NONE (Free)".to_string(),
            };

            if let Some(id) = selected_body {
                let b = &bodies[id];
                let dist = if id > 0 { b.pos.norm() } else { 0.0 };
                let sel_txt = format!(
                    "> SELECTED: {} < | Mass: {:.0} | Distance to Sun: {:.1} AU | {}",
                    b.name, b.mass, dist, track_status
                );
                chart_ref.text(&sel_txt, 0.02, 0.14, Some(Color::BrightYellow));
            } else {
                let default_txt = format!(
                    "L-Click: Select | R-Click: Follow | Middle: FIRE | [X]: Clear | {}",
                    track_status
                );
                chart_ref.text(&default_txt, 0.02, 0.14, Some(Color::BrightBlack));
            }
        }

        execute!(stdout, cursor::MoveTo(0, 0))?;
        print!(
            "{}",
            chart_ref
                .canvas
                .render_with_options(true, Some("Real Physics Simulator (N-Body - Leapfrog)"))
                .replace('\n', "\r\n")
        );
        stdout.flush()?;

        if ms < 16 {
            thread::sleep(Duration::from_millis(16 - ms as u64));
        }
    }
}
