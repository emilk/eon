#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BENCH_ARGS="--sample-count 8 --min-time 0.005"

cd "$ROOT_DIR"

cargo bench -q -p eon --bench bench_parse -- $BENCH_ARGS
cargo bench -q -p eon --bench bench_core_vs_serde -- $BENCH_ARGS
