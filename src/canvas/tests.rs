use super::{BrailleCanvas, HalfBlockCanvas, QuadrantCanvas};
use colored::Color;

#[test]
fn overlay_replaces_existing_braille_cells() {
    let mut background = BrailleCanvas::new(1, 1);
    background.line_screen(0, 0, 0, 3, None);

    let mut foreground = BrailleCanvas::new(1, 1);
    foreground.line_screen(0, 0, 1, 0, None);

    let mut direct_merge = BrailleCanvas::new(1, 1);
    direct_merge.line_screen(0, 0, 0, 3, None);
    direct_merge.line_screen(0, 0, 1, 0, None);

    let foreground_render = foreground.render_no_color();
    background.overlay(&foreground);

    assert_eq!(background.render_no_color(), foreground_render);
    assert_ne!(background.render_no_color(), direct_merge.render_no_color());
}

#[test]
fn overlay_without_background_prefers_foreground_over_background() {
    let mut background = BrailleCanvas::new(1, 1);
    background.line(0, 0, 1, 0, Some(Color::White));
    let mask = background.cell_masks().to_vec();

    let mut foreground = BrailleCanvas::new(1, 1);
    foreground.line(0, 0, 1, 3, Some(Color::Cyan));

    background.overlay_without_background(&foreground, &mask);

    assert_eq!(background.colors[0], Some(Color::Cyan));
    assert_eq!(background.render_no_color(), foreground.render_no_color());
}

#[test]
fn overlay_without_background_keeps_first_foreground_color() {
    let mut canvas = BrailleCanvas::new(1, 1);
    canvas.line_screen(0, 0, 0, 3, Some(Color::Green));

    let mut overlay = BrailleCanvas::new(1, 1);
    overlay.line_screen(1, 0, 1, 3, Some(Color::Magenta));

    canvas.overlay_without_background(&overlay, &[0]);

    assert_eq!(canvas.colors[0], Some(Color::Green));
    assert_eq!(canvas.render_no_color(), "⣿\n");
}

#[test]
fn quadrant_canvas_uses_renderer_dimensions() {
    let canvas = QuadrantCanvas::new(3, 2);
    assert_eq!(canvas.pixel_width(), 6);
    assert_eq!(canvas.pixel_height(), 4);
}

#[test]
fn quadrant_canvas_renders_quadrant_blocks() {
    let mut canvas = QuadrantCanvas::new(1, 1);
    canvas.set_pixel_screen(0, 0, None);
    canvas.set_pixel_screen(1, 1, None);
    assert_eq!(canvas.render_no_color(), "▚\n");

    canvas.set_pixel_screen(1, 0, None);
    canvas.set_pixel_screen(0, 1, None);
    assert_eq!(canvas.render_no_color(), "█\n");
}

#[test]
fn half_block_canvas_uses_renderer_dimensions() {
    let canvas = HalfBlockCanvas::new(3, 2);
    assert_eq!(canvas.pixel_width(), 3);
    assert_eq!(canvas.pixel_height(), 4);
}

#[test]
fn half_block_canvas_renders_full_blocks_without_color() {
    let mut canvas = HalfBlockCanvas::new(1, 1);
    canvas.set_pixel_screen(0, 0, Some(Color::Red));
    canvas.set_pixel_screen(0, 1, Some(Color::Blue));
    assert_eq!(canvas.render_no_color(), "█\n");
}

#[test]
fn half_block_canvas_splits_foreground_and_background_colors() {
    let mut canvas = HalfBlockCanvas::new(1, 1);
    canvas.set_pixel_screen(0, 0, Some(Color::Red));
    canvas.set_pixel_screen(0, 1, Some(Color::Blue));

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[31m"));
    assert!(rendered.contains("\x1b[44m"));
    assert!(rendered.contains("▀"));
}

#[test]
fn half_block_canvas_uses_cell_background_for_empty_half() {
    let mut canvas = HalfBlockCanvas::new(1, 1);
    canvas.set_pixel_screen(0, 0, Some(Color::BrightWhite));
    canvas.set_cell_background_screen(0, 0, Some(Color::BrightBlack));

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[97m"));
    assert!(rendered.contains("\x1b[100m"));
    assert!(rendered.contains("▀"));
}

#[test]
fn render_with_background_color_emits_ansi_background() {
    let mut canvas = BrailleCanvas::new(1, 1);
    canvas.set_char(0, 0, 'A', None);
    canvas.set_cell_background(0, 0, Some(Color::Blue));

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[44m"));
    assert!(rendered.contains("A"));
}

#[test]
fn background_only_cells_render_as_spaces_with_background() {
    let mut canvas = BrailleCanvas::new(1, 1);
    canvas.set_cell_background(0, 0, Some(Color::BrightBlack));

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[100m "));
}
