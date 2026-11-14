/* Game Boy (SM83) linker script for ld.lld + rrt0 */

ENTRY(_reset)

MEMORY {
    ROM  (rx) : ORIGIN = 0x0000, LENGTH = 0x8000
    WRAM (rw) : ORIGIN = 0xC000, LENGTH = 0x2000
    HRAM (rw) : ORIGIN = 0xFF80, LENGTH = 0x007F
}

SECTIONS {
    _RST00      (0x0000) : { KEEP(*(_RST00)) } > ROM
    _RST08      (0x0008) : { KEEP(*(_RST08)) } > ROM
    _RST10      (0x0010) : { KEEP(*(_RST10)) } > ROM
    _RST18      (0x0018) : { KEEP(*(_RST18)) } > ROM
    _RST20      (0x0020) : { KEEP(*(_RST20)) } > ROM
    _RST28      (0x0028) : { KEEP(*(_RST28)) } > ROM
    _RST30      (0x0030) : { KEEP(*(_RST30)) } > ROM
    _RST38      (0x0038) : { KEEP(*(_RST38)) } > ROM

    _INT_VBL    (0x0040) : { KEEP(*(_INT_VBL)) } > ROM
    _INT_STAT   (0x0048) : { KEEP(*(_INT_STAT)) } > ROM
    _INT_TIMER  (0x0050) : { KEEP(*(_INT_TIMER)) } > ROM
    _INT_SERIAL (0x0058) : { KEEP(*(_INT_SERIAL)) } > ROM
    _INT_JOYPAD (0x0060) : { KEEP(*(_INT_JOYPAD)) } > ROM

    _ENTRY      (0x0100) : { KEEP(*(_ENTRY)) } > ROM
    _LOGO       (0x0104) : { KEEP(*(_LOGO)) } > ROM
    _TITLE      (0x0134) : { KEEP(*(_TITLE)) } > ROM
    _HEADER     (0x0143) : { KEEP(*(_HEADER)) } > ROM

    .text.rrt0 : { KEEP(*(.text.rrt0)) } > ROM

    .text : {
        *(.text .text.*)
        *(.rodata .rodata.*)
    } > ROM

    /* The initializers follow .text in bank 0, where rrt0 can reach them. */
    .data 0xC0A0 : {
        *(.data .data.*)
        *(.data.initialized)
    } > WRAM AT> ROM

    /* Named by rrt0, which copies the initializers at reset. */
    s__INITIALIZER = LOADADDR(.data);
    l__INITIALIZER = SIZEOF(.data);
    s__INITIALIZED = ADDR(.data);

    .bss (NOLOAD) : {
        *(.bss .bss.*)
        *(COMMON)
    } > WRAM

    /* The build tool generates gb_banked.ld with the .bankN sections. */
    INCLUDE "gb_banked.ld"

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame*)
        *(.debug*)
    }
}
