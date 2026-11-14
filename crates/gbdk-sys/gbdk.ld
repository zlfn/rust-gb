/* gbdk.ld: GBDK fixed-address sections, layered over gb.ld */

/* `_cpu` is the raw boot-A value rrt0 captures. So alias it to rrt0's `__boot_a`. */
__cpu = __boot_a;

/* GBDK's interrupt dispatchers */
PROVIDE(_on_vblank   = DEFINED(_gbdk_on_vblank)   ? _gbdk_on_vblank   : _isr_noop);
PROVIDE(_on_lcd_stat = DEFINED(_gbdk_on_lcd_stat) ? _gbdk_on_lcd_stat : _isr_noop);
PROVIDE(_on_timer    = DEFINED(_gbdk_on_timer)    ? _gbdk_on_timer    : _isr_noop);
PROVIDE(_on_serial   = DEFINED(_gbdk_on_serial)   ? _gbdk_on_serial   : _isr_noop);
PROVIDE(_on_joypad   = DEFINED(_gbdk_on_joypad)   ? _gbdk_on_joypad   : _isr_noop);

.STACK = 0xE000;

SECTIONS {
    _DRAW_HEADER0 (0x0070) : { KEEP(*(_DRAW_HEADER0)) } > ROM
    _DRAW_HEADER1 (0x00F4) : { KEEP(*(_DRAW_HEADER1)) } > ROM
    _FONT_HEADER0 (0x00F8) : { KEEP(*(_FONT_HEADER0)) } > ROM
}

/* The shadow OAM page that GBDK's OAM DMA copies from. */
SECTIONS {
    .shadow_oam 0xC000 (NOLOAD) : {
        _shadow_OAM = .;
        . += 0xA0;
    } > WRAM
}
INSERT BEFORE .data;
