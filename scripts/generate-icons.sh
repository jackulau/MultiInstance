#!/bin/bash
# Script to generate macOS .icns file from SVG
# Requires ImageMagick and iconutil (comes with Xcode)

set -e

SVG_PATH="${1:-assets/MultiInstance_Logo.svg}"
OUTPUT_PATH="${2:-resources/macos/AppIcon.icns}"

echo "Generating macOS icon from SVG..."

# Check if ImageMagick is available
if ! command -v magick &> /dev/null && ! command -v convert &> /dev/null; then
    echo "ERROR: ImageMagick not found."
    echo "Please install ImageMagick: brew install imagemagick"
    exit 1
fi

# Check if iconutil is available (macOS only)
if ! command -v iconutil &> /dev/null; then
    echo "ERROR: iconutil not found. This script must be run on macOS."
    exit 1
fi

# Create temporary iconset directory
ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
mkdir -p "$ICONSET_DIR"

echo "Creating icon sizes..."

# Generate all required icon sizes for macOS
# Using magick (ImageMagick 7) or convert (ImageMagick 6)
CONVERT_CMD="magick"
if ! command -v magick &> /dev/null; then
    CONVERT_CMD="convert"
fi

# Standard sizes and their @2x variants
$CONVERT_CMD "$SVG_PATH" -background none -resize 16x16 "$ICONSET_DIR/icon_16x16.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 32x32 "$ICONSET_DIR/icon_16x16@2x.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 32x32 "$ICONSET_DIR/icon_32x32.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 64x64 "$ICONSET_DIR/icon_32x32@2x.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 128x128 "$ICONSET_DIR/icon_128x128.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 256x256 "$ICONSET_DIR/icon_128x128@2x.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 256x256 "$ICONSET_DIR/icon_256x256.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 512x512 "$ICONSET_DIR/icon_256x256@2x.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 512x512 "$ICONSET_DIR/icon_512x512.png"
$CONVERT_CMD "$SVG_PATH" -background none -resize 1024x1024 "$ICONSET_DIR/icon_512x512@2x.png"

echo "Converting iconset to icns..."

# Create the .icns file
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_PATH"

# Cleanup
rm -rf "$(dirname "$ICONSET_DIR")"

echo "Icon generated successfully: $OUTPUT_PATH"
