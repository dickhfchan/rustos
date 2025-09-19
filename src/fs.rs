use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub fd: i32,
    pub file_type: FileType,
    pub offset: usize,
    pub flags: OpenFlags,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Regular(String),    // Regular file with path
    Pipe(PipeEnd),      // Pipe file descriptor
    Device(DeviceType), // Device file
}

#[derive(Debug, Clone)]
pub enum PipeEnd {
    Read(u32),  // Pipe ID for reading
    Write(u32), // Pipe ID for writing
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Stdin,
    Stdout,
    Stderr,
    Null,
}

bitflags::bitflags! {
    #[derive(Debug, Clone)]
    pub struct OpenFlags: i32 {
        const O_RDONLY = 0;
        const O_WRONLY = 1;
        const O_RDWR = 2;
        const O_CREAT = 64;
        const O_EXCL = 128;
        const O_TRUNC = 512;
        const O_APPEND = 1024;
        const O_NONBLOCK = 2048;
    }
}

pub struct FileSystem {
    open_files: BTreeMap<i32, FileDescriptor>,
    next_fd: i32,
    files: BTreeMap<String, Vec<u8>>, // Simple in-memory file system
}

impl FileSystem {
    pub fn new() -> Self {
        let mut fs = FileSystem {
            open_files: BTreeMap::new(),
            next_fd: 3, // Start after stdin, stdout, stderr
            files: BTreeMap::new(),
        };
        
        // Set up standard file descriptors
        fs.open_files.insert(0, FileDescriptor {
            fd: 0,
            file_type: FileType::Device(DeviceType::Stdin),
            offset: 0,
            flags: OpenFlags::O_RDONLY,
        });
        
        fs.open_files.insert(1, FileDescriptor {
            fd: 1,
            file_type: FileType::Device(DeviceType::Stdout),
            offset: 0,
            flags: OpenFlags::O_WRONLY,
        });
        
        fs.open_files.insert(2, FileDescriptor {
            fd: 2,
            file_type: FileType::Device(DeviceType::Stderr),
            offset: 0,
            flags: OpenFlags::O_WRONLY,
        });
        
        fs
    }
    
    pub fn open(&mut self, path: &str, flags: i32, _mode: u32) -> Result<i32, &'static str> {
        let open_flags = OpenFlags::from_bits(flags).ok_or("Invalid flags")?;
        let fd = self.next_fd;
        self.next_fd += 1;
        
        // Handle special device files
        let file_type = match path {
            "/dev/null" => FileType::Device(DeviceType::Null),
            _ => {
                // Regular file
                if open_flags.contains(OpenFlags::O_CREAT) && !self.files.contains_key(path) {
                    self.files.insert(path.into(), Vec::new());
                }
                
                if !self.files.contains_key(path) && !open_flags.contains(OpenFlags::O_CREAT) {
                    return Err("File not found");
                }
                
                FileType::Regular(path.into())
            }
        };
        
        let descriptor = FileDescriptor {
            fd,
            file_type,
            offset: 0,
            flags: open_flags,
        };
        
