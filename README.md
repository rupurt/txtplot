# txtplot

High-performance terminal plotting for Rust.

`txtplot` renders mathematical plots, 3D visualizations, games, and complex terminal interfaces using Unicode Braille characters and ANSI colors.

Unlike many TUI plotting libraries, `txtplot` is designed for speed: it uses flat memory buffers (`Vec<u8>`), bitwise operations, clipping, and a zero-allocation rendering path to support real-time terminal graphics.

License: [MIT](LICENSE).

## Project Navigation

This repository carries a small set of foundational documents for users, contributors, and agents:

- [AGENTS.md](AGENTS.md) - implementation guidance for AI agents and contributors
- [CONSTITUTION.md](CONSTITUTION.md) - project principles, decision hierarchy, and compatibility posture
- [ARCHITECTURE.md](ARCHITECTURE.md) - module boundaries, data flow, and extension seams
- [GUIDE.md](GUIDE.md) - user guide for plots, 3D visuals, games, and terminal interfaces
- [CONFIGURATION.md](CONFIGURATION.md) - toolchain, shell, and API-level configuration surfaces
- [RELEASE.md](RELEASE.md) - release checklist and versioning process

If you use Nix, `nix develop` provides the Rust toolchain plus `cargo-nextest`, `just`, `keel`, and `sift`.

## Key Features

- High resolution: 8 sub-pixels per character (Braille 2x4). A 100x50 terminal yields a 200x200 effective pixel canvas.
- Performance-oriented design:
  - flat buffers for cache-friendly access
  - zero-allocation rendering via `render_to`
  - Cohen-Sutherland line clipping to discard off-screen geometry before rasterization
- Advanced pixel and color control:
  - `unset_pixel` and `toggle_pixel`
  - color blending modes with `Overwrite` and `KeepFirst`
- Drawing primitives:
  - lines, circles, polygons
  - filled shapes via `rect_filled` and `circle_filled`
  - text overlay support
- Ready-to-use charts:
  - `scatter()`, `line_chart()`, `bar_chart()`, `pie_chart()`, `plot_function()`
  - linear and `log10` axes through `AxisScale`
- Auto-range and axis helpers for chart setup

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
txtplot = "0.1.0"
colored = "2.0"
```

## Quick Start

```rust
use colored::Color;
use txtplot::ChartContext;

fn main() {
    let mut chart = ChartContext::new(60, 15);
    chart.draw_grid(10, 4, Some(Color::BrightBlack));
    chart.draw_axes((0.0, 10.0), (-1.5, 1.5), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));
    chart.plot_function(
        |x: f64| (x * 0.5).cos() * 0.5,
        0.0,
        10.0,
        Some(Color::Magenta),
    );
    chart.text("sin(x)", 0.75, 0.85, Some(Color::Cyan));
    chart.text("0.5*cos(0.5x)", 0.56, 0.10, Some(Color::Magenta));

    println!("{}", chart.canvas.render());
}
```

### Logarithmic Axes

```rust
use colored::Color;
use txtplot::{AxisScale, ChartContext};

fn main() {
    let mut chart = ChartContext::new(60, 15);
    chart.set_scales(AxisScale::Linear, AxisScale::Log10);

    let points = vec![(1.0, 1.0), (2.0, 10.0), (3.0, 100.0), (4.0, 1000.0)];
    let (range_x, range_y) = ChartContext::get_auto_range_scaled(
        &points,
        0.05,
        AxisScale::Linear,
        AxisScale::Log10,
    );

    chart.draw_axes(range_x, range_y, Some(Color::White));
    chart.line_chart(&points, Some(Color::Cyan));
    println!("{}", chart.canvas.render());
}
```

### Zero-Allocation Render Loop

If you are building a real-time app, avoid `render()` and use `render_to()`:

```rust
use std::fmt;
use colored::Color;
use txtplot::ChartContext;

