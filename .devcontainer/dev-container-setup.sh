#!/bin/bash

# Update and install common tools
sudo apt-get update
sudo apt-get install -y imagemagick imagemagick-6.q16 # image magick for image manipulation
sudo apt-get install -y pkg-config libudev-dev # rust/cargo dependencies

cargo install elf2uf2-rs # elf2uf2 for producing uf2 files

# Clean up to save space
sudo apt-get clean
sudo rm -rf /var/lib/apt/lists/*
cargo clean

# Let user know we are ready
echo "Container Setup Complete"
