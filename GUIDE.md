# User Guide

This guide is for people using `txtplot` to build:

- mathematical plots
- 3D visualizations
- games
- complex terminal interfaces

Use `README.md` for the fast overview. Use this guide when you want to choose the right API surface, renderer, and usage pattern for your application.

## Choose the Right API

`txtplot` has two main layers:

| If you are building... | Start with | Why |
| --- | --- | --- |
| Mathematical plots and charts | `ChartContext` | Axes, auto-range, scales, labels, and chart helpers are already built in |
| The same plots with a different cell renderer | `HalfBlockChartContext`, `QuadrantChartContext`, or `CellChartContext<R>` | Same charting flow, different terminal cell encoding |
| 3D scenes, games, sprites, dashboards, or custom UIs | `BrailleCanvas`, `HalfBlockCanvas`, `QuadrantCanvas`, or `CellCanvas<R>` | Direct pixel control without chart assumptions |

Rule of thumb:

- If your data starts as `(x, y)` values, use `ChartContext`
- If your data starts as projected pixels, use a canvas directly

## Renderer Choice

### Braille

Use `ChartContext` or `BrailleCanvas` when you want the default renderer.

- Highest current spatial density: `2x4` sub-pixels per terminal cell
- Best fit for mathematical plots, dense curves, and fine detail
- This is the default path used throughout the crate

### Quadrants

Use `QuadrantChartContext`, `QuadrantCanvas`, or `CellChartContext<QuadrantRenderer>` when you want the quadrant renderer.

- `2x2` sub-pixels per cell
- Bolder and simpler terminal glyphs
- Useful when you want chunkier shapes or to compare cell encodings

### HalfBlocks

Use `HalfBlockChartContext`, `HalfBlockCanvas`, or `CellChartContext<HalfBlockRenderer>` when you want the half-block renderer.

- `1x2` sub-pixels per cell
- Uses foreground and background ANSI channels to split top and bottom halves
- Useful for dashboards, sprite-like UI, and bold chart variants where color contrast matters more than maximum density

## Mathematical Plots

For plots, the normal flow is:

1. Create a `ChartContext`
2. Set scales if needed
3. Draw a grid or axes
4. Plot functions or series
5. Render the canvas

Basic shape:

```rust
use colored::Color;
use txtplot::ChartContext;

fn main() {
    let mut chart = ChartContext::new(60, 15);
    chart.draw_grid(10, 4, Some(Color::BrightBlack));
    chart.draw_axes((0.0, 10.0), (-1.5, 1.5), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    println!("{}", chart.canvas.render());
}
```

Useful plot features:

- `scatter()` for point clouds
- `line_chart()` for connected series
- `bar_chart()` and `pie_chart()` for categorical summaries
- `plot_function()` for mathematical curves
- `get_auto_range()` and `get_auto_range_scaled()` for automatic axes
- `AxisScale::Log10` for logarithmic plots

If you want the same chart flow with a different renderer:

```rust
use colored::Color;
use txtplot::QuadrantChartContext;

fn main() {
    let mut chart = QuadrantChartContext::with_dimensions(60, 15);
    chart.draw_axes((0.0, 10.0), (-1.0, 1.0), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    println!("{}", chart.canvas.render());
}
```

## 3D Visualizations

`txtplot` does not provide a high-level scene graph or camera API. The intended pattern is:

1. Keep your own scene, camera, and projection math
2. Project world-space points into screen-space pixels
3. Draw with `set_pixel_screen()`, `line_screen()`, and text helpers
4. Add your own z-buffer or visibility logic if needed

This works well because the canvas is just a fast terminal raster target.

Recommended examples:

- `cargo run --release --example vol_surface`
- `cargo run --release --example 3dengine`
- `cargo run --release --example solarsystem_kepler`

When building 3D views:

- Prefer screen-space APIs (`*_screen`) after projection
- Keep your z-buffer outside the canvas
- Reuse a single canvas per frame and call `clear()`
- Use `render_to()` when you want a reusable output buffer

## Games and Complex Terminal Interfaces

For games, simulations, dashboards, and custom TUIs, start with a canvas instead of the chart layer.

Typical flow:

1. Create a canvas once
2. Clear and redraw each frame
3. Use screen coordinates for sprites, UI chrome, and projected geometry
4. Overlay text with `text_screen()`, `label_screen()`, `panel_screen()`, and `set_cell_background()`
5. Render into a reusable `String` with `render_to()`

Good fits for direct-canvas work:

- sprite movement
- particle systems
- minimaps and status HUDs
- terminal dashboards
- custom widgets that do not map cleanly to `(x, y)` data

Useful HUD helpers:

- `text_screen()` for top-left text placement in terminal cell coordinates
- `label_screen()` for compact status pills and colored labels
- `panel_screen()` for bordered boxes with optional background fill and title text

Recommended examples:

- `cargo run --release --example sprite_demo`
- `cargo run --release --example renderer_showcase`
- `cargo run --release --example fractalmove`

## Coordinate Systems

Use the right coordinate mode for the job:

| API style | Origin | Best for |
| --- | --- | --- |
| Cartesian (`set_pixel`, `line`) | Bottom-left | plots, math, chart primitives |
| Screen (`set_pixel_screen`, `line_screen`) | Top-left | games, UIs, 3D projections, sprites |

If you already have projected screen coordinates, do not convert back into chart coordinates. Draw directly in screen space.

## Rendering Patterns

### Simple output

Use `render()` when convenience matters more than allocations:

```rust
println!("{}", chart.canvas.render());
```

### Reused output buffer

Use `render_to()` in real-time loops:

```rust
use std::fmt;

fn draw_frame(chart: &txtplot::ChartContext, buffer: &mut String) -> fmt::Result {
    buffer.clear();
    chart.canvas.render_to(buffer, true, Some("frame"))?;
    Ok(())
}
```

This is the preferred path for:

- animations
- games
- live dashboards
- anything redrawing many times per second

## Example Map

Use the examples directory by intent:

- `demo` for the general chart gallery
- `renderer_showcase` for Braille vs half-block vs quadrant comparison
- `vol_surface` for 3D projected surfaces
- `3dengine` for low-level 3D raster patterns
- `solarsystem_kepler` for larger scene orchestration
- `sprite_demo` for direct-canvas sprite work
- `fractalmove` for interactive rendering loops
- `tsne_neighbors` for a richer analytical plotting example

## Performance Checklist

If performance matters:

1. Run in release mode
2. Reuse canvases instead of recreating them every frame
3. Prefer `render_to()` over `render()` in loops
4. Keep clipping and projection logic outside hot text formatting paths
5. Choose the chart layer only when you actually need scales and axes

## Where to Look Next

- `README.md` for the quick-start path and feature overview
- `examples/` for runnable patterns
- `ARCHITECTURE.md` if you are modifying the crate itself rather than just using it
