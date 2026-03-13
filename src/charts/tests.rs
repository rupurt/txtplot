use super::{AxisScale, CellChartContext, ChartContext};
use crate::canvas::CellRenderer;

fn visible_render(chart: &ChartContext) -> String {
    chart
        .canvas
        .render_with_options(false, None)
        .replace('\u{2800}', " ")
}

fn visible_renderer_chart_render<R: CellRenderer>(chart: &CellChartContext<R>) -> String {
    chart
        .canvas
        .render_with_options(false, None)
        .replace('\u{2800}', " ")
}

fn build_renderer_chart<R: CellRenderer>() -> CellChartContext<R> {
    let mut chart = CellChartContext::<R>::with_dimensions(8, 4);
    chart.draw_grid(4, 2, None);
    chart.draw_axes((0.0, 4.0), (-1.0, 1.0), None);
    chart.plot_function(|x: f64| x.sin(), 0.0, 4.0, None);
    chart
}

#[test]
fn plot_function_renders_over_grid_without_cell_artifacts() {
    let mut chart = ChartContext::new(12, 6);
    chart.draw_grid(4, 2, None);
    chart.draw_axes((0.0, 6.0), (-1.0, 1.0), None);
    chart.plot_function(|x: f64| x.sin(), 0.0, 6.0, None);

    assert_eq!(
        chart.canvas.render_no_color(),
        concat!(
            "⢸⠀⢠⠒⠢⡀⡇⠀⠀⡇⠀⠀\n",
            "⢸⢠⠃⡇⠀⠱⡀⠀⠀⡇⠀⠀\n",
            "⢠⠃⣀⣇⣀⣀⡇⣀⣀⣇⣀⣀\n",
            "⢸⠀⠀⡇⠀⠀⠸⡀⠀⡇⠀⢠\n",
            "⢸⠀⠀⡇⠀⠀⡇⠱⡀⡇⢠⠃\n",
            "⠸⠤⠤⡧⠤⠤⡧⠤⠑⠒⠁⠤\n",
        ),
    );
}

#[test]
fn multiple_foreground_plots_keep_crossings() {
    let mut chart = ChartContext::new(10, 5);
    chart.draw_grid(2, 2, None);
    chart.draw_axes((0.0, 6.0), (-1.0, 1.0), None);
    chart.plot_function(|x: f64| x.sin(), 0.0, 6.0, None);
    chart.plot_function(|x: f64| (x * 0.5).cos() * 0.5, 0.0, 6.0, None);

    assert_eq!(
        chart.canvas.render_no_color(),
        concat!(
            "⠐⠒⡴⡒⢄⡇⠀⠀⠀⠀\n",
            "⢸⡜⠀⠈⢺⡄⠀⠀⠀⠀\n",
            "⠘⠒⠒⠒⠒⢣⡀⠒⠒⢀\n",
            "⢸⠀⠀⠀⠀⠈⢗⢄⢀⠎\n",
            "⠸⠤⠤⠤⠤⡧⠈⠒⠋⠒\n",
        ),
    );
}

#[test]
fn line_chart_uses_full_x_span() {
    let mut chart = ChartContext::new(6, 3);
    chart.line_chart(&[(0.0, 0.0), (1.0, 1.0)], None);

    let rendered = chart.canvas.render_no_color();
    let rows: Vec<_> = rendered.lines().collect();
    let blank = '\u{2800}';

    assert!(rows
        .iter()
        .any(|row| row.chars().next().unwrap_or(blank) != blank));
    assert!(rows
        .iter()
        .any(|row| row.chars().last().unwrap_or(blank) != blank));
}

#[test]
fn log_scatter_renders_even_spacing_across_decades() {
    let mut chart = ChartContext::new(12, 6);
    chart.set_scales(AxisScale::Log10, AxisScale::Log10);
    chart.scatter(
        &[(1.0, 1.0), (10.0, 10.0), (100.0, 100.0), (1000.0, 1000.0)],
        None,
    );

    assert_eq!(
        visible_render(&chart),
        "           ⠂\n            \n       ⠈    \n    ⡀       \n            \n⠠           \n"
    );
}

#[test]
fn log_axes_render_power_of_ten_labels() {
    let mut chart = ChartContext::new(18, 6);
    chart.set_scales(AxisScale::Log10, AxisScale::Log10);
    chart.draw_axes((1.0, 1000.0), (1.0, 1000.0), None);

    assert_eq!(
        visible_render(&chart),
        "1e3               \n⢸                 \n100               \n10                \n⢸                 \n1⠤⠤⠤⠤⠤10⠤⠤⠤100⠤1e3\n"
    );
}

#[test]
fn renderer_chart_scene_outputs_remain_stable() {
    assert_eq!(
        visible_renderer_chart_render(&build_renderer_chart::<crate::BrailleRenderer>()),
        "1.0⠒⢆ ⡇ \n0.3⣀⣇⠣⡀⣀\n-0.3⡇ ⠱⡀\n-11.34.0\n"
    );
    assert_eq!(
        visible_renderer_chart_render(&build_renderer_chart::<crate::HalfBlockRenderer>()),
        "1.0▄█ █ \n0.3▄█▀▄▄\n-0.3█ ▀▄\n-11.34.0\n"
    );
    assert_eq!(
        visible_renderer_chart_render(&build_renderer_chart::<crate::QuadrantRenderer>()),
        "1.0▀▙ ▌ \n0.3▄▙▚▙▄\n-0.3▌ ▀▖\n-11.34.0\n"
    );
}
