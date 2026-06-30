#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo -e "${BLUE}=== Static-Memory Uninstallation ===${NC}"
read -p "Apakah Anda yakin ingin menghapus Static-Memory? [y/N] " confirm
if [[ $confirm != "y" && $confirm != "Y" ]]; then
    info "Uninstalasi dibatalkan."
    exit 0
fi

read -p "Apakah Anda juga ingin menghapus seluruh file database log lokal (.db dan .db.bak)? [y/N] " delete_data
delete_data=${delete_data:-N}

info "Menghentikan dan menonaktifkan layanan..."
systemctl --user stop static-memory.service 2>/dev/null || true
systemctl --user disable static-memory.service 2>/dev/null || true

info "Menghapus biner dan file layanan..."
rm -f "$HOME/.local/bin/static-memory"
rm -f "$HOME/.config/systemd/user/static-memory.service"
systemctl --user daemon-reload

if [[ $delete_data == "y" || $delete_data == "Y" ]]; then
    info "Menghapus direktori data dan konfigurasi..."
    rm -rf "$HOME/.local/share/static-memory/"
    rm -rf "$HOME/.config/static-memory/"
else
    warn "Data database dan konfigurasi dipertahankan di ~/.local/share/static-memory/ dan ~/.config/static-memory/"
fi

success "Uninstalasi selesai."
