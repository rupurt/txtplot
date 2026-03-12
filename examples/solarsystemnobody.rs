mod support;

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
use support::solar::{line_z, plot_z, PickingZBuffer as ZBuffer};
use support::three_d::{
    make_sphere_points, project_with_projection, rotate_x, rotate_y, Projection, Vec3,
};
use txtplot::ChartContext;

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
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
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
                },
                Event::Mouse(me) => match me.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        is_dragging = true;
                        last_mouse_pos = Some((me.column, me.row));
                        selected_body = zb.pick_body(me.column, me.row);
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        follow_body = zb.pick_body(me.column, me.row);
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
                },
                _ => {}
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
            project_with_projection(
                v_cam,
                cw,
                ch,
                Projection::new(1.5, 0.5, 0.5, scale * 2.0, scale),
            )
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
