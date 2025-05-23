#!/bin/bash

# SPDX-FileCopyrightText: 2025 2025 KOINSLOT Inc.
#
# SPDX-License-Identifier: GPL-3.0-or-later

# Setup environment and add scripts to path
chmod +x .devcontainer/scripts/*
echo 'export PATH="$PATH:/workspaces/kywy-rust/.devcontainer/scripts"' >> ~/.bashrc
echo 'export PATH="$PATH:/workspaces/kywy-rust/.devcontainer/scripts"' >> ~/.zshrc

# Welcome Message
chmod +x .devcontainer/welcome-message.sh
echo 'source /workspaces/kywy-rust/.devcontainer/welcome-message.sh' >> ~/.bashrc
echo 'source /workspaces/kywy-rust/.devcontainer/welcome-message.sh' >> ~/.zshrc

# Update and install common tools
sudo apt-get update
sudo apt-get install -y pkg-config libudev-dev # rust/cargo dependencies

#install rust target
rustup target add thumbv6m-none-eabi

#install elf2uf2
cargo install elf2uf2-rs # elf2uf2 for producing uf2 files

#install python
sudo apt-get install -y python3 python3-pip pipenv

make dependencies

# Install ImageMagick
sudo apt-get install -y build-essential pkg-config libx11-dev libxext-dev zlib1g-dev


# Clean up to save space
sudo apt-get clean
sudo rm -rf /var/lib/apt/lists/*
cargo clean
pip cache purge

# Remove unused SDKs and preinstalled junk
sudo rm -rf /usr/share/dotnet
sudo rm -rf ~/.dotnet
sudo rm -rf /usr/local/lib/node_modules
sudo rm -rf /usr/local/bin/node /usr/local/bin/npm
rm -rf ~/.npm
sudo apt-get remove -y docker-ce docker-ce-cli containerd.io
sudo rm -rf /usr/share/doc/*
sudo apt-get clean
rm -rf ~/.cache
sudo apt-get autoremove -y
sudo apt-get autoclean

# Let user know we are ready
echo "Container Setup Complete"
