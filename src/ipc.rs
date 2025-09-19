use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use lazy_static::lazy_static;

const PIPE_BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Pipe {
    id: u32,
    buffer: VecDeque<u8>,
    read_closed: bool,
    write_closed: bool,
    readers: u32,
    writers: u32,
}

impl Pipe {
    pub fn new(id: u32) -> Self {
        Pipe {
            id,
            buffer: VecDeque::new(),
            read_closed: false,
            write_closed: false,
            readers: 0,
            writers: 0,
        }
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.read_closed {
            return Err("Pipe read end closed");
        }
        
        let bytes_to_read = core::cmp::min(buf.len(), self.buffer.len());
        
        if bytes_to_read == 0 {
            if self.writers == 0 {
                return Ok(0); // EOF - no writers left
            } else {
                return Err("Would block"); // No data available but writers exist
            }
        }
        
        for i in 0..bytes_to_read {
            buf[i] = self.buffer.pop_front().unwrap();
        }
        
        Ok(bytes_to_read)
    }
    
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, &'static str> {
        if self.write_closed {
            return Err("Pipe write end closed");
        }
        
        if self.readers == 0 {
            return Err("Broken pipe"); // SIGPIPE in real systems
        }
        
        let available_space = PIPE_BUFFER_SIZE - self.buffer.len();
        if available_space == 0 {
            return Err("Would block"); // Pipe buffer full
        }
        
        let bytes_to_write = core::cmp::min(buf.len(), available_space);
        
        for i in 0..bytes_to_write {
            self.buffer.push_back(buf[i]);
        }
        
        Ok(bytes_to_write)
    }
    
    pub fn close_read(&mut self) {
        self.read_closed = true;
        if self.readers > 0 {
            self.readers -= 1;
        }
    }
    
    pub fn close_write(&mut self) {
        self.write_closed = true;
        if self.writers > 0 {
            self.writers -= 1;
        }
    }
    
    pub fn add_reader(&mut self) {
        self.readers += 1;
    }
    
    pub fn add_writer(&mut self) {
        self.writers += 1;
    }
}

pub struct IPCManager {
    pipes: BTreeMap<u32, Pipe>,
    next_pipe_id: u32,
}

impl IPCManager {
    pub fn new() -> Self {
        IPCManager {
            pipes: BTreeMap::new(),
            next_pipe_id: 1,
        }
    }
    
    pub fn create_pipe(&mut self) -> Result<(i32, i32), &'static str> {
        let pipe_id = self.next_pipe_id;
        self.next_pipe_id += 1;
        
        let mut pipe = Pipe::new(pipe_id);
        pipe.add_reader();
        pipe.add_writer();
        
        self.pipes.insert(pipe_id, pipe);
        
        // Create file descriptors for the pipe
        crate::fs::create_pipe_fds(pipe_id)
    }
    
    pub fn read_pipe(&mut self, pipe_id: u32, buf: &mut [u8]) -> Result<usize, &'static str> {
        let pipe = self.pipes.get_mut(&pipe_id).ok_or("Invalid pipe")?;
        pipe.read(buf)
    }
    
    pub fn write_pipe(&mut self, pipe_id: u32, buf: &[u8]) -> Result<usize, &'static str> {
        let pipe = self.pipes.get_mut(&pipe_id).ok_or("Invalid pipe")?;
        pipe.write(buf)
    }
    
    pub fn close_pipe_read(&mut self, pipe_id: u32) -> Result<(), &'static str> {
        let pipe = self.pipes.get_mut(&pipe_id).ok_or("Invalid pipe")?;
        pipe.close_read();
        
        // Remove pipe if both ends are closed
        if pipe.read_closed && pipe.write_closed {
            self.pipes.remove(&pipe_id);
        }
        
        Ok(())
    }
    
    pub fn close_pipe_write(&mut self, pipe_id: u32) -> Result<(), &'static str> {
        let pipe = self.pipes.get_mut(&pipe_id).ok_or("Invalid pipe")?;
        pipe.close_write();
        
        // Remove pipe if both ends are closed
        if pipe.read_closed && pipe.write_closed {
            self.pipes.remove(&pipe_id);
        }
        
        Ok(())
    }
}

