#!/bin/bash

# RustOS ARM64 ISO Creation Script
# This script creates an installable ISO image for RustOS

set -e

RUSTOS_VERSION="0.1.0"
TARGET="aarch64-unknown-none-softfloat"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "============================================"
echo "RustOS ARM64 ISO Creation Script"
echo "Version: $RUSTOS_VERSION"
echo "Target: $TARGET"
echo "============================================"
echo

cd "$SCRIPT_DIR"

# Check if required tools are installed
echo "Checking for required tools..."
if ! command -v xorriso &> /dev/null; then
    echo "Error: xorriso is not installed. Please run 'sudo apt install xorriso'"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust toolchain."
    exit 1
fi

# Check if target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Installing Rust target: $TARGET"
    rustup target add "$TARGET"
fi

# Build the kernel in release mode
echo "Building RustOS kernel..."
if ! make release; then
    echo "Error: Failed to build RustOS kernel"
    exit 1
fi

# Create the ISO
echo "Creating ISO image..."
if ! make iso; then
    echo "Error: Failed to create ISO image"
    exit 1
fi

# Find the generated ISO
ISO_FILE=$(find . -name "rustos-*.iso" -type f -printf '%T@ %p\n' | sort -nr | head -1 | cut -d' ' -f2-)

if [ -z "$ISO_FILE" ]; then
    echo "Error: No ISO file found"
    exit 1
fi

echo
echo "============================================"
echo "ISO Creation Completed Successfully!"
echo "============================================"
echo
echo "Generated ISO: $ISO_FILE"
echo "Size: $(du -h "$ISO_FILE" | cut -f1)"
echo
echo "The ISO contains:"
echo "- RustOS ARM64 kernel"
echo "- GRUB bootloader configuration"
echo "- Installation script"
echo "- Documentation"
echo
echo "To test the ISO in QEMU:"
echo "  make run-iso"
echo
echo "To burn to USB/DVD:"
echo "  dd if=$ISO_FILE of=/dev/sdX bs=4M status=progress"
echo "  (Replace /dev/sdX with your target device)"
echo
echo "To mount and inspect:"
echo "  mkdir -p /tmp/rustos-iso"
echo "  sudo mount -o loop $ISO_FILE /tmp/rustos-iso"
echo "  ls -la /tmp/rustos-iso"
echo "  sudo umount /tmp/rustos-iso"