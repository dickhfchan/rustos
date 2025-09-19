#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
mod memory;
mod uart;
mod process;
mod syscall;
mod fs;
mod ipc;
mod userspace;
mod coreutils;

#[cfg(test)]
mod test_framework;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    println!("RustOS ARM64 Microkernel v0.1.0");
    
    // Initialize memory management
    memory::init();
    println!("Memory management initialized");
    
    // Initialize UART for communication
    uart::init();
    println!("UART initialized");
    
    // Initialize process management
    process::init();
    println!("Process management initialized");
    
    // Initialize system call interface
    syscall::init();
    println!("System call interface initialized");
    
    // Initialize file system abstraction
    fs::init();
    println!("File system abstraction initialized");
    
    // Initialize IPC mechanisms
    ipc::init();
    println!("IPC mechanisms initialized");
    
    // Initialize userspace integration
    userspace::init();
    println!("Userspace integration initialized");
    
    // Initialize coreutils
    coreutils::init();
    println!("Coreutils initialized");
    
    println!("Microkernel initialization complete");
    println!("Ready to load userspace applications");
    
    // Demonstrate coreutils functionality
    println!("\n=== Coreutils Demo ===");
    let _ = coreutils::execute_command("echo", &["Hello", "from", "RustOS!"]);
    let _ = coreutils::execute_command("pwd", &[]);
    let _ = coreutils::execute_command("ls", &["/"]);
    let _ = coreutils::execute_command("help", &[]);
    println!("=== Demo Complete ===\n");
    
    // Main kernel loop
    loop {
        // Process scheduling and system calls
        process::schedule();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel panic: {}", info);
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    println!("RustOS Test Mode - ARM64 Microkernel");
    
    // Initialize all systems for testing
    memory::init();
    uart::init();
    process::init();
    syscall::init();
    fs::init();
    ipc::init();
    userspace::init();
    
    println!("All systems initialized for testing");
    
    // Run the test main function
    test_main();
    
    loop {}
}

// Assembly boot code
core::arch::global_asm!(include_str!("boot.s"));