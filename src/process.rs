#![allow(dead_code)]

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug)]
pub struct Process {
    pub pid: u32,
    pub state: ProcessState,
    pub priority: u8,
    pub stack_pointer: u64,
    pub page_table: u64,
    pub registers: [u64; 31], // ARM64 general purpose registers
    pub entry_point: u64,
    pub memory_regions: Vec<MemoryRegion>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub permissions: MemoryPermissions,
}

bitflags::bitflags! {
    #[derive(Debug, Clone)]
    pub struct MemoryPermissions: u8 {
        const READ = 1;
        const WRITE = 2;
        const EXECUTE = 4;
    }
}

pub struct ProcessManager {
    processes: Vec<Process>,
    ready_queue: VecDeque<u32>,
    current_pid: Option<u32>,
    next_pid: u32,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: Vec::new(),
            ready_queue: VecDeque::new(),
            current_pid: None,
            next_pid: 1,
        }
    }
    
    pub fn create_process(&mut self, entry_point: u64, stack_size: u64) -> Result<u32, &'static str> {
        let pid = self.next_pid;
        self.next_pid += 1;
        
        // Allocate stack
        let stack_start = self.allocate_memory(stack_size)?;
        let stack_pointer = stack_start + stack_size;
        
        // Create page table for the process
        let page_table = self.create_page_table()?;
        
        let process = Process {
            pid,
            state: ProcessState::Ready,
            priority: 128, // Default priority
            stack_pointer,
            page_table,
            registers: [0; 31],
            entry_point,
            memory_regions: Vec::new(),
        };
        
        self.processes.push(process);
        self.ready_queue.push_back(pid);
        
        Ok(pid)
    }
    
    pub fn schedule(&mut self) -> Option<u32> {
        if let Some(next_pid) = self.ready_queue.pop_front() {
            // Mark current process as ready if it's still running
            if let Some(current_pid) = self.current_pid {
                if let Some(current_process) = self.get_process_mut(current_pid) {
                    if current_process.state == ProcessState::Running {
                        current_process.state = ProcessState::Ready;
                        self.ready_queue.push_back(current_pid);
                    }
                }
            }
            
            // Set new process as running
            if let Some(next_process) = self.get_process_mut(next_pid) {
                next_process.state = ProcessState::Running;
                self.current_pid = Some(next_pid);
                return Some(next_pid);
            }
        }
        
        self.current_pid
    }
    
    pub fn get_process(&self, pid: u32) -> Option<&Process> {
        self.processes.iter().find(|p| p.pid == pid)
    }
    
    pub fn get_process_mut(&mut self, pid: u32) -> Option<&mut Process> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }
    
    pub fn terminate_process(&mut self, pid: u32) -> Result<(), &'static str> {
        if let Some(process) = self.get_process_mut(pid) {
            process.state = ProcessState::Terminated;
            
            // Remove from ready queue if present
            self.ready_queue.retain(|&p| p != pid);
            
            // If it's the current process, clear current_pid
            if self.current_pid == Some(pid) {
                self.current_pid = None;
            }
            
            Ok(())
        } else {
            Err("Process not found")
        }
    }
    
    fn allocate_memory(&self, size: u64) -> Result<u64, &'static str> {
        // Simple memory allocation - in a real kernel this would be more sophisticated
        // For now, just return a fixed address offset
        static mut NEXT_ADDR: u64 = 0x50000000;
        unsafe {
            let addr = NEXT_ADDR;
            NEXT_ADDR += size.next_power_of_two();
            Ok(addr)
        }
    }
    
    fn create_page_table(&self) -> Result<u64, &'static str> {
        // Create a new page table for the process
        // This is simplified - real implementation would set up proper page tables
        self.allocate_memory(4096) // One page for page table
    }
}

lazy_static! {
    static ref PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());
}

pub fn init() {
    // Process manager is initialized statically
}

pub fn create_process(entry_point: u64, stack_size: u64) -> Result<u32, &'static str> {
    PROCESS_MANAGER.lock().create_process(entry_point, stack_size)
}

pub fn schedule() {
    let mut manager = PROCESS_MANAGER.lock();
    if let Some(pid) = manager.schedule() {
        if let Some(process) = manager.get_process(pid) {
            // Context switch to the selected process
            context_switch(process);
        }
    }
}

pub fn terminate_current_process() -> Result<(), &'static str> {
    let mut manager = PROCESS_MANAGER.lock();
    if let Some(current_pid) = manager.current_pid {
        manager.terminate_process(current_pid)
    } else {
        Err("No current process")
    }
}

pub fn get_current_pid() -> Option<u32> {
    PROCESS_MANAGER.lock().current_pid
}

fn context_switch(process: &Process) {
    unsafe {
        // Switch page table
        asm!(
            "msr ttbr0_el1, {}",
            "tlbi vmalle1is",
            "dsb sy",
            "isb",
            in(reg) process.page_table
        );
        
        // This is where we would restore registers and jump to user space
        // For now, we'll just return to continue kernel execution
    }
}

// System call handlers for process management
pub fn sys_fork() -> u32 {
    // Fork implementation would go here
    0
}

pub fn sys_exec(entry_point: u64) -> Result<(), &'static str> {
    let mut manager = PROCESS_MANAGER.lock();
    if let Some(current_pid) = manager.current_pid {
        if let Some(process) = manager.get_process_mut(current_pid) {
            process.entry_point = entry_point;
            // Reset registers and stack
            process.registers = [0; 31];
            Ok(())
        } else {
            Err("Current process not found")
        }
    } else {
        Err("No current process")
    }
}

pub fn sys_exit(_exit_code: i32) -> ! {
    if let Ok(_) = terminate_current_process() {
        schedule();
    }
    
    // If we can't terminate properly, halt
    loop {
        unsafe {
            asm!("wfe");
        }
    }
}
