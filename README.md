# RustOS - ARM64 Microkernel

A minimal microkernel written in Rust for ARM64 architecture, designed to work with [uutils/coreutils](https://github.com/uutils/coreutils) to provide a complete operating system environment.

## Features

### Microkernel Architecture
- **Minimal kernel design** - Only essential OS functions in kernel space
- **ARM64 support** - Native ARM64/AArch64 architecture support
- **Memory management** - Basic page allocator and virtual memory
- **Process management** - Task scheduling and process primitives
- **System call interface** - POSIX-compatible system calls
- **File system abstraction** - VFS layer for file operations
- **Inter-process communication** - Pipes and shared memory

### Integration with uutils/coreutils
- **Userspace utilities** - Full coreutils suite running in userspace
- **POSIX compatibility** - Standard UNIX utilities work out of the box
- **Shell integration** - Simple shell for executing commands
- **Pipeline support** - Command chaining with pipes

## Building and Running

### Prerequisites

Install required tools:
```bash
make setup
```

### Building

Build the kernel:
```bash
make kernel           # Debug build
make release         # Release build
```

### Running in QEMU

```bash
make run             # Run debug build
make run-release     # Run release build
```

### Building with uutils/coreutils

```bash
make coreutils       # Build coreutils for ARM64
```

## Architecture

### Kernel Components

- **Boot sequence** (`src/boot.s`) - ARM64 assembly bootstrap
- **Memory management** (`src/memory.rs`) - Page allocator and virtual memory
- **Process management** (`src/process.rs`) - Task scheduling and process control
- **System calls** (`src/syscall.rs`) - Kernel-userspace interface
- **File system** (`src/fs.rs`) - Virtual file system abstraction
- **IPC** (`src/ipc.rs`) - Inter-process communication mechanisms
- **Userspace integration** (`src/userspace.rs`) - ELF loading and coreutils support

### System Calls

Supported POSIX system calls:
- File I/O: `read`, `write`, `open`, `close`
- Process management: `fork`, `execve`, `exit`, `getpid`
- Memory management: `mmap`, `munmap`
- IPC: `pipe`, `dup`, `dup2`

### Memory Layout

```
0x40080000  Kernel text (entry point)
0x50000000  Process memory regions  
0x60000000  Dynamic memory allocation
0x70000000  Shared memory segments
```

## Supported Coreutils

The microkernel supports running these uutils/coreutils programs:

**File operations:**
- `ls` - List directory contents
- `cat` - Display file contents
- `cp` - Copy files
- `mv` - Move/rename files
- `rm` - Remove files
- `mkdir` - Create directories

**Text processing:**
- `grep` - Search text patterns
- `sed` - Stream editor
- `awk` - Text processing language
- `sort` - Sort lines
- `wc` - Word/line/byte count
- `head`/`tail` - Display file portions
- `cut` - Extract columns
- `tr` - Character translation

**Utilities:**
- `echo` - Display text

## Examples

### Running Commands

```bash
# List files
ls /

# Display file contents  
cat /etc/hosts

# Search for patterns
grep "kernel" /proc/version

# Count lines in a file
wc -l /var/log/messages

# Pipe commands together
cat /etc/passwd | grep root | wc -l
```

### Creating and Running Programs

The kernel can load and execute ARM64 ELF binaries compiled with uutils/coreutils.

## Development

### Code Structure

```
src/
├── main.rs          # Kernel entry point
├── boot.s           # ARM64 boot assembly  
├── memory.rs        # Memory management
├── process.rs       # Process management
├── syscall.rs       # System call handling
├── fs.rs            # File system layer
├── ipc.rs           # Inter-process communication
├── userspace.rs     # Userspace integration
└── uart.rs          # Serial I/O
```

### Testing

RustOS includes a comprehensive testing framework designed for Cortex-A72:

```bash
# Run all test suites
make test

# Run specific test suites
make test-kernel         # Kernel unit tests
make test-syscalls       # System call integration tests  
make test-stress         # Stress and stability tests

# Advanced testing
make test-release        # Run tests in release mode
make test-perf          # Performance testing
make test-memory        # Memory-focused tests
make test-ci            # CI/CD automated testing

# Development testing
make test-watch         # Continuous testing on file changes
make test-suite SUITE=kernel_tests  # Run specific test binary

# Integration testing
make test-integration   # Test with uutils/coreutils
```

#### Test Categories

**Kernel Unit Tests (`test-kernel`)**
- Basic kernel functionality
- Memory allocation and management  
- UART communication
- Heap operations
- Page allocation and memory patterns

**System Call Tests (`test-syscalls`)**
- All POSIX system calls (read, write, open, close, etc.)
- File descriptor operations
- Memory management calls (mmap, munmap)
- IPC operations (pipe, dup)
- Error handling and edge cases
- Performance benchmarks

**Stress Tests (`test-stress`)**
- Memory fragmentation and large allocations
- Process creation and scheduling under load
- File system operations at scale
- IPC capacity and throughput testing
- Combined system stress scenarios
- Long-running stability tests

#### Test Framework Features

- **Custom test framework** optimized for bare-metal ARM64
- **Performance counters** using ARM cycle counters
- **Memory testing utilities** (pattern tests, walking ones)
- **System call testing** with direct ARM64 assembly
- **QEMU integration** with automatic exit codes
- **CI/CD support** with GitHub Actions

### Debugging

```bash
make debug              # Start with GDB support
# In another terminal:
gdb-multiarch target/aarch64-unknown-none-softfloat/debug/rustos
(gdb) target remote :1234
```

## Deployment

### Real Hardware

Create a bootable image for ARM64 hardware:

```bash
make image
```

This creates `kernel8.img` that can be loaded on ARM64 devices like Raspberry Pi 4.

### QEMU Options

The kernel is configured to run on QEMU's `virt` machine with:
- Cortex-A57 CPU
- 4 cores
- 1GB RAM
- Serial console output

## Contributing

1. Ensure ARM64 cross-compilation tools are installed
2. Follow Rust formatting guidelines (`make fmt`)
3. Run clippy for linting (`make clippy`)  
4. Test with both kernel and integration tests
5. Update documentation for new features

## License

This project is open source. See individual files for license information.

## References

- [ARM Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest)
- [uutils/coreutils](https://github.com/uutils/coreutils)
- [OSDev Wiki](https://wiki.osdev.org/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)# rustos
