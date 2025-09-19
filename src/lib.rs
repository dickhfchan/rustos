#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_framework::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod memory;
pub mod uart;
pub mod process;
pub mod syscall;
pub mod fs;
pub mod ipc;
pub mod userspace;
pub mod test_framework;

// Re-export macros for tests (commented out to avoid redefinition)
// pub use crate::{print, println, assert_eq_test, assert_test, assert_ne_test};

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    // Initialize kernel for tests
    memory::init();
    uart::init();
    process::init();
    syscall::init();
    fs::init();
    ipc::init();
    userspace::init();
    
    test_main();
    
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Test panic: {}", info);
    loop {}
}