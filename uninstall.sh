#!/bin/bash

echo "Stopping and disabling service..."
systemctl --user stop static-memory.service
systemctl --user disable static-memory.service

echo "Removing binary and service file..."
rm ~/.local/bin/static-memory
rm ~/.config/systemd/user/static-memory.service

echo "Removing data directory..."
rm -rf ~/.local/share/static-memory/

echo "Uninstall complete."
