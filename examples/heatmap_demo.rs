use colored::Color;
use txtplot::{ChartAnchor, ChartContext, Greyscale, HalfBlockChartContext, TextStyle, Viridis};

fn main() {
    println!("Heatmap Demo (Viridis Scale)");

    let data_width = 40;
    let data_height = 20;
    let mut data = vec![0.0; data_width * data_height];

    // Generate a simple pattern (interference pattern)
    for y in 0..data_height {
        for x in 0..data_width {
            let val1 = (x as f64 * 0.2).sin();
            let val2 = (y as f64 * 0.3).cos();
            let val3 = ((x as f64 - 20.0).powi(2) + (y as f64 - 10.0).powi(2)).sqrt() * 0.2;
            let val = ((val1 + val2 + val3.sin()).abs() / 3.0).clamp(0.0, 1.0);
            data[y * data_width + x] = val;
        }
    }

    println!("\n1. Braille Heatmap (Hybrid - Dithering + Color):");
    let mut braille_chart = ChartContext::new(60, 20);
    braille_chart.draw_axes((0.0, 1.0), (0.0, 1.0), Some(Color::White));
    braille_chart.heatmap(&data, data_width, data_height, &Viridis);
    braille_chart.anchored_text_styled(
        " BRAILLE DITHER ",
        ChartAnchor::TopLeft,
        TextStyle::new().with_foreground(Color::Yellow).bold(),
    );
    println!("{}", braille_chart.canvas.render());

    println!("\n2. HalfBlock Heatmap (High-Res Color):");
    let mut half_block_chart = HalfBlockChartContext::with_dimensions(60, 20);
    half_block_chart.draw_axes((0.0, 1.0), (0.0, 1.0), Some(Color::White));
    half_block_chart.heatmap(&data, data_width, data_height, &Viridis);
    
    let legend_entries = [
        ("High", TextStyle::new().with_foreground(Color::TrueColor { r: 253, g: 231, b: 37 })),
        ("Mid", TextStyle::new().with_foreground(Color::TrueColor { r: 33, g: 145, b: 140 })),
        ("Low", TextStyle::new().with_foreground(Color::TrueColor { r: 68, g: 1, b: 84 })),
    ];
    half_block_chart.legend(ChartAnchor::TopRight, &legend_entries);
    half_block_chart.anchored_text_styled(
        " HALFBLOCK COLOR ",
        ChartAnchor::TopLeft,
        TextStyle::new().with_foreground(Color::Yellow).bold(),
    );

    println!("{}", half_block_chart.canvas.render());
}
