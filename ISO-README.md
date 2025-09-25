# RustOS ARM64 Installable ISO

This document describes the RustOS ARM64 installable ISO image and how to use it.

## Overview

The RustOS ISO contains a complete installable distribution of the RustOS ARM64 microkernel operating system with COSMIC desktop environment integration.

## Contents

The ISO image includes:

- **RustOS ARM64 Kernel**: The main microkernel compiled for ARM64 architecture
- **GRUB Bootloader**: Multi-boot configuration supporting both BIOS and UEFI systems
- **Installation Script**: Automated installer for deploying RustOS to storage devices
- **COSMIC Desktop**: Integrated desktop environment components
- **Documentation**: System information and usage guides

## System Requirements

### Minimum Requirements
- ARM64 (AArch64) processor
- 2 GB RAM
- 4 GB storage space
- UEFI or Legacy BIOS support

### Recommended Requirements
- ARM64 processor (Cortex-A72 or newer)
- 4 GB RAM
- 16 GB storage space
- Graphics card with basic framebuffer support
- USB ports for input devices

## Creating the ISO

### Using Make
```bash
make iso
```

### Using the Creation Script
```bash
./create-iso.sh
```

### Manual Build Process
```bash
# Build kernel in release mode
make release

# Create ISO structure
mkdir -p iso_temp/boot/grub
mkdir -p iso_temp/EFI/BOOT

# Copy kernel
objcopy --strip-all -O binary target/aarch64-unknown-none-softfloat/release/kernel iso_temp/boot/kernel.bin

# Copy bootloader configs
cp iso/boot/grub/grub.cfg iso_temp/boot/grub/
cp iso/EFI/BOOT/grub.cfg iso_temp/EFI/BOOT/

# Generate ISO
xorriso -as mkisofs -r -J -joliet-long -V "RUSTOS_ARM64" -o rustos-arm64.iso iso_temp/
```

## Installation Methods

### Method 1: USB Installation
1. Write ISO to USB drive:
   ```bash
   dd if=rustos-*.iso of=/dev/sdX bs=4M status=progress
   ```
   (Replace `/dev/sdX` with your USB device)

2. Boot from USB drive
3. Select "RustOS - ARM64 Microkernel" from GRUB menu
4. Run the installer:
   ```bash
   sudo ./install.sh
   ```

### Method 2: Virtual Machine (QEMU)
```bash
# Test in QEMU
make run-iso

# Or manually
qemu-system-aarch64 -machine virt -cpu cortex-a72 -smp 4 -m 2G -cdrom rustos-*.iso -boot d
```

### Method 3: Real Hardware
1. Burn ISO to DVD or write to USB
2. Boot target ARM64 device from optical drive or USB
3. Follow on-screen installation prompts

## Boot Options

The GRUB bootloader provides several boot options:

1. **RustOS - ARM64 Microkernel**: Standard boot mode
2. **RustOS - ARM64 Microkernel (Safe Mode)**: Boot with debug flags and single CPU
3. **RustOS - Desktop Mode with COSMIC**: Boot with full desktop environment
4. **RustOS - Recovery Mode**: Minimal boot for system recovery

## Post-Installation

### First Boot
After installation, RustOS will:
1. Initialize the microkernel
2. Start the COSMIC desktop environment (if selected)
3. Load coreutils and system services
4. Present a login prompt or desktop interface

### System Configuration
- Kernel logs are available via serial console
- COSMIC desktop provides a modern GUI experience
- Core utilities are available for system administration

### Testing and Development
- Use `make run-iso` for quick testing
- QEMU provides full system emulation
- Serial console output available for debugging

## Troubleshooting

### Common Issues

1. **Boot Failures**
   - Ensure target system supports ARM64 architecture
   - Try "Safe Mode" boot option
   - Check UEFI/BIOS settings

2. **Installation Errors**
   - Verify target disk is not in use
   - Ensure sufficient disk space
   - Check file system permissions

3. **Performance Issues**
   - Increase allocated RAM in virtual machines
   - Use "Desktop Mode" only on capable hardware
   - Enable hardware acceleration in QEMU

### Debug Information
- System version: Check `/VERSION` file on ISO
- Build information: Available in kernel boot messages
- Hardware compatibility: Consult ARM64 device documentation

## Development

### Building from Source
```bash
git clone <repository-url>
cd rustos
make setup
make iso
```

### Customization
- Modify `iso/boot/grub/grub.cfg` for boot options
- Edit `iso/install.sh` for installation behavior
- Update kernel configuration in `src/main.rs`

### Testing
```bash
# Test kernel only
make run

# Test with desktop
make run-desktop

# Test ISO image
make run-iso

# Run test suites
make test
```

## License

RustOS is open source software. See LICENSE file for details.

## Support

For issues, questions, or contributions:
- Check existing documentation
- Review kernel logs and error messages
- Submit issues to the project repository