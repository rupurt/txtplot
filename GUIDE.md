# Contributor Guide

This guide explains how to extend `txtplot` without fighting the current structure.

## Start Here

1. Read `README.md` for the user-facing feature set.
2. Read `CONSTITUTION.md` for the project rules.
3. Read `ARCHITECTURE.md` for the code layout.
4. Use this guide for concrete change workflows.

## Local Setup

### With Nix

```bash
nix develop
```

### Common commands

```bash
just
just fmt
just fmt-check
just cargo-check
just clippy
just test
```

## Common Extension Workflows

### Add a drawing primitive

Use this path when the change is fundamentally pixel-oriented.

1. Implement the primitive in the appropriate file under `src/canvas/`
2. Reuse existing mask, color, clipping, and coordinate helpers
3. Expose a public method only if downstream users need it directly
4. Add or update an example when the new primitive is easier to understand visually

Use this for things like:

- new shape rasterizers
- new overlay behavior
- new screen/cartesian pixel operations

### Add a chart helper

Use this path when the change is data-oriented.

1. Implement the helper in the appropriate file under `src/charts/`
2. Reuse the existing scale, range, and mapping helpers
3. Keep tick generation and axis semantics centralized
4. Prefer composing through `ChartContext` instead of reaching into canvas internals from callers

Use this for things like:

- new chart types
- new axis behaviors
- new scale-aware annotation helpers

### Add a new public type or export

1. Add the type in the owning module
2. Export it from `src/lib.rs` if it is part of the public crate contract
3. Add it to `src/prelude.rs` only if it improves common downstream use
4. Update `README.md` if users should discover it immediately

### Add tooling or contributor ergonomics

1. Update `flake.nix` for shell dependencies
2. Update `justfile` only for repeated workflows
3. Document the change in `CONFIGURATION.md`

## Examples and Benchmarks

Use the repository support directories intentionally:

- `examples/` is for runnable demonstrations and regression-friendly visual references
- `benches/` is for performance measurement, not feature discovery
- `scripts/` is for small maintenance helpers that are easier to keep as scripts than as just recipes

## Searching the Repository

The Nix shell includes `sift`, which is useful when you need semantic or full-text search across the repo and related notes.

Example:

```bash
sift search . "logarithmic axes"
```

## Planning Larger Extensions

The Nix shell also includes `keel`. This repository does not require Keel for day-to-day development, but it is available if you want structured planning for larger changes, experiments, or release work.

## Change Hygiene

Before you call a change complete:

1. Make sure the code lives in the right module
2. Make sure public exports are intentional
3. Make sure docs changed with structural changes
4. Use examples when visual behavior needs a concrete reference
