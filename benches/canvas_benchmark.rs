use colored::Color;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use txtplot::canvas::BrailleCanvas;

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

// Register the groups and run them
criterion_group!(benches, draw_primitives_bench, render_loop_bench);
criterion_main!(benches);
