#!/bin/bash

# RustOS Test Runner Script
# Comprehensive testing for ARM64 Cortex-A72 microkernel

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGET="aarch64-unknown-none-softfloat"
QEMU_CMD="qemu-system-aarch64 -machine virt -cpu cortex-a72 -smp 4 -m 2G -serial stdio -display none"
TIMEOUT_NORMAL=60
TIMEOUT_STRESS=300

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

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

run_test() {
    local test_name=$1
    local test_binary=$2
    local timeout_val=$3
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_info "Running $test_name..."
    
    if [ ! -f "target/$TARGET/debug/$test_binary" ]; then
        log_error "$test_name: Binary not found"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
    
    # Run test with timeout
    if timeout $timeout_val $QEMU_CMD -kernel target/$TARGET/debug/$test_binary 2>/dev/null; then
        log_success "$test_name completed successfully"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_warning "$test_name timed out after ${timeout_val}s"
        else
            log_error "$test_name failed with exit code $exit_code"
        fi
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

build_tests() {
    log_info "Building all test binaries..."
    
    # Build kernel tests
    if cargo build --target $TARGET --bin kernel_tests; then
        log_success "Kernel tests built"
    else
        log_error "Failed to build kernel tests"
        exit 1
    fi
    
    # Build syscall tests
    if cargo build --target $TARGET --bin syscall_tests; then
        log_success "System call tests built"
    else
        log_error "Failed to build system call tests"
        exit 1
    fi
    
    # Build stress tests
    if cargo build --target $TARGET --bin stress_tests; then
        log_success "Stress tests built"
    else
        log_error "Failed to build stress tests"
        exit 1
    fi
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check for QEMU
    if ! command -v qemu-system-aarch64 &> /dev/null; then
        log_error "qemu-system-aarch64 not found. Please install QEMU ARM64 support."
        exit 1
    fi
    
    # Check for Rust target
    if ! rustup target list --installed | grep -q $TARGET; then
        log_warning "Target $TARGET not installed. Installing..."
        rustup target add $TARGET
    fi
    
    # Check for required components
    if ! rustup component list --installed | grep -q rust-src; then
        log_warning "rust-src component not installed. Installing..."
        rustup component add rust-src
    fi
    
    log_success "All dependencies satisfied"
}

show_summary() {
    echo
    echo "================================="
    echo "RustOS Test Summary"
    echo "================================="
    echo "Target: Cortex-A72"
    echo "Total tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    echo "================================="
    
    if [ $FAILED_TESTS -eq 0 ]; then
        log_success "ALL TESTS PASSED! ✓"
        echo "The microkernel is ready for production use."
        return 0
    else
        log_error "Some tests failed. Please review the output above."
        return 1
    fi
}

main() {
    echo "RustOS ARM64 Microkernel Test Suite"
    echo "Target: Cortex-A72"
    echo "===================================="
    
    # Parse command line arguments
    case "${1:-all}" in
        "kernel")
            TEST_SUITES="kernel"
            ;;
        "syscalls")
            TEST_SUITES="syscalls"
            ;;
        "stress")
            TEST_SUITES="stress"
            ;;
        "all")
            TEST_SUITES="kernel syscalls stress"
            ;;
        "quick")
            TEST_SUITES="kernel syscalls"
            ;;
        "help")
            echo "Usage: $0 [all|kernel|syscalls|stress|quick|help]"
            echo "  all     - Run all test suites (default)"
            echo "  kernel  - Run kernel unit tests only"
            echo "  syscalls - Run system call tests only"
            echo "  stress  - Run stress tests only"
            echo "  quick   - Run kernel and syscall tests (skip stress)"
            echo "  help    - Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown test suite: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
    
    check_dependencies
    build_tests
    
    echo
    log_info "Starting test execution..."
    
    # Run selected test suites
    for suite in $TEST_SUITES; do
        case $suite in
            "kernel")
                run_test "Kernel Unit Tests" "kernel_tests" $TIMEOUT_NORMAL
                ;;
            "syscalls")
                run_test "System Call Integration Tests" "syscall_tests" $TIMEOUT_NORMAL
                ;;
            "stress")
                run_test "Stress and Stability Tests" "stress_tests" $TIMEOUT_STRESS
                ;;
        esac
        echo
    done
    
    show_summary
}

# Handle Ctrl+C gracefully
trap 'echo; log_warning "Test execution interrupted by user"; exit 130' INT

# Run main function with all arguments
main "$@"