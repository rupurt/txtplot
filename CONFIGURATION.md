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
| `BrailleCanvas::new(width, height)` | Set the terminal cell dimensions |
| `ColorBlend` | Control how per-cell colors are combined |
| `set_plot_insets` | Reserve margin space in pixel coordinates |

### Chart-level controls

| Surface | Purpose |
|---------|---------|
| `ChartContext::new(width, height)` | Create a plotting context |
| `set_x_scale` / `set_y_scale` / `set_scales` | Configure axis transforms |
| `get_auto_range` / `get_auto_range_scaled` | Compute plotting ranges from data |
| axis and chart helpers | Control labels, ticks, and plotted output |

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
