#!/bin/bash

# RustOS ISO Test Script
# Quick verification of ISO contents and structure

set -e

ISO_FILE=$(find . -name "rustos-*.iso" -type f -printf '%T@ %p\n' | sort -nr | head -1 | cut -d' ' -f2-)

if [ -z "$ISO_FILE" ]; then
    echo "Error: No RustOS ISO file found. Please run 'make iso' first."
    exit 1
fi

echo "============================================"
echo "RustOS ISO Verification"
echo "============================================"
echo
echo "Testing ISO: $ISO_FILE"
echo "Size: $(du -h "$ISO_FILE" | cut -f1)"
echo

# Check if file is a valid ISO
echo "Checking ISO format..."
if ! file "$ISO_FILE" | grep -q "ISO 9660"; then
    echo "Error: Invalid ISO format"
    exit 1
fi
echo "✓ Valid ISO 9660 format"

# List ISO contents
echo
echo "ISO Contents:"
xorriso -indev "$ISO_FILE" -find 2>/dev/null | sort

# Create temporary mount point
TEMP_MOUNT=$(mktemp -d)
echo
echo "Mounting ISO for detailed inspection..."

# Mount the ISO (try with sudo, fall back without if not available)
if command -v sudo >/dev/null 2>&1; then
    sudo mount -o loop "$ISO_FILE" "$TEMP_MOUNT" 2>/dev/null || mount -o loop "$ISO_FILE" "$TEMP_MOUNT"
else
    mount -o loop "$ISO_FILE" "$TEMP_MOUNT"
fi

# Check for required files
echo
echo "Verifying required files:"

if [ -f "$TEMP_MOUNT/boot/kernel.bin" ]; then
    echo "✓ Kernel binary found"
    echo "  Size: $(du -h "$TEMP_MOUNT/boot/kernel.bin" | cut -f1)"
else
    echo "✗ Missing kernel binary"
fi

if [ -f "$TEMP_MOUNT/boot/grub/grub.cfg" ]; then
    echo "✓ GRUB configuration found"
else
    echo "✗ Missing GRUB configuration"
fi

if [ -f "$TEMP_MOUNT/install.sh" ]; then
    echo "✓ Installation script found"
    if [ -x "$TEMP_MOUNT/install.sh" ]; then
        echo "  (executable)"
    else
        echo "  (not executable)"
    fi
else
    echo "✗ Missing installation script"
fi

if [ -f "$TEMP_MOUNT/VERSION" ]; then
    echo "✓ Version information found"
    echo "  Content:"
    sed 's/^/    /' "$TEMP_MOUNT/VERSION"
else
    echo "✗ Missing version information"
fi

if [ -f "$TEMP_MOUNT/README.txt" ]; then
    echo "✓ README found"
else
    echo "✗ Missing README"
fi

# Check GRUB configuration content
echo
echo "GRUB Configuration Preview:"
if [ -f "$TEMP_MOUNT/boot/grub/grub.cfg" ]; then
    echo "--- BIOS/Legacy Boot ---"
    head -10 "$TEMP_MOUNT/boot/grub/grub.cfg" | sed 's/^/  /'
fi

if [ -f "$TEMP_MOUNT/EFI/BOOT/grub.cfg" ]; then
    echo "--- UEFI Boot ---"
    head -10 "$TEMP_MOUNT/EFI/BOOT/grub.cfg" | sed 's/^/  /'
fi

# Unmount
if command -v sudo >/dev/null 2>&1; then
    sudo umount "$TEMP_MOUNT" 2>/dev/null || umount "$TEMP_MOUNT"
else
    umount "$TEMP_MOUNT"
fi
rmdir "$TEMP_MOUNT"

echo
echo "============================================"
echo "ISO Verification Complete"
echo "============================================"
echo
echo "To test the ISO in QEMU:"
echo "  make run-iso"
echo
echo "To write to USB device:"
echo "  sudo dd if=$ISO_FILE of=/dev/sdX bs=4M status=progress"
echo "  (Replace /dev/sdX with your USB device)"
echo