use colored::Color;
use txtplot::{
    CellCanvas, CellChartContext, CellRect, CellRenderer, ChartContext, HalfBlockChartContext,
    PanelStyle, QuadrantChartContext,
};

fn build_signal_chart<R: CellRenderer>() -> CellChartContext<R> {
    let mut chart = CellChartContext::<R>::with_dimensions(56, 14);
    chart.draw_grid(8, 4, Some(Color::BrightBlack));
    chart.draw_axes((0.0, 10.0), (-1.5, 1.5), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    chart.plot_function(
        |x: f64| (x * 0.5).cos() * 0.5,
        0.0,
        10.0,
        Some(Color::Magenta),
    );
    chart.text("sin(x)", 0.73, 0.84, Some(Color::Cyan));
    chart.text("0.5*cos(0.5x)", 0.52, 0.08, Some(Color::Magenta));
    chart
}

fn build_raster_demo<R: CellRenderer>() -> CellCanvas<R> {
    let mut canvas = CellCanvas::<R>::new(28, 8);
    let width_px = canvas.pixel_width();
    let height_px = canvas.pixel_height();
    let max_x = width_px.saturating_sub(1) as isize;
    let max_y = height_px.saturating_sub(1) as isize;

    canvas.rect(0, 0, width_px, height_px, Some(Color::BrightBlack));
    canvas.line_screen(0, 0, max_x, max_y, Some(Color::BrightYellow));
    canvas.line_screen(0, max_y, max_x, 0, Some(Color::BrightGreen));
    canvas.rect_filled(
        (width_px / 4) as isize,
        (height_px / 4) as isize,
        (width_px / 3).max(1),
        (height_px / 3).max(1),
        Some(Color::Blue),
    );
    canvas.rect_filled(
        (width_px / 2) as isize,
        (height_px / 2) as isize,
        (width_px / 5).max(1),
        (height_px / 4).max(1),
        Some(Color::Red),
    );
    canvas.panel_screen(
        CellRect::new(1, 1, 10, 3),
        Some("HUD"),
        PanelStyle {
            border_color: Some(Color::BrightWhite),
            background_color: Some(Color::BrightBlack),
            title_color: Some(Color::BrightWhite),
            title_background: Some(Color::Blue),
        },
    );
    canvas.label_screen(
        2,
        2,
        "RAW",
        Some(Color::BrightWhite),
        Some(Color::BrightBlue),
    );
    canvas.text_screen(6, 2, "mix", Some(Color::Cyan));

    canvas
}

fn print_canvas<R: CellRenderer>(name: &str, canvas: &CellCanvas<R>) {
    let title = format!(
        "{name} Canvas ({}x{} sub-pixels)",
        canvas.pixel_width(),
        canvas.pixel_height()
    );
    println!("{}", canvas.render_with_options(true, Some(&title)));
}

fn print_chart<R: CellRenderer>(name: &str, chart: &CellChartContext<R>) {
    let title = format!(
        "{name} Chart ({}x{} sub-pixels)",
        chart.canvas.pixel_width(),
        chart.canvas.pixel_height()
    );
    println!("{}", chart.canvas.render_with_options(true, Some(&title)));
}

fn main() {
    let braille_chart = build_signal_chart::<txtplot::BrailleRenderer>();
    let half_block_chart = build_signal_chart::<txtplot::HalfBlockRenderer>();
    let quadrant_chart = build_signal_chart::<txtplot::QuadrantRenderer>();
    let braille_canvas = build_raster_demo::<txtplot::BrailleRenderer>();
    let half_block_canvas = build_raster_demo::<txtplot::HalfBlockRenderer>();
    let quadrant_canvas = build_raster_demo::<txtplot::QuadrantRenderer>();

    println!("Renderer showcase for txtplot\n");
    println!("Braille remains the default renderer used by ChartContext::new(...).\n");

    let _default_chart: ChartContext = build_signal_chart::<txtplot::BrailleRenderer>();
    let _half_block_chart: HalfBlockChartContext =
        build_signal_chart::<txtplot::HalfBlockRenderer>();
    let _quadrant_chart: QuadrantChartContext = build_signal_chart::<txtplot::QuadrantRenderer>();

    print_chart("Braille", &braille_chart);
    println!();
    print_chart("HalfBlock", &half_block_chart);
    println!();
    print_chart("Quadrant", &quadrant_chart);
    println!();
    print_canvas("Braille", &braille_canvas);
    println!();
    print_canvas("HalfBlock", &half_block_canvas);
    println!();
    print_canvas("Quadrant", &quadrant_canvas);
}
