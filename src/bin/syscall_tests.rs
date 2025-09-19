#![no_std]
#![no_main]

extern crate rustos;

use core::arch::asm;
use core::panic::PanicInfo;

use rustos::fs::{self, OpenFlags};
use rustos::{ipc, memory, panic as panic_runtime, process, syscall, uart, userspace};

type TestFn = fn();

const TESTS: &[TestFn] = &[
    unknown_syscall_returns_error,
    sys_open_write_read_file_via_handler,
    sys_pipe_roundtrip_via_handler,
];

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    uart::init();
    memory::init();
    process::init();
    syscall::init();
    fs::init();
    ipc::init();
    userspace::init();

    panic_runtime::set_handler(syscall_test_panic_handler);

    run_tests();
    exit_qemu(0);
}

fn run_tests() {
    rustos::println!("Syscall test suite started: {} cases", TESTS.len());
    for test in TESTS {
        test();
    }
    rustos::println!("Syscall test suite completed");
}

fn unknown_syscall_returns_error() {
    let result = syscall::syscall_handler(9_999, 0, 0, 0, 0, 0, 0);
    assert_eq!(result, u64::MAX);
}

fn sys_open_write_read_file_via_handler() {
    let path = b"/tmp/syscall.txt\0";
    let _ = fs::remove_file("/tmp/syscall.txt");

    let write_flags = (OpenFlags::O_CREAT | OpenFlags::O_WRONLY | OpenFlags::O_TRUNC).bits() as u64;
    let fd = syscall::syscall_handler(
        syscall::SYS_OPEN,
        path.as_ptr() as u64,
        write_flags,
        0,
        0,
        0,
        0,
    ) as i32;
    assert!(fd >= 0);

    let payload = b"syscall-data";
    let written = syscall::syscall_handler(
        syscall::SYS_WRITE,
        fd as u64,
        payload.as_ptr() as u64,
        payload.len() as u64,
        0,
        0,
        0,
    );
    assert_eq!(written, payload.len() as u64);

    syscall::syscall_handler(syscall::SYS_CLOSE, fd as u64, 0, 0, 0, 0, 0);

    let read_fd = syscall::syscall_handler(
        syscall::SYS_OPEN,
        path.as_ptr() as u64,
        OpenFlags::O_RDONLY.bits() as u64,
        0,
        0,
        0,
        0,
    ) as i32;
    assert!(read_fd >= 0);

    let mut buffer = [0u8; 32];
    let read = syscall::syscall_handler(
        syscall::SYS_READ,
        read_fd as u64,
        buffer.as_mut_ptr() as u64,
        buffer.len() as u64,
        0,
        0,
        0,
    );
    syscall::syscall_handler(syscall::SYS_CLOSE, read_fd as u64, 0, 0, 0, 0, 0);

    assert_eq!(read, payload.len() as u64);
    assert_eq!(&buffer[..payload.len()], payload);
}

fn sys_pipe_roundtrip_via_handler() {
    let mut pipe_fd = [0i32; 2];
    let result = syscall::syscall_handler(
        syscall::SYS_PIPE,
        (&mut pipe_fd) as *mut _ as u64,
        0,
        0,
        0,
        0,
        0,
    );
    assert_eq!(result, 0);

    let payload = b"ipc-roundtrip";
    let written = syscall::syscall_handler(
        syscall::SYS_WRITE,
        pipe_fd[1] as u64,
        payload.as_ptr() as u64,
        payload.len() as u64,
        0,
        0,
        0,
    );
    assert_eq!(written, payload.len() as u64);

    let mut buffer = [0u8; 32];
    let read = syscall::syscall_handler(
        syscall::SYS_READ,
        pipe_fd[0] as u64,
        buffer.as_mut_ptr() as u64,
        buffer.len() as u64,
        0,
        0,
        0,
    );
    assert_eq!(read, payload.len() as u64);
    assert_eq!(&buffer[..payload.len()], payload);

    syscall::syscall_handler(syscall::SYS_CLOSE, pipe_fd[0] as u64, 0, 0, 0, 0, 0);
    syscall::syscall_handler(syscall::SYS_CLOSE, pipe_fd[1] as u64, 0, 0, 0, 0, 0);
}

fn exit_qemu(code: u64) -> ! {
    unsafe {
        asm!(
            "mov x0, {fn_id}",
            "mov x1, {code}",
            "mov x2, xzr",
            "mov x3, xzr",
            "hvc #0",
            fn_id = in(reg) 0x84000008u64,
            code = in(reg) code,
            options(noreturn)
        );
    }
}

fn syscall_test_panic_handler(info: &PanicInfo) -> ! {
    rustos::println!("Syscall test panic: {}", info);
    exit_qemu(1);
}

core::arch::global_asm!(include_str!("../boot.s"));
