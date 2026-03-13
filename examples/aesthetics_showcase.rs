use colored::Color;
use txtplot::{ChartAnchor, ChartContext, TextStyle};

fn main() {
    let mut chart = ChartContext::new(60, 20);

    // Draw some data
    chart.draw_grid(10, 4, Some(Color::TrueColor { r: 40, g: 40, b: 40 }));
    chart.draw_axes((0.0, 10.0), (-1.2, 1.2), Some(Color::White));

    chart.plot_function(|x| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    chart.plot_function(|x| (x * 0.8).cos() * 0.5, 0.0, 10.0, Some(Color::Magenta));

    // Anchored annotations
    chart.anchored_text_styled(
        "TOP LEFT",
        ChartAnchor::TopLeft,
        TextStyle::new().with_foreground(Color::Yellow).bold(),
    );
    chart.anchored_text_styled(
        "TOP RIGHT",
        ChartAnchor::TopRight,
        TextStyle::new().with_foreground(Color::Green).dim(),
    );
    chart.anchored_text_styled(
        "BOTTOM LEFT",
        ChartAnchor::BottomLeft,
        TextStyle::new().with_foreground(Color::Red),
    );
    chart.anchored_text_styled(
        "BOTTOM RIGHT",
        ChartAnchor::BottomRight,
        TextStyle::new().with_foreground(Color::Blue),
    );
    chart.anchored_text_styled(
        "CENTERED",
        ChartAnchor::Center,
        TextStyle::new().with_foreground(Color::White).bold(),
    );

    // Legend
    let entries = [
        ("Sine Wave", TextStyle::new().with_foreground(Color::Cyan)),
        ("Cosine (Scaled)", TextStyle::new().with_foreground(Color::Magenta)),
    ];
    chart.legend(ChartAnchor::TopRight, &entries);

    println!("{}", chart.canvas.render());
}
