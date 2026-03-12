#!/usr/bin/env bash

set -euo pipefail

export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }-Dwarnings"

cargo check --all-targets --all-features
