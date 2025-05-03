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

.PHONY: help build flash lint clean check check-release license dependencies $(PYTHON_DEV_TOOLS)

help:
	@echo "make usage:"
	@echo "  make help                    Show this help message (default)"
	@echo "  make build EXAMPLE=name      Build and convert example to UF2 in $(OUTPUT_DIR)/"
	@echo "  make upload UF2=name.uf2     Flash UF2 file from output directory"
	@echo "  make lint                    Run license compliance check with reuse"
	@echo "  make clean                   Clean build and output directories"
	@echo "  make check                   Run build checks"
	@echo "  make check-release           Run build checks for release"
	@echo "  make license FILE=fileName   Create default license headder"
	@echo "  make dependencies            Install dependencies"

build:
	@if [ -z "$(EXAMPLE)" ]; then \
		echo "ERROR: EXAMPLE not specified. Use EXAMPLE=your_example_name"; \
		exit 1; \
	fi
	cargo build --target $(TARGET) --release --example $(EXAMPLE)
	cd $(BUILD_DIR) && elf2uf2-rs $(EXAMPLE)
	mkdir -p $(OUTPUT_DIR)
	cp $(BUILD_DIR)/$(EXAMPLE).uf2 $(OUTPUT_DIR)/$(EXAMPLE).uf2
	@echo "[OK] Built and saved: $(OUTPUT_DIR)/$(EXAMPLE).uf2"

upload: flash

flash:
	@if [ -z "$(UF2)" ]; then \
		echo "Usage: make flash UF2=example_name.uf2"; \
		exit 1; \
	fi
	@if [ ! -f "$(OUTPUT_DIR)/$(UF2)" ]; then \
		echo "ERROR: File not found: $(OUTPUT_DIR)/$(UF2)"; \
		exit 1; \
	fi
	python3 $(FLASH_SCRIPT) file://$(OUTPUT_DIR)/$(UF2)

clean:
	rm -rf $(BUILD_DIR) $(OUTPUT_DIR)
	cargo clean

check:
	cargo clippy --target $(TARGET) --all-features -- --no-deps
	pipenv run reuse lint

check-release:
	cargo update
	cargo clippy --target $(TARGET) --all-features -- -D warnings
	pipenv run reuse lint
	cargo publish --dry-run

license:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make license file=path/to/file.rs"; \
		exit 1; \
	fi
	@start_year=2023; \
	current_year=$$(date +%Y); \
	reuse annotate --license GPL-3.0-or-later \
		--copyright "$$start_year - $$current_year KOINSLOT, Inc." \
		$(FILE)


$(PYTHON_DEV_TOOLS):
	pipenv install --dev
	@mkdir -p $(CACHE)
	@touch $(PYTHON_DEV_TOOLS)

lint: $(PYTHON_DEV_TOOLS)
	pipenv run reuse lint

dependencies:
	@echo "🔧 Checking for Rust toolchain..."
	@if ! command -v rustup >/dev/null 2>&1; then \
		echo "Installing rustup..."; \
		curl https://sh.rustup.rs -sSf | sh -s -- -y; \
	else \
		echo "✅ rustup already installed."; \
	fi

	@echo "🔧 Ensuring Rust toolchain is up to date..."
	@rustup update

	@echo "🔧 Checking for elf2uf2-rs..."
	@if ! command -v elf2uf2-rs >/dev/null 2>&1; then \
		echo "Installing elf2uf2-rs..."; \
		cargo install elf2uf2-rs; \
	else \
		echo "✅ elf2uf2-rs already available."; \
	fi

	@echo "🐍 Checking Python environment..."
	@if [ ! -f Pipfile.lock ]; then \
		echo "Installing Python dev dependencies..."; \
		pipenv install --dev; \
	else \
		echo "✅ Pipenv environment already initialized."; \
	fi

	@mkdir -p $(CACHE)
	@touch $(PYTHON_DEV_TOOLS)
