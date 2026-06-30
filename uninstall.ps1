# Uninstall Script for Windows

function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Blue }
function Write-Success { param($msg) Write-Host "[SUCCESS] $msg" -ForegroundColor Green }
function Write-Warning { param($msg) Write-Host "[WARNING] $msg" -ForegroundColor Yellow }
function Write-Error { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }

Write-Host "=== Static-Memory Uninstallation ===" -ForegroundColor Cyan
$confirm = Read-Host "Apakah Anda yakin ingin menghapus Static-Memory? [y/N]"
if ($confirm -ne "y" -and $confirm -ne "Y") {
    Write-Info "Uninstalasi dibatalkan."
    exit 0
}

$deleteData = Read-Host "Apakah Anda juga ingin menghapus seluruh file database log lokal (.db dan .db.bak)? [y/N]"
$deleteData = if ($deleteData -eq "y" -or $deleteData -eq "Y") { $true } else { $false }

Write-Info "Menghapus Startup Registry Key..."
$registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
if (Get-ItemProperty -Path $registryPath -Name "StaticMemory" -ErrorAction SilentlyContinue) {
    Remove-ItemProperty -Path $registryPath -Name "StaticMemory"
}

$destDir = "$env:APPDATA\Static-Memory"
if (Test-Path $destDir) {
    if ($deleteData) {
        Write-Info "Menghapus seluruh direktori aplikasi dan data..."
        Remove-Item -Recurse -Force $destDir
    } else {
        Write-Info "Menghapus biner aplikasi..."
        if (Test-Path "$destDir\static-memory.exe") {
            Remove-Item "$destDir\static-memory.exe" -Force
        }
        Write-Warning "Data database dan konfigurasi dipertahankan di $destDir"
    }
}

Write-Success "Uninstalasi selesai!"
