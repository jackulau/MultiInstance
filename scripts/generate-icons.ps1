# PowerShell script to generate Windows .ico file from SVG
# Requires ImageMagick or similar tool to be installed

param(
    [string]$SvgPath = "assets/MultiInstance_Logo.svg",
    [string]$OutputPath = "resources/windows/app.ico"
)

$ErrorActionPreference = "Stop"

Write-Host "Generating Windows icon from SVG..."

# Check if ImageMagick is available
$magickCmd = Get-Command "magick" -ErrorAction SilentlyContinue
if (-not $magickCmd) {
    $magickCmd = Get-Command "convert" -ErrorAction SilentlyContinue
}

if ($magickCmd) {
    Write-Host "Using ImageMagick to convert SVG to ICO..."

    # Create ICO with multiple sizes for best display at all resolutions
    & magick convert $SvgPath `
        -background none `
        '(' -clone 0 -resize 16x16 ')' `
        '(' -clone 0 -resize 32x32 ')' `
        '(' -clone 0 -resize 48x48 ')' `
        '(' -clone 0 -resize 64x64 ')' `
        '(' -clone 0 -resize 128x128 ')' `
        '(' -clone 0 -resize 256x256 ')' `
        -delete 0 `
        $OutputPath

    Write-Host "Icon generated successfully: $OutputPath"
}
else {
    Write-Host "ImageMagick not found. Checking for alternative tools..."

    # Try using Inkscape if available
    $inkscapeCmd = Get-Command "inkscape" -ErrorAction SilentlyContinue
    if ($inkscapeCmd) {
        Write-Host "Using Inkscape to convert SVG to PNG, then to ICO..."

        $tempDir = [System.IO.Path]::GetTempPath()
        $pngPath = Join-Path $tempDir "temp_icon.png"

        # Export SVG to PNG at 256x256
        & inkscape $SvgPath --export-type=png --export-filename=$pngPath --export-width=256 --export-height=256

        Write-Host "PNG exported, but ICO conversion requires ImageMagick."
        Write-Host "Please install ImageMagick: winget install ImageMagick.ImageMagick"
        exit 1
    }
    else {
        Write-Host "ERROR: No image conversion tool found."
        Write-Host "Please install ImageMagick: winget install ImageMagick.ImageMagick"
        Write-Host "Or use an online converter to create app.ico from the SVG file."
        exit 1
    }
}
