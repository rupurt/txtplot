#![allow(dead_code)]

use colored::Color;
use txtplot::ChartContext;

const FRAME_OFFSET_COLS: u16 = 2;
const FRAME_OFFSET_ROWS: u16 = 2;
const PICK_RADIUS_PX: isize = 40;

pub struct PickingZBuffer {
    width: usize,
    height: usize,
    depth: Vec<f64>,
    body_ids: Vec<Option<usize>>,
}

impl PickingZBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            depth: vec![f64::INFINITY; width * height],
            body_ids: vec![None; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.depth.fill(f64::INFINITY);
        self.body_ids.fill(None);
    }

    pub fn pick_body(&self, col: u16, row: u16) -> Option<usize> {
        let mouse_px = (col.saturating_sub(FRAME_OFFSET_COLS) as isize) * 2;
        let mouse_py = (row.saturating_sub(FRAME_OFFSET_ROWS) as isize) * 4;

        let mut clicked_id = None;
        let mut min_dist = PICK_RADIUS_PX as f64;
        let mut closest_z = f64::INFINITY;

        let min_x = (mouse_px - PICK_RADIUS_PX).max(0) as usize;
        let max_x = (mouse_px + PICK_RADIUS_PX).min(self.width as isize - 1) as usize;
        let min_y = (mouse_py - PICK_RADIUS_PX).max(0) as usize;
        let max_y = (mouse_py + PICK_RADIUS_PX).min(self.height as isize - 1) as usize;

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let idx = self.idx(px, py);
                if let Some(id) = self.body_ids[idx] {
                    let dx = px as f64 - mouse_px as f64;
                    let dy = py as f64 - mouse_py as f64;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < min_dist {
                        min_dist = dist;
                        closest_z = self.depth[idx];
                        clicked_id = Some(id);
                    } else if (dist - min_dist).abs() < 1.0 && self.depth[idx] < closest_z {
                        closest_z = self.depth[idx];
                        clicked_id = Some(id);
                    }
                }
            }
        }

        clicked_id
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn test_and_set(&mut self, x: usize, y: usize, depth: f64, body_id: Option<usize>) -> bool {
        let idx = self.idx(x, y);
        if depth < self.depth[idx] {
            self.depth[idx] = depth;
            self.body_ids[idx] = body_id;
            true
        } else {
            false
        }
    }
}

pub fn plot_z(
    chart: &mut ChartContext,
    zbuf: &mut PickingZBuffer,
    x: isize,
    y: isize,
    depth: f64,
    color: Color,
    body_id: Option<usize>,
) {
    if x < 0 || y < 0 {
        return;
    }

    let ux = x as usize;
    let uy = y as usize;
    if ux >= zbuf.width || uy >= zbuf.height {
        return;
    }

    if zbuf.test_and_set(ux, uy, depth, body_id) {
        chart.canvas.set_pixel_screen(ux, uy, Some(color));
    }
}
