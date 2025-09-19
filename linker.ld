ENTRY(_start)

SECTIONS
{
    . = 0x40080000;

    .text.boot : {
        *(.text.boot)
    }

    .text : {
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(4096);
    __heap_start = .;
    . += 0x100000; /* 1MB heap */
    __heap_end = .;

    . = ALIGN(4096);
    __stack_start = .;
    . += 0x10000; /* 64KB stack */
    __stack_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.gnu*)
        *(.note*)
        *(.eh_frame*)
    }
}