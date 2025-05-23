# SPDX-FileCopyrightText: 2023 - 2025 KOINSLOT, Inc.
# SPDX-License-Identifier: GPL-3.0-or-later

.DEFAULT_GOAL := help

# Config
TARGET := thumbv6m-none-eabi
BUILD_DIR := target/$(TARGET)/release/examples
OUTPUT_DIR := output
FLASH_SCRIPT := ./.devcontainer/scripts/upload.py

CACHE ?= .cache
PYTHON_DEV_TOOLS := $(CACHE)/.python-dev-tools

.PHONY: help build compile flash install lint clean check check-release license dependencies update $(PYTHON_DEV_TOOLS)

help:
	@echo "make usage:"
	@echo "  make help                    Show this help message (default)"
	@echo "  make build E=example_name    Build and convert example to UF2 in $(OUTPUT_DIR)/ (pseudo compile)"
	@echo "  make flash E=name.uf2        Flash UF2 file from output directory (pseudo upload/install)"
	@echo "  make lint                    Run license compliance check with reuse"
	@echo "  make clean                   Clean build and output directories"
	@echo "  make check                   Run build checks"
	@echo "  make check-release           Run build checks for release"
	@echo "  make license FILE=fileName   Create default Koinslot license headder"
	@echo "  make dependencies            Install dependencies"
	@echo "  make update                  Update repo"

compile:build

