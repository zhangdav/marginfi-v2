#!/usr/bin/env sh

# Run clippy linter with appropriate feature flags
cargo clippy --features=test,test-bpf -- -D warnings \
    -A clippy::await_holding_refcell_ref \
    -A clippy::comparison_chain \
    -A clippy::too_many_arguments
