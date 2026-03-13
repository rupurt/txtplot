use colored::Color;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::{thread, time};
use txtplot::three_d::{
    line_z, make_sphere_points, make_torus_rings, make_triangle, plot_z, project_to_screen,
    rotate_x, rotate_y, Vec3, ZBuffer,
};
use txtplot::ChartContext;

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    // Geometry
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
    let sphere_pts = make_sphere_points(20, 40);
    let torus_rings = make_torus_rings(1.25, 0.45, 20, 26);
    let donut_rings = make_torus_rings(1.05, 0.65, 24, 32);

    // State
    let mut angle_x = 0.0_f64;
    let mut angle_y = 0.0_f64;

    let mut show_cube = true;
    let mut show_sphere = true;
    let mut show_torus = true;
    let mut show_triangle = true;
    let mut show_donut = true;

    // Camera + zoom
    let mut cam = Vec3::new(0.0, 0.0, 0.0);
    let mut zoom = 1.0_f64;

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
            chart = Some(new_chart);
            zbuf = Some(ZBuffer::from_canvas(&chart.as_ref().unwrap().canvas));
        }

        let chart_ref = chart.as_mut().unwrap();
        let zb = zbuf.as_mut().unwrap();

        chart_ref.canvas.clear();
        zb.clear();

        // Input (non-blocking)
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

                    // Camera pan (X/Y)
                    KeyCode::Left => cam.x -= 0.18,
                    KeyCode::Right => cam.x += 0.18,
                    KeyCode::Up => cam.y -= 0.18,
                    KeyCode::Down => cam.y += 0.18,

                    // Camera Z (move forward/back)
                    KeyCode::PageUp => cam.z += 0.30, // move camera "forward" => objects appear closer
                    KeyCode::PageDown => cam.z -= 0.30, // move back

                    // Zoom
                    KeyCode::Char('+') | KeyCode::Char('=') => zoom = (zoom * 1.08).min(20.0),
                    KeyCode::Char('-') => zoom = (zoom / 1.08).max(0.10),

                    // Reset
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        cam = Vec3::new(0.0, 0.0, 0.0);
                        zoom = 1.0;
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

        // Helper closure: world->camera->screen
        let to_screen = |v_world: Vec3| -> Option<(isize, isize, f64)> {
            // camera space is world - cam
            let v_cam = v_world - cam;
            project_to_screen(v_cam, cw, ch, scale)
        };

        // TRIANGLE (wire) with zbuffered lines
        if show_triangle {
            let mut p: Vec<(isize, isize, f64)> = Vec::with_capacity(3);
            for v in tri.iter() {
                let v = rotate_y(rotate_x(*v, angle_x * 0.9), angle_y * 1.1) + pos_tri;
                if let Some(ps) = to_screen(v) {
                    p.push(ps);
                }
            }
            if p.len() == 3 {
                line_z(&mut chart_ref.canvas, zb, p[0], p[1], Color::Yellow);
                line_z(&mut chart_ref.canvas, zb, p[1], p[2], Color::Yellow);
                line_z(&mut chart_ref.canvas, zb, p[2], p[0], Color::Yellow);
            }
        }

        // CUBE (wire) with zbuffered lines
        if show_cube {
            let mut proj: Vec<Option<(isize, isize, f64)>> = Vec::with_capacity(8);
            for v in cube_vertices.iter() {
                let mut vv = *v;
                vv = rotate_x(vv, angle_x);
                vv = rotate_y(vv, angle_y);
                vv = vv + pos_cube;
                proj.push(to_screen(vv));
            }
            for (a, b) in cube_edges.iter() {
                if let (Some(p1), Some(p2)) = (proj[*a], proj[*b]) {
                    line_z(&mut chart_ref.canvas, zb, p1, p2, Color::Cyan);
                }
            }
        }

        // TORUS (wire rings) with zbuffered lines
        if show_torus {
            for (ri, ring) in torus_rings.iter().enumerate() {
                if ri % 2 != 0 {
                    continue;
                }
                let mut prev: Option<(isize, isize, f64)> = None;
                for v0 in ring.iter() {
                    let mut v = *v0;
                    v = rotate_x(v, angle_x * 0.75);
                    v = rotate_y(v, angle_y * 1.25);
                    v = v + pos_torus;

                    if let Some(p) = to_screen(v) {
                        if let Some(pp) = prev {
                            line_z(&mut chart_ref.canvas, zb, pp, p, Color::Magenta);
                        }
                        prev = Some(p);
                    } else {
                        prev = None;
                    }
                }
            }
        }

        // SPHERE (point cloud) with zbuffer
        if show_sphere {
            for p0 in sphere_pts.iter() {
                let mut v = *p0;
                v = rotate_x(v, angle_x * 1.15);
                v = rotate_y(v, angle_y * 0.85);
                v = Vec3::new(v.x * 1.15, v.y * 1.15, v.z * 1.15);
                v = v + pos_sphere;

                if let Some((sx, sy, z)) = to_screen(v) {
                    let col = if p0.y > 0.35 {
                        Color::BrightBlue
                    } else if p0.y < -0.35 {
                        Color::Blue
                    } else {
                        Color::White
                    };
                    plot_z(&mut chart_ref.canvas, zb, sx, sy, z, col);
                }
            }
        }

        // DONUT (thick torus point cloud) with zbuffer
        if show_donut {
            for (ri, ring) in donut_rings.iter().enumerate() {
                if ri % 2 != 0 {
                    continue;
                }
                for (j, v0) in ring.iter().enumerate() {
                    if j % 2 != 0 {
                        continue;
                    }
                    let mut v = *v0;
                    v = rotate_x(v, angle_x * 1.05);
                    v = rotate_y(v, angle_y * 0.95);
                    v = v + pos_donut;

                    if let Some((sx, sy, z)) = to_screen(v) {
                        let col = if v0.y > 0.2 {
                            Color::BrightRed
                        } else if v0.y < -0.2 {
                            Color::Red
                        } else {
                            Color::BrightMagenta
                        };
                        plot_z(&mut chart_ref.canvas, zb, sx, sy, z, col);
                    }
                }
            }
        }

        // HUD
        let hud1 = format!(
            "3D Figures | [1]Cube:{} [2]Sphere:{} [3]Torus:{} [4]Tri:{} [5]Donut:{} | zoom={:.2} cam=({:.2},{:.2},{:.2})",
            if show_cube { "ON" } else { "off" },
            if show_sphere { "ON" } else { "off" },
            if show_torus { "ON" } else { "off" },
            if show_triangle { "ON" } else { "off" },
            if show_donut { "ON" } else { "off" },
            zoom, cam.x, cam.y, cam.z
        );
        chart_ref.text(&hud1, 0.02, 0.06, Some(Color::White));

        chart_ref.text(
            "Controls: 1-5 toggle | Arrows pan | PgUp/PgDn Z | +/- zoom | r reset | q/Esc quit",
            0.02,
            0.12,
            Some(Color::BrightBlack),
        );

        // Render
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart_ref
            .canvas
            .render_with_options(true, Some("3D Gallery (Z-buffer + Camera + Zoom)"));
        print!("{}", output.replace('\n', "\r\n"));
        stdout.flush()?;

        thread::sleep(time::Duration::from_millis(30));
    }
}
