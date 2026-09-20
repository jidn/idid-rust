set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default: verify

build:
    cargo build --verbose

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check --all-targets

test:
    cargo test --all-targets

lint:
    cargo clippy --all-targets -- -D warnings

shell-check:
    bash -n scripts/*.sh

verify: fmt-check build check test lint shell-check
    git diff --check
