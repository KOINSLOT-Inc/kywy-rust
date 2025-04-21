#!/bin/bash

# Update and install common tools
sudo apt-get update
sudo apt-get install -y imagemagick
sudo apt-get install -y elf2uf2

# Clean up
sudo apt-get clean
sudo rm -rf /var/lib/apt/lists/*

# Let user know we are ready
echo "Container Setup Complete"
