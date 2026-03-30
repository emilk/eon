#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
FUZZ_DIR="$ROOT_DIR/fuzz"
TARGET_DIR="$FUZZ_DIR/target/debug"

cd "$ROOT_DIR"

cargo build -q --manifest-path fuzz/Cargo.toml --bins

"$TARGET_DIR/parser_paths" -runs=1 "$FUZZ_DIR/corpus/parser_paths"
"$TARGET_DIR/hidden_unicode_rejection" -runs=1 "$FUZZ_DIR/corpus/hidden_unicode_rejection"
"$TARGET_DIR/typed_paths" -runs=1 -seed=1
"$TARGET_DIR/value_roundtrip" -runs=1 -seed=1
