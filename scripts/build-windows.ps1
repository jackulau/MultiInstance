# Build script for Windows
# This script builds the MultiInstance app and creates a distributable package

param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$CreateInstaller = $false,
    [switch]$SkipBuild = $false
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "MultiInstance"
$Version = "1.0.0"
$Publisher = "Jack Zhang"

Write-Host "========================================"
Write-Host "Building $AppName for Windows ($Target)"
Write-Host "========================================"
Write-Host ""

# Ensure we're in the project root
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $ProjectRoot

# Check if icon exists, if not try to generate it
$IconPath = "resources/windows/app.ico"
if (-not (Test-Path $IconPath)) {
    Write-Host "Warning: app.ico not found. The build will proceed but the exe won't have an icon."
    Write-Host "Run 'scripts/generate-icons.ps1' to generate the icon from SVG."
    Write-Host ""
}

if (-not $SkipBuild) {
    # Build release binary
    Write-Host "Building release binary..."
    cargo build --release --target $Target
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed!"
        exit 1
    }
}

$BinaryPath = "target/$Target/release/multiinstance.exe"
if (-not (Test-Path $BinaryPath)) {
    Write-Host "ERROR: Binary not found at $BinaryPath"
    exit 1
}

# Create distribution directory
$DistDir = "target/$Target/release/dist"
$ZipName = "$AppName-$Version-windows-$Target.zip"

Write-Host ""
Write-Host "Creating distribution package..."

# Clean and create dist directory
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Path $DistDir | Out-Null

# Copy executable
Copy-Item $BinaryPath "$DistDir/$AppName.exe"

# Copy README and LICENSE if they exist
if (Test-Path "README.md") {
    Copy-Item "README.md" "$DistDir/"
}
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" "$DistDir/"
}

# Create portable ZIP
Write-Host "Creating portable ZIP: $ZipName"
$ZipPath = "target/$Target/release/$ZipName"
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath
}
Compress-Archive -Path "$DistDir/*" -DestinationPath $ZipPath

Write-Host ""
Write-Host "========================================"
Write-Host "Build Complete!"
Write-Host "========================================"
Write-Host ""
Write-Host "Outputs:"
Write-Host "  Executable: $BinaryPath"
Write-Host "  Portable ZIP: $ZipPath"
Write-Host ""

# If creating installer with Inno Setup
if ($CreateInstaller) {
    Write-Host "Creating installer with Inno Setup..."

    $InnoSetupPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
    if (-not (Test-Path $InnoSetupPath)) {
        Write-Host "Warning: Inno Setup not found. Skipping installer creation."
        Write-Host "Install from: https://jrsoftware.org/isdownload.php"
    }
    else {
        $IssPath = "scripts/installer.iss"
        if (Test-Path $IssPath) {
            & $InnoSetupPath $IssPath
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Installer created successfully!"
            }
            else {
                Write-Host "Warning: Installer creation failed."
            }
        }
        else {
            Write-Host "Warning: installer.iss not found. Skipping installer creation."
        }
    }
}

Write-Host ""
Write-Host "Done!"
