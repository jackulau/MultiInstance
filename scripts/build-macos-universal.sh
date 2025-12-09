#!/bin/bash
# Build universal macOS binary (Intel + Apple Silicon)
# Requires both x86_64 and aarch64 Rust targets installed

set -e

APP_NAME="MultiInstance"
VERSION="1.0.0"

echo "Building MultiInstance universal binary for macOS..."

# Ensure both targets are installed
rustup target add x86_64-apple-darwin aarch64-apple-darwin 2>/dev/null || true

# Build for Intel
echo "Building for x86_64 (Intel)..."
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon
echo "Building for aarch64 (Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin

# Create universal binary using lipo
echo "Creating universal binary..."
mkdir -p target/universal-apple-darwin/release

lipo -create \
    target/x86_64-apple-darwin/release/multiinstance \
    target/aarch64-apple-darwin/release/multiinstance \
    -output target/universal-apple-darwin/release/multiinstance

# Create app bundle
APP_BUNDLE="target/universal-apple-darwin/release/$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "Creating app bundle at $APP_BUNDLE..."

rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy universal executable
cp "target/universal-apple-darwin/release/multiinstance" "$MACOS_DIR/multiinstance"

# Copy Info.plist
cp "resources/macos/Info.plist" "$CONTENTS_DIR/Info.plist"

# Copy icon if exists
if [ -f "resources/macos/AppIcon.icns" ]; then
    cp "resources/macos/AppIcon.icns" "$RESOURCES_DIR/AppIcon.icns"
fi

# Sign the app
echo "Signing app bundle..."
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || echo "Note: Code signing skipped"

echo ""
echo "Universal build complete!"
echo "App bundle: $APP_BUNDLE"
echo ""
echo "Architectures in binary:"
lipo -archs "target/universal-apple-darwin/release/multiinstance"
