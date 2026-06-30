# Uninstall Script for Windows
Write-Host "Removing Startup Registry Key..." -ForegroundColor Yellow
$registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Remove-ItemProperty -Path $registryPath -Name "StaticMemory" -ErrorAction SilentlyContinue

$destDir = "$env:APPDATA\Static-Memory"
if (Test-Path $destDir) {
    Write-Host "Removing data directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $destDir
}

Write-Host "Uninstall Complete!" -ForegroundColor Green
