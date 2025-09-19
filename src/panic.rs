use core::mem::transmute;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};

static PANIC_HANDLER: AtomicUsize = AtomicUsize::new(0);

pub fn set_handler(handler: fn(&PanicInfo) -> !) {
    PANIC_HANDLER.store(handler as usize, Ordering::SeqCst);
}

pub fn clear_handler() {
    PANIC_HANDLER.store(0, Ordering::SeqCst);
}

pub fn handle(info: &PanicInfo) -> ! {
    let handler = PANIC_HANDLER.load(Ordering::SeqCst);
    if handler != 0 {
        let func: fn(&PanicInfo) -> ! = unsafe { transmute(handler) };
        func(info)
    } else {
        loop {}
    }
}
