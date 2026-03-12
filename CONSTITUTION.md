# Project Constitution

> Keep the public plotting contract small, explicit, and fast.

This document captures the governing principles for evolving this repository so contributors and agents can extend it without drifting into parallel abstractions or accidental API sprawl.

## Document Hierarchy

Use repository guidance in this order:

`README.md -> CONSTITUTION.md -> ARCHITECTURE.md -> CONFIGURATION.md / GUIDE.md / RELEASE.md -> code`

`Cargo.toml` and `src/lib.rs` are the canonical public contract for the published crate.

## Core Belief

Humans decide the public contract and architectural direction. Agents and contributors implement within those constraints. Examples, benchmarks, and checks provide the proof that the extension still fits the contract.

## Project Model

This repository is a library first.

- The crate surface matters more than any one example.
- Rendering behavior should have one canonical implementation path.
- Extensions should strengthen the existing layers, not create bypasses around them.

## Principles

### 1. Library-first design

Prefer reusable APIs over repo-local shortcuts. Example programs should demonstrate the crate, not become alternate product surfaces.

### 2. One rendering core

`BrailleCanvas` is the canonical raster engine. Do not create parallel rendering cores unless a deliberate architectural decision says otherwise.

### 3. Separate data-space from pixel-space

`ChartContext` owns chart semantics and coordinate mapping. `BrailleCanvas` owns rasterization and output. Keep that split clear.

### 4. Performance is part of correctness

This crate exists for terminal plotting with a performance-oriented implementation. Avoid regressions that add unnecessary allocation, duplicate passes, or fragmented buffers on the hot path.

### 5. Public APIs stay explicit

Exports should be intentional. Additions to `src/lib.rs` and `src/prelude.rs` are product decisions, not incidental side effects of internal refactors.

### 6. Documentation moves with structure

When module boundaries, workflows, or toolchain expectations change, update the matching foundational document in the same change slice.

### 7. Hard cutover by default

Do not keep legacy aliases, dual APIs, or compatibility bridges unless a change explicitly requires them. Prefer one canonical approach.

## Human and Agent Responsibilities

| Human responsibilities | Agent and contributor responsibilities |
|------------------------|----------------------------------------|
| Approve public API direction | Implement focused changes |
| Decide compatibility posture | Respect documented module boundaries |
| Decide major architecture shifts | Keep docs aligned with code |
| Review releases and version bumps | Extend examples and tooling coherently |

## Extension Contract

When adding capability:

1. Put raster behavior in the raster layer.
2. Put chart semantics in the chart layer.
3. Expose only the public API that downstream users actually need.
4. Leave the repository easier to understand than before the change.

## Evolution

This constitution is expected to evolve as the crate grows. When the architecture changes materially, update this document and `ARCHITECTURE.md` together so future contributors inherit a coherent contract instead of archeology work.
