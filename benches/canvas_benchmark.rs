use colored::Color;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use txtplot::{
    BrailleCanvas, BrailleRenderer, CellCanvas, CellChartContext, CellRenderer, HalfBlockRenderer,
    QuadrantRenderer,
};

fn populate_raster_scene<R: CellRenderer>(canvas: &mut CellCanvas<R>) {
    let width_px = canvas.pixel_width();
    let height_px = canvas.pixel_height();
    let max_x = width_px.saturating_sub(1) as isize;
    let max_y = height_px.saturating_sub(1) as isize;

    canvas.rect(0, 0, width_px, height_px, Some(Color::BrightBlack));
    canvas.line_screen(0, 0, max_x, max_y, Some(Color::BrightYellow));
    canvas.line_screen(0, max_y, max_x, 0, Some(Color::BrightGreen));

    let block_w = (width_px / 4).max(1);
    let block_h = (height_px / 3).max(1);
    canvas.rect_filled(
        (width_px / 6) as isize,
        (height_px / 5) as isize,
        block_w,
        block_h,
        Some(Color::Blue),
    );
    canvas.rect_filled(
        (width_px / 2) as isize,
        (height_px / 2) as isize,
        block_w,
        block_h,
        Some(Color::Red),
    );
    canvas.circle(
        (width_px / 2) as isize,
        (height_px / 2) as isize,
        (height_px / 5).max(1) as isize,
        Some(Color::Cyan),
    );
}

fn build_chart_scene<R: CellRenderer>() -> CellChartContext<R> {
    let mut chart = CellChartContext::<R>::with_dimensions(80, 24);
    chart.draw_grid(8, 4, Some(Color::BrightBlack));
    chart.draw_axes((0.0, 10.0), (-1.5, 1.5), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    chart.plot_function(
        |x: f64| (x * 0.5).cos() * 0.5,
        0.0,
        10.0,
        Some(Color::Magenta),
    );
    chart.text("sin(x)", 0.72, 0.84, Some(Color::Cyan));
    chart.text("0.5*cos(0.5x)", 0.5, 0.08, Some(Color::Magenta));
    chart
}

fn bench_renderer_raster_scene<R: CellRenderer>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
) {
    let mut canvas = CellCanvas::<R>::new(100, 32);
    let mut buffer = String::with_capacity(100 * 32 * 8);

    group.bench_function(BenchmarkId::new("raster-scene", name), |b| {
        b.iter(|| {
            canvas.clear();
            populate_raster_scene(&mut canvas);
            buffer.clear();
            canvas.render_to(&mut buffer, false, None).unwrap();
            black_box(&buffer);
        })
    });
}

fn bench_renderer_chart_scene<R: CellRenderer>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
) {
    let mut buffer = String::with_capacity(80 * 24 * 8);

    group.bench_function(BenchmarkId::new("chart-scene", name), |b| {
        b.iter(|| {
            let chart = build_chart_scene::<R>();
            buffer.clear();
            chart.canvas.render_to(&mut buffer, false, None).unwrap();
            black_box(&buffer);
        })
    });
}

fn draw_primitives_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Primitive Drawing");

    // Standard large terminal canvas (100x50 characters -> 200x200 pixels)
    let mut canvas = BrailleCanvas::new(100, 50);

    group.bench_function("1000 Lines (With Clipping)", |b| {
        b.iter(|| {
            // Clear quickly so the benchmark stays focused on drawing work
            canvas.clear();
            // Draw 1000 random lines that cross and exit the screen
            for i in 0..1000 {
                let offset = (i % 300) as isize - 50; // Let many extend out of bounds so clipping kicks in
                canvas.line_screen(
                    offset,
                    -offset,
                    200 - offset,
                    200 + offset,
                    Some(Color::Green),
                );
            }
            // Prevent the compiler from optimizing the loop away
            black_box(&canvas);
        })
    });

    group.bench_function("100 Filled Circles", |b| {
        b.iter(|| {
            canvas.clear();
            for i in 0..100 {
                let pos = (i * 2) as isize;
                canvas.circle_filled(pos, pos, 15, Some(Color::Red));
            }
            black_box(&canvas);
        })
    });

    group.finish();
}

fn render_loop_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rendering (String vs Zero-Allocation)");

    // 120x40 character canvas (typical full-screen game size)
    let mut canvas = BrailleCanvas::new(120, 40);

    // Fill the canvas with "noise" so the renderer does real work processing colors and masks
    for y in 0..canvas.pixel_height() {
        for x in 0..canvas.pixel_width() {
            if (x + y) % 3 == 0 {
                let color = if x % 2 == 0 { Some(Color::Cyan) } else { None };
                canvas.set_pixel_screen(x, y, color);
            }
        }
    }

    // 1. Traditional method: allocates a new String each time
    group.bench_function("render() - Dynamic Allocation", |b| {
        b.iter(|| {
            let output = canvas.render();
            black_box(output);
        })
    });

    // 2. Reuse the same String buffer (zero allocation)
    let mut reusable_buffer = String::with_capacity(120 * 40 * 15);
    group.bench_function("render_to() - Zero-Allocation", |b| {
        b.iter(|| {
            reusable_buffer.clear(); // Clear it, but keep the allocation
            canvas
                .render_to(&mut reusable_buffer, true, Some("Benchmark"))
                .unwrap();
            black_box(&reusable_buffer);
        })
    });

    group.finish();
}

fn renderer_comparison_bench(c: &mut Criterion) {
    let mut raster_group = c.benchmark_group("Renderer Comparison - Raster Scene");
    bench_renderer_raster_scene::<BrailleRenderer>(&mut raster_group, "braille");
    bench_renderer_raster_scene::<HalfBlockRenderer>(&mut raster_group, "halfblock");
    bench_renderer_raster_scene::<QuadrantRenderer>(&mut raster_group, "quadrant");
    raster_group.finish();

    let mut chart_group = c.benchmark_group("Renderer Comparison - Chart Scene");
    bench_renderer_chart_scene::<BrailleRenderer>(&mut chart_group, "braille");
    bench_renderer_chart_scene::<HalfBlockRenderer>(&mut chart_group, "halfblock");
    bench_renderer_chart_scene::<QuadrantRenderer>(&mut chart_group, "quadrant");
    chart_group.finish();
}

// Register the groups and run them
criterion_group!(
    benches,
    draw_primitives_bench,
    render_loop_bench,
    renderer_comparison_bench
);
criterion_main!(benches);
