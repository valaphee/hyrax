MULTIBOOT_MAGIC = 0x1BADB002;
MULTIBOOT_FLAGS = 0x00010007;

KERNEL_LMA = 0x0000000000100000;
KERNEL_VMA = 0xFFFFFFFF80000000;

ENTRY(__entry)

SECTIONS {
    . = KERNEL_LMA;

    .init : {
        __init_start = .;
        // magic (required)
        LONG(MULTIBOOT_MAGIC);
        // flags (required)
        LONG(MULTIBOOT_FLAGS);
        // checksum (required)
        LONG(-(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS));
        // header_addr (present if flags[16] is set)
        LONG(__init_start);
        // load_addr (present if flags[16] is set)
        LONG(__init_start);
        // load_end_addr (present if flags[16] is set)
        LONG(__tdata_end - KERNEL_VMA);
        // bss_end_addr (present if flags[16] is set)
        LONG(__bss_end - KERNEL_VMA);
        // entry_addr (present if flags[16] is set)
        LONG(__entry);
        // mode_type
        LONG(0)
        // width
        LONG(0)
        // height
        LONG(0)
        // depth
        LONG(0)
        *(.init)
        __init_end = .;
    }

    . += KERNEL_VMA;

    .text ALIGN(CONSTANT(MAXPAGESIZE)) : AT(ADDR(.text) - KERNEL_VMA) {
        __text_start = .;
        *(.text .text.*)
        __text_end = .;
    }

    .rodata ALIGN(CONSTANT(MAXPAGESIZE)) : AT(ADDR(.rodata) - KERNEL_VMA) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data ALIGN(CONSTANT(MAXPAGESIZE)) : AT(ADDR(.data) - KERNEL_VMA) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .tdata ALIGN(CONSTANT(MAXPAGESIZE)) : AT(ADDR(.tdata) - KERNEL_VMA) {
        __tdata_start = .;
        *(.tdata .tdata.*)
        __tdata_end = .;
    }

    .tbss (NOLOAD) : AT(ADDR(.tbss) - KERNEL_VMA) {
        __tbss_start = .;
        *(.tbss .tbss.*)
        __tbss_end = .;
    }

    .bss ALIGN(CONSTANT(MAXPAGESIZE)) (NOLOAD) : AT(ADDR(.bss) - KERNEL_VMA) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame .eh_frame*)
    }
}
