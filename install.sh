#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logs
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Confirmation
echo -e "${BLUE}=== Static-Memory Installation ===${NC}"
read -p "Apakah Anda ingin melanjutkan instalasi Static-Memory? [Y/n] " confirm
confirm=${confirm:-Y}
if [[ $confirm != "Y" && $confirm != "y" ]]; then
    info "Instalasi dibatalkan oleh pengguna."
    exit 0
fi

# 1. Prasyarat
info "Memverifikasi hak akses/prasyarat sistem..."
sudo apt-get update -qq || warn "Gagal menjalankan apt-get update."
sudo apt-get install -y build-essential libx11-dev libxtst-dev libxi-dev -qq || {
    error "Gagal menginstal dependensi sistem. Pastikan Anda memiliki hak akses sudo."
    exit 1
}

# 2. Build
info "Memulai kompilasi biner dengan optimasi release..."
if ! cargo build --release; then
    error "Gagal melakukan kompilasi biner via Cargo."
    exit 1
fi

# 3. Directories
BIN_DIR="$HOME/.local/bin"
DATA_DIR="$HOME/.local/share/static-memory"
CONFIG_DIR="$HOME/.config/static-memory"

info "Menyiapkan direktori aplikasi..."
mkdir -p "$BIN_DIR"
mkdir -p "$DATA_DIR/exports"
mkdir -p "$CONFIG_DIR"

# Rollback function
cleanup_on_failure() {
    warn "Melakukan pembersihan otomatis (rollback)..."
    rm -f "$BIN_DIR/static-memory"
    systemctl --user stop static-memory.service 2>/dev/null || true
    systemctl --user disable static-memory.service 2>/dev/null || true
    rm -f "$HOME/.config/systemd/user/static-memory.service"
    info "Pembersihan selesai."
}

# 4. Installation
info "Menyalin biner bauran ke folder tujuan PATH..."
if ! cp target/release/static-memory "$BIN_DIR/static-memory"; then
    error "Gagal menyalin biner ke $BIN_DIR."
    cleanup_on_failure
    exit 1
fi

# 5. Config
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    info "Membuat file konfigurasi default di $CONFIG_DIR/config.toml..."
    cat <<EOF > "$CONFIG_DIR/config.toml"
[storage]
db_path = "$DATA_DIR/activity_log.db"
rotation_size_mb = 50
rotation_interval_days = 30
retention_days = 7

[engine]
idle_threshold_seconds = 180

[privacy]
exclude_processes = ["bitwarden.exe", "keepassxc", "1password"]
exclude_titles = ["Incognito", "Private Browsing", "Banking", "KeePass"]
EOF
fi

# 6. Service
info "Mendaftarkan layanan latar belakang (systemd user service)..."
mkdir -p "$HOME/.config/systemd/user/"
cat <<EOF > "$HOME/.config/systemd/user/static-memory.service"
[Unit]
Description=Static-Memory Activity Logger
After=network.target

[Service]
ExecStart=$BIN_DIR/static-memory
WorkingDirectory=$CONFIG_DIR
Restart=always

[Install]
WantedBy=default.target
EOF

if ! systemctl --user daemon-reload; then
    error "Gagal memuat ulang daemon systemd."
    cleanup_on_failure
    exit 1
fi

if ! systemctl --user enable static-memory.service; then
    error "Gagal mengaktifkan layanan static-memory."
    cleanup_on_failure
    exit 1
fi

if ! systemctl --user start static-memory.service; then
    warn "Gagal menjalankan layanan secara otomatis. Anda mungkin perlu menjalankannya secara manual."
fi

# 7. PATH Idempotency
info "Memastikan $BIN_DIR ada di PATH..."
add_to_path() {
    local shell_file=$1
    if [ -f "$shell_file" ]; then
        if ! grep -q "$BIN_DIR" "$shell_file"; then
            echo -e "\n# Static-Memory path\nexport PATH=\"\$PATH:$BIN_DIR\"" >> "$shell_file"
            info "Menambahkan $BIN_DIR ke $shell_file"
        else
            info "$BIN_DIR sudah terdaftar di $shell_file"
        fi
    fi
}

add_to_path "$HOME/.bashrc"
add_to_path "$HOME/.zshrc"

# 8. Permissions
info "Mengonfigurasi izin akses input..."
if ! groups $USER | grep -q "\binput\b"; then
    sudo usermod -aG input $USER || warn "Gagal menambahkan user ke grup 'input'. Jalankan secara manual: sudo usermod -aG input \$USER"
else
    info "User sudah berada dalam grup 'input'."
fi

# 9. Summary Board
echo -e "\n"
echo -e "${BLUE}┌────────────────────────────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│${NC}                  ${GREEN}✨ Static-Memory Berhasil Terpasang! ✨${NC}                       ${BLUE}│${NC}"
echo -e "${BLUE}├────────────────────────────────────────────────────────────────────────────────┤${NC}"
echo -e "${BLUE}│${NC} ${YELLOW}🚀 Cara Menjalankan:${NC}                                                           ${BLUE}│${NC}"
echo -e "${BLUE}│${NC}    Cukup ketik: ${GREEN}static-memory${NC}                                                 ${BLUE}│${NC}"
echo -e "${BLUE}│${NC}                                                                                ${BLUE}│${NC}"
echo -e "${BLUE}│${NC} ${YELLOW}🕵️  Latar Belakang:${NC}                                                            ${BLUE}│${NC}"
echo -e "${BLUE}│${NC}    Daemon otomatis aktif. Tekan ${BLUE}d${NC} atau ${BLUE}Ctrl+D${NC} di dalam TUI untuk              ${BLUE}│${NC}"
echo -e "${BLUE}│${NC}    melepaskan antarmuka. Perekaman tetap berjalan senyap.                     ${BLUE}│${NC}"
echo -e "${BLUE}│${NC}                                                                                ${BLUE}│${NC}"
echo -e "${BLUE}│${NC} ${YELLOW}⚙️  Konfigurasi:${NC}                                                               ${BLUE}│${NC}"
printf "${BLUE}│${NC}    Lokasi: ${CYAN}%-66s${NC} ${BLUE}│${NC}\n" "$CONFIG_DIR/config.toml"
echo -e "${BLUE}└────────────────────────────────────────────────────────────────────────────────┘${NC}"
echo -e "\n"

success "Instalasi selesai! Silakan log out dan log in kembali agar perubahan grup 'input' dan PATH berlaku."
