#!/bin/bash

# RustOS ARM64 Installation Script
# This script installs RustOS to a target device

set -e

RUSTOS_VERSION="0.1.0"
KERNEL_FILE="/boot/kernel.bin"
INSTALL_DIR="/mnt/rustos"

echo "============================================"
echo "RustOS ARM64 Installation Script"
echo "Version: $RUSTOS_VERSION"
echo "============================================"
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: This installer must be run as root"
    exit 1
fi

echo "Available disk devices:"
lsblk -d -n -o NAME,SIZE,TYPE | grep disk

echo
read -p "Enter the target device (e.g., sda, nvme0n1): " TARGET_DEVICE

if [ -z "$TARGET_DEVICE" ]; then
    echo "Error: No device specified"
    exit 1
fi

TARGET_PATH="/dev/$TARGET_DEVICE"

if [ ! -b "$TARGET_PATH" ]; then
    echo "Error: Device $TARGET_PATH does not exist"
    exit 1
fi

echo
echo "WARNING: This will erase all data on $TARGET_PATH"
read -p "Are you sure you want to continue? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Installation cancelled"
    exit 0
fi

echo
echo "Partitioning disk..."

# Create partition table
parted -s "$TARGET_PATH" mklabel gpt

# Create EFI system partition (512MB)
parted -s "$TARGET_PATH" mkpart primary fat32 1MiB 513MiB
parted -s "$TARGET_PATH" set 1 esp on

# Create root partition (remaining space)
parted -s "$TARGET_PATH" mkpart primary ext4 513MiB 100%

echo "Formatting partitions..."

# Determine partition naming scheme
if [[ "$TARGET_DEVICE" =~ nvme ]]; then
    EFI_PART="${TARGET_PATH}p1"
    ROOT_PART="${TARGET_PATH}p2"
else
    EFI_PART="${TARGET_PATH}1"
    ROOT_PART="${TARGET_PATH}2"
fi

# Format EFI partition
mkfs.fat -F32 "$EFI_PART"

# Format root partition
mkfs.ext4 "$ROOT_PART"

echo "Mounting partitions..."
mkdir -p "$INSTALL_DIR"
mount "$ROOT_PART" "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/boot/efi"
mount "$EFI_PART" "$INSTALL_DIR/boot/efi"

echo "Installing RustOS..."

# Copy kernel
cp "$KERNEL_FILE" "$INSTALL_DIR/boot/"

# Install GRUB for ARM64
grub-install --target=arm64-efi --efi-directory="$INSTALL_DIR/boot/efi" --boot-directory="$INSTALL_DIR/boot" --removable

# Copy GRUB config
cp /boot/grub/grub.cfg "$INSTALL_DIR/boot/grub/"

# Create basic filesystem structure
mkdir -p "$INSTALL_DIR"/{bin,sbin,etc,var,tmp,home,root}
chmod 1777 "$INSTALL_DIR/tmp"

# Create a simple init script
cat > "$INSTALL_DIR/init" << 'EOF'
#!/bin/bash
echo "Starting RustOS userspace..."
exec /boot/kernel.bin
EOF
chmod +x "$INSTALL_DIR/init"

echo "Unmounting partitions..."
umount "$INSTALL_DIR/boot/efi"
umount "$INSTALL_DIR"
rmdir "$INSTALL_DIR"

echo
echo "============================================"
echo "RustOS installation completed successfully!"
echo "============================================"
echo
echo "You can now boot from $TARGET_PATH"
echo "Remove the installation media and reboot."