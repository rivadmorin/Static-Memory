# Install Script for Windows
Write-Host "Building Static-Memory..." -ForegroundColor Cyan
cargo build --release

$destDir = "$env:APPDATA\Static-Memory"
if (!(Test-Path $destDir)) {
    New-Item -ItemType Directory -Path $destDir
}

Copy-Item "target\release\static-memory.exe" -Destination "$destDir\static-memory.exe"

Write-Host "Setting up Startup Registry Key..." -ForegroundColor Cyan
$registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Set-ItemProperty -Path $registryPath -Name "StaticMemory" -Value "$destDir\static-memory.exe"

Write-Host "Installation Complete!" -ForegroundColor Green
