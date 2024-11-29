MULTIBOOT_MAGIC = 0x1BADB002;
MULTIBOOT_FLAGS = 0x00010007;

KERNEL_LMA = 0x0000000000100000;
KERNEL_VMA = 0xFFFFFFFF80000000;

ENTRY(__start)

SECTIONS {
    . = KERNEL_LMA;

    .multiboot : {
        __multiboot_start = .;
        // magic (required)
        LONG(MULTIBOOT_MAGIC);
        // flags (required)
        LONG(MULTIBOOT_FLAGS);
        // checksum (required)
        LONG(-(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS));
        // header_addr (present if flags[16] is set)
        LONG(__multiboot_start);
        // load_addr (present if flags[16] is set)
        LONG(__multiboot_start);
        // load_end_addr (present if flags[16] is set)
        LONG(__data_end - KERNEL_VMA);
        // bss_end_addr (present if flags[16] is set)
        LONG(__bss_end - KERNEL_VMA);
        // entry_addr (present if flags[16] is set)
        LONG(__start);
        // mode_type
        LONG(0)
        // width
        LONG(0)
        // height
        LONG(0)
        // depth
        LONG(0)
        *(.multiboot.init)
        __multiboot_end = .;
    }

    . += KERNEL_VMA;

    .text : AT(ADDR(.text) - KERNEL_VMA) ALIGN(CONSTANT(MAXPAGESIZE)) {
        __text_start = .;
        *(.text .text.*)
        . = ALIGN(CONSTANT(MAXPAGESIZE));
        __text_end = .;
    }

    .rodata : AT(ADDR(.rodata) - KERNEL_VMA) ALIGN(CONSTANT(MAXPAGESIZE)) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        . = ALIGN(CONSTANT(MAXPAGESIZE));
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - KERNEL_VMA) ALIGN(CONSTANT(MAXPAGESIZE)) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss (NOLOAD) : AT(ADDR(.bss) - KERNEL_VMA) ALIGN(CONSTANT(MAXPAGESIZE)) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame .eh_frame*)
    }
}
