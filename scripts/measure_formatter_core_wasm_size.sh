#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT_DIR/target"}
TARGET_TRIPLE=wasm32-unknown-unknown
PROFILE=release
EXAMPLE=wasm_size
CHECK_BUDGET=false
BUDGET_FILE="$ROOT_DIR/scripts/formatter_core_wasm_size_budget.env"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --check)
            CHECK_BUDGET=true
            ;;
        --budget-file)
            shift
            if [ "$#" -eq 0 ]; then
                echo "--budget-file requires a path" >&2
                exit 1
            fi
            BUDGET_FILE=$1
            ;;
        *)
            echo "unknown argument: $1" >&2
            exit 1
            ;;
    esac
    shift
done

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

if [ "$CHECK_BUDGET" = true ]; then
    if [ ! -f "$BUDGET_FILE" ]; then
        echo "budget file not found: $BUDGET_FILE" >&2
        exit 1
    fi

    # shellcheck disable=SC1090
    . "$BUDGET_FILE"

    : "${RAW_BYTES_MAX:?RAW_BYTES_MAX must be set in the budget file}"
    : "${GZIP_BYTES_MAX:?GZIP_BYTES_MAX must be set in the budget file}"

    if [ "$GZIP_BYTES" = unavailable ]; then
        echo "gzip is required for budget checks" >&2
        exit 1
    fi

    printf 'raw_bytes_max: %s\n' "$RAW_BYTES_MAX"
    printf 'gzip_bytes_max: %s\n' "$GZIP_BYTES_MAX"

    if [ "$RAW_BYTES" -gt "$RAW_BYTES_MAX" ]; then
        echo "raw wasm size budget exceeded: $RAW_BYTES > $RAW_BYTES_MAX" >&2
        exit 1
    fi

    if [ "$GZIP_BYTES" -gt "$GZIP_BYTES_MAX" ]; then
        echo "gzip wasm size budget exceeded: $GZIP_BYTES > $GZIP_BYTES_MAX" >&2
        exit 1
    fi
fi
