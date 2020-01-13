ENTRY(__start)

MEMORY {
    /* reserve the bottom 0x01000000 of RAM for RedBoot */
    ram : ORIGIN = 0x01000000, LENGTH = (32M - 0x01000000)
}

SECTIONS {
    . = 0x01000000;

    .bss : ALIGN(4)
    {
        __BSS_START__ = .;
        *(.bss.*);
        __BSS_END__ = ALIGN(4);
    } > ram

    .data : ALIGN(4)
    {
        __DATA_START__ = .;
        *(.data.*);
        __DATA_END__ = ALIGN(4);
    } > ram

    .text : ALIGN(4)
    {
        __TEXT_START__ = .;
        *(.text)
        *(.text.*)
        *(.rodata .rodata.*)
        *(.got)
        *(.got.plt)
        __TEXT_END__ = ALIGN(4);
    } > ram

    /DISCARD/ :
    {
        *(.ARM.exidx*) /* index entries for section unwinding */
        *(.ARM.extab*) /* exception unwinding information */
    }
}
