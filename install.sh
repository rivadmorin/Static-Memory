#!/bin/bash
set -e

echo "Installing Static-Memory dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential libx11-dev libxtst-dev libxi-dev

echo "Building Static-Memory..."
cargo build --release

echo "Configuring permissions..."
sudo usermod -aG input $USER

echo "Installing binary..."
mkdir -p ~/.local/bin
cp target/release/static-memory ~/.local/bin/

echo "Creating data directory..."
mkdir -p ~/.local/share/static-memory/exports/

echo "Setting up systemd user service..."
mkdir -p ~/.config/systemd/user/
cat <<EOF > ~/.config/systemd/user/static-memory.service
[Unit]
Description=Static-Memory Activity Logger
After=network.target

[Service]
ExecStart=%h/.local/bin/static-memory
Restart=always

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable static-memory.service

echo "Installation complete! Please log out and log back in for group changes to take effect."
