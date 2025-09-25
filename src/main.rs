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
mod wayland;
mod graphics;
mod input;
mod cosmic;

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
    
    // Initialize COSMIC desktop environment
    if let Err(_) = cosmic::cosmic_init(1920, 1080) {
        println!("Warning: COSMIC desktop initialization failed, continuing without window manager");
    } else {
        println!("COSMIC desktop environment initialized");
        
        // Create some demo windows
        if let Ok(window1) = cosmic::cosmic_create_window("Terminal", 800, 600) {
            println!("Created demo terminal window: {}", window1);
        }
        
        if let Ok(window2) = cosmic::cosmic_create_window("File Manager", 600, 400) {
            println!("Created demo file manager window: {}", window2);
        }
        
        // Show a welcome notification
        let _ = cosmic::cosmic_show_notification(
            "Welcome to RustOS".into(),
            "COSMIC desktop environment is now running!".into(),
            cosmic::NotificationUrgency::Normal
        );
    }
    
    println!("Microkernel initialization complete");
    println!("Ready to load userspace applications with window manager");
    
    // Demonstrate coreutils functionality
    println!("\n=== Coreutils Demo ===");
    let _ = coreutils::execute_command("echo", &["Hello", "from", "RustOS!"]);
    let _ = coreutils::execute_command("pwd", &[]);
    let _ = coreutils::execute_command("ls", &["/"]);
    let _ = coreutils::execute_command("help", &[]);
    println!("=== Demo Complete ===\n");
    
    // Main kernel loop with COSMIC integration
    loop {
        // Process COSMIC events if active
        if cosmic::cosmic_is_session_active() {
            let _ = cosmic::cosmic_process_events();
            let _ = cosmic::cosmic_render_frame();
        }
        
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