        self.open_files.insert(fd, descriptor);
        Ok(fd)
    }
    
    pub fn close(&mut self, fd: i32) -> Result<(), &'static str> {
        self.open_files.remove(&fd).ok_or("Invalid file descriptor")?;
        Ok(())
    }
    
    pub fn read(&mut self, fd: i32, buf: &mut [u8]) -> Result<usize, &'static str> {
        let descriptor = self.open_files.get_mut(&fd).ok_or("Invalid file descriptor")?;
        
        match &descriptor.file_type {
            FileType::Regular(path) => {
                let file_data = self.files.get(path).ok_or("File not found")?;
                let bytes_to_read = core::cmp::min(buf.len(), file_data.len() - descriptor.offset);
                
                if bytes_to_read == 0 {
                    return Ok(0); // EOF
                }
                
                buf[..bytes_to_read].copy_from_slice(
                    &file_data[descriptor.offset..descriptor.offset + bytes_to_read]
                );
                descriptor.offset += bytes_to_read;
                Ok(bytes_to_read)
            }
            FileType::Device(DeviceType::Stdin) => {
                // For now, return empty read for stdin
                Ok(0)
            }
            FileType::Device(DeviceType::Null) => {
                Ok(0) // /dev/null always returns EOF on read
            }
            FileType::Pipe(PipeEnd::Read(pipe_id)) => {
                crate::ipc::read_pipe(*pipe_id, buf)
            }
            _ => Err("Cannot read from this file descriptor"),
        }
    }
    
    pub fn write(&mut self, fd: i32, buf: &[u8]) -> Result<usize, &'static str> {
        let descriptor = self.open_files.get_mut(&fd).ok_or("Invalid file descriptor")?;
        
        match &descriptor.file_type {
            FileType::Regular(path) => {
                let path_clone = path.clone();
                let file_data = self.files.get_mut(&path_clone).ok_or("File not found")?;
                
                if descriptor.flags.contains(OpenFlags::O_APPEND) {
                    file_data.extend_from_slice(buf);
                } else {
                    // Ensure file is large enough
                    if descriptor.offset + buf.len() > file_data.len() {
                        file_data.resize(descriptor.offset + buf.len(), 0);
                    }
                    
                    file_data[descriptor.offset..descriptor.offset + buf.len()].copy_from_slice(buf);
                    descriptor.offset += buf.len();
                }
                
                Ok(buf.len())
            }
            FileType::Device(DeviceType::Stdout) | FileType::Device(DeviceType::Stderr) => {
                // Write to UART
                for &byte in buf {
                    crate::uart::_print(format_args!("{}", byte as char));
                }
                Ok(buf.len())
            }
            FileType::Device(DeviceType::Null) => {
                Ok(buf.len()) // /dev/null accepts all writes
            }
            FileType::Pipe(PipeEnd::Write(pipe_id)) => {
                crate::ipc::write_pipe(*pipe_id, buf)
            }
            _ => Err("Cannot write to this file descriptor"),
        }
    }
    
    pub fn duplicate_fd(&mut self, fd: i32) -> Result<i32, &'static str> {
        let descriptor = self.open_files.get(&fd).ok_or("Invalid file descriptor")?.clone();
        let new_fd = self.next_fd;
        self.next_fd += 1;
        
        let mut new_descriptor = descriptor;
        new_descriptor.fd = new_fd;
        
        self.open_files.insert(new_fd, new_descriptor);
        Ok(new_fd)
    }
    
    pub fn duplicate_fd_to(&mut self, oldfd: i32, newfd: i32) -> Result<i32, &'static str> {
        let descriptor = self.open_files.get(&oldfd).ok_or("Invalid file descriptor")?.clone();
        
        // Close newfd if it's already open
        self.open_files.remove(&newfd);
        
        let mut new_descriptor = descriptor;
        new_descriptor.fd = newfd;
        
        self.open_files.insert(newfd, new_descriptor);
        Ok(newfd)
    }
    
    pub fn create_pipe_fds(&mut self, pipe_id: u32) -> Result<(i32, i32), &'static str> {
        let read_fd = self.next_fd;
        self.next_fd += 1;
        let write_fd = self.next_fd;
        self.next_fd += 1;
        
        let read_descriptor = FileDescriptor {
            fd: read_fd,
            file_type: FileType::Pipe(PipeEnd::Read(pipe_id)),
            offset: 0,
            flags: OpenFlags::O_RDONLY,
        };
        
        let write_descriptor = FileDescriptor {
            fd: write_fd,
            file_type: FileType::Pipe(PipeEnd::Write(pipe_id)),
            offset: 0,
            flags: OpenFlags::O_WRONLY,
        };
        
        self.open_files.insert(read_fd, read_descriptor);
        self.open_files.insert(write_fd, write_descriptor);
        
        Ok((read_fd, write_fd))
    }
}

lazy_static! {
    static ref FILE_SYSTEM: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}

pub fn init() {
    // File system is initialized statically
}

pub fn open(path: &str, flags: i32, mode: u32) -> Result<i32, &'static str> {
    FILE_SYSTEM.lock().open(path, flags, mode)
}