fn main() -> fmt::Result {
    let mut chart = ChartContext::new(60, 15);
    chart.draw_axes((0.0, 10.0), (-1.0, 1.0), Some(Color::White));
    chart.plot_function(|x: f64| x.sin(), 0.0, 10.0, Some(Color::Cyan));

    let mut buffer = String::with_capacity(8000);
    chart.canvas.render_to(&mut buffer, true, Some("60 FPS UI"))?;
    print!("{buffer}");
    Ok(())
}
```

### Renderer Showcase

Braille remains the default renderer, and the new quadrant renderer is available through `QuadrantCanvas` and `QuadrantChartContext`. The showcase example renders the same chart and raster scene through both encodings:

```bash
cargo run --release --example renderer_showcase
```

### 3D Surface Example

`txtplot` does not ship a high-level 3D scene API, but the screen-space pixel primitives are enough to build one. The new surface example projects a volatility mesh into terminal pixels, uses a tiny z-buffer for occlusion, and overlays a gradient-ascent path:

```bash
cargo run --release --example vol_surface
```

### t-SNE and Nearest-Neighbor Graph

The examples also include a small self-contained manifold-learning demo: it builds clustered 6D data, runs a lightweight t-SNE embedding, and overlays the original-space 3-nearest-neighbor graph on top of the 2D layout.

```bash
cargo run --release --example tsne_neighbors
```

## Coordinate System and Pixel API

To avoid mathematical confusion, `txtplot` offers two coordinate modes and multiple pixel operators:

| Coordinate Mode | Origin (0,0) | Y Direction | Best For |
| --- | --- | --- | --- |
| Cartesian | Bottom-left | Grows up | Math plots, functions, charts |
| Screen | Top-left | Grows down | UI, games, sprites, 3D projections |

Pixel manipulation methods:

- `set_pixel / set_pixel_screen`: turn a dot on
- `unset_pixel / unset_pixel_screen`: turn a dot off
- `toggle_pixel_screen`: flip the current state of a dot

## Demo Output

The following block is captured from `cargo run --example demo` using the built-in `render_no_color` path:

```text
A) Plain Render (render_no_color):
⣀⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀
⠀⠀⠈⠉⠢⢄⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⡠⠔⠉⠁⠀⠀
⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠱⡀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢢⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠱⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⢀⠜⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⡗⠒⠈⢆⠒⠒⠒⠒⠒⠒⠒⠒⡗⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⡗⠒⠒⠒⠒⠒⠒⠒⡠⠃⠒⠒⡗⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒⠒
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠣⡀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠘⢄⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⢠⠊⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠈⠢⡀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠘⢄⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠢⢄⡀⠀⠀⠀⠀⢀⡠⠔⠉⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠈⠉⠉⠉⠉⠁⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
```

## Examples and Demos

The repository includes advanced examples and demos:

1. Function and chart gallery

```bash
cargo run --example demo
```

2. Primitive shapes and blending

```bash
cargo run --release --example primitives_demo
```

3. 3D volatility surface with gradient ascent

```bash
cargo run --release --example vol_surface
```

4. Braille vs quadrant renderer showcase

```bash
cargo run --release --example renderer_showcase
```

5. t-SNE embedding with nearest-neighbor graph

```bash
cargo run --release --example tsne_neighbors
```

6. 3D gallery with camera, z-buffer, and zoom

```bash
cargo run --release --example 3dengine
```

7. Solar system Kepler 3D

```bash
cargo run --release --example solarsystem_kepler
```

8. Sprite engine

```bash
cargo run --release --example sprite_demo
```

8. Interactive fractals

```bash
cargo run --release --example fractalmove
```

## Performance

`txtplot` is optimized for real-time terminal rendering.

In a benchmark with a 236x104 sub-pixel canvas filled with trigonometric noise and particles on a modern machine:

- Debug mode: about 60 FPS
- Release mode: about 1600+ FPS

## Roadmap

- [x] Flat `Vec<u8>` buffers for memory efficiency
- [x] Explicit coordinate APIs for screen and cartesian modes
- [x] Cohen-Sutherland line clipping
- [x] Zero-allocation rendering via `render_to`
- [x] Filled primitives and erasers
- [x] Color blending policies
- [x] Logarithmic scaling support
- [ ] Automatic legend box
- [ ] Trait-based pluggable terminal renderers

## License

This project is licensed under the [MIT License](LICENSE).
