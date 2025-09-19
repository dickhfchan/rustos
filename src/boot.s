.section .text.boot
.global _start

_start:
    // Disable interrupts
    msr daifset, #0xf
    
    // Set up stack pointer
    ldr x1, =__stack_end
    mov sp, x1
    
    // Clear BSS section
    ldr x1, =__bss_start
    ldr x2, =__bss_end
clear_bss:
    cmp x1, x2
    b.eq clear_bss_done
    str xzr, [x1], #8
    b clear_bss
clear_bss_done:

    // Jump to Rust code
    bl kernel_main

    // Halt if kernel_main returns
halt:
    wfe
    b halt