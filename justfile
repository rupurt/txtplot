set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all --check

cargo-check:
  cargo check --all-targets --all-features

clippy:
  cargo clippy --all-targets --all-features -- -W clippy::all -D warnings

test:
  cargo nextest run

check: fmt-check cargo-check clippy test

flake-check:
  nix flake check
