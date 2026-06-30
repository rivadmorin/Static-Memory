# Install Script for Windows

$ErrorActionPreference = "Stop"

function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Blue }
function Write-Success { param($msg) Write-Host "[SUCCESS] $msg" -ForegroundColor Green }
function Write-Warning { param($msg) Write-Host "[WARNING] $msg" -ForegroundColor Yellow }
function Write-Error { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }

Write-Host "=== Static-Memory Installation ===" -ForegroundColor Cyan
$confirm = Read-Host "Apakah Anda ingin melanjutkan instalasi Static-Memory? [Y/n]"
if ($confirm -ne "" -and $confirm -ne "Y" -and $confirm -ne "y") {
    Write-Info "Instalasi dibatalkan oleh pengguna."
    exit 0
}

# 1. Build
Write-Info "Memulai kompilasi biner dengan optimasi release..."
try {
    cargo build --release
} catch {
    Write-Error "Gagal melakukan kompilasi biner via Cargo."
    exit 1
}

# 2. Directories
$destDir = "$env:APPDATA\Static-Memory"
if (!(Test-Path $destDir)) {
    Write-Info "Menyiapkan direktori aplikasi..."
    New-Item -ItemType Directory -Path $destDir | Out-Null
}

# Rollback function
function Cleanup-OnFailure {
    Write-Warning "Melakukan pembersihan otomatis (rollback)..."
    if (Test-Path "$destDir\static-memory.exe") {
        Remove-Item "$destDir\static-memory.exe" -Force
    }
    $registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
    if (Get-ItemProperty -Path $registryPath -Name "StaticMemory" -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $registryPath -Name "StaticMemory"
    }
    Write-Info "Pembersihan selesai."
}

# 3. Installation
Write-Info "Menyalin biner bauran ke folder tujuan..."
try {
    Copy-Item "target\release\static-memory.exe" -Destination "$destDir\static-memory.exe" -Force
} catch {
    Write-Error "Gagal menyalin biner ke $destDir."
    Cleanup-OnFailure
    exit 1
}

# 4. Config
$configFile = "$destDir\config.toml"
if (!(Test-Path $configFile)) {
    Write-Info "Membuat file konfigurasi default..."
    $dbPath = "$destDir\activity_log.db".Replace('\', '\\')
    $configContent = @"
[storage]
db_path = "$dbPath"
rotation_size_mb = 50
rotation_interval_days = 30
retention_days = 7

[engine]
idle_threshold_seconds = 180

[privacy]
exclude_processes = ["bitwarden.exe", "keepassxc", "1password"]
exclude_titles = ["Incognito", "Private Browsing", "Banking", "KeePass"]
"@
    Set-Content -Path $configFile -Value $configContent
}

# 5. Registry Run Key
Write-Info "Mendaftarkan layanan latar belakang (Registry Run Key)..."
try {
    $registryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
    Set-ItemProperty -Path $registryPath -Name "StaticMemory" -Value "`"$destDir\static-memory.exe`""
} catch {
    Write-Error "Gagal mendaftarkan Registry Run Key."
    Cleanup-OnFailure
    exit 1
}

# 6. Summary Board
Write-Host ""
Write-Host "┌────────────────────────────────────────────────────────────────────────────────┐" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "                  ✨ Static-Memory Berhasil Terpasang! ✨                       " -NoNewline -ForegroundColor Green; Write-Host "│" -ForegroundColor Blue
Write-Host "├────────────────────────────────────────────────────────────────────────────────┤" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host " " -NoNewline; Write-Host "🚀 Cara Menjalankan:" -NoNewline -ForegroundColor Yellow; Write-Host "                                                           " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "    Aplikasi otomatis berjalan saat startup.                                     " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "    Untuk membuka UI, jalankan: " -NoNewline; Write-Host "$destDir\static-memory.exe" -NoNewline -ForegroundColor Green; Write-Host "         " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "                                                                                " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host " " -NoNewline; Write-Host "🕵️  Latar Belakang:" -NoNewline -ForegroundColor Yellow; Write-Host "                                                            " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "    Tekan " -NoNewline; Write-Host "d" -NoNewline -ForegroundColor Blue; Write-Host " atau " -NoNewline; Write-Host "Ctrl+D" -NoNewline -ForegroundColor Blue; Write-Host " di dalam TUI untuk melepaskan antarmuka.             " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "    Perekaman tetap berjalan senyap di latar belakang.                          " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "                                                                                " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host " " -NoNewline; Write-Host "⚙️  Konfigurasi:" -NoNewline -ForegroundColor Yellow; Write-Host "                                                               " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "│" -NoNewline -ForegroundColor Blue; Write-Host "    Lokasi: " -NoNewline; Write-Host "$configFile" -NoNewline -ForegroundColor Cyan; Write-Host " " -NoNewline; Write-Host "│" -ForegroundColor Blue
Write-Host "└────────────────────────────────────────────────────────────────────────────────┘" -ForegroundColor Blue
Write-Host ""

Write-Success "Instalasi selesai!"
