use colored::Color;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::{thread, time};
use txtplot::ChartContext;

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
}

fn rotate_x(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}
fn rotate_y(v: Vec3, a: f64) -> Vec3 {
    let (s, c) = a.sin_cos();
    Vec3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}

/// Z-buffer per pixel of the canvas (screen pixel coords).
struct ZBuffer {
    w: usize,
    h: usize,
    z: Vec<f64>,
}
impl ZBuffer {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            z: vec![f64::INFINITY; w * h],
        }
    }
    fn clear(&mut self) {
        self.z.fill(f64::INFINITY);
    }
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    /// Depth test: smaller z means "closer". Returns true if pixel wins.
    fn test_and_set(&mut self, x: usize, y: usize, depth: f64) -> bool {
        let i = self.idx(x, y);
        if depth < self.z[i] {
            self.z[i] = depth;
            true
        } else {
            false
        }
    }
}

fn project_to_screen(
    v_cam: Vec3,
    canvas_w: f64,
    canvas_h: f64,
    scale: f64,
) -> Option<(isize, isize, f64)> {
    // Perspective projection. Only points in front of camera.
    if v_cam.z <= 0.06 {
        return None;
    }
    let px = (v_cam.x / v_cam.z) * 2.0;
    let py = v_cam.y / v_cam.z;

    let cx = canvas_w / 2.0;
    let cy = canvas_h / 2.0;

    let sx = cx + px * scale;
    let sy = cy + py * scale;

    Some((sx.round() as isize, sy.round() as isize, v_cam.z))
}

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
            let (sp, cp) = phi.sin_cos();
            pts.push(Vec3::new(st * cp, ct, st * sp));
        }
    }
    pts
}

