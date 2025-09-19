use core::arch::asm;
use crate::process;
use crate::fs;
use crate::ipc;
use crate::println;

// System call numbers
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_EXIT: u64 = 60;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_GETPID: u64 = 39;
pub const SYS_PIPE: u64 = 22;
pub const SYS_DUP: u64 = 32;
pub const SYS_DUP2: u64 = 33;

pub fn init() {
    // Set up exception vector table for system calls
    unsafe {
        setup_exception_vector();
    }
}

unsafe fn setup_exception_vector() {
    // ARM64 exception vector setup
    extern "C" {
        fn exception_vector_table();
    }
    
    asm!(
        "msr vbar_el1, {}",
        in(reg) exception_vector_table as *const () as u64
    );
}

#[no_mangle]
pub extern "C" fn syscall_handler(
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    match syscall_num {
        SYS_READ => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        SYS_WRITE => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        SYS_OPEN => sys_open(arg1 as *const u8, arg2 as i32, arg3 as u32),
        SYS_CLOSE => sys_close(arg1 as i32),
        SYS_EXIT => {
            process::sys_exit(arg1 as i32);
        }
        SYS_FORK => process::sys_fork() as u64,
        SYS_EXECVE => {
            match process::sys_exec(arg1) {
                Ok(_) => 0,
                Err(_) => u64::MAX, // -1 in two's complement
            }
        }
        SYS_GETPID => {
            process::get_current_pid().unwrap_or(0) as u64
        }
        SYS_PIPE => sys_pipe(arg1 as *mut [i32; 2]),
        SYS_DUP => sys_dup(arg1 as i32),
        SYS_DUP2 => sys_dup2(arg1 as i32, arg2 as i32),
        SYS_MMAP => sys_mmap(arg1, arg2 as usize, arg3 as i32, arg4 as i32, arg5 as i32, arg6 as i64),
        SYS_MUNMAP => sys_munmap(arg1, arg2 as usize),
        _ => {
            println!("Unknown system call: {}", syscall_num);
            u64::MAX // Return -1 for unknown syscalls
        }
    }
}

// File I/O system calls
fn sys_read(fd: i32, buf: *mut u8, count: usize) -> u64 {
    match fs::read(fd, unsafe { core::slice::from_raw_parts_mut(buf, count) }) {
        Ok(bytes_read) => bytes_read as u64,
        Err(_) => u64::MAX,
    }
}

fn sys_write(fd: i32, buf: *const u8, count: usize) -> u64 {
    match fs::write(fd, unsafe { core::slice::from_raw_parts(buf, count) }) {
        Ok(bytes_written) => bytes_written as u64,
        Err(_) => u64::MAX,
    }
}

fn sys_open(pathname: *const u8, flags: i32, mode: u32) -> u64 {
    // Convert C string to Rust string
    let path_str = unsafe {
        let mut len = 0;
        let mut ptr = pathname;
        while *ptr != 0 {
            len += 1;
            ptr = ptr.add(1);
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(pathname, len))
    };
    
    match fs::open(path_str, flags, mode) {
        Ok(fd) => fd as u64,
        Err(_) => u64::MAX,
    }
}

fn sys_close(fd: i32) -> u64 {
    match fs::close(fd) {
        Ok(_) => 0,
        Err(_) => u64::MAX,
    }
}

// IPC system calls
fn sys_pipe(pipefd: *mut [i32; 2]) -> u64 {
    match ipc::create_pipe() {
        Ok((read_fd, write_fd)) => {
            unsafe {
                (*pipefd)[0] = read_fd;
                (*pipefd)[1] = write_fd;
            }
            0
        }
        Err(_) => u64::MAX,
    }
}

fn sys_dup(fd: i32) -> u64 {
    match fs::duplicate_fd(fd) {
        Ok(new_fd) => new_fd as u64,
        Err(_) => u64::MAX,
    }
}

fn sys_dup2(oldfd: i32, newfd: i32) -> u64 {
    match fs::duplicate_fd_to(oldfd, newfd) {
        Ok(fd) => fd as u64,
        Err(_) => u64::MAX,
    }
}

// Memory management system calls
fn sys_mmap(addr: u64, length: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> u64 {
    // Simple memory mapping implementation
    // In a real kernel, this would handle virtual memory mapping
    match crate::memory::allocate_pages(length) {
        Ok(allocated_addr) => allocated_addr,
        Err(_) => u64::MAX,
    }
}

fn sys_munmap(addr: u64, length: usize) -> u64 {
    // Memory unmapping implementation
    match crate::memory::deallocate_pages(addr, length) {
        Ok(_) => 0,
        Err(_) => u64::MAX,
    }
}

// Exception vector table in assembly
core::arch::global_asm!(r#"
.align 11
exception_vector_table:
    // Current EL with SP0
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .
    
    // Current EL with SPx
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .
    
    // Lower EL using AArch64
    .align 7
    b handle_sync_exception
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .
    
    // Lower EL using AArch32
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .
    .align 7
    b .

handle_sync_exception:
    // Save registers
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x30, xzr, [sp, #-16]!
    
    // Check if this is a system call (SVC instruction)
    mrs x9, esr_el1
    and x9, x9, #0x3f000000
    mov x10, #0x15000000  // SVC exception code
    cmp x9, x10
    b.ne not_syscall
    
    // Call syscall handler
    // x8 contains syscall number, x0-x5 contain arguments
    mov x9, x8  // Move syscall number to x9
    bl syscall_handler
    
    // Result is in x0, restore registers
    ldp x30, xzr, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x1, xzr, [sp], #16  // Skip x1, keep x0 (return value)
    
    eret

not_syscall:
    // Handle other exceptions
    ldp x30, xzr, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    
    eret
"#);