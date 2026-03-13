use colored::Color;
use std::env;
use std::process;
use txtplot::{with_renderer, CellChartContext, CellRenderer, RendererKind};

fn build_chart<R: CellRenderer>() -> CellChartContext<R> {
    let mut chart = CellChartContext::<R>::with_dimensions(60, 15);
    chart.draw_grid(10, 5, Some(Color::BrightBlack));
    chart.draw_axes((0.0, 10.0), (-1.5, 1.5), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    chart.plot_function(
        |x: f64| 0.5 * (x * 0.5).cos(),
        0.0,
        10.0,
        Some(Color::Magenta),
    );
    chart.text("sin(x)", 0.73, 0.82, Some(Color::Cyan));
    chart.text("0.5*cos(0.5x)", 0.52, 0.10, Some(Color::Magenta));
    chart
}

fn parse_renderer_kind() -> RendererKind {
    let Some(raw) = env::args().nth(1) else {
        return RendererKind::default();
    };

    match raw.parse() {
        Ok(kind) => kind,
        Err(_) => {
            let available = RendererKind::ALL
                .iter()
                .map(|kind| kind.name())
                .collect::<Vec<_>>()
                .join(", ");
            eprintln!("Unknown renderer `{raw}`. Available renderers: {available}");
            process::exit(2);
        }
    }
}

fn main() {
    let kind = parse_renderer_kind();

    with_renderer!(kind, |Renderer| {
        let chart = build_chart::<Renderer>();
        let title = format!(
            "runtime renderer selection ({kind}, {}x{} sub-pixels)",
            chart.canvas.pixel_width(),
            chart.canvas.pixel_height()
        );
        println!("{}", chart.canvas.render_with_options(true, Some(&title)));
    });
}
