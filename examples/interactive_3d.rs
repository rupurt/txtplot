use colored::Color;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::{thread, time};
use txtplot::prelude::*;
use txtplot::three_d::{
    line_z_id, make_sphere_points, make_torus_rings, plot_z_id,
};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let mut cam = OrbitCamera::new(Vec3::new(0.0, 0.0, 0.0), 8.0, 0.5, 0.5);
    
    // Geometry
    let sphere = make_sphere_points(30, 60);
    let torus = make_torus_rings(1.5, 0.5, 40, 20);

    let mut last_picked_id = None;

    loop {
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let width = (cols as usize).saturating_sub(2);
        let height = (rows as usize).saturating_sub(4);
        
        let mut chart = ChartContext::new(width, height);
        let mut zbuf = ZBuffer::from_canvas(&chart.canvas);
        let mut idbuf = IdBuffer::from_canvas(&chart.canvas);

        let cw = chart.canvas.pixel_width() as f64;
        let ch = chart.canvas.pixel_height() as f64;
        let scale = cw.min(ch) / 3.0;
        let proj = Projection::new(0.1, 0.5, 0.5, scale * 2.0, scale);

        // Input
        if event::poll(time::Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left => cam.yaw -= 0.1,
                    KeyCode::Right => cam.yaw += 0.1,
                    KeyCode::Up => cam.pitch = (cam.pitch + 0.1).min(1.5),
                    KeyCode::Down => cam.pitch = (cam.pitch - 0.1).max(-1.5),
                    KeyCode::Char('+') | KeyCode::Char('=') => cam.distance = (cam.distance - 0.5).max(1.0),
                    KeyCode::Char('-') => cam.distance += 0.5,
                    _ => {}
                },
                _ => {}
            }
        }

        // Auto-orbit a bit
        cam.yaw += 0.01;

        // Draw Sphere (ID 1)
        let sphere_color = if last_picked_id == Some(1) { Color::Yellow } else { Color::Cyan };
        for p in &sphere {
            if let Some((sx, sy, z)) = cam.project(*p + Vec3::new(-2.0, 0.0, 0.0), cw, ch, proj) {
                plot_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, sx, sy, z, sphere_color, 1);
            }
        }

        // Draw Torus (ID 2)
        let torus_color = if last_picked_id == Some(2) { Color::Yellow } else { Color::Magenta };
        for ring in &torus {
            let mut prev = None;
            for p in ring {
                if let Some(curr) = cam.project(*p + Vec3::new(2.0, 0.0, 0.0), cw, ch, proj) {
                    if let Some(p_prev) = prev {
                        line_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, p_prev, curr, torus_color, 2);
                    }
                    prev = Some(curr);
                } else {
                    prev = None;
                }
            }
        }

        // Middle cursor picking simulation (center of screen)
        let pick_x = chart.canvas.pixel_width() / 2;
        let pick_y = chart.canvas.pixel_height() / 2;
        last_picked_id = idbuf.get(pick_x, pick_y);

        // Draw Crosshair
        chart.canvas.set_pixel_screen(pick_x, pick_y, Some(Color::White));
        chart.canvas.line_screen(pick_x as isize - 2, pick_y as isize, pick_x as isize + 2, pick_y as isize, Some(Color::White));
        chart.canvas.line_screen(pick_x as isize, pick_y as isize - 1, pick_x as isize, pick_y as isize + 1, Some(Color::White));

        // Info
        let picked_name = match last_picked_id {
            Some(1) => "Sphere",
            Some(2) => "Torus",
            _ => "None",
        };
        chart.anchored_text_styled(
            &format!(" PICKED: {} ", picked_name),
            ChartAnchor::TopLeft,
            TextStyle::new().with_foreground(Color::Yellow).bold(),
        );
        chart.anchored_text(
            "Arrows: Orbit | +/-: Zoom | q: Quit",
            ChartAnchor::BottomLeft,
            Some(Color::BrightBlack),
        );

        // Render
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart.canvas.render_with_options(true, Some("INTERACTIVE 3D PICKING"));
        print!("{}", output.replace('\n', "\r\n"));
        stdout.flush()?;

        thread::sleep(time::Duration::from_millis(30));
    }

    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
