#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
CRATE="eframe-ios-runner"
BUILD_TYPE=release
OUT_DIR="$REPO_ROOT/target/ios"
HEADERS_DIR="$REPO_ROOT/crates/$CRATE/include"

# Ensure the iOS targets are installed.
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing Rust target: $target" >&2
        rustup target add "$target"
    fi
    cargo build -p "$CRATE" --target "$target" --release
done

DEVICE_LIB="$REPO_ROOT/target/aarch64-apple-ios/$BUILD_TYPE/lib${CRATE//-/_}.a"
SIM_LIB="$REPO_ROOT/target/aarch64-apple-ios-sim/$BUILD_TYPE/lib${CRATE//-/_}.a"

if [[ ! -f "$DEVICE_LIB" || ! -f "$SIM_LIB" ]]; then
    echo "Failed to locate built static libraries" >&2
    exit 1
fi

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

xcodebuild -create-xcframework \
    -library "$DEVICE_LIB" -headers "$HEADERS_DIR" \
    -library "$SIM_LIB" -headers "$HEADERS_DIR" \
    -output "$OUT_DIR/EframeIos.xcframework"

cp "$HEADERS_DIR"/eframe_ios_runner.h "$OUT_DIR"/

echo "Created $OUT_DIR/EframeIos.xcframework"
echo "Headers copied to $OUT_DIR/eframe_ios_runner.h"
