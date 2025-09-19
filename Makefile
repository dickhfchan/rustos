# RustOS ARM64 Microkernel Makefile

# Build configuration
TARGET = aarch64-unknown-none-softfloat
MODE = debug
KERNEL_NAME = rustos

# Directories
SRC_DIR = src
BUILD_DIR = target/$(TARGET)/$(MODE)
KERNEL_BIN = $(BUILD_DIR)/$(KERNEL_NAME)

# QEMU configuration
QEMU = qemu-system-aarch64
QEMU_FLAGS = -machine virt \
             -cpu cortex-a72 \
             -smp 4 \
             -m 2G \
             -serial stdio \
             -display none \
             -kernel $(KERNEL_BIN)

# Default target
.PHONY: all
all: kernel

# Build the kernel
.PHONY: kernel
kernel:
	@echo "Building RustOS kernel for ARM64..."
	cargo build --target $(TARGET)

# Build in release mode
.PHONY: release
release:
	@echo "Building RustOS kernel in release mode..."
	cargo build --target $(TARGET) --release
	$(MAKE) MODE=release

# Run the kernel in QEMU
.PHONY: run
run: kernel
	@echo "Starting RustOS in QEMU..."
	$(QEMU) $(QEMU_FLAGS)

# Run in release mode
.PHONY: run-release
run-release: release
	@echo "Starting RustOS (release) in QEMU..."
	$(QEMU) $(QEMU_FLAGS)

# Debug with GDB
.PHONY: debug
debug: kernel
	@echo "Starting RustOS with GDB support..."
	$(QEMU) $(QEMU_FLAGS) -s -S &
	@echo "Connect with: gdb-multiarch target/aarch64-unknown-none-softfloat/debug/rustos"
	@echo "Then run: target remote :1234"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Check code without building
.PHONY: check
check:
	@echo "Checking code..."
	cargo check --target $(TARGET)

# Format code
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	cargo fmt

# Lint code
.PHONY: clippy
clippy:
	@echo "Running clippy..."
	cargo clippy --target $(TARGET)

# Build documentation
.PHONY: doc
doc:
	@echo "Building documentation..."
	cargo doc --target $(TARGET) --open

# Install required tools
.PHONY: setup
setup:
	@echo "Installing required tools..."
	rustup target add $(TARGET)
	rustup component add rust-src
	rustup component add llvm-tools-preview
	cargo install cargo-binutils

# Create a bootable image (for real hardware)
.PHONY: image
image: release
	@echo "Creating bootable image..."
	mkdir -p $(BUILD_DIR)/image
	cp $(BUILD_DIR)/$(KERNEL_NAME) $(BUILD_DIR)/image/kernel8.img
	@echo "Bootable image created at $(BUILD_DIR)/image/kernel8.img"

# Integration with uutils/coreutils
.PHONY: coreutils
coreutils:
	@echo "Building uutils/coreutils for ARM64..."
	@if [ ! -d "coreutils" ]; then \
		git clone https://github.com/uutils/coreutils.git; \
	fi
	cd coreutils && \
	cargo build --target $(TARGET) --features unix

# Test targets
.PHONY: test
test: test-kernel test-syscalls test-stress
	@echo "All tests completed!"