build:
	@if [ -z "$(E)" ]; then \
		echo "ERROR: EXAMPLE not specified. Use E=your_example_name or E=all"; \
		exit 1; \
	fi
	@mkdir -p $(OUTPUT_DIR)
	@if [ "$(E)" = "all" ]; then \
		echo "🔨 Building all examples..."; \
		for file in examples/*.rs; do \
			ex=$$(basename $$file .rs); \
			echo "→ Building $$ex"; \
			cargo build --target $(TARGET) --release --example $$ex || exit 1; \
			elf2uf2-rs $(BUILD_DIR)/$$ex $(BUILD_DIR)/$$ex.uf2; \
			cp $(BUILD_DIR)/$$ex.uf2 $(OUTPUT_DIR)/$$ex.uf2; \
			echo "✔️  Saved: $(OUTPUT_DIR)/$$ex.uf2"; \
		done; \
	else \
		echo "🔨 Building example: $(E)"; \
		cargo build --target $(TARGET) --release --example $(E); \
		elf2uf2-rs $(BUILD_DIR)/$(E) $(BUILD_DIR)/$(E).uf2; \
		cp $(BUILD_DIR)/$(E).uf2 $(OUTPUT_DIR)/$(E).uf2; \
		echo "✔️  Saved: $(OUTPUT_DIR)/$(E).uf2"; \
	fi

upload: flash
install: flash

flash:
	@if [ -z "$(E)" ]; then \
		echo "Usage: make flash E=example_name"; \
		exit 1; \
	fi
	@if [ ! -f "$(OUTPUT_DIR)/$(E).uf2" ]; then \
		echo "UF2 not found: $(OUTPUT_DIR)/$(E).uf2"; \
		echo "→ 🔧 Building..."; \
		$(MAKE) build E=$(E); \
	fi
	@if command -v pipenv >/dev/null 2>&1; then \
		echo "Using pipenv..."; \
		pipenv run python $(FLASH_SCRIPT) file://$(OUTPUT_DIR)/$(E).uf2; \
	else \
		echo "Pipenv not found, trying Python directly..."; \
		if command -v python3 >/dev/null 2>&1; then \
			echo "→ Using python3"; \
			PIP_CMD="pip3"; \
			if ! python3 -c "import serial, requests, pyudev" 2>/dev/null; then \
				echo "→ Installing required dependencies..."; \
				$$PIP_CMD install -r $(dir $(FLASH_SCRIPT))requirements.txt; \
			fi; \
			python3 $(FLASH_SCRIPT) file://$(OUTPUT_DIR)/$(E).uf2; \
		elif command -v python >/dev/null 2>&1; then \
			echo "→ Using python"; \
			PIP_CMD="pip"; \
			if ! python -c "import serial, requests, pyudev" 2>/dev/null; then \
				echo "→ Installing required dependencies..."; \
				$$PIP_CMD install -r $(dir $(FLASH_SCRIPT))requirements.txt; \
			fi; \
			python $(FLASH_SCRIPT) file://$(OUTPUT_DIR)/$(E).uf2; \
		else \
			echo "Error: Neither pipenv nor python is available. Please install Python."; \
			exit 1; \
		fi; \
	fi

clean:
	rm -rf $(BUILD_DIR) $(OUTPUT_DIR)
	cargo clean

check:
	cargo clippy --target $(TARGET) --all-features -- --no-deps
	@echo "📦 Verifying all examples build..."
	$(MAKE) build E=all
	$(MAKE) lint

check-release:
	cargo update
	cargo clippy --target $(TARGET) --all-features -- -D warnings
	$(MAKE) lint
	cargo publish --dry-run --target $(TARGET)

license:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make license FILE=path/to/file.rs"; \
		exit 1; \
	fi
	@start_year=2023; \
	current_year=$$(date +%Y); \
	@if command -v pipenv >/dev/null 2>&1; then \
		pipenv run reuse annotate --license GPL-3.0-or-later \
			--copyright "$$start_year - $$current_year KOINSLOT, Inc." \
			$(FILE); \
	else \
		if command -v python3 >/dev/null 2>&1; then \
			PIP_CMD="pip3"; \
			if ! python3 -c "import reuse" 2>/dev/null; then \
				echo "→ Installing required dependencies..."; \
				$$PIP_CMD install -r $(dir $(FLASH_SCRIPT))requirements.txt; \
			fi; \
			python3 -m reuse annotate --license GPL-3.0-or-later \
				--copyright "$$start_year - $$current_year KOINSLOT, Inc." \
				$(FILE); \
		elif command -v python >/dev/null 2>&1; then \
			PIP_CMD="pip"; \
			if ! python -c "import reuse" 2>/dev/null; then \
				echo "→ Installing required dependencies..."; \
				$$PIP_CMD install -r $(dir $(FLASH_SCRIPT))requirements.txt; \
			fi; \
			python -m reuse annotate --license GPL-3.0-or-later \
				--copyright "$$start_year - $$current_year KOINSLOT, Inc." \
				$(FILE); \
		else \
			echo "Error: Neither pipenv nor python is available. Please install Python."; \
			exit 1; \
		fi; \
	fi

$(PYTHON_DEV_TOOLS):
	pipenv install --dev
	@mkdir -p $(CACHE)
	@touch $(PYTHON_DEV_TOOLS)

lint:
	@if command -v pipenv >/dev/null 2>&1; then \
		echo "Using pipenv..."; \
		pipenv run reuse lint; \
	else \
		echo "Pipenv not found, trying Python directly..."; \
		if command -v python3 >/dev/null 2>&1; then \
			echo "→ Using python3"; \
			PIP_CMD="pip3"; \
			if ! python3 -c "import reuse" 2>/dev/null; then \
				echo "→ Installing required dependencies..."; \
				$$PIP_CMD install -r $(dir $(FLASH_SCRIPT))requirements.txt; \
			fi; \
			python3 -m reuse lint; \
		elif command -v python >/dev/null 2>&1; then \
			echo "→ Using python"; \
			PIP_CMD="pip"; \
			if ! python -c "import reuse" 2>/dev/null; then \
				echo "→ Installing required dependencies..."; \
				$$PIP_CMD install -r $(dir $(FLASH_SCRIPT))requirements.txt; \
			fi; \
			python -m reuse lint; \
		else \
			echo "Error: Neither pipenv nor python is available. Please install Python."; \
			exit 1; \
		fi; \
	fi

dependencies:
	@echo "🔧 Checking for Rust toolchain..."
	@if ! command -v rustup >/dev/null 2>&1; then \
		echo "Installing rustup..."; \
		curl https://sh.rustup.rs -sSf | sh -s -- -y; \
		cargo install cargo-update; \
	else \
		echo "✅ rustup already installed. Checking for updates..."; \
		rustup update; \
	fi

	@echo "🔧 Checking for elf2uf2-rs..."
	@if ! command -v elf2uf2-rs >/dev/null 2>&1; then \
		echo "Installing elf2uf2-rs..."; \
		cargo install elf2uf2-rs; \
	else \
		echo "✅ elf2uf2-rs already available."; \
		if ! command -v cargo-install-update >/dev/null 2>&1; then \
			echo "Installing cargo-install-update..."; \
			cargo install cargo-update; \
		fi; \
		echo "🔄 Updating elf2uf2-rs via cargo-install-update..."; \
		cargo install-update -a elf2uf2-rs; \
	fi

	@echo "🐍 Setting up Python environment..."
	@pipenv install --dev "PyQt6>=6.0.0" "pyserial>=3.5" "requests>=2.25.1" "pyudev>=0.22" "reuse>=5.0.2"
	@pipenv update

	@mkdir -p $(CACHE)
	@touch $(PYTHON_DEV_TOOLS)

update:
	git pull
	@make dependencies
	@echo "✅ Everything up to date."
