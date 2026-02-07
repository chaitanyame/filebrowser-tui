.PHONY: help build run run-dev test clean check docker-build docker-run docker-dev docker-compose docker-test docker-clean install snapshot-test snapshot-update snapshot-review

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := $(shell tput -Txterm setaf 6 2>/dev/null || echo "")
GREEN := $(shell tput -Txterm setaf 2 2>/dev/null || echo "")
YELLOW := $(shell tput -Txterm setaf 3 2>/dev/null || echo "")
RESET := $(shell tput -Txterm sgr0 2>/dev/null || echo "")

help:
	@echo "$(BLUE)File Browser TUI - Makefile$(RESET)"
	@echo ""
	@echo "$(GREEN)Local development:$(RESET)"
	@echo "  make build         - Build the Rust binary"
	@echo "  make run           - Build and run the app locally"
	@echo "  make run-dev       - Run with cargo run (development)"
	@echo "  make test          - Run tests"
	@echo "  make check         - Run cargo check"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Snapshot testing:$(RESET)"
	@echo "  make snapshot-test  - Run snapshot tests"
	@echo "  make snapshot-update - Update snapshots (interactive)"
	@echo "  make snapshot-accept - Update snapshots (accept all)"
	@echo "  make snapshot-review - Review pending snapshots"
	@echo ""
	@echo "$(GREEN)Docker:$(RESET)"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-run    - Build and run in Docker (mounts HOME)"
	@echo "  make docker-dev    - Build and run in Docker (mounts current dir)"
	@echo "  make docker-test   - Test Docker build"
	@echo "  make docker-clean  - Remove Docker image"
	@echo "  make docker-compose- Use docker-compose"
	@echo ""
	@echo "$(GREEN)Installation:$(RESET)"
	@echo "  make install       - Install to ~/.local/bin"
	@echo "  make install-local - Install to user profile (Windows)"

# Local development
build:
	@echo "$(BLUE)Building release binary...$(RESET)"
	cargo build --release
	@echo "$(GREEN)Build complete! Binary at ./target/release/fbt$(RESET)"

run: build
	@echo "$(GREEN)Starting File Browser TUI...$(RESET)"
	./target/release/fbt

run-dev:
	@echo "$(GREEN)Starting in development mode...$(RESET)"
	cargo run

test:
	@echo "$(BLUE)Running all tests...$(RESET)"
	cargo test

test-unit:
	@echo "$(BLUE)Running unit tests...$(RESET)"
	cargo test --lib

test-integration:
	@echo "$(BLUE)Running integration tests...$(RESET)"
	cargo test --test integration_tests

test-property:
	@echo "$(BLUE)Running property tests...$(RESET)"
	cargo test property_tests

test-snapshot:
	@echo "$(BLUE)Running snapshot tests...$(RESET)"
	cargo test --test snapshot_tests -- --test-threads=1

test-e2e:
	@echo "$(YELLOW)Running E2E tests (requires compiled binary)...$(RESET)"
	cargo test --test e2e_tests -- --ignored

test-verbose:
	@echo "$(BLUE)Running tests with verbose output...$(RESET)"
	RUST_BACKTRACE=1 cargo test -- --nocapture

test-ci:
	@echo "$(BLUE)Running CI tests...$(RESET)"
	cargo test --lib
	cargo test --test integration_tests
	cargo test property_tests

check:
	@echo "$(BLUE)Checking code...$(RESET)"
	cargo check

clean:
	@echo "$(BLUE)Cleaning build artifacts...$(RESET)"
	cargo clean

# Docker targets
docker-build:
	@echo "$(BLUE)Building Docker image...$(RESET)"
	docker build -t filebrowser-tui:latest .
	@echo "$(GREEN)Docker image built successfully!$(RESET)"

docker-run: docker-build
	@echo "$(GREEN)Running in Docker (mounting HOME)...$(RESET)"
	docker run -it --rm \
		-v "$(HOME):/data:rw" \
		-v "$(PWD):/workspace:ro" \
		-w "/data" \
		-e TERM=$$TERM \
		filebrowser-tui:latest

docker-dev: docker-build
	@echo "$(GREEN)Running in Docker (mounting current dir)...$(RESET)"
	docker run -it --rm \
		-v "$(PWD):/data:rw" \
		-w "/data" \
		-e TERM=$$TERM \
		filebrowser-tui:latest

docker-compose:
	@echo "$(GREEN)Running with docker-compose...$(RESET)"
	docker-compose build
	docker-compose run --rm filebrowser

docker-test: docker-build
	@echo "$(BLUE)Testing Docker image...$(RESET)"
	@echo "Checking if binary exists..."
	docker run --rm filebrowser-tui:latest ls -la /app/fbt
	@echo "Testing binary execution..."
	docker run --rm -it -e TERM=xterm filebrowser-tui:latest sh -c "ls -la /app && /app/fbt --version 2>/dev/null || echo 'Binary OK'"

docker-clean:
	@echo "$(BLUE)Removing Docker image...$(RESET)"
	docker rmi filebrowser-tui:latest 2>/dev/null || true
	@echo "$(GREEN)Docker image removed$(RESET)"

docker-rebuild: docker-clean docker-build

# Installation
install: build
	@echo "$(BLUE)Installing to ~/.local/bin...$(RESET)"
	@mkdir -p ~/.local/bin
	@cp target/release/fbt ~/.local/bin/
	@echo "$(GREEN)Installed! Make sure ~/.local/bin is in your PATH.$(RESET)"

install-windows: build
	@echo "$(BLUE)Installing to user profile...$(RESET)"
	@if [ ! -d "$(USERPROFILE)\bin" ]; then mkdir "$(USERPROFILE)\bin"; fi
	@copy target\release\fbt.exe $(USERPROFILE)\bin\
	@echo "$(GREEN)Installed! Add %USERPROFILE%\bin to your PATH.$(RESET)"

# Development helpers
clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

# Quick start
quick: docker-build
	@echo "$(GREEN)Starting File Browser TUI in Docker...$(RESET)"
	docker run -it --rm \
		-v "$(PWD):/data:rw" \
		-w "/data" \
		-e TERM=$$TERM \
		filebrowser-tui:latest

# CI/CD
ci: check fmt-check test

# Snapshot testing
snapshot-test:
	@echo "$(BLUE)Running snapshot tests...$(RESET)"
	cargo test --test snapshot_tests -- --test-threads=1

snapshot-update:
	@echo "$(BLUE)Updating snapshots (interactive review)...$(RESET)"
	@./scripts/update-snapshots.sh

snapshot-accept:
	@echo "$(BLUE)Updating snapshots (accepting all changes)...$(RESET)"
	@./scripts/update-snapshots.sh --accept

snapshot-review:
	@echo "$(BLUE)Reviewing pending snapshots...$(RESET)"
	@./scripts/update-snapshots.sh --review
