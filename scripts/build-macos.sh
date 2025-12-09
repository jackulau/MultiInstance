#!/bin/bash
# Build script for macOS
# This script builds the MultiInstance app and creates a proper .app bundle

set -e

# Configuration
APP_NAME="MultiInstance"
BUNDLE_ID="com.jackzhang.multiinstance"
VERSION="1.0.0"

# Determine architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
else
    TARGET="x86_64-apple-darwin"
fi

echo "Building MultiInstance for macOS ($ARCH)..."

# Build release binary
cargo build --release --target "$TARGET"

# Create app bundle structure
APP_BUNDLE="target/$TARGET/release/$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "Creating app bundle at $APP_BUNDLE..."

rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy executable
cp "target/$TARGET/release/multiinstance" "$MACOS_DIR/multiinstance"

# Copy Info.plist
cp "resources/macos/Info.plist" "$CONTENTS_DIR/Info.plist"

# Copy icon if exists
if [ -f "resources/macos/AppIcon.icns" ]; then
    cp "resources/macos/AppIcon.icns" "$RESOURCES_DIR/AppIcon.icns"
fi

# Sign the app (ad-hoc signing for local use)
echo "Signing app bundle..."
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || echo "Note: Code signing skipped (requires Xcode tools)"

echo ""
echo "Build complete!"
echo "App bundle: $APP_BUNDLE"
echo ""
echo "To install, copy the .app bundle to /Applications:"
echo "  cp -r \"$APP_BUNDLE\" /Applications/"
