#![allow(dead_code)]

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

extern "C" {
    static __heap_start: u8;
    static __heap_end: u8;
}

pub fn init() {
    let heap_start = unsafe { &__heap_start as *const u8 as usize };
    let heap_end = unsafe { &__heap_end as *const u8 as usize };
    let heap_size = heap_end - heap_start;
    
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static [MemoryRegion]) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
    
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.start..r.start + r.size);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub fn new(addr: u64) -> PhysAddr {
        PhysAddr(addr)
    }
    
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub fn new(addr: u64) -> VirtAddr {
        VirtAddr(addr)
    }
    
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame {
    start_address: PhysAddr,
}

impl PhysFrame {
    pub fn containing_address(address: PhysAddr) -> PhysFrame {
        PhysFrame {
            start_address: PhysAddr::new(address.as_u64() & !0xfff),
        }
    }
    
    pub fn start_address(self) -> PhysAddr {
        self.start_address
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    start_address: VirtAddr,
}

impl Page {
    pub fn containing_address(address: VirtAddr) -> Page {
        Page {
            start_address: VirtAddr::new(address.as_u64() & !0xfff),
        }
    }
    
    pub fn start_address(self) -> VirtAddr {
        self.start_address
    }
}

// Simple page table management for ARM64
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

#[derive(Clone, Copy)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    pub fn new() -> Self {
        PageTableEntry { entry: 0 }
    }
    
    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }
    
    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.entry = frame.start_address().as_u64() | flags.bits();
    }
}

bitflags::bitflags! {
    pub struct PageTableFlags: u64 {
        const PRESENT = 1;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

// Additional memory management functions for system calls
pub fn allocate_pages(size: usize) -> Result<u64, &'static str> {
    // Simple page allocation - align to page boundary
    let pages = (size + 4095) / 4096;
    static mut NEXT_ADDR: u64 = 0x60000000;
    unsafe {
        let addr = NEXT_ADDR;
        NEXT_ADDR += pages as u64 * 4096;
        Ok(addr)
    }
}

pub fn deallocate_pages(_addr: u64, _size: usize) -> Result<(), &'static str> {
    // Simple deallocation - in a real kernel this would free the pages
    Ok(())
}
