mod support;

use colored::Color;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind},
    terminal,
};
use std::io;
use std::thread; // <-- FIX: thread import was missing
use std::time::{Duration, Instant};
use support::kepler::{body, solve_kepler};
use support::solar::{plot_z, PickingZBuffer as ZBuffer};
use support::terminal::TerminalSession;
use txtplot::three_d::{
    make_sphere_points, project_with_projection, rotate_x, rotate_y, Projection, Vec3,
};
use txtplot::ChartContext;

fn main() -> io::Result<()> {
    let mut terminal_session = TerminalSession::new(true)?;

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
    // BUCLE PRINCIPAL (Game Loop)
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
        // eliminamos el clear del zbuffer para poder hacer track. se limpian DESPUES
        //chart_ref.canvas.clear();
        //zb.clear();
        //drawn_vertices = 0;

        // --- ENTRADA (INPUT) ---
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // Eventos de Teclado
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                            return Ok(());
                        }

                        // Movimiento anula el seguimiento del planeta
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

                        // Zoom y Detalles
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

                        // Opciones Visuales y Funciones
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
                    match me.kind {
                        // LEFT CLICK: drag camera or select
                        MouseEventKind::Down(MouseButton::Left) => {
                            is_dragging = true;
                            last_mouse_pos = Some((me.column, me.row));
                            // Select the touched body (even near misses are caught by the magnet)
                            selected_body = zb.pick_body(me.column, me.row);
                        }
                        // RIGHT CLICK: follow planet
                        MouseEventKind::Down(MouseButton::Right) => {
                            let id = zb.pick_body(me.column, me.row);
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

        //LIMPIAMOS EL CANVAS AHORA YA QUE USAMOS EL FOTOGRAMA ANTERIOR PARA DETECTAR EL TRACK
        chart_ref.canvas.clear();
        zb.clear();
        let mut drawn_vertices = 0;

        // --- PHYSICAL STATE UPDATE ---
        sim_time += time_scale;
        let cw = chart_ref.canvas.pixel_width() as f64;
        let ch = chart_ref.canvas.pixel_height() as f64;
        let scale = (cw.min(ch) / 2.0) * zoom;
        let projection = Projection::new(0.1, 0.5, 0.5, scale * 2.0, scale);

        // Absolute position calculation
        let mut abs_pos = vec![Vec3::new(0.0, 0.0, 0.0); bodies.len()];
        for (i, body) in bodies.iter().enumerate() {
            let mut p = body.get_local_orbit_pos(sim_time);
            if let Some(parent_idx) = body.parent {
                p = p + abs_pos[parent_idx];
            }
            abs_pos[i] = p;
        }

        let camera_target_offset = if let Some(id) = follow_body {
            abs_pos[id]
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };

        let to_screen = |v_world: Vec3| -> Option<(isize, isize, f64)> {
            let mut v_cam = v_world - camera_target_offset - cam_pos;
            v_cam = rotate_y(v_cam, -cam_yaw);
            v_cam = rotate_x(v_cam, -cam_pitch);
            project_with_projection(v_cam, cw, ch, projection)
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
                    p = p + parent_pos;

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

            // Esferas con Sombreado Lambert
            for p0 in sphere_pts.iter() {
                let (v_world, normal) = body.get_vertex_data(*p0, sim_time, orbit_pos);

                if let Some((sx, sy, z)) = to_screen(v_world) {
                    drawn_vertices += 1;
                    let final_color;

                    if body.is_star {
                        final_color = body.color;
                    } else {
                        let light_dir = (sun_pos - v_world).normalize();
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

        // --- HUD Y MONITOR ---
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
        terminal_session.present(
            &chart_ref
                .canvas
                .render_with_options(true, Some("Professional Solar System Simulator"))
                .replace('\n', "\r\n"),
        )?;

        // Frame Pacing
        if ms < 16 {
            thread::sleep(Duration::from_millis(16 - ms as u64));
        }
    }
}
