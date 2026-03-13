use super::{
    BrailleCanvas, CellCanvas, CellRect, CellRenderer, HalfBlockCanvas, PanelStyle, QuadrantCanvas,
    TextIntensity, TextStyle,
};
use colored::Color;

fn visible_canvas_render<R: CellRenderer>(canvas: &CellCanvas<R>) -> String {
    canvas.render_no_color().replace('\u{2800}', " ")
}

fn build_renderer_raster_scene<R: CellRenderer>() -> CellCanvas<R> {
    let mut canvas = CellCanvas::<R>::new(4, 2);
    let max_x = canvas.pixel_width().saturating_sub(1) as isize;
    let max_y = canvas.pixel_height().saturating_sub(1) as isize;

    canvas.line_screen(0, 0, max_x, max_y, None);
    canvas.line_screen(0, max_y, max_x, 0, None);
    canvas.rect_filled(
        (canvas.pixel_width() / 3) as isize,
        (canvas.pixel_height() / 4) as isize,
        (canvas.pixel_width() / 3).max(1),
        (canvas.pixel_height() / 2).max(1),
        None,
    );

    canvas
}

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

#[test]
fn text_screen_starts_from_top_left() {
    let mut canvas = BrailleCanvas::new(4, 2);
    canvas.text_screen(1, 0, "HI", None);

    let rendered = canvas
        .render_with_options(false, None)
        .replace('\u{2800}', " ");
    let rows: Vec<_> = rendered.lines().collect();

    assert_eq!(rows[0], " HI ");
    assert_eq!(rows[1], "    ");
}

#[test]
fn label_screen_applies_per_cell_background() {
    let mut canvas = BrailleCanvas::new(3, 1);
    canvas.label_screen(0, 0, "OK", Some(Color::White), Some(Color::Blue));

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[44m"));
    assert!(rendered.contains("OK"));
}

#[test]
fn text_screen_styled_emits_bold_and_resets_to_normal() {
    let mut canvas = BrailleCanvas::new(2, 1);
    canvas.text_screen_styled(
        0,
        0,
        "A",
        TextStyle::new().with_foreground(Color::BrightWhite).bold(),
    );
    canvas.text_screen_styled(
        1,
        0,
        "B",
        TextStyle::new().with_foreground(Color::BrightWhite),
    );

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[1mA"));
    assert!(rendered.contains("\x1b[22mB"));
}

#[test]
fn label_screen_styled_emits_dim_text() {
    let mut canvas = BrailleCanvas::new(2, 1);
    canvas.label_screen_styled(
        0,
        0,
        "OK",
        TextStyle {
            foreground: Some(Color::White),
            background: Some(Color::Blue),
            intensity: TextIntensity::Dim,
        },
    );

    let rendered = canvas.render_with_options(false, None);

    assert!(rendered.contains("\x1b[44m"));
    assert!(rendered.contains("\x1b[2m"));
    assert!(rendered.contains("OK"));
}

#[test]
fn panel_screen_draws_box_and_title() {
    let mut canvas = BrailleCanvas::new(10, 5);
    canvas.panel_screen(
        CellRect::new(1, 1, 8, 3),
        Some("CPU"),
        PanelStyle {
            border_color: Some(Color::White),
            background_color: Some(Color::BrightBlack),
            title_color: Some(Color::Yellow),
            title_background: Some(Color::Blue),
        },
    );

    let rendered = canvas
        .render_with_options(false, None)
        .replace('\u{2800}', " ");

    assert!(rendered.contains('┌'));
    assert!(rendered.contains('┐'));
    assert!(rendered.contains('└'));
    assert!(rendered.contains('┘'));
    assert!(rendered.contains("CPU"));
}

#[test]
fn renderer_raster_scene_outputs_remain_stable() {
    assert_eq!(
        visible_canvas_render(&build_renderer_raster_scene::<super::BrailleRenderer>()),
        "⠑⣤⡠⠊\n⡠⠛⠑⢄\n"
    );
    assert_eq!(
        visible_canvas_render(&build_renderer_raster_scene::<super::HalfBlockRenderer>()),
        "▀▄▄▀\n▄▀▀▄\n"
    );
    assert_eq!(
        visible_canvas_render(&build_renderer_raster_scene::<super::QuadrantRenderer>()),
        "▀▄▄▀\n▄▀▀▄\n"
    );
}
