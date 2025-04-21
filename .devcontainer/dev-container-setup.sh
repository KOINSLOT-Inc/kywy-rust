#!/bin/bash

# Update and install common tools
sudo apt-get update
sudo apt-get install -y imagemagick
cargo install elf2uf2-rs

# Clean up
sudo apt-get clean
sudo rm -rf /var/lib/apt/lists/*
cargo clean

# Set build target to thumbv6m-none-eabi
echo '[build]\\ntarget = \"thumbv6m-none-eabi\"' > .cargo/config.toml"

# Let user know we are ready
echo "Container Setup Complete"
