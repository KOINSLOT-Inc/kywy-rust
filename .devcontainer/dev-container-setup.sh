#!/bin/bash

# Setup environment and add scripts to path
chmod +x .devcontainer/scripts/*.sh
echo 'export PATH="$PATH:/workspaces/kywy-rust/.devcontainer/scripts"' >> ~/.bashrc
echo 'export PATH="$PATH:/workspaces/kywy-rust/.devcontainer/scripts"' >> ~/.zshrc

# Update and install common tools
sudo apt-get update
sudo apt-get install -y pkg-config libudev-dev # rust/cargo dependencies

cargo install elf2uf2-rs # elf2uf2 for producing uf2 files

pip install reuse # reuse for easy licensing 

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