// Shared memory implementation
#[derive(Debug)]
pub struct SharedMemorySegment {
    id: u32,
    size: usize,
    data: Vec<u8>,
    permissions: SharedMemoryPermissions,
    attached_processes: Vec<u32>,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct SharedMemoryPermissions: u8 {
        const READ = 1;
        const WRITE = 2;
        const EXECUTE = 4;
    }
}

impl SharedMemorySegment {
    pub fn new(id: u32, size: usize, permissions: SharedMemoryPermissions) -> Self {
        SharedMemorySegment {
            id,
            size,
            data: vec![0; size],
            permissions,
            attached_processes: Vec::new(),
        }
    }
}

pub struct SharedMemoryManager {
    segments: BTreeMap<u32, SharedMemorySegment>,
    next_segment_id: u32,
}

impl SharedMemoryManager {
    pub fn new() -> Self {
        SharedMemoryManager {
            segments: BTreeMap::new(),
            next_segment_id: 1,
        }
    }
    
    pub fn create_segment(&mut self, size: usize, permissions: SharedMemoryPermissions) -> u32 {
        let id = self.next_segment_id;
        self.next_segment_id += 1;
        
        let segment = SharedMemorySegment::new(id, size, permissions);
        self.segments.insert(id, segment);
        
        id
    }
    
    pub fn attach_segment(&mut self, segment_id: u32, process_id: u32) -> Result<*mut u8, &'static str> {
        let segment = self.segments.get_mut(&segment_id).ok_or("Invalid segment")?;
        
        if !segment.attached_processes.contains(&process_id) {
            segment.attached_processes.push(process_id);
        }
        
        Ok(segment.data.as_mut_ptr())
    }
    
    pub fn detach_segment(&mut self, segment_id: u32, process_id: u32) -> Result<(), &'static str> {
        let segment = self.segments.get_mut(&segment_id).ok_or("Invalid segment")?;
        
        segment.attached_processes.retain(|&pid| pid != process_id);
        
        // If no processes are attached, we could optionally remove the segment
        // For now, we'll keep it until explicitly deleted
        
        Ok(())
    }
    
    pub fn delete_segment(&mut self, segment_id: u32) -> Result<(), &'static str> {
        self.segments.remove(&segment_id).ok_or("Invalid segment")?;
        Ok(())
    }
}

lazy_static! {
    static ref IPC_MANAGER: Mutex<IPCManager> = Mutex::new(IPCManager::new());
    static ref SHMEM_MANAGER: Mutex<SharedMemoryManager> = Mutex::new(SharedMemoryManager::new());
}

pub fn init() {
    // IPC managers are initialized statically
}

pub fn create_pipe() -> Result<(i32, i32), &'static str> {
    IPC_MANAGER.lock().create_pipe()
}

pub fn read_pipe(pipe_id: u32, buf: &mut [u8]) -> Result<usize, &'static str> {
    IPC_MANAGER.lock().read_pipe(pipe_id, buf)
}

pub fn write_pipe(pipe_id: u32, buf: &[u8]) -> Result<usize, &'static str> {
    IPC_MANAGER.lock().write_pipe(pipe_id, buf)
}

pub fn close_pipe_read(pipe_id: u32) -> Result<(), &'static str> {
    IPC_MANAGER.lock().close_pipe_read(pipe_id)
}

pub fn close_pipe_write(pipe_id: u32) -> Result<(), &'static str> {
    IPC_MANAGER.lock().close_pipe_write(pipe_id)
}

// Shared memory system calls
pub fn sys_shmget(size: usize, flags: i32) -> u32 {
    let permissions = SharedMemoryPermissions::READ | SharedMemoryPermissions::WRITE;
    SHMEM_MANAGER.lock().create_segment(size, permissions)
}

pub fn sys_shmat(segment_id: u32, process_id: u32) -> Result<*mut u8, &'static str> {
    SHMEM_MANAGER.lock().attach_segment(segment_id, process_id)
}

pub fn sys_shmdt(segment_id: u32, process_id: u32) -> Result<(), &'static str> {
    SHMEM_MANAGER.lock().detach_segment(segment_id, process_id)
}

pub fn sys_shmctl_delete(segment_id: u32) -> Result<(), &'static str> {
    SHMEM_MANAGER.lock().delete_segment(segment_id)
}