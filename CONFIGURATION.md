# Configuration Guide

`txtplot` does not currently use a project config file at runtime. Configuration lives in the toolchain, the crate manifest, and the public Rust API.

## Configuration Layers

Use these layers in order:

1. `flake.nix` for the development environment
2. `Cargo.toml` for package metadata and dependency graph
3. `justfile` for common local workflows
4. Rust API calls for rendering behavior

## Development Environment

If you use Nix:

```bash
nix develop
```

The default shell provides:

- the Rust toolchain with `clippy`, `rustfmt`, and `rust-src`
- `cargo-nextest` for faster local test execution
- `just` for local commands
- `keel` for structured planning workflows
- `sift` for fast local search

## Workflow Commands

The repository-standard command surface is the `justfile`.

| Command | Purpose |
|---------|---------|
| `just fmt` | Format the crate |
| `just fmt-check` | Check formatting |
| `just cargo-check` | Type-check the crate |
| `just clippy` | Run lint checks with warnings denied |
| `just test` | Run tests via `cargo nextest run` |
| `just check` | Run formatting, cargo-check, clippy, and tests |
| `just flake-check` | Validate the flake |

## API-Level Configuration

The library is configured through Rust types, not a config file.

### Canvas-level controls

| Surface | Purpose |
|---------|---------|
| `BrailleCanvas::new(width, height)` / `HalfBlockCanvas::new(...)` / `QuadrantCanvas::new(...)` | Set terminal cell dimensions with a concrete renderer |
| `CellCanvas::<R>::new(width, height)` | Construct a canvas with an explicit renderer type |
| `RendererKind` | Represent a renderer selected from runtime input |
| `with_renderer!` | Dispatch once from a runtime renderer choice into a concrete generic type |
| `ColorBlend` | Control how per-cell colors are combined |
| `set_plot_insets` | Reserve margin space in pixel coordinates |

### Chart-level controls

| Surface | Purpose |
|---------|---------|
| `ChartContext::new(width, height)` / `HalfBlockChartContext::with_dimensions(...)` / `QuadrantChartContext::with_dimensions(...)` | Create a plotting context with a concrete renderer |
| `CellChartContext::<R>::with_dimensions(width, height)` | Construct a plotting context with an explicit renderer type |
| `set_x_scale` / `set_y_scale` / `set_scales` | Configure axis transforms |
| `get_auto_range` / `get_auto_range_scaled` | Compute plotting ranges from data |
| axis and chart helpers | Control labels, ticks, and plotted output |

### 3D-level controls

| Surface | Purpose |
|---------|---------|
| `txtplot::three_d::Projection` | Configure camera-space to screen-space projection |
| `txtplot::three_d::ZBuffer` | Track visible depth during 3D rasterization |
| `project_with_projection` / `project_to_screen` | Project 3D points into terminal screen space |
| `plot_z` / `line_z` | Draw depth-tested 3D points and lines on a canvas |

## Cargo Configuration

`Cargo.toml` is the canonical place for:

- crate name and version
- dependencies and dev-dependencies
- benchmark targets
- package metadata for publication

If you change package-level capabilities, update `Cargo.toml` first and then document the effect in `README.md` or `GUIDE.md`.

## Environment Variables

No crate-specific environment variables are required today.

Common optional Rust development variables still apply:

- `CARGO_TARGET_DIR` to redirect build artifacts
- `RUST_BACKTRACE=1` for debugging panics

## Non-goals

These are intentionally not part of the current configuration model:

- a `termplot.toml` runtime config file
- hidden global state
- alternate per-example configuration systems

Runtime behavior should remain explicit in user code.
