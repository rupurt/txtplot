//use colored::{Color, Colorize};
use colored::Color;
use txtplot::ChartContext;

fn main() {
    let width = 60;
    let mut chart = ChartContext::new(width, 10);
    let data_bars = vec![
        (10.0, Some(Color::Red)),
        (20.0, Some(Color::Green)),
        (30.0, Some(Color::Blue)),
        (40.0, Some(Color::Yellow)),
        (50.0, Some(Color::Magenta)),
        (100.0, Some(Color::BrightMagenta)),
    ];

    chart.bar_chart(&data_bars);

    let labels = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
    let num_bars = data_bars.len();

    // Width of each bar in characters
    let bar_char_width = width / num_bars;

    for (i, label) in labels.iter().enumerate().take(num_bars) {
        // Compute the character column where the label should start to stay centered
        let start_col = i * bar_char_width;
        let center_offset = (bar_char_width as i32 - label.len() as i32) / 2;
        let target_col = (start_col as i32 + center_offset).max(0) as f64;

        // Convert to a normalized coordinate (0.0 - 1.0)
        // The .text method uses (x_norm * (width - 1)) internally
        let x_norm = target_col / (width as f64 - 1.0);

        // y_norm = 0.0 is the baseline
        chart.text(label, x_norm, 0.0, Some(Color::White));
    }

    println!("{}", chart.canvas.render());
}
