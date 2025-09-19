use core::fmt;
use spin::Mutex;

// UART base address for ARM64 virt machine
const UART_BASE: usize = 0x9000000;

pub struct Uart {
    base_address: usize,
}

impl Uart {
    const fn new(base_address: usize) -> Self {
        Uart { base_address }
    }
    
    fn write_byte(&self, byte: u8) {
        unsafe {
            let ptr = self.base_address as *mut u8;
            ptr.write_volatile(byte);
        }
    }
    
    #[allow(dead_code)]
    fn read_byte(&self) -> Option<u8> {
        unsafe {
            let status_ptr = (self.base_address + 0x18) as *mut u32;
            let data_ptr = self.base_address as *mut u8;
            
            if status_ptr.read_volatile() & (1 << 4) == 0 {
                Some(data_ptr.read_volatile())
            } else {
                None
            }
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

static UART: Mutex<Uart> = Mutex::new(Uart::new(UART_BASE));

pub fn init() {
    // UART initialization is minimal for ARM64 virt machine
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    UART.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
