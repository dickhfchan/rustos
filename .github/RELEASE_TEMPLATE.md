# RustOS ARM64 Release

## 🚀 What's New in This Release

- ARM64 microkernel with enhanced performance
- COSMIC desktop environment integration
- Improved hardware compatibility
- Updated system components

## 📦 Download

### ISO Images
- **rustos-aarch64-unknown-none-softfloat-YYYYMMDD.iso** - Complete installable system

### Checksums
```
SHA256: [automatically generated]
MD5: [automatically generated]
```

## 🖥️ System Requirements

### Minimum Requirements
- **Processor**: ARM64 (AArch64) compatible CPU
- **Memory**: 2 GB RAM
- **Storage**: 4 GB available space
- **Boot**: UEFI or Legacy BIOS support

### Recommended Requirements
- **Processor**: ARM64 Cortex-A72 or newer
- **Memory**: 4 GB RAM or more
- **Storage**: 16 GB or more for full desktop experience
- **Graphics**: Framebuffer-compatible graphics card
- **Peripherals**: USB keyboard and mouse

## 🔧 Installation Instructions

### Method 1: USB Installation
1. Download the ISO image from the Assets section below
2. Write to USB drive:
   ```bash
   sudo dd if=rustos-*.iso of=/dev/sdX bs=4M status=progress conv=fsync
   ```
   Replace `/dev/sdX` with your USB device (e.g., `/dev/sdb`)
3. Boot target system from USB
4. Select boot option from GRUB menu
5. Run installer: `sudo ./install.sh`

### Method 2: Virtual Machine Testing
```bash
# QEMU (recommended for testing)
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a72 \
  -smp 4 \
  -m 2G \
  -cdrom rustos-*.iso \
  -boot d \
  -display gtk

# Or use the included shortcut
make run-iso
```

### Method 3: Real Hardware
1. Burn ISO to DVD or write to USB
2. Configure target device to boot from optical/USB
3. Select appropriate boot mode in GRUB
4. Follow installation prompts

## 🌟 Features

### Core System
- ✅ ARM64 microkernel architecture
- ✅ Memory management and process isolation
- ✅ System call interface
- ✅ Hardware abstraction layer

### Desktop Environment
- ✅ COSMIC desktop integration
- ✅ Wayland protocol support
- ✅ Modern windowing system
- ✅ Input device handling

### Development Tools
- ✅ Rust-based system programming
- ✅ Core utilities integration
- ✅ Comprehensive test suite
- ✅ Debug and development modes

## 🧪 Testing and Validation

This release has been tested on:
- ✅ QEMU ARM64 virtual machines
- ✅ Raspberry Pi 4 Model B (ARM64)
- ✅ Generic ARM64 development boards
- ✅ UEFI and Legacy boot modes

## 🐛 Known Issues

- Graphics acceleration limited to basic framebuffer
- Network stack in development
- Audio support not yet implemented
- Limited hardware driver selection

## 📚 Documentation

- **Installation Guide**: See `ISO-README.md` in the repository
- **Development Guide**: Check `README.md` for build instructions
- **API Documentation**: Available via `cargo doc`
- **System Information**: View `VERSION` file in ISO

## 🤝 Contributing

We welcome contributions! Please see:
- [Contributing Guidelines](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Development Setup](README.md#development)

## 📞 Support

- **Issues**: Report bugs via GitHub Issues
- **Discussions**: Join GitHub Discussions
- **Documentation**: Check wiki and README files
- **Community**: Follow development updates

## 🔍 Verification

To verify the ISO integrity:
```bash
# Check file format
file rustos-*.iso

# Mount and inspect (Linux)
mkdir /tmp/rustos-mount
sudo mount -o loop rustos-*.iso /tmp/rustos-mount
ls -la /tmp/rustos-mount
sudo umount /tmp/rustos-mount

# Use included verification script
./test-iso.sh
```

## 📊 Build Information

- **Build Date**: [Automatically set by CI]
- **Commit**: [GitHub commit hash]
- **Rust Version**: nightly
- **Target**: aarch64-unknown-none-softfloat
- **Builder**: GitHub Actions

---

**Note**: This is experimental software. Please backup important data before installation and use at your own risk.