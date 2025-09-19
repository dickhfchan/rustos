#![no_std]
#![no_main]

extern crate rustos;

use core::arch::asm;
use core::panic::PanicInfo;

use rustos::fs::{self, OpenFlags};
use rustos::{ipc, memory, panic as panic_runtime, process, syscall, uart, userspace};

type TestFn = fn();

const TESTS: &[TestFn] = &[
    bulk_page_allocations_remain_unique,
    sustained_pipe_throughput_succeeds,
    file_create_write_read_remove_cycles,
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

    panic_runtime::set_handler(stress_test_panic_handler);

    run_tests();
    exit_qemu(0);
}

fn run_tests() {
    rustos::println!("Stress test suite started: {} cases", TESTS.len());
    for test in TESTS {
        test();
    }
    rustos::println!("Stress test suite completed");
}

fn bulk_page_allocations_remain_unique() {
    const ALLOCATIONS: usize = 64;
    let mut previous = 0;
    for _ in 0..ALLOCATIONS {
        let addr = memory::allocate_pages(4096).expect("allocate stress page");
        if previous != 0 {
            assert!(addr >= previous + 4096, "addresses must grow without overlap");
        }
        previous = addr;
    }
}

fn sustained_pipe_throughput_succeeds() {
    const ITERATIONS: usize = 32;
    let (read_fd, write_fd) = ipc::create_pipe().expect("create stress pipe");
    let mut buffer = [0u8; 64];
    for i in 0..ITERATIONS {
        let payload = [i as u8; 48];
        let written = fs::write(write_fd, &payload).expect("write to stress pipe");
        assert_eq!(written, payload.len());

        let read = fs::read(read_fd, &mut buffer).expect("read from stress pipe");
        assert_eq!(read, payload.len());
        assert_eq!(&buffer[..read], &payload);
    }
    fs::close(read_fd).expect("close pipe read fd");
    fs::close(write_fd).expect("close pipe write fd");
}

fn file_create_write_read_remove_cycles() {
    const ITERATIONS: usize = 24;
    for i in 0..ITERATIONS {
        let mut path_buf = [0u8; 32];
        let path = format_path(i, &mut path_buf);
        let write_flags = (OpenFlags::O_CREAT | OpenFlags::O_WRONLY | OpenFlags::O_TRUNC).bits();
        let fd = fs::open(path, write_flags, 0).expect("open stress file for write");
        let payload = [i as u8; 32];
        fs::write(fd, &payload).expect("write stress payload");
        fs::close(fd).expect("close write fd");

        let fd_read = fs::open(path, OpenFlags::O_RDONLY.bits(), 0).expect("open stress file for read");
        let mut buffer = [0u8; 32];
        let read = fs::read(fd_read, &mut buffer).expect("read stress payload");
        fs::close(fd_read).expect("close read fd");
        assert_eq!(read, payload.len());
        assert_eq!(buffer, payload);

        fs::remove_file(path).expect("remove stress file");
    }
}

fn format_path(iteration: usize, buffer: &mut [u8; 32]) -> &str {
    let template = b"/tmp/stress";
    let mut len = 0;
    for &byte in template {
        buffer[len] = byte;
        len += 1;
    }
    buffer[len] = ((iteration / 10) % 10) as u8 + b'0';
    len += 1;
    buffer[len] = (iteration % 10) as u8 + b'0';
    len += 1;
    core::str::from_utf8(&buffer[..len]).expect("valid utf8 path")
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

fn stress_test_panic_handler(info: &PanicInfo) -> ! {
    rustos::println!("Stress test panic: {}", info);
    exit_qemu(1);
}

core::arch::global_asm!(include_str!("../boot.s"));
