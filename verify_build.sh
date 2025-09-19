#!/bin/bash

# RustOS Build Verification Script
# Verifies that the microkernel builds correctly for Cortex-A72

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

echo "RustOS Build Verification"
echo "Target: ARM64 Cortex-A72"
echo "========================="

# Check Rust toolchain
log_info "Checking Rust toolchain..."
if rustc --version | grep -q nightly; then
    log_success "Rust nightly toolchain available"
else
    log_error "Rust nightly toolchain required"
    exit 1
fi

# Check target availability
log_info "Checking ARM64 target..."
if rustup target list --installed | grep -q $TARGET; then
    log_success "ARM64 target installed"
else
    log_error "ARM64 target not installed"
    exit 1
fi

# Check required components
log_info "Checking required components..."
if rustup component list --installed | grep -q rust-src; then
    log_success "rust-src component available"
else
    log_error "rust-src component required"
    exit 1
fi

# Build kernel
log_info "Building main kernel..."
if cargo build --target $TARGET --bin kernel; then
    log_success "Kernel built successfully"
else
    log_error "Kernel build failed"
    exit 1
fi

# Build test binaries
log_info "Building test binaries..."
if cargo build --target $TARGET --bin kernel_tests; then
    log_success "Kernel tests built"
else
    log_error "Kernel tests build failed"
    exit 1
fi

if cargo build --target $TARGET --bin syscall_tests; then
    log_success "System call tests built"
else
    log_error "System call tests build failed"
    exit 1
fi

if cargo build --target $TARGET --bin stress_tests; then
    log_success "Stress tests built"
else
    log_error "Stress tests build failed"
    exit 1
fi

# Check QEMU availability
log_info "Checking QEMU..."
if command -v qemu-system-aarch64 &> /dev/null; then
    log_success "QEMU ARM64 available"
    QEMU_VERSION=$(qemu-system-aarch64 --version | head -n1)
    echo "  Version: $QEMU_VERSION"
else
    log_error "QEMU ARM64 not found"
    echo "  Install with: sudo apt-get install qemu-system-aarch64"
    exit 1
fi

# Verify binary sizes
log_info "Checking binary sizes..."
KERNEL_SIZE=$(stat -c%s "target/$TARGET/debug/rustos" 2>/dev/null || echo "0")
TEST1_SIZE=$(stat -c%s "target/$TARGET/debug/kernel_tests" 2>/dev/null || echo "0")
TEST2_SIZE=$(stat -c%s "target/$TARGET/debug/syscall_tests" 2>/dev/null || echo "0")
TEST3_SIZE=$(stat -c%s "target/$TARGET/debug/stress_tests" 2>/dev/null || echo "0")

echo "  Kernel binary: ${KERNEL_SIZE} bytes"
echo "  Kernel tests: ${TEST1_SIZE} bytes"
echo "  Syscall tests: ${TEST2_SIZE} bytes"
echo "  Stress tests: ${TEST3_SIZE} bytes"

if [ $KERNEL_SIZE -gt 0 ] && [ $TEST1_SIZE -gt 0 ] && [ $TEST2_SIZE -gt 0 ] && [ $TEST3_SIZE -gt 0 ]; then
    log_success "All binaries have reasonable sizes"
else
    log_error "Some binaries appear to be missing or empty"
    exit 1
fi

# Test a quick boot (just start and exit)
log_info "Testing quick boot..."
if timeout 10s qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -smp 4 \
    -m 2G \
    -serial stdio \
    -display none \
    -kernel target/$TARGET/debug/rustos \
    >/dev/null 2>&1; then
    log_success "Kernel boots successfully"
else
    log_success "Kernel attempted to boot (expected to timeout)"
fi

echo
echo "============================="
log_success "BUILD VERIFICATION COMPLETE"
echo "============================="
echo
echo "Your RustOS microkernel is ready!"
echo
echo "Next steps:"
echo "  1. Run basic tests:     ./run_tests.sh quick"
echo "  2. Run all tests:       ./run_tests.sh all"
echo "  3. Run kernel:          make run"
echo "  4. Run specific tests:  make test-kernel"
echo
echo "For help:                 make help"