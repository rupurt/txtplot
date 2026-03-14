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
use txtplot::three_d::{line_z_id, plot_z_id, make_circle_3d, make_box_3d, rotate_y};

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    let mut cam = OrbitCamera::new(Vec3::new(0.0, 0.0, 0.0), 10.0, 0.8, 0.0);
    
    // Geometry definitions
    let platter_l_center = Vec3::new(-3.0, 0.0, 0.0);
    let platter_r_center = Vec3::new(3.0, 0.0, 0.0);
    let platter_radius = 2.0;
    
    let mixer_box = make_box_3d(Vec3::new(0.0, -0.2, 0.0), 2.5, 0.4, 4.0);
    
    let mut angle_l = 0.0_f64;
    let mut angle_r = 0.0_f64;
    let mut crossfader_x = 0.0_f64; // -1.0 to 1.0
    
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
        let scale = cw.min(ch) / 3.5;
        let proj = Projection::new(0.1, 0.5, 0.5, scale * 2.0, scale);

        // Input
        if event::poll(time::Duration::from_millis(5))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left => cam.yaw -= 0.1,
                    KeyCode::Right => cam.yaw += 0.1,
                    KeyCode::Up => cam.pitch = (cam.pitch + 0.1).min(1.5),
                    KeyCode::Down => cam.pitch = (cam.pitch - 0.1).max(0.1),
                    _ => {}
                }
            }
        }

        // Logic: Spinning
        // If we "touch" (pick) a deck, it stops spinning (scratching)
        if last_picked_id != Some(1) { angle_l += 0.1; }
        if last_picked_id != Some(2) { angle_r += 0.15; }
        
        // If we pick the mixer, the fader moves
        if last_picked_id == Some(3) {
            crossfader_x = (time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis() as f64 * 0.005).sin();
        }

        // --- DRAWING ---

        // Mixer (ID 3)
        let mixer_color = if last_picked_id == Some(3) { Color::Yellow } else { Color::BrightBlack };
        let mixer_vertices: Vec<Option<(isize, isize, f64)>> = mixer_box.vertices.iter()
            .map(|&v| cam.project(v, cw, ch, proj)).collect();
        for &(a, b) in &mixer_box.edges {
            if let (Some(p1), Some(p2)) = (mixer_vertices[a], mixer_vertices[b]) {
                line_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, p1, p2, mixer_color, 3);
            }
        }
        
        // Crossfader (part of ID 3)
        let fader_pos = Vec3::new(crossfader_x * 0.8, 0.2, 0.5);
        if let Some(p) = cam.project(fader_pos, cw, ch, proj) {
            plot_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, p.0, p.1, p.2 - 0.1, Color::White, 3);
            chart.canvas.line_screen(p.0 - 2, p.1, p.0 + 2, p.1, Some(Color::White));
        }

        // Left Platter (ID 1)
        let color_l = if last_picked_id == Some(1) { Color::Cyan } else { Color::Blue };
        let points_l = make_circle_3d(platter_l_center, platter_radius, 40);
        let mut prev_l = None;
        for (i, p) in points_l.iter().enumerate() {
            let mut p_rot = *p - platter_l_center;
            p_rot = rotate_y(p_rot, angle_l);
            let p_final = p_rot + platter_l_center;
            
            if let Some(curr) = cam.project(p_final, cw, ch, proj) {
                if let Some(p_prev) = prev_l {
                    line_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, p_prev, curr, color_l, 1);
                }
                prev_l = Some(curr);
                
                // Draw a "needle" or marker on the platter
                if i == 0 {
                    if let Some(center_proj) = cam.project(platter_l_center, cw, ch, proj) {
                        line_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, center_proj, curr, Color::White, 1);
                    }
                }
            } else { prev_l = None; }
        }

        // Right Platter (ID 2)
        let color_r = if last_picked_id == Some(2) { Color::Magenta } else { Color::Red };
        let points_r = make_circle_3d(platter_r_center, platter_radius, 40);
        let mut prev_r = None;
        for (i, p) in points_r.iter().enumerate() {
            let mut p_rot = *p - platter_r_center;
            p_rot = rotate_y(p_rot, angle_r);
            let p_final = p_rot + platter_r_center;
            
            if let Some(curr) = cam.project(p_final, cw, ch, proj) {
                if let Some(p_prev) = prev_r {
                    line_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, p_prev, curr, color_r, 2);
                }
                prev_r = Some(curr);
                
                if i == 0 {
                    if let Some(center_proj) = cam.project(platter_r_center, cw, ch, proj) {
                        line_z_id(&mut chart.canvas, &mut zbuf, &mut idbuf, center_proj, curr, Color::White, 2);
                    }
                }
            } else { prev_r = None; }
        }

        // Interactivity: Center crosshair picking
        let pick_x = chart.canvas.pixel_width() / 2;
        let pick_y = chart.canvas.pixel_height() / 2;
        last_picked_id = idbuf.get(pick_x, pick_y);

        // UI Crosshair
        chart.canvas.set_pixel_screen(pick_x, pick_y, Some(Color::White));
        chart.canvas.line_screen(pick_x as isize - 2, pick_y as isize, pick_x as isize + 2, pick_y as isize, Some(Color::White));
        chart.canvas.line_screen(pick_x as isize, pick_y as isize - 1, pick_x as isize, pick_y as isize + 1, Some(Color::White));

        // Labels
        let status = match last_picked_id {
            Some(1) => " SCRATCHING LEFT DECK ",
            Some(2) => " SCRATCHING RIGHT DECK ",
            Some(3) => " ADJUSTING CROSSFADER ",
            _ => " 3D DJ CONSOLE ",
        };
        chart.anchored_text_styled(status, ChartAnchor::TopLeft, TextStyle::new().with_foreground(Color::Yellow).bold());
        chart.anchored_text("Arrows: Orbit | q: Quit", ChartAnchor::BottomLeft, Some(Color::BrightBlack));

        // Output
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let output = chart.canvas.render_with_options(true, Some("TXTPLOT 3D DJ DECKS"));
        print!("{}", output.replace('\n', "\r\n"));
        stdout.flush()?;

        thread::sleep(time::Duration::from_millis(33));
    }

    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
