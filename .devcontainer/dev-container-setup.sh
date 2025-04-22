#!/bin/bash

# Setup environment and add scripts to path
chmod +x .devcontainer/scripts/*.sh
echo 'export PATH="$PATH:/workspaces/kywy-rust/scripts"' >> ~/.bashrc

# Update and install common tools
sudo apt-get update
sudo apt-get install -y pkg-config libudev-dev # rust/cargo dependencies

cargo install elf2uf2-rs # elf2uf2 for producing uf2 files

# Install ImageMagick
sudo apt-get install -y build-essential pkg-config libx11-dev libxext-dev zlib1g-dev

# Clean up to save space
sudo apt-get clean
sudo rm -rf /var/lib/apt/lists/*
cargo clean

# Let user know we are ready
echo "Container Setup Complete"
