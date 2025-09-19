#![no_std]
#![no_main]

extern crate rustos;

use core::arch::asm;
use core::panic::PanicInfo;

use rustos::fs::{self, OpenFlags};
use rustos::{ipc, memory, panic as panic_runtime, process, syscall, uart, userspace};

type TestFn = fn();

const TESTS: &[TestFn] = &[
    memory_allocation_is_page_aligned,
    memory_allocation_grows_monotonically,
    process_creation_returns_distinct_pids,
    file_round_trip_preserves_payload,
    pipe_transports_data_between_ends,
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

    panic_runtime::set_handler(kernel_test_panic_handler);

    run_tests();
    exit_qemu(0);
}

fn run_tests() {
    rustos::println!("Kernel test suite started: {} cases", TESTS.len());
    for test in TESTS {
        test();
    }
    rustos::println!("Kernel test suite completed");
}

fn memory_allocation_is_page_aligned() {
    let addr = memory::allocate_pages(4096).expect("allocate first page");
    assert_eq!(addr & 0xFFF, 0, "allocation should be 4K aligned");
}

fn memory_allocation_grows_monotonically() {
    let first = memory::allocate_pages(4096).expect("first allocation");
    let second = memory::allocate_pages(4096).expect("second allocation");
    assert!(second >= first + 4096, "subsequent allocation should not overlap");
}

fn process_creation_returns_distinct_pids() {
    let pid_a = process::create_process(0x4000_0000, 4096).expect("create first process");
    let pid_b = process::create_process(0x4001_0000, 4096).expect("create second process");
    assert!(pid_b > pid_a, "process IDs should monotonically increase");
}

fn file_round_trip_preserves_payload() {
    let path = "/tmp/kernel-test.txt";
    let _ = fs::remove_file(path);
    let write_flags = (OpenFlags::O_CREAT | OpenFlags::O_WRONLY | OpenFlags::O_TRUNC).bits();
    let fd_write = fs::open(path, write_flags, 0).expect("open file for write");
    let payload = b"rustos-kernel";
    fs::write(fd_write, payload).expect("write payload");
    fs::close(fd_write).expect("close write fd");

    let fd_read = fs::open(path, OpenFlags::O_RDONLY.bits(), 0).expect("reopen file for read");
    let mut buffer = [0u8; 32];
    let bytes = fs::read(fd_read, &mut buffer).expect("read payload");
    fs::close(fd_read).expect("close read fd");
    assert_eq!(bytes, payload.len());
    assert_eq!(&buffer[..payload.len()], payload);
}

fn pipe_transports_data_between_ends() {
    let (read_fd, write_fd) = ipc::create_pipe().expect("create pipe");
    let payload = b"hello-ipc";
    let written = fs::write(write_fd, payload).expect("write to pipe");
    assert_eq!(written, payload.len());

    let mut buffer = [0u8; 16];
    let read = fs::read(read_fd, &mut buffer).expect("read from pipe");
    assert_eq!(read, payload.len());
    assert_eq!(&buffer[..read], payload);

    fs::close(read_fd).expect("close read fd");
    fs::close(write_fd).expect("close write fd");
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

fn kernel_test_panic_handler(info: &PanicInfo) -> ! {
    rustos::println!("Kernel test panic: {}", info);
    exit_qemu(1);
}

core::arch::global_asm!(include_str!("../boot.s"));
