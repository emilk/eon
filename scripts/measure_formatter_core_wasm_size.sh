#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT_DIR/target"}
TARGET_TRIPLE=wasm32-unknown-unknown
PROFILE=release
EXAMPLE=wasm_size

cd "$ROOT_DIR"

CARGO_TARGET_DIR="$TARGET_DIR" \
CARGO_PROFILE_RELEASE_LTO=fat \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
CARGO_PROFILE_RELEASE_PANIC=abort \
RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-C opt-level=z -C strip=symbols" \
cargo build -q -p eon_formatter_core --example "$EXAMPLE" --target "$TARGET_TRIPLE" --release

ARTIFACT=$(
    find "$TARGET_DIR/$TARGET_TRIPLE/$PROFILE/examples" -maxdepth 1 -name "${EXAMPLE}*.wasm" \
        | sort \
        | head -n 1
)

if [ -z "$ARTIFACT" ]; then
    echo "wasm artifact not found for $EXAMPLE" >&2
    exit 1
fi

RAW_BYTES=$(wc -c < "$ARTIFACT" | tr -d '[:space:]')

if command -v gzip >/dev/null 2>&1; then
    GZIP_BYTES=$(gzip -9c "$ARTIFACT" | wc -c | tr -d '[:space:]')
else
    GZIP_BYTES=unavailable
fi

printf 'artifact: %s\n' "$ARTIFACT"
printf 'raw_bytes: %s\n' "$RAW_BYTES"
printf 'gzip_bytes: %s\n' "$GZIP_BYTES"
