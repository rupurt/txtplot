# AGENTS.md

Shared guidance for AI agents and contributors working in this repository.

## First Read Order

Use this order when you need project context:

1. `README.md` for product and user-facing context
2. `CONSTITUTION.md` for principles and compatibility posture
3. `ARCHITECTURE.md` for source layout and extension seams
4. `GUIDE.md` for concrete implementation workflows
5. `CONFIGURATION.md` for tooling and shell details
6. `RELEASE.md` only for versioning and publication work

## Project Overview

This repository publishes the `txtplot` Rust crate, a terminal plotting library built around Unicode Braille rendering.

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Stable crate-root facade |
| `src/canvas.rs` | Braille raster engine, pixel operations, render pipeline |
| `src/charts.rs` | Axis scaling, range math, chart composition, labels, ticks |
| `src/prelude.rs` | Ergonomic re-exports for downstream users |
| `examples/` | Runnable usage and showcase programs |
| `benches/` | Benchmark entrypoints |
| `scripts/` | Small repo maintenance helpers |
| `justfile` | Local workflow commands |
| `flake.nix` | Nix development environment |

## Execution Workflow

1. Start from the smallest relevant surface. Do not read the whole repository by default.
2. Keep the public API coherent:
   - `src/lib.rs` is the canonical crate-root contract.
   - `src/prelude.rs` should track user-facing ergonomic exports.
3. Put logic in the right layer:
   - Pixel-space drawing, compositing, and rendering belong in `src/canvas.rs`.
   - Data-space transforms, axes, ticks, and chart helpers belong in `src/charts.rs`.
4. When adding a new public capability, update documentation in the same change slice.
5. Prefer examples as proof surfaces for new behavior that is easier to understand visually than through unit tests alone.

## Extension Rules

### New primitive

- Add it to `BrailleCanvas` when the operation fundamentally acts on pixels or cell buffers.
- Preserve coordinate intent. If the primitive matters in both spaces, expose the cartesian and screen variants explicitly instead of hiding a conversion.
- Keep hot-path allocations out of per-point or per-frame loops.

### New chart or plotting helper

- Add it to `ChartContext` when the behavior needs ranges, axes, labels, or scale transforms.
- Reuse the central scale and range helpers rather than inventing parallel mapping code.
- Respect the background/foreground layering model instead of bypassing it.

### New public export

- Wire it through `src/lib.rs`.
- Add it to `src/prelude.rs` only if it improves common downstream ergonomics.
- Document the new surface in `README.md` or `GUIDE.md` when users should discover it directly.

### New tooling

- Add toolchain dependencies to `flake.nix`.
- Document the purpose and usage in `CONFIGURATION.md`.
- Prefer `justfile` recipes only when they simplify a repeated local workflow.

## Commands

Use the existing `just` recipes as the default workflow surface:

| Command | Purpose |
|---------|---------|
| `just fmt` | Format the crate |
| `just fmt-check` | Check formatting |
| `just cargo-check` | Type-check all targets and features |
| `just clippy` | Run clippy with warnings denied |
| `just test` | Run tests via `cargo nextest run` |
| `just check` | Run fmt-check, cargo-check, clippy, and tests |
| `just flake-check` | Run `nix flake check` |

## Agent Tooling

The Nix development shell includes two extra tools intended to make extension work easier:

- `cargo-nextest` for faster Rust test execution
- `keel` for structured planning and workflow management if the repository adopts board-driven planning later
- `sift` for fast local search across docs and code

Neither tool is required to build or use the crate. They exist to improve contributor and agent ergonomics.

## Compatibility Policy

This repository follows a hard-cutover posture by default:

1. Keep one canonical API path for each feature.
2. Do not add compatibility shims or alias layers unless the change explicitly calls for them.
3. Prefer replacing outdated internals in one change slice over supporting two rendering paths indefinitely.

## Foundational Documents

Use this order when interpreting constraints:

`CONSTITUTION.md -> ARCHITECTURE.md -> CONFIGURATION.md / GUIDE.md / RELEASE.md -> code and examples`