fn make_torus_rings(r_major: f64, r_minor: f64, u_steps: usize, v_steps: usize) -> Vec<Vec<Vec3>> {
    // Returns rings: for each u (major circle), a polyline over v (minor circle)
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

fn make_triangle() -> [Vec3; 3] {
    [
        Vec3::new(-1.2, -0.8, 0.0),
        Vec3::new(1.2, -0.8, 0.0),
        Vec3::new(0.0, 1.3, 0.0),
    ]
}

fn plot_z(chart: &mut ChartContext, zb: &mut ZBuffer, x: isize, y: isize, z: f64, col: Color) {
    if x < 0 || y < 0 {
        return;
    }
    let ux = x as usize;
    let uy = y as usize;
    if ux >= zb.w || uy >= zb.h {
        return;
    }
    if zb.test_and_set(ux, uy, z) {
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
    let dx = (p2.0 - p1.0).abs();
    let dy = (p2.1 - p1.1).abs();
    let steps = dx.max(dy).max(1) as i32;

    for i in 0..=steps {
        let t = (i as f64) / (steps as f64);
        let xf = p1.0 as f64 + (p2.0 as f64 - p1.0 as f64) * t;
        let yf = p1.1 as f64 + (p2.1 as f64 - p1.1 as f64) * t;
        let zf = p1.2 + (p2.2 - p1.2) * t;
        plot_z(chart, zb, xf.round() as isize, yf.round() as isize, zf, col);
    }
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    // Static geometry
    let cube_vertices: [Vec3; 8] = [
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
    ];
    let cube_edges: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let tri = make_triangle();

    // Animation
    let mut angle_x = 0.0_f64;
    let mut angle_y = 0.0_f64;

    // Toggles
    let mut show_cube = true;
    let mut show_sphere = true;
    let mut show_torus = true;
    let mut show_triangle = true;
    let mut show_donut = true;

    // Camera + zoom
    let mut cam = Vec3::new(0.0, 0.0, 0.0);
    let mut zoom = 1.0_f64;

    // Detail (interactive mesh resolution)
    let mut detail: i32 = 4;
    let detail_min: i32 = 1;
    let detail_max: i32 = 512;
    let mut regen_mesh = true;

    // Dynamic geometry (regenerated)
    let mut sphere_pts: Vec<Vec3> = Vec::new();
    let mut torus_rings: Vec<Vec<Vec3>> = Vec::new();
    let mut donut_rings: Vec<Vec<Vec3>> = Vec::new();

    // Resizable chart + zbuffer
    let mut chart: Option<ChartContext> = None;
    let mut zbuf: Option<ZBuffer> = None;
    let mut last_term: (u16, u16) = (0, 0);

    loop {
        // Resize
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        if (cols, rows) != last_term || chart.is_none() {
            last_term = (cols, rows);
            let width = (cols as usize).saturating_sub(4);
            let height = (rows as usize).saturating_sub(6);

            let new_chart = ChartContext::new(width, height);
            let w_px = new_chart.canvas.pixel_width();
            let h_px = new_chart.canvas.pixel_height();

            chart = Some(new_chart);
            zbuf = Some(ZBuffer::new(w_px, h_px));

            // Rebuild meshes so they match new perf envelope if needed
            regen_mesh = true;
        }

        // Regenerate geometry if detail changed
        if regen_mesh {
            let d = detail as usize;

            let lat = (10 + d * 4).min(140);
            let lon = (20 + d * 8).min(260);
            sphere_pts = make_sphere_points(lat, lon);

            let u = (10 + d * 3).min(180);
            let v = (12 + d * 4).min(240);
            torus_rings = make_torus_rings(1.25, 0.45, u, v);

            let du = (12 + d * 4).min(220);
            let dv = (16 + d * 6).min(300);
            donut_rings = make_torus_rings(1.05, 0.65, du, dv);

            regen_mesh = false;
        }

        let chart_ref = chart.as_mut().unwrap();
        let zb = zbuf.as_mut().unwrap();

        chart_ref.canvas.clear();
        zb.clear();

        // Input (consume all pending events)
        while event::poll(time::Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        execute!(stdout, cursor::Show)?;
                        terminal::disable_raw_mode()?;
                        return Ok(());
                    }

                    // Toggle shapes
                    KeyCode::Char('1') => show_cube = !show_cube,
                    KeyCode::Char('2') => show_sphere = !show_sphere,
                    KeyCode::Char('3') => show_torus = !show_torus,
                    KeyCode::Char('4') => show_triangle = !show_triangle,
                    KeyCode::Char('5') => show_donut = !show_donut,

                    // Detail
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        if detail < detail_max {
                            detail += 1;
                            regen_mesh = true;
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        if detail > detail_min {
                            detail -= 1;
                            regen_mesh = true;
                        }
                    }

                    // Camera pan (X/Y)
                    KeyCode::Left => cam.x -= 0.18,
                    KeyCode::Right => cam.x += 0.18,
                    KeyCode::Up => cam.y -= 0.18,
                    KeyCode::Down => cam.y += 0.18,

                    // Camera Z (move forward/back)
                    KeyCode::PageUp => cam.z += 0.30,
                    KeyCode::PageDown => cam.z -= 0.30,

                    // Zoom
                    KeyCode::Char('+') | KeyCode::Char('=') => zoom = (zoom * 1.08).min(25.0),
                    KeyCode::Char('-') => zoom = (zoom / 1.08).max(0.10),

                    // Reset
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        cam = Vec3::new(0.0, 0.0, 0.0);
                        zoom = 1.0;
                        detail = 4;
                        regen_mesh = true;
                        show_cube = true;
                        show_sphere = true;
                        show_torus = true;
                        show_triangle = true;
                        show_donut = true;
                    }

                    _ => {}
                }
            }
        }

        // Exit if requested (break from loop above)
        if event::poll(time::Duration::from_millis(0))? {
            // no-op; keeps rust happy about unused? (safe)
        }

        // If quit key pressed: we used `break` from inner match only; so handle here by checking last event isn't easy.
        // Simpler: detect quit via a flag:
        // We'll do that properly:
        // (We keep it minimal by using a dedicated flag below.)
        // ----
        // To avoid refactor mid-file, we handle quit with a bool:
        // (But we already broke only from match; so implement properly now.)
        // ----
        // We'll instead rely on Esc/q break by setting a flag in the input loop.
        // NOTE: This comment block kept to be explicit. Logic below uses `quit`.
        let mut quit = false;
        // Re-process input quickly for quit only (cheap, still responsive)
        while event::poll(time::Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if matches!(code, KeyCode::Char('q') | KeyCode::Esc) {
                    quit = true;
                    break;
                }
            }
        }
        if quit {
            break;
        }

        // Animate
        angle_x += 0.035;
        angle_y += 0.022;

        let cw = chart_ref.canvas.pixel_width() as f64;
        let ch = chart_ref.canvas.pixel_height() as f64;
        let mut scale = cw.min(ch) / 3.2;
        scale *= zoom;

        // Layout: world positions
        let z_push = 6.0;
        let pos_cube = Vec3::new(-2.2, -1.2, z_push);
        let pos_sphere = Vec3::new(2.2, -1.2, z_push);
        let pos_torus = Vec3::new(-2.2, 1.4, z_push);
        let pos_donut = Vec3::new(2.2, 1.4, z_push);
        let pos_tri = Vec3::new(0.0, 0.1, z_push - 0.2);

        // World -> camera -> screen
        let to_screen = |v_world: Vec3| -> Option<(isize, isize, f64)> {
            let v_cam = v_world.sub(cam);
            project_to_screen(v_cam, cw, ch, scale)
        };

        // Triangle (wire)
        if show_triangle {
            let mut p: Vec<(isize, isize, f64)> = Vec::with_capacity(3);
            for v in tri.iter() {
                let v = rotate_y(rotate_x(*v, angle_x * 0.9), angle_y * 1.1).add(pos_tri);
                if let Some(ps) = to_screen(v) {
                    p.push(ps);
                }
            }
            if p.len() == 3 {
                line_z(chart_ref, zb, p[0], p[1], Color::Yellow);
                line_z(chart_ref, zb, p[1], p[2], Color::Yellow);
                line_z(chart_ref, zb, p[2], p[0], Color::Yellow);
            }
        }

        // Cube (wire)
        if show_cube {
            let mut proj: Vec<Option<(isize, isize, f64)>> = Vec::with_capacity(8);
            for v in cube_vertices.iter() {
                let mut vv = *v;
                vv = rotate_x(vv, angle_x);
                vv = rotate_y(vv, angle_y);
                vv = vv.add(pos_cube);
                proj.push(to_screen(vv));
            }
            for (a, b) in cube_edges.iter() {
                if let (Some(p1), Some(p2)) = (proj[*a], proj[*b]) {
                    line_z(chart_ref, zb, p1, p2, Color::Cyan);
                }
            }
        }

        // Torus (wire rings)
        if show_torus {
            let ring_skip = if detail <= 3 {
                3
            } else if detail <= 6 {
                2
            } else {
                1
            };
            for (ri, ring) in torus_rings.iter().enumerate() {
                if ri % ring_skip != 0 {
                    continue;
                }
                let mut prev: Option<(isize, isize, f64)> = None;
                for v0 in ring.iter() {
                    let mut v = *v0;
                    v = rotate_x(v, angle_x * 0.75);
                    v = rotate_y(v, angle_y * 1.25);
                    v = v.add(pos_torus);

                    if let Some(p) = to_screen(v) {
                        if let Some(pp) = prev {
                            line_z(chart_ref, zb, pp, p, Color::Magenta);
                        }
                        prev = Some(p);
                    } else {
                        prev = None;
                    }
                }
            }
        }

        // Sphere (points)
        if show_sphere {
            // With more detail, sphere can be extremely dense. We keep it full: it's a stress option.
            for p0 in sphere_pts.iter() {
                let mut v = *p0;
                v = rotate_x(v, angle_x * 1.15);
                v = rotate_y(v, angle_y * 0.85);
                v = Vec3::new(v.x * 1.15, v.y * 1.15, v.z * 1.15);
                v = v.add(pos_sphere);

                if let Some((sx, sy, z)) = to_screen(v) {
                    let col = if p0.y > 0.35 {
                        Color::BrightBlue
                    } else if p0.y < -0.35 {
                        Color::Blue
                    } else {
                        Color::White
                    };
                    plot_z(chart_ref, zb, sx, sy, z, col);
                }
            }
        }

        // Donut (thick torus point cloud)
        if show_donut {
            let ring_skip = if detail <= 3 {
                3
            } else if detail <= 6 {
                2
            } else {
                1
            };
            let pt_skip = if detail <= 3 {
                3
            } else if detail <= 6 {
                2
            } else {
                1
            };

            for (ri, ring) in donut_rings.iter().enumerate() {
                if ri % ring_skip != 0 {
                    continue;
                }
                for (j, v0) in ring.iter().enumerate() {
                    if j % pt_skip != 0 {
                        continue;
                    }
                    let mut v = *v0;
                    v = rotate_x(v, angle_x * 1.05);
                    v = rotate_y(v, angle_y * 0.95);
                    v = v.add(pos_donut);

                    if let Some((sx, sy, z)) = to_screen(v) {
                        let col = if v0.y > 0.2 {
                            Color::BrightRed
                        } else if v0.y < -0.2 {
                            Color::Red
                        } else {
                            Color::BrightMagenta
                        };
                        plot_z(chart_ref, zb, sx, sy, z, col);
                    }
                }
            }
        }

        // HUD
        let hud1 = format!(
            "3D Figures | detail={} ([/]) | [1]Cube:{} [2]Sphere:{} [3]Torus:{} [4]Tri:{} [5]Donut:{} | zoom={:.2} cam=({:.2},{:.2},{:.2})",
            detail,
            if show_cube { "ON" } else { "off" },
            if show_sphere { "ON" } else { "off" },
            if show_torus { "ON" } else { "off" },
            if show_triangle { "ON" } else { "off" },
            if show_donut { "ON" } else { "off" },
            zoom,
            cam.x,
            cam.y,
            cam.z
        );
        chart_ref.text(&hud1, 0.02, 0.06, Some(Color::White));

        chart_ref.text(
            "Controls: 1-5 toggle | Arrows pan | PgUp/PgDn Z | +/- zoom | m/n detail | r reset | q/Esc quit",
            0.02,
            0.12,
            Some(Color::BrightBlack),
        );

        // Render
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart_ref
            .canvas
            .render_with_options(true, Some("3D Gallery (Z-buffer + Camera + Zoom + Detail)"));
        print!("{}", output.replace('\n', "\r\n"));
        stdout.flush()?;

        // Frame pacing (keep it stable)
        thread::sleep(time::Duration::from_millis(30));
    }

    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
