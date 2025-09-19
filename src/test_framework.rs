use core::panic::PanicInfo;
use core::fmt::Write;
use crate::{print, println};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    Passed,
    Failed,
    Skipped,
}

pub struct TestRunner {
    test_count: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner {
            test_count: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
        }
    }
    
    pub fn run_test<F>(&mut self, name: &str, test_fn: F) -> TestResult
    where
        F: FnOnce() -> TestResult,
    {
        self.test_count += 1;
        print!("Running test: {} ... ", name);
        
        let result = test_fn();
        
        match result {
            TestResult::Passed => {
                self.passed += 1;
                println!("PASSED");
            }
            TestResult::Failed => {
                self.failed += 1;
                println!("FAILED");
            }
            TestResult::Skipped => {
                self.skipped += 1;
                println!("SKIPPED");
            }
        }
        
        result
    }
    
    pub fn summary(&self) {
        println!("\n=== Test Summary ===");
        println!("Total tests: {}", self.test_count);
        println!("Passed: {}", self.passed);
        println!("Failed: {}", self.failed);
        println!("Skipped: {}", self.skipped);
        
        if self.failed == 0 {
            println!("All tests passed!");
        } else {
            println!("Some tests failed!");
        }
    }
    
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// Test assertion macros
#[macro_export]
macro_rules! assert_eq_test {
    ($left:expr, $right:expr) => {
        if $left != $right {
            println!("Assertion failed: {} != {}", stringify!($left), stringify!($right));
            return TestResult::Failed;
        }
    };
}

#[macro_export]
macro_rules! assert_test {
    ($condition:expr) => {
        if !$condition {
            println!("Assertion failed: {}", stringify!($condition));
            return TestResult::Failed;
        }
    };
}

#[macro_export]
macro_rules! assert_ne_test {
    ($left:expr, $right:expr) => {
        if $left == $right {
            println!("Assertion failed: {} == {}", stringify!($left), stringify!($right));
            return TestResult::Failed;
        }
    };
}

// Test cases collection
pub trait TestCase {
    fn run(&self) -> TestResult;
    fn name(&self) -> &'static str;
}

pub struct TestSuite {
    name: &'static str,
    tests: &'static [&'static dyn TestCase],
}

impl TestSuite {
    pub fn new(name: &'static str, tests: &'static [&'static dyn TestCase]) -> Self {
        TestSuite { name, tests }
    }
    
    pub fn run(&self) -> bool {
        println!("\n=== Running Test Suite: {} ===", self.name);
        let mut runner = TestRunner::new();
        
        for test in self.tests {
            runner.run_test(test.name(), || test.run());
        }
        
        runner.summary();
        runner.all_passed()
    }
}

// Kernel panic handler for tests
#[cfg(test)]
#[panic_handler]
fn test_panic_handler(info: &PanicInfo) -> ! {
    println!("Test panic: {}", info);
    
    // Exit QEMU with error code
    unsafe {
        // QEMU exit
        core::arch::asm!(
            "mov x0, #1",    // Exit code 1 (failure)
            "hlt #0xf000"    // QEMU semihosting exit
        );
    }
    
    loop {}
}

// Test-specific UART output
pub fn test_println(args: core::fmt::Arguments) {
    use crate::uart::_print;
    _print(args);
}

#[macro_export]
macro_rules! test_println {
    ($($arg:tt)*) => ($crate::test_framework::test_println(format_args!($($arg)*)));
}

// Memory testing utilities
pub fn test_memory_pattern(addr: *mut u8, size: usize, pattern: u8) -> bool {
    unsafe {
        // Write pattern
        for i in 0..size {
            addr.add(i).write_volatile(pattern);
        }
        
        // Read and verify pattern
        for i in 0..size {
            if addr.add(i).read_volatile() != pattern {
                return false;
            }
        }
    }
    true
}

pub fn test_memory_walking_ones(addr: *mut u32, count: usize) -> bool {
    unsafe {
        for i in 0..32 {
            let pattern = 1u32 << i;
            
            // Write walking ones pattern
            for j in 0..count {
                addr.add(j).write_volatile(pattern);
            }
            
            // Verify pattern
            for j in 0..count {
                if addr.add(j).read_volatile() != pattern {
                    return false;
                }
            }
        }
    }
    true
}

// System call testing utilities
pub fn test_syscall_with_args(syscall_num: u64, args: &[u64]) -> u64 {
    unsafe {
        match args.len() {
            0 => {
                let result: u64;
                core::arch::asm!(
                    "mov x8, {}",
                    "svc #0",
                    "mov {}, x0",
                    in(reg) syscall_num,
                    out(reg) result
                );
                result
            }
            1 => {
                let result: u64;
                core::arch::asm!(
                    "mov x8, {}",
                    "mov x0, {}",
                    "svc #0",
                    "mov {}, x0",
                    in(reg) syscall_num,
                    in(reg) args[0],
                    out(reg) result
                );
                result
            }
            2 => {
                let result: u64;
                core::arch::asm!(
                    "mov x8, {}",
                    "mov x0, {}",
                    "mov x1, {}",
                    "svc #0",
                    "mov {}, x0",
                    in(reg) syscall_num,
                    in(reg) args[0],
                    in(reg) args[1],
                    out(reg) result
                );
                result
            }
            3 => {
                let result: u64;
                core::arch::asm!(
                    "mov x8, {}",
                    "mov x0, {}",
                    "mov x1, {}",
                    "mov x2, {}",
                    "svc #0",
                    "mov {}, x0",
                    in(reg) syscall_num,
                    in(reg) args[0],
                    in(reg) args[1],
                    in(reg) args[2],
                    out(reg) result
                );
                result
            }
            _ => panic!("Too many syscall arguments"),
        }
    }
}

// Performance testing utilities
pub struct PerformanceTimer {
    start_cycles: u64,
}

impl PerformanceTimer {
    pub fn new() -> Self {
        PerformanceTimer {
            start_cycles: Self::read_cycles(),
        }
    }
    
    pub fn elapsed_cycles(&self) -> u64 {
        Self::read_cycles() - self.start_cycles
    }
    
    fn read_cycles() -> u64 {
        unsafe {
            let cycles: u64;
            core::arch::asm!(
                "mrs {}, pmccntr_el0",
                out(reg) cycles
            );
            cycles
        }
    }
    
    pub fn enable_cycle_counter() {
        unsafe {
            // Enable user-mode access to cycle counter
            core::arch::asm!(
                "msr pmuserenr_el0, #1",
                "msr pmcntenset_el0, #0x80000000"
            );
        }
    }
}