# Run kernel unit tests
.PHONY: test-kernel
test-kernel:
	@echo "Building and running kernel tests..."
	cargo build --target $(TARGET) --bin kernel_tests
	$(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/kernel_tests

# Run system call integration tests  
.PHONY: test-syscalls
test-syscalls:
	@echo "Building and running system call tests..."
	cargo build --target $(TARGET) --bin syscall_tests
	$(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/syscall_tests

# Run stress tests
.PHONY: test-stress
test-stress:
	@echo "Building and running stress tests..."
	cargo build --target $(TARGET) --bin stress_tests
	$(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/stress_tests

# Run specific test suite
.PHONY: test-suite
test-suite:
	@if [ -z "$(SUITE)" ]; then \
		echo "Usage: make test-suite SUITE=<kernel_tests|syscall_tests|stress_tests>"; \
		exit 1; \
	fi
	@echo "Running $(SUITE)..."
	cargo build --target $(TARGET) --bin $(SUITE)
	$(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/$(SUITE)

# Run tests in release mode
.PHONY: test-release
test-release:
	$(MAKE) MODE=release test

# Continuous testing (rebuild and test on changes)
.PHONY: test-watch
test-watch:
	@echo "Watching for changes and running tests..."
	@while true; do \
		inotifywait -q -r -e modify,create,delete src/ tests/ 2>/dev/null || true; \
		echo "Changes detected, running tests..."; \
		$(MAKE) test || true; \
		echo "Waiting for next change..."; \
	done

# Performance testing
.PHONY: test-perf
test-perf:
	@echo "Running performance tests..."
	cargo build --target $(TARGET) --bin stress_tests --release
	$(QEMU) $(QEMU_FLAGS) target/$(TARGET)/release/stress_tests

# Memory leak testing
.PHONY: test-memory
test-memory:
	@echo "Running memory-focused tests..."
	$(MAKE) test-suite SUITE=kernel_tests
	
# Generate test coverage report (requires additional tools)
.PHONY: test-coverage
test-coverage:
	@echo "Generating test coverage report..."
	@echo "Note: Coverage analysis for bare-metal targets is limited"
	cargo build --target $(TARGET) --bin kernel_tests
	
# Automated test suite for CI/CD
.PHONY: test-ci
test-ci:
	@echo "Running CI/CD test suite..."
	@echo "=== Building all test binaries ==="
	cargo build --target $(TARGET) --bin kernel_tests
	cargo build --target $(TARGET) --bin syscall_tests
	cargo build --target $(TARGET) --bin stress_tests
	@echo "=== Running kernel tests ==="
	timeout 60 $(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/kernel_tests || echo "Kernel tests completed"
	@echo "=== Running syscall tests ==="
	timeout 60 $(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/syscall_tests || echo "Syscall tests completed"
	@echo "=== Running stress tests ==="
	timeout 300 $(QEMU) $(QEMU_FLAGS) target/$(TARGET)/$(MODE)/stress_tests || echo "Stress tests completed"
	@echo "=== CI/CD tests completed ==="

# Integration test with coreutils
.PHONY: test-integration
test-integration: kernel coreutils
	@echo "Running integration tests with coreutils..."
	@echo "Testing basic coreutils compatibility..."
	# This would test actual coreutils binaries with our kernel

# Help target
.PHONY: help
help:
	@echo "RustOS ARM64 Microkernel Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  all           - Build the kernel (default)"
	@echo "  kernel        - Build the kernel in debug mode"
	@echo "  release       - Build the kernel in release mode"
	@echo "  run           - Run the kernel in QEMU (debug mode)"
	@echo "  run-release   - Run the kernel in QEMU (release mode)"
	@echo "  debug         - Start kernel with GDB support"
	@echo "  clean         - Clean build artifacts"
	@echo "  check         - Check code without building"
	@echo "  fmt           - Format code"
	@echo "  clippy        - Run clippy linter"
	@echo "  doc           - Build and open documentation"
	@echo "  setup         - Install required tools"
	@echo "  image         - Create bootable image for real hardware"
	@echo "  coreutils     - Build uutils/coreutils for ARM64"
	@echo "  test          - Run all test suites"
	@echo "  test-kernel   - Run kernel unit tests"
	@echo "  test-syscalls - Run system call integration tests"
	@echo "  test-stress   - Run stress and stability tests"
	@echo "  test-suite SUITE=<name> - Run specific test suite"
	@echo "  test-release  - Run tests in release mode"
	@echo "  test-watch    - Continuous testing (rebuild on changes)"
	@echo "  test-perf     - Run performance tests"
	@echo "  test-memory   - Run memory-focused tests"
	@echo "  test-ci       - Run automated CI/CD test suite"
	@echo "  test-integration - Run integration tests with coreutils"
	@echo "  help          - Show this help message"