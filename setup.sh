#!/bin/bash

# RustOS Development Environment Setup
# Sets up everything needed to build and test the ARM64 microkernel

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TARGET="aarch64-unknown-none-softfloat"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo "RustOS Development Environment Setup"
echo "Target: ARM64 Cortex-A72"
echo "===================================="

# Update package lists
log_info "Updating package lists..."
sudo apt-get update

# Install QEMU for ARM64
log_info "Installing QEMU ARM64 support..."
if command -v qemu-system-aarch64 &> /dev/null; then
    log_success "QEMU ARM64 already installed"
else
    sudo apt-get install -y qemu-system-aarch64
    log_success "QEMU ARM64 installed"
fi

# Install build essentials
log_info "Installing build tools..."
sudo apt-get install -y build-essential curl wget git

# Install Rust if not present
if command -v rustc &> /dev/null; then
    log_info "Rust already installed"
else
    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    log_success "Rust installed"
fi

# Switch to nightly toolchain
log_info "Setting up Rust nightly toolchain..."
rustup default nightly
log_success "Nightly toolchain set as default"

# Install ARM64 target
log_info "Installing ARM64 target..."
rustup target add $TARGET
log_success "ARM64 target installed"

# Install required components
log_info "Installing required Rust components..."
rustup component add rust-src
rustup component add llvm-tools-preview
log_success "Required components installed"

# Install cargo tools
log_info "Installing cargo tools..."
cargo install cargo-binutils || log_warning "cargo-binutils already installed"
log_success "Cargo tools installed"

# Install development tools
log_info "Installing development tools..."
sudo apt-get install -y \
    gdb-multiarch \
    inotify-tools \
    tree \
    htop

log_success "Development tools installed"

echo
echo "================================="
log_success "SETUP COMPLETE!"
echo "================================="
echo
echo "Your development environment is ready for RustOS!"
echo
echo "Next steps:"
echo "  1. Verify build:        ./verify_build.sh"
echo "  2. Run quick tests:     ./run_tests.sh quick"
echo "  3. Build kernel:        make kernel"
echo "  4. Run kernel:          make run"
echo
echo "For help:                 make help"
echo
echo "Note: If this is a new shell session, run:"
echo "  source ~/.cargo/env"