pub fn close(fd: i32) -> Result<(), &'static str> {
    FILE_SYSTEM.lock().close(fd)
}

pub fn read(fd: i32, buf: &mut [u8]) -> Result<usize, &'static str> {
    FILE_SYSTEM.lock().read(fd, buf)
}

pub fn write(fd: i32, buf: &[u8]) -> Result<usize, &'static str> {
    FILE_SYSTEM.lock().write(fd, buf)
}

pub fn duplicate_fd(fd: i32) -> Result<i32, &'static str> {
    FILE_SYSTEM.lock().duplicate_fd(fd)
}

pub fn duplicate_fd_to(oldfd: i32, newfd: i32) -> Result<i32, &'static str> {
    FILE_SYSTEM.lock().duplicate_fd_to(oldfd, newfd)
}

pub fn create_pipe_fds(pipe_id: u32) -> Result<(i32, i32), &'static str> {
    FILE_SYSTEM.lock().create_pipe_fds(pipe_id)
}

// Additional functions for coreutils support

pub fn read_file(path: &str) -> Result<String, &'static str> {
    let mut fs = FILE_SYSTEM.lock();
    
    // Check if file exists
    if !fs.files.contains_key(path) {
        return Err("File not found");
    }
    
    // Read file content as string
    let file_data = fs.files.get(path).ok_or("File not found")?;
    let content = String::from_utf8_lossy(file_data).to_string();
    Ok(content)
}

pub fn list_directory(path: &str) -> Result<Vec<String>, &'static str> {
    let fs = FILE_SYSTEM.lock();
    let mut entries = Vec::new();
    
    // Add standard directory entries
    entries.push(".".to_string());
    entries.push("..".to_string());
    
    // Add simulated files based on path
    match path {
        "/" => {
            entries.push("bin".to_string());
            entries.push("etc".to_string());
            entries.push("home".to_string());
            entries.push("tmp".to_string());
            entries.push("usr".to_string());
            entries.push("var".to_string());
        }
        "/home" => {
            entries.push("user".to_string());
            entries.push("guest".to_string());
        }
        _ => {
            // List files that actually exist in the filesystem
            for (file_path, _) in fs.files.iter() {
                if file_path.starts_with(path) && file_path != path {
                    let relative_path = file_path.strip_prefix(path).unwrap_or(file_path);
                    if !relative_path.is_empty() && !relative_path.starts_with("/") {
                        entries.push(relative_path.to_string());
                    } else if relative_path.starts_with("/") && relative_path.len() > 1 {
                        entries.push(relative_path[1..].to_string());
                    }
                }
            }
            
            if entries.len() == 2 { // Only . and ..
                entries.push("example.txt".to_string());
                entries.push("readme.md".to_string());
            }
        }
    }
    
    Ok(entries)
}

pub fn get_current_directory() -> Result<String, &'static str> {
    Ok("/".to_string())
}

pub fn create_directory(path: &str) -> Result<(), &'static str> {
    // For now, just simulate directory creation
    // In a real filesystem, this would create directory metadata
    Ok(())
}

pub fn create_file(path: &str) -> Result<(), &'static str> {
    let mut fs = FILE_SYSTEM.lock();
    fs.files.insert(path.to_string(), Vec::new());
    Ok(())
}

pub fn remove_file(path: &str) -> Result<(), &'static str> {
    let mut fs = FILE_SYSTEM.lock();
    if fs.files.remove(path).is_some() {
        Ok(())
    } else {
        Err("File not found")
    }
}

pub fn copy_file(source: &str, dest: &str) -> Result<(), &'static str> {
    let mut fs = FILE_SYSTEM.lock();
    let source_data = fs.files.get(source).ok_or("Source file not found")?.clone();
    fs.files.insert(dest.to_string(), source_data);
    Ok(())
}

pub fn move_file(source: &str, dest: &str) -> Result<(), &'static str> {
    let mut fs = FILE_SYSTEM.lock();
    let source_data = fs.files.remove(source).ok_or("Source file not found")?;
    fs.files.insert(dest.to_string(), source_data);
    Ok(())
}