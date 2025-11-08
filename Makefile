# VM-Tools Makefile
# Provides easy deployment and installation targets

.PHONY: help install install-user deploy clean test build release dev uninstall uninstall-safe post-install

# Default target
help:
	@echo "VM-Tools Build and Deployment"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Main targets:"
	@echo "  install       Quick system install (requires sudo)"
	@echo "  install-user  Install for current user only"
	@echo "  deploy        Full deployment with post-install setup"
	@echo "  dev          Development setup"
	@echo ""
	@echo "Build targets:"
	@echo "  build         Debug build"
	@echo "  release       Release build"
	@echo "  test          Run tests"
	@echo "  clean         Clean build artifacts"
	@echo ""
	@echo "Maintenance:"
	@echo "  post-install  Run post-install configuration"
	@echo "  uninstall     Basic removal (use uninstall-safe for complete)"
	@echo "  uninstall-safe Complete safe uninstall (preserves VMs)"
	@echo ""
	@echo "Quick start:"
	@echo "  make deploy   # Recommended for first-time setup"

# Quick install (system-wide)
install:
	@echo "🚀 Installing vmtools system-wide..."
	./build.sh install

# User-only install
install-user:
	@echo "👤 Installing vmtools for current user..."
	./build.sh user

# Full deployment with post-install
deploy:
	@echo "🎯 Running full deployment..."
	./build.sh deploy

# Development setup
dev:
	@echo "🛠 Setting up development environment..."
	./build.sh deps
	./build.sh debug
	@echo "✅ Development environment ready!"
	@echo "   Run: ./target/debug/vmtools --help"

# Build targets
build:
	@echo "🔨 Building in debug mode..."
	./build.sh debug

release:
	@echo "🚀 Building in release mode..."
	./build.sh release

test:
	@echo "🧪 Running tests..."
	./build.sh test

clean:
	@echo "🧹 Cleaning build artifacts..."
	./build.sh clean

# Post-install configuration
post-install:
	@echo "⚙️ Running post-install configuration..."
	./post-install.sh all

# Uninstall
uninstall:
	@echo "🗑 Removing vmtools..."
	@sudo rm -f /usr/local/bin/vmtools
	@rm -f ~/.local/bin/vmtools
	@echo "✅ vmtools removed"
	@echo "   Config directory preserved: ~/.config/vmtools"

# Format and lint
format:
	./build.sh format

lint:
	./build.sh lint

# Safe uninstallation (preserves VMs)
uninstall-safe:
	@echo "🗑️ Running safe uninstall (VMs preserved)..."
	./uninstall.sh

# Complete development cycle
all:
	./build.sh all

# One-liner remote install (for documentation)
remote-install:
	@echo "📡 Remote installation command:"
	@echo "curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | bash"